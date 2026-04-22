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
for path in \
  "/health/live" \
  "/health/ready" \
  "/health/deps" \
  "/internal/runtime" \
  "/internal/dev/trace-links" \
  "/internal/dev/overview" \
  "/internal/notifications/templates/preview" \
  "/internal/notifications/send" \
  "/internal/notifications/audit/search" \
  "/internal/notifications/dead-letters/{dead_letter_event_id}/replay"; do
  grep -q "$path" "$ops_file" || {
    echo "[error] $ops_file missing path: $path" >&2
    exit 1
  }
done

for token in "aggregate_type" "event_type" "target_topic" "step_up_ticket" "dtp.notification.dispatch"; do
  grep -q "$token" "$ops_file" || {
    echo "[error] $ops_file missing notification contract token: $token" >&2
    exit 1
  }
done

docs_ops_file="docs/02-openapi/ops.yaml"
if [[ ! -f "$docs_ops_file" ]]; then
  echo "[error] missing archive copy: $docs_ops_file" >&2
  exit 1
fi
cmp -s "$ops_file" "$docs_ops_file" || {
  echo "[error] $docs_ops_file is not synced with $ops_file" >&2
  exit 1
}

audit_file="$OPENAPI_DIR/audit.yaml"
for path in \
  "/api/v1/audit/orders/{id}" \
  "/api/v1/audit/traces" \
  "/api/v1/audit/packages/export" \
  "/api/v1/audit/replay-jobs" \
  "/api/v1/audit/replay-jobs/{id}"; do
  grep -q "$path" "$audit_file" || {
    echo "[error] $audit_file missing path: $path" >&2
    exit 1
  }
done

for token in "AUDIT_REPLAY_DRY_RUN_ONLY" "state_replay" "execution_policy"; do
  grep -q "$token" "$audit_file" || {
    echo "[error] $audit_file missing audit replay token: $token" >&2
    exit 1
  }
done

docs_audit_file="docs/02-openapi/audit.yaml"
if [[ ! -f "$docs_audit_file" ]]; then
  echo "[error] missing archive copy: $docs_audit_file" >&2
  exit 1
fi
cmp -s "$audit_file" "$docs_audit_file" || {
  echo "[error] $docs_audit_file is not synced with $audit_file" >&2
  exit 1
}

search_file="$OPENAPI_DIR/search.yaml"
for path in \
  "/api/v1/catalog/search" \
  "/api/v1/ops/search/sync" \
  "/api/v1/ops/search/reindex" \
  "/api/v1/ops/search/aliases/switch" \
  "/api/v1/ops/search/cache/invalidate" \
  "/api/v1/ops/search/ranking-profiles" \
  "/api/v1/ops/search/ranking-profiles/{id}"; do
  grep -q "$path" "$search_file" || {
    echo "[error] $search_file missing path: $path" >&2
    exit 1
  }
done

recommendation_file="$OPENAPI_DIR/recommendation.yaml"
for path in \
  "/api/v1/recommendations" \
  "/api/v1/recommendations/track/exposure" \
  "/api/v1/recommendations/track/click" \
  "/api/v1/ops/recommendation/placements" \
  "/api/v1/ops/recommendation/placements/{placement_code}" \
  "/api/v1/ops/recommendation/ranking-profiles" \
  "/api/v1/ops/recommendation/ranking-profiles/{id}" \
  "/api/v1/ops/recommendation/rebuild"; do
  grep -q "$path" "$recommendation_file" || {
    echo "[error] $recommendation_file missing path: $path" >&2
    exit 1
  }
done

echo "[ok] openapi schema skeleton check passed"
