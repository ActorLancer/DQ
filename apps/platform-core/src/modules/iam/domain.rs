use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterOrganizationRequest {
    pub org_name: String,
    pub org_type: String,
    pub jurisdiction_code: Option<String>,
    pub compliance_level: Option<String>,
    pub certification_level: Option<String>,
    #[serde(default)]
    pub whitelist_refs: Vec<String>,
    #[serde(default)]
    pub graylist_refs: Vec<String>,
    #[serde(default)]
    pub blacklist_refs: Vec<String>,
    #[serde(default)]
    pub risk_profile: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OrganizationAggregateView {
    pub org_id: String,
    pub org_name: String,
    pub org_type: String,
    pub org_status: String,
    pub jurisdiction_code: Option<String>,
    pub compliance_level: Option<String>,
    pub certification_level: Option<String>,
    pub whitelist_refs: Vec<String>,
    pub graylist_refs: Vec<String>,
    pub blacklist_refs: Vec<String>,
    pub blacklist_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDepartmentRequest {
    pub org_id: String,
    pub department_name: String,
    pub parent_department_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DepartmentView {
    pub department_id: String,
    pub org_id: String,
    pub department_name: String,
    pub parent_department_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub org_id: String,
    pub department_id: Option<String>,
    pub login_id: String,
    pub display_name: String,
    pub user_type: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UserView {
    pub user_id: String,
    pub org_id: String,
    pub department_id: Option<String>,
    pub login_id: String,
    pub display_name: String,
    pub user_type: String,
    pub status: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApplicationRequest {
    pub org_id: String,
    pub app_name: String,
    pub app_type: Option<String>,
    pub client_id: String,
    pub client_secret_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchApplicationRequest {
    pub app_name: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ApplicationView {
    pub app_id: String,
    pub org_id: String,
    pub app_name: String,
    pub app_type: String,
    pub status: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConnectorRequest {
    pub org_id: Option<String>,
    pub connector_name: String,
    pub connector_type: String,
    pub endpoint_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConnectorView {
    pub connector_id: String,
    pub org_id: Option<String>,
    pub connector_name: String,
    pub connector_type: String,
    pub status: String,
    pub endpoint_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateExecutionEnvironmentRequest {
    pub org_id: Option<String>,
    pub connector_id: Option<String>,
    pub environment_name: String,
    pub environment_type: String,
    pub region_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExecutionEnvironmentView {
    pub environment_id: String,
    pub org_id: Option<String>,
    pub connector_id: Option<String>,
    pub environment_name: String,
    pub environment_type: String,
    pub status: String,
    pub region_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionContextView {
    pub mode: String,
    pub user_id: Option<String>,
    pub org_id: Option<String>,
    pub login_id: Option<String>,
    pub display_name: Option<String>,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
    pub auth_context_level: String,
}
