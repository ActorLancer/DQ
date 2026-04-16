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
    echo "[fail] missing expected scenario mapping: ${label}" >&2
    exit 1
  fi
}

check_count_ge "SELECT COUNT(*) FROM developer.test_application WHERE metadata->>'seed' = 'db035';" 5 "developer.test_application five scenarios"

check_exists "SELECT COUNT(*) FROM developer.test_application WHERE scenario_name='工业设备运行指标 API 订阅' AND metadata->>'primary_sku'='API_SUB';" "S1 API_SUB"
check_exists "SELECT COUNT(*) FROM developer.test_application WHERE scenario_name='工业质量与产线日报文件包交付' AND metadata->>'primary_sku'='FILE_STD';" "S2 FILE_STD"
check_exists "SELECT COUNT(*) FROM developer.test_application WHERE scenario_name='供应链协同查询沙箱' AND metadata->>'primary_sku'='SBX_STD';" "S3 SBX_STD"
check_exists "SELECT COUNT(*) FROM developer.test_application WHERE scenario_name='零售门店经营分析 API / 报告订阅' AND metadata->>'primary_sku'='API_SUB';" "S4 API_SUB"
check_exists "SELECT COUNT(*) FROM developer.test_application WHERE scenario_name='商圈/门店选址查询服务' AND metadata->>'primary_sku'='QRY_LITE';" "S5 QRY_LITE"

check_count_ge "SELECT COUNT(*) FROM developer.test_application WHERE metadata->>'contract_template_id'='20000000-0000-0000-0000-000000000501' AND metadata->>'acceptance_template_id'='20000000-0000-0000-0000-000000000502' AND metadata->>'refund_template_id'='20000000-0000-0000-0000-000000000503';" 5 "contract/acceptance/refund template mapping"

echo "[ok] seed 032 five scenarios mapping verified"
