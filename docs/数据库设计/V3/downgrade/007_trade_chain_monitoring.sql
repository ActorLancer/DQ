ALTER TABLE ops.chain_projection_gap
  DROP COLUMN IF EXISTS compensation_task_id,
  DROP COLUMN IF EXISTS partner_id,
  DROP COLUMN IF EXISTS ccr_id;

ALTER TABLE risk.fairness_incident
  DROP COLUMN IF EXISTS crypto_transfer_id,
  DROP COLUMN IF EXISTS partner_id,
  DROP COLUMN IF EXISTS ccr_id;

ALTER TABLE ops.external_fact_receipt
  DROP COLUMN IF EXISTS fact_scope,
  DROP COLUMN IF EXISTS crypto_transfer_id,
  DROP COLUMN IF EXISTS partner_id,
  DROP COLUMN IF EXISTS ack_id,
  DROP COLUMN IF EXISTS ccr_id;

ALTER TABLE ops.trade_lifecycle_checkpoint
  DROP COLUMN IF EXISTS compensation_task_id,
  DROP COLUMN IF EXISTS crypto_transfer_id,
  DROP COLUMN IF EXISTS partner_id,
  DROP COLUMN IF EXISTS ack_id,
  DROP COLUMN IF EXISTS ccr_id;
