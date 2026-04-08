ALTER TABLE payment.crypto_settlement_transfer
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE crosschain.cross_chain_request
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS authority_model;
