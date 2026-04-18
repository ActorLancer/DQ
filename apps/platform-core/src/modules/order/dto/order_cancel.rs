use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub data: CancelOrderResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponseData {
    pub order_id: String,
    pub previous_state: String,
    pub current_state: String,
    pub payment_status: String,
    pub refund_branch: String,
    pub refund_required: bool,
    pub reason_code: String,
    pub canceled_at: String,
}
