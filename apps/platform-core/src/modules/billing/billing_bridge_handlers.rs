use crate::AppState;
use crate::modules::billing::handlers::{header, map_db_connect, require_permission};
use crate::modules::billing::models::{BillingBridgeProcessRequest, BillingBridgeProcessView};
use crate::modules::billing::repo::billing_bridge_repository::process_billing_bridge_events_for_order;
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::ErrorResponse;

pub async fn process_billing_bridge_for_order_api(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<BillingBridgeProcessRequest>,
) -> Result<Json<ApiResponse<BillingBridgeProcessView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::BillingBridgeProcess,
        "billing bridge process",
    )?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let mut client = state.db.client().map_err(map_db_connect)?;
    let result = process_billing_bridge_events_for_order(
        &mut client,
        &order_id,
        payload.outbox_event_id.as_deref(),
        &role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(BillingBridgeProcessView {
        order_id: result.order_id,
        processed_count: result.processed_count as u32,
        ignored_count: result.ignored_count as u32,
        replayed_count: result.replayed_count as u32,
        processed_outbox_event_ids: result.processed_outbox_event_ids,
        processed_billing_event_ids: result.processed_billing_event_ids,
        ignored_outbox_event_ids: result.ignored_outbox_event_ids,
    }))
}
