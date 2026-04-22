#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/go-env.sh"

pushd "$(cd "$(dirname "$0")/../services/fabric-event-listener" && pwd)" >/dev/null
go mod tidy
popd >/dev/null

echo "[done] fabric-event-listener Go dependencies synchronized"
