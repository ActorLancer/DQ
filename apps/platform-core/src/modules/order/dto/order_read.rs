use crate::modules::order::domain::OrderPriceSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderDetailResponse {
    pub data: GetOrderDetailResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderDetailResponseData {
    pub order_id: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub product_id: String,
    pub sku_id: String,
    pub current_state: String,
    pub payment_status: String,
    pub delivery_status: String,
    pub acceptance_status: String,
    pub settlement_status: String,
    pub dispute_status: String,
    pub amount: String,
    pub currency_code: String,
    pub price_snapshot: Option<OrderPriceSnapshot>,
    pub created_at: String,
    pub updated_at: String,
}
