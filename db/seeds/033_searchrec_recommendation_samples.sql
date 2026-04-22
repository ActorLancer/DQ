INSERT INTO core.organization (
  org_id,
  org_name,
  org_type,
  status,
  real_name_status,
  compliance_level,
  credit_level,
  risk_level,
  partner_type,
  industry_tags,
  country_code,
  region_code,
  metadata
) VALUES (
  '10000000-0000-0000-0000-000000000104'::uuid,
  'Luna Retail Seller Org',
  'tenant',
  'active',
  'approved',
  'L2',
  78,
  1,
  'seller',
  ARRAY['industry_retail', 'industry_location_service'],
  'CN',
  'CN-SH',
  '{"seed":"searchrec014","tenant_type":"seller","industry_scope":"retail"}'::jsonb
)
ON CONFLICT (org_id) DO UPDATE
SET
  org_name = EXCLUDED.org_name,
  org_type = EXCLUDED.org_type,
  status = EXCLUDED.status,
  real_name_status = EXCLUDED.real_name_status,
  compliance_level = EXCLUDED.compliance_level,
  credit_level = EXCLUDED.credit_level,
  risk_level = EXCLUDED.risk_level,
  partner_type = EXCLUDED.partner_type,
  industry_tags = EXCLUDED.industry_tags,
  country_code = EXCLUDED.country_code,
  region_code = EXCLUDED.region_code,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

