use super::notification_test_support::wait_for_mock_log_chain_if_enabled;
use crate::modules::integration::application::{
    AcceptanceOutcomeNotificationDispatchInput, BillingResolutionNotificationDispatchInput,
    queue_acceptance_outcome_notifications, queue_billing_resolution_notifications,
};
use db::{Client, GenericClient, NoTls, connect};
use serde_json::{Value, json};

fn live_db_enabled() -> bool {
    std::env::var("NOTIF_DB_SMOKE").ok().as_deref() == Some("1")
}

#[tokio::test]
async fn notif006_acceptance_outcome_notifications_db_smoke() {
    if !live_db_enabled() {
        eprintln!("skip notif006_acceptance_outcome_notifications_db_smoke; set NOTIF_DB_SMOKE=1");
        return;
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let (client, connection) = connect(&database_url, NoTls)
        .await
        .expect("connect postgres");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    assert_active_template_version(&client, "NOTIFY_ACCEPTANCE_PASSED_V1", 2).await;
    assert_active_template_version(&client, "NOTIFY_ACCEPTANCE_REJECTED_V1", 2).await;

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let seed = seed_acceptance_graph(&client, &suffix).await;

    let passed = queue_acceptance_outcome_notifications(
        &client,
        AcceptanceOutcomeNotificationDispatchInput {
            order_id: &seed.passed.order_id,
            acceptance_record_id: &seed.passed.delivery_id,
            scene: "acceptance.passed",
            occurred_at: Some("2026-04-22T00:00:00.000Z"),
            reason_code: "delivery_accept_passed",
            reason_detail: Some("hash verified and contract matched"),
            verification_summary: Some(&json!({
                "hash_match": true,
                "contract_template_match": true
            })),
            request_id: Some(&seed.passed.request_id),
            trace_id: Some(&seed.passed.trace_id),
        },
    )
    .await
    .expect("queue acceptance passed notifications");
    assert_eq!(passed.inserted_count, 3);
    assert_eq!(passed.replayed_count, 0);

    let passed_replay = queue_acceptance_outcome_notifications(
        &client,
        AcceptanceOutcomeNotificationDispatchInput {
            order_id: &seed.passed.order_id,
            acceptance_record_id: &seed.passed.delivery_id,
            scene: "acceptance.passed",
            occurred_at: Some("2026-04-22T00:00:00.000Z"),
            reason_code: "delivery_accept_passed",
            reason_detail: Some("hash verified and contract matched"),
            verification_summary: Some(&json!({
                "hash_match": true,
                "contract_template_match": true
            })),
            request_id: Some(&seed.passed.request_id),
            trace_id: Some(&seed.passed.trace_id),
        },
    )
    .await
    .expect("replay acceptance passed notifications");
    assert_eq!(passed_replay.inserted_count, 0);
    assert_eq!(passed_replay.replayed_count, 3);
    assert_eq!(passed_replay.idempotency_keys, passed.idempotency_keys);

    let passed_payloads = load_notification_payloads(&client, &seed.passed.request_id).await;
    assert_eq!(passed_payloads.len(), 3);
    let buyer = find_payload(&passed_payloads, "buyer");
    assert_eq!(
        buyer["payload"]["notification_code"].as_str(),
        Some("acceptance.passed")
    );
    assert_eq!(
        buyer["payload"]["template_code"].as_str(),
        Some("NOTIFY_ACCEPTANCE_PASSED_V1")
    );
    assert_eq!(
        buyer["payload"]["source_event"]["aggregate_type"].as_str(),
        Some("trade.acceptance_record")
    );
    assert_eq!(
        buyer["payload"]["source_event"]["event_type"].as_str(),
        Some("acceptance.passed")
    );
    assert_eq!(
        buyer["payload"]["source_event"]["aggregate_id"].as_str(),
        Some(seed.passed.delivery_id.as_str())
    );
    assert_eq!(
        buyer["payload"]["variables"]["action_href"].as_str(),
        Some(format!("/trade/orders/{}", seed.passed.order_id).as_str())
    );
    assert_eq!(
        buyer["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(false)
    );
    assert!(
        buyer["payload"]["variables"]["action_summary"]
            .as_str()
            .is_some()
    );

    let seller = find_payload(&passed_payloads, "seller");
    assert_eq!(
        seller["payload"]["variables"]["action_href"].as_str(),
        Some(format!("/trade/orders/{}", seed.passed.order_id).as_str())
    );
    assert_eq!(
        seller["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(false)
    );

    let ops = find_payload(&passed_payloads, "ops");
    assert_eq!(
        ops["payload"]["variables"]["action_href"].as_str(),
        Some(format!("/billing?order_id={}", seed.passed.order_id).as_str())
    );
    assert_eq!(
        ops["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(true)
    );
    assert_eq!(
        ops["payload"]["variables"]["acceptance_record_id"].as_str(),
        Some(seed.passed.delivery_id.as_str())
    );

    let rejected = queue_acceptance_outcome_notifications(
        &client,
        AcceptanceOutcomeNotificationDispatchInput {
            order_id: &seed.rejected.order_id,
            acceptance_record_id: &seed.rejected.delivery_id,
            scene: "acceptance.rejected",
            occurred_at: Some("2026-04-22T00:30:00.000Z"),
            reason_code: "report_quality_failed",
            reason_detail: Some("sample section mismatched template"),
            verification_summary: Some(&json!({
                "hash_match": true,
                "report_section_check": false
            })),
            request_id: Some(&seed.rejected.request_id),
            trace_id: Some(&seed.rejected.trace_id),
        },
    )
    .await
    .expect("queue acceptance rejected notifications");
    assert_eq!(rejected.inserted_count, 3);
    assert_eq!(rejected.replayed_count, 0);

    let rejected_payloads = load_notification_payloads(&client, &seed.rejected.request_id).await;
    assert_eq!(rejected_payloads.len(), 3);
    let buyer = find_payload(&rejected_payloads, "buyer");
    assert_eq!(
        buyer["payload"]["notification_code"].as_str(),
        Some("acceptance.rejected")
    );
    assert_eq!(
        buyer["payload"]["template_code"].as_str(),
        Some("NOTIFY_ACCEPTANCE_REJECTED_V1")
    );
    assert_eq!(
        buyer["payload"]["variables"]["action_href"].as_str(),
        Some(format!("/support/cases/new?order_id={}", seed.rejected.order_id).as_str())
    );

    let seller = find_payload(&rejected_payloads, "seller");
    assert_eq!(
        seller["payload"]["variables"]["action_href"].as_str(),
        Some(format!("/trade/orders/{}", seed.rejected.order_id).as_str())
    );

    let ops = find_payload(&rejected_payloads, "ops");
    assert_eq!(
        ops["payload"]["variables"]["action_href"].as_str(),
        Some(format!("/support/cases/new?order_id={}", seed.rejected.order_id).as_str())
    );
    assert_eq!(
        ops["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(true)
    );

    let passed_live_chain = wait_for_mock_log_chain_if_enabled(
        &client,
        &seed.passed.request_id,
        &[
            "acceptance.passed",
            "acceptance.passed",
            "acceptance.passed",
        ],
    )
    .await;
    let rejected_live_chain = wait_for_mock_log_chain_if_enabled(
        &client,
        &seed.rejected.request_id,
        &[
            "acceptance.rejected",
            "acceptance.rejected",
            "acceptance.rejected",
        ],
    )
    .await;
    crate::write_test027_artifact(
        "notif006-acceptance-outcome.json",
        &json!({
            "passed": {
                "request_id": &seed.passed.request_id,
                "trace_id": &seed.passed.trace_id,
                "order_id": &seed.passed.order_id,
                "acceptance_record_id": &seed.passed.delivery_id,
                "notification_codes": passed_payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["notification_code"].as_str())
                    .collect::<Vec<_>>(),
                "template_codes": passed_payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["template_code"].as_str())
                    .collect::<Vec<_>>(),
                "live_chain": passed_live_chain,
            },
            "rejected": {
                "request_id": &seed.rejected.request_id,
                "trace_id": &seed.rejected.trace_id,
                "order_id": &seed.rejected.order_id,
                "acceptance_record_id": &seed.rejected.delivery_id,
                "notification_codes": rejected_payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["notification_code"].as_str())
                    .collect::<Vec<_>>(),
                "template_codes": rejected_payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["template_code"].as_str())
                    .collect::<Vec<_>>(),
                "live_chain": rejected_live_chain,
            },
        }),
    );

    cleanup_acceptance_graph(&client, &seed).await;
}

#[tokio::test]
async fn notif006_billing_resolution_notifications_db_smoke() {
    if !live_db_enabled() {
        eprintln!("skip notif006_billing_resolution_notifications_db_smoke; set NOTIF_DB_SMOKE=1");
        return;
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let (client, connection) = connect(&database_url, NoTls)
        .await
        .expect("connect postgres");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    assert_active_template_version(&client, "NOTIFY_REFUND_COMPLETED_V1", 2).await;
    assert_active_template_version(&client, "NOTIFY_COMPENSATION_COMPLETED_V1", 2).await;

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let seed = seed_billing_graph(&client, &suffix).await;

    let refund = queue_billing_resolution_notifications(
        &client,
        BillingResolutionNotificationDispatchInput {
            order_id: &seed.refund.order_id,
            billing_event_id: &seed.refund.billing_event_id,
            scene: "refund.completed",
            occurred_at: Some("2026-04-22T01:00:00.000Z"),
            request_id: Some(&seed.refund.request_id),
            trace_id: Some(&seed.refund.trace_id),
        },
    )
    .await
    .expect("queue refund notifications");
    assert_eq!(refund.inserted_count, 3);
    assert_eq!(refund.replayed_count, 0);
    let refund_replay = queue_billing_resolution_notifications(
        &client,
        BillingResolutionNotificationDispatchInput {
            order_id: &seed.refund.order_id,
            billing_event_id: &seed.refund.billing_event_id,
            scene: "refund.completed",
            occurred_at: Some("2026-04-22T01:00:00.000Z"),
            request_id: Some(&seed.refund.request_id),
            trace_id: Some(&seed.refund.trace_id),
        },
    )
    .await
    .expect("replay refund notifications");
    assert_eq!(refund_replay.inserted_count, 0);
    assert_eq!(refund_replay.replayed_count, 3);
    assert_eq!(refund_replay.idempotency_keys, refund.idempotency_keys);

    let refund_payloads = load_notification_payloads(&client, &seed.refund.request_id).await;
    assert_eq!(refund_payloads.len(), 3);
    let buyer = find_payload(&refund_payloads, "buyer");
    assert_eq!(
        buyer["payload"]["notification_code"].as_str(),
        Some("refund.completed")
    );
    assert_eq!(
        buyer["payload"]["template_code"].as_str(),
        Some("NOTIFY_REFUND_COMPLETED_V1")
    );
    assert_eq!(
        buyer["payload"]["source_event"]["aggregate_id"].as_str(),
        Some(seed.refund.billing_event_id.as_str())
    );
    assert_eq!(
        buyer["payload"]["variables"]["action_href"].as_str(),
        Some(
            format!(
                "/billing/refunds?order_id={}&case_id={}",
                seed.refund.order_id, seed.refund.case_id
            )
            .as_str()
        )
    );
    assert_eq!(
        buyer["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(false)
    );

    let ops = find_payload(&refund_payloads, "ops");
    assert_eq!(
        ops["payload"]["variables"]["provider_result_id"].as_str(),
        Some(seed.refund.provider_result_id.as_str())
    );
    assert_eq!(
        ops["payload"]["variables"]["resolution_ref_type"].as_str(),
        Some("refund_record")
    );
    assert_eq!(
        ops["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(true)
    );

    let compensation = queue_billing_resolution_notifications(
        &client,
        BillingResolutionNotificationDispatchInput {
            order_id: &seed.compensation.order_id,
            billing_event_id: &seed.compensation.billing_event_id,
            scene: "compensation.completed",
            occurred_at: Some("2026-04-22T01:30:00.000Z"),
            request_id: Some(&seed.compensation.request_id),
            trace_id: Some(&seed.compensation.trace_id),
        },
    )
    .await
    .expect("queue compensation notifications");
    assert_eq!(compensation.inserted_count, 3);
    assert_eq!(compensation.replayed_count, 0);

    let compensation_payloads =
        load_notification_payloads(&client, &seed.compensation.request_id).await;
    assert_eq!(compensation_payloads.len(), 3);
    let buyer = find_payload(&compensation_payloads, "buyer");
    assert_eq!(
        buyer["payload"]["notification_code"].as_str(),
        Some("compensation.completed")
    );
    assert_eq!(
        buyer["payload"]["template_code"].as_str(),
        Some("NOTIFY_COMPENSATION_COMPLETED_V1")
    );
    assert_eq!(
        buyer["payload"]["variables"]["action_href"].as_str(),
        Some(
            format!(
                "/billing/refunds?order_id={}&case_id={}",
                seed.compensation.order_id, seed.compensation.case_id
            )
            .as_str()
        )
    );

    let ops = find_payload(&compensation_payloads, "ops");
    assert_eq!(
        ops["payload"]["variables"]["provider_result_id"].as_str(),
        Some(seed.compensation.provider_result_id.as_str())
    );
    assert_eq!(
        ops["payload"]["variables"]["resolution_ref_type"].as_str(),
        Some("compensation_record")
    );
    assert_eq!(
        ops["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(true)
    );

    crate::write_test027_artifact(
        "notif006-billing-resolution.json",
        &json!({
            "refund": {
                "request_id": &seed.refund.request_id,
                "trace_id": &seed.refund.trace_id,
                "order_id": &seed.refund.order_id,
                "case_id": &seed.refund.case_id,
                "billing_event_id": &seed.refund.billing_event_id,
                "notification_codes": refund_payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["notification_code"].as_str())
                    .collect::<Vec<_>>(),
            },
            "compensation": {
                "request_id": &seed.compensation.request_id,
                "trace_id": &seed.compensation.trace_id,
                "order_id": &seed.compensation.order_id,
                "case_id": &seed.compensation.case_id,
                "billing_event_id": &seed.compensation.billing_event_id,
                "notification_codes": compensation_payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["notification_code"].as_str())
                    .collect::<Vec<_>>(),
            },
        }),
    );

    cleanup_billing_graph(&client, &seed).await;
}

async fn load_notification_payloads(client: &Client, request_id: &str) -> Vec<Value> {
    client
        .query(
            "SELECT payload
             FROM ops.outbox_event
             WHERE request_id = $1
               AND target_topic = 'dtp.notification.dispatch'
             ORDER BY created_at ASC, outbox_event_id ASC",
            &[&request_id],
        )
        .await
        .expect("load notification payloads")
        .into_iter()
        .map(|row| row.get(0))
        .collect()
}

fn find_payload<'a>(payloads: &'a [Value], audience_scope: &str) -> &'a Value {
    payloads
        .iter()
        .find(|payload| payload["payload"]["audience_scope"].as_str() == Some(audience_scope))
        .unwrap_or_else(|| panic!("missing payload for audience {audience_scope}"))
}

async fn assert_active_template_version(
    client: &Client,
    template_code: &str,
    expected_version: i32,
) {
    let row = client
        .query_one(
            "SELECT version_no, metadata
             FROM ops.notification_template
             WHERE template_code = $1
               AND language_code = 'zh-CN'
               AND channel = 'mock-log'
               AND enabled = TRUE
               AND status = 'active'
             ORDER BY version_no DESC
             LIMIT 1",
            &[&template_code],
        )
        .await
        .expect("load active notification template");
    assert_eq!(row.get::<_, i32>(0), expected_version);
    assert_eq!(
        row.get::<_, Value>(1)["seed_task"].as_str(),
        Some("NOTIF-006")
    );
}

struct AcceptanceGraph {
    buyer_org_id: String,
    seller_org_id: String,
    platform_org_id: String,
    buyer_user_id: String,
    seller_user_id: String,
    platform_user_id: String,
    asset_id: String,
    asset_version_id: String,
    passed: AcceptanceOrder,
    rejected: AcceptanceOrder,
}

struct AcceptanceOrder {
    product_id: String,
    sku_id: String,
    order_id: String,
    delivery_id: String,
    request_id: String,
    trace_id: String,
}

async fn seed_acceptance_graph(client: &Client, suffix: &str) -> AcceptanceGraph {
    let buyer_org_id = seed_org(
        client,
        &format!("notif006-acc-buyer-{suffix}"),
        "enterprise",
    )
    .await;
    let seller_org_id = seed_org(
        client,
        &format!("notif006-acc-seller-{suffix}"),
        "enterprise",
    )
    .await;
    let platform_org_id = seed_org(
        client,
        &format!("notif006-acc-platform-{suffix}"),
        "platform",
    )
    .await;
    let buyer_user_id = seed_user(
        client,
        &buyer_org_id,
        &format!("notif006-acc-buyer-{suffix}"),
        "buyer_operator",
    )
    .await;
    let seller_user_id = seed_user(
        client,
        &seller_org_id,
        &format!("notif006-acc-seller-{suffix}"),
        "seller_operator",
    )
    .await;
    let platform_user_id = seed_user(
        client,
        &platform_org_id,
        &format!("notif006-acc-platform-{suffix}"),
        "platform_admin",
    )
    .await;
    let asset_id = seed_asset(client, &seller_org_id, &format!("notif006-acc-{suffix}")).await;
    let asset_version_id = seed_asset_version(client, &asset_id).await;

    let passed = seed_acceptance_order(
        client,
        &buyer_org_id,
        &seller_org_id,
        &asset_id,
        &asset_version_id,
        &format!("notif006-acc-passed-{suffix}"),
        "FILE_STD",
        "file_download",
        "result_package",
        "accepted",
        "accepted",
        "pending_settlement",
        "none",
    )
    .await;
    let rejected = seed_acceptance_order(
        client,
        &buyer_org_id,
        &seller_org_id,
        &asset_id,
        &asset_version_id,
        &format!("notif006-acc-rejected-{suffix}"),
        "RPT_STD",
        "report_delivery",
        "result_package",
        "rejected",
        "rejected",
        "blocked",
        "open",
    )
    .await;

    AcceptanceGraph {
        buyer_org_id,
        seller_org_id,
        platform_org_id,
        buyer_user_id,
        seller_user_id,
        platform_user_id,
        asset_id,
        asset_version_id,
        passed,
        rejected,
    }
}

#[allow(clippy::too_many_arguments)]
async fn seed_acceptance_order(
    client: &Client,
    buyer_org_id: &str,
    seller_org_id: &str,
    asset_id: &str,
    asset_version_id: &str,
    suffix: &str,
    sku_type: &str,
    delivery_type: &str,
    delivery_route: &str,
    order_status: &str,
    acceptance_status: &str,
    settlement_status: &str,
    dispute_status: &str,
) -> AcceptanceOrder {
    let product_id = seed_product(
        client,
        asset_id,
        asset_version_id,
        seller_org_id,
        suffix,
        delivery_type,
    )
    .await;
    let sku_id = seed_sku(
        client,
        &product_id,
        suffix,
        sku_type,
        "one_time",
        "manual_accept",
    )
    .await;
    let order_id: String = client
        .query_one(
            "INSERT INTO trade.order_main (
               product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
               status, payment_status, payment_mode, amount, currency_code,
               delivery_status, acceptance_status, settlement_status, dispute_status,
               price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
               $6, 'paid', 'online', 88.00, 'CNY',
               'delivered', $7, $8, $9,
               '{}'::jsonb,
               jsonb_build_object('delivery_mode', $10),
               $11
             )
             RETURNING order_id::text",
            &[
                &product_id,
                &asset_version_id,
                &buyer_org_id,
                &seller_org_id,
                &sku_id,
                &order_status,
                &acceptance_status,
                &settlement_status,
                &dispute_status,
                &delivery_type,
                &delivery_route,
            ],
        )
        .await
        .expect("insert acceptance order")
        .get(0);
    let delivery_id: String = client
        .query_one(
            "INSERT INTO delivery.delivery_record (
               order_id, delivery_type, delivery_route, status, trust_boundary_snapshot,
               sensitive_delivery_mode, disclosure_review_status, committed_at
             ) VALUES (
               $1::text::uuid, $2, $3, $4, '{}'::jsonb,
               'standard', 'not_required', now()
             )
             RETURNING delivery_id::text",
            &[
                &order_id,
                &delivery_type,
                &delivery_route,
                &acceptance_status,
            ],
        )
        .await
        .expect("insert acceptance delivery")
        .get(0);

    AcceptanceOrder {
        product_id,
        sku_id,
        order_id,
        delivery_id,
        request_id: format!("req-notif006-acceptance-{suffix}"),
        trace_id: format!("trace-notif006-acceptance-{suffix}"),
    }
}

