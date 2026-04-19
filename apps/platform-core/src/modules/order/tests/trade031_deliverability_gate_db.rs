#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tokio_postgres::{Client, NoTls};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn trade031_deliverability_gate_db_smoke() {
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

        let missing_contract = post_enable_share(
            &app,
            &seed.order_id,
            &seed.buyer_org_id,
            &format!("req-trade031-{suffix}-missing-contract"),
        )
        .await;
        assert_eq!(
            missing_contract.status,
            StatusCode::CONFLICT,
            "{}",
            missing_contract.body
        );
        assert!(
            missing_contract
                .body
                .contains("contract is not signed/effective")
        );
        assert_eq!(delivery_count(&client, &seed.order_id).await, 0);

        client
            .execute(
                r#"INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{"term_days":30}'::jsonb
                 )"#,
                &[&seed.order_id, &format!("sha256:trade031:{suffix}")],
            )
            .await
            .expect("insert contract");
        client
            .execute(
                "UPDATE core.organization
                 SET metadata = jsonb_set(metadata, '{risk_status}', '\"blocked\"'::jsonb, true),
                     updated_at = now()
                 WHERE org_id = $1::text::uuid",
                &[&seed.seller_org_id],
            )
            .await
            .expect("set subject blocked");

        let blocked_subject = post_enable_share(
            &app,
            &seed.order_id,
            &seed.buyer_org_id,
            &format!("req-trade031-{suffix}-subject-blocked"),
        )
        .await;
        assert_eq!(
            blocked_subject.status,
            StatusCode::CONFLICT,
            "{}",
            blocked_subject.body
        );
        assert!(
            blocked_subject
                .body
                .contains("buyer/seller organization is blocked by subject risk policy")
        );
        assert_eq!(delivery_count(&client, &seed.order_id).await, 0);

        client
            .execute(
                "UPDATE core.organization
                 SET metadata = jsonb_set(metadata, '{risk_status}', '\"normal\"'::jsonb, true),
                     updated_at = now()
                 WHERE org_id = $1::text::uuid",
                &[&seed.seller_org_id],
            )
            .await
            .expect("reset subject risk");
        client
            .execute(
                "UPDATE catalog.product
                 SET metadata = jsonb_set(metadata, '{review_status}', '\"rejected\"'::jsonb, true),
                     updated_at = now()
                 WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await
            .expect("set rejected review");

        let rejected_product = post_enable_share(
            &app,
            &seed.order_id,
            &seed.buyer_org_id,
            &format!("req-trade031-{suffix}-review-rejected"),
        )
        .await;
        assert_eq!(
            rejected_product.status,
            StatusCode::CONFLICT,
            "{}",
            rejected_product.body
        );
        assert!(
            rejected_product
                .body
                .contains("product review status is not approved")
        );
        assert_eq!(delivery_count(&client, &seed.order_id).await, 0);

        client
            .execute(
                "UPDATE catalog.product
                 SET metadata = jsonb_set(metadata, '{review_status}', '\"approved\"'::jsonb, true),
                     updated_at = now()
                 WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await
            .expect("set approved review");

        let success = post_enable_share(
            &app,
            &seed.order_id,
            &seed.buyer_org_id,
            &format!("req-trade031-{suffix}-success"),
        )
        .await;
        assert_eq!(success.status, StatusCode::OK, "{}", success.body);
        assert_eq!(
            success.json["data"]["data"]["current_state"].as_str(),
            Some("share_enabled")
        );
        assert_eq!(
            success.json["data"]["data"]["delivery_status"].as_str(),
            Some("in_progress")
        );

        let order_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, acceptance_status, settlement_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order");
        assert_eq!(order_row.get::<_, String>(0), "share_enabled");
        assert_eq!(order_row.get::<_, String>(1), "paid");
        assert_eq!(order_row.get::<_, String>(2), "in_progress");
        assert_eq!(order_row.get::<_, String>(3), "not_started");
        assert_eq!(order_row.get::<_, String>(4), "pending_settlement");

        let delivery_row = client
            .query_one(
                "SELECT delivery_type, delivery_route, status
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("query delivery record");
        assert_eq!(delivery_row.get::<_, String>(0), "share_grant");
        assert_eq!(delivery_row.get::<_, String>(1), "share_link");
        assert_eq!(delivery_row.get::<_, String>(2), "prepared");

        assert_audit(
            &client,
            &format!("req-trade031-{suffix}-success"),
            "trade.order.delivery_gate.prepared",
        )
        .await;
        assert_audit(
            &client,
            &format!("req-trade031-{suffix}-success"),
            "trade.order.share_ro.transition",
        )
        .await;

        cleanup_seed_graph(&client, &seed).await;
    }

    struct HttpResult {
        status: StatusCode,
        body: String,
        json: Value,
    }

    async fn post_enable_share(
        app: &axum::Router,
        order_id: &str,
        tenant_id: &str,
        request_id: &str,
    ) -> HttpResult {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/share-ro/transition"))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"enable_share"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let body_text = String::from_utf8(body.to_vec()).expect("utf8 body");
        let json = serde_json::from_str::<Value>(&body_text).unwrap_or_else(|_| Value::Null);
        HttpResult {
            status,
            body: body_text,
            json,
        }
    }

    async fn delivery_count(client: &Client, order_id: &str) -> i64 {
        client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid",
                &[&order_id],
            )
            .await
            .expect("count delivery")
            .get(0)
    }

    async fn assert_audit(client: &Client, request_id: &str, action_name: &str) {
        let count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = $2",
                &[&request_id, &action_name],
            )
            .await
            .expect("query audit")
            .get(0);
        assert_eq!(count, 1, "missing audit `{action_name}` for `{request_id}`");
    }

    async fn seed_order_graph(
        client: &Client,
        suffix: &str,
    ) -> Result<SeedGraph, tokio_postgres::Error> {
        let buyer_org_id: String = client
            .query_one(
                r#"INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES (
                   $1, 'enterprise', 'active',
                   '{"risk_status":"normal","sellable_status":"enabled"}'::jsonb
                 )
                 RETURNING org_id::text"#,
                &[&format!("trade031-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                r#"INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES (
                   $1, 'enterprise', 'active',
                   '{"risk_status":"normal","sellable_status":"enabled"}'::jsonb
                 )
                 RETURNING org_id::text"#,
                &[&format!("trade031-seller-{suffix}")],
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
                    &format!("trade031-asset-{suffix}"),
                    &format!("trade031 asset {suffix}"),
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
                   1024, 'CN', ARRAY['CN']::text[], false,
                   '{\"zone\":\"platform\"}'::jsonb, 'active'
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
                   $5, 'listed', 'subscription', 19.90, 'CNY', 'read_only_share',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade031-product-{suffix}"),
                    &format!("trade031 product {suffix}"),
                    &format!("trade031 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '月', 'subscription', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE031-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, trust_boundary_snapshot, delivery_route_snapshot, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none',
                   'online', 19.90, 'CNY', '{"zone":"platform"}'::jsonb, 'share_link', '{}'::jsonb
                 )
                 RETURNING order_id::text"#,
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
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
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await;
    }
}
