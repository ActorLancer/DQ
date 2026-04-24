#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        seller_org_id: String,
        buyer_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        connector_id: String,
        environment_id: String,
        query_surface_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn dlv022_sensitive_execution_policy_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_millis()
            .to_string();
        let seed = seed_graph(&client, &suffix).await.expect("seed graph");
        let app = crate::with_live_test_state(delivery_router()).await;
        let request_id = format!("req-dlv022-{suffix}");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/sensitive-execution-policies",
                        seed.order_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "query_surface_id": seed.query_surface_id,
                            "output_boundary_json": {
                                "allow_export": false,
                                "allow_raw_export": false,
                                "allowed_formats": ["json"],
                                "requires_disclosure_review": true,
                                "max_rows": 500
                            },
                            "export_control_json": {
                                "allow_export": false,
                                "allow_raw_export": false,
                                "allowed_formats": ["json"],
                                "network_access": "deny",
                                "copy_control": "deny",
                                "max_exports": 0
                            },
                            "step_up_required": true,
                            "attestation_required": false
                        })
                        .to_string(),
                    ))
                    .expect("request build"),
            )
            .await
            .expect("response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let json: Value = serde_json::from_slice(&body).expect("response json");
        let data = &json["data"];
        let policy_id = data["sensitive_execution_policy_id"]
            .as_str()
            .expect("policy id")
            .to_string();
        assert_eq!(data["sku_type"].as_str(), Some("QRY_LITE"));
        assert_eq!(
            data["policy"]["execution_mode"].as_str(),
            Some("template_query_lite")
        );
        assert_eq!(data["policy"]["policy_scope"].as_str(), Some("order"));
        assert_eq!(data["policy"]["step_up_required"].as_bool(), Some(true));
        assert_eq!(
            data["policy"]["output_boundary_json"]["requires_disclosure_review"].as_bool(),
            Some(true)
        );

        let row = client
            .query_one(
                "SELECT execution_mode,
                        policy_scope,
                        step_up_required,
                        attestation_required,
                        output_boundary_json ->> 'max_rows',
                        policy_snapshot -> 'order' ->> 'delivery_route_snapshot'
                 FROM delivery.sensitive_execution_policy
                 WHERE sensitive_execution_policy_id = $1::text::uuid",
                &[&policy_id],
            )
            .await
            .expect("policy row");
        assert_eq!(row.get::<_, String>(0), "template_query_lite");
        assert_eq!(row.get::<_, String>(1), "order");
        assert!(row.get::<_, bool>(2));
        assert!(!row.get::<_, bool>(3));
        assert_eq!(row.get::<_, Option<String>>(4).as_deref(), Some("500"));
        assert_eq!(
            row.get::<_, Option<String>>(5).as_deref(),
            Some("template_query")
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.sensitive_execution.manage'
                   AND request_id = $1",
                &[&request_id],
            )
            .await
            .expect("audit count")
            .get(0);
        assert_eq!(audit_count, 1);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv022-seller-{suffix}")],
            )
            .await?
            .get(0);
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv022-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'high', 'active', 'dlv022 sensitive asset'
                 ) RETURNING asset_id::text",
                &[&seller_org_id, &format!("dlv022-asset-{suffix}")],
            )
            .await?
            .get(0);
        let asset_version_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   4096, 'CN', ARRAY['CN']::text[], true,
                   '{"required_execution_boundary":"template_query_lite"}'::jsonb, 'active'
                 ) RETURNING asset_version_id::text"#,
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
                   'dlv022 sensitive product', 'listed', 'subscription', 66.00, 'CNY', 'template_query',
                   ARRAY['internal_use']::text[], $5,
                   '{"review_status":"approved","sellable_status":"active"}'::jsonb
                 ) RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv022-product-{suffix}"),
                    &format!("dlv022-search-{suffix}"),
                ],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'QRY_LITE', '次', 'usage', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("DLV022-QRY-{suffix}")],
            )
            .await?
            .get(0);
        let connector_id: String = client
            .query_one(
                r#"INSERT INTO core.connector (
                   org_id, connector_name, connector_type, status, version,
                   network_zone, health_status, endpoint_ref, metadata
                 ) VALUES (
                   $1::text::uuid, $2, 'query_runtime', 'active', 'v1',
                   'private', 'healthy', $3, '{"provider":"local"}'::jsonb
                 ) RETURNING connector_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv022-connector-{suffix}"),
                    &format!("https://connector.example.com/{suffix}"),
                ],
            )
            .await?
            .get(0);
        let environment_id: String = client
            .query_one(
                r#"INSERT INTO core.execution_environment (
                   org_id, connector_id, environment_name, environment_type,
                   status, network_zone, region_code, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, 'query_runtime',
                   'active', 'private', 'cn-east-1', '{"mode":"template_query"}'::jsonb
                 ) RETURNING environment_id::text"#,
                &[
                    &seller_org_id,
                    &connector_id,
                    &format!("dlv022-env-{suffix}"),
                ],
            )
            .await?
            .get(0);
        let query_surface_id: String = client
            .query_one(
                r#"INSERT INTO catalog.query_surface_definition (
                   asset_version_id, asset_object_id, environment_id, surface_type,
                   binding_mode, execution_scope, input_contract_json, output_boundary_json,
                   query_policy_json, quota_policy_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, NULL, $2::text::uuid, 'template_query_lite',
                   'managed_surface', 'curated_zone', '{}'::jsonb,
                   '{"allow_export":false,"allow_raw_export":false,"allowed_formats":["json"],"requires_disclosure_review":false}'::jsonb,
                   '{"step_up_required":true,"attestation_required":false,"network_access":"deny"}'::jsonb,
                   '{"daily_runs":10}'::jsonb,
                   'active', '{"surface":"dlv022"}'::jsonb
                 ) RETURNING query_surface_id::text"#,
                &[&asset_version_id, &environment_id],
            )
            .await?
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json, delivery_route_snapshot, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none',
                   'online', 66.00, 'CNY', '{}'::jsonb, 'template_query', 'delivery_template_query_enabled'
                 ) RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);

        Ok(SeedGraph {
            seller_org_id,
            buyer_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            connector_id,
            environment_id,
            query_surface_id,
            order_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM delivery.sensitive_execution_policy WHERE order_id = $1::text::uuid",
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
                "DELETE FROM catalog.query_surface_definition WHERE query_surface_id = $1::text::uuid",
                &[&seed.query_surface_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.execution_environment WHERE environment_id = $1::text::uuid",
                &[&seed.environment_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.connector WHERE connector_id = $1::text::uuid",
                &[&seed.connector_id],
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
