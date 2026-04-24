#[cfg(test)]
mod tests {
    use super::super::super::api::router as billing_router;
    use crate::modules::delivery::api::router as delivery_router;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use std::sync::OnceLock;
    use tokio::sync::Mutex;
    use tower::util::ServiceExt;

    fn smoke_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct RejectSeed {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        delivery_id: String,
    }

    struct ManualAdjustmentSeed {
        buyer_org_id: String,
        seller_org_id: String,
        platform_org_id: String,
        buyer_user_id: String,
        platform_user_id: String,
        finance_user_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        payment_intent_id: String,
        settlement_id: String,
        provider_account_id: String,
        payout_preference_id: String,
    }

    #[tokio::test]
    async fn bil025_billing_adjustment_freeze_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let _guard = smoke_lock().lock().await;
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_millis()
            .to_string();
        let seed = seed_reject_order(&client, &suffix)
            .await
            .expect("seed reject order");

        let app = crate::with_live_test_state(delivery_router()).await;
        let request_id = format!("req-bil025-reject-{suffix}");
        let reject = reject_order(
            &app,
            &seed.buyer_org_id,
            &seed.order_id,
            &request_id,
            "report_quality_failed",
        )
        .await;
        let data = &reject["data"];
        assert_eq!(data["current_state"].as_str(), Some("rejected"));
        assert_eq!(data["settlement_status"].as_str(), Some("blocked"));
        assert_eq!(data["dispute_status"].as_str(), Some("open"));

        let adjustment_row = client
            .query_one(
                "SELECT COUNT(*)::bigint,
                        COALESCE(SUM(amount), 0)::float8,
                        MIN(event_source),
                        MIN(metadata ->> 'adjustment_class')
                 FROM billing.billing_event
                 WHERE order_id = $1::text::uuid
                   AND event_type = 'refund_adjustment'
                   AND event_source = 'settlement_dispute_hold'",
                &[&seed.order_id],
            )
            .await
            .expect("query reject adjustment");
        assert_eq!(adjustment_row.get::<_, i64>(0), 1);
        assert!((adjustment_row.get::<_, f64>(1) - 66.60).abs() < 0.0001);
        assert_eq!(
            adjustment_row.get::<_, Option<String>>(2).as_deref(),
            Some("settlement_dispute_hold")
        );
        assert_eq!(
            adjustment_row.get::<_, Option<String>>(3).as_deref(),
            Some("provisional_dispute_hold")
        );

