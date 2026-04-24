use super::notification_test_support::wait_for_mock_log_chain_if_enabled;
use crate::modules::integration::application::{
    DisputeLifecycleNotificationDispatchInput, SettlementResumeNotificationDispatchInput,
    queue_dispute_lifecycle_notifications, queue_settlement_resume_notifications,
};
use db::{Client, GenericClient, NoTls, connect};
use serde_json::{Value, json};

fn live_db_enabled() -> bool {
    std::env::var("NOTIF_DB_SMOKE").ok().as_deref() == Some("1")
}

#[tokio::test]
async fn notif007_dispute_settlement_notifications_db_smoke() {
    if !live_db_enabled() {
        eprintln!("skip notif007_dispute_settlement_notifications_db_smoke; set NOTIF_DB_SMOKE=1");
        return;
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let (client, connection) = connect(&database_url, NoTls)
        .await
        .expect("connect postgres");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    assert_active_template_version(&client, "NOTIFY_DISPUTE_ESCALATED_V1", 2).await;
    assert_active_template_version(&client, "NOTIFY_SETTLEMENT_FROZEN_V1", 2).await;
    assert_active_template_version(&client, "NOTIFY_SETTLEMENT_RESUMED_V1", 2).await;

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let seed = seed_graph(&client, &suffix).await;

    let dispute = queue_dispute_lifecycle_notifications(
        &client,
        DisputeLifecycleNotificationDispatchInput {
            order_id: &seed.order_id,
            case_id: &seed.case_id,
            dispute_occurred_at: Some("2026-04-22T02:00:00.000Z"),
            settlement_hold_event_id: Some(&seed.hold_event_id),
            settlement_hold_occurred_at: Some("2026-04-22T02:00:01.000Z"),
            request_id: Some(&seed.dispute_request_id),
            trace_id: Some(&seed.dispute_trace_id),
        },
    )
    .await
    .expect("queue dispute lifecycle notifications");
    assert_eq!(dispute.inserted_count, 6);
    assert_eq!(dispute.replayed_count, 0);

    let dispute_replay = queue_dispute_lifecycle_notifications(
        &client,
        DisputeLifecycleNotificationDispatchInput {
            order_id: &seed.order_id,
            case_id: &seed.case_id,
            dispute_occurred_at: Some("2026-04-22T02:00:00.000Z"),
            settlement_hold_event_id: Some(&seed.hold_event_id),
            settlement_hold_occurred_at: Some("2026-04-22T02:00:01.000Z"),
            request_id: Some(&seed.dispute_request_id),
            trace_id: Some(&seed.dispute_trace_id),
        },
    )
    .await
    .expect("replay dispute lifecycle notifications");
    assert_eq!(dispute_replay.inserted_count, 0);
    assert_eq!(dispute_replay.replayed_count, 6);

    let dispute_payloads = load_notification_payloads(&client, &seed.dispute_request_id).await;
    assert_eq!(dispute_payloads.len(), 6);
    let buyer_dispute = find_payload(&dispute_payloads, "dispute.escalated", "buyer");
    assert_eq!(
        buyer_dispute["payload"]["template_code"].as_str(),
        Some("NOTIFY_DISPUTE_ESCALATED_V1")
    );
    assert_eq!(
        buyer_dispute["payload"]["source_event"]["aggregate_type"].as_str(),
        Some("support.dispute_case")
    );
    assert_eq!(
        buyer_dispute["payload"]["variables"]["action_href"].as_str(),
        Some(format!("/support/cases/new?order_id={}", seed.order_id).as_str())
    );
    assert!(
        buyer_dispute["payload"]["variables"]
            .get("freeze_ticket_id")
            .is_none()
    );
    let seller_frozen = find_payload(&dispute_payloads, "settlement.frozen", "seller");
    assert_eq!(
        seller_frozen["payload"]["source_event"]["aggregate_id"].as_str(),
        Some(seed.hold_event_id.as_str())
    );
    assert_eq!(
        seller_frozen["payload"]["variables"]["action_href"].as_str(),
        Some(format!("/billing?order_id={}", seed.order_id).as_str())
    );
    let ops_frozen = find_payload(&dispute_payloads, "settlement.frozen", "ops");
    assert_eq!(
        ops_frozen["payload"]["variables"]["action_href"].as_str(),
        Some(
            format!(
                "/ops/risk?order_id={}&case_id={}",
                seed.order_id, seed.case_id
            )
            .as_str()
        )
    );
    assert_eq!(
        ops_frozen["payload"]["variables"]["show_ops_context"].as_bool(),
        Some(true)
    );
    assert_eq!(
        ops_frozen["payload"]["variables"]["hold_billing_event_id"].as_str(),
        Some(seed.hold_event_id.as_str())
    );

    client
        .execute(
            "UPDATE trade.order_main
             SET settlement_status = 'pending_settlement',
                 dispute_status = 'resolved',
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update order main resolved");
    client
        .execute(
            "UPDATE billing.settlement_record
             SET settlement_status = 'pending',
                 reason_code = 'resolved:manual_payout_execute',
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[&seed.order_id],
        )
        .await
        .expect("update settlement record pending");
    client
        .execute(
            "UPDATE support.dispute_case
             SET status = 'resolved',
                 decision_code = 'manual_adjustment',
                 penalty_code = 'seller_warning',
                 resolved_at = now(),
                 updated_at = now()
             WHERE case_id = $1::text::uuid",
            &[&seed.case_id],
        )
        .await
        .expect("resolve dispute case");

    let resumed = queue_settlement_resume_notifications(
        &client,
        SettlementResumeNotificationDispatchInput {
            order_id: &seed.order_id,
            billing_event_id: &seed.release_event_id,
            occurred_at: Some("2026-04-22T02:30:00.000Z"),
            request_id: Some(&seed.resume_request_id),
            trace_id: Some(&seed.resume_trace_id),
        },
    )
    .await
    .expect("queue settlement resumed notifications");
    assert_eq!(resumed.inserted_count, 3);
    assert_eq!(resumed.replayed_count, 0);

    let resumed_replay = queue_settlement_resume_notifications(
        &client,
        SettlementResumeNotificationDispatchInput {
            order_id: &seed.order_id,
            billing_event_id: &seed.release_event_id,
            occurred_at: Some("2026-04-22T02:30:00.000Z"),
            request_id: Some(&seed.resume_request_id),
            trace_id: Some(&seed.resume_trace_id),
        },
    )
    .await
    .expect("replay settlement resumed notifications");
    assert_eq!(resumed_replay.inserted_count, 0);
    assert_eq!(resumed_replay.replayed_count, 3);

    let resumed_payloads = load_notification_payloads(&client, &seed.resume_request_id).await;
    assert_eq!(resumed_payloads.len(), 3);
    let buyer_resumed = find_payload(&resumed_payloads, "settlement.resumed", "buyer");
    assert_eq!(
        buyer_resumed["payload"]["template_code"].as_str(),
        Some("NOTIFY_SETTLEMENT_RESUMED_V1")
    );
    assert_eq!(
        buyer_resumed["payload"]["source_event"]["aggregate_id"].as_str(),
        Some(seed.release_event_id.as_str())
    );
    assert_eq!(
        buyer_resumed["payload"]["variables"]["action_href"].as_str(),
        Some(
            format!(
                "/billing/refunds?order_id={}&case_id={}",
                seed.order_id, seed.case_id
            )
            .as_str()
        )
    );
    assert_eq!(
        buyer_resumed["payload"]["variables"]["billing_event_source"].as_str(),
        Some("settlement_dispute_release")
    );
    assert!(
        buyer_resumed["payload"]["variables"]
            .get("resolution_ref_id")
            .is_none()
    );
    let seller_resumed = find_payload(&resumed_payloads, "settlement.resumed", "seller");
    assert_eq!(
        seller_resumed["payload"]["variables"]["billing_event_source"].as_str(),
        Some("settlement_dispute_release")
    );
    let ops_resumed = find_payload(&resumed_payloads, "settlement.resumed", "ops");
    assert_eq!(
        ops_resumed["payload"]["variables"]["action_href"].as_str(),
        Some(
            format!(
                "/ops/audit/trace?order_id={}&case_id={}",
                seed.order_id, seed.case_id
            )
            .as_str()
        )
    );
    assert_eq!(
        ops_resumed["payload"]["variables"]["resolution_action"].as_str(),
        Some("manual_payout_execute")
    );
    assert_eq!(
        ops_resumed["payload"]["variables"]["resolution_ref_id"].as_str(),
        Some(seed.release_ref_id.as_str())
    );

    let dispute_live_chain = wait_for_mock_log_chain_if_enabled(
        &client,
        &seed.dispute_request_id,
        &[
            "dispute.escalated",
            "dispute.escalated",
            "dispute.escalated",
            "settlement.frozen",
            "settlement.frozen",
            "settlement.frozen",
        ],
    )
    .await;
    let resumed_live_chain = wait_for_mock_log_chain_if_enabled(
        &client,
        &seed.resume_request_id,
        &[
            "settlement.resumed",
            "settlement.resumed",
            "settlement.resumed",
        ],
    )
    .await;
    crate::write_test027_artifact(
        "notif007-dispute-settlement.json",
        &json!({
            "dispute": {
                "request_id": &seed.dispute_request_id,
                "trace_id": &seed.dispute_trace_id,
                "order_id": &seed.order_id,
                "case_id": &seed.case_id,
                "hold_event_id": &seed.hold_event_id,
                "notification_codes": dispute_payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["notification_code"].as_str())
                    .collect::<Vec<_>>(),
                "live_chain": dispute_live_chain,
            },
            "settlement_resumed": {
                "request_id": &seed.resume_request_id,
                "trace_id": &seed.resume_trace_id,
                "order_id": &seed.order_id,
                "case_id": &seed.case_id,
                "release_event_id": &seed.release_event_id,
                "release_ref_id": &seed.release_ref_id,
                "notification_codes": resumed_payloads
                    .iter()
                    .filter_map(|payload| payload["payload"]["notification_code"].as_str())
                    .collect::<Vec<_>>(),
                "live_chain": resumed_live_chain,
            },
        }),
    );

    cleanup_graph(&client, &seed).await;
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
        Some("NOTIF-007")
    );
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
    case_id: String,
    hold_event_id: String,
    release_event_id: String,
    release_ref_id: String,
    dispute_request_id: String,
    dispute_trace_id: String,
    resume_request_id: String,
    resume_trace_id: String,
}

