#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
FIXTURE_DIR="fixtures/smoke/test-005"
RUNTIME_BASELINE_FILE="${FIXTURE_DIR}/runtime-baseline.env"
CONTROL_PLANE_ENDPOINTS_FILE="${FIXTURE_DIR}/required-control-plane-endpoints.json"
TOPIC_CATALOG="${TOPIC_CATALOG:-infra/kafka/topics.v1.json}"
APP_LOG_DIR="${APP_LOG_DIR:-target/test-artifacts}"
APP_LOG_FILE="${APP_LOG_FILE:-${APP_LOG_DIR}/test-005-platform-core.log}"
KAFKA_CONTAINER="${KAFKA_CONTAINER:-datab-kafka}"
MINIO_ALIAS="${MINIO_ALIAS:-local}"
MINIO_MC_IMAGE="${MINIO_MC_IMAGE:-minio/mc:RELEASE.2025-08-13T08-35-41Z}"
KCAT_IMAGE="${KCAT_IMAGE:-edenhill/kcat:1.7.1}"
APP_PID=""
APP_STARTED_BY_SMOKE="false"

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

resolve_base_url() {
  case "$1" in
    platform-core)
      printf 'http://%s:%s\n' "${APP_PUBLIC_HOST}" "${APP_PORT}"
      ;;
    prometheus)
      printf '%s\n' "${PROM_URL}"
      ;;
    alertmanager)
      printf '%s\n' "${ALERTMANAGER_URL}"
      ;;
    grafana)
      printf '%s\n' "${GRAFANA_URL}"
      ;;
    keycloak)
      printf '%s\n' "${KEYCLOAK_URL}"
      ;;
    mock-payment)
      printf '%s\n' "${MOCK_BASE_URL}"
      ;;
    *)
      fail "unsupported control-plane target: $1"
      ;;
  esac
}

stop_platform_core() {
  if [[ "${APP_STARTED_BY_SMOKE}" == "true" ]] && [[ -n "${APP_PID}" ]] && kill -0 "${APP_PID}" >/dev/null 2>&1; then
    kill "${APP_PID}" >/dev/null 2>&1 || true
    wait "${APP_PID}" >/dev/null 2>&1 || true
  fi
}

trap stop_platform_core EXIT

