use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageShareGrantRequest {
    pub operation: Option<String>,
    pub asset_object_id: Option<String>,
    pub recipient_ref: Option<String>,
    pub subscriber_ref: Option<String>,
    pub share_protocol: Option<String>,
    pub access_locator: Option<String>,
    pub scope_json: Option<Value>,
    pub expires_at: Option<String>,
    pub receipt_hash: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageShareGrantResponse {
    pub data: ShareGrantResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetShareGrantResponse {
    pub data: ShareGrantListResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareGrantListResponseData {
    pub order_id: String,
    pub sku_id: String,
    pub sku_type: String,
    pub current_state: String,
    pub payment_status: String,
    pub grants: Vec<ShareGrantResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareGrantResponseData {
    pub data_share_grant_id: String,
    pub order_id: String,
    pub asset_object_id: String,
    pub sku_id: String,
    pub sku_type: String,
    pub recipient_ref: String,
    pub subscriber_ref: Option<String>,
    pub share_protocol: String,
    pub access_locator: Option<String>,
    pub grant_status: String,
    pub read_only: bool,
    pub receipt_hash: Option<String>,
    pub granted_at: Option<String>,
    pub revoked_at: Option<String>,
    pub expires_at: Option<String>,
    pub operation: Option<String>,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}
