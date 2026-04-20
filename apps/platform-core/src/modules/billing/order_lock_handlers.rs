use crate::AppState;
use crate::modules::billing::db::write_audit_event;
use crate::modules::billing::handlers::{header, map_db_connect, require_permission};
use crate::modules::billing::models::{LockOrderRequest, OrderLockView};
use crate::modules::billing::repo::order_lock_repository;
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::ErrorResponse;
use tracing::info;

pub async fn lock_order_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<LockOrderRequest>,
) -> Result<Json<ApiResponse<OrderLockView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, BillingPermission::OrderLock, "order lock")?;
    let client = state.db.client().map_err(map_db_connect)?;
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let tenant_scope_id = header(&headers, "x-tenant-id");
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());

    let (view, replayed) = order_lock_repository::lock_order_payment(
        &client,
        &order_id,
        &payload,
        tenant_scope_id.as_deref(),
        request_id.as_deref(),
    )
    .await?;

    write_audit_event(
        &client,
        "trade",
        "order",
        &view.order_id,
        &actor_role,
        if replayed {
            "order.payment.lock.idempotent_replay"
        } else {
            "order.payment.lock"
        },
        if replayed {
            "idempotent_replay"
        } else {
            "success"
        },
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;
    info!(
        action = "order.payment.lock",
        order_id = %view.order_id,
        payment_intent_id = %view.payment_intent_id,
        payment_status = %view.payment_status,
        replayed,
        "order lock operation completed"
    );
    Ok(ApiResponse::ok(view))
}
