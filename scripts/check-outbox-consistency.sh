#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
NOTIFICATION_WORKER_URL="${NOTIFICATION_WORKER_URL:-http://127.0.0.1:8097}"
NOTIFICATION_WORKER_REDIS_URL="${NOTIFICATION_WORKER_REDIS_URL:-redis://:datab_redis_pass@127.0.0.1:6379/2}"

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

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline file ${RUNTIME_BASELINE_FILE}"

require_cmd cargo
require_cmd curl
require_cmd docker
require_cmd jq

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

export DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
export KAFKA_BROKERS="${KAFKA_BROKERS:-127.0.0.1:9094}"
export KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-${KAFKA_BROKERS}}"

log "running TEST-008 outbox consistency checker with env=${ENV_FILE}"

log "ensuring local environment baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

if curl -fsS "${NOTIFICATION_WORKER_URL}/health/live" >/dev/null 2>&1; then
  fail "notification-worker appears to be running at ${NOTIFICATION_WORKER_URL}; stop it before running TEST-008"
fi

log "verifying transactional success + rollback/no-outbox on order creation"
TRADE_DB_SMOKE=1 \
cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture
ok "platform-core transactional outbox assertions passed"

log "verifying outbox publish success and dead-letter isolation"
AUD_DB_SMOKE=1 \
cargo test -p outbox-publisher outbox_publisher_db_smoke -- --nocapture
ok "outbox-publisher publish / dead-letter smoke passed"

log "verifying duplicate consumer handling does not repeat side effects"
NOTIF_WORKER_DB_SMOKE=1 \
REDIS_URL="${NOTIFICATION_WORKER_REDIS_URL}" \
cargo test -p notification-worker notif012_notification_worker_live_smoke -- --nocapture
ok "notification-worker duplicate consume smoke passed"

ok "TEST-008 outbox consistency checker passed"
