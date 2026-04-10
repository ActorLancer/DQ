ALTER TABLE catalog.tag
  ADD COLUMN IF NOT EXISTS tag_code text,
  ADD COLUMN IF NOT EXISTS tag_type text NOT NULL DEFAULT 'keyword',
  ADD COLUMN IF NOT EXISTS tag_group text,
  ADD COLUMN IF NOT EXISTS parent_tag_id uuid REFERENCES catalog.tag(tag_id),
  ADD COLUMN IF NOT EXISTS status text NOT NULL DEFAULT 'active',
  ADD COLUMN IF NOT EXISTS display_order integer NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS searchable_aliases text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now();

UPDATE catalog.tag
SET tag_code = COALESCE(tag_code, lower(regexp_replace(tag_name, '[^a-zA-Z0-9]+', '_', 'g')))
WHERE tag_code IS NULL;

ALTER TABLE catalog.tag
  ALTER COLUMN tag_code SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uq_catalog_tag_code ON catalog.tag(tag_code);
CREATE INDEX IF NOT EXISTS idx_catalog_tag_group_status ON catalog.tag(tag_group, status);

ALTER TABLE catalog.product_tag
  ADD COLUMN IF NOT EXISTS tag_source text NOT NULL DEFAULT 'manual',
  ADD COLUMN IF NOT EXISTS tag_weight numeric(8, 4) NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();

ALTER TABLE search.product_search_document
  ADD COLUMN IF NOT EXISTS product_type text NOT NULL DEFAULT 'data_product',
  ADD COLUMN IF NOT EXISTS subtitle text,
  ADD COLUMN IF NOT EXISTS industry text,
  ADD COLUMN IF NOT EXISTS use_cases text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS seller_name text,
  ADD COLUMN IF NOT EXISTS seller_type text,
  ADD COLUMN IF NOT EXISTS seller_country_code text,
  ADD COLUMN IF NOT EXISTS seller_industry_tags text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS seller_reputation_score numeric(10, 4) NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS seller_credit_level integer NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS seller_risk_level integer NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS sku_types text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS rights_types text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS delivery_modes text[] NOT NULL DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS price_mode text,
  ADD COLUMN IF NOT EXISTS price_amount numeric(20, 8) NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS price_min numeric(20, 8) NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS price_max numeric(20, 8) NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS currency_code text NOT NULL DEFAULT 'CNY',
  ADD COLUMN IF NOT EXISTS listing_status text NOT NULL DEFAULT 'draft',
  ADD COLUMN IF NOT EXISTS sample_available boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS data_classification text,
  ADD COLUMN IF NOT EXISTS quality_score numeric(10, 4) NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS recent_trade_count bigint NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS hotness_score numeric(10, 4) NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS ranking_features jsonb NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN IF NOT EXISTS document_version bigint NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS source_updated_at timestamptz NOT NULL DEFAULT now(),
  ADD COLUMN IF NOT EXISTS index_sync_status text NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS index_backend text NOT NULL DEFAULT 'opensearch',
  ADD COLUMN IF NOT EXISTS indexed_at timestamptz,
  ADD COLUMN IF NOT EXISTS last_index_error text;

CREATE INDEX IF NOT EXISTS idx_product_search_document_listing_status
  ON search.product_search_document(listing_status, product_type);
CREATE INDEX IF NOT EXISTS idx_product_search_document_org_id
  ON search.product_search_document(org_id);
CREATE INDEX IF NOT EXISTS idx_product_search_document_price
  ON search.product_search_document(currency_code, price_min, price_max);
