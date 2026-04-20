use crate::AppState;
use crate::modules::billing::db::write_audit_event;
use crate::modules::billing::handlers::{header, map_db_connect, require_permission};
use crate::modules::billing::models::{
    MockPaymentSimulationRequest, MockPaymentSimulationView, PaymentWebhookRequest,
};
use crate::modules::billing::repo::mock_payment_repository::{
    create_mock_payment_case, load_mock_payment_intent_context, update_mock_payment_case_result,
};
use crate::modules::billing::service::BillingPermission;
use crate::modules::billing::webhook_handlers::handle_payment_webhook;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use http::ApiResponse;
use kernel::ErrorResponse;
use provider_kit::{MockPaymentScenario, ProviderBackend, build_payment_provider};
use serde_json::json;
use tokio::time::{Duration, sleep};
use tracing::info;

pub async fn simulate_payment_success(
    state: State<AppState>,
    headers: HeaderMap,
    id: Path<String>,
    payload: Json<MockPaymentSimulationRequest>,
) -> Result<Json<ApiResponse<MockPaymentSimulationView>>, (StatusCode, Json<ErrorResponse>)> {
    simulate_payment(state, headers, id, payload, MockPaymentScenario::Success).await
}

pub async fn simulate_payment_fail(
    state: State<AppState>,
    headers: HeaderMap,
    id: Path<String>,
    payload: Json<MockPaymentSimulationRequest>,
) -> Result<Json<ApiResponse<MockPaymentSimulationView>>, (StatusCode, Json<ErrorResponse>)> {
    simulate_payment(state, headers, id, payload, MockPaymentScenario::Fail).await
}

pub async fn simulate_payment_timeout(
    state: State<AppState>,
    headers: HeaderMap,
    id: Path<String>,
    payload: Json<MockPaymentSimulationRequest>,
) -> Result<Json<ApiResponse<MockPaymentSimulationView>>, (StatusCode, Json<ErrorResponse>)> {
    simulate_payment(state, headers, id, payload, MockPaymentScenario::Timeout).await
}

