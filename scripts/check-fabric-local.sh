#!/usr/bin/env bash
set -euo pipefail

docker ps --format '{{.Names}}' | rg '^datab-fabric-ca$' >/dev/null
docker ps --format '{{.Names}}' | rg '^datab-fabric-orderer$' >/dev/null
docker ps --format '{{.Names}}' | rg '^datab-fabric-peer$' >/dev/null
test -f "infra/fabric/state/${FABRIC_CHANNEL_NAME:-datab-channel}.tx" || test -f "infra/fabric/state/channel/${FABRIC_CHANNEL_NAME:-datab-channel}.tx"
test -f "infra/fabric/state/chaincode/contracts.json"
echo "[ok] fabric local runtime artifacts verified"
