CREATE TABLE IF NOT EXISTS audit.audit_event (
  audit_id uuid NOT NULL DEFAULT gen_random_uuid(),
  domain_name text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid,
  actor_type text NOT NULL,
  actor_id uuid,
  action_name text NOT NULL,
  result_code text NOT NULL,
  request_id text,
  trace_id text,
  source_ip inet,
  client_fingerprint text,
  tx_hash text,
  evidence_hash text,
  payload_digest text,
  event_time timestamptz NOT NULL DEFAULT now(),
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
) PARTITION BY RANGE (event_time);

CREATE TABLE IF NOT EXISTS audit.audit_event_default
PARTITION OF audit.audit_event DEFAULT;

ALTER TABLE audit.audit_event
  ADD CONSTRAINT audit_event_pk PRIMARY KEY (audit_id, event_time);

CREATE TABLE IF NOT EXISTS audit.evidence_package (
  evidence_package_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  package_type text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  package_digest text,
  storage_uri text,
  created_by uuid REFERENCES core.user_account(user_id),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.outbox_event (
  outbox_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  aggregate_type text NOT NULL,
  aggregate_id uuid,
  event_type text NOT NULL,
  payload jsonb NOT NULL,
  status text NOT NULL DEFAULT 'pending',
  retry_count integer NOT NULL DEFAULT 0,
  available_at timestamptz NOT NULL DEFAULT now(),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.dead_letter_event (
  dead_letter_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  outbox_event_id uuid,
  aggregate_type text NOT NULL,
  aggregate_id uuid,
  event_type text NOT NULL,
  payload jsonb NOT NULL,
  failed_reason text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.job_run (
  job_run_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  job_name text NOT NULL,
  status text NOT NULL DEFAULT 'running',
  request_id text,
  started_at timestamptz NOT NULL DEFAULT now(),
  finished_at timestamptz,
  result_summary jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS ops.system_log (
  system_log_id uuid NOT NULL DEFAULT gen_random_uuid(),
  service_name text NOT NULL,
  log_level text NOT NULL,
  request_id text,
  trace_id text,
  message_text text NOT NULL,
  structured_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
) PARTITION BY RANGE (created_at);

CREATE TABLE IF NOT EXISTS ops.system_log_default
PARTITION OF ops.system_log DEFAULT;

ALTER TABLE ops.system_log
  ADD CONSTRAINT system_log_pk PRIMARY KEY (system_log_id, created_at);

CREATE TABLE IF NOT EXISTS search.product_search_document (
  product_id uuid PRIMARY KEY REFERENCES catalog.product(product_id) ON DELETE CASCADE,
  org_id uuid NOT NULL REFERENCES core.organization(org_id),
  title text NOT NULL,
  category text,
  tags text[] NOT NULL DEFAULT '{}',
  description text,
  searchable_tsv tsvector,
  embedding vector(1536),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_product_search_document_tsv
  ON search.product_search_document USING GIN (searchable_tsv);
CREATE INDEX IF NOT EXISTS idx_product_search_document_embedding
  ON search.product_search_document USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

CREATE TABLE IF NOT EXISTS developer.test_wallet (
  test_wallet_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  wallet_name text NOT NULL,
  chain_id text NOT NULL,
  address text NOT NULL UNIQUE,
  token_code text,
  balance numeric(24, 8) NOT NULL DEFAULT 0,
  faucet_last_at timestamptz,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS developer.test_application (
  test_application_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  app_id uuid REFERENCES core.application(app_id) ON DELETE CASCADE,
  owner_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  scenario_name text,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS developer.mock_provider_binding (
  mock_provider_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  binding_scope text NOT NULL,
  binding_scope_id uuid,
  provider_type text NOT NULL,
  provider_mode text NOT NULL,
  config_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS developer.mock_payment_case (
  mock_payment_case_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  payment_intent_id uuid REFERENCES payment.payment_intent(payment_intent_id) ON DELETE CASCADE,
  provider_key text NOT NULL REFERENCES payment.provider(provider_key),
  scenario_type text NOT NULL,
  delay_seconds integer NOT NULL DEFAULT 0,
  duplicate_webhook boolean NOT NULL DEFAULT false,
  partial_refund_amount numeric(24, 8),
  payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'pending',
  executed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS chain.contract_event_projection (
  contract_event_projection_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  chain_id text NOT NULL,
  contract_name text NOT NULL,
  event_name text NOT NULL,
  ref_type text,
  ref_id uuid,
  tx_hash text,
  block_no bigint,
  event_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  projected_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS chain.chain_anchor (
  chain_anchor_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  chain_id text NOT NULL,
  anchor_type text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid,
  digest text NOT NULL,
  tx_hash text,
  status text NOT NULL DEFAULT 'pending',
  anchored_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION search.tg_refresh_product_search_document()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO search.product_search_document (
    product_id, org_id, title, category, tags, description, searchable_tsv, updated_at
  )
  SELECT
    p.product_id,
    p.seller_org_id,
    p.title,
    p.category,
    COALESCE(array_agg(t.tag_name) FILTER (WHERE t.tag_name IS NOT NULL), '{}'),
    p.description,
    to_tsvector('simple', COALESCE(p.title, '') || ' ' || COALESCE(p.category, '') || ' ' || COALESCE(p.description, '')),
    now()
  FROM catalog.product p
  LEFT JOIN catalog.product_tag pt ON pt.product_id = p.product_id
  LEFT JOIN catalog.tag t ON t.tag_id = pt.tag_id
  WHERE p.product_id = NEW.product_id
  GROUP BY p.product_id
  ON CONFLICT (product_id) DO UPDATE
  SET
    title = EXCLUDED.title,
    category = EXCLUDED.category,
    tags = EXCLUDED.tags,
    description = EXCLUDED.description,
    searchable_tsv = EXCLUDED.searchable_tsv,
    updated_at = now();

  RETURN NEW;
END;
$$;

CREATE TRIGGER trg_product_search_refresh
AFTER INSERT OR UPDATE ON catalog.product
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document();

CREATE TRIGGER trg_test_wallet_updated_at BEFORE UPDATE ON developer.test_wallet
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_test_application_updated_at BEFORE UPDATE ON developer.test_application
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_mock_provider_binding_updated_at BEFORE UPDATE ON developer.mock_provider_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_mock_payment_case_updated_at BEFORE UPDATE ON developer.mock_payment_case
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

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
-- Trust-boundary baseline sync: audit/search/devops schema remains valid; new trust-boundary evidence reuses audit and developer objects.
-- Payment settlement sync: developer schema now persists mock payment scenarios and outbox covers payment intent / payout events.
