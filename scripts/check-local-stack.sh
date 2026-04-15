#!/usr/bin/env bash
set -euo pipefail

# Compatibility wrapper: the project originally used verify-local-stack.sh.
# Keep this script as the canonical acceptance entrypoint for ENV tasks.
./scripts/verify-local-stack.sh "$@"
