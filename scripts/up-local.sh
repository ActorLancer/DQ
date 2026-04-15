#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_ENV_FILE="${COMPOSE_ENV_FILE:-infra/docker/.env.local}"
COMPOSE_PROFILES="${COMPOSE_PROFILES:-core}"

./scripts/bootstrap.sh "${COMPOSE_FILE}" "${COMPOSE_ENV_FILE}"
export COMPOSE_PROFILES
docker compose --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" up -d
echo "[done] local stack started (profiles=${COMPOSE_PROFILES})"
