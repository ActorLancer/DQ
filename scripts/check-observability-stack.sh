#!/usr/bin/env bash
set -euo pipefail

PROM_URL="${PROM_URL:-http://127.0.0.1:${PROMETHEUS_PORT:-9090}}"
GRAFANA_URL="${GRAFANA_URL:-http://127.0.0.1:${GRAFANA_PORT:-3000}}"
ALERTMANAGER_URL="${ALERTMANAGER_URL:-http://127.0.0.1:${ALERTMANAGER_PORT:-9093}}"
GRAFANA_USER="${GRAFANA_ADMIN_USER:-admin}"
GRAFANA_PASSWORD="${GRAFANA_ADMIN_PASSWORD:-admin123456}"

for _ in $(seq 1 40); do
  if curl -fsS "${PROM_URL}/-/ready" >/dev/null 2>&1 \
    && curl -fsS "${ALERTMANAGER_URL}/-/ready" >/dev/null 2>&1 \
    && curl -fsS "${GRAFANA_URL}/api/health" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

required_jobs=(
  platform-core
  mock-payment-provider
  kafka-exporter
  postgres-exporter
  redis-exporter
  minio-exporter
  opensearch-exporter
)

for job in "${required_jobs[@]}"; do
  curl -fsS "${PROM_URL}/api/v1/targets" \
    | jq -e --arg job "${job}" '.data.activeTargets[] | select(.labels.job == $job)' >/dev/null
done

required_alerts=(
  ServiceDown
  KafkaConsumerLagBacklog
  DBConnectionFailure
  ChainAdapterFailure
  OutboxRetryAnomaly
  DLQGrowth
)

for alert in "${required_alerts[@]}"; do
  curl -fsS "${PROM_URL}/api/v1/rules" \
    | jq -e --arg alert "${alert}" '.data.groups[].rules[] | select(.name == $alert)' >/dev/null
done

required_datasources=(Prometheus Loki Tempo)
for ds in "${required_datasources[@]}"; do
  curl -fsS -u "${GRAFANA_USER}:${GRAFANA_PASSWORD}" "${GRAFANA_URL}/api/datasources" \
    | jq -e --arg ds "${ds}" '.[] | select(.name == $ds)' >/dev/null
done

required_dashboards=(
  "Platform Overview"
  "Database Overview"
  "Kafka Overview"
  "Application Tracing"
)

for db in "${required_dashboards[@]}"; do
  curl -fsS -u "${GRAFANA_USER}:${GRAFANA_PASSWORD}" --get --data-urlencode "query=${db}" "${GRAFANA_URL}/api/search" \
    | jq -e --arg db "${db}" '.[] | select(.title == $db)' >/dev/null
done

echo "[ok] observability stack targets/rules/datasources/dashboards verified"
