INSERT INTO trade.inquiry (
  inquiry_id, buyer_org_id, product_id, status, message_text, created_by
) VALUES
  ('30000000-0000-0000-0000-000000000001'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '20000000-0000-0000-0000-000000000301'::uuid, 'open', 'Need FILE_STD sample', '10000000-0000-0000-0000-000000000302'::uuid),
  ('30000000-0000-0000-0000-000000000002'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '20000000-0000-0000-0000-000000000304'::uuid, 'open', 'Need API access', '10000000-0000-0000-0000-000000000305'::uuid),
  ('30000000-0000-0000-0000-000000000003'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '20000000-0000-0000-0000-000000000307'::uuid, 'open', 'Need sandbox run', '10000000-0000-0000-0000-000000000306'::uuid)
ON CONFLICT (inquiry_id) DO UPDATE
SET
  buyer_org_id = EXCLUDED.buyer_org_id,
  product_id = EXCLUDED.product_id,
  status = EXCLUDED.status,
  message_text = EXCLUDED.message_text,
  created_by = EXCLUDED.created_by,
  updated_at = NOW();

INSERT INTO trade.order_main (
  order_id, inquiry_id, product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id, status, payment_status, payment_mode, amount, currency_code,
  fee_preview_snapshot, payment_channel_snapshot, buyer_deposit_amount, seller_deposit_amount, price_snapshot_json, trust_boundary_snapshot,
  storage_mode_snapshot, delivery_route_snapshot, platform_plaintext_access_snapshot, idempotency_key, created_at, delivered_at, accepted_at, settled_at
) VALUES
  ('30000000-0000-0000-0000-000000000101'::uuid, '30000000-0000-0000-0000-000000000001'::uuid, '20000000-0000-0000-0000-000000000301'::uuid, '20000000-0000-0000-0000-000000000201'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000401'::uuid, 'accepted', 'paid', 'online', 1999.00, 'CNY', '{"seed":"db029","sku":"FILE_STD"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":1999}'::jsonb, '{}'::jsonb, 'platform_custody', 'ticket', false, 'seed-db029-file-std-001', NOW() - INTERVAL '10 days', NOW() - INTERVAL '9 days', NOW() - INTERVAL '8 days', NOW() - INTERVAL '8 days'),
  ('30000000-0000-0000-0000-000000000102'::uuid, NULL, '20000000-0000-0000-0000-000000000302'::uuid, '20000000-0000-0000-0000-000000000202'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000402'::uuid, 'delivered', 'paid', 'online', 3999.00, 'CNY', '{"seed":"db029","sku":"FILE_SUB"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":3999}'::jsonb, '{}'::jsonb, 'platform_custody', 'subscription_ticket', false, 'seed-db029-file-sub-001', NOW() - INTERVAL '7 days', NOW() - INTERVAL '6 days', NULL, NULL),
  ('30000000-0000-0000-0000-000000000103'::uuid, NULL, '20000000-0000-0000-0000-000000000303'::uuid, '20000000-0000-0000-0000-000000000203'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000403'::uuid, 'delivered', 'paid', 'online', 2999.00, 'CNY', '{"seed":"db029","sku":"SHARE_RO"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":2999}'::jsonb, '{}'::jsonb, 'platform_custody', 'share_grant', false, 'seed-db029-share-ro-001', NOW() - INTERVAL '7 days', NOW() - INTERVAL '6 days', NULL, NULL),
  ('30000000-0000-0000-0000-000000000104'::uuid, '30000000-0000-0000-0000-000000000002'::uuid, '20000000-0000-0000-0000-000000000304'::uuid, '20000000-0000-0000-0000-000000000204'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000404'::uuid, 'active', 'paid', 'online', 4999.00, 'CNY', '{"seed":"db029","sku":"API_SUB"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":4999}'::jsonb, '{}'::jsonb, 'platform_custody', 'api_key', false, 'seed-db029-api-sub-001', NOW() - INTERVAL '5 days', NOW() - INTERVAL '4 days', NULL, NULL),
  ('30000000-0000-0000-0000-000000000105'::uuid, NULL, '20000000-0000-0000-0000-000000000305'::uuid, '20000000-0000-0000-0000-000000000205'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000405'::uuid, 'active', 'paid', 'online', 200.00, 'CNY', '{"seed":"db029","sku":"API_PPU"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"unit_price":0.8}'::jsonb, '{}'::jsonb, 'platform_custody', 'api_key', false, 'seed-db029-api-ppu-001', NOW() - INTERVAL '4 days', NOW() - INTERVAL '3 days', NULL, NULL),
  ('30000000-0000-0000-0000-000000000106'::uuid, NULL, '20000000-0000-0000-0000-000000000306'::uuid, '20000000-0000-0000-0000-000000000206'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000406'::uuid, 'delivered', 'paid', 'online', 2599.00, 'CNY', '{"seed":"db029","sku":"QRY_LITE"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":2599}'::jsonb, '{}'::jsonb, 'platform_custody', 'query_grant', false, 'seed-db029-qry-lite-001', NOW() - INTERVAL '4 days', NOW() - INTERVAL '3 days', NULL, NULL),
  ('30000000-0000-0000-0000-000000000107'::uuid, '30000000-0000-0000-0000-000000000003'::uuid, '20000000-0000-0000-0000-000000000307'::uuid, '20000000-0000-0000-0000-000000000207'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000407'::uuid, 'active', 'paid', 'online', 6999.00, 'CNY', '{"seed":"db029","sku":"SBX_STD"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":6999}'::jsonb, '{}'::jsonb, 'platform_custody', 'sandbox', false, 'seed-db029-sbx-std-001', NOW() - INTERVAL '3 days', NOW() - INTERVAL '2 days', NULL, NULL),
  ('30000000-0000-0000-0000-000000000108'::uuid, NULL, '20000000-0000-0000-0000-000000000308'::uuid, '20000000-0000-0000-0000-000000000208'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000408'::uuid, 'delivered', 'paid', 'online', 3299.00, 'CNY', '{"seed":"db029","sku":"RPT_STD"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":3299}'::jsonb, '{}'::jsonb, 'platform_custody', 'report_download', false, 'seed-db029-rpt-std-001', NOW() - INTERVAL '3 days', NOW() - INTERVAL '2 days', NULL, NULL),
  ('30000000-0000-0000-0000-000000000109'::uuid, NULL, '20000000-0000-0000-0000-000000000301'::uuid, '20000000-0000-0000-0000-000000000201'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000401'::uuid, 'delivered', 'paid', 'online', 1999.00, 'CNY', '{"seed":"db029","scenario":"SCN-01"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":1999}'::jsonb, '{}'::jsonb, 'platform_custody', 'ticket', false, 'seed-db029-scn-01', NOW() - INTERVAL '2 days', NOW() - INTERVAL '1 day', NULL, NULL),
  ('30000000-0000-0000-0000-000000000110'::uuid, NULL, '20000000-0000-0000-0000-000000000303'::uuid, '20000000-0000-0000-0000-000000000203'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000403'::uuid, 'delivered', 'paid', 'online', 2999.00, 'CNY', '{"seed":"db029","scenario":"SCN-02"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":2999}'::jsonb, '{}'::jsonb, 'platform_custody', 'share_grant', false, 'seed-db029-scn-02', NOW() - INTERVAL '2 days', NOW() - INTERVAL '1 day', NULL, NULL),
  ('30000000-0000-0000-0000-000000000111'::uuid, NULL, '20000000-0000-0000-0000-000000000304'::uuid, '20000000-0000-0000-0000-000000000204'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000404'::uuid, 'active', 'paid', 'online', 4999.00, 'CNY', '{"seed":"db029","scenario":"SCN-03"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":4999}'::jsonb, '{}'::jsonb, 'platform_custody', 'api_key', false, 'seed-db029-scn-03', NOW() - INTERVAL '36 hours', NOW() - INTERVAL '30 hours', NULL, NULL),
  ('30000000-0000-0000-0000-000000000112'::uuid, NULL, '20000000-0000-0000-0000-000000000306'::uuid, '20000000-0000-0000-0000-000000000206'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000406'::uuid, 'delivered', 'paid', 'online', 2599.00, 'CNY', '{"seed":"db029","scenario":"SCN-04"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":2599}'::jsonb, '{}'::jsonb, 'platform_custody', 'query_grant', false, 'seed-db029-scn-04', NOW() - INTERVAL '30 hours', NOW() - INTERVAL '24 hours', NULL, NULL),
  ('30000000-0000-0000-0000-000000000113'::uuid, NULL, '20000000-0000-0000-0000-000000000307'::uuid, '20000000-0000-0000-0000-000000000207'::uuid, '10000000-0000-0000-0000-000000000102'::uuid, '10000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000407'::uuid, 'active', 'paid', 'online', 6999.00, 'CNY', '{"seed":"db029","scenario":"SCN-05"}'::jsonb, '{"channel":"mockpay"}'::jsonb, 0, 0, '{"price":6999}'::jsonb, '{}'::jsonb, 'platform_custody', 'sandbox', false, 'seed-db029-scn-05', NOW() - INTERVAL '24 hours', NOW() - INTERVAL '20 hours', NULL, NULL)
ON CONFLICT (order_id) DO UPDATE
SET
  inquiry_id = EXCLUDED.inquiry_id,
  product_id = EXCLUDED.product_id,
  asset_version_id = EXCLUDED.asset_version_id,
  buyer_org_id = EXCLUDED.buyer_org_id,
  seller_org_id = EXCLUDED.seller_org_id,
  sku_id = EXCLUDED.sku_id,
  status = EXCLUDED.status,
  payment_status = EXCLUDED.payment_status,
  payment_mode = EXCLUDED.payment_mode,
  amount = EXCLUDED.amount,
  currency_code = EXCLUDED.currency_code,
  fee_preview_snapshot = EXCLUDED.fee_preview_snapshot,
  payment_channel_snapshot = EXCLUDED.payment_channel_snapshot,
  buyer_deposit_amount = EXCLUDED.buyer_deposit_amount,
  seller_deposit_amount = EXCLUDED.seller_deposit_amount,
  price_snapshot_json = EXCLUDED.price_snapshot_json,
  trust_boundary_snapshot = EXCLUDED.trust_boundary_snapshot,
  storage_mode_snapshot = EXCLUDED.storage_mode_snapshot,
  delivery_route_snapshot = EXCLUDED.delivery_route_snapshot,
  platform_plaintext_access_snapshot = EXCLUDED.platform_plaintext_access_snapshot,
  idempotency_key = EXCLUDED.idempotency_key,
  created_at = EXCLUDED.created_at,
  delivered_at = EXCLUDED.delivered_at,
  accepted_at = EXCLUDED.accepted_at,
  settled_at = EXCLUDED.settled_at,
  updated_at = NOW();

INSERT INTO trade.order_line (order_line_id, order_id, sku_id, quantity, unit_price, amount) VALUES
  ('30000000-0000-0000-0000-000000001101'::uuid, '30000000-0000-0000-0000-000000000101'::uuid, '20000000-0000-0000-0000-000000000401'::uuid, 1, 1999.00, 1999.00),
  ('30000000-0000-0000-0000-000000001102'::uuid, '30000000-0000-0000-0000-000000000102'::uuid, '20000000-0000-0000-0000-000000000402'::uuid, 1, 3999.00, 3999.00),
  ('30000000-0000-0000-0000-000000001103'::uuid, '30000000-0000-0000-0000-000000000103'::uuid, '20000000-0000-0000-0000-000000000403'::uuid, 1, 2999.00, 2999.00),
  ('30000000-0000-0000-0000-000000001104'::uuid, '30000000-0000-0000-0000-000000000104'::uuid, '20000000-0000-0000-0000-000000000404'::uuid, 1, 4999.00, 4999.00),
  ('30000000-0000-0000-0000-000000001105'::uuid, '30000000-0000-0000-0000-000000000105'::uuid, '20000000-0000-0000-0000-000000000405'::uuid, 250, 0.80, 200.00),
  ('30000000-0000-0000-0000-000000001106'::uuid, '30000000-0000-0000-0000-000000000106'::uuid, '20000000-0000-0000-0000-000000000406'::uuid, 1, 2599.00, 2599.00),
  ('30000000-0000-0000-0000-000000001107'::uuid, '30000000-0000-0000-0000-000000000107'::uuid, '20000000-0000-0000-0000-000000000407'::uuid, 1, 6999.00, 6999.00),
  ('30000000-0000-0000-0000-000000001108'::uuid, '30000000-0000-0000-0000-000000000108'::uuid, '20000000-0000-0000-0000-000000000408'::uuid, 1, 3299.00, 3299.00),
  ('30000000-0000-0000-0000-000000001109'::uuid, '30000000-0000-0000-0000-000000000109'::uuid, '20000000-0000-0000-0000-000000000401'::uuid, 1, 1999.00, 1999.00),
  ('30000000-0000-0000-0000-000000001110'::uuid, '30000000-0000-0000-0000-000000000110'::uuid, '20000000-0000-0000-0000-000000000403'::uuid, 1, 2999.00, 2999.00),
  ('30000000-0000-0000-0000-000000001111'::uuid, '30000000-0000-0000-0000-000000000111'::uuid, '20000000-0000-0000-0000-000000000404'::uuid, 1, 4999.00, 4999.00),
  ('30000000-0000-0000-0000-000000001112'::uuid, '30000000-0000-0000-0000-000000000112'::uuid, '20000000-0000-0000-0000-000000000406'::uuid, 1, 2599.00, 2599.00),
  ('30000000-0000-0000-0000-000000001113'::uuid, '30000000-0000-0000-0000-000000000113'::uuid, '20000000-0000-0000-0000-000000000407'::uuid, 1, 6999.00, 6999.00)
ON CONFLICT (order_line_id) DO UPDATE
SET
  order_id = EXCLUDED.order_id,
  sku_id = EXCLUDED.sku_id,
  quantity = EXCLUDED.quantity,
  unit_price = EXCLUDED.unit_price,
  amount = EXCLUDED.amount;

INSERT INTO trade.authorization_grant (
  authorization_grant_id, order_id, grant_type, granted_to_type, granted_to_id, policy_snapshot, valid_from, status
) VALUES
  ('30000000-0000-0000-0000-000000002101'::uuid, '30000000-0000-0000-0000-000000000101'::uuid, 'file_download', 'org', '10000000-0000-0000-0000-000000000102'::uuid, '{"seed":"db029"}'::jsonb, NOW() - INTERVAL '9 days', 'active'),
  ('30000000-0000-0000-0000-000000002102'::uuid, '30000000-0000-0000-0000-000000000102'::uuid, 'file_subscription', 'org', '10000000-0000-0000-0000-000000000102'::uuid, '{"seed":"db029"}'::jsonb, NOW() - INTERVAL '6 days', 'active'),
  ('30000000-0000-0000-0000-000000002103'::uuid, '30000000-0000-0000-0000-000000000103'::uuid, 'share_readonly', 'org', '10000000-0000-0000-0000-000000000102'::uuid, '{"seed":"db029"}'::jsonb, NOW() - INTERVAL '6 days', 'active'),
  ('30000000-0000-0000-0000-000000002104'::uuid, '30000000-0000-0000-0000-000000000104'::uuid, 'api_access', 'app', '10000000-0000-0000-0000-000000000401'::uuid, '{"seed":"db029"}'::jsonb, NOW() - INTERVAL '4 days', 'active'),
  ('30000000-0000-0000-0000-000000002105'::uuid, '30000000-0000-0000-0000-000000000105'::uuid, 'api_metered', 'app', '10000000-0000-0000-0000-000000000401'::uuid, '{"seed":"db029"}'::jsonb, NOW() - INTERVAL '3 days', 'active'),
  ('30000000-0000-0000-0000-000000002106'::uuid, '30000000-0000-0000-0000-000000000106'::uuid, 'query_template', 'org', '10000000-0000-0000-0000-000000000102'::uuid, '{"seed":"db029"}'::jsonb, NOW() - INTERVAL '3 days', 'active'),
  ('30000000-0000-0000-0000-000000002107'::uuid, '30000000-0000-0000-0000-000000000107'::uuid, 'sandbox_access', 'org', '10000000-0000-0000-0000-000000000102'::uuid, '{"seed":"db029"}'::jsonb, NOW() - INTERVAL '2 days', 'active'),
  ('30000000-0000-0000-0000-000000002108'::uuid, '30000000-0000-0000-0000-000000000108'::uuid, 'report_download', 'org', '10000000-0000-0000-0000-000000000102'::uuid, '{"seed":"db029"}'::jsonb, NOW() - INTERVAL '2 days', 'active')
ON CONFLICT (authorization_grant_id) DO UPDATE
SET
  order_id = EXCLUDED.order_id,
  grant_type = EXCLUDED.grant_type,
  granted_to_type = EXCLUDED.granted_to_type,
  granted_to_id = EXCLUDED.granted_to_id,
  policy_snapshot = EXCLUDED.policy_snapshot,
  valid_from = EXCLUDED.valid_from,
  status = EXCLUDED.status,
  updated_at = NOW();

INSERT INTO delivery.api_credential (
  api_credential_id, order_id, app_id, api_key_hash, upstream_mode, quota_json, status, valid_from
) VALUES
  ('30000000-0000-0000-0000-000000003001'::uuid, '30000000-0000-0000-0000-000000000104'::uuid, '10000000-0000-0000-0000-000000000401'::uuid, 'seed-api-sub-key-hash', 'platform_proxy', '{"qps":100,"monthly_calls":100000}'::jsonb, 'active', NOW() - INTERVAL '4 days'),
  ('30000000-0000-0000-0000-000000003002'::uuid, '30000000-0000-0000-0000-000000000105'::uuid, '10000000-0000-0000-0000-000000000401'::uuid, 'seed-api-ppu-key-hash', 'platform_proxy', '{"qps":50,"billing_mode":"ppu"}'::jsonb, 'active', NOW() - INTERVAL '3 days'),
  ('30000000-0000-0000-0000-000000003003'::uuid, '30000000-0000-0000-0000-000000000111'::uuid, '10000000-0000-0000-0000-000000000401'::uuid, 'seed-api-scn3-key-hash', 'platform_proxy', '{"qps":100,"monthly_calls":50000}'::jsonb, 'active', NOW() - INTERVAL '30 hours')
ON CONFLICT (api_credential_id) DO UPDATE
SET
  order_id = EXCLUDED.order_id,
  app_id = EXCLUDED.app_id,
  api_key_hash = EXCLUDED.api_key_hash,
  upstream_mode = EXCLUDED.upstream_mode,
  quota_json = EXCLUDED.quota_json,
  status = EXCLUDED.status,
  valid_from = EXCLUDED.valid_from,
  updated_at = NOW();

INSERT INTO delivery.sandbox_workspace (
  sandbox_workspace_id, order_id, workspace_name, status, data_residency_mode, export_policy, output_boundary_json
) VALUES
  ('30000000-0000-0000-0000-000000004001'::uuid, '30000000-0000-0000-0000-000000000107'::uuid, 'sbx-order-107', 'running', 'seller_self_hosted', '{"allow_export":false}'::jsonb, '{"masking":"strict"}'::jsonb),
  ('30000000-0000-0000-0000-000000004002'::uuid, '30000000-0000-0000-0000-000000000113'::uuid, 'sbx-order-113', 'running', 'seller_self_hosted', '{"allow_export":false}'::jsonb, '{"masking":"strict"}'::jsonb)
ON CONFLICT (sandbox_workspace_id) DO UPDATE
SET
  order_id = EXCLUDED.order_id,
  workspace_name = EXCLUDED.workspace_name,
  status = EXCLUDED.status,
  data_residency_mode = EXCLUDED.data_residency_mode,
  export_policy = EXCLUDED.export_policy,
  output_boundary_json = EXCLUDED.output_boundary_json,
  updated_at = NOW();

INSERT INTO delivery.report_artifact (
  report_artifact_id, order_id, report_type, version_no, status
) VALUES
  ('30000000-0000-0000-0000-000000005001'::uuid, '30000000-0000-0000-0000-000000000108'::uuid, 'pdf_report', 1, 'ready')
ON CONFLICT (report_artifact_id) DO UPDATE
SET
  order_id = EXCLUDED.order_id,
  report_type = EXCLUDED.report_type,
  version_no = EXCLUDED.version_no,
  status = EXCLUDED.status,
  updated_at = NOW();