INSERT INTO catalog.data_asset (
  asset_id,
  owner_org_id,
  title,
  category,
  sensitivity_level,
  status,
  storage_mode,
  payload_location_type,
  custody_mode,
  key_control_mode,
  description,
  metadata
) VALUES
  (
    '20000000-0000-0000-0000-000000000109'::uuid,
    '10000000-0000-0000-0000-000000000101'::uuid,
    '工业设备运行指标 API 订阅',
    'category_marketing',
    'internal',
    'active',
    'platform_custody',
    'platform_object_storage',
    'platform_managed',
    'seller_managed',
    'SEARCHREC-014 official scenario asset S1',
    '{"seed":"searchrec014","standard_scenario_code":"S1","primary_sku":"API_SUB","supplementary_skus":["API_PPU"]}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000110'::uuid,
    '10000000-0000-0000-0000-000000000101'::uuid,
    '工业质量与产线日报文件包交付',
    'category_supply_chain',
    'internal',
    'active',
    'platform_custody',
    'platform_object_storage',
    'platform_managed',
    'seller_managed',
    'SEARCHREC-014 official scenario asset S2',
    '{"seed":"searchrec014","standard_scenario_code":"S2","primary_sku":"FILE_STD","supplementary_skus":["FILE_SUB"]}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000111'::uuid,
    '10000000-0000-0000-0000-000000000101'::uuid,
    '供应链协同查询沙箱',
    'category_supply_chain',
    'restricted',
    'active',
    'platform_custody',
    'platform_object_storage',
    'platform_managed',
    'seller_managed',
    'SEARCHREC-014 official scenario asset S3',
    '{"seed":"searchrec014","standard_scenario_code":"S3","primary_sku":"SBX_STD","supplementary_skus":["SHARE_RO"]}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000112'::uuid,
    '10000000-0000-0000-0000-000000000104'::uuid,
    '零售门店经营分析 API / 报告订阅',
    'category_marketing',
    'internal',
    'active',
    'platform_custody',
    'platform_object_storage',
    'platform_managed',
    'seller_managed',
    'SEARCHREC-014 official scenario asset S4',
    '{"seed":"searchrec014","standard_scenario_code":"S4","primary_sku":"API_SUB","supplementary_skus":["RPT_STD"]}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000113'::uuid,
    '10000000-0000-0000-0000-000000000104'::uuid,
    '商圈/门店选址查询服务',
    'category_credit',
    'restricted',
    'active',
    'platform_custody',
    'platform_object_storage',
    'platform_managed',
    'seller_managed',
    'SEARCHREC-014 official scenario asset S5',
    '{"seed":"searchrec014","standard_scenario_code":"S5","primary_sku":"QRY_LITE","supplementary_skus":["RPT_STD"]}'::jsonb
  )
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
  asset_version_id,
  asset_id,
  version_no,
  schema_version,
  schema_hash,
  sample_hash,
  full_hash,
  data_size_bytes,
  origin_region,
  allowed_region,
  status,
  release_mode,
  is_revision_subscribable,
  update_frequency,
  metadata
) VALUES
  (
    '20000000-0000-0000-0000-000000000209'::uuid,
    '20000000-0000-0000-0000-000000000109'::uuid,
    1,
    'v1',
    'searchrec014-s1-schema',
    'searchrec014-s1-sample',
    'searchrec014-s1-full',
    8388608,
    'cn-sh',
    ARRAY['cn-sh'],
    'published',
    'snapshot',
    true,
    'monthly',
    '{"seed":"searchrec014","standard_scenario_code":"S1"}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000210'::uuid,
    '20000000-0000-0000-0000-000000000110'::uuid,
    1,
    'v1',
    'searchrec014-s2-schema',
    'searchrec014-s2-sample',
    'searchrec014-s2-full',
    12582912,
    'cn-sh',
    ARRAY['cn-sh'],
    'published',
    'snapshot',
    true,
    'monthly',
    '{"seed":"searchrec014","standard_scenario_code":"S2"}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000211'::uuid,
    '20000000-0000-0000-0000-000000000111'::uuid,
    1,
    'v1',
    'searchrec014-s3-schema',
    'searchrec014-s3-sample',
    'searchrec014-s3-full',
    15728640,
    'cn-sz',
    ARRAY['cn-sz'],
    'published',
    'snapshot',
    false,
    NULL,
    '{"seed":"searchrec014","standard_scenario_code":"S3"}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000212'::uuid,
    '20000000-0000-0000-0000-000000000112'::uuid,
    1,
    'v1',
    'searchrec014-s4-schema',
    'searchrec014-s4-sample',
    'searchrec014-s4-full',
    8388608,
    'cn-sh',
    ARRAY['cn-sh'],
    'published',
    'snapshot',
    true,
    'monthly',
    '{"seed":"searchrec014","standard_scenario_code":"S4"}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000213'::uuid,
    '20000000-0000-0000-0000-000000000113'::uuid,
    1,
    'v1',
    'searchrec014-s5-schema',
    'searchrec014-s5-sample',
    'searchrec014-s5-full',
    6291456,
    'cn-sh',
    ARRAY['cn-sh'],
    'published',
    'snapshot',
    false,
    NULL,
    '{"seed":"searchrec014","standard_scenario_code":"S5"}'::jsonb
  )
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
  product_id,
  asset_id,
  asset_version_id,
  seller_org_id,
  title,
  category,
  product_type,
  description,
  status,
  price_mode,
  price,
  currency_code,
  delivery_type,
  allowed_usage,
  searchable_text,
  metadata
) VALUES
  (
    '20000000-0000-0000-0000-000000000309'::uuid,
    '20000000-0000-0000-0000-000000000109'::uuid,
    '20000000-0000-0000-0000-000000000209'::uuid,
    '10000000-0000-0000-0000-000000000101'::uuid,
    '工业设备运行指标 API 订阅',
    'manufacturing',
    'service',
    '设备运行状态、稼动率与能耗指标 API 订阅样例。',
    'listed',
    'subscription',
    5999.00,
    'CNY',
    'api_subscription',
    ARRAY['analytics', 'api_access'],
    '工业设备运行指标 API 订阅 稼动率 能耗 API_SUB API_PPU',
    '{"seed":"searchrec014","standard_scenario_code":"S1","scenario_name":"工业设备运行指标 API 订阅","subtitle":"首页固定推荐样例 S1","industry":"industrial_manufacturing","primary_sku":"API_SUB","supplementary_skus":["API_PPU"],"review_status":"approved","visibility_status":"visible","visible_to_search":true,"recommended_placement_code":"home_featured"}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000310'::uuid,
    '20000000-0000-0000-0000-000000000110'::uuid,
    '20000000-0000-0000-0000-000000000210'::uuid,
    '10000000-0000-0000-0000-000000000101'::uuid,
    '工业质量与产线日报文件包交付',
    'manufacturing',
    'data_product',
    '按周/月交付低敏质量汇总文件包的演示样例。',
    'listed',
    'one_time',
    2399.00,
    'CNY',
    'file_download',
    ARRAY['analytics', 'report_delivery'],
    '工业质量 产线日报 文件包 FILE_STD FILE_SUB',
    '{"seed":"searchrec014","standard_scenario_code":"S2","scenario_name":"工业质量与产线日报文件包交付","subtitle":"首页固定推荐样例 S2","industry":"industrial_manufacturing","primary_sku":"FILE_STD","supplementary_skus":["FILE_SUB"],"review_status":"approved","visibility_status":"visible","visible_to_search":true,"recommended_placement_code":"home_featured"}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000311'::uuid,
    '20000000-0000-0000-0000-000000000111'::uuid,
    '20000000-0000-0000-0000-000000000211'::uuid,
    '10000000-0000-0000-0000-000000000101'::uuid,
    '供应链协同查询沙箱',
    'supply_chain',
    'service',
    '用于查询履约、库存周转与补货建议的沙箱演示样例。',
    'listed',
    'one_time',
    6999.00,
    'CNY',
    'sandbox',
    ARRAY['sandbox_execution', 'query_execution'],
    '供应链 协同 查询 沙箱 SBX_STD SHARE_RO',
    '{"seed":"searchrec014","standard_scenario_code":"S3","scenario_name":"供应链协同查询沙箱","subtitle":"首页固定推荐样例 S3","industry":"industrial_manufacturing","primary_sku":"SBX_STD","supplementary_skus":["SHARE_RO"],"review_status":"approved","visibility_status":"visible","visible_to_search":true,"recommended_placement_code":"home_featured"}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000312'::uuid,
    '20000000-0000-0000-0000-000000000112'::uuid,
    '20000000-0000-0000-0000-000000000212'::uuid,
    '10000000-0000-0000-0000-000000000104'::uuid,
    '零售门店经营分析 API / 报告订阅',
    'retail',
    'service',
    '门店客流、转化与销售结构 API / 报告订阅演示样例。',
    'listed',
    'subscription',
    4599.00,
    'CNY',
    'api_subscription',
    ARRAY['analytics', 'api_access', 'report_delivery'],
    '零售 门店 经营分析 API 报告订阅 API_SUB RPT_STD',
    '{"seed":"searchrec014","standard_scenario_code":"S4","scenario_name":"零售门店经营分析 API / 报告订阅","subtitle":"首页固定推荐样例 S4","industry":"retail","primary_sku":"API_SUB","supplementary_skus":["RPT_STD"],"review_status":"approved","visibility_status":"visible","visible_to_search":true,"recommended_placement_code":"home_featured"}'::jsonb
  ),
  (
    '20000000-0000-0000-0000-000000000313'::uuid,
    '20000000-0000-0000-0000-000000000113'::uuid,
    '20000000-0000-0000-0000-000000000213'::uuid,
    '10000000-0000-0000-0000-000000000104'::uuid,
    '商圈/门店选址查询服务',
    'retail',
    'service',
    '按次查询候选区域画像与评分的选址服务演示样例。',
    'listed',
    'one_time',
    1699.00,
    'CNY',
    'query_template',
    ARRAY['query_execution', 'report_delivery'],
    '商圈 门店 选址 查询 服务 QRY_LITE RPT_STD',
    '{"seed":"searchrec014","standard_scenario_code":"S5","scenario_name":"商圈/门店选址查询服务","subtitle":"首页固定推荐样例 S5","industry":"retail","primary_sku":"QRY_LITE","supplementary_skus":["RPT_STD"],"review_status":"approved","visibility_status":"visible","visible_to_search":true,"recommended_placement_code":"home_featured"}'::jsonb
  )
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
  sku_id,
  product_id,
  sku_code,
  sku_type,
  unit_name,
  billing_mode,
  acceptance_mode,
  refund_mode,
  status,
  trade_mode,
  delivery_object_kind,
  subscription_cadence,
  share_protocol,
  result_form,
  metadata
) VALUES
  ('20000000-0000-0000-0000-000000000409'::uuid, '20000000-0000-0000-0000-000000000309'::uuid, 'API_SUB', 'api_subscription', 'request', 'subscription', 'auto', 'partial', 'active', 'api_subscription', 'api', 'monthly', 'https', 'api', '{"seed":"searchrec014","standard_scenario_code":"S1","role":"primary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000410'::uuid, '20000000-0000-0000-0000-000000000309'::uuid, 'API_PPU', 'api_ppu', 'request', 'metered', 'auto', 'partial', 'active', 'api_ppu', 'api', NULL, 'https', 'api', '{"seed":"searchrec014","standard_scenario_code":"S1","role":"supplementary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000411'::uuid, '20000000-0000-0000-0000-000000000310'::uuid, 'FILE_STD', 'file', 'dataset', 'one_time', 'manual', 'full', 'active', 'snapshot_sale', 'file', NULL, NULL, 'file', '{"seed":"searchrec014","standard_scenario_code":"S2","role":"primary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000412'::uuid, '20000000-0000-0000-0000-000000000310'::uuid, 'FILE_SUB', 'file_subscription', 'dataset', 'subscription', 'manual', 'partial', 'active', 'subscription_sale', 'file', 'monthly', NULL, 'file', '{"seed":"searchrec014","standard_scenario_code":"S2","role":"supplementary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000413'::uuid, '20000000-0000-0000-0000-000000000311'::uuid, 'SBX_STD', 'sandbox', 'workspace', 'one_time', 'manual', 'none', 'active', 'sandbox_access', 'sandbox', NULL, NULL, 'sandbox_output', '{"seed":"searchrec014","standard_scenario_code":"S3","role":"primary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000414'::uuid, '20000000-0000-0000-0000-000000000311'::uuid, 'SHARE_RO', 'share', 'dataset', 'one_time', 'auto', 'none', 'active', 'share_grant', 'table', NULL, 'db_link', 'share', '{"seed":"searchrec014","standard_scenario_code":"S3","role":"supplementary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000415'::uuid, '20000000-0000-0000-0000-000000000312'::uuid, 'API_SUB', 'api_subscription', 'request', 'subscription', 'auto', 'partial', 'active', 'api_subscription', 'api', 'monthly', 'https', 'api', '{"seed":"searchrec014","standard_scenario_code":"S4","role":"primary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000416'::uuid, '20000000-0000-0000-0000-000000000312'::uuid, 'RPT_STD', 'report', 'report', 'one_time', 'manual', 'full', 'active', 'report_delivery', 'report', NULL, NULL, 'report', '{"seed":"searchrec014","standard_scenario_code":"S4","role":"supplementary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000417'::uuid, '20000000-0000-0000-0000-000000000313'::uuid, 'QRY_LITE', 'query_template', 'query_run', 'one_time', 'manual', 'none', 'active', 'template_query', 'query_template', NULL, NULL, 'query_result', '{"seed":"searchrec014","standard_scenario_code":"S5","role":"primary"}'::jsonb),
  ('20000000-0000-0000-0000-000000000418'::uuid, '20000000-0000-0000-0000-000000000313'::uuid, 'RPT_STD', 'report', 'report', 'one_time', 'manual', 'full', 'active', 'report_delivery', 'report', NULL, NULL, 'report', '{"seed":"searchrec014","standard_scenario_code":"S5","role":"supplementary"}'::jsonb)
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

