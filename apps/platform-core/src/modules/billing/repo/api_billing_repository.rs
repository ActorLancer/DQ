use crate::modules::billing::domain::{ApiBillingBasisView, api_billing_basis_rule_for_sku};
use crate::modules::billing::repo::billing_event_repository::{
    RecordBillingEventRequest, record_billing_event_in_tx,
};
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

pub struct ApiSubCycleChargeParams {
    pub billing_cycle_code: String,
    pub billing_amount: Option<String>,
    pub reason_note: Option<String>,
}

pub struct ApiPpuUsageChargeParams {
    pub billing_amount: String,
    pub usage_units: String,
    pub meter_window_code: Option<String>,
    pub reason_note: Option<String>,
}

struct ApiBillingContext {
    sku_type: String,
    quota_json: Value,
    latest_usage_call_count: i64,
    latest_usage_units: String,
}

pub async fn load_api_billing_basis_view(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<Option<ApiBillingBasisView>, (StatusCode, Json<ErrorResponse>)> {
    let Some(context) = load_api_billing_context(client, order_id, request_id).await? else {
        return Ok(None);
    };
    let Some(rule) = api_billing_basis_rule_for_sku(&context.sku_type) else {
        return Ok(None);
    };

    Ok(Some(ApiBillingBasisView {
        sku_type: context.sku_type,
        base_event_type: rule.base_event_type.map(str::to_string),
        usage_event_type: rule.usage_event_type.map(str::to_string),
        cycle_period: json_value_to_string(context.quota_json.get("period"))
            .or_else(|| rule.default_cycle_period.map(str::to_string)),
        included_units: json_value_to_string(context.quota_json.get("included_calls"))
            .or_else(|| json_value_to_string(context.quota_json.get("included_units"))),
        overage_policy: json_value_to_string(context.quota_json.get("overage_policy"))
            .or_else(|| rule.default_overage_policy.map(str::to_string)),
        usage_meter_source: rule.usage_meter_source.map(str::to_string),
        success_only: rule.success_only,
        latest_usage_call_count: context.latest_usage_call_count.to_string(),
        latest_usage_units: context.latest_usage_units,
    }))
}

pub async fn record_api_sub_cycle_charge_in_tx(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    params: &ApiSubCycleChargeParams,
) -> Result<(crate::modules::billing::domain::BillingEvent, bool), (StatusCode, Json<ErrorResponse>)>
{
    let cycle_code = params.billing_cycle_code.trim();
    if cycle_code.is_empty() {
        return Err(billing_bad_request(
            "API_SUB bill_cycle requires billing_cycle_code",
            request_id,
            StatusCode::BAD_REQUEST,
        ));
    }
    let Some(basis) = load_api_billing_basis_view(client, order_id, request_id).await? else {
        return Err(billing_bad_request(
            "API_SUB bill_cycle requires API_SUB order billing basis",
            request_id,
            StatusCode::CONFLICT,
        ));
    };
    if basis.sku_type != "API_SUB" || basis.base_event_type.as_deref() != Some("recurring_charge") {
        return Err(billing_bad_request(
            "API_SUB bill_cycle requires recurring billing basis",
            request_id,
            StatusCode::CONFLICT,
        ));
    }

    let mut metadata = json!({
        "idempotency_key": format!(
            "billing_event:{order_id}:recurring_charge:billing_cycle:{cycle_code}"
        ),
        "reason_code": "subscription_cycle",
        "billing_cycle_code": cycle_code,
        "api_billing_basis": api_billing_basis_snapshot(&basis),
    });
    if let Some(note) = params
        .reason_note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        metadata["reason_note"] = Value::String(note.to_string());
    }

    record_billing_event_in_tx(
        client,
        &RecordBillingEventRequest {
            order_id: order_id.to_string(),
            event_type: "recurring_charge".to_string(),
            event_source: "billing_cycle".to_string(),
            amount: params.billing_amount.clone(),
            currency_code: None,
            units: Some("1".to_string()),
            occurred_at: None,
            metadata,
        },
        None,
        actor_role,
        "billing.event.record.api_sub_cycle",
        request_id,
        trace_id,
    )
    .await
}

