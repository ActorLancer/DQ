#!/usr/bin/env bash

SCRIPT_SOURCE="${BASH_SOURCE[0]:-}"
if [[ -z "${SCRIPT_SOURCE}" && -n "${ZSH_VERSION:-}" ]]; then
  SCRIPT_SOURCE="${(%):-%x}"
fi

if [[ -z "${SCRIPT_SOURCE}" ]]; then
  echo "unable to resolve fabric-env.sh source path" >&2
  return 1 2>/dev/null || exit 1
fi

if [[ "${SCRIPT_SOURCE}" == "${0}" ]]; then
  echo "source scripts/fabric-env.sh from another script" >&2
  exit 1
fi

if command -v realpath >/dev/null 2>&1; then
  SCRIPT_SOURCE="$(realpath "${SCRIPT_SOURCE}")"
fi

SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_SOURCE}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

fabric_abs_path() {
  local value="${1:-}"
  if [[ -z "${value}" ]]; then
    return 0
  fi
  if [[ "${value}" == /* ]]; then
    printf '%s\n' "${value}"
  else
    printf '%s\n' "${REPO_ROOT}/${value}"
  fi
}

fabric_first_file() {
  local dir_path="${1:-}"
  if [[ -z "${dir_path}" || ! -d "${dir_path}" ]]; then
    return 1
  fi
  find "${dir_path}" -maxdepth 1 -type f | sort | head -n 1
}

fabric_use_org1_admin() {
  export CORE_PEER_TLS_ENABLED=true
  export CORE_PEER_LOCALMSPID="Org1MSP"
  export CORE_PEER_TLS_ROOTCERT_FILE="${FABRIC_ORG1_PEER_TLS_CERT_PATH}"
  export CORE_PEER_MSPCONFIGPATH="${FABRIC_ORG1_ADMIN_MSP_PATH}"
  export CORE_PEER_ADDRESS="${FABRIC_ORG1_PEER_ADDRESS}"
}

fabric_use_org2_admin() {
  export CORE_PEER_TLS_ENABLED=true
  export CORE_PEER_LOCALMSPID="Org2MSP"
  export CORE_PEER_TLS_ROOTCERT_FILE="${FABRIC_ORG2_PEER_TLS_CERT_PATH}"
  export CORE_PEER_MSPCONFIGPATH="${FABRIC_ORG2_ADMIN_MSP_PATH}"
  export CORE_PEER_ADDRESS="${FABRIC_ORG2_PEER_ADDRESS}"
}

export FABRIC_VERSION="${FABRIC_VERSION:-2.5.15}"
export FABRIC_CA_VERSION="${FABRIC_CA_VERSION:-1.5.17}"
export FABRIC_EXTERNAL_DEPS_ROOT="${FABRIC_EXTERNAL_DEPS_ROOT:-${REPO_ROOT}/third_party/external-deps/fabric}"
export FABRIC_SAMPLES_ROOT="${FABRIC_SAMPLES_ROOT:-${FABRIC_EXTERNAL_DEPS_ROOT}/fabric-samples}"
export FABRIC_TEST_NETWORK_ROOT="${FABRIC_TEST_NETWORK_ROOT:-${FABRIC_SAMPLES_ROOT}/test-network}"
export FABRIC_STATE_DIR="${FABRIC_STATE_DIR:-${REPO_ROOT}/infra/fabric/state}"
export FABRIC_NETWORK_NAME="${FABRIC_NETWORK_NAME:-fabric-test-network}"
export FABRIC_CHANNEL_NAME="${FABRIC_CHANNEL_NAME:-datab-channel}"
export FABRIC_CHAINCODE_NAME="${FABRIC_CHAINCODE_NAME:-datab-audit-anchor}"
export FABRIC_CHAINCODE_VERSION="${FABRIC_CHAINCODE_VERSION:-1.0}"
export FABRIC_CHAINCODE_SEQUENCE="${FABRIC_CHAINCODE_SEQUENCE:-1}"
export FABRIC_CHAINCODE_PATH="${FABRIC_CHAINCODE_PATH:-${REPO_ROOT}/infra/fabric/chaincode/${FABRIC_CHAINCODE_NAME}}"
export FABRIC_GATEWAY_ENDPOINT="${FABRIC_GATEWAY_ENDPOINT:-dns:///localhost:7051}"
export FABRIC_GATEWAY_PEER="${FABRIC_GATEWAY_PEER:-peer0.org1.example.com}"
export FABRIC_MSP_ID="${FABRIC_MSP_ID:-Org1MSP}"

default_tls_cert="third_party/external-deps/fabric/fabric-samples/test-network/organizations/peerOrganizations/org1.example.com/peers/peer0.org1.example.com/tls/ca.crt"
default_sign_cert="third_party/external-deps/fabric/fabric-samples/test-network/organizations/peerOrganizations/org1.example.com/users/User1@org1.example.com/msp/signcerts/cert.pem"
default_private_key_dir="third_party/external-deps/fabric/fabric-samples/test-network/organizations/peerOrganizations/org1.example.com/users/User1@org1.example.com/msp/keystore"

export FABRIC_TLS_CERT_PATH="$(fabric_abs_path "${FABRIC_TLS_CERT_PATH:-${default_tls_cert}}")"
export FABRIC_SIGN_CERT_PATH="$(fabric_abs_path "${FABRIC_SIGN_CERT_PATH:-${default_sign_cert}}")"
export FABRIC_PRIVATE_KEY_DIR="$(fabric_abs_path "${FABRIC_PRIVATE_KEY_DIR:-${default_private_key_dir}}")"
if [[ -z "${FABRIC_PRIVATE_KEY_PATH:-}" ]]; then
  if key_path="$(fabric_first_file "${FABRIC_PRIVATE_KEY_DIR}")"; then
    export FABRIC_PRIVATE_KEY_PATH="${key_path}"
  else
    export FABRIC_PRIVATE_KEY_PATH=""
  fi
else
  export FABRIC_PRIVATE_KEY_PATH="$(fabric_abs_path "${FABRIC_PRIVATE_KEY_PATH}")"
fi

export FABRIC_ORG1_PEER_TLS_CERT_PATH="$(fabric_abs_path "third_party/external-deps/fabric/fabric-samples/test-network/organizations/peerOrganizations/org1.example.com/peers/peer0.org1.example.com/tls/ca.crt")"
export FABRIC_ORG1_ADMIN_MSP_PATH="$(fabric_abs_path "third_party/external-deps/fabric/fabric-samples/test-network/organizations/peerOrganizations/org1.example.com/users/Admin@org1.example.com/msp")"
export FABRIC_ORG1_PEER_ADDRESS="${FABRIC_ORG1_PEER_ADDRESS:-localhost:7051}"
export FABRIC_ORG2_PEER_TLS_CERT_PATH="$(fabric_abs_path "third_party/external-deps/fabric/fabric-samples/test-network/organizations/peerOrganizations/org2.example.com/peers/peer0.org2.example.com/tls/ca.crt")"
export FABRIC_ORG2_ADMIN_MSP_PATH="$(fabric_abs_path "third_party/external-deps/fabric/fabric-samples/test-network/organizations/peerOrganizations/org2.example.com/users/Admin@org2.example.com/msp")"
export FABRIC_ORG2_PEER_ADDRESS="${FABRIC_ORG2_PEER_ADDRESS:-localhost:9051}"

export PATH="${FABRIC_SAMPLES_ROOT}/bin:${PATH}"
export FABRIC_CFG_PATH="${FABRIC_CFG_PATH:-${FABRIC_SAMPLES_ROOT}/config}"

mkdir -p "${FABRIC_STATE_DIR}" "${FABRIC_EXTERNAL_DEPS_ROOT}"
