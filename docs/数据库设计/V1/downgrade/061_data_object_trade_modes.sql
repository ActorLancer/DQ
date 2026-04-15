DROP TRIGGER IF EXISTS trg_template_query_grant_updated_at ON delivery.template_query_grant;
DROP TRIGGER IF EXISTS trg_revision_subscription_updated_at ON delivery.revision_subscription;
DROP TRIGGER IF EXISTS trg_data_share_grant_updated_at ON delivery.data_share_grant;
DROP TRIGGER IF EXISTS trg_asset_object_binding_updated_at ON catalog.asset_object_binding;

DROP TABLE IF EXISTS delivery.template_query_grant CASCADE;
DROP TABLE IF EXISTS delivery.revision_subscription CASCADE;
DROP TABLE IF EXISTS delivery.data_share_grant CASCADE;
DROP TABLE IF EXISTS catalog.asset_object_binding CASCADE;

ALTER TABLE catalog.product_sku
  DROP COLUMN IF EXISTS result_form,
  DROP COLUMN IF EXISTS share_protocol,
  DROP COLUMN IF EXISTS subscription_cadence,
  DROP COLUMN IF EXISTS delivery_object_kind,
  DROP COLUMN IF EXISTS trade_mode;

ALTER TABLE catalog.asset_version
  DROP COLUMN IF EXISTS release_notes_json,
  DROP COLUMN IF EXISTS update_frequency,
  DROP COLUMN IF EXISTS is_revision_subscribable,
  DROP COLUMN IF EXISTS release_mode;
