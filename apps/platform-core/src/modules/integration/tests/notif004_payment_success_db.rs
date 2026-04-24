use super::notification_test_support::wait_for_mock_log_chain_if_enabled;
use crate::modules::billing::api::router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, GenericClient, NoTls, connect};
use serde_json::{Value, json};
use tower::util::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("NOTIF_DB_SMOKE").ok().as_deref() == Some("1")
}

#[tokio::test]
async fn notif004_payment_success_notifications_db_smoke() {
    if !live_db_enabled() {
        eprintln!("skip notif004_payment_success_notifications_db_smoke; set NOTIF_DB_SMOKE=1");
        return;
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let (client, connection) = connect(&database_url, NoTls)
        .await
        .expect("connect postgres");
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
    let occurred_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis() as i64;
    let request_id = format!("req-notif004-webhook-{suffix}");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments/webhooks/mock_payment")
                .header("content-type", "application/json")
                .header("x-provider-signature", "mock-signature")
                .header("x-webhook-timestamp", occurred_at_ms.to_string())
                .header("x-request-id", &request_id)
                .body(Body::from(format!(
                    r#"{{
                      "provider_event_id":"evt-notif004-success-{suffix}",
                      "event_type":"payment.succeeded",
                      "payment_intent_id":"{}",
                      "provider_status":"succeeded",
                      "occurred_at_ms":{}
                    }}"#,
                    seed.payment_intent_id, occurred_at_ms
                )))
                .expect("request should build"),
        )
        .await
        .expect("webhook response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let json: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(json["data"]["processed_status"], "processed");

    let rows = client
        .query(
            "SELECT
               payload,
               idempotency_key,
               target_topic
             FROM ops.outbox_event
             WHERE target_topic = 'dtp.notification.dispatch'
               AND request_id = $1
             ORDER BY created_at ASC, outbox_event_id ASC",
            &[&request_id],
        )
        .await
        .expect("load notification outbox rows");
    assert_eq!(
        rows.len(),
        3,
        "payment success should emit buyer/seller/ops notifications"
    );

    let payloads = rows
        .iter()
        .map(|row| row.get::<_, Value>(0))
        .collect::<Vec<_>>();
    for row in &rows {
        assert_eq!(row.get::<_, String>(2), "dtp.notification.dispatch");
    }

    let buyer = find_payload(&payloads, "buyer");
    assert_eq!(
        buyer["payload"]["notification_code"].as_str(),
        Some("payment.succeeded")
    );
    assert_eq!(
        buyer["payload"]["template_code"].as_str(),
        Some("NOTIFY_PAYMENT_SUCCEEDED_V1")
    );
    assert_eq!(
        buyer["payload"]["source_event"]["aggregate_type"].as_str(),
        Some("billing.billing_event")
    );
    assert_eq!(
        buyer["payload"]["source_event"]["event_type"].as_str(),
        Some("billing.event.recorded")
    );
    assert!(
        buyer["payload"]["metadata"]
            .get("payment_intent_id")
            .is_none(),
        "buyer payload metadata must not expose internal ids"
    );
    assert!(
        buyer["payload"]["variables"]
            .get("provider_reference_id")
            .is_none(),
        "buyer variables must not expose provider reference"
    );

    let seller = find_payload(&payloads, "seller");
    assert_eq!(
        seller["payload"]["notification_code"].as_str(),
        Some("order.pending_delivery")
    );
    assert_eq!(
        seller["payload"]["template_code"].as_str(),
        Some("NOTIFY_PENDING_DELIVERY_V1")
    );
    assert_eq!(
        seller["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(false)
    );
    assert!(
        seller["payload"]["metadata"]
            .get("payment_intent_id")
            .is_none(),
        "seller payload metadata must not expose internal ids"
    );

    let ops = find_payload(&payloads, "ops");
    assert_eq!(
        ops["payload"]["notification_code"].as_str(),
        Some("order.pending_delivery")
    );
    assert_eq!(
        ops["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(true)
    );
    assert_eq!(
        ops["payload"]["variables"]["payment_intent_id"].as_str(),
        Some(seed.payment_intent_id.as_str())
    );
    assert!(
        ops["payload"]["metadata"]
            .get("payment_intent_id")
            .is_some(),
        "ops payload metadata should preserve internal linkage"
    );

    let audit_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = 'payment.webhook.processed'",
            &[&request_id],
        )
        .await
        .expect("count webhook audit")
        .get(0);
    assert_eq!(audit_count, 1);

    let live_chain = wait_for_mock_log_chain_if_enabled(
        &client,
        &request_id,
        &[
            "payment.succeeded",
            "order.pending_delivery",
            "order.pending_delivery",
        ],
    )
    .await;

    crate::write_test027_artifact(
        "notif004-payment-success.json",
        &json!({
            "request_id": &request_id,
            "seed": {
                "order_id": &seed.order_id,
                "payment_intent_id": &seed.payment_intent_id,
            },
            "response": json,
            "webhook_audit_count": audit_count,
            "outbox": {
                "count": rows.len(),
                "notification_codes": payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["notification_code"].as_str())
                    .collect::<Vec<_>>(),
                "template_codes": payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["template_code"].as_str())
                    .collect::<Vec<_>>(),
            },
            "live_chain": live_chain,
        }),
    );

    cleanup_seed_graph(&client, &seed, &request_id).await;
}

