DELETE FROM ops.event_route_policy
WHERE (aggregate_type, event_type) IN (
  ('trade.acceptance_record', 'acceptance.passed'),
  ('trade.acceptance_record', 'acceptance.rejected')
);
