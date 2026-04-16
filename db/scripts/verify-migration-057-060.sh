#!/usr/bin/env bash
set -euo pipefail

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-55432}"
DB_NAME="${DB_NAME:-luna_data_trading}"
DB_USER="${DB_USER:-luna}"
DB_PASSWORD="${DB_PASSWORD:-5686}"

export PGPASSWORD="${DB_PASSWORD}"
PSQL=(psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 -X -q -tA)

required_tables=(
  search.seller_search_document
  search.search_signal_aggregate
  search.ranking_profile
  search.index_alias_binding
  search.index_sync_task
  recommend.placement_definition
  recommend.ranking_profile
  recommend.behavior_event
  recommend.recommendation_request
  recommend.recommendation_result
  recommend.recommendation_result_item
  ops.observability_backend
  ops.log_retention_policy
  ops.trace_index
  ops.alert_rule
  ops.alert_event
  ops.incident_ticket
  ops.slo_definition
  ops.slo_snapshot
)

required_indexes=(
  search.idx_index_sync_task_status
  search.idx_product_search_document_sync_status
  recommend.idx_behavior_event_subject_time
  recommend.idx_recommend_result_item_entity
  ops.idx_system_log_service_level
  ops.idx_trace_index_trace
  ops.idx_alert_event_status
  ops.idx_incident_ticket_status
  ops.idx_slo_snapshot_def
)

required_triggers=(
  catalog.trg_product_search_refresh
  search.trg_search_signal_refresh
  search.trg_index_sync_task_updated_at
  recommend.trg_recommend_behavior_event_outbox
  recommend.trg_recommend_placement_updated_at
  ops.trg_alert_rule_updated_at
  ops.trg_slo_definition_updated_at
)

required_seed_roles=(
  platform_admin
  tenant_admin
  seller_operator
  buyer_operator
  platform_audit_security
)

required_seed_permissions=(
  payment.intent.create
  delivery.file.download
  dispute.case.resolve
  audit.anchor.manage
  ops.search_reindex.execute
)

check_table() {
  local fq_table="$1"
  local schema="${fq_table%%.*}"
  local table="${fq_table#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema='${schema}' AND table_name='${table}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] table missing: ${fq_table}" >&2
    exit 1
  fi
}

check_index() {
  local fq_index="$1"
  local schema="${fq_index%%.*}"
  local index="${fq_index#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname='${schema}' AND indexname='${index}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] index missing: ${fq_index}" >&2
    exit 1
  fi
}

check_trigger() {
  local fq_trigger="$1"
  local schema="${fq_trigger%%.*}"
  local trigger="${fq_trigger#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM information_schema.triggers WHERE trigger_schema='${schema}' AND trigger_name='${trigger}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] trigger missing: ${fq_trigger}" >&2
    exit 1
  fi
}

check_role_seed() {
  local role_key="$1"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM authz.role_definition WHERE role_key='${role_key}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] role seed missing: authz.role_definition.${role_key}" >&2
    exit 1
  fi
}

check_permission_seed() {
  local permission_code="$1"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM authz.permission_definition WHERE permission_code='${permission_code}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] permission seed missing: authz.permission_definition.${permission_code}" >&2
    exit 1
  fi
}

check_min_counts() {
  local role_count
  local permission_count
  role_count="$("${PSQL[@]}" -c "SELECT COUNT(*) FROM authz.role_definition;")"
  permission_count="$("${PSQL[@]}" -c "SELECT COUNT(*) FROM authz.permission_definition;")"
  if (( role_count < 12 )); then
    echo "[fail] role seed count too small: ${role_count} (< 12)" >&2
    exit 1
  fi
  if (( permission_count < 200 )); then
    echo "[fail] permission seed count too small: ${permission_count} (< 200)" >&2
    exit 1
  fi
}

for table in "${required_tables[@]}"; do
  check_table "${table}"
done

for index in "${required_indexes[@]}"; do
  check_index "${index}"
done

for trigger in "${required_triggers[@]}"; do
  check_trigger "${trigger}"
done

for role_key in "${required_seed_roles[@]}"; do
  check_role_seed "${role_key}"
done

for permission_code in "${required_seed_permissions[@]}"; do
  check_permission_seed "${permission_code}"
done

check_min_counts

echo "[ok] migrations 057/058/059/060 baseline verified"
