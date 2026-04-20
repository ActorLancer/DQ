#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use super::super::super::repo::billing_event_repository::{
        RecordBillingEventRequest, list_billing_events_for_order, record_billing_event,
    };
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
    async fn bil006_billing_event_db_smoke() {
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
        let buyer_org_id = seed_org(&client, &format!("bil006-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil006-seller-{suffix}")).await;
        let provider_account_id =
            seed_provider_account(&client, &seller_org_id, &suffix, "mock_payment").await;
        let corridor_policy_id = active_corridor_policy_id(&client).await;

        let file_order = seed_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "file",
            "FILE_STD",
            "one_time",
            "one_time",
        )
        .await;
        let recurring_order = seed_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "recurring",
            "API_SUB",
            "subscription_cycle",
            "subscription",
        )
        .await;
        let usage_order = seed_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "usage",
            "API_PPU",
            "api_ppu",
            "usage_metered",
        )
        .await;

        let app = crate::with_live_test_state(router()).await;
        let success_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &file_order.order_id,
            &suffix,
            "file",
            &format!("req-bil006-create-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &file_order.order_id,
            &success_intent_id,
            &format!("req-bil006-lock-{suffix}"),
        )
        .await;
        let success_request_id = format!("req-bil006-webhook-success-{suffix}");
        let occurred_at = current_rfc3339(&client).await;
        let success_response = post_webhook(
            &app,
            "mock_payment",
            Some("mock-signature"),
            &success_request_id,
            json!({
                "provider_event_id": format!("evt-bil006-success-{suffix}"),
                "event_type": "payment.succeeded",
                "provider_transaction_no": format!("txn-bil006-success-{suffix}"),
                "payment_intent_id": success_intent_id,
                "transaction_amount": "88.00000000",
                "currency_code": "SGD",
                "provider_status": "succeeded",
                "occurred_at": occurred_at,
                "raw_payload": {
                    "source": "bil006-db-smoke",
                    "scenario": "payment-success"
                }
            }),
        )
        .await;
        assert_eq!(
            success_response["data"]["processed_status"].as_str(),
            Some("processed")
        );

        let recurring_request_id = format!("req-bil006-recurring-{suffix}");
        let (recurring_event, recurring_replayed) = record_billing_event(
            &client,
            &RecordBillingEventRequest {
                order_id: recurring_order.order_id.clone(),
                event_type: "recurring_charge".to_string(),
                event_source: "billing_cycle".to_string(),
                amount: Some("66.00000000".to_string()),
                currency_code: Some("SGD".to_string()),
                units: Some("1".to_string()),
                occurred_at: Some(current_rfc3339(&client).await),
                metadata: json!({
                    "idempotency_key": format!("bil006:recurring:{suffix}"),
                    "cycle_code": format!("2026-04:{suffix}"),
                    "reason_code": "subscription_cycle"
                }),
            },
            Some(buyer_org_id.as_str()),
            "platform_finance_operator",
            "billing.event.record",
            Some(recurring_request_id.as_str()),
            Some("trace-bil006-recurring"),
        )
        .await
        .expect("record recurring billing event");
        assert!(!recurring_replayed);
        assert_eq!(recurring_event.event_type, "recurring_charge");

        let usage_request_id = format!("req-bil006-usage-{suffix}");
        let usage_payload = RecordBillingEventRequest {
            order_id: usage_order.order_id.clone(),
            event_type: "usage_charge".to_string(),
            event_source: "usage_meter".to_string(),
            amount: Some("12.50000000".to_string()),
            currency_code: Some("SGD".to_string()),
            units: Some("128".to_string()),
            occurred_at: Some(current_rfc3339(&client).await),
            metadata: json!({
                "idempotency_key": format!("bil006:usage:{suffix}"),
                "reason_code": "api_usage_meter",
                "metered_quantity": "128"
            }),
        };
        let (usage_event, usage_replayed) = record_billing_event(
            &client,
            &usage_payload,
            Some(buyer_org_id.as_str()),
            "platform_finance_operator",
            "billing.event.record",
            Some(usage_request_id.as_str()),
            Some("trace-bil006-usage"),
        )
        .await
        .expect("record usage billing event");
        assert!(!usage_replayed);
        let usage_replay_request_id = format!("req-bil006-usage-replay-{suffix}");
        let (usage_replay_event, usage_replay_flag) = record_billing_event(
            &client,
            &usage_payload,
            Some(buyer_org_id.as_str()),
            "platform_finance_operator",
            "billing.event.record",
            Some(usage_replay_request_id.as_str()),
            Some("trace-bil006-usage-replay"),
        )
        .await
        .expect("replay usage billing event");
        assert!(usage_replay_flag);
        assert_eq!(
            usage_event.billing_event_id,
            usage_replay_event.billing_event_id
        );

        let refund_request_id = format!("req-bil006-refund-{suffix}");
        let (refund_event, refund_replayed) = record_billing_event(
            &client,
            &RecordBillingEventRequest {
                order_id: file_order.order_id.clone(),
                event_type: "refund".to_string(),
                event_source: "refund_execute".to_string(),
                amount: Some("8.00000000".to_string()),
                currency_code: Some("SGD".to_string()),
                units: None,
                occurred_at: Some(current_rfc3339(&client).await),
                metadata: json!({
                    "idempotency_key": format!("bil006:refund:{suffix}"),
                    "reason_code": "manual_refund_policy",
                    "refund_mode": "manual_refund"
                }),
            },
            Some(buyer_org_id.as_str()),
            "platform_risk_settlement",
            "billing.event.record",
            Some(refund_request_id.as_str()),
            Some("trace-bil006-refund"),
        )
        .await
        .expect("record refund billing event");
        assert!(!refund_replayed);
        assert_eq!(refund_event.event_type, "refund");

        let compensation_request_id = format!("req-bil006-compensation-{suffix}");
        let (comp_event, comp_replayed) = record_billing_event(
            &client,
            &RecordBillingEventRequest {
                order_id: file_order.order_id.clone(),
                event_type: "compensation".to_string(),
                event_source: "dispute_resolution".to_string(),
                amount: Some("5.00000000".to_string()),
                currency_code: Some("SGD".to_string()),
                units: None,
                occurred_at: Some(current_rfc3339(&client).await),
                metadata: json!({
                    "idempotency_key": format!("bil006:compensation:{suffix}"),
                    "reason_code": "sla_breach"
                }),
            },
            Some(buyer_org_id.as_str()),
            "platform_risk_settlement",
            "billing.event.record",
            Some(compensation_request_id.as_str()),
            Some("trace-bil006-compensation"),
        )
        .await
        .expect("record compensation billing event");
        assert!(!comp_replayed);
        assert_eq!(comp_event.event_type, "compensation");

        let manual_request_id = format!("req-bil006-manual-{suffix}");
        let (manual_event, manual_replayed) = record_billing_event(
            &client,
            &RecordBillingEventRequest {
                order_id: file_order.order_id.clone(),
                event_type: "manual_settlement".to_string(),
                event_source: "manual_transfer".to_string(),
                amount: Some("75.00000000".to_string()),
                currency_code: Some("SGD".to_string()),
                units: None,
                occurred_at: Some(current_rfc3339(&client).await),
                metadata: json!({
                    "idempotency_key": format!("bil006:manual:{suffix}"),
                    "reason_code": "finance_manual_instruction",
                    "settlement_direction": "payable"
                }),
            },
            Some(seller_org_id.as_str()),
            "platform_finance_operator",
            "billing.event.record",
            Some(manual_request_id.as_str()),
            Some("trace-bil006-manual"),
        )
        .await
        .expect("record manual settlement billing event");
        assert!(!manual_replayed);
        assert_eq!(manual_event.event_type, "manual_settlement");

        let file_events = list_billing_events_for_order(
            &client,
            &file_order.order_id,
            Some(buyer_org_id.as_str()),
            Some(success_request_id.as_str()),
        )
        .await
        .expect("list billing events for file order");
        assert_eq!(file_events.len(), 4);
        assert_eq!(file_events[0].event_type, "one_time_charge");
        assert_eq!(file_events[1].event_type, "refund");
        assert_eq!(file_events[2].event_type, "compensation");
        assert_eq!(file_events[3].event_type, "manual_settlement");

        let recurring_events = list_billing_events_for_order(
            &client,
            &recurring_order.order_id,
            Some(buyer_org_id.as_str()),
            Some(recurring_request_id.as_str()),
        )
        .await
        .expect("list recurring billing events");
        assert_eq!(recurring_events.len(), 1);
        assert_eq!(recurring_events[0].event_type, "recurring_charge");

        let usage_events = list_billing_events_for_order(
            &client,
            &usage_order.order_id,
            Some(buyer_org_id.as_str()),
            Some(usage_request_id.as_str()),
        )
        .await
        .expect("list usage billing events");
        assert_eq!(usage_events.len(), 1);
        assert_eq!(usage_events[0].event_type, "usage_charge");
        assert_eq!(usage_events[0].units.as_deref(), Some("128.00000000"));

        let order_ids = vec![
            file_order.order_id.clone(),
            recurring_order.order_id.clone(),
            usage_order.order_id.clone(),
        ];
        let summary_row = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = ANY($1::uuid[])),
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE event_type = 'billing.event.recorded' AND target_topic = 'billing.events' AND partition_key = ANY($2::text[])),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'billing.event.generated'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE action_name = 'billing.event.record.idempotent_replay' AND request_id = $4)",
                &[
                    &order_ids,
                    &order_ids,
                    &success_request_id,
                    &usage_replay_request_id,
                ],
            )
            .await
            .expect("query billing event summary");
        let summary = (
            summary_row.get::<_, i64>(0),
            summary_row.get::<_, i64>(1),
            summary_row.get::<_, i64>(2),
            summary_row.get::<_, i64>(3),
        );
        assert_eq!(summary.0, 6);
        assert_eq!(summary.1, 6);
        assert_eq!(summary.2, 1);
        assert_eq!(summary.3, 1);

        cleanup(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &[&success_intent_id],
            &[&file_order, &recurring_order, &usage_order],
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
              "metadata":{{"source":"bil006-db-smoke","scenario":"{scenario}","suffix":"{suffix}"}}
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
                    .header("x-step-up-token", "bil006-stepup")
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
                        r#"{{"payment_intent_id":"{payment_intent_id}","lock_reason":"bil006_billing_event_smoke"}}"#
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
                    &format!("bil006-account-{suffix}"),
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
        sku_type: &str,
        billing_mode: &str,
        pricing_mode: &str,
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
                    &format!("bil006-asset-{scenario}-{suffix}"),
                    &format!("bil006 asset {scenario} {suffix}"),
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
                    &format!("bil006-product-{scenario}-{suffix}"),
                    &format!("bil006 product {scenario} {suffix}"),
                    &format!("bil006 summary {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert product")
            .get(0);
        let sku_code = format!("BIL006-SKU-{scenario}-{suffix}");
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, $3, '份', $4, 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &sku_code, &sku_type, &billing_mode],
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
                     'pricing_mode', $7,
                     'platform_fee_amount', '2.00',
                     'channel_fee_amount', '1.00',
                     'payable_total_amount', '88.00',
                     'currency_code', 'SGD'
                   ),
                   jsonb_build_object(
                     'pricing_mode', $7,
                     'billing_mode', $8,
                     'settlement_basis', CASE WHEN $8 LIKE '%subscription%' THEN 'periodic' WHEN $8 LIKE '%ppu%' THEN 'usage' ELSE 'one_time' END,
                     'price_currency_code', 'USD',
                     'currency_code', 'SGD',
                     'captured_at', '1776659000000',
                     'source', 'bil006-db-smoke',
                     'scenario_snapshot', jsonb_build_object(
                       'scenario_code', CASE WHEN $6 = 'FILE_STD' THEN 'S1' WHEN $6 = 'API_SUB' THEN 'S4' ELSE 'S4' END,
                       'selected_sku_id', $5::text::uuid,
                       'selected_sku_code', $9,
                       'selected_sku_type', $6,
                       'selected_sku_role', 'primary',
                       'primary_sku_id', $5::text::uuid,
                       'primary_sku_code', $9,
                       'primary_sku_type', $6,
                       'supplementary_sku_ids', '[]'::jsonb,
                       'contract_template_code', 'CONTRACT_BIL006_V1',
                       'accept_template_code', 'ACCEPT_BIL006_V1',
                       'refund_policy_code', 'REFUND_BIL006_V1',
                       'flow_code', 'BIL006_STANDARD',
                       'per_sku_snapshot_required', true
                     )
                   ),
                   $10
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &sku_type,
                    &pricing_mode,
                    &billing_mode,
                    &sku_code,
                    &format!("bil006-order-{scenario}-{suffix}"),
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
