#!/usr/bin/env bash
set -euo pipefail

KAFKA_CONTAINER="${KAFKA_CONTAINER:-datab-kafka}"
KAFKA_BOOTSTRAP="${KAFKA_BOOTSTRAP:-localhost:9092}"
KAFKA_DEFAULT_RETENTION_MS="${KAFKA_DEFAULT_RETENTION_MS:-604800000}"
KAFKA_DLQ_RETENTION_MS="${KAFKA_DLQ_RETENTION_MS:-1209600000}"
KAFKA_DEFAULT_CLEANUP_POLICY="${KAFKA_DEFAULT_CLEANUP_POLICY:-delete}"
KAFKA_DLQ_CLEANUP_POLICY="${KAFKA_DLQ_CLEANUP_POLICY:-delete}"

TOPICS=(
  "${TOPIC_OUTBOX_EVENTS:-outbox.events}"
  "${TOPIC_SEARCH_SYNC:-search.sync}"
  "${TOPIC_AUDIT_ANCHOR:-audit.anchor}"
  "${TOPIC_BILLING_EVENTS:-billing.events}"
  "${TOPIC_RECOMMENDATION_BEHAVIOR:-recommendation.behavior}"
  "${TOPIC_DEAD_LETTER_EVENTS:-dead-letter.events}"
)

for topic in "${TOPICS[@]}"; do
  retention_ms="${KAFKA_DEFAULT_RETENTION_MS}"
  cleanup_policy="${KAFKA_DEFAULT_CLEANUP_POLICY}"
  if [[ "${topic}" == "${TOPIC_DEAD_LETTER_EVENTS:-dead-letter.events}" ]]; then
    retention_ms="${KAFKA_DLQ_RETENTION_MS}"
    cleanup_policy="${KAFKA_DLQ_CLEANUP_POLICY}"
  fi

  docker exec "${KAFKA_CONTAINER}" /opt/kafka/bin/kafka-topics.sh \
    --bootstrap-server "${KAFKA_BOOTSTRAP}" \
    --create --if-not-exists \
    --topic "${topic}" \
    --partitions 3 \
    --replication-factor 1 >/dev/null

  docker exec "${KAFKA_CONTAINER}" /opt/kafka/bin/kafka-configs.sh \
    --bootstrap-server "${KAFKA_BOOTSTRAP}" \
    --entity-type topics \
    --entity-name "${topic}" \
    --alter \
    --add-config "retention.ms=${retention_ms},cleanup.policy=${cleanup_policy}" >/dev/null

  echo "[ok] topic ready: ${topic}"
done

echo "[done] kafka topics initialized"
