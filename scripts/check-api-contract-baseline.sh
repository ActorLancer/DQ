#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

CONTRACT_DIR="fixtures/contracts/test-003"
KEY_FIELDS_FILE="${CONTRACT_DIR}/key-response-fields.tsv"
ERROR_CODES_FILE="${CONTRACT_DIR}/error-code-baseline.tsv"
STATE_MACHINE_FILE="${CONTRACT_DIR}/state-machine-contracts.tsv"
OPENAPI_DIR="packages/openapi"
STATE_DOC="docs/05-test-cases/order-state-machine.md"
TRADE_OPENAPI="packages/openapi/trade.yaml"

fail() {
  echo "[fail] $*" >&2
  exit 1
}

ok() {
  echo "[ok]   $*"
}

log() {
  echo "[info] $*"
}

command -v rg >/dev/null 2>&1 || fail "rg not found"
command -v awk >/dev/null 2>&1 || fail "awk not found"

[[ -d "${OPENAPI_DIR}" ]] || fail "missing ${OPENAPI_DIR}"
[[ -f "${KEY_FIELDS_FILE}" ]] || fail "missing ${KEY_FIELDS_FILE}"
[[ -f "${ERROR_CODES_FILE}" ]] || fail "missing ${ERROR_CODES_FILE}"
[[ -f "${STATE_MACHINE_FILE}" ]] || fail "missing ${STATE_MACHINE_FILE}"
[[ -f "${STATE_DOC}" ]] || fail "missing ${STATE_DOC}"
[[ -f "${TRADE_OPENAPI}" ]] || fail "missing ${TRADE_OPENAPI}"

schema_block() {
  local file="$1"
  local schema="$2"
  awk -v schema="${schema}" '
    $0 == "    " schema ":" {
      capture = 1
      print
      next
    }
    capture && /^    [A-Za-z0-9_]+:/ {
      exit
    }
    capture {
      print
    }
  ' "${file}"
}

assert_contains_literal() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  printf '%s\n' "${haystack}" | rg -Fq "${needle}" || fail "${label} missing literal: ${needle}"
}

assert_required_field() {
  local haystack="$1"
  local field_name="$2"
  local label="$3"
  if printf '%s\n' "${haystack}" | rg -q "required: \\[[^]]*\\b${field_name}\\b"; then
    return
  fi
  printf '%s\n' "${haystack}" | rg -q "^[[:space:]]+- ${field_name}$" \
    || fail "${label} missing required field ${field_name}"
}

assert_absent_field() {
  local haystack="$1"
  local field_name="$2"
  local label="$3"
  if printf '%s\n' "${haystack}" | rg -q "^[[:space:]]+${field_name}:$"; then
    fail "${label} should not expose legacy field: ${field_name}"
  fi
}

