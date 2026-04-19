use crate::modules::authorization::domain::AuthorizationModelSnapshot;
use crate::modules::delivery::domain::StorageGatewaySnapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderLifecycleSnapshotsResponse {
    pub data: GetOrderLifecycleSnapshotsResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderLifecycleSnapshotsResponseData {
    pub order: OrderLifecycleSnapshot,
    pub contract: Option<ContractLifecycleSnapshot>,
    pub authorization: Option<AuthorizationLifecycleSnapshot>,
    pub delivery: Option<DeliveryLifecycleSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLifecycleSnapshot {
    pub order_id: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub current_state: String,
    pub payment: PaymentLifecycleSnapshot,
    pub acceptance: AcceptanceLifecycleSnapshot,
    pub settlement: SettlementLifecycleSnapshot,
    pub dispute: DisputeLifecycleSnapshot,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentLifecycleSnapshot {
    pub current_status: String,
    pub payment_mode: String,
    pub amount: String,
    pub currency_code: String,
    pub buyer_locked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceLifecycleSnapshot {
    pub current_status: String,
    pub accepted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementLifecycleSnapshot {
    pub current_status: String,
    pub settled_at: Option<String>,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeLifecycleSnapshot {
    pub current_status: String,
    pub last_reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLifecycleSnapshot {
    pub contract_id: String,
    pub contract_status: String,
    pub contract_digest: Option<String>,
    pub signed_at: Option<String>,
    pub variables_json: Value,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationLifecycleSnapshot {
    pub authorization_id: String,
    pub current_status: String,
    pub grant_type: String,
    pub granted_to_type: String,
    pub granted_to_id: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub authorization_model: AuthorizationModelSnapshot,
    pub policy_snapshot: Value,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryLifecycleSnapshot {
    pub delivery_id: String,
    pub delivery_type: String,
    pub delivery_route: Option<String>,
    pub current_status: String,
    pub committed_at: Option<String>,
    pub expires_at: Option<String>,
    pub receipt_hash: Option<String>,
    pub storage_gateway: Option<StorageGatewaySnapshot>,
    pub updated_at: String,
}
