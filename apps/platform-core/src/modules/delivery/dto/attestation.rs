use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderAttestationsResponse {
    pub data: OrderAttestationListResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAttestationListResponseData {
    pub order_id: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub attestations: Vec<OrderAttestationResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAttestationResponseData {
    pub attestation_record_id: String,
    pub order_id: String,
    pub query_run_id: Option<String>,
    pub sandbox_session_id: Option<String>,
    pub environment_id: Option<String>,
    pub environment_name: Option<String>,
    pub environment_type: Option<String>,
    pub attestation_type: String,
    pub attestation_uri: Option<String>,
    pub attestation_hash: Option<String>,
    pub verifier_ref: Option<String>,
    pub verified_at: Option<String>,
    pub status: String,
    pub metadata_json: Value,
    pub source_type: String,
    pub query_run_status: Option<String>,
    pub sandbox_session_status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageDestructionAttestationRequest {
    pub destruction_attestation_id: Option<String>,
    pub object_id: Option<String>,
    pub ref_type: Option<String>,
    pub retention_action: Option<String>,
    pub attestation_uri: Option<String>,
    pub attestation_hash: Option<String>,
    pub executed_by_type: Option<String>,
    pub executed_by_id: Option<String>,
    pub approval_ticket_id: Option<String>,
    pub executed_at: Option<String>,
    pub status: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageDestructionAttestationResponse {
    pub data: DestructionAttestationResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructionAttestationResponseData {
    pub destruction_attestation_id: String,
    pub order_id: String,
    pub object_id: Option<String>,
    pub ref_type: String,
    pub retention_action: String,
    pub attestation_uri: Option<String>,
    pub attestation_hash: Option<String>,
    pub executed_by_type: String,
    pub executed_by_id: Option<String>,
    pub approval_ticket_id: Option<String>,
    pub executed_at: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub object_bucket_name: Option<String>,
    pub object_key: Option<String>,
    pub object_link_type: String,
    pub object_link_status: Option<String>,
    pub operation: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub created_at: String,
    pub updated_at: String,
}
