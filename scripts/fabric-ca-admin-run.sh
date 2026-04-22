#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
source "${REPO_ROOT}/scripts/go-env.sh"

if [[ -f "${REPO_ROOT}/infra/docker/.env.local" ]]; then
  set -a
  # shellcheck disable=SC1091
  source "${REPO_ROOT}/infra/docker/.env.local"
  set +a
fi

export DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
export FABRIC_CA_ADMIN_PORT="${FABRIC_CA_ADMIN_PORT:-18112}"
export FABRIC_CA_ADMIN_BASE_URL="${FABRIC_CA_ADMIN_BASE_URL:-http://127.0.0.1:${FABRIC_CA_ADMIN_PORT}}"
export FABRIC_CA_ADMIN_MODE="${FABRIC_CA_ADMIN_MODE:-mock}"

pushd "${REPO_ROOT}/services/fabric-ca-admin" >/dev/null
go run ./cmd/fabric-ca-admin
popd >/dev/null
