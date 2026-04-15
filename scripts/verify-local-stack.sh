#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-core}"
ENV_FILE="${ENV_FILE:-.env}"

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

ok()   { echo "[ok]   $1"; }
warn() { echo "[warn] $1"; }
fail() { echo "[fail] $1"; exit 1; }

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Required command not found: $1"
}

check_tcp() {
  local name="$1"
  local host="$2"
  local port="$3"

  if command -v nc >/dev/null 2>&1; then
    if nc -z "${host}" "${port}" >/dev/null 2>&1; then
      ok "${name} is reachable on ${host}:${port} (nc)"
      return 0
    fi
  elif bash -c "exec 3<>/dev/tcp/${host}/${port}" >/dev/null 2>&1; then
    ok "${name} is reachable on ${host}:${port}"
    return 0
  fi
  fail "${name} is not reachable on ${host}:${port}"
}

check_docker_exec() {
  local name="$1"
  shift
  for _ in $(seq 1 10); do
    if docker exec "$@" >/dev/null 2>&1; then
      ok "${name} command probe passed"
      return 0
    fi
    sleep 1
  done
  fail "${name} command probe failed"
}

check_http() {
  local name="$1"
  local url="$2"
  local expected_regex="$3"

  local code=""
  for _ in $(seq 1 15); do
    code="$(curl -k -sS -o /dev/null -w '%{http_code}' "$url" || true)"
    if [[ "$code" =~ $expected_regex ]]; then
      ok "${name} responded at ${url} with HTTP ${code}"
      return 0
    fi
    sleep 1
  done
  fail "${name} check failed at ${url} with HTTP ${code}"
}

require_cmd bash
require_cmd docker
require_cmd curl

POSTGRES_PORT="${POSTGRES_PORT:-5432}"
REDIS_PORT="${REDIS_PORT:-6379}"
KAFKA_EXTERNAL_PORT="${KAFKA_EXTERNAL_PORT:-9094}"
MINIO_API_PORT="${MINIO_API_PORT:-9000}"
MINIO_CONSOLE_PORT="${MINIO_CONSOLE_PORT:-9001}"
OPENSEARCH_HTTP_PORT="${OPENSEARCH_HTTP_PORT:-9200}"
KEYCLOAK_PORT="${KEYCLOAK_PORT:-8081}"
PROMETHEUS_PORT="${PROMETHEUS_PORT:-9090}"
GRAFANA_PORT="${GRAFANA_PORT:-3000}"
LOKI_PORT="${LOKI_PORT:-3100}"
TEMPO_PORT="${TEMPO_PORT:-3200}"
MOCK_PAYMENT_PORT="${MOCK_PAYMENT_PORT:-8089}"
OTEL_COLLECTOR_HEALTH_PORT="${OTEL_COLLECTOR_HEALTH_PORT:-13133}"
OTEL_COLLECTOR_METRICS_PORT="${OTEL_COLLECTOR_METRICS_PORT:-8889}"
POSTGRES_USER="${POSTGRES_USER:-datab}"
POSTGRES_DB="${POSTGRES_DB:-datab}"
REDIS_PASSWORD="${REDIS_PASSWORD:-datab_redis_pass}"
MINIO_ROOT_USER="${MINIO_ROOT_USER:-datab}"
MINIO_ROOT_PASSWORD="${MINIO_ROOT_PASSWORD:-datab_local_pass}"
MINIO_MC_IMAGE="${MINIO_MC_IMAGE:-minio/mc:RELEASE.2025-08-13T08-35-41Z}"
KCAT_IMAGE="${KCAT_IMAGE:-edenhill/kcat:1.7.1}"

echo "[info] Verifying local stack mode: ${MODE}"
echo "[info] Using env file: ${ENV_FILE}"

