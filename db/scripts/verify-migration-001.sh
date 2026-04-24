#!/usr/bin/env bash
set -euo pipefail

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"

export PGPASSWORD="${DB_PASSWORD}"
PSQL=(psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 -X -q -tA)

required_extensions=(pgcrypto citext pg_trgm btree_gist vector)
required_schemas=(common core iam authz catalog contract review trade delivery billing payment support risk audit chain search developer ops ml crosschain ecosystem)
required_common_functions=(tg_set_updated_at tg_order_status_history tg_dispute_status_history tg_write_outbox)

check_extension() {
  local ext="$1"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname='${ext}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] extension missing: ${ext}" >&2
    exit 1
  fi
}

check_schema() {
  local schema="$1"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name='${schema}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] schema missing: ${schema}" >&2
    exit 1
  fi
}

check_common_function() {
  local fn="$1"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname='common' AND p.proname='${fn}' AND p.prorettype='trigger'::regtype);")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] common trigger function missing: common.${fn}" >&2
    exit 1
  fi
}

for ext in "${required_extensions[@]}"; do
  check_extension "${ext}"
done

for schema in "${required_schemas[@]}"; do
  check_schema "${schema}"
done

for fn in "${required_common_functions[@]}"; do
  check_common_function "${fn}"
done

echo "[ok] migration 001 baseline verified"
