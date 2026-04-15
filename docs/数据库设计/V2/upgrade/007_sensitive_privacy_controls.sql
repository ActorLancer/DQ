CREATE TABLE IF NOT EXISTS delivery.privacy_budget_ledger (
  privacy_budget_ledger_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  query_surface_id uuid REFERENCES catalog.query_surface_definition(query_surface_id) ON DELETE SET NULL,
  task_id uuid REFERENCES ml.compute_task(task_id) ON DELETE SET NULL,
  query_run_id uuid REFERENCES delivery.query_execution_run(query_run_id) ON DELETE SET NULL,
  budget_scope text NOT NULL DEFAULT 'order',
  privacy_mechanism text NOT NULL DEFAULT 'differential_privacy',
  budget_epsilon numeric(20, 8),
  budget_delta numeric(20, 12),
  consumed_epsilon numeric(20, 8) NOT NULL DEFAULT 0,
  consumed_delta numeric(20, 12) NOT NULL DEFAULT 0,
  review_status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE delivery.sensitive_execution_policy
  ADD COLUMN IF NOT EXISTS privacy_mode text NOT NULL DEFAULT 'none',
  ADD COLUMN IF NOT EXISTS privacy_budget_required boolean NOT NULL DEFAULT false;

ALTER TABLE delivery.attestation_record
  ADD COLUMN IF NOT EXISTS proof_ref uuid REFERENCES ml.proof_artifact(proof_ref) ON DELETE SET NULL;

ALTER TABLE ml.compute_task
  ADD COLUMN IF NOT EXISTS sensitive_policy_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE ml.compute_result
  ADD COLUMN IF NOT EXISTS disclosure_review_status text NOT NULL DEFAULT 'pending';

CREATE INDEX IF NOT EXISTS idx_privacy_budget_ledger_order
  ON delivery.privacy_budget_ledger(order_id, review_status);
CREATE INDEX IF NOT EXISTS idx_privacy_budget_ledger_task
  ON delivery.privacy_budget_ledger(task_id, review_status);

CREATE TRIGGER trg_privacy_budget_ledger_updated_at BEFORE UPDATE ON delivery.privacy_budget_ledger
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
