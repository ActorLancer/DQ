use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageQueryTemplateRequest {
    pub query_template_id: Option<String>,
    pub template_name: Option<String>,
    pub template_type: Option<String>,
    pub template_body_ref: Option<String>,
    pub version_no: Option<i32>,
    pub parameter_schema_json: Option<Value>,
    pub analysis_rule_json: Option<Value>,
    pub result_schema_json: Option<Value>,
    pub export_policy_json: Option<Value>,
    pub risk_guard_json: Option<Value>,
    pub whitelist_fields: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageQueryTemplateResponse {
    pub data: QueryTemplateResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTemplateResponseData {
    pub query_template_id: String,
    pub query_surface_id: String,
    pub template_name: String,
    pub template_type: String,
    pub template_body_ref: Option<String>,
    pub version_no: i32,
    pub parameter_schema_json: Value,
    pub analysis_rule_json: Value,
    pub result_schema_json: Value,
    pub export_policy_json: Value,
    pub risk_guard_json: Value,
    pub whitelist_fields: Vec<String>,
    pub status: String,
    pub operation: String,
    pub created_at: String,
    pub updated_at: String,
}
