ALTER TABLE audit.audit_event
  ADD COLUMN IF NOT EXISTS event_schema_version text NOT NULL DEFAULT 'v1',
  ADD COLUMN IF NOT EXISTS event_class text NOT NULL DEFAULT 'business',
  ADD COLUMN IF NOT EXISTS actor_org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS session_id uuid REFERENCES iam.user_session(session_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS trusted_device_id uuid REFERENCES iam.trusted_device(trusted_device_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS application_id uuid REFERENCES core.application(app_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS parent_audit_id uuid,
  ADD COLUMN IF NOT EXISTS error_code text,
  ADD COLUMN IF NOT EXISTS auth_assurance_level text,
  ADD COLUMN IF NOT EXISTS step_up_challenge_id uuid REFERENCES iam.step_up_challenge(step_up_challenge_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS before_state_digest text,
  ADD COLUMN IF NOT EXISTS after_state_digest text,
  ADD COLUMN IF NOT EXISTS previous_event_hash text,
  ADD COLUMN IF NOT EXISTS event_hash text,
  ADD COLUMN IF NOT EXISTS evidence_manifest_id uuid,
  ADD COLUMN IF NOT EXISTS anchor_policy text NOT NULL DEFAULT 'batched_fabric',
  ADD COLUMN IF NOT EXISTS retention_class text NOT NULL DEFAULT 'audit_default',
  ADD COLUMN IF NOT EXISTS legal_hold_status text NOT NULL DEFAULT 'none',
  ADD COLUMN IF NOT EXISTS sensitivity_level text NOT NULL DEFAULT 'normal',
  ADD COLUMN IF NOT EXISTS ingested_at timestamptz NOT NULL DEFAULT now();

ALTER TABLE ops.system_log
  ADD COLUMN IF NOT EXISTS source_type text NOT NULL DEFAULT 'application',
  ADD COLUMN IF NOT EXISTS span_id text,
  ADD COLUMN IF NOT EXISTS traceparent text,
  ADD COLUMN IF NOT EXISTS previous_log_hash text,
  ADD COLUMN IF NOT EXISTS log_hash text,
  ADD COLUMN IF NOT EXISTS retention_class text NOT NULL DEFAULT 'ops_default',
  ADD COLUMN IF NOT EXISTS legal_hold_status text NOT NULL DEFAULT 'none';

CREATE TABLE IF NOT EXISTS audit.retention_policy (
  retention_policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  policy_key text NOT NULL UNIQUE,
  scope_type text NOT NULL,
  scope_id uuid,
  retention_class text NOT NULL,
  hot_days integer,
  warm_days integer,
  cold_days integer,
  delete_after_days integer,
  worm_required boolean NOT NULL DEFAULT false,
  legal_hold_allowed boolean NOT NULL DEFAULT true,
  status text NOT NULL DEFAULT 'active',
  created_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit.evidence_item (
  evidence_item_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  item_type text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid,
  object_uri text,
  object_hash text NOT NULL,
  content_type text,
  size_bytes bigint,
  source_system text,
  storage_mode text,
  retention_policy_id uuid REFERENCES audit.retention_policy(retention_policy_id) ON DELETE SET NULL,
  worm_enabled boolean NOT NULL DEFAULT false,
  legal_hold_status text NOT NULL DEFAULT 'none',
  created_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS audit.evidence_manifest (
  evidence_manifest_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  manifest_scope text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid,
  manifest_hash text NOT NULL,
  item_count integer NOT NULL DEFAULT 0,
  storage_uri text,
  created_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS audit.evidence_manifest_item (
  evidence_manifest_item_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  evidence_manifest_id uuid NOT NULL REFERENCES audit.evidence_manifest(evidence_manifest_id) ON DELETE CASCADE,
  evidence_item_id uuid NOT NULL REFERENCES audit.evidence_item(evidence_item_id) ON DELETE CASCADE,
  item_digest text NOT NULL,
  ordinal_no integer NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_evidence_manifest_item UNIQUE (evidence_manifest_id, evidence_item_id),
  CONSTRAINT uq_evidence_manifest_ordinal UNIQUE (evidence_manifest_id, ordinal_no)
);

ALTER TABLE audit.audit_event
  ADD CONSTRAINT fk_audit_event_manifest
  FOREIGN KEY (evidence_manifest_id) REFERENCES audit.evidence_manifest(evidence_manifest_id) ON DELETE SET NULL;

ALTER TABLE audit.evidence_package
  ADD COLUMN IF NOT EXISTS evidence_manifest_id uuid REFERENCES audit.evidence_manifest(evidence_manifest_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS masked_level text NOT NULL DEFAULT 'summary',
  ADD COLUMN IF NOT EXISTS access_mode text NOT NULL DEFAULT 'export',
  ADD COLUMN IF NOT EXISTS retention_class text NOT NULL DEFAULT 'audit_default',
  ADD COLUMN IF NOT EXISTS legal_hold_status text NOT NULL DEFAULT 'none';

CREATE TABLE IF NOT EXISTS audit.anchor_batch (
  anchor_batch_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  batch_scope text NOT NULL,
  chain_id text NOT NULL,
  record_count integer NOT NULL DEFAULT 0,
  batch_root text NOT NULL,
  window_started_at timestamptz,
  window_ended_at timestamptz,
  status text NOT NULL DEFAULT 'pending',
  chain_anchor_id uuid REFERENCES chain.chain_anchor(chain_anchor_id) ON DELETE SET NULL,
  created_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  anchored_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit.anchor_item (
  anchor_item_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  anchor_batch_id uuid NOT NULL REFERENCES audit.anchor_batch(anchor_batch_id) ON DELETE CASCADE,
  audit_id uuid,
  evidence_manifest_id uuid REFERENCES audit.evidence_manifest(evidence_manifest_id) ON DELETE SET NULL,
  object_type text NOT NULL,
  object_id uuid,
  object_digest text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit.replay_job (
  replay_job_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  replay_type text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid,
  dry_run boolean NOT NULL DEFAULT true,
  status text NOT NULL DEFAULT 'pending',
  requested_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  step_up_challenge_id uuid REFERENCES iam.step_up_challenge(step_up_challenge_id) ON DELETE SET NULL,
  request_reason text,
  options_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  started_at timestamptz,
  finished_at timestamptz,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit.replay_result (
  replay_result_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  replay_job_id uuid NOT NULL REFERENCES audit.replay_job(replay_job_id) ON DELETE CASCADE,
  step_name text NOT NULL,
  result_code text NOT NULL,
  expected_digest text,
  actual_digest text,
  diff_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit.legal_hold (
  legal_hold_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  hold_scope_type text NOT NULL,
  hold_scope_id uuid,
  reason_code text NOT NULL,
  status text NOT NULL DEFAULT 'active',
  retention_policy_id uuid REFERENCES audit.retention_policy(retention_policy_id) ON DELETE SET NULL,
  requested_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  approved_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  hold_until timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  released_at timestamptz,
  updated_at timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS audit.access_audit (
  access_audit_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  accessor_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  accessor_role_key text,
  access_mode text NOT NULL,
  target_type text NOT NULL,
  target_id uuid,
  masked_view boolean NOT NULL DEFAULT true,
  breakglass_reason text,
  step_up_challenge_id uuid REFERENCES iam.step_up_challenge(step_up_challenge_id) ON DELETE SET NULL,
  request_id text,
  trace_id text,
  created_at timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_audit_event_ref ON audit.audit_event (ref_type, ref_id, event_time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_event_request ON audit.audit_event (request_id, event_time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_event_trace ON audit.audit_event (trace_id, event_time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_event_tx ON audit.audit_event (tx_hash, event_time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_event_manifest ON audit.audit_event (evidence_manifest_id, event_time DESC);
CREATE INDEX IF NOT EXISTS idx_evidence_item_ref ON audit.evidence_item (ref_type, ref_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_anchor_batch_status ON audit.anchor_batch (status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_replay_job_ref ON audit.replay_job (ref_type, ref_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_access_audit_target ON audit.access_audit (target_type, target_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_system_log_traceparent ON ops.system_log (traceparent, created_at DESC);

CREATE OR REPLACE FUNCTION audit.tg_prepare_audit_event()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  NEW.ingested_at = COALESCE(NEW.ingested_at, now());
  NEW.event_schema_version = COALESCE(NEW.event_schema_version, 'v1');
  NEW.event_class = COALESCE(NEW.event_class, 'business');
  NEW.anchor_policy = COALESCE(NEW.anchor_policy, 'batched_fabric');
  NEW.retention_class = COALESCE(NEW.retention_class, 'audit_default');
  NEW.legal_hold_status = COALESCE(NEW.legal_hold_status, 'none');
  NEW.sensitivity_level = COALESCE(NEW.sensitivity_level, 'normal');
  IF NEW.event_hash IS NULL THEN
    NEW.event_hash := encode(
      digest(
        COALESCE(NEW.domain_name, '') || '|' ||
        COALESCE(NEW.ref_type, '') || '|' ||
        COALESCE(NEW.ref_id::text, '') || '|' ||
        COALESCE(NEW.actor_id::text, '') || '|' ||
        COALESCE(NEW.action_name, '') || '|' ||
        COALESCE(NEW.result_code, '') || '|' ||
        COALESCE(NEW.request_id, '') || '|' ||
        COALESCE(NEW.trace_id, '') || '|' ||
        COALESCE(NEW.previous_event_hash, '') || '|' ||
        COALESCE(NEW.event_time::text, now()::text),
        'sha256'
      ),
      'hex'
    );
  END IF;
  RETURN NEW;
END;
$$;

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

CREATE OR REPLACE FUNCTION audit.tg_append_only_guard()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  RAISE EXCEPTION 'append-only audit object % cannot be %', TG_TABLE_NAME, TG_OP;
END;
$$;

CREATE TRIGGER trg_prepare_audit_event_default
BEFORE INSERT ON audit.audit_event_default
FOR EACH ROW EXECUTE FUNCTION audit.tg_prepare_audit_event();

CREATE TRIGGER trg_prepare_system_log_default
BEFORE INSERT ON ops.system_log_default
FOR EACH ROW EXECUTE FUNCTION ops.tg_prepare_system_log();

CREATE TRIGGER trg_audit_event_default_append_only
BEFORE UPDATE OR DELETE ON audit.audit_event_default
FOR EACH ROW EXECUTE FUNCTION audit.tg_append_only_guard();

CREATE TRIGGER trg_system_log_default_append_only
BEFORE UPDATE OR DELETE ON ops.system_log_default
FOR EACH ROW EXECUTE FUNCTION audit.tg_append_only_guard();

CREATE TRIGGER trg_evidence_item_append_only
BEFORE UPDATE OR DELETE ON audit.evidence_item
FOR EACH ROW EXECUTE FUNCTION audit.tg_append_only_guard();

CREATE TRIGGER trg_evidence_manifest_append_only
BEFORE UPDATE OR DELETE ON audit.evidence_manifest
FOR EACH ROW EXECUTE FUNCTION audit.tg_append_only_guard();

CREATE TRIGGER trg_evidence_manifest_item_append_only
BEFORE UPDATE OR DELETE ON audit.evidence_manifest_item
FOR EACH ROW EXECUTE FUNCTION audit.tg_append_only_guard();

CREATE TRIGGER trg_evidence_package_append_only
BEFORE UPDATE OR DELETE ON audit.evidence_package
FOR EACH ROW EXECUTE FUNCTION audit.tg_append_only_guard();

CREATE TRIGGER trg_access_audit_append_only
BEFORE UPDATE OR DELETE ON audit.access_audit
FOR EACH ROW EXECUTE FUNCTION audit.tg_append_only_guard();

CREATE TRIGGER trg_retention_policy_updated_at
BEFORE UPDATE ON audit.retention_policy
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

CREATE TRIGGER trg_anchor_batch_updated_at
BEFORE UPDATE ON audit.anchor_batch
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

CREATE TRIGGER trg_replay_job_updated_at
BEFORE UPDATE ON audit.replay_job
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

CREATE TRIGGER trg_legal_hold_updated_at
BEFORE UPDATE ON audit.legal_hold
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