check_success_envelopes() {
  local file schema block
  for file in "${OPENAPI_DIR}"/*.yaml; do
    while IFS= read -r schema; do
      [[ -n "${schema}" ]] || continue
      schema="${schema%:}"
      block="$(schema_block "${file}" "${schema}")"
      [[ -n "${block}" ]] || fail "${file} missing schema block ${schema}"
      for field in code message request_id data; do
        assert_required_field "${block}" "${field}" "${file}:${schema}"
        printf '%s\n' "${block}" | rg -q "^[[:space:]]+${field}:$" \
          || fail "${file}:${schema} missing success envelope field ${field}"
      done
      printf '%s\n' "${block}" | rg -q "^[[:space:]]+success:$" \
        && fail "${file}:${schema} should not expose legacy success flag"
      printf '%s\n' "${block}" | rg -Fq "required: [data]" \
        && fail "${file}:${schema} still contains legacy data.data wrapper"
    done < <(awk '/^    (ApiResponse|ApiEnvelope)[A-Za-z0-9_]+:/{print $1}' "${file}")
  done
  ok "success envelopes aligned"
}

check_error_envelopes() {
  local file block
  for file in "${OPENAPI_DIR}"/*.yaml; do
    if ! rg -q '^    ErrorResponse:$' "${file}"; then
      continue
    fi
    block="$(schema_block "${file}" "ErrorResponse")"
    [[ -n "${block}" ]] || fail "${file} missing ErrorResponse block"
    for field in code message request_id details; do
      assert_required_field "${block}" "${field}" "${file}:ErrorResponse"
      printf '%s\n' "${block}" | rg -q "^[[:space:]]+${field}:$" \
        || fail "${file}:ErrorResponse missing failure envelope field ${field}"
    done
    assert_contains_literal "${block}" "additionalProperties: true" "${file}:ErrorResponse"
    printf '%s\n' "${block}" | rg -q 'nullable: true' \
      && fail "${file}:ErrorResponse should not use nullable request_id"
  done
  ok "error envelopes aligned"
}

check_key_response_fields() {
  local file schema required_csv forbidden_csv block field
  while IFS='|' read -r file schema required_csv forbidden_csv; do
    [[ -n "${file}" ]] || continue
    [[ "${file}" =~ ^# ]] && continue
    block="$(schema_block "${file}" "${schema}")"
    [[ -n "${block}" ]] || fail "${file} missing schema block ${schema}"
    IFS=',' read -r -a required_fields <<< "${required_csv}"
    for field in "${required_fields[@]}"; do
      [[ -n "${field}" ]] || continue
      printf '%s\n' "${block}" | rg -q "^[[:space:]]+${field}:$" \
        || fail "${file}:${schema} missing required frozen field ${field}"
    done
    IFS=',' read -r -a forbidden_fields <<< "${forbidden_csv}"
    for field in "${forbidden_fields[@]}"; do
      [[ -n "${field}" ]] || continue
      assert_absent_field "${block}" "${field}" "${file}:${schema}"
    done
  done < "${KEY_FIELDS_FILE}"
  ok "key response fields aligned"
}

check_error_code_baseline() {
  local path token
  while IFS='|' read -r path token; do
    [[ -n "${path}" ]] || continue
    [[ "${path}" =~ ^# ]] && continue
    [[ -f "${path}" ]] || fail "missing baseline file ${path}"
    rg -Fq "${token}" "${path}" || fail "${path} missing error-code token ${token}"
  done < "${ERROR_CODES_FILE}"
  ok "error-code baseline aligned"
}

check_state_machine_contracts() {
  local schema actions_csv test_file error_code block action
  while IFS='|' read -r schema actions_csv test_file error_code; do
    [[ -n "${schema}" ]] || continue
    [[ "${schema}" =~ ^# ]] && continue
    block="$(schema_block "${TRADE_OPENAPI}" "${schema}")"
    [[ -n "${block}" ]] || fail "${TRADE_OPENAPI} missing schema block ${schema}"
    IFS=',' read -r -a actions <<< "${actions_csv}"
    for action in "${actions[@]}"; do
      [[ -n "${action}" ]] || continue
      printf '%s\n' "${block}" | rg -Fq "${action}" \
        || fail "${TRADE_OPENAPI}:${schema} missing frozen action ${action}"
    done
    [[ -f "${test_file}" ]] || fail "missing test file ${test_file}"
    rg -Fq "${error_code}" "${STATE_DOC}" \
      || fail "${STATE_DOC} missing state-machine error code ${error_code}"
    rg -Fq "${error_code}" "${test_file}" \
      || fail "${test_file} missing state-machine error code ${error_code}"
  done < "${STATE_MACHINE_FILE}"
  ok "state-machine contract baseline aligned"
}

main() {
  log "running TEST-003 API contract baseline checker"
  ./scripts/check-openapi-schema.sh
  check_success_envelopes
  check_error_envelopes
  check_key_response_fields
  check_error_code_baseline
  check_state_machine_contracts
  ok "TEST-003 API contract baseline checker passed"
}

main "$@"