smoke_db_migratable() {
  local db_host="${DB_HOST:-127.0.0.1}"
  local db_port="${DB_PORT:-5432}"
  local db_name="${DB_NAME:-datab}"
  local db_user="${DB_USER:-datab}"
  local db_password="${DB_PASSWORD:-datab_local_pass}"

  if [[ -n "${DATABASE_URL:-}" ]]; then
    psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 <<'SQL' >/dev/null
BEGIN;
CREATE TABLE IF NOT EXISTS smoke_migration_probe (
  id BIGSERIAL PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE smoke_migration_probe ADD COLUMN IF NOT EXISTS note TEXT;
DROP TABLE smoke_migration_probe;
COMMIT;
SQL
    ok "database migration probe passed"
    return 0
  fi

  export PGPASSWORD="${db_password}"
  psql -h "${db_host}" -p "${db_port}" -U "${db_user}" -d "${db_name}" -v ON_ERROR_STOP=1 <<'SQL' >/dev/null
BEGIN;
CREATE TABLE IF NOT EXISTS smoke_migration_probe (
  id BIGSERIAL PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE smoke_migration_probe ADD COLUMN IF NOT EXISTS note TEXT;
DROP TABLE smoke_migration_probe;
COMMIT;
SQL
  ok "database migration probe passed"
}

smoke_minio_buckets() {
  local access_key="${MINIO_ACCESS_KEY:-${MINIO_ROOT_USER:-datab}}"
  local secret_key="${MINIO_SECRET_KEY:-${MINIO_ROOT_PASSWORD:-datab_local_pass}}"
  local host_value="${MINIO_ENDPOINT/http:\/\//http://${access_key}:${secret_key}@}"
  host_value="${host_value/https:\/\//https://${access_key}:${secret_key}@}"
  local buckets=(
    "${BUCKET_RAW_DATA:-raw-data}"
    "${BUCKET_PREVIEW_ARTIFACTS:-preview-artifacts}"
    "${BUCKET_DELIVERY_OBJECTS:-delivery-objects}"
    "${BUCKET_REPORT_RESULTS:-report-results}"
    "${BUCKET_EVIDENCE_PACKAGES:-evidence-packages}"
    "${BUCKET_MODEL_ARTIFACTS:-model-artifacts}"
  )

  for bucket in "${buckets[@]}"; do
    docker run --rm --network host \
      -e "MC_HOST_${MINIO_ALIAS}=${host_value}" \
      "${MINIO_MC_IMAGE}" ls "${MINIO_ALIAS}/${bucket}" >/dev/null
  done

  ok "minio buckets probe passed"
}

smoke_keycloak_realm() {
  ./scripts/check-keycloak-realm.sh >/dev/null
  ok "keycloak realm import and password grant probe passed"
}

smoke_topic_topology() {
  ./scripts/check-topic-topology.sh >/dev/null
  ok "topic topology probe passed"
}

smoke_kafka_topics() {
  [[ -f "${TOPIC_CATALOG}" ]] || fail "topic catalog missing: ${TOPIC_CATALOG}"

  local listed
  listed="$(docker exec "${KAFKA_CONTAINER}" /opt/kafka/bin/kafka-topics.sh --bootstrap-server "${CONTAINER_LOCAL_KAFKA_BROKER}" --list)"
  while IFS= read -r topic_entry; do
    local env_key default_name topic
    env_key="$(jq -r '.env_key' <<<"${topic_entry}")"
    default_name="$(jq -r '.name' <<<"${topic_entry}")"
    topic="${!env_key:-${default_name}}"
    grep -qx "${topic}" <<<"${listed}" || fail "topic missing: ${topic}"
  done < <(jq -c '.topics[] | select(.required_in_smoke == true)' "${TOPIC_CATALOG}")

  ok "canonical kafka topics probe passed"
}

smoke_kafka_boundary() {
  local advertised
  local metadata_file

  advertised="$(docker exec "${KAFKA_CONTAINER}" printenv KAFKA_ADVERTISED_LISTENERS)"
  [[ "${advertised}" == *"PLAINTEXT://${CONTAINER_KAFKA_BROKER}"* ]] \
    || fail "kafka advertised listeners missing compose boundary ${CONTAINER_KAFKA_BROKER}: ${advertised}"
  [[ "${advertised}" == *"EXTERNAL://${HOST_KAFKA_BROKERS}"* ]] \
    || fail "kafka advertised listeners missing host boundary ${HOST_KAFKA_BROKERS}: ${advertised}"

  docker exec "${KAFKA_CONTAINER}" /opt/kafka/bin/kafka-topics.sh \
    --bootstrap-server "${CONTAINER_LOCAL_KAFKA_BROKERS:-${CONTAINER_LOCAL_KAFKA_BROKER}}" \
    --list >/dev/null

  metadata_file="$(mktemp)"
  if ! docker run --rm --network host "${KCAT_IMAGE}" -b "${HOST_KAFKA_BROKERS}" -L >"${metadata_file}" 2>&1; then
    rm -f "${metadata_file}"
    fail "host kafka metadata probe failed via ${HOST_KAFKA_BROKERS}"
  fi
  rg -Fq "${HOST_KAFKA_BROKERS}" "${metadata_file}" \
    || fail "host kafka metadata did not expose ${HOST_KAFKA_BROKERS}"
  rm -f "${metadata_file}"

  ok "host/container kafka boundary probe passed"
}

smoke_observability() {
  local deadline=$((SECONDS + 40))
  while (( SECONDS < deadline )); do
    if ./scripts/check-observability-stack.sh >/dev/null 2>&1; then
      ok "observability datasources / dashboards / targets probe passed"
      return 0
    fi
    sleep 2
  done

  ./scripts/check-observability-stack.sh >/dev/null
}

smoke_mock_payment_callback() {
  wait_http_code "mock payment readiness" "${MOCK_BASE_URL}/health/ready" '^200$' 120 2
  ./scripts/check-mock-payment.sh >/dev/null
  ok "mock payment callback probe passed"
}

smoke_platform_core_runtime() {
  local base_url="http://${APP_PUBLIC_HOST}:${APP_PORT}"
  local live_json ready_json deps_json runtime_json

  live_json="$(curl -fsS "${base_url}${HEALTH_LIVE_PATH}")"
  ready_json="$(curl -fsS "${base_url}${HEALTH_READY_PATH}")"
  deps_json="$(curl -fsS "${base_url}${HEALTH_DEPS_PATH}")"
  runtime_json="$(curl -fsS "${base_url}${RUNTIME_PATH}")"

  assert_success_envelope "${live_json}" "platform-core live"
  assert_json_contains "${live_json}" '"data":"ok"' "platform-core live"
  assert_success_envelope "${ready_json}" "platform-core ready"
  assert_json_contains "${ready_json}" '"data":"ready"' "platform-core ready"

  assert_success_envelope "${deps_json}" "platform-core deps"
  assert_dependency_reachable "${deps_json}" "db" "127.0.0.1:${DB_PORT}" "platform-core deps"
  assert_dependency_reachable "${deps_json}" "redis" "127.0.0.1:${REDIS_PORT}" "platform-core deps"
  assert_dependency_reachable "${deps_json}" "kafka" "127.0.0.1:${KAFKA_PORT}" "platform-core deps"
  assert_dependency_reachable "${deps_json}" "minio" "127.0.0.1:${MINIO_API_PORT:-9000}" "platform-core deps"
  assert_dependency_reachable "${deps_json}" "keycloak" "127.0.0.1:${KEYCLOAK_PORT:-8081}" "platform-core deps"

  assert_success_envelope "${runtime_json}" "platform-core runtime"
  assert_json_contains "${runtime_json}" "\"mode\":\"${APP_MODE}\"" "platform-core runtime"
  assert_json_contains "${runtime_json}" "\"provider\":\"${PROVIDER_MODE}\"" "platform-core runtime"
  assert_json_contains "${runtime_json}" "\"migration_version\":\"${LATEST_MIGRATION_VERSION}\"" "platform-core runtime"

  ok "platform-core runtime probe passed"
}

run_control_plane_endpoint_checks() {
  [[ -f "${CONTROL_PLANE_ENDPOINTS_FILE}" ]] || fail "missing ${CONTROL_PLANE_ENDPOINTS_FILE}"

  while IFS= read -r endpoint; do
    local name target path expected_status envelope url response_file code body base_url
    name="$(jq -r '.name' <<<"${endpoint}")"
    target="$(jq -r '.target' <<<"${endpoint}")"
    path="$(jq -r '.path' <<<"${endpoint}")"
    expected_status="$(jq -r '.expected_status' <<<"${endpoint}")"
    envelope="$(jq -r '.envelope // empty' <<<"${endpoint}")"
    base_url="$(resolve_base_url "${target}")"
    url="${base_url}${path}"
    response_file="$(mktemp)"

    curl_args=(-sS -o "${response_file}" -w '%{http_code}')
    while IFS= read -r header; do
      [[ -n "${header}" ]] || continue
      curl_args+=(-H "${header}")
    done < <(jq -r '.headers // {} | to_entries[]? | "\(.key): \(.value)"' <<<"${endpoint}")

    code="$(curl "${curl_args[@]}" "${url}" || true)"
    [[ "${code}" =~ ${expected_status} ]] \
      || fail "${name} expected HTTP ${expected_status} at ${url}, got ${code}"

    body="$(jq -c . "${response_file}" 2>/dev/null || cat "${response_file}")"
    rm -f "${response_file}"

    if [[ "${envelope}" == "success" ]]; then
      assert_success_envelope "${body}" "${name}"
    fi

    while IFS= read -r needle; do
      [[ -n "${needle}" ]] || continue
      assert_json_contains "${body}" "${needle}" "${name}"
    done < <(jq -r '.contains[]?' <<<"${endpoint}")

    ok "${name} probe passed"
  done < <(jq -c '.endpoints[]' "${CONTROL_PLANE_ENDPOINTS_FILE}")
}

start_or_reuse_platform_core() {
  local base_url="http://${APP_PUBLIC_HOST}:${APP_PORT}"
  local live_code runtime_code runtime_json

  live_code="$(curl -sS -o /dev/null -w '%{http_code}' "${base_url}${HEALTH_LIVE_PATH}" || true)"
  runtime_code="$(curl -sS -o /dev/null -w '%{http_code}' "${base_url}${RUNTIME_PATH}" || true)"
  if [[ "${live_code}" == "200" && "${runtime_code}" == "200" ]]; then
    runtime_json="$(curl -fsS "${base_url}${RUNTIME_PATH}")"
    assert_success_envelope "${runtime_json}" "existing platform-core runtime"
    assert_json_contains "${runtime_json}" "\"mode\":\"${APP_MODE}\"" "existing platform-core runtime"
    assert_json_contains "${runtime_json}" "\"provider\":\"${PROVIDER_MODE}\"" "existing platform-core runtime"
    ok "reusing existing platform-core on ${base_url}"
    return 0
  fi

  mkdir -p "${APP_LOG_DIR}"
  rm -f "${APP_LOG_FILE}"

  log "starting host platform-core smoke instance on ${base_url}"
  cargo run -p "${APP_PACKAGE}" >"${APP_LOG_FILE}" 2>&1 &
  APP_PID="$!"
  APP_STARTED_BY_SMOKE="true"

  wait_http_code "platform-core live" "${base_url}${HEALTH_LIVE_PATH}" '^200$' 240 2
  wait_http_code "platform-core ready" "${base_url}${HEALTH_READY_PATH}" '^200$' 240 2
}

require_cmd docker
require_cmd cargo
require_cmd curl
require_cmd jq
require_cmd psql
require_cmd rg

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline ${RUNTIME_BASELINE_FILE}"
[[ -f "${CONTROL_PLANE_ENDPOINTS_FILE}" ]] || fail "missing control-plane endpoint baseline ${CONTROL_PLANE_ENDPOINTS_FILE}"

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-${POSTGRES_PORT:-5432}}"
DB_NAME="${DB_NAME:-${POSTGRES_DB:-datab}}"
DB_USER="${DB_USER:-${POSTGRES_USER:-datab}}"
DB_PASSWORD="${DB_PASSWORD:-${POSTGRES_PASSWORD:-datab_local_pass}}"
REDIS_HOST="${REDIS_HOST:-127.0.0.1}"
REDIS_PORT="${REDIS_PORT:-6379}"
KAFKA_PORT="${KAFKA_PORT:-${KAFKA_EXTERNAL_PORT:-9094}}"
HOST_KAFKA_BROKERS="${HOST_KAFKA_BROKERS:-127.0.0.1:${KAFKA_EXTERNAL_PORT:-9094}}"
KAFKA_BROKERS="${KAFKA_BROKERS:-${HOST_KAFKA_BROKERS}}"
KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-${KAFKA_BROKERS}}"
MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://127.0.0.1:${MINIO_API_PORT:-9000}}"
MOCK_BASE_URL="${MOCK_BASE_URL:-http://127.0.0.1:${MOCK_PAYMENT_PORT:-8089}}"
GRAFANA_URL="${GRAFANA_URL:-http://127.0.0.1:${GRAFANA_PORT:-3000}}"
PROM_URL="${PROM_URL:-http://127.0.0.1:${PROMETHEUS_PORT:-9090}}"
ALERTMANAGER_URL="${ALERTMANAGER_URL:-http://127.0.0.1:${ALERTMANAGER_PORT:-9093}}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://127.0.0.1:${KEYCLOAK_PORT:-8081}}"
LATEST_MIGRATION_VERSION="$(awk -F, 'NR > 1 {version = $1} END {print version}' db/migrations/v1/manifest.csv)"
[[ -n "${LATEST_MIGRATION_VERSION}" ]] || fail "cannot resolve latest migration version"
MIGRATION_VERSION="${MIGRATION_VERSION:-${LATEST_MIGRATION_VERSION}}"

