DROP TRIGGER IF EXISTS trg_privacy_budget_ledger_updated_at ON delivery.privacy_budget_ledger;

ALTER TABLE ml.compute_result
  DROP COLUMN IF EXISTS disclosure_review_status;

ALTER TABLE ml.compute_task
  DROP COLUMN IF EXISTS sensitive_policy_snapshot;

ALTER TABLE delivery.attestation_record
  DROP COLUMN IF EXISTS proof_ref;

ALTER TABLE delivery.sensitive_execution_policy
  DROP COLUMN IF EXISTS privacy_budget_required,
  DROP COLUMN IF EXISTS privacy_mode;

DROP TABLE IF EXISTS delivery.privacy_budget_ledger CASCADE;
