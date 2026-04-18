use crate::modules::authorization::domain::AuthorizationExpectationSnapshot;
use crate::modules::contract::domain::ContractExpectationSnapshot;
use crate::modules::order::domain::{PreOrderRequestKind, TradePreRequest, TradePreRequestDetails};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTradePreRequestRequest {
    pub buyer_org_id: String,
    pub product_id: Option<String>,
    pub created_by: Option<String>,
    pub request_kind: PreOrderRequestKind,
    pub details: TradePreRequestDetails,
    pub contract_expectation: Option<ContractExpectationSnapshot>,
    pub authorization_expectation: Option<AuthorizationExpectationSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePreRequestResponseData {
    pub inquiry_id: String,
    pub buyer_org_id: String,
    pub product_id: Option<String>,
    pub created_by: Option<String>,
    pub status: String,
    pub request_kind: String,
    pub details: TradePreRequestDetails,
    pub contract_expectation: ContractExpectationSnapshot,
    pub authorization_expectation: AuthorizationExpectationSnapshot,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePreRequestResponse {
    pub data: TradePreRequestResponseData,
}

impl From<TradePreRequest> for TradePreRequestResponseData {
    fn from(value: TradePreRequest) -> Self {
        Self {
            inquiry_id: value.inquiry_id,
            buyer_org_id: value.buyer_org_id,
            product_id: value.product_id,
            created_by: value.created_by,
            status: value.status.as_str().to_string(),
            request_kind: value.request_payload.request_kind.as_str().to_string(),
            details: value.request_payload.details,
            contract_expectation: value.request_payload.contract_expectation,
            authorization_expectation: value.request_payload.authorization_expectation,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
