DROP TRIGGER IF EXISTS trg_legal_hold_updated_at ON audit.legal_hold;
DROP TRIGGER IF EXISTS trg_replay_job_updated_at ON audit.replay_job;
DROP TRIGGER IF EXISTS trg_anchor_batch_updated_at ON audit.anchor_batch;
DROP TRIGGER IF EXISTS trg_retention_policy_updated_at ON audit.retention_policy;
DROP TRIGGER IF EXISTS trg_access_audit_append_only ON audit.access_audit;
DROP TRIGGER IF EXISTS trg_evidence_package_append_only ON audit.evidence_package;
DROP TRIGGER IF EXISTS trg_evidence_manifest_item_append_only ON audit.evidence_manifest_item;
DROP TRIGGER IF EXISTS trg_evidence_manifest_append_only ON audit.evidence_manifest;
DROP TRIGGER IF EXISTS trg_evidence_item_append_only ON audit.evidence_item;
DROP TRIGGER IF EXISTS trg_system_log_default_append_only ON ops.system_log_default;
DROP TRIGGER IF EXISTS trg_audit_event_default_append_only ON audit.audit_event_default;
DROP TRIGGER IF EXISTS trg_prepare_system_log_default ON ops.system_log_default;
DROP TRIGGER IF EXISTS trg_prepare_audit_event_default ON audit.audit_event_default;

DROP FUNCTION IF EXISTS audit.tg_append_only_guard();
DROP FUNCTION IF EXISTS ops.tg_prepare_system_log();
DROP FUNCTION IF EXISTS audit.tg_prepare_audit_event();

DROP TABLE IF EXISTS audit.access_audit CASCADE;
DROP TABLE IF EXISTS audit.legal_hold CASCADE;
DROP TABLE IF EXISTS audit.replay_result CASCADE;
DROP TABLE IF EXISTS audit.replay_job CASCADE;
DROP TABLE IF EXISTS audit.anchor_item CASCADE;
DROP TABLE IF EXISTS audit.anchor_batch CASCADE;
DROP TABLE IF EXISTS audit.evidence_manifest_item CASCADE;
DROP TABLE IF EXISTS audit.evidence_manifest CASCADE;
DROP TABLE IF EXISTS audit.evidence_item CASCADE;
DROP TABLE IF EXISTS audit.retention_policy CASCADE;

ALTER TABLE audit.evidence_package
  DROP COLUMN IF EXISTS legal_hold_status,
  DROP COLUMN IF EXISTS retention_class,
  DROP COLUMN IF EXISTS access_mode,
  DROP COLUMN IF EXISTS masked_level,
  DROP COLUMN IF EXISTS evidence_manifest_id;

ALTER TABLE ops.system_log
  DROP COLUMN IF EXISTS legal_hold_status,
  DROP COLUMN IF EXISTS retention_class,
  DROP COLUMN IF EXISTS log_hash,
  DROP COLUMN IF EXISTS previous_log_hash,
  DROP COLUMN IF EXISTS traceparent,
  DROP COLUMN IF EXISTS span_id,
  DROP COLUMN IF EXISTS source_type;

ALTER TABLE audit.audit_event
  DROP CONSTRAINT IF EXISTS fk_audit_event_manifest;

ALTER TABLE audit.audit_event
  DROP COLUMN IF EXISTS ingested_at,
  DROP COLUMN IF EXISTS sensitivity_level,
  DROP COLUMN IF EXISTS legal_hold_status,
  DROP COLUMN IF EXISTS retention_class,
  DROP COLUMN IF EXISTS anchor_policy,
  DROP COLUMN IF EXISTS evidence_manifest_id,
  DROP COLUMN IF EXISTS event_hash,
  DROP COLUMN IF EXISTS previous_event_hash,
  DROP COLUMN IF EXISTS after_state_digest,
  DROP COLUMN IF EXISTS before_state_digest,
  DROP COLUMN IF EXISTS step_up_challenge_id,
  DROP COLUMN IF EXISTS auth_assurance_level,
  DROP COLUMN IF EXISTS error_code,
  DROP COLUMN IF EXISTS parent_audit_id,
  DROP COLUMN IF EXISTS application_id,
  DROP COLUMN IF EXISTS trusted_device_id,
  DROP COLUMN IF EXISTS session_id,
  DROP COLUMN IF EXISTS actor_org_id,
  DROP COLUMN IF EXISTS event_class,
  DROP COLUMN IF EXISTS event_schema_version;

DROP INDEX IF EXISTS idx_system_log_traceparent;
DROP INDEX IF EXISTS idx_access_audit_target;
DROP INDEX IF EXISTS idx_replay_job_ref;
DROP INDEX IF EXISTS idx_anchor_batch_status;
DROP INDEX IF EXISTS idx_evidence_item_ref;
DROP INDEX IF EXISTS idx_audit_event_manifest;
DROP INDEX IF EXISTS idx_audit_event_tx;
DROP INDEX IF EXISTS idx_audit_event_trace;
DROP INDEX IF EXISTS idx_audit_event_request;
DROP INDEX IF EXISTS idx_audit_event_ref;
