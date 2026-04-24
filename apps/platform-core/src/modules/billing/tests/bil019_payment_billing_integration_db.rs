#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use config::RuntimeConfig;
    use db::{AppDb, Client, DbPoolConfig, GenericClient};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use std::sync::OnceLock;
    use tokio::sync::Mutex;
    use tower::util::ServiceExt;

    fn smoke_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct PaymentSeedOrder {
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    struct DisputeSeedOrder {
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        payment_intent_id: String,
        settlement_id: String,
        delivery_id: String,
        ticket_id: String,
    }

    #[tokio::test]
    async fn bil019_payment_lifecycle_integration_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let _guard = smoke_lock().lock().await;
        let base_dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
            .to_string();
        let client = seeded_client(&dsn_with_app_name(
            &base_dsn,
            &format!("bil019-seed-pay-{suffix}"),
        ))
        .await;
        let buyer_org_id =
            seed_org(&client, &format!("bil019-pay-buyer-{suffix}"), "enterprise").await;
        let seller_org_id = seed_org(
            &client,
            &format!("bil019-pay-seller-{suffix}"),
            "enterprise",
        )
        .await;
        let provider_account_id =
            seed_provider_account(&client, &seller_org_id, &suffix, "mock_payment").await;
        let corridor_policy_id = active_corridor_policy_id(&client).await;

        let success_order =
            seed_payable_order(&client, &buyer_org_id, &seller_org_id, &suffix, "success").await;
        let failed_order =
            seed_payable_order(&client, &buyer_org_id, &seller_org_id, &suffix, "failed").await;
        let timeout_order =
            seed_payable_order(&client, &buyer_org_id, &seller_org_id, &suffix, "timeout").await;

        let app = live_test_router(&dsn_with_app_name(
            &base_dsn,
            &format!("bil019-app-pay-{suffix}"),
        ))
        .await;

        let success_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &success_order.order_id,
            &suffix,
            "success",
            &format!("req-bil019-create-success-{suffix}"),
        )
        .await;
        let failed_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &failed_order.order_id,
            &suffix,
            "failed",
            &format!("req-bil019-create-failed-{suffix}"),
        )
        .await;
        let timeout_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &timeout_order.order_id,
            &suffix,
            "timeout",
            &format!("req-bil019-create-timeout-{suffix}"),
        )
        .await;

        lock_order(
            &app,
            &buyer_org_id,
            &success_order.order_id,
            &success_intent_id,
            &format!("req-bil019-lock-success-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &failed_order.order_id,
            &failed_intent_id,
            &format!("req-bil019-lock-failed-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &timeout_order.order_id,
            &timeout_intent_id,
            &format!("req-bil019-lock-timeout-{suffix}"),
        )
        .await;

        let occurred_at = current_rfc3339(&client).await;
        let success_json = post_webhook(
            &app,
            "mock_payment",
            &format!("req-bil019-webhook-success-{suffix}"),
            json!({
                "provider_event_id": format!("evt-bil019-success-{suffix}"),
                "event_type": "payment.succeeded",
                "provider_transaction_no": format!("txn-bil019-success-{suffix}"),
                "payment_intent_id": success_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "succeeded",
                "occurred_at": occurred_at,
                "raw_payload": {
                    "entry": "bil019",
                    "scenario": "success"
                }
            }),
        )
        .await;
        assert_eq!(
            success_json["data"]["processed_status"].as_str(),
            Some("processed")
        );
        assert_eq!(
            success_json["data"]["applied_payment_status"].as_str(),
            Some("succeeded")
        );

        let failed_json = post_webhook(
            &app,
            "mock_payment",
            &format!("req-bil019-webhook-failed-{suffix}"),
            json!({
                "provider_event_id": format!("evt-bil019-failed-{suffix}"),
                "event_type": "payment.failed",
                "provider_transaction_no": format!("txn-bil019-failed-{suffix}"),
                "payment_intent_id": failed_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "failed",
                "raw_payload": {
                    "entry": "bil019",
                    "scenario": "failed"
                }
            }),
        )
        .await;
        assert_eq!(
            failed_json["data"]["processed_status"].as_str(),
            Some("processed")
        );
        assert_eq!(
            failed_json["data"]["applied_payment_status"].as_str(),
            Some("failed")
        );

        let timeout_json = post_webhook(
            &app,
            "mock_payment",
            &format!("req-bil019-webhook-timeout-{suffix}"),
            json!({
                "provider_event_id": format!("evt-bil019-timeout-{suffix}"),
                "event_type": "payment.timeout",
                "provider_transaction_no": format!("txn-bil019-timeout-{suffix}"),
                "payment_intent_id": timeout_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "timeout",
                "raw_payload": {
                    "entry": "bil019",
                    "scenario": "timeout"
                }
            }),
        )
        .await;
        assert_eq!(
            timeout_json["data"]["processed_status"].as_str(),
            Some("processed")
        );
        assert_eq!(
            timeout_json["data"]["applied_payment_status"].as_str(),
            Some("expired")
        );

        let success_detail = get_billing_order(
            &app,
            &success_order.order_id,
            &buyer_org_id,
            &format!("req-bil019-billing-success-{suffix}"),
        )
        .await;
        assert_eq!(
            success_detail["data"]["billing_events"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            success_detail["data"]["billing_events"][0]["event_type"].as_str(),
            Some("one_time_charge")
        );
        assert_eq!(
            success_detail["data"]["settlement_summary"]["summary_state"].as_str(),
            Some("order_settlement:pending:manual")
        );

        let success_row = client
            .query_one(
                "SELECT status, payment_status, buyer_locked_at IS NOT NULL
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&success_order.order_id],
            )
            .await
            .expect("query success order");
        assert_eq!(success_row.get::<_, String>(0), "buyer_locked");
        assert_eq!(success_row.get::<_, String>(1), "paid");
        assert!(success_row.get::<_, bool>(2));

        let failed_row = client
            .query_one(
                "SELECT status, payment_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&failed_order.order_id],
            )
            .await
            .expect("query failed order");
        assert_eq!(
            failed_row.get::<_, String>(0),
            "payment_failed_pending_resolution"
        );
        assert_eq!(failed_row.get::<_, String>(1), "failed");

        let timeout_row = client
            .query_one(
                "SELECT status, payment_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&timeout_order.order_id],
            )
            .await
            .expect("query timeout order");
        assert_eq!(
            timeout_row.get::<_, String>(0),
            "payment_timeout_pending_compensation_cancel"
        );
        assert_eq!(timeout_row.get::<_, String>(1), "expired");

        let counts = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'one_time_charge'),
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $2::text::uuid),
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $3::text::uuid),
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE partition_key = $1 AND target_topic = 'dtp.outbox.domain-events'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $4 AND action_name = 'payment.webhook.processed'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $5 AND action_name = 'payment.webhook.processed'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $6 AND action_name = 'payment.webhook.processed')",
                &[
                    &success_order.order_id,
                    &failed_order.order_id,
                    &timeout_order.order_id,
                    &format!("req-bil019-webhook-success-{suffix}"),
                    &format!("req-bil019-webhook-failed-{suffix}"),
                    &format!("req-bil019-webhook-timeout-{suffix}"),
                ],
            )
            .await
            .expect("query lifecycle counts");
        assert_eq!(counts.get::<_, i64>(0), 1);
        assert_eq!(counts.get::<_, i64>(1), 0);
        assert_eq!(counts.get::<_, i64>(2), 0);
        assert_eq!(counts.get::<_, i64>(3), 1);
        assert_eq!(counts.get::<_, i64>(4), 1);
        assert_eq!(counts.get::<_, i64>(5), 1);
        assert_eq!(counts.get::<_, i64>(6), 1);

        cleanup_payment_flow(
            &client,
            &provider_account_id,
            &[&success_intent_id, &failed_intent_id, &timeout_intent_id],
            &[&success_order, &failed_order, &timeout_order],
            &buyer_org_id,
            &seller_org_id,
        )
        .await;
    }

    #[tokio::test]
    async fn bil019_dispute_refund_compensation_recompute_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let _guard = smoke_lock().lock().await;
        let base_dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
            .to_string();
        let client = seeded_client(&dsn_with_app_name(
            &base_dsn,
            &format!("bil019-seed-dispute-{suffix}"),
        ))
        .await;
        let buyer_org_id = seed_org(
            &client,
            &format!("bil019-dispute-buyer-{suffix}"),
            "enterprise",
        )
        .await;
        let seller_org_id = seed_org(
            &client,
            &format!("bil019-dispute-seller-{suffix}"),
            "enterprise",
        )
        .await;
        let platform_org_id = seed_org(
            &client,
            &format!("bil019-dispute-platform-{suffix}"),
            "platform",
        )
        .await;
        let buyer_user_id = seed_user(&client, &buyer_org_id, &format!("buyer-{suffix}")).await;
        let platform_user_id =
            seed_user(&client, &platform_org_id, &format!("platform-{suffix}")).await;

        let refund_order = seed_dispute_ready_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "refund",
            Some("REFUND_FILE_V1"),
            None,
        )
        .await;
        let compensation_order = seed_dispute_ready_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "compensation",
            None,
            Some("COMPENSATION_FILE_V1"),
        )
        .await;

        let app = live_test_router(&dsn_with_app_name(
            &base_dsn,
            &format!("bil019-app-dispute-{suffix}"),
        ))
        .await;

        let refund_case = create_case(
            &app,
            &buyer_org_id,
            &buyer_user_id,
            &refund_order.order_id,
            "refund_full",
            &format!("req-bil019-case-refund-{suffix}"),
        )
        .await;
        let refund_case_id = refund_case["data"]["case_id"]
            .as_str()
            .expect("refund case id")
            .to_string();
        assert_dispute_frozen_snapshot(
            &client,
            &app,
            &refund_order,
            &buyer_org_id,
            &format!("req-bil019-billing-refund-frozen-{suffix}"),
        )
        .await;
        let refund_resolution = resolve_case(
            &app,
            &platform_user_id,
            &refund_case_id,
            "refund_full",
            &format!("req-bil019-resolve-refund-{suffix}"),
        )
        .await;
        assert_eq!(
            refund_resolution["data"]["decision_code"].as_str(),
            Some("refund_full")
        );
        let refund = execute_refund(
            &app,
            &refund_order.order_id,
            &refund_case_id,
            &buyer_org_id,
            &buyer_user_id,
            &format!("req-bil019-refund-{suffix}"),
            &format!("refund:{refund_case_id}"),
        )
        .await;
        assert_eq!(refund["data"]["current_status"].as_str(), Some("succeeded"));

        let compensation_case = create_case(
            &app,
            &buyer_org_id,
            &buyer_user_id,
            &compensation_order.order_id,
            "compensation_full",
            &format!("req-bil019-case-compensation-{suffix}"),
        )
        .await;
        let compensation_case_id = compensation_case["data"]["case_id"]
            .as_str()
            .expect("compensation case id")
            .to_string();
        assert_dispute_frozen_snapshot(
            &client,
            &app,
            &compensation_order,
            &buyer_org_id,
            &format!("req-bil019-billing-compensation-frozen-{suffix}"),
        )
        .await;
        let compensation_resolution = resolve_case(
            &app,
            &platform_user_id,
            &compensation_case_id,
            "compensation_full",
            &format!("req-bil019-resolve-compensation-{suffix}"),
        )
        .await;
        assert_eq!(
            compensation_resolution["data"]["decision_code"].as_str(),
            Some("compensation_full")
        );
        let compensation = execute_compensation(
            &app,
            &compensation_order.order_id,
            &compensation_case_id,
            &buyer_org_id,
            &buyer_user_id,
            &format!("req-bil019-compensation-{suffix}"),
            &format!("compensation:{compensation_case_id}"),
        )
        .await;
        assert_eq!(
            compensation["data"]["current_status"].as_str(),
            Some("succeeded")
        );

        let refund_order_row = client
            .query_one(
                "SELECT settlement_status, dispute_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&refund_order.order_id],
            )
            .await
            .expect("query refund order row");
        assert_eq!(refund_order_row.get::<_, String>(0), "pending_settlement");
        assert_eq!(refund_order_row.get::<_, String>(1), "resolved");
        assert_eq!(
            refund_order_row.get::<_, Option<String>>(2).as_deref(),
            Some("billing_dispute_resolved")
        );
        let refund_settlement_row = client
            .query_one(
                "SELECT settlement_status, refund_amount::text, compensation_amount::text
                 FROM billing.settlement_record
                 WHERE settlement_id = $1::text::uuid",
                &[&refund_order.settlement_id],
            )
            .await
            .expect("query refund settlement row");
        assert_eq!(refund_settlement_row.get::<_, String>(0), "pending");
        assert_eq!(refund_settlement_row.get::<_, String>(1), "20.00000000");
        assert_eq!(refund_settlement_row.get::<_, String>(2), "0.00000000");

        let refund_detail = get_billing_order(
            &app,
            &refund_order.order_id,
            &buyer_org_id,
            &format!("req-bil019-billing-refund-{suffix}"),
        )
        .await;
        assert_eq!(
            refund_detail["data"]["refunds"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            refund_detail["data"]["settlement_summary"]["refund_adjustment_amount"].as_str(),
            Some("20.00000000")
        );
        assert_eq!(
            refund_detail["data"]["settlement_summary"]["summary_state"].as_str(),
            Some("order_settlement:pending:manual")
        );

        let compensation_detail = get_billing_order(
            &app,
            &compensation_order.order_id,
            &buyer_org_id,
            &format!("req-bil019-billing-compensation-{suffix}"),
        )
        .await;
        assert_eq!(
            compensation_detail["data"]["compensations"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            compensation_detail["data"]["settlement_summary"]["compensation_adjustment_amount"]
                .as_str(),
            Some("20.00000000")
        );
        assert_eq!(
            compensation_detail["data"]["settlement_summary"]["summary_state"].as_str(),
            Some("order_settlement:pending:manual")
        );
        let compensation_settlement_row = client
            .query_one(
                "SELECT settlement_status, refund_amount::text, compensation_amount::text
                 FROM billing.settlement_record
                 WHERE settlement_id = $1::text::uuid",
                &[&compensation_order.settlement_id],
            )
            .await
            .expect("query compensation settlement row");
        assert_eq!(compensation_settlement_row.get::<_, String>(0), "pending");
        assert_eq!(
            compensation_settlement_row.get::<_, String>(1),
            "0.00000000"
        );
        assert_eq!(
            compensation_settlement_row.get::<_, String>(2),
            "20.00000000"
        );

        let adjustment_counts = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint
                    FROM billing.billing_event
                    WHERE order_id = $1::text::uuid
                      AND event_type = 'refund_adjustment'
                      AND event_source = 'settlement_dispute_hold'),
                   (SELECT COUNT(*)::bigint
                    FROM billing.billing_event
                    WHERE order_id = $1::text::uuid
                      AND event_type = 'refund_adjustment'
                      AND event_source = 'settlement_dispute_release'),
                   (SELECT COUNT(*)::bigint
                    FROM billing.billing_event
                    WHERE order_id = $2::text::uuid
                      AND event_type = 'refund_adjustment'
                      AND event_source = 'settlement_dispute_hold'),
                   (SELECT COUNT(*)::bigint
                    FROM billing.billing_event
                    WHERE order_id = $2::text::uuid
                      AND event_type = 'refund_adjustment'
                      AND event_source = 'settlement_dispute_release')",
                &[&refund_order.order_id, &compensation_order.order_id],
            )
            .await
            .expect("query adjustment counts");
        assert_eq!(adjustment_counts.get::<_, i64>(0), 1);
        assert_eq!(adjustment_counts.get::<_, i64>(1), 1);
        assert_eq!(adjustment_counts.get::<_, i64>(2), 1);
        assert_eq!(adjustment_counts.get::<_, i64>(3), 1);

        let counts = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE aggregate_type = 'support.dispute_case' AND aggregate_id = $1::text::uuid),
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE aggregate_type = 'billing.refund_record' AND aggregate_id = $2::text::uuid),
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE aggregate_type = 'billing.compensation_record' AND aggregate_id = $3::text::uuid),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $4 AND action_name = 'dispute.case.create'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $5 AND action_name = 'dispute.case.resolve'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $6 AND action_name = 'billing.refund.execute'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $7 AND action_name = 'billing.compensation.execute')",
                &[
                    &refund_case_id,
                    &refund["data"]["refund_id"].as_str().expect("refund id"),
                    &compensation["data"]["compensation_id"].as_str().expect("compensation id"),
                    &format!("req-bil019-case-refund-{suffix}"),
                    &format!("req-bil019-resolve-refund-{suffix}"),
                    &format!("req-bil019-refund-{suffix}"),
                    &format!("req-bil019-compensation-{suffix}"),
                ],
            )
            .await
            .expect("query dispute integration counts");
        assert_eq!(counts.get::<_, i64>(0), 2);
        assert_eq!(counts.get::<_, i64>(1), 1);
        assert_eq!(counts.get::<_, i64>(2), 1);
        assert_eq!(counts.get::<_, i64>(3), 1);
        assert_eq!(counts.get::<_, i64>(4), 1);
        assert_eq!(counts.get::<_, i64>(5), 1);
        assert_eq!(counts.get::<_, i64>(6), 1);

        cleanup_dispute_flow(
            &client,
            &[&refund_order, &compensation_order],
            &[&refund_case_id, &compensation_case_id],
            &buyer_user_id,
            &platform_user_id,
            &[&buyer_org_id, &seller_org_id, &platform_org_id],
        )
        .await;
    }

    async fn assert_dispute_frozen_snapshot(
        client: &Client,
        app: &Router,
        order: &DisputeSeedOrder,
        tenant_id: &str,
        request_id: &str,
    ) {
        let order_row = client
            .query_one(
                "SELECT settlement_status, dispute_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await
            .expect("query frozen order row");
        assert_eq!(order_row.get::<_, String>(0), "frozen");
        assert_eq!(order_row.get::<_, String>(1), "opened");
        assert_eq!(
            order_row.get::<_, Option<String>>(2).as_deref(),
            Some("billing_dispute_linkage_applied")
        );

        let settlement_row = client
            .query_one(
                "SELECT settlement_status, reason_code
                 FROM billing.settlement_record
                 WHERE settlement_id = $1::text::uuid",
                &[&order.settlement_id],
            )
            .await
            .expect("query frozen settlement row");
        assert_eq!(settlement_row.get::<_, String>(0), "frozen");
        assert_eq!(
            settlement_row.get::<_, Option<String>>(1).as_deref(),
            Some("dispute_opened:delivery_failed")
        );

        let adjustment_row = client
            .query_one(
                "SELECT
                   COUNT(*) FILTER (
                     WHERE event_type = 'refund_adjustment'
                       AND event_source = 'settlement_dispute_hold'
                   )::bigint,
                   COUNT(*) FILTER (
                     WHERE event_type = 'refund_adjustment'
                       AND event_source = 'settlement_dispute_release'
                   )::bigint
                 FROM billing.billing_event
                 WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await
            .expect("query frozen adjustment row");
        assert_eq!(adjustment_row.get::<_, i64>(0), 1);
        assert_eq!(adjustment_row.get::<_, i64>(1), 0);

        let detail = get_billing_order(app, &order.order_id, tenant_id, request_id).await;
        assert_eq!(
            detail["data"]["settlement_summary"]["summary_state"].as_str(),
            Some("order_settlement:frozen:manual")
        );
    }

    async fn create_payment_intent(
        app: &Router,
        buyer_org_id: &str,
        seller_org_id: &str,
        provider_account_id: &str,
        corridor_policy_id: &str,
        order_id: &str,
        suffix: &str,
        scenario: &str,
        request_id: &str,
    ) -> String {
        let create_payload = format!(
            r#"{{
              "order_id":"{order_id}",
              "intent_type":"order_payment",
              "provider_key":"mock_payment",
              "provider_account_id":"{provider_account_id}",
              "payer_subject_type":"organization",
              "payer_subject_id":"{buyer_org_id}",
              "payee_subject_type":"organization",
              "payee_subject_id":"{seller_org_id}",
              "payer_jurisdiction_code":"SG",
              "payee_jurisdiction_code":"SG",
              "launch_jurisdiction_code":"SG",
              "corridor_policy_id":"{corridor_policy_id}",
              "payment_amount":"88.00",
              "price_currency_code":"USD",
              "currency_code":"SGD",
              "payment_method":"wallet",
              "expire_at":"2026-04-20T12:00:00Z",
              "metadata":{{"source":"bil019-db-smoke","scenario":"{scenario}","suffix":"{suffix}"}}
            }}"#
        );
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payments/intents")
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", buyer_org_id)
                    .header("x-step-up-token", "bil019-stepup")
                    .header("x-request-id", request_id)
                    .header(
                        "x-idempotency-key",
                        format!("pay:{order_id}:order_payment:{scenario}"),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(create_payload))
                    .expect("create payment intent request"),
            )
            .await
            .expect("create payment intent response");
        let json = json_response(response, StatusCode::OK).await;
        json["data"]["payment_intent_id"]
            .as_str()
            .expect("payment intent id")
            .to_string()
    }

    async fn lock_order(
        app: &Router,
        buyer_org_id: &str,
        order_id: &str,
        payment_intent_id: &str,
        request_id: &str,
    ) {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/lock"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", buyer_org_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"payment_intent_id":"{payment_intent_id}","lock_reason":"bil019_payment_integration"}}"#
                    )))
                    .expect("lock request"),
            )
            .await
            .expect("lock response");
        let _ = json_response(response, StatusCode::OK).await;
    }

    async fn post_webhook(app: &Router, provider: &str, request_id: &str, body: Value) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/payments/webhooks/{provider}"))
                    .header("x-provider-signature", "mock-signature")
                    .header("content-type", "application/json")
                    .header("x-request-id", request_id)
                    .body(Body::from(body.to_string()))
                    .expect("webhook request"),
            )
            .await
            .expect("webhook response");
        json_response(response, StatusCode::OK).await
    }

    async fn create_case(
        app: &Router,
        buyer_org_id: &str,
        buyer_user_id: &str,
        order_id: &str,
        resolution: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cases")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", buyer_org_id)
                    .header("x-user-id", buyer_user_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "order_id": order_id,
                            "reason_code": "delivery_failed",
                            "requested_resolution": resolution,
                            "claimed_amount": "20.00000000",
                            "evidence_scope": "delivery_receipt,download_log",
                            "blocking_effect": "freeze_settlement",
                            "metadata": {
                                "entry": "bil019"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("create case request"),
            )
            .await
            .expect("create case response");
        json_response(response, StatusCode::OK).await
    }

    async fn resolve_case(
        app: &Router,
        platform_user_id: &str,
        case_id: &str,
        decision_code: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/cases/{case_id}/resolve"))
                    .header("x-role", "platform_risk_settlement")
                    .header("x-user-id", platform_user_id)
                    .header("x-step-up-token", "bil019-stepup")
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "decision_type": "manual_resolution",
                            "decision_code": decision_code,
                            "liability_type": "seller",
                            "penalty_code": "seller_warning",
                            "decision_text": format!("{decision_code} approved by bil019"),
                            "metadata": {
                                "entry": "bil019"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("resolve case request"),
            )
            .await
            .expect("resolve case response");
        json_response(response, StatusCode::OK).await
    }

    async fn execute_refund(
        app: &Router,
        order_id: &str,
        case_id: &str,
        tenant_id: &str,
        user_id: &str,
        request_id: &str,
        idempotency_key: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/refunds")
                    .header("x-role", "platform_risk_settlement")
                    .header("x-tenant-id", tenant_id)
                    .header("x-user-id", user_id)
                    .header("x-request-id", request_id)
                    .header("x-idempotency-key", idempotency_key)
                    .header("x-step-up-token", "bil019-stepup")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "order_id": order_id,
                            "case_id": case_id,
                            "decision_code": "refund_full",
                            "amount": "20.00000000",
                            "reason_code": "delivery_failed",
                            "metadata": {
                                "entry": "bil019"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("refund request"),
            )
            .await
            .expect("refund response");
        json_response(response, StatusCode::OK).await
    }

    async fn execute_compensation(
        app: &Router,
        order_id: &str,
        case_id: &str,
        tenant_id: &str,
        user_id: &str,
        request_id: &str,
        idempotency_key: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/compensations")
                    .header("x-role", "platform_risk_settlement")
                    .header("x-tenant-id", tenant_id)
                    .header("x-user-id", user_id)
                    .header("x-request-id", request_id)
                    .header("x-idempotency-key", idempotency_key)
                    .header("x-step-up-token", "bil019-stepup")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "order_id": order_id,
                            "case_id": case_id,
                            "decision_code": "compensation_full",
                            "amount": "20.00000000",
                            "reason_code": "delivery_failed",
                            "metadata": {
                                "entry": "bil019"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("compensation request"),
            )
            .await
            .expect("compensation response");
        json_response(response, StatusCode::OK).await
    }

    async fn get_billing_order(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/billing/{order_id}"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .body(Body::empty())
                    .expect("billing read request"),
            )
            .await
            .expect("billing read response");
        json_response(response, StatusCode::OK).await
    }

    async fn json_response(response: axum::response::Response, expected: StatusCode) -> Value {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body bytes");
        assert_eq!(status, expected, "{}", String::from_utf8_lossy(&bytes));
        serde_json::from_slice(&bytes).expect("response json")
    }

    fn dsn_with_app_name(base_dsn: &str, app_name: &str) -> String {
        let separator = if base_dsn.contains('?') { '&' } else { '?' };
        format!("{base_dsn}{separator}application_name={app_name}")
    }

    async fn seeded_client(dsn: &str) -> Client {
        let db = AppDb::connect(
            DbPoolConfig {
                dsn: dsn.to_string(),
                max_connections: 2,
            }
            .into(),
        )
        .await
        .expect("connect seeded app db");
        db.client().expect("seed client")
    }

    async fn live_test_router(dsn: &str) -> Router {
        let db = AppDb::connect(
            DbPoolConfig {
                dsn: dsn.to_string(),
                max_connections: 8,
            }
            .into(),
        )
        .await
        .expect("connect app db");
        router().with_state(crate::AppState {
            runtime: RuntimeConfig::from_env().expect("test runtime config should load"),
            db: Arc::new(db),
        })
    }

    async fn seed_org(client: &Client, org_name: &str, org_type: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, $2, 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&org_name, &org_type],
            )
            .await
            .expect("insert org")
            .get(0)
    }

    async fn seed_user(client: &Client, org_id: &str, suffix: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status, email, attrs)
                 VALUES ($1::text::uuid, $2, $3, 'human', 'active', 'verified', $4, '{}'::jsonb)
                 RETURNING user_id::text",
                &[
                    &org_id,
                    &format!("bil019-user-{suffix}"),
                    &format!("BIL019 User {suffix}"),
                    &format!("bil019-{suffix}@example.com"),
                ],
            )
            .await
            .expect("insert user")
            .get(0)
    }

    async fn seed_provider_account(
        client: &Client,
        subject_id: &str,
        suffix: &str,
        provider_key: &str,
    ) -> String {
        client
            .query_one(
                "INSERT INTO payment.provider_account (
                   provider_key, account_scope, account_scope_id, account_name,
                   settlement_subject_type, settlement_subject_id, jurisdiction_code,
                   account_mode, status, config_json
                 ) VALUES (
                   $1, 'tenant', $2::text::uuid, $3,
                   'organization', $2::text::uuid, 'SG',
                   'sandbox', 'active', '{}'::jsonb
                 )
                 RETURNING provider_account_id::text",
                &[
                    &provider_key,
                    &subject_id,
                    &format!("bil019-account-{suffix}"),
                ],
            )
            .await
            .expect("insert provider account")
            .get(0)
    }

    async fn active_corridor_policy_id(client: &Client) -> String {
        client
            .query_one(
                "SELECT corridor_policy_id::text
                 FROM payment.corridor_policy
                 WHERE status = 'active'
                   AND COALESCE((policy_snapshot ->> 'real_payment_enabled')::boolean, false) = true
                 ORDER BY effective_from DESC NULLS LAST, created_at DESC
                 LIMIT 1",
                &[],
            )
            .await
            .expect("query active corridor")
            .get(0)
    }

    async fn seed_payable_order(
        client: &Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
        scenario: &str,
    ) -> PaymentSeedOrder {
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'active', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil019-asset-{scenario}-{suffix}"),
                    &format!("bil019 asset {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert payable asset")
            .get(0);
        let asset_version_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   2048, 'SG', ARRAY['SG']::text[], false,
                   '{"payment_mode":"online"}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await
            .expect("insert payable asset version")
            .get(0);
        let product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', 'one_time', 88.00, 'SGD', 'file_download',
                   ARRAY['billing_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil019-product-{scenario}-{suffix}"),
                    &format!("bil019 product {scenario} {suffix}"),
                    &format!("bil019 summary {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert payable product")
            .get(0);
        let sku_code = format!("BIL019-SKU-{scenario}-{suffix}");
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &sku_code],
            )
            .await
            .expect("insert payable sku")
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, fee_preview_snapshot, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'contract_effective', 'unpaid', 'not_started', 'not_started', 'pending_settlement', 'none',
                   'online', 88.00, 'SGD',
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'platform_fee_amount', '2.00',
                     'channel_fee_amount', '1.00',
                     'payable_total_amount', '88.00',
                     'currency_code', 'SGD'
                   ),
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'price_currency_code', 'USD',
                     'currency_code', 'SGD',
                     'captured_at', '1776659000000',
                     'source', 'bil019-db-smoke',
                     'scenario_snapshot', jsonb_build_object(
                       'scenario_code', 'S1',
                       'selected_sku_id', $5::text::uuid,
                       'selected_sku_code', $6,
                       'selected_sku_type', 'FILE_STD',
                       'selected_sku_role', 'primary',
                       'primary_sku_id', $5::text::uuid,
                       'primary_sku_code', $6,
                       'primary_sku_type', 'FILE_STD',
                       'supplementary_sku_ids', '[]'::jsonb,
                       'contract_template_code', 'CONTRACT_FILE_STD_V1',
                       'accept_template_code', 'ACCEPT_FILE_STD_V1',
                       'refund_policy_code', 'REFUND_FILE_STD_V1',
                       'flow_code', 'FILE_STD_STANDARD',
                       'per_sku_snapshot_required', true
                     )
                   ),
                   $7
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &sku_code,
                    &format!("bil019-order-{scenario}-{suffix}"),
                ],
            )
            .await
            .expect("insert payable order")
            .get(0);
        PaymentSeedOrder {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
        }
    }

    async fn seed_dispute_ready_order(
        client: &Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
        scenario: &str,
        refund_template: Option<&str>,
        compensation_template: Option<&str>,
    ) -> DisputeSeedOrder {
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description)
                 VALUES ($1::text::uuid, $2, 'finance', 'internal', 'active', $3)
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil019-dispute-asset-{scenario}-{suffix}"),
                    &format!("bil019 dispute asset {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert dispute asset")
            .get(0);
        let asset_version_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   2048, 'SG', ARRAY['SG']::text[], false,
                   '{"payment_mode":"online"}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await
            .expect("insert dispute asset version")
            .get(0);
        let product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', 'one_time', 88.00, 'SGD', 'file_download',
                   ARRAY['billing_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil019-dispute-product-{scenario}-{suffix}"),
                    &format!("bil019 dispute product {scenario} {suffix}"),
                    &format!("bil019 dispute summary {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert dispute product")
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("BIL019-DISPUTE-SKU-{scenario}-{suffix}")],
            )
            .await
            .expect("insert dispute sku")
            .get(0);
        let price_snapshot = json!({
            "product_id": product_id,
            "sku_id": sku_id,
            "sku_type": "FILE_STD",
            "selected_sku_type": "FILE_STD",
            "billing_mode": "one_time",
            "pricing_mode": "one_time",
            "settlement_basis": "gross_amount",
            "refund_mode": "manual_refund",
            "refund_template": refund_template,
            "compensation_template": compensation_template,
            "price_currency_code": "SGD"
        });
        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id,
                   asset_version_id,
                   buyer_org_id,
                   seller_org_id,
                   sku_id,
                   status,
                   payment_status,
                   delivery_status,
                   acceptance_status,
                   settlement_status,
                   dispute_status,
                   payment_mode,
                   amount,
                   currency_code,
                   price_snapshot_json,
                   delivery_route_snapshot,
                   trust_boundary_snapshot,
                   last_reason_code
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3::text::uuid,
                   $4::text::uuid,
                   $5::text::uuid,
                   'delivered',
                   'paid',
                   'delivered',
                   'pending_acceptance',
                   'pending_settlement',
                   'none',
                   'online',
                   88.00,
                   'SGD',
                   $6::jsonb,
                   'signed_url',
                   '{"delivery_mode":"file_download"}'::jsonb,
                   'bil019_seed_dispute'
                 ) RETURNING order_id::text"#,
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &price_snapshot,
                ],
            )
            .await
            .expect("insert dispute order")
            .get(0);
        let payment_intent_id: String = client
            .query_one(
                r#"INSERT INTO payment.payment_intent (
                   order_id, intent_type, provider_key, provider_account_id, payer_subject_type, payer_subject_id,
                   payee_subject_type, payee_subject_id, payer_jurisdiction_code, payee_jurisdiction_code,
                   launch_jurisdiction_code, amount, payment_method, currency_code, price_currency_code,
                   status, request_id, idempotency_key, capability_snapshot, metadata
                 ) VALUES (
                   $1::text::uuid, 'order_payment', 'mock_payment', NULL, 'organization', $2::text::uuid,
                   'organization', $3::text::uuid, 'SG', 'SG', 'SG', 88.00, 'wallet', 'SGD', 'SGD',
                   'succeeded', $4, $5, '{"supports_refund":true}'::jsonb, '{}'::jsonb
                 ) RETURNING payment_intent_id::text"#,
                &[
                    &order_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("bil019-pay-req-{scenario}-{suffix}"),
                    &format!("pay:bil019:{scenario}:{suffix}"),
                ],
            )
            .await
            .expect("insert dispute payment intent")
            .get(0);
        let settlement_id: String = client
            .query_one(
                "INSERT INTO billing.settlement_record (
                   order_id, settlement_type, settlement_status, settlement_mode,
                   payable_amount, platform_fee_amount, channel_fee_amount,
                   net_receivable_amount, refund_amount, compensation_amount,
                   reason_code, settled_at
                 ) VALUES (
                   $1::text::uuid, 'order_settlement', 'pending', 'manual',
                   88.00000000, 2.00000000, 1.00000000,
                   85.00000000, 0.00000000, 0.00000000,
                   'bil019_seed', NULL
                 ) RETURNING settlement_id::text",
                &[&order_id],
            )
            .await
            .expect("insert dispute settlement")
            .get(0);
        let delivery_id: String = client
            .query_one(
                r#"INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, delivery_commit_hash,
                   trust_boundary_snapshot, receipt_hash, committed_at, expires_at
                 ) VALUES (
                   $1::text::uuid, 'file_download', 'signed_url', 'committed', $2,
                   '{"delivery_mode":"file_download"}'::jsonb, $3, now() - interval '1 hour', now() + interval '7 days'
                 ) RETURNING delivery_id::text"#,
                &[
                    &order_id,
                    &format!("bil019-commit-{scenario}-{suffix}"),
                    &format!("bil019-receipt-{scenario}-{suffix}"),
                ],
            )
            .await
            .expect("insert dispute delivery")
            .get(0);
        let ticket_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_ticket (
                   order_id, buyer_org_id, token_hash, expire_at, download_limit, download_count, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, now() + interval '6 days', 5, 1, 'active'
                 ) RETURNING ticket_id::text",
                &[
                    &order_id,
                    &buyer_org_id,
                    &format!("bil019-ticket-{scenario}-{suffix}"),
                ],
            )
            .await
            .expect("insert dispute ticket")
            .get(0);
        DisputeSeedOrder {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            payment_intent_id,
            settlement_id,
            delivery_id,
            ticket_id,
        }
    }

    async fn current_rfc3339(client: &Client) -> String {
        client
            .query_one(
                "SELECT to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[],
            )
            .await
            .expect("current rfc3339")
            .get(0)
    }

    async fn cleanup_payment_flow(
        client: &Client,
        provider_account_id: &str,
        payment_intent_ids: &[&str],
        orders: &[&PaymentSeedOrder],
        buyer_org_id: &str,
        seller_org_id: &str,
    ) {
        for payment_intent_id in payment_intent_ids {
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_webhook_event WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_transaction WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
        }
        for order in orders {
            let _ = client
                .execute(
                    "DELETE FROM ops.outbox_event WHERE partition_key = $1 OR ordering_key = $1",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                    &[&order.sku_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                    &[&order.product_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                    &[&order.asset_version_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                    &[&order.asset_id],
                )
                .await;
        }
        let _ = client
            .execute(
                "DELETE FROM payment.provider_account WHERE provider_account_id = $1::text::uuid",
                &[&provider_account_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::uuid[])",
                &[&vec![buyer_org_id.to_string(), seller_org_id.to_string()]],
            )
            .await;
    }

    async fn cleanup_dispute_flow(
        client: &Client,
        orders: &[&DisputeSeedOrder],
        case_ids: &[&str],
        buyer_user_id: &str,
        platform_user_id: &str,
        org_ids: &[&str],
    ) {
        for case_id in case_ids {
            let _ = client
                .execute(
                    "DELETE FROM audit.legal_hold WHERE metadata ->> 'case_id' = $1",
                    &[case_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM support.dispute_case WHERE case_id = $1::text::uuid",
                    &[case_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM ops.outbox_event WHERE aggregate_id = $1::text::uuid",
                    &[case_id],
                )
                .await;
        }
        for order in orders {
            let _ = client
                .execute(
                    "DELETE FROM risk.governance_action_log WHERE freeze_ticket_id IN (
                       SELECT freeze_ticket_id FROM risk.freeze_ticket WHERE ref_type = 'order' AND ref_id = $1::text::uuid
                     )",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM risk.freeze_ticket WHERE ref_type = 'order' AND ref_id = $1::text::uuid",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM ops.outbox_event WHERE ordering_key = $1 OR partition_key = $1",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM billing.refund_record WHERE order_id = $1::text::uuid",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM billing.compensation_record WHERE order_id = $1::text::uuid",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.delivery_ticket WHERE ticket_id = $1::text::uuid",
                    &[&order.ticket_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.delivery_record WHERE delivery_id = $1::text::uuid",
                    &[&order.delivery_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM billing.settlement_record WHERE settlement_id = $1::text::uuid",
                    &[&order.settlement_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                    &[&order.payment_intent_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                    &[&order.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                    &[&order.sku_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                    &[&order.product_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                    &[&order.asset_version_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                    &[&order.asset_id],
                )
                .await;
        }
        let _ = client
            .execute(
                "DELETE FROM core.user_account WHERE user_id = ANY($1::uuid[])",
                &[&vec![
                    buyer_user_id.to_string(),
                    platform_user_id.to_string(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::uuid[])",
                &[&org_ids
                    .iter()
                    .map(|id| (*id).to_string())
                    .collect::<Vec<_>>()],
            )
            .await;
    }
}
