ALTER TABLE catalog.asset_version
  ADD COLUMN IF NOT EXISTS release_mode text NOT NULL DEFAULT 'snapshot',
  ADD COLUMN IF NOT EXISTS is_revision_subscribable boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS update_frequency text,
  ADD COLUMN IF NOT EXISTS release_notes_json jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE catalog.product_sku
  ADD COLUMN IF NOT EXISTS trade_mode text NOT NULL DEFAULT 'snapshot_sale',
  ADD COLUMN IF NOT EXISTS delivery_object_kind text,
  ADD COLUMN IF NOT EXISTS subscription_cadence text,
  ADD COLUMN IF NOT EXISTS share_protocol text,
  ADD COLUMN IF NOT EXISTS result_form text;

CREATE TABLE IF NOT EXISTS catalog.asset_object_binding (
  asset_object_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  asset_version_id uuid NOT NULL REFERENCES catalog.asset_version(asset_version_id) ON DELETE CASCADE,
  object_kind text NOT NULL,
  object_name text NOT NULL,
  object_locator text,
  share_protocol text,
  schema_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  output_schema_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  freshness_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  access_constraints jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (asset_version_id, object_kind, object_name)
);

CREATE TABLE IF NOT EXISTS delivery.data_share_grant (
  data_share_grant_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  asset_object_id uuid NOT NULL REFERENCES catalog.asset_object_binding(asset_object_id) ON DELETE RESTRICT,
  recipient_ref text NOT NULL,
  share_protocol text NOT NULL,
  access_locator text,
  grant_status text NOT NULL DEFAULT 'pending',
  read_only boolean NOT NULL DEFAULT true,
  receipt_hash text,
  granted_at timestamptz,
  revoked_at timestamptz,
  expires_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS delivery.revision_subscription (
  revision_subscription_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  asset_id uuid NOT NULL REFERENCES catalog.data_asset(asset_id) ON DELETE RESTRICT,
  sku_id uuid NOT NULL REFERENCES catalog.product_sku(sku_id) ON DELETE RESTRICT,
  cadence text NOT NULL DEFAULT 'monthly',
  delivery_channel text NOT NULL DEFAULT 'file_ticket',
  start_version_no integer,
  last_delivered_version_no integer,
  next_delivery_at timestamptz,
  subscription_status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (order_id)
);

CREATE TABLE IF NOT EXISTS delivery.template_query_grant (
  template_query_grant_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid NOT NULL REFERENCES trade.order_main(order_id) ON DELETE CASCADE,
  asset_object_id uuid NOT NULL REFERENCES catalog.asset_object_binding(asset_object_id) ON DELETE RESTRICT,
  sandbox_workspace_id uuid REFERENCES delivery.sandbox_workspace(sandbox_workspace_id) ON DELETE SET NULL,
  environment_id uuid REFERENCES core.execution_environment(environment_id) ON DELETE SET NULL,
  template_type text NOT NULL DEFAULT 'sql_template',
  template_digest text,
  output_boundary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  run_quota_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  grant_status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_asset_object_binding_version
  ON catalog.asset_object_binding(asset_version_id, object_kind);
CREATE INDEX IF NOT EXISTS idx_data_share_grant_order
  ON delivery.data_share_grant(order_id, grant_status);
CREATE INDEX IF NOT EXISTS idx_revision_subscription_order
  ON delivery.revision_subscription(order_id, subscription_status);
CREATE INDEX IF NOT EXISTS idx_revision_subscription_asset
  ON delivery.revision_subscription(asset_id, subscription_status);
CREATE INDEX IF NOT EXISTS idx_template_query_grant_order
  ON delivery.template_query_grant(order_id, grant_status);

CREATE TRIGGER trg_asset_object_binding_updated_at BEFORE UPDATE ON catalog.asset_object_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_data_share_grant_updated_at BEFORE UPDATE ON delivery.data_share_grant
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_revision_subscription_updated_at BEFORE UPDATE ON delivery.revision_subscription
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_template_query_grant_updated_at BEFORE UPDATE ON delivery.template_query_grant
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
