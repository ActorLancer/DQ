ALTER TABLE ops.system_log
  ADD COLUMN IF NOT EXISTS logger_name text,
  ADD COLUMN IF NOT EXISTS environment_code text NOT NULL DEFAULT 'default',
  ADD COLUMN IF NOT EXISTS host_name text,
  ADD COLUMN IF NOT EXISTS node_name text,
  ADD COLUMN IF NOT EXISTS pod_name text,
  ADD COLUMN IF NOT EXISTS backend_type text NOT NULL DEFAULT 'database_mirror',
  ADD COLUMN IF NOT EXISTS severity_number integer,
  ADD COLUMN IF NOT EXISTS object_type text,
  ADD COLUMN IF NOT EXISTS object_id uuid,
  ADD COLUMN IF NOT EXISTS masked_status text NOT NULL DEFAULT 'masked',
  ADD COLUMN IF NOT EXISTS resource_attrs jsonb NOT NULL DEFAULT '{}'::jsonb;

CREATE TABLE IF NOT EXISTS ops.observability_backend (
  observability_backend_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  backend_key text NOT NULL UNIQUE,
  backend_type text NOT NULL,
  endpoint_uri text,
  auth_mode text NOT NULL DEFAULT 'none',
  enabled boolean NOT NULL DEFAULT true,
  stage_from text NOT NULL DEFAULT 'V1',
  capability_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.log_retention_policy (
  log_retention_policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  policy_key text NOT NULL UNIQUE,
  target_scope text NOT NULL,
  hot_days integer,
  warm_days integer,
  cold_days integer,
  delete_after_days integer,
  storage_backend_key text REFERENCES ops.observability_backend(backend_key) ON DELETE SET NULL,
  object_lock_enabled boolean NOT NULL DEFAULT false,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.trace_index (
  trace_index_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  trace_id text NOT NULL,
  traceparent text,
  backend_key text REFERENCES ops.observability_backend(backend_key) ON DELETE SET NULL,
  root_service_name text,
  root_span_name text,
  request_id text,
  ref_type text,
  ref_id uuid,
  object_type text,
  object_id uuid,
  status text NOT NULL DEFAULT 'ok',
  span_count integer,
  started_at timestamptz,
  ended_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.alert_rule (
  alert_rule_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  rule_key text NOT NULL UNIQUE,
  source_backend_key text REFERENCES ops.observability_backend(backend_key) ON DELETE SET NULL,
  severity text NOT NULL,
  alert_type text NOT NULL,
  expression_text text NOT NULL,
  target_scope_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  notification_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  runbook_uri text,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.alert_event (
  alert_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  alert_rule_id uuid REFERENCES ops.alert_rule(alert_rule_id) ON DELETE SET NULL,
  source_backend_key text REFERENCES ops.observability_backend(backend_key) ON DELETE SET NULL,
  fingerprint text NOT NULL,
  alert_type text NOT NULL,
  severity text NOT NULL,
  title_text text NOT NULL,
  summary_text text,
  ref_type text,
  ref_id uuid,
  request_id text,
  trace_id text,
  labels_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  annotations_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'open',
  acknowledged_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  acknowledged_at timestamptz,
  fired_at timestamptz NOT NULL DEFAULT now(),
  resolved_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.incident_ticket (
  incident_ticket_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  incident_key text NOT NULL UNIQUE,
  source_alert_event_id uuid REFERENCES ops.alert_event(alert_event_id) ON DELETE SET NULL,
  severity text NOT NULL,
  title_text text NOT NULL,
  summary_text text,
  status text NOT NULL DEFAULT 'open',
  owner_role_key text REFERENCES authz.role_definition(role_key) ON DELETE SET NULL,
  owner_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  runbook_uri text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  started_at timestamptz NOT NULL DEFAULT now(),
  resolved_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.incident_event (
  incident_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  incident_ticket_id uuid NOT NULL REFERENCES ops.incident_ticket(incident_ticket_id) ON DELETE CASCADE,
  event_type text NOT NULL,
  actor_type text NOT NULL DEFAULT 'system',
  actor_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  from_status text,
  to_status text,
  note_text text,
  request_id text,
  trace_id text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.slo_definition (
  slo_definition_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  slo_key text NOT NULL UNIQUE,
  service_name text NOT NULL,
  indicator_type text NOT NULL,
  objective_value numeric(12, 6) NOT NULL,
  window_code text NOT NULL,
  source_backend_key text REFERENCES ops.observability_backend(backend_key) ON DELETE SET NULL,
  alert_rule_id uuid REFERENCES ops.alert_rule(alert_rule_id) ON DELETE SET NULL,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.slo_snapshot (
  slo_snapshot_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  slo_definition_id uuid NOT NULL REFERENCES ops.slo_definition(slo_definition_id) ON DELETE CASCADE,
  source_backend_key text REFERENCES ops.observability_backend(backend_key) ON DELETE SET NULL,
  window_started_at timestamptz NOT NULL,
  window_ended_at timestamptz NOT NULL,
  measured_value numeric(12, 6),
  error_budget_remaining numeric(12, 6),
  status text NOT NULL DEFAULT 'ok',
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_system_log_service_level
  ON ops.system_log (service_name, log_level, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_system_log_object
  ON ops.system_log (object_type, object_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_system_log_request
  ON ops.system_log (request_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_trace_index_trace
  ON ops.trace_index (trace_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_trace_index_ref
  ON ops.trace_index (ref_type, ref_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_event_status
  ON ops.alert_event (status, severity, fired_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_event_trace
  ON ops.alert_event (trace_id, fired_at DESC);
CREATE INDEX IF NOT EXISTS idx_incident_ticket_status
  ON ops.incident_ticket (status, severity, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_slo_snapshot_def
  ON ops.slo_snapshot (slo_definition_id, window_ended_at DESC);

CREATE TRIGGER trg_observability_backend_updated_at BEFORE UPDATE ON ops.observability_backend
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_log_retention_policy_updated_at BEFORE UPDATE ON ops.log_retention_policy
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_trace_index_updated_at BEFORE UPDATE ON ops.trace_index
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_alert_rule_updated_at BEFORE UPDATE ON ops.alert_rule
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_alert_event_updated_at BEFORE UPDATE ON ops.alert_event
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_incident_ticket_updated_at BEFORE UPDATE ON ops.incident_ticket
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_slo_definition_updated_at BEFORE UPDATE ON ops.slo_definition
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

CREATE OR REPLACE FUNCTION ops.tg_prepare_system_log()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  NEW.source_type = COALESCE(NEW.source_type, 'application');
  NEW.environment_code = COALESCE(NEW.environment_code, 'default');
  NEW.backend_type = COALESCE(NEW.backend_type, 'database_mirror');
  NEW.masked_status = COALESCE(NEW.masked_status, 'masked');
  NEW.retention_class = COALESCE(NEW.retention_class, 'ops_default');
  NEW.legal_hold_status = COALESCE(NEW.legal_hold_status, 'none');
  NEW.resource_attrs = COALESCE(NEW.resource_attrs, '{}'::jsonb);
  IF NEW.log_hash IS NULL THEN
    NEW.log_hash := encode(
      digest(
        COALESCE(NEW.service_name, '') || '|' ||
        COALESCE(NEW.logger_name, '') || '|' ||
        COALESCE(NEW.log_level, '') || '|' ||
        COALESCE(NEW.request_id, '') || '|' ||
        COALESCE(NEW.trace_id, '') || '|' ||
        COALESCE(NEW.environment_code, '') || '|' ||
        COALESCE(NEW.backend_type, '') || '|' ||
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

INSERT INTO ops.observability_backend (
  backend_key, backend_type, endpoint_uri, auth_mode, enabled, stage_from, capability_json, metadata
) VALUES
('otel_collector', 'collector', 'otel-collector.internal:4317', 'none', true, 'V1', '{"signals":["logs","metrics","traces"]}'::jsonb, '{"recommended":"true"}'::jsonb),
('prometheus_main', 'metrics', 'prometheus.internal:9090', 'none', true, 'V1', '{"signals":["metrics","alerts"]}'::jsonb, '{}'::jsonb),
('alertmanager_main', 'alerting', 'alertmanager.internal:9093', 'none', true, 'V1', '{"signals":["alerts"]}'::jsonb, '{}'::jsonb),
('grafana_main', 'dashboard', 'grafana.internal:3000', 'none', true, 'V1', '{"signals":["dashboards"]}'::jsonb, '{}'::jsonb),
('loki_main', 'logs', 'loki.internal:3100', 'none', true, 'V1', '{"signals":["logs"]}'::jsonb, '{}'::jsonb),
('tempo_main', 'traces', 'tempo.internal:3200', 'none', true, 'V1', '{"signals":["traces"]}'::jsonb, '{}'::jsonb)
ON CONFLICT (backend_key) DO NOTHING;

INSERT INTO ops.log_retention_policy (
  policy_key, target_scope, hot_days, warm_days, cold_days, delete_after_days, storage_backend_key, object_lock_enabled, status, metadata
) VALUES
('ops_default', 'system_log', 7, 30, 180, 365, 'loki_main', false, 'active', '{}'::jsonb),
('security_sensitive', 'security_log', 30, 90, 365, 730, 'loki_main', true, 'active', '{"requires_masking":"true"}'::jsonb),
('trace_index_default', 'trace_index', 3, 14, 90, 180, 'tempo_main', false, 'active', '{}'::jsonb)
ON CONFLICT (policy_key) DO NOTHING;

INSERT INTO ops.alert_rule (
  rule_key, source_backend_key, severity, alert_type, expression_text, target_scope_json, notification_policy_json, runbook_uri, status, metadata
) VALUES
('outbox_dead_letter_backlog', 'prometheus_main', 'high', 'event_pipeline', 'dead_letter_event backlog > threshold', '{"domain":["ops"]}'::jsonb, '{"notify":["oncall","ops-console"]}'::jsonb, '/runbooks/outbox-dead-letter', 'active', '{}'::jsonb),
('fabric_writer_failure', 'prometheus_main', 'high', 'chain_writer', 'fabric writer failures > threshold', '{"domain":["chain"]}'::jsonb, '{"notify":["oncall","chain-ops"]}'::jsonb, '/runbooks/fabric-writer', 'active', '{}'::jsonb),
('search_sync_lag', 'prometheus_main', 'medium', 'search_sync', 'search sync lag > threshold', '{"domain":["search"]}'::jsonb, '{"notify":["search-ops"]}'::jsonb, '/runbooks/search-sync', 'active', '{}'::jsonb),
('payment_webhook_failure_spike', 'prometheus_main', 'high', 'payment_webhook', 'payment webhook failures > threshold', '{"domain":["payment"]}'::jsonb, '{"notify":["finance-ops","payment-admin"]}'::jsonb, '/runbooks/payment-webhook', 'active', '{}'::jsonb)
ON CONFLICT (rule_key) DO NOTHING;

-- Observability baseline: raw logs/traces/metrics live in Loki/Tempo/Prometheus; PostgreSQL stores governance, alert, incident and key log mirrors only.
