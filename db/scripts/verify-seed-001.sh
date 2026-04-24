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

check_group_min() {
  local group="$1"
  local min="$2"
  local count
  count="$("${PSQL[@]}" -c "SELECT COUNT(*) FROM catalog.tag WHERE tag_type='lookup' AND tag_group='${group}' AND status='active';")"
  if [[ "$count" -lt "$min" ]]; then
    echo "[fail] lookup group '${group}' count too small: ${count} (< ${min})" >&2
    exit 1
  fi
}

check_group_min "product_category" 4
check_group_min "industry" 4
check_group_min "risk_level" 4
check_group_min "delivery_mode" 8
check_group_min "status_dictionary" 4
check_group_min "trade_mode" 4

echo "[ok] seed 001 base lookup baseline verified"
