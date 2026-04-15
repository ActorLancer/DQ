#!/usr/bin/env bash
set -euo pipefail

docker compose -f infra/fabric/docker-compose.fabric.local.yml down
echo "[done] fabric local network stopped"
