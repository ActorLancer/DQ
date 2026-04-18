#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tokio_postgres::{Client, NoTls};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedOrder {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        cancelable_order_id: String,
        locked_order_id: String,
        non_cancelable_order_id: String,
    }

    #[tokio::test]
    async fn trade005_order_cancel_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
            .await
            .expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix epoch")
                .as_millis()
        );
        let seed = seed_order_graph(&client, &suffix)
            .await
            .expect("seed order graph");

        let app = router();
        let lock_request_id = format!("req-trade005-lock-{suffix}");
        let lock_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/cancel", seed.locked_order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &lock_request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(lock_resp.status(), StatusCode::OK);
        let lock_body = to_bytes(lock_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let lock_json: Value = serde_json::from_slice(&lock_body).expect("json");
        assert_eq!(
            lock_json["data"]["data"]["current_state"].as_str(),
            Some("closed")
        );
        assert_eq!(
            lock_json["data"]["data"]["payment_status"].as_str(),
            Some("refund_pending")
        );
        assert_eq!(
            lock_json["data"]["data"]["refund_required"].as_bool(),
            Some(true)
        );

        let row = client
            .query_one(
                "SELECT status, payment_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.locked_order_id],
            )
            .await
            .expect("query closed order");
        assert_eq!(row.get::<_, String>(0), "closed");
        assert_eq!(row.get::<_, String>(1), "refund_pending");
        assert_eq!(
            row.get::<_, Option<String>>(2).as_deref(),
            Some("order_cancel_refund_required_after_lock")
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'trade.order.cancel'",
                &[&lock_request_id],
            )
            .await
            .expect("query audit")
            .get(0);
        assert!(audit_count >= 1);

        let non_cancel_request_id = format!("req-trade005-delivered-{suffix}");
        let non_cancel_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/cancel",
                        seed.non_cancelable_order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &non_cancel_request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(non_cancel_resp.status(), StatusCode::CONFLICT);
        let non_cancel_body = to_bytes(non_cancel_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let non_cancel_json: Value = serde_json::from_slice(&non_cancel_body).expect("json");
        let msg = non_cancel_json["message"]
            .as_str()
            .or_else(|| non_cancel_json["error"]["message"].as_str())
            .unwrap_or_default()
            .to_string();
        assert!(msg.contains("ORDER_CANCEL_FORBIDDEN"));

        let cancelable_request_id = format!("req-trade005-created-{suffix}");
        let created_cancel_resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/cancel",
                        seed.cancelable_order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &cancelable_request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(created_cancel_resp.status(), StatusCode::OK);
        let created_cancel_body = to_bytes(created_cancel_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let created_cancel_json: Value =
            serde_json::from_slice(&created_cancel_body).expect("json");
        assert_eq!(
            created_cancel_json["data"]["data"]["refund_required"].as_bool(),
            Some(false)
        );
        assert_eq!(
            created_cancel_json["data"]["data"]["refund_branch"].as_str(),
            Some("no_refund")
        );

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_order_graph(
        client: &Client,
        suffix: &str,
    ) -> Result<SeedOrder, tokio_postgres::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade005-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade005-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade005-asset-{suffix}"),
                    &format!("trade005 asset {suffix}"),
                ],
            )
            .await?
            .get(0);

        let asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text",
                &[&asset_id],
            )
            .await?
            .get(0);

        let product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 18.80, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade005-product-{suffix}"),
                    &format!("trade005 product {suffix}"),
                    &format!("trade005 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE005-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let cancelable_order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'created', 'unpaid', 'online', 18.80, 'CNY'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                ],
            )
            .await?
            .get(0);

        let locked_order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'online', 18.80, 'CNY'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                ],
            )
            .await?
            .get(0);

        let non_cancelable_order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'delivered', 'paid', 'online', 18.80, 'CNY'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                ],
            )
            .await?
            .get(0);

        Ok(SeedOrder {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            cancelable_order_id,
            locked_order_id,
            non_cancelable_order_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedOrder) {
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id IN (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3::text::uuid
                 )",
                &[
                    &seed.cancelable_order_id,
                    &seed.locked_order_id,
                    &seed.non_cancelable_order_id,
                ],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&seed.asset_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seed.buyer_org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seed.seller_org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&seed.sku_id],
            )
            .await;
    }
}
