DROP TRIGGER IF EXISTS trg_usage_policy_updated_at ON contract.usage_policy;
DROP TRIGGER IF EXISTS trg_template_definition_updated_at ON contract.template_definition;
DROP TRIGGER IF EXISTS trg_product_sku_updated_at ON catalog.product_sku;
DROP TRIGGER IF EXISTS trg_product_updated_at ON catalog.product;
DROP TRIGGER IF EXISTS trg_asset_custody_profile_updated_at ON catalog.asset_custody_profile;
DROP TRIGGER IF EXISTS trg_asset_structured_dataset_updated_at ON catalog.asset_structured_dataset;
DROP TRIGGER IF EXISTS trg_asset_version_updated_at ON catalog.asset_version;
DROP TRIGGER IF EXISTS trg_data_asset_updated_at ON catalog.data_asset;

DROP TABLE IF EXISTS contract.policy_binding CASCADE;
DROP TABLE IF EXISTS contract.usage_policy CASCADE;
DROP TABLE IF EXISTS contract.template_binding CASCADE;
DROP TABLE IF EXISTS contract.template_definition CASCADE;
DROP TABLE IF EXISTS catalog.product_sku CASCADE;
DROP TABLE IF EXISTS catalog.product_tag CASCADE;
DROP TABLE IF EXISTS catalog.product CASCADE;
DROP TABLE IF EXISTS catalog.tag CASCADE;
DROP TABLE IF EXISTS catalog.asset_structured_row CASCADE;
DROP TABLE IF EXISTS catalog.asset_structured_dataset CASCADE;
DROP TABLE IF EXISTS catalog.asset_sample CASCADE;
DROP TABLE IF EXISTS catalog.asset_trust_evidence CASCADE;
DROP TABLE IF EXISTS catalog.asset_custody_profile CASCADE;
DROP TABLE IF EXISTS catalog.asset_storage_binding CASCADE;
DROP TABLE IF EXISTS catalog.asset_version CASCADE;
DROP TABLE IF EXISTS catalog.data_asset CASCADE;

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
