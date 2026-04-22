#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${REPO_ROOT}/scripts/fabric-env.sh"

if [[ -d "${FABRIC_TEST_NETWORK_ROOT}" ]]; then
  pushd "${FABRIC_TEST_NETWORK_ROOT}" >/dev/null
  ./network.sh down
  popd >/dev/null
fi

rm -f "${FABRIC_STATE_DIR}/runtime.env"
echo "[done] fabric local test-network stopped"