        let settlement_row = client
            .query_one(
                "SELECT settlement_status,
                        refund_amount::float8,
                        net_receivable_amount::float8
                 FROM billing.settlement_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC, settlement_id DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("query reject settlement");
        assert_eq!(settlement_row.get::<_, String>(0), "frozen");
        assert!((settlement_row.get::<_, f64>(1) - 66.60).abs() < 0.0001);
        assert!(settlement_row.get::<_, f64>(2).abs() < 0.0001);

        let order_row = client
            .query_one(
                "SELECT settlement_status, dispute_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query reject order");
        assert_eq!(order_row.get::<_, String>(0), "blocked");
        assert_eq!(order_row.get::<_, String>(1), "open");

        let audit_row = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'delivery.reject'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'billing.adjustment.provisional_hold')",
                &[&request_id],
            )
            .await
            .expect("query reject audit");
        assert_eq!(audit_row.get::<_, i64>(0), 1);
        assert_eq!(audit_row.get::<_, i64>(1), 1);

        cleanup_reject_seed(&client, &seed).await;
        let manual_suffix = format!("{suffix}-manual");
        let seed = seed_manual_adjustment_order(&client, &manual_suffix).await;
        let app = crate::with_live_test_state(billing_router()).await;

        let create_request_id = format!("req-bil025-case-create-{manual_suffix}");
        let dispute_case = create_case(
            &app,
            &seed.buyer_org_id,
            &seed.buyer_user_id,
            &seed.order_id,
            "manual_adjustment",
            &create_request_id,
        )
        .await;
        let case_id = dispute_case["data"]["case_id"]
            .as_str()
            .expect("case id")
            .to_string();

        let resolve_request_id = format!("req-bil025-case-resolve-{manual_suffix}");
        let resolution = resolve_case(
            &app,
            &seed.platform_user_id,
            &case_id,
            "manual_adjustment",
            &resolve_request_id,
        )
        .await;
        assert_eq!(
            resolution["data"]["decision_code"].as_str(),
            Some("manual_adjustment")
        );

        let frozen_row = client
            .query_one(
                "SELECT settlement_status, dispute_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query frozen order");
        assert_eq!(frozen_row.get::<_, String>(0), "frozen");
        assert_eq!(frozen_row.get::<_, String>(1), "resolved");

        let payout_request_id = format!("req-bil025-payout-{manual_suffix}");
        let payout = execute_manual_payout(
            &app,
            &seed.order_id,
            &seed.settlement_id,
            &seed.finance_user_id,
            &payout_request_id,
            &format!("payout:{}:{}", seed.settlement_id, seed.finance_user_id),
        )
        .await;
        assert_eq!(payout["data"]["current_status"].as_str(), Some("succeeded"));
        assert_eq!(payout["data"]["amount"].as_str(), Some("85.00000000"));

        let hold_balance_row = client
            .query_one(
                "SELECT
                   COUNT(*) FILTER (WHERE event_source = 'settlement_dispute_hold')::bigint,
                   COUNT(*) FILTER (WHERE event_source = 'settlement_dispute_release')::bigint,
                   COALESCE(SUM(amount), 0)::float8
                 FROM billing.billing_event
                 WHERE order_id = $1::text::uuid
                   AND event_type = 'refund_adjustment'
                   AND COALESCE(metadata ->> 'adjustment_class', '') = 'provisional_dispute_hold'",
                &[&seed.order_id],
            )
            .await
            .expect("query hold balance");
        assert_eq!(hold_balance_row.get::<_, i64>(0), 1);
        assert_eq!(hold_balance_row.get::<_, i64>(1), 1);
        assert!(hold_balance_row.get::<_, f64>(2).abs() < 0.0001);

        let settlement_row = client
            .query_one(
                "SELECT settlement_status,
                        refund_amount::float8,
                        net_receivable_amount::float8,
                        settled_at IS NOT NULL
                 FROM billing.settlement_record
                 WHERE settlement_id = $1::text::uuid",
                &[&seed.settlement_id],
            )
            .await
            .expect("query payout settlement");
        assert_eq!(settlement_row.get::<_, String>(0), "settled");
        assert!(settlement_row.get::<_, f64>(1).abs() < 0.0001);
        assert!((settlement_row.get::<_, f64>(2) - 85.00).abs() < 0.0001);
        assert!(settlement_row.get::<_, bool>(3));

        let order_row = client
            .query_one(
                "SELECT settlement_status, dispute_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query payout order");
        assert_eq!(order_row.get::<_, String>(0), "settled");
        assert_eq!(order_row.get::<_, String>(1), "resolved");
        assert_eq!(
            order_row.get::<_, Option<String>>(2).as_deref(),
            Some("billing_manual_payout_succeeded")
        );

        let notification_rows = client
            .query(
                "SELECT payload
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND target_topic = 'dtp.notification.dispatch'
                 ORDER BY created_at ASC, outbox_event_id ASC",
                &[&payout_request_id],
            )
            .await
            .expect("query settlement resumed notifications");
        assert_eq!(
            notification_rows.len(),
            3,
            "manual payout release should emit settlement.resumed for buyer/seller/ops"
        );
        let notification_payloads = notification_rows
            .iter()
            .map(|row| row.get::<_, Value>(0))
            .collect::<Vec<_>>();
        let buyer_resumed = find_payload(&notification_payloads, "settlement.resumed", "buyer");
        assert_eq!(
            buyer_resumed["payload"]["source_event"]["aggregate_type"].as_str(),
            Some("billing.billing_event")
        );
        assert_eq!(
            buyer_resumed["payload"]["source_event"]["event_type"].as_str(),
            Some("billing.event.recorded")
        );
        assert_eq!(
            buyer_resumed["payload"]["variables"]["action_href"].as_str(),
            Some(
                format!(
                    "/billing/refunds?order_id={}&case_id={}",
                    seed.order_id, case_id
                )
                .as_str()
            )
        );
        assert!(
            buyer_resumed["payload"]["variables"]
                .get("resolution_ref_id")
                .is_none(),
            "buyer payload must not expose internal resolution ref id"
        );
        let ops_resumed = find_payload(&notification_payloads, "settlement.resumed", "ops");
        assert_eq!(
            ops_resumed["payload"]["variables"]["action_href"].as_str(),
            Some(
                format!(
                    "/ops/audit/trace?order_id={}&case_id={}",
                    seed.order_id, case_id
                )
                .as_str()
            )
        );
        assert_eq!(
            ops_resumed["payload"]["variables"]["resolution_action"].as_str(),
            Some("manual_payout_execute")
        );
        assert_eq!(
            ops_resumed["payload"]["variables"]["show_ops_context"].as_bool(),
            Some(true)
        );

        let detail = get_billing_order(
            &app,
            &seed.order_id,
            &format!("req-bil025-read-{manual_suffix}"),
        )
        .await;
        assert_eq!(detail["data"]["payouts"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            detail["data"]["settlement_summary"]["summary_state"].as_str(),
            Some("order_settlement:settled:manual")
        );

        let audit_row = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'dispute.case.create'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'dispute.case.resolve'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'billing.adjustment.provisional_release'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'billing.payout.execute_manual')",
                &[&create_request_id, &resolve_request_id, &payout_request_id],
            )
            .await
            .expect("query payout audit");
        assert_eq!(audit_row.get::<_, i64>(0), 1);
        assert_eq!(audit_row.get::<_, i64>(1), 1);
        assert_eq!(audit_row.get::<_, i64>(2), 1);
        assert_eq!(audit_row.get::<_, i64>(3), 1);

        cleanup_manual_adjustment_seed(
            &client,
            &seed,
            &case_id,
            &[&create_request_id, &resolve_request_id, &payout_request_id],
        )
        .await;
    }

    async fn reject_order(
        app: &Router,
        buyer_org_id: &str,
        order_id: &str,
        request_id: &str,
        reason_code: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/reject"))
                    .header("content-type", "application/json")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", buyer_org_id)
                    .header("x-request-id", request_id)
                    .header(
                        "x-idempotency-key",
                        format!("delivery-reject:{order_id}:{request_id}"),
                    )
                    .body(Body::from(
                        json!({
                            "reason_code": reason_code,
                            "reason_detail": "bil025 reject quality check failed",
                            "verification_summary": {
                                "hash_match": true,
                                "template_match": false
                            }
                        })
                        .to_string(),
                    ))
                    .expect("reject request"),
            )
            .await
            .expect("reject response");
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
                                "entry": "bil025"
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
                    .header("x-step-up-token", "bil025-stepup")
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "decision_type": "manual_resolution",
                            "decision_code": decision_code,
                            "liability_type": "seller",
                            "penalty_code": "seller_warning",
                            "decision_text": format!("{decision_code} approved by bil025"),
                            "metadata": {
                                "entry": "bil025"
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

    async fn execute_manual_payout(
        app: &Router,
        order_id: &str,
        settlement_id: &str,
        user_id: &str,
        request_id: &str,
        idempotency_key: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payouts/manual")
                    .header("x-role", "platform_finance_operator")
                    .header("x-user-id", user_id)
                    .header("x-request-id", request_id)
                    .header("x-idempotency-key", idempotency_key)
                    .header("x-step-up-token", "bil025-payout-stepup")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "order_id": order_id,
                            "settlement_id": settlement_id,
                            "amount": "85.00000000",
                            "metadata": {"entry": "bil025"}
                        })
                        .to_string(),
                    ))
                    .expect("payout request should build"),
            )
            .await
            .expect("payout response");
        json_response(response, StatusCode::OK).await
    }

    async fn get_billing_order(app: &Router, order_id: &str, request_id: &str) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/billing/{order_id}"))
                    .header("x-role", "platform_admin")
                    .header("x-request-id", request_id)
                    .body(Body::empty())
                    .expect("billing request should build"),
            )
            .await
            .expect("billing response");
        json_response(response, StatusCode::OK).await
    }

    async fn json_response(
        response: axum::response::Response,
        expected_status: StatusCode,
    ) -> Value {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        assert_eq!(
            status,
            expected_status,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("response json")
    }

    async fn seed_reject_order(client: &Client, suffix: &str) -> Result<RejectSeed, db::Error> {
        let buyer_org_id = seed_org(
            client,
            &format!("bil025-reject-buyer-{suffix}"),
            "enterprise",
        )
        .await;
        let seller_org_id = seed_org(
            client,
            &format!("bil025-reject-seller-{suffix}"),
            "enterprise",
        )
        .await;
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'analysis', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil025-reject-asset-{suffix}"),
                    &format!("bil025 reject asset {suffix}"),
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'analysis', 'data_product',
                   $5, 'listed', 'one_time', 66.60, 'CNY', 'report_delivery',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil025-reject-product-{suffix}"),
                    &format!("bil025 reject product {suffix}"),
                    &format!("bil025 reject search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'RPT_STD', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("BIL025-RPT-{suffix}")],
            )
            .await?
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code,
                   delivery_status, acceptance_status, settlement_status, dispute_status,
                   price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'report_delivered', 'paid', 'online', 66.60, 'CNY',
                   'delivered', 'in_progress', 'pending_settlement', 'none',
                   '{}'::jsonb,
                   jsonb_build_object('delivery_mode', 'report_delivery'),
                   'result_package'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                ],
            )
            .await?
            .get(0);
        let delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, trust_boundary_snapshot,
                   sensitive_delivery_mode, disclosure_review_status, committed_at
                 ) VALUES (
                   $1::text::uuid, 'report_delivery', 'result_package', 'committed',
                   jsonb_build_object('delivery_mode', 'report_delivery'),
                   'standard', 'not_required', now()
                 )
                 RETURNING delivery_id::text",
                &[&order_id],
            )
            .await?
            .get(0);

        Ok(RejectSeed {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            delivery_id,
        })
    }

    async fn seed_manual_adjustment_order(client: &Client, suffix: &str) -> ManualAdjustmentSeed {
        let buyer_org_id = seed_org(
            client,
            &format!("bil025-manual-buyer-{suffix}"),
            "enterprise",
        )
        .await;
        let seller_org_id = seed_org(
            client,
            &format!("bil025-manual-seller-{suffix}"),
            "enterprise",
        )
        .await;
        let platform_org_id = seed_org(
            client,
            &format!("bil025-manual-platform-{suffix}"),
            "platform",
        )
        .await;
        let buyer_user_id = seed_user(client, &buyer_org_id, &format!("buyer-{suffix}")).await;
        let platform_user_id =
            seed_user(client, &platform_org_id, &format!("platform-{suffix}")).await;
        let finance_user_id =
            seed_user(client, &platform_org_id, &format!("finance-{suffix}")).await;
        let provider_account_id = seed_provider_account(client, &seller_org_id, suffix).await;
        let payout_preference_id =
            seed_payout_preference(client, &seller_org_id, &provider_account_id, suffix).await;
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description)
                 VALUES ($1::text::uuid, $2, 'finance', 'internal', 'active', $3)
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil025-manual-asset-{suffix}"),
                    &format!("bil025 manual asset {suffix}"),
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
                    &format!("bil025-manual-product-{suffix}"),
                    &format!("bil025 manual product {suffix}"),
                    &format!("bil025 manual summary {suffix}"),
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
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("BIL025-SKU-{suffix}")],
            )
            .await
            .expect("insert sku")
            .get(0);
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
                   'accepted',
                   'paid',
                   'delivered',
                   'accepted',
                   'pending_settlement',
                   'none',
                   'online',
                   88.00,
                   'SGD',
                   jsonb_build_object(
                     'product_id', $1,
                     'sku_id', $5,
                     'sku_type', 'FILE_STD',
                     'selected_sku_type', 'FILE_STD',
                     'billing_mode', 'one_time',
                     'pricing_mode', 'one_time',
                     'settlement_basis', 'gross_amount',
                     'refund_mode', 'manual_refund',
                     'refund_template', 'REFUND_FILE_V1',
                     'price_currency_code', 'SGD'
                   ),
                   'file_download',
                   '{"delivery_mode":"file_download"}'::jsonb,
                   'bil025_seed_manual_adjustment'
                 ) RETURNING order_id::text"#,
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
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
                   $1::text::uuid, 'order_payment', 'mock_payment', $2::text::uuid, 'organization', $3::text::uuid,
                   'organization', $4::text::uuid, 'SG', 'SG', 'SG', 88.00, 'wallet', 'SGD', 'SGD',
                   'succeeded', $5, $6, '{"supports_payout":true,"supports_refund":true}'::jsonb, '{}'::jsonb
                 ) RETURNING payment_intent_id::text"#,
                &[
                    &order_id,
                    &provider_account_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("bil025-pay-req-{suffix}"),
                    &format!("pay:bil025:{suffix}"),
                ],
            )
            .await
            .expect("insert payment intent")
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
                   'bil025_seed', NULL
                 ) RETURNING settlement_id::text",
                &[&order_id],
            )
            .await
            .expect("insert settlement")
            .get(0);

        ManualAdjustmentSeed {
            buyer_org_id,
            seller_org_id,
            platform_org_id,
            buyer_user_id,
            platform_user_id,
            finance_user_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            payment_intent_id,
            settlement_id,
            provider_account_id,
            payout_preference_id,
        }
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
                    &format!("bil025-user-{suffix}"),
                    &format!("BIL025 User {suffix}"),
                    &format!("bil025-{suffix}@example.com"),
                ],
            )
            .await
            .expect("insert user")
            .get(0)
    }

    async fn seed_provider_account(client: &Client, seller_org_id: &str, suffix: &str) -> String {
        client
            .query_one(
                "INSERT INTO payment.provider_account (
                   provider_key, account_scope, account_scope_id, account_name,
                   settlement_subject_type, settlement_subject_id, jurisdiction_code,
                   account_mode, status, config_json
                 ) VALUES (
                   'mock_payment', 'tenant', $1::text::uuid, $2,
                   'organization', $1::text::uuid, 'SG',
                   'sandbox', 'active', '{}'::jsonb
                 )
                 RETURNING provider_account_id::text",
                &[&seller_org_id, &format!("bil025-account-{suffix}")],
            )
            .await
            .expect("insert provider account")
            .get(0)
    }

    async fn seed_payout_preference(
        client: &Client,
        seller_org_id: &str,
        provider_account_id: &str,
        suffix: &str,
    ) -> String {
        client
            .query_one(
                "INSERT INTO payment.payout_preference (
                   beneficiary_subject_type,
                   beneficiary_subject_id,
                   destination_jurisdiction_code,
                   preferred_currency_code,
                   payout_method,
                   preferred_provider_key,
                   preferred_provider_account_id,
                   beneficiary_snapshot,
                   is_default,
                   status
                 ) VALUES (
                   'organization',
                   $1::text::uuid,
                   'SG',
                   'SGD',
                   'bank_transfer',
                   'mock_payment',
                   $2::text::uuid,
                   jsonb_build_object('seller_name', $3),
                   true,
                   'active'
                 )
                 RETURNING payout_preference_id::text",
                &[
                    &seller_org_id,
                    &provider_account_id,
                    &format!("BIL025 Seller {suffix}"),
                ],
            )
            .await
            .expect("insert payout preference")
            .get(0)
    }

    async fn cleanup_reject_seed(client: &Client, seed: &RejectSeed) {
        let _ = client
            .execute(
                "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE delivery_id = $1::text::uuid",
                &[&seed.delivery_id],
            )
            .await;
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
                "DELETE FROM core.organization WHERE org_id = ANY($1::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await;
    }

    fn find_payload<'a>(
        payloads: &'a [Value],
        notification_code: &str,
        audience_scope: &str,
    ) -> &'a Value {
        payloads
            .iter()
            .find(|payload| {
                payload["payload"]["notification_code"].as_str() == Some(notification_code)
                    && payload["payload"]["audience_scope"].as_str() == Some(audience_scope)
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing payload for notification_code={} audience_scope={}",
                    notification_code, audience_scope
                )
            })
    }

    async fn cleanup_manual_adjustment_seed(
        client: &Client,
        seed: &ManualAdjustmentSeed,
        case_id: &str,
        request_ids: &[&str],
    ) {
        let request_ids = request_ids
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>();
        let _ = client
            .execute(
                "DELETE FROM audit.legal_hold WHERE hold_scope_type = 'order' AND hold_scope_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM risk.freeze_ticket WHERE ref_type = 'order' AND ref_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM support.decision_record WHERE case_id = $1::text::uuid",
                &[&case_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM support.dispute_case WHERE case_id = $1::text::uuid",
                &[&case_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event
                 WHERE request_id = ANY($1::text[])
                    OR aggregate_id = $2::text::uuid
                    OR ordering_key = $3",
                &[&request_ids, &case_id, &seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.sub_merchant_binding WHERE provider_account_id = $1::text::uuid",
                &[&seed.provider_account_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.split_instruction WHERE settlement_id = $1::text::uuid",
                &[&seed.settlement_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payout_instruction WHERE settlement_id = $1::text::uuid",
                &[&seed.settlement_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE settlement_id = $1::text::uuid",
                &[&seed.settlement_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&seed.payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payout_preference WHERE payout_preference_id = $1::text::uuid",
                &[&seed.payout_preference_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.provider_account WHERE provider_account_id = $1::text::uuid",
                &[&seed.provider_account_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.user_account WHERE user_id = ANY($1::uuid[])",
                &[&vec![
                    seed.buyer_user_id.clone(),
                    seed.platform_user_id.clone(),
                    seed.finance_user_id.clone(),
                ]],
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
                "DELETE FROM core.organization WHERE org_id = ANY($1::uuid[])",
                &[&vec![
                    seed.buyer_org_id.clone(),
                    seed.seller_org_id.clone(),
                    seed.platform_org_id.clone(),
                ]],
            )
            .await;
    }
}
