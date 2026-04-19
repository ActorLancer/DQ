use crate::AppState;
use crate::modules::delivery::dto::{
    CommitOrderDeliveryRequest, CommitOrderDeliveryResponse, DownloadTicketResponse,
};
use crate::modules::delivery::repo::{commit_file_delivery, issue_download_ticket};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use db::Error;
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryPermission {
    CommitFileDelivery,
    IssueDownloadTicket,
}

pub async fn commit_order_delivery_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CommitOrderDeliveryRequest>,
) -> Result<Json<ApiResponse<CommitOrderDeliveryResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::CommitFileDelivery,
        "file delivery commit",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let committed = commit_file_delivery(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(CommitOrderDeliveryResponse {
        data: committed,
    }))
}

pub async fn issue_download_ticket_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<DownloadTicketResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::IssueDownloadTicket,
        "download ticket issuance",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let ticket = issue_download_ticket(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(DownloadTicketResponse { data: ticket }))
}

fn is_allowed(role: &str, permission: DeliveryPermission) -> bool {
    match permission {
        DeliveryPermission::CommitFileDelivery => matches!(
            role,
            "seller_operator" | "tenant_admin" | "platform_admin" | "platform_risk_settlement"
        ),
        DeliveryPermission::IssueDownloadTicket => matches!(
            role,
            "buyer_operator" | "tenant_admin" | "platform_admin" | "platform_risk_settlement"
        ),
    }
}

fn require_permission(
    headers: &HeaderMap,
    permission: DeliveryPermission,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = headers
        .get("x-role")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if is_allowed(role, permission) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("{action} is forbidden for current role"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn map_db_connect(err: Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database connection failed: {err}"),
            request_id: None,
        }),
    )
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}
