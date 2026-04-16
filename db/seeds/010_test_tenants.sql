INSERT INTO core.organization (
  org_id, org_name, org_type, status, real_name_status, compliance_level, credit_level, risk_level, partner_type, industry_tags, country_code, region_code, metadata
) VALUES
  ('10000000-0000-0000-0000-000000000101'::uuid, 'Luna Seller Org', 'tenant', 'active', 'approved', 'L2', 75, 1, 'seller', ARRAY['industry_manufacturing','industry_finance'], 'CN', 'CN-SH', '{"seed":"db027","tenant_type":"seller"}'::jsonb),
  ('10000000-0000-0000-0000-000000000102'::uuid, 'Luna Buyer Org', 'tenant', 'active', 'approved', 'L2', 72, 1, 'buyer', ARRAY['industry_retail','industry_transport'], 'CN', 'CN-BJ', '{"seed":"db027","tenant_type":"buyer"}'::jsonb),
  ('10000000-0000-0000-0000-000000000103'::uuid, 'Luna Platform Ops Org', 'platform', 'active', 'approved', 'L3', 90, 1, 'platform', ARRAY['industry_finance'], 'CN', 'CN-SH', '{"seed":"db027","tenant_type":"ops"}'::jsonb)
ON CONFLICT (org_id) DO UPDATE
SET
  org_name = EXCLUDED.org_name,
  org_type = EXCLUDED.org_type,
  status = EXCLUDED.status,
  real_name_status = EXCLUDED.real_name_status,
  compliance_level = EXCLUDED.compliance_level,
  credit_level = EXCLUDED.credit_level,
  risk_level = EXCLUDED.risk_level,
  partner_type = EXCLUDED.partner_type,
  industry_tags = EXCLUDED.industry_tags,
  country_code = EXCLUDED.country_code,
  region_code = EXCLUDED.region_code,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

