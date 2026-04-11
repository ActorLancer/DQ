ALTER TABLE catalog.data_asset
  ADD COLUMN IF NOT EXISTS contains_pi boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS contains_spi boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS data_classification text,
  ADD COLUMN IF NOT EXISTS sensitive_tags text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS default_sensitive_delivery_mode text NOT NULL DEFAULT 'standard';

ALTER TABLE catalog.asset_version
  ADD COLUMN IF NOT EXISTS field_sensitivity_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN IF NOT EXISTS safe_preview_required boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS result_review_required boolean NOT NULL DEFAULT false;

ALTER TABLE catalog.product
  ADD COLUMN IF NOT EXISTS sensitive_handling_required boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS safe_preview_mode text NOT NULL DEFAULT 'standard',
  ADD COLUMN IF NOT EXISTS result_delivery_mode text NOT NULL DEFAULT 'standard';

CREATE TABLE IF NOT EXISTS catalog.sensitive_handling_policy (
  sensitive_policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  product_id uuid REFERENCES catalog.product(product_id) ON DELETE SET NULL,
  policy_name text NOT NULL,
  sensitivity_level text NOT NULL DEFAULT 'normal',
  classification_level text,
  contains_pi boolean NOT NULL DEFAULT false,
  contains_spi boolean NOT NULL DEFAULT false,
  legal_basis_required boolean NOT NULL DEFAULT false,
  safe_preview_required boolean NOT NULL DEFAULT true,
  allowed_delivery_modes text[] NOT NULL DEFAULT '{}',
  required_execution_boundary text,
  export_boundary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  result_review_required boolean NOT NULL DEFAULT false,
  revoke_on_expire boolean NOT NULL DEFAULT true,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (asset_version_id)
);