pub async fn record_api_ppu_usage_charge_in_tx(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    params: &ApiPpuUsageChargeParams,
) -> Result<(crate::modules::billing::domain::BillingEvent, bool), (StatusCode, Json<ErrorResponse>)>
{
    let usage_units = params.usage_units.trim();
    if usage_units.is_empty() {
        return Err(billing_bad_request(
            "API_PPU settle_success_call requires usage_units",
            request_id,
            StatusCode::BAD_REQUEST,
        ));
    }
    let billing_amount = params.billing_amount.trim();
    if billing_amount.is_empty() {
        return Err(billing_bad_request(
            "API_PPU settle_success_call requires billing_amount",
            request_id,
            StatusCode::BAD_REQUEST,
        ));
    }
    let Some(basis) = load_api_billing_basis_view(client, order_id, request_id).await? else {
        return Err(billing_bad_request(
            "API_PPU usage charge requires API_PPU order billing basis",
            request_id,
            StatusCode::CONFLICT,
        ));
    };
    if basis.sku_type != "API_PPU" || basis.usage_event_type.as_deref() != Some("usage_charge") {
        return Err(billing_bad_request(
            "API_PPU settle_success_call requires usage billing basis",
            request_id,
            StatusCode::CONFLICT,
        ));
    }
    let idempotency_token = request_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            billing_bad_request(
                "API_PPU settle_success_call requires request_id for idempotent usage billing",
                request_id,
                StatusCode::BAD_REQUEST,
            )
        })?;

    let mut metadata = json!({
        "idempotency_key": format!(
            "billing_event:{order_id}:usage_charge:usage_meter:{idempotency_token}"
        ),
        "reason_code": "api_usage_meter",
        "meter_window_code": params
            .meter_window_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(idempotency_token),
        "successful_call_count": 1,
        "api_billing_basis": api_billing_basis_snapshot(&basis),
    });
    if let Some(note) = params
        .reason_note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        metadata["reason_note"] = Value::String(note.to_string());
    }

    record_billing_event_in_tx(
        client,
        &RecordBillingEventRequest {
            order_id: order_id.to_string(),
            event_type: "usage_charge".to_string(),
            event_source: "usage_meter".to_string(),
            amount: Some(billing_amount.to_string()),
            currency_code: None,
            units: Some(usage_units.to_string()),
            occurred_at: None,
            metadata,
        },
        None,
        actor_role,
        "billing.event.record.api_ppu_usage",
        request_id,
        trace_id,
    )
    .await
}

fn api_billing_basis_snapshot(basis: &ApiBillingBasisView) -> Value {
    json!({
        "sku_type": basis.sku_type,
        "base_event_type": basis.base_event_type,
        "usage_event_type": basis.usage_event_type,
        "cycle_period": basis.cycle_period,
        "included_units": basis.included_units,
        "overage_policy": basis.overage_policy,
        "usage_meter_source": basis.usage_meter_source,
        "success_only": basis.success_only,
        "latest_usage_call_count": basis.latest_usage_call_count,
        "latest_usage_units": basis.latest_usage_units,
    })
}

async fn load_api_billing_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<Option<ApiBillingContext>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               COALESCE(s.sku_type, COALESCE(o.price_snapshot_json ->> 'sku_type', o.price_snapshot_json ->> 'selected_sku_type', 'unknown')),
               COALESCE(ac.quota_json, '{}'::jsonb),
               COALESCE(usage_summary.success_call_count, 0)::bigint,
               COALESCE(usage_summary.success_usage_units, '0')
             FROM trade.order_main o
             LEFT JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             LEFT JOIN LATERAL (
               SELECT quota_json
               FROM delivery.api_credential
               WHERE order_id = $1::text::uuid
               ORDER BY created_at DESC, api_credential_id DESC
               LIMIT 1
             ) ac ON true
             LEFT JOIN LATERAL (
               SELECT
                 COUNT(*) FILTER (
                   WHERE COALESCE(response_code, 0) >= 200
                     AND COALESCE(response_code, 0) < 400
                 ) AS success_call_count,
                 COALESCE(
                   SUM(usage_units) FILTER (
                     WHERE COALESCE(response_code, 0) >= 200
                       AND COALESCE(response_code, 0) < 400
                   ),
                   0
                 )::text AS success_usage_units
               FROM delivery.api_usage_log
               WHERE order_id = $1::text::uuid
             ) usage_summary ON true
             WHERE o.order_id = $1::text::uuid",
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

    Ok(Some(ApiBillingContext {
        sku_type: row.get(0),
        quota_json: row.get(1),
        latest_usage_call_count: row.get(2),
        latest_usage_units: row.get(3),
    }))
}

fn json_value_to_string(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(string) = value.as_str() {
        return Some(string.to_string());
    }
    if value.is_number() || value.is_boolean() {
        return Some(value.to_string());
    }
    None
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
