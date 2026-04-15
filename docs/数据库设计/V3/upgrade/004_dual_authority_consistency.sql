ALTER TABLE crosschain.cross_chain_request
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE payment.crypto_settlement_transfer
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;
