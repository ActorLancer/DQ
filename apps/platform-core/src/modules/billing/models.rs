use crate::modules::billing::domain::{CorridorPolicy, JurisdictionProfile};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BillingPolicyView {
    pub jurisdictions: Vec<JurisdictionProfile>,
    pub corridor_policies: Vec<CorridorPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateJurisdictionProfileRequest {
    pub jurisdiction_code: String,
    pub jurisdiction_name: String,
    pub regulator_name: Option<String>,
    pub launch_phase: String,
    pub supports_fiat_collection: bool,
    pub supports_fiat_payout: bool,
    pub supports_crypto_settlement: bool,
    pub jurisdiction_status: String,
    #[serde(default = "default_json_object")]
    pub policy_snapshot: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCorridorPolicyRequest {
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
    #[serde(default = "default_json_object")]
    pub policy_snapshot: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePayoutPreferenceRequest {
    pub beneficiary_subject_type: String,
    pub beneficiary_subject_id: String,
    pub destination_jurisdiction_code: String,
    pub preferred_currency_code: String,
    pub payout_method: String,
    pub preferred_provider_key: String,
    pub preferred_provider_account_id: Option<String>,
    #[serde(default = "default_json_object")]
    pub beneficiary_snapshot: Value,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListPayoutPreferenceQuery {
    pub beneficiary_subject_type: Option<String>,
    pub beneficiary_subject_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub order_id: String,
    pub provider_key: String,
    pub provider_account_id: Option<String>,
    pub payer_subject_type: String,
    pub payer_subject_id: String,
    pub payee_subject_type: Option<String>,
    pub payee_subject_id: Option<String>,
    #[serde(alias = "amount")]
    pub payment_amount: String,
    pub payment_method: String,
    pub currency_code: Option<String>,
    pub price_currency_code: Option<String>,
    pub intent_type: Option<String>,
    pub payer_jurisdiction_code: Option<String>,
    pub payee_jurisdiction_code: Option<String>,
    pub launch_jurisdiction_code: Option<String>,
    pub corridor_policy_id: Option<String>,
    pub fee_preview_id: Option<String>,
    pub expire_at: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaymentIntentView {
    pub payment_intent_id: String,
    pub order_id: String,
    pub intent_type: String,
    pub provider_key: String,
    pub provider_account_id: Option<String>,
    pub payer_subject_type: String,
    pub payer_subject_id: String,
    pub payee_subject_type: Option<String>,
    pub payee_subject_id: Option<String>,
    pub payer_jurisdiction_code: Option<String>,
    pub payee_jurisdiction_code: Option<String>,
    pub launch_jurisdiction_code: String,
    pub corridor_policy_id: Option<String>,
    pub fee_preview_id: Option<String>,
    pub payment_amount: String,
    pub payment_method: String,
    pub currency_code: String,
    pub price_currency_code: String,
    pub payment_status: String,
    pub provider_intent_no: Option<String>,
    pub channel_reference_no: Option<String>,
    pub idempotency_key: Option<String>,
    pub request_id: Option<String>,
    pub expire_at: Option<String>,
    pub capability_snapshot: Value,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaymentTransactionSummaryView {
    pub payment_transaction_id: String,
    pub transaction_type: String,
    pub provider_transaction_no: Option<String>,
    pub provider_status: Option<String>,
    pub transaction_amount: String,
    pub currency_code: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaymentWebhookSummaryView {
    pub webhook_event_id: String,
    pub provider_event_id: String,
    pub event_type: String,
    pub processed_status: String,
    pub duplicate_flag: bool,
    pub signature_verified: bool,
    pub received_at: String,
    pub processed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaymentIntentDetailView {
    pub payment_intent: PaymentIntentView,
    pub latest_transaction_summary: Option<PaymentTransactionSummaryView>,
    pub webhook_summary: Option<PaymentWebhookSummaryView>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LockOrderRequest {
    pub payment_intent_id: String,
    pub lock_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OrderLockView {
    pub order_id: String,
    pub payment_intent_id: String,
    pub order_status: String,
    pub payment_status: String,
    pub buyer_locked_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentWebhookRequest {
    pub provider_event_id: String,
    pub event_type: String,
    pub provider_transaction_no: Option<String>,
    pub payment_intent_id: Option<String>,
    pub transaction_amount: Option<String>,
    pub currency_code: Option<String>,
    pub provider_status: Option<String>,
    pub occurred_at: Option<String>,
    pub occurred_at_ms: Option<i64>,
    #[serde(default, alias = "payload")]
    pub raw_payload: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaymentWebhookResultView {
    pub webhook_event_id: String,
    pub provider_key: String,
    pub provider_event_id: String,
    pub processed_status: String,
    pub duplicate: bool,
    pub signature_verified: bool,
    pub out_of_order_ignored: bool,
    pub payment_intent_id: Option<String>,
    pub payment_transaction_id: Option<String>,
    pub applied_payment_status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MockPaymentSimulationRequest {
    pub delay_seconds: Option<i32>,
    pub duplicate_webhook: Option<bool>,
    pub partial_refund_amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MockPaymentSimulationView {
    pub mock_payment_case_id: String,
    pub payment_intent_id: String,
    pub scenario_type: String,
    pub provider_key: String,
    pub provider_kind: String,
    pub provider_event_id: String,
    pub provider_status: String,
    pub http_status_code: Option<u16>,
    pub webhook_processed_status: String,
    pub duplicate_webhook: bool,
    pub duplicate_processed_status: Option<String>,
    pub payment_transaction_id: Option<String>,
    pub applied_payment_status: Option<String>,
}

fn default_json_object() -> Value {
    Value::Object(Default::default())
}
