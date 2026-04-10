DROP TRIGGER IF EXISTS trg_search_signal_refresh ON search.search_signal_aggregate;
DROP TRIGGER IF EXISTS trg_reputation_search_refresh ON risk.reputation_snapshot;
DROP TRIGGER IF EXISTS trg_asset_sample_search_refresh ON catalog.asset_sample;
DROP TRIGGER IF EXISTS trg_asset_version_search_refresh ON catalog.asset_version;
DROP TRIGGER IF EXISTS trg_org_search_refresh ON core.organization;
DROP TRIGGER IF EXISTS trg_tag_search_refresh ON catalog.tag;
DROP TRIGGER IF EXISTS trg_product_sku_search_refresh ON catalog.product_sku;
DROP TRIGGER IF EXISTS trg_product_tag_search_refresh ON catalog.product_tag;
DROP TRIGGER IF EXISTS trg_product_search_refresh ON catalog.product;
DROP TRIGGER IF EXISTS trg_index_sync_task_updated_at ON search.index_sync_task;
DROP TRIGGER IF EXISTS trg_index_alias_binding_updated_at ON search.index_alias_binding;
DROP TRIGGER IF EXISTS trg_ranking_profile_updated_at ON search.ranking_profile;
DROP TRIGGER IF EXISTS trg_search_signal_aggregate_updated_at ON search.search_signal_aggregate;
DROP TRIGGER IF EXISTS trg_seller_search_document_updated_at ON search.seller_search_document;
DROP TRIGGER IF EXISTS trg_catalog_tag_updated_at ON catalog.tag;

DROP FUNCTION IF EXISTS search.tg_refresh_search_document_from_signal();
DROP FUNCTION IF EXISTS search.tg_refresh_search_document_from_reputation();
DROP FUNCTION IF EXISTS search.tg_refresh_product_search_document_from_asset_version();
DROP FUNCTION IF EXISTS search.tg_refresh_seller_search_document_from_org();
DROP FUNCTION IF EXISTS search.tg_refresh_product_search_document_from_tag();
DROP FUNCTION IF EXISTS search.tg_refresh_product_search_document_from_product_sku();
DROP FUNCTION IF EXISTS search.tg_refresh_product_search_document_from_product_tag();
DROP FUNCTION IF EXISTS search.refresh_search_documents_for_org_products(uuid);
DROP FUNCTION IF EXISTS search.refresh_seller_search_document_by_id(uuid);
DROP FUNCTION IF EXISTS search.refresh_product_search_document_by_id(uuid);

DROP TABLE IF EXISTS search.index_sync_task CASCADE;
DROP TABLE IF EXISTS search.index_alias_binding CASCADE;
DROP TABLE IF EXISTS search.ranking_profile CASCADE;
DROP TABLE IF EXISTS search.search_signal_aggregate CASCADE;
DROP TABLE IF EXISTS search.seller_search_document CASCADE;

DROP INDEX IF EXISTS idx_catalog_tag_group_status;
DROP INDEX IF EXISTS uq_catalog_tag_code;
DROP INDEX IF EXISTS idx_product_search_document_listing_status;
DROP INDEX IF EXISTS idx_product_search_document_org_id;
DROP INDEX IF EXISTS idx_product_search_document_price;
DROP INDEX IF EXISTS idx_product_search_document_sync_status;
DROP INDEX IF EXISTS idx_product_search_document_tags_gin;
DROP INDEX IF EXISTS idx_product_search_document_use_cases_gin;
DROP INDEX IF EXISTS idx_product_search_document_seller_industry_tags_gin;

ALTER TABLE search.product_search_document
  DROP COLUMN IF EXISTS product_type,
  DROP COLUMN IF EXISTS subtitle,
  DROP COLUMN IF EXISTS industry,
  DROP COLUMN IF EXISTS use_cases,
  DROP COLUMN IF EXISTS seller_name,
  DROP COLUMN IF EXISTS seller_type,
  DROP COLUMN IF EXISTS seller_country_code,
  DROP COLUMN IF EXISTS seller_industry_tags,
  DROP COLUMN IF EXISTS seller_reputation_score,
  DROP COLUMN IF EXISTS seller_credit_level,
  DROP COLUMN IF EXISTS seller_risk_level,
  DROP COLUMN IF EXISTS sku_types,
  DROP COLUMN IF EXISTS rights_types,
  DROP COLUMN IF EXISTS delivery_modes,
  DROP COLUMN IF EXISTS price_mode,
  DROP COLUMN IF EXISTS price_amount,
  DROP COLUMN IF EXISTS price_min,
  DROP COLUMN IF EXISTS price_max,
  DROP COLUMN IF EXISTS currency_code,
  DROP COLUMN IF EXISTS listing_status,
  DROP COLUMN IF EXISTS sample_available,
  DROP COLUMN IF EXISTS data_classification,
  DROP COLUMN IF EXISTS quality_score,
  DROP COLUMN IF EXISTS recent_trade_count,
  DROP COLUMN IF EXISTS hotness_score,
  DROP COLUMN IF EXISTS ranking_features,
  DROP COLUMN IF EXISTS document_version,
  DROP COLUMN IF EXISTS source_updated_at,
  DROP COLUMN IF EXISTS index_sync_status,
  DROP COLUMN IF EXISTS index_backend,
  DROP COLUMN IF EXISTS indexed_at,
  DROP COLUMN IF EXISTS last_index_error;

ALTER TABLE catalog.product_tag
  DROP COLUMN IF EXISTS tag_source,
  DROP COLUMN IF EXISTS tag_weight,
  DROP COLUMN IF EXISTS created_at;

ALTER TABLE catalog.tag
  DROP COLUMN IF EXISTS tag_code,
  DROP COLUMN IF EXISTS tag_type,
  DROP COLUMN IF EXISTS tag_group,
  DROP COLUMN IF EXISTS parent_tag_id,
  DROP COLUMN IF EXISTS status,
  DROP COLUMN IF EXISTS display_order,
  DROP COLUMN IF EXISTS searchable_aliases,
  DROP COLUMN IF EXISTS metadata,
  DROP COLUMN IF EXISTS updated_at;

CREATE OR REPLACE FUNCTION search.tg_refresh_product_search_document()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO search.product_search_document (
    product_id, org_id, title, category, tags, description, searchable_tsv, updated_at
  )
  SELECT
    p.product_id,
    p.seller_org_id,
    p.title,
    p.category,
    COALESCE(array_agg(t.tag_name) FILTER (WHERE t.tag_name IS NOT NULL), '{}'),
    p.description,
    to_tsvector('simple', COALESCE(p.title, '') || ' ' || COALESCE(p.category, '') || ' ' || COALESCE(p.description, '')),
    now()
  FROM catalog.product p
  LEFT JOIN catalog.product_tag pt ON pt.product_id = p.product_id
  LEFT JOIN catalog.tag t ON t.tag_id = pt.tag_id
  WHERE p.product_id = NEW.product_id
  GROUP BY p.product_id
  ON CONFLICT (product_id) DO UPDATE
  SET
    title = EXCLUDED.title,
    category = EXCLUDED.category,
    tags = EXCLUDED.tags,
    description = EXCLUDED.description,
    searchable_tsv = EXCLUDED.searchable_tsv,
    updated_at = now();

  RETURN NEW;
END;
$$;

CREATE TRIGGER trg_product_search_refresh
AFTER INSERT OR UPDATE ON catalog.product
FOR EACH ROW EXECUTE FUNCTION search.tg_refresh_product_search_document();
