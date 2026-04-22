#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${REPO_ROOT}/scripts/fabric-env.sh"

"${REPO_ROOT}/infra/fabric/install-deps.sh"

if [[ ! -d "${FABRIC_TEST_NETWORK_ROOT}" ]]; then
  echo "fabric test-network root not found: ${FABRIC_TEST_NETWORK_ROOT}" >&2
  exit 1
fi

pushd "${FABRIC_TEST_NETWORK_ROOT}" >/dev/null
./network.sh createChannel -ca -c "${FABRIC_CHANNEL_NAME}"
popd >/dev/null

"${REPO_ROOT}/infra/fabric/deploy-chaincode.sh"
echo "[done] fabric channel ensured: ${FABRIC_CHANNEL_NAME}"
