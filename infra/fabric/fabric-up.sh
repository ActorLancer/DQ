#!/usr/bin/env bash
set -euo pipefail

docker compose -f infra/fabric/docker-compose.fabric.local.yml up -d
echo "[done] fabric local network started"
