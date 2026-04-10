CREATE TABLE IF NOT EXISTS catalog.query_surface_definition (
  query_surface_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  asset_object_id uuid REFERENCES catalog.asset_object_binding(asset_object_id) ON DELETE SET NULL,
  environment_id uuid REFERENCES core.execution_environment(environment_id) ON DELETE SET NULL,
  surface_type text NOT NULL DEFAULT 'template_query_lite',
  binding_mode text NOT NULL DEFAULT 'managed_surface',
  execution_scope text NOT NULL DEFAULT 'curated_zone',
  input_contract_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  output_boundary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  query_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  quota_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_query_surface_asset_version
  ON catalog.query_surface_definition(asset_version_id, surface_type, status);

CREATE TABLE IF NOT EXISTS delivery.query_template_definition (
  query_template_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  query_surface_id uuid NOT NULL REFERENCES catalog.query_surface_definition(query_surface_id) ON DELETE CASCADE,
  template_name text NOT NULL,
  template_type text NOT NULL DEFAULT 'sql_template',
  template_body_ref text,
  parameter_schema_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  analysis_rule_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  result_schema_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  export_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  risk_guard_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  version_no integer NOT NULL DEFAULT 1,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (query_surface_id, template_name, version_no)
);

CREATE INDEX IF NOT EXISTS idx_query_template_surface
  ON delivery.query_template_definition(query_surface_id, status);

CREATE TABLE IF NOT EXISTS delivery.query_execution_run (
  query_run_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  template_query_grant_id uuid REFERENCES delivery.template_query_grant(template_query_grant_id) ON DELETE SET NULL,
  sandbox_session_id uuid REFERENCES delivery.sandbox_session(sandbox_session_id) ON DELETE SET NULL,
  query_template_id uuid REFERENCES delivery.query_template_definition(query_template_id) ON DELETE SET NULL,
  query_surface_id uuid REFERENCES catalog.query_surface_definition(query_surface_id) ON DELETE SET NULL,
  requester_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  execution_mode text NOT NULL DEFAULT 'template_query',
  request_payload_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  result_summary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  result_object_id uuid REFERENCES delivery.storage_object(object_id) ON DELETE SET NULL,
  result_row_count bigint,
  billed_units numeric(20, 8) NOT NULL DEFAULT 0,
  export_attempt_count integer NOT NULL DEFAULT 0,
  status text NOT NULL DEFAULT 'queued',
  started_at timestamptz,
  completed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_query_execution_run_order
  ON delivery.query_execution_run(order_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_query_execution_run_surface
  ON delivery.query_execution_run(query_surface_id, created_at DESC);

ALTER TABLE delivery.template_query_grant
  ADD COLUMN IF NOT EXISTS query_surface_id uuid REFERENCES catalog.query_surface_definition(query_surface_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS allowed_template_ids jsonb NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN IF NOT EXISTS execution_rule_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE delivery.sandbox_workspace
  ADD COLUMN IF NOT EXISTS query_surface_id uuid REFERENCES catalog.query_surface_definition(query_surface_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS clean_room_mode text NOT NULL DEFAULT 'lite';

CREATE INDEX IF NOT EXISTS idx_template_query_grant_surface
  ON delivery.template_query_grant(query_surface_id, grant_status);

CREATE INDEX IF NOT EXISTS idx_sandbox_workspace_surface
  ON delivery.sandbox_workspace(query_surface_id, status);

CREATE TRIGGER trg_query_surface_definition_updated_at BEFORE UPDATE ON catalog.query_surface_definition
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

CREATE TRIGGER trg_query_template_definition_updated_at BEFORE UPDATE ON delivery.query_template_definition
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

CREATE TRIGGER trg_query_execution_run_updated_at BEFORE UPDATE ON delivery.query_execution_run
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
