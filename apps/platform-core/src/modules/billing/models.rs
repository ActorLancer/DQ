use crate::modules::billing::domain::{
    BillingEvent, CorridorPolicy, JurisdictionProfile, Settlement, SettlementSummary,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BillingPolicyView {
    pub jurisdictions: Vec<JurisdictionProfile>,
    pub corridor_policies: Vec<CorridorPolicy>,
}

pub type BillingEventView = BillingEvent;
pub type BillingSettlementView = Settlement;
pub type BillingSettlementSummaryView = SettlementSummary;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingRefundView {
    pub refund_id: String,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub executed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingCompensationView {
    pub compensation_id: String,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub executed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingPayoutView {
    pub payout_instruction_id: String,
    pub settlement_id: Option<String>,
    pub provider_key: String,
    pub provider_account_id: Option<String>,
    pub payout_preference_id: Option<String>,
    pub beneficiary_subject_type: String,
    pub beneficiary_subject_id: String,
    pub destination_jurisdiction_code: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub payout_mode: String,
    pub current_status: String,
    pub provider_payout_no: Option<String>,
    pub executed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingSplitInstructionView {
    pub split_instruction_id: String,
    pub settlement_id: Option<String>,
    pub reward_id: Option<String>,
    pub provider_account_id: Option<String>,
    pub sub_merchant_binding_id: Option<String>,
    pub split_mode: String,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub provider_split_no: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconciliationImportDiffInput {
    pub diff_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub provider_reference_no: Option<String>,
    pub internal_amount: Option<String>,
    pub provider_amount: Option<String>,
    pub diff_status: Option<String>,
    pub resolution_note: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateReconciliationImportRequest {
    pub provider_key: String,
    pub provider_account_id: String,
    pub statement_date: String,
    pub statement_type: String,
    pub file_name: String,
    pub file_content_type: Option<String>,
    pub file_bytes: Vec<u8>,
    pub diffs: Vec<ReconciliationImportDiffInput>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReconciliationStatementView {
    pub reconciliation_statement_id: String,
    pub provider_key: String,
    pub provider_account_id: Option<String>,
    pub statement_date: String,
    pub statement_type: String,
    pub file_uri: Option<String>,
    pub file_hash: Option<String>,
    pub import_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReconciliationDiffView {
    pub reconciliation_diff_id: String,
    pub reconciliation_statement_id: String,
    pub diff_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub provider_reference_no: Option<String>,
    pub internal_amount: Option<String>,
    pub provider_amount: Option<String>,
    pub diff_status: String,
    pub resolution_note: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReconciliationImportView {
    pub statement: ReconciliationStatementView,
    pub diffs: Vec<ReconciliationDiffView>,
    pub imported_diff_count: usize,
    pub open_diff_count: usize,
    pub idempotent_replay: bool,
    pub step_up_bound: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDisputeCaseRequest {
    pub order_id: String,
    pub reason_code: String,
    pub requested_resolution: Option<String>,
    pub claimed_amount: Option<String>,
    pub evidence_scope: Option<String>,
    pub blocking_effect: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Debug, Clone)]
pub struct UploadDisputeEvidenceRequest {
    pub object_type: String,
    pub file_name: String,
    pub content_type: Option<String>,
    pub file_bytes: Vec<u8>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DisputeCaseView {
    pub case_id: String,
    pub order_id: String,
    pub complainant_type: String,
    pub complainant_id: String,
    pub reason_code: String,
    pub current_status: String,
    pub decision_code: Option<String>,
    pub penalty_code: Option<String>,
    pub opened_at: String,
    pub resolved_at: Option<String>,
    pub updated_at: String,
    pub evidence_count: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DisputeEvidenceView {
    pub evidence_id: String,
    pub case_id: String,
    pub object_type: String,
    pub object_uri: Option<String>,
    pub object_hash: Option<String>,
    pub metadata: Value,
    pub created_at: String,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResolveDisputeCaseRequest {
    pub decision_type: Option<String>,
    pub decision_code: String,
    pub liability_type: Option<String>,
    pub penalty_code: Option<String>,
    pub decision_text: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DisputeResolutionView {
    pub case_id: String,
    pub order_id: String,
    pub current_status: String,
    pub decision_id: String,
    pub decision_type: String,
    pub decision_code: String,
    pub liability_type: Option<String>,
    pub penalty_code: Option<String>,
    pub decision_text: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: String,
    pub resolved_at: Option<String>,
    pub step_up_bound: bool,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingInvoiceView {
    pub invoice_request_id: String,
    pub settlement_id: Option<String>,
    pub requester_org_id: String,
    pub invoice_title: String,
    pub tax_no: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingTaxPlaceholderView {
    pub tax_engine_status: String,
    pub tax_rule_code: String,
    pub currency_code: String,
    pub latest_invoice_title: Option<String>,
    pub latest_tax_no: Option<String>,
    pub tax_breakdown_ready: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingInvoicePlaceholderView {
    pub invoice_mode: String,
    pub invoice_required: bool,
    pub latest_invoice_request_id: Option<String>,
    pub latest_invoice_status: Option<String>,
    pub latest_invoice_title: Option<String>,
    pub pending_invoice_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRefundRequest {
    pub order_id: String,
    pub case_id: String,
    pub decision_code: String,
    pub penalty_code: Option<String>,
    pub amount: String,
    pub currency_code: Option<String>,
    pub reason_code: String,
    pub refund_mode: Option<String>,
    pub refund_template: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RefundExecutionView {
    pub refund_id: String,
    pub order_id: String,
    pub case_id: String,
    pub decision_code: String,
    pub penalty_code: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub provider_key: String,
    pub provider_refund_id: Option<String>,
    pub provider_status: Option<String>,
    pub step_up_bound: bool,
    pub idempotent_replay: bool,
    pub executed_at: Option<String>,
    pub updated_at: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCompensationRequest {
    pub order_id: String,
    pub case_id: String,
    pub decision_code: String,
    pub penalty_code: Option<String>,
    pub amount: String,
    pub currency_code: Option<String>,
    pub reason_code: String,
    pub compensation_mode: Option<String>,
    pub compensation_template: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CompensationExecutionView {
    pub compensation_id: String,
    pub order_id: String,
    pub case_id: String,
    pub decision_code: String,
    pub penalty_code: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub current_status: String,
    pub provider_key: String,
    pub provider_transfer_id: Option<String>,
    pub provider_status: Option<String>,
    pub step_up_bound: bool,
    pub idempotent_replay: bool,
    pub executed_at: Option<String>,
    pub updated_at: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateManualPayoutRequest {
    pub order_id: String,
    pub settlement_id: String,
    pub amount: String,
    pub currency_code: Option<String>,
    pub payout_preference_id: Option<String>,
    pub provider_key: Option<String>,
    pub provider_account_id: Option<String>,
    pub payout_mode: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ManualPayoutExecutionView {
    pub payout_instruction_id: String,
    pub order_id: String,
    pub settlement_id: String,
    pub beneficiary_subject_type: String,
    pub beneficiary_subject_id: String,
    pub destination_jurisdiction_code: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub payout_mode: String,
    pub current_status: String,
    pub provider_key: String,
    pub provider_account_id: Option<String>,
    pub payout_preference_id: Option<String>,
    pub provider_payout_no: Option<String>,
    pub step_up_bound: bool,
    pub idempotent_replay: bool,
    pub executed_at: Option<String>,
    pub updated_at: String,
    pub metadata: Value,
    pub split_placeholder: BillingSplitInstructionView,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BillingOrderDetailView {
    pub order_id: String,
    pub order_status: String,
    pub payment_status: String,
    pub settlement_status: String,
    pub dispute_status: String,
    pub order_amount: String,
    pub currency_code: String,
    pub api_billing_basis: Option<ApiBillingBasisView>,
    pub billing_events: Vec<BillingEventView>,
    pub settlements: Vec<BillingSettlementView>,
    pub settlement_summary: Option<BillingSettlementSummaryView>,
    pub refunds: Vec<BillingRefundView>,
    pub compensations: Vec<BillingCompensationView>,
    pub payouts: Vec<BillingPayoutView>,
    pub split_placeholders: Vec<BillingSplitInstructionView>,
    pub invoices: Vec<BillingInvoiceView>,
    pub tax_placeholder: BillingTaxPlaceholderView,
    pub invoice_placeholder: BillingInvoicePlaceholderView,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ApiBillingBasisView {
    pub sku_type: String,
    pub base_event_type: Option<String>,
    pub usage_event_type: Option<String>,
    pub cycle_period: Option<String>,
    pub included_units: Option<String>,
    pub overage_policy: Option<String>,
    pub usage_meter_source: Option<String>,
    pub success_only: bool,
    pub latest_usage_call_count: String,
    pub latest_usage_units: String,
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
