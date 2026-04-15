#!/usr/bin/env bash
set -euo pipefail

mkdir -p infra/fabric/state/chaincode

cat > infra/fabric/state/chaincode/contracts.json <<'EOF'
{
  "contracts": [
    {
      "name": "order_digest",
      "description": "Order hash anchor placeholder interface"
    },
    {
      "name": "authorization_digest",
      "description": "Authorization hash anchor placeholder interface"
    },
    {
      "name": "acceptance_digest",
      "description": "Acceptance hash anchor placeholder interface"
    },
    {
      "name": "evidence_batch_root",
      "description": "Evidence batch Merkle root placeholder interface"
    }
  ]
}
EOF

echo "[done] fabric chaincode placeholder artifact generated: infra/fabric/state/chaincode/contracts.json"
