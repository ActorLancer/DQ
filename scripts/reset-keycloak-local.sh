#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_ENV_FILE="${COMPOSE_ENV_FILE:-infra/docker/.env.local}"
KEYCLOAK_SERVICE="${KEYCLOAK_SERVICE:-keycloak}"
POSTGRES_CONTAINER="${POSTGRES_CONTAINER:-datab-postgres}"

if [[ -f "${COMPOSE_ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${COMPOSE_ENV_FILE}"
  set +a
fi

POSTGRES_USER="${POSTGRES_USER:-datab}"
KEYCLOAK_DB_NAME="${KEYCLOAK_DB_NAME:-keycloak}"

compose() {
  docker compose --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" "$@"
}

echo "[info] stopping local keycloak service"
compose stop "${KEYCLOAK_SERVICE}" >/dev/null

echo "[info] resetting keycloak database: ${KEYCLOAK_DB_NAME}"
docker exec "${POSTGRES_CONTAINER}" \
  psql -U "${POSTGRES_USER}" -d postgres -v ON_ERROR_STOP=1 <<SQL
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = '${KEYCLOAK_DB_NAME}'
  AND pid <> pg_backend_pid();
DROP DATABASE IF EXISTS "${KEYCLOAK_DB_NAME}";
CREATE DATABASE "${KEYCLOAK_DB_NAME}";
ALTER DATABASE "${KEYCLOAK_DB_NAME}" SET timezone TO 'Asia/Shanghai';
SQL

echo "[info] restarting local keycloak service"
compose up -d "${KEYCLOAK_SERVICE}" >/dev/null

for _ in $(seq 1 60); do
  if ./scripts/check-keycloak-realm.sh >/dev/null 2>&1; then
    echo "[ok] local keycloak realm reset and verified"
    exit 0
  fi
  sleep 1
done

echo "[fail] local keycloak realm did not become ready after reset" >&2
docker logs datab-keycloak --tail 120 >&2 || true
exit 1
