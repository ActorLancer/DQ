use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::ErrorResponse;
use tracing::info;

use crate::AppState;
use crate::modules::catalog::domain::StandardScenarioTemplateView;
use crate::modules::catalog::service::CatalogPermission;
use crate::modules::catalog::standard_scenarios::standard_scenario_templates;

use super::super::support::*;

const STANDARD_SCENARIO_REF_ID: &str = "00000000-0000-0000-0000-000000000023";

pub(in crate::modules::catalog) async fn get_standard_scenario_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<StandardScenarioTemplateView>>>, (StatusCode, Json<ErrorResponse>)>
{
    require_permission(
        &headers,
        CatalogPermission::ProductRead,
        "catalog standard scenario template read",
    )?;
    if let Ok(client) = state.db.client() {
        write_audit_event(
            &client,
            "catalog_standard_scenarios",
            STANDARD_SCENARIO_REF_ID,
            header(&headers, "x-role").as_deref().unwrap_or("unknown"),
            "catalog.standard.scenarios.read",
            "success",
            header(&headers, "x-request-id").as_deref(),
            header(&headers, "x-trace-id").as_deref(),
        )
        .await?;
    }
    info!(
        action = "catalog.standard.scenarios.read",
        "standard scenarios queried"
    );
    Ok(ApiResponse::ok(standard_scenario_templates()))
}
