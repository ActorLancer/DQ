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
  python3 - "${OPENAPI_DIR}" <<'PY'
import glob
import sys
from typing import Any

import yaml

openapi_dir = sys.argv[1]
http_methods = {"get", "post", "put", "patch", "delete", "options", "head"}
errors: list[str] = []


def ref_name(ref: Any) -> str | None:
    if not isinstance(ref, str):
        return None
    prefix = "#/components/schemas/"
    if not ref.startswith(prefix):
        return None
    return ref[len(prefix):]


def looks_like_legacy_data_wrapper(schema: Any, components: dict[str, Any], seen: set[str]) -> bool:
    if not isinstance(schema, dict):
        return False
    if "$ref" in schema:
        name = ref_name(schema["$ref"])
        if name is None or name in seen:
            return False
        target = components.get(name)
        if target is None:
            return False
        return looks_like_legacy_data_wrapper(target, components, seen | {name})
    for key in ("oneOf", "allOf"):
        if key in schema and isinstance(schema[key], list):
            return any(looks_like_legacy_data_wrapper(item, components, seen) for item in schema[key])
    props = schema.get("properties")
    if not isinstance(props, dict):
        return False
    keys = set(props.keys())
    required = schema.get("required")
    required_set = set(required) if isinstance(required, list) else set()
    return "data" in keys and not ({"code", "message", "request_id"} & keys) and (
        keys == {"data"} or required_set == {"data"}
    )


def validate_envelope(
    schema: Any,
    components: dict[str, Any],
    label: str,
    seen_refs: set[str],
) -> list[str]:
    if not isinstance(schema, dict):
        return [f"{label} schema is not an object"]

    if "$ref" in schema:
        name = ref_name(schema["$ref"])
        if name is None:
            return [f"{label} has unsupported schema ref {schema['$ref']}"]
        if name in seen_refs:
            return []
        target = components.get(name)
        if target is None:
            return [f"{label} references missing schema {name}"]
        return validate_envelope(target, components, f"{label}->{name}", seen_refs | {name})

    for key in ("oneOf", "allOf"):
        if key in schema:
            variants = schema.get(key)
            if not isinstance(variants, list) or not variants:
                return [f"{label} has empty {key}"]
            collected: list[str] = []
            for idx, variant in enumerate(variants):
                variant_errors = validate_envelope(
                    variant, components, f"{label}[{key}[{idx}]]", seen_refs
                )
                if not variant_errors:
                    return []
                collected.extend(variant_errors)
            head = f"{label} has no envelope-valid {key} variants"
            return [head, *collected[:1]]

    props = schema.get("properties")
    if not isinstance(props, dict):
        return [f"{label} missing object properties"]

    required = schema.get("required")
    required_set = set(required) if isinstance(required, list) else set()

    local_errors: list[str] = []
    for field in ("code", "message", "request_id", "data"):
        if field not in required_set:
            local_errors.append(f"{label} missing required field {field}")
        if field not in props:
            local_errors.append(f"{label} missing envelope field {field}")
    if "success" in props:
        local_errors.append(f"{label} should not expose legacy success flag")
    data_schema = props.get("data")
    if looks_like_legacy_data_wrapper(data_schema, components, set()):
        local_errors.append(f"{label} still contains nested data.data wrapper")
    return local_errors


for file in sorted(glob.glob(f"{openapi_dir}/*.yaml")):
    with open(file, "r", encoding="utf-8") as fp:
        doc = yaml.safe_load(fp)
    if not isinstance(doc, dict):
        errors.append(f"{file} is not a valid OpenAPI document")
        continue
    components = ((doc.get("components") or {}).get("schemas") or {})
    if not isinstance(components, dict):
        errors.append(f"{file} components.schemas is invalid")
        continue
    paths = doc.get("paths") or {}
    if not isinstance(paths, dict):
        continue
    for path, path_item in paths.items():
        if not isinstance(path_item, dict):
            continue
        for method, operation in path_item.items():
            if method.lower() not in http_methods:
                continue
            if not isinstance(operation, dict):
                continue
            responses = operation.get("responses") or {}
            if not isinstance(responses, dict):
                continue
            response_200 = responses.get("200")
            if response_200 is None:
                response_200 = responses.get(200)
            if not isinstance(response_200, dict):
                continue
            content = response_200.get("content") or {}
            if not isinstance(content, dict):
                continue
            json_content = content.get("application/json")
            if not isinstance(json_content, dict):
                continue
            schema = json_content.get("schema")
            if schema is None:
                continue
            label = f"{file}:{method.upper()} {path}:200"
            errors.extend(validate_envelope(schema, components, label, set()))

if errors:
    seen: set[str] = set()
    unique_errors: list[str] = []
    for err in errors:
        if err in seen:
            continue
        seen.add(err)
        unique_errors.append(err)
    for err in unique_errors[:20]:
        print(f"[fail] {err}", file=sys.stderr)
    if len(unique_errors) > 20:
        print(f"[fail] ... and {len(unique_errors) - 20} more", file=sys.stderr)
    sys.exit(1)
PY
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
