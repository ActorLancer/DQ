DROP TRIGGER IF EXISTS trg_slo_definition_updated_at ON ops.slo_definition;
DROP TRIGGER IF EXISTS trg_incident_ticket_updated_at ON ops.incident_ticket;
DROP TRIGGER IF EXISTS trg_alert_event_updated_at ON ops.alert_event;
DROP TRIGGER IF EXISTS trg_alert_rule_updated_at ON ops.alert_rule;
DROP TRIGGER IF EXISTS trg_trace_index_updated_at ON ops.trace_index;
DROP TRIGGER IF EXISTS trg_log_retention_policy_updated_at ON ops.log_retention_policy;
DROP TRIGGER IF EXISTS trg_observability_backend_updated_at ON ops.observability_backend;

DROP INDEX IF EXISTS idx_slo_snapshot_def;
DROP INDEX IF EXISTS idx_incident_ticket_status;
DROP INDEX IF EXISTS idx_alert_event_trace;
DROP INDEX IF EXISTS idx_alert_event_status;
DROP INDEX IF EXISTS idx_trace_index_ref;
DROP INDEX IF EXISTS idx_trace_index_trace;
DROP INDEX IF EXISTS idx_system_log_request;
DROP INDEX IF EXISTS idx_system_log_object;
DROP INDEX IF EXISTS idx_system_log_service_level;

DROP TABLE IF EXISTS ops.slo_snapshot CASCADE;
DROP TABLE IF EXISTS ops.slo_definition CASCADE;
DROP TABLE IF EXISTS ops.incident_event CASCADE;
DROP TABLE IF EXISTS ops.incident_ticket CASCADE;
DROP TABLE IF EXISTS ops.alert_event CASCADE;
DROP TABLE IF EXISTS ops.alert_rule CASCADE;
DROP TABLE IF EXISTS ops.trace_index CASCADE;
DROP TABLE IF EXISTS ops.log_retention_policy CASCADE;
DROP TABLE IF EXISTS ops.observability_backend CASCADE;

ALTER TABLE ops.system_log
  DROP COLUMN IF EXISTS logger_name,
  DROP COLUMN IF EXISTS environment_code,
  DROP COLUMN IF EXISTS host_name,
  DROP COLUMN IF EXISTS node_name,
  DROP COLUMN IF EXISTS pod_name,
  DROP COLUMN IF EXISTS backend_type,
  DROP COLUMN IF EXISTS severity_number,
  DROP COLUMN IF EXISTS object_type,
  DROP COLUMN IF EXISTS object_id,
  DROP COLUMN IF EXISTS masked_status,
  DROP COLUMN IF EXISTS resource_attrs;

CREATE OR REPLACE FUNCTION ops.tg_prepare_system_log()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  NEW.source_type = COALESCE(NEW.source_type, 'application');
  NEW.retention_class = COALESCE(NEW.retention_class, 'ops_default');
  NEW.legal_hold_status = COALESCE(NEW.legal_hold_status, 'none');
  IF NEW.log_hash IS NULL THEN
    NEW.log_hash := encode(
      digest(
        COALESCE(NEW.service_name, '') || '|' ||
        COALESCE(NEW.log_level, '') || '|' ||
        COALESCE(NEW.request_id, '') || '|' ||
        COALESCE(NEW.trace_id, '') || '|' ||
        COALESCE(NEW.previous_log_hash, '') || '|' ||
        COALESCE(NEW.created_at::text, now()::text),
        'sha256'
      ),
      'hex'
    );
  END IF;
  RETURN NEW;
END;
$$;
