#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/test-artifacts/rollback-recovery}"
SUMMARY_FILE="${SUMMARY_FILE:-${ARTIFACT_DIR}/summary.json}"
DEMO_SEED_VERSION="${DEMO_SEED_VERSION:-demo-v1-core-standard-scenarios}"

BASELINE_ORDER_COUNT="0"
BASELINE_PAYMENT_INTENT_COUNT="0"
BASELINE_DELIVERY_COUNT="0"
BASELINE_SEED_HISTORY_COUNT="0"
POST_RESET_ORDER_COUNT="0"
POST_RESET_SEED_HISTORY_COUNT="0"
RESTORED_ORDER_COUNT="0"
RESTORED_PAYMENT_INTENT_COUNT="0"
RESTORED_DELIVERY_COUNT="0"
RESTORED_SEED_HISTORY_COUNT="0"
RUN_ID=""

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

query_db_value() {
  local sql="$1"
  psql "${DATABASE_URL}" -Atqc "${sql}"
}

query_optional_table_count() {
  local table_name="$1"
  psql "${DATABASE_URL}" -Atqc "SELECT COUNT(*)::text FROM ${table_name}" 2>/dev/null || printf '0\n'
}

assert_equals() {
  local actual="$1"
  local expected="$2"
  local label="$3"
  [[ "${actual}" == "${expected}" ]] || fail "${label} expected ${expected}, got ${actual}"
}

extract_demo_count() {
  local file="$1"
  local prefix="$2"
  local label="$3"
  local value

  value="$(
    awk -v prefix="${prefix}" '
      index($0, "  - " prefix ": ") == 1 {
        print substr($0, length("  - " prefix ": ") + 1)
      }
    ' "${file}" | tail -n 1 | tr -d "\r"
  )"
  [[ -n "${value}" ]] || fail "missing ${label} in ${file}"
  printf '%s\n' "${value}"
}

stop_host_platform_core() {
  local pids
  pids="$(pgrep -f 'platform-core-bin' || true)"
  if [[ -z "${pids}" ]]; then
    return 0
  fi

  log "stopping host platform-core before rollback"
  while IFS= read -r pid; do
    [[ -n "${pid}" ]] || continue
    kill "${pid}" >/dev/null 2>&1 || true
  done <<<"${pids}"

  sleep 2
  while IFS= read -r pid; do
    [[ -n "${pid}" ]] || continue
    if kill -0 "${pid}" >/dev/null 2>&1; then
      kill -9 "${pid}" >/dev/null 2>&1 || true
    fi
  done <<<"${pids}"
}

write_summary_json() {
  jq -n \
    --arg run_id "${RUN_ID}" \
    --arg baseline_order_count "${BASELINE_ORDER_COUNT}" \
    --arg baseline_payment_intent_count "${BASELINE_PAYMENT_INTENT_COUNT}" \
    --arg baseline_delivery_count "${BASELINE_DELIVERY_COUNT}" \
    --arg baseline_seed_history_count "${BASELINE_SEED_HISTORY_COUNT}" \
    --arg post_reset_order_count "${POST_RESET_ORDER_COUNT}" \
    --arg post_reset_seed_history_count "${POST_RESET_SEED_HISTORY_COUNT}" \
    --arg restored_order_count "${RESTORED_ORDER_COUNT}" \
    --arg restored_payment_intent_count "${RESTORED_PAYMENT_INTENT_COUNT}" \
    --arg restored_delivery_count "${RESTORED_DELIVERY_COUNT}" \
    --arg restored_seed_history_count "${RESTORED_SEED_HISTORY_COUNT}" \
    '{
      task_id: "TEST-020",
      run_id: $run_id,
      baseline: {
        demo_order_count: ($baseline_order_count | tonumber),
        demo_payment_intent_count: ($baseline_payment_intent_count | tonumber),
        demo_delivery_record_count: ($baseline_delivery_count | tonumber),
        demo_seed_history_count: ($baseline_seed_history_count | tonumber)
      },
      post_reset: {
        total_order_count: ($post_reset_order_count | tonumber),
        total_seed_history_count: ($post_reset_seed_history_count | tonumber)
      },
      restored: {
        demo_order_count: ($restored_order_count | tonumber),
        demo_payment_intent_count: ($restored_payment_intent_count | tonumber),
        demo_delivery_record_count: ($restored_delivery_count | tonumber),
        demo_seed_history_count: ($restored_seed_history_count | tonumber)
      }
    }' >"${SUMMARY_FILE}"
}

require_cmd bash
require_cmd cargo
require_cmd docker
require_cmd jq
require_cmd node
require_cmd psql

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline ${RUNTIME_BASELINE_FILE}"

mkdir -p "${ARTIFACT_DIR}"

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

export DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
RUN_ID="$(date +%s%N)"

log "running TEST-020 rollback recovery checker with env=${ENV_FILE}"

log "establishing formal baseline before rollback"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh
./scripts/seed-local-iam-test-identities.sh
./scripts/seed-demo.sh --skip-base-seeds
./scripts/check-demo-seed.sh | tee "${ARTIFACT_DIR}/baseline-demo-seed.txt"

