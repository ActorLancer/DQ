use crate::modules::billing::domain::{CorridorPolicy, JurisdictionProfile};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingPolicyView {
    pub jurisdictions: Vec<JurisdictionProfile>,
    pub corridor_policies: Vec<CorridorPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub order_id: String,
    pub provider_key: String,
    pub payer_subject_type: String,
    pub payer_subject_id: String,
    pub payee_subject_type: Option<String>,
    pub payee_subject_id: Option<String>,
    pub amount: String,
    pub payment_method: String,
    pub currency_code: Option<String>,
    pub price_currency_code: Option<String>,
    pub intent_type: Option<String>,
    pub payer_jurisdiction_code: Option<String>,
    pub payee_jurisdiction_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaymentIntentView {
    pub payment_intent_id: String,
    pub order_id: String,
    pub intent_type: String,
    pub provider_key: String,
    pub payer_subject_type: String,
    pub payer_subject_id: String,
    pub payee_subject_type: Option<String>,
    pub payee_subject_id: Option<String>,
    pub amount: String,
    pub payment_method: String,
    pub currency_code: String,
    pub price_currency_code: String,
    pub status: String,
    pub idempotency_key: Option<String>,
    pub request_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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
    pub payment_intent_id: Option<String>,
    pub provider_status: Option<String>,
    pub occurred_at_ms: Option<i64>,
    #[serde(default)]
    pub payload: Value,
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
    pub applied_payment_status: Option<String>,
}
