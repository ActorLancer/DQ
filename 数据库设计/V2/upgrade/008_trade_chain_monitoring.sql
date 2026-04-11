ALTER TABLE ops.trade_lifecycle_checkpoint
  ADD COLUMN IF NOT EXISTS compute_task_id uuid REFERENCES ml.compute_task(task_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS training_task_id uuid REFERENCES ml.training_task(task_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS compute_result_id uuid REFERENCES ml.compute_result(compute_result_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS reward_id uuid REFERENCES billing.reward_record(reward_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS environment_id uuid REFERENCES core.execution_environment(environment_id) ON DELETE SET NULL;

ALTER TABLE ops.external_fact_receipt
  ADD COLUMN IF NOT EXISTS compute_task_id uuid REFERENCES ml.compute_task(task_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS training_task_id uuid REFERENCES ml.training_task(task_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS reward_id uuid REFERENCES billing.reward_record(reward_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS environment_id uuid REFERENCES core.execution_environment(environment_id) ON DELETE SET NULL;

ALTER TABLE risk.fairness_incident
  ADD COLUMN IF NOT EXISTS compute_task_id uuid REFERENCES ml.compute_task(task_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS training_task_id uuid REFERENCES ml.training_task(task_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS reward_id uuid REFERENCES billing.reward_record(reward_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_trade_lifecycle_checkpoint_compute_task
  ON ops.trade_lifecycle_checkpoint(compute_task_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_trade_lifecycle_checkpoint_training_task
  ON ops.trade_lifecycle_checkpoint(training_task_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_external_fact_receipt_compute_task
  ON ops.external_fact_receipt(compute_task_id, received_at DESC);
CREATE INDEX IF NOT EXISTS idx_fairness_incident_compute_task
  ON risk.fairness_incident(compute_task_id, created_at DESC);
