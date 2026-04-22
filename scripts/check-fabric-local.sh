#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
source "${REPO_ROOT}/scripts/fabric-env.sh"

for container_name in ca_org1 ca_org2 ca_orderer orderer.example.com peer0.org1.example.com peer0.org2.example.com; do
  docker ps --format '{{.Names}}' | rg "^${container_name}$" >/dev/null
done

fabric_use_org1_admin

peer channel getinfo -c "${FABRIC_CHANNEL_NAME}" >/dev/null
peer lifecycle chaincode querycommitted -C "${FABRIC_CHANNEL_NAME}" --name "${FABRIC_CHAINCODE_NAME}" \
  | rg "Version: ${FABRIC_CHAINCODE_VERSION}, Sequence: ${FABRIC_CHAINCODE_SEQUENCE}" >/dev/null

ping_result="$(peer chaincode query -C "${FABRIC_CHANNEL_NAME}" -n "${FABRIC_CHAINCODE_NAME}" -c '{"function":"Ping","Args":[]}' | tr -d '\r')"
[[ "${ping_result}" == "ok" ]]

if [[ -f "${FABRIC_STATE_DIR}/runtime.env" ]]; then
  source "${FABRIC_STATE_DIR}/runtime.env"
fi

echo "[ok] fabric local test-network runtime verified"
