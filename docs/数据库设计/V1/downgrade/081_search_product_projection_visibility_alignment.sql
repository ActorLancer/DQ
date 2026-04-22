DROP TRIGGER IF EXISTS trg_review_task_search_refresh ON review.review_task;
DROP FUNCTION IF EXISTS search.tg_refresh_product_search_document_from_review_task();
DROP FUNCTION IF EXISTS search.resolve_product_visibility_status(text, text, text, boolean);
DROP FUNCTION IF EXISTS search.resolve_product_review_status(text, text);

CREATE OR REPLACE FUNCTION search.refresh_product_search_document_by_id(p_product_id uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO search.product_search_document (
    product_id,
    org_id,
    title,
    category,
    tags,
    description,
    searchable_tsv,
    embedding,
    updated_at,
    product_type,
    subtitle,
    industry,
    use_cases,
    seller_name,
    seller_type,
    seller_country_code,
    seller_industry_tags,
    seller_reputation_score,
    seller_credit_level,
    seller_risk_level,
    sku_types,
    rights_types,
    delivery_modes,
    price_mode,
    price_amount,
    price_min,
    price_max,
    currency_code,
    listing_status,
    sample_available,
    data_classification,
    quality_score,
    recent_trade_count,
    hotness_score,
    ranking_features,
    document_version,
    source_updated_at,
    index_sync_status,
    index_backend,
    indexed_at,
    last_index_error
  )
  SELECT
    p.product_id,
    p.seller_org_id,
    p.title,
    p.category,
    COALESCE(tag_agg.tag_names, '{}'),
    p.description,
    to_tsvector(
      'simple',
      concat_ws(
        ' ',
        p.title,
        p.category,
        p.description,
        COALESCE(p.searchable_text, ''),
        COALESCE(org.org_name, ''),
        array_to_string(COALESCE(tag_agg.search_terms, '{}'), ' ')
      )
    ),
    NULL,
    now(),
    p.product_type,
    NULLIF(p.metadata ->> 'subtitle', ''),
    COALESCE(p.metadata ->> 'industry', p.category),
    COALESCE(
      ARRAY(
        SELECT jsonb_array_elements_text(
          CASE
            WHEN jsonb_typeof(p.metadata -> 'use_cases') = 'array' THEN p.metadata -> 'use_cases'
            ELSE '[]'::jsonb
          END
        )
      ),
      '{}'
    ),
    org.org_name,
    org.org_type,
    org.country_code,
    COALESCE(org.industry_tags, '{}'),
    COALESCE(rep.score, 0),
    COALESCE(rep.credit_level, org.credit_level, 0),
    COALESCE(rep.risk_level, org.risk_level, 0),
    COALESCE(sku_agg.sku_types, ARRAY[p.product_type]),
    COALESCE(p.allowed_usage, '{}'),
    ARRAY[p.delivery_type],
    p.price_mode,
    p.price,
    p.price,
    p.price,
    p.currency_code,
    p.status,
    EXISTS (
      SELECT 1
      FROM catalog.asset_sample s
      WHERE s.asset_version_id = p.asset_version_id
    ),
    COALESCE(av.metadata ->> 'data_classification', p.metadata ->> 'data_classification'),
    CASE
      WHEN COALESCE(p.metadata ->> 'quality_score', '') ~ '^-?[0-9]+(\.[0-9]+)?$'
        THEN (p.metadata ->> 'quality_score')::numeric
      ELSE 0
    END,
    COALESCE(sig.order_count, 0),
    COALESCE(sig.hotness_score, 0),
    jsonb_build_object(
      'quality_score',
      CASE
        WHEN COALESCE(p.metadata ->> 'quality_score', '') ~ '^-?[0-9]+(\.[0-9]+)?$'
          THEN (p.metadata ->> 'quality_score')::numeric
        ELSE 0
      END,
      'seller_reputation_score', COALESCE(rep.score, 0),
      'seller_credit_level', COALESCE(rep.credit_level, org.credit_level, 0),
      'seller_risk_level', COALESCE(rep.risk_level, org.risk_level, 0),
      'recent_trade_count', COALESCE(sig.order_count, 0),
      'hotness_score', COALESCE(sig.hotness_score, 0)
    ),
    1,
    GREATEST(p.updated_at, org.updated_at, COALESCE(av.updated_at, p.updated_at)),
    'pending',
    'opensearch',
    NULL,
    NULL
  FROM catalog.product p
  JOIN core.organization org ON org.org_id = p.seller_org_id
  JOIN catalog.asset_version av ON av.asset_version_id = p.asset_version_id
  LEFT JOIN LATERAL (
    SELECT
      array_remove(array_agg(DISTINCT t.tag_name), NULL) AS tag_names,
      array_remove(array_agg(DISTINCT t.tag_name), NULL) AS search_terms
    FROM catalog.product_tag pt
    JOIN catalog.tag t ON t.tag_id = pt.tag_id
    WHERE pt.product_id = p.product_id
      AND t.status = 'active'
  ) AS tag_agg ON true
  LEFT JOIN LATERAL (
    SELECT
      array_remove(array_agg(DISTINCT sku.sku_type), NULL) AS sku_types
    FROM catalog.product_sku sku
    WHERE sku.product_id = p.product_id
      AND sku.status <> 'retired'
  ) AS sku_agg ON true
  LEFT JOIN LATERAL (
    SELECT rs.score, rs.credit_level, rs.risk_level
    FROM risk.reputation_snapshot rs
    WHERE rs.subject_type = 'organization'
      AND rs.subject_id = p.seller_org_id
    ORDER BY rs.effective_at DESC
    LIMIT 1
  ) AS rep ON true
  LEFT JOIN search.search_signal_aggregate sig
    ON sig.entity_scope = 'product' AND sig.entity_id = p.product_id
  WHERE p.product_id = p_product_id
  ON CONFLICT (product_id) DO UPDATE
  SET
    org_id = EXCLUDED.org_id,
    title = EXCLUDED.title,
    category = EXCLUDED.category,
    tags = EXCLUDED.tags,
    description = EXCLUDED.description,
    searchable_tsv = EXCLUDED.searchable_tsv,
    updated_at = now(),
    product_type = EXCLUDED.product_type,
    subtitle = EXCLUDED.subtitle,
    industry = EXCLUDED.industry,
    use_cases = EXCLUDED.use_cases,
    seller_name = EXCLUDED.seller_name,
    seller_type = EXCLUDED.seller_type,
    seller_country_code = EXCLUDED.seller_country_code,
    seller_industry_tags = EXCLUDED.seller_industry_tags,
    seller_reputation_score = EXCLUDED.seller_reputation_score,
    seller_credit_level = EXCLUDED.seller_credit_level,
    seller_risk_level = EXCLUDED.seller_risk_level,
    sku_types = EXCLUDED.sku_types,
    rights_types = EXCLUDED.rights_types,
    delivery_modes = EXCLUDED.delivery_modes,
    price_mode = EXCLUDED.price_mode,
    price_amount = EXCLUDED.price_amount,
    price_min = EXCLUDED.price_min,
    price_max = EXCLUDED.price_max,
    currency_code = EXCLUDED.currency_code,
    listing_status = EXCLUDED.listing_status,
    sample_available = EXCLUDED.sample_available,
    data_classification = EXCLUDED.data_classification,
    quality_score = EXCLUDED.quality_score,
    recent_trade_count = EXCLUDED.recent_trade_count,
    hotness_score = EXCLUDED.hotness_score,
    ranking_features = EXCLUDED.ranking_features,
    document_version = search.product_search_document.document_version + 1,
    source_updated_at = EXCLUDED.source_updated_at,
    index_sync_status = 'pending',
    index_backend = EXCLUDED.index_backend,
    indexed_at = NULL,
    last_index_error = NULL;
END;
$$;

DROP INDEX IF EXISTS idx_product_search_document_visibility;

ALTER TABLE search.product_search_document
  DROP COLUMN IF EXISTS review_status,
  DROP COLUMN IF EXISTS visibility_status,
  DROP COLUMN IF EXISTS visible_to_search;
