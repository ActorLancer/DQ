CREATE TABLE IF NOT EXISTS core.organization (
  org_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_name text NOT NULL,
  org_type text NOT NULL,
  status text NOT NULL DEFAULT 'draft',
  real_name_status text NOT NULL DEFAULT 'pending',
  compliance_level text,
  credit_level integer NOT NULL DEFAULT 0,
  risk_level integer NOT NULL DEFAULT 0,
  partner_type text,
  industry_tags text[] NOT NULL DEFAULT '{}',
  country_code text,
  region_code text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS core.department (
  department_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid NOT NULL REFERENCES core.organization(org_id) ON DELETE CASCADE,
  department_name text NOT NULL,
  parent_department_id uuid REFERENCES core.department(department_id),
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS core.user_account (
  user_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid NOT NULL REFERENCES core.organization(org_id) ON DELETE CASCADE,
  department_id uuid REFERENCES core.department(department_id),
  login_id citext NOT NULL UNIQUE,
  display_name text NOT NULL,
  user_type text NOT NULL DEFAULT 'human',
  status text NOT NULL DEFAULT 'active',
  mfa_status text NOT NULL DEFAULT 'pending',
  email citext,
  phone text,
  last_login_at timestamptz,
  attrs jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS core.did_binding (
  did_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid REFERENCES core.organization(org_id) ON DELETE CASCADE,
  user_id uuid REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  chain_id text NOT NULL,
  did_value text NOT NULL,
  cert_sn text,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT did_binding_owner_ck CHECK (org_id IS NOT NULL OR user_id IS NOT NULL)
);

CREATE TABLE IF NOT EXISTS core.application (
  app_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid NOT NULL REFERENCES core.organization(org_id) ON DELETE CASCADE,
  app_name text NOT NULL,
  app_type text NOT NULL DEFAULT 'api_client',
  status text NOT NULL DEFAULT 'active',
  client_id text NOT NULL UNIQUE,
  client_secret_hash text,
  ip_whitelist cidr[] NOT NULL DEFAULT '{}',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS core.service_identity (
  service_identity_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  service_name text NOT NULL UNIQUE,
  identity_type text NOT NULL,
  status text NOT NULL DEFAULT 'active',
  cert_sn text,
  public_key_ref text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS core.connector (
  connector_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  connector_name text NOT NULL,
  connector_type text NOT NULL,
  status text NOT NULL DEFAULT 'draft',
  version text,
  network_zone text,
  health_status text NOT NULL DEFAULT 'unknown',
  endpoint_ref text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS core.execution_environment (
  environment_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  connector_id uuid REFERENCES core.connector(connector_id) ON DELETE SET NULL,
  environment_name text NOT NULL,
  environment_type text NOT NULL,
  status text NOT NULL DEFAULT 'draft',
  network_zone text,
  region_code text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS authz.role_definition (
  role_key text PRIMARY KEY,
  role_name text NOT NULL,
  role_scope text NOT NULL,
  stage_from text NOT NULL DEFAULT 'V1',
  status text NOT NULL DEFAULT 'active',
  description text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS authz.permission_definition (
  permission_code text PRIMARY KEY,
  domain_name text NOT NULL,
  resource_name text NOT NULL,
  action_name text NOT NULL,
  stage_from text NOT NULL DEFAULT 'V1',
  risk_level text NOT NULL DEFAULT 'normal',
  description text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS authz.role_permission (
  role_key text NOT NULL REFERENCES authz.role_definition(role_key) ON DELETE CASCADE,
  permission_code text NOT NULL REFERENCES authz.permission_definition(permission_code) ON DELETE CASCADE,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (role_key, permission_code)
);

CREATE TABLE IF NOT EXISTS authz.subject_role_binding (
  subject_role_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subject_type text NOT NULL,
  subject_id uuid NOT NULL,
  role_key text NOT NULL REFERENCES authz.role_definition(role_key) ON DELETE CASCADE,
  scope_type text NOT NULL,
  scope_id uuid,
  status text NOT NULL DEFAULT 'active',
  attrs jsonb NOT NULL DEFAULT '{}'::jsonb,
  effective_from timestamptz NOT NULL DEFAULT now(),
  effective_to timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_department_org_id ON core.department(org_id);
CREATE INDEX IF NOT EXISTS idx_user_account_org_id ON core.user_account(org_id);
CREATE INDEX IF NOT EXISTS idx_application_org_id ON core.application(org_id);
CREATE INDEX IF NOT EXISTS idx_subject_role_binding_subject ON authz.subject_role_binding(subject_type, subject_id);

CREATE TRIGGER trg_organization_updated_at BEFORE UPDATE ON core.organization
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_department_updated_at BEFORE UPDATE ON core.department
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_user_account_updated_at BEFORE UPDATE ON core.user_account
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_application_updated_at BEFORE UPDATE ON core.application
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_connector_updated_at BEFORE UPDATE ON core.connector
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_execution_environment_updated_at BEFORE UPDATE ON core.execution_environment
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_subject_role_binding_updated_at BEFORE UPDATE ON authz.subject_role_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
-- Trust-boundary baseline sync: identity schema remains compatible; storage-trust roles/permissions are added in seed files.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
