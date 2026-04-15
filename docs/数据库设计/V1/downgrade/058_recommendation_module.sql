DROP TRIGGER IF EXISTS trg_recommend_behavior_event_outbox ON recommend.behavior_event;
DROP TRIGGER IF EXISTS trg_recommend_behavior_event_cohort ON recommend.behavior_event;
DROP TRIGGER IF EXISTS trg_recommend_behavior_event_profile ON recommend.behavior_event;

DROP FUNCTION IF EXISTS recommend.tg_update_cohort_popularity();
DROP FUNCTION IF EXISTS recommend.tg_refresh_subject_profile();

DROP SCHEMA IF EXISTS recommend CASCADE;

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
