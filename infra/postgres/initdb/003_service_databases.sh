#!/usr/bin/env bash
set -euo pipefail

keycloak_db="${KEYCLOAK_DB_NAME:-keycloak}"

existing_db="$(psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname postgres -tAc "SELECT datname FROM pg_database WHERE datname = '${keycloak_db}'")"
if [[ -z "${existing_db}" ]]; then
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname postgres -c "CREATE DATABASE \"${keycloak_db}\";"
fi

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname postgres -c "ALTER DATABASE \"${keycloak_db}\" SET timezone TO 'Asia/Shanghai';"
