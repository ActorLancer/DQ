use crate::modules::audit::api::router;
use crate::modules::audit::application::{EvidenceWriteCommand, record_evidence_snapshot};
use crate::modules::audit::domain::{
    ChainProjectionGapQuery, ConsumerIdempotencyQuery, ExternalFactReceiptQuery,
};
use crate::modules::audit::repo;
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

    #[tokio::test]
    async fn rejects_ops_outbox_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/outbox")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud008-outbox-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn ops_dead_letters_requires_request_id() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/dead-letters")
            .header("x-role", "platform_audit_security")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_dead_letter_reprocess_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/dead-letters/10000000-0000-0000-0000-000000000010/reprocess")
            .header("x-role", "buyer_operator")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud010-reprocess-forbidden")
            .header("x-step-up-token", "aud010-stepup")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"reason":"forbidden"}"#))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn dead_letter_reprocess_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/dead-letters/10000000-0000-0000-0000-000000000010/reprocess")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud010-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"reason":"missing step-up"}"#))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn dead_letter_reprocess_enforces_dry_run_only() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/dead-letters/10000000-0000-0000-0000-000000000010/reprocess")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud010-dry-run")
            .header("x-step-up-token", "aud010-stepup")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"reason":"dry-run only","dry_run":false}"#))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn rejects_ops_consistency_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/consistency/order/10000000-0000-0000-0000-000000000011")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud011-consistency-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_ops_consistency_with_unsupported_ref_type() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/consistency/unsupported/10000000-0000-0000-0000-000000000011")
            .header("x-role", "platform_audit_security")
            .header("x-request-id", "req-aud011-consistency-invalid-ref-type")
            .body(Body::empty())
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
    let ops_outbox_request_id = format!("req-aud008-outbox-{suffix}");
    let ops_dead_letter_request_id = format!("req-aud008-dead-letters-{suffix}");
    let trace_id = format!("trace-aud003-{suffix}");
    let ops_trace_id = format!("trace-aud008-{suffix}");

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

    let (anchor_batch_id, chain_anchor_id) =
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

    let outbox_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud008 outbox id")
        .get(0);
    let external_fact_receipt_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud008 external fact id")
        .get(0);
    let chain_projection_gap_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud008 projection gap id")
        .get(0);

    client
        .execute(
            "INSERT INTO ops.outbox_event (
               outbox_event_id,
               aggregate_type,
               aggregate_id,
               event_type,
               payload,
               status,
               retry_count,
               max_retries,
               available_at,
               created_at,
               event_schema_version,
               request_id,
               trace_id,
               idempotency_key,
               authority_scope,
               source_of_truth,
               proof_commit_policy,
               target_bus,
               target_topic,
               partition_key,
               ordering_key,
               payload_hash,
               last_error_code,
               last_error_message
             ) VALUES (
               $1::text::uuid,
               'search.index_sync_task',
               $2::text::uuid,
               'search.product.changed',
               jsonb_build_object(
                 'event_id', $1,
                 'event_type', 'search.product.changed',
                 'aggregate_type', 'search.index_sync_task',
                 'aggregate_id', $2,
                 'request_id', $3,
                 'trace_id', $4,
                 'payload', jsonb_build_object('seed', $5)
               ),
               'failed',
               2,
               16,
               now() - interval '2 minutes',
               now() - interval '5 minutes',
               'v1',
               $3,
               $4,
               $6,
               'business',
               'database',
               'async_evidence',
               'kafka',
               'dtp.search.sync',
               $2,
               $2,
               $7,
               'KAFKA_TIMEOUT',
               'publisher timeout while relaying search sync'
             )",
            &[
                &outbox_event_id,
                &seed.product_id,
                &ops_outbox_request_id,
                &ops_trace_id,
                &suffix,
                &format!("aud008-idempotency-{suffix}"),
                &format!("aud008-payload-hash-{suffix}"),
            ],
        )
        .await
        .expect("insert aud008 outbox event");

    client
        .execute(
            "INSERT INTO ops.outbox_publish_attempt (
               outbox_event_id,
               worker_id,
               target_bus,
               target_topic,
               attempt_no,
               result_code,
               error_code,
               error_message,
               attempted_at,
               completed_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               'outbox-publisher-local',
               'kafka',
               'dtp.search.sync',
               2,
               'failed',
               'KAFKA_TIMEOUT',
               'broker timeout during publish',
               now() - interval '90 seconds',
               now() - interval '89 seconds',
               jsonb_build_object('seed', $2, 'worker', 'outbox-publisher-local')
             )",
            &[&outbox_event_id, &suffix],
        )
        .await
        .expect("insert aud008 outbox publish attempt");

    client
        .execute(
            "INSERT INTO ops.dead_letter_event (
               dead_letter_event_id,
               outbox_event_id,
               aggregate_type,
               aggregate_id,
               event_type,
               payload,
               failed_reason,
               created_at,
               request_id,
               trace_id,
               authority_scope,
               source_of_truth,
               target_bus,
               target_topic,
               failure_stage,
               first_failed_at,
               last_failed_at,
               reprocess_status
             ) VALUES (
               gen_random_uuid(),
               $1::text::uuid,
               'search.index_sync_task',
               $2::text::uuid,
               'search.product.changed',
               jsonb_build_object('event_id', $1, 'seed', $3, 'target_topic', 'dtp.search.sync'),
               'search projection consumer isolated after repeated failure',
               now() - interval '80 seconds',
               $4,
               $5,
               'business',
               'database',
               'kafka',
               'dtp.search.sync',
               'consumer_handler',
               now() - interval '85 seconds',
               now() - interval '75 seconds',
               'not_reprocessed'
             )",
            &[
                &outbox_event_id,
                &seed.product_id,
                &suffix,
                &ops_dead_letter_request_id,
                &ops_trace_id,
            ],
        )
        .await
        .expect("insert aud008 dead letter");

    client
        .execute(
            "INSERT INTO ops.consumer_idempotency_record (
               consumer_name,
               event_id,
               aggregate_type,
               aggregate_id,
               trace_id,
               result_code,
               metadata
             ) VALUES (
               'search-indexer',
               $1::text::uuid,
               'search.index_sync_task',
               $2::text::uuid,
               $3,
               'dead_lettered',
               jsonb_build_object('seed', $4, 'dlq_target', 'dtp.dead-letter')
             )",
            &[&outbox_event_id, &seed.product_id, &ops_trace_id, &suffix],
        )
        .await
        .expect("insert aud008 consumer idempotency");

    client
        .execute(
            "INSERT INTO ops.external_fact_receipt (
               external_fact_receipt_id,
               order_id,
               ref_domain,
               ref_type,
               ref_id,
               fact_type,
               provider_type,
               provider_key,
               provider_reference,
               receipt_status,
               receipt_payload,
               receipt_hash,
               occurred_at,
               received_at,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'payment',
               'order',
               $2::text::uuid,
               'payment_callback',
               'mock_payment_provider',
               'mockpay',
               $3,
               'pending',
               jsonb_build_object('seed', $4, 'provider_status', 'received'),
               $5,
               now() - interval '70 seconds',
               now() - interval '69 seconds',
               $6,
               $7,
               jsonb_build_object('seed', $4)
             )",
            &[
                &external_fact_receipt_id,
                &seed.order_id,
                &format!("provider-ref-{suffix}"),
                &suffix,
                &format!("receipt-hash-{suffix}"),
                &ops_outbox_request_id,
                &ops_trace_id,
            ],
        )
        .await
        .expect("insert aud008 external fact receipt");

    client
        .execute(
            "INSERT INTO ops.chain_projection_gap (
               chain_projection_gap_id,
               aggregate_type,
               aggregate_id,
               order_id,
               chain_id,
               source_event_type,
               expected_tx_id,
               projected_tx_hash,
               gap_type,
               gap_status,
               first_detected_at,
               last_detected_at,
               request_id,
               trace_id,
               outbox_event_id,
               anchor_id,
               resolution_summary,
               metadata
             ) VALUES (
               $1::text::uuid,
               'order',
               $2::text::uuid,
               $2::text::uuid,
               'fabric-local',
               'audit.anchor_requested',
               $3,
               NULL,
               'missing_projection',
               'open',
               now() - interval '60 seconds',
               now() - interval '55 seconds',
               $4,
               $5,
               $6::text::uuid,
               $7::text::uuid,
               '{}'::jsonb,
               jsonb_build_object('seed', $8)
             )",
            &[
                &chain_projection_gap_id,
                &seed.order_id,
                &format!("expected-tx-{suffix}"),
                &ops_outbox_request_id,
                &ops_trace_id,
                &outbox_event_id,
                &chain_anchor_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud008 projection gap");

    let ops_outbox_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/outbox?request_id={}&target_topic=dtp.search.sync&event_type=search.product.changed",
                    ops_outbox_request_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &ops_outbox_request_id)
                .header("x-trace-id", &ops_trace_id)
                .body(Body::empty())
                .expect("ops outbox request"),
        )
        .await
        .expect("call ops outbox");
    let ops_outbox_status = ops_outbox_resp.status();
    let ops_outbox_body = to_bytes(ops_outbox_resp.into_body(), usize::MAX)
        .await
        .expect("read ops outbox body");
    assert_eq!(
        ops_outbox_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&ops_outbox_body)
    );
    let ops_outbox_json: Value =
        serde_json::from_slice(&ops_outbox_body).expect("decode ops outbox body");
    assert_eq!(ops_outbox_json["data"]["total"].as_i64(), Some(1));
    assert_eq!(
        ops_outbox_json["data"]["items"][0]["outbox_event_id"].as_str(),
        Some(outbox_event_id.as_str())
    );
    assert_eq!(
        ops_outbox_json["data"]["items"][0]["target_topic"].as_str(),
        Some("dtp.search.sync")
    );
    assert_eq!(
        ops_outbox_json["data"]["items"][0]["latest_publish_attempt"]["result_code"].as_str(),
        Some("failed")
    );

    let ops_dead_letter_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/dead-letters?trace_id={}&failure_stage=consumer_handler",
                    ops_trace_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &audit_user_id)
                .header("x-request-id", &ops_dead_letter_request_id)
                .header("x-trace-id", &ops_trace_id)
                .body(Body::empty())
                .expect("ops dead letter request"),
        )
        .await
        .expect("call ops dead letters");
    let ops_dead_letter_status = ops_dead_letter_resp.status();
    let ops_dead_letter_body = to_bytes(ops_dead_letter_resp.into_body(), usize::MAX)
        .await
        .expect("read ops dead letters body");
    assert_eq!(
        ops_dead_letter_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&ops_dead_letter_body)
    );
    let ops_dead_letter_json: Value =
        serde_json::from_slice(&ops_dead_letter_body).expect("decode ops dead letters body");
    assert_eq!(ops_dead_letter_json["data"]["total"].as_i64(), Some(1));
    assert_eq!(
        ops_dead_letter_json["data"]["items"][0]["outbox_event_id"].as_str(),
        Some(outbox_event_id.as_str())
    );
    assert_eq!(
        ops_dead_letter_json["data"]["items"][0]["consumer_idempotency_records"][0]["consumer_name"]
            .as_str(),
        Some("search-indexer")
    );
    assert_eq!(
        ops_dead_letter_json["data"]["items"][0]["consumer_idempotency_records"][0]["result_code"]
            .as_str(),
        Some("dead_lettered")
    );

    let external_fact_page = repo::search_external_fact_receipts(
        &client,
        &ExternalFactReceiptQuery {
            order_id: Some(seed.order_id.clone()),
            receipt_status: Some("pending".to_string()),
            page: Some(1),
            page_size: Some(20),
            ..Default::default()
        },
        20,
        0,
    )
    .await
    .expect("search external fact receipts");
    assert_eq!(external_fact_page.total, 1);
    assert_eq!(
        external_fact_page.items[0]
            .external_fact_receipt_id
            .as_deref(),
        Some(external_fact_receipt_id.as_str())
    );

    let projection_gap_page = repo::search_chain_projection_gaps(
        &client,
        &ChainProjectionGapQuery {
            order_id: Some(seed.order_id.clone()),
            gap_status: Some("open".to_string()),
            page: Some(1),
            page_size: Some(20),
            ..Default::default()
        },
        20,
        0,
    )
    .await
    .expect("search projection gaps");
    assert_eq!(projection_gap_page.total, 1);
    assert_eq!(
        projection_gap_page.items[0]
            .chain_projection_gap_id
            .as_deref(),
        Some(chain_projection_gap_id.as_str())
    );
    assert_eq!(
        projection_gap_page.items[0].outbox_event_id.as_deref(),
        Some(outbox_event_id.as_str())
    );

    let idempotency_page = repo::search_consumer_idempotency_records(
        &client,
        &ConsumerIdempotencyQuery {
            consumer_name: Some("search-indexer".to_string()),
            event_id: Some(outbox_event_id.clone()),
            page: Some(1),
            page_size: Some(20),
            ..Default::default()
        },
        20,
        0,
    )
    .await
    .expect("search consumer idempotency");
    assert_eq!(idempotency_page.total, 1);
    assert_eq!(idempotency_page.items[0].consumer_name, "search-indexer");
    assert_eq!(idempotency_page.items[0].event_id, outbox_event_id);

    let ops_access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND target_type = ANY($2::text[])
               AND access_mode = 'masked'",
            &[
                &vec![
                    ops_outbox_request_id.clone(),
                    ops_dead_letter_request_id.clone(),
                ],
                &vec![
                    "ops_outbox_query".to_string(),
                    "dead_letter_query".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud008 access audit")
        .get(0);
    assert_eq!(ops_access_count, 2);

    let ops_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = ANY($2::text[])",
            &[
                &vec![
                    ops_outbox_request_id.clone(),
                    ops_dead_letter_request_id.clone(),
                ],
                &vec![
                    "ops lookup executed: GET /api/v1/ops/outbox".to_string(),
                    "ops lookup executed: GET /api/v1/ops/dead-letters".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud008 system logs")
        .get(0);
    assert_eq!(ops_log_count, 2);

    let _ = client
        .execute(
            "DELETE FROM ops.chain_projection_gap WHERE chain_projection_gap_id = $1::text::uuid",
            &[&chain_projection_gap_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.external_fact_receipt WHERE external_fact_receipt_id = $1::text::uuid",
            &[&external_fact_receipt_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.consumer_idempotency_record
             WHERE consumer_name = 'search-indexer'
               AND event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.dead_letter_event WHERE outbox_event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.outbox_publish_attempt WHERE outbox_event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.outbox_event WHERE outbox_event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await;

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

#[tokio::test]
async fn audit_dead_letter_reprocess_db_smoke() {
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
    let operator_org_id: String = client
        .query_one(
            "INSERT INTO core.organization (org_name, org_type, status, metadata)
             VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
             RETURNING org_id::text",
            &[&format!("AUD010 Ops Org {suffix}")],
        )
        .await
        .expect("insert ops org")
        .get(0);
    let operator_user_id = seed_user(&client, &operator_org_id, &format!("aud010-{suffix}"))
        .await
        .expect("seed aud010 user");

    let search_dead_letter = seed_searchrec_dead_letter(
        &client,
        &suffix,
        "dtp.search.sync",
        "search-indexer",
        "search.product.changed",
        "search.index_sync_task",
    )
    .await
    .expect("seed search dead letter");
    let recommendation_dead_letter = seed_searchrec_dead_letter(
        &client,
        &format!("{suffix}-rec"),
        "dtp.recommend.behavior",
        "recommendation-aggregator",
        "recommend.behavior_recorded",
        "recommend.behavior_event",
    )
    .await
    .expect("seed recommendation dead letter");

    let search_step_up_id = seed_verified_step_up_challenge(
        &client,
        &operator_user_id,
        "ops.dead_letter.reprocess",
        "dead_letter_event",
        Some(search_dead_letter.dead_letter_event_id.as_str()),
        &format!("aud010-search-{suffix}"),
    )
    .await
    .expect("seed search dead-letter step-up");
    let recommendation_step_up_id = seed_verified_step_up_challenge(
        &client,
        &operator_user_id,
        "ops.dead_letter.reprocess",
        "dead_letter_event",
        Some(recommendation_dead_letter.dead_letter_event_id.as_str()),
        &format!("aud010-rec-{suffix}"),
    )
    .await
    .expect("seed recommendation dead-letter step-up");

    let app = crate::with_live_test_state(router()).await;
    let search_request_id = format!("req-aud010-search-{suffix}");
    let recommendation_request_id = format!("req-aud010-rec-{suffix}");
    let search_trace_id = format!("trace-aud010-search-{suffix}");
    let recommendation_trace_id = format!("trace-aud010-rec-{suffix}");

    let search_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/ops/dead-letters/{}/reprocess",
                    search_dead_letter.dead_letter_event_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &search_request_id)
                .header("x-trace-id", &search_trace_id)
                .header("x-step-up-challenge-id", &search_step_up_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"reason":"preview search-indexer reprocess","dry_run":true,"metadata":{"source":"aud010-smoke"}}"#,
                ))
                .expect("search request"),
        )
        .await
        .expect("search response");
    assert_eq!(search_response.status(), StatusCode::OK);
    let search_json: Value = serde_json::from_slice(
        &to_bytes(search_response.into_body(), usize::MAX)
            .await
            .expect("read search body"),
    )
    .expect("decode search response");
    assert_eq!(search_json["data"]["status"], "dry_run_ready");
    assert_eq!(search_json["data"]["dry_run"].as_bool(), Some(true));
    assert_eq!(search_json["data"]["step_up_bound"].as_bool(), Some(true));
    assert_eq!(
        search_json["data"]["consumer_names"][0].as_str(),
        Some("search-indexer")
    );
    assert_eq!(
        search_json["data"]["consumer_groups"][0].as_str(),
        Some("cg-search-indexer")
    );
    assert_eq!(
        search_json["data"]["replay_target_topic"].as_str(),
        Some("dtp.search.sync")
    );

    let recommendation_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/ops/dead-letters/{}/reprocess",
                    recommendation_dead_letter.dead_letter_event_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &recommendation_request_id)
                .header("x-trace-id", &recommendation_trace_id)
                .header("x-step-up-challenge-id", &recommendation_step_up_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"reason":"preview recommendation reprocess","dry_run":true,"metadata":{"source":"aud010-smoke"}}"#,
                ))
                .expect("recommendation request"),
        )
        .await
        .expect("recommendation response");
    assert_eq!(recommendation_response.status(), StatusCode::OK);
    let recommendation_json: Value = serde_json::from_slice(
        &to_bytes(recommendation_response.into_body(), usize::MAX)
            .await
            .expect("read recommendation body"),
    )
    .expect("decode recommendation response");
    assert_eq!(recommendation_json["data"]["status"], "dry_run_ready");
    assert_eq!(
        recommendation_json["data"]["consumer_names"][0].as_str(),
        Some("recommendation-aggregator")
    );
    assert_eq!(
        recommendation_json["data"]["consumer_groups"][0].as_str(),
        Some("cg-recommendation-aggregator")
    );
    assert_eq!(
        recommendation_json["data"]["replay_target_topic"].as_str(),
        Some("dtp.recommend.behavior")
    );

    let reprocess_rows = client
        .query(
            "SELECT dead_letter_event_id::text, reprocess_status, reprocessed_at IS NULL
             FROM ops.dead_letter_event
             WHERE dead_letter_event_id::text = ANY($1::text[])
             ORDER BY dead_letter_event_id::text",
            &[&vec![
                search_dead_letter.dead_letter_event_id.clone(),
                recommendation_dead_letter.dead_letter_event_id.clone(),
            ]],
        )
        .await
        .expect("load dead-letter reprocess rows");
    assert_eq!(reprocess_rows.len(), 2);
    for row in reprocess_rows {
        assert_eq!(row.get::<_, String>(1), "not_reprocessed");
        assert!(row.get::<_, bool>(2));
    }

    let audit_rows = client
        .query(
            "SELECT action_name, result_code, request_id
             FROM audit.audit_event
             WHERE request_id = ANY($1::text[])
               AND action_name = 'ops.dead_letter.reprocess.dry_run'
             ORDER BY request_id",
            &[&vec![
                search_request_id.clone(),
                recommendation_request_id.clone(),
            ]],
        )
        .await
        .expect("load aud010 audit rows");
    assert_eq!(audit_rows.len(), 2);
    assert_eq!(audit_rows[0].get::<_, String>(1), "dry_run_completed");
    assert_eq!(audit_rows[1].get::<_, String>(1), "dry_run_completed");

    let access_rows = client
        .query(
            "SELECT access_mode, target_type, request_id
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
             ORDER BY request_id, created_at",
            &[&vec![
                search_request_id.clone(),
                recommendation_request_id.clone(),
            ]],
        )
        .await
        .expect("load aud010 access rows");
    assert_eq!(access_rows.len(), 2);
    for row in access_rows {
        assert_eq!(row.get::<_, String>(0), "reprocess");
        assert_eq!(row.get::<_, String>(1), "dead_letter_event");
    }

    let system_log_rows = client
        .query(
            "SELECT message_text, request_id
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = 'ops dead letter reprocess prepared: POST /api/v1/ops/dead-letters/{id}/reprocess'
             ORDER BY request_id, created_at",
            &[&vec![search_request_id.clone(), recommendation_request_id.clone()]],
        )
        .await
        .expect("load aud010 system log rows");
    assert_eq!(system_log_rows.len(), 2);

    cleanup_searchrec_dead_letter(&client, &search_dead_letter)
        .await
        .expect("cleanup search dead letter");
    cleanup_searchrec_dead_letter(&client, &recommendation_dead_letter)
        .await
        .expect("cleanup recommendation dead letter");
}

