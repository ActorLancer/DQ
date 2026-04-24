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
        query_surface_id: String,
    }

    #[tokio::test]
    async fn dlv010_query_template_db_smoke() {
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

        let create_request_id = format!("req-dlv010-create-v1-{suffix}");
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/query-surfaces/{}/templates",
                        seed.query_surface_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &create_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "template_name": "sales_overview",
                            "template_type": "sql_template",
                            "template_body_ref": format!("minio://delivery-objects/templates/{suffix}/sales_overview_v1.sql"),
                            "parameter_schema_json": {
                                "type": "object",
                                "properties": {
                                    "start_date": {"type": "string"},
                                    "end_date": {"type": "string"},
                                    "limit": {"type": "integer"}
                                }
                            },
                            "analysis_rule_json": {
                                "analysis_rule": "whitelist_only",
                                "template_review_status": "approved"
                            },
                            "result_schema_json": {
                                "fields": [
                                    {"name": "city", "type": "string"},
                                    {"name": "total_amount", "type": "number"},
                                    {"name": "query_date", "type": "string"}
                                ]
                            },
                            "export_policy_json": {
                                "allow_raw_export": false,
                                "allowed_formats": ["json", "csv"],
                                "max_export_rows": 500
                            },
                            "risk_guard_json": {
                                "risk_mode": "strict",
                                "approval_required": true
                            },
                            "whitelist_fields": ["city", "total_amount"],
                            "status": "draft"
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
        let query_template_v1_id = create_json["data"]["query_template_id"]
            .as_str()
            .expect("query_template_v1_id")
            .to_string();
        assert_eq!(create_json["data"]["operation"].as_str(), Some("created"));
        assert_eq!(create_json["data"]["version_no"].as_i64(), Some(1));

        let create_v2_request_id = format!("req-dlv010-create-v2-{suffix}");
        let create_v2_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/query-surfaces/{}/templates",
                        seed.query_surface_id
                    ))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &create_v2_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "template_name": "sales_overview",
                            "template_body_ref": format!("minio://delivery-objects/templates/{suffix}/sales_overview_v2.sql"),
                            "parameter_schema_json": {
                                "type": "object",
                                "properties": {
                                    "start_date": {"type": "string"},
                                    "end_date": {"type": "string"},
                                    "region": {"type": "string"}
                                }
                            },
                            "analysis_rule_json": {
                                "analysis_rule": "whitelist_only",
                                "template_review_status": "approved"
                            },
                            "result_schema_json": {
                                "fields": [
                                    {"name": "city", "type": "string"},
                                    {"name": "total_amount", "type": "number"},
                                    {"name": "confidence", "type": "number"}
                                ]
                            },
                            "export_policy_json": {
                                "allow_raw_export": false,
                                "allowed_formats": ["json"],
                                "max_export_rows": 100
                            },
                            "risk_guard_json": {
                                "risk_mode": "strict",
                                "approval_required": true,
                                "dual_review": true
                            },
                            "whitelist_fields": ["city", "confidence"],
                            "status": "active"
                        })
                        .to_string(),
                    ))
                    .expect("create v2 request"),
            )
            .await
            .expect("create v2 response");
        let create_v2_status = create_v2_response.status();
        let create_v2_body = to_bytes(create_v2_response.into_body(), usize::MAX)
            .await
            .expect("create v2 body");
        assert_eq!(
            create_v2_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&create_v2_body)
        );
        let create_v2_json: Value =
            serde_json::from_slice(&create_v2_body).expect("create v2 json");
        let query_template_v2_id = create_v2_json["data"]["query_template_id"]
            .as_str()
            .expect("query_template_v2_id")
            .to_string();
        assert_eq!(
            create_v2_json["data"]["operation"].as_str(),
            Some("created")
        );
        assert_eq!(create_v2_json["data"]["version_no"].as_i64(), Some(2));

        let update_request_id = format!("req-dlv010-update-v2-{suffix}");
        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/query-surfaces/{}/templates",
                        seed.query_surface_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &update_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_template_id": query_template_v2_id,
                            "template_name": "sales_overview",
                            "template_body_ref": format!("minio://delivery-objects/templates/{suffix}/sales_overview_v2_patch.sql"),
                            "analysis_rule_json": {
                                "analysis_rule": "whitelist_only",
                                "template_review_status": "approved",
                                "requires_attestation": true
                            },
                            "result_schema_json": {
                                "fields": [
                                    {"name": "city", "type": "string"},
                                    {"name": "total_amount", "type": "number"},
                                    {"name": "confidence", "type": "number"}
                                ]
                            },
                            "export_policy_json": {
                                "allow_raw_export": false,
                                "allowed_formats": ["json"],
                                "max_export_rows": 80
                            },
                            "risk_guard_json": {
                                "risk_mode": "strict",
                                "approval_required": true,
                                "dual_review": true
                            },
                            "whitelist_fields": ["city", "total_amount", "confidence"],
                            "status": "active"
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
        assert_eq!(
            update_json["data"]["query_template_id"].as_str(),
            Some(query_template_v2_id.as_str())
        );
        assert_eq!(update_json["data"]["operation"].as_str(), Some("updated"));
        assert_eq!(update_json["data"]["version_no"].as_i64(), Some(2));
        assert_eq!(
            update_json["data"]["whitelist_fields"],
            json!(["city", "total_amount", "confidence"])
        );

        let invalid_request_id = format!("req-dlv010-invalid-{suffix}");
        let invalid_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/query-surfaces/{}/templates",
                        seed.query_surface_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &invalid_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "template_name": "invalid_template",
                            "parameter_schema_json": {"type": "object"},
                            "analysis_rule_json": {"analysis_rule": "whitelist_only"},
                            "result_schema_json": {
                                "fields": [
                                    {"name": "city", "type": "string"}
                                ]
                            },
                            "export_policy_json": {
                                "allow_raw_export": false,
                                "allowed_formats": ["json"]
                            },
                            "risk_guard_json": {
                                "risk_mode": "strict"
                            },
                            "whitelist_fields": ["missing_field"]
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

        let version_rows = client
            .query(
                "SELECT query_template_id::text,
                        version_no,
                        status,
                        template_body_ref,
                        analysis_rule_json -> 'whitelist_fields',
                        export_policy_json -> 'whitelist_fields'
                 FROM delivery.query_template_definition
                 WHERE query_surface_id = $1::text::uuid
                   AND template_name = 'sales_overview'
                 ORDER BY version_no",
                &[&seed.query_surface_id],
            )
            .await
            .expect("query template rows");
        assert_eq!(version_rows.len(), 2);
        assert_eq!(version_rows[0].get::<_, String>(0), query_template_v1_id);
        assert_eq!(version_rows[0].get::<_, i32>(1), 1);
        assert_eq!(version_rows[0].get::<_, String>(2), "draft");
        assert_eq!(
            version_rows[1].get::<_, Option<String>>(3).as_deref(),
            Some(
                format!("minio://delivery-objects/templates/{suffix}/sales_overview_v2_patch.sql")
                    .as_str(),
            )
        );
        assert_eq!(version_rows[1].get::<_, i32>(1), 2);
        assert_eq!(version_rows[1].get::<_, String>(2), "active");
        assert_eq!(
            version_rows[1].get::<_, Value>(4),
            json!(["city", "total_amount", "confidence"])
        );
        assert_eq!(
            version_rows[1].get::<_, Value>(5),
            json!(["city", "total_amount", "confidence"])
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.query_template.manage'
                   AND request_id IN ($1, $2, $3)",
                &[
                    &create_request_id,
                    &create_v2_request_id,
                    &update_request_id,
                ],
            )
            .await
            .expect("query template audit")
            .get(0);
        assert_eq!(audit_count, 3);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv010-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'low', 'active', 'dlv010 query asset'
                 )
                 RETURNING asset_id::text",
                &[&seller_org_id, &format!("dlv010-asset-{suffix}")],
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
                   8192, 'CN', ARRAY['CN']::text[], true,
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
                    &format!("dlv010-structured-{suffix}"),
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
                   $5, 'listed', 'subscription', 88.00, 'CNY', 'template_query',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv010-product-{suffix}"),
                    &format!("dlv010 product {suffix}"),
                    &format!("dlv010 search {suffix}"),
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
                    &format!("dlv010-connector-{suffix}"),
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
                    &format!("dlv010-env-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let query_surface_id = create_query_surface(
            client,
            &product_id,
            &seller_org_id,
            &asset_object_id,
            &environment_id,
            suffix,
        )
        .await?;

        Ok(SeedGraph {
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            asset_object_id,
            connector_id,
            environment_id,
            query_surface_id,
        })
    }

    async fn create_query_surface(
        _client: &Client,
        product_id: &str,
        seller_org_id: &str,
        asset_object_id: &str,
        environment_id: &str,
        suffix: &str,
    ) -> Result<String, db::Error> {
        let app = crate::with_live_test_state(delivery_router()).await;
        let request_id = format!("req-dlv010-seed-surface-{suffix}");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/products/{product_id}/query-surfaces"))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", seller_org_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "asset_object_id": asset_object_id,
                            "environment_id": environment_id,
                            "surface_type": "template_query_lite",
                            "binding_mode": "managed_surface",
                            "execution_scope": "curated_zone",
                            "input_contract_json": {
                                "source_zones": ["curated", "product_zone"]
                            },
                            "output_boundary_json": {
                                "max_rows": 100,
                                "max_cells": 2000,
                                "allow_raw_export": false,
                                "allowed_formats": ["json", "csv"]
                            },
                            "query_policy_json": {
                                "analysis_rule": "whitelist_only",
                                "template_review_status": "approved"
                            },
                            "quota_policy_json": {
                                "daily_limit": 60,
                                "billing_unit": "query_count"
                            },
                            "status": "active"
                        })
                        .to_string(),
                    ))
                    .expect("seed surface request"),
            )
            .await
            .expect("seed surface response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("seed surface body");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let payload: Value = serde_json::from_slice(&body).expect("seed surface json");
        Ok(payload["data"]["query_surface_id"]
            .as_str()
            .expect("seed query_surface_id")
            .to_string())
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM delivery.query_template_definition WHERE query_surface_id = $1::text::uuid",
                &[&seed.query_surface_id],
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
