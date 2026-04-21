use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::domain::Settlement;
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
use axum::Json;
use axum::http::StatusCode;
use db::{GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::json;

#[derive(Debug, Clone)]
struct SettlementAggregateSnapshot {
    settlement_id: Option<String>,
    settlement_type: String,
    settlement_status: String,
    settlement_mode: String,
    payable_amount: String,
    platform_fee_amount: String,
    channel_fee_amount: String,
    net_receivable_amount: String,
    refund_amount: String,
    compensation_amount: String,
    reason_code: Option<String>,
    settled_at: Option<String>,
}

#[derive(Debug, Clone)]
struct SettlementOutboxContext {
    buyer_org_id: String,
    seller_org_id: String,
    order_status: String,
    payment_status: String,
    settlement_status: String,
    dispute_status: String,
    currency_code: String,
}

pub async fn recompute_settlement_for_order(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<Settlement, (StatusCode, Json<ErrorResponse>)> {
    let snapshot = load_settlement_aggregate_snapshot(client, order_id, request_id).await?;
    let row = if let Some(settlement_id) = snapshot.settlement_id.as_deref() {
        client
            .query_one(
                "UPDATE billing.settlement_record
                 SET settlement_type = $2,
                     settlement_status = $3,
                     settlement_mode = $4,
                     payable_amount = $5::text::numeric,
                     platform_fee_amount = $6::text::numeric,
                     channel_fee_amount = $7::text::numeric,
                     net_receivable_amount = $8::text::numeric,
                     refund_amount = $9::text::numeric,
                     compensation_amount = $10::text::numeric,
                     reason_code = $11,
                     settled_at = CASE
                       WHEN $12::timestamptz IS NULL THEN settled_at
                       ELSE $12::timestamptz
                     END,
                     updated_at = now()
                 WHERE settlement_id = $1::text::uuid
                 RETURNING
                   settlement_id::text,
                   settlement_type,
                   settlement_status,
                   settlement_mode,
                   payable_amount::text,
                   platform_fee_amount::text,
                   channel_fee_amount::text,
                   net_receivable_amount::text,
                   refund_amount::text,
                   compensation_amount::text,
                   reason_code,
                   CASE WHEN settled_at IS NULL THEN NULL ELSE to_char(settled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') END,
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &settlement_id,
                    &snapshot.settlement_type,
                    &snapshot.settlement_status,
                    &snapshot.settlement_mode,
                    &snapshot.payable_amount,
                    &snapshot.platform_fee_amount,
                    &snapshot.channel_fee_amount,
                    &snapshot.net_receivable_amount,
                    &snapshot.refund_amount,
                    &snapshot.compensation_amount,
                    &snapshot.reason_code,
                    &snapshot.settled_at,
                ],
            )
            .await
            .map_err(map_db_error)?
    } else {
        client
            .query_one(
                "INSERT INTO billing.settlement_record (
                   order_id,
                   settlement_type,
                   settlement_status,
                   settlement_mode,
                   payable_amount,
                   platform_fee_amount,
                   channel_fee_amount,
                   net_receivable_amount,
                   refund_amount,
                   compensation_amount,
                   reason_code,
                   settled_at
                 ) VALUES (
                   $1::text::uuid,
                   $2,
                   $3,
                   $4,
                   $5::text::numeric,
                   $6::text::numeric,
                   $7::text::numeric,
                   $8::text::numeric,
                   $9::text::numeric,
                   $10::text::numeric,
                   $11,
                   $12::timestamptz
                 )
                 RETURNING
                   settlement_id::text,
                   settlement_type,
                   settlement_status,
                   settlement_mode,
                   payable_amount::text,
                   platform_fee_amount::text,
                   channel_fee_amount::text,
                   net_receivable_amount::text,
                   refund_amount::text,
                   compensation_amount::text,
                   reason_code,
                   CASE WHEN settled_at IS NULL THEN NULL ELSE to_char(settled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') END,
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &order_id,
                    &snapshot.settlement_type,
                    &snapshot.settlement_status,
                    &snapshot.settlement_mode,
                    &snapshot.payable_amount,
                    &snapshot.platform_fee_amount,
                    &snapshot.channel_fee_amount,
                    &snapshot.net_receivable_amount,
                    &snapshot.refund_amount,
                    &snapshot.compensation_amount,
                    &snapshot.reason_code,
                    &snapshot.settled_at,
                ],
            )
            .await
            .map_err(map_db_error)?
    };
    let settlement = parse_settlement_row(&row);
    sync_order_settlement_state(client, order_id, &settlement)
        .await
        .map_err(map_db_error)?;
    let outbox_written =
        write_settlement_summary_outbox(client, order_id, &settlement, request_id, trace_id)
            .await?;
    write_audit_event(
        client,
        "billing",
        "settlement",
        &settlement.settlement_id,
        actor_role,
        "billing.settlement.recomputed",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    if outbox_written {
        write_audit_event(
            client,
            "billing",
            "settlement",
            &settlement.settlement_id,
            actor_role,
            "billing.settlement.summary.outbox",
            "success",
            request_id,
            trace_id,
        )
        .await?;
    }
    Ok(settlement)
}

async fn load_settlement_aggregate_snapshot(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<SettlementAggregateSnapshot, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            r#"WITH order_ctx AS (
                 SELECT
                   order_id,
                   amount AS order_amount,
                   settlement_status AS order_settlement_status,
                   COALESCE(NULLIF(fee_preview_snapshot ->> 'platform_fee_amount', '')::numeric, 0) AS fee_platform_amount,
                   COALESCE(NULLIF(fee_preview_snapshot ->> 'channel_fee_amount', '')::numeric, 0) AS fee_channel_amount
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid
               ),
               existing AS (
                 SELECT
                   settlement_id,
                   settlement_type,
                   settlement_status,
                   settlement_mode,
                   payable_amount,
                   platform_fee_amount,
                   channel_fee_amount,
                   refund_amount,
                   compensation_amount,
                   reason_code,
                   settled_at
                 FROM billing.settlement_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC, settlement_id DESC
                 LIMIT 1
               ),
               event_sums AS (
                 SELECT
                   COALESCE(SUM(CASE WHEN event_type IN ('one_time_charge', 'recurring_charge', 'usage_charge') THEN amount ELSE 0 END), 0) AS charge_amount,
                   COALESCE(SUM(CASE WHEN event_type IN ('refund', 'refund_adjustment') THEN amount ELSE 0 END), 0) AS refund_amount,
                   COALESCE(SUM(CASE WHEN event_type IN ('compensation', 'compensation_adjustment') THEN amount ELSE 0 END), 0) AS compensation_amount,
                   COALESCE(BOOL_OR(event_type IN ('refund', 'refund_adjustment')), false) AS has_refund_events,
                   COALESCE(BOOL_OR(event_type IN ('compensation', 'compensation_adjustment')), false) AS has_compensation_events,
                   COALESCE(BOOL_OR(event_type = 'manual_settlement'), false) AS has_manual_settlement,
                   MAX(occurred_at) FILTER (WHERE event_type = 'manual_settlement') AS manual_settlement_at
                 FROM billing.billing_event
                 WHERE order_id = $1::text::uuid
               ),
               resolved AS (
                 SELECT
                   existing.settlement_id,
                   COALESCE(existing.settlement_type, 'order_settlement') AS settlement_type,
                   COALESCE(existing.settlement_mode, 'manual') AS settlement_mode,
                   COALESCE(NULLIF(event_sums.charge_amount, 0), NULLIF(existing.payable_amount, 0), order_ctx.order_amount, 0) AS payable_amount,
                   COALESCE(NULLIF(order_ctx.fee_platform_amount, 0), existing.platform_fee_amount, 0) AS platform_fee_amount,
                   COALESCE(NULLIF(order_ctx.fee_channel_amount, 0), existing.channel_fee_amount, 0) AS channel_fee_amount,
                   CASE
                     WHEN event_sums.has_refund_events THEN event_sums.refund_amount
                     ELSE COALESCE(existing.refund_amount, 0)
                   END AS refund_amount,
                   CASE
                     WHEN event_sums.has_compensation_events THEN event_sums.compensation_amount
                     ELSE COALESCE(existing.compensation_amount, 0)
                   END AS compensation_amount,
                   existing.reason_code,
                   existing.settled_at,
                   event_sums.has_manual_settlement,
                   event_sums.manual_settlement_at,
                   COALESCE(existing.settlement_status, order_ctx.order_settlement_status, 'pending') AS previous_status
                 FROM order_ctx
                 LEFT JOIN existing ON true
                 LEFT JOIN event_sums ON true
               )
               SELECT
                 CASE WHEN settlement_id IS NULL THEN NULL ELSE settlement_id::text END,
                 settlement_type,
                 CASE
                   WHEN previous_status IN ('frozen', 'blocked', 'closed', 'canceled')
                     THEN CASE WHEN previous_status = 'blocked' THEN 'frozen' ELSE previous_status END
                   WHEN has_manual_settlement OR previous_status = 'settled' OR settled_at IS NOT NULL THEN 'settled'
                   WHEN payable_amount > 0 AND refund_amount >= payable_amount THEN 'refunded'
                   ELSE 'pending'
                 END AS settlement_status,
                 settlement_mode,
                 payable_amount::text,
                 platform_fee_amount::text,
                 channel_fee_amount::text,
                 GREATEST(payable_amount - platform_fee_amount - channel_fee_amount - refund_amount - compensation_amount, 0)::text,
                 refund_amount::text,
                 compensation_amount::text,
                 reason_code,
                 CASE
                   WHEN (
                     CASE
                       WHEN previous_status IN ('frozen', 'blocked', 'closed', 'canceled')
                         THEN CASE WHEN previous_status = 'blocked' THEN 'frozen' ELSE previous_status END
                       WHEN has_manual_settlement OR previous_status = 'settled' OR settled_at IS NOT NULL THEN 'settled'
                       WHEN payable_amount > 0 AND refund_amount >= payable_amount THEN 'refunded'
                       ELSE 'pending'
                     END
                   ) = 'settled'
                   THEN to_char(COALESCE(settled_at, manual_settlement_at, now()) AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
                   ELSE CASE WHEN settled_at IS NULL THEN NULL ELSE to_char(settled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END
                 END AS settled_at
               FROM resolved"#,
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(settlement_error(
            StatusCode::NOT_FOUND,
            &format!("order not found: {order_id}"),
            request_id,
        ));
    };
    Ok(SettlementAggregateSnapshot {
        settlement_id: row.get(0),
        settlement_type: row.get(1),
        settlement_status: row.get(2),
        settlement_mode: row.get(3),
        payable_amount: row.get(4),
        platform_fee_amount: row.get(5),
        channel_fee_amount: row.get(6),
        net_receivable_amount: row.get(7),
        refund_amount: row.get(8),
        compensation_amount: row.get(9),
        reason_code: row.get(10),
        settled_at: row.get(11),
    })
}

async fn sync_order_settlement_state(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    settlement: &Settlement,
) -> Result<(), db::Error> {
    let order_settlement_status = match settlement.settlement_status.as_str() {
        "pending" => "pending_settlement",
        "settled" => "settled",
        "refunded" => "refunded",
        "frozen" => "frozen",
        "closed" => "closed",
        "canceled" => "canceled",
        other => other,
    };
    client
        .execute(
            "UPDATE trade.order_main
             SET settlement_status = $2,
                 settled_at = CASE
                   WHEN $2 = 'settled' THEN COALESCE(settled_at, $3::timestamptz)
                   ELSE settled_at
                 END,
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[&order_id, &order_settlement_status, &settlement.settled_at],
        )
        .await?;
    Ok(())
}

async fn write_settlement_summary_outbox(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    settlement: &Settlement,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<bool, (StatusCode, Json<ErrorResponse>)> {
    let context = load_settlement_outbox_context(client, order_id, request_id).await?;
    let event_type = settlement_outbox_event_type(settlement);
    let summary_state = format!(
        "{}:{}:{}",
        settlement.settlement_type, settlement.settlement_status, settlement.settlement_mode
    );
    let proof_commit_state = "pending_anchor";
    let payload = json!({
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "pending_fabric_anchor",
        "proof_commit_state": proof_commit_state,
        "order_id": order_id,
        "settlement_id": settlement.settlement_id,
        "settlement_type": settlement.settlement_type,
        "settlement_status": settlement.settlement_status,
        "settlement_mode": settlement.settlement_mode,
        "settled_at": settlement.settled_at,
        "reason_code": settlement.reason_code,
        "summary": {
            "gross_amount": settlement.payable_amount,
            "platform_commission_amount": settlement.platform_fee_amount,
            "channel_fee_amount": settlement.channel_fee_amount,
            "refund_adjustment_amount": settlement.refund_amount,
            "compensation_adjustment_amount": settlement.compensation_amount,
            "supplier_receivable_amount": settlement.net_receivable_amount,
            "summary_state": summary_state,
            "proof_commit_state": proof_commit_state,
        },
        "order_snapshot": {
            "buyer_org_id": context.buyer_org_id,
            "seller_org_id": context.seller_org_id,
            "order_status": context.order_status,
            "payment_status": context.payment_status,
            "settlement_status": context.settlement_status,
            "dispute_status": context.dispute_status,
            "currency_code": context.currency_code,
        }
    });
    let idempotency_key = format!(
        "settlement-summary:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        settlement.settlement_id,
        event_type,
        settlement.payable_amount,
        settlement.platform_fee_amount,
        settlement.channel_fee_amount,
        settlement.net_receivable_amount,
        settlement.refund_amount,
        settlement.compensation_amount,
        settlement.reason_code.as_deref().unwrap_or(""),
        settlement.settled_at.as_deref().unwrap_or(""),
        context.order_status,
        context.payment_status,
        context.settlement_status,
        context.dispute_status,
    );
    let inserted = write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: "billing.settlement_record",
            aggregate_id: &settlement.settlement_id,
            event_type,
            producer_service: "platform-core.billing",
            request_id,
            trace_id,
            idempotency_key: Some(idempotency_key.as_str()),
            occurred_at: settlement.settled_at.as_deref(),
            business_payload: &payload,
            deduplicate_by_idempotency_key: true,
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(inserted)
}

async fn load_settlement_outbox_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<SettlementOutboxContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               buyer_org_id::text,
               seller_org_id::text,
               status,
               payment_status,
               settlement_status,
               dispute_status,
               currency_code
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(settlement_error(
            StatusCode::NOT_FOUND,
            &format!("order not found for settlement summary outbox: {order_id}"),
            request_id,
        ));
    };
    Ok(SettlementOutboxContext {
        buyer_org_id: row.get(0),
        seller_org_id: row.get(1),
        order_status: row.get(2),
        payment_status: row.get(3),
        settlement_status: row.get(4),
        dispute_status: row.get(5),
        currency_code: row.get(6),
    })
}

fn settlement_outbox_event_type(settlement: &Settlement) -> &'static str {
    match settlement.settlement_status.as_str() {
        "settled" | "refunded" | "closed" | "canceled" => "settlement.completed",
        _ => "settlement.created",
    }
}

fn parse_settlement_row(row: &Row) -> Settlement {
    Settlement {
        settlement_id: row.get(0),
        settlement_type: row.get(1),
        settlement_status: row.get(2),
        settlement_mode: row.get(3),
        payable_amount: row.get(4),
        platform_fee_amount: row.get(5),
        channel_fee_amount: row.get(6),
        net_receivable_amount: row.get(7),
        refund_amount: row.get(8),
        compensation_amount: row.get(9),
        reason_code: row.get(10),
        settled_at: row.get(11),
        updated_at: row.get(12),
    }
}

fn settlement_error(
    status: StatusCode,
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
