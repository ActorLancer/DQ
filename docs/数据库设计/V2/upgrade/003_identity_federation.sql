ALTER TABLE iam.sso_connection
  ADD COLUMN IF NOT EXISTS saml_entity_id text,
  ADD COLUMN IF NOT EXISTS saml_metadata_digest text,
  ADD COLUMN IF NOT EXISTS scim_base_url text,
  ADD COLUMN IF NOT EXISTS scim_token_ref text,
  ADD COLUMN IF NOT EXISTS provisioning_mode text NOT NULL DEFAULT 'manual';

CREATE TABLE IF NOT EXISTS iam.provisioning_job (
  provisioning_job_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  sso_connection_id uuid NOT NULL REFERENCES iam.sso_connection(sso_connection_id) ON DELETE CASCADE,
  job_type text NOT NULL DEFAULT 'scim_sync',
  trigger_source text NOT NULL DEFAULT 'manual',
  job_status text NOT NULL DEFAULT 'queued',
  started_at timestamptz,
  completed_at timestamptz,
  result_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_by_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_provisioning_job_connection_status
ON iam.provisioning_job(sso_connection_id, job_status);

CREATE TRIGGER trg_provisioning_job_updated_at BEFORE UPDATE ON iam.provisioning_job
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
