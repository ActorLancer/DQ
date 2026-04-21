#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_ENV_FILE="${COMPOSE_ENV_FILE:-infra/docker/.env.local}"

docker compose --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" down -v
echo "[done] local stack reset with volumes removed"
