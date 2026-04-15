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

  if bash -c "exec 3<>/dev/tcp/${host}/${port}" >/dev/null 2>&1; then
    ok "${name} is reachable on ${host}:${port}"
  else
    fail "${name} is not reachable on ${host}:${port}"
  fi
}

check_http() {
  local name="$1"
  local url="$2"
  local expected_regex="$3"

  local code
  code="$(curl -k -sS -o /dev/null -w '%{http_code}' "$url" || true)"
  if [[ "$code" =~ $expected_regex ]]; then
    ok "${name} responded at ${url} with HTTP ${code}"
  else
    fail "${name} check failed at ${url} with HTTP ${code}"
  fi
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
    check_http "Mock payment provider" "http://127.0.0.1:${MOCK_PAYMENT_PORT}/__admin" '^(200)$'
    ;;
  full|all)
    check_http "Prometheus" "http://127.0.0.1:${PROMETHEUS_PORT}/-/healthy" '^(200)$'
    check_http "Grafana" "http://127.0.0.1:${GRAFANA_PORT}/api/health" '^(200)$'
    check_http "Loki" "http://127.0.0.1:${LOKI_PORT}/ready" '^(200)$'
    check_http "Tempo" "http://127.0.0.1:${TEMPO_PORT}/ready" '^(200)$'
    check_http "Mock payment provider" "http://127.0.0.1:${MOCK_PAYMENT_PORT}/__admin" '^(200)$'
    ;;
  *)
    fail "Unknown mode: ${MODE}. Use one of: core, observability, obs, mocks, full"
    ;;
esac

ok "Verification completed successfully"
