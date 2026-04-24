#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    struct SeedOrderGraph {
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn bil005_payment_webhook_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
            .to_string();
        let buyer_org_id = seed_org(&client, &format!("bil005-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil005-seller-{suffix}")).await;
        let provider_account_id =
            seed_provider_account(&client, &seller_org_id, &suffix, "mock_payment").await;
        let corridor_policy_id = active_corridor_policy_id(&client).await;

        let success_order =
            seed_order(&client, &buyer_org_id, &seller_org_id, &suffix, "success").await;
        let invalid_order =
            seed_order(&client, &buyer_org_id, &seller_org_id, &suffix, "invalid").await;
        let replay_order =
            seed_order(&client, &buyer_org_id, &seller_org_id, &suffix, "replay").await;
        let out_of_order_order = seed_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "out_of_order",
        )
        .await;
        let timeout_then_success_order = seed_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "timeout_then_success",
        )
        .await;

        let app = crate::with_live_test_state(router()).await;

        let success_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &success_order.order_id,
            &suffix,
            "success",
            &format!("req-bil005-create-success-{suffix}"),
        )
        .await;
        let invalid_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &invalid_order.order_id,
            &suffix,
            "invalid",
            &format!("req-bil005-create-invalid-{suffix}"),
        )
        .await;
        let replay_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &replay_order.order_id,
            &suffix,
            "replay",
            &format!("req-bil005-create-replay-{suffix}"),
        )
        .await;
        let out_of_order_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &out_of_order_order.order_id,
            &suffix,
            "out_of_order",
            &format!("req-bil005-create-ooo-{suffix}"),
        )
        .await;
        let timeout_then_success_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &timeout_then_success_order.order_id,
            &suffix,
            "timeout_then_success",
            &format!("req-bil005-create-timeout-then-success-{suffix}"),
        )
        .await;

        lock_order(
            &app,
            &buyer_org_id,
            &success_order.order_id,
            &success_intent_id,
            &format!("req-bil005-lock-success-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &invalid_order.order_id,
            &invalid_intent_id,
            &format!("req-bil005-lock-invalid-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &replay_order.order_id,
            &replay_intent_id,
            &format!("req-bil005-lock-replay-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &out_of_order_order.order_id,
            &out_of_order_intent_id,
            &format!("req-bil005-lock-ooo-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &timeout_then_success_order.order_id,
            &timeout_then_success_intent_id,
            &format!("req-bil005-lock-timeout-then-success-{suffix}"),
        )
        .await;

        let occurred_at_text = current_rfc3339(&client).await;
        let success_request_id = format!("req-bil005-webhook-success-{suffix}");
        let success_event_id = format!("evt-bil005-success-{suffix}");
        let success_json = post_webhook(
            &app,
            "mock_payment",
            Some("mock-signature"),
            None,
            &success_request_id,
            json!({
                "provider_event_id": success_event_id,
                "event_type": "payment.succeeded",
                "provider_transaction_no": format!("txn-bil005-success-{suffix}"),
                "payment_intent_id": success_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "succeeded",
                "occurred_at": occurred_at_text,
                "raw_payload": {
                    "source": "bil005-db-smoke-success",
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
            success_json["data"]["signature_verified"].as_bool(),
            Some(true)
        );
        assert_eq!(success_json["data"]["duplicate"].as_bool(), Some(false));
        assert_eq!(
            success_json["data"]["payment_intent_id"].as_str(),
            Some(success_intent_id.as_str())
        );
        let success_payment_transaction_id = success_json["data"]["payment_transaction_id"]
            .as_str()
            .expect("success payment transaction id")
            .to_string();
        assert_eq!(
            success_json["data"]["applied_payment_status"].as_str(),
            Some("succeeded")
        );

        let duplicate_request_id = format!("req-bil005-webhook-duplicate-{suffix}");
        let duplicate_json = post_webhook(
            &app,
            "mock_payment",
            Some("mock-signature"),
            Some(now_utc_ms()),
            &duplicate_request_id,
            json!({
                "provider_event_id": format!("evt-bil005-success-{suffix}"),
                "event_type": "payment.succeeded",
                "provider_transaction_no": format!("txn-bil005-success-{suffix}"),
                "payment_intent_id": success_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "succeeded",
                "raw_payload": {
                    "source": "bil005-db-smoke-duplicate"
                }
            }),
        )
        .await;
        assert_eq!(
            duplicate_json["data"]["processed_status"].as_str(),
            Some("duplicate")
        );
        assert_eq!(duplicate_json["data"]["duplicate"].as_bool(), Some(true));
        assert_eq!(
            duplicate_json["data"]["payment_transaction_id"].as_str(),
            Some(success_payment_transaction_id.as_str())
        );

        let invalid_request_id = format!("req-bil005-webhook-invalid-{suffix}");
        let invalid_json = post_webhook(
            &app,
            "mock_payment",
            Some("invalid-signature"),
            Some(now_utc_ms()),
            &invalid_request_id,
            json!({
                "provider_event_id": format!("evt-bil005-invalid-{suffix}"),
                "event_type": "payment.succeeded",
                "provider_transaction_no": format!("txn-bil005-invalid-{suffix}"),
                "payment_intent_id": invalid_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "succeeded",
                "raw_payload": {
                    "source": "bil005-db-smoke-invalid"
                }
            }),
        )
        .await;
        assert_eq!(
            invalid_json["data"]["processed_status"].as_str(),
            Some("rejected_signature")
        );
        assert_eq!(
            invalid_json["data"]["signature_verified"].as_bool(),
            Some(false)
        );
        assert!(invalid_json["data"]["payment_transaction_id"].is_null());

        let replay_request_id = format!("req-bil005-webhook-replay-{suffix}");
        let replay_timestamp_ms = now_utc_ms() - (16 * 60 * 1000);
        let replay_json = post_webhook(
            &app,
            "mock_payment",
            Some("mock-signature"),
            Some(replay_timestamp_ms),
            &replay_request_id,
            json!({
                "provider_event_id": format!("evt-bil005-replay-{suffix}"),
                "event_type": "payment.succeeded",
                "provider_transaction_no": format!("txn-bil005-replay-{suffix}"),
                "payment_intent_id": replay_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "succeeded",
                "occurred_at_ms": replay_timestamp_ms,
                "raw_payload": {
                    "source": "bil005-db-smoke-replay"
                }
            }),
        )
        .await;
        assert_eq!(
            replay_json["data"]["processed_status"].as_str(),
            Some("rejected_replay")
        );
        assert!(replay_json["data"]["payment_transaction_id"].is_null());

        let out_of_order_processed_request_id =
            format!("req-bil005-webhook-ooo-processed-{suffix}");
        let newer_ts = now_utc_ms();
        let out_of_order_processed_json = post_webhook(
            &app,
            "mock_payment",
            Some("mock-signature"),
            Some(newer_ts),
            &out_of_order_processed_request_id,
            json!({
                "provider_event_id": format!("evt-bil005-ooo-success-{suffix}"),
                "event_type": "payment.succeeded",
                "provider_transaction_no": format!("txn-bil005-ooo-success-{suffix}"),
                "payment_intent_id": out_of_order_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "succeeded",
                "occurred_at_ms": newer_ts,
                "raw_payload": {
                    "source": "bil005-db-smoke-ooo-success"
                }
            }),
        )
        .await;
        assert_eq!(
            out_of_order_processed_json["data"]["processed_status"].as_str(),
            Some("processed")
        );

        let out_of_order_request_id = format!("req-bil005-webhook-ooo-ignored-{suffix}");
        let older_ts = newer_ts - 5_000;
        let out_of_order_json = post_webhook(
            &app,
            "mock_payment",
            Some("mock-signature"),
            Some(older_ts),
            &out_of_order_request_id,
            json!({
                "provider_event_id": format!("evt-bil005-ooo-failed-{suffix}"),
                "event_type": "payment.failed",
                "provider_transaction_no": format!("txn-bil005-ooo-failed-{suffix}"),
                "payment_intent_id": out_of_order_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "failed",
                "occurred_at_ms": older_ts,
                "raw_payload": {
                    "source": "bil005-db-smoke-ooo-ignored"
                }
            }),
        )
        .await;
        assert_eq!(
            out_of_order_json["data"]["processed_status"].as_str(),
            Some("out_of_order_ignored")
        );
        assert_eq!(
            out_of_order_json["data"]["out_of_order_ignored"].as_bool(),
            Some(true)
        );
        assert!(out_of_order_json["data"]["payment_transaction_id"].is_null());

        let timeout_processed_request_id = format!("req-bil005-webhook-timeout-processed-{suffix}");
        let timeout_ts = now_utc_ms() + 5_000;
        let timeout_processed_json = post_webhook(
            &app,
            "mock_payment",
            Some("mock-signature"),
            Some(timeout_ts),
            &timeout_processed_request_id,
            json!({
                "provider_event_id": format!("evt-bil005-timeout-{suffix}"),
                "event_type": "payment.timeout",
                "provider_transaction_no": format!("txn-bil005-timeout-{suffix}"),
                "payment_intent_id": timeout_then_success_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "timeout",
                "occurred_at_ms": timeout_ts,
                "raw_payload": {
                    "source": "bil005-db-smoke-timeout-processed"
                }
            }),
        )
        .await;
        assert_eq!(
            timeout_processed_json["data"]["processed_status"].as_str(),
            Some("processed")
        );
        assert_eq!(
            timeout_processed_json["data"]["applied_payment_status"].as_str(),
            Some("expired")
        );

        let timeout_late_success_request_id =
            format!("req-bil005-webhook-timeout-late-success-{suffix}");
        let timeout_late_success_json = post_webhook(
            &app,
            "mock_payment",
            Some("mock-signature"),
            Some(timeout_ts + 1_000),
            &timeout_late_success_request_id,
            json!({
                "provider_event_id": format!("evt-bil005-timeout-late-success-{suffix}"),
                "event_type": "payment.succeeded",
                "provider_transaction_no": format!("txn-bil005-timeout-late-success-{suffix}"),
                "payment_intent_id": timeout_then_success_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "succeeded",
                "occurred_at_ms": timeout_ts + 1_000,
                "raw_payload": {
                    "source": "bil005-db-smoke-timeout-late-success"
                }
            }),
        )
        .await;
        assert_eq!(
            timeout_late_success_json["data"]["processed_status"].as_str(),
            Some("out_of_order_ignored")
        );
        assert_eq!(
            timeout_late_success_json["data"]["out_of_order_ignored"].as_bool(),
            Some(true)
        );
        assert!(timeout_late_success_json["data"]["payment_transaction_id"].is_null());

        let success_tx_row = client
            .query_one(
                "SELECT
                   provider_transaction_no,
                   provider_status,
                   to_char(amount, 'FM999999999999999990.00000000'),
                   currency_code,
                   COALESCE(raw_payload ->> 'source', ''),
                   payment_intent_id::text
                 FROM payment.payment_transaction
                 WHERE payment_intent_id = $1::text::uuid",
                &[&success_intent_id],
            )
            .await
            .expect("query success transaction row");
        let expected_success_tx = format!("txn-bil005-success-{suffix}");
        assert_eq!(
            success_tx_row.get::<_, Option<String>>(0).as_deref(),
            Some(expected_success_tx.as_str())
        );
        assert_eq!(
            success_tx_row.get::<_, Option<String>>(1).as_deref(),
            Some("succeeded")
        );
        assert_eq!(success_tx_row.get::<_, String>(2), "88.00000000");
        assert_eq!(success_tx_row.get::<_, String>(3), "SGD");
        assert_eq!(
            success_tx_row.get::<_, String>(4),
            "bil005-db-smoke-success"
        );
        assert_eq!(success_tx_row.get::<_, String>(5), success_intent_id);

        let success_webhook_row = client
            .query_one(
                "SELECT
                   processed_status,
                   duplicate_flag,
                   signature_verified,
                   payment_transaction_id::text,
                   COALESCE(payload ->> 'source', '')
                 FROM payment.payment_webhook_event
                 WHERE provider_key = 'mock_payment' AND provider_event_id = $1",
                &[&format!("evt-bil005-success-{suffix}")],
            )
            .await
            .expect("query success webhook row");
        assert_eq!(success_webhook_row.get::<_, String>(0), "duplicate");
        assert!(success_webhook_row.get::<_, bool>(1));
        assert!(success_webhook_row.get::<_, bool>(2));
        assert_eq!(
            success_webhook_row.get::<_, Option<String>>(3).as_deref(),
            Some(success_payment_transaction_id.as_str())
        );

        let success_tx_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM payment.payment_transaction WHERE payment_intent_id = $1::text::uuid",
                &[&success_intent_id],
            )
            .await
            .expect("count success tx")
            .get(0);
        assert_eq!(success_tx_count, 1);

        let invalid_webhook_row = client
            .query_one(
                "SELECT processed_status, signature_verified, payment_transaction_id::text
                 FROM payment.payment_webhook_event
                 WHERE provider_key = 'mock_payment' AND provider_event_id = $1",
                &[&format!("evt-bil005-invalid-{suffix}")],
            )
            .await
            .expect("query invalid webhook row");
        assert_eq!(
            invalid_webhook_row.get::<_, String>(0),
            "rejected_signature"
        );
        assert!(!invalid_webhook_row.get::<_, bool>(1));
        assert!(invalid_webhook_row.get::<_, Option<String>>(2).is_none());
        assert_eq!(
            count_transactions(&client, &invalid_intent_id).await,
            0,
            "invalid signature must not insert payment_transaction"
        );

        let replay_webhook_row = client
            .query_one(
                "SELECT processed_status, signature_verified, payment_transaction_id::text
                 FROM payment.payment_webhook_event
                 WHERE provider_key = 'mock_payment' AND provider_event_id = $1",
                &[&format!("evt-bil005-replay-{suffix}")],
            )
            .await
            .expect("query replay webhook row");
        assert_eq!(replay_webhook_row.get::<_, String>(0), "rejected_replay");
        assert!(replay_webhook_row.get::<_, bool>(1));
        assert!(replay_webhook_row.get::<_, Option<String>>(2).is_none());
        assert_eq!(count_transactions(&client, &replay_intent_id).await, 0);

        let out_of_order_status_row = client
            .query_one(
                "SELECT status, order_id::text FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&out_of_order_intent_id],
            )
            .await
            .expect("query ooo intent status");
        assert_eq!(out_of_order_status_row.get::<_, String>(0), "succeeded");
        assert_eq!(
            out_of_order_status_row.get::<_, String>(1),
            out_of_order_order.order_id
        );
        assert_eq!(
            count_transactions(&client, &out_of_order_intent_id).await,
            1
        );
        let out_of_order_order_row = client
            .query_one(
                "SELECT status, payment_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&out_of_order_order.order_id],
            )
            .await
            .expect("query ooo order status");
        assert_eq!(out_of_order_order_row.get::<_, String>(0), "buyer_locked");
        assert_eq!(out_of_order_order_row.get::<_, String>(1), "paid");

        let timeout_status_row = client
            .query_one(
                "SELECT status, order_id::text FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&timeout_then_success_intent_id],
            )
            .await
            .expect("query timeout intent status");
        assert_eq!(timeout_status_row.get::<_, String>(0), "expired");
        assert_eq!(
            timeout_status_row.get::<_, String>(1),
            timeout_then_success_order.order_id
        );
        assert_eq!(
            count_transactions(&client, &timeout_then_success_intent_id).await,
            1
        );
        let timeout_order_row = client
            .query_one(
                "SELECT status, payment_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&timeout_then_success_order.order_id],
            )
            .await
            .expect("query timeout order status");
        assert_eq!(
            timeout_order_row.get::<_, String>(0),
            "payment_timeout_pending_compensation_cancel"
        );
        assert_eq!(timeout_order_row.get::<_, String>(1), "expired");

        let audit_row = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'payment.webhook.processed'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'payment.webhook.duplicate'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'payment.webhook.rejected_signature'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $4 AND action_name = 'payment.webhook.rejected_replay'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $5 AND action_name = 'payment.webhook.out_of_order_ignored'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $6 AND action_name = 'payment.webhook.processed'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $7 AND action_name = 'payment.webhook.out_of_order_ignored')",
                &[
                    &success_request_id,
                    &duplicate_request_id,
                    &invalid_request_id,
                    &replay_request_id,
                    &out_of_order_request_id,
                    &timeout_processed_request_id,
                    &timeout_late_success_request_id,
                ],
            )
            .await
            .expect("query audit counts");
        assert_eq!(audit_row.get::<_, i64>(0), 1);
        assert_eq!(audit_row.get::<_, i64>(1), 1);
        assert_eq!(audit_row.get::<_, i64>(2), 1);
        assert_eq!(audit_row.get::<_, i64>(3), 1);
        assert_eq!(audit_row.get::<_, i64>(4), 1);
        assert_eq!(audit_row.get::<_, i64>(5), 1);
        assert_eq!(audit_row.get::<_, i64>(6), 1);

        cleanup(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &[
                &success_intent_id,
                &invalid_intent_id,
                &replay_intent_id,
                &out_of_order_intent_id,
                &timeout_then_success_intent_id,
            ],
            &[
                &success_order,
                &invalid_order,
                &replay_order,
                &out_of_order_order,
                &timeout_then_success_order,
            ],
            &[
                format!("req-bil005-create-success-{suffix}"),
                format!("req-bil005-create-invalid-{suffix}"),
                format!("req-bil005-create-replay-{suffix}"),
                format!("req-bil005-create-ooo-{suffix}"),
                format!("req-bil005-create-timeout-then-success-{suffix}"),
                format!("req-bil005-lock-success-{suffix}"),
                format!("req-bil005-lock-invalid-{suffix}"),
                format!("req-bil005-lock-replay-{suffix}"),
                format!("req-bil005-lock-ooo-{suffix}"),
                format!("req-bil005-lock-timeout-then-success-{suffix}"),
                success_request_id,
                duplicate_request_id,
                invalid_request_id,
                replay_request_id,
                out_of_order_processed_request_id,
                out_of_order_request_id,
                timeout_processed_request_id,
                timeout_late_success_request_id,
            ],
        )
        .await;
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
              "metadata":{{"source":"bil005-db-smoke","scenario":"{scenario}","suffix":"{suffix}"}}
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
                    .header("x-step-up-token", "bil005-stepup")
                    .header("x-request-id", request_id)
                    .header(
                        "x-idempotency-key",
                        format!("pay:{order_id}:order_payment:{scenario}"),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(create_payload))
                    .expect("create request should build"),
            )
            .await
            .expect("create payment intent response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("create payment intent body");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let json: Value = serde_json::from_slice(&body).expect("create payment intent json");
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
                        r#"{{"payment_intent_id":"{payment_intent_id}","lock_reason":"bil005_webhook_smoke"}}"#
                    )))
                    .expect("lock request should build"),
            )
            .await
            .expect("lock response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("lock body");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    }

    async fn post_webhook(
        app: &Router,
        provider: &str,
        signature: Option<&str>,
        header_timestamp_ms: Option<i64>,
        request_id: &str,
        body: Value,
    ) -> Value {
        let mut builder = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/payments/webhooks/{provider}"))
            .header("content-type", "application/json")
            .header("x-request-id", request_id);
        if let Some(signature) = signature {
            builder = builder.header("x-provider-signature", signature);
        }
        if let Some(timestamp_ms) = header_timestamp_ms {
            builder = builder.header("x-webhook-timestamp", timestamp_ms.to_string());
        }
        let response = app
            .clone()
            .oneshot(
                builder
                    .body(Body::from(body.to_string()))
                    .expect("webhook request should build"),
            )
            .await
            .expect("webhook response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("webhook body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("webhook json")
    }

    async fn count_transactions(client: &db::Client, payment_intent_id: &str) -> i64 {
        client
            .query_one(
                "SELECT COUNT(*)::bigint FROM payment.payment_transaction WHERE payment_intent_id = $1::text::uuid",
                &[&payment_intent_id],
            )
            .await
            .expect("count transactions")
            .get(0)
    }

    async fn current_rfc3339(client: &db::Client) -> String {
        client
            .query_one(
                "SELECT to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[],
            )
            .await
            .expect("current rfc3339")
            .get(0)
    }

    fn now_utc_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis() as i64
    }

    async fn seed_org(client: &db::Client, org_name: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&org_name],
            )
            .await
            .expect("insert org")
            .get(0)
    }

    async fn seed_provider_account(
        client: &db::Client,
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
                    &format!("bil005-account-{suffix}"),
                ],
            )
            .await
            .expect("insert provider account")
            .get(0)
    }

    async fn active_corridor_policy_id(client: &db::Client) -> String {
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

    async fn seed_order(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
        scenario: &str,
    ) -> SeedOrderGraph {
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
                    &format!("bil005-asset-{scenario}-{suffix}"),
                    &format!("bil005 asset {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert asset")
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
            .expect("insert asset version")
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
                    &format!("bil005-product-{scenario}-{suffix}"),
                    &format!("bil005 product {scenario} {suffix}"),
                    &format!("bil005 summary {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert product")
            .get(0);
        let sku_code = format!("BIL005-SKU-{scenario}-{suffix}");
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
            .expect("insert sku")
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
                     'source', 'bil005-db-smoke',
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
                    &format!("bil005-order-{scenario}-{suffix}"),
                ],
            )
            .await
            .expect("insert order")
            .get(0);
        SeedOrderGraph {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
        }
    }

    async fn cleanup(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        provider_account_id: &str,
        payment_intent_ids: &[&str],
        orders: &[&SeedOrderGraph],
        request_ids: &[String],
    ) {
        for request_id in request_ids {
            let _ = client
                .execute(
                    "DELETE FROM audit.audit_event WHERE request_id = $1",
                    &[request_id],
                )
                .await;
        }
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
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&buyer_org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seller_org_id],
            )
            .await;
    }
}