fn find_payload<'a>(payloads: &'a [Value], audience_scope: &str) -> &'a Value {
    payloads
        .iter()
        .find(|payload| payload["payload"]["audience_scope"].as_str() == Some(audience_scope))
        .unwrap_or_else(|| panic!("missing payload for audience {audience_scope}"))
}

struct SeedGraph {
    buyer_org_id: String,
    seller_org_id: String,
    platform_org_id: String,
    buyer_user_id: String,
    seller_user_id: String,
    platform_user_id: String,
    asset_id: String,
    asset_version_id: String,
    product_id: String,
    sku_id: String,
    order_id: String,
    payment_intent_id: String,
}

async fn seed_graph(client: &Client, suffix: &str) -> SeedGraph {
    let buyer_org_id = seed_org(client, &format!("notif004-buyer-{suffix}"), "enterprise").await;
    let seller_org_id = seed_org(client, &format!("notif004-seller-{suffix}"), "enterprise").await;
    let platform_org_id =
        seed_org(client, &format!("notif004-platform-{suffix}"), "platform").await;
    let buyer_user_id = seed_user(
        client,
        &buyer_org_id,
        &format!("notif004-buyer-user-{suffix}"),
        "buyer_operator",
    )
    .await;
    let seller_user_id = seed_user(
        client,
        &seller_org_id,
        &format!("notif004-seller-user-{suffix}"),
        "seller_operator",
    )
    .await;
    let platform_user_id = seed_user(
        client,
        &platform_org_id,
        &format!("notif004-platform-user-{suffix}"),
        "platform_admin",
    )
    .await;

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
                &format!("notif004-asset-{suffix}"),
                &format!("notif004 asset {suffix}"),
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
               1024, 'CN', ARRAY['CN']::text[], false,
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
               $5, 'listed', 'one_time', 128.00, 'CNY', 'file_download',
               ARRAY['analytics']::text[], $6, '{"review_status":"approved"}'::jsonb
             )
             RETURNING product_id::text"#,
            &[
                &asset_id,
                &asset_version_id,
                &seller_org_id,
                &format!("notif004-product-{suffix}"),
                &format!("notif004 product {suffix}"),
                &format!("notif004 summary {suffix}"),
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
             )
             RETURNING sku_id::text",
            &[&product_id, &format!("NOTIF004-SKU-{suffix}")],
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
               payment_mode,
               amount,
               currency_code,
               fee_preview_snapshot,
               payment_channel_snapshot,
               buyer_deposit_amount,
               seller_deposit_amount,
               price_snapshot_json,
               trust_boundary_snapshot,
               storage_mode_snapshot,
               delivery_route_snapshot,
               platform_plaintext_access_snapshot,
               idempotency_key,
               delivery_status,
               acceptance_status,
               settlement_status,
               dispute_status,
               last_reason_code
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3::text::uuid,
               $4::text::uuid,
               $5::text::uuid,
               'contract_effective',
               'unpaid',
               'online',
               128.00,
               'CNY',
               '{}'::jsonb,
               '{"channel":"mock_payment"}'::jsonb,
               0,
               0,
               '{"pricing_mode":"one_time","billing_mode":"one_time","settlement_basis":"one_time"}'::jsonb,
               '{}'::jsonb,
               'platform_custody',
               'signed_url',
               false,
               $6,
               'not_started',
               'not_started',
               'not_started',
               'none',
               'notif004_seed_contract_effective'
             )
             RETURNING order_id::text"#,
            &[
                &product_id,
                &asset_version_id,
                &buyer_org_id,
                &seller_org_id,
                &sku_id,
                &format!("notif004-order-{suffix}"),
            ],
        )
        .await
        .expect("insert order")
        .get(0);
    let payment_intent_id: String = client
        .query_one(
            r#"INSERT INTO payment.payment_intent (
               order_id,
               intent_type,
               provider_key,
               payer_subject_type,
               payer_subject_id,
               payee_subject_type,
               payee_subject_id,
               payer_jurisdiction_code,
               payee_jurisdiction_code,
               launch_jurisdiction_code,
               amount,
               payment_method,
               price_currency_code,
               currency_code,
               status,
               request_id,
               expire_at,
               capability_snapshot,
               metadata
             ) VALUES (
               $1::text::uuid,
               'order_payment',
               'mock_payment',
               'organization',
               $2::text::uuid,
               'organization',
               $3::text::uuid,
               'SG',
               'SG',
               'SG',
               128.00,
               'wallet',
               'CNY',
               'CNY',
               'pending',
               $4,
               now() + interval '1 day',
               '{}'::jsonb,
               '{"seed":"notif004"}'::jsonb
             )
             RETURNING payment_intent_id::text"#,
            &[
                &order_id,
                &buyer_org_id,
                &seller_org_id,
                &format!("req-notif004-intent-{suffix}"),
            ],
        )
        .await
        .expect("insert payment intent")
        .get(0);

    SeedGraph {
        buyer_org_id,
        seller_org_id,
        platform_org_id,
        buyer_user_id,
        seller_user_id,
        platform_user_id,
        asset_id,
        asset_version_id,
        product_id,
        sku_id,
        order_id,
        payment_intent_id,
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

async fn seed_user(client: &Client, org_id: &str, suffix: &str, persona: &str) -> String {
    client
        .query_one(
            "INSERT INTO core.user_account (
               org_id, login_id, display_name, user_type, status, mfa_status, email, attrs
             ) VALUES (
               $1::text::uuid, $2, $3, 'human', 'active', 'enabled', $4, $5::jsonb
             )
             RETURNING user_id::text",
            &[
                &org_id,
                &format!("{persona}.{suffix}@example.test"),
                &format!("NOTIF004 {}", persona.replace('_', " ")),
                &format!("{persona}.{suffix}@example.test"),
                &json!({ "persona": persona }),
            ],
        )
        .await
        .expect("insert user")
        .get(0)
}

