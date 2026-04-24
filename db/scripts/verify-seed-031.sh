#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"

export PGPASSWORD="$DB_PASSWORD"
PSQL=(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -X -q -tA)

check_count_eq() {
  local sql="$1"
  local expected="$2"
  local label="$3"
  local count
  count="$("${PSQL[@]}" -c "$sql")"
  if [[ "$count" -ne "$expected" ]]; then
    echo "[fail] ${label} count mismatch: ${count} (!= ${expected})" >&2
    exit 1
  fi
}

check_exists() {
  local sql="$1"
  local label="$2"
  local exists
  exists="$("${PSQL[@]}" -c "$sql")"
  if [[ "$exists" -lt 1 ]]; then
    echo "[fail] missing expected trigger mapping: ${label}" >&2
    exit 1
  fi
}

check_count_eq "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix;" 8 "billing.sku_billing_trigger_matrix"
check_count_eq "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE metadata->>'seed'='db034';" 8 "seed marker db034"

check_exists "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE sku_code='FILE_STD' AND settlement_cycle='t_plus_1_once';" "FILE_STD"
check_exists "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE sku_code='FILE_SUB' AND settlement_cycle='monthly_cycle';" "FILE_SUB"
check_exists "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE sku_code='SHARE_RO' AND billing_trigger='bill_once_on_grant_effective';" "SHARE_RO"
check_exists "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE sku_code='API_SUB' AND dispute_freeze_trigger='freeze_current_cycle_on_sla_dispute';" "API_SUB"
check_exists "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE sku_code='API_PPU' AND settlement_cycle='daily_with_monthly_statement';" "API_PPU"
check_exists "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE sku_code='QRY_LITE' AND refund_entry='refund_if_task_failed_or_unavailable';" "QRY_LITE"
check_exists "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE sku_code='SBX_STD' AND compensation_entry='compensate_on_resource_or_isolation_fault';" "SBX_STD"
check_exists "SELECT COUNT(*) FROM billing.sku_billing_trigger_matrix WHERE sku_code='RPT_STD' AND resume_settlement_trigger='resume_on_review_passed_dispute_closed';" "RPT_STD"

echo "[ok] seed 031 sku trigger matrix verified"
