#!/usr/bin/env bash
set -euo pipefail

OUT="${1:-merged-placeholder.yaml}"
cat > "${OUT}" <<'EOF'
openapi: 3.1.0
info:
  title: Merged API Placeholder
  version: v1
paths: {}
EOF
echo "[done] wrote ${OUT}"
