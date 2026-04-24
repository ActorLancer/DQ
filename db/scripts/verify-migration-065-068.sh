#!/usr/bin/env bash
set -euo pipefail

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"

export PGPASSWORD="${DB_PASSWORD}"
PSQL=(psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 -X -q -tA)

required_tables=(
  catalog.query_surface_definition
  delivery.query_template_definition
  delivery.query_execution_run
  catalog.sensitive_handling_policy
  contract.legal_basis_evidence
  catalog.safe_preview_artifact
  delivery.sensitive_execution_policy
  delivery.attestation_record
  delivery.result_disclosure_review
  delivery.destruction_attestation
  ops.monitoring_policy_profile
  ops.trade_lifecycle_checkpoint
  ops.external_fact_receipt
  risk.fairness_incident
  ops.chain_projection_gap
)

required_indexes=(
  catalog.idx_query_surface_asset_version
  delivery.idx_query_template_surface
  delivery.idx_query_execution_run_order
  catalog.idx_sensitive_handling_policy_asset_version
  delivery.idx_sensitive_execution_policy_order
  ops.idx_trade_lifecycle_checkpoint_order
  ops.idx_external_fact_receipt_order
  risk.idx_fairness_incident_order
  ops.idx_chain_projection_gap_status
)

required_triggers=(
  catalog.trg_query_surface_definition_updated_at
  delivery.trg_query_execution_run_updated_at
  catalog.trg_sensitive_handling_policy_updated_at
  delivery.trg_result_disclosure_review_updated_at
  ops.trg_monitoring_policy_profile_updated_at
  risk.trg_fairness_incident_updated_at
  ops.trg_chain_projection_gap_updated_at
)

required_columns=(
  "delivery.template_query_grant.query_surface_id"
  "delivery.template_query_grant.allowed_template_ids"
  "delivery.sandbox_workspace.query_surface_id"
  "delivery.sandbox_workspace.clean_room_mode"
  "delivery.query_execution_run.masked_level"
  "delivery.query_execution_run.sensitive_policy_snapshot"
  "delivery.delivery_record.sensitive_delivery_mode"
  "delivery.api_credential.sensitive_scope_snapshot"
  "catalog.data_asset.contains_spi"
)

required_permissions=(
  ops.trade_monitor.read
  ops.trade_monitor.manage
  ops.external_fact.read
  ops.external_fact.manage
  ops.monitor_policy.read
  ops.monitor_policy.manage
  ops.projection_gap.read
  ops.projection_gap.manage
  risk.fairness_incident.read
  risk.fairness_incident.handle
)

required_role_permissions=(
  "platform_admin|ops.trade_monitor.manage"
  "platform_audit_security|ops.trade_monitor.read"
  "platform_risk_settlement|risk.fairness_incident.handle"
  "tenant_admin|ops.trade_monitor.read"
  "tenant_audit_readonly|ops.trade_monitor.read"
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

check_column() {
  local fq_column="$1"
  local schema="${fq_column%%.*}"
  local remain="${fq_column#*.}"
  local table="${remain%%.*}"
  local column="${remain#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema='${schema}' AND table_name='${table}' AND column_name='${column}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] column missing: ${fq_column}" >&2
    exit 1
  fi
}

check_permission() {
  local permission_code="$1"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM authz.permission_definition WHERE permission_code='${permission_code}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] permission missing: authz.permission_definition.${permission_code}" >&2
    exit 1
  fi
}

check_role_permission() {
  local item="$1"
  local role_key="${item%%|*}"
  local permission_code="${item#*|}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM authz.role_permission WHERE role_key='${role_key}' AND permission_code='${permission_code}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] role-permission missing: authz.role_permission (${role_key}, ${permission_code})" >&2
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

for column in "${required_columns[@]}"; do
  check_column "${column}"
done

for permission_code in "${required_permissions[@]}"; do
  check_permission "${permission_code}"
done

for rp in "${required_role_permissions[@]}"; do
  check_role_permission "${rp}"
done

echo "[ok] migrations 065/066/067/068 baseline verified"
