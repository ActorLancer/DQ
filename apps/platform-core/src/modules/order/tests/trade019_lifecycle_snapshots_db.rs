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
        order_id: String,
        authorization_id: String,
        delivery_id: String,
    }

    #[tokio::test]
    async fn trade019_order_lifecycle_snapshots_db_smoke() {
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
        let request_id = format!("req-trade019-{suffix}");

        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/orders/{}/lifecycle-snapshots",
                        seed.order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");

        assert_eq!(
            json["data"]["data"]["order"]["order_id"].as_str(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["order"]["current_state"].as_str(),
            Some("shared_active")
        );
        assert_eq!(
            json["data"]["data"]["order"]["payment"]["current_status"].as_str(),
            Some("paid")
        );
        assert_eq!(
            json["data"]["data"]["order"]["settlement"]["current_status"].as_str(),
            Some("pending_settlement")
        );
        assert_eq!(
            json["data"]["data"]["order"]["dispute"]["current_status"].as_str(),
            Some("none")
        );
        assert_eq!(
            json["data"]["data"]["contract"]["contract_status"].as_str(),
            Some("signed")
        );
        assert_eq!(
            json["data"]["data"]["authorization"]["authorization_id"].as_str(),
            Some(seed.authorization_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["authorization"]["current_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            json["data"]["data"]["authorization"]["authorization_model"]["scope"]["order_id"]
                .as_str(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["authorization"]["authorization_model"]["resource"]["sku_id"]
                .as_str(),
            Some(seed.sku_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["delivery"]["delivery_id"].as_str(),
            Some(seed.delivery_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["delivery"]["current_status"].as_str(),
            Some("delivered")
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'trade.order.lifecycle_snapshots.read'",
                &[&request_id],
            )
            .await
            .expect("query audit")
            .get(0);
        assert!(audit_count >= 1);

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
                &[&format!("trade019-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade019-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'retail', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade019-asset-{suffix}"),
                    &format!("trade019 asset {suffix}"),
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'retail', 'data_product',
                   $5, 'listed', 'one_time', 66.60, 'CNY', 'share_grant',
                   ARRAY['internal_use']::text[], $6,
                   '{"tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade019-product-{suffix}"),
                    &format!("trade019 product {suffix}"),
                    &format!("trade019 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE019-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, last_reason_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'shared_active', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 66.60, 'CNY', 'trade019_seed', '{}'::jsonb
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

        let contract_id: String = client
            .query_one(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb
                 )
                 RETURNING contract_id::text",
                &[&order_id, &format!("sha256:trade019:{suffix}")],
            )
            .await?
            .get(0);
        client
            .execute(
                "UPDATE trade.order_main
                 SET contract_id = $2::text::uuid,
                     updated_at = now()
                 WHERE order_id = $1::text::uuid",
                &[&order_id, &contract_id],
            )
            .await?;

        let authorization_id: String = client
            .query_one(
                "INSERT INTO trade.authorization_grant (
                   order_id, grant_type, granted_to_type, granted_to_id, policy_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 'share_grant', 'org', $2::text::uuid,
                   jsonb_build_object('policy_id', gen_random_uuid()::text, 'policy_name', 'trade019-policy'),
                   'active'
                 )
                 RETURNING authorization_grant_id::text",
                &[&order_id, &buyer_org_id],
            )
            .await?
            .get(0);

        let delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, committed_at, receipt_hash
                 ) VALUES (
                   $1::text::uuid, 'share_grant', 'ro_link', 'delivered', now(), $2
                 )
                 RETURNING delivery_id::text",
                &[&order_id, &format!("trade019-receipt-{suffix}")],
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
            order_id,
            authorization_id,
            delivery_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedOrder) {
        let _ = client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE delivery_id = $1::text::uuid",
                &[&seed.delivery_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.authorization_grant WHERE authorization_grant_id = $1::text::uuid",
                &[&seed.authorization_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&seed.sku_id],
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
    }
}
