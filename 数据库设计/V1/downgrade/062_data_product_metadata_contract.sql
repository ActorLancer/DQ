DROP TRIGGER IF EXISTS trg_data_contract_updated_at ON contract.data_contract;
DROP TRIGGER IF EXISTS trg_asset_processing_job_updated_at ON catalog.asset_processing_job;
DROP TRIGGER IF EXISTS trg_asset_quality_report_updated_at ON catalog.asset_quality_report;
DROP TRIGGER IF EXISTS trg_asset_field_definition_updated_at ON catalog.asset_field_definition;
DROP TRIGGER IF EXISTS trg_product_metadata_profile_updated_at ON catalog.product_metadata_profile;

DROP TABLE IF EXISTS contract.data_contract CASCADE;
DROP TABLE IF EXISTS catalog.asset_processing_input CASCADE;
DROP TABLE IF EXISTS catalog.asset_processing_job CASCADE;
DROP TABLE IF EXISTS catalog.asset_quality_report CASCADE;
DROP TABLE IF EXISTS catalog.asset_field_definition CASCADE;
DROP TABLE IF EXISTS catalog.product_metadata_profile CASCADE;

ALTER TABLE contract.digital_contract
  DROP COLUMN IF EXISTS data_contract_digest,
  DROP COLUMN IF EXISTS data_contract_id;

ALTER TABLE catalog.asset_sample
  DROP COLUMN IF EXISTS preview_policy_json,
  DROP COLUMN IF EXISTS masking_status;

ALTER TABLE catalog.asset_version
  DROP COLUMN IF EXISTS lineage_hash,
  DROP COLUMN IF EXISTS processing_mode;

ALTER TABLE catalog.product
  DROP COLUMN IF EXISTS prohibited_use_tags,
  DROP COLUMN IF EXISTS target_buyer_tags,
  DROP COLUMN IF EXISTS use_case_tags,
  DROP COLUMN IF EXISTS industry_tags,
  DROP COLUMN IF EXISTS product_subtitle;
