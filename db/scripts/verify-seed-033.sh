#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-55432}"
DB_NAME="${DB_NAME:-luna_data_trading}"
DB_USER="${DB_USER:-luna}"
DB_PASSWORD="${DB_PASSWORD:-5686}"

export PGPASSWORD="$DB_PASSWORD"
PSQL=(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -X -q -tA)

check_count_ge() {
  local sql="$1"
  local min="$2"
  local label="$3"
  local count
  count="$("${PSQL[@]}" -c "$sql")"
  if [[ "$count" -lt "$min" ]]; then
    echo "[fail] ${label} count too small: ${count} (< ${min})" >&2
    exit 1
  fi
}

check_exists() {
  local sql="$1"
  local label="$2"
  local exists
  exists="$("${PSQL[@]}" -c "$sql")"
  if [[ "$exists" -lt 1 ]]; then
    echo "[fail] missing expected recommendation sample: ${label}" >&2
    exit 1
  fi
}

check_exists "SELECT COUNT(*) FROM core.organization WHERE org_id = '10000000-0000-0000-0000-000000000104';" "retail seller org"
check_count_ge "SELECT COUNT(*) FROM catalog.product WHERE metadata->>'seed'='searchrec014' AND metadata->>'recommended_placement_code'='home_featured';" 5 "SEARCHREC-014 scenario products"
check_count_ge "SELECT COUNT(*) FROM catalog.product_sku WHERE metadata->>'seed'='searchrec014';" 10 "SEARCHREC-014 scenario skus"
check_count_ge "SELECT COUNT(*) FROM catalog.product_sku WHERE metadata->>'seed'='searchrec014' AND sku_code = sku_type;" 10 "SEARCHREC-014 standard sku_type truth"
check_count_ge "SELECT COUNT(*) FROM contract.template_binding WHERE template_binding_id::text BETWEEN '20000000-0000-0000-0000-000000000613' AND '20000000-0000-0000-0000-000000000642';" 30 "SEARCHREC-014 template bindings"

check_exists "SELECT COUNT(*) FROM catalog.product WHERE title='工业设备运行指标 API 订阅' AND metadata->>'standard_scenario_code'='S1';" "S1 product"
check_exists "SELECT COUNT(*) FROM catalog.product WHERE title='工业质量与产线日报文件包交付' AND metadata->>'standard_scenario_code'='S2';" "S2 product"
check_exists "SELECT COUNT(*) FROM catalog.product WHERE title='供应链协同查询沙箱' AND metadata->>'standard_scenario_code'='S3';" "S3 product"
check_exists "SELECT COUNT(*) FROM catalog.product WHERE title='零售门店经营分析 API / 报告订阅' AND metadata->>'standard_scenario_code'='S4';" "S4 product"
check_exists "SELECT COUNT(*) FROM catalog.product WHERE title='商圈/门店选址查询服务' AND metadata->>'standard_scenario_code'='S5';" "S5 product"

check_exists "SELECT COUNT(*) FROM recommend.placement_definition WHERE placement_code='home_featured' AND metadata->>'fixed_sample_set'='five_standard_scenarios_v1' AND jsonb_array_length(metadata->'fixed_samples') = 5;" "home_featured fixed samples"
check_count_ge "SELECT COUNT(*) FROM developer.test_application WHERE metadata->>'seed'='db035' AND metadata->>'recommended_placement_code'='home_featured' AND metadata ? 'primary_product_id';" 5 "scenario registry fixed sample links"
check_count_ge "SELECT COUNT(*) FROM search.product_search_document WHERE product_id IN ('20000000-0000-0000-0000-000000000309','20000000-0000-0000-0000-000000000310','20000000-0000-0000-0000-000000000311','20000000-0000-0000-0000-000000000312','20000000-0000-0000-0000-000000000313');" 5 "scenario search projections"

echo "[ok] seed 033 recommendation homepage samples verified"
