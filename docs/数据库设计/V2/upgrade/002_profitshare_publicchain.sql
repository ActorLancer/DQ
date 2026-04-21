CREATE TABLE IF NOT EXISTS billing.profit_share_rule (
  profit_share_rule_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  rule_name text NOT NULL,
  version_no integer NOT NULL DEFAULT 1,
  scope_type text NOT NULL,
  scope_id uuid,
  rule_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.reward_pool (
  reward_pool_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  token_code text NOT NULL REFERENCES billing.token_asset(token_code),
  total_amount numeric(24, 8) NOT NULL DEFAULT 0,
  locked_amount numeric(24, 8) NOT NULL DEFAULT 0,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.contribution_record (
  contribution_record_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id uuid REFERENCES ml.training_task(task_id) ON DELETE CASCADE,
  participant_id uuid REFERENCES ml.task_participant(task_participant_id) ON DELETE CASCADE,
  relative_metric_gain numeric(12, 6) NOT NULL DEFAULT 0,
  data_quality_score numeric(12, 6) NOT NULL DEFAULT 0,
  availability_score numeric(12, 6) NOT NULL DEFAULT 0,
  timeliness_score numeric(12, 6) NOT NULL DEFAULT 0,
  protocol_compliance_score numeric(12, 6) NOT NULL DEFAULT 0,
  anomaly_penalty numeric(12, 6) NOT NULL DEFAULT 0,
  contribution_score numeric(12, 6) NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (task_id, participant_id)
);

CREATE TABLE IF NOT EXISTS billing.reward_record (
  reward_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  reward_pool_id uuid REFERENCES billing.reward_pool(reward_pool_id) ON DELETE SET NULL,
  task_id uuid REFERENCES ml.training_task(task_id) ON DELETE SET NULL,
  settlement_rule_id uuid REFERENCES billing.profit_share_rule(profit_share_rule_id),
  beneficiary_type text NOT NULL,
  beneficiary_id uuid NOT NULL,
  contribution_record_id uuid REFERENCES billing.contribution_record(contribution_record_id),
  amount numeric(24, 8) NOT NULL DEFAULT 0,
  token_code text NOT NULL REFERENCES billing.token_asset(token_code),
  status text NOT NULL DEFAULT 'pending',
  risk_status text NOT NULL DEFAULT 'clean',
  reviewed_by uuid REFERENCES core.user_account(user_id),
  reviewed_at timestamptz,
  settled_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment.sub_merchant_binding (
  sub_merchant_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  provider_account_id uuid NOT NULL REFERENCES payment.provider_account(provider_account_id) ON DELETE CASCADE,
  beneficiary_type text NOT NULL,
  beneficiary_id uuid NOT NULL,
  external_sub_merchant_id text NOT NULL,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider_account_id, beneficiary_type, beneficiary_id)
);

CREATE TABLE IF NOT EXISTS payment.split_instruction (
  split_instruction_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  reward_id uuid REFERENCES billing.reward_record(reward_id) ON DELETE SET NULL,
  settlement_id uuid REFERENCES billing.settlement_record(settlement_id) ON DELETE SET NULL,
  provider_account_id uuid REFERENCES payment.provider_account(provider_account_id) ON DELETE SET NULL,
  sub_merchant_binding_id uuid REFERENCES payment.sub_merchant_binding(sub_merchant_binding_id) ON DELETE SET NULL,
  split_mode text NOT NULL DEFAULT 'platform_ledger_then_payout',
  amount numeric(24, 8) NOT NULL DEFAULT 0,
  currency_code text NOT NULL DEFAULT 'CNY',
  status text NOT NULL DEFAULT 'pending',
  provider_split_no text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment.recurring_charge_plan (
  recurring_charge_plan_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  provider_account_id uuid REFERENCES payment.provider_account(provider_account_id) ON DELETE SET NULL,
  billing_cycle text NOT NULL,
  next_charge_at timestamptz,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS chain.public_anchor_batch (
  public_anchor_batch_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  chain_id text NOT NULL,
  batch_type text NOT NULL,
  status text NOT NULL DEFAULT 'pending',
  object_count integer NOT NULL DEFAULT 0,
  tx_hash text,
  anchored_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS chain.credential_token (
  credential_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  credential_type text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  issuer_service_identity_id uuid REFERENCES core.service_identity(service_identity_id),
  chain_id text,
  token_uri text,
  digest text,
  visibility_status text NOT NULL DEFAULT 'visible',
  status text NOT NULL DEFAULT 'issued',
  issued_at timestamptz NOT NULL DEFAULT now(),
  revoked_at timestamptz
);

CREATE TABLE IF NOT EXISTS chain.credential_status_history (
  credential_status_history_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  credential_id uuid NOT NULL REFERENCES chain.credential_token(credential_id) ON DELETE CASCADE,
  old_status text,
  new_status text NOT NULL,
  changed_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_reward_record_task_id ON billing.reward_record(task_id);
CREATE INDEX IF NOT EXISTS idx_reward_record_beneficiary ON billing.reward_record(beneficiary_type, beneficiary_id);

DROP TRIGGER IF EXISTS trg_profit_share_rule_updated_at ON billing.profit_share_rule;
CREATE TRIGGER trg_profit_share_rule_updated_at BEFORE UPDATE ON billing.profit_share_rule
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
DROP TRIGGER IF EXISTS trg_reward_pool_updated_at ON billing.reward_pool;
CREATE TRIGGER trg_reward_pool_updated_at BEFORE UPDATE ON billing.reward_pool
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
DROP TRIGGER IF EXISTS trg_reward_record_updated_at ON billing.reward_record;
CREATE TRIGGER trg_reward_record_updated_at BEFORE UPDATE ON billing.reward_record
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
DROP TRIGGER IF EXISTS trg_sub_merchant_binding_updated_at ON payment.sub_merchant_binding;
CREATE TRIGGER trg_sub_merchant_binding_updated_at BEFORE UPDATE ON payment.sub_merchant_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
DROP TRIGGER IF EXISTS trg_split_instruction_updated_at ON payment.split_instruction;
CREATE TRIGGER trg_split_instruction_updated_at BEFORE UPDATE ON payment.split_instruction
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
DROP TRIGGER IF EXISTS trg_recurring_charge_plan_updated_at ON payment.recurring_charge_plan;
CREATE TRIGGER trg_recurring_charge_plan_updated_at BEFORE UPDATE ON payment.recurring_charge_plan
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
-- Trust-boundary baseline sync: V2 trust-boundary impacts are carried by ml and permission models; profitshare/publicchain schema stays structurally compatible.
-- Payment settlement sync: V2 extends payout and profitshare with sub-merchant, split and recurring charge planning.
