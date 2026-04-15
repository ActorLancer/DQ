#!/usr/bin/env bash
set -euo pipefail

docker compose -f 部署脚本/docker-compose.local.yml down -v
echo "[done] local stack reset with volumes removed"
