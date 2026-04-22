#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/go-env.sh"

pushd "$(cd "$(dirname "$0")/../services/fabric-adapter" && pwd)" >/dev/null
go test ./...
popd >/dev/null
