use crate::modules::order::domain::OrderPriceSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezeOrderPriceSnapshotResponseData {
    pub order_id: String,
    pub snapshot: OrderPriceSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezeOrderPriceSnapshotResponse {
    pub data: FreezeOrderPriceSnapshotResponseData,
}
