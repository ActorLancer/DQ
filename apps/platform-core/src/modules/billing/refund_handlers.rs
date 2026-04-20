use crate::AppState;
use crate::modules::billing::handlers::{
    header, map_db_connect, require_permission, require_step_up_placeholder,
};
use crate::modules::billing::models::{CreateRefundRequest, RefundExecutionView};
use crate::modules::billing::repo::refund_repository::execute_refund as execute_refund_repo;
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

pub async fn create_refund(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateRefundRequest>,
) -> Result<Json<ApiResponse<RefundExecutionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, BillingPermission::RefundExecute, "refund execute")?;
    require_step_up_placeholder(&headers, "refund execute")?;
    let request_id = header(&headers, "x-request-id");
    let idempotency_key = header(&headers, "x-idempotency-key").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: "x-idempotency-key is required for refund execute".to_string(),
                request_id: request_id.clone(),
            }),
        )
    })?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let tenant_scope_id = tenant_scope_id_for_role(&headers, &role)?;
    let actor_user_id = header(&headers, "x-user-id");
    let client = state.db.client().map_err(map_db_connect)?;
    let refund = execute_refund_repo(
        &client,
        &payload,
        &idempotency_key,
        tenant_scope_id.as_deref(),
        actor_user_id.as_deref(),
        role.as_str(),
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = if refund.idempotent_replay {"billing.refund.execute.idempotent_replay"} else {"billing.refund.execute"},
        refund_id = %refund.refund_id,
        order_id = %refund.order_id,
        case_id = %refund.case_id,
        current_status = %refund.current_status,
        "refund execute handled"
    );
    Ok(ApiResponse::ok(refund))
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
                message: "x-tenant-id is required for tenant-scoped refund actions".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })
}
