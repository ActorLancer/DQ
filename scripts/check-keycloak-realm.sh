#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
if [[ -f "${ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${ENV_FILE}"
  set +a
fi

KEYCLOAK_BASE_URL="${KEYCLOAK_BASE_URL:-http://127.0.0.1:8081}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-${REALM_NAME:-platform-local}}"
KEYCLOAK_TOKEN_CLIENT_ID="${KEYCLOAK_TOKEN_CLIENT_ID:-portal-web}"
KEYCLOAK_TOKEN_USERNAME="${KEYCLOAK_TOKEN_USERNAME:-local-platform-admin}"
KEYCLOAK_TOKEN_PASSWORD="${KEYCLOAK_TOKEN_PASSWORD:-LocalPlatformAdmin123!}"
KEYCLOAK_EXPECTED_ROLE="${KEYCLOAK_EXPECTED_ROLE:-platform_admin}"

resp="$(curl -fsS "${KEYCLOAK_BASE_URL}/realms/${KEYCLOAK_REALM}/.well-known/openid-configuration")"
echo "${resp}" | jq -e ".issuer | contains(\"/realms/${KEYCLOAK_REALM}\")" >/dev/null

token_resp="$(
  curl -sS -X POST \
    "${KEYCLOAK_BASE_URL}/realms/${KEYCLOAK_REALM}/protocol/openid-connect/token" \
    -H 'content-type: application/x-www-form-urlencoded' \
    --data-urlencode 'grant_type=password' \
    --data-urlencode "client_id=${KEYCLOAK_TOKEN_CLIENT_ID}" \
    --data-urlencode "username=${KEYCLOAK_TOKEN_USERNAME}" \
    --data-urlencode "password=${KEYCLOAK_TOKEN_PASSWORD}"
)"
access_token="$(echo "${token_resp}" | jq -r '.access_token // empty')"
if [[ -z "${access_token}" ]]; then
  echo "[fail] keycloak password grant failed for realm ${KEYCLOAK_REALM}: ${token_resp}" >&2
  exit 1
fi

decode_jwt_payload() {
  local token="$1"
  local payload
  payload="$(cut -d '.' -f2 <<<"${token}")"
  payload="${payload//-/+}"
  payload="${payload//_/\/}"
  case $(( ${#payload} % 4 )) in
    2) payload="${payload}==" ;;
    3) payload="${payload}=" ;;
    1)
      echo "[fail] invalid jwt payload length" >&2
      exit 1
      ;;
  esac
  printf '%s' "${payload}" | base64 --decode
}

payload_json="$(decode_jwt_payload "${access_token}")"
echo "${payload_json}" | jq -e \
  --arg role "${KEYCLOAK_EXPECTED_ROLE}" \
  '
    (.user_id | type == "string" and test("^[0-9a-fA-F-]{36}$")) and
    (.org_id | type == "string" and test("^[0-9a-fA-F-]{36}$")) and
    (.realm_access.roles | type == "array" and index($role) != null)
  ' >/dev/null

echo "[ok] keycloak realm imported and password grant passed: ${KEYCLOAK_REALM}"
