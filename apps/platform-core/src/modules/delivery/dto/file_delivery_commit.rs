use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitOrderDeliveryRequest {
    pub branch: String,
    pub object_uri: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
    pub content_hash: String,
    pub encryption_algo: Option<String>,
    pub plaintext_visible_to_platform: Option<bool>,
    pub storage_namespace_id: Option<String>,
    pub key_cipher: String,
    pub key_control_mode: Option<String>,
    pub unwrap_policy_json: Option<Value>,
    pub key_version: Option<String>,
    pub expire_at: String,
    pub download_limit: i32,
    pub delivery_commit_hash: String,
    pub receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitOrderDeliveryResponse {
    pub data: CommitOrderDeliveryResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitOrderDeliveryResponseData {
    pub order_id: String,
    pub delivery_id: String,
    pub object_id: String,
    pub envelope_id: String,
    pub ticket_id: String,
    pub branch: String,
    pub previous_state: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub acceptance_status: String,
    pub settlement_status: String,
    pub dispute_status: String,
    pub bucket_name: String,
    pub object_key: String,
    pub expires_at: String,
    pub download_limit: i32,
    pub receipt_hash: String,
    pub delivery_commit_hash: String,
    pub committed_at: String,
}