CREATE TABLE IF NOT EXISTS contract.legal_basis_evidence (
  legal_basis_evidence_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  product_id uuid REFERENCES catalog.product(product_id) ON DELETE SET NULL,
  subject_org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  evidence_type text NOT NULL,
  legal_basis_type text,
  jurisdiction_code text,
  evidence_uri text,
  evidence_hash text,
  summary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  verified_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  status text NOT NULL DEFAULT 'draft',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.safe_preview_artifact (
  safe_preview_artifact_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  product_id uuid REFERENCES catalog.product(product_id) ON DELETE SET NULL,
  object_id uuid REFERENCES delivery.storage_object(object_id) ON DELETE SET NULL,
  preview_type text NOT NULL,
  masking_level text NOT NULL DEFAULT 'masked',
  preview_scope text NOT NULL DEFAULT 'schema_summary',
  preview_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  approved_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  approval_ticket_id uuid REFERENCES ops.approval_ticket(approval_ticket_id) ON DELETE SET NULL,
  status text NOT NULL DEFAULT 'draft',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.sensitive_execution_policy (
  sensitive_execution_policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  query_surface_id uuid REFERENCES catalog.query_surface_definition(query_surface_id) ON DELETE SET NULL,
  template_query_grant_id uuid REFERENCES delivery.template_query_grant(template_query_grant_id) ON DELETE SET NULL,
  sandbox_workspace_id uuid REFERENCES delivery.sandbox_workspace(sandbox_workspace_id) ON DELETE SET NULL,
  policy_scope text NOT NULL DEFAULT 'order',
  execution_mode text NOT NULL DEFAULT 'template_query_lite',
  output_boundary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  export_control_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  step_up_required boolean NOT NULL DEFAULT false,
  attestation_required boolean NOT NULL DEFAULT false,
  approval_ticket_id uuid REFERENCES ops.approval_ticket(approval_ticket_id) ON DELETE SET NULL,
  policy_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.attestation_record (
  attestation_record_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  query_run_id uuid REFERENCES delivery.query_execution_run(query_run_id) ON DELETE SET NULL,
  sandbox_session_id uuid REFERENCES delivery.sandbox_session(sandbox_session_id) ON DELETE SET NULL,
  environment_id uuid REFERENCES core.execution_environment(environment_id) ON DELETE SET NULL,
  attestation_type text NOT NULL DEFAULT 'execution_receipt',
  attestation_uri text,
  attestation_hash text,
  verifier_ref text,
  verified_at timestamptz,
  status text NOT NULL DEFAULT 'pending',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.result_disclosure_review (
  result_disclosure_review_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  query_run_id uuid REFERENCES delivery.query_execution_run(query_run_id) ON DELETE SET NULL,
  report_artifact_id uuid REFERENCES delivery.report_artifact(report_artifact_id) ON DELETE SET NULL,
  result_object_id uuid REFERENCES delivery.storage_object(object_id) ON DELETE SET NULL,
  review_status text NOT NULL DEFAULT 'pending',
  masking_level text NOT NULL DEFAULT 'masked',
  export_scope text NOT NULL DEFAULT 'summary_only',
  reviewer_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  approval_ticket_id uuid REFERENCES ops.approval_ticket(approval_ticket_id) ON DELETE SET NULL,
  review_notes text,
  decision_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  reviewed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.destruction_attestation (
  destruction_attestation_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  object_id uuid REFERENCES delivery.storage_object(object_id) ON DELETE SET NULL,
  ref_type text NOT NULL DEFAULT 'delivery_object',
  retention_action text NOT NULL DEFAULT 'destroy',
  attestation_uri text,
  attestation_hash text,
  executed_by_type text NOT NULL DEFAULT 'platform',
  executed_by_id uuid,
  approval_ticket_id uuid REFERENCES ops.approval_ticket(approval_ticket_id) ON DELETE SET NULL,
  executed_at timestamptz,
  status text NOT NULL DEFAULT 'pending',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE delivery.query_execution_run
  ADD COLUMN IF NOT EXISTS masked_level text NOT NULL DEFAULT 'masked',
  ADD COLUMN IF NOT EXISTS export_scope text NOT NULL DEFAULT 'none',
  ADD COLUMN IF NOT EXISTS approval_ticket_id uuid REFERENCES ops.approval_ticket(approval_ticket_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS sensitive_policy_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE delivery.delivery_record
  ADD COLUMN IF NOT EXISTS sensitive_delivery_mode text NOT NULL DEFAULT 'standard',
  ADD COLUMN IF NOT EXISTS disclosure_review_status text NOT NULL DEFAULT 'not_required';

ALTER TABLE delivery.api_credential
  ADD COLUMN IF NOT EXISTS sensitive_scope_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE delivery.sandbox_workspace
  ADD COLUMN IF NOT EXISTS sensitive_boundary_level text NOT NULL DEFAULT 'standard';

CREATE INDEX IF NOT EXISTS idx_sensitive_handling_policy_asset_version
  ON catalog.sensitive_handling_policy(asset_version_id, status);
CREATE INDEX IF NOT EXISTS idx_legal_basis_evidence_asset_version
  ON contract.legal_basis_evidence(asset_version_id, status);
CREATE INDEX IF NOT EXISTS idx_safe_preview_artifact_asset_version
  ON catalog.safe_preview_artifact(asset_version_id, status);
CREATE INDEX IF NOT EXISTS idx_sensitive_execution_policy_order
  ON delivery.sensitive_execution_policy(order_id, status);
CREATE INDEX IF NOT EXISTS idx_attestation_record_order
  ON delivery.attestation_record(order_id, status);
CREATE INDEX IF NOT EXISTS idx_result_disclosure_review_order
  ON delivery.result_disclosure_review(order_id, review_status);
CREATE INDEX IF NOT EXISTS idx_destruction_attestation_order
  ON delivery.destruction_attestation(order_id, status);

CREATE TRIGGER trg_sensitive_handling_policy_updated_at BEFORE UPDATE ON catalog.sensitive_handling_policy
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_legal_basis_evidence_updated_at BEFORE UPDATE ON contract.legal_basis_evidence
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_safe_preview_artifact_updated_at BEFORE UPDATE ON catalog.safe_preview_artifact
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_sensitive_execution_policy_updated_at BEFORE UPDATE ON delivery.sensitive_execution_policy
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_attestation_record_updated_at BEFORE UPDATE ON delivery.attestation_record
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_result_disclosure_review_updated_at BEFORE UPDATE ON delivery.result_disclosure_review
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_destruction_attestation_updated_at BEFORE UPDATE ON delivery.destruction_attestation
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
