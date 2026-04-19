use crate::modules::authorization::domain::AuthorizationModelSnapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderRelations {
    pub contract: Option<OrderContractRelation>,
    pub authorizations: Vec<OrderAuthorizationRelation>,
    pub deliveries: Vec<OrderDeliveryRelation>,
    pub billing: OrderBillingRelations,
    pub disputes: Vec<OrderDisputeRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderContractRelation {
    pub contract_id: String,
    pub contract_template_id: Option<String>,
    pub contract_status: String,
    pub contract_digest: Option<String>,
    pub data_contract_id: Option<String>,
    pub data_contract_digest: Option<String>,
    pub signed_at: Option<String>,
    pub variables_json: Value,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAuthorizationRelation {
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
pub struct OrderDeliveryRelation {
    pub delivery_id: String,
    pub delivery_type: String,
    pub delivery_route: Option<String>,
    pub current_status: String,
    pub delivery_commit_hash: Option<String>,
    pub receipt_hash: Option<String>,
    pub committed_at: Option<String>,
    pub expires_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBillingRelations {
    pub billing_events: Vec<OrderBillingEventRelation>,
    pub settlements: Vec<OrderSettlementRelation>,
    pub refunds: Vec<OrderRefundRelation>,
    pub compensations: Vec<OrderCompensationRelation>,
    pub invoices: Vec<OrderInvoiceRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBillingEventRelation {
    pub billing_event_id: String,
    pub event_type: String,
    pub event_source: String,
    pub amount: String,
    pub currency_code: String,
    pub units: Option<String>,
    pub occurred_at: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSettlementRelation {
    pub settlement_id: String,
    pub settlement_type: String,
    pub settlement_status: String,
    pub settlement_mode: String,
    pub payable_amount: String,
    pub refund_amount: String,
    pub compensation_amount: String,
    pub reason_code: Option<String>,
    pub settled_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRefundRelation {
    pub refund_id: String,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub executed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCompensationRelation {
    pub compensation_id: String,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub executed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderInvoiceRelation {
    pub invoice_request_id: String,
    pub settlement_id: Option<String>,
    pub requester_org_id: String,
    pub invoice_title: String,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDisputeRelation {
    pub case_id: String,
    pub complainant_type: String,
    pub complainant_id: String,
    pub reason_code: String,
    pub current_status: String,
    pub decision_code: Option<String>,
    pub penalty_code: Option<String>,
    pub evidence_count: i64,
    pub opened_at: String,
    pub resolved_at: Option<String>,
    pub updated_at: String,
}
