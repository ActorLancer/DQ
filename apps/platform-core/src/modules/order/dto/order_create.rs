use crate::modules::order::domain::OrderPriceSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub buyer_org_id: String,
    pub product_id: String,
    pub sku_id: String,
    pub scenario_code: Option<String>,
    pub inquiry_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub data: CreateOrderResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderResponseData {
    pub order_id: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub product_id: String,
    pub sku_id: String,
    pub current_state: String,
    pub payment_status: String,
    pub order_amount: String,
    pub currency_code: String,
    pub price_snapshot: OrderPriceSnapshot,
    pub created_at: String,
}
