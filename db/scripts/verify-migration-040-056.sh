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
  billing.billing_event
  billing.settlement_record
  payment.payment_intent
  payment.payment_transaction
  payment.payout_instruction
  payment.sub_merchant_binding
  payment.split_instruction
  support.dispute_case
  support.dispute_status_history
  risk.reputation_snapshot
  risk.freeze_ticket
  risk.governance_action_log
  audit.audit_event
  audit.evidence_package
  audit.evidence_manifest
  audit.anchor_batch
  audit.replay_job
  audit.legal_hold
  ops.outbox_event
  ops.dead_letter_event
  ops.outbox_publish_attempt
  ops.consumer_idempotency_record
  search.product_search_document
  developer.mock_payment_case
  chain.chain_anchor
)

required_indexes=(
  billing.idx_billing_event_order_id
  payment.idx_payment_intent_status
  payment.idx_split_instruction_settlement_id
  payment.idx_split_instruction_reward_id
  support.idx_dispute_case_order_id
  risk.idx_reputation_snapshot_subject
  risk.idx_freeze_ticket_ref_status
  risk.idx_governance_action_log_ticket_id
  search.idx_product_search_document_tsv
  audit.idx_audit_event_trace
  audit.idx_anchor_batch_status
  ops.idx_outbox_pending
  ops.idx_dead_letter_reprocess
)

required_triggers=(
  payment.trg_payment_intent_updated_at
  payment.trg_sub_merchant_binding_updated_at
  payment.trg_split_instruction_updated_at
  support.trg_dispute_status_history
  risk.trg_freeze_ticket_updated_at
  catalog.trg_product_search_refresh
  trade.trg_order_outbox
  audit.trg_anchor_batch_updated_at
  audit.trg_audit_event_default_append_only
  ops.trg_event_route_policy_updated_at
)

required_constraints=(
  audit.fk_audit_event_manifest
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

check_constraint() {
  local fq_constraint="$1"
  local schema="${fq_constraint%%.*}"
  local constraint="${fq_constraint#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM information_schema.table_constraints WHERE constraint_schema='${schema}' AND constraint_name='${constraint}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] constraint missing: ${fq_constraint}" >&2
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

for constraint in "${required_constraints[@]}"; do
  check_constraint "${constraint}"
done

echo "[ok] migrations 040/050/055/056 baseline verified"
