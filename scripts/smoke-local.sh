#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
KAFKA_CONTAINER="${KAFKA_CONTAINER:-datab-kafka}"
MINIO_ALIAS="${MINIO_ALIAS:-local}"
MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://127.0.0.1:9000}"
MINIO_MC_IMAGE="${MINIO_MC_IMAGE:-minio/mc:RELEASE.2025-08-13T08-35-41Z}"
MOCK_BASE_URL="${MOCK_BASE_URL:-http://127.0.0.1:${MOCK_PAYMENT_PORT:-8089}}"
GRAFANA_URL="${GRAFANA_URL:-http://127.0.0.1:${GRAFANA_PORT:-3000}}"

if [[ -f "${ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${ENV_FILE}"
  set +a
fi

log() { echo "[info] $*"; }
ok() { echo "[ok]   $*"; }
fail() { echo "[fail] $*" >&2; exit 1; }

command -v docker >/dev/null 2>&1 || fail "docker not found"
command -v curl >/dev/null 2>&1 || fail "curl not found"
command -v psql >/dev/null 2>&1 || fail "psql not found"
command -v jq >/dev/null 2>&1 || fail "jq not found"

wait_http_ok() {
  local url="$1"
  local retries="${2:-60}"
  local code
  for _ in $(seq 1 "${retries}"); do
    code="$(curl -sS -o /dev/null -w '%{http_code}' "${url}" || true)"
    if [[ "${code}" == "200" || "${code}" == "302" ]]; then
      return 0
    fi
    sleep 1
  done
  return 1
}

smoke_db_migratable() {
  local db_host="${POSTGRES_HOST:-127.0.0.1}"
  local db_port="${POSTGRES_PORT:-5432}"
  local db_name="${POSTGRES_DB:-datab}"
  local db_user="${POSTGRES_USER:-datab}"
  local db_password="${POSTGRES_PASSWORD:-datab_local_pass}"
  export PGPASSWORD="${db_password}"

  psql -h "${db_host}" -p "${db_port}" -U "${db_user}" -d "${db_name}" -v ON_ERROR_STOP=1 <<'SQL' >/dev/null
BEGIN;
CREATE TABLE IF NOT EXISTS smoke_migration_probe (
  id BIGSERIAL PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE smoke_migration_probe ADD COLUMN IF NOT EXISTS note TEXT;
DROP TABLE smoke_migration_probe;
COMMIT;
SQL
  ok "database migration probe passed"
}

smoke_minio_buckets() {
  local access_key="${MINIO_ROOT_USER:-datab}"
  local secret_key="${MINIO_ROOT_PASSWORD:-datab_local_pass}"
  local host_value="${MINIO_ENDPOINT/http:\/\//http://${access_key}:${secret_key}@}"
  host_value="${host_value/https:\/\//https://${access_key}:${secret_key}@}"

  local buckets=(
    "${BUCKET_RAW_DATA:-raw-data}"
    "${BUCKET_PREVIEW_ARTIFACTS:-preview-artifacts}"
    "${BUCKET_DELIVERY_OBJECTS:-delivery-objects}"
    "${BUCKET_REPORT_RESULTS:-report-results}"
    "${BUCKET_EVIDENCE_PACKAGES:-evidence-packages}"
    "${BUCKET_MODEL_ARTIFACTS:-model-artifacts}"
  )

  for bucket in "${buckets[@]}"; do
    docker run --rm --network host \
      -e "MC_HOST_${MINIO_ALIAS}=${host_value}" \
      "${MINIO_MC_IMAGE}" ls "${MINIO_ALIAS}/${bucket}" >/dev/null
  done
  ok "minio buckets probe passed"
}

smoke_keycloak_realm() {
  ./scripts/check-keycloak-realm.sh >/dev/null
  ok "keycloak realm imported"
}

smoke_kafka_topics() {
  local topics=(
    "${TOPIC_OUTBOX_EVENTS:-outbox.events}"
    "${TOPIC_SEARCH_SYNC:-search.sync}"
    "${TOPIC_AUDIT_ANCHOR:-audit.anchor}"
    "${TOPIC_BILLING_EVENTS:-billing.events}"
    "${TOPIC_RECOMMENDATION_BEHAVIOR:-recommendation.behavior}"
    "${TOPIC_DEAD_LETTER_EVENTS:-dead-letter.events}"
  )

  local listed
  listed="$(docker exec "${KAFKA_CONTAINER}" /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list)"
  for topic in "${topics[@]}"; do
    grep -qx "${topic}" <<<"${listed}" || fail "topic missing: ${topic}"
  done
  ok "kafka topics probe passed"
}

smoke_grafana_login() {
  local user="${GRAFANA_ADMIN_USER:-admin}"
  local password="${GRAFANA_ADMIN_PASSWORD:-admin123456}"
  wait_http_ok "${GRAFANA_URL}/api/health" 60 || fail "grafana health check failed"
  curl -fsS -u "${user}:${password}" "${GRAFANA_URL}/api/user" \
    | jq -e --arg login "${user}" '.login == $login or .isGrafanaAdmin == true' >/dev/null
  ok "grafana login probe passed"
}

smoke_mock_payment_callback() {
  wait_http_ok "${MOCK_BASE_URL}/health/ready" 60 || fail "mock payment readiness probe failed"
  ./scripts/check-mock-payment.sh >/dev/null
  ok "mock payment callback probe passed"
}

log "Running local smoke suite (ENV-040)"
smoke_db_migratable
smoke_minio_buckets
smoke_keycloak_realm
smoke_kafka_topics
smoke_grafana_login
smoke_mock_payment_callback
ok "smoke suite passed"
