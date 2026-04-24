#[cfg(test)]
mod tests {
    use crate::modules::{billing, order};
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeededOrder {
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
        success: SeededOrder,
        failed: SeededOrder,
        timeout: SeededOrder,
        ignored: SeededOrder,
    }

    #[tokio::test]
    async fn trade030_payment_result_orchestrator_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix epoch")
                .as_millis()
        );
        let seed = seed_graph(&client, &suffix).await.expect("seed graph");
        let app = app().await;

        let success_request_id = format!("req-trade030-success-{suffix}");
        let failed_request_id = format!("req-trade030-failed-{suffix}");
        let timeout_request_id = format!("req-trade030-timeout-{suffix}");
        let ignored_request_id = format!("req-trade030-ignored-{suffix}");
        let base_timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_millis();

        let success_json = post_webhook(
            &app,
            &seed.success.payment_intent_id,
            &success_request_id,
            &format!("evt-trade030-success-{suffix}"),
            "payment.succeeded",
            "succeeded",
            base_timestamp_ms as i64,
        )
        .await;
        assert_success_envelope(&success_json, &success_request_id);
        assert_eq!(success_json["data"]["processed_status"], "processed");
        assert_eq!(success_json["data"]["applied_payment_status"], "succeeded");

        let failed_json = post_webhook(
            &app,
            &seed.failed.payment_intent_id,
            &failed_request_id,
            &format!("evt-trade030-failed-{suffix}"),
            "payment.failed",
            "failed",
            base_timestamp_ms as i64 + 1,
        )
        .await;
        assert_success_envelope(&failed_json, &failed_request_id);
        assert_eq!(failed_json["data"]["processed_status"], "processed");
        assert_eq!(failed_json["data"]["applied_payment_status"], "failed");

        let timeout_json = post_webhook(
            &app,
            &seed.timeout.payment_intent_id,
            &timeout_request_id,
            &format!("evt-trade030-timeout-{suffix}"),
            "payment.timeout",
            "timeout",
            base_timestamp_ms as i64 + 2,
        )
        .await;
        assert_success_envelope(&timeout_json, &timeout_request_id);
        assert_eq!(timeout_json["data"]["processed_status"], "processed");
        assert_eq!(timeout_json["data"]["applied_payment_status"], "expired");

        let ignored_json = post_webhook(
            &app,
            &seed.ignored.payment_intent_id,
            &ignored_request_id,
            &format!("evt-trade030-ignored-{suffix}"),
            "payment.failed",
            "failed",
            base_timestamp_ms as i64 + 3,
        )
        .await;
        assert_success_envelope(&ignored_json, &ignored_request_id);
        assert_eq!(ignored_json["data"]["processed_status"], "processed");
        assert_eq!(ignored_json["data"]["applied_payment_status"], "failed");

        let success_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, settlement_status, last_reason_code, buyer_locked_at IS NOT NULL
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.success.order_id],
            )
            .await
            .expect("query success order");
        assert_eq!(success_row.get::<_, String>(0), "buyer_locked");
        assert_eq!(success_row.get::<_, String>(1), "paid");
        assert_eq!(success_row.get::<_, String>(2), "pending_delivery");
        assert_eq!(success_row.get::<_, String>(3), "pending_settlement");
        assert_eq!(
            success_row.get::<_, Option<String>>(4).as_deref(),
            Some("payment_succeeded_to_buyer_locked")
        );
        assert!(success_row.get::<_, bool>(5));

        let failed_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, settlement_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.failed.order_id],
            )
            .await
            .expect("query failed order");
        assert_eq!(
            failed_row.get::<_, String>(0),
            "payment_failed_pending_resolution"
        );
        assert_eq!(failed_row.get::<_, String>(1), "failed");
        assert_eq!(failed_row.get::<_, String>(2), "pending_delivery");
        assert_eq!(failed_row.get::<_, String>(3), "not_started");
        assert_eq!(
            failed_row.get::<_, Option<String>>(4).as_deref(),
            Some("payment_failed_pending_resolution")
        );

        let timeout_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, settlement_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.timeout.order_id],
            )
            .await
            .expect("query timeout order");
        assert_eq!(
            timeout_row.get::<_, String>(0),
            "payment_timeout_pending_compensation_cancel"
        );
        assert_eq!(timeout_row.get::<_, String>(1), "expired");
        assert_eq!(timeout_row.get::<_, String>(2), "pending_delivery");
        assert_eq!(timeout_row.get::<_, String>(3), "not_started");
        assert_eq!(
            timeout_row.get::<_, Option<String>>(4).as_deref(),
            Some("payment_timeout_pending_compensation_cancel")
        );

        let ignored_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, settlement_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.ignored.order_id],
            )
            .await
            .expect("query ignored order");
        assert_eq!(ignored_row.get::<_, String>(0), "contract_pending");
        assert_eq!(ignored_row.get::<_, String>(1), "unpaid");
        assert_eq!(ignored_row.get::<_, String>(2), "not_started");
        assert_eq!(ignored_row.get::<_, String>(3), "not_started");
        assert_eq!(
            ignored_row.get::<_, Option<String>>(4).as_deref(),
            Some("trade030_seed_contract_pending")
        );

        let success_intent = client
            .query_one(
                "SELECT status FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&seed.success.payment_intent_id],
            )
            .await
            .expect("query success intent");
        assert_eq!(success_intent.get::<_, String>(0), "succeeded");

        let failed_intent = client
            .query_one(
                "SELECT status FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&seed.failed.payment_intent_id],
            )
            .await
            .expect("query failed intent");
        assert_eq!(failed_intent.get::<_, String>(0), "failed");

        let timeout_intent = client
            .query_one(
                "SELECT status FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&seed.timeout.payment_intent_id],
            )
            .await
            .expect("query timeout intent");
        assert_eq!(timeout_intent.get::<_, String>(0), "expired");

        let ignored_intent = client
            .query_one(
                "SELECT status FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&seed.ignored.payment_intent_id],
            )
            .await
            .expect("query ignored intent");
        assert_eq!(ignored_intent.get::<_, String>(0), "failed");

        assert_order_audit(
            &client,
            &success_request_id,
            &seed.success.order_id,
            "order.payment.result.applied",
            "success",
            Some("created"),
            Some("buyer_locked"),
        )
        .await;
        assert_order_audit(
            &client,
            &failed_request_id,
            &seed.failed.order_id,
            "order.payment.result.applied",
            "success",
            Some("contract_effective"),
            Some("payment_failed_pending_resolution"),
        )
        .await;
        assert_order_audit(
            &client,
            &timeout_request_id,
            &seed.timeout.order_id,
            "order.payment.result.applied",
            "success",
            Some("contract_effective"),
            Some("payment_timeout_pending_compensation_cancel"),
        )
        .await;
        assert_order_audit(
            &client,
            &ignored_request_id,
            &seed.ignored.order_id,
            "order.payment.result.ignored",
            "ignored",
            Some("contract_pending"),
            None,
        )
        .await;

        cleanup_seed_graph(
            &client,
            &seed,
            &[
                &success_request_id,
                &failed_request_id,
                &timeout_request_id,
                &ignored_request_id,
            ],
        )
        .await;
    }

    async fn app() -> Router {
        crate::with_live_test_state(
            Router::new()
                .merge(billing::api::router())
                .merge(order::api::router()),
        )
        .await
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

    fn assert_success_envelope(json: &Value, request_id: &str) {
        assert_eq!(json["code"].as_str(), Some("OK"));
        assert_eq!(json["message"].as_str(), Some("success"));
        assert_eq!(json["request_id"].as_str(), Some(request_id));
    }

    async fn assert_order_audit(
        client: &Client,
        request_id: &str,
        order_id: &str,
        action_name: &str,
        result_code: &str,
        previous_status: Option<&str>,
        next_status: Option<&str>,
    ) {
        let row = client
            .query_one(
                "SELECT action_name, result_code, metadata
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND ref_type = 'order'
                   AND ref_id = $2::text::uuid
                 ORDER BY event_time DESC
                 LIMIT 1",
                &[&request_id, &order_id],
            )
            .await
            .expect("query order audit");
        assert_eq!(row.get::<_, String>(0), action_name);
        assert_eq!(row.get::<_, String>(1), result_code);
        let metadata: serde_json::Value = row.get(2);
        assert_eq!(
            metadata["previous_status"].as_str(),
            previous_status,
            "unexpected previous_status for {request_id}"
        );
        assert_eq!(
            metadata["next_status"].as_str(),
            next_status,
            "unexpected next_status for {request_id}"
        );
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (
                   org_name, org_type, status, metadata
                 ) VALUES (
                   $1::text, 'enterprise', 'active', '{}'::jsonb
                 )
                 RETURNING org_id::text",
                &[&format!("trade030-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (
                   org_name, org_type, status, metadata
                 ) VALUES (
                   $1::text, 'enterprise', 'active', '{}'::jsonb
                 )
                 RETURNING org_id::text",
                &[&format!("trade030-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade030-asset-{suffix}"),
                    &format!("trade030 asset desc {suffix}"),
                ],
            )
            .await?
            .get(0);

        let asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text",
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 29.90, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade030-product-{suffix}"),
                    &format!("trade030 product desc {suffix}"),
                    &format!("trade030 search text {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE030-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let success = insert_seeded_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &sku_id,
            "created",
            "unpaid",
            "pending_delivery",
            "not_started",
            "trade030_seed_created",
            suffix,
            "success",
        )
        .await?;

        let failed = insert_seeded_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &sku_id,
            "contract_effective",
            "unpaid",
            "pending_delivery",
            "not_started",
            "trade030_seed_contract_effective_failed",
            suffix,
            "failed",
        )
        .await?;

        let timeout = insert_seeded_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &sku_id,
            "contract_effective",
            "unpaid",
            "pending_delivery",
            "not_started",
            "trade030_seed_contract_effective_timeout",
            suffix,
            "timeout",
        )
        .await?;

        let ignored = insert_seeded_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &sku_id,
            "contract_pending",
            "unpaid",
            "not_started",
            "not_started",
            "trade030_seed_contract_pending",
            suffix,
            "ignored",
        )
        .await?;

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            success,
            failed,
            timeout,
            ignored,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_seeded_order(
        client: &Client,
        product_id: &str,
        asset_version_id: &str,
        buyer_org_id: &str,
        seller_org_id: &str,
        sku_id: &str,
        order_status: &str,
        payment_status: &str,
        delivery_status: &str,
        settlement_status: &str,
        last_reason_code: &str,
        suffix: &str,
        label: &str,
    ) -> Result<SeededOrder, db::Error> {
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   $6, $7, $8, 'not_started', $9, 'none',
                   'online', 29.90, 'CNY',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', 'TRADE030-SKU',
                     'sku_type', 'FILE_STD',
                     'pricing_mode', 'one_time',
                     'unit_price', '29.90',
                     'currency_code', 'CNY',
                     'billing_mode', 'one_time',
                     'refund_mode', 'manual_refund',
                     'settlement_terms', jsonb_build_object('settlement_basis', 'one_time_final', 'settlement_mode', 'manual_v1'),
                     'tax_terms', jsonb_build_object('tax_policy', 'platform_default', 'tax_code', 'VAT', 'tax_inclusive', false),
                     'captured_at', '1776570000030',
                     'source', 'seed'
                   )::jsonb,
                   $10
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &order_status,
                    &payment_status,
                    &delivery_status,
                    &settlement_status,
                    &last_reason_code,
                ],
            )
            .await?
            .get(0);

        let payment_intent_id: String = client
            .query_one(
                "INSERT INTO payment.payment_intent (
                   order_id, intent_type, provider_key, payer_subject_type, payer_subject_id,
                   payee_subject_type, payee_subject_id, payer_jurisdiction_code, payee_jurisdiction_code,
                   launch_jurisdiction_code, amount, payment_method, currency_code, price_currency_code,
                   status, request_id, metadata
                 ) VALUES (
                   $1::text::uuid, 'order_payment', 'mock_payment', 'organization', $2::text::uuid,
                   'organization', $3::text::uuid, 'SG', 'SG', 'SG', 29.90, 'wallet', 'CNY', 'CNY',
                   'processing', $4, '{}'::jsonb
                 )
                 RETURNING payment_intent_id::text",
                &[
                    &order_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("req-trade030-intent-{label}-{suffix}"),
                ],
            )
            .await?
            .get(0);

        Ok(SeededOrder {
            order_id,
            payment_intent_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph, request_ids: &[&str]) {
        for request_id in request_ids {
            let _ = client
                .execute(
                    "DELETE FROM audit.audit_event WHERE request_id = $1",
                    &[request_id],
                )
                .await;
        }

        for payment_intent_id in [
            &seed.success.payment_intent_id,
            &seed.failed.payment_intent_id,
            &seed.timeout.payment_intent_id,
            &seed.ignored.payment_intent_id,
        ] {
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_webhook_event
                     WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_intent
                     WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
        }

        for order_id in [
            &seed.success.order_id,
            &seed.failed.order_id,
            &seed.timeout.order_id,
            &seed.ignored.order_id,
        ] {
            let _ = client
                .execute(
                    "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
        }

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
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid OR org_id = $2::text::uuid",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await;
    }
}
