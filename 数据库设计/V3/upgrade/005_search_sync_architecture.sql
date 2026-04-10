CREATE TABLE IF NOT EXISTS search.partner_search_document (
  partner_id uuid PRIMARY KEY REFERENCES ecosystem.partner(partner_id) ON DELETE CASCADE,
  org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  partner_name text NOT NULL,
  partner_type text NOT NULL,
  status text NOT NULL DEFAULT 'draft',
  industry_tags text[] NOT NULL DEFAULT '{}',
  country_code text,
  searchable_tsv tsvector,
  embedding vector(1536),
  ranking_features jsonb NOT NULL DEFAULT '{}'::jsonb,
  document_version bigint NOT NULL DEFAULT 1,
  source_updated_at timestamptz NOT NULL DEFAULT now(),
  index_sync_status text NOT NULL DEFAULT 'pending',
  index_backend text NOT NULL DEFAULT 'opensearch',
  indexed_at timestamptz,
  last_index_error text,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_partner_search_document_tsv
  ON search.partner_search_document USING GIN (searchable_tsv);
CREATE INDEX IF NOT EXISTS idx_partner_search_document_embedding
  ON search.partner_search_document USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_partner_search_document_sync_status
  ON search.partner_search_document(index_sync_status, updated_at DESC);

CREATE OR REPLACE FUNCTION search.refresh_partner_search_document_by_id(p_partner_id uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO search.partner_search_document (
    partner_id,
    org_id,
    partner_name,
    partner_type,
    status,
    industry_tags,
    country_code,
    searchable_tsv,
    embedding,
    ranking_features,
    document_version,
    source_updated_at,
    index_sync_status,
    index_backend,
    indexed_at,
    last_index_error,
    updated_at
  )
  SELECT
    ep.partner_id,
    ep.org_id,
    ep.partner_name,
    ep.partner_type,
    ep.status,
    COALESCE(org.industry_tags, '{}'),
    org.country_code,
    to_tsvector(
      'simple',
      concat_ws(' ', ep.partner_name, ep.partner_type, COALESCE(array_to_string(org.industry_tags, ' '), ''), COALESCE(org.country_code, ''))
    ),
    NULL,
    jsonb_build_object(
      'partner_type', ep.partner_type,
      'status', ep.status,
      'mutual_recognition_count', COALESCE(mr.active_count, 0)
    ),
    1,
    ep.updated_at,
    'pending',
    'opensearch',
    NULL,
    NULL,
    now()
  FROM ecosystem.partner ep
  LEFT JOIN core.organization org ON org.org_id = ep.org_id
  LEFT JOIN LATERAL (
    SELECT COUNT(*)::integer AS active_count
    FROM ecosystem.mutual_recognition mr
    WHERE mr.partner_id = ep.partner_id
      AND mr.status = 'active'
  ) AS mr ON true
  WHERE ep.partner_id = p_partner_id
  ON CONFLICT (partner_id) DO UPDATE
  SET
    org_id = EXCLUDED.org_id,
    partner_name = EXCLUDED.partner_name,
    partner_type = EXCLUDED.partner_type,
    status = EXCLUDED.status,
    industry_tags = EXCLUDED.industry_tags,
    country_code = EXCLUDED.country_code,
    searchable_tsv = EXCLUDED.searchable_tsv,
    ranking_features = EXCLUDED.ranking_features,
    document_version = search.partner_search_document.document_version + 1,
    source_updated_at = EXCLUDED.source_updated_at,
    index_sync_status = 'pending',
    index_backend = EXCLUDED.index_backend,
    indexed_at = NULL,
    last_index_error = NULL,
    updated_at = now();
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_partner_search_document()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM search.refresh_partner_search_document_by_id(COALESCE(NEW.partner_id, OLD.partner_id));
  RETURN COALESCE(NEW, OLD);
END;
$$;

DROP TRIGGER IF EXISTS trg_partner_search_refresh ON ecosystem.partner;
CREATE TRIGGER trg_partner_search_refresh
AFTER INSERT OR UPDATE ON ecosystem.partner
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_partner_search_document();

DROP TRIGGER IF EXISTS trg_mutual_recognition_search_refresh ON ecosystem.mutual_recognition;
CREATE TRIGGER trg_mutual_recognition_search_refresh
AFTER INSERT OR UPDATE OR DELETE ON ecosystem.mutual_recognition
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_partner_search_document();

CREATE TRIGGER trg_partner_search_document_updated_at BEFORE UPDATE ON search.partner_search_document
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

SELECT search.refresh_partner_search_document_by_id(ep.partner_id)
FROM ecosystem.partner ep;
