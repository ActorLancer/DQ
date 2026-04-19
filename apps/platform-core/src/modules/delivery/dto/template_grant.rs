use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageTemplateGrantRequest {
    pub template_query_grant_id: Option<String>,
    pub query_surface_id: Option<String>,
    pub asset_object_id: Option<String>,
    pub environment_id: Option<String>,
    pub template_type: Option<String>,
    pub allowed_template_ids: Option<Vec<String>>,
    pub execution_rule_snapshot: Option<Value>,
    pub output_boundary_json: Option<Value>,
    pub run_quota_json: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageTemplateGrantResponse {
    pub data: TemplateGrantResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateGrantResponseData {
    pub template_query_grant_id: String,
    pub order_id: String,
    pub query_surface_id: String,
    pub asset_object_id: String,
    pub environment_id: Option<String>,
    pub sku_id: String,
    pub sku_type: String,
    pub template_type: String,
    pub template_digest: String,
    pub allowed_template_ids: Vec<String>,
    pub execution_rule_snapshot: Value,
    pub output_boundary_json: Value,
    pub run_quota_json: Value,
    pub grant_status: String,
    pub operation: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub created_at: String,
    pub updated_at: String,
}