async fn simulate_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(payment_intent_id): Path<String>,
    Json(payload): Json<MockPaymentSimulationRequest>,
    scenario: MockPaymentScenario,
) -> Result<Json<ApiResponse<MockPaymentSimulationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::MockPaymentSimulate,
        "mock payment simulate",
    )?;
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let tenant_scope_id = header(&headers, "x-tenant-id");
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let client = state.db.client().map_err(map_db_connect)?;

    let intent = load_mock_payment_intent_context(
        &client,
        &payment_intent_id,
        tenant_scope_id.as_deref(),
        request_id.as_deref(),
    )
    .await?;
    let duplicate_webhook = payload.duplicate_webhook.unwrap_or(false);
    let delay_seconds = payload.delay_seconds.unwrap_or(0).max(0);
    let partial_refund_amount = payload.partial_refund_amount.clone();
    let scenario_type = match scenario {
        MockPaymentScenario::Success => "success",
        MockPaymentScenario::Fail => "fail",
        MockPaymentScenario::Timeout => "timeout",
    };
    let case_seed_payload = json!({
        "requested_by_role": actor_role,
        "requested_tenant_scope_id": tenant_scope_id,
        "scenario_type": scenario_type,
        "delay_seconds": delay_seconds,
        "duplicate_webhook": duplicate_webhook,
        "payment_status_before": intent.payment_status,
        "partial_refund_amount": partial_refund_amount,
    });
    let mock_payment_case_id = create_mock_payment_case(
        &client,
        &payment_intent_id,
        &intent.provider_key,
        scenario_type,
        delay_seconds,
        duplicate_webhook,
        partial_refund_amount.as_deref(),
        &case_seed_payload,
    )
    .await?;

    let provider = build_payment_provider(ProviderBackend::Mock);
    let provider_event = provider
        .simulate_webhook(&payment_intent_id, scenario)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    code: kernel::ErrorCode::BilProviderFailed.as_str().to_string(),
                    message: format!("mock payment provider simulate failed: {err}"),
                    request_id: request_id.clone(),
                }),
            )
        })?;

    if delay_seconds > 0 {
        sleep(Duration::from_secs(delay_seconds as u64)).await;
    }

    let mut webhook_headers = HeaderMap::new();
    webhook_headers.insert(
        "x-provider-signature",
        HeaderValue::from_static("mock-signature"),
    );
    webhook_headers.insert(
        "x-webhook-timestamp",
        HeaderValue::from_str(&format!(
            "{}",
            crate::modules::billing::webhook::now_utc_ms()
        ))
        .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    if let Some(request_id) = request_id.as_deref() {
        if let Ok(value) = HeaderValue::from_str(request_id) {
            webhook_headers.insert("x-request-id", value);
        }
    }
    if let Some(trace_id) = trace_id.as_deref() {
        if let Ok(value) = HeaderValue::from_str(trace_id) {
            webhook_headers.insert("x-trace-id", value);
        }
    }

    let webhook_request = PaymentWebhookRequest {
        provider_event_id: provider_event.provider_event_id.clone(),
        event_type: provider_event.event_type.clone(),
        provider_transaction_no: Some(format!("mocktxn-{}", provider_event.provider_event_id)),
        payment_intent_id: Some(payment_intent_id.clone()),
        transaction_amount: Some("88.00".to_string()),
        currency_code: Some("SGD".to_string()),
        provider_status: Some(provider_event.provider_status.clone()),
        occurred_at: None,
        occurred_at_ms: Some(crate::modules::billing::webhook::now_utc_ms()),
        raw_payload: json!({
            "source": "mock_payment_simulate",
            "scenario_type": scenario_type,
            "http_status_code": provider_event.http_status_code,
            "order_id": intent.order_id,
            "payee_subject_id": intent.payee_subject_id,
            "payer_subject_id": intent.payer_subject_id,
        }),
    };
    let webhook_result = handle_payment_webhook(
        State(state.clone()),
        webhook_headers.clone(),
        Path("mock_payment".to_string()),
        Json(webhook_request),
    )
    .await?
    .0
    .data;

    let duplicate_processed_status = if duplicate_webhook {
        let duplicate_result = handle_payment_webhook(
            State(state.clone()),
            webhook_headers,
            Path("mock_payment".to_string()),
            Json(PaymentWebhookRequest {
                provider_event_id: provider_event.provider_event_id.clone(),
                event_type: provider_event.event_type.clone(),
                provider_transaction_no: Some(format!(
                    "mocktxn-{}",
                    provider_event.provider_event_id
                )),
                payment_intent_id: Some(payment_intent_id.clone()),
                transaction_amount: Some("88.00".to_string()),
                currency_code: Some("SGD".to_string()),
                provider_status: Some(provider_event.provider_status.clone()),
                occurred_at: None,
                occurred_at_ms: Some(crate::modules::billing::webhook::now_utc_ms()),
                raw_payload: json!({
                    "source": "mock_payment_simulate_duplicate",
                    "scenario_type": scenario_type,
                }),
            }),
        )
        .await?
        .0
        .data;
        Some(duplicate_result.processed_status)
    } else {
        None
    };

    let status = if webhook_result.processed_status.starts_with("processed")
        || webhook_result.processed_status == "duplicate"
        || webhook_result.processed_status == "out_of_order_ignored"
    {
        "executed"
    } else {
        "failed"
    };
    let case_result_payload = json!({
        "provider_event_id": provider_event.provider_event_id,
        "event_type": provider_event.event_type,
        "provider_status": provider_event.provider_status,
        "http_status_code": provider_event.http_status_code,
        "webhook_processed_status": webhook_result.processed_status,
        "duplicate_processed_status": duplicate_processed_status,
        "applied_payment_status": webhook_result.applied_payment_status,
    });
    update_mock_payment_case_result(&client, &mock_payment_case_id, status, &case_result_payload)
        .await?;

    write_audit_event(
        &client,
        "developer",
        "mock_payment_case",
        &mock_payment_case_id,
        &actor_role,
        "mock.payment.simulate",
        status,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    let view = MockPaymentSimulationView {
        mock_payment_case_id,
        payment_intent_id,
        scenario_type: scenario_type.to_string(),
        provider_key: intent.provider_key,
        provider_kind: provider.kind().to_string(),
        provider_event_id: case_result_payload["provider_event_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        provider_status: case_result_payload["provider_status"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        http_status_code: provider_event.http_status_code,
        webhook_processed_status: case_result_payload["webhook_processed_status"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        duplicate_webhook,
        duplicate_processed_status,
        payment_transaction_id: webhook_result.payment_transaction_id,
        applied_payment_status: webhook_result.applied_payment_status,
    };
    info!(
        action = "mock.payment.simulate",
        mock_payment_case_id = %view.mock_payment_case_id,
        payment_intent_id = %view.payment_intent_id,
        scenario_type = %view.scenario_type,
        webhook_processed_status = %view.webhook_processed_status,
        "mock payment simulation completed"
    );
    Ok(ApiResponse::ok(view))
}
