DROP TRIGGER IF EXISTS trg_chain_projection_gap_updated_at ON ops.chain_projection_gap;
DROP TRIGGER IF EXISTS trg_fairness_incident_updated_at ON risk.fairness_incident;
DROP TRIGGER IF EXISTS trg_external_fact_receipt_updated_at ON ops.external_fact_receipt;
DROP TRIGGER IF EXISTS trg_trade_lifecycle_checkpoint_updated_at ON ops.trade_lifecycle_checkpoint;
DROP TRIGGER IF EXISTS trg_monitoring_policy_profile_updated_at ON ops.monitoring_policy_profile;

DROP TABLE IF EXISTS ops.chain_projection_gap CASCADE;
DROP TABLE IF EXISTS risk.fairness_incident CASCADE;
DROP TABLE IF EXISTS ops.external_fact_receipt CASCADE;
DROP TABLE IF EXISTS ops.trade_lifecycle_checkpoint CASCADE;
DROP TABLE IF EXISTS ops.monitoring_policy_profile CASCADE;
