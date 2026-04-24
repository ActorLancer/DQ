#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
FIXTURE_DIR="fixtures/smoke/test-004"
RUNTIME_BASELINE_FILE="${FIXTURE_DIR}/runtime-baseline.env"
SEED_VERSIONS_FILE="${FIXTURE_DIR}/required-seed-versions.txt"
APP_LOG_DIR="${APP_LOG_DIR:-target/test-artifacts}"
APP_LOG_FILE="${APP_LOG_FILE:-${APP_LOG_DIR}/test-004-platform-core.log}"
COMPOSE_PROFILES="${COMPOSE_PROFILES:-core}"
APP_PID=""

log() {
  echo "[info] $*"
}

ok() {
  echo "[ok]   $*"
}

fail() {
  echo "[fail] $*" >&2
  if [[ -f "${APP_LOG_FILE}" ]]; then
    echo "[fail] platform-core log: ${APP_LOG_FILE}" >&2
    tail -n 40 "${APP_LOG_FILE}" >&2 || true
  fi
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

wait_http_code() {
  local name="$1"
  local url="$2"
  local expected_regex="$3"
  local timeout_seconds="${4:-120}"
  local sleep_seconds="${5:-2}"
  local deadline=$((SECONDS + timeout_seconds))
  local code=""

  while (( SECONDS < deadline )); do
    code="$(curl -sS -o /dev/null -w '%{http_code}' "${url}" || true)"
    if [[ "${code}" =~ ${expected_regex} ]]; then
      ok "${name} responded with HTTP ${code}"
      return 0
    fi
    sleep "${sleep_seconds}"
  done

  fail "${name} did not reach expected HTTP state at ${url} (last=${code})"
}

assert_json_contains() {
  local json="$1"
  local needle="$2"
  local label="$3"
  printf '%s' "${json}" | rg -Fq "${needle}" || fail "${label} missing ${needle}"
}

assert_success_envelope() {
  local json="$1"
  local label="$2"
  assert_json_contains "${json}" '"code":"OK"' "${label}"
  assert_json_contains "${json}" '"message":"success"' "${label}"
  assert_json_contains "${json}" '"request_id":"' "${label}"
}

assert_dependency_reachable() {
  local json="$1"
  local name="$2"
  local endpoint="$3"
  local label="$4"
  local pattern_one="\"name\":\"${name}\".*\"endpoint\":\"${endpoint}\".*\"reachable\":true"
  local pattern_two="\"endpoint\":\"${endpoint}\".*\"name\":\"${name}\".*\"reachable\":true"

  printf '%s' "${json}" | rg -q "${pattern_one}|${pattern_two}" \
    || fail "${label} missing reachable dependency ${name}@${endpoint}"
}

verify_seed_history() {
  export PGPASSWORD="${DB_PASSWORD}"

  while IFS= read -r version; do
    [[ -n "${version}" ]] || continue
    [[ "${version}" =~ ^# ]] && continue

    local exists
    exists="$(
      psql \
        -h "${DB_HOST}" \
        -p "${DB_PORT}" \
        -U "${DB_USER}" \
        -d "${DB_NAME}" \
        -Atqc "select exists(select 1 from public.seed_history where version='${version}');"
    )"
    [[ "${exists}" == "t" ]] || fail "seed_history missing version ${version}"
  done < "${SEED_VERSIONS_FILE}"

  ok "required seed_history versions recorded"
}

stop_platform_core() {
  if [[ -n "${APP_PID}" ]] && kill -0 "${APP_PID}" >/dev/null 2>&1; then
    kill "${APP_PID}" >/dev/null 2>&1 || true
    wait "${APP_PID}" >/dev/null 2>&1 || true
  fi
}

trap stop_platform_core EXIT

require_cmd docker
require_cmd cargo
require_cmd curl
require_cmd psql
require_cmd rg
require_cmd awk

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline ${RUNTIME_BASELINE_FILE}"
[[ -f "${SEED_VERSIONS_FILE}" ]] || fail "missing seed version baseline ${SEED_VERSIONS_FILE}"

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

APP_PORT="${TEST004_APP_PORT:-${APP_PORT}}"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-${POSTGRES_PORT:-5432}}"
DB_NAME="${DB_NAME:-${POSTGRES_DB:-datab}}"
DB_USER="${DB_USER:-${POSTGRES_USER:-datab}}"
DB_PASSWORD="${DB_PASSWORD:-${POSTGRES_PASSWORD:-datab_local_pass}}"

REDIS_HOST="${REDIS_HOST:-127.0.0.1}"
REDIS_PORT="${REDIS_PORT:-6379}"
KAFKA_HOST="${KAFKA_HOST:-127.0.0.1}"
KAFKA_PORT="${KAFKA_PORT:-${KAFKA_EXTERNAL_PORT:-9094}}"
MINIO_HOST="${MINIO_HOST:-127.0.0.1}"
MINIO_PORT="${MINIO_PORT:-${MINIO_API_PORT:-9000}}"
KEYCLOAK_HOST="${KEYCLOAK_HOST:-127.0.0.1}"
KEYCLOAK_PORT="${KEYCLOAK_PORT:-8081}"
KAFKA_BROKERS="${KAFKA_BROKERS:-127.0.0.1:${KAFKA_EXTERNAL_PORT:-9094}}"
KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-${KAFKA_BROKERS}}"
LATEST_MIGRATION_VERSION="$(awk -F, 'NR > 1 {version = $1} END {print version}' db/migrations/v1/manifest.csv)"
[[ -n "${LATEST_MIGRATION_VERSION}" ]] || fail "cannot resolve latest migration version"
MIGRATION_VERSION="${MIGRATION_VERSION:-${LATEST_MIGRATION_VERSION}}"

