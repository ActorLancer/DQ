#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-core}"
./scripts/verify-local-stack.sh "${MODE}"

