DROP TRIGGER IF EXISTS trg_preview_artifact_updated_at ON catalog.preview_artifact;
DROP TRIGGER IF EXISTS trg_extraction_job_updated_at ON catalog.extraction_job;
DROP TRIGGER IF EXISTS trg_raw_object_manifest_updated_at ON catalog.raw_object_manifest;
DROP TRIGGER IF EXISTS trg_raw_ingest_batch_updated_at ON catalog.raw_ingest_batch;

DROP TABLE IF EXISTS catalog.preview_artifact CASCADE;
DROP TABLE IF EXISTS catalog.extraction_job CASCADE;
DROP TABLE IF EXISTS catalog.format_detection_result CASCADE;
DROP TABLE IF EXISTS catalog.raw_object_manifest CASCADE;
DROP TABLE IF EXISTS catalog.raw_ingest_batch CASCADE;

ALTER TABLE catalog.asset_version
  DROP COLUMN IF EXISTS standardization_status,
  DROP COLUMN IF EXISTS processing_stage;