async fn cleanup_acceptance_graph(client: &Client, graph: &AcceptanceGraph) {
    cleanup_notification_rows(
        client,
        &[
            graph.passed.request_id.as_str(),
            graph.rejected.request_id.as_str(),
        ],
    )
    .await;
    cleanup_acceptance_order(client, &graph.passed).await;
    cleanup_acceptance_order(client, &graph.rejected).await;
    cleanup_base_graph(client, graph).await;
}

async fn cleanup_acceptance_order(client: &Client, order: &AcceptanceOrder) {
    let _ = client
        .execute(
            "DELETE FROM delivery.delivery_record WHERE delivery_id = $1::text::uuid",
            &[&order.delivery_id],
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
}

struct BillingGraph {
    buyer_org_id: String,
    seller_org_id: String,
    platform_org_id: String,
    buyer_user_id: String,
    seller_user_id: String,
    platform_user_id: String,
    asset_id: String,
    asset_version_id: String,
    refund: BillingResolutionSeed,
    compensation: BillingResolutionSeed,
}

struct BillingResolutionSeed {
    product_id: String,
    sku_id: String,
    order_id: String,
    billing_event_id: String,
    resolution_record_id: String,
    case_id: String,
    request_id: String,
    trace_id: String,
    provider_result_id: String,
}

async fn seed_billing_graph(client: &Client, suffix: &str) -> BillingGraph {
    let buyer_org_id = seed_org(
        client,
        &format!("notif006-bil-buyer-{suffix}"),
        "enterprise",
    )
    .await;
    let seller_org_id = seed_org(
        client,
        &format!("notif006-bil-seller-{suffix}"),
        "enterprise",
    )
    .await;
    let platform_org_id = seed_org(
        client,
        &format!("notif006-bil-platform-{suffix}"),
        "platform",
    )
    .await;
    let buyer_user_id = seed_user(
        client,
        &buyer_org_id,
        &format!("notif006-bil-buyer-{suffix}"),
        "buyer_operator",
    )
    .await;
    let seller_user_id = seed_user(
        client,
        &seller_org_id,
        &format!("notif006-bil-seller-{suffix}"),
        "seller_operator",
    )
    .await;
    let platform_user_id = seed_user(
        client,
        &platform_org_id,
        &format!("notif006-bil-platform-{suffix}"),
        "platform_risk_settlement",
    )
    .await;
    let asset_id = seed_asset(client, &seller_org_id, &format!("notif006-bil-{suffix}")).await;
    let asset_version_id = seed_asset_version(client, &asset_id).await;

    let refund = seed_billing_resolution(
        client,
        &buyer_org_id,
        &seller_org_id,
        &asset_id,
        &asset_version_id,
        &format!("notif006-refund-{suffix}"),
        "refund.completed",
        "refund_id",
        "provider_refund_id",
        "REFUND_SUCCESS",
    )
    .await;
    let compensation = seed_billing_resolution(
        client,
        &buyer_org_id,
        &seller_org_id,
        &asset_id,
        &asset_version_id,
        &format!("notif006-compensation-{suffix}"),
        "compensation.completed",
        "compensation_id",
        "provider_transfer_id",
        "MANUAL_TRANSFER_SUCCESS",
    )
    .await;

    BillingGraph {
        buyer_org_id,
        seller_org_id,
        platform_org_id,
        buyer_user_id,
        seller_user_id,
        platform_user_id,
        asset_id,
        asset_version_id,
        refund,
        compensation,
    }
}

#[allow(clippy::too_many_arguments)]
async fn seed_billing_resolution(
    client: &Client,
    buyer_org_id: &str,
    seller_org_id: &str,
    asset_id: &str,
    asset_version_id: &str,
    suffix: &str,
    scene: &str,
    resolution_record_field: &str,
    provider_result_field: &str,
    provider_status: &str,
) -> BillingResolutionSeed {
    let product_id = seed_product(
        client,
        asset_id,
        asset_version_id,
        seller_org_id,
        suffix,
        "file_download",
    )
    .await;
    let sku_id = seed_sku(
        client,
        &product_id,
        suffix,
        "FILE_STD",
        "one_time",
        "manual_accept",
    )
    .await;
    let order_id: String = client
        .query_one(
            "INSERT INTO trade.order_main (
               product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
               status, payment_status, payment_mode, amount, currency_code,
               delivery_status, acceptance_status, settlement_status, dispute_status,
               price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
               'accepted', 'refunded', 'online', 128.00, 'CNY',
               'delivered', 'accepted', 'resolved', 'resolved',
               '{}'::jsonb,
               '{}'::jsonb,
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
        .await
        .expect("insert billing resolution order")
        .get(0);
    let case_id = new_uuid(client).await;
    let resolution_record_id = new_uuid(client).await;
    let billing_event_id = new_uuid(client).await;
    let provider_result_id = format!("{scene}-{suffix}");
    let metadata = json!({
        "case_id": case_id,
        "decision_code": if scene == "refund.completed" { "refund_full" } else { "compensation_full" },
        "penalty_code": "seller_fault",
        "reason_code": if scene == "refund.completed" { "delivery_failed" } else { "sla_breach" },
        "liability_type": "seller",
        "provider_key": "mock_payment",
        "provider_status": provider_status,
        "step_up_bound": true,
        resolution_record_field: resolution_record_id,
        provider_result_field: provider_result_id,
    });
    client
        .execute(
            "INSERT INTO billing.billing_event (
               billing_event_id, order_id, event_type, event_source, amount, currency_code, units, occurred_at, metadata
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3, $4, '20.00000000'::numeric, 'CNY', NULL, now(), $5::jsonb
             )",
            &[
                &billing_event_id,
                &order_id,
                &if scene == "refund.completed" { "refund" } else { "compensation" },
                &if scene == "refund.completed" {
                    "refund_execute"
                } else {
                    "compensation_execute"
                },
                &metadata,
            ],
        )
        .await
        .expect("insert billing event");

    BillingResolutionSeed {
        product_id,
        sku_id,
        order_id,
        billing_event_id,
        resolution_record_id,
        case_id,
        request_id: format!("req-{suffix}"),
        trace_id: format!("trace-{suffix}"),
        provider_result_id,
    }
}

async fn cleanup_billing_graph(client: &Client, graph: &BillingGraph) {
    cleanup_notification_rows(
        client,
        &[
            graph.refund.request_id.as_str(),
            graph.compensation.request_id.as_str(),
        ],
    )
    .await;
    cleanup_billing_resolution(client, &graph.refund).await;
    cleanup_billing_resolution(client, &graph.compensation).await;
    cleanup_billing_base_graph(client, graph).await;
}

async fn cleanup_billing_resolution(client: &Client, seed: &BillingResolutionSeed) {
    let _ = client
        .execute(
            "DELETE FROM billing.billing_event WHERE billing_event_id = $1::text::uuid",
            &[&seed.billing_event_id],
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
}

async fn seed_org(client: &Client, name: &str, org_type: &str) -> String {
    client
        .query_one(
            "INSERT INTO core.organization (org_name, org_type, status, metadata)
             VALUES ($1, $2, 'active', '{}'::jsonb)
             RETURNING org_id::text",
            &[&name, &org_type],
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
                &format!("NOTIF006 {}", persona.replace('_', " ")),
                &format!("{persona}.{suffix}@example.test"),
                &json!({ "persona": persona }),
            ],
        )
        .await
        .expect("insert user")
        .get(0)
}

async fn seed_asset(client: &Client, seller_org_id: &str, suffix: &str) -> String {
    client
        .query_one(
            "INSERT INTO catalog.data_asset (
               owner_org_id, title, category, sensitivity_level, status, description
             ) VALUES (
               $1::text::uuid, $2, 'finance', 'internal', 'active', $3
             )
             RETURNING asset_id::text",
            &[
                &seller_org_id,
                &format!("notif006-asset-{suffix}"),
                &format!("notif006 asset {suffix}"),
            ],
        )
        .await
        .expect("insert asset")
        .get(0)
}

