CREATE TABLE IF NOT EXISTS catalog.data_asset (
  asset_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  title text NOT NULL,
  category text NOT NULL,
  sensitivity_level text NOT NULL DEFAULT 'internal',
  status text NOT NULL DEFAULT 'draft',
  storage_mode text NOT NULL DEFAULT 'platform_custody',
  payload_location_type text NOT NULL DEFAULT 'platform_object_storage',
  custody_mode text NOT NULL DEFAULT 'platform_managed',
  key_control_mode text NOT NULL DEFAULT 'seller_managed',
  platform_plaintext_access boolean NOT NULL DEFAULT false,
  platform_unilateral_decrypt_allowed boolean NOT NULL DEFAULT false,
  description text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.asset_version (
  asset_version_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_id uuid NOT NULL REFERENCES catalog.data_asset(asset_id) ON DELETE CASCADE,
  version_no integer NOT NULL,
  schema_version text,
  schema_hash text,
  sample_hash text,
  full_hash text,
  data_size_bytes bigint,
  origin_region text,
  allowed_region text[],
  requires_controlled_execution boolean NOT NULL DEFAULT false,
  trust_boundary_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (asset_id, version_no)
);

CREATE TABLE IF NOT EXISTS catalog.asset_storage_binding (
  asset_storage_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  storage_type text NOT NULL,
  object_uri text NOT NULL,
  payload_role text NOT NULL DEFAULT 'primary_payload',
  managed_by_org_id uuid REFERENCES core.organization(org_id),
  connector_id uuid REFERENCES core.connector(connector_id),
  environment_id uuid REFERENCES core.execution_environment(environment_id),
  access_mode text NOT NULL DEFAULT 'read_only',
  object_hash text,
  encryption_algo text,
  plaintext_visible_to_platform boolean NOT NULL DEFAULT false,
  unilateral_decrypt_allowed boolean NOT NULL DEFAULT false,
  worm_enabled boolean NOT NULL DEFAULT false,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.asset_custody_profile (
  custody_profile_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_id uuid REFERENCES catalog.data_asset(asset_id) ON DELETE CASCADE,
  asset_version_id uuid REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  storage_mode text NOT NULL,
  payload_location_type text NOT NULL,
  custody_mode text NOT NULL,
  key_control_mode text NOT NULL,
  platform_plaintext_access boolean NOT NULL DEFAULT false,
  platform_unilateral_decrypt_allowed boolean NOT NULL DEFAULT false,
  default_delivery_route text NOT NULL DEFAULT 'platform_delivery',
  requires_controlled_execution boolean NOT NULL DEFAULT false,
  retention_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  destroy_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT asset_custody_profile_target_ck CHECK (asset_id IS NOT NULL OR asset_version_id IS NOT NULL),
  UNIQUE (asset_version_id)
);

CREATE TABLE IF NOT EXISTS catalog.asset_trust_evidence (
  trust_evidence_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_id uuid REFERENCES catalog.data_asset(asset_id) ON DELETE CASCADE,
  asset_version_id uuid REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  evidence_type text NOT NULL,
  evidence_scope text NOT NULL DEFAULT 'asset',
  issuer_org_id uuid REFERENCES core.organization(org_id),
  evidence_uri text,
  evidence_hash text,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT asset_trust_evidence_target_ck CHECK (asset_id IS NOT NULL OR asset_version_id IS NOT NULL)
);

CREATE TABLE IF NOT EXISTS catalog.asset_sample (
  asset_sample_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  sample_type text NOT NULL DEFAULT 'json',
  sample_uri text,
  sample_payload jsonb,
  sample_hash text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.asset_structured_dataset (
  dataset_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  dataset_name text NOT NULL,
  storage_mode text NOT NULL DEFAULT 'postgres_jsonb',
  row_count bigint NOT NULL DEFAULT 0,
  schema_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.asset_structured_row (
  row_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  dataset_id uuid NOT NULL REFERENCES catalog.asset_structured_dataset(dataset_id) ON DELETE CASCADE,
  row_no bigint NOT NULL,
  row_payload jsonb NOT NULL,
  embedding vector(1536),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (dataset_id, row_no)
);

CREATE TABLE IF NOT EXISTS catalog.tag (
  tag_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tag_name text NOT NULL UNIQUE,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.product (
  product_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_id uuid NOT NULL REFERENCES catalog.data_asset(asset_id),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id),
  seller_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  title text NOT NULL,
  category text NOT NULL,
  product_type text NOT NULL,
  description text,
  status text NOT NULL DEFAULT 'draft',
  price_mode text NOT NULL DEFAULT 'fixed',
  price numeric(20, 8) NOT NULL DEFAULT 0,
  currency_code text NOT NULL DEFAULT 'CNY',
  delivery_type text NOT NULL,
  allowed_usage text[] NOT NULL DEFAULT '{}',
  searchable_text text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.product_tag (
  product_id uuid NOT NULL REFERENCES catalog.product(product_id) ON DELETE CASCADE,
  tag_id uuid NOT NULL REFERENCES catalog.tag(tag_id) ON DELETE CASCADE,
  PRIMARY KEY (product_id, tag_id)
);

CREATE TABLE IF NOT EXISTS catalog.product_sku (
  sku_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id uuid NOT NULL REFERENCES catalog.product(product_id) ON DELETE CASCADE,
  sku_code text NOT NULL,
  sku_type text NOT NULL,
  unit_name text,
  billing_mode text NOT NULL,
  acceptance_mode text NOT NULL,
  refund_mode text NOT NULL,
  sla_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  quota_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (product_id, sku_code)
);

CREATE TABLE IF NOT EXISTS contract.template_definition (
  template_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  template_type text NOT NULL,
  template_name text NOT NULL,
  version_no integer NOT NULL DEFAULT 1,
  applicable_sku_types text[] NOT NULL DEFAULT '{}',
  configurable_fields jsonb NOT NULL DEFAULT '[]'::jsonb,
  locked_fields jsonb NOT NULL DEFAULT '[]'::jsonb,
  content_digest text,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS contract.template_binding (
  template_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  sku_id uuid NOT NULL REFERENCES catalog.product_sku(sku_id) ON DELETE CASCADE,
  template_id uuid NOT NULL REFERENCES contract.template_definition(template_id) ON DELETE CASCADE,
  binding_type text NOT NULL,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (sku_id, template_id, binding_type)
);

CREATE TABLE IF NOT EXISTS contract.usage_policy (
  policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  policy_name text NOT NULL,
  stage_from text NOT NULL DEFAULT 'V1',
  subject_constraints jsonb NOT NULL DEFAULT '{}'::jsonb,
  usage_constraints jsonb NOT NULL DEFAULT '{}'::jsonb,
  time_constraints jsonb NOT NULL DEFAULT '{}'::jsonb,
  region_constraints jsonb NOT NULL DEFAULT '{}'::jsonb,
  output_constraints jsonb NOT NULL DEFAULT '{}'::jsonb,
  exportable boolean NOT NULL DEFAULT false,
  status text NOT NULL DEFAULT 'draft',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS contract.policy_binding (
  policy_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  policy_id uuid NOT NULL REFERENCES contract.usage_policy(policy_id) ON DELETE CASCADE,
  product_id uuid REFERENCES catalog.product(product_id) ON DELETE CASCADE,
  sku_id uuid REFERENCES catalog.product_sku(sku_id) ON DELETE CASCADE,
  binding_scope text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT policy_binding_target_ck CHECK (product_id IS NOT NULL OR sku_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_asset_version_asset_id ON catalog.asset_version(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_storage_binding_asset_version_id ON catalog.asset_storage_binding(asset_version_id);
CREATE INDEX IF NOT EXISTS idx_asset_custody_profile_asset_id ON catalog.asset_custody_profile(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_trust_evidence_asset_id ON catalog.asset_trust_evidence(asset_id);
CREATE INDEX IF NOT EXISTS idx_product_asset_version_id ON catalog.product(asset_version_id);
CREATE INDEX IF NOT EXISTS idx_product_seller_org_id ON catalog.product(seller_org_id);
CREATE INDEX IF NOT EXISTS idx_product_status_type ON catalog.product(status, product_type);
CREATE INDEX IF NOT EXISTS idx_product_sku_product_id ON catalog.product_sku(product_id);

CREATE TRIGGER trg_data_asset_updated_at BEFORE UPDATE ON catalog.data_asset
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_asset_version_updated_at BEFORE UPDATE ON catalog.asset_version
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_asset_structured_dataset_updated_at BEFORE UPDATE ON catalog.asset_structured_dataset
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_asset_custody_profile_updated_at BEFORE UPDATE ON catalog.asset_custody_profile
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_product_updated_at BEFORE UPDATE ON catalog.product
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_product_sku_updated_at BEFORE UPDATE ON catalog.product_sku
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_template_definition_updated_at BEFORE UPDATE ON contract.template_definition
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_usage_policy_updated_at BEFORE UPDATE ON contract.usage_policy
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
