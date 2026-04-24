#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
CANONICAL_MODE="${CANONICAL_MODE:-static}"

fail() {
  echo "[fail] $*" >&2
  exit 1
}

ok() {
  echo "[ok]   $*"
}

log() {
  echo "[info] $*"
}

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"

log "running TEST-016 compose smoke with env=${ENV_FILE}"
ENV_FILE="${ENV_FILE}" ./scripts/smoke-local.sh
CANONICAL_CHECK_MODE="${CANONICAL_MODE}" ENV_FILE="${ENV_FILE}" ./scripts/check-canonical-contracts.sh
ok "TEST-016 compose smoke passed"