INSERT INTO core.department (department_id, org_id, department_name, status) VALUES
  ('10000000-0000-0000-0000-000000000201'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Seller Business', 'active'),
  ('10000000-0000-0000-0000-000000000202'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, 'Buyer Procurement', 'active'),
  ('10000000-0000-0000-0000-000000000203'::uuid, '10000000-0000-0000-0000-000000000103'::uuid, 'Platform Operations', 'active')
ON CONFLICT (department_id) DO UPDATE
SET
  org_id = EXCLUDED.org_id,
  department_name = EXCLUDED.department_name,
  status = EXCLUDED.status,
  updated_at = NOW();

INSERT INTO core.user_account (
  user_id, org_id, department_id, login_id, display_name, user_type, status, mfa_status, email, phone, attrs
) VALUES
  ('10000000-0000-0000-0000-000000000301'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '10000000-0000-0000-0000-000000000201'::uuid, 'seller.operator@luna.local', 'Seller Operator', 'human', 'active', 'enabled', 'seller.operator@luna.local', '13800000001', '{"seed":"db027","persona":"seller_operator"}'::jsonb),
  ('10000000-0000-0000-0000-000000000302'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000202'::uuid, 'buyer.operator@luna.local', 'Buyer Operator', 'human', 'active', 'enabled', 'buyer.operator@luna.local', '13800000002', '{"seed":"db027","persona":"buyer_operator"}'::jsonb),
  ('10000000-0000-0000-0000-000000000303'::uuid, '10000000-0000-0000-0000-000000000103'::uuid, '10000000-0000-0000-0000-000000000203'::uuid, 'platform.ops@luna.local', 'Platform Ops', 'human', 'active', 'enabled', 'platform.ops@luna.local', '13800000003', '{"seed":"db027","persona":"platform_admin"}'::jsonb),
  ('10000000-0000-0000-0000-000000000304'::uuid, '10000000-0000-0000-0000-000000000103'::uuid, '10000000-0000-0000-0000-000000000203'::uuid, 'platform.audit@luna.local', 'Platform Auditor', 'human', 'active', 'enabled', 'platform.audit@luna.local', '13800000004', '{"seed":"db027","persona":"platform_audit_security"}'::jsonb),
  ('10000000-0000-0000-0000-000000000305'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000202'::uuid, 'buyer.developer@luna.local', 'Buyer Developer', 'human', 'active', 'enabled', 'buyer.developer@luna.local', '13800000005', '{"seed":"db027","persona":"tenant_developer"}'::jsonb),
  ('10000000-0000-0000-0000-000000000306'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000202'::uuid, 'buyer.admin@luna.local', 'Buyer Admin', 'human', 'active', 'enabled', 'buyer.admin@luna.local', '13800000006', '{"seed":"db027","persona":"tenant_admin"}'::jsonb)
ON CONFLICT (user_id) DO UPDATE
SET
  org_id = EXCLUDED.org_id,
  department_id = EXCLUDED.department_id,
  login_id = EXCLUDED.login_id,
  display_name = EXCLUDED.display_name,
  user_type = EXCLUDED.user_type,
  status = EXCLUDED.status,
  mfa_status = EXCLUDED.mfa_status,
  email = EXCLUDED.email,
  phone = EXCLUDED.phone,
  attrs = EXCLUDED.attrs,
  updated_at = NOW();

INSERT INTO core.application (
  app_id, org_id, app_name, app_type, status, client_id, client_secret_hash, metadata
) VALUES
  ('10000000-0000-0000-0000-000000000401'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, 'Luna Buyer API App', 'api_client', 'active', 'luna-buyer-api-app', 'seeded-secret-hash', '{"seed":"db027","purpose":"api_delivery_demo"}'::jsonb)
ON CONFLICT (app_id) DO UPDATE
SET
  org_id = EXCLUDED.org_id,
  app_name = EXCLUDED.app_name,
  app_type = EXCLUDED.app_type,
  status = EXCLUDED.status,
  client_id = EXCLUDED.client_id,
  client_secret_hash = EXCLUDED.client_secret_hash,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

INSERT INTO authz.subject_role_binding (
  subject_role_binding_id, subject_type, subject_id, role_key, scope_type, scope_id, status, attrs
) VALUES
  ('10000000-0000-0000-0000-000000000501'::uuid, 'user', '10000000-0000-0000-0000-000000000301'::uuid, 'seller_operator', 'org', '10000000-0000-0000-0000-000000000101'::uuid, 'active', '{"seed":"db027"}'::jsonb),
  ('10000000-0000-0000-0000-000000000502'::uuid, 'user', '10000000-0000-0000-0000-000000000302'::uuid, 'buyer_operator', 'org', '10000000-0000-0000-0000-000000000102'::uuid, 'active', '{"seed":"db027"}'::jsonb),
  ('10000000-0000-0000-0000-000000000503'::uuid, 'user', '10000000-0000-0000-0000-000000000303'::uuid, 'platform_admin', 'org', '10000000-0000-0000-0000-000000000103'::uuid, 'active', '{"seed":"db027"}'::jsonb),
  ('10000000-0000-0000-0000-000000000504'::uuid, 'user', '10000000-0000-0000-0000-000000000304'::uuid, 'platform_audit_security', 'org', '10000000-0000-0000-0000-000000000103'::uuid, 'active', '{"seed":"db027"}'::jsonb),
  ('10000000-0000-0000-0000-000000000505'::uuid, 'user', '10000000-0000-0000-0000-000000000305'::uuid, 'tenant_developer', 'org', '10000000-0000-0000-0000-000000000102'::uuid, 'active', '{"seed":"db027"}'::jsonb),
  ('10000000-0000-0000-0000-000000000506'::uuid, 'user', '10000000-0000-0000-0000-000000000306'::uuid, 'tenant_admin', 'org', '10000000-0000-0000-0000-000000000102'::uuid, 'active', '{"seed":"db027"}'::jsonb)
ON CONFLICT (subject_role_binding_id) DO UPDATE
SET
  subject_type = EXCLUDED.subject_type,
  subject_id = EXCLUDED.subject_id,
  role_key = EXCLUDED.role_key,
  scope_type = EXCLUDED.scope_type,
  scope_id = EXCLUDED.scope_id,
  status = EXCLUDED.status,
  attrs = EXCLUDED.attrs,
  updated_at = NOW();