async fn seed_asset_version(client: &Client, asset_id: &str) -> String {
    client
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
        .get(0)
}

async fn seed_product(
    client: &Client,
    asset_id: &str,
    asset_version_id: &str,
    seller_org_id: &str,
    suffix: &str,
    delivery_type: &str,
) -> String {
    client
        .query_one(
            r#"INSERT INTO catalog.product (
               asset_id, asset_version_id, seller_org_id, title, category, product_type,
               description, status, price_mode, price, currency_code, delivery_type,
               allowed_usage, searchable_text, metadata
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
               $5, 'listed', 'one_time', 128.00, 'CNY', $6,
               ARRAY['analytics']::text[], $7, '{"review_status":"approved"}'::jsonb
             )
             RETURNING product_id::text"#,
            &[
                &asset_id,
                &asset_version_id,
                &seller_org_id,
                &format!("notif006-product-{suffix}"),
                &format!("notif006 product {suffix}"),
                &delivery_type,
                &format!("notif006 search {suffix}"),
            ],
        )
        .await
        .expect("insert product")
        .get(0)
}

async fn seed_sku(
    client: &Client,
    product_id: &str,
    suffix: &str,
    sku_type: &str,
    billing_mode: &str,
    acceptance_mode: &str,
) -> String {
    client
        .query_one(
            "INSERT INTO catalog.product_sku (
               product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
             ) VALUES (
               $1::text::uuid, $2, $3, '份', $4, $5, 'manual_refund', 'active'
             )
             RETURNING sku_id::text",
            &[
                &product_id,
                &format!("NOTIF006-{sku_type}-{suffix}"),
                &sku_type,
                &billing_mode,
                &acceptance_mode,
            ],
        )
        .await
        .expect("insert sku")
        .get(0)
}

