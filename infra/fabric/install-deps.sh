#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${REPO_ROOT}/scripts/fabric-env.sh"

INSTALL_SCRIPT="${FABRIC_EXTERNAL_DEPS_ROOT}/install-fabric.sh"
PATCH_SCRIPT="${REPO_ROOT}/infra/fabric/patch-samples.sh"

mkdir -p "${FABRIC_EXTERNAL_DEPS_ROOT}"

if [[ ! -f "${INSTALL_SCRIPT}" || "${FABRIC_FORCE_REFRESH_SCRIPT:-0}" == "1" ]]; then
  curl -fsSLo "${INSTALL_SCRIPT}" https://raw.githubusercontent.com/hyperledger/fabric/main/scripts/install-fabric.sh
  chmod +x "${INSTALL_SCRIPT}"
fi

has_peer_binary=0
has_ca_binary=0
has_peer_image=0
has_ca_image=0

if [[ -x "${FABRIC_SAMPLES_ROOT}/bin/peer" ]]; then
  has_peer_binary=1
fi
if [[ -x "${FABRIC_SAMPLES_ROOT}/bin/fabric-ca-client" ]]; then
  has_ca_binary=1
fi
if docker image inspect "hyperledger/fabric-peer:${FABRIC_VERSION}" >/dev/null 2>&1; then
  has_peer_image=1
fi
if docker image inspect "hyperledger/fabric-ca:${FABRIC_CA_VERSION}" >/dev/null 2>&1; then
  has_ca_image=1
fi

if [[ "${FABRIC_FORCE_REINSTALL:-0}" == "1" || "${has_peer_binary}" == "0" || "${has_ca_binary}" == "0" || "${has_peer_image}" == "0" || "${has_ca_image}" == "0" ]]; then
  pushd "${FABRIC_EXTERNAL_DEPS_ROOT}" >/dev/null
  "${INSTALL_SCRIPT}" -f "${FABRIC_VERSION}" -c "${FABRIC_CA_VERSION}" samples binary docker
  popd >/dev/null
else
  echo "[skip] fabric samples, binaries, and docker images already prepared"
fi

"${PATCH_SCRIPT}"

echo "[done] fabric dependencies ready"
echo "  FABRIC_EXTERNAL_DEPS_ROOT=${FABRIC_EXTERNAL_DEPS_ROOT}"
echo "  FABRIC_SAMPLES_ROOT=${FABRIC_SAMPLES_ROOT}"
echo "  FABRIC_TEST_NETWORK_ROOT=${FABRIC_TEST_NETWORK_ROOT}"
