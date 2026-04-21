DELETE FROM ops.event_route_policy
WHERE (aggregate_type, event_type) IN (
  ('recommend.behavior_event', 'recommend.behavior_recorded')
);

INSERT INTO ops.event_route_policy (
  aggregate_type,
  event_type,
  authority_scope,
  proof_commit_policy,
  target_bus,
  target_topic,
  partition_key_template,
  ordering_scope,
  consumer_group_hint,
  status,
  metadata
)
VALUES
  (
    'recommend.behavior_event',
    'recommend.behavior_recorded',
    'business',
    'async_evidence',
    'kafka',
    'dtp.recommend.behavior',
    'aggregate_id',
    'partition_key',
    'cg-recommendation-aggregator',
    'active',
    '{"producer_service":"platform-core.recommendation"}'::jsonb
  );

DROP TRIGGER IF EXISTS trg_recommend_behavior_event_outbox ON recommend.behavior_event;

CREATE INDEX IF NOT EXISTS idx_behavior_event_request_result
  ON recommend.behavior_event(recommendation_request_id, recommendation_result_id, occurred_at DESC);

CREATE INDEX IF NOT EXISTS idx_behavior_event_idempotency
  ON recommend.behavior_event((attrs ->> 'idempotency_key'))
  WHERE attrs ? 'idempotency_key';
