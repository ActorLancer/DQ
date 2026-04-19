use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageRevisionSubscriptionRequest {
    pub cadence: Option<String>,
    pub delivery_channel: Option<String>,
    pub start_version_no: Option<i32>,
    pub last_delivered_version_no: Option<i32>,
    pub next_delivery_at: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageRevisionSubscriptionResponse {
    pub data: RevisionSubscriptionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRevisionSubscriptionResponse {
    pub data: RevisionSubscriptionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionSubscriptionResponseData {
    pub revision_subscription_id: String,
    pub order_id: String,
    pub asset_id: String,
    pub sku_id: String,
    pub sku_type: String,
    pub cadence: String,
    pub delivery_channel: String,
    pub start_version_no: i32,
    pub last_delivered_version_no: Option<i32>,
    pub current_version_no: i32,
    pub next_delivery_at: String,
    pub subscription_status: String,
    pub current_state: String,
    pub payment_status: String,
    pub operation: Option<String>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}
