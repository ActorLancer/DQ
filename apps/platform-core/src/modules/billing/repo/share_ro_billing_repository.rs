use crate::modules::billing::domain::SkuBillingBasisView;
use crate::modules::billing::models::{CreateShareRoCycleChargeRequest, ShareRoCycleChargeView};
use crate::modules::billing::repo::billing_event_repository::{
    RecordBillingEventRequest, record_billing_event_in_tx,
};
use crate::modules::billing::repo::sku_billing_repository::load_sku_billing_basis_view;
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

struct ShareRoOrderContext {
    current_state: String,
    payment_status: String,
    settlement_status: String,
    dispute_status: String,
}

pub async fn record_share_ro_cycle_charge_in_tx(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    payload: &CreateShareRoCycleChargeRequest,
) -> Result<ShareRoCycleChargeView, (StatusCode, Json<ErrorResponse>)> {
    let cycle_code = payload.billing_cycle_code.trim();
    if cycle_code.is_empty() {
        return Err(billing_bad_request(
            "SHARE_RO cycle charge requires billing_cycle_code",
            request_id,
            StatusCode::BAD_REQUEST,
        ));
    }

    let Some(basis) = load_sku_billing_basis_view(client, order_id, request_id).await? else {
        return Err(billing_bad_request(
            "SHARE_RO cycle charge requires SHARE_RO order billing basis",
            request_id,
            StatusCode::CONFLICT,
        ));
    };
    if basis.sku_type != "SHARE_RO" || basis.cycle_event_type.as_deref() != Some("recurring_charge")
    {
        return Err(billing_bad_request(
            "SHARE_RO cycle charge requires recurring cycle billing basis",
            request_id,
            StatusCode::CONFLICT,
        ));
    }

    let context = load_share_ro_order_context(client, order_id, request_id).await?;
    if !matches!(
        context.current_state.as_str(),
        "share_enabled" | "share_granted" | "shared_active"
    ) {
        return Err(billing_bad_request(
            &format!(
                "SHARE_RO cycle charge is not allowed from current_state `{}`",
                context.current_state
            ),
            request_id,
            StatusCode::CONFLICT,
        ));
    }
    if context.payment_status != "paid" {
        return Err(billing_bad_request(
            &format!(
                "SHARE_RO cycle charge is not allowed from payment_status `{}`",
                context.payment_status
            ),
            request_id,
            StatusCode::CONFLICT,
        ));
    }

    let mut metadata = json!({
        "idempotency_key": format!("billing_event:{order_id}:share_ro_cycle:{cycle_code}"),
        "reason_code": "share_cycle_charge",
        "billing_cycle_code": cycle_code,
        "sku_billing_basis": sku_billing_basis_snapshot(&basis),
    });
    if let Some(note) = payload
        .reason_note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        metadata["reason_note"] = Value::String(note.to_string());
    }

    let (event, replayed) = record_billing_event_in_tx(
        client,
        &RecordBillingEventRequest {
            order_id: order_id.to_string(),
            event_type: "recurring_charge".to_string(),
            event_source: "share_cycle".to_string(),
            amount: payload.billing_amount.clone(),
            currency_code: None,
            units: Some("1".to_string()),
            occurred_at: None,
            metadata,
        },
        None,
        actor_role,
        "billing.event.record.share_ro_cycle",
        request_id,
        trace_id,
    )
    .await?;

    let context = load_share_ro_order_context(client, order_id, request_id).await?;
    Ok(ShareRoCycleChargeView {
        order_id: order_id.to_string(),
        billing_cycle_code: cycle_code.to_string(),
        billing_event_id: event.billing_event_id,
        billing_event_type: event.event_type,
        billing_event_replayed: replayed,
        current_state: context.current_state,
        payment_status: context.payment_status,
        settlement_status: context.settlement_status,
        dispute_status: context.dispute_status,
    })
}

