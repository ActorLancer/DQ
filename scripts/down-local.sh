#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_ENV_FILE="${COMPOSE_ENV_FILE:-infra/docker/.env.local}"

compose_profile_args=()
for profile in core observability mocks fabric demo topic-init; do
  compose_profile_args+=(--profile "${profile}")
done

docker compose "${compose_profile_args[@]}" --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" down --remove-orphans
echo "[done] local stack stopped"
