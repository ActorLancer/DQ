#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::storage::application::{delete_object, fetch_object_bytes};
    use axum::body::{Body, to_bytes};
    use axum::http::{Method, Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use redis::AsyncCommands;
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct FileSeed {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        storage_namespace_id: String,
        delivery_id: String,
    }

    #[derive(Debug)]
    struct ApiSeed {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        asset_object_id: String,
    }

    #[derive(Debug)]
    struct QuerySeed {
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
        template_id: String,
    }

    #[derive(Debug)]
    struct SandboxSeed {
        buyer_org_id: String,
        seller_org_id: String,
        buyer_user_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        asset_object_id: String,
        connector_id: String,
        environment_id: String,
        query_surface_id: String,
    }

    #[derive(Debug)]
    struct ReportSeed {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        storage_namespace_id: String,
        delivery_id: String,
        contract_id: String,
    }

    #[tokio::test]
    async fn dlv025_delivery_storage_query_integration_db_smoke() {
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

        let file_seed = seed_file_graph(&client, &suffix)
            .await
            .expect("seed file graph");
        let api_seed = seed_api_graph(&client, &suffix)
            .await
            .expect("seed api graph");
        let query_seed = seed_query_graph(&client, &suffix)
            .await
            .expect("seed query graph");
        let sandbox_seed = seed_sandbox_graph(&client, &suffix)
            .await
            .expect("seed sandbox graph");
        let report_seed = seed_report_graph(&client, &suffix)
            .await
            .expect("seed report graph");

        let app = crate::with_live_test_state(delivery_router()).await;

        let file_commit_req = format!("req-dlv025-file-commit-{suffix}");
        let file_commit_response = app
            .clone()
            .oneshot(build_request(
                Method::POST,
                format!("/api/v1/orders/{}/deliver", file_seed.order_id),
                "seller_operator",
                &file_seed.seller_org_id,
                &file_commit_req,
                None,
                Some(json!({
                    "branch": "file",
                    "object_uri": format!("s3://delivery-objects/orders/{suffix}/payload.enc"),
                    "content_type": "application/octet-stream",
                    "size_bytes": 2048,
                    "content_hash": format!("sha256:dlv025:file:{suffix}"),
                    "encryption_algo": "AES-GCM",
                    "key_cipher": format!("cipher-{suffix}"),
                    "key_control_mode": "seller_managed",
                    "unwrap_policy_json": {
                        "kms": "local-mock",
                        "buyer_org_id": file_seed.buyer_org_id
                    },
                    "key_version": "v1",
                    "expire_at": "2027-01-01T00:00:00Z",
                    "download_limit": 3,
                    "delivery_commit_hash": format!("dlv025-file-commit-{suffix}"),
                    "receipt_hash": format!("dlv025-file-receipt-{suffix}")
                })),
            ))
            .await
            .expect("file deliver response");
        let (file_commit_status, _, file_commit_json) = decode_json(file_commit_response).await;
        assert_eq!(file_commit_status, StatusCode::OK);
        let file_ticket_id = file_commit_json["data"]["data"]["ticket_id"]
            .as_str()
            .expect("file ticket id")
            .to_string();
        assert_eq!(
            file_commit_json["data"]["data"]["current_state"].as_str(),
            Some("delivered")
        );

        let file_ticket_req = format!("req-dlv025-file-ticket-{suffix}");
        let file_ticket_response = app
            .clone()
            .oneshot(build_request(
                Method::GET,
                format!("/api/v1/orders/{}/download-ticket", file_seed.order_id),
                "buyer_operator",
                &file_seed.buyer_org_id,
                &file_ticket_req,
                None,
                None,
            ))
            .await
            .expect("file ticket response");
        let (file_ticket_status, _, file_ticket_json) = decode_json(file_ticket_response).await;
        assert_eq!(file_ticket_status, StatusCode::OK);
        assert_eq!(
            file_ticket_json["data"]["data"]["ticket_id"].as_str(),
            Some(file_ticket_id.as_str())
        );
        let redis_key = format!("datab:v1:download-ticket:{}", file_ticket_id);
        let mut redis_conn = redis_connection().await;
        let cached: String = redis_conn.get(&redis_key).await.expect("redis get");
        let cached_json: Value = serde_json::from_str(&cached).expect("cached json");
        assert_eq!(
            cached_json["ticket_id"].as_str(),
            Some(file_ticket_id.as_str())
        );

        let file_accept_req = format!("req-dlv025-file-accept-{suffix}");
        let file_accept_response = app
            .clone()
            .oneshot(build_request(
                Method::POST,
                format!("/api/v1/orders/{}/accept", file_seed.order_id),
                "buyer_operator",
                &file_seed.buyer_org_id,
                &file_accept_req,
                None,
                Some(json!({
                    "note": "hash verified",
                    "verification_summary": {"hash_match": true, "ticket_checked": true}
                })),
            ))
            .await
            .expect("file accept response");
        let (file_accept_status, _, file_accept_json) = decode_json(file_accept_response).await;
        assert_eq!(file_accept_status, StatusCode::OK);
        assert_eq!(
            file_accept_json["data"]["data"]["current_state"].as_str(),
            Some("accepted")
        );

        let api_req = format!("req-dlv025-api-{suffix}");
        let api_response = app
            .clone()
            .oneshot(build_request(
                Method::POST,
                format!("/api/v1/orders/{}/deliver", api_seed.order_id),
                "tenant_developer",
                &api_seed.buyer_org_id,
                &api_req,
                None,
                Some(json!({
                    "branch": "api",
                    "asset_object_id": api_seed.asset_object_id,
                    "app_name": format!("dlv025-api-app-{suffix}"),
                    "quota_json": {"billing_mode": "subscription", "period": "monthly", "included_calls": 1000},
                    "rate_limit_json": {"requests_per_minute": 60, "burst": 10, "concurrency": 3},
                    "upstream_mode": "platform_proxy",
                    "expire_at": "2027-01-01T00:00:00Z",
                    "delivery_commit_hash": format!("dlv025-api-commit-{suffix}"),
                    "receipt_hash": format!("dlv025-api-receipt-{suffix}")
                })),
            ))
            .await
            .expect("api deliver response");
        let (api_status, _, api_json) = decode_json(api_response).await;
        assert_eq!(api_status, StatusCode::OK);
        assert_eq!(api_json["data"]["data"]["branch"].as_str(), Some("api"));
        assert!(
            api_json["data"]["data"]["api_credential_id"]
                .as_str()
                .is_some()
        );

        let grant_req = format!("req-dlv025-template-grant-{suffix}");
        let grant_response = app
            .clone()
            .oneshot(build_request(
                Method::POST,
                format!("/api/v1/orders/{}/template-grants", query_seed.order_id),
                "seller_operator",
                &query_seed.seller_org_id,
                &grant_req,
                None,
                Some(json!({
                    "query_surface_id": query_seed.query_surface_id,
                    "allowed_template_ids": [query_seed.template_id],
                    "execution_rule_snapshot": {
                        "entrypoint": "template_query_lite",
                        "grant_source": "dlv025-smoke"
                    },
                    "output_boundary_json": {
                        "allowed_formats": ["json"],
                        "max_rows": 5,
                        "max_cells": 15
                    },
                    "run_quota_json": {
                        "max_runs": 5,
                        "daily_limit": 2,
                        "monthly_limit": 8
                    }
                })),
            ))
            .await
            .expect("grant response");
        let (grant_status, _, grant_json) = decode_json(grant_response).await;
        assert_eq!(grant_status, StatusCode::OK);
        let template_query_grant_id = grant_json["data"]["data"]["template_query_grant_id"]
            .as_str()
            .expect("template_query_grant_id")
            .to_string();
        assert_eq!(
            grant_json["data"]["data"]["current_state"].as_str(),
            Some("template_authorized")
        );

        let run_req = format!("req-dlv025-template-run-{suffix}");
        let run_response = app
            .clone()
            .oneshot(build_request(
                Method::POST,
                format!("/api/v1/orders/{}/template-runs", query_seed.order_id),
                "buyer_operator",
                &query_seed.buyer_org_id,
                &run_req,
                Some(&query_seed.buyer_user_id),
                Some(json!({
                    "template_query_grant_id": template_query_grant_id,
                    "query_template_id": query_seed.template_id,
                    "requester_user_id": query_seed.buyer_user_id,
                    "request_payload_json": {"start_date": "2026-01-01", "limit": 2},
                    "output_boundary_json": {"selected_format": "json", "allowed_formats": ["json"], "max_rows": 2, "max_cells": 6},
                    "approval_ticket_id": query_seed.approval_ticket_id,
                    "execution_metadata_json": {"entrypoint": "dlv025-smoke"}
                })),
            ))
            .await
            .expect("run response");
        let (run_status, _, run_json) = decode_json(run_response).await;
        assert_eq!(run_status, StatusCode::OK);
        let result_bucket = run_json["data"]["data"]["bucket_name"]
            .as_str()
            .expect("result bucket")
            .to_string();
        let result_key = run_json["data"]["data"]["object_key"]
            .as_str()
            .expect("result key")
            .to_string();
        let result_bytes = fetch_object_bytes(&result_bucket, &result_key)
            .await
            .expect("fetch result object");
        let result_json: Value = serde_json::from_slice(&result_bytes.bytes).expect("result json");
        assert_eq!(result_json["row_count"].as_i64(), Some(2));

        let sandbox_req = format!("req-dlv025-sandbox-{suffix}");
        let sandbox_response = app
            .clone()
            .oneshot(build_request(
                Method::POST,
                format!("/api/v1/orders/{}/sandbox-workspaces", sandbox_seed.order_id),
                "seller_operator",
                &sandbox_seed.seller_org_id,
                &sandbox_req,
                None,
                Some(json!({
                    "query_surface_id": sandbox_seed.query_surface_id,
                    "workspace_name": format!("dlv025-workspace-{suffix}"),
                    "seat_user_id": sandbox_seed.buyer_user_id,
                    "expire_at": "2027-01-01T00:00:00Z",
                    "export_policy_json": {"allow_export": false, "allowed_formats": ["json"], "max_exports": 0, "network_access": "deny"},
                    "clean_room_mode": "lite",
                    "data_residency_mode": "seller_self_hosted"
                })),
            ))
            .await
            .expect("sandbox response");
        let (sandbox_status, _, sandbox_json) = decode_json(sandbox_response).await;
        assert_eq!(sandbox_status, StatusCode::OK);
        assert_eq!(
            sandbox_json["data"]["data"]["workspace_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            sandbox_json["data"]["data"]["environment_type"].as_str(),
            Some("sandbox")
        );

        let report_commit_req = format!("req-dlv025-report-commit-{suffix}");
        let report_commit_response = app
            .clone()
            .oneshot(build_request(
                Method::POST,
                format!("/api/v1/orders/{}/deliver", report_seed.order_id),
                "seller_operator",
                &report_seed.seller_org_id,
                &report_commit_req,
                None,
                Some(json!({
                    "branch": "report",
                    "object_uri": format!("s3://report-results/orders/{suffix}/monthly-report.pdf"),
                    "content_type": "application/pdf",
                    "size_bytes": 4096,
                    "content_hash": format!("sha256:dlv025:report:{suffix}"),
                    "report_type": "pdf_report",
                    "storage_namespace_id": report_seed.storage_namespace_id,
                    "delivery_commit_hash": format!("dlv025-report-commit-{suffix}"),
                    "receipt_hash": format!("dlv025-report-receipt-{suffix}"),
                    "metadata": {"title": format!("DLV025 report {suffix}"), "template_code": "REPORT_V1"}
                })),
            ))
            .await
            .expect("report deliver response");
        let (report_commit_status, _, report_commit_json) =
            decode_json(report_commit_response).await;
        assert_eq!(report_commit_status, StatusCode::OK);
        assert_eq!(
            report_commit_json["data"]["data"]["current_state"].as_str(),
            Some("report_delivered")
        );

        let report_reject_req = format!("req-dlv025-report-reject-{suffix}");
        let report_reject_response = app
            .clone()
            .oneshot(build_request(
                Method::POST,
                format!("/api/v1/orders/{}/reject", report_seed.order_id),
                "buyer_operator",
                &report_seed.buyer_org_id,
                &report_reject_req,
                None,
                Some(json!({
                    "reason_code": "report_quality_failed",
                    "reason_detail": "section mismatch",
                    "verification_summary": {"hash_match": true, "report_section_check": false}
                })),
            ))
            .await
            .expect("report reject response");
        let (report_reject_status, _, report_reject_json) =
            decode_json(report_reject_response).await;
        assert_eq!(report_reject_status, StatusCode::OK);
        assert_eq!(
            report_reject_json["data"]["data"]["current_state"].as_str(),
            Some("rejected")
        );

        let file_order_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, acceptance_status
                 FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&file_seed.order_id],
            )
            .await
            .expect("file order row");
        assert_eq!(file_order_row.get::<_, String>(0), "accepted");
        assert_eq!(file_order_row.get::<_, String>(1), "paid");
        assert_eq!(file_order_row.get::<_, String>(2), "delivered");
        assert_eq!(file_order_row.get::<_, String>(3), "accepted");

        let api_order_row = client
            .query_one(
                "SELECT status, delivery_status FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&api_seed.order_id],
            )
            .await
            .expect("api order row");
        assert_eq!(api_order_row.get::<_, String>(0), "api_key_issued");
        assert_eq!(api_order_row.get::<_, String>(1), "in_progress");

        let query_order_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, acceptance_status
                 FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&query_seed.order_id],
            )
            .await
            .expect("query order row");
        assert_eq!(query_order_row.get::<_, String>(0), "query_executed");
        assert_eq!(query_order_row.get::<_, String>(1), "paid");
        assert_eq!(query_order_row.get::<_, String>(2), "delivered");
        assert_eq!(query_order_row.get::<_, String>(3), "accepted");

        let sandbox_order_row = client
            .query_one(
                "SELECT status, delivery_status FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&sandbox_seed.order_id],
            )
            .await
            .expect("sandbox order row");
        assert_eq!(sandbox_order_row.get::<_, String>(0), "seat_issued");
        assert_eq!(sandbox_order_row.get::<_, String>(1), "in_progress");

        let report_order_row = client
            .query_one(
                "SELECT status, delivery_status, acceptance_status, dispute_status
                 FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&report_seed.order_id],
            )
            .await
            .expect("report order row");
        assert_eq!(report_order_row.get::<_, String>(0), "rejected");
        assert_eq!(report_order_row.get::<_, String>(1), "delivered");
        assert_eq!(report_order_row.get::<_, String>(2), "rejected");
        assert_eq!(report_order_row.get::<_, String>(3), "open");

        assert_eq!(
            audit_count(&client, &file_commit_req, "delivery.file.commit").await,
            1
        );
        assert_eq!(
            audit_count(&client, &file_ticket_req, "delivery.file.download").await,
            1
        );
        assert_eq!(
            audit_count(&client, &file_accept_req, "delivery.accept").await,
            1
        );
        assert_eq!(
            audit_count(&client, &api_req, "delivery.api.enable").await,
            1
        );
        assert_eq!(
            audit_count(&client, &grant_req, "delivery.template_query.enable").await,
            1
        );
        assert_eq!(
            audit_count(&client, &run_req, "delivery.template_query.use").await,
            1
        );
        assert_eq!(
            audit_count(&client, &sandbox_req, "delivery.sandbox.enable").await,
            1
        );
        assert_eq!(
            audit_count(&client, &report_commit_req, "delivery.report.commit").await,
            1
        );
        assert_eq!(
            audit_count(&client, &report_reject_req, "delivery.reject").await,
            1
        );

        assert_eq!(outbox_count(&client, &file_commit_req).await, 1);
        assert_eq!(outbox_count(&client, &api_req).await, 1);
        assert_eq!(outbox_count(&client, &grant_req).await, 1);
        assert_eq!(outbox_count(&client, &sandbox_req).await, 1);
        assert_eq!(outbox_count(&client, &report_commit_req).await, 1);

        let _: () = redis_conn.del(&redis_key).await.expect("redis cleanup");
        let _ = delete_object(&result_bucket, &result_key).await;

        cleanup_report_graph(&client, &report_seed).await;
        cleanup_sandbox_graph(&client, &sandbox_seed).await;
        cleanup_query_graph(&client, &query_seed).await;
        cleanup_api_graph(&client, &api_seed).await;
        cleanup_file_graph(&client, &file_seed).await;
    }

    fn build_request(
        method: Method,
        uri: String,
        role: &str,
        tenant_id: &str,
        request_id: &str,
        user_id: Option<&str>,
        body: Option<Value>,
    ) -> Request<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("x-role", role)
            .header("x-tenant-id", tenant_id)
            .header("x-request-id", request_id);
        if let Some(user_id) = user_id {
            builder = builder.header("x-user-id", user_id);
        }
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        builder
            .body(match body {
                Some(body) => Body::from(body.to_string()),
                None => Body::empty(),
            })
            .expect("request build")
    }

    async fn decode_json(response: axum::response::Response) -> (StatusCode, String, Value) {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        let body_text = String::from_utf8_lossy(&body).to_string();
        let json: Value = serde_json::from_slice(&body).expect("json body");
        (status, body_text, json)
    }

    async fn redis_connection() -> redis::aio::MultiplexedConnection {
        let client = redis::Client::open("redis://:datab_redis_pass@127.0.0.1:6379/3")
            .expect("redis client");
        client
            .get_multiplexed_async_connection()
            .await
            .expect("redis connection")
    }

    async fn audit_count(client: &Client, request_id: &str, action_name: &str) -> i64 {
        client
            .query_one(
                "SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = $2",
                &[&request_id, &action_name],
            )
            .await
            .expect("audit count")
            .get(0)
    }

    async fn outbox_count(client: &Client, request_id: &str) -> i64 {
        client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'delivery.committed'
                   AND target_topic = 'dtp.outbox.domain-events'",
                &[&request_id],
            )
            .await
            .expect("outbox count")
            .get(0)
    }

    async fn seed_file_graph(client: &Client, suffix: &str) -> Result<FileSeed, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv025-file-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv025-file-seller-{suffix}")],
            )
            .await?
            .get(0);
        let asset_id: String = client
            .query_one(
                r#"INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 ) RETURNING asset_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv025-file-asset-{suffix}"),
                    &format!("dlv025 file asset {suffix}"),
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
                   2048, 'CN', ARRAY['CN']::text[], false,
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $2),
                   'active'
                 ) RETURNING asset_version_id::text"#,
                &[&asset_id, &format!("wmk-{suffix}")],
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
                   $5, 'listed', 'one_time', 88.00, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 ) RETURNING product_id::text"#,
                &[&asset_id, &asset_version_id, &seller_org_id, &format!("dlv025-file-product-{suffix}"), &format!("dlv025 file product {suffix}"), &format!("dlv025 file search {suffix}")],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status)
                 VALUES ($1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active')
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV025-FILE-SKU-{suffix}")],
            )
            .await?
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code,
                   delivery_status, acceptance_status, settlement_status, dispute_status,
                   price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'seller_delivering', 'paid', 'online', 88.00, 'CNY',
                   'in_progress', 'not_started', 'pending_settlement', 'none',
                   jsonb_build_object('product_id', $1::text, 'sku_id', $5::text, 'sku_code', $6::text, 'sku_type', 'FILE_STD', 'pricing_mode', 'one_time', 'unit_price', '88.00', 'currency_code', 'CNY'),
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $7),
                   'signed_url'
                 ) RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id, &format!("DLV025-FILE-SKU-{suffix}"), &format!("wmk-{suffix}")],
            )
            .await?
            .get(0);
        let storage_namespace_id: String = client
            .query_one(
                "INSERT INTO catalog.storage_namespace (
                   owner_org_id, namespace_name, provider_type, namespace_kind, bucket_name, prefix_rule, status
                 ) VALUES (
                   $1::text::uuid, $2, 's3_compatible', 'product', 'delivery-objects', 'orders/{order_id}', 'active'
                 ) RETURNING storage_namespace_id::text",
                &[&seller_org_id, &format!("dlv025-file-ns-{suffix}")],
            )
            .await?
            .get(0);
        let delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, trust_boundary_snapshot, sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, 'file_download', 'signed_url', 'prepared',
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $2),
                   'standard', 'not_required'
                 ) RETURNING delivery_id::text",
                &[&order_id, &format!("wmk-{suffix}")],
            )
            .await?
            .get(0);
        Ok(FileSeed {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            storage_namespace_id,
            delivery_id,
        })
    }

    async fn cleanup_file_graph(client: &Client, seed: &FileSeed) {
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.storage_namespace WHERE storage_namespace_id = $1::text::uuid",
                &[&seed.storage_namespace_id],
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

    async fn seed_api_graph(client: &Client, suffix: &str) -> Result<ApiSeed, db::Error> {
        let buyer_org_id: String = client.query_one("INSERT INTO core.organization (org_name, org_type, status, metadata) VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb) RETURNING org_id::text", &[&format!("dlv025-api-buyer-{suffix}")]).await?.get(0);
        let seller_org_id: String = client.query_one("INSERT INTO core.organization (org_name, org_type, status, metadata) VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb) RETURNING org_id::text", &[&format!("dlv025-api-seller-{suffix}")]).await?.get(0);
        let asset_id: String = client.query_one(r#"INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description) VALUES ($1::text::uuid, $2, 'manufacturing', 'low', 'active', $3) RETURNING asset_id::text"#, &[&seller_org_id, &format!("dlv025-api-asset-{suffix}"), &format!("dlv025 api asset {suffix}")]).await?.get(0);
        let asset_version_id: String = client.query_one(r#"INSERT INTO catalog.asset_version (asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash, data_size_bytes, origin_region, allowed_region, requires_controlled_execution, trust_boundary_snapshot, status) VALUES ($1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash', 1024, 'CN', ARRAY['CN']::text[], false, '{"delivery_mode":"api_access"}'::jsonb, 'active') RETURNING asset_version_id::text"#, &[&asset_id]).await?.get(0);
        let asset_object_id: String = client.query_one(r#"INSERT INTO catalog.asset_object_binding (asset_version_id, object_kind, object_name, object_locator, schema_json, output_schema_json, freshness_json, access_constraints, metadata) VALUES ($1::text::uuid, 'api_endpoint', $2, $3, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{"preview":false}'::jsonb, '{"zone":"api"}'::jsonb) RETURNING asset_object_id::text"#, &[&asset_version_id, &format!("dlv025-api-object-{suffix}"), &format!("https://api.example.com/{suffix}")]).await?.get(0);
        let product_id: String = client.query_one(r#"INSERT INTO catalog.product (asset_id, asset_version_id, seller_org_id, title, category, product_type, description, status, price_mode, price, currency_code, delivery_type, allowed_usage, searchable_text, metadata) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product', $5, 'listed', 'subscription', 88.00, 'CNY', 'api_access', ARRAY['internal_use']::text[], $6, '{"review_status":"approved"}'::jsonb) RETURNING product_id::text"#, &[&asset_id, &asset_version_id, &seller_org_id, &format!("dlv025-api-product-{suffix}"), &format!("dlv025 api product {suffix}"), &format!("dlv025 api search {suffix}")]).await?.get(0);
        let sku_id: String = client.query_one("INSERT INTO catalog.product_sku (product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode, delivery_object_kind, acceptance_mode, refund_mode, status) VALUES ($1::text::uuid, $2, 'API_SUB', '月', 'subscription', 'subscription_access', 'api_access', 'auto_accept', 'manual_refund', 'active') RETURNING sku_id::text", &[&product_id, &format!("DLV025-API-SUB-{suffix}")]).await?.get(0);
        let order_id: String = client.query_one(r#"INSERT INTO trade.order_main (product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id, status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status, payment_mode, amount, currency_code, price_snapshot_json, delivery_route_snapshot, trust_boundary_snapshot) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid, 'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none', 'online', 88.00, 'CNY', jsonb_build_object('sku_type', 'API_SUB', 'delivery_mode', 'api_access'), 'api_gateway', '{"api_delivery":"platform_proxy"}'::jsonb) RETURNING order_id::text"#, &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id]).await?.get(0);
        client.execute("INSERT INTO contract.digital_contract (order_id, contract_digest, status, signed_at, variables_json) VALUES ($1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb)", &[&order_id, &format!("sha256:dlv025:api:{suffix}")]).await?;
        Ok(ApiSeed {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            asset_object_id,
        })
    }

    async fn cleanup_api_graph(client: &Client, seed: &ApiSeed) {
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
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
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await;
    }

    async fn seed_query_graph(client: &Client, suffix: &str) -> Result<QuerySeed, db::Error> {
        let buyer_org_id: String = client.query_one("INSERT INTO core.organization (org_name, org_type, status, metadata) VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb) RETURNING org_id::text", &[&format!("dlv025-query-buyer-{suffix}")]).await?.get(0);
        let seller_org_id: String = client.query_one("INSERT INTO core.organization (org_name, org_type, status, metadata) VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb) RETURNING org_id::text", &[&format!("dlv025-query-seller-{suffix}")]).await?.get(0);
        let buyer_user_id: String = client.query_one("INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status) VALUES ($1::text::uuid, $2, $3, 'person', 'active', 'disabled') RETURNING user_id::text", &[&buyer_org_id, &format!("dlv025-query-user-{suffix}"), &format!("DLV025 Query User {suffix}")]).await?.get(0);
        let asset_id: String = client.query_one(r#"INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description) VALUES ($1::text::uuid, $2, 'manufacturing', 'low', 'active', $3) RETURNING asset_id::text"#, &[&seller_org_id, &format!("dlv025-query-asset-{suffix}"), &format!("dlv025 query asset {suffix}")]).await?.get(0);
        let asset_version_id: String = client.query_one(r#"INSERT INTO catalog.asset_version (asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash, data_size_bytes, origin_region, allowed_region, requires_controlled_execution, trust_boundary_snapshot, status) VALUES ($1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash', 4096, 'CN', ARRAY['CN']::text[], true, '{"query_mode":"controlled"}'::jsonb, 'active') RETURNING asset_version_id::text"#, &[&asset_id]).await?.get(0);
        let asset_object_id: String = client.query_one(r#"INSERT INTO catalog.asset_object_binding (asset_version_id, object_kind, object_name, object_locator, schema_json, output_schema_json, freshness_json, access_constraints, metadata) VALUES ($1::text::uuid, 'structured_dataset', $2, $3, '{"fields":[{"name":"city"},{"name":"amount"}]}'::jsonb, '{"fields":[{"name":"city"},{"name":"total_amount"}]}'::jsonb, '{}'::jsonb, '{"preview":false}'::jsonb, '{"zone":"curated"}'::jsonb) RETURNING asset_object_id::text"#, &[&asset_version_id, &format!("dlv025-query-object-{suffix}"), &format!("warehouse://curated/{suffix}/sales")]).await?.get(0);
        let product_id: String = client.query_one(r#"INSERT INTO catalog.product (asset_id, asset_version_id, seller_org_id, title, category, product_type, description, status, price_mode, price, currency_code, delivery_type, allowed_usage, searchable_text, metadata) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product', $5, 'listed', 'usage', 48.00, 'CNY', 'template_query', ARRAY['internal_use']::text[], $6, '{"review_status":"approved"}'::jsonb) RETURNING product_id::text"#, &[&asset_id, &asset_version_id, &seller_org_id, &format!("dlv025-query-product-{suffix}"), &format!("dlv025 query product {suffix}"), &format!("dlv025 query search {suffix}")]).await?.get(0);
        let sku_id: String = client.query_one("INSERT INTO catalog.product_sku (product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode, delivery_object_kind, acceptance_mode, refund_mode, status) VALUES ($1::text::uuid, $2, 'QRY_LITE', '次', 'usage', 'query_service', 'template_grant', 'manual_accept', 'manual_refund', 'active') RETURNING sku_id::text", &[&product_id, &format!("DLV025-QRY-SKU-{suffix}")]).await?.get(0);
        let order_id: String = client.query_one(r#"INSERT INTO trade.order_main (product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id, status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status, payment_mode, amount, currency_code, price_snapshot_json, delivery_route_snapshot, trust_boundary_snapshot) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid, 'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none', 'online', 48.00, 'CNY', '{"query_surface_type":"template_query_lite"}'::jsonb, 'template_query', '{"query_delivery":"controlled"}'::jsonb) RETURNING order_id::text"#, &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id]).await?.get(0);
        client.execute("INSERT INTO contract.digital_contract (order_id, contract_digest, status, signed_at, variables_json) VALUES ($1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb)", &[&order_id, &format!("sha256:dlv025:query:{suffix}")]).await?;
        let connector_id: String = client.query_one(r#"INSERT INTO core.connector (org_id, connector_name, connector_type, status, version, network_zone, health_status, endpoint_ref, metadata) VALUES ($1::text::uuid, $2, 'query_runtime', 'active', 'v1', 'private', 'healthy', $3, '{"provider":"local"}'::jsonb) RETURNING connector_id::text"#, &[&seller_org_id, &format!("dlv025-query-connector-{suffix}"), &format!("https://connector.example.com/{suffix}")]).await?.get(0);
        let environment_id: String = client.query_one(r#"INSERT INTO core.execution_environment (org_id, connector_id, environment_name, environment_type, status, network_zone, region_code, metadata) VALUES ($1::text::uuid, $2::text::uuid, $3, 'query_runtime', 'active', 'private', 'cn-east-1', '{"mode":"template_query"}'::jsonb) RETURNING environment_id::text"#, &[&seller_org_id, &connector_id, &format!("dlv025-query-env-{suffix}")]).await?.get(0);
        let query_surface_id: String = client.query_one(r#"INSERT INTO catalog.query_surface_definition (asset_version_id, asset_object_id, environment_id, surface_type, binding_mode, execution_scope, input_contract_json, output_boundary_json, query_policy_json, quota_policy_json, status, metadata) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, 'template_query_lite', 'managed_surface', 'curated_zone', '{"source_zones":["curated_zone"]}'::jsonb, '{"max_rows":10,"max_cells":30,"allow_raw_export":false,"allowed_formats":["json"]}'::jsonb, '{"analysis_rule":"whitelist_only","template_review_status":"approved"}'::jsonb, '{"daily_limit":10,"monthly_limit":100}'::jsonb, 'active', '{"owner":"seller"}'::jsonb) RETURNING query_surface_id::text"#, &[&asset_version_id, &asset_object_id, &environment_id]).await?.get(0);
        let template_id: String = client.query_one(r#"INSERT INTO delivery.query_template_definition (query_surface_id, template_name, template_type, template_body_ref, parameter_schema_json, analysis_rule_json, result_schema_json, export_policy_json, risk_guard_json, status, version_no) VALUES ($1::text::uuid, 'sales_overview', 'sql_template', $2, '{"type":"object","properties":{"start_date":{"type":"string"},"limit":{"type":"integer"}}}'::jsonb, '{"analysis_rule":"whitelist_only","template_review_status":"approved"}'::jsonb, '{"fields":[{"name":"city","type":"string"},{"name":"total_amount","type":"number"}]}'::jsonb, '{"allow_raw_export":false,"allowed_formats":["json"],"max_export_rows":100,"max_export_cells":1000}'::jsonb, '{"risk_mode":"strict"}'::jsonb, 'active', 1) RETURNING query_template_id::text"#, &[&query_surface_id, &format!("minio://delivery-objects/templates/{suffix}/sales_overview_v1.sql")]).await?.get(0);
        let approval_ticket_id: String = client.query_one("INSERT INTO ops.approval_ticket (ticket_type, ref_type, ref_id, requested_by, status, requires_second_review) VALUES ('query_run', 'order', $1::text::uuid, $2::text::uuid, 'approved', false) RETURNING approval_ticket_id::text", &[&order_id, &buyer_user_id]).await?.get(0);
        Ok(QuerySeed {
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
            template_id,
        })
    }

    async fn cleanup_query_graph(client: &Client, seed: &QuerySeed) {
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client.execute("DELETE FROM delivery.query_template_definition WHERE query_surface_id = $1::text::uuid", &[&seed.query_surface_id]).await;
        let _ = client.execute("DELETE FROM catalog.query_surface_definition WHERE query_surface_id = $1::text::uuid", &[&seed.query_surface_id]).await;
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
                "DELETE FROM ops.approval_ticket WHERE approval_ticket_id = $1::text::uuid",
                &[&seed.approval_ticket_id],
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
                "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
                &[&seed.buyer_user_id],
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

    async fn seed_sandbox_graph(client: &Client, suffix: &str) -> Result<SandboxSeed, db::Error> {
        let buyer_org_id: String = client.query_one("INSERT INTO core.organization (org_name, org_type, status, metadata) VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb) RETURNING org_id::text", &[&format!("dlv025-sbx-buyer-{suffix}")]).await?.get(0);
        let seller_org_id: String = client.query_one("INSERT INTO core.organization (org_name, org_type, status, metadata) VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb) RETURNING org_id::text", &[&format!("dlv025-sbx-seller-{suffix}")]).await?.get(0);
        let buyer_user_id: String = client.query_one("INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status) VALUES ($1::text::uuid, $2, $3, 'person', 'active', 'disabled') RETURNING user_id::text", &[&buyer_org_id, &format!("dlv025-sbx-user-{suffix}"), &format!("DLV025 Sandbox User {suffix}")]).await?.get(0);
        let asset_id: String = client.query_one(r#"INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description) VALUES ($1::text::uuid, $2, 'manufacturing', 'low', 'active', $3) RETURNING asset_id::text"#, &[&seller_org_id, &format!("dlv025-sbx-asset-{suffix}"), &format!("dlv025 sandbox asset {suffix}")]).await?.get(0);
        let asset_version_id: String = client.query_one(r#"INSERT INTO catalog.asset_version (asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash, data_size_bytes, origin_region, allowed_region, requires_controlled_execution, trust_boundary_snapshot, status) VALUES ($1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash', 4096, 'CN', ARRAY['CN']::text[], true, '{"sandbox_mode":"controlled"}'::jsonb, 'active') RETURNING asset_version_id::text"#, &[&asset_id]).await?.get(0);
        let asset_object_id: String = client.query_one(r#"INSERT INTO catalog.asset_object_binding (asset_version_id, object_kind, object_name, object_locator, schema_json, output_schema_json, freshness_json, access_constraints, metadata) VALUES ($1::text::uuid, 'structured_dataset', $2, $3, '{"fields":[{"name":"city"}]}'::jsonb, '{"fields":[{"name":"city"}]}'::jsonb, '{}'::jsonb, '{"preview":false}'::jsonb, '{"zone":"curated"}'::jsonb) RETURNING asset_object_id::text"#, &[&asset_version_id, &format!("dlv025-sbx-object-{suffix}"), &format!("warehouse://sandbox/{suffix}/dataset")]).await?.get(0);
        let product_id: String = client.query_one(r#"INSERT INTO catalog.product (asset_id, asset_version_id, seller_org_id, title, category, product_type, description, status, price_mode, price, currency_code, delivery_type, allowed_usage, searchable_text, metadata) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product', $5, 'listed', 'subscription', 88.00, 'CNY', 'sandbox_workspace', ARRAY['internal_use']::text[], $6, '{"review_status":"approved"}'::jsonb) RETURNING product_id::text"#, &[&asset_id, &asset_version_id, &seller_org_id, &format!("dlv025-sbx-product-{suffix}"), &format!("dlv025 sandbox product {suffix}"), &format!("dlv025 sandbox search {suffix}")]).await?.get(0);
        let sku_id: String = client.query_one("INSERT INTO catalog.product_sku (product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode, delivery_object_kind, acceptance_mode, refund_mode, status) VALUES ($1::text::uuid, $2, 'SBX_STD', '席位月', 'subscription', 'query_service', 'sandbox_workspace', 'manual_accept', 'manual_refund', 'active') RETURNING sku_id::text", &[&product_id, &format!("DLV025-SBX-SKU-{suffix}")]).await?.get(0);
        let connector_id: String = client.query_one(r#"INSERT INTO core.connector (org_id, connector_name, connector_type, status, version, network_zone, health_status, endpoint_ref, metadata) VALUES ($1::text::uuid, $2, 'sandbox_bridge', 'active', 'v1', 'seller_vpc', 'healthy', $3, '{"driver":"container"}'::jsonb) RETURNING connector_id::text"#, &[&seller_org_id, &format!("dlv025-sbx-connector-{suffix}"), &format!("sandbox://{suffix}")]).await?.get(0);
        let environment_id: String = client.query_one(r#"INSERT INTO core.execution_environment (org_id, connector_id, environment_name, environment_type, status, network_zone, region_code, metadata) VALUES ($1::text::uuid, $2::text::uuid, $3, 'sandbox', 'active', 'seller_vpc', 'CN', '{"egress":"deny","clipboard":"masked_only"}'::jsonb) RETURNING environment_id::text"#, &[&seller_org_id, &connector_id, &format!("dlv025-sbx-env-{suffix}")]).await?.get(0);
        let query_surface_id: String = client.query_one(r#"INSERT INTO catalog.query_surface_definition (asset_version_id, asset_object_id, environment_id, surface_type, binding_mode, execution_scope, input_contract_json, output_boundary_json, query_policy_json, quota_policy_json, status, metadata) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, 'sandbox_query', 'managed_surface', 'curated_zone', '{"source_zones":["curated"],"seat_required":true}'::jsonb, '{"allow_export":false,"allowed_formats":["json"],"max_exports":0}'::jsonb, '{"network_boundary":"deny","clipboard":"masked_only"}'::jsonb, '{"seat_limit":1}'::jsonb, 'active', '{"owner":"seller"}'::jsonb) RETURNING query_surface_id::text"#, &[&asset_version_id, &asset_object_id, &environment_id]).await?.get(0);
        let order_id: String = client.query_one("INSERT INTO trade.order_main (product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id, status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status, payment_mode, amount, currency_code, price_snapshot_json) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid, 'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none', 'online', 88.00, 'CNY', '{}'::jsonb) RETURNING order_id::text", &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id]).await?.get(0);
        client.execute("INSERT INTO contract.digital_contract (order_id, contract_digest, status, signed_at, variables_json) VALUES ($1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb)", &[&order_id, &format!("sha256:dlv025:sbx:{suffix}")]).await?;
        Ok(SandboxSeed {
            buyer_org_id,
            seller_org_id,
            buyer_user_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            asset_object_id,
            connector_id,
            environment_id,
            query_surface_id,
        })
    }

    async fn cleanup_sandbox_graph(client: &Client, seed: &SandboxSeed) {
        let _ = client.execute("DELETE FROM delivery.sandbox_session WHERE sandbox_workspace_id IN (SELECT sandbox_workspace_id FROM delivery.sandbox_workspace WHERE order_id = $1::text::uuid)", &[&seed.order_id]).await;
        let _ = client
            .execute(
                "DELETE FROM delivery.sandbox_workspace WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client.execute("DELETE FROM catalog.query_surface_definition WHERE query_surface_id = $1::text::uuid", &[&seed.query_surface_id]).await;
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
                "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
                &[&seed.buyer_user_id],
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

    async fn seed_report_graph(client: &Client, suffix: &str) -> Result<ReportSeed, db::Error> {
        let buyer_org_id: String = client.query_one("INSERT INTO core.organization (org_name, org_type, status, metadata) VALUES ($1, 'enterprise', 'active', '{}'::jsonb) RETURNING org_id::text", &[&format!("dlv025-report-buyer-{suffix}")]).await?.get(0);
        let seller_org_id: String = client.query_one("INSERT INTO core.organization (org_name, org_type, status, metadata) VALUES ($1, 'enterprise', 'active', '{}'::jsonb) RETURNING org_id::text", &[&format!("dlv025-report-seller-{suffix}")]).await?.get(0);
        let asset_id: String = client.query_one(r#"INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description) VALUES ($1::text::uuid, $2, 'analysis', 'low', 'active', $3) RETURNING asset_id::text"#, &[&seller_org_id, &format!("dlv025-report-asset-{suffix}"), &format!("dlv025 report asset {suffix}")]).await?.get(0);
        let asset_version_id: String = client.query_one(r#"INSERT INTO catalog.asset_version (asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash, data_size_bytes, origin_region, allowed_region, requires_controlled_execution, trust_boundary_snapshot, status) VALUES ($1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash', 1024, 'CN', ARRAY['CN']::text[], false, '{"delivery_mode":"report_delivery"}'::jsonb, 'active') RETURNING asset_version_id::text"#, &[&asset_id]).await?.get(0);
        let product_id: String = client.query_one(r#"INSERT INTO catalog.product (asset_id, asset_version_id, seller_org_id, title, category, product_type, description, status, price_mode, price, currency_code, delivery_type, allowed_usage, searchable_text, metadata) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'analysis', 'data_product', $5, 'listed', 'one_time', 88.00, 'CNY', 'report_delivery', ARRAY['internal_use']::text[], $6, '{"review_status":"approved"}'::jsonb) RETURNING product_id::text"#, &[&asset_id, &asset_version_id, &seller_org_id, &format!("dlv025-report-product-{suffix}"), &format!("dlv025 report product {suffix}"), &format!("dlv025 report search {suffix}")]).await?.get(0);
        let sku_id: String = client.query_one("INSERT INTO catalog.product_sku (product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status) VALUES ($1::text::uuid, $2, 'RPT_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active') RETURNING sku_id::text", &[&product_id, &format!("DLV025-RPT-SKU-{suffix}")]).await?.get(0);
        let order_id: String = client.query_one("INSERT INTO trade.order_main (product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id, status, payment_status, payment_mode, amount, currency_code, delivery_status, acceptance_status, settlement_status, dispute_status, price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot) VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid, 'report_generated', 'paid', 'online', 88.00, 'CNY', 'in_progress', 'not_started', 'pending_settlement', 'none', jsonb_build_object('sku_type', 'RPT_STD', 'delivery_mode', 'report_delivery'), jsonb_build_object('delivery_mode', 'report_delivery', 'watermark_rule', 'result_attested'), 'result_package') RETURNING order_id::text", &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id]).await?.get(0);
        let contract_id: String = client.query_one("INSERT INTO contract.digital_contract (order_id, contract_digest, status, signed_at, variables_json) VALUES ($1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb) RETURNING contract_id::text", &[&order_id, &format!("sha256:dlv025:report:{suffix}")]).await?.get(0);
        let storage_namespace_id: String = client.query_one("INSERT INTO catalog.storage_namespace (owner_org_id, namespace_name, provider_type, namespace_kind, bucket_name, prefix_rule, status) VALUES ($1::text::uuid, $2, 's3_compatible', 'product', 'report-results', 'orders/{order_id}', 'active') RETURNING storage_namespace_id::text", &[&seller_org_id, &format!("dlv025-report-ns-{suffix}")]).await?.get(0);
        let delivery_id: String = client.query_one("INSERT INTO delivery.delivery_record (order_id, delivery_type, delivery_route, status, trust_boundary_snapshot, sensitive_delivery_mode, disclosure_review_status) VALUES ($1::text::uuid, 'report_delivery', 'result_package', 'prepared', jsonb_build_object('delivery_mode', 'report_delivery', 'result_package', true), 'standard', 'not_required') RETURNING delivery_id::text", &[&order_id]).await?.get(0);
        Ok(ReportSeed {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            storage_namespace_id,
            delivery_id,
            contract_id,
        })
    }

    async fn cleanup_report_graph(client: &Client, seed: &ReportSeed) {
        let _ = client
            .execute(
                "DELETE FROM delivery.report_artifact WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.storage_object WHERE org_id = $1::text::uuid",
                &[&seed.seller_org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.storage_namespace WHERE storage_namespace_id = $1::text::uuid",
                &[&seed.storage_namespace_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.digital_contract WHERE contract_id = $1::text::uuid",
                &[&seed.contract_id],
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
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await;
    }
}
