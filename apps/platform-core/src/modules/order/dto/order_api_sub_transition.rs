use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSubTransitionRequest {
    pub action: String,
    pub reason_note: Option<String>,
    pub billing_cycle_code: Option<String>,
    pub billing_amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSubTransitionResponse {
    pub data: ApiSubTransitionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSubTransitionResponseData {
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
    pub billing_event_id: Option<String>,
    pub billing_event_type: Option<String>,
    pub billing_event_replayed: bool,
    pub transitioned_at: String,
}
