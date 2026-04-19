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
    pub sensitive_execution_policy_id: String,
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
    pub workspace: SandboxWorkspaceModel,
    pub session: SandboxSessionModel,
    pub seat: SandboxSeatModel,
    pub export_control: SandboxExportControlModel,
    pub attestation: Option<SandboxAttestationRefModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxWorkspaceModel {
    pub sandbox_workspace_id: String,
    pub query_surface_id: String,
    pub environment_id: String,
    pub workspace_name: String,
    pub workspace_status: String,
    pub clean_room_mode: String,
    pub data_residency_mode: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSessionModel {
    pub sandbox_session_id: String,
    pub session_status: String,
    pub session_started_at: String,
    pub expire_at: String,
    pub session_query_count: i32,
    pub export_attempt_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSeatModel {
    pub seat_user_id: String,
    pub login_id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub seat_status: String,
    pub seat_limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExportControlModel {
    pub sensitive_execution_policy_id: String,
    pub policy_scope: String,
    pub execution_mode: String,
    pub policy_status: String,
    pub export_control_json: Value,
    pub output_boundary_json: Value,
    pub policy_snapshot_json: Value,
    pub step_up_required: bool,
    pub attestation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxAttestationRefModel {
    pub attestation_record_id: String,
    pub attestation_type: String,
    pub status: String,
    pub attestation_uri: Option<String>,
    pub attestation_hash: Option<String>,
    pub verifier_ref: Option<String>,
    pub verified_at: Option<String>,
    pub metadata_json: Value,
}
