use crate::modules::audit::api::router;
use crate::modules::audit::application::{EvidenceWriteCommand, record_evidence_snapshot};
use crate::modules::order::repo::write_trade_audit_event;
use crate::modules::storage::application::fetch_object_bytes;
use crate::modules::storage::application::put_object_bytes;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("AUD_DB_SMOKE").ok().as_deref() == Some("1")
}

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

#[cfg(test)]
mod route_tests {
    use super::*;

    #[tokio::test]
    async fn rejects_audit_trace_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/audit/traces")
            .header("x-request-id", "req-aud003-route-forbidden")
            .header("x-role", "buyer_operator")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn audit_trace_requires_request_id() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/audit/traces")
            .header("x-role", "platform_audit_security")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_package_export_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/packages/export")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud004-forbidden")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"ref_type":"order","ref_id":"10000000-0000-0000-0000-000000000001","reason":"forbidden"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn package_export_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/packages/export")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud004-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"ref_type":"order","ref_id":"10000000-0000-0000-0000-000000000001","reason":"missing step-up"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_replay_job_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/replay-jobs")
            .header("x-role", "buyer_operator")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud005-forbidden")
            .header("x-step-up-token", "aud005-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"replay_type":"state_replay","ref_type":"order","ref_id":"10000000-0000-0000-0000-000000000001","reason":"forbidden"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn replay_job_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/replay-jobs")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud005-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"replay_type":"state_replay","ref_type":"order","ref_id":"10000000-0000-0000-0000-000000000001","reason":"missing step-up"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn replay_job_enforces_dry_run_only() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/replay-jobs")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud005-dry-run")
            .header("x-step-up-token", "aud005-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"replay_type":"state_replay","ref_type":"order","ref_id":"10000000-0000-0000-0000-000000000001","reason":"dry-run only","dry_run":false}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn rejects_replay_lookup_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/audit/replay-jobs/10000000-0000-0000-0000-000000000005")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud005-read-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_legal_hold_create_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/legal-holds")
            .header("x-role", "buyer_operator")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud006-forbidden")
            .header("x-step-up-token", "aud006-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"hold_scope_type":"order","hold_scope_id":"10000000-0000-0000-0000-000000000001","reason_code":"regulator_hold"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn legal_hold_create_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/legal-holds")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud006-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"hold_scope_type":"order","hold_scope_id":"10000000-0000-0000-0000-000000000001","reason_code":"regulator_hold"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_legal_hold_release_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/legal-holds/10000000-0000-0000-0000-000000000006/release")
            .header("x-role", "buyer_operator")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud006-release-forbidden")
            .header("x-step-up-token", "aud006-stepup")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"reason":"release forbidden"}"#))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_anchor_batch_lookup_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/audit/anchor-batches")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud007-read-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn anchor_batch_retry_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/anchor-batches/10000000-0000-0000-0000-000000000007/retry")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud007-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"reason":"retry missing step-up"}"#))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn audit_trace_api_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let dsn = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string());
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
    );
    let seed = seed_order_graph(&client, &suffix)
        .await
        .expect("seed order graph");
    let app = crate::with_live_test_state(router()).await;
    let order_request_id = format!("req-aud003-order-{suffix}");
    let trace_request_id = format!("req-aud003-traces-{suffix}");
    let tenant_request_id = format!("req-aud003-tenant-{suffix}");
    let export_request_id = format!("req-aud004-export-{suffix}");
    let replay_request_id = format!("req-aud005-replay-{suffix}");
    let replay_lookup_request_id = format!("req-aud005-replay-get-{suffix}");
    let legal_hold_request_id = format!("req-aud006-hold-{suffix}");
    let legal_hold_conflict_request_id = format!("req-aud006-hold-conflict-{suffix}");
    let legal_hold_release_request_id = format!("req-aud006-hold-release-{suffix}");
    let anchor_list_request_id = format!("req-aud007-list-{suffix}");
    let anchor_retry_request_id = format!("req-aud007-retry-{suffix}");
    let trace_id = format!("trace-aud003-{suffix}");

    write_trade_audit_event(
        &client,
        "order",
        &seed.order_id,
        "buyer_operator",
        "trade.order.create",
        "accepted",
        Some(&order_request_id),
        Some(&trace_id),
    )
    .await
    .expect("write order create audit");
    write_trade_audit_event(
        &client,
        "order",
        &seed.order_id,
        "buyer_operator",
        "trade.order.lock",
        "accepted",
        Some(&order_request_id),
        Some(&trace_id),
    )
    .await
    .expect("write order lock audit");

    let order_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/audit/orders/{}", seed.order_id))
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &order_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("order request"),
        )
        .await
        .expect("call order audit");
    assert_eq!(order_resp.status(), StatusCode::OK);
    let order_body = to_bytes(order_resp.into_body(), usize::MAX)
        .await
        .expect("read order body");
    let order_json: Value = serde_json::from_slice(&order_body).expect("decode order body");
    assert_eq!(
        order_json["data"]["order_id"].as_str(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        order_json["data"]["buyer_org_id"].as_str(),
        Some(seed.buyer_org_id.as_str())
    );
    assert_eq!(
        order_json["data"]["seller_org_id"].as_str(),
        Some(seed.seller_org_id.as_str())
    );
    assert_eq!(order_json["data"]["total"].as_i64(), Some(2));
    assert_eq!(
        order_json["data"]["traces"][0]["trace_id"].as_str(),
        Some(trace_id.as_str())
    );

    let traces_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/audit/traces?order_id={}&trace_id={}",
                    seed.order_id, trace_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &trace_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("trace request"),
        )
        .await
        .expect("call trace audit");
    assert_eq!(traces_resp.status(), StatusCode::OK);
    let traces_body = to_bytes(traces_resp.into_body(), usize::MAX)
        .await
        .expect("read traces body");
    let traces_json: Value = serde_json::from_slice(&traces_body).expect("decode traces body");
    assert_eq!(traces_json["data"]["total"].as_i64(), Some(2));
    assert_eq!(
        traces_json["data"]["items"][0]["ref_type"].as_str(),
        Some("order")
    );

    let tenant_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/audit/traces?order_id={}", seed.order_id))
                .header("x-role", "tenant_audit_readonly")
                .header("x-tenant-id", &seed.buyer_org_id)
                .header("x-request-id", &tenant_request_id)
                .body(Body::empty())
                .expect("tenant trace request"),
        )
        .await
        .expect("call tenant trace audit");
    assert_eq!(tenant_resp.status(), StatusCode::OK);

    let foreign_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/audit/orders/{}", seed.order_id))
                .header("x-role", "tenant_audit_readonly")
                .header("x-tenant-id", "00000000-0000-0000-0000-000000000999")
                .header("x-request-id", format!("req-aud003-foreign-{suffix}"))
                .body(Body::empty())
                .expect("foreign order request"),
        )
        .await
        .expect("call foreign order audit");
    assert_eq!(foreign_resp.status(), StatusCode::FORBIDDEN);

    let access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])",
            &[&vec![
                order_request_id.clone(),
                trace_request_id.clone(),
                tenant_request_id.clone(),
            ]],
        )
        .await
        .expect("count access audit")
        .get(0);
    assert_eq!(access_count, 3);

    let log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text LIKE 'audit lookup executed:%'",
            &[&vec![
                order_request_id.clone(),
                trace_request_id.clone(),
                tenant_request_id.clone(),
            ]],
        )
        .await
        .expect("count system logs")
        .get(0);
    assert_eq!(log_count, 3);

    let audit_user_id = seed_user(&client, &seed.buyer_org_id, &suffix)
        .await
        .expect("seed audit export user");
    let challenge_id = seed_verified_step_up_challenge(
        &client,
        &audit_user_id,
        "audit.package.export",
        "order",
        Some(seed.order_id.as_str()),
        "aud004",
    )
    .await
    .expect("seed verified export step-up challenge");
    let source_bucket = std::env::var("BUCKET_EVIDENCE_PACKAGES")
        .unwrap_or_else(|_| "evidence-packages".to_string());
    let source_key = format!("audit-source/orders/{}/{}.json", seed.order_id, suffix);
    let source_uri = format!("s3://{source_bucket}/{source_key}");
    let source_bytes = serde_json::to_vec(&json!({
        "seed": "aud004",
        "order_id": seed.order_id,
        "suffix": suffix,
    }))
    .expect("serialize source evidence");
    put_object_bytes(
        &source_bucket,
        &source_key,
        source_bytes.clone(),
        Some("application/json"),
    )
    .await
    .expect("upload source evidence object");
    let source_hash = format!("{:x}", Sha256::digest(source_bytes.as_slice()));
    let evidence_snapshot = record_evidence_snapshot(
        &client,
        &EvidenceWriteCommand {
            item_type: "order_snapshot".to_string(),
            ref_type: "order".to_string(),
            ref_id: Some(seed.order_id.clone()),
            object_uri: source_uri.clone(),
            object_hash: source_hash,
            content_type: Some("application/json".to_string()),
            size_bytes: Some(source_bytes.len() as i64),
            source_system: "audit.test".to_string(),
            storage_mode: "minio".to_string(),
            retention_policy_id: None,
            worm_enabled: false,
            legal_hold_status: "none".to_string(),
            created_by: Some(audit_user_id.clone()),
            metadata: json!({
                "seed": "aud004",
                "source": "integration_test",
            }),
            manifest_scope: "order_export_seed".to_string(),
            manifest_ref_type: "order".to_string(),
            manifest_ref_id: Some(seed.order_id.clone()),
            manifest_storage_uri: Some(source_uri.clone()),
            manifest_metadata: json!({
                "seed": "aud004",
                "order_id": seed.order_id,
            }),
            legacy_bridge: None,
        },
    )
    .await
    .expect("record export seed evidence snapshot");
    assert!(
        evidence_snapshot
            .evidence_manifest
            .evidence_manifest_id
            .is_some()
    );

    let export_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/audit/packages/export")
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &export_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &challenge_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "ref_type": "order",
                        "ref_id": seed.order_id,
                        "reason": "regulator check export",
                        "masked_level": "masked"
                    })
                    .to_string(),
                ))
                .expect("export request"),
        )
        .await
        .expect("call audit package export");
    let export_status = export_resp.status();
    let export_body = to_bytes(export_resp.into_body(), usize::MAX)
        .await
        .expect("read export body");
    assert_eq!(
        export_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&export_body)
    );
    let export_json: Value = serde_json::from_slice(&export_body).expect("decode export body");
    let evidence_package_id = export_json["data"]["evidence_package"]["evidence_package_id"]
        .as_str()
        .expect("evidence_package_id")
        .to_string();
    let evidence_manifest_id = export_json["data"]["evidence_manifest"]["evidence_manifest_id"]
        .as_str()
        .expect("evidence_manifest_id")
        .to_string();
    let storage_uri = export_json["data"]["evidence_package"]["storage_uri"]
        .as_str()
        .expect("storage uri");
    assert_eq!(export_json["data"]["step_up_bound"].as_bool(), Some(true));
    assert_eq!(
        export_json["data"]["legal_hold_status"].as_str(),
        Some("none")
    );
    assert!(
        export_json["data"]["evidence_item_count"]
            .as_i64()
            .unwrap_or_default()
            >= 1
    );

    let (bucket_name, object_key) = parse_s3_uri(storage_uri);
    let fetched_export = fetch_object_bytes(&bucket_name, &object_key)
        .await
        .expect("fetch exported package object");
    let fetched_export_json: Value =
        serde_json::from_slice(&fetched_export.bytes).expect("decode exported package");
    assert_eq!(
        fetched_export_json["reason"].as_str(),
        Some("regulator check export")
    );
    assert_eq!(
        fetched_export_json["target"]["order_id"].as_str(),
        Some(seed.order_id.as_str())
    );

    let export_row = client
        .query_one(
            "SELECT evidence_manifest_id::text,
                    package_digest,
                    storage_uri,
                    package_type,
                    masked_level,
                    access_mode,
                    legal_hold_status
             FROM audit.evidence_package
             WHERE evidence_package_id = $1::text::uuid",
            &[&evidence_package_id],
        )
        .await
        .expect("query evidence package row");
    let db_manifest_id: Option<String> = export_row.get(0);
    let db_storage_uri: Option<String> = export_row.get(2);
    let db_package_type: String = export_row.get(3);
    let db_masked_level: Option<String> = export_row.get(4);
    let db_access_mode: String = export_row.get(5);
    let db_legal_hold_status: String = export_row.get(6);
    assert_eq!(
        db_manifest_id.as_deref(),
        Some(evidence_manifest_id.as_str())
    );
    assert_eq!(db_storage_uri.as_deref(), Some(storage_uri));
    assert_eq!(db_package_type, "order_evidence_package");
    assert_eq!(db_masked_level.as_deref(), Some("masked"));
    assert_eq!(db_access_mode, "export");
    assert_eq!(db_legal_hold_status, "none");

    let export_audit_row = client
        .query_one(
            "SELECT COUNT(*)::bigint,
                    max(metadata ->> 'reason'),
                    max(metadata ->> 'masked_level')
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'audit.package.export'
               AND ref_type = 'order'
               AND ref_id = $2::text::uuid",
            &[&export_request_id, &seed.order_id],
        )
        .await
        .expect("count export audit event");
    let export_audit_count: i64 = export_audit_row.get(0);
    let export_audit_reason: Option<String> = export_audit_row.get(1);
    let export_audit_masked_level: Option<String> = export_audit_row.get(2);
    assert_eq!(export_audit_count, 1);
    assert_eq!(
        export_audit_reason.as_deref(),
        Some("regulator check export")
    );
    assert_eq!(export_audit_masked_level.as_deref(), Some("masked"));

    let export_access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = $1
               AND access_mode = 'export'
               AND step_up_challenge_id = $2::text::uuid",
            &[&export_request_id, &challenge_id],
        )
        .await
        .expect("count export access audit")
        .get(0);
    assert_eq!(export_access_count, 1);

    let export_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = 'audit package export executed: POST /api/v1/audit/packages/export'",
            &[&export_request_id],
        )
        .await
        .expect("count export system logs")
        .get(0);
    assert_eq!(export_log_count, 1);

    let manifest_item_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.evidence_manifest_item
             WHERE evidence_manifest_id = $1::text::uuid",
            &[&evidence_manifest_id],
        )
        .await
        .expect("count export manifest items")
        .get(0);
    assert!(manifest_item_count >= 2);

    let replay_challenge_id = seed_verified_step_up_challenge(
        &client,
        &audit_user_id,
        "audit.replay.execute",
        "order",
        Some(seed.order_id.as_str()),
        "aud005",
    )
    .await
    .expect("seed verified replay step-up challenge");

    let replay_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/audit/replay-jobs")
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &replay_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &replay_challenge_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "replay_type": "state_replay",
                        "ref_type": "order",
                        "ref_id": seed.order_id,
                        "reason": "investigate audit drift",
                        "dry_run": true,
                        "options": {
                            "trigger": "integration_test",
                            "source_export_id": evidence_package_id,
                        }
                    })
                    .to_string(),
                ))
                .expect("replay request"),
        )
        .await
        .expect("call audit replay create");
    let replay_status = replay_resp.status();
    let replay_body = to_bytes(replay_resp.into_body(), usize::MAX)
        .await
        .expect("read replay body");
    assert_eq!(
        replay_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&replay_body)
    );
    let replay_json: Value = serde_json::from_slice(&replay_body).expect("decode replay body");
    let replay_job_id = replay_json["data"]["replay_job"]["replay_job_id"]
        .as_str()
        .expect("replay_job_id")
        .to_string();
    assert_eq!(
        replay_json["data"]["replay_job"]["replay_status"].as_str(),
        Some("completed")
    );
    assert_eq!(
        replay_json["data"]["replay_job"]["dry_run"].as_bool(),
        Some(true)
    );
    assert_eq!(
        replay_json["data"]["results"]
            .as_array()
            .map(|items| items.len())
            .unwrap_or_default(),
        4
    );
    assert_eq!(
        replay_json["data"]["results"][3]["result_code"].as_str(),
        Some("AUDIT_REPLAY_DRY_RUN_ONLY")
    );

    let replay_lookup_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/audit/replay-jobs/{replay_job_id}"))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &replay_lookup_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("replay lookup request"),
        )
        .await
        .expect("call audit replay lookup");
    assert_eq!(replay_lookup_resp.status(), StatusCode::OK);
    let replay_lookup_body = to_bytes(replay_lookup_resp.into_body(), usize::MAX)
        .await
        .expect("read replay lookup body");
    let replay_lookup_json: Value =
        serde_json::from_slice(&replay_lookup_body).expect("decode replay lookup body");
    assert_eq!(
        replay_lookup_json["data"]["replay_job"]["replay_job_id"].as_str(),
        Some(replay_job_id.as_str())
    );

    let replay_job_row = client
        .query_one(
            "SELECT replay_type,
                    ref_type,
                    ref_id::text,
                    dry_run,
                    status,
                    requested_by::text,
                    request_reason,
                    options_json ->> 'report_storage_uri'
             FROM audit.replay_job
             WHERE replay_job_id = $1::text::uuid",
            &[&replay_job_id],
        )
        .await
        .expect("query replay job row");
    let replay_report_uri: Option<String> = replay_job_row.get(7);
    assert_eq!(replay_job_row.get::<_, String>(0), "state_replay");
    assert_eq!(replay_job_row.get::<_, String>(1), "order");
    assert_eq!(
        replay_job_row.get::<_, Option<String>>(2).as_deref(),
        Some(seed.order_id.as_str())
    );
    assert!(replay_job_row.get::<_, bool>(3));
    assert_eq!(replay_job_row.get::<_, String>(4), "completed");
    assert_eq!(
        replay_job_row.get::<_, Option<String>>(5).as_deref(),
        Some(audit_user_id.as_str())
    );
    assert_eq!(
        replay_job_row.get::<_, Option<String>>(6).as_deref(),
        Some("investigate audit drift")
    );

    let replay_report_uri = replay_report_uri.expect("replay report uri");
    let (replay_bucket, replay_key) = parse_s3_uri(&replay_report_uri);
    let replay_report = fetch_object_bytes(&replay_bucket, &replay_key)
        .await
        .expect("fetch replay report");
    let replay_report_json: Value =
        serde_json::from_slice(&replay_report.bytes).expect("decode replay report");
    assert_eq!(
        replay_report_json["recommendation"].as_str(),
        Some("dry_run_completed")
    );
    assert_eq!(
        replay_report_json["target"]["order_id"].as_str(),
        Some(seed.order_id.as_str())
    );

    let replay_result_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.replay_result
             WHERE replay_job_id = $1::text::uuid",
            &[&replay_job_id],
        )
        .await
        .expect("count replay results")
        .get(0);
    assert_eq!(replay_result_count, 4);

    let replay_audit_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND ref_type = 'replay_job'
               AND ref_id = $2::text::uuid
               AND action_name IN ('audit.replay.requested', 'audit.replay.completed')",
            &[&replay_request_id, &replay_job_id],
        )
        .await
        .expect("count replay audit events")
        .get(0);
    assert_eq!(replay_audit_count, 2);

    let replay_access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND access_mode = 'replay'",
            &[&vec![
                replay_request_id.clone(),
                replay_lookup_request_id.clone(),
            ]],
        )
        .await
        .expect("count replay access audit")
        .get(0);
    assert_eq!(replay_access_count, 2);

    let replay_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text LIKE 'audit replay%'",
            &[&vec![
                replay_request_id.clone(),
                replay_lookup_request_id.clone(),
            ]],
        )
        .await
        .expect("count replay logs")
        .get(0);
    assert_eq!(replay_log_count, 2);

    let legal_hold_challenge_id = seed_verified_step_up_challenge(
        &client,
        &audit_user_id,
        "audit.legal_hold.manage",
        "order",
        Some(seed.order_id.as_str()),
        "aud006",
    )
    .await
    .expect("seed verified legal hold step-up challenge");

    let legal_hold_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/audit/legal-holds")
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &legal_hold_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &legal_hold_challenge_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "hold_scope_type": "order",
                        "hold_scope_id": seed.order_id,
                        "reason_code": "regulator_investigation",
                        "metadata": {
                            "trigger": "integration_test",
                            "case_ref": "aud006-smoke",
                        }
                    })
                    .to_string(),
                ))
                .expect("legal hold request"),
        )
        .await
        .expect("call audit legal hold create");
    let legal_hold_status = legal_hold_resp.status();
    let legal_hold_body = to_bytes(legal_hold_resp.into_body(), usize::MAX)
        .await
        .expect("read legal hold body");
    assert_eq!(
        legal_hold_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&legal_hold_body)
    );
    let legal_hold_json: Value =
        serde_json::from_slice(&legal_hold_body).expect("decode legal hold body");
    let legal_hold_id = legal_hold_json["data"]["legal_hold"]["legal_hold_id"]
        .as_str()
        .expect("legal_hold_id")
        .to_string();
    assert_eq!(
        legal_hold_json["data"]["legal_hold"]["status"].as_str(),
        Some("active")
    );
    assert_eq!(
        legal_hold_json["data"]["legal_hold"]["hold_scope_type"].as_str(),
        Some("order")
    );
    assert_eq!(
        legal_hold_json["data"]["step_up_bound"].as_bool(),
        Some(true)
    );

    let legal_hold_conflict_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/audit/legal-holds")
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &legal_hold_conflict_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &legal_hold_challenge_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "hold_scope_type": "order",
                        "hold_scope_id": seed.order_id,
                        "reason_code": "regulator_investigation",
                    })
                    .to_string(),
                ))
                .expect("legal hold conflict request"),
        )
        .await
        .expect("call audit legal hold conflict");
    assert_eq!(legal_hold_conflict_resp.status(), StatusCode::CONFLICT);
    let legal_hold_conflict_body = to_bytes(legal_hold_conflict_resp.into_body(), usize::MAX)
        .await
        .expect("read legal hold conflict body");
    let legal_hold_conflict_json: Value =
        serde_json::from_slice(&legal_hold_conflict_body).expect("decode legal hold conflict");
    assert_eq!(
        legal_hold_conflict_json["code"].as_str(),
        Some("AUDIT_LEGAL_HOLD_ACTIVE")
    );

    let legal_hold_row = client
        .query_one(
            "SELECT hold_scope_type,
                    hold_scope_id::text,
                    reason_code,
                    status,
                    requested_by::text,
                    metadata ->> 'order_id',
                    metadata -> 'request_metadata' ->> 'trigger'
             FROM audit.legal_hold
             WHERE legal_hold_id = $1::text::uuid",
            &[&legal_hold_id],
        )
        .await
        .expect("query legal hold row");
    assert_eq!(legal_hold_row.get::<_, String>(0), "order");
    assert_eq!(
        legal_hold_row.get::<_, Option<String>>(1).as_deref(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        legal_hold_row.get::<_, String>(2),
        "regulator_investigation"
    );
    assert_eq!(legal_hold_row.get::<_, String>(3), "active");
    assert_eq!(
        legal_hold_row.get::<_, Option<String>>(4).as_deref(),
        Some(audit_user_id.as_str())
    );
    assert_eq!(
        legal_hold_row.get::<_, Option<String>>(5).as_deref(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        legal_hold_row.get::<_, Option<String>>(6).as_deref(),
        Some("integration_test")
    );

    let active_scope_status = client
        .query_one(
            "SELECT
               (SELECT COUNT(*)::bigint
                FROM audit.legal_hold
                WHERE hold_scope_type = 'order'
                  AND hold_scope_id = $1::text::uuid
                  AND status = 'active'),
               (SELECT legal_hold_status
                FROM audit.evidence_item
                WHERE ref_type = 'order'
                  AND ref_id = $1::text::uuid
                ORDER BY created_at DESC
                LIMIT 1),
               (SELECT legal_hold_status
                FROM audit.evidence_package
                WHERE evidence_package_id = $2::text::uuid)",
            &[&seed.order_id, &evidence_package_id],
        )
        .await
        .expect("query active legal hold statuses");
    assert_eq!(active_scope_status.get::<_, i64>(0), 1);
    assert_eq!(
        active_scope_status.get::<_, Option<String>>(1).as_deref(),
        Some("none")
    );
    assert_eq!(
        active_scope_status.get::<_, Option<String>>(2).as_deref(),
        Some("none")
    );

    let legal_hold_audit_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'audit.legal_hold.create'
               AND ref_type = 'order'
               AND ref_id = $2::text::uuid",
            &[&legal_hold_request_id, &seed.order_id],
        )
        .await
        .expect("count legal hold create audit event")
        .get(0);
    assert_eq!(legal_hold_audit_count, 1);

    let legal_hold_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = 'audit legal hold created: POST /api/v1/audit/legal-holds'",
            &[&legal_hold_request_id],
        )
        .await
        .expect("count legal hold create logs")
        .get(0);
    assert_eq!(legal_hold_log_count, 1);

    let legal_hold_release_challenge_id = seed_verified_step_up_challenge(
        &client,
        &audit_user_id,
        "audit.legal_hold.manage",
        "legal_hold",
        Some(legal_hold_id.as_str()),
        "aud006-release",
    )
    .await
    .expect("seed verified legal hold release challenge");

    let legal_hold_release_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/audit/legal-holds/{legal_hold_id}/release"))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &legal_hold_release_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &legal_hold_release_challenge_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "reason": "manual review cleared hold",
                        "metadata": {
                            "resolution": "cleared",
                        }
                    })
                    .to_string(),
                ))
                .expect("legal hold release request"),
        )
        .await
        .expect("call audit legal hold release");
    let legal_hold_release_status = legal_hold_release_resp.status();
    let legal_hold_release_body = to_bytes(legal_hold_release_resp.into_body(), usize::MAX)
        .await
        .expect("read legal hold release body");
    assert_eq!(
        legal_hold_release_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&legal_hold_release_body)
    );
    let legal_hold_release_json: Value =
        serde_json::from_slice(&legal_hold_release_body).expect("decode legal hold release");
    assert_eq!(
        legal_hold_release_json["data"]["legal_hold"]["status"].as_str(),
        Some("released")
    );
    assert_eq!(
        legal_hold_release_json["data"]["legal_hold"]["approved_by"].as_str(),
        Some(audit_user_id.as_str())
    );
    assert!(
        legal_hold_release_json["data"]["legal_hold"]["released_at"]
            .as_str()
            .is_some()
    );

    let released_scope_status = client
        .query_one(
            "SELECT
               (SELECT COUNT(*)::bigint
                FROM audit.legal_hold
                WHERE hold_scope_type = 'order'
                  AND hold_scope_id = $1::text::uuid
                  AND status = 'active'),
               (SELECT legal_hold_status
                FROM audit.evidence_item
                WHERE ref_type = 'order'
                  AND ref_id = $1::text::uuid
                ORDER BY created_at DESC
                LIMIT 1),
               (SELECT legal_hold_status
                FROM audit.evidence_package
                WHERE evidence_package_id = $2::text::uuid),
               (SELECT status FROM audit.legal_hold WHERE legal_hold_id = $3::text::uuid),
               (SELECT metadata ->> 'release_reason' FROM audit.legal_hold WHERE legal_hold_id = $3::text::uuid)",
            &[&seed.order_id, &evidence_package_id, &legal_hold_id],
        )
        .await
        .expect("query released legal hold statuses");
    assert_eq!(released_scope_status.get::<_, i64>(0), 0);
    assert_eq!(
        released_scope_status.get::<_, Option<String>>(1).as_deref(),
        Some("none")
    );
    assert_eq!(
        released_scope_status.get::<_, Option<String>>(2).as_deref(),
        Some("none")
    );
    assert_eq!(
        released_scope_status.get::<_, Option<String>>(3).as_deref(),
        Some("released")
    );
    assert_eq!(
        released_scope_status.get::<_, Option<String>>(4).as_deref(),
        Some("manual review cleared hold")
    );

    let legal_hold_release_audit_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'audit.legal_hold.release'
               AND ref_type = 'order'
               AND ref_id = $2::text::uuid",
            &[&legal_hold_release_request_id, &seed.order_id],
        )
        .await
        .expect("count legal hold release audit event")
        .get(0);
    assert_eq!(legal_hold_release_audit_count, 1);

    let legal_hold_release_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = 'audit legal hold released: POST /api/v1/audit/legal-holds/{id}/release'",
            &[&legal_hold_release_request_id],
        )
        .await
        .expect("count legal hold release logs")
        .get(0);
    assert_eq!(legal_hold_release_log_count, 1);

    let (anchor_batch_id, _chain_anchor_id) =
        seed_failed_anchor_batch(&client, &audit_user_id, &suffix)
            .await
            .expect("seed failed anchor batch");
    let anchor_list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/audit/anchor-batches?anchor_status=failed&batch_scope=audit_event&chain_id=fabric-local")
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &anchor_list_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("anchor batch list request"),
        )
        .await
        .expect("call anchor batch list");
    let anchor_list_status = anchor_list_resp.status();
    let anchor_list_body = to_bytes(anchor_list_resp.into_body(), usize::MAX)
        .await
        .expect("read anchor batch list body");
    assert_eq!(
        anchor_list_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&anchor_list_body)
    );
    let anchor_list_json: Value =
        serde_json::from_slice(&anchor_list_body).expect("decode anchor batch list");
    assert_eq!(anchor_list_json["data"]["total"].as_i64(), Some(1));
    assert_eq!(
        anchor_list_json["data"]["items"][0]["anchor_batch_id"].as_str(),
        Some(anchor_batch_id.as_str())
    );
    assert_eq!(
        anchor_list_json["data"]["items"][0]["anchor_status"].as_str(),
        Some("failed")
    );
    assert_eq!(
        anchor_list_json["data"]["items"][0]["tx_hash"].as_str(),
        Some("0xaud007failedanchor")
    );

    let anchor_retry_challenge_id = seed_verified_step_up_challenge(
        &client,
        &audit_user_id,
        "audit.anchor.manage",
        "anchor_batch",
        Some(anchor_batch_id.as_str()),
        "aud007",
    )
    .await
    .expect("seed verified anchor retry challenge");

    let anchor_retry_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/audit/anchor-batches/{anchor_batch_id}/retry"
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &anchor_retry_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &anchor_retry_challenge_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "reason": "retry failed batch after fabric gateway timeout",
                        "metadata": {
                            "trigger": "integration_test",
                            "ticket_id": "AUD-007-smoke",
                        }
                    })
                    .to_string(),
                ))
                .expect("anchor batch retry request"),
        )
        .await
        .expect("call anchor batch retry");
    let anchor_retry_status = anchor_retry_resp.status();
    let anchor_retry_body = to_bytes(anchor_retry_resp.into_body(), usize::MAX)
        .await
        .expect("read anchor batch retry body");
    assert_eq!(
        anchor_retry_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&anchor_retry_body)
    );
    let anchor_retry_json: Value =
        serde_json::from_slice(&anchor_retry_body).expect("decode anchor batch retry");
    assert_eq!(
        anchor_retry_json["data"]["anchor_batch"]["anchor_batch_id"].as_str(),
        Some(anchor_batch_id.as_str())
    );
    assert_eq!(
        anchor_retry_json["data"]["anchor_batch"]["anchor_status"].as_str(),
        Some("retry_requested")
    );
    assert_eq!(
        anchor_retry_json["data"]["step_up_bound"].as_bool(),
        Some(true)
    );

    let anchor_batch_row = client
        .query_one(
            "SELECT status,
                    metadata ->> 'previous_status',
                    metadata -> 'retry_request' ->> 'reason',
                    metadata -> 'retry_request' ->> 'request_id',
                    metadata -> 'retry_request' -> 'request_metadata' ->> 'trigger'
             FROM audit.anchor_batch
             WHERE anchor_batch_id = $1::text::uuid",
            &[&anchor_batch_id],
        )
        .await
        .expect("query anchor batch row");
    assert_eq!(anchor_batch_row.get::<_, String>(0), "retry_requested");
    assert_eq!(
        anchor_batch_row.get::<_, Option<String>>(1).as_deref(),
        Some("failed")
    );
    assert_eq!(
        anchor_batch_row.get::<_, Option<String>>(2).as_deref(),
        Some("retry failed batch after fabric gateway timeout")
    );
    assert_eq!(
        anchor_batch_row.get::<_, Option<String>>(3).as_deref(),
        Some(anchor_retry_request_id.as_str())
    );
    assert_eq!(
        anchor_batch_row.get::<_, Option<String>>(4).as_deref(),
        Some("integration_test")
    );

    let anchor_outbox_row = client
        .query_one(
            "SELECT target_topic,
                    event_type,
                    aggregate_type,
                    aggregate_id::text,
                    payload ->> 'anchor_status',
                    payload ->> 'previous_anchor_status',
                    payload ->> 'retry_reason',
                    status
             FROM ops.outbox_event
             WHERE request_id = $1
             ORDER BY created_at DESC, outbox_event_id DESC
             LIMIT 1",
            &[&anchor_retry_request_id],
        )
        .await
        .expect("query anchor retry outbox row");
    assert_eq!(anchor_outbox_row.get::<_, String>(0), "dtp.audit.anchor");
    assert_eq!(
        anchor_outbox_row.get::<_, String>(1),
        "audit.anchor_requested"
    );
    assert_eq!(anchor_outbox_row.get::<_, String>(2), "audit.anchor_batch");
    assert_eq!(anchor_outbox_row.get::<_, String>(3), anchor_batch_id);
    assert_eq!(
        anchor_outbox_row.get::<_, Option<String>>(4).as_deref(),
        Some("retry_requested")
    );
    assert_eq!(
        anchor_outbox_row.get::<_, Option<String>>(5).as_deref(),
        Some("failed")
    );
    assert_eq!(
        anchor_outbox_row.get::<_, Option<String>>(6).as_deref(),
        Some("retry failed batch after fabric gateway timeout")
    );
    assert_eq!(anchor_outbox_row.get::<_, String>(7), "pending");

    let anchor_audit_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'audit.anchor.retry'
               AND ref_type = 'anchor_batch'
               AND ref_id = $2::text::uuid",
            &[&anchor_retry_request_id, &anchor_batch_id],
        )
        .await
        .expect("count anchor retry audit event")
        .get(0);
    assert_eq!(anchor_audit_count, 1);

    let anchor_access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND target_type = ANY($2::text[])
               AND access_mode = ANY($3::text[])",
            &[
                &vec![
                    anchor_list_request_id.clone(),
                    anchor_retry_request_id.clone(),
                ],
                &vec!["anchor_batch_query".to_string(), "anchor_batch".to_string()],
                &vec!["masked".to_string(), "retry".to_string()],
            ],
        )
        .await
        .expect("count anchor access audit")
        .get(0);
    assert_eq!(anchor_access_count, 2);

    let anchor_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = ANY($2::text[])",
            &[
                &vec![
                    anchor_list_request_id.clone(),
                    anchor_retry_request_id.clone(),
                ],
                &vec![
                "audit lookup executed: GET /api/v1/audit/anchor-batches".to_string(),
                "audit anchor batch retry requested: POST /api/v1/audit/anchor-batches/{id}/retry"
                    .to_string(),
            ],
            ],
        )
        .await
        .expect("count anchor logs")
        .get(0);
    assert_eq!(anchor_log_count, 2);

    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&challenge_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&replay_challenge_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&legal_hold_challenge_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&legal_hold_release_challenge_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&anchor_retry_challenge_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
            &[&audit_user_id],
        )
        .await;
    cleanup_business_rows(&client, &seed).await;
}

