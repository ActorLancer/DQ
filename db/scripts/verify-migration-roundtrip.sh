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

query_single() {
  local sql="$1"
  "${PSQL[@]}" -c "$sql"
}

echo "==> reset and full upgrade"
./db/scripts/migrate-reset.sh

echo "==> full downgrade drill"
./db/scripts/migrate-down.sh

latest_down_count="$(query_single "WITH ranked AS (SELECT version, direction, ROW_NUMBER() OVER (PARTITION BY version ORDER BY executed_at DESC, id DESC) AS rn FROM public.schema_migration_history) SELECT COUNT(*) FROM ranked WHERE rn = 1 AND direction = 'down';")"
if [[ "$latest_down_count" -lt 22 ]]; then
  echo "[fail] downgrade drill latest-direction check failed: down_count=${latest_down_count} (< 22)" >&2
  exit 1
fi

echo "==> full re-upgrade drill"
./db/scripts/migrate-up.sh

latest_up_count="$(query_single "WITH ranked AS (SELECT version, direction, ROW_NUMBER() OVER (PARTITION BY version ORDER BY executed_at DESC, id DESC) AS rn FROM public.schema_migration_history) SELECT COUNT(*) FROM ranked WHERE rn = 1 AND direction = 'up';")"
if [[ "$latest_up_count" -lt 22 ]]; then
  echo "[fail] re-upgrade drill latest-direction check failed: up_count=${latest_up_count} (< 22)" >&2
  exit 1
fi

echo "[ok] migration roundtrip baseline verified"