pub async fn record_share_ro_revoke_refund_placeholder_in_tx(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    reason_code: &str,
) -> Result<
    Option<(crate::modules::billing::domain::BillingEvent, bool)>,
    (StatusCode, Json<ErrorResponse>),
> {
    let Some(basis) = load_sku_billing_basis_view(client, order_id, request_id).await? else {
        return Ok(None);
    };
    if basis.sku_type != "SHARE_RO"
        || basis.refund_placeholder_event_type.as_deref() != Some("refund_adjustment")
    {
        return Ok(None);
    }

    let refundable_amount = load_share_ro_refundable_amount(client, order_id, request_id).await?;
    if refundable_amount == "0.00000000" {
        return Ok(None);
    }

    let metadata = json!({
        "idempotency_key": format!("billing_event:{order_id}:share_ro_revoke_refund_placeholder"),
        "reason_code": reason_code,
        "adjustment_class": "share_ro_revoke_refund_placeholder",
        "placeholder": true,
        "sku_billing_basis": sku_billing_basis_snapshot(&basis),
    });

    let result = record_billing_event_in_tx(
        client,
        &RecordBillingEventRequest {
            order_id: order_id.to_string(),
            event_type: "refund_adjustment".to_string(),
            event_source: "share_revoke_refund_placeholder".to_string(),
            amount: Some(refundable_amount),
            currency_code: None,
            units: Some("1".to_string()),
            occurred_at: None,
            metadata,
        },
        None,
        actor_role,
        "billing.event.record.share_ro_revoke_refund_placeholder",
        request_id,
        trace_id,
    )
    .await?;
    Ok(Some(result))
}

async fn load_share_ro_order_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<ShareRoOrderContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT status, payment_status, settlement_status, dispute_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(billing_bad_request(
            &format!("order not found: {order_id}"),
            request_id,
            StatusCode::NOT_FOUND,
        ));
    };
    Ok(ShareRoOrderContext {
        current_state: row.get(0),
        payment_status: row.get(1),
        settlement_status: row.get(2),
        dispute_status: row.get(3),
    })
}

async fn load_share_ro_refundable_amount(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_one(
            "SELECT to_char(
                GREATEST(
                  COALESCE(
                    SUM(
                      CASE
                        WHEN event_type IN ('one_time_charge', 'recurring_charge') THEN amount
                        WHEN event_type IN ('refund', 'refund_adjustment') THEN -amount
                        ELSE 0
                      END
                    ),
                    0
                  ),
                  0
                ),
                'FM999999999999990.00000000'
             )
             FROM billing.billing_event
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let amount: Option<String> = row.get(0);
    Ok(amount.unwrap_or_else(|| "0.00000000".to_string()))
}

fn sku_billing_basis_snapshot(basis: &SkuBillingBasisView) -> Value {
    json!({
        "sku_type": basis.sku_type,
        "default_event_type": basis.default_event_type,
        "cycle_event_type": basis.cycle_event_type,
        "usage_event_type": basis.usage_event_type,
        "payment_trigger": basis.payment_trigger,
        "delivery_trigger": basis.delivery_trigger,
        "acceptance_trigger": basis.acceptance_trigger,
        "billing_trigger": basis.billing_trigger,
        "settlement_cycle": basis.settlement_cycle,
        "periodic_settlement_cycle": basis.periodic_settlement_cycle,
        "refund_entry": basis.refund_entry,
        "refund_placeholder_entry": basis.refund_placeholder_entry,
        "refund_placeholder_event_type": basis.refund_placeholder_event_type,
        "refund_mode": basis.refund_mode,
        "refund_template_code": basis.refund_template_code,
        "compensation_entry": basis.compensation_entry,
        "dispute_freeze_trigger": basis.dispute_freeze_trigger,
        "resume_settlement_trigger": basis.resume_settlement_trigger,
        "policy_stage": basis.policy_stage,
    })
}

fn billing_bad_request(
    message: &str,
    request_id: Option<&str>,
    status: StatusCode,
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

fn map_db_error(error: db::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: error.to_string(),
            request_id: None,
        }),
    )
}
