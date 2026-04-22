#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${REPO_ROOT}/scripts/fabric-env.sh"

if [[ ! -d "${FABRIC_TEST_NETWORK_ROOT}" ]]; then
  echo "[skip] fabric samples test-network not present" >&2
  exit 0
fi

patch_in_place() {
  local file_path="${1}"
  local search="${2}"
  local replace="${3}"
  python3 - "$file_path" "$search" "$replace" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
search = sys.argv[2]
replace = sys.argv[3]
content = path.read_text()
updated = content.replace(search, replace)
if updated != content:
    path.write_text(updated)
PY
}

patch_in_place \
  "${FABRIC_TEST_NETWORK_ROOT}/network.sh" \
  'DOCKER_IMAGE_VERSION=$(${CONTAINER_CLI} run --rm hyperledger/fabric-peer:latest peer version | sed -ne '\''s/^ Version: //p'\'')' \
  'DOCKER_IMAGE_VERSION=$(${CONTAINER_CLI} run --rm hyperledger/fabric-peer:'"${FABRIC_VERSION}"' peer version | sed -ne '\''s/^ Version: //p'\'')'

patch_in_place \
  "${FABRIC_TEST_NETWORK_ROOT}/network.sh" \
  'CA_DOCKER_IMAGE_VERSION=$(${CONTAINER_CLI} run --rm hyperledger/fabric-ca:latest fabric-ca-client version | sed -ne '\''s/ Version: //p'\'' | head -1)' \
  'CA_DOCKER_IMAGE_VERSION=$(${CONTAINER_CLI} run --rm hyperledger/fabric-ca:'"${FABRIC_CA_VERSION}"' fabric-ca-client version | sed -ne '\''s/ Version: //p'\'' | head -1)'

for yaml_file in \
  "${FABRIC_TEST_NETWORK_ROOT}/compose/compose-test-net.yaml" \
  "${FABRIC_TEST_NETWORK_ROOT}/compose/compose-bft-test-net.yaml" \
  "${FABRIC_TEST_NETWORK_ROOT}/compose/compose-ca.yaml" \
  "${FABRIC_TEST_NETWORK_ROOT}/compose/docker/docker-compose-test-net.yaml" \
  "${FABRIC_TEST_NETWORK_ROOT}/compose/docker/docker-compose-bft-test-net.yaml"; do
  if [[ -f "${yaml_file}" ]]; then
    patch_in_place "${yaml_file}" "hyperledger/fabric-peer:latest" "hyperledger/fabric-peer:${FABRIC_VERSION}"
    patch_in_place "${yaml_file}" "hyperledger/fabric-orderer:latest" "hyperledger/fabric-orderer:${FABRIC_VERSION}"
    patch_in_place "${yaml_file}" "hyperledger/fabric-ca:latest" "hyperledger/fabric-ca:${FABRIC_CA_VERSION}"
  fi
done

echo "[done] fabric samples patched to pinned image tags"
echo "  FABRIC_VERSION=${FABRIC_VERSION}"
echo "  FABRIC_CA_VERSION=${FABRIC_CA_VERSION}"
