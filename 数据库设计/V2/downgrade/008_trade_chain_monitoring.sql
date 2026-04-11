ALTER TABLE risk.fairness_incident
  DROP COLUMN IF EXISTS reward_id,
  DROP COLUMN IF EXISTS training_task_id,
  DROP COLUMN IF EXISTS compute_task_id;

ALTER TABLE ops.external_fact_receipt
  DROP COLUMN IF EXISTS environment_id,
  DROP COLUMN IF EXISTS reward_id,
  DROP COLUMN IF EXISTS training_task_id,
  DROP COLUMN IF EXISTS compute_task_id;

ALTER TABLE ops.trade_lifecycle_checkpoint
  DROP COLUMN IF EXISTS environment_id,
  DROP COLUMN IF EXISTS reward_id,
  DROP COLUMN IF EXISTS compute_result_id,
  DROP COLUMN IF EXISTS training_task_id,
  DROP COLUMN IF EXISTS compute_task_id;
