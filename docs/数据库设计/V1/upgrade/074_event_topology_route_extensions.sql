DELETE FROM ops.event_route_policy
WHERE (aggregate_type, event_type) IN (
  ('notification.dispatch_request', 'notification.requested'),
  ('audit.anchor_batch', 'audit.anchor_requested'),
  ('chain.chain_anchor', 'fabric.proof_submit_requested')
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
    'notification.dispatch_request',
    'notification.requested',
    'business',
    'async_evidence',
    'kafka',
    'dtp.notification.dispatch',
    'aggregate_id',
    'partition_key',
    'cg-notification-worker',
    'active',
    '{"producer_service":"platform-core.integration"}'::jsonb
  ),
  (
    'audit.anchor_batch',
    'audit.anchor_requested',
    'governance',
    'async_evidence',
    'kafka',
    'dtp.audit.anchor',
    'aggregate_id',
    'partition_key',
    'cg-fabric-adapter',
    'active',
    '{"producer_service":"platform-core.audit"}'::jsonb
  ),
  (
    'chain.chain_anchor',
    'fabric.proof_submit_requested',
    'governance',
    'async_evidence',
    'kafka',
    'dtp.fabric.requests',
    'aggregate_id',
    'partition_key',
    'cg-fabric-adapter',
    'active',
    '{"producer_service":"platform-core.integration"}'::jsonb
  );
