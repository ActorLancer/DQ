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
require_cmd go
require_cmd jq
require_cmd make

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

export DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
export MOCK_PAYMENT_BASE_URL="${MOCK_PAYMENT_BASE_URL:-http://127.0.0.1:${MOCK_PAYMENT_PORT:-8089}}"
export FABRIC_ADAPTER_PROVIDER_MODE="${FABRIC_ADAPTER_PROVIDER_MODE:-mock}"

log "running TEST-007 provider switch checker with env=${ENV_FILE}"

log "ensuring local core + mocks baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh
./scripts/check-mock-payment.sh
ok "mock payment runtime verified"

log "verifying provider-kit mock/real entrypoints"
cargo test -p provider-kit -- --nocapture
MOCK_PAYMENT_ADAPTER_MODE=live \
MOCK_PAYMENT_BASE_URL="${MOCK_PAYMENT_BASE_URL}" \
cargo test -p provider-kit live_mock_payment_adapter_hits_three_mock_paths -- --ignored --nocapture
ok "provider-kit payment/signing/fabric mock/real coverage passed"

log "verifying platform-core startup gate for real provider mode"
cargo test -p platform-core startup_self_check -- --nocapture
ok "platform-core startup gate tests passed"

log "verifying contract signing provider switch over the same business path"
TRADE_DB_SMOKE=1 \
PROVIDER_MODE=mock \
cargo test -p platform-core trade026_contract_signing_provider_db_smoke -- --nocapture
TRADE_DB_SMOKE=1 \
PROVIDER_MODE=real \
FF_REAL_PROVIDER=true \
cargo test -p platform-core trade026_contract_signing_provider_db_smoke -- --nocapture
ok "platform-core contract signing provider switch passed"

log "verifying fabric-adapter provider factory + config tests"
./scripts/fabric-adapter-test.sh
ok "fabric-adapter unit tests passed"

if ! ./scripts/check-fabric-local.sh >/dev/null 2>&1; then
  log "fabric local test-network is not ready; starting it via make up-fabric"
  make up-fabric
fi

./scripts/check-fabric-local.sh
./scripts/fabric-adapter-live-smoke.sh
ok "fabric-adapter mock/fabric-test-network switch passed"

ok "TEST-007 provider switch checker passed"
