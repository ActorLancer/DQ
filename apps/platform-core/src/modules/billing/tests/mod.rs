mod bil001_payment_policy_db;
mod bil002_payment_intent_db;
mod bil003_order_lock_db;
mod bil004_mock_payment_adapter_db;
mod bil005_payment_webhook_db;
mod bil006_billing_event_db;
mod bil007_billing_read_db;
mod bil008_settlement_summary_db;

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
