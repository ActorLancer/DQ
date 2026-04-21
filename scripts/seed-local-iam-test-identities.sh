#!/usr/bin/env bash
set -euo pipefail

# IAM-018: one-click local identity preparation for demo personas.
# Usage:
#   DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab ./scripts/seed-local-iam-test-identities.sh

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
if [[ -z "${DATABASE_URL:-}" && -f "${ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${ENV_FILE}"
  set +a
fi

DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
PSQL="${PSQL_BIN:-psql}"

echo "[info] seeding local IAM test identities into: ${DATABASE_URL}"

"${PSQL}" "${DATABASE_URL}" -v ON_ERROR_STOP=1 <<'SQL'
INSERT INTO core.organization (
  org_id, org_name, org_type, status, real_name_status, compliance_level, country_code, region_code, metadata
) VALUES
  ('10000000-0000-0000-0000-000000000101'::uuid, 'Luna Seller Org', 'tenant', 'active', 'approved', 'L2', 'CN', 'CN-SH', '{"seed":"iam018-bootstrap","tenant_type":"seller"}'::jsonb),
  ('10000000-0000-0000-0000-000000000102'::uuid, 'Luna Buyer Org', 'tenant', 'active', 'approved', 'L2', 'CN', 'CN-BJ', '{"seed":"iam018-bootstrap","tenant_type":"buyer"}'::jsonb),
  ('10000000-0000-0000-0000-000000000103'::uuid, 'Luna Platform Ops Org', 'platform', 'active', 'approved', 'L3', 'CN', 'CN-SH', '{"seed":"iam018-bootstrap","tenant_type":"ops"}'::jsonb)
ON CONFLICT (org_id) DO UPDATE
SET
  org_name = EXCLUDED.org_name,
  org_type = EXCLUDED.org_type,
  status = EXCLUDED.status,
  real_name_status = EXCLUDED.real_name_status,
  compliance_level = EXCLUDED.compliance_level,
  country_code = EXCLUDED.country_code,
  region_code = EXCLUDED.region_code,
  metadata = EXCLUDED.metadata,
  updated_at = now();

INSERT INTO core.department (department_id, org_id, department_name, status) VALUES
  ('10000000-0000-0000-0000-000000000201'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Seller Business', 'active'),
  ('10000000-0000-0000-0000-000000000202'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, 'Buyer Procurement', 'active'),
  ('10000000-0000-0000-0000-000000000203'::uuid, '10000000-0000-0000-0000-000000000103'::uuid, 'Platform Operations', 'active')
ON CONFLICT (department_id) DO UPDATE
SET
  org_id = EXCLUDED.org_id,
  department_name = EXCLUDED.department_name,
  status = EXCLUDED.status,
  updated_at = now();

INSERT INTO core.user_account (
  user_id, org_id, department_id, login_id, display_name, user_type, status, mfa_status, email, phone, attrs
) VALUES
  ('10000000-0000-0000-0000-000000000351'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '10000000-0000-0000-0000-000000000201'::uuid, 'seller.admin@luna.local', 'Seller Admin', 'human', 'active', 'enabled', 'seller.admin@luna.local', '13800000101', '{"seed":"iam018","persona":"seller_admin"}'::jsonb),
  ('10000000-0000-0000-0000-000000000352'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000202'::uuid, 'buyer.admin.v2@luna.local', 'Buyer Admin V2', 'human', 'active', 'enabled', 'buyer.admin.v2@luna.local', '13800000102', '{"seed":"iam018","persona":"buyer_admin"}'::jsonb),
  ('10000000-0000-0000-0000-000000000353'::uuid, '10000000-0000-0000-0000-000000000103'::uuid, '10000000-0000-0000-0000-000000000203'::uuid, 'ops.admin@luna.local', 'Ops Admin', 'human', 'active', 'enabled', 'ops.admin@luna.local', '13800000103', '{"seed":"iam018","persona":"ops_admin"}'::jsonb),
  ('10000000-0000-0000-0000-000000000354'::uuid, '10000000-0000-0000-0000-000000000103'::uuid, '10000000-0000-0000-0000-000000000203'::uuid, 'auditor.admin@luna.local', 'Auditor Admin', 'human', 'active', 'enabled', 'auditor.admin@luna.local', '13800000104', '{"seed":"iam018","persona":"auditor_admin"}'::jsonb),
  ('10000000-0000-0000-0000-000000000355'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000202'::uuid, 'developer.admin@luna.local', 'Developer Admin', 'human', 'active', 'enabled', 'developer.admin@luna.local', '13800000105', '{"seed":"iam018","persona":"developer_admin"}'::jsonb)
ON CONFLICT (user_id) DO UPDATE
SET
  org_id = EXCLUDED.org_id,
  department_id = EXCLUDED.department_id,
  login_id = EXCLUDED.login_id,
  display_name = EXCLUDED.display_name,
  status = EXCLUDED.status,
  mfa_status = EXCLUDED.mfa_status,
  email = EXCLUDED.email,
  phone = EXCLUDED.phone,
  attrs = EXCLUDED.attrs,
  updated_at = now();

INSERT INTO authz.subject_role_binding (
  subject_role_binding_id, subject_type, subject_id, role_key, scope_type, scope_id, status, attrs
) VALUES
  ('10000000-0000-0000-0000-000000000551'::uuid, 'user', '10000000-0000-0000-0000-000000000351'::uuid, 'seller_operator', 'org', '10000000-0000-0000-0000-000000000101'::uuid, 'active', '{"seed":"iam018"}'::jsonb),
  ('10000000-0000-0000-0000-000000000552'::uuid, 'user', '10000000-0000-0000-0000-000000000352'::uuid, 'tenant_admin', 'org', '10000000-0000-0000-0000-000000000102'::uuid, 'active', '{"seed":"iam018"}'::jsonb),
  ('10000000-0000-0000-0000-000000000553'::uuid, 'user', '10000000-0000-0000-0000-000000000353'::uuid, 'platform_admin', 'org', '10000000-0000-0000-0000-000000000103'::uuid, 'active', '{"seed":"iam018"}'::jsonb),
  ('10000000-0000-0000-0000-000000000554'::uuid, 'user', '10000000-0000-0000-0000-000000000354'::uuid, 'platform_audit_security', 'org', '10000000-0000-0000-0000-000000000103'::uuid, 'active', '{"seed":"iam018"}'::jsonb),
  ('10000000-0000-0000-0000-000000000555'::uuid, 'user', '10000000-0000-0000-0000-000000000355'::uuid, 'tenant_developer', 'org', '10000000-0000-0000-0000-000000000102'::uuid, 'active', '{"seed":"iam018"}'::jsonb)
ON CONFLICT (subject_role_binding_id) DO UPDATE
SET
  role_key = EXCLUDED.role_key,
  scope_id = EXCLUDED.scope_id,
  status = EXCLUDED.status,
  attrs = EXCLUDED.attrs,
  updated_at = now();

SELECT login_id, display_name, org_id::text AS org_id
FROM core.user_account
WHERE user_id IN (
  '10000000-0000-0000-0000-000000000351'::uuid,
  '10000000-0000-0000-0000-000000000352'::uuid,
  '10000000-0000-0000-0000-000000000353'::uuid,
  '10000000-0000-0000-0000-000000000354'::uuid,
  '10000000-0000-0000-0000-000000000355'::uuid
)
ORDER BY login_id;
SQL

echo "[ok] IAM local test identities ready"
