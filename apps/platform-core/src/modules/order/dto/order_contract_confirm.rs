use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmOrderContractRequest {
    pub contract_template_id: String,
    pub contract_digest: String,
    #[serde(default)]
    pub variables_json: Value,
    pub signer_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmOrderContractResponse {
    pub data: ConfirmOrderContractResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmOrderContractResponseData {
    pub order_id: String,
    pub contract_id: String,
    pub contract_status: String,
    pub order_status: String,
    pub signer_role: String,
    pub signed_at: String,
}
