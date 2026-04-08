ALTER TABLE ml.compute_task
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'n/a',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE ml.compute_result
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'n/a',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE ml.training_task
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'n/a',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE billing.reward_record
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;
