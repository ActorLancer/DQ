#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
echo "[compat] deploy-chaincode-placeholder.sh has been replaced by deploy-chaincode.sh" >&2
exec "${SCRIPT_DIR}/deploy-chaincode.sh" "$@"
