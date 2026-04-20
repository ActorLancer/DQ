use serde::Serialize;
use serde_json::Value;

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
