CREATE TABLE IF NOT EXISTS billing.token_asset (
  token_code text PRIMARY KEY,
  token_name text NOT NULL,
  token_type text NOT NULL,
  decimals integer NOT NULL DEFAULT 8,
  chain_id text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO billing.token_asset (token_code, token_name, token_type, decimals)
VALUES
  ('CNY_SETTLEMENT', 'CNY Settlement Unit', 'fiat_virtual', 2),
  ('PLATFORM_STAKE', 'Platform Stake Unit', 'internal', 8)
ON CONFLICT (token_code) DO NOTHING;

CREATE TABLE IF NOT EXISTS billing.fee_rule (
  fee_rule_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  rule_name text NOT NULL,
  fee_domain text NOT NULL,
  scope_type text NOT NULL,
  scope_id uuid,
  calculation_method text NOT NULL,
  currency_code text NOT NULL DEFAULT 'CNY',
  rule_status text NOT NULL DEFAULT 'draft',
  created_by uuid REFERENCES core.user_account(user_id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.fee_rule_version (
  fee_rule_version_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  fee_rule_id uuid NOT NULL REFERENCES billing.fee_rule(fee_rule_id) ON DELETE CASCADE,
  version_no integer NOT NULL DEFAULT 1,
  parameter_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  effective_from timestamptz,
  effective_to timestamptz,
  is_current boolean NOT NULL DEFAULT false,
  published_by uuid REFERENCES core.user_account(user_id),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (fee_rule_id, version_no)
);

CREATE TABLE IF NOT EXISTS billing.fee_preview (
  fee_preview_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  product_id uuid REFERENCES catalog.product(product_id) ON DELETE SET NULL,
  sku_id uuid REFERENCES catalog.product_sku(sku_id) ON DELETE SET NULL,
  fee_rule_id uuid REFERENCES billing.fee_rule(fee_rule_id) ON DELETE SET NULL,
  preview_scope text NOT NULL,
  amount_before_fee numeric(24, 8) NOT NULL DEFAULT 0,
  platform_fee_amount numeric(24, 8) NOT NULL DEFAULT 0,
  channel_fee_amount numeric(24, 8) NOT NULL DEFAULT 0,
  deposit_amount numeric(24, 8) NOT NULL DEFAULT 0,
  value_added_fee_amount numeric(24, 8) NOT NULL DEFAULT 0,
  payable_total_amount numeric(24, 8) NOT NULL DEFAULT 0,
  currency_code text NOT NULL DEFAULT 'CNY',
  fee_snapshot_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.wallet_account (
  wallet_account_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subject_type text NOT NULL,
  subject_id uuid NOT NULL,
  token_code text NOT NULL REFERENCES billing.token_asset(token_code),
  available_balance numeric(24, 8) NOT NULL DEFAULT 0,
  locked_balance numeric(24, 8) NOT NULL DEFAULT 0,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (subject_type, subject_id, token_code)
);

CREATE TABLE IF NOT EXISTS billing.account_ledger_entry (
  account_ledger_entry_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  wallet_account_id uuid NOT NULL REFERENCES billing.wallet_account(wallet_account_id) ON DELETE CASCADE,
  entry_type text NOT NULL,
  direction text NOT NULL,
  reference_type text,
  reference_id uuid,
  amount numeric(24, 8) NOT NULL,
  available_after numeric(24, 8),
  locked_after numeric(24, 8),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.escrow_ledger (
  escrow_ledger_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  token_code text NOT NULL REFERENCES billing.token_asset(token_code),
  lock_amount numeric(24, 8) NOT NULL DEFAULT 0,
  buyer_deposit_amount numeric(24, 8) NOT NULL DEFAULT 0,
  seller_deposit_amount numeric(24, 8) NOT NULL DEFAULT 0,
  status text NOT NULL DEFAULT 'pending_lock',
  locked_at timestamptz,
  released_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.deposit_record (
  deposit_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  subject_type text NOT NULL,
  subject_id uuid NOT NULL,
  deposit_type text NOT NULL,
  token_code text NOT NULL REFERENCES billing.token_asset(token_code),
  amount numeric(24, 8) NOT NULL,
  status text NOT NULL DEFAULT 'pending_lock',
  wallet_account_id uuid REFERENCES billing.wallet_account(wallet_account_id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.billing_event (
  billing_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  event_type text NOT NULL,
  event_source text NOT NULL,
  amount numeric(24, 8) NOT NULL DEFAULT 0,
  currency_code text NOT NULL DEFAULT 'CNY',
  units numeric(24, 8),
  occurred_at timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS billing.settlement_record (
  settlement_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  settlement_type text NOT NULL,
  settlement_status text NOT NULL DEFAULT 'pending',
  settlement_mode text NOT NULL DEFAULT 'manual',
  payable_amount numeric(24, 8) NOT NULL DEFAULT 0,
  platform_fee_amount numeric(24, 8) NOT NULL DEFAULT 0,
  channel_fee_amount numeric(24, 8) NOT NULL DEFAULT 0,
  net_receivable_amount numeric(24, 8) NOT NULL DEFAULT 0,
  refund_amount numeric(24, 8) NOT NULL DEFAULT 0,
  compensation_amount numeric(24, 8) NOT NULL DEFAULT 0,
  reason_code text,
  settled_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.penalty_event (
  penalty_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  subject_type text NOT NULL,
  subject_id uuid NOT NULL,
  penalty_type text NOT NULL,
  amount numeric(24, 8) NOT NULL DEFAULT 0,
  token_code text NOT NULL REFERENCES billing.token_asset(token_code),
  reason_code text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.refund_record (
  refund_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  amount numeric(24, 8) NOT NULL,
  currency_code text NOT NULL DEFAULT 'CNY',
  status text NOT NULL DEFAULT 'pending',
  executed_by uuid REFERENCES core.user_account(user_id),
  executed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.compensation_record (
  compensation_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  amount numeric(24, 8) NOT NULL,
  currency_code text NOT NULL DEFAULT 'CNY',
  status text NOT NULL DEFAULT 'pending',
  executed_by uuid REFERENCES core.user_account(user_id),
  executed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.invoice_request (
  invoice_request_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE SET NULL,
  settlement_id uuid REFERENCES billing.settlement_record(settlement_id) ON DELETE SET NULL,
  requester_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  invoice_title text NOT NULL,
  tax_no text,
  amount numeric(24, 8) NOT NULL,
  currency_code text NOT NULL DEFAULT 'CNY',
  status text NOT NULL DEFAULT 'pending',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment.provider (
  provider_key text PRIMARY KEY,
  provider_name text NOT NULL,
  provider_type text NOT NULL,
  settlement_category text NOT NULL,
  supports_sandbox boolean NOT NULL DEFAULT false,
  supports_payin boolean NOT NULL DEFAULT true,
  supports_refund boolean NOT NULL DEFAULT true,
  supports_payout boolean NOT NULL DEFAULT false,
  supports_split boolean NOT NULL DEFAULT false,
  supports_webhook boolean NOT NULL DEFAULT true,
  supports_recurring boolean NOT NULL DEFAULT false,
  supports_multi_currency boolean NOT NULL DEFAULT false,
  status text NOT NULL DEFAULT 'active',
  config_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO payment.provider (
  provider_key, provider_name, provider_type, settlement_category, supports_sandbox,
  supports_payin, supports_refund, supports_payout, supports_split, supports_webhook,
  supports_recurring, supports_multi_currency
) VALUES
  ('mock_payment', 'Mock Payment Provider', 'mock', 'internal', true, true, true, true, true, true, true, true),
  ('offline_bank', 'Offline Bank Transfer', 'bank_transfer', 'domestic', false, true, true, true, false, false, false, false),
  ('alipay', 'Alipay', 'wallet_qr', 'domestic', true, true, true, true, false, true, true, false),
  ('wechat_pay', 'WeChat Pay', 'wallet_qr', 'domestic', true, true, true, true, false, true, true, false),
  ('unionpay', 'UnionPay', 'bank_card', 'domestic', true, true, true, true, false, true, false, false),
  ('paypal', 'PayPal', 'international_wallet', 'international', true, true, true, true, false, true, true, true)
ON CONFLICT (provider_key) DO NOTHING;

CREATE TABLE IF NOT EXISTS payment.provider_account (
  provider_account_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  provider_key text NOT NULL REFERENCES payment.provider(provider_key),
  account_scope text NOT NULL,
  account_scope_id uuid,
  account_name text NOT NULL,
  merchant_id text,
  sub_merchant_id text,
  settlement_subject_type text,
  settlement_subject_id uuid,
  account_mode text NOT NULL DEFAULT 'production',
  status text NOT NULL DEFAULT 'active',
  config_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider_key, account_scope, account_scope_id, account_name)
);

CREATE TABLE IF NOT EXISTS payment.payment_intent (
  payment_intent_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  intent_type text NOT NULL,
  provider_key text NOT NULL REFERENCES payment.provider(provider_key),
  provider_account_id uuid REFERENCES payment.provider_account(provider_account_id) ON DELETE SET NULL,
  payer_subject_type text NOT NULL,
  payer_subject_id uuid NOT NULL,
  payee_subject_type text,
  payee_subject_id uuid,
  fee_preview_id uuid REFERENCES billing.fee_preview(fee_preview_id) ON DELETE SET NULL,
  amount numeric(24, 8) NOT NULL,
  currency_code text NOT NULL DEFAULT 'CNY',
  payment_method text NOT NULL,
  status text NOT NULL DEFAULT 'created',
  provider_intent_no text,
  channel_reference_no text,
  request_id text,
  idempotency_key text,
  expire_at timestamptz,
  capability_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (idempotency_key)
);

CREATE TABLE IF NOT EXISTS payment.payment_transaction (
  payment_transaction_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  payment_intent_id uuid NOT NULL REFERENCES payment.payment_intent(payment_intent_id) ON DELETE CASCADE,
  transaction_type text NOT NULL,
  direction text NOT NULL,
  provider_transaction_no text,
  provider_status text,
  amount numeric(24, 8) NOT NULL DEFAULT 0,
  currency_code text NOT NULL DEFAULT 'CNY',
  channel_fee_amount numeric(24, 8) NOT NULL DEFAULT 0,
  settled_amount numeric(24, 8) NOT NULL DEFAULT 0,
  occurred_at timestamptz NOT NULL DEFAULT now(),
  raw_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment.payment_webhook_event (
  webhook_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  provider_key text NOT NULL REFERENCES payment.provider(provider_key),
  provider_event_id text NOT NULL,
  event_type text NOT NULL,
  signature_verified boolean NOT NULL DEFAULT false,
  payment_intent_id uuid REFERENCES payment.payment_intent(payment_intent_id) ON DELETE SET NULL,
  payment_transaction_id uuid REFERENCES payment.payment_transaction(payment_transaction_id) ON DELETE SET NULL,
  payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  processed_status text NOT NULL DEFAULT 'pending',
  duplicate_flag boolean NOT NULL DEFAULT false,
  received_at timestamptz NOT NULL DEFAULT now(),
  processed_at timestamptz,
  UNIQUE (provider_key, provider_event_id)
);

CREATE TABLE IF NOT EXISTS payment.refund_intent (
  refund_intent_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  refund_id uuid REFERENCES billing.refund_record(refund_id) ON DELETE SET NULL,
  payment_intent_id uuid REFERENCES payment.payment_intent(payment_intent_id) ON DELETE SET NULL,
  provider_key text NOT NULL REFERENCES payment.provider(provider_key),
  provider_account_id uuid REFERENCES payment.provider_account(provider_account_id) ON DELETE SET NULL,
  amount numeric(24, 8) NOT NULL,
  currency_code text NOT NULL DEFAULT 'CNY',
  status text NOT NULL DEFAULT 'pending',
  provider_refund_no text,
  reason_code text,
  executed_by uuid REFERENCES core.user_account(user_id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment.payout_instruction (
  payout_instruction_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id uuid REFERENCES billing.settlement_record(settlement_id) ON DELETE SET NULL,
  provider_key text NOT NULL REFERENCES payment.provider(provider_key),
  provider_account_id uuid REFERENCES payment.provider_account(provider_account_id) ON DELETE SET NULL,
  beneficiary_subject_type text NOT NULL,
  beneficiary_subject_id uuid NOT NULL,
  amount numeric(24, 8) NOT NULL,
  currency_code text NOT NULL DEFAULT 'CNY',
  payout_mode text NOT NULL DEFAULT 'manual',
  status text NOT NULL DEFAULT 'pending_review',
  provider_payout_no text,
  reviewed_by uuid REFERENCES core.user_account(user_id),
  executed_by uuid REFERENCES core.user_account(user_id),
  executed_at timestamptz,
  idempotency_key text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (idempotency_key)
);

CREATE TABLE IF NOT EXISTS payment.reconciliation_statement (
  reconciliation_statement_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  provider_key text NOT NULL REFERENCES payment.provider(provider_key),
  provider_account_id uuid REFERENCES payment.provider_account(provider_account_id) ON DELETE SET NULL,
  statement_date date NOT NULL,
  statement_type text NOT NULL,
  file_uri text,
  file_hash text,
  import_status text NOT NULL DEFAULT 'pending_import',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider_key, provider_account_id, statement_date, statement_type)
);

CREATE TABLE IF NOT EXISTS payment.reconciliation_diff (
  reconciliation_diff_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  reconciliation_statement_id uuid NOT NULL REFERENCES payment.reconciliation_statement(reconciliation_statement_id) ON DELETE CASCADE,
  diff_type text NOT NULL,
  ref_type text,
  ref_id uuid,
  provider_reference_no text,
  internal_amount numeric(24, 8),
  provider_amount numeric(24, 8),
  diff_status text NOT NULL DEFAULT 'open',
  resolution_note text,
  resolved_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS support.dispute_case (
  case_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  complainant_type text NOT NULL,
  complainant_id uuid NOT NULL,
  reason_code text NOT NULL,
  status text NOT NULL DEFAULT 'opened',
  decision_code text,
  penalty_code text,
  opened_at timestamptz NOT NULL DEFAULT now(),
  resolved_at timestamptz,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS support.dispute_status_history (
  dispute_status_history_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  case_id uuid NOT NULL REFERENCES support.dispute_case(case_id) ON DELETE CASCADE,
  old_status text,
  new_status text NOT NULL,
  changed_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS support.evidence_object (
  evidence_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  case_id uuid REFERENCES support.dispute_case(case_id) ON DELETE CASCADE,
  object_type text NOT NULL,
  object_uri text,
  object_hash text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS support.decision_record (
  decision_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  case_id uuid NOT NULL UNIQUE REFERENCES support.dispute_case(case_id) ON DELETE CASCADE,
  decision_type text NOT NULL,
  decision_code text NOT NULL,
  liability_type text,
  decision_text text,
  decided_by uuid REFERENCES core.user_account(user_id),
  decided_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk.rating_record (
  rating_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  rater_id uuid,
  target_id uuid,
  score numeric(5, 2) NOT NULL,
  weight numeric(8, 4) NOT NULL DEFAULT 1,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk.reputation_snapshot (
  reputation_snapshot_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subject_type text NOT NULL,
  subject_id uuid NOT NULL,
  score numeric(10, 4) NOT NULL DEFAULT 0,
  risk_level integer NOT NULL DEFAULT 0,
  credit_level integer NOT NULL DEFAULT 0,
  effective_at timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS risk.blacklist_entry (
  blacklist_entry_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subject_type text NOT NULL,
  subject_id uuid NOT NULL,
  reason_code text NOT NULL,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  released_at timestamptz
);

CREATE INDEX IF NOT EXISTS idx_billing_event_order_id ON billing.billing_event(order_id);
CREATE INDEX IF NOT EXISTS idx_fee_preview_order_id ON billing.fee_preview(order_id);
CREATE INDEX IF NOT EXISTS idx_dispute_case_order_id ON support.dispute_case(order_id);
CREATE INDEX IF NOT EXISTS idx_payment_intent_order_id ON payment.payment_intent(order_id);
CREATE INDEX IF NOT EXISTS idx_payment_intent_status ON payment.payment_intent(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_payment_transaction_intent_id ON payment.payment_transaction(payment_intent_id);
CREATE INDEX IF NOT EXISTS idx_payout_instruction_settlement_id ON payment.payout_instruction(settlement_id);
CREATE INDEX IF NOT EXISTS idx_reputation_snapshot_subject ON risk.reputation_snapshot(subject_type, subject_id, effective_at DESC);

CREATE TRIGGER trg_fee_rule_updated_at BEFORE UPDATE ON billing.fee_rule
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_wallet_account_updated_at BEFORE UPDATE ON billing.wallet_account
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_escrow_ledger_updated_at BEFORE UPDATE ON billing.escrow_ledger
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_deposit_record_updated_at BEFORE UPDATE ON billing.deposit_record
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_settlement_record_updated_at BEFORE UPDATE ON billing.settlement_record
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_refund_record_updated_at BEFORE UPDATE ON billing.refund_record
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_compensation_record_updated_at BEFORE UPDATE ON billing.compensation_record
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_invoice_request_updated_at BEFORE UPDATE ON billing.invoice_request
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_payment_provider_updated_at BEFORE UPDATE ON payment.provider
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_payment_provider_account_updated_at BEFORE UPDATE ON payment.provider_account
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_payment_intent_updated_at BEFORE UPDATE ON payment.payment_intent
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_refund_intent_updated_at BEFORE UPDATE ON payment.refund_intent
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_payout_instruction_updated_at BEFORE UPDATE ON payment.payout_instruction
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_reconciliation_statement_updated_at BEFORE UPDATE ON payment.reconciliation_statement
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_reconciliation_diff_updated_at BEFORE UPDATE ON payment.reconciliation_diff
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_dispute_case_updated_at BEFORE UPDATE ON support.dispute_case
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_dispute_status_history AFTER INSERT OR UPDATE ON support.dispute_case
FOR EACH ROW EXECUTE FUNCTION common.tg_dispute_status_history();
-- Trust-boundary baseline sync: billing/support/risk schema remains valid; storage-trust disputes are represented through existing dispute/evidence objects.
-- Payment settlement sync: V1 adds fee rule, payment intent, payout and reconciliation objects while preserving manual settlement as the initial execution baseline.
