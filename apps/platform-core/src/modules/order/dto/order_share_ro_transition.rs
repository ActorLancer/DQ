use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRoTransitionRequest {
    pub action: String,
    pub reason_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRoTransitionResponse {
    pub data: ShareRoTransitionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRoTransitionResponseData {
    pub order_id: String,
    pub action: String,
    pub previous_state: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub acceptance_status: String,
    pub settlement_status: String,
    pub dispute_status: String,
    pub reason_code: String,
    pub transitioned_at: String,
}
