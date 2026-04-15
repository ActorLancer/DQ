CREATE TABLE IF NOT EXISTS catalog.storage_namespace (
  storage_namespace_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  namespace_name text NOT NULL UNIQUE,
  provider_type text NOT NULL DEFAULT 's3_compatible',
  namespace_kind text NOT NULL DEFAULT 'product',
  bucket_name text NOT NULL,
  prefix_rule text,
  region_code text,
  encryption_scope text NOT NULL DEFAULT 'server_side',
  immutability_mode text NOT NULL DEFAULT 'mutable',
  lifecycle_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  retention_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.storage_policy_profile (
  storage_policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_org_id uuid NOT NULL REFERENCES core.organization(org_id) ON DELETE CASCADE,
  policy_name text NOT NULL,
  object_family text,
  raw_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  curated_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  product_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  preview_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  delivery_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  archive_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  evidence_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  model_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  query_surface_type text NOT NULL DEFAULT 'download_only',
  encryption_required boolean NOT NULL DEFAULT true,
  preview_allowed boolean NOT NULL DEFAULT true,
  lifecycle_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  retention_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (owner_org_id, policy_name)
);

ALTER TABLE catalog.asset_version
  ADD COLUMN IF NOT EXISTS storage_policy_id uuid REFERENCES catalog.storage_policy_profile(storage_policy_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS query_surface_type text NOT NULL DEFAULT 'download_only';

ALTER TABLE catalog.asset_storage_binding
  ADD COLUMN IF NOT EXISTS storage_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS storage_zone text NOT NULL DEFAULT 'product',
  ADD COLUMN IF NOT EXISTS storage_class text,
  ADD COLUMN IF NOT EXISTS access_path_type text NOT NULL DEFAULT 'object_uri',
  ADD COLUMN IF NOT EXISTS retention_until timestamptz;

ALTER TABLE delivery.storage_object
  ADD COLUMN IF NOT EXISTS storage_namespace_id uuid REFERENCES catalog.storage_namespace(storage_namespace_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS storage_zone text NOT NULL DEFAULT 'delivery',
  ADD COLUMN IF NOT EXISTS storage_class text,
  ADD COLUMN IF NOT EXISTS retention_until timestamptz;

CREATE INDEX IF NOT EXISTS idx_storage_namespace_owner_kind
  ON catalog.storage_namespace(owner_org_id, namespace_kind, status);
CREATE INDEX IF NOT EXISTS idx_storage_policy_profile_owner
  ON catalog.storage_policy_profile(owner_org_id, status);
CREATE INDEX IF NOT EXISTS idx_asset_version_storage_policy
  ON catalog.asset_version(storage_policy_id, query_surface_type);
CREATE INDEX IF NOT EXISTS idx_asset_storage_binding_namespace
  ON catalog.asset_storage_binding(storage_namespace_id, storage_zone);
CREATE INDEX IF NOT EXISTS idx_storage_object_namespace
  ON delivery.storage_object(storage_namespace_id, storage_zone);

CREATE TRIGGER trg_storage_namespace_updated_at BEFORE UPDATE ON catalog.storage_namespace
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_storage_policy_profile_updated_at BEFORE UPDATE ON catalog.storage_policy_profile
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
