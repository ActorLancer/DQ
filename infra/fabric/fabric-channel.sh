#!/usr/bin/env bash
set -euo pipefail

FABRIC_CHANNEL_NAME="${FABRIC_CHANNEL_NAME:-datab-channel}"
mkdir -p infra/fabric/state/channel
cat > "infra/fabric/state/channel/${FABRIC_CHANNEL_NAME}.tx" <<EOF
channel=${FABRIC_CHANNEL_NAME}
created_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
EOF
echo "[done] fabric channel artifact prepared: infra/fabric/state/channel/${FABRIC_CHANNEL_NAME}.tx"
