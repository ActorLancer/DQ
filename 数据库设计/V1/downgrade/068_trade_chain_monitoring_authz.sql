DELETE FROM authz.role_permission
WHERE permission_code IN (
  'ops.trade_monitor.read',
  'ops.trade_monitor.manage',
  'ops.external_fact.read',
  'ops.external_fact.manage',
  'ops.monitor_policy.read',
  'ops.monitor_policy.manage',
  'ops.projection_gap.read',
  'ops.projection_gap.manage',
  'risk.fairness_incident.read',
  'risk.fairness_incident.handle'
);

DELETE FROM authz.permission_definition
WHERE permission_code IN (
  'ops.trade_monitor.read',
  'ops.trade_monitor.manage',
  'ops.external_fact.read',
  'ops.external_fact.manage',
  'ops.monitor_policy.read',
  'ops.monitor_policy.manage',
  'ops.projection_gap.read',
  'ops.projection_gap.manage',
  'risk.fairness_incident.read',
  'risk.fairness_incident.handle'
);
