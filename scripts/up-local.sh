#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_ENV_FILE="${COMPOSE_ENV_FILE:-infra/docker/.env.local}"

./scripts/bootstrap.sh "${COMPOSE_FILE}" "${COMPOSE_ENV_FILE}"
docker compose --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" up -d
echo "[done] local stack started"
