DELETE FROM ops.event_route_policy
WHERE (aggregate_type, event_type) IN (
  ('notification.dispatch_request', 'notification.requested'),
  ('audit.anchor_batch', 'audit.anchor_requested'),
  ('chain.chain_anchor', 'fabric.proof_submit_requested')
);
