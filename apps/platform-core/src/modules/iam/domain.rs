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
pub struct RotateApplicationSecretRequest {
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
    pub client_secret_status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateInvitationRequest {
    pub org_id: String,
    pub invited_email: Option<String>,
    pub invited_phone: Option<String>,
    #[serde(default)]
    pub invited_roles: Vec<String>,
    pub invitation_type: Option<String>,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvitationListQuery {
    pub org_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InvitationView {
    pub invitation_id: String,
    pub org_id: String,
    pub invited_email: Option<String>,
    pub invited_phone: Option<String>,
    pub invitation_type: String,
    pub status: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionListQuery {
    pub user_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionView {
    pub session_id: String,
    pub user_id: String,
    pub trusted_device_id: Option<String>,
    pub session_type: String,
    pub auth_context_level: String,
    pub session_status: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceListQuery {
    pub user_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DeviceView {
    pub trusted_device_id: String,
    pub user_id: String,
    pub device_name: Option<String>,
    pub platform: Option<String>,
    pub browser: Option<String>,
    pub trust_level: String,
    pub status: String,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AccessPermissionRuleView {
    pub permission_code: String,
    pub scopes: Vec<String>,
    pub api_patterns: Vec<String>,
    pub button_keys: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessCheckRequest {
    pub permission_code: String,
    pub scope: Option<String>,
    pub api: Option<String>,
    pub button_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AccessCheckView {
    pub allowed: bool,
    pub permission_code: String,
    pub matched_role: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StepUpCheckRequest {
    pub action_name: String,
    pub target_ref_type: Option<String>,
    pub target_ref_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StepUpCheckView {
    pub challenge_id: String,
    pub action_name: String,
    pub requires_step_up: bool,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StepUpVerifyRequest {
    pub verification_code: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MfaAuthenticatorView {
    pub authenticator_id: String,
    pub user_id: String,
    pub authenticator_type: String,
    pub device_label: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMfaAuthenticatorRequest {
    pub user_id: String,
    pub authenticator_type: String,
    pub device_label: Option<String>,
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
