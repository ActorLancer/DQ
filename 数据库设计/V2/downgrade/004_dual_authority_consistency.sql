ALTER TABLE billing.reward_record
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE ml.training_task
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE ml.compute_result
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE ml.compute_task
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS authority_model;
