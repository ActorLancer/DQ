ALTER TABLE search.seller_search_document
  ADD COLUMN IF NOT EXISTS certification_tags text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS featured_products jsonb NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN IF NOT EXISTS rating_summary jsonb NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX IF NOT EXISTS idx_seller_search_document_certification_tags_gin
  ON search.seller_search_document USING GIN (certification_tags);

CREATE OR REPLACE FUNCTION search.resolve_seller_certification_tags(
  p_real_name_status text,
  p_compliance_level text,
  p_metadata jsonb
)
RETURNS text[]
LANGUAGE sql
AS $$
  WITH raw_tags AS (
    SELECT tag
    FROM (
      SELECT CASE
        WHEN lower(COALESCE(p_real_name_status, '')) IN ('verified', 'approved') THEN
          'real_name_verified'
        WHEN NULLIF(trim(COALESCE(p_real_name_status, '')), '') IS NOT NULL THEN
          format('real_name:%s', lower(trim(p_real_name_status)))
        ELSE NULL
      END AS tag
      UNION ALL
      SELECT CASE
        WHEN NULLIF(trim(COALESCE(p_compliance_level, '')), '') IS NOT NULL THEN
          format('compliance:%s', lower(trim(p_compliance_level)))
        ELSE NULL
      END
      UNION ALL
      SELECT CASE
        WHEN NULLIF(trim(COALESCE(p_metadata ->> 'certification_level', '')), '') IS NOT NULL THEN
          format(
            'certification:%s',
            lower(trim(COALESCE(p_metadata ->> 'certification_level', '')))
          )
        ELSE NULL
      END
      UNION ALL
      SELECT NULLIF(lower(trim(value)), '')
      FROM jsonb_array_elements_text(
        CASE
          WHEN jsonb_typeof(p_metadata -> 'certification_tags') = 'array' THEN
            p_metadata -> 'certification_tags'
          ELSE
            '[]'::jsonb
        END
      ) AS value
    ) tags
    WHERE tag IS NOT NULL
  )
  SELECT COALESCE(array_agg(DISTINCT tag ORDER BY tag), '{}')
  FROM raw_tags;