async fn seed_graph(client: &Client, suffix: &str) -> SeedGraph {
    let buyer_org_id = seed_org(client, &format!("notif007-buyer-{suffix}"), "enterprise").await;
    let seller_org_id = seed_org(client, &format!("notif007-seller-{suffix}"), "enterprise").await;
    let platform_org_id =
        seed_org(client, &format!("notif007-platform-{suffix}"), "platform").await;
    let buyer_user_id = seed_user(
        client,
        &buyer_org_id,
        &format!("notif007-buyer-{suffix}"),
        "buyer_operator",
    )
    .await;
    let seller_user_id = seed_user(
        client,
        &seller_org_id,
        &format!("notif007-seller-{suffix}"),
        "seller_operator",
    )
    .await;
    let platform_user_id = seed_user(
        client,
        &platform_org_id,
        &format!("notif007-platform-{suffix}"),
        "platform_risk_settlement",
    )
    .await;
    let asset_id = seed_asset(client, &seller_org_id, &format!("notif007-{suffix}")).await;
    let asset_version_id = seed_asset_version(client, &asset_id).await;
    let product_id = seed_product(
        client,
        &asset_id,
        &asset_version_id,
        &seller_org_id,
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
               'accepted', 'paid', 'online', 128.00, 'CNY',
               'blocked', 'blocked', 'frozen', 'opened',
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
        .expect("insert order")
        .get(0);
    let case_id = new_uuid(client).await;
    client
        .execute(
            "INSERT INTO support.dispute_case (
               case_id, order_id, complainant_type, complainant_id, reason_code, status
             ) VALUES (
               $1::text::uuid, $2::text::uuid, 'organization', $3::text::uuid, 'delivery_failed', 'opened'
             )",
            &[&case_id, &order_id, &buyer_org_id],
        )
        .await
        .expect("insert dispute case");
    client
        .execute(
            "INSERT INTO billing.settlement_record (
               order_id, settlement_type, settlement_status, settlement_mode,
               payable_amount, platform_fee_amount, channel_fee_amount, net_receivable_amount,
               refund_amount, compensation_amount, reason_code
             ) VALUES (
               $1::text::uuid, 'order_settlement', 'frozen', 'manual',
               128.00000000, 8.00000000, 1.00000000, 0.00000000,
               128.00000000, 0.00000000, 'dispute_opened:delivery_failed'
             )",
            &[&order_id],
        )
        .await
        .expect("insert settlement record");

    let freeze_ticket_id = new_uuid(client).await;
    client
        .execute(
            "INSERT INTO risk.freeze_ticket (
               freeze_ticket_id, ref_type, ref_id, freeze_type, status, reason_code
             ) VALUES (
               $1::text::uuid, 'order', $2::text::uuid, 'dispute_hold', 'executed', 'delivery_failed'
             )",
            &[&freeze_ticket_id, &order_id],
        )
        .await
        .expect("insert freeze ticket");
    let legal_hold_id = new_uuid(client).await;
    client
        .execute(
            "INSERT INTO audit.legal_hold (
               legal_hold_id, hold_scope_type, hold_scope_id, status, reason_code, metadata
             ) VALUES (
               $1::text::uuid, 'order', $2::text::uuid, 'active', 'delivery_failed', $3::jsonb
             )",
            &[&legal_hold_id, &order_id, &json!({ "case_id": case_id })],
        )
        .await
        .expect("insert legal hold");
    for action_type in [
        "freeze_settlement",
        "suspend_delivery",
        "activate_legal_hold",
    ] {
        client
            .execute(
                "INSERT INTO risk.governance_action_log (
                   freeze_ticket_id, action_type, action_payload
                 ) VALUES (
                   $1::text::uuid, $2, '{}'::jsonb
                 )",
                &[&freeze_ticket_id, &action_type],
            )
            .await
            .expect("insert governance action");
    }

    let hold_event_id = new_uuid(client).await;
    client
        .execute(
            "INSERT INTO billing.billing_event (
               billing_event_id, order_id, event_type, event_source, amount, currency_code, units, occurred_at, metadata
             ) VALUES (
               $1::text::uuid, $2::text::uuid, 'refund_adjustment', 'settlement_dispute_hold',
               '128.00000000'::numeric, 'CNY', NULL, now(), $3::jsonb
             )",
            &[
                &hold_event_id,
                &order_id,
                &json!({
                    "idempotency_key": format!("notif007:hold:{suffix}"),
                    "reason_code": "delivery_failed",
                    "adjustment_class": "provisional_dispute_hold",
                    "adjustment_effect": "freeze_receivable",
                }),
            ],
        )
        .await
        .expect("insert hold billing event");

    let release_event_id = new_uuid(client).await;
    let release_ref_id = new_uuid(client).await;
    client
        .execute(
            "INSERT INTO support.decision_record (
               case_id, decision_type, decision_code, liability_type, decision_text, decided_by
             ) VALUES (
               $1::text::uuid, 'manual_resolution', 'manual_adjustment', 'shared', 'manual payout resolved', $2::text::uuid
             )",
            &[&case_id, &platform_user_id],
        )
        .await
        .expect("insert decision record");
    client
        .execute(
            "INSERT INTO billing.billing_event (
               billing_event_id, order_id, event_type, event_source, amount, currency_code, units, occurred_at, metadata
             ) VALUES (
               $1::text::uuid, $2::text::uuid, 'refund_adjustment', 'settlement_dispute_release',
               '-128.00000000'::numeric, 'CNY', NULL, now(), $3::jsonb
             )",
            &[
                &release_event_id,
                &order_id,
                &json!({
                    "idempotency_key": format!("notif007:release:{suffix}"),
                    "reason_code": "settlement_freeze_released",
                    "adjustment_class": "provisional_dispute_hold",
                    "adjustment_effect": "release_receivable_hold",
                    "resolution_action": "manual_payout_execute",
                    "resolution_ref_id": release_ref_id,
                }),
            ],
        )
        .await
        .expect("insert release billing event");

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
        case_id,
        hold_event_id,
        release_event_id,
        release_ref_id,
        dispute_request_id: format!("req-notif007-dispute-{suffix}"),
        dispute_trace_id: format!("trace-notif007-dispute-{suffix}"),
        resume_request_id: format!("req-notif007-resume-{suffix}"),
        resume_trace_id: format!("trace-notif007-resume-{suffix}"),
    }
}

