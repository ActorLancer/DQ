#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::storage::application::{delete_object, fetch_object_bytes};
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
    }

    #[tokio::test]
    async fn dlv012_template_run_db_smoke() {
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

        let success_request_id = format!("req-dlv012-success-{suffix}");
        let success_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/template-runs", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", &success_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_template_id": seed.template_v1_id,
                            "requester_user_id": seed.buyer_user_id,
                            "request_payload_json": {
                                "start_date": "2026-01-01",
                                "limit": 2
                            },
                            "output_boundary_json": {
                                "selected_format": "json",
                                "allowed_formats": ["json"],
                                "max_rows": 2,
                                "max_cells": 6
                            },
                            "approval_ticket_id": seed.approval_ticket_id,
                            "execution_metadata_json": {
                                "entrypoint": "smoke"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("success request"),
            )
            .await
            .expect("success response");
        let success_status = success_response.status();
        let success_body = to_bytes(success_response.into_body(), usize::MAX)
            .await
            .expect("success body");
        assert_eq!(
            success_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&success_body)
        );
        let success_json: Value = serde_json::from_slice(&success_body).expect("success json");
        let data = &success_json["data"]["data"];
        let query_run_id = data["query_run_id"]
            .as_str()
            .expect("query_run_id")
            .to_string();
        let result_object_id = data["result_object_id"]
            .as_str()
            .expect("result_object_id")
            .to_string();
        let bucket_name = data["bucket_name"]
            .as_str()
            .expect("bucket_name")
            .to_string();
        let object_key = data["object_key"].as_str().expect("object_key").to_string();
        assert_eq!(data["status"].as_str(), Some("completed"));
        assert_eq!(data["current_state"].as_str(), Some("query_executed"));
        assert_eq!(data["result_row_count"].as_i64(), Some(2));
        assert_eq!(data["masked_level"].as_str(), Some("masked"));
        assert_eq!(data["export_scope"].as_str(), Some("none"));
        assert_eq!(data["query_template_version"].as_i64(), Some(1));
        assert_eq!(
            data["approval_ticket_id"].as_str(),
            Some(seed.approval_ticket_id.as_str())
        );

        let fetched_object = fetch_object_bytes(&bucket_name, &object_key)
            .await
            .expect("fetch minio object");
        let fetched_json: Value =
            serde_json::from_slice(&fetched_object.bytes).expect("fetched json");
        assert_eq!(
            fetched_json["query_run_id"].as_str(),
            Some(query_run_id.as_str())
        );
        assert_eq!(fetched_json["row_count"].as_i64(), Some(2));
        assert_eq!(fetched_json["selected_format"].as_str(), Some("json"));

        let missing_approval_request_id = format!("req-dlv012-missing-approval-{suffix}");
        let missing_approval_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/template-runs", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", &missing_approval_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_template_id": seed.template_v1_id,
                            "request_payload_json": {
                                "start_date": "2026-01-01",
                                "limit": 1
                            },
                            "output_boundary_json": {
                                "selected_format": "json",
                                "allowed_formats": ["json"],
                                "max_rows": 1,
                                "max_cells": 3
                            }
                        })
                        .to_string(),
                    ))
                    .expect("missing approval request"),
            )
            .await
            .expect("missing approval response");
        let missing_approval_status = missing_approval_response.status();
        let missing_approval_body = to_bytes(missing_approval_response.into_body(), usize::MAX)
            .await
            .expect("missing approval body");
        assert_eq!(
            missing_approval_status,
            StatusCode::CONFLICT,
            "{}",
            String::from_utf8_lossy(&missing_approval_body)
        );
        let missing_approval_json: Value =
            serde_json::from_slice(&missing_approval_body).expect("missing approval json");
        let missing_approval_msg = missing_approval_json["message"]
            .as_str()
            .or_else(|| missing_approval_json["error"]["message"].as_str())
            .unwrap_or_default()
            .to_string();
        assert!(missing_approval_msg.contains("approval_ticket_id is required"));

        let invalid_format_request_id = format!("req-dlv012-invalid-format-{suffix}");
        let invalid_format_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/template-runs", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", &invalid_format_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_template_id": seed.template_v2_id,
                            "request_payload_json": {
                                "start_date": "2026-01-01",
                                "limit": 1,
                                "country": "CN"
                            },
                            "output_boundary_json": {
                                "selected_format": "csv",
                                "allowed_formats": ["csv"],
                                "max_rows": 1,
                                "max_cells": 3
                            },
                            "approval_ticket_id": seed.approval_ticket_id
                        })
                        .to_string(),
                    ))
                    .expect("invalid format request"),
            )
            .await
            .expect("invalid format response");
        let invalid_format_status = invalid_format_response.status();
        let invalid_format_body = to_bytes(invalid_format_response.into_body(), usize::MAX)
            .await
            .expect("invalid format body");
        assert_eq!(
            invalid_format_status,
            StatusCode::CONFLICT,
            "{}",
            String::from_utf8_lossy(&invalid_format_body)
        );
        let invalid_format_json: Value =
            serde_json::from_slice(&invalid_format_body).expect("invalid format json");
        let invalid_format_msg = invalid_format_json["message"]
            .as_str()
            .or_else(|| invalid_format_json["error"]["message"].as_str())
            .unwrap_or_default()
            .to_string();
        assert!(invalid_format_msg.contains("output formats exceed template grant boundary"));

        let run_row = client
            .query_one(
                "SELECT query_template_id::text,
                        requester_user_id::text,
                        execution_mode,
                        request_payload_json,
                        result_summary_json,
                        result_object_id::text,
                        result_row_count,
                        billed_units::text,
                        export_attempt_count,
                        status,
                        masked_level,
                        export_scope,
                        approval_ticket_id::text,
                        sensitive_policy_snapshot,
                        to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM delivery.query_execution_run
                 WHERE query_run_id = $1::text::uuid",
                &[&query_run_id],
            )
            .await
            .expect("query run row");
        assert_eq!(run_row.get::<_, String>(0), seed.template_v1_id);
        assert_eq!(
            run_row.get::<_, Option<String>>(1),
            Some(seed.buyer_user_id.clone())
        );
        assert_eq!(run_row.get::<_, String>(2), "template_query");
        assert_eq!(
            run_row.get::<_, Value>(3)["parameter_summary"]["parameter_count"].as_i64(),
            Some(2)
        );
        assert_eq!(
            run_row.get::<_, Value>(4)["result_hash"].as_str().is_some(),
            true
        );
        assert_eq!(run_row.get::<_, Option<String>>(5), Some(result_object_id));
        assert_eq!(run_row.get::<_, i64>(6), 2);
        assert_eq!(run_row.get::<_, String>(7), "1.00000000");
        assert_eq!(run_row.get::<_, i32>(8), 0);
        assert_eq!(run_row.get::<_, String>(9), "completed");
        assert_eq!(run_row.get::<_, String>(10), "masked");
        assert_eq!(run_row.get::<_, String>(11), "none");
        assert_eq!(
            run_row.get::<_, Option<String>>(12),
            Some(seed.approval_ticket_id.clone())
        );
        assert_eq!(
            run_row.get::<_, Value>(13)["risk_guard_json"]["approval_required"].as_bool(),
            Some(true)
        );
        assert!(run_row.get::<_, Option<String>>(14).is_some());

        let order_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, acceptance_status, settlement_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order row");
        assert_eq!(order_row.get::<_, String>(0), "query_executed");
        assert_eq!(order_row.get::<_, String>(1), "paid");
        assert_eq!(order_row.get::<_, String>(2), "delivered");
        assert_eq!(order_row.get::<_, String>(3), "accepted");
        assert_eq!(order_row.get::<_, String>(4), "pending_settlement");

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE ref_id = $1::text::uuid
                   AND action_name = 'delivery.template_query.use'",
                &[&query_run_id],
            )
            .await
            .expect("query audit count")
            .get(0);
        assert_eq!(audit_count, 1);
        let billing_bridge_row = client
            .query_one(
                "SELECT target_topic,
                        payload ->> 'delivery_branch',
                        payload ->> 'trigger_stage',
                        payload -> 'billing_trigger_matrix' ->> 'billing_trigger'
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'billing.trigger.bridge'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&success_request_id],
            )
            .await
            .expect("query billing bridge row");
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(0).as_deref(),
            Some("billing.events")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(1).as_deref(),
            Some("query_run")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(2).as_deref(),
            Some("execution_completed")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(3).as_deref(),
            Some("bill_once_after_task_acceptance")
        );

        let _ = delete_object(&bucket_name, &object_key).await;
        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv012-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv012-seller-{suffix}")],
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
                    &format!("dlv012-buyer-user-{suffix}"),
                    &format!("DLV012 Buyer {suffix}"),
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
                    &format!("dlv012-asset-{suffix}"),
                    &format!("dlv012 asset {suffix}"),
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
                    &format!("dlv012-structured-{suffix}"),
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
                    &format!("dlv012-product-{suffix}"),
                    &format!("dlv012 product {suffix}"),
                    &format!("dlv012 search {suffix}"),
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
                &[&product_id, &format!("DLV012-SKU-{suffix}")],
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
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb
                 )",
                &[&order_id, &format!("sha256:dlv012:{suffix}")],
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
                    &format!("dlv012-connector-{suffix}"),
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
                    &format!("dlv012-env-{suffix}"),
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

        client
            .execute(
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
                 )"#,
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
            .await?;

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
                    &format!("minio://delivery-objects/templates/{suffix}/{template_name}_v{version_no}.sql"),
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
