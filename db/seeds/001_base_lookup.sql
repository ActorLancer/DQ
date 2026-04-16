WITH seed_tags(tag_id, tag_name, tag_code, tag_type, tag_group, status, metadata) AS (
  VALUES
    ('00000000-0000-0000-0000-000000000101'::uuid, '金融数据', 'category_finance', 'lookup', 'product_category', 'active', '{"label":"产品类目","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000102'::uuid, '企业征信', 'category_credit', 'lookup', 'product_category', 'active', '{"label":"产品类目","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000103'::uuid, '供应链数据', 'category_supply_chain', 'lookup', 'product_category', 'active', '{"label":"产品类目","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000104'::uuid, '营销数据', 'category_marketing', 'lookup', 'product_category', 'active', '{"label":"产品类目","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000201'::uuid, '制造业', 'industry_manufacturing', 'lookup', 'industry', 'active', '{"label":"行业标签","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000202'::uuid, '金融业', 'industry_finance', 'lookup', 'industry', 'active', '{"label":"行业标签","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000203'::uuid, '零售电商', 'industry_retail', 'lookup', 'industry', 'active', '{"label":"行业标签","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000204'::uuid, '交通物流', 'industry_transport', 'lookup', 'industry', 'active', '{"label":"行业标签","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000301'::uuid, '低风险', 'risk_low', 'lookup', 'risk_level', 'active', '{"risk_level":1,"label":"风险等级","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000302'::uuid, '中风险', 'risk_medium', 'lookup', 'risk_level', 'active', '{"risk_level":2,"label":"风险等级","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000303'::uuid, '高风险', 'risk_high', 'lookup', 'risk_level', 'active', '{"risk_level":3,"label":"风险等级","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000304'::uuid, '阻断风险', 'risk_blocked', 'lookup', 'risk_level', 'active', '{"risk_level":4,"label":"风险等级","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000401'::uuid, '文件标准交付', 'delivery_file_standard', 'lookup', 'delivery_mode', 'active', '{"sku":"FILE_STD","label":"交付模式","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000402'::uuid, '文件订阅交付', 'delivery_file_subscription', 'lookup', 'delivery_mode', 'active', '{"sku":"FILE_SUB","label":"交付模式","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000403'::uuid, '数据共享交付', 'delivery_share_grant', 'lookup', 'delivery_mode', 'active', '{"sku":"SHARE_RO","label":"交付模式","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000404'::uuid, 'API 订阅交付', 'delivery_api_subscription', 'lookup', 'delivery_mode', 'active', '{"sku":"API_SUB","label":"交付模式","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000405'::uuid, 'API 按量计费交付', 'delivery_api_ppu', 'lookup', 'delivery_mode', 'active', '{"sku":"API_PPU","label":"交付模式","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000406'::uuid, '模板查询交付', 'delivery_query_template', 'lookup', 'delivery_mode', 'active', '{"sku":"QRY_LITE","label":"交付模式","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000407'::uuid, '沙箱交付', 'delivery_sandbox', 'lookup', 'delivery_mode', 'active', '{"sku":"SBX_STD","label":"交付模式","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000408'::uuid, '报告交付', 'delivery_report', 'lookup', 'delivery_mode', 'active', '{"sku":"RPT_STD","label":"交付模式","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000501'::uuid, '草稿', 'status_draft', 'lookup', 'status_dictionary', 'active', '{"scope":"workflow","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000502'::uuid, '待审核', 'status_pending_review', 'lookup', 'status_dictionary', 'active', '{"scope":"workflow","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000503'::uuid, '已发布', 'status_published', 'lookup', 'status_dictionary', 'active', '{"scope":"workflow","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000504'::uuid, '已冻结', 'status_frozen', 'lookup', 'status_dictionary', 'active', '{"scope":"workflow","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000601'::uuid, '一次性购买', 'trade_mode_one_time', 'lookup', 'trade_mode', 'active', '{"sku":"FILE_STD","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000602'::uuid, '订阅购买', 'trade_mode_subscription', 'lookup', 'trade_mode', 'active', '{"sku":"FILE_SUB","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000603'::uuid, '授权共享', 'trade_mode_share', 'lookup', 'trade_mode', 'active', '{"sku":"SHARE_RO","stage":"V1"}'::jsonb),
    ('00000000-0000-0000-0000-000000000604'::uuid, '按调用计费', 'trade_mode_ppu', 'lookup', 'trade_mode', 'active', '{"sku":"API_PPU","stage":"V1"}'::jsonb)
)
INSERT INTO catalog.tag (tag_id, tag_name, tag_code, tag_type, tag_group, status, metadata)
SELECT tag_id, tag_name, tag_code, tag_type, tag_group, status, metadata
FROM seed_tags
ON CONFLICT (tag_code) DO UPDATE
SET
  tag_name = EXCLUDED.tag_name,
  tag_type = EXCLUDED.tag_type,
  tag_group = EXCLUDED.tag_group,
  status = EXCLUDED.status,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();