CREATE INDEX IF NOT EXISTS idx_product_search_document_sync_status
  ON search.product_search_document(index_sync_status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_product_search_document_tags_gin
  ON search.product_search_document USING GIN (tags);
CREATE INDEX IF NOT EXISTS idx_product_search_document_use_cases_gin
  ON search.product_search_document USING GIN (use_cases);
CREATE INDEX IF NOT EXISTS idx_product_search_document_seller_industry_tags_gin
  ON search.product_search_document USING GIN (seller_industry_tags);

CREATE TABLE IF NOT EXISTS search.seller_search_document (
  org_id uuid PRIMARY KEY REFERENCES core.organization(org_id) ON DELETE CASCADE,
  seller_name text NOT NULL,
  seller_type text NOT NULL,
  description text,
  country_code text,
  region_code text,
  industry_tags text[] NOT NULL DEFAULT '{}',
  credit_level integer NOT NULL DEFAULT 0,
  risk_level integer NOT NULL DEFAULT 0,
  reputation_score numeric(10, 4) NOT NULL DEFAULT 0,
  listing_product_count integer NOT NULL DEFAULT 0,
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

CREATE INDEX IF NOT EXISTS idx_seller_search_document_tsv
  ON search.seller_search_document USING GIN (searchable_tsv);
CREATE INDEX IF NOT EXISTS idx_seller_search_document_embedding
  ON search.seller_search_document USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_seller_search_document_sync_status
  ON search.seller_search_document(index_sync_status, updated_at DESC);

CREATE TABLE IF NOT EXISTS search.search_signal_aggregate (
  entity_scope text NOT NULL,
  entity_id uuid NOT NULL,
  exposure_count bigint NOT NULL DEFAULT 0,
  click_count bigint NOT NULL DEFAULT 0,
  order_count bigint NOT NULL DEFAULT 0,
  hotness_score numeric(10, 4) NOT NULL DEFAULT 0,
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (entity_scope, entity_id)
);

CREATE TABLE IF NOT EXISTS search.ranking_profile (
  ranking_profile_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_key text NOT NULL UNIQUE,
  entity_scope text NOT NULL,
  backend_type text NOT NULL DEFAULT 'opensearch',
  weights_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  filter_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  stage_from text NOT NULL DEFAULT 'V1',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS search.index_alias_binding (
  alias_binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_scope text NOT NULL,
  backend_type text NOT NULL DEFAULT 'opensearch',
  read_alias text NOT NULL,
  write_alias text NOT NULL,
  active_index_name text,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (entity_scope, backend_type)
);

CREATE TABLE IF NOT EXISTS search.index_sync_task (
  index_sync_task_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_scope text NOT NULL,
  entity_id uuid NOT NULL,
  document_version bigint NOT NULL DEFAULT 1,
  target_backend text NOT NULL DEFAULT 'opensearch',
  target_index text,
  source_event_id uuid REFERENCES ops.outbox_event(outbox_event_id) ON DELETE SET NULL,
  sync_status text NOT NULL DEFAULT 'pending',
  retry_count integer NOT NULL DEFAULT 0,
  last_error_code text,
  last_error_message text,
  scheduled_at timestamptz NOT NULL DEFAULT now(),
  started_at timestamptz,
  completed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_index_sync_task_status
  ON search.index_sync_task(sync_status, scheduled_at);
CREATE INDEX IF NOT EXISTS idx_index_sync_task_scope_entity
  ON search.index_sync_task(entity_scope, entity_id, document_version DESC);

INSERT INTO search.ranking_profile (profile_key, entity_scope, backend_type, weights_json, filter_policy_json, status)
VALUES
  (
    'default_product_v1',
    'product',
    'opensearch',
    '{"lexical":0.40,"quality":0.20,"reputation":0.15,"freshness":0.10,"trade":0.10,"completeness":0.05}'::jsonb,
    '{"exclude_status":["draft","suspended","retired"],"exclude_risk":["blocked","frozen"]}'::jsonb,
    'active'
  ),
  (
    'default_seller_v1',
    'seller',
    'opensearch',
    '{"lexical":0.40,"reputation":0.25,"freshness":0.10,"trade":0.15,"completeness":0.10}'::jsonb,
    '{"exclude_status":["suspended","frozen"]}'::jsonb,
    'active'
  )
ON CONFLICT (profile_key) DO NOTHING;

INSERT INTO search.index_alias_binding (entity_scope, backend_type, read_alias, write_alias, active_index_name, status)
VALUES
  ('product', 'opensearch', 'product_search_read', 'product_search_write', 'product_search_v1_bootstrap', 'active'),
  ('seller', 'opensearch', 'seller_search_read', 'seller_search_write', 'seller_search_v1_bootstrap', 'active')
ON CONFLICT (entity_scope, backend_type) DO NOTHING;

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
      concat_ws(' ', org.org_name, org.org_type, org.country_code, array_to_string(COALESCE(org.industry_tags, '{}'), ' '), COALESCE(org.metadata ->> 'description', ''))
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

CREATE OR REPLACE FUNCTION search.refresh_search_documents_for_org_products(p_org_id uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
  v_product_id uuid;
BEGIN
  PERFORM search.refresh_seller_search_document_by_id(p_org_id);

  FOR v_product_id IN
    SELECT product_id
    FROM catalog.product
    WHERE seller_org_id = p_org_id
  LOOP
    PERFORM search.refresh_product_search_document_by_id(v_product_id);
  END LOOP;
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_product_search_document()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM search.refresh_product_search_document_by_id(NEW.product_id);
  PERFORM search.refresh_seller_search_document_by_id(NEW.seller_org_id);
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_product_search_document_from_product_tag()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM search.refresh_product_search_document_by_id(COALESCE(NEW.product_id, OLD.product_id));
  RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_product_search_document_from_product_sku()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_product_id uuid := COALESCE(NEW.product_id, OLD.product_id);
BEGIN
  PERFORM search.refresh_product_search_document_by_id(v_product_id);
  RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_product_search_document_from_tag()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_product_id uuid;
  v_tag_id uuid := COALESCE(NEW.tag_id, OLD.tag_id);
BEGIN
  FOR v_product_id IN
    SELECT pt.product_id
    FROM catalog.product_tag pt
    WHERE pt.tag_id = v_tag_id
  LOOP
    PERFORM search.refresh_product_search_document_by_id(v_product_id);
  END LOOP;
  RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_seller_search_document_from_org()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM search.refresh_search_documents_for_org_products(COALESCE(NEW.org_id, OLD.org_id));
  RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_product_search_document_from_asset_version()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_product_id uuid;
  v_asset_version_id uuid := COALESCE(NEW.asset_version_id, OLD.asset_version_id);
BEGIN
  FOR v_product_id IN
    SELECT p.product_id
    FROM catalog.product p
    WHERE p.asset_version_id = v_asset_version_id
  LOOP
    PERFORM search.refresh_product_search_document_by_id(v_product_id);
  END LOOP;
  RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_search_document_from_reputation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  IF COALESCE(NEW.subject_type, OLD.subject_type) = 'organization' THEN
    PERFORM search.refresh_search_documents_for_org_products(COALESCE(NEW.subject_id, OLD.subject_id));
  END IF;
  RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE OR REPLACE FUNCTION search.tg_refresh_search_document_from_signal()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_scope text := COALESCE(NEW.entity_scope, OLD.entity_scope);
  v_entity_id uuid := COALESCE(NEW.entity_id, OLD.entity_id);
BEGIN
  IF v_scope = 'product' THEN
    PERFORM search.refresh_product_search_document_by_id(v_entity_id);
  ELSIF v_scope IN ('seller', 'organization') THEN
    PERFORM search.refresh_search_documents_for_org_products(v_entity_id);
  END IF;
  RETURN COALESCE(NEW, OLD);
END;
$$;

DROP TRIGGER IF EXISTS trg_product_search_refresh ON catalog.product;
CREATE TRIGGER trg_product_search_refresh
AFTER INSERT OR UPDATE ON catalog.product
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document();

DROP TRIGGER IF EXISTS trg_product_tag_search_refresh ON catalog.product_tag;
CREATE TRIGGER trg_product_tag_search_refresh
AFTER INSERT OR UPDATE OR DELETE ON catalog.product_tag
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document_from_product_tag();

DROP TRIGGER IF EXISTS trg_product_sku_search_refresh ON catalog.product_sku;
CREATE TRIGGER trg_product_sku_search_refresh
AFTER INSERT OR UPDATE OR DELETE ON catalog.product_sku
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document_from_product_sku();

DROP TRIGGER IF EXISTS trg_tag_search_refresh ON catalog.tag;
CREATE TRIGGER trg_tag_search_refresh
AFTER UPDATE OR DELETE ON catalog.tag
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document_from_tag();

DROP TRIGGER IF EXISTS trg_org_search_refresh ON core.organization;
CREATE TRIGGER trg_org_search_refresh
AFTER INSERT OR UPDATE ON core.organization
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_seller_search_document_from_org();

DROP TRIGGER IF EXISTS trg_asset_version_search_refresh ON catalog.asset_version;
CREATE TRIGGER trg_asset_version_search_refresh
AFTER UPDATE ON catalog.asset_version
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document_from_asset_version();

DROP TRIGGER IF EXISTS trg_asset_sample_search_refresh ON catalog.asset_sample;
CREATE TRIGGER trg_asset_sample_search_refresh
AFTER INSERT OR UPDATE OR DELETE ON catalog.asset_sample
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document_from_asset_version();

DROP TRIGGER IF EXISTS trg_reputation_search_refresh ON risk.reputation_snapshot;
CREATE TRIGGER trg_reputation_search_refresh
AFTER INSERT OR UPDATE ON risk.reputation_snapshot
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_search_document_from_reputation();

DROP TRIGGER IF EXISTS trg_search_signal_refresh ON search.search_signal_aggregate;
CREATE TRIGGER trg_search_signal_refresh
AFTER INSERT OR UPDATE ON search.search_signal_aggregate
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_search_document_from_signal();

CREATE TRIGGER trg_catalog_tag_updated_at BEFORE UPDATE ON catalog.tag
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_seller_search_document_updated_at BEFORE UPDATE ON search.seller_search_document
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_search_signal_aggregate_updated_at BEFORE UPDATE ON search.search_signal_aggregate
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_ranking_profile_updated_at BEFORE UPDATE ON search.ranking_profile
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_index_alias_binding_updated_at BEFORE UPDATE ON search.index_alias_binding
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_index_sync_task_updated_at BEFORE UPDATE ON search.index_sync_task
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

SELECT search.refresh_seller_search_document_by_id(org.org_id)
FROM core.organization org;

SELECT search.refresh_product_search_document_by_id(p.product_id)
FROM catalog.product p;

-- Search sync baseline: PostgreSQL search projection remains authoritative for business visibility, OpenSearch is a read model, and Redis is cache only.
