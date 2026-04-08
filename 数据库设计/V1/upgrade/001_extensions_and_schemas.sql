CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS btree_gist;
CREATE EXTENSION IF NOT EXISTS vector;

CREATE SCHEMA IF NOT EXISTS common;
CREATE SCHEMA IF NOT EXISTS core;
CREATE SCHEMA IF NOT EXISTS authz;
CREATE SCHEMA IF NOT EXISTS catalog;
CREATE SCHEMA IF NOT EXISTS contract;
CREATE SCHEMA IF NOT EXISTS review;
CREATE SCHEMA IF NOT EXISTS trade;
CREATE SCHEMA IF NOT EXISTS delivery;
CREATE SCHEMA IF NOT EXISTS billing;
CREATE SCHEMA IF NOT EXISTS payment;
CREATE SCHEMA IF NOT EXISTS support;
CREATE SCHEMA IF NOT EXISTS risk;
CREATE SCHEMA IF NOT EXISTS audit;
CREATE SCHEMA IF NOT EXISTS chain;
CREATE SCHEMA IF NOT EXISTS search;
CREATE SCHEMA IF NOT EXISTS developer;
CREATE SCHEMA IF NOT EXISTS ops;
CREATE SCHEMA IF NOT EXISTS ml;
CREATE SCHEMA IF NOT EXISTS crosschain;
CREATE SCHEMA IF NOT EXISTS ecosystem;

CREATE OR REPLACE FUNCTION common.tg_set_updated_at()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION common.tg_order_status_history()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  IF TG_OP = 'INSERT' THEN
    INSERT INTO trade.order_status_history (
      order_status_history_id, order_id, old_status, new_status, changed_by_type,
      changed_by_id, reason_code, changed_at
    )
    VALUES (
      gen_random_uuid(), NEW.order_id, NULL, NEW.status, 'system',
      NULL, 'ORDER_CREATED', now()
    );
    RETURN NEW;
  END IF;

  IF NEW.status IS DISTINCT FROM OLD.status THEN
    INSERT INTO trade.order_status_history (
      order_status_history_id, order_id, old_status, new_status, changed_by_type,
      changed_by_id, reason_code, changed_at
    )
    VALUES (
      gen_random_uuid(), NEW.order_id, OLD.status, NEW.status, 'system',
      NULL, COALESCE(NEW.last_reason_code, 'STATUS_CHANGED'), now()
    );
  END IF;
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION common.tg_dispute_status_history()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  IF TG_OP = 'INSERT' THEN
    INSERT INTO support.dispute_status_history (
      dispute_status_history_id, case_id, old_status, new_status, changed_at
    )
    VALUES (gen_random_uuid(), NEW.case_id, NULL, NEW.status, now());
    RETURN NEW;
  END IF;

  IF NEW.status IS DISTINCT FROM OLD.status THEN
    INSERT INTO support.dispute_status_history (
      dispute_status_history_id, case_id, old_status, new_status, changed_at
    )
    VALUES (gen_random_uuid(), NEW.case_id, OLD.status, NEW.status, now());
  END IF;
  RETURN NEW;
END;
$$;

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
-- Trust-boundary baseline sync: schemas/functions remain valid; no extra structural change required in this file.
