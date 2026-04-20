use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageSensitiveExecutionPolicyRequest {
    pub sensitive_execution_policy_id: Option<String>,
    pub query_surface_id: Option<String>,
    pub template_query_grant_id: Option<String>,
    pub sandbox_workspace_id: Option<String>,
    pub policy_scope: Option<String>,
    pub execution_mode: Option<String>,
    pub output_boundary_json: Option<Value>,
    pub export_control_json: Option<Value>,
    pub step_up_required: Option<bool>,
    pub attestation_required: Option<bool>,
    pub approval_ticket_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageSensitiveExecutionPolicyResponse {
    pub data: SensitiveExecutionPolicyResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveExecutionPolicyResponseData {
    pub sensitive_execution_policy_id: String,
    pub order_id: String,
    pub sku_id: String,
    pub sku_type: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub acceptance_status: String,
    pub settlement_status: String,
    pub dispute_status: String,
    pub operation: String,
    pub created_at: String,
    pub updated_at: String,
    pub policy: SensitiveExecutionPolicyModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveExecutionPolicyModel {
    pub sensitive_execution_policy_id: String,
    pub query_surface_id: Option<String>,
    pub template_query_grant_id: Option<String>,
    pub sandbox_workspace_id: Option<String>,
    pub policy_scope: String,
    pub execution_mode: String,
    pub policy_status: String,
    pub output_boundary_json: Value,
    pub export_control_json: Value,
    pub policy_snapshot_json: Value,
    pub step_up_required: bool,
    pub attestation_required: bool,
    pub approval_ticket_id: Option<String>,
}