async fn cleanup_notification_rows(client: &Client, request_ids: &[&str]) {
    let request_ids = request_ids
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let _ = client
        .execute(
            "DELETE FROM ops.outbox_event WHERE request_id = ANY($1::text[])",
            &[&request_ids],
        )
        .await;
}

async fn cleanup_base_graph(client: &Client, graph: &AcceptanceGraph) {
    let user_ids = vec![
        graph.buyer_user_id.clone(),
        graph.seller_user_id.clone(),
        graph.platform_user_id.clone(),
    ];
    let _ = client
        .execute(
            "DELETE FROM core.user_account WHERE user_id::text = ANY($1::text[])",
            &[&user_ids],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
            &[&graph.asset_version_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
            &[&graph.asset_id],
        )
        .await;
    let org_ids = vec![
        graph.buyer_org_id.clone(),
        graph.seller_org_id.clone(),
        graph.platform_org_id.clone(),
    ];
    let _ = client
        .execute(
            "DELETE FROM core.organization WHERE org_id::text = ANY($1::text[])",
            &[&org_ids],
        )
        .await;
}

async fn cleanup_billing_base_graph(client: &Client, graph: &BillingGraph) {
    let user_ids = vec![
        graph.buyer_user_id.clone(),
        graph.seller_user_id.clone(),
        graph.platform_user_id.clone(),
    ];
    let _ = client
        .execute(
            "DELETE FROM core.user_account WHERE user_id::text = ANY($1::text[])",
            &[&user_ids],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
            &[&graph.asset_version_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
            &[&graph.asset_id],
        )
        .await;
    let org_ids = vec![
        graph.buyer_org_id.clone(),
        graph.seller_org_id.clone(),
        graph.platform_org_id.clone(),
    ];
    let _ = client
        .execute(
            "DELETE FROM core.organization WHERE org_id::text = ANY($1::text[])",
            &[&org_ids],
        )
        .await;
}

async fn new_uuid(client: &Client) -> String {
    client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate uuid")
        .get(0)
}
