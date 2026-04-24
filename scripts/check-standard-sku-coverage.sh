#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
MATRIX_FILE="${MATRIX_FILE:-fixtures/demo/sku-coverage-matrix.json}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/test-artifacts/standard-sku-coverage}"
APP_LOG_DIR="${APP_LOG_DIR:-target/test-artifacts}"
APP_LOG_FILE="${APP_LOG_FILE:-${APP_LOG_DIR}/test-023-platform-core.log}"
APP_PID=""
APP_STARTED_BY_CHECKER="false"

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

  log "starting host platform-core TEST-023 instance on ${base_url}"
  cargo run -p "${APP_PACKAGE}" >"${APP_LOG_FILE}" 2>&1 &
  APP_PID="$!"
  APP_STARTED_BY_CHECKER="true"

  wait_http_code "platform-core live" "${base_url}${HEALTH_LIVE_PATH}" '^200$' 240 2
  wait_http_code "platform-core ready" "${base_url}${HEALTH_READY_PATH}" '^200$' 240 2
}

trap stop_platform_core EXIT

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline file ${RUNTIME_BASELINE_FILE}"
[[ -f "${MATRIX_FILE}" ]] || fail "missing matrix file ${MATRIX_FILE}"

require_cmd cargo
require_cmd curl
require_cmd docker
require_cmd jq
require_cmd node

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

export DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
export KAFKA_BROKERS="${KAFKA_BROKERS:-127.0.0.1:9094}"
export KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-${KAFKA_BROKERS}}"
export PLATFORM_CORE_BASE_URL="${PLATFORM_CORE_BASE_URL:-http://${APP_PUBLIC_HOST:-127.0.0.1}:${APP_PORT:-8094}}"
export STANDARD_SKU_COVERAGE_ARTIFACT_DIR="${ARTIFACT_DIR}"

mkdir -p "${ARTIFACT_DIR}"

log "running TEST-023 standard sku coverage checker with env=${ENV_FILE}"

log "ensuring local environment baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

start_or_reuse_platform_core

log "verifying demo fixture matrix authority"
./scripts/check-demo-fixtures.sh
ok "demo fixtures authority passed"

log "loading formal demo order baseline"
./scripts/seed-demo.sh --skip-base-seeds
./scripts/check-demo-seed.sh | tee "${ARTIFACT_DIR}/demo-seed.txt" >/dev/null
ok "demo seed baseline passed"

readarray -t cargo_tests < <(
  jq -r '
    (
      .shared_checks[]?,
      .standard_sku_matrix[].main_path_evidence.checks[]?,
      .standard_sku_matrix[].exception_path_evidence.checks[]?,
      .standard_sku_matrix[].refund_or_dispute_evidence.checks[]?
    )
    | select(.kind == "cargo_test")
    | .target
  ' "${MATRIX_FILE}" | sort -u
)

[[ "${#cargo_tests[@]}" -gt 0 ]] || fail "no cargo tests resolved from ${MATRIX_FILE}"
printf '%s\n' "${cargo_tests[@]}" > "${ARTIFACT_DIR}/executed-cargo-tests.txt"

for test_name in "${cargo_tests[@]}"; do
  log "running cargo test ${test_name}"
  TRADE_DB_SMOKE=1 CATALOG_DB_SMOKE=1 cargo test -p platform-core "${test_name}" -- --nocapture
done
ok "cargo evidence suite passed"

log "verifying standard sku matrix through live catalog/billing api"
node ./scripts/check-standard-sku-coverage.mjs
ok "live api matrix verification passed"

ok "TEST-023 standard sku coverage checker passed"
