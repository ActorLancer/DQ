#!/usr/bin/env bash
set -euo pipefail

OTEL_HEALTH_PORT="${OTEL_COLLECTOR_HEALTH_PORT:-13133}"
OTEL_METRICS_PORT="${OTEL_COLLECTOR_METRICS_PORT:-8889}"

for _ in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:${OTEL_HEALTH_PORT}/" >/dev/null 2>&1 \
    && curl -fsS "http://127.0.0.1:${OTEL_METRICS_PORT}/metrics" | head -n 1 >/dev/null 2>&1; then
    echo "[ok] otel collector health/metrics endpoints reachable"
    exit 0
  fi
  sleep 1
done

echo "[error] otel collector health/metrics endpoints unreachable after retries" >&2
exit 1
