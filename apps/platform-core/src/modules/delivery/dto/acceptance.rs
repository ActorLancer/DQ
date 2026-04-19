use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcceptOrderRequest {
    pub note: Option<String>,
    pub verification_summary: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectOrderRequest {
    pub reason_code: String,
    pub reason_detail: Option<String>,
    pub verification_summary: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptOrderResponse {
    pub data: OrderAcceptanceResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectOrderResponse {
    pub data: OrderAcceptanceResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAcceptanceResponseData {
    pub order_id: String,
    pub delivery_id: Option<String>,
    pub delivery_branch: Option<String>,
    pub action: String,
    pub previous_state: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub acceptance_status: String,
    pub settlement_status: String,
    pub dispute_status: String,
    pub reason_code: String,
    pub reason_detail: Option<String>,
    pub accepted_at: Option<String>,
    pub processed_at: String,
    pub operation: Option<String>,
}
