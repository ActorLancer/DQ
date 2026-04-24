#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
APP_LOG_DIR="${APP_LOG_DIR:-target/test-artifacts/share-ro-e2e}"
APP_LOG_FILE="${APP_LOG_FILE:-${APP_LOG_DIR}/test-025-platform-core.log}"
ARTIFACT_DIR_INPUT="${ARTIFACT_DIR:-target/test-artifacts/share-ro-e2e}"
RAW_ARTIFACT_DIR_INPUT="${RAW_ARTIFACT_DIR:-${ARTIFACT_DIR_INPUT}/raw}"
FIXTURE_FILE_INPUT="${FIXTURE_FILE:-${ARTIFACT_DIR_INPUT}/live-fixture.json}"
APP_PID=""
APP_STARTED_BY_CHECKER="false"
FIXTURE_READY="false"

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

to_abs_path() {
  case "$1" in
    /*) printf '%s\n' "$1" ;;
    *) printf '%s/%s\n' "${ROOT_DIR}" "$1" ;;
  esac
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

wait_http_code() {
  local name="$1"
  local url="$2"
  local expected_regex="$3"
  local timeout_seconds="${4:-180}"
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

stop_platform_core() {
  if [[ "${APP_STARTED_BY_CHECKER}" == "true" ]] && [[ -n "${APP_PID}" ]] && kill -0 "${APP_PID}" >/dev/null 2>&1; then
    kill "${APP_PID}" >/dev/null 2>&1 || true
    wait "${APP_PID}" >/dev/null 2>&1 || true
  fi
}

cleanup_live_fixture() {
  if [[ "${FIXTURE_READY}" == "true" ]] && [[ -f "${FIXTURE_FILE}" ]]; then
    ENV_FILE="${ENV_FILE}" bash ./scripts/share-ro-live-fixture.sh cleanup --fixture "${FIXTURE_FILE}" || true
  fi
}

cleanup() {
  cleanup_live_fixture
  stop_platform_core
}

trap cleanup EXIT

start_or_reuse_platform_core() {
  local base_url="http://${APP_PUBLIC_HOST}:${APP_PORT}"
  local live_code ready_code

  live_code="$(curl -sS -o /dev/null -w '%{http_code}' "${base_url}${HEALTH_LIVE_PATH}" || true)"
  ready_code="$(curl -sS -o /dev/null -w '%{http_code}' "${base_url}${HEALTH_READY_PATH}" || true)"
  if [[ "${live_code}" == "200" && "${ready_code}" == "200" ]]; then
    ok "reusing existing platform-core on ${base_url}"
    return 0
  fi

  mkdir -p "${APP_LOG_DIR}"
  rm -f "${APP_LOG_FILE}"

  log "starting host platform-core TEST-025 instance on ${base_url}"
  cargo run -p "${APP_PACKAGE}" >"${APP_LOG_FILE}" 2>&1 &
  APP_PID="$!"
  APP_STARTED_BY_CHECKER="true"

  wait_http_code "platform-core live" "${base_url}${HEALTH_LIVE_PATH}" '^200$' 240 2
  wait_http_code "platform-core ready" "${base_url}${HEALTH_READY_PATH}" '^200$' 240 2
}

require_cmd cargo
require_cmd curl
require_cmd docker
require_cmd jq
require_cmd node
require_cmd pnpm
require_cmd psql

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline ${RUNTIME_BASELINE_FILE}"

ARTIFACT_DIR="$(to_abs_path "${ARTIFACT_DIR_INPUT}")"
RAW_ARTIFACT_DIR="$(to_abs_path "${RAW_ARTIFACT_DIR_INPUT}")"
FIXTURE_FILE="$(to_abs_path "${FIXTURE_FILE_INPUT}")"
PORTAL_ARTIFACT_FILE="${RAW_ARTIFACT_DIR}/portal-share-live.json"

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

HOST_KAFKA_BROKERS="${HOST_KAFKA_BROKERS:-127.0.0.1:${KAFKA_EXTERNAL_PORT:-9094}}"
KAFKA_BROKERS="${KAFKA_BROKERS:-${HOST_KAFKA_BROKERS}}"
KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-${KAFKA_BROKERS}}"
APP_PUBLIC_HOST="${APP_PUBLIC_HOST:-127.0.0.1}"
LATEST_MIGRATION_VERSION="$(awk -F, 'NR > 1 {version = $1} END {print version}' db/migrations/v1/manifest.csv)"
[[ -n "${LATEST_MIGRATION_VERSION}" ]] || fail "cannot resolve latest migration version"
MIGRATION_VERSION="${MIGRATION_VERSION:-${LATEST_MIGRATION_VERSION}}"

export APP_MODE PROVIDER_MODE APP_HOST APP_PORT APP_PACKAGE MIGRATION_VERSION
export DATABASE_URL KAFKA_BROKERS KAFKA_BOOTSTRAP_SERVERS

mkdir -p "${ARTIFACT_DIR}" "${RAW_ARTIFACT_DIR}"

readarray -t cargo_tests <<'EOF'
trade012_share_ro_state_machine_db_smoke
dlv006_share_grant_db_smoke
bil026_share_ro_billing_db_smoke
EOF

printf '%s\n' "${cargo_tests[@]}" > "${ARTIFACT_DIR}/executed-cargo-tests.txt"

log "running TEST-025 SHARE_RO checker with env=${ENV_FILE}"
start_or_reuse_platform_core

log "verifying TEST-005 local baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

log "aligning local Keycloak identities with core/authz principals"
./scripts/seed-local-iam-test-identities.sh

log "loading formal demo order baseline"
./scripts/seed-demo.sh --skip-base-seeds
./scripts/check-demo-seed.sh

log "verifying seller/buyer Keycloak password grant"
KEYCLOAK_TOKEN_USERNAME="local-seller-operator" \
KEYCLOAK_TOKEN_PASSWORD="LocalSellerOperator123!" \
KEYCLOAK_EXPECTED_ROLE="seller_operator" \
./scripts/check-keycloak-realm.sh >/dev/null
KEYCLOAK_TOKEN_USERNAME="local-buyer-operator" \
KEYCLOAK_TOKEN_PASSWORD="LocalBuyerOperator123!" \
KEYCLOAK_EXPECTED_ROLE="buyer_operator" \
./scripts/check-keycloak-realm.sh >/dev/null
ok "keycloak seller/buyer grant probe passed"

for test_name in "${cargo_tests[@]}"; do
  log "running cargo test ${test_name}"
  TRADE_DB_SMOKE=1 TEST025_ARTIFACT_DIR="${RAW_ARTIFACT_DIR}" \
    cargo test -p platform-core "${test_name}" -- --nocapture \
    | tee -a "${ARTIFACT_DIR}/cargo-tests.log"
done
ok "backend SHARE_RO suite passed"

log "preparing TEST-025 live portal fixture"
ENV_FILE="${ENV_FILE}" bash ./scripts/share-ro-live-fixture.sh prepare > "${FIXTURE_FILE}"
FIXTURE_READY="true"
WEB_E2E_SHARE_ORDER_ID="$(jq -r '.order_id' "${FIXTURE_FILE}")"
WEB_E2E_SHARE_ASSET_OBJECT_ID="$(jq -r '.asset_object_id' "${FIXTURE_FILE}")"
[[ -n "${WEB_E2E_SHARE_ORDER_ID}" && "${WEB_E2E_SHARE_ORDER_ID}" != "null" ]] || fail "fixture missing order_id"
[[ -n "${WEB_E2E_SHARE_ASSET_OBJECT_ID}" && "${WEB_E2E_SHARE_ASSET_OBJECT_ID}" != "null" ]] || fail "fixture missing asset_object_id"
ok "prepared live SHARE_RO fixture order ${WEB_E2E_SHARE_ORDER_ID}"

log "running portal-web TEST-025 live E2E"
WEB_E2E_LIVE=1 \
WEB_E2E_SELLER_USERNAME="${WEB_E2E_SELLER_USERNAME:-local-seller-operator}" \
WEB_E2E_SELLER_PASSWORD="${WEB_E2E_SELLER_PASSWORD:-LocalSellerOperator123!}" \
WEB_E2E_BUYER_USERNAME="${WEB_E2E_BUYER_USERNAME:-local-buyer-operator}" \
WEB_E2E_BUYER_PASSWORD="${WEB_E2E_BUYER_PASSWORD:-LocalBuyerOperator123!}" \
WEB_E2E_SHARE_ORDER_ID="${WEB_E2E_SHARE_ORDER_ID}" \
WEB_E2E_SHARE_ASSET_OBJECT_ID="${WEB_E2E_SHARE_ASSET_OBJECT_ID}" \
TEST025_PORTAL_ARTIFACT_FILE="${PORTAL_ARTIFACT_FILE}" \
PLATFORM_CORE_BASE_URL="http://${APP_PUBLIC_HOST}:${APP_PORT}" \
pnpm --filter @datab/portal-web test:e2e:share-live \
  | tee "${ARTIFACT_DIR}/portal-live.log"
ok "portal SHARE_RO live E2E passed"

log "verifying TEST-025 artifact summary"
TEST025_ARTIFACT_DIR="${RAW_ARTIFACT_DIR}" \
TEST025_SUMMARY_DIR="${ARTIFACT_DIR}" \
TEST025_PORTAL_ARTIFACT_FILE="${PORTAL_ARTIFACT_FILE}" \
TEST025_FIXTURE_FILE="${FIXTURE_FILE}" \
node ./scripts/check-share-ro-e2e.mjs
ok "TEST-025 SHARE_RO checker passed"
