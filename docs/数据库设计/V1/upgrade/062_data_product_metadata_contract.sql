ALTER TABLE catalog.product
  ADD COLUMN IF NOT EXISTS product_subtitle text,
  ADD COLUMN IF NOT EXISTS industry_tags text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS use_case_tags text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS target_buyer_tags text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS prohibited_use_tags text[] NOT NULL DEFAULT '{}';

ALTER TABLE catalog.asset_version
  ADD COLUMN IF NOT EXISTS processing_mode text NOT NULL DEFAULT 'seller_self_managed',
  ADD COLUMN IF NOT EXISTS lineage_hash text;

ALTER TABLE catalog.asset_sample
  ADD COLUMN IF NOT EXISTS masking_status text NOT NULL DEFAULT 'masked',
  ADD COLUMN IF NOT EXISTS preview_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb;

CREATE TABLE IF NOT EXISTS catalog.product_metadata_profile (
  product_metadata_profile_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id uuid NOT NULL REFERENCES catalog.product(product_id) ON DELETE CASCADE,
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  metadata_version_no integer NOT NULL DEFAULT 1,
  business_description_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  data_content_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  structure_description_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  quality_description_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  compliance_description_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  delivery_description_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  version_description_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  authorization_description_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  responsibility_description_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  processing_overview_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (product_id, metadata_version_no)
);

CREATE TABLE IF NOT EXISTS catalog.asset_field_definition (
  field_definition_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  object_name text,
  field_name text NOT NULL,
  field_path text NOT NULL,
  field_type text NOT NULL,
  is_nullable boolean NOT NULL DEFAULT true,
  is_primary_key boolean NOT NULL DEFAULT false,
  is_partition_key boolean NOT NULL DEFAULT false,
  is_time_field boolean NOT NULL DEFAULT false,
  code_rule text,
  unit_text text,
  enum_values_json jsonb NOT NULL DEFAULT '[]'::jsonb,
  description text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (asset_version_id, field_path)
);

CREATE TABLE IF NOT EXISTS catalog.asset_quality_report (
  quality_report_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  report_no integer NOT NULL DEFAULT 1,
  report_type text NOT NULL DEFAULT 'seller_declared',
  coverage_range_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  freshness_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  missing_rate numeric(8,6),
  duplicate_rate numeric(8,6),
  anomaly_rate numeric(8,6),
  sampling_method text,
  assessed_at timestamptz,
  assessor_org_id uuid REFERENCES core.organization(org_id),
  report_uri text,
  report_hash text,
  metrics_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (asset_version_id, report_no)
);

CREATE TABLE IF NOT EXISTS catalog.asset_processing_job (
  processing_job_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  output_asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  processing_mode text NOT NULL,
  processor_org_id uuid REFERENCES core.organization(org_id),
  executor_type text NOT NULL DEFAULT 'seller',
  job_name text,
  transform_spec_version text,
  desensitization_profile text,
  standardization_profile text,
  labeling_profile text,
  model_artifact_ref text,
  evidence_uri text,
  evidence_hash text,
  started_at timestamptz,
  completed_at timestamptz,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.asset_processing_input (
  processing_input_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  processing_job_id uuid NOT NULL REFERENCES catalog.asset_processing_job(processing_job_id) ON DELETE CASCADE,
  input_asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE RESTRICT,
  input_role text NOT NULL DEFAULT 'primary_input',
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (processing_job_id, input_asset_version_id, input_role)
);

CREATE TABLE IF NOT EXISTS contract.data_contract (
  data_contract_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  product_id uuid REFERENCES catalog.product(product_id) ON DELETE CASCADE,
  sku_id uuid REFERENCES catalog.product_sku(sku_id) ON DELETE CASCADE,
  contract_name text NOT NULL,
  version_no integer NOT NULL DEFAULT 1,
  contract_scope text NOT NULL DEFAULT 'sku',
  business_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  structure_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  quality_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  compliance_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  delivery_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  version_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  acceptance_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  rights_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  responsibility_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  processing_terms_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  content_digest text,
  status text NOT NULL DEFAULT 'draft',
  effective_from timestamptz,
  effective_to timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT data_contract_target_ck CHECK (asset_version_id IS NOT NULL OR product_id IS NOT NULL OR sku_id IS NOT NULL)
);

ALTER TABLE contract.digital_contract
  ADD COLUMN IF NOT EXISTS data_contract_id uuid REFERENCES contract.data_contract(data_contract_id),
  ADD COLUMN IF NOT EXISTS data_contract_digest text;

CREATE INDEX IF NOT EXISTS idx_product_metadata_profile_product
  ON catalog.product_metadata_profile(product_id, status);
CREATE INDEX IF NOT EXISTS idx_asset_field_definition_version
  ON catalog.asset_field_definition(asset_version_id, field_name);
CREATE INDEX IF NOT EXISTS idx_asset_quality_report_version
  ON catalog.asset_quality_report(asset_version_id, report_no DESC);
CREATE INDEX IF NOT EXISTS idx_asset_processing_job_output
  ON catalog.asset_processing_job(output_asset_version_id, status);
CREATE INDEX IF NOT EXISTS idx_asset_processing_input_job
  ON catalog.asset_processing_input(processing_job_id);
CREATE INDEX IF NOT EXISTS idx_data_contract_product
  ON contract.data_contract(product_id, status, version_no DESC);
CREATE INDEX IF NOT EXISTS idx_data_contract_sku
  ON contract.data_contract(sku_id, status, version_no DESC);

CREATE TRIGGER trg_product_metadata_profile_updated_at BEFORE UPDATE ON catalog.product_metadata_profile
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_asset_field_definition_updated_at BEFORE UPDATE ON catalog.asset_field_definition
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_asset_quality_report_updated_at BEFORE UPDATE ON catalog.asset_quality_report
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_asset_processing_job_updated_at BEFORE UPDATE ON catalog.asset_processing_job
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_data_contract_updated_at BEFORE UPDATE ON contract.data_contract
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
