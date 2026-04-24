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

required_indexes=(
  "search.idx_product_search_document_tsv"
  "search.idx_product_search_document_embedding"
  "trade.idx_order_main_status_created_at"
  "trade.idx_order_line_order_id"
  "trade.idx_authorization_grant_order_id"
  "delivery.idx_delivery_record_order_id"
  "delivery.idx_delivery_ticket_order_id"
  "audit.idx_audit_event_ref"
  "audit.idx_audit_event_trace"
  "ops.idx_outbox_pending"
  "ops.idx_outbox_topic_pending"
  "ops.idx_job_run_status_started"
)

for idx in "${required_indexes[@]}"; do
  exists="$("${PSQL[@]}" -c "SELECT CASE WHEN to_regclass('${idx}') IS NULL THEN 0 ELSE 1 END;")"
  if [[ "$exists" != "1" ]]; then
    echo "[fail] missing required index: ${idx}" >&2
    exit 1
  fi
done

echo "[ok] index baseline review passed"
