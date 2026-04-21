#!/usr/bin/env bash
set -euo pipefail

docker compose -f infra/fabric/docker-compose.fabric.local.yml down -v --remove-orphans
docker rm -f datab-fabric-ca datab-fabric-orderer datab-fabric-peer >/dev/null 2>&1 || true
rm -rf infra/fabric/state
mkdir -p infra/fabric/state/{ca,orderer,peer,channel,chaincode}
echo "[done] fabric local network state reset"