async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph, request_id: &str) {
    let _ = client
        .execute(
            "DELETE FROM ops.outbox_event
             WHERE request_id = $1
                OR aggregate_id = $2::text::uuid
                OR aggregate_id = $3::text::uuid
                OR aggregate_id = $4::text::uuid",
            &[
                &request_id,
                &seed.order_id,
                &seed.payment_intent_id,
                &seed.product_id,
            ],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM audit.audit_event
             WHERE request_id = $1
                OR ref_id = $2::text::uuid
                OR ref_id = $3::text::uuid",
            &[&request_id, &seed.order_id, &seed.payment_intent_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM payment.payment_transaction WHERE payment_intent_id = $1::text::uuid",
            &[&seed.payment_intent_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM payment.payment_webhook_event WHERE payment_intent_id = $1::text::uuid",
            &[&seed.payment_intent_id],
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
            "DELETE FROM core.user_account
             WHERE user_id = ANY($1::uuid[])",
            &[&vec![
                seed.buyer_user_id.to_string(),
                seed.seller_user_id.to_string(),
                seed.platform_user_id.to_string(),
            ]],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM core.organization
             WHERE org_id = ANY($1::uuid[])",
            &[&vec![
                seed.buyer_org_id.to_string(),
                seed.seller_org_id.to_string(),
                seed.platform_org_id.to_string(),
            ]],
        )
        .await;
}
