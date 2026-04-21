DELETE FROM ops.event_route_policy
WHERE (aggregate_type, event_type) IN (
  ('trade.acceptance_record', 'acceptance.passed'),
  ('trade.acceptance_record', 'acceptance.rejected')
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
    'trade.acceptance_record',
    'acceptance.passed',
    'business',
    'async_evidence',
    'kafka',
    'dtp.outbox.domain-events',
    'aggregate_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.delivery"}'::jsonb
  ),
  (
    'trade.acceptance_record',
    'acceptance.rejected',
    'business',
    'async_evidence',
    'kafka',
    'dtp.outbox.domain-events',
    'aggregate_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.delivery"}'::jsonb
  );
