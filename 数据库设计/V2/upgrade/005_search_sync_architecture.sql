ALTER TABLE search.model_search_document
  ADD COLUMN IF NOT EXISTS ranking_features jsonb NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN IF NOT EXISTS document_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS source_updated_at timestamptz NOT NULL DEFAULT now(),
  ADD COLUMN IF NOT EXISTS index_sync_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS index_backend text NOT NULL DEFAULT 'opensearch',
  ADD COLUMN IF NOT EXISTS indexed_at timestamptz,
  ADD COLUMN IF NOT EXISTS last_index_error text;

CREATE INDEX IF NOT EXISTS idx_model_search_document_sync_status
  ON search.model_search_document(index_sync_status, updated_at DESC);

CREATE TABLE IF NOT EXISTS search.synonym_rule (
  synonym_rule_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_scope text NOT NULL,
  locale_code text NOT NULL DEFAULT 'zh-CN',
  rule_type text NOT NULL DEFAULT 'equivalent',
  input_terms text[] NOT NULL DEFAULT '{}',
  output_terms text[] NOT NULL DEFAULT '{}',
  status text NOT NULL DEFAULT 'draft',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_synonym_rule_scope_status
  ON search.synonym_rule(entity_scope, status);

CREATE OR REPLACE FUNCTION search.refresh_model_search_document_by_id(p_model_id uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO search.model_search_document (
    model_id,
    owner_org_id,
    model_name,
    model_family,
    metric_summary,
    searchable_tsv,
    embedding,
    updated_at,
    ranking_features,
    document_version,
    source_updated_at,
    index_sync_status,
    index_backend,
    indexed_at,
    last_index_error
  )
  SELECT
    m.model_id,
    m.owner_org_id,
    m.model_name,
    m.model_family,
    m.metric_summary::text,
    to_tsvector('simple', concat_ws(' ', m.model_name, COALESCE(m.model_family, ''), COALESCE(m.metric_summary::text, ''))),
    NULL,
    now(),
    jsonb_build_object(
      'owner_org_id', m.owner_org_id,
      'status', m.status,
      'metric_summary', m.metric_summary
    ),
    1,
    m.updated_at,
    'pending',
    'opensearch',
    NULL,
    NULL
  FROM ml.model_asset m
  WHERE m.model_id = p_model_id
  ON CONFLICT (model_id) DO UPDATE
  SET
    owner_org_id = EXCLUDED.owner_org_id,
    model_name = EXCLUDED.model_name,
    model_family = EXCLUDED.model_family,
    metric_summary = EXCLUDED.metric_summary,
    searchable_tsv = EXCLUDED.searchable_tsv,
    updated_at = now(),
    ranking_features = EXCLUDED.ranking_features,
    document_version = search.model_search_document.document_version + 1,
    source_updated_at = EXCLUDED.source_updated_at,
    index_sync_status = 'pending',
    index_backend = EXCLUDED.index_backend,
    indexed_at = NULL,
    last_index_error = NULL;
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_model_search_document_from_model()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM search.refresh_model_search_document_by_id(COALESCE(NEW.model_id, OLD.model_id));
  RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_model_search_document_from_model_version()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM search.refresh_model_search_document_by_id(COALESCE(NEW.model_id, OLD.model_id));
  RETURN COALESCE(NEW, OLD);
END;
$$;

DROP TRIGGER IF EXISTS trg_model_search_refresh ON ml.model_asset;
CREATE TRIGGER trg_model_search_refresh
AFTER INSERT OR UPDATE ON ml.model_asset
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_model_search_document_from_model();

DROP TRIGGER IF EXISTS trg_model_version_search_refresh ON ml.model_version;
CREATE TRIGGER trg_model_version_search_refresh
AFTER INSERT OR UPDATE OR DELETE ON ml.model_version
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_model_search_document_from_model_version();

CREATE TRIGGER trg_synonym_rule_updated_at BEFORE UPDATE ON search.synonym_rule
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

SELECT search.refresh_model_search_document_by_id(m.model_id)
FROM ml.model_asset m;
