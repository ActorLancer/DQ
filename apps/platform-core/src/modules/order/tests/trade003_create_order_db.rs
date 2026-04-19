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
    }

    #[tokio::test]
    async fn trade003_create_order_db_smoke() {
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
        let seed = seed_graph(&client, &suffix).await.expect("seed graph");
        let request_id = format!("req-trade003-{suffix}");
        let idempotency_key = format!("idem-trade003-{suffix}");

        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &request_id)
                    .header("x-idempotency-key", &idempotency_key)
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                          "buyer_org_id":"{}",
                          "product_id":"{}",
                          "sku_id":"{}"
                        }}"#,
                        seed.buyer_org_id, seed.product_id, seed.sku_id
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        let order_id = json["data"]["data"]["order_id"]
            .as_str()
            .expect("order id")
            .to_string();
        assert_eq!(
            json["data"]["data"]["price_snapshot"]["product_id"].as_str(),
            Some(seed.product_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["price_snapshot"]["sku_id"].as_str(),
            Some(seed.sku_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["price_snapshot"]["scenario_snapshot"]["scenario_code"].as_str(),
            Some("S2")
        );
        assert_eq!(
            json["data"]["data"]["price_snapshot"]["scenario_snapshot"]["selected_sku_role"]
                .as_str(),
            Some("primary")
        );
        assert_eq!(json["data"]["data"]["status"].as_str(), Some("created"));

        let row = client
            .query_one(
                "SELECT
                   product_id::text,
                   sku_id::text,
                   buyer_org_id::text,
                   seller_org_id::text,
                   status,
                   payment_status,
                   delivery_status,
                   acceptance_status,
                   settlement_status,
                   dispute_status,
                   price_snapshot_json
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&order_id],
            )
            .await
            .expect("query order");
        let snapshot: Value = row.get(10);
        assert_eq!(row.get::<_, String>(0), seed.product_id);
        assert_eq!(row.get::<_, String>(1), seed.sku_id);
        assert_eq!(row.get::<_, String>(2), seed.buyer_org_id);
        assert_eq!(row.get::<_, String>(3), seed.seller_org_id);
        assert_eq!(row.get::<_, String>(4), "created");
        assert_eq!(row.get::<_, String>(5), "unpaid");
        assert_eq!(row.get::<_, String>(6), "pending_delivery");
        assert_eq!(row.get::<_, String>(7), "not_started");
        assert_eq!(row.get::<_, String>(8), "not_started");
        assert_eq!(row.get::<_, String>(9), "none");
        assert_eq!(snapshot["billing_mode"].as_str(), Some("one_time"));
        assert_eq!(
            snapshot["scenario_snapshot"]["scenario_code"].as_str(),
            Some("S2")
        );
        assert_eq!(
            snapshot["scenario_snapshot"]["contract_template"].as_str(),
            Some("CONTRACT_FILE_V1")
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'trade.order.create'",
                &[&request_id],
            )
            .await
            .expect("query audit")
            .get(0);
        assert!(audit_count >= 1);
        let outbox_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND aggregate_type = 'trade.order'
                   AND event_type = 'trade.order.created'",
                &[&request_id],
            )
            .await
            .expect("query outbox")
            .get(0);
        assert!(outbox_count >= 1);

        cleanup_graph(&client, &seed, &order_id, &request_id).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, tokio_postgres::Error> {
        let buyer_org = client
            .query_one(
                "INSERT INTO core.organization (
                   org_name, org_type, status, metadata
                 ) VALUES (
                   $1::text, 'enterprise', 'active', '{}'::jsonb
                 )
                 RETURNING org_id::text",
                &[&format!("trade003-buyer-{suffix}")],
            )
            .await?;
        let buyer_org_id: String = buyer_org.get(0);

        let seller_org = client
            .query_one(
                "INSERT INTO core.organization (
                   org_name, org_type, status, metadata
                 ) VALUES (
                   $1::text, 'enterprise', 'active', '{}'::jsonb
                 )
                 RETURNING org_id::text",
                &[&format!("trade003-seller-{suffix}")],
            )
            .await?;
        let seller_org_id: String = seller_org.get(0);

        let asset = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade003-asset-{suffix}"),
                    &format!("trade003 asset desc {suffix}"),
                ],
            )
            .await?;
        let asset_id: String = asset.get(0);

        let version = client
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
            .await?;
        let asset_version_id: String = version.get(0);

        let product = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 88.80, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6, '{"tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade003-product-{suffix}"),
                    &format!("trade003 product desc {suffix}"),
                    &format!("trade003 search text {suffix}"),
                ],
            )
            .await?;
        let product_id: String = product.get(0);

        let sku = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE003-SKU-{suffix}")],
            )
            .await?;
        let sku_id: String = sku.get(0);

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
        })
    }

    async fn cleanup_graph(client: &Client, seed: &SeedGraph, order_id: &str, request_id: &str) {
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event
                 WHERE request_id = $1
                   AND aggregate_type = 'trade.order'
                   AND event_type = 'trade.order.created'",
                &[&request_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&order_id],
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
