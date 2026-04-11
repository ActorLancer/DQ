ALTER TABLE ops.trade_lifecycle_checkpoint
  ADD COLUMN IF NOT EXISTS ccr_id uuid REFERENCES crosschain.cross_chain_request(ccr_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS ack_id uuid REFERENCES crosschain.cross_chain_ack(ack_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS partner_id uuid REFERENCES ecosystem.partner(partner_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS crypto_transfer_id uuid REFERENCES payment.crypto_settlement_transfer(crypto_transfer_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS compensation_task_id uuid REFERENCES crosschain.compensation_task(compensation_task_id) ON DELETE SET NULL;

ALTER TABLE ops.external_fact_receipt
  ADD COLUMN IF NOT EXISTS ccr_id uuid REFERENCES crosschain.cross_chain_request(ccr_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS ack_id uuid REFERENCES crosschain.cross_chain_ack(ack_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS partner_id uuid REFERENCES ecosystem.partner(partner_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS crypto_transfer_id uuid REFERENCES payment.crypto_settlement_transfer(crypto_transfer_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS fact_scope text NOT NULL DEFAULT 'local';

ALTER TABLE risk.fairness_incident
  ADD COLUMN IF NOT EXISTS ccr_id uuid REFERENCES crosschain.cross_chain_request(ccr_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS partner_id uuid REFERENCES ecosystem.partner(partner_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS crypto_transfer_id uuid REFERENCES payment.crypto_settlement_transfer(crypto_transfer_id) ON DELETE SET NULL;

ALTER TABLE ops.chain_projection_gap
  ADD COLUMN IF NOT EXISTS ccr_id uuid REFERENCES crosschain.cross_chain_request(ccr_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS partner_id uuid REFERENCES ecosystem.partner(partner_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS compensation_task_id uuid REFERENCES crosschain.compensation_task(compensation_task_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_trade_lifecycle_checkpoint_ccr
  ON ops.trade_lifecycle_checkpoint(ccr_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_external_fact_receipt_ccr
  ON ops.external_fact_receipt(ccr_id, received_at DESC);
CREATE INDEX IF NOT EXISTS idx_fairness_incident_ccr
  ON risk.fairness_incident(ccr_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chain_projection_gap_ccr
  ON ops.chain_projection_gap(ccr_id, created_at DESC);
