use crate::modules::billing::service::{BillingPermission, is_allowed};
use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use db::Error;
use kernel::{ErrorCode, ErrorResponse};

pub fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

pub fn require_permission(
    headers: &HeaderMap,
    permission: BillingPermission,
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

pub fn require_step_up_placeholder(
    headers: &HeaderMap,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if header(headers, "x-step-up-token").is_some()
        || header(headers, "x-step-up-challenge-id").is_some()
    {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("x-step-up-token or x-step-up-challenge-id is required for {action}"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

pub(crate) fn map_db_connect(err: Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database connection failed: {err}"),
            request_id: None,
        }),
    )
}
