DROP INDEX IF EXISTS idx_seller_search_document_certification_tags_gin;

ALTER TABLE search.seller_search_document
  DROP COLUMN IF EXISTS rating_summary,
  DROP COLUMN IF EXISTS featured_products,
  DROP COLUMN IF EXISTS certification_tags;

DROP FUNCTION IF EXISTS search.resolve_seller_certification_tags(text, text, jsonb);

CREATE OR REPLACE FUNCTION search.refresh_seller_search_document_by_id(p_org_id uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO search.seller_search_document (
    org_id,
    seller_name,
    seller_type,
    description,
    country_code,
    region_code,
    industry_tags,
    credit_level,
    risk_level,
    reputation_score,
    listing_product_count,
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
    org.org_id,
    org.org_name,
    org.org_type,
    COALESCE(org.metadata ->> 'description', ''),
    org.country_code,
    org.region_code,
    COALESCE(org.industry_tags, '{}'),
    COALESCE(rep.credit_level, org.credit_level, 0),
    COALESCE(rep.risk_level, org.risk_level, 0),
    COALESCE(rep.score, 0),
    COALESCE(listed.listing_product_count, 0),
    to_tsvector(
      'simple',
      concat_ws(
        ' ',
        org.org_name,
        org.org_type,
        org.country_code,
        array_to_string(COALESCE(org.industry_tags, '{}'), ' '),
        COALESCE(org.metadata ->> 'description', '')
      )
    ),
    NULL,
    jsonb_build_object(
      'reputation_score', COALESCE(rep.score, 0),
      'credit_level', COALESCE(rep.credit_level, org.credit_level, 0),
      'risk_level', COALESCE(rep.risk_level, org.risk_level, 0),
      'listing_product_count', COALESCE(listed.listing_product_count, 0)
    ),
    1,
    org.updated_at,
    'pending',
    'opensearch',
    NULL,
    NULL,
    now()
  FROM core.organization org
  LEFT JOIN LATERAL (
    SELECT rs.score, rs.credit_level, rs.risk_level
    FROM risk.reputation_snapshot rs
    WHERE rs.subject_type = 'organization'
      AND rs.subject_id = org.org_id
    ORDER BY rs.effective_at DESC
    LIMIT 1
  ) AS rep ON true
  LEFT JOIN LATERAL (
    SELECT COUNT(*)::integer AS listing_product_count
    FROM catalog.product p
    WHERE p.seller_org_id = org.org_id
      AND p.status = 'listed'
  ) AS listed ON true
  WHERE org.org_id = p_org_id
  ON CONFLICT (org_id) DO UPDATE
  SET
    seller_name = EXCLUDED.seller_name,
    seller_type = EXCLUDED.seller_type,
    description = EXCLUDED.description,
    country_code = EXCLUDED.country_code,
    region_code = EXCLUDED.region_code,
    industry_tags = EXCLUDED.industry_tags,
    credit_level = EXCLUDED.credit_level,
    risk_level = EXCLUDED.risk_level,
    reputation_score = EXCLUDED.reputation_score,
    listing_product_count = EXCLUDED.listing_product_count,
    searchable_tsv = EXCLUDED.searchable_tsv,
    ranking_features = EXCLUDED.ranking_features,
    document_version = search.seller_search_document.document_version + 1,
    source_updated_at = EXCLUDED.source_updated_at,
    index_sync_status = 'pending',
    index_backend = EXCLUDED.index_backend,
    indexed_at = NULL,
    last_index_error = NULL,
    updated_at = now();
END;
$$;

SELECT search.refresh_seller_search_document_by_id(org.org_id)
FROM core.organization org;
