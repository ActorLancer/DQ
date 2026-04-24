use crate::modules::audit::api::router;
use crate::modules::audit::application::{EvidenceWriteCommand, record_evidence_snapshot};
use crate::modules::audit::domain::{
    ChainProjectionGapQuery, ConsumerIdempotencyQuery, ExternalFactReceiptQuery,
};
use crate::modules::audit::repo;
use crate::modules::order::repo::write_trade_audit_event;
use crate::modules::storage::application::{delete_object, fetch_object_bytes, put_object_bytes};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
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

    #[tokio::test]
    async fn rejects_ops_consistency_reconcile_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/consistency/reconcile")
            .header("x-role", "buyer_operator")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud012-reconcile-forbidden")
            .header("x-step-up-token", "aud012-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"ref_type":"order","ref_id":"10000000-0000-0000-0000-000000000011","reason":"forbidden"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn ops_consistency_reconcile_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/consistency/reconcile")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud012-reconcile-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"ref_type":"order","ref_id":"10000000-0000-0000-0000-000000000011","reason":"missing step-up"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn ops_consistency_reconcile_enforces_dry_run_only() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/consistency/reconcile")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud012-reconcile-dry-run")
            .header("x-step-up-token", "aud012-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"ref_type":"order","ref_id":"10000000-0000-0000-0000-000000000011","mode":"full","reason":"dry-run only","dry_run":false}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn rejects_trade_monitor_overview_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/trade-monitor/orders/10000000-0000-0000-0000-000000000018")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud018-trade-monitor-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn trade_monitor_checkpoints_requires_request_id() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri(
                "/api/v1/ops/trade-monitor/orders/10000000-0000-0000-0000-000000000018/checkpoints",
            )
            .header("x-role", "platform_audit_security")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_external_fact_query_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/external-facts")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud019-external-facts-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn external_fact_confirm_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/external-facts/10000000-0000-0000-0000-000000000019/confirm")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud019-external-fact-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"confirm_result":"confirmed","reason":"missing step-up"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_fairness_incident_query_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/fairness-incidents")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud020-fairness-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn fairness_incident_handle_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/fairness-incidents/10000000-0000-0000-0000-000000000020/handle")
            .header("x-role", "platform_risk_settlement")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud020-fairness-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"action":"close","resolution_summary":"missing step-up"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_projection_gap_query_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/projection-gaps")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud021-projection-gap-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn projection_gap_resolve_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/projection-gaps/10000000-0000-0000-0000-000000000021/resolve")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud021-projection-gap-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"reason":"missing step-up","resolution_mode":"manual_close"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_observability_overview_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/observability/overview")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud023-observability-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn ops_logs_query_requires_request_id() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/logs/query")
            .header("x-role", "platform_audit_security")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn ops_logs_export_requires_step_up() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/logs/export")
            .header("x-role", "platform_audit_security")
            .header("x-user-id", "10000000-0000-0000-0000-000000000304")
            .header("x-request-id", "req-aud023-log-export-missing-stepup")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"reason":"missing step-up","trace_id":"trace-aud023"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_ops_trace_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/traces/trace-aud023")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud023-trace-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_ops_alerts_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/alerts")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud023-alerts-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_ops_incidents_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/incidents")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud023-incidents-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_ops_slos_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/slos")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud023-slos-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_developer_trace_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/developer/trace?order_id=10000000-0000-0000-0000-000000000001")
            .header("x-role", "buyer_operator")
            .header("x-request-id", "req-aud024-forbidden")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn developer_trace_requires_single_selector() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/developer/trace")
            .header("x-role", "tenant_developer")
            .header("x-request-id", "req-aud024-missing-selector")
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
    assert_eq!(
        replay_lookup_json["data"]["results"]
            .as_array()
            .map(|items| items.len()),
        Some(4)
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
    assert_eq!(replay_report_json["dry_run"].as_bool(), Some(true));
    let replay_trace_total = replay_report_json["counts"]["audit_trace_total"]
        .as_i64()
        .expect("replay trace total");
    assert!(replay_trace_total >= 3);
    assert_eq!(
        replay_report_json["results"][0]["step_name"].as_str(),
        Some("target_snapshot")
    );
    assert_eq!(
        replay_report_json["results"][0]["diff_summary"]["target"]["order_id"].as_str(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        replay_report_json["results"][0]["diff_summary"]["target"]["payment_status"].as_str(),
        Some("paid")
    );
    assert_eq!(
        replay_report_json["results"][0]["diff_summary"]["target"]["delivery_status"].as_str(),
        Some("pending_delivery")
    );
    assert_eq!(
        replay_report_json["results"][0]["diff_summary"]["target"]["settlement_status"].as_str(),
        Some("pending_settlement")
    );
    assert_eq!(
        replay_report_json["results"][1]["step_name"].as_str(),
        Some("audit_timeline")
    );
    assert_eq!(
        replay_report_json["results"][1]["diff_summary"]["trace_total"].as_i64(),
        Some(replay_trace_total)
    );
    let replay_trace_preview = replay_report_json["results"][1]["diff_summary"]["preview"]
        .as_array()
        .expect("replay trace preview");
    assert!(!replay_trace_preview.is_empty());
    assert_eq!(replay_trace_preview.len() as i64, replay_trace_total);
    let replay_trace_actions = replay_trace_preview
        .iter()
        .filter_map(|item| {
            item.get("action_name")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    assert!(replay_trace_actions.contains(&"trade.order.create".to_string()));
    assert!(replay_trace_actions.contains(&"trade.order.lock".to_string()));
    assert!(replay_trace_actions.contains(&"audit.package.export".to_string()));
    assert_eq!(
        replay_report_json["results"][2]["step_name"].as_str(),
        Some("evidence_projection")
    );
    assert!(
        replay_report_json["results"][2]["diff_summary"]["manifest_count"]
            .as_i64()
            .unwrap_or_default()
            >= 1
    );
    assert!(
        replay_report_json["results"][2]["diff_summary"]["item_count"]
            .as_i64()
            .unwrap_or_default()
            >= 1
    );
    assert_eq!(
        replay_report_json["results"][2]["diff_summary"]["legal_hold_status"].as_str(),
        Some("none")
    );
    assert_eq!(
        replay_report_json["results"][3]["step_name"].as_str(),
        Some("execution_policy")
    );
    assert_eq!(
        replay_report_json["results"][3]["diff_summary"]["dry_run"].as_bool(),
        Some(true)
    );
    assert_eq!(
        replay_report_json["results"][3]["diff_summary"]["side_effects_executed"].as_bool(),
        Some(false)
    );
    assert_eq!(
        replay_report_json["results"][3]["diff_summary"]["recommendation"].as_str(),
        Some("dry_run_completed")
    );

    let replay_result_rows = client
        .query(
            "SELECT step_name, result_code, diff_summary
             FROM audit.replay_result
             WHERE replay_job_id = $1::text::uuid
             ORDER BY created_at, replay_result_id",
            &[&replay_job_id],
        )
        .await
        .expect("query replay results");
    assert_eq!(replay_result_rows.len(), 4);
    let mut replay_result_by_step = HashMap::new();
    for row in replay_result_rows.iter() {
        let inserted = replay_result_by_step.insert(
            row.get::<_, String>(0),
            (row.get::<_, String>(1), row.get::<_, Value>(2)),
        );
        assert!(inserted.is_none());
    }
    assert!(replay_result_by_step.contains_key("target_snapshot"));
    assert!(replay_result_by_step.contains_key("audit_timeline"));
    assert!(replay_result_by_step.contains_key("evidence_projection"));
    assert!(replay_result_by_step.contains_key("execution_policy"));
    assert_eq!(
        replay_result_by_step["target_snapshot"].0,
        "loaded".to_string()
    );
    assert_eq!(
        replay_result_by_step["audit_timeline"].0,
        "ready".to_string()
    );
    assert_eq!(
        replay_result_by_step["evidence_projection"].0,
        "ready".to_string()
    );
    assert_eq!(
        replay_result_by_step["execution_policy"].0,
        "AUDIT_REPLAY_DRY_RUN_ONLY".to_string()
    );
    let target_diff = replay_result_by_step["target_snapshot"].1.clone();
    assert_eq!(
        target_diff["target"]["order_id"].as_str(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        target_diff["target"]["settlement_status"].as_str(),
        Some("pending_settlement")
    );
    let timeline_diff = replay_result_by_step["audit_timeline"].1.clone();
    assert_eq!(
        timeline_diff["trace_total"].as_i64(),
        Some(replay_trace_total)
    );
    let timeline_preview = timeline_diff["preview"]
        .as_array()
        .expect("timeline preview diff");
    assert_eq!(timeline_preview.len() as i64, replay_trace_total);
    let evidence_diff = replay_result_by_step["evidence_projection"].1.clone();
    assert_eq!(evidence_diff["legal_hold_status"].as_str(), Some("none"));
    let execution_policy_diff = replay_result_by_step["execution_policy"].1.clone();
    assert_eq!(
        execution_policy_diff["side_effects_executed"].as_bool(),
        Some(false)
    );
    assert_eq!(
        execution_policy_diff["recommendation"].as_str(),
        Some("dry_run_completed")
    );

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
                    status,
                    payload
             FROM ops.outbox_event
             WHERE request_id = $1
             ORDER BY created_at DESC, outbox_event_id DESC
             LIMIT 1",
            &[&anchor_retry_request_id],
        )
        .await
        .expect("query anchor retry outbox row");
    let anchor_route_row = client
        .query_one(
            "SELECT target_topic, authority_scope, proof_commit_policy
             FROM ops.event_route_policy
             WHERE aggregate_type = 'audit.anchor_batch'
               AND event_type = 'audit.anchor_requested'
               AND status = 'active'
             ORDER BY updated_at DESC, created_at DESC
             LIMIT 1",
            &[],
        )
        .await
        .expect("query anchor route policy");
    let anchor_route_target_topic: String = anchor_route_row.get(0);
    let anchor_route_authority_scope: String = anchor_route_row.get(1);
    let anchor_route_proof_commit_policy: String = anchor_route_row.get(2);
    let anchor_outbox_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.outbox_event
             WHERE request_id = $1
               AND event_type = 'audit.anchor_requested'
               AND aggregate_type = 'audit.anchor_batch'",
            &[&anchor_retry_request_id],
        )
        .await
        .expect("count anchor retry outbox rows")
        .get(0);
    assert_eq!(anchor_outbox_count, 1);
    assert_eq!(
        anchor_outbox_row.get::<_, String>(0),
        anchor_route_target_topic
    );
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
    let anchor_outbox_payload: Value = anchor_outbox_row.get(8);
    assert_eq!(anchor_outbox_payload["event_version"].as_i64(), Some(1));
    assert_eq!(
        anchor_outbox_payload["event_schema_version"].as_str(),
        Some("v1")
    );
    assert_eq!(
        anchor_outbox_payload["authority_scope"].as_str(),
        Some(anchor_route_authority_scope.as_str())
    );
    assert_eq!(
        anchor_outbox_payload["source_of_truth"].as_str(),
        Some("database")
    );
    assert_eq!(
        anchor_outbox_payload["proof_commit_policy"].as_str(),
        Some(anchor_route_proof_commit_policy.as_str())
    );
    assert_eq!(
        anchor_outbox_payload["request_id"].as_str(),
        Some(anchor_retry_request_id.as_str())
    );
    assert_eq!(
        anchor_outbox_payload["trace_id"].as_str(),
        Some(trace_id.as_str())
    );
    assert!(anchor_outbox_payload.get("event_name").is_none());

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
async fn developer_trace_api_db_smoke() {
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
    let seed = seed_order_graph(&client, &format!("aud024-{suffix}"))
        .await
        .expect("seed order graph");
    let operator_user_id = seed_user(&client, &seed.buyer_org_id, &format!("aud024-{suffix}"))
        .await
        .expect("seed aud024 user");
    let app = crate::with_live_test_state(router()).await;

    let order_request_id = format!("req-aud024-order-{suffix}");
    let event_request_id = format!("req-aud024-event-{suffix}");
    let tx_request_id = format!("req-aud024-tx-{suffix}");
    let seed_request_id = format!("req-aud024-seed-{suffix}");
    let trace_id = format!("trace-aud024-{suffix}");
    let tx_hash = format!("0xaud024{suffix}");
    let outbox_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud024 outbox id")
        .get(0);
    let dead_letter_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud024 dead letter id")
        .get(0);
    let chain_anchor_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud024 chain anchor id")
        .get(0);
    let chain_projection_gap_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud024 projection gap id")
        .get(0);
    let checkpoint_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud024 checkpoint id")
        .get(0);
    let external_fact_receipt_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud024 external fact id")
        .get(0);
    let trace_index_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud024 trace index id")
        .get(0);
    let system_log_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud024 system log id")
        .get(0);

    client
        .execute(
            "UPDATE trade.order_main
             SET authority_model = 'dual_layer',
                 business_state_version = 9,
                 proof_commit_state = 'anchored',
                 proof_commit_policy = 'async_evidence',
                 external_fact_status = 'confirmed',
                 reconcile_status = 'matched',
                 last_reconciled_at = now() - interval '2 minutes'
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update aud024 order");

    write_trade_audit_event(
        &client,
        "order",
        &seed.order_id,
        "tenant_developer",
        "trade.order.debug.lookup",
        "accepted",
        Some(&seed_request_id),
        Some(&trace_id),
    )
    .await
    .expect("write aud024 trade audit");

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
                 'order_id', $2,
                 'request_id', $3,
                 'trace_id', $4
               ),
               'published',
               0,
               8,
               now() - interval '5 minutes',
               now() - interval '4 minutes',
               now() - interval '5 minutes',
               'v1',
               $3,
               $4,
               $5,
               'business',
               'database',
               'async_evidence',
               'kafka',
               'dtp.fabric.requests',
               $2,
               $2,
               $6
             )",
            &[
                &outbox_event_id,
                &seed.order_id,
                &seed_request_id,
                &trace_id,
                &format!("aud024-idempotency-{suffix}"),
                &format!("aud024-payload-hash-{suffix}"),
            ],
        )
        .await
        .expect("insert aud024 outbox");

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
               'order',
               $3::text::uuid,
               'fabric.proof_submit_requested',
               jsonb_build_object('seed', $4),
               'temporary broker outage',
               $5,
               $6,
               'business',
               'database',
               'kafka',
               'dtp.fabric.requests',
               'consumer_handler',
               now() - interval '4 minutes',
               now() - interval '3 minutes',
               'not_reprocessed'
             )",
            &[
                &dead_letter_event_id,
                &outbox_event_id,
                &seed.order_id,
                &suffix,
                &seed_request_id,
                &trace_id,
            ],
        )
        .await
        .expect("insert aud024 dead letter");

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
               'fabric',
               'order',
               $2::text::uuid,
               'fabric_submit_receipt',
               'fabric_gateway',
               'fabric-local',
               $3,
               'confirmed',
               jsonb_build_object('seed', $4),
               $5,
               now() - interval '3 minutes',
               now() - interval '3 minutes',
               now() - interval '2 minutes',
               $6,
               $7,
               jsonb_build_object('seed', $4, 'tx_hash', $8)
             )",
            &[
                &external_fact_receipt_id,
                &seed.order_id,
                &format!("aud024-provider-ref-{suffix}"),
                &suffix,
                &format!("aud024-receipt-hash-{suffix}"),
                &seed_request_id,
                &trace_id,
                &tx_hash,
            ],
        )
        .await
        .expect("insert aud024 external fact");

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
               $4,
               'anchored',
               now() - interval '2 minutes',
               now() - interval '2 minutes' - interval '5 seconds',
               'proof_layer',
               'matched',
               now() - interval '1 minute'
             )",
            &[
                &chain_anchor_id,
                &seed.order_id,
                &format!("aud024-anchor-digest-{suffix}"),
                &tx_hash,
            ],
        )
        .await
        .expect("insert aud024 chain anchor");

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
               'fabric.commit_confirmed',
               $3,
               $4,
               'projection_lag',
               'open',
               now() - interval '2 minutes',
               now() - interval '1 minute',
               $5,
               $6,
               $7::text::uuid,
               $8::text::uuid,
               jsonb_build_object('seed', $9),
               jsonb_build_object('seed', $9)
             )",
            &[
                &chain_projection_gap_id,
                &seed.order_id,
                &format!("aud024-expected-tx-{suffix}"),
                &tx_hash,
                &seed_request_id,
                &trace_id,
                &outbox_event_id,
                &chain_anchor_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud024 projection gap");

    client
        .execute(
            "INSERT INTO ops.trade_lifecycle_checkpoint (
               trade_lifecycle_checkpoint_id,
               order_id,
               ref_domain,
               ref_type,
               ref_id,
               checkpoint_code,
               lifecycle_stage,
               checkpoint_status,
               occurred_at,
               source_type,
               related_tx_hash,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'trade',
               'order',
               $2::text::uuid,
               'proof_committed',
               'settlement',
               'completed',
               now() - interval '90 seconds',
               'system',
               $3,
               $4,
               $5,
               jsonb_build_object('seed', $6)
             )",
            &[
                &checkpoint_id,
                &seed.order_id,
                &tx_hash,
                &seed_request_id,
                &trace_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud024 checkpoint");

    client
        .execute(
            "INSERT INTO ops.trace_index (
               trace_index_id,
               trace_id,
               traceparent,
               backend_key,
               root_service_name,
               root_span_name,
               request_id,
               ref_type,
               ref_id,
               object_type,
               object_id,
               status,
               span_count,
               started_at,
               ended_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               '00-aud024trace-00000000000000000000000000000000-0000000000000000-01',
               'tempo_main',
               'platform-core',
               'GET /api/v1/developer/trace',
               $3,
               'order',
               $4::text::uuid,
               'order',
               $4::text::uuid,
               'ok',
               4,
               now() - interval '2 minutes',
               now() - interval '1 minute',
               jsonb_build_object('seed', 'aud024')
             )",
            &[&trace_index_id, &trace_id, &seed_request_id, &seed.order_id],
        )
        .await
        .expect("insert aud024 trace index");

    client
        .execute(
            "INSERT INTO ops.system_log (
               system_log_id,
               service_name,
               logger_name,
               log_level,
               request_id,
               trace_id,
               message_text,
               structured_payload,
               environment_code,
               backend_type,
               severity_number,
               object_type,
               object_id,
               masked_status,
               retention_class,
               legal_hold_status,
               resource_attrs
             ) VALUES (
               $1::text::uuid,
               'platform-core',
               'developer.trace',
               'INFO',
               $2,
               $3,
               $4,
               $5::jsonb,
               'local',
               'database_mirror',
               9,
               'order',
               $6::text::uuid,
               'masked',
               'ops_default',
               'none',
               '{}'::jsonb
             )",
            &[
                &system_log_id,
                &seed_request_id,
                &trace_id,
                &format!("aud024 seed developer trace log {suffix}"),
                &json!({"seed":"aud024","tx_hash":tx_hash}),
                &seed.order_id,
            ],
        )
        .await
        .expect("insert aud024 system log");

    let order_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/developer/trace?order_id={}",
                    seed.order_id
                ))
                .header("x-role", "tenant_developer")
                .header("x-user-id", &operator_user_id)
                .header("x-tenant-id", &seed.buyer_org_id)
                .header("x-request-id", &order_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud024 order request"),
        )
        .await
        .expect("call aud024 order lookup");
    assert_eq!(order_response.status(), StatusCode::OK);
    let order_body = to_bytes(order_response.into_body(), usize::MAX)
        .await
        .expect("read aud024 order body");
    let order_json: Value = serde_json::from_slice(&order_body).expect("decode aud024 order");
    assert_eq!(
        order_json["data"]["subject"]["resolved_order_id"].as_str(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        order_json["data"]["subject"]["payment_status"].as_str(),
        Some("paid")
    );
    assert!(
        order_json["data"]["recent_logs"]
            .as_array()
            .expect("aud024 recent logs")
            .len()
            >= 1
    );

    let event_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/developer/trace?event_id={outbox_event_id}"
                ))
                .header("x-role", "tenant_developer")
                .header("x-user-id", &operator_user_id)
                .header("x-tenant-id", &seed.buyer_org_id)
                .header("x-request-id", &event_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud024 event request"),
        )
        .await
        .expect("call aud024 event lookup");
    assert_eq!(event_response.status(), StatusCode::OK);
    let event_body = to_bytes(event_response.into_body(), usize::MAX)
        .await
        .expect("read aud024 event body");
    let event_json: Value = serde_json::from_slice(&event_body).expect("decode aud024 event");
    assert_eq!(
        event_json["data"]["matched_outbox_event"]["outbox_event_id"].as_str(),
        Some(outbox_event_id.as_str())
    );
    assert_eq!(
        event_json["data"]["matched_dead_letter"]["dead_letter_event_id"].as_str(),
        Some(dead_letter_event_id.as_str())
    );
    assert_eq!(
        event_json["data"]["trace"]["backend_key"].as_str(),
        Some("tempo_main")
    );

    let tx_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/developer/trace?tx_hash={tx_hash}"))
                .header("x-role", "tenant_developer")
                .header("x-user-id", &operator_user_id)
                .header("x-tenant-id", &seed.buyer_org_id)
                .header("x-request-id", &tx_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud024 tx request"),
        )
        .await
        .expect("call aud024 tx lookup");
    assert_eq!(tx_response.status(), StatusCode::OK);
    let tx_body = to_bytes(tx_response.into_body(), usize::MAX)
        .await
        .expect("read aud024 tx body");
    let tx_json: Value = serde_json::from_slice(&tx_body).expect("decode aud024 tx");
    assert_eq!(
        tx_json["data"]["matched_chain_anchor"]["chain_anchor_id"].as_str(),
        Some(chain_anchor_id.as_str())
    );
    assert_eq!(
        tx_json["data"]["subject"]["trace_id"].as_str(),
        Some(trace_id.as_str())
    );

    let access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND target_type = 'developer_trace_query'",
            &[&vec![
                order_request_id.clone(),
                event_request_id.clone(),
                tx_request_id.clone(),
            ]],
        )
        .await
        .expect("count aud024 access audit")
        .get(0);
    assert_eq!(access_count, 3);

    let system_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = 'developer trace lookup executed: GET /api/v1/developer/trace'",
            &[&vec![
                order_request_id.clone(),
                event_request_id.clone(),
                tx_request_id.clone(),
            ]],
        )
        .await
        .expect("count aud024 system logs")
        .get(0);
    assert_eq!(system_log_count, 3);

    let _ = client
        .execute(
            "DELETE FROM ops.trace_index WHERE trace_index_id = $1::text::uuid",
            &[&trace_index_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.trade_lifecycle_checkpoint WHERE trade_lifecycle_checkpoint_id = $1::text::uuid",
            &[&checkpoint_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.chain_projection_gap WHERE chain_projection_gap_id = $1::text::uuid",
            &[&chain_projection_gap_id],
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
            "DELETE FROM ops.external_fact_receipt WHERE external_fact_receipt_id = $1::text::uuid",
            &[&external_fact_receipt_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.dead_letter_event WHERE dead_letter_event_id = $1::text::uuid",
            &[&dead_letter_event_id],
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
            "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
            &[&operator_user_id],
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

#[tokio::test]
async fn audit_consistency_reconcile_db_smoke() {
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
    let seed = seed_order_graph(&client, &format!("aud012-{suffix}"))
        .await
        .expect("seed order graph");
    let operator_user_id = seed_user(&client, &seed.buyer_org_id, &format!("aud012-{suffix}"))
        .await
        .expect("seed aud012 operator");
    let reconcile_step_up_id = seed_verified_step_up_challenge(
        &client,
        &operator_user_id,
        "ops.consistency.reconcile",
        "order",
        Some(seed.order_id.as_str()),
        "aud012",
    )
    .await
    .expect("seed aud012 reconcile step-up challenge");
    let app = crate::with_live_test_state(router()).await;
    let reconcile_request_id = format!("req-aud012-reconcile-{suffix}");
    let reconcile_trace_id = format!("trace-aud012-reconcile-{suffix}");
    let seed_request_id = format!("req-aud012-seed-{suffix}");
    let seed_trace_id = format!("trace-aud012-seed-{suffix}");

    client
        .execute(
            "UPDATE trade.order_main
             SET authority_model = 'dual_layer',
                 business_state_version = 8,
                 proof_commit_state = 'pending_anchor',
                 proof_commit_policy = 'async_evidence',
                 external_fact_status = 'pending_receipt',
                 reconcile_status = 'pending_check',
                 last_reconciled_at = now() - interval '5 minutes'
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update aud012 consistency fields");

    let outbox_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud012 outbox id")
        .get(0);
    let chain_projection_gap_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud012 projection gap id")
        .get(0);
    let chain_anchor_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud012 chain anchor id")
        .get(0);
    let external_fact_receipt_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud012 receipt id")
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
               now() - interval '6 minutes',
               now() - interval '5 minutes',
               now() - interval '7 minutes',
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
                &format!("aud012-idempotency-{suffix}"),
                &format!("aud012-payload-hash-{suffix}"),
            ],
        )
        .await
        .expect("insert aud012 outbox event");

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
               gen_random_uuid(),
               $1::text::uuid,
               'order',
               $2::text::uuid,
               'fabric.proof_submit_requested',
               jsonb_build_object('event_id', $1, 'seed', $3, 'target_topic', 'dtp.fabric.requests'),
               'fabric callback consumer still pending projection',
               $4,
               $5,
               'business',
               'database',
               'kafka',
               'dtp.fabric.requests',
               'consumer_handler',
               now() - interval '4 minutes',
               now() - interval '3 minutes',
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
        .expect("insert aud012 dead letter");

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
               'received',
               jsonb_build_object('seed', $4, 'provider_status', 'received'),
               $5,
               now() - interval '4 minutes',
               now() - interval '4 minutes' + interval '15 seconds',
               $6,
               $7,
               jsonb_build_object('seed', $4)
             )",
            &[
                &external_fact_receipt_id,
                &seed.order_id,
                &format!("provider-ref-aud012-{suffix}"),
                &suffix,
                &format!("aud012-receipt-hash-{suffix}"),
                &seed_request_id,
                &seed_trace_id,
            ],
        )
        .await
        .expect("insert aud012 external receipt");

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
               '0xaud012anchor',
               'submitted',
               now() - interval '5 minutes',
               now() - interval '5 minutes' - interval '3 seconds',
               'proof_layer',
               'pending_check',
               now() - interval '4 minutes'
             )",
            &[
                &chain_anchor_id,
                &seed.order_id,
                &format!("aud012-digest-{suffix}"),
            ],
        )
        .await
        .expect("insert aud012 chain anchor");

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
               '0xaud012anchor',
               'missing_callback',
               'open',
               now() - interval '180 seconds',
               now() - interval '170 seconds',
               $4,
               $5,
               $6::text::uuid,
               $7::text::uuid,
               jsonb_build_object('seed', $8, 'recommendation', 'trigger reconcile dry-run'),
               jsonb_build_object('seed', $8, 'source', 'aud012-smoke')
             )",
            &[
                &chain_projection_gap_id,
                &seed.order_id,
                &format!("expected-tx-aud012-{suffix}"),
                &seed_request_id,
                &seed_trace_id,
                &outbox_event_id,
                &chain_anchor_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud012 projection gap");

    write_trade_audit_event(
        &client,
        "order",
        &seed.order_id,
        "platform_audit_security",
        "trade.order.reconcile_preview_seeded",
        "accepted",
        Some(&seed_request_id),
        Some(&seed_trace_id),
    )
    .await
    .expect("write aud012 audit trace");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ops/consistency/reconcile")
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &reconcile_request_id)
                .header("x-trace-id", &reconcile_trace_id)
                .header("x-step-up-challenge-id", &reconcile_step_up_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "ref_type": "order",
                        "ref_id": seed.order_id,
                        "mode": "full",
                        "dry_run": true,
                        "reason": "preview consistency repair"
                    })
                    .to_string(),
                ))
                .expect("aud012 reconcile request"),
        )
        .await
        .expect("call aud012 reconcile");
    let response_status = response.status();
    let response_body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read aud012 reconcile body");
    assert_eq!(
        response_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&response_body)
    );
    let response_json: Value =
        serde_json::from_slice(&response_body).expect("decode aud012 reconcile body");
    assert_eq!(response_json["data"]["ref_type"].as_str(), Some("order"));
    assert_eq!(
        response_json["data"]["ref_id"].as_str(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(response_json["data"]["mode"].as_str(), Some("full"));
    assert_eq!(response_json["data"]["dry_run"].as_bool(), Some(true));
    assert_eq!(
        response_json["data"]["reconcile_target_topic"].as_str(),
        Some("dtp.consistency.reconcile")
    );
    assert_eq!(response_json["data"]["step_up_bound"].as_bool(), Some(true));
    assert_eq!(
        response_json["data"]["status"].as_str(),
        Some("dry_run_ready")
    );
    assert!(
        response_json["data"]["recommendation_count"]
            .as_i64()
            .unwrap_or_default()
            >= 2
    );
    assert_eq!(
        response_json["data"]["related_projection_gaps"][0]["chain_projection_gap_id"].as_str(),
        Some(chain_projection_gap_id.as_str())
    );
    assert!(
        response_json["data"]["recommendations"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );

    let reconcile_audit_row = client
        .query_one(
            "SELECT COUNT(*)::bigint,
                    max(metadata ->> 'reason'),
                    max(metadata ->> 'mode')
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'ops.consistency.reconcile.dry_run'
               AND ref_type = 'order'
               AND ref_id = $2::text::uuid",
            &[&reconcile_request_id, &seed.order_id],
        )
        .await
        .expect("count aud012 audit event");
    assert_eq!(reconcile_audit_row.get::<_, i64>(0), 1);
    assert_eq!(
        reconcile_audit_row.get::<_, Option<String>>(1).as_deref(),
        Some("preview consistency repair")
    );
    assert_eq!(
        reconcile_audit_row.get::<_, Option<String>>(2).as_deref(),
        Some("full")
    );

    let access_row = client
        .query_one(
            "SELECT access_mode,
                    target_type,
                    target_id::text,
                    step_up_challenge_id::text
             FROM audit.access_audit
             WHERE request_id = $1",
            &[&reconcile_request_id],
        )
        .await
        .expect("load aud012 access audit");
    assert_eq!(access_row.get::<_, String>(0), "reconcile");
    assert_eq!(access_row.get::<_, String>(1), "consistency_reconcile");
    assert_eq!(
        access_row.get::<_, Option<String>>(2).as_deref(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        access_row.get::<_, Option<String>>(3).as_deref(),
        Some(reconcile_step_up_id.as_str())
    );

    let system_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = 'ops consistency reconcile prepared: POST /api/v1/ops/consistency/reconcile'",
            &[&reconcile_request_id],
        )
        .await
        .expect("count aud012 system log")
        .get(0);
    assert_eq!(system_log_count, 1);

    let reconcile_outbox_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.outbox_event
             WHERE request_id = $1
               AND target_topic = 'dtp.consistency.reconcile'",
            &[&reconcile_request_id],
        )
        .await
        .expect("count aud012 reconcile outbox")
        .get(0);
    assert_eq!(reconcile_outbox_count, 0);

    let gap_row = client
        .query_one(
            "SELECT gap_status, resolution_summary ->> 'seed'
             FROM ops.chain_projection_gap
             WHERE chain_projection_gap_id = $1::text::uuid",
            &[&chain_projection_gap_id],
        )
        .await
        .expect("load aud012 projection gap");
    assert_eq!(gap_row.get::<_, String>(0), "open");
    assert_eq!(
        gap_row.get::<_, Option<String>>(1).as_deref(),
        Some(suffix.as_str())
    );

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
            "DELETE FROM ops.dead_letter_event WHERE outbox_event_id = $1::text::uuid",
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
    cleanup_business_rows(&client, &seed).await;
}

#[tokio::test]
async fn audit_trade_monitor_db_smoke() {
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
    let seed = seed_order_graph(&client, &format!("aud018-{suffix}"))
        .await
        .expect("seed order graph");
    let operator_user_id = seed_user(&client, &seed.buyer_org_id, &format!("aud018-{suffix}"))
        .await
        .expect("seed aud018 operator");
    let app = crate::with_live_test_state(router()).await;
    let overview_request_id = format!("req-aud018-overview-{suffix}");
    let checkpoints_request_id = format!("req-aud018-checkpoints-{suffix}");
    let tenant_overview_request_id = format!("req-aud018-tenant-overview-{suffix}");
    let forbidden_request_id = format!("req-aud018-tenant-forbidden-{suffix}");
    let trace_id = format!("trace-aud018-{suffix}");

    client
        .execute(
            "UPDATE trade.order_main
             SET authority_model = 'dual_layer',
                 business_state_version = 11,
                 proof_commit_state = 'anchored',
                 proof_commit_policy = 'async_evidence',
                 external_fact_status = 'confirmed',
                 reconcile_status = 'pending_check',
                 last_reconciled_at = now() - interval '2 minutes'
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update aud018 order consistency fields");

    let checkpoint_registered_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud018 registered checkpoint id")
        .get(0);
    let checkpoint_delivery_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud018 delivery checkpoint id")
        .get(0);
    let external_fact_receipt_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud018 receipt id")
        .get(0);
    let fairness_incident_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud018 fairness incident id")
        .get(0);
    let chain_projection_gap_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud018 projection gap id")
        .get(0);
    let chain_anchor_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud018 chain anchor id")
        .get(0);

    client
        .execute(
            "INSERT INTO ops.trade_lifecycle_checkpoint (
               trade_lifecycle_checkpoint_id,
               order_id,
               ref_domain,
               ref_type,
               ref_id,
               checkpoint_code,
               lifecycle_stage,
               checkpoint_status,
               expected_by,
               occurred_at,
               source_type,
               request_id,
               trace_id,
               metadata
             ) VALUES
             (
               $1::text::uuid,
               $2::text::uuid,
               'trade',
               'order',
               $2::text::uuid,
               'funds_locked',
               'payment',
               'completed',
               now() - interval '20 minutes',
               now() - interval '19 minutes',
               'system',
               $3,
               $4,
               jsonb_build_object('seed', $5, 'stage_rank', 1)
             ),
             (
               $6::text::uuid,
               $2::text::uuid,
               'trade',
               'order',
               $2::text::uuid,
               'delivery_prepared',
               'delivery',
               'pending',
               now() + interval '10 minutes',
               now() - interval '4 minutes',
               'system',
               $3,
               $4,
               jsonb_build_object('seed', $5, 'stage_rank', 2)
             )",
            &[
                &checkpoint_registered_id,
                &seed.order_id,
                &overview_request_id,
                &trace_id,
                &suffix,
                &checkpoint_delivery_id,
            ],
        )
        .await
        .expect("insert aud018 trade checkpoints");

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
               now() - interval '6 minutes',
               now() - interval '5 minutes',
               now() - interval '4 minutes',
               $6,
               $7,
               jsonb_build_object('seed', $4)
             )",
            &[
                &external_fact_receipt_id,
                &seed.order_id,
                &format!("provider-ref-aud018-{suffix}"),
                &suffix,
                &format!("aud018-receipt-hash-{suffix}"),
                &overview_request_id,
                &trace_id,
            ],
        )
        .await
        .expect("insert aud018 external receipt");

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
               '0xaud018anchor',
               'anchored',
               now() - interval '3 minutes',
               now() - interval '3 minutes' - interval '5 seconds',
               'proof_layer',
               'matched',
               now() - interval '2 minutes'
             )",
            &[
                &chain_anchor_id,
                &seed.order_id,
                &format!("aud018-digest-{suffix}"),
            ],
        )
        .await
        .expect("insert aud018 chain anchor");

    client
        .execute(
            "INSERT INTO risk.fairness_incident (
               fairness_incident_id,
               order_id,
               ref_type,
               ref_id,
               incident_type,
               severity,
               lifecycle_stage,
               detected_by_type,
               source_checkpoint_id,
               source_receipt_id,
               status,
               auto_action_code,
               assigned_role_key,
               resolution_summary,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'order',
               $2::text::uuid,
               'seller_delivery_delay',
               'high',
               'delivery',
               'rule_engine',
               $3::text::uuid,
               $4::text::uuid,
               'open',
               'notify_ops',
               'platform_risk_settlement',
               'awaiting delivery callback',
               $5,
               $6,
               jsonb_build_object('seed', $7, 'source', 'aud018-smoke')
             )",
            &[
                &fairness_incident_id,
                &seed.order_id,
                &checkpoint_delivery_id,
                &external_fact_receipt_id,
                &overview_request_id,
                &trace_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud018 fairness incident");

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
               anchor_id,
               resolution_summary,
               metadata
             ) VALUES (
               $1::text::uuid,
               'order',
               $2::text::uuid,
               $2::text::uuid,
               'fabric-local',
               'fabric.commit_confirmed',
               $3,
               '0xaud018anchor',
               'projection_lag',
               'open',
               now() - interval '2 minutes',
               now() - interval '1 minute',
               $4,
               $5,
               $6::text::uuid,
               jsonb_build_object('seed', $7, 'summary', 'callback already confirmed but order monitor still open'),
               jsonb_build_object('seed', $7, 'source', 'aud018-smoke')
             )",
            &[
                &chain_projection_gap_id,
                &seed.order_id,
                &format!("expected-tx-aud018-{suffix}"),
                &overview_request_id,
                &trace_id,
                &chain_anchor_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud018 projection gap");

    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/trade-monitor/orders/{}",
                    seed.order_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &overview_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud018 overview request"),
        )
        .await
        .expect("call aud018 overview");
    let overview_status = overview_response.status();
    let overview_body = to_bytes(overview_response.into_body(), usize::MAX)
        .await
        .expect("read aud018 overview body");
    assert_eq!(
        overview_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&overview_body)
    );
    let overview_json: Value =
        serde_json::from_slice(&overview_body).expect("decode aud018 overview body");
    assert_eq!(
        overview_json["data"]["order_id"].as_str(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        overview_json["data"]["current_checkpoint_code"].as_str(),
        Some("delivery_prepared")
    );
    assert_eq!(
        overview_json["data"]["current_checkpoint_status"].as_str(),
        Some("pending")
    );
    assert_eq!(
        overview_json["data"]["proof_commit_state"].as_str(),
        Some("anchored")
    );
    assert_eq!(
        overview_json["data"]["external_fact_status"].as_str(),
        Some("confirmed")
    );
    assert_eq!(
        overview_json["data"]["open_fairness_incident_count"].as_i64(),
        Some(1)
    );
    assert_eq!(
        overview_json["data"]["recent_checkpoints"][0]["trade_lifecycle_checkpoint_id"].as_str(),
        Some(checkpoint_delivery_id.as_str())
    );
    assert_eq!(
        overview_json["data"]["recent_external_facts"][0]["external_fact_receipt_id"].as_str(),
        Some(external_fact_receipt_id.as_str())
    );
    assert_eq!(
        overview_json["data"]["recent_fairness_incidents"][0]["fairness_incident_id"].as_str(),
        Some(fairness_incident_id.as_str())
    );
    assert_eq!(
        overview_json["data"]["recent_projection_gaps"][0]["chain_projection_gap_id"].as_str(),
        Some(chain_projection_gap_id.as_str())
    );

    let checkpoints_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/trade-monitor/orders/{}/checkpoints?checkpoint_status=pending&lifecycle_stage=delivery&page=1&page_size=10",
                    seed.order_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &checkpoints_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud018 checkpoints request"),
        )
        .await
        .expect("call aud018 checkpoints");
    let checkpoints_status = checkpoints_response.status();
    let checkpoints_body = to_bytes(checkpoints_response.into_body(), usize::MAX)
        .await
        .expect("read aud018 checkpoints body");
    assert_eq!(
        checkpoints_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&checkpoints_body)
    );
    let checkpoints_json: Value =
        serde_json::from_slice(&checkpoints_body).expect("decode aud018 checkpoints body");
    assert_eq!(checkpoints_json["data"]["total"].as_i64(), Some(1));
    assert_eq!(
        checkpoints_json["data"]["items"][0]["checkpoint_code"].as_str(),
        Some("delivery_prepared")
    );
    assert_eq!(
        checkpoints_json["data"]["items"][0]["checkpoint_status"].as_str(),
        Some("pending")
    );

    let tenant_overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/trade-monitor/orders/{}",
                    seed.order_id
                ))
                .header("x-role", "tenant_admin")
                .header("x-tenant-id", &seed.buyer_org_id)
                .header("x-request-id", &tenant_overview_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud018 tenant overview request"),
        )
        .await
        .expect("call aud018 tenant overview");
    assert_eq!(tenant_overview_response.status(), StatusCode::OK);

    let forbidden_overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/trade-monitor/orders/{}",
                    seed.order_id
                ))
                .header("x-role", "tenant_admin")
                .header("x-tenant-id", "10000000-0000-0000-0000-00000000f018")
                .header("x-request-id", &forbidden_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud018 forbidden tenant overview request"),
        )
        .await
        .expect("call aud018 forbidden tenant overview");
    assert_eq!(forbidden_overview_response.status(), StatusCode::FORBIDDEN);

    let access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND target_type = ANY($2::text[])
               AND access_mode = 'masked'",
            &[
                &vec![
                    overview_request_id.clone(),
                    checkpoints_request_id.clone(),
                    tenant_overview_request_id.clone(),
                ],
                &vec![
                    "trade_monitor_query".to_string(),
                    "trade_checkpoint_query".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud018 access audit")
        .get(0);
    assert_eq!(access_count, 3);

    let log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = ANY($2::text[])",
            &[
                &vec![
                    overview_request_id.clone(),
                    checkpoints_request_id.clone(),
                    tenant_overview_request_id.clone(),
                ],
                &vec![
                    "ops lookup executed: GET /api/v1/ops/trade-monitor/orders/{orderId}"
                        .to_string(),
                    "ops lookup executed: GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints"
                        .to_string(),
                ],
            ],
        )
        .await
        .expect("count aud018 system log")
        .get(0);
    assert_eq!(log_count, 3);

    let _ = client
        .execute(
            "DELETE FROM ops.trade_lifecycle_checkpoint WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM risk.fairness_incident WHERE fairness_incident_id = $1::text::uuid",
            &[&fairness_incident_id],
        )
        .await;
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
            "DELETE FROM chain.chain_anchor WHERE chain_anchor_id = $1::text::uuid",
            &[&chain_anchor_id],
        )
        .await;
    cleanup_business_rows(&client, &seed).await;
}

