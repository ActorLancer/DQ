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

CREATE OR REPLACE FUNCTION common.tg_write_outbox()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_ref_id uuid;
  v_payload jsonb;
  v_request_id text;
  v_trace_id text;
  v_idempotency_key text;
  v_target_topic text;
  v_partition_key text;
  v_payload_hash text;
BEGIN
  v_payload := to_jsonb(NEW);
  v_ref_id := COALESCE(
    NEW.order_id,
    NEW.product_id,
    NEW.case_id,
    NEW.audit_id,
    NEW.billing_event_id,
    NEW.delivery_id,
    NEW.payment_intent_id,
    NEW.refund_intent_id,
    NEW.payout_instruction_id,
    NEW.reconciliation_statement_id,
    NEW.crypto_transfer_id
  );
  v_request_id := v_payload ->> 'request_id';
  v_trace_id := COALESCE(v_payload ->> 'trace_id', v_payload ->> 'event_trace_id');
  v_idempotency_key := v_payload ->> 'idempotency_key';
  v_target_topic := replace(TG_TABLE_SCHEMA || '.' || TG_TABLE_NAME, '.', '_');
  v_partition_key := COALESCE(v_ref_id::text, v_request_id, gen_random_uuid()::text);
  v_payload_hash := encode(digest(v_payload::text, 'sha256'), 'hex');

  INSERT INTO ops.outbox_event (
    outbox_event_id,
    aggregate_type,
    aggregate_id,
    event_type,
    payload,
    status,
    created_at,
    event_schema_version,
    request_id,
    trace_id,
    idempotency_key,
    authority_scope,
    source_of_truth,
    proof_commit_policy,
    target_bus,
    target_topic,
    partition_key,
    ordering_key,
    payload_hash
  )
  VALUES (
    gen_random_uuid(),
    TG_TABLE_SCHEMA || '.' || TG_TABLE_NAME,
    v_ref_id,
    TG_OP,
    v_payload,
    'pending',
    now(),
    'v1',
    v_request_id,
    v_trace_id,
    v_idempotency_key,
    'business',
    'database',
    COALESCE(v_payload ->> 'proof_commit_policy', 'async_evidence'),
    'kafka',
    v_target_topic,
    v_partition_key,
    v_partition_key,
    v_payload_hash
  );
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_order_outbox ON trade.order_main;
DROP TRIGGER IF EXISTS trg_product_outbox ON catalog.product;
DROP TRIGGER IF EXISTS trg_dispute_outbox ON support.dispute_case;
DROP TRIGGER IF EXISTS trg_payment_intent_outbox ON payment.payment_intent;
DROP TRIGGER IF EXISTS trg_payout_instruction_outbox ON payment.payout_instruction;

CREATE TRIGGER trg_order_outbox AFTER INSERT OR UPDATE ON trade.order_main
FOR EACH ROW EXECUTE FUNCTION common.tg_write_outbox();

CREATE TRIGGER trg_product_outbox AFTER INSERT OR UPDATE ON catalog.product
FOR EACH ROW EXECUTE FUNCTION common.tg_write_outbox();

CREATE TRIGGER trg_dispute_outbox AFTER INSERT OR UPDATE ON support.dispute_case
FOR EACH ROW EXECUTE FUNCTION common.tg_write_outbox();

CREATE TRIGGER trg_payment_intent_outbox AFTER INSERT OR UPDATE ON payment.payment_intent
FOR EACH ROW EXECUTE FUNCTION common.tg_write_outbox();

CREATE TRIGGER trg_payout_instruction_outbox AFTER INSERT OR UPDATE ON payment.payout_instruction
FOR EACH ROW EXECUTE FUNCTION common.tg_write_outbox();