#[tokio::test]
async fn audit_consistency_lookup_db_smoke() {
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
    let seed = seed_order_graph(&client, &format!("aud011-{suffix}"))
        .await
        .expect("seed order graph");
    let operator_user_id = seed_user(&client, &seed.buyer_org_id, &format!("aud011-{suffix}"))
        .await
        .expect("seed aud011 operator");
    let app = crate::with_live_test_state(router()).await;
    let consistency_request_id = format!("req-aud011-consistency-{suffix}");
    let consistency_trace_id = format!("trace-aud011-consistency-{suffix}");
    let seed_request_id = format!("req-aud011-seed-{suffix}");
    let seed_trace_id = format!("trace-aud011-seed-{suffix}");

    client
        .execute(
            "UPDATE trade.order_main
             SET authority_model = 'dual_layer',
                 business_state_version = 7,
                 proof_commit_state = 'pending_anchor',
                 proof_commit_policy = 'async_evidence',
                 external_fact_status = 'confirmed',
                 reconcile_status = 'pending_check',
                 last_reconciled_at = now() - interval '3 minutes'
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update consistency fields");

    let outbox_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud011 outbox id")
        .get(0);
    let external_fact_receipt_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud011 external fact id")
        .get(0);
    let chain_projection_gap_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud011 projection gap id")
        .get(0);
    let chain_anchor_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud011 chain anchor id")
        .get(0);

    client
        .execute(
            "INSERT INTO ops.outbox_event (
               outbox_event_id,
               aggregate_type,
               aggregate_id,
               event_type,
               payload,
               status,
               retry_count,
               max_retries,
               available_at,
               published_at,
               created_at,
               event_schema_version,
               request_id,
               trace_id,
               idempotency_key,
               authority_scope,
               source_of_truth,
               proof_commit_policy,
               target_bus,
               target_topic,
               partition_key,
               ordering_key,
               payload_hash
             ) VALUES (
               $1::text::uuid,
               'order',
               $2::text::uuid,
               'fabric.proof_submit_requested',
               jsonb_build_object(
                 'event_id', $1,
                 'event_type', 'fabric.proof_submit_requested',
                 'aggregate_type', 'order',
                 'aggregate_id', $2,
                 'request_id', $3,
                 'trace_id', $4,
                 'payload', jsonb_build_object('seed', $5)
               ),
               'published',
               0,
               16,
               now() - interval '4 minutes',
               now() - interval '3 minutes',
               now() - interval '5 minutes',
               'v1',
               $3,
               $4,
               $6,
               'business',
               'database',
               'async_evidence',
               'kafka',
               'dtp.fabric.requests',
               $2,
               $2,
               $7
             )",
            &[
                &outbox_event_id,
                &seed.order_id,
                &seed_request_id,
                &seed_trace_id,
                &suffix,
                &format!("aud011-idempotency-{suffix}"),
                &format!("aud011-payload-hash-{suffix}"),
            ],
        )
        .await
        .expect("insert aud011 outbox event");

    client
        .execute(
            "INSERT INTO ops.outbox_publish_attempt (
               outbox_event_id,
               worker_id,
               target_bus,
               target_topic,
               attempt_no,
               result_code,
               attempted_at,
               completed_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               'outbox-publisher-local',
               'kafka',
               'dtp.fabric.requests',
               1,
               'published',
               now() - interval '4 minutes',
               now() - interval '4 minutes' + interval '1 second',
               jsonb_build_object('seed', $2, 'worker', 'outbox-publisher-local')
             )",
            &[&outbox_event_id, &suffix],
        )
        .await
        .expect("insert aud011 outbox publish attempt");

    client
        .execute(
            "INSERT INTO ops.dead_letter_event (
               dead_letter_event_id,
               outbox_event_id,
               aggregate_type,
               aggregate_id,
               event_type,
               payload,
               failed_reason,
               created_at,
               request_id,
               trace_id,
               authority_scope,
               source_of_truth,
               target_bus,
               target_topic,
               failure_stage,
               first_failed_at,
               last_failed_at,
               reprocess_status
             ) VALUES (
               gen_random_uuid(),
               $1::text::uuid,
               'order',
               $2::text::uuid,
               'fabric.proof_submit_requested',
               jsonb_build_object('event_id', $1, 'seed', $3, 'target_topic', 'dtp.fabric.requests'),
               'fabric callback projection gap isolated after repeated retry',
               now() - interval '2 minutes',
               $4,
               $5,
               'business',
               'database',
               'kafka',
               'dtp.fabric.requests',
               'consumer_handler',
               now() - interval '125 seconds',
               now() - interval '120 seconds',
               'not_reprocessed'
             )",
            &[
                &outbox_event_id,
                &seed.order_id,
                &suffix,
                &seed_request_id,
                &seed_trace_id,
            ],
        )
        .await
        .expect("insert aud011 dead letter");

    client
        .execute(
            "INSERT INTO ops.consumer_idempotency_record (
               consumer_name,
               event_id,
               aggregate_type,
               aggregate_id,
               trace_id,
               result_code,
               metadata
             ) VALUES (
               'platform-core.consistency',
               $1::text::uuid,
               'order',
               $2::text::uuid,
               $3,
               'dead_lettered',
               jsonb_build_object('seed', $4, 'dlq_topic', 'dtp.dead-letter')
             )",
            &[&outbox_event_id, &seed.order_id, &seed_trace_id, &suffix],
        )
        .await
        .expect("insert aud011 consumer idempotency");

    client
        .execute(
            "INSERT INTO ops.external_fact_receipt (
               external_fact_receipt_id,
               order_id,
               ref_domain,
               ref_type,
               ref_id,
               fact_type,
               provider_type,
               provider_key,
               provider_reference,
               receipt_status,
               receipt_payload,
               receipt_hash,
               occurred_at,
               received_at,
               confirmed_at,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'payment',
               'order',
               $2::text::uuid,
               'payment_callback',
               'mock_payment_provider',
               'mockpay',
               $3,
               'confirmed',
               jsonb_build_object('seed', $4, 'provider_status', 'confirmed'),
               $5,
               now() - interval '4 minutes',
               now() - interval '4 minutes' + interval '3 seconds',
               now() - interval '4 minutes' + interval '10 seconds',
               $6,
               $7,
               jsonb_build_object('seed', $4)
             )",
            &[
                &external_fact_receipt_id,
                &seed.order_id,
                &format!("provider-ref-aud011-{suffix}"),
                &suffix,
                &format!("aud011-receipt-hash-{suffix}"),
                &seed_request_id,
                &seed_trace_id,
            ],
        )
        .await
        .expect("insert aud011 external fact receipt");

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
               anchored_at,
               created_at,
               authority_model,
               reconcile_status,
               last_reconciled_at
             ) VALUES (
               $1::text::uuid,
               'fabric-local',
               'order_proof',
               'order',
               $2::text::uuid,
               $3,
               '0xaud011anchor',
               'anchored',
               now() - interval '3 minutes',
               now() - interval '3 minutes' - interval '10 seconds',
               'proof_layer',
               'matched',
               now() - interval '2 minutes'
             )",
            &[
                &chain_anchor_id,
                &seed.order_id,
                &format!("aud011-digest-{suffix}"),
            ],
        )
        .await
        .expect("insert aud011 chain anchor");

    client
        .execute(
            "INSERT INTO ops.chain_projection_gap (
               chain_projection_gap_id,
               aggregate_type,
               aggregate_id,
               order_id,
               chain_id,
               source_event_type,
               expected_tx_id,
               projected_tx_hash,
               gap_type,
               gap_status,
               first_detected_at,
               last_detected_at,
               request_id,
               trace_id,
               outbox_event_id,
               anchor_id,
               resolution_summary,
               metadata
             ) VALUES (
               $1::text::uuid,
               'order',
               $2::text::uuid,
               $2::text::uuid,
               'fabric-local',
               'fabric.proof_submit_requested',
               $3,
               '0xaud011anchor',
               'missing_callback',
               'open',
               now() - interval '100 seconds',
               now() - interval '90 seconds',
               $4,
               $5,
               $6::text::uuid,
               $7::text::uuid,
               jsonb_build_object('recommendation', 'trigger reconcile dry-run'),
               jsonb_build_object('seed', $8)
             )",
            &[
                &chain_projection_gap_id,
                &seed.order_id,
                &format!("expected-tx-aud011-{suffix}"),
                &seed_request_id,
                &seed_trace_id,
                &outbox_event_id,
                &chain_anchor_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud011 projection gap");

    write_trade_audit_event(
        &client,
        "order",
        &seed.order_id,
        "platform_audit_security",
        "trade.order.consistency_seeded",
        "accepted",
        Some(&seed_request_id),
        Some(&seed_trace_id),
    )
    .await
    .expect("write aud011 audit trace");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/ops/consistency/order/{}", seed.order_id))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &consistency_request_id)
                .header("x-trace-id", &consistency_trace_id)
                .body(Body::empty())
                .expect("consistency request"),
        )
        .await
        .expect("call ops consistency");
    let response_status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read consistency body");
    assert_eq!(
        response_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let response_json: Value =
        serde_json::from_slice(&response_body).expect("decode consistency response");
    assert_eq!(response_json["data"]["ref_type"].as_str(), Some("order"));
    assert_eq!(
        response_json["data"]["ref_id"].as_str(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        response_json["data"]["business_state"]["business_status"].as_str(),
        Some("buyer_locked")
    );
    assert_eq!(
        response_json["data"]["business_state"]["proof_commit_state"].as_str(),
        Some("pending_anchor")
    );
    assert_eq!(
        response_json["data"]["business_state"]["external_fact_status"].as_str(),
        Some("confirmed")
    );
    assert_eq!(
        response_json["data"]["proof_state"]["latest_chain_anchor"]["status"].as_str(),
        Some("anchored")
    );
    assert_eq!(
        response_json["data"]["proof_state"]["open_projection_gap_count"].as_i64(),
        Some(1)
    );
    assert_eq!(
        response_json["data"]["proof_state"]["latest_projection_gap"]["chain_projection_gap_id"]
            .as_str(),
        Some(chain_projection_gap_id.as_str())
    );
    assert_eq!(
        response_json["data"]["external_fact_state"]["total_receipts"].as_i64(),
        Some(1)
    );
    assert_eq!(
        response_json["data"]["external_fact_state"]["latest_receipt"]["external_fact_receipt_id"]
            .as_str(),
        Some(external_fact_receipt_id.as_str())
    );
    assert_eq!(
        response_json["data"]["recent_outbox_events"][0]["outbox_event_id"].as_str(),
        Some(outbox_event_id.as_str())
    );
    assert_eq!(
        response_json["data"]["recent_dead_letters"][0]["outbox_event_id"].as_str(),
        Some(outbox_event_id.as_str())
    );
    assert_eq!(
        response_json["data"]["recent_audit_traces"][0]["ref_type"].as_str(),
        Some("order")
    );

    let access_row = client
        .query_one(
            "SELECT access_mode, target_type, target_id::text
             FROM audit.access_audit
             WHERE request_id = $1",
            &[&consistency_request_id],
        )
        .await
        .expect("load aud011 access audit");
    assert_eq!(access_row.get::<_, String>(0), "masked");
    assert_eq!(access_row.get::<_, String>(1), "consistency_query");
    assert_eq!(
        access_row.get::<_, Option<String>>(2).as_deref(),
        Some(seed.order_id.as_str())
    );

    let system_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = 'ops lookup executed: GET /api/v1/ops/consistency/{refType}/{refId}'",
            &[&consistency_request_id],
        )
        .await
        .expect("load aud011 system log count")
        .get(0);
    assert_eq!(system_log_count, 1);

    let _ = client
        .execute(
            "DELETE FROM ops.chain_projection_gap WHERE chain_projection_gap_id = $1::text::uuid",
            &[&chain_projection_gap_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.external_fact_receipt WHERE external_fact_receipt_id = $1::text::uuid",
            &[&external_fact_receipt_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.consumer_idempotency_record WHERE event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.dead_letter_event WHERE outbox_event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.outbox_publish_attempt WHERE outbox_event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.outbox_event WHERE outbox_event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM chain.chain_anchor WHERE chain_anchor_id = $1::text::uuid",
            &[&chain_anchor_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
            &[&operator_user_id],
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

#[derive(Debug)]
struct SeededSearchrecDeadLetter {
    dead_letter_event_id: String,
    outbox_event_id: String,
}

async fn seed_searchrec_dead_letter(
    client: &Client,
    suffix: &str,
    target_topic: &str,
    consumer_name: &str,
    event_type: &str,
    aggregate_type: &str,
) -> Result<SeededSearchrecDeadLetter, Error> {
    let dead_letter_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await?
        .get(0);
    let outbox_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await?
        .get(0);
    let aggregate_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await?
        .get(0);
    let trace_id = format!("trace-aud010-seed-{suffix}");
    let request_id = format!("req-aud010-seed-{suffix}");

    client
        .execute(
            "INSERT INTO ops.dead_letter_event (
               dead_letter_event_id,
               outbox_event_id,
               aggregate_type,
               aggregate_id,
               event_type,
               payload,
               failed_reason,
               request_id,
               trace_id,
               authority_scope,
               source_of_truth,
               target_bus,
               target_topic,
               failure_stage,
               first_failed_at,
               last_failed_at,
               reprocess_status
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3,
               $4::text::uuid,
               $5,
               jsonb_build_object(
                 'event_id', $2,
                 'event_type', $5,
                 'target_topic', $6,
                 'seed', $7
               ),
               $8,
               $9,
               $10,
               'business',
               'database',
               'kafka',
               $6,
               'consumer_handler',
               now() - interval '2 minutes',
               now() - interval '1 minute',
               'not_reprocessed'
             )",
            &[
                &dead_letter_event_id,
                &outbox_event_id,
                &aggregate_type,
                &aggregate_id,
                &event_type,
                &target_topic,
                &suffix,
                &format!("{consumer_name} failed while handling replay candidate"),
                &request_id,
                &trace_id,
            ],
        )
        .await?;

    client
        .execute(
            "INSERT INTO ops.consumer_idempotency_record (
               consumer_name,
               event_id,
               aggregate_type,
               aggregate_id,
               trace_id,
               result_code,
               metadata
             ) VALUES (
               $1,
               $2::text::uuid,
               $3,
               $4::text::uuid,
               $5,
               'dead_lettered',
               jsonb_build_object('seed', $6, 'target_topic', $7)
             )",
            &[
                &consumer_name,
                &outbox_event_id,
                &aggregate_type,
                &aggregate_id,
                &trace_id,
                &suffix,
                &target_topic,
            ],
        )
        .await?;

    Ok(SeededSearchrecDeadLetter {
        dead_letter_event_id,
        outbox_event_id,
    })
}

async fn cleanup_searchrec_dead_letter(
    client: &Client,
    seed: &SeededSearchrecDeadLetter,
) -> Result<(), Error> {
    client
        .execute(
            "DELETE FROM ops.consumer_idempotency_record
             WHERE event_id = $1::text::uuid",
            &[&seed.outbox_event_id],
        )
        .await?;
    client
        .execute(
            "DELETE FROM ops.dead_letter_event
             WHERE dead_letter_event_id = $1::text::uuid",
            &[&seed.dead_letter_event_id],
        )
        .await?;
    Ok(())
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
