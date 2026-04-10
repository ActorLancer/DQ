DROP TRIGGER IF EXISTS trg_synonym_rule_updated_at ON search.synonym_rule;
DROP TRIGGER IF EXISTS trg_model_version_search_refresh ON ml.model_version;
DROP TRIGGER IF EXISTS trg_model_search_refresh ON ml.model_asset;

DROP FUNCTION IF EXISTS search.tg_refresh_model_search_document_from_model_version();
DROP FUNCTION IF EXISTS search.tg_refresh_model_search_document_from_model();
DROP FUNCTION IF EXISTS search.refresh_model_search_document_by_id(uuid);

DROP TABLE IF EXISTS search.synonym_rule CASCADE;

DROP INDEX IF EXISTS idx_model_search_document_sync_status;

ALTER TABLE search.model_search_document
  DROP COLUMN IF EXISTS ranking_features,
  DROP COLUMN IF EXISTS document_version,
  DROP COLUMN IF EXISTS source_updated_at,
  DROP COLUMN IF EXISTS index_sync_status,
  DROP COLUMN IF EXISTS index_backend,
  DROP COLUMN IF EXISTS indexed_at,
  DROP COLUMN IF EXISTS last_index_error;