$$;

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
    certification_tags,
    featured_products,
    rating_summary,
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
    COALESCE(cert.certification_tags, '{}'),
    COALESCE(featured.featured_products, '[]'::jsonb),
    jsonb_strip_nulls(
      jsonb_build_object(
        'rating_count',
        COALESCE(
          CASE
            WHEN COALESCE(rep.metadata ->> 'rating_count', '') ~ '^[0-9]+$' THEN
              (rep.metadata ->> 'rating_count')::integer
            ELSE 0
          END,
          0
        ),
        'average_rating',
        CASE
          WHEN COALESCE(rep.metadata ->> 'average_rating', '') ~ '^-?[0-9]+(\.[0-9]+)?$' THEN
            (rep.metadata ->> 'average_rating')::numeric
          ELSE NULL
        END,
        'last_rating_at',
        NULLIF(rep.metadata ->> 'last_rating_at', ''),
        'reputation_score',
        COALESCE(rep.score, 0),
        'credit_level',
        COALESCE(rep.credit_level, org.credit_level, 0),
        'risk_level',
        COALESCE(rep.risk_level, org.risk_level, 0),
        'effective_at',
        CASE
          WHEN rep.effective_at IS NULL THEN NULL
          ELSE to_char(
            rep.effective_at AT TIME ZONE 'UTC',
            'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'
          )
        END
      )
    ),
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
        COALESCE(org.country_code, ''),
        COALESCE(org.region_code, ''),
        array_to_string(COALESCE(org.industry_tags, '{}'), ' '),
        array_to_string(COALESCE(cert.certification_tags, '{}'), ' '),
        array_to_string(COALESCE(featured.featured_titles, '{}'), ' '),
        COALESCE(org.metadata ->> 'description', '')
      )
    ),
    NULL,
    jsonb_strip_nulls(
      jsonb_build_object(
        'reputation_score',
        COALESCE(rep.score, 0),
        'credit_level',
        COALESCE(rep.credit_level, org.credit_level, 0),
        'risk_level',
        COALESCE(rep.risk_level, org.risk_level, 0),
        'listing_product_count',
        COALESCE(listed.listing_product_count, 0),
        'rating_count',
        COALESCE(
          CASE
            WHEN COALESCE(rep.metadata ->> 'rating_count', '') ~ '^[0-9]+$' THEN
              (rep.metadata ->> 'rating_count')::integer
            ELSE 0
          END,
          0
        ),
        'average_rating',
        CASE
          WHEN COALESCE(rep.metadata ->> 'average_rating', '') ~ '^-?[0-9]+(\.[0-9]+)?$' THEN
            (rep.metadata ->> 'average_rating')::numeric
          ELSE NULL
        END
      )
    ),
    1,
    GREATEST(
      org.updated_at,
      COALESCE(rep.effective_at, org.updated_at),
      COALESCE(featured.featured_updated_at, org.updated_at)
    ),
    'pending',
    'opensearch',
    NULL,
    NULL,
    now()
  FROM core.organization org
  LEFT JOIN LATERAL (
    SELECT rs.score, rs.credit_level, rs.risk_level, rs.effective_at, rs.metadata
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
  LEFT JOIN LATERAL (
    SELECT search.resolve_seller_certification_tags(
      org.real_name_status,
      org.compliance_level,
      org.metadata
    ) AS certification_tags
  ) AS cert ON true
  LEFT JOIN LATERAL (
    WITH ranked_products AS (
      SELECT
        p.product_id::text AS product_id,
        p.title,
        NULLIF(p.metadata ->> 'subtitle', '') AS subtitle,
        p.category,
        p.price,
        p.currency_code,
        p.updated_at,
        row_number() OVER (
          ORDER BY
            CASE
              WHEN lower(COALESCE(p.metadata ->> 'is_featured', 'false')) IN ('true', '1') THEN
                0
              ELSE
                1
            END,
            CASE
              WHEN COALESCE(p.metadata ->> 'featured_rank', '') ~ '^[0-9]+$' THEN
                (p.metadata ->> 'featured_rank')::integer
              ELSE
                2147483647
            END,
            CASE
              WHEN COALESCE(p.metadata ->> 'quality_score', '') ~ '^-?[0-9]+(\.[0-9]+)?$' THEN
                (p.metadata ->> 'quality_score')::numeric
              ELSE
                0
            END DESC,
            p.updated_at DESC,
            p.product_id
        ) AS rank_no
      FROM catalog.product p
      WHERE p.seller_org_id = org.org_id
        AND p.status = 'listed'
    )
    SELECT
      COALESCE(array_agg(title ORDER BY rank_no), '{}') AS featured_titles,
      COALESCE(
        jsonb_agg(
          jsonb_build_object(
            'product_id', product_id,
            'title', title,
            'subtitle', subtitle,
            'category', category,
            'price_amount', price,
            'currency_code', currency_code
          )
          ORDER BY rank_no
        ),
        '[]'::jsonb
      ) AS featured_products,
      MAX(updated_at) AS featured_updated_at
    FROM ranked_products
    WHERE rank_no <= 3
  ) AS featured ON true
  WHERE org.org_id = p_org_id
  ON CONFLICT (org_id) DO UPDATE
  SET
    seller_name = EXCLUDED.seller_name,
    seller_type = EXCLUDED.seller_type,
    description = EXCLUDED.description,
    country_code = EXCLUDED.country_code,
    region_code = EXCLUDED.region_code,
    industry_tags = EXCLUDED.industry_tags,
    certification_tags = EXCLUDED.certification_tags,
    featured_products = EXCLUDED.featured_products,
    rating_summary = EXCLUDED.rating_summary,
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
