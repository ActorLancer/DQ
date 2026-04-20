#[cfg(test)]
mod tests {
    use super::super::super::api::router as billing_router;
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::order::api::router as order_router;
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

    struct ShareOrderSeed {
        buyer_org_id: String,
        seller_org_id: String,
        buyer_user_id: String,
        asset_id: String,
        asset_version_id: String,
        asset_object_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn bil026_share_ro_billing_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let _guard = smoke_lock().lock().await;
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
        let cycle_seed = seed_share_order(&client, &format!("{suffix}-cycle"))
            .await
            .expect("seed share cycle order");
        let dispute_seed = seed_share_order(&client, &format!("{suffix}-dispute"))
            .await
            .expect("seed share dispute order");

        let app = crate::with_live_test_state(
            billing_router()
                .merge(order_router())
                .merge(delivery_router()),
        )
        .await;

        let before = get_billing_order(
            &app,
            &cycle_seed.order_id,
            &cycle_seed.buyer_org_id,
            &format!("req-bil026-read-before-{suffix}"),
        )
        .await;
        assert_eq!(
            before["data"]["sku_billing_basis"]["sku_type"].as_str(),
            Some("SHARE_RO")
        );
        assert_eq!(
            before["data"]["sku_billing_basis"]["default_event_type"].as_str(),
            Some("one_time_charge")
        );
        assert_eq!(
            before["data"]["sku_billing_basis"]["cycle_event_type"].as_str(),
            Some("recurring_charge")
        );
        assert_eq!(
            before["data"]["sku_billing_basis"]["periodic_settlement_cycle"].as_str(),
            Some("monthly_cycle")
        );
        assert_eq!(
            before["data"]["sku_billing_basis"]["refund_placeholder_event_type"].as_str(),
            Some("refund_adjustment")
        );

        let enable_request_id = format!("req-bil026-enable-{suffix}");
        let enable = transition_share_enable(
            &app,
            &cycle_seed.order_id,
            &cycle_seed.buyer_org_id,
            &enable_request_id,
        )
        .await;
        assert_eq!(
            enable["data"]["data"]["billing_event_type"].as_str(),
            Some("one_time_charge")
        );
        let opening_event_id = enable["data"]["data"]["billing_event_id"]
            .as_str()
            .expect("opening event id")
            .to_string();

        let grant_request_id = format!("req-bil026-grant-{suffix}");
        let grant = grant_share(
            &app,
            &cycle_seed.order_id,
            &cycle_seed.seller_org_id,
            &cycle_seed.asset_object_id,
            &grant_request_id,
            &suffix,
        )
        .await;
        assert_eq!(
            grant["data"]["data"]["current_state"].as_str(),
            Some("share_granted")
        );

        let cycle_request_id = format!("req-bil026-cycle-{suffix}");
        let cycle = record_cycle_charge(
            &app,
            &cycle_seed.order_id,
            &cycle_request_id,
            "2026-04",
            "33.00000000",
        )
        .await;
        assert_eq!(
            cycle["data"]["billing_event_type"].as_str(),
            Some("recurring_charge")
        );
        assert_eq!(
            cycle["data"]["billing_event_replayed"].as_bool(),
            Some(false)
        );
        let cycle_event_id = cycle["data"]["billing_event_id"]
            .as_str()
            .expect("cycle event id")
            .to_string();

        let cycle_replay = record_cycle_charge(
            &app,
            &cycle_seed.order_id,
            &format!("req-bil026-cycle-replay-{suffix}"),
            "2026-04",
            "33.00000000",
        )
        .await;
        assert_eq!(
            cycle_replay["data"]["billing_event_id"].as_str(),
            Some(cycle_event_id.as_str())
        );
        assert_eq!(
            cycle_replay["data"]["billing_event_replayed"].as_bool(),
            Some(true)
        );

        let revoke_request_id = format!("req-bil026-revoke-{suffix}");
        let revoke = revoke_share(
            &app,
            &cycle_seed.order_id,
            &cycle_seed.seller_org_id,
            &revoke_request_id,
            &suffix,
        )
        .await;
        assert_eq!(
            revoke["data"]["data"]["current_state"].as_str(),
            Some("revoked")
        );

        let after = get_billing_order(
            &app,
            &cycle_seed.order_id,
            &cycle_seed.buyer_org_id,
            &format!("req-bil026-read-after-{suffix}"),
        )
        .await;
        assert_eq!(
            after["data"]["billing_events"].as_array().map(Vec::len),
            Some(3)
        );
        assert_eq!(
            after["data"]["settlement_summary"]["refund_adjustment_amount"].as_str(),
            Some("99.00000000")
        );

