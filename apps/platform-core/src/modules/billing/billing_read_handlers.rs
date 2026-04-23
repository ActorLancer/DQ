use crate::AppState;
use crate::modules::billing::db::write_audit_event;
use crate::modules::billing::handlers::{header, map_db_connect, require_permission};
use crate::modules::billing::models::BillingOrderDetailView;
use crate::modules::billing::repo::billing_read_repository::get_billing_order_detail;
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

pub async fn get_billing_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<String>,
) -> Result<Json<ApiResponse<BillingOrderDetailView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::BillingEventRead,
        "billing order read",
    )?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let tenant_scope_id = tenant_scope_id_for_role(&headers, &role)?;
    let client = state.db.client().map_err(map_db_connect)?;
    let detail = get_billing_order_detail(
        &client,
        &order_id,
        tenant_scope_id.as_deref(),
        header(&headers, "x-request-id").as_deref(),
    )
    .await?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("billing order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;

    write_audit_event(
        &client,
        "billing",
        "order",
        &detail.order_id,
        role.as_str(),
        "billing.order.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "billing.order.read",
        order_id = %detail.order_id,
        settlement_status = %detail.settlement_status,
        "billing order queried"
    );
    Ok(ApiResponse::ok(detail))
}

fn tenant_scope_id_for_role(
    headers: &HeaderMap,
    role: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    if !matches!(role, "buyer_operator" | "tenant_admin" | "tenant_operator") {
        return Ok(None);
    }
    header(headers, "x-tenant-id").map(Some).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "x-tenant-id is required for tenant-scoped billing read".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })
}
