#!/usr/bin/env bash
set -euo pipefail

reference_type="${1:-}"
reference_id="${2:-}"
submission_kind="${3:-}"

if [[ -z "${reference_type}" || -z "${reference_id}" || -z "${submission_kind}" ]]; then
  echo "usage: $0 <reference_type> <reference_id> <submission_kind>" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${REPO_ROOT}/scripts/fabric-env.sh"
fabric_use_org1_admin

peer chaincode query \
  -C "${FABRIC_CHANNEL_NAME}" \
  -n "${FABRIC_CHAINCODE_NAME}" \
  -c "{\"function\":\"GetAnchorByReference\",\"Args\":[\"${reference_type}\",\"${reference_id}\",\"${submission_kind}\"]}"