export APP_MODE PROVIDER_MODE APP_HOST APP_PORT
export DB_HOST DB_PORT DB_NAME DB_USER DB_PASSWORD
export REDIS_HOST REDIS_PORT
export KAFKA_HOST KAFKA_PORT KAFKA_BROKERS KAFKA_BOOTSTRAP_SERVERS
export MINIO_HOST MINIO_PORT MINIO_ENDPOINT MINIO_ACCESS_KEY MINIO_SECRET_KEY
export KEYCLOAK_HOST KEYCLOAK_PORT KEYCLOAK_BASE_URL KEYCLOAK_REALM
export MIGRATION_VERSION

mkdir -p "${APP_LOG_DIR}"
rm -f "${APP_LOG_FILE}"

log "running TEST-004 migration smoke with env=${ENV_FILE}"
log "bringing up local core stack"
COMPOSE_FILE="${COMPOSE_FILE}" \
COMPOSE_ENV_FILE="${ENV_FILE}" \
COMPOSE_PROFILES="${COMPOSE_PROFILES}" \
./scripts/up-local.sh

log "waiting for minio health before bucket bootstrap"
wait_http_code \
  "MinIO live health" \
  "http://127.0.0.1:${MINIO_API_PORT:-9000}/minio/health/live" \
  '^200$' \
  120 \
  2

log "initializing required minio buckets"
./infra/minio/init-minio.sh

log "verifying current local core stack"
ENV_FILE="${ENV_FILE}" ./scripts/check-local-stack.sh core

log "running DB compatibility baseline"
./db/scripts/verify-db-compatibility.sh

log "verifying seed history baseline"
verify_seed_history

log "starting platform-core smoke instance"
cargo run -p "${APP_PACKAGE}" >"${APP_LOG_FILE}" 2>&1 &
APP_PID="$!"

BASE_URL="http://${APP_HOST}:${APP_PORT}"

wait_http_code "platform-core live" "${BASE_URL}${HEALTH_LIVE_PATH}" '^200$' 180 2
wait_http_code "platform-core ready" "${BASE_URL}${HEALTH_READY_PATH}" '^200$' 180 2

LIVE_JSON="$(curl -fsS "${BASE_URL}${HEALTH_LIVE_PATH}")"
READY_JSON="$(curl -fsS "${BASE_URL}${HEALTH_READY_PATH}")"
DEPS_JSON="$(curl -fsS "${BASE_URL}${HEALTH_DEPS_PATH}")"
RUNTIME_JSON="$(curl -fsS "${BASE_URL}${RUNTIME_PATH}")"

assert_success_envelope "${LIVE_JSON}" "live health"
assert_json_contains "${LIVE_JSON}" '"data":"ok"' "live health"
assert_success_envelope "${READY_JSON}" "ready health"
assert_json_contains "${READY_JSON}" '"data":"ready"' "ready health"

assert_success_envelope "${DEPS_JSON}" "dependency health"
assert_dependency_reachable "${DEPS_JSON}" "db" "127.0.0.1:${DB_PORT}" "dependency health"
assert_dependency_reachable "${DEPS_JSON}" "redis" "127.0.0.1:${REDIS_PORT}" "dependency health"
assert_dependency_reachable "${DEPS_JSON}" "kafka" "127.0.0.1:${KAFKA_PORT}" "dependency health"
assert_dependency_reachable "${DEPS_JSON}" "minio" "127.0.0.1:${MINIO_PORT}" "dependency health"
assert_dependency_reachable "${DEPS_JSON}" "keycloak" "127.0.0.1:${KEYCLOAK_PORT}" "dependency health"

assert_success_envelope "${RUNTIME_JSON}" "runtime endpoint"
assert_json_contains "${RUNTIME_JSON}" '"mode":"'"${APP_MODE}"'"' "runtime endpoint"
assert_json_contains "${RUNTIME_JSON}" '"provider":"'"${PROVIDER_MODE}"'"' "runtime endpoint"
assert_json_contains "${RUNTIME_JSON}" '"migration_version":"'"${LATEST_MIGRATION_VERSION}"'"' "runtime endpoint"
assert_json_contains "${RUNTIME_JSON}" '"enable_demo_features":true' "runtime endpoint"

ok "TEST-004 migration smoke checker passed"
