use crate::modules::authorization::domain::AuthorizationExpectationSnapshot;
use crate::modules::contract::domain::ContractExpectationSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreOrderRequestKind {
    Rfq,
    SampleRequest,
    PocRequest,
}

impl PreOrderRequestKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PreOrderRequestKind::Rfq => "rfq",
            PreOrderRequestKind::SampleRequest => "sample_request",
            PreOrderRequestKind::PocRequest => "poc_request",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InquiryStatus {
    Open,
    Submitted,
    Closed,
}

impl InquiryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            InquiryStatus::Open => "open",
            InquiryStatus::Submitted => "submitted",
            InquiryStatus::Closed => "closed",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw {
            "submitted" => InquiryStatus::Submitted,
            "closed" => InquiryStatus::Closed,
            _ => InquiryStatus::Open,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TradePreRequestDetails {
    pub title: Option<String>,
    pub description: Option<String>,
    pub expected_budget_range: Option<String>,
    pub sample_field_scope: Option<Vec<String>>,
    pub poc_goal: Option<String>,
    pub expected_start_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreOrderRequestPayload {
    pub request_kind: PreOrderRequestKind,
    pub details: TradePreRequestDetails,
    pub contract_expectation: ContractExpectationSnapshot,
    pub authorization_expectation: AuthorizationExpectationSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradePreRequest {
    pub inquiry_id: String,
    pub buyer_org_id: String,
    pub product_id: Option<String>,
    pub created_by: Option<String>,
    pub status: InquiryStatus,
    pub request_payload: PreOrderRequestPayload,
    pub created_at: String,
    pub updated_at: String,
}
