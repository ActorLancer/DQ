#!/usr/bin/env bash
set -euo pipefail

KEYCLOAK_BASE_URL="${KEYCLOAK_BASE_URL:-http://127.0.0.1:8081}"
REALM_NAME="${REALM_NAME:-platform-local}"

resp="$(curl -fsS "${KEYCLOAK_BASE_URL}/realms/${REALM_NAME}/.well-known/openid-configuration")"
echo "${resp}" | jq -e ".issuer | contains(\"/realms/${REALM_NAME}\")" >/dev/null
echo "[ok] keycloak realm imported: ${REALM_NAME}"
