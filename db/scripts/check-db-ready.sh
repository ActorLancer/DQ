#!/usr/bin/env bash
set -euo pipefail

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"
DB_READY_TIMEOUT="${DB_READY_TIMEOUT:-60}"

export PGPASSWORD="${DB_PASSWORD}"

deadline=$((SECONDS + DB_READY_TIMEOUT))
while (( SECONDS < deadline )); do
  if pg_isready -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! pg_isready -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" >/dev/null 2>&1; then
  echo "[fail] postgres not ready within ${DB_READY_TIMEOUT}s" >&2
  exit 1
fi

required_schemas=(iam catalog trade delivery billing audit ops)
for schema in "${required_schemas[@]}"; do
  exists="$(psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -Atqc "select exists(select 1 from information_schema.schemata where schema_name='${schema}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] required schema missing: ${schema}" >&2
    exit 1
  fi
done

required_extensions=(pgcrypto citext pg_trgm btree_gist vector)
for ext in "${required_extensions[@]}"; do
  exists="$(psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -Atqc "select exists(select 1 from pg_extension where extname='${ext}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] required extension missing: ${ext}" >&2
    exit 1
  fi
done

echo "[ok] postgres is ready with required schemas and extensions"
