DELETE FROM authz.role_permission
WHERE role_key IN (
  'platform_super_admin','subject_reviewer','product_reviewer','compliance_reviewer',
  'risk_operator','finance_operator','dispute_operator','audit_admin','tenant_admin',
  'payment_channel_admin','pricing_admin','reconciliation_operator',
  'data_custody_admin','key_custody_admin','data_governance_admin','listing_manager',
  'seller_storage_operator','retention_admin','procurement_manager','finance_manager',
  'developer_admin','business_analyst','regulator_observer'
);
