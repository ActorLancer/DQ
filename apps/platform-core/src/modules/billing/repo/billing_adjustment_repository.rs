use crate::modules::billing::domain::BillingEvent;
use crate::modules::billing::repo::billing_event_repository::{
    RecordBillingEventRequest, record_billing_event_in_tx,
};
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::{ErrorCode, ErrorResponse};
use serde_json::json;

const PROVISIONAL_HOLD_CLASS: &str = "provisional_dispute_hold";
const PROVISIONAL_HOLD_EVENT_TYPE: &str = "refund_adjustment";
const PROVISIONAL_HOLD_EVENT_SOURCE: &str = "settlement_dispute_hold";
const PROVISIONAL_RELEASE_EVENT_SOURCE: &str = "settlement_dispute_release";

pub async fn ensure_provisional_dispute_hold_in_tx(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    reason_code: &str,
    origin_action: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<Option<BillingEvent>, (StatusCode, Json<ErrorResponse>)> {
    let outstanding = load_provisional_hold_balance(client, order_id)
        .await
        .map_err(billing_adjustment_db_error)?;
    if !has_positive_balance(&outstanding) {
        let amount = load_adjustment_amount(client, order_id)
            .await
            .map_err(billing_adjustment_db_error)?;
        let metadata = json!({
            "idempotency_key": format!("billing_adjustment:{order_id}:{PROVISIONAL_HOLD_CLASS}"),
            "reason_code": reason_code,
            "settlement_direction": "adjustment",
            "adjustment_class": PROVISIONAL_HOLD_CLASS,
            "adjustment_effect": "freeze_receivable",
            "origin_action": origin_action,
        });
        let (event, _) = record_billing_event_in_tx(
            client,
            &RecordBillingEventRequest {
                order_id: order_id.to_string(),
                event_type: PROVISIONAL_HOLD_EVENT_TYPE.to_string(),
                event_source: PROVISIONAL_HOLD_EVENT_SOURCE.to_string(),
                amount: Some(amount),
                currency_code: None,
                units: None,
                occurred_at: None,
                metadata,
            },
            None,
            actor_role,
            "billing.adjustment.provisional_hold",
            request_id,
            trace_id,
        )
        .await?;
        return Ok(Some(event));
    }
    Ok(None)
}

pub async fn release_provisional_dispute_hold_in_tx(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    resolution_action: &str,
    resolution_ref_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<Option<BillingEvent>, (StatusCode, Json<ErrorResponse>)> {
    thaw_settlement_records_for_resolution(client, order_id, resolution_action)
        .await
        .map_err(billing_adjustment_db_error)?;

    let outstanding = load_provisional_hold_balance(client, order_id)
        .await
        .map_err(billing_adjustment_db_error)?;
    if !has_positive_balance(&outstanding) {
        return Ok(None);
    }

    let metadata = json!({
        "idempotency_key": format!(
            "billing_adjustment:{order_id}:{PROVISIONAL_HOLD_CLASS}:release:{resolution_action}:{resolution_ref_id}"
        ),
        "reason_code": "settlement_freeze_released",
        "settlement_direction": "adjustment",
        "adjustment_class": PROVISIONAL_HOLD_CLASS,
        "adjustment_effect": "release_receivable_hold",
        "resolution_action": resolution_action,
        "resolution_ref_id": resolution_ref_id,
    });
    let (event, _) = record_billing_event_in_tx(
        client,
        &RecordBillingEventRequest {
            order_id: order_id.to_string(),
            event_type: PROVISIONAL_HOLD_EVENT_TYPE.to_string(),
            event_source: PROVISIONAL_RELEASE_EVENT_SOURCE.to_string(),
            amount: Some(negate_decimal_text(&outstanding)),
            currency_code: None,
            units: None,
            occurred_at: None,
            metadata,
        },
        None,
        actor_role,
        "billing.adjustment.provisional_release",
        request_id,
        trace_id,
    )
    .await?;
    Ok(Some(event))
}

async fn load_adjustment_amount(
    client: &(impl GenericClient + Sync),
    order_id: &str,
) -> Result<String, db::Error> {
    client
        .query_one(
            "SELECT COALESCE(
                (
                  SELECT payable_amount::text
                  FROM billing.settlement_record
                  WHERE order_id = $1::text::uuid
                  ORDER BY created_at DESC, settlement_id DESC
                  LIMIT 1
                ),
                (
                  SELECT amount::text
                  FROM trade.order_main
                  WHERE order_id = $1::text::uuid
                )
             )",
            &[&order_id],
        )
        .await
        .map(|row| row.get(0))
}

async fn load_provisional_hold_balance(
    client: &(impl GenericClient + Sync),
    order_id: &str,
) -> Result<String, db::Error> {
    client
        .query_one(
            "SELECT COALESCE(SUM(amount), 0)::text
             FROM billing.billing_event
             WHERE order_id = $1::text::uuid
               AND event_type = $2
               AND COALESCE(metadata ->> 'adjustment_class', '') = $3",
            &[
                &order_id,
                &PROVISIONAL_HOLD_EVENT_TYPE,
                &PROVISIONAL_HOLD_CLASS,
            ],
        )
        .await
        .map(|row| row.get(0))
}

async fn thaw_settlement_records_for_resolution(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    resolution_action: &str,
) -> Result<(), db::Error> {
    client
        .execute(
            "UPDATE billing.settlement_record
             SET settlement_status = 'pending',
                 reason_code = $2,
                 updated_at = now()
             WHERE order_id = $1::text::uuid
               AND settlement_status = 'frozen'",
            &[&order_id, &format!("resolved:{resolution_action}")],
        )
        .await?;
    client
        .execute(
            "UPDATE trade.order_main
             SET settlement_status = CASE
                   WHEN settlement_status IN ('frozen', 'blocked') THEN 'pending_settlement'
                   ELSE settlement_status
                 END,
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await?;
    Ok(())
}

fn has_positive_balance(value: &str) -> bool {
    value
        .parse::<f64>()
        .map(|parsed| parsed > 0.0)
        .unwrap_or(false)
}

fn negate_decimal_text(value: &str) -> String {
    if value.trim_start().starts_with('-') {
        value.trim_start_matches('-').to_string()
    } else {
        format!("-{value}")
    }
}

fn billing_adjustment_db_error(error: db::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: error.to_string(),
            request_id: None,
        }),
    )
}
