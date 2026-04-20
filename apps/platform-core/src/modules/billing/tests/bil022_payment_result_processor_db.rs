#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeededPaymentIntent {
        order_id: String,
        payment_intent_id: String,
    }

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        poll_success: SeededPaymentIntent,
        webhook_success_then_poll_fail: SeededPaymentIntent,
        poll_timeout_then_webhook_success: SeededPaymentIntent,
    }

    #[tokio::test]
    async fn bil022_payment_result_processor_db_smoke() {
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
        let seed = seed_graph(&client, &suffix).await;
        let app = crate::with_live_test_state(router()).await;
        let base_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis() as i64;

        let poll_success_request_id = format!("req-bil022-poll-success-{suffix}");
        let poll_success_json = post_poll_result(
            &app,
            &seed.buyer_org_id,
            &seed.poll_success.payment_intent_id,
            &poll_success_request_id,
            json!({
                "provider_result_id": format!("poll-bil022-success-{suffix}"),
                "provider_transaction_no": format!("txn-bil022-success-{suffix}"),
                "provider_status": "succeeded",
                "transaction_amount": "88.00",
                "currency_code": "SGD",
                "occurred_at_ms": base_ms,
                "raw_payload": {"source":"bil022-poll-success"}
            }),
        )
        .await;
        assert_eq!(poll_success_json["data"]["processed_status"], "processed");
        assert_eq!(poll_success_json["data"]["duplicate"], false);
        assert_eq!(
            poll_success_json["data"]["applied_payment_status"],
            "succeeded"
        );

        let poll_duplicate_request_id = format!("req-bil022-poll-duplicate-{suffix}");
        let poll_duplicate_json = post_poll_result(
            &app,
            &seed.buyer_org_id,
            &seed.poll_success.payment_intent_id,
            &poll_duplicate_request_id,
            json!({
                "provider_result_id": format!("poll-bil022-success-{suffix}"),
                "provider_transaction_no": format!("txn-bil022-success-{suffix}"),
                "provider_status": "succeeded",
                "occurred_at_ms": base_ms + 1,
                "raw_payload": {"source":"bil022-poll-duplicate"}
            }),
        )
        .await;
        assert_eq!(poll_duplicate_json["data"]["processed_status"], "duplicate");
        assert_eq!(poll_duplicate_json["data"]["duplicate"], true);

        let webhook_success_request_id = format!("req-bil022-webhook-success-{suffix}");
        let webhook_success_json = post_webhook(
            &app,
            &seed.webhook_success_then_poll_fail.payment_intent_id,
            &webhook_success_request_id,
            &format!("evt-bil022-success-{suffix}"),
            "payment.succeeded",
            "succeeded",
            base_ms + 2,
        )
        .await;
        assert_eq!(
            webhook_success_json["data"]["processed_status"],
            "processed"
        );
        assert_eq!(
            webhook_success_json["data"]["applied_payment_status"],
            "succeeded"
        );

        let poll_fail_request_id = format!("req-bil022-poll-fail-{suffix}");
        let poll_fail_json = post_poll_result(
            &app,
            &seed.buyer_org_id,
            &seed.webhook_success_then_poll_fail.payment_intent_id,
            &poll_fail_request_id,
            json!({
                "provider_result_id": format!("poll-bil022-fail-{suffix}"),
                "provider_transaction_no": format!("txn-bil022-fail-{suffix}"),
                "provider_status": "failed",
                "occurred_at_ms": base_ms + 3,
                "raw_payload": {"source":"bil022-poll-fail"}
            }),
        )
        .await;
        assert_eq!(
            poll_fail_json["data"]["processed_status"],
            "out_of_order_ignored"
        );
        assert_eq!(poll_fail_json["data"]["out_of_order_ignored"], true);

        let poll_timeout_request_id = format!("req-bil022-poll-timeout-{suffix}");
        let poll_timeout_json = post_poll_result(
            &app,
            &seed.buyer_org_id,
            &seed.poll_timeout_then_webhook_success.payment_intent_id,
            &poll_timeout_request_id,
            json!({
                "provider_result_id": format!("poll-bil022-timeout-{suffix}"),
                "provider_transaction_no": format!("txn-bil022-timeout-{suffix}"),
                "provider_status": "timeout",
                "occurred_at_ms": base_ms + 4,
                "raw_payload": {"source":"bil022-poll-timeout"}
            }),
        )
        .await;
        assert_eq!(poll_timeout_json["data"]["processed_status"], "processed");
        assert_eq!(
            poll_timeout_json["data"]["applied_payment_status"],
            "expired"
        );

        let webhook_late_success_request_id = format!("req-bil022-webhook-late-success-{suffix}");
        let webhook_late_success_json = post_webhook(
            &app,
            &seed.poll_timeout_then_webhook_success.payment_intent_id,
            &webhook_late_success_request_id,
            &format!("evt-bil022-late-success-{suffix}"),
            "payment.succeeded",
            "succeeded",
            base_ms + 5,
        )
        .await;
        assert_eq!(
            webhook_late_success_json["data"]["processed_status"],
            "out_of_order_ignored"
        );
        assert_eq!(
            webhook_late_success_json["data"]["out_of_order_ignored"],
            true
        );

        assert_order_state(
            &client,
            &seed.poll_success.order_id,
            "buyer_locked",
            "paid",
            Some("payment_succeeded_to_buyer_locked"),
        )
        .await;
        assert_intent_state(
            &client,
            &seed.poll_success.payment_intent_id,
            "succeeded",
            Some("polling"),
        )
        .await;

        assert_order_state(
            &client,
            &seed.webhook_success_then_poll_fail.order_id,
            "buyer_locked",
            "paid",
            Some("payment_succeeded_to_buyer_locked"),
        )
        .await;
        assert_intent_state(
            &client,
            &seed.webhook_success_then_poll_fail.payment_intent_id,
            "succeeded",
            Some("webhook"),
        )
        .await;

        assert_order_state(
            &client,
            &seed.poll_timeout_then_webhook_success.order_id,
            "payment_timeout_pending_compensation_cancel",
            "expired",
            Some("payment_timeout_pending_compensation_cancel"),
        )
        .await;
        assert_intent_state(
            &client,
            &seed.poll_timeout_then_webhook_success.payment_intent_id,
            "expired",
            Some("polling"),
        )
        .await;

        assert_eq!(
            count_transactions(&client, &seed.poll_success.payment_intent_id).await,
            1
        );
        assert_eq!(
            count_transactions(
                &client,
                &seed.webhook_success_then_poll_fail.payment_intent_id
            )
            .await,
            1
        );
        assert_eq!(
            count_transactions(
                &client,
                &seed.poll_timeout_then_webhook_success.payment_intent_id
            )
            .await,
            1
        );

        let outbox_counts = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE partition_key = $1 AND target_topic = 'billing.events'),
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE partition_key = $2 AND target_topic = 'billing.events'),
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE partition_key = $3 AND target_topic = 'billing.events')",
                &[
                    &seed.poll_success.order_id,
                    &seed.webhook_success_then_poll_fail.order_id,
                    &seed.poll_timeout_then_webhook_success.order_id,
                ],
            )
            .await
            .expect("query outbox counts");
        assert_eq!(outbox_counts.get::<_, i64>(0), 1);
        assert_eq!(outbox_counts.get::<_, i64>(1), 1);
        assert_eq!(outbox_counts.get::<_, i64>(2), 0);

        let audit_counts = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'payment.polling.processed'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'payment.polling.duplicate'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'payment.webhook.processed'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $4 AND action_name = 'payment.polling.out_of_order_ignored'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $5 AND action_name = 'payment.webhook.out_of_order_ignored')",
                &[
                    &poll_success_request_id,
                    &poll_duplicate_request_id,
                    &webhook_success_request_id,
                    &poll_fail_request_id,
                    &webhook_late_success_request_id,
                ],
            )
            .await
            .expect("query audit counts");
        assert_eq!(audit_counts.get::<_, i64>(0), 1);
        assert_eq!(audit_counts.get::<_, i64>(1), 1);
        assert_eq!(audit_counts.get::<_, i64>(2), 1);
        assert_eq!(audit_counts.get::<_, i64>(3), 1);
        assert_eq!(audit_counts.get::<_, i64>(4), 1);

        cleanup_seed_graph(
            &client,
            &seed,
            &[
                &poll_success_request_id,
                &poll_duplicate_request_id,
                &webhook_success_request_id,
                &poll_fail_request_id,
                &poll_timeout_request_id,
                &webhook_late_success_request_id,
            ],
        )
        .await;
    }

    async fn post_poll_result(
        app: &Router,
        tenant_id: &str,
        payment_intent_id: &str,
        request_id: &str,
        body: Value,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/payments/intents/{payment_intent_id}/poll-result"
                    ))
                    .header("content-type", "application/json")
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .body(Body::from(body.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        serde_json::from_slice(&body).expect("json")
    }

    async fn post_webhook(
        app: &Router,
        payment_intent_id: &str,
        request_id: &str,
        provider_event_id: &str,
        event_type: &str,
        provider_status: &str,
        occurred_at_ms: i64,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payments/webhooks/mock_payment")
                    .header("content-type", "application/json")
                    .header("x-provider-signature", "mock-signature")
                    .header("x-webhook-timestamp", occurred_at_ms.to_string())
                    .header("x-request-id", request_id)
                    .body(Body::from(format!(
                        r#"{{
                          "provider_event_id":"{}",
                          "event_type":"{}",
                          "payment_intent_id":"{}",
                          "provider_status":"{}",
                          "occurred_at_ms":{}
                        }}"#,
                        provider_event_id,
                        event_type,
                        payment_intent_id,
                        provider_status,
                        occurred_at_ms
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        serde_json::from_slice(&body).expect("json")
    }

    async fn assert_order_state(
        client: &Client,
        order_id: &str,
        expected_status: &str,
        expected_payment_status: &str,
        expected_reason: Option<&str>,
    ) {
        let row = client
            .query_one(
                "SELECT status, payment_status, last_reason_code FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&order_id],
            )
            .await
            .expect("query order state");
        assert_eq!(row.get::<_, String>(0), expected_status);
        assert_eq!(row.get::<_, String>(1), expected_payment_status);
        assert_eq!(row.get::<_, Option<String>>(2).as_deref(), expected_reason);
    }

    async fn assert_intent_state(
        client: &Client,
        payment_intent_id: &str,
        expected_status: &str,
        expected_source: Option<&str>,
    ) {
        let row = client
            .query_one(
                "SELECT status, metadata ->> 'payment_result_last_source' FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&payment_intent_id],
            )
            .await
            .expect("query payment intent state");
        assert_eq!(row.get::<_, String>(0), expected_status);
        assert_eq!(row.get::<_, Option<String>>(1).as_deref(), expected_source);
    }

    async fn count_transactions(client: &Client, payment_intent_id: &str) -> i64 {
        client
            .query_one(
                "SELECT COUNT(*)::bigint FROM payment.payment_transaction WHERE payment_intent_id = $1::text::uuid",
                &[&payment_intent_id],
            )
            .await
            .expect("count transactions")
            .get(0)
    }

    async fn seed_graph(client: &Client, suffix: &str) -> SeedGraph {
        let buyer_org_id = seed_org(client, &format!("bil022-buyer-{suffix}")).await;
        let seller_org_id = seed_org(client, &format!("bil022-seller-{suffix}")).await;
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description)
                 VALUES ($1::text::uuid, $2, 'finance', 'internal', 'active', $3)
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil022-asset-{suffix}"),
                    &format!("bil022 asset {suffix}"),
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
                    &format!("bil022-product-{suffix}"),
                    &format!("bil022 product {suffix}"),
                    &format!("bil022 summary {suffix}"),
                ],
            )
            .await
            .expect("insert product")
            .get(0);
        let sku_code = format!("BIL022-SKU-{suffix}");
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'auto_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &sku_code],
            )
            .await
            .expect("insert sku")
            .get(0);

        let poll_success = seed_payment_intent(
            client,
            &buyer_org_id,
            &seller_org_id,
            &asset_version_id,
            &product_id,
            &sku_id,
            &sku_code,
            suffix,
            "poll_success",
        )
        .await;
        let webhook_success_then_poll_fail = seed_payment_intent(
            client,
            &buyer_org_id,
            &seller_org_id,
            &asset_version_id,
            &product_id,
            &sku_id,
            &sku_code,
            suffix,
            "webhook_then_poll_fail",
        )
        .await;
        let poll_timeout_then_webhook_success = seed_payment_intent(
            client,
            &buyer_org_id,
            &seller_org_id,
            &asset_version_id,
            &product_id,
            &sku_id,
            &sku_code,
            suffix,
            "poll_timeout_then_webhook_success",
        )
        .await;

        SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            poll_success,
            webhook_success_then_poll_fail,
            poll_timeout_then_webhook_success,
        }
    }

    async fn seed_payment_intent(
        client: &Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        asset_version_id: &str,
        product_id: &str,
        sku_id: &str,
        sku_code: &str,
        suffix: &str,
        scenario: &str,
    ) -> SeededPaymentIntent {
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, payment_channel_snapshot, fee_preview_snapshot, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'created', 'locked', 'not_started', 'not_started', 'pending_settlement', 'none',
                   'online', 88.00, 'SGD',
                   '{}'::jsonb,
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'platform_fee_amount', '2.00',
                     'channel_fee_amount', '1.00',
                     'payable_total_amount', '88.00',
                     'currency_code', 'SGD'
                   ),
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'billing_mode', 'one_time',
                     'settlement_basis', 'gross_amount',
                     'price_currency_code', 'USD',
                     'currency_code', 'SGD',
                     'selected_sku_type', 'FILE_STD',
                     'scenario_snapshot', jsonb_build_object(
                       'scenario_code', 'S2',
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
                 ) RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &sku_code,
                    &format!("bil022-order-{scenario}-{suffix}"),
                ],
            )
            .await
            .expect("insert order")
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
                   'organization', $3::text::uuid, 'SG', 'SG', 'SG', 88.00, 'wallet', 'SGD', 'USD',
                   'processing', $4, $5, '{"supports_refund":true,"supports_webhook":true}'::jsonb, '{}'::jsonb
                 ) RETURNING payment_intent_id::text"#,
                &[
                    &order_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("bil022-pay-req-{scenario}-{suffix}"),
                    &format!("pay:bil022:{scenario}:{suffix}"),
                ],
            )
            .await
            .expect("insert payment intent")
            .get(0);
        client
            .execute(
                "UPDATE trade.order_main
                 SET payment_channel_snapshot = jsonb_build_object(
                   'payment_intent_id', $2::text,
                   'provider_key', 'mock_payment',
                   'payment_amount', '88.00',
                   'currency_code', 'SGD'
                 )
                 WHERE order_id = $1::text::uuid",
                &[&order_id, &payment_intent_id],
            )
            .await
            .expect("link payment channel snapshot");
        SeededPaymentIntent {
            order_id,
            payment_intent_id,
        }
    }

    async fn seed_org(client: &Client, name: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb)
                 RETURNING org_id::text",
                &[&name],
            )
            .await
            .expect("insert org")
            .get(0)
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph, _request_ids: &[&str]) {
        for payment_intent_id in [
            &seed.poll_success.payment_intent_id,
            &seed.webhook_success_then_poll_fail.payment_intent_id,
            &seed.poll_timeout_then_webhook_success.payment_intent_id,
        ] {
            client
                .execute(
                    "DELETE FROM payment.payment_webhook_event WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await
                .expect("delete webhook rows");
            client
                .execute(
                    "DELETE FROM payment.payment_transaction WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await
                .expect("delete payment transactions");
            client
                .execute(
                    "DELETE FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await
                .expect("delete payment intent");
        }
        for order_id in [
            &seed.poll_success.order_id,
            &seed.webhook_success_then_poll_fail.order_id,
            &seed.poll_timeout_then_webhook_success.order_id,
        ] {
            client
                .execute(
                    "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await
                .expect("delete billing events");
            client
                .execute(
                    "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await
                .expect("delete settlement record");
            client
                .execute(
                    "DELETE FROM ops.outbox_event WHERE partition_key = $1 OR ordering_key = $1 OR aggregate_id = $1::text::uuid",
                    &[order_id],
                )
                .await
                .expect("delete outbox rows");
            client
                .execute(
                    "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await
                .expect("delete orders");
        }
        client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&seed.sku_id],
            )
            .await
            .expect("delete sku");
        client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await
            .expect("delete product");
        client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
            )
            .await
            .expect("delete asset version");
        client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&seed.asset_id],
            )
            .await
            .expect("delete asset");
        client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await
            .expect("delete orgs");
    }
}
