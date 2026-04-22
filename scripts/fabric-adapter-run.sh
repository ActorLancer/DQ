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

export KAFKA_BROKERS="${KAFKA_BROKERS:-${KAFKA_BOOTSTRAP_SERVERS:-127.0.0.1:${KAFKA_EXTERNAL_PORT:-9094}}}"
export DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"

pushd "${REPO_ROOT}/services/fabric-adapter" >/dev/null
go run ./cmd/fabric-adapter
popd >/dev/null
