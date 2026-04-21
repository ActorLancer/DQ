#!/usr/bin/env bash
set -euo pipefail

MINIO_ALIAS="${MINIO_ALIAS:-local}"
MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://127.0.0.1:9000}"
MINIO_ACCESS_KEY="${MINIO_ACCESS_KEY:-${MINIO_ROOT_USER:-datab}}"
MINIO_SECRET_KEY="${MINIO_SECRET_KEY:-${MINIO_ROOT_PASSWORD:-datab_local_pass}}"
MINIO_MC_IMAGE="${MINIO_MC_IMAGE:-minio/mc:RELEASE.2025-08-13T08-35-41Z}"

BUCKETS=(
  "${BUCKET_RAW_DATA:-raw-data}"
  "${BUCKET_PREVIEW_ARTIFACTS:-preview-artifacts}"
  "${BUCKET_DELIVERY_OBJECTS:-delivery-objects}"
  "${BUCKET_REPORT_RESULTS:-report-results}"
  "${BUCKET_EVIDENCE_PACKAGES:-evidence-packages}"
  "${BUCKET_MODEL_ARTIFACTS:-model-artifacts}"
)

MC_HOST_VALUE="${MINIO_ENDPOINT/http:\/\//http://${MINIO_ACCESS_KEY}:${MINIO_SECRET_KEY}@}"
MC_HOST_VALUE="${MC_HOST_VALUE/https:\/\//https://${MINIO_ACCESS_KEY}:${MINIO_SECRET_KEY}@}"

mc_run() {
  docker run --rm --network host \
    -e "MC_HOST_${MINIO_ALIAS}=${MC_HOST_VALUE}" \
    "${MINIO_MC_IMAGE}" "$@"
}

for bucket in "${BUCKETS[@]}"; do
  mc_run mb --ignore-existing "${MINIO_ALIAS}/${bucket}" >/dev/null
  echo "[ok] bucket ready: ${bucket}"
done

# Bucket policy: keep preview artifacts downloadable for demo viewing.
mc_run anonymous set download "${MINIO_ALIAS}/${BUCKET_PREVIEW_ARTIFACTS:-preview-artifacts}" >/dev/null
echo "[ok] policy set: ${BUCKET_PREVIEW_ARTIFACTS:-preview-artifacts} -> download"

# Lifecycle example: short retention for preview artifacts.
mc_run ilm rule add --expire-days "30" "${MINIO_ALIAS}/${BUCKET_PREVIEW_ARTIFACTS:-preview-artifacts}" >/dev/null
echo "[ok] lifecycle set: ${BUCKET_PREVIEW_ARTIFACTS:-preview-artifacts} expire 30d"

# Test object upload
tmp_file="$(mktemp)"
echo "v1-core minio init $(date -u +%Y-%m-%dT%H:%M:%SZ)" > "${tmp_file}"
docker run --rm --network host \
  -e "MC_HOST_${MINIO_ALIAS}=${MC_HOST_VALUE}" \
  -v "${tmp_file}:/tmp/init.txt:ro" \
  "${MINIO_MC_IMAGE}" cp "/tmp/init.txt" "${MINIO_ALIAS}/${BUCKET_EVIDENCE_PACKAGES:-evidence-packages}/_health/init.txt" >/dev/null
rm -f "${tmp_file}"
echo "[ok] test object uploaded: ${BUCKET_EVIDENCE_PACKAGES:-evidence-packages}/_health/init.txt"

echo "[done] minio initialization complete"
