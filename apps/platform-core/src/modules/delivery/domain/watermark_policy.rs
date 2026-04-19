use super::storage_gateway::StorageGatewayWatermarkPolicy;
use serde_json::{Value, json};

const RESERVED_PIPELINE_STAGE: &str = "post_delivery_commit";
const RESERVED_PIPELINE_STATUS: &str = "reserved";
const RESERVED_PIPELINE_PROVIDER: &str = "pending";
const RESERVED_FINGERPRINT_STRATEGY: &str = "reserved_for_pipeline";

pub fn build_watermark_placeholder_patch(
    source_snapshot: &Value,
    delivery_branch: &str,
    metadata: Option<&Value>,
) -> Value {
    let watermark_rule = source_snapshot
        .get("watermark_rule")
        .cloned()
        .or_else(|| {
            source_snapshot
                .get("watermark_policy")
                .and_then(extract_policy_value)
        })
        .or_else(|| {
            metadata
                .and_then(|value| value.get("watermark_rule"))
                .cloned()
        })
        .unwrap_or_else(|| Value::String("reserved_for_pipeline".to_string()));

    let fingerprint_fields = source_snapshot
        .get("fingerprint_fields")
        .cloned()
        .or_else(|| {
            metadata
                .and_then(|value| value.get("fingerprint_fields"))
                .cloned()
        })
        .filter(Value::is_array)
        .unwrap_or_else(|| Value::Array(Vec::new()));

    let watermark_hash = source_snapshot
        .get("watermark_hash")
        .cloned()
        .or_else(|| {
            metadata
                .and_then(|value| value.get("watermark_hash"))
                .cloned()
        })
        .unwrap_or(Value::Null);

    let mode = source_snapshot
        .get("watermark_mode")
        .and_then(Value::as_str)
        .or_else(|| {
            metadata
                .and_then(|value| value.get("watermark_mode"))
                .and_then(Value::as_str)
        })
        .map(str::to_string)
        .unwrap_or_else(|| {
            if source_snapshot.get("watermark_rule").is_some()
                || metadata
                    .and_then(|value| value.get("watermark_rule"))
                    .is_some()
            {
                "rule_bound".to_string()
            } else {
                "placeholder".to_string()
            }
        });

    let pipeline_provider = metadata
        .and_then(|value| value.get("pipeline_provider"))
        .and_then(Value::as_str)
        .unwrap_or(RESERVED_PIPELINE_PROVIDER);
    let pipeline_status = metadata
        .and_then(|value| value.get("pipeline_status"))
        .and_then(Value::as_str)
        .unwrap_or(RESERVED_PIPELINE_STATUS);
    let fingerprint_strategy = metadata
        .and_then(|value| value.get("fingerprint_strategy"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            if fingerprint_fields
                .as_array()
                .map(|items| !items.is_empty())
                .unwrap_or(false)
            {
                "field_bound".to_string()
            } else {
                RESERVED_FINGERPRINT_STRATEGY.to_string()
            }
        });

    let watermark_rule_policy = watermark_rule.clone();
    let fingerprint_fields_policy = fingerprint_fields.clone();
    let watermark_hash_policy = watermark_hash.clone();

    json!({
        "watermark_mode": mode,
        "watermark_rule": watermark_rule,
        "fingerprint_fields": fingerprint_fields,
        "watermark_hash": watermark_hash,
        "watermark_policy": {
            "policy": watermark_rule_policy,
            "delivery_branch": delivery_branch,
            "pipeline": {
                "stage": RESERVED_PIPELINE_STAGE,
                "status": pipeline_status,
                "provider": pipeline_provider,
            },
            "fingerprint_strategy": fingerprint_strategy,
            "fingerprint_fields": fingerprint_fields_policy,
            "watermark_hash": watermark_hash_policy,
        },
    })
}

pub fn merge_snapshot_patch(base: &Value, patch: &Value) -> Value {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            let mut merged = base_map.clone();
            for (key, value) in patch_map {
                merged.insert(key.clone(), value.clone());
            }
            Value::Object(merged)
        }
        (Value::Object(_), _) => base.clone(),
        (_, Value::Object(patch_map)) => Value::Object(patch_map.clone()),
        _ => base.clone(),
    }
}

pub fn derive_storage_gateway_watermark_policy(
    trust_boundary_snapshot: &Value,
    sensitive_delivery_mode: &str,
    disclosure_review_status: &str,
) -> StorageGatewayWatermarkPolicy {
    let policy = trust_boundary_snapshot
        .get("watermark_policy")
        .cloned()
        .or_else(|| {
            trust_boundary_snapshot.get("watermark_rule").map(|value| {
                json!({
                    "policy": value,
                })
            })
        })
        .unwrap_or_else(|| json!({"policy": "reserved_for_pipeline"}));

    let fingerprint_fields = trust_boundary_snapshot
        .get("fingerprint_fields")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mode = trust_boundary_snapshot
        .get("watermark_mode")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            if trust_boundary_snapshot.get("watermark_rule").is_some() {
                "rule_bound".to_string()
            } else {
                "placeholder".to_string()
            }
        });

    let watermark_hash = trust_boundary_snapshot
        .get("watermark_hash")
        .and_then(Value::as_str)
        .map(str::to_string);

    StorageGatewayWatermarkPolicy {
        mode,
        rule: policy,
        fingerprint_fields,
        watermark_hash,
        sensitive_delivery_mode: sensitive_delivery_mode.to_string(),
        disclosure_review_status: disclosure_review_status.to_string(),
    }
}

fn extract_policy_value(policy: &Value) -> Option<Value> {
    match policy {
        Value::Object(items) => items.get("policy").cloned(),
        _ => None,
    }
}
