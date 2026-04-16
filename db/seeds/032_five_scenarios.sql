INSERT INTO developer.test_application (
  test_application_id,
  app_id,
  owner_user_id,
  scenario_name,
  status,
  metadata
) VALUES
  (
    '32000000-0000-0000-0000-000000000001'::uuid,
    '10000000-0000-0000-0000-000000000401'::uuid,
    '10000000-0000-0000-0000-000000000305'::uuid,
    '工业设备运行指标 API 订阅',
    'active',
    jsonb_build_object(
      'seed', 'db035',
      'scenario_code', 'S1',
      'official_name', '工业设备运行指标 API 订阅',
      'primary_sku', 'API_SUB',
      'supplementary_skus', jsonb_build_array('API_PPU'),
      'contract_template_id', '20000000-0000-0000-0000-000000000501',
      'acceptance_template_id', '20000000-0000-0000-0000-000000000502',
      'refund_template_id', '20000000-0000-0000-0000-000000000503',
      'primary_sku_id', '20000000-0000-0000-0000-000000000404',
      'supplementary_sku_ids', jsonb_build_array('20000000-0000-0000-0000-000000000405'),
      'scenario_order_id', '30000000-0000-0000-0000-000000000111'
    )
  ),
  (
    '32000000-0000-0000-0000-000000000002'::uuid,
    '10000000-0000-0000-0000-000000000401'::uuid,
    '10000000-0000-0000-0000-000000000305'::uuid,
    '工业质量与产线日报文件包交付',
    'active',
    jsonb_build_object(
      'seed', 'db035',
      'scenario_code', 'S2',
      'official_name', '工业质量与产线日报文件包交付',
      'primary_sku', 'FILE_STD',
      'supplementary_skus', jsonb_build_array('FILE_SUB'),
      'contract_template_id', '20000000-0000-0000-0000-000000000501',
      'acceptance_template_id', '20000000-0000-0000-0000-000000000502',
      'refund_template_id', '20000000-0000-0000-0000-000000000503',
      'primary_sku_id', '20000000-0000-0000-0000-000000000401',
      'supplementary_sku_ids', jsonb_build_array('20000000-0000-0000-0000-000000000402'),
      'scenario_order_id', '30000000-0000-0000-0000-000000000109'
    )
  ),
  (
    '32000000-0000-0000-0000-000000000003'::uuid,
    '10000000-0000-0000-0000-000000000401'::uuid,
    '10000000-0000-0000-0000-000000000305'::uuid,
    '供应链协同查询沙箱',
    'active',
    jsonb_build_object(
      'seed', 'db035',
      'scenario_code', 'S3',
      'official_name', '供应链协同查询沙箱',
      'primary_sku', 'SBX_STD',
      'supplementary_skus', jsonb_build_array('SHARE_RO'),
      'contract_template_id', '20000000-0000-0000-0000-000000000501',
      'acceptance_template_id', '20000000-0000-0000-0000-000000000502',
      'refund_template_id', '20000000-0000-0000-0000-000000000503',
      'primary_sku_id', '20000000-0000-0000-0000-000000000407',
      'supplementary_sku_ids', jsonb_build_array('20000000-0000-0000-0000-000000000403'),
      'scenario_order_id', '30000000-0000-0000-0000-000000000113'
    )
  ),
  (
    '32000000-0000-0000-0000-000000000004'::uuid,
    '10000000-0000-0000-0000-000000000401'::uuid,
    '10000000-0000-0000-0000-000000000305'::uuid,
    '零售门店经营分析 API / 报告订阅',
    'active',
    jsonb_build_object(
      'seed', 'db035',
      'scenario_code', 'S4',
      'official_name', '零售门店经营分析 API / 报告订阅',
      'primary_sku', 'API_SUB',
      'supplementary_skus', jsonb_build_array('RPT_STD'),
      'contract_template_id', '20000000-0000-0000-0000-000000000501',
      'acceptance_template_id', '20000000-0000-0000-0000-000000000502',
      'refund_template_id', '20000000-0000-0000-0000-000000000503',
      'primary_sku_id', '20000000-0000-0000-0000-000000000404',
      'supplementary_sku_ids', jsonb_build_array('20000000-0000-0000-0000-000000000408'),
      'scenario_order_id', '30000000-0000-0000-0000-000000000111'
    )
  ),
  (
    '32000000-0000-0000-0000-000000000005'::uuid,
    '10000000-0000-0000-0000-000000000401'::uuid,
    '10000000-0000-0000-0000-000000000305'::uuid,
    '商圈/门店选址查询服务',
    'active',
    jsonb_build_object(
      'seed', 'db035',
      'scenario_code', 'S5',
      'official_name', '商圈/门店选址查询服务',
      'primary_sku', 'QRY_LITE',
      'supplementary_skus', jsonb_build_array('RPT_STD'),
      'contract_template_id', '20000000-0000-0000-0000-000000000501',
      'acceptance_template_id', '20000000-0000-0000-0000-000000000502',
      'refund_template_id', '20000000-0000-0000-0000-000000000503',
      'primary_sku_id', '20000000-0000-0000-0000-000000000406',
      'supplementary_sku_ids', jsonb_build_array('20000000-0000-0000-0000-000000000408'),
      'scenario_order_id', '30000000-0000-0000-0000-000000000112'
    )
  )
ON CONFLICT (test_application_id) DO UPDATE
SET
  app_id = EXCLUDED.app_id,
  owner_user_id = EXCLUDED.owner_user_id,
  scenario_name = EXCLUDED.scenario_name,
  status = EXCLUDED.status,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();
