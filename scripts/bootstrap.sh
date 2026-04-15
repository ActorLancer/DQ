#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${1:-${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}}"
COMPOSE_ENV_FILE="${2:-${COMPOSE_ENV_FILE:-infra/docker/.env.local}}"

./scripts/check-local-env.sh "${COMPOSE_FILE}" "${COMPOSE_ENV_FILE}" "${COMPOSE_ENV_FILE}" runtime
echo "[done] bootstrap checks finished"
