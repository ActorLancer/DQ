#!/usr/bin/env bash
set -euo pipefail

docker compose -f infra/fabric/docker-compose.fabric.local.yml down --remove-orphans >/dev/null 2>&1 || true
docker rm -f datab-fabric-ca datab-fabric-orderer datab-fabric-peer >/dev/null 2>&1 || true
docker compose -f infra/fabric/docker-compose.fabric.local.yml up -d
echo "[done] fabric local network started"