BASELINE_ORDER_COUNT="$(extract_demo_count "${ARTIFACT_DIR}/baseline-demo-seed.txt" "demo orders" "baseline demo order count")"
BASELINE_PAYMENT_INTENT_COUNT="$(extract_demo_count "${ARTIFACT_DIR}/baseline-demo-seed.txt" "payment intents/transactions/webhooks/billing" "baseline payment intent count")"
BASELINE_PAYMENT_INTENT_COUNT="${BASELINE_PAYMENT_INTENT_COUNT%%/*}"
BASELINE_DELIVERY_COUNT="$(extract_demo_count "${ARTIFACT_DIR}/baseline-demo-seed.txt" "delivery records/api creds/api usage" "baseline delivery count")"
BASELINE_DELIVERY_COUNT="${BASELINE_DELIVERY_COUNT%%/*}"
BASELINE_SEED_HISTORY_COUNT="$(query_db_value "SELECT COUNT(*)::text FROM public.seed_history WHERE version = '${DEMO_SEED_VERSION}'")"
assert_equals "${BASELINE_ORDER_COUNT}" "10" "baseline demo order count"
assert_equals "${BASELINE_PAYMENT_INTENT_COUNT}" "10" "baseline payment intent count"
assert_equals "${BASELINE_DELIVERY_COUNT}" "11" "baseline delivery record count"
assert_equals "${BASELINE_SEED_HISTORY_COUNT}" "1" "baseline demo seed history count"
printf '%s\n' "${BASELINE_ORDER_COUNT}" >"${ARTIFACT_DIR}/baseline-order-count.txt"
printf '%s\n' "${BASELINE_PAYMENT_INTENT_COUNT}" >"${ARTIFACT_DIR}/baseline-payment-intent-count.txt"
printf '%s\n' "${BASELINE_DELIVERY_COUNT}" >"${ARTIFACT_DIR}/baseline-delivery-count.txt"
printf '%s\n' "${BASELINE_SEED_HISTORY_COUNT}" >"${ARTIFACT_DIR}/baseline-seed-history-count.txt"

log "stopping host app and local stack before rollback drill"
stop_host_platform_core
COMPOSE_ENV_FILE="${ENV_FILE}" bash ./scripts/down-local.sh

log "restarting local infrastructure for rollback drill"
COMPOSE_PROFILES="core,observability,mocks" \
COMPOSE_ENV_FILE="${ENV_FILE}" \
bash ./scripts/up-local.sh
ENV_FILE="${ENV_FILE}" bash ./scripts/check-local-stack.sh core

log "resetting local business database through formal migrate-reset path"
bash ./db/scripts/migrate-reset.sh
POST_RESET_ORDER_COUNT="$(query_db_value "SELECT COUNT(*)::text FROM trade.order_main")"
POST_RESET_SEED_HISTORY_COUNT="$(query_optional_table_count "public.seed_history")"
assert_equals "${POST_RESET_ORDER_COUNT}" "0" "post-reset order count"
assert_equals "${POST_RESET_SEED_HISTORY_COUNT}" "0" "post-reset seed history count"
printf '%s\n' "${POST_RESET_ORDER_COUNT}" >"${ARTIFACT_DIR}/post-reset-order-count.txt"
printf '%s\n' "${POST_RESET_SEED_HISTORY_COUNT}" >"${ARTIFACT_DIR}/post-reset-seed-history-count.txt"

log "replaying formal runtime bootstrap and base seed"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

log "restoring IAM test principals and formal demo data"
./scripts/seed-local-iam-test-identities.sh
./scripts/seed-demo.sh --skip-base-seeds
./scripts/check-demo-seed.sh | tee "${ARTIFACT_DIR}/restored-demo-seed.txt"

KEYCLOAK_TOKEN_USERNAME="local-buyer-operator" \
KEYCLOAK_TOKEN_PASSWORD="LocalBuyerOperator123!" \
KEYCLOAK_EXPECTED_ROLE="buyer_operator" \
./scripts/check-keycloak-realm.sh >/dev/null

RESTORED_ORDER_COUNT="$(extract_demo_count "${ARTIFACT_DIR}/restored-demo-seed.txt" "demo orders" "restored demo order count")"
RESTORED_PAYMENT_INTENT_COUNT="$(extract_demo_count "${ARTIFACT_DIR}/restored-demo-seed.txt" "payment intents/transactions/webhooks/billing" "restored payment intent count")"
RESTORED_PAYMENT_INTENT_COUNT="${RESTORED_PAYMENT_INTENT_COUNT%%/*}"
RESTORED_DELIVERY_COUNT="$(extract_demo_count "${ARTIFACT_DIR}/restored-demo-seed.txt" "delivery records/api creds/api usage" "restored delivery count")"
RESTORED_DELIVERY_COUNT="${RESTORED_DELIVERY_COUNT%%/*}"
RESTORED_SEED_HISTORY_COUNT="$(query_db_value "SELECT COUNT(*)::text FROM public.seed_history WHERE version = '${DEMO_SEED_VERSION}'")"
assert_equals "${RESTORED_ORDER_COUNT}" "10" "restored demo order count"
assert_equals "${RESTORED_PAYMENT_INTENT_COUNT}" "10" "restored payment intent count"
assert_equals "${RESTORED_DELIVERY_COUNT}" "11" "restored delivery record count"
assert_equals "${RESTORED_SEED_HISTORY_COUNT}" "1" "restored demo seed history count"
printf '%s\n' "${RESTORED_ORDER_COUNT}" >"${ARTIFACT_DIR}/restored-order-count.txt"
printf '%s\n' "${RESTORED_PAYMENT_INTENT_COUNT}" >"${ARTIFACT_DIR}/restored-payment-intent-count.txt"
printf '%s\n' "${RESTORED_DELIVERY_COUNT}" >"${ARTIFACT_DIR}/restored-delivery-count.txt"
printf '%s\n' "${RESTORED_SEED_HISTORY_COUNT}" >"${ARTIFACT_DIR}/restored-seed-history-count.txt"

write_summary_json
ok "TEST-020 rollback recovery checker passed"
