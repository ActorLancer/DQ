#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/部署脚本/docker-compose.postgres-test.yml"
COMPOSE_PROJECT="luna_db_test"
DB_HOST="127.0.0.1"
DB_PORT="55432"
DB_NAME="luna_data_trading"
DB_USER="luna"
DB_PASSWORD="5686"

export PGPASSWORD="$DB_PASSWORD"

wait_for_db() {
  local retries=60
  until psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "select version();" >/dev/null 2>&1; do
    retries=$((retries - 1))
    if [ "$retries" -le 0 ]; then
      echo "database is not ready"
      exit 1
    fi
    sleep 2
  done
}

reset_db() {
  psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -v ON_ERROR_STOP=1 -c "DROP DATABASE IF EXISTS $DB_NAME;"
  psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE $DB_NAME;"
}

run_dir() {
  local dir="$1"
  for file in "$dir"/*.sql; do
    [ -f "$file" ] || continue
    echo "==> running $file"
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -f "$file"
  done
}

echo "==> starting postgres test container"
docker-compose -p "$COMPOSE_PROJECT" -f "$COMPOSE_FILE" down -v >/dev/null 2>&1 || true
docker-compose -p "$COMPOSE_PROJECT" -f "$COMPOSE_FILE" up -d

echo "==> waiting for postgres"
wait_for_db

echo "==> reset database"
reset_db

echo "==> run V1 upgrade"
run_dir "$ROOT_DIR/数据库设计/V1/upgrade"

echo "==> run V1 downgrade"
for file in $(ls "$ROOT_DIR/数据库设计/V1/downgrade"/*.sql | sort -r); do
  echo "==> running $file"
  psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -f "$file"
done

echo "==> reset database"
reset_db

echo "==> run V1+V2 upgrade"
run_dir "$ROOT_DIR/数据库设计/V1/upgrade"
run_dir "$ROOT_DIR/数据库设计/V2/upgrade"

echo "==> run V2 downgrade"
for file in $(ls "$ROOT_DIR/数据库设计/V2/downgrade"/*.sql | sort -r); do
  echo "==> running $file"
  psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -f "$file"
done

echo "==> run V1 downgrade"
for file in $(ls "$ROOT_DIR/数据库设计/V1/downgrade"/*.sql | sort -r); do
  echo "==> running $file"
  psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -f "$file"
done

echo "==> reset database"
reset_db

echo "==> run V1+V2+V3 upgrade"
run_dir "$ROOT_DIR/数据库设计/V1/upgrade"
run_dir "$ROOT_DIR/数据库设计/V2/upgrade"
run_dir "$ROOT_DIR/数据库设计/V3/upgrade"

echo "==> run V3 downgrade"
for file in $(ls "$ROOT_DIR/数据库设计/V3/downgrade"/*.sql | sort -r); do
  echo "==> running $file"
  psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -f "$file"
done

echo "==> run V2 downgrade"
for file in $(ls "$ROOT_DIR/数据库设计/V2/downgrade"/*.sql | sort -r); do
  echo "==> running $file"
  psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -f "$file"
done

echo "==> run V1 downgrade"
for file in $(ls "$ROOT_DIR/数据库设计/V1/downgrade"/*.sql | sort -r); do
  echo "==> running $file"
  psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -f "$file"
done

echo "==> migration validation completed"
