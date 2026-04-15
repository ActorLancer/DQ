#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-core}"
WAIT_TIMEOUT_SECONDS="${WAIT_TIMEOUT_SECONDS:-120}"
WAIT_INTERVAL_SECONDS="${WAIT_INTERVAL_SECONDS:-5}"

TIMEOUT_SECONDS="${WAIT_TIMEOUT_SECONDS}" \
INTERVAL_SECONDS="${WAIT_INTERVAL_SECONDS}" \
./scripts/wait-for-services.sh "${MODE}"

./scripts/verify-local-stack.sh "${MODE}"
