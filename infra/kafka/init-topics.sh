#!/usr/bin/env bash
set -euo pipefail

KAFKA_EXEC_MODE="${KAFKA_EXEC_MODE:-docker}"
KAFKA_CONTAINER="${KAFKA_CONTAINER:-datab-kafka}"
KAFKA_BOOTSTRAP="${KAFKA_BOOTSTRAP:-}"
KAFKA_BIN_DIR="${KAFKA_BIN_DIR:-/opt/kafka/bin}"
KAFKA_DEFAULT_RETENTION_MS="${KAFKA_DEFAULT_RETENTION_MS:-604800000}"
KAFKA_DLQ_RETENTION_MS="${KAFKA_DLQ_RETENTION_MS:-1209600000}"
KAFKA_DEFAULT_CLEANUP_POLICY="${KAFKA_DEFAULT_CLEANUP_POLICY:-delete}"
KAFKA_DLQ_CLEANUP_POLICY="${KAFKA_DLQ_CLEANUP_POLICY:-delete}"
KAFKA_WAIT_FOR_READY="${KAFKA_WAIT_FOR_READY:-false}"
KAFKA_INIT_MAX_RETRIES="${KAFKA_INIT_MAX_RETRIES:-30}"
KAFKA_INIT_RETRY_DELAY_SECONDS="${KAFKA_INIT_RETRY_DELAY_SECONDS:-2}"
TOPIC_CATALOG="${TOPIC_CATALOG:-infra/kafka/topics.v1.json}"

if [[ -z "${KAFKA_BOOTSTRAP}" ]]; then
  if [[ "${KAFKA_EXEC_MODE}" == "local" ]]; then
    KAFKA_BOOTSTRAP="kafka:9092"
  else
    KAFKA_BOOTSTRAP="localhost:9092"
  fi
fi

command -v jq >/dev/null 2>&1 || {
  echo "[fail] jq not found" >&2
  exit 1
}

if [[ "${KAFKA_EXEC_MODE}" == "docker" ]]; then
  command -v docker >/dev/null 2>&1 || {
    echo "[fail] docker not found for KAFKA_EXEC_MODE=docker" >&2
    exit 1
  }
else
  [[ -x "${KAFKA_BIN_DIR}/kafka-topics.sh" ]] || {
    echo "[fail] kafka-topics.sh not found under ${KAFKA_BIN_DIR}" >&2
    exit 1
  }
  [[ -x "${KAFKA_BIN_DIR}/kafka-configs.sh" ]] || {
    echo "[fail] kafka-configs.sh not found under ${KAFKA_BIN_DIR}" >&2
    exit 1
  }
fi

[[ -f "${TOPIC_CATALOG}" ]] || {
  echo "[fail] topic catalog not found: ${TOPIC_CATALOG}" >&2
  exit 1
}

resolve_topic_name() {
  local env_key="$1"
  local default_name="$2"
  local resolved="${!env_key:-${default_name}}"
  if [[ -z "${resolved}" ]]; then
    echo "[fail] resolved topic is empty for ${env_key}" >&2
    exit 1
  fi
  printf '%s\n' "${resolved}"
}

run_kafka_admin() {
  local tool="$1"
  shift

  if [[ "${KAFKA_EXEC_MODE}" == "docker" ]]; then
    docker exec "${KAFKA_CONTAINER}" "${KAFKA_BIN_DIR}/${tool}" "$@"
    return
  fi

  "${KAFKA_BIN_DIR}/${tool}" "$@"
}

wait_for_kafka_ready() {
  if [[ "${KAFKA_WAIT_FOR_READY}" != "true" ]]; then
    return 0
  fi

  local attempt
  for attempt in $(seq 1 "${KAFKA_INIT_MAX_RETRIES}"); do
    if run_kafka_admin kafka-topics.sh --bootstrap-server "${KAFKA_BOOTSTRAP}" --list >/dev/null 2>&1; then
      echo "[ok] kafka ready: ${KAFKA_BOOTSTRAP}"
      return 0
    fi
    sleep "${KAFKA_INIT_RETRY_DELAY_SECONDS}"
  done

  echo "[fail] kafka not ready after ${KAFKA_INIT_MAX_RETRIES} attempts: ${KAFKA_BOOTSTRAP}" >&2
  exit 1
}

wait_for_kafka_ready

while IFS= read -r topic_entry; do
  env_key="$(jq -r '.env_key' <<<"${topic_entry}")"
  default_name="$(jq -r '.name' <<<"${topic_entry}")"
  partitions="$(jq -r '.partitions // 3' <<<"${topic_entry}")"
  retention_ms="$(jq -r '.retention_ms // empty' <<<"${topic_entry}")"
  cleanup_policy="$(jq -r '.cleanup_policy // empty' <<<"${topic_entry}")"
  topic="$(resolve_topic_name "${env_key}" "${default_name}")"
  retention_ms="${retention_ms:-${KAFKA_DEFAULT_RETENTION_MS}}"
  cleanup_policy="${cleanup_policy:-${KAFKA_DEFAULT_CLEANUP_POLICY}}"
  if [[ "${topic}" == "${TOPIC_DEAD_LETTER_EVENTS:-dtp.dead-letter}" ]]; then
    retention_ms="${retention_ms:-${KAFKA_DLQ_RETENTION_MS}}"
    cleanup_policy="${cleanup_policy:-${KAFKA_DLQ_CLEANUP_POLICY}}"
  fi

  run_kafka_admin kafka-topics.sh \
    --bootstrap-server "${KAFKA_BOOTSTRAP}" \
    --create --if-not-exists \
    --topic "${topic}" \
    --partitions "${partitions}" \
    --replication-factor 1 >/dev/null

  run_kafka_admin kafka-configs.sh \
    --bootstrap-server "${KAFKA_BOOTSTRAP}" \
    --entity-type topics \
    --entity-name "${topic}" \
    --alter \
    --add-config "retention.ms=${retention_ms},cleanup.policy=${cleanup_policy}" >/dev/null

  echo "[ok] topic ready: ${topic}"
done < <(jq -c '.topics[]' "${TOPIC_CATALOG}")

echo "[done] kafka topics initialized"
