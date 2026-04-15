DELETE FROM authz.role_permission
WHERE role_key IN (
  'platform_admin','platform_reviewer','platform_risk_settlement','platform_audit_security',
  'tenant_admin','seller_operator','buyer_operator','tenant_developer','tenant_audit_readonly',
  'tenant_app_identity','platform_service_identity','regulator_readonly'
);
