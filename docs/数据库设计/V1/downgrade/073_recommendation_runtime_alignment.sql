DELETE FROM ops.event_route_policy
WHERE (aggregate_type, event_type) IN (
  ('recommend.behavior_event', 'recommend.behavior_recorded')
);

DROP INDEX IF EXISTS recommend.idx_behavior_event_request_result;
DROP INDEX IF EXISTS recommend.idx_behavior_event_idempotency;
