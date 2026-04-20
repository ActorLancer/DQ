use crate::AppState;
use crate::modules::billing::handlers::{
    header, map_db_connect, require_permission, require_step_up_placeholder,
};
use crate::modules::billing::models::{CompensationExecutionView, CreateCompensationRequest};
use crate::modules::billing::repo::compensation_repository::execute_compensation as execute_compensation_repo;
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

pub async fn create_compensation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateCompensationRequest>,
) -> Result<Json<ApiResponse<CompensationExecutionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::CompensationExecute,
        "compensation execute",
    )?;
    require_step_up_placeholder(&headers, "compensation execute")?;
    let request_id = header(&headers, "x-request-id");
    let idempotency_key = header(&headers, "x-idempotency-key").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: "x-idempotency-key is required for compensation execute".to_string(),
                request_id: request_id.clone(),
            }),
        )
    })?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let actor_user_id = header(&headers, "x-user-id");
    let client = state.db.client().map_err(map_db_connect)?;
    let compensation = execute_compensation_repo(
        &client,
        &payload,
        &idempotency_key,
        None,
        actor_user_id.as_deref(),
        role.as_str(),
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = if compensation.idempotent_replay {"billing.compensation.execute.idempotent_replay"} else {"billing.compensation.execute"},
        compensation_id = %compensation.compensation_id,
        order_id = %compensation.order_id,
        case_id = %compensation.case_id,
        current_status = %compensation.current_status,
        "compensation execute handled"
    );
    Ok(ApiResponse::ok(compensation))
}
