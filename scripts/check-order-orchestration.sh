#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
ARTIFACT_DIR_INPUT="${ARTIFACT_DIR:-target/test-artifacts/order-orchestration}"
RAW_ARTIFACT_DIR_INPUT="${RAW_ARTIFACT_DIR:-${ARTIFACT_DIR_INPUT}/raw}"

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

to_abs_path() {
  case "$1" in
    /*) printf '%s\n' "$1" ;;
    *) printf '%s/%s\n' "${ROOT_DIR}" "$1" ;;
  esac
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline ${RUNTIME_BASELINE_FILE}"

require_cmd cargo
require_cmd jq
require_cmd node

ARTIFACT_DIR="$(to_abs_path "${ARTIFACT_DIR_INPUT}")"
RAW_ARTIFACT_DIR="$(to_abs_path "${RAW_ARTIFACT_DIR_INPUT}")"

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

export DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
export KAFKA_BROKERS="${KAFKA_BROKERS:-127.0.0.1:9094}"
export KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-${KAFKA_BROKERS}}"

mkdir -p "${ARTIFACT_DIR}" "${RAW_ARTIFACT_DIR}"

readarray -t cargo_tests <<'EOF'
trade030_payment_result_orchestrator_db_smoke
dlv029_delivery_task_autocreation_db_smoke
dlv017_report_delivery_db_smoke
dlv018_acceptance_db_smoke
dlv025_delivery_storage_query_integration_db_smoke
bil024_billing_trigger_bridge_db_smoke
bil025_billing_adjustment_freeze_db_smoke
EOF

printf '%s\n' "${cargo_tests[@]}" > "${ARTIFACT_DIR}/executed-cargo-tests.txt"

log "running TEST-024 order orchestration checker with env=${ENV_FILE}"

log "running TEST-006 frontend and scenario baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/check-order-e2e.sh | tee "${ARTIFACT_DIR}/order-e2e.log"
ok "frontend scenario baseline passed"

for test_name in "${cargo_tests[@]}"; do
  log "running cargo test ${test_name}"
  TRADE_DB_SMOKE=1 TEST024_ARTIFACT_DIR="${RAW_ARTIFACT_DIR}" \
    cargo test -p platform-core "${test_name}" -- --nocapture \
    | tee -a "${ARTIFACT_DIR}/cargo-tests.log"
done
ok "backend orchestration suite passed"

log "verifying orchestration artifact summary"
TEST024_ARTIFACT_DIR="${RAW_ARTIFACT_DIR}" \
TEST024_SUMMARY_DIR="${ARTIFACT_DIR}" \
node ./scripts/check-order-orchestration.mjs
ok "orchestration summary verified"

ok "TEST-024 order orchestration checker passed"
