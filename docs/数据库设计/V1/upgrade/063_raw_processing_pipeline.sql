ALTER TABLE catalog.asset_version
  ADD COLUMN IF NOT EXISTS processing_stage text NOT NULL DEFAULT 'raw_registered',
  ADD COLUMN IF NOT EXISTS standardization_status text NOT NULL DEFAULT 'not_started';

CREATE TABLE IF NOT EXISTS catalog.raw_ingest_batch (
  raw_ingest_batch_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  asset_id uuid REFERENCES catalog.data_asset(asset_id) ON DELETE SET NULL,
  ingest_source_type text NOT NULL,
  declared_object_family text,
  source_declared_rights_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  ingest_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  created_by uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.raw_object_manifest (
  raw_object_manifest_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  raw_ingest_batch_id uuid NOT NULL REFERENCES catalog.raw_ingest_batch(raw_ingest_batch_id) ON DELETE CASCADE,
  storage_binding_id uuid REFERENCES catalog.asset_storage_binding(asset_storage_binding_id) ON DELETE SET NULL,
  object_name text NOT NULL,
  object_uri text,
  mime_type text,
  container_type text,
  byte_size bigint,
  object_hash text,
  source_time_range_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  manifest_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'registered',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.format_detection_result (
  format_detection_result_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  raw_object_manifest_id uuid NOT NULL REFERENCES catalog.raw_object_manifest(raw_object_manifest_id) ON DELETE CASCADE,
  detected_object_family text NOT NULL,
  detected_format text,
  schema_hint_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  recommended_processing_path text,
  classification_confidence numeric(8,4),
  detected_at timestamptz,
  status text NOT NULL DEFAULT 'detected',
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.extraction_job (
  extraction_job_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  raw_object_manifest_id uuid NOT NULL REFERENCES catalog.raw_object_manifest(raw_object_manifest_id) ON DELETE CASCADE,
  asset_version_id uuid REFERENCES catalog.asset_version(asset_version_id) ON DELETE SET NULL,
  job_type text NOT NULL,
  job_config_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  result_summary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  output_uri text,
  output_hash text,
  status text NOT NULL DEFAULT 'draft',
  started_at timestamptz,
  completed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS catalog.preview_artifact (
  preview_artifact_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  raw_object_manifest_id uuid REFERENCES catalog.raw_object_manifest(raw_object_manifest_id) ON DELETE SET NULL,
  preview_type text NOT NULL,
  preview_uri text,
  preview_hash text,
  preview_payload jsonb,
  preview_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_raw_ingest_batch_owner_status
  ON catalog.raw_ingest_batch(owner_org_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_raw_ingest_batch_asset
  ON catalog.raw_ingest_batch(asset_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_raw_object_manifest_batch
  ON catalog.raw_object_manifest(raw_ingest_batch_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_raw_object_manifest_hash
  ON catalog.raw_object_manifest(object_hash);
CREATE INDEX IF NOT EXISTS idx_format_detection_manifest
  ON catalog.format_detection_result(raw_object_manifest_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_extraction_job_manifest
  ON catalog.extraction_job(raw_object_manifest_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_extraction_job_version
  ON catalog.extraction_job(asset_version_id, job_type, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_preview_artifact_version
  ON catalog.preview_artifact(asset_version_id, preview_type, status, created_at DESC);

CREATE TRIGGER trg_raw_ingest_batch_updated_at BEFORE UPDATE ON catalog.raw_ingest_batch
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_raw_object_manifest_updated_at BEFORE UPDATE ON catalog.raw_object_manifest
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_extraction_job_updated_at BEFORE UPDATE ON catalog.extraction_job
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_preview_artifact_updated_at BEFORE UPDATE ON catalog.preview_artifact
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
