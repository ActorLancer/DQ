#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

OPENAPI_DIR="packages/openapi"
if [[ ! -d "$OPENAPI_DIR" ]]; then
  echo "[error] missing directory: $OPENAPI_DIR" >&2
  exit 1
fi

shopt -s nullglob
yaml_files=("$OPENAPI_DIR"/*.yaml)
if [[ ${#yaml_files[@]} -eq 0 ]]; then
  echo "[error] no openapi yaml files found under $OPENAPI_DIR" >&2
  exit 1
fi

for file in "${yaml_files[@]}"; do
  grep -qE '^openapi:[[:space:]]+3\.' "$file" || {
    echo "[error] $file missing openapi 3.x header" >&2
    exit 1
  }
  grep -qE '^[[:space:]]*title:' "$file" || {
    echo "[error] $file missing info.title" >&2
    exit 1
  }
  grep -qE '^[[:space:]]*version:' "$file" || {
    echo "[error] $file missing info.version" >&2
    exit 1
  }
  grep -qE '^paths:' "$file" || {
    echo "[error] $file missing paths section" >&2
    exit 1
  }
done

# V1 skeleton drift guard for currently implemented internal/ops endpoints.
ops_file="$OPENAPI_DIR/ops.yaml"
for path in "/health/live" "/health/ready" "/health/deps" "/internal/runtime" "/internal/dev/trace-links"; do
  grep -q "$path" "$ops_file" || {
    echo "[error] $ops_file missing path: $path" >&2
    exit 1
  }
done

echo "[ok] openapi schema skeleton check passed"
