#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        buyer_user_id: String,
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
    async fn bil014_dispute_linkage_db_smoke() {
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

        let request_id = format!("req-bil014-create-{suffix}");
        let dispute_case = create_case(&app, &seed, &request_id).await;
        let case_id = dispute_case["data"]["case_id"]
            .as_str()
            .expect("case id")
            .to_string();
        assert_eq!(
            dispute_case["data"]["current_status"].as_str(),
            Some("opened")
        );

        let order_row = client
            .query_one(
                "SELECT delivery_status, acceptance_status, settlement_status, dispute_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order status");
        assert_eq!(order_row.get::<_, String>(0), "blocked");
        assert_eq!(order_row.get::<_, String>(1), "blocked");
        assert_eq!(order_row.get::<_, String>(2), "frozen");
        assert_eq!(order_row.get::<_, String>(3), "opened");
        assert_eq!(
            order_row.get::<_, Option<String>>(4).as_deref(),
            Some("billing_dispute_linkage_applied")
        );

        let settlement_row = client
            .query_one(
                "SELECT settlement_status, reason_code
                 FROM billing.settlement_record
                 WHERE settlement_id = $1::text::uuid",
                &[&seed.settlement_id],
            )
            .await
            .expect("query settlement");
        assert_eq!(settlement_row.get::<_, String>(0), "frozen");
        assert_eq!(
            settlement_row.get::<_, Option<String>>(1).as_deref(),
            Some("dispute_opened:delivery_failed")
        );

        let delivery_row = client
            .query_one(
                "SELECT status FROM delivery.delivery_record WHERE delivery_id = $1::text::uuid",
                &[&seed.delivery_id],
            )
            .await
            .expect("query delivery");
        assert_eq!(delivery_row.get::<_, String>(0), "suspended");

        let ticket_row = client
            .query_one(
                "SELECT status FROM delivery.delivery_ticket WHERE ticket_id = $1::text::uuid",
                &[&seed.ticket_id],
            )
            .await
            .expect("query ticket");
        assert_eq!(ticket_row.get::<_, String>(0), "suspended");

        let freeze_ticket_row = client
            .query_one(
                "SELECT freeze_type, status, reason_code
                 FROM risk.freeze_ticket
                 WHERE ref_type = 'order' AND ref_id = $1::text::uuid
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("query freeze ticket");
        assert_eq!(freeze_ticket_row.get::<_, String>(0), "dispute_hold");
        assert_eq!(freeze_ticket_row.get::<_, String>(1), "executed");
        assert_eq!(freeze_ticket_row.get::<_, String>(2), "delivery_failed");

        let governance_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM risk.governance_action_log g
                 JOIN risk.freeze_ticket f ON f.freeze_ticket_id = g.freeze_ticket_id
                 WHERE f.ref_type = 'order'
                   AND f.ref_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("count governance actions")
            .get(0);
        assert_eq!(governance_count, 3);

        let legal_hold_row = client
            .query_one(
                "SELECT status, reason_code, metadata ->> 'case_id'
                 FROM audit.legal_hold
                 WHERE hold_scope_type = 'order'
                   AND hold_scope_id = $1::text::uuid
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("query legal hold");
        assert_eq!(legal_hold_row.get::<_, String>(0), "active");
        assert_eq!(legal_hold_row.get::<_, String>(1), "delivery_failed");
        assert_eq!(
            legal_hold_row.get::<_, Option<String>>(2).as_deref(),
            Some(case_id.as_str())
        );

        let outbox_row = client
            .query_one(
                "SELECT
                   payload #>> '{linkage,order_delivery_status}',
                   payload #>> '{linkage,order_acceptance_status}',
                   payload #>> '{linkage,order_settlement_status}'
                 FROM ops.outbox_event
                 WHERE aggregate_type = 'support.dispute_case'
                   AND aggregate_id = $1::text::uuid
                   AND event_type = 'dispute.created'
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&case_id],
            )
            .await
            .expect("query dispute outbox");
        assert_eq!(
            outbox_row.get::<_, Option<String>>(0).as_deref(),
            Some("blocked")
        );
        assert_eq!(
            outbox_row.get::<_, Option<String>>(1).as_deref(),
            Some("blocked")
        );
        assert_eq!(
            outbox_row.get::<_, Option<String>>(2).as_deref(),
            Some("frozen")
        );

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
            .expect("query notif007 outbox");
        assert_eq!(
            notification_rows.len(),
            6,
            "create dispute case should emit dispute and settlement freeze notifications"
        );
        let notification_payloads = notification_rows
            .iter()
            .map(|row| row.get::<_, Value>(0))
            .collect::<Vec<_>>();
        let buyer_dispute = find_payload(&notification_payloads, "dispute.escalated", "buyer");
        assert_eq!(
            buyer_dispute["payload"]["source_event"]["aggregate_type"].as_str(),
            Some("support.dispute_case")
        );
        assert_eq!(
            buyer_dispute["payload"]["source_event"]["event_type"].as_str(),
            Some("dispute.created")
        );
        assert_eq!(
            buyer_dispute["payload"]["variables"]["action_href"].as_str(),
            Some(format!("/support/cases/new?order_id={}", seed.order_id).as_str())
        );
        assert!(
            buyer_dispute["payload"]["variables"]
                .get("freeze_ticket_id")
                .is_none(),
            "buyer dispute payload must not expose freeze ticket"
        );
        let ops_frozen = find_payload(&notification_payloads, "settlement.frozen", "ops");
        assert_eq!(
            ops_frozen["payload"]["source_event"]["aggregate_type"].as_str(),
            Some("billing.billing_event")
        );
        assert_eq!(
            ops_frozen["payload"]["source_event"]["event_type"].as_str(),
            Some("billing.event.recorded")
        );
        assert_eq!(
            ops_frozen["payload"]["variables"]["action_href"].as_str(),
            Some(format!("/ops/risk?order_id={}&case_id={}", seed.order_id, case_id).as_str())
        );
        assert_eq!(
            ops_frozen["payload"]["variables"]["show_ops_context"].as_bool(),
            Some(true)
        );
        assert!(
            ops_frozen["payload"]["variables"]["freeze_ticket_id"]
                .as_str()
                .is_some()
        );
        assert!(
            ops_frozen["payload"]["variables"]["legal_hold_id"]
                .as_str()
                .is_some()
        );

        let audit_row = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'dispute.case.create'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'billing.settlement.freeze'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'audit.legal_hold.activate'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'delivery.file.auto_cutoff.suspended')",
                &[&request_id],
            )
            .await
            .expect("query audit counts");
        assert_eq!(audit_row.get::<_, i64>(0), 1);
        assert_eq!(audit_row.get::<_, i64>(1), 1);
        assert_eq!(audit_row.get::<_, i64>(2), 1);
        assert_eq!(audit_row.get::<_, i64>(3), 1);

        cleanup(&client, &seed, &case_id, &request_id).await;
    }

    async fn create_case(app: &Router, seed: &SeedGraph, request_id: &str) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cases")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "order_id": seed.order_id,
                            "reason_code": "delivery_failed",
                            "requested_resolution": "refund_full",
                            "claimed_amount": "88.00000000",
                            "evidence_scope": "delivery_receipt,download_log",
                            "blocking_effect": "freeze_settlement",
                            "metadata": {
                                "entry": "bil014"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("create case request should build"),
            )
            .await
            .expect("create case response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("create case body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("create case json")
    }

    async fn seed_graph(client: &Client, suffix: &str) -> SeedGraph {
        let buyer_org_id = seed_org(client, &format!("bil014-buyer-{suffix}"), "enterprise").await;
        let seller_org_id =
            seed_org(client, &format!("bil014-seller-{suffix}"), "enterprise").await;
        let buyer_user_id = seed_user(client, &buyer_org_id, &format!("buyer-{suffix}")).await;
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description)
                 VALUES ($1::text::uuid, $2, 'finance', 'internal', 'active', $3)
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil014-asset-{suffix}"),
                    &format!("bil014 asset {suffix}"),
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
                    &format!("bil014-product-{suffix}"),
                    &format!("bil014 product {suffix}"),
                    &format!("bil014 summary {suffix}"),
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
                &[&product_id, &format!("BIL014-SKU-{suffix}")],
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
                   'delivered',
                   'paid',
                   'delivered',
                   'pending_acceptance',
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
                   'signed_url',
                   '{"delivery_mode":"file_download"}'::jsonb,
                   'bil014_seed_dispute'
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
                   'succeeded', $4, $5, '{"supports_refund":true}'::jsonb, '{}'::jsonb
                 ) RETURNING payment_intent_id::text"#,
                &[
                    &order_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("bil014-pay-req-{suffix}"),
                    &format!("pay:bil014:{suffix}"),
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
                   'bil014_seed', NULL
                 ) RETURNING settlement_id::text",
                &[&order_id],
            )
            .await
            .expect("insert settlement")
            .get(0);
        let delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, delivery_commit_hash,
                   trust_boundary_snapshot, receipt_hash, committed_at, expires_at
                 ) VALUES (
                   $1::text::uuid, 'file_download', 'signed_url', 'committed', $2,
                   '{\"delivery_mode\":\"file_download\"}'::jsonb, $3, now() - interval '1 hour', now() + interval '7 days'
                 ) RETURNING delivery_id::text",
                &[
                    &order_id,
                    &format!("bil014-commit-{suffix}"),
                    &format!("bil014-receipt-{suffix}"),
                ],
            )
            .await
            .expect("insert delivery record")
            .get(0);
        let ticket_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_ticket (
                   order_id, buyer_org_id, token_hash, expire_at, download_limit, download_count, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, now() + interval '6 days', 5, 1, 'active'
                 )
                 RETURNING ticket_id::text",
                &[&order_id, &buyer_org_id, &format!("bil014-ticket-{suffix}")],
            )
            .await
            .expect("insert delivery ticket")
            .get(0);

        SeedGraph {
            buyer_org_id,
            seller_org_id,
            buyer_user_id,
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
                    &format!("bil014-user-{suffix}"),
                    &format!("BIL014 User {suffix}"),
                    &format!("bil014-{suffix}@example.com"),
                ],
            )
            .await
            .expect("insert user")
            .get(0)
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

    async fn cleanup(client: &Client, seed: &SeedGraph, case_id: &str, request_id: &str) {
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
                "DELETE FROM support.dispute_case WHERE case_id = $1::text::uuid",
                &[&case_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event
                 WHERE request_id = $1
                    OR aggregate_id = $2::text::uuid
                    OR ordering_key = $3",
                &[&request_id, &case_id, &seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.delivery_ticket WHERE ticket_id = $1::text::uuid",
                &[&seed.ticket_id],
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
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
                &[&seed.buyer_user_id],
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
}