async fn seed_verified_step_up_challenge(
    client: &Client,
    user_id: &str,
    target_action: &str,
    target_ref_type: &str,
    target_ref_id: Option<&str>,
    seed_label: &str,
) -> Result<String, Error> {
    client
        .query_one(
            "INSERT INTO iam.step_up_challenge (
               user_id,
               challenge_type,
               target_action,
               target_ref_type,
               target_ref_id,
               challenge_status,
               expires_at,
               completed_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               'mock_otp',
               $2,
               $3,
               $4::text::uuid,
               'verified',
               now() + interval '10 minutes',
               now(),
               jsonb_build_object('seed', $5)
             )
             RETURNING step_up_challenge_id::text",
            &[
                &user_id,
                &target_action,
                &target_ref_type,
                &target_ref_id,
                &seed_label,
            ],
        )
        .await
        .map(|row| row.get(0))
}

async fn seed_failed_anchor_batch(
    client: &Client,
    user_id: &str,
    suffix: &str,
) -> Result<(String, String), Error> {
    let anchor_batch_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await?
        .get(0);
    let chain_anchor_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await?
        .get(0);

    client
        .execute(
            "INSERT INTO chain.chain_anchor (
               chain_anchor_id,
               chain_id,
               anchor_type,
               ref_type,
               ref_id,
               digest,
               tx_hash,
               status,
               authority_model,
               reconcile_status,
               created_at
             ) VALUES (
               $1::text::uuid,
               'fabric-local',
               'audit_batch',
               'anchor_batch',
               $2::text::uuid,
               $3,
               '0xaud007failedanchor',
               'failed',
               'dual_authority',
               'pending',
               now()
             )",
            &[
                &chain_anchor_id,
                &anchor_batch_id,
                &format!("aud007-digest-{suffix}"),
            ],
        )
        .await?;

    client
        .execute(
            "INSERT INTO audit.anchor_batch (
               anchor_batch_id,
               batch_scope,
               chain_id,
               record_count,
               batch_root,
               window_started_at,
               window_ended_at,
               status,
               chain_anchor_id,
               created_by,
               metadata
             ) VALUES (
               $1::text::uuid,
               'audit_event',
               'fabric-local',
               2,
               $2,
               now() - interval '5 minutes',
               now() - interval '1 minute',
               'failed',
               $3::text::uuid,
               $4::text::uuid,
               jsonb_build_object('seed', $5)
             )",
            &[
                &anchor_batch_id,
                &format!("aud007-batch-root-{suffix}"),
                &chain_anchor_id,
                &user_id,
                &suffix,
            ],
        )
        .await?;

    Ok((anchor_batch_id, chain_anchor_id))
}

