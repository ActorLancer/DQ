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

CREATE TABLE IF NOT EXISTS iam.invitation (
  invitation_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid NOT NULL REFERENCES core.organization(org_id) ON DELETE CASCADE,
  invited_email citext,
  invited_phone text,
  invited_role_snapshot jsonb NOT NULL DEFAULT '[]'::jsonb,
  invitation_type text NOT NULL DEFAULT 'member',
  token_hash text NOT NULL UNIQUE,
  status text NOT NULL DEFAULT 'pending',
  expires_at timestamptz NOT NULL,
  accepted_by_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_by_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT invitation_target_ck CHECK (invited_email IS NOT NULL OR invited_phone IS NOT NULL)
);

CREATE TABLE IF NOT EXISTS iam.identity_proof (
  identity_proof_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subject_type text NOT NULL,
  subject_id uuid NOT NULL,
  proof_type text NOT NULL,
  assurance_level text,
  proof_status text NOT NULL DEFAULT 'pending',
  verifier_type text,
  evidence_ref text,
  provider_ref text,
  verified_at timestamptz,
  expires_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS iam.auth_method_binding (
  auth_method_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  auth_method_type text NOT NULL,
  provider_key text NOT NULL,
  external_subject text,
  credential_ref text,
  is_primary boolean NOT NULL DEFAULT false,
  status text NOT NULL DEFAULT 'active',
  last_used_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS iam.mfa_authenticator (
  mfa_authenticator_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  authenticator_type text NOT NULL,
  device_label text,
  credential_id text,
  public_key_ref text,
  secret_ref text,
  status text NOT NULL DEFAULT 'active',
  last_verified_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS iam.trusted_device (
  trusted_device_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  device_fingerprint_hash text NOT NULL,
  device_name text,
  platform text,
  browser text,
  trust_level text NOT NULL DEFAULT 'unknown',
  status text NOT NULL DEFAULT 'active',
  last_ip inet,
  last_country_code text,
  last_seen_at timestamptz,
  expires_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS iam.refresh_token_family (
  refresh_token_family_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  trusted_device_id uuid REFERENCES iam.trusted_device(trusted_device_id) ON DELETE SET NULL,
  client_type text NOT NULL DEFAULT 'web',
  current_token_hash text NOT NULL UNIQUE,
  status text NOT NULL DEFAULT 'active',
  issued_at timestamptz NOT NULL DEFAULT now(),
  last_rotated_at timestamptz NOT NULL DEFAULT now(),
  expires_at timestamptz NOT NULL,
  revoked_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS iam.user_session (
  session_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  trusted_device_id uuid REFERENCES iam.trusted_device(trusted_device_id) ON DELETE SET NULL,
  refresh_token_family_id uuid REFERENCES iam.refresh_token_family(refresh_token_family_id) ON DELETE SET NULL,
  login_method text NOT NULL,
  auth_context_level text NOT NULL DEFAULT 'aal1',
  session_type text NOT NULL DEFAULT 'web',
  current_ip inet,
  current_country_code text,
  session_status text NOT NULL DEFAULT 'active',
  last_activity_at timestamptz NOT NULL DEFAULT now(),
  expires_at timestamptz NOT NULL,
  revoked_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS iam.sso_connection (
  sso_connection_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid NOT NULL REFERENCES core.organization(org_id) ON DELETE CASCADE,
  connection_name text NOT NULL,
  protocol_type text NOT NULL DEFAULT 'oidc',
  issuer text,
  client_id text,
  client_secret_ref text,
  metadata_url text,
  redirect_uri text,
  jit_provisioning boolean NOT NULL DEFAULT false,
  status text NOT NULL DEFAULT 'draft',
  claim_mapping jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_sso_connection_org_name UNIQUE (org_id, connection_name)
);

CREATE TABLE IF NOT EXISTS iam.external_identity_binding (
  external_identity_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  sso_connection_id uuid NOT NULL REFERENCES iam.sso_connection(sso_connection_id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  external_subject text NOT NULL,
  external_email citext,
  attrs_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'active',
  last_login_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_external_identity_binding_subject UNIQUE (sso_connection_id, external_subject)
);

CREATE TABLE IF NOT EXISTS iam.step_up_challenge (
  step_up_challenge_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  challenge_type text NOT NULL,
  target_action text NOT NULL,
  target_ref_type text,
  target_ref_id uuid,
  challenge_status text NOT NULL DEFAULT 'pending',
  expires_at timestamptz NOT NULL,
  completed_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS iam.fabric_ca_registry (
  fabric_ca_registry_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  registry_name text NOT NULL,
  msp_id text NOT NULL,
  ca_name text,
  ca_url text,
  ca_type text NOT NULL DEFAULT 'fabric-ca',
  status text NOT NULL DEFAULT 'draft',
  enrollment_profile text,
  config_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_fabric_ca_registry_name UNIQUE (msp_id, registry_name)
);

CREATE TABLE IF NOT EXISTS iam.certificate_record (
  certificate_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  fabric_ca_registry_id uuid NOT NULL REFERENCES iam.fabric_ca_registry(fabric_ca_registry_id) ON DELETE CASCADE,
  certificate_scope text NOT NULL DEFAULT 'fabric',
  serial_number text NOT NULL,
  certificate_digest text NOT NULL UNIQUE,
  subject_dn text,
  issuer_dn text,
  key_ref text,
  not_before timestamptz,
  not_after timestamptz,
  status text NOT NULL DEFAULT 'issued',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_certificate_serial_issuer UNIQUE (serial_number, issuer_dn)
);

CREATE TABLE IF NOT EXISTS iam.fabric_identity_binding (
  fabric_identity_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  fabric_ca_registry_id uuid NOT NULL REFERENCES iam.fabric_ca_registry(fabric_ca_registry_id) ON DELETE CASCADE,
  org_id uuid REFERENCES core.organization(org_id) ON DELETE CASCADE,
  user_id uuid REFERENCES core.user_account(user_id) ON DELETE CASCADE,
  service_identity_id uuid REFERENCES core.service_identity(service_identity_id) ON DELETE CASCADE,
  certificate_id uuid UNIQUE REFERENCES iam.certificate_record(certificate_id) ON DELETE SET NULL,
  msp_id text NOT NULL,
  affiliation text,
  enrollment_id text NOT NULL,
  identity_type text NOT NULL,
  attrs_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'active',
  issued_at timestamptz,
  revoked_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_fabric_identity_enrollment UNIQUE (fabric_ca_registry_id, enrollment_id),
  CONSTRAINT fabric_identity_binding_owner_ck CHECK (num_nonnulls(org_id, user_id, service_identity_id) >= 1)
);

CREATE TABLE IF NOT EXISTS iam.certificate_revocation_record (
  certificate_revocation_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  certificate_id uuid NOT NULL UNIQUE REFERENCES iam.certificate_record(certificate_id) ON DELETE CASCADE,
  revoked_by_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  revoke_reason text,
  revoke_source text,
  revoked_at timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
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
CREATE INDEX IF NOT EXISTS idx_invitation_org_id ON iam.invitation(org_id);
CREATE INDEX IF NOT EXISTS idx_identity_proof_subject ON iam.identity_proof(subject_type, subject_id);
CREATE INDEX IF NOT EXISTS idx_auth_method_binding_user_id ON iam.auth_method_binding(user_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_mfa_authenticator_credential ON iam.mfa_authenticator(credential_id) WHERE credential_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_trusted_device_user_fingerprint ON iam.trusted_device(user_id, device_fingerprint_hash);
CREATE INDEX IF NOT EXISTS idx_refresh_token_family_user_id ON iam.refresh_token_family(user_id);
CREATE INDEX IF NOT EXISTS idx_user_session_user_status ON iam.user_session(user_id, session_status);
CREATE INDEX IF NOT EXISTS idx_sso_connection_org_id ON iam.sso_connection(org_id);
CREATE INDEX IF NOT EXISTS idx_external_identity_binding_user_id ON iam.external_identity_binding(user_id);
CREATE INDEX IF NOT EXISTS idx_step_up_challenge_user_status ON iam.step_up_challenge(user_id, challenge_status);
CREATE INDEX IF NOT EXISTS idx_fabric_ca_registry_org_id ON iam.fabric_ca_registry(org_id);
CREATE INDEX IF NOT EXISTS idx_certificate_record_registry_status ON iam.certificate_record(fabric_ca_registry_id, status);
CREATE INDEX IF NOT EXISTS idx_fabric_identity_binding_user_id ON iam.fabric_identity_binding(user_id);
CREATE INDEX IF NOT EXISTS idx_fabric_identity_binding_service_id ON iam.fabric_identity_binding(service_identity_id);
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
CREATE TRIGGER trg_invitation_updated_at BEFORE UPDATE ON iam.invitation
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_identity_proof_updated_at BEFORE UPDATE ON iam.identity_proof
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_auth_method_binding_updated_at BEFORE UPDATE ON iam.auth_method_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_mfa_authenticator_updated_at BEFORE UPDATE ON iam.mfa_authenticator
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_trusted_device_updated_at BEFORE UPDATE ON iam.trusted_device
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_refresh_token_family_updated_at BEFORE UPDATE ON iam.refresh_token_family
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_user_session_updated_at BEFORE UPDATE ON iam.user_session
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_sso_connection_updated_at BEFORE UPDATE ON iam.sso_connection
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_external_identity_binding_updated_at BEFORE UPDATE ON iam.external_identity_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_step_up_challenge_updated_at BEFORE UPDATE ON iam.step_up_challenge
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_fabric_ca_registry_updated_at BEFORE UPDATE ON iam.fabric_ca_registry
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_certificate_record_updated_at BEFORE UPDATE ON iam.certificate_record
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_fabric_identity_binding_updated_at BEFORE UPDATE ON iam.fabric_identity_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_subject_role_binding_updated_at BEFORE UPDATE ON authz.subject_role_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
-- Trust-boundary baseline sync: identity schema remains compatible; storage-trust roles/permissions are added in seed files.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