# Core services
check_tcp "Postgres" "127.0.0.1" "${POSTGRES_PORT}"
check_tcp "Redis" "127.0.0.1" "${REDIS_PORT}"
check_tcp "Kafka external listener" "127.0.0.1" "${KAFKA_EXTERNAL_PORT}"
check_http "MinIO API" "http://127.0.0.1:${MINIO_API_PORT}/minio/health/live" '^(200)$'
check_http "MinIO Console" "http://127.0.0.1:${MINIO_CONSOLE_PORT}" '^(200|301|302|307|403)$'
check_http "OpenSearch" "http://127.0.0.1:${OPENSEARCH_HTTP_PORT}" '^(200|401|403)$'
check_http "Keycloak" "http://127.0.0.1:${KEYCLOAK_PORT}" '^(200|301|302)$'
check_http "OTel Collector Health" "http://127.0.0.1:${OTEL_COLLECTOR_HEALTH_PORT}/" '^(200)$'
check_http "OTel Collector Metrics" "http://127.0.0.1:${OTEL_COLLECTOR_METRICS_PORT}/metrics" '^(200)$'

# Command-level probes (curl/nc/psql/redis-cli/kcat-or-kafka-tools/mc)
check_docker_exec "Postgres psql" datab-postgres psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -c "select 1;"
check_docker_exec "Redis redis-cli" datab-redis sh -c "redis-cli -a '${REDIS_PASSWORD}' ping | grep -q PONG"

if docker exec datab-kafka sh -c "command -v kcat >/dev/null 2>&1"; then
  check_docker_exec "Kafka kcat metadata" datab-kafka kcat -b localhost:9092 -L
elif docker run --rm --network host "${KCAT_IMAGE}" -b "127.0.0.1:${KAFKA_EXTERNAL_PORT}" -L >/dev/null 2>&1; then
  ok "Kafka kcat metadata probe passed (ephemeral container)"
else
  warn "kcat unavailable in kafka container and ephemeral kcat probe failed; using kafka-topics metadata probe fallback"
  check_docker_exec "Kafka topic metadata fallback" datab-kafka /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list
fi

check_docker_exec "OpenSearch index API" datab-opensearch sh -c "curl -fsS http://127.0.0.1:9200/_cluster/health >/dev/null"
check_docker_exec "Keycloak container TCP probe" datab-keycloak bash -ec "exec 3<>/dev/tcp/127.0.0.1/8080"

if docker run --rm --network host \
  -e "MC_HOST_local=http://${MINIO_ROOT_USER}:${MINIO_ROOT_PASSWORD}@127.0.0.1:${MINIO_API_PORT}" \
  "${MINIO_MC_IMAGE}" ls "local/${BUCKET_RAW_DATA:-raw-data}" >/dev/null 2>&1; then
  ok "MinIO mc probe passed"
else
  fail "MinIO mc probe failed"
fi

case "${MODE}" in
  core)
    ;;
  observability|obs)
    check_http "Prometheus" "http://127.0.0.1:${PROMETHEUS_PORT}/-/healthy" '^(200)$'
    check_http "Grafana" "http://127.0.0.1:${GRAFANA_PORT}/api/health" '^(200)$'
    check_http "Loki" "http://127.0.0.1:${LOKI_PORT}/ready" '^(200)$'
    check_http "Tempo" "http://127.0.0.1:${TEMPO_PORT}/ready" '^(200)$'
    ;;
  mocks)
    check_http "Mock payment provider" "http://127.0.0.1:${MOCK_PAYMENT_PORT}/__admin/" '^(200)$'
    ;;
  full|all)
    check_http "Prometheus" "http://127.0.0.1:${PROMETHEUS_PORT}/-/healthy" '^(200)$'
    check_http "Grafana" "http://127.0.0.1:${GRAFANA_PORT}/api/health" '^(200)$'
    check_http "Loki" "http://127.0.0.1:${LOKI_PORT}/ready" '^(200)$'
    check_http "Tempo" "http://127.0.0.1:${TEMPO_PORT}/ready" '^(200)$'
    check_http "Mock payment provider" "http://127.0.0.1:${MOCK_PAYMENT_PORT}/__admin/" '^(200)$'
    ;;
  *)
    fail "Unknown mode: ${MODE}. Use one of: core, observability, obs, mocks, full"
    ;;
esac

ok "Verification completed successfully"
