#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/go-env.sh"

pushd "$(cd "$(dirname "$0")/../services/fabric-ca-admin" && pwd)" >/dev/null
go mod tidy
popd >/dev/null

echo "[done] fabric-ca-admin Go dependencies synchronized"
