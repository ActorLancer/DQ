use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageQuerySurfaceRequest {
    pub query_surface_id: Option<String>,
    pub asset_object_id: Option<String>,
    pub environment_id: Option<String>,
    pub surface_type: Option<String>,
    pub binding_mode: Option<String>,
    pub execution_scope: Option<String>,
    pub input_contract_json: Option<Value>,
    pub output_boundary_json: Option<Value>,
    pub query_policy_json: Option<Value>,
    pub quota_policy_json: Option<Value>,
    pub status: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageQuerySurfaceResponse {
    pub data: QuerySurfaceResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySurfaceResponseData {
    pub query_surface_id: String,
    pub product_id: String,
    pub asset_version_id: String,
    pub asset_object_id: Option<String>,
    pub environment_id: String,
    pub surface_type: String,
    pub binding_mode: String,
    pub execution_scope: String,
    pub input_contract_json: Value,
    pub output_boundary_json: Value,
    pub query_policy_json: Value,
    pub quota_policy_json: Value,
    pub status: String,
    pub operation: String,
    pub created_at: String,
    pub updated_at: String,
}