async fn seed_user(client: &Client, org_id: &str, suffix: &str) -> Result<String, Error> {
    client
        .query_one(
            "INSERT INTO core.user_account (
               org_id,
               login_id,
               display_name,
               user_type,
               status,
               mfa_status,
               email,
               attrs
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               'human',
               'active',
               'verified',
               $4,
               '{}'::jsonb
             )
             RETURNING user_id::text",
            &[
                &org_id,
                &format!("aud004-user-{suffix}"),
                &format!("AUD004 User {suffix}"),
                &format!("aud004-{suffix}@example.com"),
            ],
        )
        .await
        .map(|row| row.get(0))
}

fn parse_s3_uri(uri: &str) -> (String, String) {
    let without_scheme = uri.trim_start_matches("s3://");
    let mut parts = without_scheme.splitn(2, '/');
    let bucket = parts.next().unwrap_or_default().to_string();
    let key = parts.next().unwrap_or_default().to_string();
    (bucket, key)
}

async fn seed_order_graph(client: &Client, suffix: &str) -> Result<SeedGraph, Error> {
    let buyer_org_id: String = client
        .query_one(
            "INSERT INTO core.organization (org_name, org_type, status, metadata)
             VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
             RETURNING org_id::text",
            &[&format!("aud003-buyer-{suffix}")],
        )
        .await?
        .get(0);
    let seller_org_id: String = client
        .query_one(
            "INSERT INTO core.organization (org_name, org_type, status, metadata)
             VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
             RETURNING org_id::text",
            &[&format!("aud003-seller-{suffix}")],
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
                &format!("aud003-asset-{suffix}"),
                &format!("audit trace asset {suffix}"),
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
                 1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
               )
               RETURNING asset_version_id::text"#,
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
                 $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing',
                 'data_product', $5, 'listed', 'one_time', 88.00, 'CNY', 'file_download',
                 ARRAY['internal_use']::text[], $6, '{"review_status":"approved"}'::jsonb
               )
               RETURNING product_id::text"#,
            &[
                &asset_id,
                &asset_version_id,
                &seller_org_id,
                &format!("aud003-product-{suffix}"),
                &format!("audit trace product {suffix}"),
                &format!("audit trace search {suffix}"),
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
               $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'file_download',
               'download_file', 'manual_accept', 'manual_refund', 'active'
             )
             RETURNING sku_id::text",
            &[&product_id, &format!("AUD003-FILE-STD-{suffix}")],
        )
        .await?
        .get(0);
    let order_id: String = client
        .query_one(
            "INSERT INTO trade.order_main (
               product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
               status, payment_status, delivery_status, acceptance_status,
               settlement_status, dispute_status, payment_mode, amount, currency_code,
               price_snapshot_json
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
               'buyer_locked', 'paid', 'pending_delivery', 'not_started',
               'pending_settlement', 'none', 'online', 88.00, 'CNY',
               jsonb_build_object('sku_type', 'FILE_STD', 'audit_seed', $6)
             )
             RETURNING order_id::text",
            &[
                &product_id,
                &asset_version_id,
                &buyer_org_id,
                &seller_org_id,
                &sku_id,
                &suffix,
            ],
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

async fn cleanup_business_rows(client: &Client, seed: &SeedGraph) {
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
            "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
            &[&seed.buyer_org_id, &seed.seller_org_id],
        )
        .await;
}
