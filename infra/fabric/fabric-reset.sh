#!/usr/bin/env bash
set -euo pipefail

docker compose -f infra/fabric/docker-compose.fabric.local.yml down -v
rm -rf infra/fabric/state
mkdir -p infra/fabric/state/{ca,orderer,peer,channel,chaincode}
echo "[done] fabric local network state reset"