async fn cleanup_graph(client: &Client, graph: &SeedGraph) {
    cleanup_notification_rows(
        client,
        &[
            graph.dispute_request_id.as_str(),
            graph.resume_request_id.as_str(),
        ],
    )
    .await;
    let _ = client
        .execute(
            "DELETE FROM billing.billing_event
             WHERE billing_event_id = $1::text::uuid
                OR billing_event_id = $2::text::uuid",
            &[&graph.hold_event_id, &graph.release_event_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM support.decision_record WHERE case_id = $1::text::uuid",
            &[&graph.case_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM support.dispute_case WHERE case_id = $1::text::uuid",
            &[&graph.case_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM audit.legal_hold WHERE hold_scope_type = 'order' AND hold_scope_id = $1::text::uuid",
            &[&graph.order_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM risk.governance_action_log
             WHERE freeze_ticket_id IN (
               SELECT freeze_ticket_id FROM risk.freeze_ticket WHERE ref_type = 'order' AND ref_id = $1::text::uuid
             )",
            &[&graph.order_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM risk.freeze_ticket WHERE ref_type = 'order' AND ref_id = $1::text::uuid",
            &[&graph.order_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
            &[&graph.order_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
            &[&graph.order_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
            &[&graph.sku_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
            &[&graph.product_id],
        )
        .await;
    cleanup_base_graph(client, graph).await;
}

async fn cleanup_base_graph(client: &Client, graph: &SeedGraph) {
    let _ = client
        .execute(
            "DELETE FROM core.user_account
             WHERE user_id = $1::text::uuid
                OR user_id = $2::text::uuid
                OR user_id = $3::text::uuid",
            &[
                &graph.buyer_user_id,
                &graph.seller_user_id,
                &graph.platform_user_id,
            ],
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
    let _ = client
        .execute(
            "DELETE FROM core.organization
             WHERE org_id = $1::text::uuid
                OR org_id = $2::text::uuid
                OR org_id = $3::text::uuid",
            &[
                &graph.buyer_org_id,
                &graph.seller_org_id,
                &graph.platform_org_id,
            ],
        )
        .await;
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
                &format!("NOTIF007 {}", persona.replace('_', " ")),
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
                &format!("notif007-asset-{suffix}"),
                &format!("notif007 asset {suffix}"),
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
                &format!("notif007-product-{suffix}"),
                &format!("notif007 product {suffix}"),
                &delivery_type,
                &format!("notif007 search {suffix}"),
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
                &format!("NOTIF007-{sku_type}-{suffix}"),
                &sku_type,
                &billing_mode,
                &acceptance_mode,
            ],
        )
        .await
        .expect("insert sku")
        .get(0)
}

async fn new_uuid(client: &Client) -> String {
    client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate uuid")
        .get(0)
}
