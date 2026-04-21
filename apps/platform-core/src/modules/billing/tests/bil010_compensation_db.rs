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
        payment_intent_id: String,
        settlement_id: String,
        case_id: String,
        decision_id: String,
        buyer_user_id: String,
    }

    #[tokio::test]
    async fn bil010_compensation_db_smoke() {
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
        let buyer_org_id = seed_org(&client, &format!("bil010-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil010-seller-{suffix}")).await;
        let order = seed_order(&client, &buyer_org_id, &seller_org_id, &suffix).await;

        let app = crate::with_live_test_state(router()).await;
        let request_id = format!("req-bil010-compensation-{suffix}");
        let idempotency_key = format!("compensation:{}", order.case_id);
        let compensation = execute_compensation(
            &app,
            &order.order_id,
            &order.case_id,
            &buyer_org_id,
            &order.buyer_user_id,
            &request_id,
            &idempotency_key,
        )
        .await;
        assert_eq!(
            compensation["data"]["current_status"].as_str(),
            Some("succeeded")
        );
        assert_eq!(
            compensation["data"]["decision_code"].as_str(),
            Some("compensation_full")
        );
        assert_eq!(compensation["data"]["amount"].as_str(), Some("20.00000000"));
        assert_eq!(compensation["data"]["step_up_bound"].as_bool(), Some(true));
        assert_eq!(
            compensation["data"]["idempotent_replay"].as_bool(),
            Some(false)
        );
        assert!(
            compensation["data"]["provider_transfer_id"]
                .as_str()
                .is_some()
        );

        let replay = execute_compensation(
            &app,
            &order.order_id,
            &order.case_id,
            &buyer_org_id,
            &order.buyer_user_id,
            &format!("{request_id}-replay"),
            &idempotency_key,
        )
        .await;
        assert_eq!(
            replay["data"]["compensation_id"],
            compensation["data"]["compensation_id"]
        );
        assert_eq!(replay["data"]["idempotent_replay"].as_bool(), Some(true));

        let compensation_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM billing.compensation_record WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await
            .expect("count compensation")
            .get(0);
        assert_eq!(compensation_count, 1);

        let settlement_compensation_amount: String = client
            .query_one(
                "SELECT compensation_amount::text FROM billing.settlement_record WHERE settlement_id = $1::text::uuid",
                &[&order.settlement_id],
            )
            .await
            .expect("compensation amount")
            .get(0);
        assert_eq!(settlement_compensation_amount, "20.00000000");

        let detail = get_billing_order(
            &app,
            &order.order_id,
            &buyer_org_id,
            &format!("req-bil010-read-{suffix}"),
        )
        .await;
        assert_eq!(
            detail["data"]["compensations"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            detail["data"]["settlement_summary"]["compensation_adjustment_amount"].as_str(),
            Some("20.00000000")
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'billing.compensation.execute'",
                &[&request_id],
            )
            .await
            .expect("query audit")
            .get(0);
        assert_eq!(audit_count, 1);

        let compensation_id = compensation["data"]["compensation_id"]
            .as_str()
            .unwrap()
            .to_string();
        let billing_event_id = compensation["data"]["metadata"]["billing_event_id"]
            .as_str()
            .unwrap()
            .to_string();
        let outbox_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE aggregate_type = 'billing.compensation_record' AND aggregate_id = $1::text::uuid",
                &[&compensation_id],
            )
            .await
            .expect("query outbox")
            .get(0);
        assert_eq!(outbox_count, 1);
        let notification_rows = client
            .query(
                "SELECT payload
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND target_topic = 'dtp.notification.dispatch'
                 ORDER BY created_at ASC, outbox_event_id ASC",
                &[&request_id],
            )
            .await
            .expect("query compensation notifications");
        assert_eq!(notification_rows.len(), 3);
        let notification_payloads = notification_rows
            .iter()
            .map(|row| row.get::<_, Value>(0))
            .collect::<Vec<_>>();
        let buyer = find_notification_payload(&notification_payloads, "buyer");
        assert_eq!(
            buyer["payload"]["notification_code"].as_str(),
            Some("compensation.completed")
        );
        assert_eq!(
            buyer["payload"]["template_code"].as_str(),
            Some("NOTIFY_COMPENSATION_COMPLETED_V1")
        );
        assert_eq!(
            buyer["payload"]["source_event"]["aggregate_id"].as_str(),
            Some(billing_event_id.as_str())
        );
        let ops = find_notification_payload(&notification_payloads, "ops");
        assert_eq!(
            ops["payload"]["variables"]["show_ops_context"].as_bool(),
            Some(true)
        );

        cleanup(&client, &buyer_org_id, &seller_org_id, &order).await;
    }

    fn find_notification_payload<'a>(payloads: &'a [Value], audience_scope: &str) -> &'a Value {
        payloads
            .iter()
            .find(|payload| payload["payload"]["audience_scope"].as_str() == Some(audience_scope))
            .unwrap_or_else(|| panic!("missing payload for audience {audience_scope}"))
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
                    .header("x-step-up-token", "bil010-stepup")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "order_id": order_id,
                            "case_id": case_id,
                            "decision_code": "compensation_full",
                            "amount": "20.00000000",
                            "reason_code": "sla_breach",
                            "metadata": {
                                "entry": "bil010"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("compensation request should build"),
            )
            .await
            .expect("compensation response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("compensation body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("compensation json")
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
                    .header("x-role", "platform_risk_settlement")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .body(Body::empty())
                    .expect("billing request should build"),
            )
            .await
            .expect("billing response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("billing body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("billing json")
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

    async fn seed_user(client: &db::Client, org_id: &str, suffix: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status, email, attrs)
                 VALUES ($1::text::uuid, $2, $3, 'human', 'active', 'verified', $4, '{}'::jsonb)
                 RETURNING user_id::text",
                &[
                    &org_id,
                    &format!("bil010-user-{suffix}"),
                    &format!("BIL010 User {suffix}"),
                    &format!("bil010-{suffix}@example.com"),
                ],
            )
            .await
            .expect("insert user")
            .get(0)
    }

    async fn seed_order(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
    ) -> SeedOrderGraph {
        let buyer_user_id = seed_user(client, buyer_org_id, suffix).await;
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description)
                 VALUES ($1::text::uuid, $2, 'finance', 'internal', 'active', $3)
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil010-asset-{suffix}"),
                    &format!("bil010 asset {suffix}"),
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
                    &format!("bil010-product-{suffix}"),
                    &format!("bil010 product {suffix}"),
                    &format!("bil010 summary {suffix}"),
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
                &[&product_id, &format!("BIL010-SKU-{suffix}")],
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
                   'buyer_locked',
                   'paid',
                   'delivered',
                   'accepted',
                   'pending_settlement',
                   'opened',
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
                     'compensation_mode', 'manual_transfer',
                     'compensation_template', 'COMPENSATION_FILE_V1',
                     'price_currency_code', 'SGD'
                   ),
                   'file_download',
                   '{"delivery_mode":"file_download"}'::jsonb,
                   'bil010_seed_compensation'
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
                   $1::text::uuid, 'order_payment', 'mock_payment', NULL, 'organization', $2::text::uuid,
                   'organization', $3::text::uuid, 'SG', 'SG', 'SG', 88.00, 'wallet', 'SGD', 'SGD',
                   'succeeded', $4, $5, '{"supports_payout":true}'::jsonb, '{}'::jsonb
                 ) RETURNING payment_intent_id::text"#,
                &[
                    &order_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("bil010-pay-req-{suffix}"),
                    &format!("pay:bil010:{suffix}"),
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
                   'bil010_seed', NULL
                 ) RETURNING settlement_id::text",
                &[&order_id],
            )
            .await
            .expect("insert settlement")
            .get(0);
        let case_id: String = client
            .query_one(
                "INSERT INTO support.dispute_case (
                   order_id, complainant_type, complainant_id, reason_code, status, decision_code, penalty_code, resolved_at
                 ) VALUES (
                   $1::text::uuid, 'organization', $2::text::uuid, 'sla_breach', 'manual_review', 'compensation_full', 'seller_warning', now()
                 ) RETURNING case_id::text",
                &[&order_id, &buyer_org_id],
            )
            .await
            .expect("insert dispute case")
            .get(0);
        let decision_id: String = client
            .query_one(
                "INSERT INTO support.decision_record (
                   case_id, decision_type, decision_code, liability_type, decision_text, decided_by
                 ) VALUES (
                   $1::text::uuid, 'manual_resolution', 'compensation_full', 'seller', 'compensation approved', $2::text::uuid
                 ) RETURNING decision_id::text",
                &[&case_id, &buyer_user_id],
            )
            .await
            .expect("insert decision")
            .get(0);
        SeedOrderGraph {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            payment_intent_id,
            settlement_id,
            case_id,
            decision_id,
            buyer_user_id,
        }
    }

    async fn cleanup(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        order: &SeedOrderGraph,
    ) {
        let _ = client
            .execute(
                "DELETE FROM support.decision_record WHERE decision_id = $1::text::uuid",
                &[&order.decision_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM support.dispute_case WHERE case_id = $1::text::uuid",
                &[&order.case_id],
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
                "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
                &[&order.order_id],
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
                "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
                &[&order.buyer_user_id],
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
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::uuid[])",
                &[&vec![buyer_org_id.to_string(), seller_org_id.to_string()]],
            )
            .await;
    }
}
