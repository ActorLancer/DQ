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
    echo "[fail] missing expected seed entry: ${label}" >&2
    exit 1
  fi
}

check_count_ge "SELECT COUNT(*) FROM core.organization WHERE org_id IN ('10000000-0000-0000-0000-000000000101','10000000-0000-0000-0000-000000000102','10000000-0000-0000-0000-000000000103');" 3 "core.organization demo tenants"
check_count_ge "SELECT COUNT(*) FROM core.user_account WHERE user_id BETWEEN '10000000-0000-0000-0000-000000000301' AND '10000000-0000-0000-0000-000000000399';" 6 "core.user_account demo users"
check_count_ge "SELECT COUNT(*) FROM authz.subject_role_binding WHERE subject_role_binding_id BETWEEN '10000000-0000-0000-0000-000000000501' AND '10000000-0000-0000-0000-000000000599';" 6 "authz.subject_role_binding demo bindings"

for sku in FILE_STD FILE_SUB SHARE_RO API_SUB API_PPU QRY_LITE SBX_STD RPT_STD; do
  check_exists "SELECT COUNT(*) FROM catalog.product_sku WHERE sku_code='${sku}' AND sku_id::text LIKE '20000000-0000-0000-0000-0000000004%';" "catalog.product_sku.${sku}"
done

check_count_ge "SELECT COUNT(*) FROM contract.template_binding WHERE template_binding_id::text LIKE '20000000-0000-0000-0000-0000000006%';" 12 "contract.template_binding demo entries"
check_count_ge "SELECT COUNT(*) FROM trade.order_main WHERE order_id::text LIKE '30000000-0000-0000-0000-0000000001%';" 13 "trade.order_main demo orders"
check_count_ge "SELECT COUNT(*) FROM trade.order_main WHERE idempotency_key LIKE 'seed-db029-scn-%';" 5 "trade.order_main five scenarios"
check_count_ge "SELECT COUNT(*) FROM delivery.api_credential WHERE api_credential_id::text LIKE '30000000-0000-0000-0000-000000003%';" 3 "delivery.api_credential demo entries"

echo "[ok] seeds 010/020/030 baseline verified"
