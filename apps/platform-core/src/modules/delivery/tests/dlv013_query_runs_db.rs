#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::storage::application::delete_object;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        buyer_user_id: String,
        approval_ticket_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        asset_object_id: String,
        connector_id: String,
        environment_id: String,
        query_surface_id: String,
        template_v1_id: String,
        template_v2_id: String,
        template_query_grant_id: String,
    }

    #[tokio::test]
    async fn dlv013_query_runs_db_smoke() {
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

        let mut objects_to_delete = Vec::new();
        let first_run = execute_run(
            app.clone(),
            &seed,
            &suffix,
            "first",
            json!({
                "start_date": "2026-01-01",
                "limit": 1,
                "country": "CN"
            }),
        )
        .await;
        objects_to_delete.push((first_run.1.clone(), first_run.2.clone()));

        let second_run = execute_run(
            app.clone(),
            &seed,
            &suffix,
            "second",
            json!({
                "start_date": "2026-02-01",
                "limit": 2,
                "country": "US"
            }),
        )
        .await;
        objects_to_delete.push((second_run.1.clone(), second_run.2.clone()));

        let read_request_id = format!("req-dlv013-read-{suffix}");
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}/template-runs", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", &read_request_id)
                    .body(Body::empty())
                    .expect("read request"),
            )
            .await
            .expect("read response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let json: Value = serde_json::from_slice(&body).expect("read json");
        let data = &json["data"];
        assert_eq!(data["order_id"].as_str(), Some(seed.order_id.as_str()));
        assert_eq!(data["current_state"].as_str(), Some("query_executed"));
        assert_eq!(data["payment_status"].as_str(), Some("paid"));
        assert_eq!(data["delivery_status"].as_str(), Some("delivered"));
        let query_runs = data["query_runs"].as_array().expect("query_runs");
        assert_eq!(query_runs.len(), 2);
        assert_eq!(
            query_runs[0]["query_run_id"].as_str(),
            Some(second_run.0.as_str())
        );
        assert_eq!(
            query_runs[1]["query_run_id"].as_str(),
            Some(first_run.0.as_str())
        );
        assert_eq!(
            query_runs[0]["query_template_name"].as_str(),
            Some("sales_overview")
        );
        assert_eq!(query_runs[0]["query_template_version"].as_i64(), Some(1));
        assert_eq!(
            query_runs[0]["parameter_summary_json"]["parameter_count"].as_i64(),
            Some(3)
        );
        assert_eq!(
            query_runs[0]["policy_hits"][0].as_str(),
            Some("template_whitelist_passed")
        );
        assert_eq!(
            query_runs[0]["audit_refs"][0]["action_name"].as_str(),
            Some("delivery.template_query.use")
        );
        assert_eq!(
            query_runs[0]["audit_refs"][0]["result_code"].as_str(),
            Some("completed")
        );
        assert_eq!(query_runs[0]["status"].as_str(), Some("completed"));
        assert_eq!(
            query_runs[0]["bucket_name"].as_str(),
            Some("report-results")
        );
        assert!(
            query_runs[0]["object_key"]
                .as_str()
                .unwrap_or_default()
                .contains(&seed.order_id)
        );
        assert_eq!(query_runs[0]["result_row_count"].as_i64(), Some(2));
        assert_eq!(query_runs[0]["export_attempt_count"].as_i64(), Some(0));

        let read_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE ref_id = $1::text::uuid
                   AND action_name = 'delivery.template_query.run.read'",
                &[&seed.order_id],
            )
            .await
            .expect("query read audit count")
            .get(0);
        assert_eq!(read_audit_count, 1);

        crate::write_test026_artifact(
            "dlv013-query-runs.json",
            &json!({
                "test_id": "dlv013_query_runs_db_smoke",
                "order_id": seed.order_id,
                "read_request_id": read_request_id,
                "query_runs_response": {
                    "current_state": data["current_state"],
                    "payment_status": data["payment_status"],
                    "delivery_status": data["delivery_status"],
                    "query_run_count": query_runs.len(),
                    "latest_query_run_id": query_runs[0]["query_run_id"],
                    "latest_query_template_name": query_runs[0]["query_template_name"],
                    "latest_result_row_count": query_runs[0]["result_row_count"],
                    "latest_policy_hits": query_runs[0]["policy_hits"],
                    "latest_audit_refs": query_runs[0]["audit_refs"],
                    "latest_bucket_name": query_runs[0]["bucket_name"],
                    "latest_object_key": query_runs[0]["object_key"],
                },
                "runs": {
                    "first": {
                        "query_run_id": first_run.0,
                        "bucket_name": first_run.1,
                        "object_key": first_run.2,
                    },
                    "second": {
                        "query_run_id": second_run.0,
                        "bucket_name": second_run.1,
                        "object_key": second_run.2,
                    }
                },
                "audit_counts": {
                    "delivery_template_query_run_read": read_audit_count,
                }
            }),
        );

        for (bucket_name, object_key) in objects_to_delete {
            let _ = delete_object(&bucket_name, &object_key).await;
        }
        cleanup_seed_graph(&client, &seed).await;
    }

    async fn execute_run(
        app: axum::Router,
        seed: &SeedGraph,
        suffix: &str,
        label: &str,
        request_payload_json: Value,
    ) -> (String, String, String) {
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/template-runs", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", format!("req-dlv013-{label}-{suffix}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_template_id": seed.template_v1_id,
                            "template_query_grant_id": seed.template_query_grant_id,
                            "requester_user_id": seed.buyer_user_id,
                            "request_payload_json": request_payload_json,
                            "output_boundary_json": {
                                "selected_format": "json",
                                "allowed_formats": ["json"],
                                "max_rows": 3,
                                "max_cells": 9
                            },
                            "approval_ticket_id": seed.approval_ticket_id,
                            "execution_metadata_json": {
                                "entrypoint": format!("dlv013-{label}")
                            }
                        })
                        .to_string(),
                    ))
                    .expect("execute request"),
            )
            .await
            .expect("execute response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("execute body");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let json: Value = serde_json::from_slice(&body).expect("execute json");
        (
            json["data"]["query_run_id"]
                .as_str()
                .expect("query_run_id")
                .to_string(),
            json["data"]["bucket_name"]
                .as_str()
                .expect("bucket_name")
                .to_string(),
            json["data"]["object_key"]
                .as_str()
                .expect("object_key")
                .to_string(),
        )
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                r#"INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{"risk_status":"normal"}'::jsonb)
                 RETURNING org_id::text"#,
                &[&format!("dlv013-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                r#"INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{"risk_status":"normal"}'::jsonb)
                 RETURNING org_id::text"#,
                &[&format!("dlv013-seller-{suffix}")],
            )
            .await?
            .get(0);

        let buyer_user_id: String = client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status)
                 VALUES ($1::text::uuid, $2, $3, 'person', 'active', 'disabled')
                 RETURNING user_id::text",
                &[
                    &buyer_org_id,
                    &format!("dlv013-buyer-user-{suffix}"),
                    &format!("DLV013 Buyer {suffix}"),
                ],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                r#"INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'low', 'active', $3
                 )
                 RETURNING asset_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv013-asset-{suffix}"),
                    &format!("dlv013 asset {suffix}"),
                ],
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
                   '{"fields":[{"name":"city"},{"name":"amount"},{"name":"country"}]}'::jsonb,
                   '{"fields":[{"name":"city"},{"name":"total_amount"},{"name":"country"}]}'::jsonb,
                   '{}'::jsonb,
                   '{"preview":false}'::jsonb,
                   '{"zone":"curated"}'::jsonb
                 )
                 RETURNING asset_object_id::text"#,
                &[
                    &asset_version_id,
                    &format!("dlv013-structured-{suffix}"),
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
                   $5, 'listed', 'usage', 48.00, 'CNY', 'template_query',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv013-product-{suffix}"),
                    &format!("dlv013 product {suffix}"),
                    &format!("dlv013 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
                   delivery_object_kind, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'QRY_LITE', '次', 'usage', 'query_service',
                   'template_grant', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV013-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json, delivery_route_snapshot, trust_boundary_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'template_authorized', 'paid', 'in_progress', 'not_started', 'pending_settlement', 'none',
                   'online', 48.00, 'CNY',
                   '{"query_surface_type":"template_query_lite"}'::jsonb,
                   'template_query',
                   '{"query_delivery":"controlled"}'::jsonb
                 )
                 RETURNING order_id::text"#,
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);

        client
            .execute(
                r#"INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{"term_days":30}'::jsonb
                 )"#,
                &[&order_id, &format!("sha256:dlv013:{suffix}")],
            )
            .await?;

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
                    &format!("dlv013-connector-{suffix}"),
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
                    &format!("dlv013-env-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let query_surface_id: String = client
            .query_one(
                r#"INSERT INTO catalog.query_surface_definition (
                   asset_version_id, asset_object_id, environment_id, surface_type, binding_mode,
                   execution_scope, input_contract_json, output_boundary_json, query_policy_json,
                   quota_policy_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, 'template_query_lite', 'managed_surface',
                   'curated_zone',
                   '{"source_zones":["curated_zone"]}'::jsonb,
                   '{"max_rows":10,"max_cells":30,"allow_raw_export":false,"allowed_formats":["json"]}'::jsonb,
                   '{"analysis_rule":"whitelist_only","template_review_status":"approved"}'::jsonb,
                   '{"daily_limit":10,"monthly_limit":100}'::jsonb,
                   'active',
                   '{"owner":"seller"}'::jsonb
                 )
                 RETURNING query_surface_id::text"#,
                &[&asset_version_id, &asset_object_id, &environment_id],
            )
            .await?
            .get(0);

        let template_v1_id =
            insert_template(client, &query_surface_id, suffix, "sales_overview", 1).await?;
        let template_v2_id =
            insert_template(client, &query_surface_id, suffix, "sales_breakdown", 2).await?;

        let template_query_grant_id: String = client
            .query_one(
                r#"INSERT INTO delivery.template_query_grant (
                   order_id, asset_object_id, environment_id, query_surface_id, template_type,
                   template_digest, allowed_template_ids, execution_rule_snapshot,
                   output_boundary_json, run_quota_json, grant_status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, 'sql_template',
                   $5, $6::jsonb,
                   $7::jsonb,
                   '{"allow_raw_export":false,"allowed_formats":["json"],"max_rows":5,"max_cells":10}'::jsonb,
                   '{"max_runs":5,"daily_limit":3,"monthly_limit":10}'::jsonb,
                   'active'
                 )
                 RETURNING template_query_grant_id::text"#,
                &[
                    &order_id,
                    &asset_object_id,
                    &environment_id,
                    &query_surface_id,
                    &format!("sha256:template-grant:{suffix}"),
                    &json!([template_v1_id.clone(), template_v2_id.clone()]),
                    &json!({
                        "query_surface_id": query_surface_id,
                        "allowed_template_ids": [template_v1_id.clone(), template_v2_id.clone()],
                        "template_summaries": [
                            {"query_template_id": template_v1_id, "template_name": "sales_overview", "version_no": 1},
                            {"query_template_id": template_v2_id, "template_name": "sales_breakdown", "version_no": 2}
                        ]
                    }),
                ],
            )
            .await?
            .get(0);

        let approval_ticket_id: String = client
            .query_one(
                "INSERT INTO ops.approval_ticket (ticket_type, ref_type, ref_id, requested_by, status, requires_second_review)
                 VALUES ('query_run', 'order', $1::text::uuid, $2::text::uuid, 'approved', false)
                 RETURNING approval_ticket_id::text",
                &[&order_id, &buyer_user_id],
            )
            .await?
            .get(0);

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            buyer_user_id,
            approval_ticket_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            asset_object_id,
            connector_id,
            environment_id,
            query_surface_id,
            template_v1_id,
            template_v2_id,
            template_query_grant_id,
        })
    }

    async fn insert_template(
        client: &Client,
        query_surface_id: &str,
        suffix: &str,
        template_name: &str,
        version_no: i32,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                r#"INSERT INTO delivery.query_template_definition (
                   query_surface_id, template_name, template_type, template_body_ref,
                   parameter_schema_json, analysis_rule_json, result_schema_json,
                   export_policy_json, risk_guard_json, status, version_no
                 ) VALUES (
                   $1::text::uuid, $2, 'sql_template', $3,
                   '{"type":"object","properties":{"start_date":{"type":"string"},"limit":{"type":"integer","minimum":1},"country":{"type":"string","enum":["CN","US"]}},"required":["start_date","limit"]}'::jsonb,
                   '{"analysis_rule":"whitelist_only","template_review_status":"approved","whitelist_fields":["city","total_amount"]}'::jsonb,
                   '{"fields":[{"name":"city","type":"string"},{"name":"total_amount","type":"number"},{"name":"country","type":"string"}]}'::jsonb,
                   '{"allow_raw_export":false,"allowed_formats":["json"],"max_export_rows":5,"max_export_cells":10,"whitelist_fields":["city","total_amount"]}'::jsonb,
                   '{"risk_mode":"strict","approval_required":true}'::jsonb,
                   'active',
                   $4
                 )
                 RETURNING query_template_id::text"#,
                &[
                    &query_surface_id,
                    &template_name,
                    &format!(
                        "minio://delivery-objects/templates/{suffix}/{template_name}_v{version_no}.sql"
                    ),
                    &version_no,
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM delivery.storage_object WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.buyer_org_id, &seed.seller_org_id],
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
                "DELETE FROM ops.approval_ticket WHERE approval_ticket_id = $1::text::uuid",
                &[&seed.approval_ticket_id],
            )
            .await;
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
                "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
                &[&seed.buyer_user_id],
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
