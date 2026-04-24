#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_PROFILES="${COMPOSE_PROFILES:-core}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/test-artifacts/schema-drift}"
ENTITY_DIR="apps/platform-core/crates/db/src/entity"
WAIT_TIMEOUT_SECONDS="${WAIT_TIMEOUT_SECONDS:-120}"
WAIT_INTERVAL_SECONDS="${WAIT_INTERVAL_SECONDS:-2}"

log() {
  echo "[info] $*"
}

ok() {
  echo "[ok]   $*"
}

fail() {
  echo "[fail] $*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

query_public_tables() {
  local database="$1"
  psql \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d "${database}" \
    -At \
    -c "select tablename from pg_tables where schemaname = 'public' order by 1;"
}

wait_for_public_table() {
  local database="$1"
  local table_name="$2"
  local deadline=$((SECONDS + WAIT_TIMEOUT_SECONDS))

  while (( SECONDS < deadline )); do
    if psql \
      -h "${DB_HOST}" \
      -p "${DB_PORT}" \
      -U "${DB_USER}" \
      -d "${database}" \
      -At \
      -c "select 1 from pg_tables where schemaname = 'public' and tablename = '${table_name}'" \
      | grep -qx '1'; then
      ok "table ready: ${database}.public.${table_name}"
      return 0
    fi
    sleep "${WAIT_INTERVAL_SECONDS}"
  done

  fail "table did not appear before timeout: ${database}.public.${table_name}"
}

capture_entity_catalog() {
  local raw_file="${ARTIFACT_DIR}/entity-table-catalog.raw.txt"
  local final_file="${ARTIFACT_DIR}/entity-table-catalog.txt"
  local duplicate_file="${ARTIFACT_DIR}/entity-table-catalog.duplicates.txt"

  rg -oN --no-filename 'table_name = "[^"]+"' "${ENTITY_DIR}" \
    --glob '*.rs' \
    --glob '!mod.rs' \
    --glob '!prelude.rs' \
    | sed -E 's/.*table_name = "([^"]+)".*/\1/' \
    >"${raw_file}"

  sort -u "${raw_file}" >"${final_file}"
  sort "${raw_file}" | uniq -d >"${duplicate_file}"

  if [[ -s "${duplicate_file}" ]]; then
    fail "duplicate table_name entries found in db::entity catalog; see ${duplicate_file}"
  fi

  ok "captured db::entity catalog"
}

capture_public_catalog() {
  local database="$1"
  local output_file="$2"
  query_public_tables "${database}" >"${output_file}"
  ok "captured public schema catalog: ${database}"
}

check_entity_catalog_drift() {
  local entity_file="${ARTIFACT_DIR}/entity-table-catalog.txt"
  local keycloak_file="${ARTIFACT_DIR}/keycloak-public-tables.txt"
  local datab_file="${ARTIFACT_DIR}/datab-public-tables.txt"
  local -a entity_tables=()
  local -a keycloak_tables=()
  local -a datab_tables=()
  local -a missing_tables=()
  local -a extra_keycloak_tables=()
  declare -A entity_table_set=()
  declare -A keycloak_table_set=()
  declare -A datab_table_set=()

  mapfile -t entity_tables <"${entity_file}"
  mapfile -t keycloak_tables <"${keycloak_file}"
  mapfile -t datab_tables <"${datab_file}"

  for table_name in "${entity_tables[@]}"; do
    entity_table_set["${table_name}"]=1
  done
  for table_name in "${keycloak_tables[@]}"; do
    keycloak_table_set["${table_name}"]=1
  done
  for table_name in "${datab_tables[@]}"; do
    datab_table_set["${table_name}"]=1
  done

  for table_name in "${entity_tables[@]}"; do
    if [[ "${table_name}" == "schema_migration_history" ]]; then
      [[ -n "${datab_table_set[${table_name}]:-}" ]] \
        || missing_tables+=("${table_name} (expected in datab.public)")
      continue
    fi

    [[ -n "${keycloak_table_set[${table_name}]:-}" ]] \
      || missing_tables+=("${table_name} (expected in keycloak.public)")
  done

  for table_name in "${keycloak_tables[@]}"; do
    [[ -n "${entity_table_set[${table_name}]:-}" ]] \
      || extra_keycloak_tables+=("${table_name}")
  done

  if (( ${#missing_tables[@]} > 0 )) || (( ${#extra_keycloak_tables[@]} > 0 )); then
    {
      echo "db::entity catalog drift detected"
      if (( ${#missing_tables[@]} > 0 )); then
        echo "missing live tables for entity catalog:"
        printf '  - %s\n' "${missing_tables[@]}"
      fi
      if (( ${#extra_keycloak_tables[@]} > 0 )); then
        echo "keycloak public tables missing entity definitions:"
        printf '  - %s\n' "${extra_keycloak_tables[@]}"
      fi
      echo "artifact dir: ${ARTIFACT_DIR}"
    } >&2
    exit 1
  fi

  ok "db::entity catalog aligned with keycloak/public and datab.public.schema_migration_history"
}

require_cmd cargo
require_cmd docker
require_cmd psql
require_cmd rg
require_cmd sort

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -d "${ENTITY_DIR}" ]] || fail "missing entity directory ${ENTITY_DIR}"

USER_COMPOSE_PROFILES="${COMPOSE_PROFILES:-}"

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
if [[ -f "${RUNTIME_BASELINE_FILE}" ]]; then
  # shellcheck disable=SC1090
  source "${RUNTIME_BASELINE_FILE}"
fi
set +a

mkdir -p "${ARTIFACT_DIR}"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-${POSTGRES_PORT:-5432}}"
DB_NAME="${DB_NAME:-${POSTGRES_DB:-datab}}"
DB_USER="${DB_USER:-${POSTGRES_USER:-datab}}"
DB_PASSWORD="${DB_PASSWORD:-${POSTGRES_PASSWORD:-datab_local_pass}}"
KEYCLOAK_DB_NAME="${KEYCLOAK_DB_NAME:-keycloak}"
COMPOSE_PROFILES="${USER_COMPOSE_PROFILES:-core}"
export PGPASSWORD="${DB_PASSWORD}"

log "running TEST-017 schema drift checker with env=${ENV_FILE}"
./scripts/check-local-env.sh "${COMPOSE_FILE}" "${ENV_FILE}" "${ENV_FILE}" >/dev/null

log "bringing up local stack profiles=${COMPOSE_PROFILES}"
COMPOSE_FILE="${COMPOSE_FILE}" \
COMPOSE_ENV_FILE="${ENV_FILE}" \
COMPOSE_PROFILES="${COMPOSE_PROFILES}" \
./scripts/up-local.sh

log "verifying core stack readiness"
ENV_FILE="${ENV_FILE}" ./scripts/check-local-stack.sh core >/dev/null

log "applying business migrations"
./db/scripts/migrate-up.sh

wait_for_public_table "${DB_NAME}" "schema_migration_history"
wait_for_public_table "${KEYCLOAK_DB_NAME}" "realm"

capture_entity_catalog
capture_public_catalog "${DB_NAME}" "${ARTIFACT_DIR}/datab-public-tables.txt"
capture_public_catalog "${KEYCLOAK_DB_NAME}" "${ARTIFACT_DIR}/keycloak-public-tables.txt"
check_entity_catalog_drift

log "checking sqlx metadata drift"
cargo sqlx prepare --workspace --check

log "checking offline query compile baseline"
./scripts/check-query-compile.sh

log "checking openapi archive drift"
./scripts/check-openapi-schema.sh

ok "TEST-017 schema drift checker passed"
