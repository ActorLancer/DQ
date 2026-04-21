#!/usr/bin/env bash
set -euo pipefail

KEYCLOAK_BASE_URL="${KEYCLOAK_BASE_URL:-http://127.0.0.1:8081}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-${REALM_NAME:-platform-local}}"

resp="$(curl -fsS "${KEYCLOAK_BASE_URL}/realms/${KEYCLOAK_REALM}/.well-known/openid-configuration")"
echo "${resp}" | jq -e ".issuer | contains(\"/realms/${KEYCLOAK_REALM}\")" >/dev/null
echo "[ok] keycloak realm imported: ${KEYCLOAK_REALM}"
