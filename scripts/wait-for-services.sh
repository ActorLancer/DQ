#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-core}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"
INTERVAL_SECONDS="${INTERVAL_SECONDS:-5}"

if [[ "${TIMEOUT_SECONDS}" -le 0 ]]; then
  echo "[fail] TIMEOUT_SECONDS must be > 0" >&2
  exit 1
fi

if [[ "${INTERVAL_SECONDS}" -le 0 ]]; then
  echo "[fail] INTERVAL_SECONDS must be > 0" >&2
  exit 1
fi

echo "[info] Waiting for local services mode=${MODE} timeout=${TIMEOUT_SECONDS}s interval=${INTERVAL_SECONDS}s"

elapsed=0
while true; do
  if ./scripts/verify-local-stack.sh "${MODE}" >/tmp/wait-for-services.last.log 2>&1; then
    echo "[ok] local services are ready for mode=${MODE}"
    exit 0
  fi

  if [[ "${elapsed}" -ge "${TIMEOUT_SECONDS}" ]]; then
    echo "[fail] services not ready within ${TIMEOUT_SECONDS}s (mode=${MODE})" >&2
    cat /tmp/wait-for-services.last.log >&2
    exit 1
  fi

  sleep "${INTERVAL_SECONDS}"
  elapsed=$((elapsed + INTERVAL_SECONDS))
done
