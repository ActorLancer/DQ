use crate::modules::integration::application::{
    DeliveryCompletionNotificationDispatchInput, queue_delivery_completion_notifications,
};
use db::{Client, GenericClient, NoTls, connect};
use serde_json::{Value, json};

fn live_db_enabled() -> bool {
    std::env::var("NOTIF_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Clone, Copy)]
struct BranchCase {
    delivery_branch: &'static str,
    sku_type: &'static str,
    billing_mode: &'static str,
    acceptance_mode: &'static str,
    order_status: &'static str,
    delivery_status: &'static str,
    acceptance_status: &'static str,
    delivery_type: &'static str,
    delivery_route: &'static str,
    result_ref_type: &'static str,
    source_event_aggregate_type: &'static str,
    source_event_event_type: &'static str,
    expected_buyer_code: &'static str,
    expected_buyer_template: &'static str,
}

const BRANCH_CASES: &[BranchCase] = &[
    BranchCase {
        delivery_branch: "file",
        sku_type: "FILE_STD",
        billing_mode: "one_time",
        acceptance_mode: "manual_accept",
        order_status: "delivered",
        delivery_status: "delivered",
        acceptance_status: "pending_acceptance",
        delivery_type: "file_download",
        delivery_route: "signed_url",
        result_ref_type: "delivery_record",
        source_event_aggregate_type: "delivery.delivery_record",
        source_event_event_type: "delivery.committed",
        expected_buyer_code: "order.pending_acceptance",
        expected_buyer_template: "NOTIFY_PENDING_ACCEPTANCE_V1",
    },
    BranchCase {
        delivery_branch: "share",
        sku_type: "SHARE_RO",
        billing_mode: "subscription",
        acceptance_mode: "auto_accept",
        order_status: "share_granted",
        delivery_status: "delivered",
        acceptance_status: "accepted",
        delivery_type: "share_grant",
        delivery_route: "share_link",
        result_ref_type: "delivery_record",
        source_event_aggregate_type: "delivery.delivery_record",
        source_event_event_type: "delivery.committed",
        expected_buyer_code: "delivery.completed",
        expected_buyer_template: "NOTIFY_DELIVERY_COMPLETED_V1",
    },
    BranchCase {
        delivery_branch: "api",
        sku_type: "API_SUB",
        billing_mode: "subscription",
        acceptance_mode: "auto_accept",
        order_status: "api_key_issued",
        delivery_status: "in_progress",
        acceptance_status: "not_started",
        delivery_type: "api_access",
        delivery_route: "api_key",
        result_ref_type: "delivery_record",
        source_event_aggregate_type: "delivery.delivery_record",
        source_event_event_type: "delivery.committed",
        expected_buyer_code: "delivery.completed",
        expected_buyer_template: "NOTIFY_DELIVERY_COMPLETED_V1",
    },
    BranchCase {
        delivery_branch: "query_run",
        sku_type: "QRY_LITE",
        billing_mode: "usage",
        acceptance_mode: "manual_accept",
        order_status: "result_available",
        delivery_status: "delivered",
        acceptance_status: "accepted",
        delivery_type: "query_result",
        delivery_route: "template_query",
        result_ref_type: "query_execution_run",
        source_event_aggregate_type: "delivery.query_execution_run",
        source_event_event_type: "delivery.template_query.use",
        expected_buyer_code: "delivery.completed",
        expected_buyer_template: "NOTIFY_DELIVERY_COMPLETED_V1",
    },
    BranchCase {
        delivery_branch: "sandbox",
        sku_type: "SBX_STD",
        billing_mode: "subscription",
        acceptance_mode: "manual_accept",
        order_status: "seat_issued",
        delivery_status: "delivered",
        acceptance_status: "accepted",
        delivery_type: "sandbox_workspace",
        delivery_route: "workspace_console",
        result_ref_type: "delivery_record",
        source_event_aggregate_type: "delivery.delivery_record",
        source_event_event_type: "delivery.committed",
        expected_buyer_code: "delivery.completed",
        expected_buyer_template: "NOTIFY_DELIVERY_COMPLETED_V1",
    },
    BranchCase {
        delivery_branch: "report",
        sku_type: "RPT_STD",
        billing_mode: "one_time",
        acceptance_mode: "manual_accept",
        order_status: "report_delivered",
        delivery_status: "delivered",
        acceptance_status: "in_progress",
        delivery_type: "report_delivery",
        delivery_route: "result_package",
        result_ref_type: "delivery_record",
        source_event_aggregate_type: "delivery.delivery_record",
        source_event_event_type: "delivery.committed",
        expected_buyer_code: "order.pending_acceptance",
        expected_buyer_template: "NOTIFY_PENDING_ACCEPTANCE_V1",
    },
];

#[tokio::test]
async fn notif005_delivery_completion_notifications_db_smoke() {
    if !live_db_enabled() {
        eprintln!("skip notif005_delivery_completion_notifications_db_smoke; set NOTIF_DB_SMOKE=1");
        return;
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let (client, connection) = connect(&database_url, NoTls)
        .await
        .expect("connect postgres");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    assert_active_template_version(&client, "NOTIFY_DELIVERY_COMPLETED_V1", 2).await;
    assert_active_template_version(&client, "NOTIFY_PENDING_ACCEPTANCE_V1", 2).await;

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let seed = seed_graph(&client, &suffix).await;

    for seeded_order in &seed.orders {
        let first = queue_delivery_completion_notifications(&client, dispatch_input(seeded_order))
            .await
            .expect("first delivery completion notification insert");
        assert_eq!(
            first.inserted_count, 3,
            "{} should emit buyer/seller/ops notifications",
            seeded_order.case.delivery_branch
        );
        assert_eq!(first.replayed_count, 0);

        let replay = queue_delivery_completion_notifications(&client, dispatch_input(seeded_order))
            .await
            .expect("replayed delivery completion notification insert");
        assert_eq!(replay.inserted_count, 0);
        assert_eq!(replay.replayed_count, 3);
        assert_eq!(replay.idempotency_keys, first.idempotency_keys);

        let rows = client
            .query(
                "SELECT
                   payload,
                   idempotency_key,
                   target_topic
                 FROM ops.outbox_event
                 WHERE request_id = $1
                 ORDER BY created_at ASC, outbox_event_id ASC",
                &[&seeded_order.request_id],
            )
            .await
            .expect("load delivery completion notification outbox rows");
        assert_eq!(rows.len(), 3);

        let payloads = rows
            .iter()
            .map(|row| row.get::<_, Value>(0))
            .collect::<Vec<_>>();
        let persisted_keys = rows
            .iter()
            .map(|row| row.get::<_, String>(1))
            .collect::<Vec<_>>();
        assert_eq!(persisted_keys, first.idempotency_keys);
        for row in &rows {
            assert_eq!(row.get::<_, String>(2), "dtp.notification.dispatch");
        }

        let buyer = find_payload(&payloads, "buyer");
        assert_eq!(
            buyer["payload"]["notification_code"].as_str(),
            Some(seeded_order.case.expected_buyer_code)
        );
        assert_eq!(
            buyer["payload"]["template_code"].as_str(),
            Some(seeded_order.case.expected_buyer_template)
        );
        assert_eq!(
            buyer["payload"]["source_event"]["aggregate_type"].as_str(),
            Some(seeded_order.case.source_event_aggregate_type)
        );
        assert_eq!(
            buyer["payload"]["source_event"]["event_type"].as_str(),
            Some(seeded_order.case.source_event_event_type)
        );
        assert_eq!(
            buyer["payload"]["source_event"]["aggregate_id"].as_str(),
            Some(seeded_order.result_ref_id.as_str())
        );
        assert_eq!(
            buyer["payload"]["source_event"]["target_topic"].as_str(),
            Some("dtp.outbox.domain-events")
        );
        assert_eq!(
            buyer["payload"]["variables"]["show_ops_context"].as_bool(),
            Some(false)
        );
        assert!(
            buyer["payload"]["metadata"]
                .get("delivery_ref_id")
                .is_none(),
            "buyer metadata must not expose delivery linkage"
        );
        assert!(
            buyer["payload"]["variables"]
                .get("delivery_ref_id")
                .is_none(),
            "buyer variables must not expose delivery linkage"
        );

        let seller = find_payload(&payloads, "seller");
        assert_eq!(
            seller["payload"]["notification_code"].as_str(),
            Some("delivery.completed")
        );
        assert_eq!(
            seller["payload"]["template_code"].as_str(),
            Some("NOTIFY_DELIVERY_COMPLETED_V1")
        );
        assert_eq!(
            seller["payload"]["variables"]["show_ops_context"].as_bool(),
            Some(false)
        );
        assert!(
            seller["payload"]["metadata"]
                .get("delivery_ref_id")
                .is_none(),
            "seller metadata must not expose delivery linkage"
        );

        let ops = find_payload(&payloads, "ops");
        assert_eq!(
            ops["payload"]["notification_code"].as_str(),
            Some("delivery.completed")
        );
        assert_eq!(
            ops["payload"]["template_code"].as_str(),
            Some("NOTIFY_DELIVERY_COMPLETED_V1")
        );
        assert_eq!(
            ops["payload"]["variables"]["show_ops_context"].as_bool(),
            Some(true)
        );
        assert_eq!(
            ops["payload"]["variables"]["delivery_ref_type"].as_str(),
            Some(seeded_order.case.result_ref_type)
        );
        assert_eq!(
            ops["payload"]["variables"]["delivery_ref_id"].as_str(),
            Some(seeded_order.result_ref_id.as_str())
        );
        assert_eq!(
            ops["payload"]["metadata"]["delivery_ref_type"].as_str(),
            Some(seeded_order.case.result_ref_type)
        );
        assert_eq!(
            ops["payload"]["metadata"]["delivery_ref_id"].as_str(),
            Some(seeded_order.result_ref_id.as_str())
        );
        assert_eq!(
            ops["payload"]["metadata"]["receipt_hash"].as_str(),
            Some(seeded_order.receipt_hash.as_str())
        );
        assert_eq!(
            ops["payload"]["metadata"]["delivery_commit_hash"].as_str(),
            Some(seeded_order.delivery_commit_hash.as_str())
        );
    }

    cleanup_seed_graph(&client, &seed).await;
}

fn dispatch_input<'a>(
    seeded_order: &'a SeedOrder,
) -> DeliveryCompletionNotificationDispatchInput<'a> {
    DeliveryCompletionNotificationDispatchInput {
        order_id: &seeded_order.order_id,
        delivery_branch: seeded_order.case.delivery_branch,
        result_ref_type: seeded_order.case.result_ref_type,
        result_ref_id: &seeded_order.result_ref_id,
        source_event_aggregate_type: seeded_order.case.source_event_aggregate_type,
        source_event_event_type: seeded_order.case.source_event_event_type,
        source_event_occurred_at: Some("2026-04-22T00:00:00.000Z"),
        delivery_type: Some(seeded_order.case.delivery_type),
        delivery_route: Some(seeded_order.case.delivery_route),
        receipt_hash: Some(&seeded_order.receipt_hash),
        delivery_commit_hash: Some(&seeded_order.delivery_commit_hash),
        request_id: Some(&seeded_order.request_id),
        trace_id: Some(&seeded_order.trace_id),
    }
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
        Some("NOTIF-005")
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
    orders: Vec<SeedOrder>,
}

struct SeedOrder {
    case: BranchCase,
    product_id: String,
    sku_id: String,
    order_id: String,
    result_ref_id: String,
    request_id: String,
    trace_id: String,
    receipt_hash: String,
    delivery_commit_hash: String,
}

async fn seed_graph(client: &Client, suffix: &str) -> SeedGraph {
    let buyer_org_id = seed_org(client, &format!("notif005-buyer-{suffix}"), "enterprise").await;
    let seller_org_id = seed_org(client, &format!("notif005-seller-{suffix}"), "enterprise").await;
    let platform_org_id =
        seed_org(client, &format!("notif005-platform-{suffix}"), "platform").await;
    let buyer_user_id = seed_user(
        client,
        &buyer_org_id,
        &format!("notif005-buyer-user-{suffix}"),
        "buyer_operator",
    )
    .await;
    let seller_user_id = seed_user(
        client,
        &seller_org_id,
        &format!("notif005-seller-user-{suffix}"),
        "seller_operator",
    )
    .await;
    let platform_user_id = seed_user(
        client,
        &platform_org_id,
        &format!("notif005-platform-user-{suffix}"),
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
                &format!("notif005-asset-{suffix}"),
                &format!("notif005 asset {suffix}"),
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

    let mut orders = Vec::with_capacity(BRANCH_CASES.len());
    for case in BRANCH_CASES {
        let product_id: String = client
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
                    &format!("notif005-{}-product-{suffix}", case.delivery_branch),
                    &format!("notif005 {} product {suffix}", case.delivery_branch),
                    &case.delivery_type,
                    &format!("notif005 {} search {suffix}", case.delivery_branch),
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
                   $1::text::uuid, $2, $3, '份', $4, $5, 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[
                    &product_id,
                    &format!("NOTIF005-{}-{suffix}", case.sku_type),
                    &case.sku_type,
                    &case.billing_mode,
                    &case.acceptance_mode,
                ],
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
                   price_snapshot_json,
                   trust_boundary_snapshot,
                   delivery_route_snapshot,
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
                   $6,
                   'paid',
                   'online',
                   128.00,
                   'CNY',
                   jsonb_build_object('sku_type', $7, 'delivery_mode', $8),
                   jsonb_build_object('delivery_mode', $8, 'task_id', 'NOTIF-005'),
                   $9,
                   $10,
                   $11,
                   $12,
                   'pending_settlement',
                   'none',
                   $13
                 )
                 RETURNING order_id::text"#,
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &case.order_status,
                    &case.sku_type,
                    &case.delivery_type,
                    &case.delivery_route,
                    &format!("notif005-order-{}-{suffix}", case.delivery_branch),
                    &case.delivery_status,
                    &case.acceptance_status,
                    &format!("notif005_{}_delivery_complete", case.delivery_branch),
                ],
            )
            .await
            .expect("insert order")
            .get(0);
        let result_ref_id = new_uuid(client).await;
        orders.push(SeedOrder {
            case: *case,
            product_id,
            sku_id,
            order_id,
            result_ref_id,
            request_id: format!("req-notif005-{}-{suffix}", case.delivery_branch),
            trace_id: format!("trace-notif005-{}-{suffix}", case.delivery_branch),
            receipt_hash: format!("receipt-hash-{}-{suffix}", case.delivery_branch),
            delivery_commit_hash: format!("commit-hash-{}-{suffix}", case.delivery_branch),
        });
    }

    SeedGraph {
        buyer_org_id,
        seller_org_id,
        platform_org_id,
        buyer_user_id,
        seller_user_id,
        platform_user_id,
        asset_id,
        asset_version_id,
        orders,
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
                &format!("NOTIF005 {}", persona.replace('_', " ")),
                &format!("{persona}.{suffix}@example.test"),
                &json!({ "persona": persona }),
            ],
        )
        .await
        .expect("insert user")
        .get(0)
}

async fn new_uuid(client: &Client) -> String {
    client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate uuid")
        .get(0)
}

async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
    let request_ids = seed
        .orders
        .iter()
        .map(|order| order.request_id.clone())
        .collect::<Vec<_>>();
    let order_ids = seed
        .orders
        .iter()
        .map(|order| order.order_id.clone())
        .collect::<Vec<_>>();
    let product_ids = seed
        .orders
        .iter()
        .map(|order| order.product_id.clone())
        .collect::<Vec<_>>();
    let sku_ids = seed
        .orders
        .iter()
        .map(|order| order.sku_id.clone())
        .collect::<Vec<_>>();

    let _ = client
        .execute(
            "DELETE FROM ops.outbox_event
             WHERE request_id = ANY($1::text[])",
            &[&request_ids],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM trade.order_main
             WHERE order_id = ANY($1::text[]::uuid[])",
            &[&order_ids],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.product_sku
             WHERE sku_id = ANY($1::text[]::uuid[])",
            &[&sku_ids],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.product
             WHERE product_id = ANY($1::text[]::uuid[])",
            &[&product_ids],
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
