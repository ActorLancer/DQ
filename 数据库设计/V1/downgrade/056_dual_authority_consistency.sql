DROP TRIGGER IF EXISTS trg_event_route_policy_updated_at ON ops.event_route_policy;

DROP INDEX IF EXISTS idx_consumer_idempotency_consumer;
DROP INDEX IF EXISTS idx_publish_attempt_event;
DROP INDEX IF EXISTS idx_dead_letter_reprocess;
DROP INDEX IF EXISTS idx_outbox_trace;
DROP INDEX IF EXISTS idx_outbox_topic_pending;
DROP INDEX IF EXISTS idx_outbox_pending;
DROP INDEX IF EXISTS idx_payment_intent_reconcile;
DROP INDEX IF EXISTS idx_order_main_reconcile;

DROP TABLE IF EXISTS ops.consumer_idempotency_record CASCADE;
DROP TABLE IF EXISTS ops.outbox_publish_attempt CASCADE;
DROP TABLE IF EXISTS ops.event_route_policy CASCADE;

ALTER TABLE ops.dead_letter_event
  DROP COLUMN IF EXISTS reprocessed_at,
  DROP COLUMN IF EXISTS reprocess_status,
  DROP COLUMN IF EXISTS last_failed_at,
  DROP COLUMN IF EXISTS first_failed_at,
  DROP COLUMN IF EXISTS failure_stage,
  DROP COLUMN IF EXISTS target_topic,
  DROP COLUMN IF EXISTS target_bus,
  DROP COLUMN IF EXISTS source_of_truth,
  DROP COLUMN IF EXISTS authority_scope,
  DROP COLUMN IF EXISTS trace_id,
  DROP COLUMN IF EXISTS request_id;

ALTER TABLE ops.outbox_event
  DROP COLUMN IF EXISTS dead_lettered_at,
  DROP COLUMN IF EXISTS last_error_message,
  DROP COLUMN IF EXISTS last_error_code,
  DROP COLUMN IF EXISTS max_retries,
  DROP COLUMN IF EXISTS published_at,
  DROP COLUMN IF EXISTS locked_at,
  DROP COLUMN IF EXISTS lock_owner,
  DROP COLUMN IF EXISTS payload_hash,
  DROP COLUMN IF EXISTS ordering_key,
  DROP COLUMN IF EXISTS partition_key,
  DROP COLUMN IF EXISTS target_topic,
  DROP COLUMN IF EXISTS target_bus,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS source_of_truth,
  DROP COLUMN IF EXISTS authority_scope,
  DROP COLUMN IF EXISTS idempotency_key,
  DROP COLUMN IF EXISTS trace_id,
  DROP COLUMN IF EXISTS request_id,
  DROP COLUMN IF EXISTS event_schema_version;

ALTER TABLE chain.chain_anchor
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE payment.payout_instruction
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS business_state_version,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE payment.refund_intent
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS business_state_version,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE payment.payment_intent
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS business_state_version,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE billing.settlement_record
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS business_state_version,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE delivery.delivery_record
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS business_state_version,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE contract.digital_contract
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS business_state_version,
  DROP COLUMN IF EXISTS authority_model;

ALTER TABLE trade.order_main
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status,
  DROP COLUMN IF EXISTS external_fact_status,
  DROP COLUMN IF EXISTS proof_commit_policy,
  DROP COLUMN IF EXISTS proof_commit_state,
  DROP COLUMN IF EXISTS business_state_version,
  DROP COLUMN IF EXISTS authority_model;

CREATE OR REPLACE FUNCTION common.tg_write_outbox()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_ref_id uuid;
BEGIN
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
  INSERT INTO ops.outbox_event (
    outbox_event_id, aggregate_type, aggregate_id, event_type, payload, status, created_at
  )
  VALUES (
    gen_random_uuid(),
    TG_TABLE_SCHEMA || '.' || TG_TABLE_NAME,
    v_ref_id,
    TG_OP,
    to_jsonb(NEW),
    'pending',
    now()
  );
  RETURN NEW;
END;
$$;
