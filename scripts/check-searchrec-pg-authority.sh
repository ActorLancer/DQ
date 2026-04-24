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

log "running TEST-010 search/recommendation PostgreSQL authority checker with env=${ENV_FILE}"

log "ensuring local environment baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

log "verifying search alias switch still returns PostgreSQL-filtered results"
SEARCH_DB_SMOKE=1 \
cargo test -p platform-core search_visibility_and_alias_consistency_db_smoke -- --nocapture
ok "search alias switch + PostgreSQL final gate smoke passed"

log "verifying search PostgreSQL fallback path"
SEARCH_DB_SMOKE=1 \
APP_MODE=local \
OPENSEARCH_ENDPOINT=http://127.0.0.1:1 \
cargo test -p platform-core search_catalog_pg_fallback_db_smoke -- --nocapture
ok "search PostgreSQL fallback smoke passed"

log "verifying recommendation baseline request / result persistence"
RECOMMEND_DB_SMOKE=1 \
cargo test -p platform-core recommendation_get_api_db_smoke -- --nocapture
ok "recommendation baseline smoke passed"

log "verifying recommendation filters frozen products by PostgreSQL authority"
RECOMMEND_DB_SMOKE=1 \
cargo test -p platform-core recommendation_filters_frozen_product_db_smoke -- --nocapture
ok "recommendation PostgreSQL final gate smoke passed"

ok "TEST-010 search/recommendation PostgreSQL authority checker passed"
