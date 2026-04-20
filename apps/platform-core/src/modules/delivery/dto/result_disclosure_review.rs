use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResultDisclosureRequest {
    pub result_disclosure_review_id: Option<String>,
    pub review_status: Option<String>,
    pub masking_level: Option<String>,
    pub export_scope: Option<String>,
    pub reviewer_user_id: Option<String>,
    pub approval_ticket_id: Option<String>,
    pub review_notes: Option<String>,
    pub decision_snapshot: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResultDisclosureResponse {
    pub data: ResultDisclosureReviewResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultDisclosureReviewResponseData {
    pub result_disclosure_review_id: String,
    pub order_id: String,
    pub query_run_id: String,
    pub result_object_id: String,
    pub review_status: String,
    pub masking_level: String,
    pub export_scope: String,
    pub reviewer_user_id: Option<String>,
    pub approval_ticket_id: Option<String>,
    pub review_notes: Option<String>,
    pub decision_snapshot: Value,
    pub requires_disclosure_review: bool,
    pub output_boundary_json: Value,
    pub operation: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub created_at: String,
    pub updated_at: String,
    pub reviewed_at: Option<String>,
}
