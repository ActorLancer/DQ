use crate::AppState;
use crate::modules::billing::handlers::{
    header, map_db_connect, require_permission, require_step_up_placeholder,
};
use crate::modules::billing::models::{CreateManualPayoutRequest, ManualPayoutExecutionView};
use crate::modules::billing::repo::payout_repository::execute_manual_payout as execute_manual_payout_repo;
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

pub async fn create_manual_payout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateManualPayoutRequest>,
) -> Result<Json<ApiResponse<ManualPayoutExecutionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PayoutExecuteManual,
        "manual payout execute",
    )?;
    require_step_up_placeholder(&headers, "manual payout execute")?;
    let request_id = header(&headers, "x-request-id");
    let idempotency_key = header(&headers, "x-idempotency-key").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: "x-idempotency-key is required for manual payout execute".to_string(),
                request_id: request_id.clone(),
            }),
        )
    })?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let actor_user_id = header(&headers, "x-user-id");
    let client = state.db.client().map_err(map_db_connect)?;
    let payout = execute_manual_payout_repo(
        &client,
        &payload,
        &idempotency_key,
        actor_user_id.as_deref(),
        role.as_str(),
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = if payout.idempotent_replay {
            "billing.payout.execute_manual.idempotent_replay"
        } else {
            "billing.payout.execute_manual"
        },
        payout_instruction_id = %payout.payout_instruction_id,
        order_id = %payout.order_id,
        settlement_id = %payout.settlement_id,
        current_status = %payout.current_status,
        "manual payout execute handled"
    );
    Ok(ApiResponse::ok(payout))
}
