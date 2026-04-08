DROP TRIGGER IF EXISTS trg_risk_auth_policy_updated_at ON iam.risk_auth_policy;
DROP TABLE IF EXISTS iam.risk_auth_policy CASCADE;

ALTER TABLE IF EXISTS iam.external_identity_binding
  DROP COLUMN IF EXISTS federation_scope,
  DROP COLUMN IF EXISTS partner_id;
