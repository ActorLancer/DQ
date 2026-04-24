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
        reviewer_org_id: String,
        buyer_user_id: String,
        reviewer_user_id: String,
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
    }

    #[tokio::test]
    async fn dlv023_result_disclosure_review_db_smoke() {
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

        let run_request_id = format!("req-dlv023-run-{suffix}");
        let run_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/template-runs", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", &run_request_id)
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
                                "max_cells": 6,
                                "requires_disclosure_review": true
                            },
                            "approval_ticket_id": seed.approval_ticket_id,
                            "execution_metadata_json": {
                                "entrypoint": "dlv023-smoke"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("run request"),
            )
            .await
            .expect("run response");
        let run_status = run_response.status();
        let run_body = to_bytes(run_response.into_body(), usize::MAX)
            .await
            .expect("run body");
        assert_eq!(
            run_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&run_body)
        );
        let run_json: Value = serde_json::from_slice(&run_body).expect("run json");
        let query_run_id = run_json["data"]["query_run_id"]
            .as_str()
            .expect("query_run_id")
            .to_string();
        let result_object_id = run_json["data"]["result_object_id"]
            .as_str()
            .expect("result_object_id")
            .to_string();
        let bucket_name = run_json["data"]["bucket_name"]
            .as_str()
            .expect("bucket_name")
            .to_string();
        let object_key = run_json["data"]["object_key"]
            .as_str()
            .expect("object_key")
            .to_string();

        let create_request_id = format!("req-dlv023-create-{suffix}");
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/query-runs/{}/disclosure-review",
                        query_run_id
                    ))
                    .header("x-role", "platform_reviewer")
                    .header("x-user-id", &seed.reviewer_user_id)
                    .header("x-request-id", &create_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "review_status": "approved",
                            "approval_ticket_id": seed.approval_ticket_id,
                            "review_notes": "approved for masked disclosure",
                            "decision_snapshot": {
                                "review_reason": "manual_check"
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
        let create_data = &create_json["data"];
        let review_id = create_data["result_disclosure_review_id"]
            .as_str()
            .expect("review id")
            .to_string();
        assert_eq!(create_data["operation"].as_str(), Some("created"));
        assert_eq!(create_data["review_status"].as_str(), Some("approved"));
        assert_eq!(
            create_data["result_object_id"].as_str(),
            Some(result_object_id.as_str())
        );
        assert_eq!(
            create_data["requires_disclosure_review"].as_bool(),
            Some(true)
        );

        let update_request_id = format!("req-dlv023-update-{suffix}");
        let update_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/query-runs/{}/disclosure-review",
                        query_run_id
                    ))
                    .header("x-role", "platform_reviewer")
                    .header("x-user-id", &seed.reviewer_user_id)
                    .header("x-request-id", &update_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "result_disclosure_review_id": review_id,
                            "review_status": "rejected",
                            "review_notes": "rejected after replay",
                            "decision_snapshot": {
                                "review_reason": "replay_failed"
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
        let update_data = &update_json["data"];
        assert_eq!(update_data["operation"].as_str(), Some("updated"));
        assert_eq!(update_data["review_status"].as_str(), Some("rejected"));
        assert_eq!(
            update_data["approval_ticket_id"].as_str(),
            Some(seed.approval_ticket_id.as_str())
        );

        let review_row = client
            .query_one(
                "SELECT review_status,
                        masking_level,
                        export_scope,
                        reviewer_user_id::text,
                        approval_ticket_id::text,
                        decision_snapshot -> 'output_boundary_json' ->> 'requires_disclosure_review',
                        decision_snapshot -> 'client_snapshot' ->> 'review_reason'
                 FROM delivery.result_disclosure_review
                 WHERE result_disclosure_review_id = $1::text::uuid",
                &[&review_id],
            )
            .await
            .expect("review row");
        assert_eq!(review_row.get::<_, String>(0), "rejected");
        assert_eq!(review_row.get::<_, String>(1), "masked");
        assert_eq!(review_row.get::<_, String>(2), "none");
        assert_eq!(
            review_row.get::<_, Option<String>>(3).as_deref(),
            Some(seed.reviewer_user_id.as_str())
        );
        assert_eq!(
            review_row.get::<_, Option<String>>(4).as_deref(),
            Some(seed.approval_ticket_id.as_str())
        );
        assert_eq!(
            review_row.get::<_, Option<String>>(5).as_deref(),
            Some("true")
        );
        assert_eq!(
            review_row.get::<_, Option<String>>(6).as_deref(),
            Some("replay_failed")
        );

        let query_run_row = client
            .query_one(
                "SELECT result_summary_json ->> 'disclosure_review_status',
                        result_summary_json ->> 'result_disclosure_review_id'
                 FROM delivery.query_execution_run
                 WHERE query_run_id = $1::text::uuid",
                &[&query_run_id],
            )
            .await
            .expect("query run row");
        assert_eq!(
            query_run_row.get::<_, Option<String>>(0).as_deref(),
            Some("rejected")
        );
        assert_eq!(
            query_run_row.get::<_, Option<String>>(1).as_deref(),
            Some(review_id.as_str())
        );

        let delivery_row = client
            .query_one(
                "SELECT disclosure_review_status
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                   AND delivery_route = 'template_query'
                 ORDER BY updated_at DESC, delivery_id DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("delivery row");
        assert_eq!(delivery_row.get::<_, String>(0), "rejected");

        let create_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.result_disclosure.review'
                   AND request_id = $1",
                &[&create_request_id],
            )
            .await
            .expect("create audit count")
            .get(0);
        let update_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.result_disclosure.review'
                   AND request_id = $1",
                &[&update_request_id],
            )
            .await
            .expect("update audit count")
            .get(0);
        assert_eq!(create_audit_count, 1);
        assert_eq!(update_audit_count, 1);

        let _ = delete_object(&bucket_name, &object_key).await;
        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv023-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv023-seller-{suffix}")],
            )
            .await?
            .get(0);
        let reviewer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'platform', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv023-reviewer-{suffix}")],
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
                    &format!("dlv023-buyer-user-{suffix}"),
                    &format!("DLV023 Buyer {suffix}"),
                ],
            )
            .await?
            .get(0);
        let reviewer_user_id: String = client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status)
                 VALUES ($1::text::uuid, $2, $3, 'person', 'active', 'enabled')
                 RETURNING user_id::text",
                &[
                    &reviewer_org_id,
                    &format!("dlv023-reviewer-user-{suffix}"),
                    &format!("DLV023 Reviewer {suffix}"),
                ],
            )
            .await?
            .get(0);
        let asset_id: String = client
            .query_one(
                r#"INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'high', 'active', $3
                 )
                 RETURNING asset_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv023-asset-{suffix}"),
                    &format!("dlv023 asset {suffix}"),
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
                    &format!("dlv023-structured-{suffix}"),
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
                    &format!("dlv023-product-{suffix}"),
                    &format!("dlv023 product {suffix}"),
                    &format!("dlv023 search {suffix}"),
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
                &[&product_id, &format!("DLV023-SKU-{suffix}")],
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
                r#"INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, trust_boundary_snapshot, sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, 'template_query_access', 'template_query', 'active',
                   '{"query_delivery":"controlled"}'::jsonb,
                   'controlled', 'pending'
                 )"#,
                &[&order_id],
            )
            .await?;
        client
            .execute(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb
                 )",
                &[&order_id, &format!("sha256:dlv023:{suffix}")],
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
                    &format!("dlv023-connector-{suffix}"),
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
                    &format!("dlv023-env-{suffix}"),
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
        let template_v1_id = insert_template(client, &query_surface_id, suffix).await?;
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
                   '{"allow_raw_export":false,"allowed_formats":["json"],"max_rows":5,"max_cells":10,"requires_disclosure_review":true}'::jsonb,
                   '{"max_runs":5,"daily_limit":3,"monthly_limit":10}'::jsonb,
                   'active'
                 )"#,
                &[
                    &order_id,
                    &asset_object_id,
                    &environment_id,
                    &query_surface_id,
                    &format!("sha256:template-grant:{suffix}"),
                    &json!([template_v1_id.clone()]),
                    &json!({
                        "query_surface_id": query_surface_id,
                        "allowed_template_ids": [template_v1_id.clone()],
                        "template_summaries": [
                            {"query_template_id": template_v1_id.clone(), "template_name": "sales_overview", "version_no": 1}
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
            reviewer_org_id,
            buyer_user_id,
            reviewer_user_id,
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
        })
    }

    async fn insert_template(
        client: &Client,
        query_surface_id: &str,
        suffix: &str,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                r#"INSERT INTO delivery.query_template_definition (
                   query_surface_id, template_name, template_type, template_body_ref,
                   parameter_schema_json, analysis_rule_json, result_schema_json,
                   export_policy_json, risk_guard_json, status, version_no
                 ) VALUES (
                   $1::text::uuid, 'sales_overview', 'sql_template', $2,
                   '{"type":"object","properties":{"start_date":{"type":"string"},"limit":{"type":"integer","minimum":1}},"required":["start_date","limit"]}'::jsonb,
                   '{"analysis_rule":"whitelist_only","template_review_status":"approved","whitelist_fields":["city","total_amount"]}'::jsonb,
                   '{"fields":[{"name":"city","type":"string"},{"name":"total_amount","type":"number"}]}'::jsonb,
                   '{"allow_raw_export":false,"allowed_formats":["json"],"max_export_rows":5,"max_export_cells":10,"whitelist_fields":["city","total_amount"]}'::jsonb,
                   '{"risk_mode":"strict","approval_required":true}'::jsonb,
                   'active',
                   1
                 )
                 RETURNING query_template_id::text"#,
                &[
                    &query_surface_id,
                    &format!(
                        "minio://delivery-objects/templates/{suffix}/sales_overview_v1.sql"
                    ),
                ],
            )
            .await
            .map(|row| row.get(0))
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
                "DELETE FROM core.user_account WHERE user_id = $1::text::uuid OR user_id = $2::text::uuid",
                &[&seed.buyer_user_id, &seed.reviewer_user_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid OR org_id = $2::text::uuid OR org_id = $3::text::uuid",
                &[&seed.buyer_org_id, &seed.seller_org_id, &seed.reviewer_org_id],
            )
            .await;
    }
}
