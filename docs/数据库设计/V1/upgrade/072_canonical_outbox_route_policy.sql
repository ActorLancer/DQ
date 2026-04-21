DELETE FROM ops.event_route_policy
WHERE (aggregate_type, event_type) IN (
  ('trade.order', 'trade.order.created'),
  ('delivery.delivery_record', 'delivery.task.auto_created'),
  ('delivery.delivery_record', 'delivery.committed'),
  ('trade.order_main', 'billing.trigger.bridge'),
  ('billing.billing_event', 'billing.event.recorded'),
  ('billing.refund_record', 'billing.event.recorded'),
  ('billing.compensation_record', 'billing.event.recorded'),
  ('payment.payout_instruction', 'billing.event.recorded'),
  ('billing.settlement_record', 'settlement.created'),
  ('billing.settlement_record', 'settlement.completed'),
  ('support.dispute_case', 'dispute.created'),
  ('support.dispute_case', 'dispute.resolved'),
  ('product', 'search.product.changed'),
  ('product', 'catalog.product.submitted'),
  ('product', 'catalog.product.status.changed')
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
    'trade.order',
    'trade.order.created',
    'business',
    'async_evidence',
    'kafka',
    'dtp.outbox.domain-events',
    'aggregate_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.order"}'::jsonb
  ),
  (
    'delivery.delivery_record',
    'delivery.task.auto_created',
    'business',
    'async_evidence',
    'kafka',
    'dtp.outbox.domain-events',
    'order_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.order"}'::jsonb
  ),
  (
    'delivery.delivery_record',
    'delivery.committed',
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
    'trade.order_main',
    'billing.trigger.bridge',
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
    'billing.billing_event',
    'billing.event.recorded',
    'business',
    'pending_fabric_anchor',
    'kafka',
    'dtp.outbox.domain-events',
    'order_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.billing"}'::jsonb
  ),
  (
    'billing.refund_record',
    'billing.event.recorded',
    'business',
    'pending_fabric_anchor',
    'kafka',
    'dtp.outbox.domain-events',
    'order_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.billing"}'::jsonb
  ),
  (
    'billing.compensation_record',
    'billing.event.recorded',
    'business',
    'pending_fabric_anchor',
    'kafka',
    'dtp.outbox.domain-events',
    'order_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.billing"}'::jsonb
  ),
  (
    'payment.payout_instruction',
    'billing.event.recorded',
    'business',
    'pending_fabric_anchor',
    'kafka',
    'dtp.outbox.domain-events',
    'order_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.billing"}'::jsonb
  ),
  (
    'billing.settlement_record',
    'settlement.created',
    'business',
    'pending_fabric_anchor',
    'kafka',
    'dtp.outbox.domain-events',
    'order_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.billing"}'::jsonb
  ),
  (
    'billing.settlement_record',
    'settlement.completed',
    'business',
    'pending_fabric_anchor',
    'kafka',
    'dtp.outbox.domain-events',
    'order_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.billing"}'::jsonb
  ),
  (
    'support.dispute_case',
    'dispute.created',
    'business',
    'pending_fabric_anchor',
    'kafka',
    'dtp.outbox.domain-events',
    'aggregate_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.billing"}'::jsonb
  ),
  (
    'support.dispute_case',
    'dispute.resolved',
    'business',
    'pending_fabric_anchor',
    'kafka',
    'dtp.outbox.domain-events',
    'aggregate_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.billing"}'::jsonb
  ),
  (
    'product',
    'search.product.changed',
    'business',
    'async_evidence',
    'kafka',
    'dtp.search.sync',
    'aggregate_id',
    'partition_key',
    'cg-search-indexer',
    'active',
    '{"producer_service":"platform-core.catalog"}'::jsonb
  ),
  (
    'product',
    'catalog.product.submitted',
    'business',
    'async_evidence',
    'kafka',
    'dtp.outbox.domain-events',
    'aggregate_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.catalog"}'::jsonb
  ),
  (
    'product',
    'catalog.product.status.changed',
    'business',
    'async_evidence',
    'kafka',
    'dtp.outbox.domain-events',
    'aggregate_id',
    'partition_key',
    'cg-outbox-publisher',
    'active',
    '{"producer_service":"platform-core.catalog"}'::jsonb
  );

DROP TRIGGER IF EXISTS trg_order_outbox ON trade.order_main;
DROP TRIGGER IF EXISTS trg_product_outbox ON catalog.product;
DROP TRIGGER IF EXISTS trg_dispute_outbox ON support.dispute_case;
DROP TRIGGER IF EXISTS trg_payment_intent_outbox ON payment.payment_intent;
DROP TRIGGER IF EXISTS trg_payout_instruction_outbox ON payment.payout_instruction;
DROP TRIGGER IF EXISTS trg_recommend_behavior_event_outbox ON recommend.behavior_event;

CREATE OR REPLACE FUNCTION common.tg_write_outbox()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  RAISE EXCEPTION
    'common.tg_write_outbox is retired; use ops.event_route_policy + application canonical outbox writer instead';
END;
$$;