INSERT INTO contract.template_binding (
  template_binding_id,
  sku_id,
  template_id,
  binding_type,
  status
) VALUES
  ('20000000-0000-0000-0000-000000000613'::uuid, '20000000-0000-0000-0000-000000000409'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000614'::uuid, '20000000-0000-0000-0000-000000000410'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000615'::uuid, '20000000-0000-0000-0000-000000000411'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000616'::uuid, '20000000-0000-0000-0000-000000000412'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000617'::uuid, '20000000-0000-0000-0000-000000000413'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000618'::uuid, '20000000-0000-0000-0000-000000000414'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000619'::uuid, '20000000-0000-0000-0000-000000000415'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000620'::uuid, '20000000-0000-0000-0000-000000000416'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000621'::uuid, '20000000-0000-0000-0000-000000000417'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000622'::uuid, '20000000-0000-0000-0000-000000000418'::uuid, '20000000-0000-0000-0000-000000000501'::uuid, 'contract', 'active'),
  ('20000000-0000-0000-0000-000000000623'::uuid, '20000000-0000-0000-0000-000000000409'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000624'::uuid, '20000000-0000-0000-0000-000000000410'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000625'::uuid, '20000000-0000-0000-0000-000000000411'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000626'::uuid, '20000000-0000-0000-0000-000000000412'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000627'::uuid, '20000000-0000-0000-0000-000000000413'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000628'::uuid, '20000000-0000-0000-0000-000000000414'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000629'::uuid, '20000000-0000-0000-0000-000000000415'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000630'::uuid, '20000000-0000-0000-0000-000000000416'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000631'::uuid, '20000000-0000-0000-0000-000000000417'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000632'::uuid, '20000000-0000-0000-0000-000000000418'::uuid, '20000000-0000-0000-0000-000000000502'::uuid, 'acceptance', 'active'),
  ('20000000-0000-0000-0000-000000000633'::uuid, '20000000-0000-0000-0000-000000000409'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000634'::uuid, '20000000-0000-0000-0000-000000000410'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000635'::uuid, '20000000-0000-0000-0000-000000000411'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000636'::uuid, '20000000-0000-0000-0000-000000000412'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000637'::uuid, '20000000-0000-0000-0000-000000000413'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000638'::uuid, '20000000-0000-0000-0000-000000000414'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000639'::uuid, '20000000-0000-0000-0000-000000000415'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000640'::uuid, '20000000-0000-0000-0000-000000000416'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000641'::uuid, '20000000-0000-0000-0000-000000000417'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active'),
  ('20000000-0000-0000-0000-000000000642'::uuid, '20000000-0000-0000-0000-000000000418'::uuid, '20000000-0000-0000-0000-000000000503'::uuid, 'refund', 'active')
ON CONFLICT (template_binding_id) DO UPDATE
SET
  sku_id = EXCLUDED.sku_id,
  template_id = EXCLUDED.template_id,
  binding_type = EXCLUDED.binding_type,
  status = EXCLUDED.status;

INSERT INTO search.search_signal_aggregate (
  entity_scope,
  entity_id,
  exposure_count,
  click_count,
  order_count,
  hotness_score,
  updated_at
) VALUES
  ('product', '20000000-0000-0000-0000-000000000309'::uuid, 180, 36, 12, 0.9800, NOW()),
  ('product', '20000000-0000-0000-0000-000000000310'::uuid, 160, 28, 10, 0.9500, NOW()),
  ('product', '20000000-0000-0000-0000-000000000311'::uuid, 140, 24, 9, 0.9300, NOW()),
  ('product', '20000000-0000-0000-0000-000000000312'::uuid, 120, 22, 8, 0.9100, NOW()),
  ('product', '20000000-0000-0000-0000-000000000313'::uuid, 100, 18, 7, 0.8900, NOW())
ON CONFLICT (entity_scope, entity_id) DO UPDATE
SET
  exposure_count = EXCLUDED.exposure_count,
  click_count = EXCLUDED.click_count,
  order_count = EXCLUDED.order_count,
  hotness_score = EXCLUDED.hotness_score,
  updated_at = NOW();

SELECT search.refresh_product_search_document_by_id('20000000-0000-0000-0000-000000000309'::uuid);
SELECT search.refresh_product_search_document_by_id('20000000-0000-0000-0000-000000000310'::uuid);
SELECT search.refresh_product_search_document_by_id('20000000-0000-0000-0000-000000000311'::uuid);
SELECT search.refresh_product_search_document_by_id('20000000-0000-0000-0000-000000000312'::uuid);
SELECT search.refresh_product_search_document_by_id('20000000-0000-0000-0000-000000000313'::uuid);
SELECT search.refresh_seller_search_document_by_id('10000000-0000-0000-0000-000000000101'::uuid);
SELECT search.refresh_seller_search_document_by_id('10000000-0000-0000-0000-000000000104'::uuid);

UPDATE developer.test_application
SET
  metadata = metadata
    || jsonb_build_object(
      'primary_product_id',
      CASE metadata ->> 'scenario_code'
        WHEN 'S1' THEN '20000000-0000-0000-0000-000000000309'
        WHEN 'S2' THEN '20000000-0000-0000-0000-000000000310'
        WHEN 'S3' THEN '20000000-0000-0000-0000-000000000311'
        WHEN 'S4' THEN '20000000-0000-0000-0000-000000000312'
        WHEN 'S5' THEN '20000000-0000-0000-0000-000000000313'
      END,
      'recommended_placement_code',
      'home_featured'
    )
    || CASE metadata ->> 'scenario_code'
      WHEN 'S1' THEN jsonb_build_object('supplementary_product_ids', jsonb_build_array('20000000-0000-0000-0000-000000000309'))
      WHEN 'S2' THEN jsonb_build_object('supplementary_product_ids', jsonb_build_array('20000000-0000-0000-0000-000000000310'))
      WHEN 'S3' THEN jsonb_build_object('supplementary_product_ids', jsonb_build_array('20000000-0000-0000-0000-000000000311'))
      WHEN 'S4' THEN jsonb_build_object('supplementary_product_ids', jsonb_build_array('20000000-0000-0000-0000-000000000312'))
      WHEN 'S5' THEN jsonb_build_object('supplementary_product_ids', jsonb_build_array('20000000-0000-0000-0000-000000000313'))
      ELSE '{}'::jsonb
    END,
  updated_at = NOW()
WHERE metadata ->> 'seed' = 'db035';

UPDATE recommend.placement_definition
SET
  metadata = COALESCE(metadata, '{}'::jsonb)
    || jsonb_build_object(
      'fixed_sample_set',
      'five_standard_scenarios_v1',
      'fixed_sample_seed',
      'searchrec014',
      'fixed_samples',
      jsonb_build_array(
        jsonb_build_object(
          'sample_order', 0,
          'scenario_code', 'S1',
          'scenario_name', '工业设备运行指标 API 订阅',
          'entity_scope', 'product',
          'entity_id', '20000000-0000-0000-0000-000000000309'
        ),
        jsonb_build_object(
          'sample_order', 1,
          'scenario_code', 'S2',
          'scenario_name', '工业质量与产线日报文件包交付',
          'entity_scope', 'product',
          'entity_id', '20000000-0000-0000-0000-000000000310'
        ),
        jsonb_build_object(
          'sample_order', 2,
          'scenario_code', 'S3',
          'scenario_name', '供应链协同查询沙箱',
          'entity_scope', 'product',
          'entity_id', '20000000-0000-0000-0000-000000000311'
        ),
        jsonb_build_object(
          'sample_order', 3,
          'scenario_code', 'S4',
          'scenario_name', '零售门店经营分析 API / 报告订阅',
          'entity_scope', 'product',
          'entity_id', '20000000-0000-0000-0000-000000000312'
        ),
        jsonb_build_object(
          'sample_order', 4,
          'scenario_code', 'S5',
          'scenario_name', '商圈/门店选址查询服务',
          'entity_scope', 'product',
          'entity_id', '20000000-0000-0000-0000-000000000313'
        )
      )
    ),
  updated_at = NOW()
WHERE placement_code = 'home_featured';
