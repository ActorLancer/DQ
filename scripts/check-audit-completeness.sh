#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"

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

log "running TEST-009 audit completeness checker with env=${ENV_FILE}"

log "verifying export route rejects unauthorized callers"
cargo test -p platform-core rejects_package_export_without_permission -- --nocapture
ok "package export permission guard passed"

log "verifying export route requires step-up"
cargo test -p platform-core package_export_requires_step_up -- --nocapture
ok "package export step-up guard passed"

log "ensuring local environment baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

log "verifying audit completeness live smoke"
AUD_DB_SMOKE=1 \
cargo test -p platform-core audit_trace_api_db_smoke -- --nocapture
ok "audit trace / export / replay / legal-hold live smoke passed"

ok "TEST-009 audit completeness checker passed"
