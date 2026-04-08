DELETE FROM authz.role_permission
WHERE role_key IN (
  'crosschain_admin','witness_admin','graph_risk_operator',
  'regulatory_collab_admin','mutual_recognition_admin',
  'connector_admin','ecosystem_admin','cross_platform_trust_admin','digital_asset_settlement_admin','regulator_operator'
);
