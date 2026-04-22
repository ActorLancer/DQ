ALTER TABLE search.product_search_document
  ADD COLUMN IF NOT EXISTS review_status text NOT NULL DEFAULT 'not_submitted',
  ADD COLUMN IF NOT EXISTS visibility_status text NOT NULL DEFAULT 'hidden',
  ADD COLUMN IF NOT EXISTS visible_to_search boolean NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_product_search_document_visibility
  ON search.product_search_document(visible_to_search, visibility_status, listing_status, updated_at DESC);

CREATE OR REPLACE FUNCTION search.resolve_product_review_status(
  p_listing_status text,
  p_review_task_status text
)
RETURNS text
LANGUAGE sql
IMMUTABLE
AS $$
  SELECT CASE
    WHEN p_listing_status = 'pending_review' THEN 'pending'
    WHEN p_listing_status IN ('listed', 'delisted', 'frozen') THEN 'approved'
    WHEN COALESCE(p_review_task_status, '') = 'approved' THEN 'approved'
    WHEN COALESCE(p_review_task_status, '') = 'rejected' THEN 'rejected'
    WHEN COALESCE(p_review_task_status, '') = 'pending' THEN 'pending'
    ELSE 'not_submitted'
  END
$$;

CREATE OR REPLACE FUNCTION search.resolve_product_visibility_status(
  p_listing_status text,
  p_review_status text,
  p_seller_status text,
  p_risk_blocked boolean
)
RETURNS text
LANGUAGE sql
IMMUTABLE
AS $$
  SELECT CASE
    WHEN COALESCE(p_risk_blocked, false) THEN 'risk_blocked'
    WHEN COALESCE(p_seller_status, '') IN ('suspended', 'frozen', 'blocked') THEN 'seller_blocked'
    WHEN p_listing_status = 'listed' AND p_review_status = 'approved' THEN 'visible'
    WHEN p_listing_status = 'pending_review' THEN 'pending_review'
    WHEN p_listing_status = 'delisted' THEN 'delisted'
    WHEN p_listing_status = 'frozen' THEN 'frozen'
    WHEN p_review_status = 'rejected' THEN 'rejected'
    ELSE 'hidden'
  END
$$;

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
    last_index_error,
    review_status,
    visibility_status,
    visible_to_search
  )
  WITH product_source AS (
    SELECT
      p.product_id,
      p.seller_org_id AS org_id,
      p.title,
      p.category,
      COALESCE(tag_agg.tag_names, '{}') AS tags,
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
      ) AS searchable_tsv,
      NULL::vector(1536) AS embedding,
      now() AS refreshed_at,
      p.product_type,
      COALESCE(NULLIF(p.product_subtitle, ''), NULLIF(p.metadata ->> 'subtitle', '')) AS subtitle,
      COALESCE(p.metadata ->> 'industry', p.category) AS industry,
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
      ) AS use_cases,
      org.org_name AS seller_name,
      org.org_type AS seller_type,
      org.country_code AS seller_country_code,
      COALESCE(org.industry_tags, '{}') AS seller_industry_tags,
      COALESCE(rep.score, 0) AS seller_reputation_score,
      COALESCE(rep.credit_level, org.credit_level, 0) AS seller_credit_level,
      COALESCE(rep.risk_level, org.risk_level, 0) AS seller_risk_level,
      COALESCE(sku_agg.sku_types, ARRAY[p.product_type]) AS sku_types,
      COALESCE(p.allowed_usage, '{}') AS rights_types,
      ARRAY[p.delivery_type] AS delivery_modes,
      p.price_mode,
      p.price AS price_amount,
      p.price AS price_min,
      p.price AS price_max,
      p.currency_code,
      p.status AS listing_status,
      EXISTS (
        SELECT 1
        FROM catalog.asset_sample s
        WHERE s.asset_version_id = p.asset_version_id
      ) AS sample_available,
      COALESCE(av.metadata ->> 'data_classification', p.metadata ->> 'data_classification') AS data_classification,
      CASE
        WHEN COALESCE(p.metadata ->> 'quality_score', '') ~ '^-?[0-9]+(\.[0-9]+)?$'
          THEN (p.metadata ->> 'quality_score')::numeric
        ELSE 0
      END AS quality_score,
      COALESCE(sig.order_count, 0) AS recent_trade_count,
      COALESCE(sig.hotness_score, 0) AS hotness_score,
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
      ) AS ranking_features,
      GREATEST(p.updated_at, org.updated_at, COALESCE(av.updated_at, p.updated_at)) AS source_updated_at,
      'opensearch'::text AS index_backend,
      latest_review.status AS latest_review_status,
      org.status AS seller_status,
      CASE
        WHEN lower(COALESCE(p.metadata ->> 'risk_blocked', 'false')) IN ('true', '1') THEN true
        WHEN lower(COALESCE(p.metadata #>> '{risk_flags,block_submit}', 'false')) IN ('true', '1') THEN true
        ELSE false
      END AS risk_blocked
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
    LEFT JOIN LATERAL (
      SELECT rt.status
      FROM review.review_task rt
      WHERE rt.review_type = 'product_review'
        AND rt.ref_type = 'product'
        AND rt.ref_id = p.product_id
      ORDER BY rt.updated_at DESC, rt.created_at DESC, rt.review_task_id DESC
      LIMIT 1
    ) AS latest_review ON true
    LEFT JOIN search.search_signal_aggregate sig
      ON sig.entity_scope = 'product' AND sig.entity_id = p.product_id
    WHERE p.product_id = p_product_id
  )
  SELECT
    product_id,
    org_id,
    title,
    category,
    tags,
    description,
    searchable_tsv,
    embedding,
    refreshed_at,
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
    1,
    source_updated_at,
    'pending',
    index_backend,
    NULL,
    NULL,
    search.resolve_product_review_status(listing_status, latest_review_status),
    search.resolve_product_visibility_status(
      listing_status,
      search.resolve_product_review_status(listing_status, latest_review_status),
      seller_status,
      risk_blocked
    ),
    search.resolve_product_visibility_status(
      listing_status,
      search.resolve_product_review_status(listing_status, latest_review_status),
      seller_status,
      risk_blocked
    ) = 'visible'
  FROM product_source
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
    last_index_error = NULL,
    review_status = EXCLUDED.review_status,
    visibility_status = EXCLUDED.visibility_status,
    visible_to_search = EXCLUDED.visible_to_search;
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_product_search_document_from_review_task()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  IF NEW.review_type = 'product_review' AND NEW.ref_type = 'product' THEN
    PERFORM search.refresh_product_search_document_by_id(NEW.ref_id);
  END IF;
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_review_task_search_refresh ON review.review_task;
CREATE TRIGGER trg_review_task_search_refresh
AFTER INSERT OR UPDATE OF status ON review.review_task
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document_from_review_task();

SELECT search.refresh_product_search_document_by_id(p.product_id)
FROM catalog.product p;