#[tokio::test]
async fn audit_external_fact_confirm_db_smoke() {
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
    let seed = seed_order_graph(&client, &format!("aud019-{suffix}"))
        .await
        .expect("seed order graph");
    let operator_user_id = seed_user(&client, &seed.buyer_org_id, &format!("aud019-{suffix}"))
        .await
        .expect("seed aud019 operator");
    let app = crate::with_live_test_state(router()).await;
    let list_request_id = format!("req-aud019-list-{suffix}");
    let confirm_request_id = format!("req-aud019-confirm-{suffix}");
    let trace_id = format!("trace-aud019-{suffix}");
    let external_fact_receipt_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud019 receipt id")
        .get(0);

    client
        .execute(
            "UPDATE trade.order_main
             SET authority_model = 'dual_layer',
                 business_state_version = 12,
                 proof_commit_state = 'pending_anchor',
                 proof_commit_policy = 'async_evidence',
                 external_fact_status = 'pending_receipt',
                 reconcile_status = 'pending_check',
                 last_reconciled_at = now() - interval '10 minutes'
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update aud019 order consistency fields");

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
               now() - interval '6 minutes',
               now() - interval '5 minutes',
               $6,
               $7,
               jsonb_build_object('seed', $4, 'source', 'aud019-smoke')
             )",
            &[
                &external_fact_receipt_id,
                &seed.order_id,
                &format!("provider-ref-aud019-{suffix}"),
                &suffix,
                &format!("aud019-receipt-hash-{suffix}"),
                &list_request_id,
                &trace_id,
            ],
        )
        .await
        .expect("insert aud019 external receipt");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/external-facts?order_id={}&receipt_status=pending&provider_type=mock_payment_provider&from=2000-01-01T00:00:00.000Z&to=2999-01-01T00:00:00.000Z&page=1&page_size=10",
                    seed.order_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &list_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud019 list request"),
        )
        .await
        .expect("call aud019 list");
    let list_status = list_response.status();
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("read aud019 list body");
    assert_eq!(
        list_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&list_body)
    );
    let list_json: Value = serde_json::from_slice(&list_body).expect("decode aud019 list body");
    assert_eq!(list_json["data"]["total"].as_i64(), Some(1));
    assert_eq!(
        list_json["data"]["items"][0]["external_fact_receipt_id"].as_str(),
        Some(external_fact_receipt_id.as_str())
    );
    assert_eq!(
        list_json["data"]["items"][0]["receipt_status"].as_str(),
        Some("pending")
    );

    let confirm_step_up_id = seed_verified_step_up_challenge(
        &client,
        &operator_user_id,
        "ops.external_fact.manage",
        "external_fact_receipt",
        Some(&external_fact_receipt_id),
        &format!("aud019-{suffix}"),
    )
    .await
    .expect("seed aud019 external fact confirm step-up");

    let confirm_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/ops/external-facts/{}/confirm",
                    external_fact_receipt_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &confirm_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &confirm_step_up_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"confirm_result":"confirmed","reason":"operator verified payment callback","operator_note":"provider callback digest matches expected invoice"}"#,
                ))
                .expect("aud019 confirm request"),
        )
        .await
        .expect("call aud019 confirm");
    let confirm_status = confirm_response.status();
    let confirm_body = to_bytes(confirm_response.into_body(), usize::MAX)
        .await
        .expect("read aud019 confirm body");
    assert_eq!(
        confirm_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&confirm_body)
    );
    let confirm_json: Value =
        serde_json::from_slice(&confirm_body).expect("decode aud019 confirm body");
    assert_eq!(
        confirm_json["data"]["confirm_result"].as_str(),
        Some("confirmed")
    );
    assert_eq!(
        confirm_json["data"]["status"].as_str(),
        Some("manual_confirmation_recorded")
    );
    assert_eq!(
        confirm_json["data"]["rule_evaluation_status"].as_str(),
        Some("pending_follow_up")
    );
    assert_eq!(
        confirm_json["data"]["external_fact_receipt"]["receipt_status"].as_str(),
        Some("confirmed")
    );
    assert!(
        confirm_json["data"]["external_fact_receipt"]["confirmed_at"]
            .as_str()
            .is_some()
    );

    let receipt_row = client
        .query_one(
            "SELECT
               receipt_status,
               confirmed_at IS NOT NULL,
               metadata -> 'manual_confirmation' ->> 'confirm_result',
               metadata -> 'manual_confirmation' ->> 'reason',
               metadata -> 'rule_evaluation' ->> 'status'
             FROM ops.external_fact_receipt
             WHERE external_fact_receipt_id = $1::text::uuid",
            &[&external_fact_receipt_id],
        )
        .await
        .expect("load aud019 receipt row");
    let receipt_status: String = receipt_row.get(0);
    let confirmed_at_present: bool = receipt_row.get(1);
    let metadata_confirm_result: Option<String> = receipt_row.get(2);
    let metadata_reason: Option<String> = receipt_row.get(3);
    let rule_evaluation_status: Option<String> = receipt_row.get(4);
    assert_eq!(receipt_status, "confirmed");
    assert!(confirmed_at_present);
    assert_eq!(metadata_confirm_result.as_deref(), Some("confirmed"));
    assert_eq!(
        metadata_reason.as_deref(),
        Some("operator verified payment callback")
    );
    assert_eq!(rule_evaluation_status.as_deref(), Some("pending_follow_up"));

    let order_external_fact_status: String = client
        .query_one(
            "SELECT external_fact_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("load aud019 order status")
        .get(0);
    assert_eq!(order_external_fact_status, "pending_receipt");

    let audit_event_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'ops.external_fact.confirm'
               AND result_code = 'confirmed'",
            &[&confirm_request_id],
        )
        .await
        .expect("count aud019 audit event")
        .get(0);
    assert_eq!(audit_event_count, 1);

    let access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND target_type = ANY($2::text[])",
            &[
                &vec![list_request_id.clone(), confirm_request_id.clone()],
                &vec![
                    "external_fact_query".to_string(),
                    "external_fact_receipt".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud019 access audit")
        .get(0);
    assert_eq!(access_count, 2);

    let log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = ANY($2::text[])",
            &[
                &vec![list_request_id.clone(), confirm_request_id.clone()],
                &vec![
                    "ops lookup executed: GET /api/v1/ops/external-facts".to_string(),
                    "ops external fact confirm executed: POST /api/v1/ops/external-facts/{id}/confirm".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud019 system log")
        .get(0);
    assert_eq!(log_count, 2);

    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&confirm_step_up_id],
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
            "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
            &[&operator_user_id],
        )
        .await;
    cleanup_business_rows(&client, &seed).await;
}

#[tokio::test]
async fn audit_fairness_incident_handle_db_smoke() {
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
    let seed = seed_order_graph(&client, &format!("aud020-{suffix}"))
        .await
        .expect("seed order graph");
    let operator_user_id = seed_user(&client, &seed.buyer_org_id, &format!("aud020-{suffix}"))
        .await
        .expect("seed aud020 operator");
    let app = crate::with_live_test_state(router()).await;
    let list_request_id = format!("req-aud020-list-{suffix}");
    let handle_request_id = format!("req-aud020-handle-{suffix}");
    let trace_id = format!("trace-aud020-{suffix}");
    let checkpoint_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud020 checkpoint id")
        .get(0);
    let external_fact_receipt_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud020 receipt id")
        .get(0);
    let fairness_incident_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud020 fairness incident id")
        .get(0);

    client
        .execute(
            "UPDATE trade.order_main
             SET authority_model = 'dual_layer',
                 business_state_version = 13,
                 proof_commit_state = 'pending_anchor',
                 proof_commit_policy = 'async_evidence',
                 external_fact_status = 'pending_receipt',
                 reconcile_status = 'pending_check',
                 last_reconciled_at = now() - interval '12 minutes'
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update aud020 order consistency fields");

    client
        .execute(
            "INSERT INTO ops.trade_lifecycle_checkpoint (
               trade_lifecycle_checkpoint_id,
               order_id,
               ref_domain,
               ref_type,
               ref_id,
               checkpoint_code,
               lifecycle_stage,
               checkpoint_status,
               expected_by,
               occurred_at,
               source_type,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'trade',
               'order',
               $2::text::uuid,
               'delivery_prepared',
               'delivery',
               'pending',
               now() + interval '5 minutes',
               now() - interval '6 minutes',
               'system',
               $3,
               $4,
               jsonb_build_object('seed', $5, 'source', 'aud020-smoke')
             )",
            &[
                &checkpoint_id,
                &seed.order_id,
                &list_request_id,
                &trace_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud020 checkpoint");

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
               'delivery',
               'order',
               $2::text::uuid,
               'delivery_callback',
               'mock_delivery_provider',
               'mock-delivery',
               $3,
               'pending',
               jsonb_build_object('seed', $4, 'provider_status', 'late'),
               $5,
               now() - interval '8 minutes',
               now() - interval '7 minutes',
               $6,
               $7,
               jsonb_build_object('seed', $4, 'source', 'aud020-smoke')
             )",
            &[
                &external_fact_receipt_id,
                &seed.order_id,
                &format!("provider-ref-aud020-{suffix}"),
                &suffix,
                &format!("aud020-receipt-hash-{suffix}"),
                &list_request_id,
                &trace_id,
            ],
        )
        .await
        .expect("insert aud020 external receipt");

    client
        .execute(
            "INSERT INTO risk.fairness_incident (
               fairness_incident_id,
               order_id,
               ref_type,
               ref_id,
               incident_type,
               severity,
               lifecycle_stage,
               detected_by_type,
               source_checkpoint_id,
               source_receipt_id,
               status,
               auto_action_code,
               assigned_role_key,
               assigned_user_id,
               resolution_summary,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'order',
               $2::text::uuid,
               'seller_delivery_delay',
               'high',
               'delivery',
               'rule_engine',
               $3::text::uuid,
               $4::text::uuid,
               'open',
               'notify_ops',
               'platform_risk_settlement',
               $5::text::uuid,
               'awaiting manual review',
               $6,
               $7,
               jsonb_build_object('seed', $8, 'source', 'aud020-smoke')
             )",
            &[
                &fairness_incident_id,
                &seed.order_id,
                &checkpoint_id,
                &external_fact_receipt_id,
                &operator_user_id,
                &list_request_id,
                &trace_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud020 fairness incident");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/fairness-incidents?order_id={}&incident_type=seller_delivery_delay&severity=high&fairness_incident_status=open&assigned_role_key=platform_risk_settlement&assigned_user_id={}&page=1&page_size=10",
                    seed.order_id, operator_user_id
                ))
                .header("x-role", "platform_risk_settlement")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &list_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud020 list request"),
        )
        .await
        .expect("call aud020 list");
    let list_status = list_response.status();
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("read aud020 list body");
    assert_eq!(
        list_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&list_body)
    );
    let list_json: Value = serde_json::from_slice(&list_body).expect("decode aud020 list body");
    assert_eq!(list_json["data"]["total"].as_i64(), Some(1));
    assert_eq!(
        list_json["data"]["items"][0]["fairness_incident_id"].as_str(),
        Some(fairness_incident_id.as_str())
    );
    assert_eq!(
        list_json["data"]["items"][0]["fairness_incident_status"].as_str(),
        Some("open")
    );

    let handle_step_up_id = seed_verified_step_up_challenge(
        &client,
        &operator_user_id,
        "risk.fairness_incident.handle",
        "fairness_incident",
        Some(&fairness_incident_id),
        &format!("aud020-{suffix}"),
    )
    .await
    .expect("seed aud020 fairness handle step-up");

    let handle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/ops/fairness-incidents/{}/handle",
                    fairness_incident_id
                ))
                .header("x-role", "platform_risk_settlement")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &handle_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &handle_step_up_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"action":"close","resolution_summary":"manual review confirmed delivery delay risk","auto_action_override":"notify_ops","freeze_settlement":true,"freeze_delivery":false,"create_dispute_suggestion":true}"#,
                ))
                .expect("aud020 handle request"),
        )
        .await
        .expect("call aud020 handle");
    let handle_status = handle_response.status();
    let handle_body = to_bytes(handle_response.into_body(), usize::MAX)
        .await
        .expect("read aud020 handle body");
    assert_eq!(
        handle_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&handle_body)
    );
    let handle_json: Value =
        serde_json::from_slice(&handle_body).expect("decode aud020 handle body");
    assert_eq!(handle_json["data"]["action"].as_str(), Some("close"));
    assert_eq!(
        handle_json["data"]["status"].as_str(),
        Some("manual_handling_recorded")
    );
    assert_eq!(
        handle_json["data"]["action_plan_status"].as_str(),
        Some("suggestion_recorded")
    );
    assert_eq!(
        handle_json["data"]["fairness_incident"]["fairness_incident_status"].as_str(),
        Some("closed")
    );
    assert!(
        handle_json["data"]["fairness_incident"]["closed_at"]
            .as_str()
            .is_some()
    );

    let incident_row = client
        .query_one(
            "SELECT
               status,
               closed_at IS NOT NULL,
               auto_action_code,
               resolution_summary,
               metadata -> 'handling' ->> 'action',
               metadata -> 'linked_action_plan' ->> 'status',
               metadata -> 'linked_action_plan' ->> 'freeze_settlement',
               metadata -> 'linked_action_plan' ->> 'create_dispute_suggestion'
             FROM risk.fairness_incident
             WHERE fairness_incident_id = $1::text::uuid",
            &[&fairness_incident_id],
        )
        .await
        .expect("load aud020 incident row");
    let incident_status: String = incident_row.get(0);
    let closed_at_present: bool = incident_row.get(1);
    let auto_action_code: Option<String> = incident_row.get(2);
    let resolution_summary: Option<String> = incident_row.get(3);
    let handled_action: Option<String> = incident_row.get(4);
    let action_plan_status: Option<String> = incident_row.get(5);
    let freeze_settlement: Option<String> = incident_row.get(6);
    let create_dispute_suggestion: Option<String> = incident_row.get(7);
    assert_eq!(incident_status, "closed");
    assert!(closed_at_present);
    assert_eq!(auto_action_code.as_deref(), Some("notify_ops"));
    assert_eq!(
        resolution_summary.as_deref(),
        Some("manual review confirmed delivery delay risk")
    );
    assert_eq!(handled_action.as_deref(), Some("close"));
    assert_eq!(action_plan_status.as_deref(), Some("suggestion_recorded"));
    assert_eq!(freeze_settlement.as_deref(), Some("true"));
    assert_eq!(create_dispute_suggestion.as_deref(), Some("true"));

    let order_state_row = client
        .query_one(
            "SELECT settlement_status, delivery_status, dispute_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("load aud020 order state");
    let settlement_status: String = order_state_row.get(0);
    let delivery_status: String = order_state_row.get(1);
    let dispute_status: String = order_state_row.get(2);
    assert_eq!(settlement_status, "pending_settlement");
    assert_eq!(delivery_status, "pending_delivery");
    assert_eq!(dispute_status, "none");

    let audit_event_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'risk.fairness_incident.handle'
               AND result_code = 'close'",
            &[&handle_request_id],
        )
        .await
        .expect("count aud020 audit event")
        .get(0);
    assert_eq!(audit_event_count, 1);

    let access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND target_type = ANY($2::text[])",
            &[
                &vec![list_request_id.clone(), handle_request_id.clone()],
                &vec![
                    "fairness_incident_query".to_string(),
                    "fairness_incident".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud020 access audit")
        .get(0);
    assert_eq!(access_count, 2);

    let log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = ANY($2::text[])",
            &[
                &vec![list_request_id.clone(), handle_request_id.clone()],
                &vec![
                    "ops lookup executed: GET /api/v1/ops/fairness-incidents".to_string(),
                    "risk fairness incident handle executed: POST /api/v1/ops/fairness-incidents/{id}/handle".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud020 system log")
        .get(0);
    assert_eq!(log_count, 2);

    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&handle_step_up_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM risk.fairness_incident WHERE fairness_incident_id = $1::text::uuid",
            &[&fairness_incident_id],
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
            "DELETE FROM ops.trade_lifecycle_checkpoint WHERE trade_lifecycle_checkpoint_id = $1::text::uuid",
            &[&checkpoint_id],
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

#[tokio::test]
async fn audit_projection_gap_resolve_db_smoke() {
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
    let seed = seed_order_graph(&client, &format!("aud021-{suffix}"))
        .await
        .expect("seed order graph");
    let operator_user_id = seed_user(&client, &seed.buyer_org_id, &format!("aud021-{suffix}"))
        .await
        .expect("seed aud021 operator");
    let app = crate::with_live_test_state(router()).await;
    let list_request_id = format!("req-aud021-list-{suffix}");
    let dry_run_request_id = format!("req-aud021-dry-run-{suffix}");
    let execute_request_id = format!("req-aud021-execute-{suffix}");
    let trace_id = format!("trace-aud021-{suffix}");
    let chain_projection_gap_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate aud021 projection gap id")
        .get(0);

    client
        .execute(
            "UPDATE trade.order_main
             SET authority_model = 'dual_layer',
                 business_state_version = 21,
                 proof_commit_state = 'pending_anchor',
                 proof_commit_policy = 'async_evidence',
                 external_fact_status = 'pending_receipt',
                 reconcile_status = 'pending_check',
                 last_reconciled_at = now() - interval '15 minutes'
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update aud021 order consistency fields");

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
               resolution_summary,
               metadata
             ) VALUES (
               $1::text::uuid,
               'order',
               $2::text::uuid,
               $2::text::uuid,
               'fabric-local',
               'fabric.anchor.confirmed',
               $3,
               $4,
               'missing_callback',
               'open',
               now() - interval '18 minutes',
               now() - interval '2 minutes',
               $5,
               $6,
               jsonb_build_object('seed', $7),
               jsonb_build_object('seed', $7, 'source', 'aud021-smoke')
             )",
            &[
                &chain_projection_gap_id,
                &seed.order_id,
                &format!("aud021-expected-tx-{suffix}"),
                &format!("aud021-projected-hash-{suffix}"),
                &list_request_id,
                &trace_id,
                &suffix,
            ],
        )
        .await
        .expect("insert aud021 projection gap");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ops/projection-gaps?aggregate_type=order&aggregate_id={}&order_id={}&chain_id=fabric-local&gap_type=missing_callback&gap_status=open&page=1&page_size=10",
                    seed.order_id, seed.order_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &list_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("aud021 list request"),
        )
        .await
        .expect("call aud021 list");
    let list_status = list_response.status();
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("read aud021 list body");
    assert_eq!(
        list_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&list_body)
    );
    let list_json: Value = serde_json::from_slice(&list_body).expect("decode aud021 list body");
    assert_eq!(list_json["data"]["total"].as_i64(), Some(1));
    assert_eq!(
        list_json["data"]["items"][0]["chain_projection_gap_id"].as_str(),
        Some(chain_projection_gap_id.as_str())
    );
    assert_eq!(
        list_json["data"]["items"][0]["gap_status"].as_str(),
        Some("open")
    );

    let resolve_step_up_id = seed_verified_step_up_challenge(
        &client,
        &operator_user_id,
        "ops.projection_gap.manage",
        "projection_gap",
        Some(&chain_projection_gap_id),
        &format!("aud021-{suffix}"),
    )
    .await
    .expect("seed aud021 projection gap resolve step-up");

    let dry_run_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/ops/projection-gaps/{}/resolve",
                    chain_projection_gap_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &dry_run_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &resolve_step_up_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"dry_run":true,"resolution_mode":"callback_confirmed","reason":"preview close projection gap after callback verification"}"#,
                ))
                .expect("aud021 dry-run request"),
        )
        .await
        .expect("call aud021 dry-run");
    let dry_run_status = dry_run_response.status();
    let dry_run_body = to_bytes(dry_run_response.into_body(), usize::MAX)
        .await
        .expect("read aud021 dry-run body");
    assert_eq!(
        dry_run_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&dry_run_body)
    );
    let dry_run_json: Value =
        serde_json::from_slice(&dry_run_body).expect("decode aud021 dry-run body");
    assert_eq!(dry_run_json["data"]["dry_run"].as_bool(), Some(true));
    assert_eq!(
        dry_run_json["data"]["status"].as_str(),
        Some("dry_run_ready")
    );
    assert_eq!(
        dry_run_json["data"]["projection_gap"]["gap_status"].as_str(),
        Some("open")
    );
    let expected_state_digest = dry_run_json["data"]["state_digest"]
        .as_str()
        .expect("aud021 dry-run state digest")
        .to_string();

    let dry_run_gap_row = client
        .query_one(
            "SELECT gap_status, resolved_at IS NOT NULL
             FROM ops.chain_projection_gap
             WHERE chain_projection_gap_id = $1::text::uuid",
            &[&chain_projection_gap_id],
        )
        .await
        .expect("load aud021 dry-run gap row");
    assert_eq!(dry_run_gap_row.get::<_, String>(0), "open");
    assert!(!dry_run_gap_row.get::<_, bool>(1));

    let execute_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/ops/projection-gaps/{}/resolve",
                    chain_projection_gap_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &execute_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &resolve_step_up_id)
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"dry_run\":false,\"resolution_mode\":\"callback_confirmed\",\"reason\":\"confirmed callback backfilled into projection gap\",\"expected_state_digest\":\"{}\"}}",
                    expected_state_digest
                )))
                .expect("aud021 execute request"),
        )
        .await
        .expect("call aud021 execute");
    let execute_status = execute_response.status();
    let execute_body = to_bytes(execute_response.into_body(), usize::MAX)
        .await
        .expect("read aud021 execute body");
    assert_eq!(
        execute_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&execute_body)
    );
    let execute_json: Value =
        serde_json::from_slice(&execute_body).expect("decode aud021 execute body");
    assert_eq!(execute_json["data"]["dry_run"].as_bool(), Some(false));
    assert_eq!(
        execute_json["data"]["status"].as_str(),
        Some("resolution_recorded")
    );
    assert_eq!(
        execute_json["data"]["projection_gap"]["gap_status"].as_str(),
        Some("resolved")
    );
    assert!(
        execute_json["data"]["projection_gap"]["resolved_at"]
            .as_str()
            .is_some()
    );

    let gap_row = client
        .query_one(
            "SELECT
               gap_status,
               resolved_at IS NOT NULL,
               request_id,
               trace_id,
               resolution_summary -> 'manual_resolution' ->> 'reason',
               resolution_summary -> 'manual_resolution' ->> 'resolution_mode',
               metadata -> 'manual_resolution' ->> 'current_state_digest'
             FROM ops.chain_projection_gap
             WHERE chain_projection_gap_id = $1::text::uuid",
            &[&chain_projection_gap_id],
        )
        .await
        .expect("load aud021 gap row");
    assert_eq!(gap_row.get::<_, String>(0), "resolved");
    assert!(gap_row.get::<_, bool>(1));
    assert_eq!(
        gap_row.get::<_, Option<String>>(2).as_deref(),
        Some(execute_request_id.as_str())
    );
    assert_eq!(
        gap_row.get::<_, Option<String>>(3).as_deref(),
        Some(trace_id.as_str())
    );
    assert_eq!(
        gap_row.get::<_, Option<String>>(4).as_deref(),
        Some("confirmed callback backfilled into projection gap")
    );
    assert_eq!(
        gap_row.get::<_, Option<String>>(5).as_deref(),
        Some("callback_confirmed")
    );
    assert_eq!(
        gap_row.get::<_, Option<String>>(6).as_deref(),
        Some(expected_state_digest.as_str())
    );

    let audit_event_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = ANY($1::text[])
               AND action_name = 'ops.projection_gap.resolve'",
            &[&vec![
                dry_run_request_id.clone(),
                execute_request_id.clone(),
            ]],
        )
        .await
        .expect("count aud021 audit event")
        .get(0);
    assert_eq!(audit_event_count, 2);

    let access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND target_type = ANY($2::text[])",
            &[
                &vec![
                    list_request_id.clone(),
                    dry_run_request_id.clone(),
                    execute_request_id.clone(),
                ],
                &vec![
                    "projection_gap_query".to_string(),
                    "projection_gap".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud021 access audit")
        .get(0);
    assert_eq!(access_count, 3);

    let log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = ANY($2::text[])",
            &[
                &vec![
                    list_request_id.clone(),
                    dry_run_request_id.clone(),
                    execute_request_id.clone(),
                ],
                &vec![
                    "ops lookup executed: GET /api/v1/ops/projection-gaps".to_string(),
                    "ops projection gap resolve prepared: POST /api/v1/ops/projection-gaps/{id}/resolve".to_string(),
                    "ops projection gap resolve executed: POST /api/v1/ops/projection-gaps/{id}/resolve".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud021 system log")
        .get(0);
    assert_eq!(log_count, 3);

    let reconcile_outbox_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.outbox_event
             WHERE request_id = ANY($1::text[])
               AND target_topic = 'dtp.consistency.reconcile'",
            &[&vec![
                dry_run_request_id.clone(),
                execute_request_id.clone(),
            ]],
        )
        .await
        .expect("count aud021 reconcile outbox")
        .get(0);
    assert_eq!(reconcile_outbox_count, 0);

    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&resolve_step_up_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.chain_projection_gap WHERE chain_projection_gap_id = $1::text::uuid",
            &[&chain_projection_gap_id],
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

#[tokio::test]
async fn observability_api_db_smoke() {
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
    let operator_user_id = seed_user(&client, &seed.buyer_org_id, &format!("aud023-{suffix}"))
        .await
        .expect("seed aud023 user");
    let log_export_step_up_id = seed_verified_step_up_challenge(
        &client,
        &operator_user_id,
        "ops.log.export",
        "system_log_query",
        None,
        "aud023",
    )
    .await
    .expect("seed aud023 log export step-up");

    let app = crate::with_live_test_state(router()).await;
    let seed_request_id = format!("req-aud023-seed-{suffix}");
    let trace_id = format!("trace-aud023-{suffix}");
    let overview_request_id = format!("req-aud023-overview-{suffix}");
    let logs_request_id = format!("req-aud023-logs-{suffix}");
    let export_request_id = format!("req-aud023-export-{suffix}");
    let trace_request_id = format!("req-aud023-trace-{suffix}");
    let alerts_request_id = format!("req-aud023-alerts-{suffix}");
    let incidents_request_id = format!("req-aud023-incidents-{suffix}");
    let slos_request_id = format!("req-aud023-slos-{suffix}");

    let trace_index_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate trace index id")
        .get(0);
    let alert_rule_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate alert rule id")
        .get(0);
    let alert_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate alert event id")
        .get(0);
    let incident_ticket_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate incident ticket id")
        .get(0);
    let incident_event_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate incident event id")
        .get(0);
    let slo_definition_id: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate slo definition id")
        .get(0);

    client
        .execute(
            "INSERT INTO ops.system_log (
               system_log_id,
               service_name,
               logger_name,
               log_level,
               request_id,
               trace_id,
               message_text,
               structured_payload,
               environment_code,
               backend_type,
               severity_number,
               object_type,
               object_id,
               masked_status,
               retention_class,
               legal_hold_status,
               resource_attrs
             ) VALUES (
               $1::text::uuid,
               'platform-core',
               'audit.observability',
               'INFO',
               $2,
               $3,
               $4,
               $5::jsonb,
               'local',
               'database_mirror',
               9,
               'order',
               $6::text::uuid,
               'masked',
               'ops_default',
               'none',
               '{}'::jsonb
             )",
            &[
                &client
                    .query_one("SELECT gen_random_uuid()::text", &[])
                    .await
                    .expect("log row 1 id")
                    .get::<_, String>(0),
                &seed_request_id,
                &trace_id,
                &format!("aud023 seed log row one {suffix}"),
                &json!({"seed":"aud023","row":1}),
                &seed.order_id,
            ],
        )
        .await
        .expect("insert aud023 system log row one");
    client
        .execute(
            "INSERT INTO ops.system_log (
               system_log_id,
               service_name,
               logger_name,
               log_level,
               request_id,
               trace_id,
               message_text,
               structured_payload,
               environment_code,
               backend_type,
               severity_number,
               object_type,
               object_id,
               masked_status,
               retention_class,
               legal_hold_status,
               resource_attrs
             ) VALUES (
               $1::text::uuid,
               'platform-core',
               'audit.observability',
               'WARN',
               $2,
               $3,
               $4,
               $5::jsonb,
               'local',
               'database_mirror',
               13,
               'order',
               $6::text::uuid,
               'masked',
               'ops_default',
               'none',
               '{}'::jsonb
             )",
            &[
                &client
                    .query_one("SELECT gen_random_uuid()::text", &[])
                    .await
                    .expect("log row 2 id")
                    .get::<_, String>(0),
                &seed_request_id,
                &trace_id,
                &format!("aud023 seed log row two {suffix}"),
                &json!({"seed":"aud023","row":2,"alert_candidate":true}),
                &seed.order_id,
            ],
        )
        .await
        .expect("insert aud023 system log row two");
    client
        .execute(
            "INSERT INTO ops.trace_index (
               trace_index_id,
               trace_id,
               traceparent,
               backend_key,
               root_service_name,
               root_span_name,
               request_id,
               ref_type,
               ref_id,
               object_type,
               object_id,
               status,
               span_count,
               started_at,
               ended_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               '00-aud023trace-00000000000000000000000000000000-0000000000000000-01',
               'tempo_main',
               'platform-core',
               'GET /api/v1/ops/logs/export',
               $3,
               'order',
               $4::text::uuid,
               'order',
               $4::text::uuid,
               'ok',
               5,
               now() - interval '2 minutes',
               now() - interval '1 minute',
               jsonb_build_object('seed', 'aud023')
             )",
            &[&trace_index_id, &trace_id, &seed_request_id, &seed.order_id],
        )
        .await
        .expect("insert aud023 trace index");
    client
        .execute(
            "INSERT INTO ops.alert_rule (
               alert_rule_id,
               rule_key,
               source_backend_key,
               severity,
               alert_type,
               expression_text,
               target_scope_json,
               notification_policy_json,
               runbook_uri,
               status,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               'prometheus_main',
               'high',
               'event_pipeline',
               'aud023 export trace count spike',
               '{}'::jsonb,
               '{}'::jsonb,
               '/runbooks/observability-local',
               'active',
               jsonb_build_object('seed', 'aud023')
             )",
            &[&alert_rule_id, &format!("aud023-alert-rule-{suffix}")],
        )
        .await
        .expect("insert aud023 alert rule");
    client
        .execute(
            "INSERT INTO ops.alert_event (
               alert_event_id,
               alert_rule_id,
               source_backend_key,
               fingerprint,
               alert_type,
               severity,
               title_text,
               summary_text,
               ref_type,
               ref_id,
               request_id,
               trace_id,
               labels_json,
               annotations_json,
               status,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'prometheus_main',
               $3,
               'event_pipeline',
               'high',
               'AUD023 log export incident',
               'log export trace produced observability signal',
               'order',
               $4::text::uuid,
               $5,
               $6,
               jsonb_build_object('service', 'platform-core'),
               jsonb_build_object('runbook', '/runbooks/observability-local'),
               'open',
               jsonb_build_object('seed', 'aud023')
             )",
            &[
                &alert_event_id,
                &alert_rule_id,
                &format!("aud023-alert-fingerprint-{suffix}"),
                &seed.order_id,
                &seed_request_id,
                &trace_id,
            ],
        )
        .await
        .expect("insert aud023 alert event");
    client
        .execute(
            "INSERT INTO ops.incident_ticket (
               incident_ticket_id,
               incident_key,
               source_alert_event_id,
               severity,
               title_text,
               summary_text,
               status,
               owner_role_key,
               owner_user_id,
               runbook_uri,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3::text::uuid,
               'high',
               'AUD023 observability incident',
               'triage ongoing',
               'open',
               'platform_audit_security',
               $4::text::uuid,
               '/runbooks/observability-local',
               jsonb_build_object(
                 'seed', 'aud023',
                 'impact_summary', 'log export path under incident triage',
                 'root_cause_summary', 'seeded observability incident'
               )
             )",
            &[
                &incident_ticket_id,
                &format!("INC-AUD023-{suffix}"),
                &alert_event_id,
                &operator_user_id,
            ],
        )
        .await
        .expect("insert aud023 incident ticket");
    client
        .execute(
            "INSERT INTO ops.incident_event (
               incident_event_id,
               incident_ticket_id,
               event_type,
               actor_type,
               actor_id,
               from_status,
               to_status,
               note_text,
               request_id,
               trace_id
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'triaged',
               'user',
               $3::text::uuid,
               'open',
               'open',
               'aud023 incident triaged',
               $4,
               $5
             )",
            &[
                &incident_event_id,
                &incident_ticket_id,
                &operator_user_id,
                &seed_request_id,
                &trace_id,
            ],
        )
        .await
        .expect("insert aud023 incident event");
    client
        .execute(
            "INSERT INTO ops.slo_definition (
               slo_definition_id,
               slo_key,
               service_name,
               indicator_type,
               objective_value,
               window_code,
               source_backend_key,
               alert_rule_id,
               status,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               'platform-core',
               'availability',
               99.900000,
               'rolling_7d',
               'prometheus_main',
               $3::text::uuid,
               'active',
               jsonb_build_object('seed', 'aud023')
             )",
            &[
                &slo_definition_id,
                &format!("aud023-platform-core-{suffix}"),
                &alert_rule_id,
            ],
        )
        .await
        .expect("insert aud023 slo definition");
    client
        .execute(
            "INSERT INTO ops.slo_snapshot (
               slo_definition_id,
               source_backend_key,
               window_started_at,
               window_ended_at,
               measured_value,
               error_budget_remaining,
               status
             ) VALUES (
               $1::text::uuid,
               'prometheus_main',
               now() - interval '7 days',
               now(),
               98.500000,
               12.300000,
               'breached'
             )",
            &[&slo_definition_id],
        )
        .await
        .expect("insert aud023 slo snapshot");

    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/ops/observability/overview")
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &overview_request_id)
                .body(Body::empty())
                .expect("overview request"),
        )
        .await
        .expect("call aud023 overview");
    assert_eq!(overview_response.status(), StatusCode::OK);
    let overview_body = to_bytes(overview_response.into_body(), usize::MAX)
        .await
        .expect("read aud023 overview body");
    let overview_json: Value =
        serde_json::from_slice(&overview_body).expect("decode aud023 overview body");
    assert!(
        overview_json["data"]["alert_summary"]["open_count"]
            .as_i64()
            .unwrap_or_default()
            >= 1
    );
    assert!(
        overview_json["data"]["backend_statuses"]
            .as_array()
            .expect("backend statuses array")
            .iter()
            .any(|item| item["backend"]["backend_key"].as_str() == Some("prometheus_main"))
    );
    assert!(
        overview_json["data"]["key_services"]
            .as_array()
            .expect("key services array")
            .iter()
            .any(|item| item["service_name"].as_str() == Some("platform-core"))
    );
    assert!(
        overview_json["data"]["recent_incidents"]
            .as_array()
            .expect("recent incidents array")
            .iter()
            .any(|item| item["incident_ticket_id"].as_str() == Some(incident_ticket_id.as_str()))
    );

    let logs_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/ops/logs/query?trace_id={trace_id}"))
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &logs_request_id)
                .body(Body::empty())
                .expect("logs request"),
        )
        .await
        .expect("call aud023 logs");
    assert_eq!(logs_response.status(), StatusCode::OK);
    let logs_body = to_bytes(logs_response.into_body(), usize::MAX)
        .await
        .expect("read aud023 logs body");
    let logs_json: Value = serde_json::from_slice(&logs_body).expect("decode aud023 logs body");
    assert_eq!(logs_json["data"]["total"].as_i64(), Some(2));

    let export_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ops/logs/export")
                .header("x-role", "platform_audit_security")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &export_request_id)
                .header("x-trace-id", &trace_id)
                .header("x-step-up-challenge-id", &log_export_step_up_id)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "reason": "incident triage export",
                        "trace_id": trace_id,
                    })
                    .to_string(),
                ))
                .expect("export request"),
        )
        .await
        .expect("call aud023 export");
    let export_status = export_response.status();
    let export_body = to_bytes(export_response.into_body(), usize::MAX)
        .await
        .expect("read aud023 export body");
    assert_eq!(
        export_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&export_body)
    );
    let export_json: Value =
        serde_json::from_slice(&export_body).expect("decode aud023 export body");
    assert_eq!(export_json["data"]["exported_count"].as_i64(), Some(2));
    assert_eq!(export_json["data"]["step_up_bound"].as_bool(), Some(true));
    let export_bucket = export_json["data"]["bucket_name"]
        .as_str()
        .expect("export bucket");
    let export_key = export_json["data"]["object_key"]
        .as_str()
        .expect("export key");
    let exported_object = fetch_object_bytes(export_bucket, export_key)
        .await
        .expect("fetch aud023 export object");
    let exported_json: Value =
        serde_json::from_slice(exported_object.bytes.as_slice()).expect("decode export object");
    assert_eq!(
        exported_json["exported_count"].as_i64(),
        Some(2),
        "{}",
        String::from_utf8_lossy(exported_object.bytes.as_slice())
    );

    let trace_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/ops/traces/{trace_id}"))
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &trace_request_id)
                .body(Body::empty())
                .expect("trace request"),
        )
        .await
        .expect("call aud023 trace");
    assert_eq!(trace_response.status(), StatusCode::OK);
    let trace_body = to_bytes(trace_response.into_body(), usize::MAX)
        .await
        .expect("read aud023 trace body");
    let trace_json: Value = serde_json::from_slice(&trace_body).expect("decode aud023 trace body");
    assert!(
        trace_json["data"]["related_log_count"]
            .as_i64()
            .unwrap_or_default()
            >= 2
    );
    assert_eq!(trace_json["data"]["related_alert_count"].as_i64(), Some(1));
    assert_eq!(
        trace_json["data"]["trace"]["backend_key"].as_str(),
        Some("tempo_main")
    );

    let alerts_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/ops/alerts?severity=high")
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &alerts_request_id)
                .body(Body::empty())
                .expect("alerts request"),
        )
        .await
        .expect("call aud023 alerts");
    assert_eq!(alerts_response.status(), StatusCode::OK);
    let alerts_body = to_bytes(alerts_response.into_body(), usize::MAX)
        .await
        .expect("read aud023 alerts body");
    let alerts_json: Value =
        serde_json::from_slice(&alerts_body).expect("decode aud023 alerts body");
    assert!(
        alerts_json["data"]["items"]
            .as_array()
            .expect("alerts items")
            .iter()
            .any(|item| item["alert_event_id"].as_str() == Some(alert_event_id.as_str()))
    );

    let incidents_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/ops/incidents?owner_role_key=platform_audit_security")
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &incidents_request_id)
                .body(Body::empty())
                .expect("incidents request"),
        )
        .await
        .expect("call aud023 incidents");
    assert_eq!(incidents_response.status(), StatusCode::OK);
    let incidents_body = to_bytes(incidents_response.into_body(), usize::MAX)
        .await
        .expect("read aud023 incidents body");
    let incidents_json: Value =
        serde_json::from_slice(&incidents_body).expect("decode aud023 incidents body");
    assert!(
        incidents_json["data"]["items"]
            .as_array()
            .expect("incidents items")
            .iter()
            .any(|item| item["incident_ticket_id"].as_str() == Some(incident_ticket_id.as_str()))
    );

    let slos_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/ops/slos?service_name=platform-core")
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &slos_request_id)
                .body(Body::empty())
                .expect("slos request"),
        )
        .await
        .expect("call aud023 slos");
    assert_eq!(slos_response.status(), StatusCode::OK);
    let slos_body = to_bytes(slos_response.into_body(), usize::MAX)
        .await
        .expect("read aud023 slos body");
    let slos_json: Value = serde_json::from_slice(&slos_body).expect("decode aud023 slos body");
    assert!(
        slos_json["data"]["items"]
            .as_array()
            .expect("slos items")
            .iter()
            .any(|item| item["slo_definition_id"].as_str() == Some(slo_definition_id.as_str()))
    );

    let audit_event_row = client
        .query_one(
            "SELECT
               COUNT(*)::bigint,
               MIN(step_up_challenge_id::text)
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'ops.log.export'",
            &[&export_request_id],
        )
        .await
        .expect("load aud023 audit event row");
    assert_eq!(audit_event_row.get::<_, i64>(0), 1);
    assert_eq!(
        audit_event_row.get::<_, Option<String>>(1).as_deref(),
        Some(log_export_step_up_id.as_str())
    );

    let access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])
               AND target_type = ANY($2::text[])",
            &[
                &vec![
                    overview_request_id.clone(),
                    logs_request_id.clone(),
                    export_request_id.clone(),
                    trace_request_id.clone(),
                    alerts_request_id.clone(),
                    incidents_request_id.clone(),
                    slos_request_id.clone(),
                ],
                &vec![
                    "observability_overview".to_string(),
                    "system_log_query".to_string(),
                    "system_log_export".to_string(),
                    "trace_lookup".to_string(),
                    "alert_query".to_string(),
                    "incident_query".to_string(),
                    "slo_query".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud023 access audit")
        .get(0);
    assert_eq!(access_count, 7);

    let system_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text = ANY($2::text[])",
            &[
                &vec![
                    overview_request_id.clone(),
                    logs_request_id.clone(),
                    export_request_id.clone(),
                    trace_request_id.clone(),
                    alerts_request_id.clone(),
                    incidents_request_id.clone(),
                    slos_request_id.clone(),
                ],
                &vec![
                    "ops lookup executed: GET /api/v1/ops/observability/overview".to_string(),
                    "ops lookup executed: GET /api/v1/ops/logs/query".to_string(),
                    "ops logs exported: POST /api/v1/ops/logs/export".to_string(),
                    "ops lookup executed: GET /api/v1/ops/traces/{traceId}".to_string(),
                    "ops lookup executed: GET /api/v1/ops/alerts".to_string(),
                    "ops lookup executed: GET /api/v1/ops/incidents".to_string(),
                    "ops lookup executed: GET /api/v1/ops/slos".to_string(),
                ],
            ],
        )
        .await
        .expect("count aud023 system log")
        .get(0);
    assert_eq!(system_log_count, 7);

    let _ = delete_object(export_bucket, export_key).await;
    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
            &[&log_export_step_up_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.incident_event WHERE incident_ticket_id = $1::text::uuid",
            &[&incident_ticket_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.incident_ticket WHERE incident_ticket_id = $1::text::uuid",
            &[&incident_ticket_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.alert_event WHERE alert_event_id = $1::text::uuid",
            &[&alert_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.alert_rule WHERE alert_rule_id = $1::text::uuid",
            &[&alert_rule_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.trace_index WHERE trace_index_id = $1::text::uuid",
            &[&trace_index_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.slo_definition WHERE slo_definition_id = $1::text::uuid",
            &[&slo_definition_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM ops.system_log WHERE request_id = $1",
            &[&seed_request_id],
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
