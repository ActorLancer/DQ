INSERT INTO catalog.data_asset (
  asset_id, owner_org_id, title, category, sensitivity_level, status, storage_mode, payload_location_type, custody_mode, key_control_mode, description, metadata
) VALUES
  ('20000000-0000-0000-0000-000000000101'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'V1 FILE_STD Asset', 'category_finance', 'internal', 'active', 'platform_custody', 'platform_object_storage', 'platform_managed', 'seller_managed', 'FILE_STD demo asset', '{"seed":"db028","sku":"FILE_STD"}'::jsonb),
  ('20000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'V1 FILE_SUB Asset', 'category_finance', 'internal', 'active', 'platform_custody', 'platform_object_storage', 'platform_managed', 'seller_managed', 'FILE_SUB demo asset', '{"seed":"db028","sku":"FILE_SUB"}'::jsonb),
  ('20000000-0000-0000-0000-000000000103'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'V1 SHARE_RO Asset', 'category_supply_chain', 'internal', 'active', 'platform_custody', 'platform_object_storage', 'platform_managed', 'seller_managed', 'SHARE_RO demo asset', '{"seed":"db028","sku":"SHARE_RO"}'::jsonb),
  ('20000000-0000-0000-0000-000000000104'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'V1 API_SUB Asset', 'category_marketing', 'internal', 'active', 'platform_custody', 'platform_object_storage', 'platform_managed', 'seller_managed', 'API_SUB demo asset', '{"seed":"db028","sku":"API_SUB"}'::jsonb),
  ('20000000-0000-0000-0000-000000000105'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'V1 API_PPU Asset', 'category_marketing', 'internal', 'active', 'platform_custody', 'platform_object_storage', 'platform_managed', 'seller_managed', 'API_PPU demo asset', '{"seed":"db028","sku":"API_PPU"}'::jsonb),
  ('20000000-0000-0000-0000-000000000106'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'V1 QRY_LITE Asset', 'category_credit', 'restricted', 'active', 'platform_custody', 'platform_object_storage', 'platform_managed', 'seller_managed', 'QRY_LITE demo asset', '{"seed":"db028","sku":"QRY_LITE"}'::jsonb),
  ('20000000-0000-0000-0000-000000000107'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'V1 SBX_STD Asset', 'category_credit', 'restricted', 'active', 'platform_custody', 'platform_object_storage', 'platform_managed', 'seller_managed', 'SBX_STD demo asset', '{"seed":"db028","sku":"SBX_STD"}'::jsonb),
  ('20000000-0000-0000-0000-000000000108'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'V1 RPT_STD Asset', 'category_supply_chain', 'internal', 'active', 'platform_custody', 'platform_object_storage', 'platform_managed', 'seller_managed', 'RPT_STD demo asset', '{"seed":"db028","sku":"RPT_STD"}'::jsonb)
ON CONFLICT (asset_id) DO UPDATE
SET
  owner_org_id = EXCLUDED.owner_org_id,
  title = EXCLUDED.title,
  category = EXCLUDED.category,
  sensitivity_level = EXCLUDED.sensitivity_level,
  status = EXCLUDED.status,
  storage_mode = EXCLUDED.storage_mode,
  payload_location_type = EXCLUDED.payload_location_type,
  custody_mode = EXCLUDED.custody_mode,
  key_control_mode = EXCLUDED.key_control_mode,
  description = EXCLUDED.description,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

INSERT INTO catalog.asset_version (
  asset_version_id, asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash, data_size_bytes, origin_region, allowed_region, status, release_mode, is_revision_subscribable, update_frequency, metadata
) VALUES
  ('20000000-0000-0000-0000-000000000201'::uuid, '20000000-0000-0000-0000-000000000101'::uuid, 1, 'v1', 'hash-file-std-v1', 'sample-file-std-v1', 'full-file-std-v1', 10485760, 'cn-sh', ARRAY['cn-sh'], 'published', 'snapshot', false, NULL, '{"seed":"db028","sku":"FILE_STD"}'::jsonb),
  ('20000000-0000-0000-0000-000000000202'::uuid, '20000000-0000-0000-0000-000000000102'::uuid, 1, 'v1', 'hash-file-sub-v1', 'sample-file-sub-v1', 'full-file-sub-v1', 20971520, 'cn-sh', ARRAY['cn-sh'], 'published', 'snapshot', true, 'monthly', '{"seed":"db028","sku":"FILE_SUB"}'::jsonb),
  ('20000000-0000-0000-0000-000000000203'::uuid, '20000000-0000-0000-0000-000000000103'::uuid, 1, 'v1', 'hash-share-ro-v1', 'sample-share-ro-v1', 'full-share-ro-v1', 15728640, 'cn-bj', ARRAY['cn-bj'], 'published', 'snapshot', false, NULL, '{"seed":"db028","sku":"SHARE_RO"}'::jsonb),
  ('20000000-0000-0000-0000-000000000204'::uuid, '20000000-0000-0000-0000-000000000104'::uuid, 1, 'v1', 'hash-api-sub-v1', 'sample-api-sub-v1', 'full-api-sub-v1', 8388608, 'cn-sh', ARRAY['cn-sh'], 'published', 'snapshot', true, 'monthly', '{"seed":"db028","sku":"API_SUB"}'::jsonb),
  ('20000000-0000-0000-0000-000000000205'::uuid, '20000000-0000-0000-0000-000000000105'::uuid, 1, 'v1', 'hash-api-ppu-v1', 'sample-api-ppu-v1', 'full-api-ppu-v1', 8388608, 'cn-sh', ARRAY['cn-sh'], 'published', 'snapshot', false, NULL, '{"seed":"db028","sku":"API_PPU"}'::jsonb),
  ('20000000-0000-0000-0000-000000000206'::uuid, '20000000-0000-0000-0000-000000000106'::uuid, 1, 'v1', 'hash-qry-lite-v1', 'sample-qry-lite-v1', 'full-qry-lite-v1', 6291456, 'cn-sz', ARRAY['cn-sz'], 'published', 'snapshot', false, NULL, '{"seed":"db028","sku":"QRY_LITE"}'::jsonb),
  ('20000000-0000-0000-0000-000000000207'::uuid, '20000000-0000-0000-0000-000000000107'::uuid, 1, 'v1', 'hash-sbx-std-v1', 'sample-sbx-std-v1', 'full-sbx-std-v1', 31457280, 'cn-sz', ARRAY['cn-sz'], 'published', 'snapshot', false, NULL, '{"seed":"db028","sku":"SBX_STD"}'::jsonb),
  ('20000000-0000-0000-0000-000000000208'::uuid, '20000000-0000-0000-0000-000000000108'::uuid, 1, 'v1', 'hash-rpt-std-v1', 'sample-rpt-std-v1', 'full-rpt-std-v1', 4194304, 'cn-bj', ARRAY['cn-bj'], 'published', 'snapshot', false, NULL, '{"seed":"db028","sku":"RPT_STD"}'::jsonb)
ON CONFLICT (asset_version_id) DO UPDATE
SET
  asset_id = EXCLUDED.asset_id,
  version_no = EXCLUDED.version_no,
  schema_version = EXCLUDED.schema_version,
  schema_hash = EXCLUDED.schema_hash,
  sample_hash = EXCLUDED.sample_hash,
  full_hash = EXCLUDED.full_hash,
  data_size_bytes = EXCLUDED.data_size_bytes,
  origin_region = EXCLUDED.origin_region,
  allowed_region = EXCLUDED.allowed_region,
  status = EXCLUDED.status,
  release_mode = EXCLUDED.release_mode,
  is_revision_subscribable = EXCLUDED.is_revision_subscribable,
  update_frequency = EXCLUDED.update_frequency,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

INSERT INTO catalog.product (
  product_id, asset_id, asset_version_id, seller_org_id, title, category, product_type, description, status, price_mode, price, currency_code, delivery_type, allowed_usage, searchable_text, metadata
) VALUES
  ('20000000-0000-0000-0000-000000000301'::uuid, '20000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000201'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Demo Product FILE_STD', 'category_finance', 'standard_data_product', 'V1 SKU FILE_STD demo product', 'listed', 'one_time', 1999.00, 'CNY', 'file', ARRAY['analytics','modeling'], 'FILE_STD finance sample', '{"seed":"db028","sku":"FILE_STD"}'::jsonb),
  ('20000000-0000-0000-0000-000000000302'::uuid, '20000000-0000-0000-0000-000000000102'::uuid, '20000000-0000-0000-0000-000000000202'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Demo Product FILE_SUB', 'category_finance', 'standard_data_product', 'V1 SKU FILE_SUB demo product', 'listed', 'subscription', 3999.00, 'CNY', 'file_subscription', ARRAY['analytics'], 'FILE_SUB finance sample', '{"seed":"db028","sku":"FILE_SUB"}'::jsonb),
  ('20000000-0000-0000-0000-000000000303'::uuid, '20000000-0000-0000-0000-000000000103'::uuid, '20000000-0000-0000-0000-000000000203'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Demo Product SHARE_RO', 'category_supply_chain', 'standard_data_product', 'V1 SKU SHARE_RO demo product', 'listed', 'one_time', 2999.00, 'CNY', 'share', ARRAY['analytics'], 'SHARE_RO supply chain sample', '{"seed":"db028","sku":"SHARE_RO"}'::jsonb),
  ('20000000-0000-0000-0000-000000000304'::uuid, '20000000-0000-0000-0000-000000000104'::uuid, '20000000-0000-0000-0000-000000000204'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Demo Product API_SUB', 'category_marketing', 'standard_data_product', 'V1 SKU API_SUB demo product', 'listed', 'subscription', 4999.00, 'CNY', 'api_subscription', ARRAY['api_access'], 'API_SUB marketing sample', '{"seed":"db028","sku":"API_SUB"}'::jsonb),
  ('20000000-0000-0000-0000-000000000305'::uuid, '20000000-0000-0000-0000-000000000105'::uuid, '20000000-0000-0000-0000-000000000205'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Demo Product API_PPU', 'category_marketing', 'standard_data_product', 'V1 SKU API_PPU demo product', 'listed', 'metered', 0.80, 'CNY', 'api_ppu', ARRAY['api_access'], 'API_PPU marketing sample', '{"seed":"db028","sku":"API_PPU"}'::jsonb),
  ('20000000-0000-0000-0000-000000000306'::uuid, '20000000-0000-0000-0000-000000000106'::uuid, '20000000-0000-0000-0000-000000000206'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Demo Product QRY_LITE', 'category_credit', 'standard_data_product', 'V1 SKU QRY_LITE demo product', 'listed', 'one_time', 2599.00, 'CNY', 'query_template', ARRAY['query_execution'], 'QRY_LITE credit sample', '{"seed":"db028","sku":"QRY_LITE"}'::jsonb),
  ('20000000-0000-0000-0000-000000000307'::uuid, '20000000-0000-0000-0000-000000000107'::uuid, '20000000-0000-0000-0000-000000000207'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Demo Product SBX_STD', 'category_credit', 'standard_data_product', 'V1 SKU SBX_STD demo product', 'listed', 'one_time', 6999.00, 'CNY', 'sandbox', ARRAY['sandbox_execution'], 'SBX_STD credit sample', '{"seed":"db028","sku":"SBX_STD"}'::jsonb),
  ('20000000-0000-0000-0000-000000000308'::uuid, '20000000-0000-0000-0000-000000000108'::uuid, '20000000-0000-0000-0000-000000000208'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, 'Demo Product RPT_STD', 'category_supply_chain', 'standard_data_product', 'V1 SKU RPT_STD demo product', 'listed', 'one_time', 3299.00, 'CNY', 'report', ARRAY['report_delivery'], 'RPT_STD supply chain sample', '{"seed":"db028","sku":"RPT_STD"}'::jsonb)
ON CONFLICT (product_id) DO UPDATE
SET
  asset_id = EXCLUDED.asset_id,
  asset_version_id = EXCLUDED.asset_version_id,
  seller_org_id = EXCLUDED.seller_org_id,
  title = EXCLUDED.title,
  category = EXCLUDED.category,
  product_type = EXCLUDED.product_type,
  description = EXCLUDED.description,
  status = EXCLUDED.status,
  price_mode = EXCLUDED.price_mode,
  price = EXCLUDED.price,
  currency_code = EXCLUDED.currency_code,
  delivery_type = EXCLUDED.delivery_type,
  allowed_usage = EXCLUDED.allowed_usage,
  searchable_text = EXCLUDED.searchable_text,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

INSERT INTO catalog.product_sku (
  sku_id, product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status, trade_mode, delivery_object_kind, subscription_cadence, share_protocol, result_form, metadata
) VALUES
  ('20000000-0000-0000-0000-000000000401'::uuid, '20000000-0000-0000-0000-000000000301'::uuid, 'FILE_STD', 'file', 'dataset', 'one_time', 'manual', 'full', 'active', 'snapshot_sale', 'file', NULL, NULL, 'file', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000402'::uuid, '20000000-0000-0000-0000-000000000302'::uuid, 'FILE_SUB', 'file_subscription', 'dataset', 'subscription', 'manual', 'partial', 'active', 'subscription_sale', 'file', 'monthly', NULL, 'file', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000403'::uuid, '20000000-0000-0000-0000-000000000303'::uuid, 'SHARE_RO', 'share', 'dataset', 'one_time', 'auto', 'none', 'active', 'share_grant', 'table', NULL, 'db_link', 'share', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000404'::uuid, '20000000-0000-0000-0000-000000000304'::uuid, 'API_SUB', 'api_subscription', 'request', 'subscription', 'auto', 'partial', 'active', 'api_subscription', 'api', 'monthly', 'https', 'api', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000405'::uuid, '20000000-0000-0000-0000-000000000305'::uuid, 'API_PPU', 'api_ppu', 'request', 'metered', 'auto', 'partial', 'active', 'api_ppu', 'api', NULL, 'https', 'api', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000406'::uuid, '20000000-0000-0000-0000-000000000306'::uuid, 'QRY_LITE', 'query_template', 'query_run', 'one_time', 'manual', 'none', 'active', 'template_query', 'query_template', NULL, NULL, 'query_result', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000407'::uuid, '20000000-0000-0000-0000-000000000307'::uuid, 'SBX_STD', 'sandbox', 'workspace', 'one_time', 'manual', 'none', 'active', 'sandbox_access', 'sandbox', NULL, NULL, 'sandbox_output', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000408'::uuid, '20000000-0000-0000-0000-000000000308'::uuid, 'RPT_STD', 'report', 'report', 'one_time', 'manual', 'full', 'active', 'report_delivery', 'report', NULL, NULL, 'report', '{"seed":"db028"}'::jsonb)
ON CONFLICT (sku_id) DO UPDATE
SET
  product_id = EXCLUDED.product_id,
  sku_code = EXCLUDED.sku_code,
  sku_type = EXCLUDED.sku_type,
  unit_name = EXCLUDED.unit_name,
  billing_mode = EXCLUDED.billing_mode,
  acceptance_mode = EXCLUDED.acceptance_mode,
  refund_mode = EXCLUDED.refund_mode,
  status = EXCLUDED.status,
  trade_mode = EXCLUDED.trade_mode,
  delivery_object_kind = EXCLUDED.delivery_object_kind,
  subscription_cadence = EXCLUDED.subscription_cadence,
  share_protocol = EXCLUDED.share_protocol,
  result_form = EXCLUDED.result_form,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

INSERT INTO contract.template_definition (
  template_id, template_type, template_name, version_no, applicable_sku_types, configurable_fields, locked_fields, content_digest, status, metadata
) VALUES
  ('20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'V1 Standard Contract Template', 1, ARRAY['file','file_subscription','share','api_subscription','api_ppu','query_template','sandbox','report'], '["term_days","license_scope"]'::jsonb, '["platform_clauses"]'::jsonb, 'digest-contract-v1', 'active', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'V1 Standard Acceptance Template', 1, ARRAY['file','file_subscription','share','api_subscription','api_ppu','query_template','sandbox','report'], '["accept_window_hours"]'::jsonb, '["default_accept_rules"]'::jsonb, 'digest-acceptance-v1', 'active', '{"seed":"db028"}'::jsonb),
  ('20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'V1 Standard Refund Template', 1, ARRAY['file','file_subscription','share','api_subscription','api_ppu','query_template','sandbox','report'], '["refund_window_hours"]'::jsonb, '["base_refund_policy"]'::jsonb, 'digest-refund-v1', 'active', '{"seed":"db028"}'::jsonb)
ON CONFLICT (template_id) DO UPDATE
SET
  template_type = EXCLUDED.template_type,
  template_name = EXCLUDED.template_name,
  version_no = EXCLUDED.version_no,
  applicable_sku_types = EXCLUDED.applicable_sku_types,
  configurable_fields = EXCLUDED.configurable_fields,
  locked_fields = EXCLUDED.locked_fields,
  content_digest = EXCLUDED.content_digest,
  status = EXCLUDED.status,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

INSERT INTO contract.template_binding (template_binding_id, sku_id, template_id, binding_type, status) VALUES
  ('20000000-0000-0000-0000-000000000601'::uuid, '20000000-0000-0000-0000-000000000401'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000602'::uuid, '20000000-0000-0000-0000-000000000402'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000603'::uuid, '20000000-0000-0000-0000-000000000403'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000604'::uuid, '20000000-0000-0000-0000-000000000404'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000605'::uuid, '20000000-0000-0000-0000-000000000405'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000606'::uuid, '20000000-0000-0000-0000-000000000406'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000607'::uuid, '20000000-0000-0000-0000-000000000407'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000608'::uuid, '20000000-0000-0000-0000-000000000408'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000609'::uuid, '20000000-0000-0000-0000-000000000401'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000610'::uuid, '20000000-0000-0000-0000-000000000402'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000611'::uuid, '20000000-0000-0000-0000-000000000404'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000612'::uuid, '20000000-0000-0000-0000-000000000405'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active')
ON CONFLICT (template_binding_id) DO UPDATE
SET
  sku_id = EXCLUDED.sku_id,
  template_id = EXCLUDED.template_id,
  binding_type = EXCLUDED.binding_type,
  status = EXCLUDED.status;
