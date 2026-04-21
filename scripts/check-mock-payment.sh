#!/usr/bin/env bash
set -euo pipefail

MOCK_BASE_URL="${MOCK_BASE_URL:-${MOCK_PAYMENT_BASE_URL:-http://127.0.0.1:${MOCK_PAYMENT_PORT:-8089}}}"

if ! curl -fsS "${MOCK_BASE_URL}/health/ready" >/dev/null; then
  echo "[fail] mock-payment-provider is not ready at ${MOCK_BASE_URL}" >&2
  echo "[hint] start it with 'make up-mocks' or 'make up-demo' before running this check" >&2
  exit 1
fi

code_success="$(curl -sS -o /tmp/mock_success.json -w '%{http_code}' -X POST "${MOCK_BASE_URL}/mock/payment/charge/success")"
code_fail="$(curl -sS -o /tmp/mock_fail.json -w '%{http_code}' -X POST "${MOCK_BASE_URL}/mock/payment/charge/fail")"
code_refund="$(curl -sS -o /tmp/mock_refund.json -w '%{http_code}' -X POST "${MOCK_BASE_URL}/mock/payment/refund/success")"
code_manual="$(curl -sS -o /tmp/mock_manual.json -w '%{http_code}' -X POST "${MOCK_BASE_URL}/mock/payment/manual-transfer/success")"

if [[ "${code_success}" != "200" || "${code_fail}" != "402" || "${code_refund}" != "200" || "${code_manual}" != "200" ]]; then
  echo "[fail] unexpected mock-payment status codes: success=${code_success}, fail=${code_fail}, refund=${code_refund}, manual=${code_manual}" >&2
  exit 1
fi

# timeout endpoint: expect client timeout under 3s because server delays 15s
if curl -m 3 -sS -X POST "${MOCK_BASE_URL}/mock/payment/charge/timeout" >/tmp/mock_timeout.json 2>/dev/null; then
  echo "[fail] timeout endpoint returned too fast" >&2
  exit 1
fi

echo "[ok] mock-payment scenarios verified"