        let cycle_counts = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'one_time_charge'),
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'recurring_charge'),
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'refund_adjustment' AND event_source = 'share_revoke_refund_placeholder'),
                   (SELECT COALESCE(SUM(amount), 0)::text FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'refund_adjustment' AND event_source = 'share_revoke_refund_placeholder'),
                   (SELECT settlement_status FROM trade.order_main WHERE order_id = $1::text::uuid),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'billing.event.record.share_ro_enable'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'billing.event.record.share_ro_cycle'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $4 AND action_name = 'billing.event.record.share_ro_revoke_refund_placeholder')",
                &[
                    &cycle_seed.order_id,
                    &enable_request_id,
                    &cycle_request_id,
                    &revoke_request_id,
                ],
            )
            .await
            .expect("query cycle counts");
        assert_eq!(cycle_counts.get::<_, i64>(0), 1);
        assert_eq!(cycle_counts.get::<_, i64>(1), 1);
        assert_eq!(cycle_counts.get::<_, i64>(2), 1);
        assert_eq!(cycle_counts.get::<_, String>(3), "99.00000000");
        assert_eq!(cycle_counts.get::<_, String>(4), "closed");
        assert_eq!(cycle_counts.get::<_, i64>(5), 1);
        assert_eq!(cycle_counts.get::<_, i64>(6), 1);
        assert_eq!(cycle_counts.get::<_, i64>(7), 1);

        transition_share_enable(
            &app,
            &dispute_seed.order_id,
            &dispute_seed.buyer_org_id,
            &format!("req-bil026-enable-dispute-{suffix}"),
        )
        .await;
        let create_case_request_id = format!("req-bil026-case-{suffix}");
        let dispute_case = create_case(
            &app,
            &dispute_seed.buyer_org_id,
            &dispute_seed.buyer_user_id,
            &dispute_seed.order_id,
            &create_case_request_id,
        )
        .await;
        assert_eq!(dispute_case["data"]["case_status"].as_str(), Some("open"));

        let dispute_row = client
            .query_one(
                "SELECT
                   (SELECT settlement_status FROM trade.order_main WHERE order_id = $1::text::uuid),
                   (SELECT dispute_status FROM trade.order_main WHERE order_id = $1::text::uuid),
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'refund_adjustment' AND event_source = 'settlement_dispute_hold'),
                   (SELECT settlement_status FROM billing.settlement_record WHERE order_id = $1::text::uuid ORDER BY created_at DESC, settlement_id DESC LIMIT 1),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'billing.adjustment.provisional_hold')",
                &[&dispute_seed.order_id, &create_case_request_id],
            )
            .await
            .expect("query dispute row");
        assert_eq!(dispute_row.get::<_, String>(0), "frozen");
        assert_eq!(dispute_row.get::<_, String>(1), "open");
        assert_eq!(dispute_row.get::<_, i64>(2), 1);
        assert_eq!(dispute_row.get::<_, String>(3), "frozen");
        assert_eq!(dispute_row.get::<_, i64>(4), 1);

        cleanup_share_order(&client, &cycle_seed, None).await;
        let case_id = dispute_case["data"]["case_id"]
            .as_str()
            .expect("dispute case id");
        cleanup_share_order(&client, &dispute_seed, Some(case_id)).await;
        let _ = opening_event_id;
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
                    .expect("billing request"),
            )
            .await
            .expect("billing response");
        json_response(response, StatusCode::OK).await
    }

    async fn transition_share_enable(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/share-ro/transition"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"enable_share"}"#))
                    .expect("enable request"),
            )
            .await
            .expect("enable response");
        json_response(response, StatusCode::OK).await
    }

    async fn grant_share(
        app: &Router,
        order_id: &str,
        seller_org_id: &str,
        asset_object_id: &str,
        request_id: &str,
        suffix: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/share-grants"))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", seller_org_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "operation": "grant",
                            "asset_object_id": asset_object_id,
                            "recipient_ref": format!("warehouse://buyer/{suffix}"),
                            "subscriber_ref": format!("sub-{suffix}"),
                            "share_protocol": "share_grant",
                            "access_locator": format!("share://seller/{suffix}/dataset"),
                            "scope_json": {"schema": "analytics", "tables": ["orders"]},
                            "expires_at": "2026-06-01T00:00:00Z",
                            "receipt_hash": format!("share-receipt-{suffix}"),
                            "metadata": {"entry": "bil026"}
                        })
                        .to_string(),
                    ))
                    .expect("grant request"),
            )
            .await
            .expect("grant response");
        json_response(response, StatusCode::OK).await
    }

    async fn revoke_share(
        app: &Router,
        order_id: &str,
        seller_org_id: &str,
        request_id: &str,
        suffix: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/share-grants"))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", seller_org_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "operation": "revoke",
                            "receipt_hash": format!("share-revoke-{suffix}"),
                            "metadata": {"reason": "manual revoke"}
                        })
                        .to_string(),
                    ))
                    .expect("revoke request"),
            )
            .await
            .expect("revoke response");
        json_response(response, StatusCode::OK).await
    }

    async fn record_cycle_charge(
        app: &Router,
        order_id: &str,
        request_id: &str,
        billing_cycle_code: &str,
        billing_amount: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/billing/{order_id}/share-ro/cycle-charge"))
                    .header("x-role", "platform_finance_operator")
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "billing_cycle_code": billing_cycle_code,
                            "billing_amount": billing_amount,
                            "reason_note": "share monthly cycle"
                        })
                        .to_string(),
                    ))
                    .expect("cycle charge request"),
            )
            .await
            .expect("cycle charge response");
        json_response(response, StatusCode::OK).await
    }

    async fn create_case(
        app: &Router,
        buyer_org_id: &str,
        buyer_user_id: &str,
        order_id: &str,
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
                            "reason_code": "share_scope_invalid",
                            "requested_resolution": "refund",
                            "claimed_amount": "66.00000000",
                            "evidence_scope": "share_locator,query_probe",
                            "blocking_effect": "freeze_settlement",
                            "metadata": {"entry": "bil026"}
                        })
                        .to_string(),
                    ))
                    .expect("create case request"),
            )
            .await
            .expect("create case response");
        json_response(response, StatusCode::OK).await
    }

    async fn json_response(response: axum::response::Response, expected: StatusCode) -> Value {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        assert_eq!(status, expected, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).expect("json body")
    }

    async fn seed_share_order(client: &Client, suffix: &str) -> Result<ShareOrderSeed, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil026-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil026-seller-{suffix}")],
            )
            .await?
            .get(0);
        let buyer_user_id: String = client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status, email, attrs)
                 VALUES ($1::text::uuid, $2, $3, 'human', 'active', 'verified', $4, '{}'::jsonb)
                 RETURNING user_id::text",
                &[
                    &buyer_org_id,
                    &format!("bil026-user-{suffix}"),
                    &format!("BIL026 User {suffix}"),
                    &format!("bil026-{suffix}@example.com"),
                ],
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
                    &format!("bil026-asset-{suffix}"),
                    &format!("bil026 asset {suffix}"),
                ],
            )
            .await?
            .get(0);
        let asset_version_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   1024, 'CN', ARRAY['CN']::text[], false,
                   '{"share_mode":"readonly"}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await?
            .get(0);
        let asset_object_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_object_binding (
                   asset_version_id, object_kind, object_name, object_locator, share_protocol,
                   schema_json, output_schema_json, freshness_json, access_constraints, metadata
                 ) VALUES (
                   $1::text::uuid, 'share_object', $2, $3, 'share_grant',
                   '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb
                 )
                 RETURNING asset_object_id::text",
                &[
                    &asset_version_id,
                    &format!("bil026-share-{suffix}"),
                    &format!("share://seller/{suffix}/source"),
                ],
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
                   $5, 'listed', 'subscription', 66.00, 'CNY', 'read_only_share',
                   ARRAY['internal_use']::text[], $6,
                   '{\"review_status\":\"approved\",\"share_protocol\":\"share_grant\"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil026-product-{suffix}"),
                    &format!("bil026 product {suffix}"),
                    &format!("bil026 search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
                   delivery_object_kind, share_protocol, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '月', 'subscription', 'subscription_access',
                   'share_grant', 'share_grant', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("BIL026-SKU-{suffix}")],
            )
            .await?
            .get(0);
        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json, delivery_route_snapshot, trust_boundary_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none',
                   'online', 66.00, 'CNY',
                   jsonb_build_object(
                     'sku_type', 'SHARE_RO',
                     'selected_sku_type', 'SHARE_RO',
                     'pricing_mode', 'subscription',
                     'billing_mode', 'subscription',
                     'settlement_basis', 'periodic',
                     'refund_mode', 'manual_refund',
                     'refund_template_code', 'REFUND_SHARE_RO_V1'
                   ),
                   'share_link',
                   '{\"share_delivery\":\"readonly\"}'::jsonb
                 )
                 RETURNING order_id::text"#,
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);
        client
            .execute(
                r#"INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{"term_days":30}'::jsonb
                 )"#,
                &[&order_id, &format!("sha256:bil026:{suffix}")],
            )
            .await?;
        Ok(ShareOrderSeed {
            buyer_org_id,
            seller_org_id,
            buyer_user_id,
            asset_id,
            asset_version_id,
            asset_object_id,
            product_id,
            sku_id,
            order_id,
        })
    }

    async fn cleanup_share_order(client: &Client, seed: &ShareOrderSeed, case_id: Option<&str>) {
        if let Some(case_id) = case_id {
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
        }
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event WHERE ordering_key = $1 OR aggregate_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.data_share_grant WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
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
                "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.digital_contract WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
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
                "DELETE FROM catalog.asset_object_binding WHERE asset_object_id = $1::text::uuid",
                &[&seed.asset_object_id],
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
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await;
    }
}
