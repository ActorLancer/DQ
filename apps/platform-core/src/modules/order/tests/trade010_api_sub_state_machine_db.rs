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
    async fn trade010_api_sub_state_machine_db_smoke() {
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
        for action in [
            "lock_funds",
            "bind_application",
            "issue_api_key",
            "trial_call",
            "activate_subscription",
            "bill_cycle",
            "terminate_subscription",
        ] {
            let request_id = format!("req-trade010-{suffix}-{action}");
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!(
                            "/api/v1/orders/{}/api-sub/transition",
                            seed.order_id
                        ))
                        .header("x-role", "buyer_operator")
                        .header("x-tenant-id", &seed.buyer_org_id)
                        .header("x-request-id", &request_id)
                        .header("content-type", "application/json")
                        .body(Body::from(format!(r#"{{"action":"{action}"}}"#)))
                        .expect("request should build"),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::OK, "{action} should succeed");
        }

        let closed_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order");
        assert_eq!(closed_row.get::<_, String>(0), "closed");
        assert_eq!(closed_row.get::<_, String>(1), "paid");
        assert_eq!(closed_row.get::<_, String>(2), "closed");
        assert_eq!(closed_row.get::<_, String>(3), "closed");
        assert_eq!(closed_row.get::<_, String>(4), "closed");
        assert_eq!(closed_row.get::<_, String>(5), "none");

        let conflict_resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/api-sub/transition",
                        seed.order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", format!("req-trade010-{suffix}-invalid"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"bill_cycle"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(conflict_resp.status(), StatusCode::CONFLICT);
        let body = to_bytes(conflict_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        let msg = json["message"]
            .as_str()
            .or_else(|| json["error"]["message"].as_str())
            .unwrap_or_default()
            .to_string();
        assert!(msg.contains("API_SUB_TRANSITION_FORBIDDEN"));

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
                &[&format!("trade010-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade010-seller-{suffix}")],
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
                    &format!("trade010-asset-{suffix}"),
                    &format!("trade010 asset {suffix}"),
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
                   $5, 'listed', 'one_time', 28.80, 'CNY', 'api_service',
                   ARRAY['internal_use']::text[], $6,
                   '{"tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade010-product-{suffix}"),
                    &format!("trade010 product {suffix}"),
                    &format!("trade010 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'API_SUB', '月', 'subscription', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE010-SKU-{suffix}")],
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
                   'online', 28.80, 'CNY',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', $6::text,
                     'sku_type', 'API_SUB',
                     'pricing_mode', 'one_time',
                     'unit_price', '28.80',
                     'currency_code', 'CNY',
                     'billing_mode', 'subscription',
                     'refund_mode', 'manual_refund',
                     'settlement_terms', jsonb_build_object(
                       'settlement_basis', 'subscription_cycle',
                       'settlement_mode', 'manual_v1'
                     ),
                     'tax_terms', jsonb_build_object(
                       'tax_policy', 'platform_default',
                       'tax_code', 'VAT',
                       'tax_inclusive', false
                     ),
                     'scenario_snapshot', jsonb_build_object(
                       'scenario_code', 'S1',
                       'scenario_name', '工业设备运行指标 API 订阅',
                       'selected_sku_id', $5::text,
                       'selected_sku_code', $6::text,
                       'selected_sku_type', 'API_SUB',
                       'selected_sku_role', 'primary',
                       'primary_sku', 'API_SUB',
                       'supplementary_skus', jsonb_build_array('API_PPU'),
                       'contract_template', 'CONTRACT_API_SUB_V1',
                       'acceptance_template', 'ACCEPT_API_SUB_V1',
                       'refund_template', 'REFUND_API_SUB_V1',
                       'per_sku_snapshot_required', true,
                       'multi_sku_requires_independent_contract_authorization_settlement', true
                     ),
                     'captured_at', to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                     'source', 'trade010-seed'
                   )
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("TRADE010-SKU-{suffix}"),
                ],
            )
            .await?
            .get(0);

        client
            .execute(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb
                 )",
                &[&order_id, &format!("sha256:trade010:{suffix}")],
            )
            .await?;

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
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid OR org_id = $2::text::uuid",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await;
    }
}
