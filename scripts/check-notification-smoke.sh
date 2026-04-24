#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
APP_LOG_DIR="${APP_LOG_DIR:-target/test-artifacts/notification-smoke}"
APP_LOG_FILE="${APP_LOG_FILE:-${APP_LOG_DIR}/test-027-platform-core.log}"
RAW_DIR="${TEST027_ARTIFACT_DIR:-${ROOT_DIR}/target/test-artifacts/notification-smoke/raw}"
SUMMARY_DIR="${TEST027_SUMMARY_DIR:-${ROOT_DIR}/target/test-artifacts/notification-smoke}"
PLATFORM_CORE_BASE_URL="${PLATFORM_CORE_BASE_URL:-http://127.0.0.1:8094}"
NOTIFICATION_WORKER_BASE_URL="${NOTIFICATION_WORKER_BASE_URL:-http://127.0.0.1:8097}"
OUTBOX_PUBLISHER_BASE_URL="${OUTBOX_PUBLISHER_BASE_URL:-http://127.0.0.1:8098}"
NOTIFICATION_WORKER_REDIS_URL="${NOTIFICATION_WORKER_REDIS_URL:-redis://:datab_redis_pass@127.0.0.1:6379/2}"
PSQL_BIN="${PSQL_BIN:-psql}"

APP_PID=""
APP_STARTED_BY_CHECKER="false"
NOTIFICATION_WORKER_PID=""
OUTBOX_PUBLISHER_PID=""

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

stop_background_services() {
  if [[ -n "${NOTIFICATION_WORKER_PID}" ]] && kill -0 "${NOTIFICATION_WORKER_PID}" >/dev/null 2>&1; then
    kill "${NOTIFICATION_WORKER_PID}" >/dev/null 2>&1 || true
    wait "${NOTIFICATION_WORKER_PID}" >/dev/null 2>&1 || true
  fi
  if [[ -n "${OUTBOX_PUBLISHER_PID}" ]] && kill -0 "${OUTBOX_PUBLISHER_PID}" >/dev/null 2>&1; then
    kill "${OUTBOX_PUBLISHER_PID}" >/dev/null 2>&1 || true
    wait "${OUTBOX_PUBLISHER_PID}" >/dev/null 2>&1 || true
  fi
  NOTIFICATION_WORKER_PID=""
  OUTBOX_PUBLISHER_PID=""
}

cleanup() {
  local exit_code=$?
  trap - EXIT
  stop_background_services
  stop_platform_core
  exit "${exit_code}"
}

wait_http() {
  local url="$1"
  local label="$2"
  local attempts="${3:-240}"
  local delay_seconds="${4:-0.5}"
  local i=1
  while (( i <= attempts )); do
    if curl -fsS "${url}" >/dev/null 2>&1; then
      ok "${label} is ready"
      return 0
    fi
    sleep "${delay_seconds}"
    ((i++))
  done
  fail "${label} did not become ready: ${url}"
}

start_or_reuse_platform_core() {
  local live_code ready_code

  live_code="$(curl -sS -o /dev/null -w '%{http_code}' "${PLATFORM_CORE_BASE_URL}/health/live" || true)"
  ready_code="$(curl -sS -o /dev/null -w '%{http_code}' "${PLATFORM_CORE_BASE_URL}/health/ready" || true)"
  if [[ "${live_code}" == "200" && "${ready_code}" == "200" ]]; then
    ok "reusing existing platform-core on ${PLATFORM_CORE_BASE_URL}"
    return 0
  fi

  mkdir -p "${APP_LOG_DIR}"
  rm -f "${APP_LOG_FILE}"

  log "starting host platform-core TEST-027 instance on ${PLATFORM_CORE_BASE_URL}"
  cargo run -p "${APP_PACKAGE}" > "${APP_LOG_FILE}" 2>&1 &
  APP_PID="$!"
  APP_STARTED_BY_CHECKER="true"

  wait_http_code "platform-core live" "${PLATFORM_CORE_BASE_URL}/health/live" '^200$' 240 2
  wait_http_code "platform-core ready" "${PLATFORM_CORE_BASE_URL}/health/ready" '^200$' 240 2
}

trap cleanup EXIT

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline file ${RUNTIME_BASELINE_FILE}"

require_cmd cargo
require_cmd curl
require_cmd jq
require_cmd node
require_cmd "${PSQL_BIN}"

