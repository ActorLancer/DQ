CREATE TABLE IF NOT EXISTS ops.monitoring_policy_profile (
  monitoring_policy_profile_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_key text NOT NULL UNIQUE,
  scope_type text NOT NULL DEFAULT 'global',
  scope_id uuid,
  policy_type text NOT NULL DEFAULT 'trade_chain',
  checkpoint_rules jsonb NOT NULL DEFAULT '{}'::jsonb,
  fairness_rules jsonb NOT NULL DEFAULT '{}'::jsonb,
  alert_rules_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  auto_action_rules jsonb NOT NULL DEFAULT '{}'::jsonb,
  enabled boolean NOT NULL DEFAULT true,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.trade_lifecycle_checkpoint (
  trade_lifecycle_checkpoint_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  monitoring_policy_profile_id uuid REFERENCES ops.monitoring_policy_profile(monitoring_policy_profile_id) ON DELETE SET NULL,
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  ref_domain text NOT NULL DEFAULT 'trade',
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  checkpoint_code text NOT NULL,
  lifecycle_stage text NOT NULL,
  checkpoint_status text NOT NULL DEFAULT 'pending',
  expected_by timestamptz,
  occurred_at timestamptz,
  source_type text NOT NULL DEFAULT 'system',
  source_ref_type text,
  source_ref_id uuid,
  request_id text,
  trace_id text,
  related_tx_hash text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.external_fact_receipt (
  external_fact_receipt_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  ref_domain text NOT NULL DEFAULT 'trade',
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  fact_type text NOT NULL,
  provider_type text NOT NULL,
  provider_key text,
  provider_reference text,
  receipt_status text NOT NULL DEFAULT 'pending',
  receipt_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  receipt_hash text,
  occurred_at timestamptz,
  received_at timestamptz NOT NULL DEFAULT now(),
  confirmed_at timestamptz,
  request_id text,
  trace_id text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk.fairness_incident (
  fairness_incident_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  ref_type text NOT NULL,
  ref_id uuid,
  incident_type text NOT NULL,
  severity text NOT NULL DEFAULT 'medium',
  lifecycle_stage text NOT NULL,
  detected_by_type text NOT NULL DEFAULT 'rule_engine',
  source_checkpoint_id uuid REFERENCES ops.trade_lifecycle_checkpoint(trade_lifecycle_checkpoint_id) ON DELETE SET NULL,
  source_receipt_id uuid REFERENCES ops.external_fact_receipt(external_fact_receipt_id) ON DELETE SET NULL,
  status text NOT NULL DEFAULT 'open',
  auto_action_code text,
  assigned_role_key text REFERENCES authz.role_definition(role_key) ON DELETE SET NULL,
  assigned_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  resolution_summary text,
  request_id text,
  trace_id text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  closed_at timestamptz
);

CREATE TABLE IF NOT EXISTS ops.chain_projection_gap (
  chain_projection_gap_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  aggregate_type text NOT NULL,
  aggregate_id uuid,
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE SET NULL,
  chain_id text,
  source_event_type text,
  expected_tx_id text,
  projected_tx_hash text,
  gap_type text NOT NULL,
  gap_status text NOT NULL DEFAULT 'open',
  first_detected_at timestamptz NOT NULL DEFAULT now(),
  last_detected_at timestamptz,
  resolved_at timestamptz,
  request_id text,
  trace_id text,
  outbox_event_id uuid REFERENCES ops.outbox_event(outbox_event_id) ON DELETE SET NULL,
  anchor_id uuid REFERENCES chain.chain_anchor(chain_anchor_id) ON DELETE SET NULL,
  resolution_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_trade_lifecycle_checkpoint_order
  ON ops.trade_lifecycle_checkpoint(order_id, checkpoint_status, expected_by);
CREATE INDEX IF NOT EXISTS idx_trade_lifecycle_checkpoint_ref
  ON ops.trade_lifecycle_checkpoint(ref_type, ref_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_trade_lifecycle_checkpoint_trace
  ON ops.trade_lifecycle_checkpoint(trace_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_external_fact_receipt_order
  ON ops.external_fact_receipt(order_id, receipt_status, received_at DESC);
CREATE INDEX IF NOT EXISTS idx_external_fact_receipt_ref
  ON ops.external_fact_receipt(ref_type, ref_id, received_at DESC);
CREATE INDEX IF NOT EXISTS idx_external_fact_receipt_trace
  ON ops.external_fact_receipt(trace_id, received_at DESC);

CREATE INDEX IF NOT EXISTS idx_fairness_incident_order
  ON risk.fairness_incident(order_id, status, severity, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_fairness_incident_ref
  ON risk.fairness_incident(ref_type, ref_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_fairness_incident_trace
  ON risk.fairness_incident(trace_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_chain_projection_gap_status
  ON ops.chain_projection_gap(gap_status, first_detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_chain_projection_gap_order
  ON ops.chain_projection_gap(order_id, gap_status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chain_projection_gap_outbox
  ON ops.chain_projection_gap(outbox_event_id, created_at DESC);

CREATE TRIGGER trg_monitoring_policy_profile_updated_at BEFORE UPDATE ON ops.monitoring_policy_profile
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_trade_lifecycle_checkpoint_updated_at BEFORE UPDATE ON ops.trade_lifecycle_checkpoint
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_external_fact_receipt_updated_at BEFORE UPDATE ON ops.external_fact_receipt
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_fairness_incident_updated_at BEFORE UPDATE ON risk.fairness_incident
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_chain_projection_gap_updated_at BEFORE UPDATE ON ops.chain_projection_gap
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
