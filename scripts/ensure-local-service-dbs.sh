#!/usr/bin/env bash
set -euo pipefail

COMPOSE_ENV_FILE="${1:-infra/docker/.env.local}"
POSTGRES_CONTAINER="${POSTGRES_CONTAINER:-datab-postgres}"

if [[ -f "${COMPOSE_ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${COMPOSE_ENV_FILE}"
  set +a
fi

POSTGRES_USER="${POSTGRES_USER:-datab}"
KEYCLOAK_DB_NAME="${KEYCLOAK_DB_NAME:-keycloak}"

existing_db="$(
  docker exec "${POSTGRES_CONTAINER}" \
    psql -U "${POSTGRES_USER}" -d postgres -tAc \
    "SELECT datname FROM pg_database WHERE datname = '${KEYCLOAK_DB_NAME}'"
)"

if [[ -z "${existing_db}" ]]; then
  docker exec "${POSTGRES_CONTAINER}" \
    psql -U "${POSTGRES_USER}" -d postgres -v ON_ERROR_STOP=1 \
    -c "CREATE DATABASE \"${KEYCLOAK_DB_NAME}\";"
fi

docker exec "${POSTGRES_CONTAINER}" \
  psql -U "${POSTGRES_USER}" -d postgres -v ON_ERROR_STOP=1 \
  -c "ALTER DATABASE \"${KEYCLOAK_DB_NAME}\" SET timezone TO 'Asia/Shanghai';"

echo "[ok] ensured service databases: ${KEYCLOAK_DB_NAME}"
