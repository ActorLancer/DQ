#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    struct SeedOrderGraph {
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn bil004_mock_payment_adapter_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        assert_eq!(
            std::env::var("MOCK_PAYMENT_ADAPTER_MODE").ok().as_deref(),
            Some("live"),
            "BIL-004 smoke must run with MOCK_PAYMENT_ADAPTER_MODE=live"
        );

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
        let buyer_org_id = seed_org(&client, &format!("bil004-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil004-seller-{suffix}")).await;
        let provider_account_id =
            seed_provider_account(&client, &seller_org_id, &suffix, "mock_payment").await;
        let corridor_policy_id = active_corridor_policy_id(&client).await;

        let success_order =
            seed_order(&client, &buyer_org_id, &seller_org_id, &suffix, "success").await;
        let fail_order = seed_order(&client, &buyer_org_id, &seller_org_id, &suffix, "fail").await;
        let timeout_order =
            seed_order(&client, &buyer_org_id, &seller_org_id, &suffix, "timeout").await;

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
            &format!("req-bil004-create-success-{suffix}"),
        )
        .await;
        let fail_intent_id = create_payment_intent(
            &app,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &corridor_policy_id,
            &fail_order.order_id,
            &suffix,
            "fail",
            &format!("req-bil004-create-fail-{suffix}"),
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
            &format!("req-bil004-create-timeout-{suffix}"),
        )
        .await;

        lock_order(
            &app,
            &buyer_org_id,
            &success_order.order_id,
            &success_intent_id,
            &format!("req-bil004-lock-success-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &fail_order.order_id,
            &fail_intent_id,
            &format!("req-bil004-lock-fail-{suffix}"),
        )
        .await;
        lock_order(
            &app,
            &buyer_org_id,
            &timeout_order.order_id,
            &timeout_intent_id,
            &format!("req-bil004-lock-timeout-{suffix}"),
        )
        .await;

        let success_json = simulate(
            &app,
            &buyer_org_id,
            &success_intent_id,
            "simulate-success",
            r#"{"duplicate_webhook":true}"#,
            &format!("req-bil004-sim-success-{suffix}"),
        )
        .await;
        assert_eq!(
            success_json["data"]["scenario_type"].as_str(),
            Some("success")
        );
        assert_eq!(success_json["data"]["provider_kind"].as_str(), Some("mock"));
        assert_eq!(
            success_json["data"]["provider_status"].as_str(),
            Some("succeeded")
        );
        assert_eq!(success_json["data"]["http_status_code"].as_u64(), Some(200));
        assert_eq!(
            success_json["data"]["webhook_processed_status"].as_str(),
            Some("processed")
        );
        assert_eq!(
            success_json["data"]["duplicate_processed_status"].as_str(),
            Some("duplicate")
        );
        assert_eq!(
            success_json["data"]["applied_payment_status"].as_str(),
            Some("succeeded")
        );

        let fail_json = simulate(
            &app,
            &buyer_org_id,
            &fail_intent_id,
            "simulate-fail",
            r#"{}"#,
            &format!("req-bil004-sim-fail-{suffix}"),
        )
        .await;
        assert_eq!(fail_json["data"]["scenario_type"].as_str(), Some("fail"));
        assert_eq!(
            fail_json["data"]["provider_status"].as_str(),
            Some("failed")
        );
        assert_eq!(fail_json["data"]["http_status_code"].as_u64(), Some(402));
        assert_eq!(
            fail_json["data"]["webhook_processed_status"].as_str(),
            Some("processed")
        );
        assert_eq!(
            fail_json["data"]["applied_payment_status"].as_str(),
            Some("failed")
        );

        let timeout_json = simulate(
            &app,
            &buyer_org_id,
            &timeout_intent_id,
            "simulate-timeout",
            r#"{}"#,
            &format!("req-bil004-sim-timeout-{suffix}"),
        )
        .await;
        assert_eq!(
            timeout_json["data"]["scenario_type"].as_str(),
            Some("timeout")
        );
        assert_eq!(
            timeout_json["data"]["provider_status"].as_str(),
            Some("timeout")
        );
        assert!(timeout_json["data"]["http_status_code"].is_null());
        assert_eq!(
            timeout_json["data"]["webhook_processed_status"].as_str(),
            Some("processed")
        );
        assert_eq!(
            timeout_json["data"]["applied_payment_status"].as_str(),
            Some("expired")
        );

        let payment_intent_ids = vec![
            success_intent_id.clone(),
            fail_intent_id.clone(),
            timeout_intent_id.clone(),
        ];
        let order_ids = vec![
            success_order.order_id.clone(),
            fail_order.order_id.clone(),
            timeout_order.order_id.clone(),
        ];

        let case_rows = client
            .query(
                "SELECT
                   payment_intent_id::text,
                   scenario_type,
                   status,
                   duplicate_webhook,
                   COALESCE(payload ->> 'webhook_processed_status', ''),
                   COALESCE(payload ->> 'duplicate_processed_status', ''),
                   COALESCE(payload ->> 'applied_payment_status', '')
                 FROM developer.mock_payment_case
                 WHERE payment_intent_id = ANY($1::uuid[])
                 ORDER BY scenario_type",
                &[&payment_intent_ids],
            )
            .await
            .expect("query mock payment case rows");
        assert_eq!(case_rows.len(), 3);

        let success_case = case_rows
            .iter()
            .find(|row| row.get::<_, String>(1) == "success")
            .expect("success case row");
        assert_eq!(success_case.get::<_, String>(0), success_intent_id);
        assert_eq!(success_case.get::<_, String>(2), "executed");
        assert!(success_case.get::<_, bool>(3));
        assert_eq!(success_case.get::<_, String>(4), "processed");
        assert_eq!(success_case.get::<_, String>(5), "duplicate");
        assert_eq!(success_case.get::<_, String>(6), "succeeded");

        let fail_case = case_rows
            .iter()
            .find(|row| row.get::<_, String>(1) == "fail")
            .expect("fail case row");
        assert_eq!(fail_case.get::<_, String>(2), "executed");
        assert!(!fail_case.get::<_, bool>(3));
        assert_eq!(fail_case.get::<_, String>(6), "failed");

        let timeout_case = case_rows
            .iter()
            .find(|row| row.get::<_, String>(1) == "timeout")
            .expect("timeout case row");
        assert_eq!(timeout_case.get::<_, String>(2), "executed");
        assert_eq!(timeout_case.get::<_, String>(6), "expired");

        let intent_rows = client
            .query(
                "SELECT payment_intent_id::text, status
                 FROM payment.payment_intent
                 WHERE payment_intent_id = ANY($1::uuid[])
                 ORDER BY payment_intent_id",
                &[&payment_intent_ids],
            )
            .await
            .expect("query payment intents");
        assert_eq!(intent_rows.len(), 3);
        assert_eq!(
            status_for(&intent_rows, &success_intent_id),
            Some("succeeded".to_string())
        );
        assert_eq!(
            status_for(&intent_rows, &fail_intent_id),
            Some("failed".to_string())
        );
        assert_eq!(
            status_for(&intent_rows, &timeout_intent_id),
            Some("expired".to_string())
        );

        let order_rows = client
            .query(
                "SELECT order_id::text, status, payment_status
                 FROM trade.order_main
                 WHERE order_id = ANY($1::uuid[])
                 ORDER BY order_id",
                &[&order_ids],
            )
            .await
            .expect("query orders");
        assert_eq!(
            order_status_for(&order_rows, &success_order.order_id),
            Some(("buyer_locked".to_string(), "paid".to_string()))
        );
        assert_eq!(
            order_status_for(&order_rows, &fail_order.order_id),
            Some((
                "payment_failed_pending_resolution".to_string(),
                "failed".to_string(),
            ))
        );
        assert_eq!(
            order_status_for(&order_rows, &timeout_order.order_id),
            Some((
                "payment_timeout_pending_compensation_cancel".to_string(),
                "expired".to_string(),
            ))
        );

        let webhook_rows = client
            .query(
                "SELECT payment_intent_id::text, processed_status, duplicate_flag
                 FROM payment.payment_webhook_event
                 WHERE payment_intent_id = ANY($1::uuid[])
                 ORDER BY payment_intent_id",
                &[&payment_intent_ids],
            )
            .await
            .expect("query webhook rows");
        assert_eq!(webhook_rows.len(), 3);
        assert_eq!(
            webhook_status_for(&webhook_rows, &success_intent_id),
            Some(("duplicate".to_string(), true))
        );
        assert_eq!(
            webhook_status_for(&webhook_rows, &fail_intent_id),
            Some(("processed".to_string(), false))
        );
        assert_eq!(
            webhook_status_for(&webhook_rows, &timeout_intent_id),
            Some(("processed".to_string(), false))
        );

        let audit_counts = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'mock.payment.simulate'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'mock.payment.simulate'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'mock.payment.simulate'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'payment.webhook.duplicate'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'payment.webhook.processed'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'payment.webhook.processed'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'payment.webhook.processed')",
                &[
                    &format!("req-bil004-sim-success-{suffix}"),
                    &format!("req-bil004-sim-fail-{suffix}"),
                    &format!("req-bil004-sim-timeout-{suffix}"),
                ],
            )
            .await
            .expect("query audit counts");
        assert!(audit_counts.get::<_, i64>(0) >= 1);
        assert!(audit_counts.get::<_, i64>(1) >= 1);
        assert!(audit_counts.get::<_, i64>(2) >= 1);
        assert!(audit_counts.get::<_, i64>(3) >= 1);
        assert!(audit_counts.get::<_, i64>(4) >= 1);
        assert!(audit_counts.get::<_, i64>(5) >= 1);
        assert!(audit_counts.get::<_, i64>(6) >= 1);

        cleanup(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &[&success_intent_id, &fail_intent_id, &timeout_intent_id],
            &[&success_order, &fail_order, &timeout_order],
            &[
                format!("req-bil004-create-success-{suffix}"),
                format!("req-bil004-create-fail-{suffix}"),
                format!("req-bil004-create-timeout-{suffix}"),
                format!("req-bil004-lock-success-{suffix}"),
                format!("req-bil004-lock-fail-{suffix}"),
                format!("req-bil004-lock-timeout-{suffix}"),
                format!("req-bil004-sim-success-{suffix}"),
                format!("req-bil004-sim-fail-{suffix}"),
                format!("req-bil004-sim-timeout-{suffix}"),
            ],
        )
        .await;
    }

    async fn create_payment_intent(
        app: &axum::Router,
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
              "metadata":{{"source":"bil004-db-smoke","scenario":"{scenario}","suffix":"{suffix}"}}
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
                    .header("x-step-up-token", "bil004-stepup")
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
        app: &axum::Router,
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
                        r#"{{"payment_intent_id":"{payment_intent_id}","lock_reason":"mock_payment_case"}}"#
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

    async fn simulate(
        app: &axum::Router,
        buyer_org_id: &str,
        payment_intent_id: &str,
        action: &str,
        payload: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/mock/payments/{payment_intent_id}/{action}"
                    ))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", buyer_org_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("simulate request should build"),
            )
            .await
            .expect("simulate response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("simulate body");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).expect("simulate response json")
    }

    fn status_for(rows: &[db::Row], payment_intent_id: &str) -> Option<String> {
        rows.iter()
            .find(|row| row.get::<_, String>(0) == payment_intent_id)
            .map(|row| row.get(1))
    }

    fn order_status_for(rows: &[db::Row], order_id: &str) -> Option<(String, String)> {
        rows.iter()
            .find(|row| row.get::<_, String>(0) == order_id)
            .map(|row| (row.get(1), row.get(2)))
    }

    fn webhook_status_for(rows: &[db::Row], payment_intent_id: &str) -> Option<(String, bool)> {
        rows.iter()
            .find(|row| row.get::<_, String>(0) == payment_intent_id)
            .map(|row| (row.get(1), row.get(2)))
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
                    &format!("bil004-account-{suffix}"),
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
                    &format!("bil004-asset-{scenario}-{suffix}"),
                    &format!("bil004 asset {scenario} {suffix}"),
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
                    &format!("bil004-product-{scenario}-{suffix}"),
                    &format!("bil004 product {scenario} {suffix}"),
                    &format!("bil004 summary {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert product")
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("BIL004-SKU-{scenario}-{suffix}")],
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
                     'source', 'bil004-db-smoke',
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
                    &format!("BIL004-SKU-{scenario}-{suffix}"),
                    &format!("bil004-order-{scenario}-{suffix}"),
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
                    "DELETE FROM developer.mock_payment_case WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
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
                    "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
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
