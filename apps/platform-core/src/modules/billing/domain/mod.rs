mod api_billing_basis;

use serde::Serialize;
use serde_json::Value;

pub use api_billing_basis::{
    API_PPU_BILLING_RULE, API_SUB_BILLING_RULE, ApiBillingBasisRule, ApiBillingBasisView,
    api_billing_basis_rule_for_sku,
};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct JurisdictionProfile {
    pub jurisdiction_code: String,
    pub jurisdiction_name: String,
    pub regulator_name: Option<String>,
    pub launch_phase: String,
    pub supports_fiat_collection: bool,
    pub supports_fiat_payout: bool,
    pub supports_crypto_settlement: bool,
    pub jurisdiction_status: String,
    pub policy_snapshot: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CorridorPolicy {
    pub corridor_policy_id: String,
    pub policy_name: String,
    pub payer_jurisdiction_code: String,
    pub payee_jurisdiction_code: String,
    pub product_scope: String,
    pub price_currency_code: String,
    pub allowed_collection_currencies: Vec<String>,
    pub allowed_payout_currencies: Vec<String>,
    pub route_mode: String,
    pub requires_manual_review: bool,
    pub allows_crypto: bool,
    pub corridor_status: String,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub policy_snapshot: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PayoutPreference {
    pub payout_preference_id: String,
    pub beneficiary_subject_type: String,
    pub beneficiary_subject_id: String,
    pub destination_jurisdiction_code: String,
    pub preferred_currency_code: String,
    pub payout_method: String,
    pub preferred_provider_key: String,
    pub preferred_provider_account_id: Option<String>,
    pub beneficiary_snapshot: Value,
    pub is_default: bool,
    pub preference_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BillingEvent {
    pub billing_event_id: String,
    pub order_id: String,
    pub event_type: String,
    pub event_source: String,
    pub amount: String,
    pub currency_code: String,
    pub units: Option<String>,
    pub occurred_at: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Settlement {
    pub settlement_id: String,
    pub settlement_type: String,
    pub settlement_status: String,
    pub settlement_mode: String,
    pub payable_amount: String,
    pub platform_fee_amount: String,
    pub channel_fee_amount: String,
    pub net_receivable_amount: String,
    pub refund_amount: String,
    pub compensation_amount: String,
    pub reason_code: Option<String>,
    pub settled_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SettlementSummary {
    pub gross_amount: String,
    pub platform_commission_amount: String,
    pub channel_fee_amount: String,
    pub refund_adjustment_amount: String,
    pub compensation_adjustment_amount: String,
    pub supplier_receivable_amount: String,
    pub summary_state: String,
    pub proof_commit_state: String,
}
