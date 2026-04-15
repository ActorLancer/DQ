DROP TRIGGER IF EXISTS trg_provisioning_job_updated_at ON iam.provisioning_job;
DROP TABLE IF EXISTS iam.provisioning_job CASCADE;

ALTER TABLE IF EXISTS iam.sso_connection
  DROP COLUMN IF EXISTS provisioning_mode,
  DROP COLUMN IF EXISTS scim_token_ref,
  DROP COLUMN IF EXISTS scim_base_url,
  DROP COLUMN IF EXISTS saml_metadata_digest,
  DROP COLUMN IF EXISTS saml_entity_id;