export APP_MODE PROVIDER_MODE APP_HOST APP_PORT APP_PACKAGE MIGRATION_VERSION
export DATABASE_URL
export KAFKA_BROKERS KAFKA_BOOTSTRAP_SERVERS

APP_PUBLIC_HOST="${APP_PUBLIC_HOST:-127.0.0.1}"

log "running TEST-005 local environment smoke with env=${ENV_FILE}"
log "checking local environment prerequisites"
./scripts/check-local-env.sh "${COMPOSE_FILE}" "${ENV_FILE}" "${ENV_FILE}" >/dev/null

log "bringing up local stack profiles=${COMPOSE_PROFILES}"
COMPOSE_FILE="${COMPOSE_FILE}" \
COMPOSE_ENV_FILE="${ENV_FILE}" \
COMPOSE_PROFILES="${COMPOSE_PROFILES}" \
./scripts/up-local.sh

wait_http_code \
  "MinIO live health" \
  "http://127.0.0.1:${MINIO_API_PORT:-9000}/minio/health/live" \
  '^200$' \
  120 \
  2

log "applying base schema and seed baseline"
./db/scripts/migrate-up.sh
./db/scripts/seed-up.sh

log "initializing required minio buckets"
./infra/minio/init-minio.sh

start_or_reuse_platform_core

log "running infrastructure and runtime probes"
ENV_FILE="${ENV_FILE}" ./scripts/check-local-stack.sh full >/dev/null
ok "compose startup and core services ready probe passed"
smoke_db_migratable
smoke_minio_buckets
smoke_keycloak_realm
smoke_platform_core_runtime
smoke_topic_topology
smoke_kafka_topics
smoke_kafka_boundary
smoke_observability
smoke_mock_payment_callback
run_control_plane_endpoint_checks

ok "TEST-005 local environment smoke passed"
