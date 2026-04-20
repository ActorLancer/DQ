use crate::AppState;
use crate::modules::billing::handlers::{header, map_db_connect, require_permission};
use crate::modules::billing::models::{CreateShareRoCycleChargeRequest, ShareRoCycleChargeView};
use crate::modules::billing::repo::share_ro_billing_repository::record_share_ro_cycle_charge_in_tx;
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::ErrorResponse;
use tracing::info;

pub async fn create_share_ro_cycle_charge(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CreateShareRoCycleChargeRequest>,
) -> Result<Json<ApiResponse<ShareRoCycleChargeView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::ShareRoCycleCharge,
        "SHARE_RO cycle charge",
    )?;
    let request_id = header(&headers, "x-request-id");
    let actor_role = header(&headers, "x-role").unwrap_or_default();
    let client = state.db.client().map_err(map_db_connect)?;
    let charge = record_share_ro_cycle_charge_in_tx(
        &client,
        &order_id,
        actor_role.as_str(),
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
        &payload,
    )
    .await?;

    info!(
        action = if charge.billing_event_replayed {
            "billing.event.record.share_ro_cycle.idempotent_replay"
        } else {
            "billing.event.record.share_ro_cycle"
        },
        order_id = %charge.order_id,
        billing_cycle_code = %charge.billing_cycle_code,
        billing_event_id = %charge.billing_event_id,
        current_state = %charge.current_state,
        "SHARE_RO cycle charge recorded"
    );
    Ok(ApiResponse::ok(charge))
}
