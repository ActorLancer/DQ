use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::domain::Settlement;
use axum::Json;
use axum::http::StatusCode;
use db::{GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};

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
                   COALESCE(SUM(CASE WHEN event_type = 'refund' THEN amount ELSE 0 END), 0) AS refund_amount,
                   COALESCE(SUM(CASE WHEN event_type = 'compensation' THEN amount ELSE 0 END), 0) AS compensation_amount,
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
                     WHEN event_sums.refund_amount > 0 THEN event_sums.refund_amount
                     ELSE COALESCE(existing.refund_amount, 0)
                   END AS refund_amount,
                   CASE
                     WHEN event_sums.compensation_amount > 0 THEN event_sums.compensation_amount
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
                   WHEN previous_status IN ('frozen', 'closed', 'canceled') THEN previous_status
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
                       WHEN previous_status IN ('frozen', 'closed', 'canceled') THEN previous_status
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
