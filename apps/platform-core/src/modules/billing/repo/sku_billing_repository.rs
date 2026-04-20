use crate::modules::billing::domain::{SkuBillingBasisView, sku_billing_basis_rule_for_sku};
use crate::modules::billing::repo::billing_event_repository::{
    RecordBillingEventRequest, record_billing_event_in_tx,
};
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

pub async fn load_sku_billing_basis_view(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    _request_id: Option<&str>,
) -> Result<Option<SkuBillingBasisView>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               COALESCE(s.sku_type, COALESCE(o.price_snapshot_json ->> 'sku_type', o.price_snapshot_json ->> 'selected_sku_type', 'unknown')),
               COALESCE(o.price_snapshot_json, '{}'::jsonb)
             FROM trade.order_main o
             LEFT JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Ok(None);
    };
    let sku_type: String = row.get(0);
    let price_snapshot_json: Value = row.get(1);
    let Some(rule) = sku_billing_basis_rule_for_sku(&sku_type) else {
        return Ok(None);
    };

    Ok(Some(SkuBillingBasisView {
        sku_type,
        default_event_type: rule.default_event_type.map(str::to_string),
        cycle_event_type: rule.cycle_event_type.map(str::to_string),
        usage_event_type: rule.usage_event_type.map(str::to_string),
        payment_trigger: rule.payment_trigger.to_string(),
        delivery_trigger: rule.delivery_trigger.to_string(),
        acceptance_trigger: rule.acceptance_trigger.to_string(),
        billing_trigger: rule.billing_trigger.to_string(),
        settlement_cycle: rule.settlement_cycle.to_string(),
        periodic_settlement_cycle: rule.periodic_settlement_cycle.map(str::to_string),
        refund_entry: rule.refund_entry.to_string(),
        refund_placeholder_entry: rule.refund_placeholder_entry.map(str::to_string),
        refund_placeholder_event_type: rule.refund_placeholder_event_type.map(str::to_string),
        refund_mode: json_value_to_string(price_snapshot_json.get("refund_mode")),
        refund_template_code: json_value_to_string(
            price_snapshot_json
                .get("refund_template")
                .or_else(|| price_snapshot_json.get("refund_template_code")),
        ),
        compensation_entry: rule.compensation_entry.to_string(),
        dispute_freeze_trigger: rule.dispute_freeze_trigger.to_string(),
        resume_settlement_trigger: rule.resume_settlement_trigger.to_string(),
        policy_stage: if rule.sku_type == "SHARE_RO" {
            "v1_share_ro_opening_cycle_placeholder".to_string()
        } else {
            "v1_default_placeholder".to_string()
        },
    }))
}

pub async fn record_share_ro_enable_charge_in_tx(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(crate::modules::billing::domain::BillingEvent, bool), (StatusCode, Json<ErrorResponse>)>
{
    let Some(basis) = load_sku_billing_basis_view(client, order_id, request_id).await? else {
        return Err(billing_bad_request(
            "SHARE_RO enable billing requires SHARE_RO order billing basis",
            request_id,
            StatusCode::CONFLICT,
        ));
    };
    if basis.sku_type != "SHARE_RO"
        || basis.default_event_type.as_deref() != Some("one_time_charge")
    {
        return Err(billing_bad_request(
            "SHARE_RO enable billing requires SHARE_RO one_time placeholder basis",
            request_id,
            StatusCode::CONFLICT,
        ));
    }

    let metadata = json!({
        "idempotency_key": format!("billing_event:{order_id}:share_enable:placeholder"),
        "reason_code": "share_grant_enable_placeholder",
        "trigger_action": "enable_share",
        "sku_billing_basis": sku_billing_basis_snapshot(&basis),
    });

    record_billing_event_in_tx(
        client,
        &RecordBillingEventRequest {
            order_id: order_id.to_string(),
            event_type: "one_time_charge".to_string(),
            event_source: "share_enable".to_string(),
            amount: None,
            currency_code: None,
            units: Some("1".to_string()),
            occurred_at: None,
            metadata,
        },
        None,
        actor_role,
        "billing.event.record.share_ro_enable",
        request_id,
        trace_id,
    )
    .await
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

fn json_value_to_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(string)) if !string.trim().is_empty() => Some(string.trim().to_string()),
        Some(Value::Number(number)) => Some(number.to_string()),
        Some(Value::Bool(boolean)) => Some(boolean.to_string()),
        _ => None,
    }
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
            message: format!("database error: {error}"),
            request_id: None,
        }),
    )
}
