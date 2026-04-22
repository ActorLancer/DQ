#!/usr/bin/env bash

SCRIPT_SOURCE="${BASH_SOURCE[0]:-}"
if [[ -z "${SCRIPT_SOURCE}" && -n "${ZSH_VERSION:-}" ]]; then
  SCRIPT_SOURCE="${(%):-%x}"
fi

if [[ -z "${SCRIPT_SOURCE}" ]]; then
  echo "unable to resolve go-env.sh source path" >&2
  return 1 2>/dev/null || exit 1
fi

if [[ "${SCRIPT_SOURCE}" == "${0}" ]]; then
  echo "source scripts/go-env.sh from another script" >&2
  exit 1
fi

if command -v realpath >/dev/null 2>&1; then
  SCRIPT_SOURCE="$(realpath "${SCRIPT_SOURCE}")"
fi

SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_SOURCE}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
export GOPATH="${REPO_ROOT}/third_party/external-deps/go"
export GOMODCACHE="${GOPATH}/pkg/mod"
export GOCACHE="${GOPATH}/cache"
export GOBIN="${GOPATH}/bin"
mkdir -p "${GOMODCACHE}" "${GOCACHE}" "${GOBIN}"
