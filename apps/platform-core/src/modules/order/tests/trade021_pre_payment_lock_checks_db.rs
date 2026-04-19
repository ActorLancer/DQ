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
    async fn trade021_pre_payment_lock_checks_db_smoke() {
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
        let blocked_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/file-std/transition",
                        seed.order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header(
                        "x-request-id",
                        format!("req-trade021-{suffix}-review-blocked"),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"lock_funds"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(blocked_resp.status(), StatusCode::CONFLICT);
        let blocked_body = to_bytes(blocked_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let blocked_json: Value = serde_json::from_slice(&blocked_body).expect("json");
        let blocked_msg = blocked_json["message"]
            .as_str()
            .or_else(|| blocked_json["error"]["message"].as_str())
            .unwrap_or_default()
            .to_string();
        assert!(blocked_msg.contains("product review status is not approved"));

        client
            .execute(
                r#"UPDATE catalog.product
                 SET metadata = jsonb_set(metadata, '{review_status}', '"approved"'::jsonb, true)
                 WHERE product_id = $1::text::uuid"#,
                &[&seed.product_id],
            )
            .await
            .expect("approve review status");

        let snapshot_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/file-std/transition",
                        seed.order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header(
                        "x-request-id",
                        format!("req-trade021-{suffix}-snapshot-blocked"),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"lock_funds"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(snapshot_resp.status(), StatusCode::CONFLICT);
        let snapshot_body = to_bytes(snapshot_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let snapshot_json: Value = serde_json::from_slice(&snapshot_body).expect("json");
        let snapshot_msg = snapshot_json["message"]
            .as_str()
            .or_else(|| snapshot_json["error"]["message"].as_str())
            .unwrap_or_default()
            .to_string();
        assert!(snapshot_msg.contains("price snapshot is incomplete"));

        client
            .execute(
                r#"UPDATE trade.order_main
                 SET price_snapshot_json = jsonb_build_object(
                   'product_id', $1::text,
                   'sku_id', $2::text,
                   'sku_code', $3::text,
                   'sku_type', 'FILE_STD',
                   'pricing_mode', 'one_time',
                   'unit_price', '66.60',
                   'currency_code', 'CNY',
                   'billing_mode', 'one_time',
                   'refund_mode', 'manual_refund',
                   'settlement_terms', jsonb_build_object(
                     'settlement_basis', 'one_time_delivery',
                     'settlement_mode', 'manual_v1'
                   ),
                   'tax_terms', jsonb_build_object(
                     'tax_policy', 'platform_default',
                     'tax_code', 'VAT',
                     'tax_inclusive', false
                   ),
                   'scenario_snapshot', jsonb_build_object(
                     'scenario_code', 'S2',
                     'scenario_name', '工业质量与产线日报文件包交付',
                     'selected_sku_id', $2::text,
                     'selected_sku_code', $3::text,
                     'selected_sku_type', 'FILE_STD',
                     'selected_sku_role', 'primary',
                     'primary_sku', 'FILE_STD',
                     'supplementary_skus', jsonb_build_array('FILE_SUB'),
                     'contract_template', 'CONTRACT_FILE_V1',
                     'acceptance_template', 'ACCEPT_FILE_V1',
                     'refund_template', 'REFUND_FILE_V1',
                     'per_sku_snapshot_required', true,
                     'multi_sku_requires_independent_contract_authorization_settlement', true
                   ),
                   'captured_at', to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                   'source', 'trade021-smoke'
                 )
                 WHERE order_id = $4::text::uuid"#,
                &[
                    &seed.product_id,
                    &seed.sku_id,
                    &format!("TRADE021-SKU-{suffix}"),
                    &seed.order_id,
                ],
            )
            .await
            .expect("hydrate snapshot");

        let ok_resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/file-std/transition",
                        seed.order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", format!("req-trade021-{suffix}-ok"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"lock_funds"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(ok_resp.status(), StatusCode::OK);

        let order_row = client
            .query_one(
                "SELECT status, payment_status FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order");
        assert_eq!(order_row.get::<_, String>(0), "buyer_locked");
        assert_eq!(order_row.get::<_, String>(1), "paid");

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_order_graph(
        client: &Client,
        suffix: &str,
    ) -> Result<SeedGraph, tokio_postgres::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade021-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade021-seller-{suffix}")],
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
                    &format!("trade021-asset-{suffix}"),
                    &format!("trade021 asset {suffix}"),
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
                   $5, 'listed', 'one_time', 66.60, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"rejected","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade021-product-{suffix}"),
                    &format!("trade021 product {suffix}"),
                    &format!("trade021 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE021-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'created', 'unpaid', 'pending_delivery', 'not_started', 'not_started', 'none',
                   'online', 66.60, 'CNY',
                   '{}'::jsonb
                 )
                 RETURNING order_id::text",
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
                "DELETE FROM audit.audit_event
                 WHERE entity_type = 'order'
                   AND entity_id = $1::text::uuid",
                &[&seed.order_id],
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
