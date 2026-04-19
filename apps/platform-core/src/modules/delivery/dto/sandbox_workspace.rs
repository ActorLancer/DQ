use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageSandboxWorkspaceRequest {
    pub query_surface_id: Option<String>,
    pub workspace_name: Option<String>,
    pub seat_user_id: Option<String>,
    pub expire_at: Option<String>,
    pub export_policy_json: Option<Value>,
    pub clean_room_mode: Option<String>,
    pub data_residency_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageSandboxWorkspaceResponse {
    pub data: SandboxWorkspaceResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxWorkspaceResponseData {
    pub sandbox_workspace_id: String,
    pub sandbox_session_id: String,
    pub order_id: String,
    pub query_surface_id: String,
    pub environment_id: String,
    pub environment_name: String,
    pub environment_type: String,
    pub network_zone: Option<String>,
    pub region_code: Option<String>,
    pub sku_id: String,
    pub sku_type: String,
    pub workspace_name: String,
    pub workspace_status: String,
    pub session_status: String,
    pub seat_user_id: String,
    pub clean_room_mode: String,
    pub data_residency_mode: String,
    pub export_policy_json: Value,
    pub output_boundary_json: Value,
    pub environment_limits_json: Value,
    pub session_started_at: String,
    pub expire_at: String,
    pub session_query_count: i32,
    pub export_attempt_count: i32,
    pub operation: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub acceptance_status: String,
    pub settlement_status: String,
    pub dispute_status: String,
    pub created_at: String,
    pub updated_at: String,
}