mkdir -p "${RAW_DIR}" "${SUMMARY_DIR}"

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

LATEST_MIGRATION_VERSION="$(awk -F, 'NR > 1 {version = $1} END {print version}' db/migrations/v1/manifest.csv)"
[[ -n "${LATEST_MIGRATION_VERSION}" ]] || fail "cannot resolve latest migration version"
MIGRATION_VERSION="${MIGRATION_VERSION:-${LATEST_MIGRATION_VERSION}}"

export DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
export KAFKA_BROKERS="${KAFKA_BROKERS:-127.0.0.1:9094}"
export KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-${KAFKA_BROKERS}}"
export APP_MODE PROVIDER_MODE APP_HOST APP_PORT APP_PACKAGE MIGRATION_VERSION
export NOTIFICATION_WORKER_BASE_URL
export TOPIC_NOTIFICATION_DISPATCH="${TOPIC_NOTIFICATION_DISPATCH:-dtp.notification.dispatch}"
export TOPIC_DEAD_LETTER_EVENTS="${TOPIC_DEAD_LETTER_EVENTS:-dtp.dead-letter}"

log "running TEST-027 notification smoke gate with env=${ENV_FILE}"
start_or_reuse_platform_core
log "ensuring local stack baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

if curl -fsS "${NOTIFICATION_WORKER_BASE_URL}/health/live" >/dev/null 2>&1; then
  fail "notification-worker already running at ${NOTIFICATION_WORKER_BASE_URL}; stop it before running TEST-027"
fi
if curl -fsS "${OUTBOX_PUBLISHER_BASE_URL}/health/live" >/dev/null 2>&1; then
  fail "outbox-publisher already running at ${OUTBOX_PUBLISHER_BASE_URL}; stop it before running TEST-027"
fi

log "starting outbox-publisher background process"
APP_PORT="${OUTBOX_PUBLISHER_BASE_URL##*:}" \
cargo run -p outbox-publisher > "${RAW_DIR}/outbox-publisher.log" 2>&1 &
OUTBOX_PUBLISHER_PID=$!
wait_http "${OUTBOX_PUBLISHER_BASE_URL}/health/live" "outbox-publisher"
curl -fsS "${OUTBOX_PUBLISHER_BASE_URL}/health/live" > "${RAW_DIR}/outbox-publisher.health.live.json"
curl -fsS "${OUTBOX_PUBLISHER_BASE_URL}/health/ready" > "${RAW_DIR}/outbox-publisher.health.ready.json"

log "starting notification-worker background process"
APP_PORT="${NOTIFICATION_WORKER_BASE_URL##*:}" \
REDIS_URL="${NOTIFICATION_WORKER_REDIS_URL}" \
TOPIC_NOTIFICATION_DISPATCH="${TOPIC_NOTIFICATION_DISPATCH}" \
TOPIC_DEAD_LETTER_EVENTS="${TOPIC_DEAD_LETTER_EVENTS}" \
cargo run -p notification-worker > "${RAW_DIR}/notification-worker.log" 2>&1 &
NOTIFICATION_WORKER_PID=$!
wait_http "${NOTIFICATION_WORKER_BASE_URL}/health/live" "notification-worker"
wait_http "${NOTIFICATION_WORKER_BASE_URL}/health/ready" "notification-worker ready endpoint"
curl -fsS "${NOTIFICATION_WORKER_BASE_URL}/health/live" > "${RAW_DIR}/notification-worker.health.live.json"
curl -fsS "${NOTIFICATION_WORKER_BASE_URL}/health/ready" > "${RAW_DIR}/notification-worker.health.ready.json"

