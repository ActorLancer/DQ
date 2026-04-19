use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationExpectationSnapshot {
    pub grant_scope: String,
    pub access_mode: String,
    pub expected_valid_days: Option<i32>,
    pub export_allowed: bool,
    pub requires_poc_environment: bool,
}

impl Default for AuthorizationExpectationSnapshot {
    fn default() -> Self {
        Self {
            grant_scope: "product_level".to_string(),
            access_mode: "read_only".to_string(),
            expected_valid_days: None,
            export_allowed: false,
            requires_poc_environment: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationModelSnapshot {
    pub scope: AuthorizationScopeSnapshot,
    pub subject: AuthorizationSubjectSnapshot,
    pub resource: AuthorizationResourceSnapshot,
    pub action: AuthorizationActionSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationScopeSnapshot {
    pub scope_type: String,
    pub order_id: String,
    pub product_id: String,
    pub sku_id: String,
    pub sku_type: String,
    pub policy_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationSubjectSnapshot {
    pub subject_type: String,
    pub subject_id: String,
    pub constraints: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationResourceSnapshot {
    pub resource_type: String,
    pub product_id: String,
    pub sku_id: String,
    pub sku_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationActionSnapshot {
    pub grant_type: String,
    pub access_mode: String,
    pub allowed_usage: Vec<String>,
    pub exportable: bool,
}

pub fn build_authorization_model_snapshot(
    order_id: &str,
    product_id: &str,
    sku_id: &str,
    sku_type: &str,
    policy_id: &str,
    granted_to_type: &str,
    granted_to_id: &str,
    grant_type: &str,
    subject_constraints: &Value,
    usage_constraints: &Value,
    exportable: bool,
) -> AuthorizationModelSnapshot {
    AuthorizationModelSnapshot {
        scope: AuthorizationScopeSnapshot {
            scope_type: "order_authorization".to_string(),
            order_id: order_id.to_string(),
            product_id: product_id.to_string(),
            sku_id: sku_id.to_string(),
            sku_type: sku_type.to_string(),
            policy_id: policy_id.to_string(),
        },
        subject: AuthorizationSubjectSnapshot {
            subject_type: granted_to_type.to_string(),
            subject_id: granted_to_id.to_string(),
            constraints: subject_constraints.clone(),
        },
        resource: AuthorizationResourceSnapshot {
            resource_type: "product_sku".to_string(),
            product_id: product_id.to_string(),
            sku_id: sku_id.to_string(),
            sku_type: sku_type.to_string(),
        },
        action: AuthorizationActionSnapshot {
            grant_type: grant_type.to_string(),
            access_mode: derive_access_mode(grant_type, usage_constraints),
            allowed_usage: extract_allowed_usage(usage_constraints),
            exportable,
        },
    }
}

pub fn normalize_policy_snapshot(
    policy_snapshot: Value,
    model: &AuthorizationModelSnapshot,
) -> Value {
    let mut normalized = match policy_snapshot {
        Value::Object(map) => map,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("legacy_policy_snapshot".to_string(), other);
            map
        }
    };
    normalized.insert("scope".to_string(), json!(model.scope));
    normalized.insert("subject".to_string(), json!(model.subject));
    normalized.insert("resource".to_string(), json!(model.resource));
    normalized.insert("action".to_string(), json!(model.action));
    Value::Object(normalized)
}

pub fn extract_or_build_authorization_model(
    policy_snapshot: &Value,
    fallback: AuthorizationModelSnapshot,
) -> AuthorizationModelSnapshot {
    let Some(obj) = policy_snapshot.as_object() else {
        return fallback;
    };
    let scope = obj
        .get("scope")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok());
    let subject = obj
        .get("subject")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok());
    let resource = obj
        .get("resource")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok());
    let action = obj
        .get("action")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok());
    match (scope, subject, resource, action) {
        (Some(scope), Some(subject), Some(resource), Some(action)) => AuthorizationModelSnapshot {
            scope,
            subject,
            resource,
            action,
        },
        _ => fallback,
    }
}

fn extract_allowed_usage(usage_constraints: &Value) -> Vec<String> {
    usage_constraints
        .get("allowed_usage")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn derive_access_mode(grant_type: &str, usage_constraints: &Value) -> String {
    if let Some(access_mode) = usage_constraints.get("access_mode").and_then(Value::as_str) {
        return access_mode.to_string();
    }
    match grant_type {
        "file_access" | "share_grant" | "template_grant" | "report_delivery" => {
            "read_only".to_string()
        }
        "api_access" => "programmatic".to_string(),
        "sandbox_grant" => "controlled_execute".to_string(),
        _ => "read_only".to_string(),
    }
}
