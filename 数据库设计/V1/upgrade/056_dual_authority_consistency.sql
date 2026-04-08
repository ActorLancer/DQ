ALTER TABLE trade.order_main
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS business_state_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'n/a',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE contract.digital_contract
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS business_state_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'n/a',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE delivery.delivery_record
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS business_state_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'n/a',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE billing.settlement_record
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS business_state_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE payment.payment_intent
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS business_state_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE payment.refund_intent
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS business_state_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE payment.payout_instruction
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'dual_layer',
  ADD COLUMN IF NOT EXISTS business_state_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS proof_commit_state text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS external_fact_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE chain.chain_anchor
  ADD COLUMN IF NOT EXISTS authority_model text NOT NULL DEFAULT 'proof_layer',
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz;

ALTER TABLE ops.outbox_event
  ADD COLUMN IF NOT EXISTS event_schema_version text NOT NULL DEFAULT 'v1',
  ADD COLUMN IF NOT EXISTS request_id text,
  ADD COLUMN IF NOT EXISTS trace_id text,
  ADD COLUMN IF NOT EXISTS idempotency_key text,
  ADD COLUMN IF NOT EXISTS authority_scope text NOT NULL DEFAULT 'business',
  ADD COLUMN IF NOT EXISTS source_of_truth text NOT NULL DEFAULT 'database',
  ADD COLUMN IF NOT EXISTS proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  ADD COLUMN IF NOT EXISTS target_bus text NOT NULL DEFAULT 'kafka',
  ADD COLUMN IF NOT EXISTS target_topic text,
  ADD COLUMN IF NOT EXISTS partition_key text,
  ADD COLUMN IF NOT EXISTS ordering_key text,
  ADD COLUMN IF NOT EXISTS payload_hash text,
  ADD COLUMN IF NOT EXISTS lock_owner text,
  ADD COLUMN IF NOT EXISTS locked_at timestamptz,
  ADD COLUMN IF NOT EXISTS published_at timestamptz,
  ADD COLUMN IF NOT EXISTS max_retries integer NOT NULL DEFAULT 16,
  ADD COLUMN IF NOT EXISTS last_error_code text,
  ADD COLUMN IF NOT EXISTS last_error_message text,
  ADD COLUMN IF NOT EXISTS dead_lettered_at timestamptz;

ALTER TABLE ops.dead_letter_event
  ADD COLUMN IF NOT EXISTS request_id text,
  ADD COLUMN IF NOT EXISTS trace_id text,
  ADD COLUMN IF NOT EXISTS authority_scope text NOT NULL DEFAULT 'business',
  ADD COLUMN IF NOT EXISTS source_of_truth text NOT NULL DEFAULT 'database',
  ADD COLUMN IF NOT EXISTS target_bus text NOT NULL DEFAULT 'kafka',
  ADD COLUMN IF NOT EXISTS target_topic text,
  ADD COLUMN IF NOT EXISTS failure_stage text,
  ADD COLUMN IF NOT EXISTS first_failed_at timestamptz NOT NULL DEFAULT now(),
  ADD COLUMN IF NOT EXISTS last_failed_at timestamptz,
  ADD COLUMN IF NOT EXISTS reprocess_status text NOT NULL DEFAULT 'not_reprocessed',
  ADD COLUMN IF NOT EXISTS reprocessed_at timestamptz;

CREATE TABLE IF NOT EXISTS ops.event_route_policy (
  event_route_policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  aggregate_type text NOT NULL,
  event_type text NOT NULL,
  authority_scope text NOT NULL DEFAULT 'business',
  proof_commit_policy text NOT NULL DEFAULT 'async_evidence',
  target_bus text NOT NULL DEFAULT 'kafka',
  target_topic text NOT NULL,
  partition_key_template text,
  ordering_scope text,
  consumer_group_hint text,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_event_route_policy UNIQUE (aggregate_type, event_type, target_bus, target_topic)
);

CREATE TABLE IF NOT EXISTS ops.outbox_publish_attempt (
  outbox_publish_attempt_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  outbox_event_id uuid NOT NULL REFERENCES ops.outbox_event(outbox_event_id) ON DELETE CASCADE,
  worker_id text,
  target_bus text NOT NULL DEFAULT 'kafka',
  target_topic text,
  attempt_no integer NOT NULL DEFAULT 1,
  result_code text NOT NULL DEFAULT 'pending',
  error_code text,
  error_message text,
  attempted_at timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS ops.consumer_idempotency_record (
  consumer_idempotency_record_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  consumer_name text NOT NULL,
  event_id uuid NOT NULL,
  aggregate_type text,
  aggregate_id uuid,
  trace_id text,
  result_code text NOT NULL DEFAULT 'processed',
  processed_at timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  CONSTRAINT uq_consumer_event UNIQUE (consumer_name, event_id)
);

CREATE INDEX IF NOT EXISTS idx_order_main_reconcile ON trade.order_main (reconcile_status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_payment_intent_reconcile ON payment.payment_intent (reconcile_status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_outbox_pending ON ops.outbox_event (status, available_at, created_at);
CREATE INDEX IF NOT EXISTS idx_outbox_topic_pending ON ops.outbox_event (target_topic, status, available_at);
CREATE INDEX IF NOT EXISTS idx_outbox_trace ON ops.outbox_event (trace_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_dead_letter_reprocess ON ops.dead_letter_event (reprocess_status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_publish_attempt_event ON ops.outbox_publish_attempt (outbox_event_id, attempt_no DESC);
CREATE INDEX IF NOT EXISTS idx_consumer_idempotency_consumer ON ops.consumer_idempotency_record (consumer_name, processed_at DESC);

CREATE TRIGGER trg_event_route_policy_updated_at BEFORE UPDATE ON ops.event_route_policy
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

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
