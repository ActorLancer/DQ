mod bil001_payment_policy_db;
mod bil002_payment_intent_db;
mod bil003_order_lock_db;
mod bil004_mock_payment_adapter_db;
mod bil005_payment_webhook_db;
mod bil006_billing_event_db;
mod bil007_billing_read_db;
mod bil008_settlement_summary_db;
mod bil009_refund_db;
mod bil010_compensation_db;
mod bil011_manual_payout_db;
mod bil012_reconciliation_import_db;
mod bil013_dispute_case_db;
mod bil014_dispute_linkage_db;
mod bil015_settlement_aggregate_db;
mod bil016_settlement_summary_outbox_db;
mod bil017_api_sku_billing_basis_db;
mod bil018_default_sku_billing_basis_db;

#[cfg(test)]
mod tests {
    use super::super::api::router;
    use super::super::webhook::{
        is_replay_window_valid, map_webhook_target_status, now_utc_ms, payment_status_rank,
    };
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn rejects_jurisdiction_request_without_role() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payment-jurisdictions")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_jurisdiction_manage_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payment-jurisdictions")
                    .method("POST")
                    .header("x-role", "platform_admin")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"jurisdiction_code":"SG","jurisdiction_name":"Singapore","regulator_name":"MAS","launch_phase":"launch_active","supports_fiat_collection":true,"supports_fiat_payout":true,"supports_crypto_settlement":false,"jurisdiction_status":"active","policy_snapshot":{"price_currency":"USD"}}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_create_payout_preference_without_permission() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payout-preferences")
                    .method("POST")
                    .header("x-role", "tenant_operator")
                    .header("x-tenant-id", "0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"beneficiary_subject_type":"organization","beneficiary_subject_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","destination_jurisdiction_code":"SG","preferred_currency_code":"SGD","payout_method":"bank_transfer","preferred_provider_key":"offline_bank","beneficiary_snapshot":{}}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_payment_intent_without_permission() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments/intents")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","provider_key":"mock_payment","payer_subject_type":"organization","payer_subject_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","amount":"10.00","payment_method":"wallet"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_payment_intent_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments/intents")
                    .method("POST")
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", "0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56")
                    .header("x-idempotency-key", "pay:test-order:order_payment:1")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","provider_key":"mock_payment","payer_subject_type":"organization","payer_subject_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","payment_amount":"10.00","payment_method":"wallet"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_manual_payout_without_permission() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payouts/manual")
                    .method("POST")
                    .header("x-role", "tenant_admin")
                    .header("x-step-up-token", "payout-stepup")
                    .header("x-idempotency-key", "payout:test:settlement")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","settlement_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","amount":"10.00"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_manual_payout_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payouts/manual")
                    .method("POST")
                    .header("x-role", "platform_finance_operator")
                    .header("x-idempotency-key", "payout:test:settlement")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","settlement_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","amount":"10.00"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_reconciliation_import_without_permission() {
        let app = crate::with_stub_test_state(router());
        let boundary = "BOUNDARY-BIL012-FORBIDDEN";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"provider_key\"\r\n\r\nmock_payment\r\n--{boundary}--\r\n"
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments/reconciliation/import")
                    .method("POST")
                    .header("x-role", "tenant_admin")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_reconciliation_import_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let boundary = "BOUNDARY-BIL012-STEPUP";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"provider_key\"\r\n\r\nmock_payment\r\n--{boundary}--\r\n"
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments/reconciliation/import")
                    .method("POST")
                    .header("x-role", "platform_risk_settlement")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_create_dispute_case_for_seller_role() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cases")
                    .method("POST")
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", "0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","reason_code":"delivery_failed"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_upload_dispute_evidence_for_seller_role() {
        let app = crate::with_stub_test_state(router());
        let boundary = "BOUNDARY-BIL013-EVIDENCE";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"object_type\"\r\n\r\ndelivery_receipt\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"evidence.txt\"\r\nContent-Type: text/plain\r\n\r\nevidence\r\n--{boundary}--\r\n"
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cases/0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56/evidence")
                    .method("POST")
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", "0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_resolve_dispute_case_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cases/0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56/resolve")
                    .method("POST")
                    .header("x-role", "platform_risk_settlement")
                    .header("x-user-id", "0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"decision_code":"refund_full","liability_type":"seller","decision_text":"resolved"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_create_payment_intent_without_idempotency_key() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments/intents")
                    .method("POST")
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", "0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56")
                    .header("x-step-up-token", "bil002-stepup")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","provider_key":"mock_payment","payer_subject_type":"organization","payer_subject_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","payment_amount":"10.00","payment_method":"wallet"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_cancel_payment_intent_for_tenant_operator() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments/intents/0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56/cancel")
                    .method("POST")
                    .header("x-role", "tenant_operator")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_lock_order_for_tenant_operator() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/orders/30000000-0000-0000-0000-000000000101/lock")
                    .method("POST")
                    .header("x-role", "tenant_operator")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"payment_intent_id":"4f4b3a2e-508b-4902-ba35-97aa905b3772"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_refund_execute_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/refunds")
                    .method("POST")
                    .header("x-role", "platform_risk_settlement")
                    .header("x-idempotency-key", "refund:test")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","case_id":"1e4f4f8f-26e2-4d0f-89a6-8e57421cbf57","decision_code":"refund_full","amount":"10.00","reason_code":"delivery_failed"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_compensation_execute_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/compensations")
                    .method("POST")
                    .header("x-role", "platform_risk_settlement")
                    .header("x-idempotency-key", "compensation:test")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","case_id":"1e4f4f8f-26e2-4d0f-89a6-8e57421cbf57","decision_code":"compensation_full","amount":"10.00","reason_code":"sla_breach"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_mock_payment_simulate_without_permission() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/mock/payments/30000000-0000-0000-0000-000000000101/simulate-success")
                    .method("POST")
                    .header("x-role", "tenant_operator")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn webhook_status_mapping_supports_success_fail_timeout() {
        assert_eq!(
            map_webhook_target_status("payment.succeeded", None),
            Some("succeeded")
        );
        assert_eq!(
            map_webhook_target_status("payment.failed", None),
            Some("failed")
        );
        assert_eq!(
            map_webhook_target_status("payment.timeout", None),
            Some("expired")
        );
    }

    #[test]
    fn replay_window_blocks_expired_timestamp() {
        let old = now_utc_ms() - 16 * 60 * 1000;
        assert!(!is_replay_window_valid(old));
        let fresh = now_utc_ms();
        assert!(is_replay_window_valid(fresh));
    }

    #[test]
    fn status_rank_prevents_regression() {
        assert!(payment_status_rank("failed") < payment_status_rank("succeeded"));
        assert!(payment_status_rank("created") < payment_status_rank("failed"));
    }
}
