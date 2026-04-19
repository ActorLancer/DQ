use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTemplateRunRequest {
    pub template_query_grant_id: Option<String>,
    pub query_template_id: String,
    pub requester_user_id: Option<String>,
    pub request_payload_json: Value,
    pub output_boundary_json: Option<Value>,
    pub masked_level: Option<String>,
    pub export_scope: Option<String>,
    pub approval_ticket_id: Option<String>,
    pub execution_metadata_json: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTemplateRunResponse {
    pub data: QueryRunResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetQueryRunsResponse {
    pub data: QueryRunListResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRunListResponseData {
    pub order_id: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub query_runs: Vec<QueryRunResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRunResponseData {
    pub query_run_id: String,
    pub order_id: String,
    pub template_query_grant_id: String,
    pub query_surface_id: String,
    pub query_template_id: String,
    pub query_template_name: String,
    pub query_template_version: i32,
    pub requester_user_id: Option<String>,
    pub execution_mode: String,
    pub request_payload_json: Value,
    pub result_summary_json: Value,
    pub result_object_id: Option<String>,
    pub result_object_uri: Option<String>,
    pub bucket_name: Option<String>,
    pub object_key: Option<String>,
    pub result_row_count: i64,
    pub billed_units: String,
    pub export_attempt_count: i32,
    pub status: String,
    pub masked_level: String,
    pub export_scope: String,
    pub approval_ticket_id: Option<String>,
    pub sensitive_policy_snapshot: Value,
    pub operation: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}
