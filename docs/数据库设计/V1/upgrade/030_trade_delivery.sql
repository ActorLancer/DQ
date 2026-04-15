CREATE TABLE IF NOT EXISTS trade.inquiry (
  inquiry_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  buyer_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  product_id uuid REFERENCES catalog.product(product_id),
  status text NOT NULL DEFAULT 'open',
  message_text text,
  created_by uuid REFERENCES core.user_account(user_id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS contract.digital_contract (
  contract_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid UNIQUE,
  contract_template_id uuid REFERENCES contract.template_definition(template_id),
  contract_digest text,
  status text NOT NULL DEFAULT 'draft',
  signed_at timestamptz,
  variables_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS trade.order_main (
  order_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  inquiry_id uuid REFERENCES trade.inquiry(inquiry_id),
  product_id uuid NOT NULL REFERENCES catalog.product(product_id),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id),
  buyer_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  seller_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  sku_id uuid NOT NULL REFERENCES catalog.product_sku(sku_id),
  contract_id uuid REFERENCES contract.digital_contract(contract_id),
  policy_id uuid REFERENCES contract.usage_policy(policy_id),
  status text NOT NULL DEFAULT 'created',
  payment_status text NOT NULL DEFAULT 'unpaid',
  payment_mode text NOT NULL DEFAULT 'online',
  amount numeric(20, 8) NOT NULL,
  currency_code text NOT NULL DEFAULT 'CNY',
  fee_preview_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  payment_channel_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  buyer_deposit_amount numeric(20, 8) NOT NULL DEFAULT 0,
  seller_deposit_amount numeric(20, 8) NOT NULL DEFAULT 0,
  price_snapshot_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  trust_boundary_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  storage_mode_snapshot text NOT NULL DEFAULT 'platform_custody',
  delivery_route_snapshot text,
  platform_plaintext_access_snapshot boolean NOT NULL DEFAULT false,
  last_reason_code text,
  idempotency_key text,
  chain_tx_create text,
  chain_tx_settle text,
  created_at timestamptz NOT NULL DEFAULT now(),
  buyer_locked_at timestamptz,
  delivered_at timestamptz,
  accepted_at timestamptz,
  settled_at timestamptz,
  closed_at timestamptz,
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (idempotency_key)
);

ALTER TABLE contract.digital_contract
  ADD CONSTRAINT fk_digital_contract_order
  FOREIGN KEY (order_id) REFERENCES trade.order_main(order_id) ON DELETE CASCADE;

ALTER TABLE trade.order_main
  ADD CONSTRAINT fk_order_contract
  FOREIGN KEY (contract_id) REFERENCES contract.digital_contract(contract_id);

CREATE TABLE IF NOT EXISTS trade.order_line (
  order_line_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  sku_id uuid NOT NULL REFERENCES catalog.product_sku(sku_id),
  quantity numeric(20, 8) NOT NULL DEFAULT 1,
  unit_price numeric(20, 8) NOT NULL DEFAULT 0,
  amount numeric(20, 8) NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS trade.order_status_history (
  order_status_history_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  old_status text,
  new_status text NOT NULL,
  changed_by_type text,
  changed_by_id uuid,
  reason_code text,
  changed_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS trade.authorization_grant (
  authorization_grant_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  grant_type text NOT NULL,
  granted_to_type text NOT NULL,
  granted_to_id uuid NOT NULL,
  policy_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  valid_from timestamptz NOT NULL DEFAULT now(),
  valid_to timestamptz,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS contract.contract_signer (
  contract_signer_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  contract_id uuid NOT NULL REFERENCES contract.digital_contract(contract_id) ON DELETE CASCADE,
  signer_type text NOT NULL,
  signer_id uuid NOT NULL,
  signer_role text,
  signature_digest text,
  signed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.storage_object (
  object_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid NOT NULL REFERENCES core.organization(org_id),
  object_type text NOT NULL,
  object_uri text NOT NULL,
  location_type text NOT NULL DEFAULT 'platform_object_storage',
  managed_by_org_id uuid REFERENCES core.organization(org_id),
  connector_id uuid REFERENCES core.connector(connector_id),
  environment_id uuid REFERENCES core.execution_environment(environment_id),
  content_type text,
  size_bytes bigint,
  content_hash text,
  encryption_algo text,
  plaintext_visible_to_platform boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.delivery_record (
  delivery_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  object_id uuid REFERENCES delivery.storage_object(object_id),
  source_binding_id uuid REFERENCES catalog.asset_storage_binding(asset_storage_binding_id),
  delivery_type text NOT NULL,
  delivery_route text,
  executor_type text NOT NULL DEFAULT 'platform',
  executor_ref_id uuid,
  status text NOT NULL DEFAULT 'prepared',
  delivery_commit_hash text,
  envelope_id uuid,
  trust_boundary_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  receipt_hash text,
  committed_at timestamptz,
  expires_at timestamptz,
  created_by uuid REFERENCES core.user_account(user_id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.key_envelope (
  envelope_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  recipient_type text NOT NULL,
  recipient_id uuid NOT NULL,
  key_cipher text NOT NULL,
  key_control_mode text NOT NULL DEFAULT 'seller_managed',
  unwrap_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  key_version text,
  created_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE delivery.delivery_record
  ADD CONSTRAINT fk_delivery_record_envelope
  FOREIGN KEY (envelope_id) REFERENCES delivery.key_envelope(envelope_id);

CREATE TABLE IF NOT EXISTS delivery.delivery_ticket (
  ticket_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  buyer_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  token_hash text NOT NULL,
  expire_at timestamptz NOT NULL,
  download_limit integer NOT NULL DEFAULT 1,
  download_count integer NOT NULL DEFAULT 0,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.delivery_receipt (
  receipt_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  delivery_id uuid NOT NULL REFERENCES delivery.delivery_record(delivery_id) ON DELETE CASCADE,
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  receipt_hash text NOT NULL,
  client_fingerprint text,
  source_ip inet,
  downloaded_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.api_credential (
  api_credential_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  app_id uuid NOT NULL REFERENCES core.application(app_id) ON DELETE CASCADE,
  source_binding_id uuid REFERENCES catalog.asset_storage_binding(asset_storage_binding_id),
  api_key_hash text NOT NULL,
  upstream_mode text NOT NULL DEFAULT 'platform_proxy',
  quota_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'active',
  valid_from timestamptz NOT NULL DEFAULT now(),
  valid_to timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.api_usage_log (
  api_usage_log_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  api_credential_id uuid NOT NULL REFERENCES delivery.api_credential(api_credential_id) ON DELETE CASCADE,
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  app_id uuid NOT NULL REFERENCES core.application(app_id),
  request_id text,
  response_code integer,
  usage_units numeric(20, 8) NOT NULL DEFAULT 1,
  occurred_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.sandbox_workspace (
  sandbox_workspace_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  environment_id uuid REFERENCES core.execution_environment(environment_id),
  workspace_name text NOT NULL,
  status text NOT NULL DEFAULT 'provisioning',
  data_residency_mode text NOT NULL DEFAULT 'seller_self_hosted',
  export_policy jsonb NOT NULL DEFAULT '{}'::jsonb,
  output_boundary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.sandbox_session (
  sandbox_session_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  sandbox_workspace_id uuid NOT NULL REFERENCES delivery.sandbox_workspace(sandbox_workspace_id) ON DELETE CASCADE,
  user_id uuid REFERENCES core.user_account(user_id),
  started_at timestamptz NOT NULL DEFAULT now(),
  ended_at timestamptz,
  session_status text NOT NULL DEFAULT 'active',
  query_count integer NOT NULL DEFAULT 0,
  export_attempt_count integer NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS delivery.report_artifact (
  report_artifact_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  object_id uuid REFERENCES delivery.storage_object(object_id),
  report_type text NOT NULL,
  version_no integer NOT NULL DEFAULT 1,
  status text NOT NULL DEFAULT 'draft',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_order_main_buyer_org_id ON trade.order_main(buyer_org_id);
CREATE INDEX IF NOT EXISTS idx_order_main_seller_org_id ON trade.order_main(seller_org_id);
CREATE INDEX IF NOT EXISTS idx_order_main_status_created_at ON trade.order_main(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_delivery_record_order_id ON delivery.delivery_record(order_id);
CREATE INDEX IF NOT EXISTS idx_api_usage_log_order_id ON delivery.api_usage_log(order_id);

CREATE TRIGGER trg_inquiry_updated_at BEFORE UPDATE ON trade.inquiry
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_order_main_updated_at BEFORE UPDATE ON trade.order_main
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_order_status_history AFTER INSERT OR UPDATE ON trade.order_main
FOR EACH ROW EXECUTE FUNCTION common.tg_order_status_history();
CREATE TRIGGER trg_authorization_grant_updated_at BEFORE UPDATE ON trade.authorization_grant
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_delivery_record_updated_at BEFORE UPDATE ON delivery.delivery_record
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_api_credential_updated_at BEFORE UPDATE ON delivery.api_credential
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_sandbox_workspace_updated_at BEFORE UPDATE ON delivery.sandbox_workspace
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_report_artifact_updated_at BEFORE UPDATE ON delivery.report_artifact
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
