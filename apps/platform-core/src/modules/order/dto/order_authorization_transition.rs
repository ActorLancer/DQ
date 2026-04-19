use crate::modules::authorization::domain::AuthorizationModelSnapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAuthorizationTransitionRequest {
    pub action: String,
    pub policy_id: Option<String>,
    pub grant_type: Option<String>,
    pub granted_to_type: Option<String>,
    pub granted_to_id: Option<String>,
    pub valid_to: Option<String>,
    pub reason_note: Option<String>,
    pub policy_snapshot: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAuthorizationTransitionResponse {
    pub data: OrderAuthorizationTransitionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAuthorizationTransitionResponseData {
    pub order_id: String,
    pub authorization_id: String,
    pub action: String,
    pub previous_status: Option<String>,
    pub current_status: String,
    pub policy_id: String,
    pub policy_name: String,
    pub policy_status: String,
    pub grant_type: String,
    pub granted_to_type: String,
    pub granted_to_id: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub reason_code: String,
    pub authorization_model: AuthorizationModelSnapshot,
    pub policy_snapshot: Value,
    pub transitioned_at: String,
}
