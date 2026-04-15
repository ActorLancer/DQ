CREATE TABLE IF NOT EXISTS crosschain.gateway_identity (
  gateway_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  gateway_name text NOT NULL,
  source_system text NOT NULL,
  status text NOT NULL DEFAULT 'active',
  service_identity_id uuid REFERENCES core.service_identity(service_identity_id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS crosschain.cross_chain_request (
  ccr_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  source_chain_id text NOT NULL,
  target_chain_id text NOT NULL,
  request_type text NOT NULL,
  source_gateway_id uuid REFERENCES crosschain.gateway_identity(gateway_id),
  target_gateway_id uuid REFERENCES crosschain.gateway_identity(gateway_id),
  payload_hash text NOT NULL,
  nonce text NOT NULL,
  request_id text,
  trust_scope_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  retention_obligation_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  ack_hash text,
  ack_status text,
  final_status text NOT NULL DEFAULT 'initiated',
  retry_count integer NOT NULL DEFAULT 0,
  terminate_flag boolean NOT NULL DEFAULT false,
  compensate_flag boolean NOT NULL DEFAULT false,
  timeout_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (source_chain_id, nonce)
);

CREATE TABLE IF NOT EXISTS crosschain.cross_chain_ack (
  ack_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  ccr_id uuid NOT NULL REFERENCES crosschain.cross_chain_request(ccr_id) ON DELETE CASCADE,
  ack_hash text NOT NULL,
  ack_status text NOT NULL,
  payload_digest text,
  received_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (ccr_id, ack_hash)
);

CREATE TABLE IF NOT EXISTS crosschain.witness_record (
  witness_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  ccr_id uuid NOT NULL REFERENCES crosschain.cross_chain_request(ccr_id) ON DELETE CASCADE,
  service_identity_id uuid REFERENCES core.service_identity(service_identity_id),
  witness_type text NOT NULL,
  signature_digest text NOT NULL,
  witnessed_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS crosschain.request_status_history (
  request_status_history_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  ccr_id uuid NOT NULL REFERENCES crosschain.cross_chain_request(ccr_id) ON DELETE CASCADE,
  old_status text,
  new_status text NOT NULL,
  changed_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS crosschain.compensation_task (
  compensation_task_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  ccr_id uuid NOT NULL REFERENCES crosschain.cross_chain_request(ccr_id) ON DELETE CASCADE,
  task_status text NOT NULL DEFAULT 'pending',
  task_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ecosystem.partner (
  partner_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  partner_name text NOT NULL,
  partner_type text NOT NULL,
  status text NOT NULL DEFAULT 'draft',
  partner_storage_capability jsonb NOT NULL DEFAULT '{}'::jsonb,
  partner_key_governance_capability jsonb NOT NULL DEFAULT '{}'::jsonb,
  partner_execution_boundary_capability jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ecosystem.connector_version (
  connector_version_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  connector_id uuid NOT NULL REFERENCES core.connector(connector_id) ON DELETE CASCADE,
  version text NOT NULL,
  compatibility_matrix jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ecosystem.mutual_recognition (
  mutual_recognition_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  partner_id uuid NOT NULL REFERENCES ecosystem.partner(partner_id) ON DELETE CASCADE,
  recognition_type text NOT NULL,
  certificate_digest text,
  capability_scope jsonb NOT NULL DEFAULT '{}'::jsonb,
  cross_platform_trust_scope jsonb NOT NULL DEFAULT '{}'::jsonb,
  external_destroy_attestation_uri text,
  status text NOT NULL DEFAULT 'draft',
  effective_from timestamptz,
  effective_to timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment.settlement_route (
  settlement_route_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  route_name text NOT NULL,
  route_type text NOT NULL,
  source_jurisdiction_code text REFERENCES payment.jurisdiction_profile(jurisdiction_code) ON DELETE SET NULL,
  target_jurisdiction_code text REFERENCES payment.jurisdiction_profile(jurisdiction_code) ON DELETE SET NULL,
  corridor_policy_id uuid REFERENCES payment.corridor_policy(corridor_policy_id) ON DELETE SET NULL,
  source_currency text NOT NULL,
  target_currency text NOT NULL,
  provider_key text REFERENCES payment.provider(provider_key) ON DELETE SET NULL,
  route_priority integer NOT NULL DEFAULT 100,
  status text NOT NULL DEFAULT 'draft',
  compliance_policy jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment.fx_quote (
  fx_quote_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_route_id uuid REFERENCES payment.settlement_route(settlement_route_id) ON DELETE CASCADE,
  base_currency text NOT NULL,
  quote_currency text NOT NULL,
  quoted_rate numeric(24, 8) NOT NULL,
  fee_amount numeric(24, 8) NOT NULL DEFAULT 0,
  ref_type text,
  ref_id uuid,
  expire_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment.crypto_settlement_transfer (
  crypto_transfer_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_route_id uuid REFERENCES payment.settlement_route(settlement_route_id) ON DELETE SET NULL,
  settlement_id uuid REFERENCES billing.settlement_record(settlement_id) ON DELETE SET NULL,
  exchange_ref_no text,
  wallet_address text,
  asset_code text NOT NULL,
  chain_id text,
  amount numeric(24, 8) NOT NULL DEFAULT 0,
  status text NOT NULL DEFAULT 'pending',
  tx_hash text,
  compliance_status text NOT NULL DEFAULT 'pending_review',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TRIGGER trg_gateway_identity_updated_at BEFORE UPDATE ON crosschain.gateway_identity
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_cross_chain_request_updated_at BEFORE UPDATE ON crosschain.cross_chain_request
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_compensation_task_updated_at BEFORE UPDATE ON crosschain.compensation_task
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_partner_updated_at BEFORE UPDATE ON ecosystem.partner
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_mutual_recognition_updated_at BEFORE UPDATE ON ecosystem.mutual_recognition
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_settlement_route_updated_at BEFORE UPDATE ON payment.settlement_route
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_crypto_settlement_transfer_updated_at BEFORE UPDATE ON payment.crypto_settlement_transfer
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
-- Payment settlement sync: V3 adds settlement routing, fx quote and digital-asset transfer objects.
