#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        asset_object_id: String,
        connector_id: String,
        environment_id: String,
    }

    #[tokio::test]
    async fn dlv009_query_surface_db_smoke() {
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

        let create_request_id = format!("req-dlv009-create-{suffix}");
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/products/{}/query-surfaces",
                        seed.product_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &create_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "asset_object_id": seed.asset_object_id,
                            "environment_id": seed.environment_id,
                            "surface_type": "template_query_lite",
                            "binding_mode": "managed_surface",
                            "execution_scope": "curated_zone",
                            "input_contract_json": {
                                "source_zones": ["curated", "product_zone"],
                                "parameter_schema_ref": "schema://dlv009/params/v1"
                            },
                            "output_boundary_json": {
                                "max_rows": 100,
                                "max_cells": 2000,
                                "allow_raw_export": false,
                                "allowed_formats": ["json", "csv"]
                            },
                            "query_policy_json": {
                                "analysis_rule": "aggregate_only",
                                "template_review_status": "approved"
                            },
                            "quota_policy_json": {
                                "daily_limit": 60,
                                "billing_unit": "query_count"
                            },
                            "status": "draft",
                            "metadata": {
                                "owner": "seller"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("create request"),
            )
            .await
            .expect("create response");
        let create_status = create_response.status();
        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create body");
        assert_eq!(
            create_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&create_body)
        );
        let create_json: Value = serde_json::from_slice(&create_body).expect("create json");
        let query_surface_id = create_json["data"]["query_surface_id"]
            .as_str()
            .expect("query_surface_id")
            .to_string();
        assert_eq!(create_json["data"]["operation"].as_str(), Some("created"));
        assert_eq!(
            create_json["data"]["execution_scope"].as_str(),
            Some("curated_zone")
        );

        let update_request_id = format!("req-dlv009-update-{suffix}");
        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/products/{}/query-surfaces",
                        seed.product_id
                    ))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &update_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_surface_id": query_surface_id,
                            "asset_object_id": seed.asset_object_id,
                            "environment_id": seed.environment_id,
                            "execution_scope": "product_zone",
                            "input_contract_json": {
                                "source_zones": ["product_zone"],
                                "read_zones": ["curated_zone", "product_zone"]
                            },
                            "output_boundary_json": {
                                "max_rows": 200,
                                "max_cells": 5000,
                                "allow_raw_export": false,
                                "allowed_formats": ["json"]
                            },
                            "query_policy_json": {
                                "analysis_rule": "whitelist_only",
                                "approved_template_count": 3
                            },
                            "quota_policy_json": {
                                "daily_limit": 120,
                                "monthly_limit": 2400
                            },
                            "status": "active",
                            "metadata": {
                                "owner": "seller",
                                "updated_by": "tenant_admin"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("update request"),
            )
            .await
            .expect("update response");
        let update_status = update_response.status();
        let update_body = to_bytes(update_response.into_body(), usize::MAX)
            .await
            .expect("update body");
        assert_eq!(
            update_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&update_body)
        );
        let update_json: Value = serde_json::from_slice(&update_body).expect("update json");
        assert_eq!(update_json["data"]["operation"].as_str(), Some("updated"));
        assert_eq!(update_json["data"]["status"].as_str(), Some("active"));
        assert_eq!(
            update_json["data"]["execution_scope"].as_str(),
            Some("product_zone")
        );
        assert_eq!(
            update_json["data"]["query_policy_json"]["analysis_rule"].as_str(),
            Some("whitelist_only")
        );

        let invalid_request_id = format!("req-dlv009-invalid-{suffix}");
        let invalid_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/products/{}/query-surfaces",
                        seed.product_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &invalid_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "environment_id": seed.environment_id,
                            "execution_scope": "raw_zone",
                            "input_contract_json": {
                                "source_zones": ["raw"]
                            }
                        })
                        .to_string(),
                    ))
                    .expect("invalid request"),
            )
            .await
            .expect("invalid response");
        let invalid_status = invalid_response.status();
        let invalid_body = to_bytes(invalid_response.into_body(), usize::MAX)
            .await
            .expect("invalid body");
        assert_eq!(
            invalid_status,
            StatusCode::CONFLICT,
            "{}",
            String::from_utf8_lossy(&invalid_body)
        );

        let surface_row = client
            .query_one(
                "SELECT execution_scope,
                        status,
                        output_boundary_json ->> 'max_rows',
                        query_policy_json ->> 'analysis_rule',
                        quota_policy_json ->> 'daily_limit'
                 FROM catalog.query_surface_definition
                 WHERE query_surface_id = $1::text::uuid",
                &[&query_surface_id],
            )
            .await
            .expect("query surface row");
        assert_eq!(surface_row.get::<_, String>(0), "product_zone");
        assert_eq!(surface_row.get::<_, String>(1), "active");
        assert_eq!(
            surface_row.get::<_, Option<String>>(2).as_deref(),
            Some("200")
        );
        assert_eq!(
            surface_row.get::<_, Option<String>>(3).as_deref(),
            Some("whitelist_only")
        );
        assert_eq!(
            surface_row.get::<_, Option<String>>(4).as_deref(),
            Some("120")
        );

        let asset_version_row = client
            .query_one(
                "SELECT query_surface_type
                 FROM catalog.asset_version
                 WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
            )
            .await
            .expect("asset version row");
        assert_eq!(
            asset_version_row.get::<_, Option<String>>(0).as_deref(),
            Some("template_query_lite")
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.query_surface.manage'
                   AND request_id IN ($1, $2)",
                &[&create_request_id, &update_request_id],
            )
            .await
            .expect("query surface audit")
            .get(0);
        assert_eq!(audit_count, 2);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv009-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'low', 'active', 'dlv009 query asset'
                 )
                 RETURNING asset_id::text",
                &[&seller_org_id, &format!("dlv009-asset-{suffix}")],
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
                   '{"query_mode":"controlled"}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await?
            .get(0);

        let asset_object_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_object_binding (
                   asset_version_id, object_kind, object_name, object_locator,
                   schema_json, output_schema_json, freshness_json, access_constraints, metadata
                 ) VALUES (
                   $1::text::uuid, 'structured_dataset', $2, $3,
                   '{"fields":[{"name":"city"},{"name":"amount"}]}'::jsonb,
                   '{"fields":[{"name":"city"},{"name":"total_amount"}]}'::jsonb,
                   '{}'::jsonb,
                   '{"preview":false}'::jsonb,
                   '{"zone":"curated"}'::jsonb
                 )
                 RETURNING asset_object_id::text"#,
                &[
                    &asset_version_id,
                    &format!("dlv009-structured-{suffix}"),
                    &format!("warehouse://curated/{suffix}/sales"),
                ],
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
                   $5, 'listed', 'subscription', 66.00, 'CNY', 'template_query',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv009-product-{suffix}"),
                    &format!("dlv009 product {suffix}"),
                    &format!("dlv009 search {suffix}"),
                ],
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
                 )
                 RETURNING connector_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv009-connector-{suffix}"),
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
                 )
                 RETURNING environment_id::text"#,
                &[
                    &seller_org_id,
                    &connector_id,
                    &format!("dlv009-env-{suffix}"),
                ],
            )
            .await?
            .get(0);

        Ok(SeedGraph {
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            asset_object_id,
            connector_id,
            environment_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM catalog.query_surface_definition WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
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
                "DELETE FROM catalog.asset_object_binding WHERE asset_object_id = $1::text::uuid",
                &[&seed.asset_object_id],
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
                &[&seed.seller_org_id],
            )
            .await;
    }
}