log "running platform-core notification business-event smokes with live publisher/worker chain"
NOTIF_DB_SMOKE=1 TEST027_LIVE_CHAIN=1 TEST027_ARTIFACT_DIR="${RAW_DIR}" \
cargo test -p platform-core notif004_payment_success_notifications_db_smoke -- --nocapture
NOTIF_DB_SMOKE=1 TEST027_LIVE_CHAIN=1 TEST027_ARTIFACT_DIR="${RAW_DIR}" \
cargo test -p platform-core notif005_delivery_completion_notifications_db_smoke -- --nocapture
NOTIF_DB_SMOKE=1 TEST027_LIVE_CHAIN=1 TEST027_ARTIFACT_DIR="${RAW_DIR}" \
cargo test -p platform-core notif006_acceptance_outcome_notifications_db_smoke -- --nocapture
NOTIF_DB_SMOKE=1 TEST027_LIVE_CHAIN=1 TEST027_ARTIFACT_DIR="${RAW_DIR}" \
cargo test -p platform-core notif007_dispute_settlement_notifications_db_smoke -- --nocapture
curl -fsS "${NOTIFICATION_WORKER_BASE_URL}/metrics" > "${RAW_DIR}/notification-worker.metrics.prom"
ok "platform-core notification business-event smokes passed with live chain"

payment_order_id="$(jq -r '.seed.order_id' "${RAW_DIR}/notif004-payment-success.json")"
payment_lookup_request_id="req-test027-payment-lookup-$(date +%s)"
curl -fsS -X POST "${PLATFORM_CORE_BASE_URL}/api/v1/ops/notifications/audit/search" \
  -H "Content-Type: application/json" \
  -H "x-login-id: local-platform-admin" \
  -H "x-role: platform_admin" \
  -H "x-step-up-token: step-up-local-1" \
  -H "x-request-id: ${payment_lookup_request_id}" \
  -d "$(jq -nc --arg order_id "${payment_order_id}" '{
        order_id: $order_id,
        aggregate_type: "notification.dispatch_request",
        event_type: "notification.requested",
        target_topic: "dtp.notification.dispatch",
        notification_code: "payment.succeeded",
        limit: 10,
        reason: "TEST-027 payment notification linkage smoke"
      }')" \
  > "${RAW_DIR}/platform-payment-audit-search.json"

dispute_case_id="$(jq -r '.dispute.case_id' "${RAW_DIR}/notif007-dispute-settlement.json")"
dispute_lookup_request_id="req-test027-dispute-lookup-$(date +%s)"
curl -fsS -X POST "${PLATFORM_CORE_BASE_URL}/api/v1/ops/notifications/audit/search" \
  -H "Content-Type: application/json" \
  -H "x-login-id: local-platform-admin" \
  -H "x-role: platform_admin" \
  -H "x-step-up-token: step-up-local-1" \
  -H "x-request-id: ${dispute_lookup_request_id}" \
  -d "$(jq -nc --arg case_id "${dispute_case_id}" '{
        case_id: $case_id,
        aggregate_type: "notification.dispatch_request",
        event_type: "notification.requested",
        target_topic: "dtp.notification.dispatch",
        notification_code: "dispute.escalated",
        limit: 10,
        reason: "TEST-027 dispute notification linkage smoke"
      }')" \
  > "${RAW_DIR}/platform-dispute-audit-search.json"

jq -n \
  --arg payment_lookup_request_id "${payment_lookup_request_id}" \
  --arg dispute_lookup_request_id "${dispute_lookup_request_id}" \
  --arg payment_order_id "${payment_order_id}" \
  --arg dispute_case_id "${dispute_case_id}" \
  '{
    payment_lookup_request_id: $payment_lookup_request_id,
    dispute_lookup_request_id: $dispute_lookup_request_id,
    payment_order_id: $payment_order_id,
    dispute_case_id: $dispute_case_id
  }' > "${RAW_DIR}/platform-audit-lookups.json"
ok "platform-core notification audit facade lookup passed"

log "stopping background publisher/worker before isolated notif012 live smoke"
stop_background_services

log "running isolated notification-worker live smoke"
NOTIF_WORKER_DB_SMOKE=1 \
REDIS_URL="${NOTIFICATION_WORKER_REDIS_URL}" \
TEST027_ARTIFACT_DIR="${RAW_DIR}" \
cargo test -p notification-worker notif012_notification_worker_live_smoke -- --nocapture
ok "notification-worker live smoke passed"

log "building TEST-027 summary"
TEST027_ARTIFACT_DIR="${RAW_DIR}" \
TEST027_SUMMARY_DIR="${SUMMARY_DIR}" \
PLATFORM_CORE_BASE_URL="${PLATFORM_CORE_BASE_URL}" \
DATABASE_URL="${DATABASE_URL}" \
PSQL_BIN="${PSQL_BIN}" \
node ./scripts/check-notification-smoke.mjs

ok "TEST-027 notification smoke gate passed"
