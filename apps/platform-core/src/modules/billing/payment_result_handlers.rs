use crate::AppState;
use crate::modules::billing::handlers::{header, map_db_connect, require_permission};
use crate::modules::billing::models::{PaymentPolledResultRequest, PaymentPolledResultView};
use crate::modules::billing::payment_result_processor::{
    PaymentResultSourceKind, ProcessPaymentResultRequest, process_payment_result,
};
use crate::modules::billing::repo::payment_intent_repository::get_payment_intent_detail;
use crate::modules::billing::service::BillingPermission;
use crate::modules::billing::webhook::map_provider_result_status;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use db::GenericClient;
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};

pub async fn process_payment_polled_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PaymentPolledResultRequest>,
) -> Result<Json<ApiResponse<PaymentPolledResultView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PaymentIntentProcessResult,
        "payment polled result process",
    )?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let tenant_scope_id = tenant_scope_id_for_role(&headers, &role)?;
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let mut client = state.db.client().map_err(map_db_connect)?;

    let detail = get_payment_intent_detail(
        &client,
        &id,
        tenant_scope_id.as_deref(),
        request_id.as_deref(),
    )
    .await?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("payment intent not found: {id}"),
                request_id: request_id.clone(),
            }),
        )
    })?;
    let target_status = map_provider_result_status(&payload.provider_status).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!(
                    "provider_status is not supported for polled result: {}",
                    payload.provider_status
                ),
                request_id: request_id.clone(),
            }),
        )
    })?;
    let occurred_at_ms = resolve_occurred_at_ms(
        &client,
        payload.occurred_at.as_deref(),
        payload.occurred_at_ms,
        request_id.as_deref(),
    )
    .await?;
    let outcome = process_payment_result(
        &mut client,
        ProcessPaymentResultRequest {
            source_kind: PaymentResultSourceKind::Polling,
            provider_key: detail.payment_intent.provider_key.clone(),
            reference_id: payload.provider_result_id.clone(),
            payment_intent_id: id.clone(),
            provider_transaction_no: payload.provider_transaction_no.clone(),
            transaction_amount: payload.transaction_amount.clone(),
            currency_code: payload.currency_code.clone(),
            target_status: target_status.to_string(),
            occurred_at: payload.occurred_at.clone(),
            occurred_at_ms,
            raw_payload: payload.raw_payload,
        },
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(PaymentPolledResultView {
        payment_intent_id: id,
        provider_key: detail.payment_intent.provider_key,
        provider_result_id: payload.provider_result_id,
        processed_status: outcome.processed_status,
        duplicate: outcome.duplicate,
        out_of_order_ignored: outcome.out_of_order_ignored,
        payment_transaction_id: outcome.payment_transaction_id,
        applied_payment_status: outcome.applied_payment_status,
    }))
}

async fn resolve_occurred_at_ms(
    client: &impl GenericClient,
    occurred_at: Option<&str>,
    occurred_at_ms: Option<i64>,
    request_id: Option<&str>,
) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
    if let Some(value) = occurred_at_ms {
        return Ok(if value < 10_000_000_000 {
            value * 1000
        } else {
            value
        });
    }
    if let Some(occurred_at) = occurred_at.map(str::trim).filter(|value| !value.is_empty()) {
        let row = client
            .query_one(
                "SELECT floor(extract(epoch from $1::timestamptz) * 1000)::bigint",
                &[&occurred_at],
            )
            .await
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        code: ErrorCode::BilProviderFailed.as_str().to_string(),
                        message: "occurred_at must be a valid RFC3339 timestamp".to_string(),
                        request_id: request_id.map(str::to_string),
                    }),
                )
            })?;
        return Ok(row.get(0));
    }
    Ok(crate::modules::billing::webhook::now_utc_ms())
}

fn tenant_scope_id_for_role(
    headers: &HeaderMap,
    role: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    if !matches!(role, "tenant_admin" | "tenant_operator") {
        return Ok(None);
    }
    header(headers, "x-tenant-id").map(Some).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "x-tenant-id is required for tenant-scoped payment intent actions"
                    .to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })
}
