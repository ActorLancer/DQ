#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${REPO_ROOT}/scripts/fabric-env.sh"

if [[ -d "${FABRIC_TEST_NETWORK_ROOT}" ]]; then
  pushd "${FABRIC_TEST_NETWORK_ROOT}" >/dev/null
  ./network.sh down >/dev/null 2>&1 || true
  rm -rf organizations system-genesis-block channel-artifacts
  popd >/dev/null
fi

rm -rf "${FABRIC_STATE_DIR}"
mkdir -p "${FABRIC_STATE_DIR}"
echo "[done] fabric local test-network state reset"
