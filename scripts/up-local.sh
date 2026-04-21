#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_ENV_FILE="${COMPOSE_ENV_FILE:-infra/docker/.env.local}"
COMPOSE_PROFILES="${COMPOSE_PROFILES:-core}"

./scripts/bootstrap.sh "${COMPOSE_FILE}" "${COMPOSE_ENV_FILE}"
if [[ -f "${COMPOSE_ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${COMPOSE_ENV_FILE}"
  set +a
fi
export COMPOSE_PROFILES

if [[ ",${COMPOSE_PROFILES}," == *",core,"* || ",${COMPOSE_PROFILES}," == *",demo,"* ]]; then
  docker compose --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" up -d postgres
  until docker exec datab-postgres pg_isready -U "${POSTGRES_USER:-datab}" -d postgres >/dev/null 2>&1; do
    sleep 1
  done
  ./scripts/ensure-local-service-dbs.sh "${COMPOSE_ENV_FILE}"
fi

docker compose --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" up -d

if [[ ",${COMPOSE_PROFILES}," == *",core,"* || ",${COMPOSE_PROFILES}," == *",demo,"* ]]; then
  compose_profile_args=()
  IFS=',' read -r -a active_profiles <<<"${COMPOSE_PROFILES}"
  for profile in "${active_profiles[@]}"; do
    [[ -n "${profile}" ]] || continue
    compose_profile_args+=(--profile "${profile}")
  done
  compose_profile_args+=(--profile topic-init)
  docker compose "${compose_profile_args[@]}" --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" run --rm kafka-topics-init
fi

echo "[done] local stack started (profiles=${COMPOSE_PROFILES})"
