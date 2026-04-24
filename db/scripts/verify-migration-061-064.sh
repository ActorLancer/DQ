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
  catalog.asset_object_binding
  delivery.data_share_grant
  delivery.revision_subscription
  delivery.template_query_grant
  catalog.product_metadata_profile
  catalog.asset_field_definition
  catalog.asset_quality_report
  catalog.asset_processing_job
  catalog.asset_processing_input
  contract.data_contract
  catalog.raw_ingest_batch
  catalog.raw_object_manifest
  catalog.format_detection_result
  catalog.extraction_job
  catalog.preview_artifact
  catalog.storage_namespace
  catalog.storage_policy_profile
)

required_indexes=(
  catalog.idx_asset_object_binding_version
  delivery.idx_data_share_grant_order
  catalog.idx_product_metadata_profile_product
  contract.idx_data_contract_product
  catalog.idx_raw_ingest_batch_owner_status
  catalog.idx_extraction_job_version
  catalog.idx_storage_namespace_owner_kind
  catalog.idx_asset_version_storage_policy
  delivery.idx_storage_object_namespace
)

required_triggers=(
  catalog.trg_asset_object_binding_updated_at
  delivery.trg_data_share_grant_updated_at
  catalog.trg_product_metadata_profile_updated_at
  contract.trg_data_contract_updated_at
  catalog.trg_raw_ingest_batch_updated_at
  catalog.trg_preview_artifact_updated_at
  catalog.trg_storage_namespace_updated_at
  catalog.trg_storage_policy_profile_updated_at
)

required_columns=(
  "catalog.asset_version.release_mode"
  "catalog.asset_version.processing_mode"
  "catalog.asset_version.processing_stage"
  "catalog.asset_version.storage_policy_id"
  "catalog.product_sku.trade_mode"
  "catalog.asset_sample.masking_status"
  "contract.digital_contract.data_contract_id"
  "catalog.asset_storage_binding.storage_namespace_id"
  "delivery.storage_object.storage_namespace_id"
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

echo "[ok] migrations 061/062/063/064 baseline verified"
