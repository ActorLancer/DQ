use crate::event::{NotificationEnvelope, NotificationPayload};
use db::GenericClient;
#[cfg(test)]
use db::{AppDb, DbPoolConfig};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_TEMPLATE_CODE: &str = "DEFAULT_NOTIFICATION_V1";
const DEFAULT_LANGUAGE_CODE: &str = "zh-CN";

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationTemplate {
    pub template_code: String,
    #[serde(default = "default_language_code")]
    pub language_code: String,
    pub channel: String,
    #[serde(default = "default_version_no")]
    pub version_no: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "empty_json_object")]
    pub variables_schema_json: Value,
    pub title_template: String,
    pub body_template: String,
    pub fallback_body_template: String,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderedNotification {
    pub template_code: String,
    pub channel: String,
    pub language_code: String,
    pub requested_language_code: String,
    pub version_no: i32,
    pub template_enabled: bool,
    pub template_status: String,
    pub template_fallback_used: bool,
    pub body_fallback_used: bool,
    pub variable_schema: Value,
    pub template_metadata: Value,
    pub title: String,
    pub body: String,
}

impl RenderedNotification {
    pub fn placeholder(payload: &NotificationPayload) -> Self {
        Self {
            template_code: payload.template_code.clone(),
            channel: payload.channel.clone(),
            language_code: requested_language_code(payload)
                .unwrap_or_else(|| DEFAULT_LANGUAGE_CODE.to_string()),
            requested_language_code: requested_language_code(payload)
                .unwrap_or_else(|| DEFAULT_LANGUAGE_CODE.to_string()),
            version_no: 0,
            template_enabled: false,
            template_status: "unknown".to_string(),
            template_fallback_used: false,
            body_fallback_used: false,
            variable_schema: empty_json_object(),
            template_metadata: empty_json_object(),
            title: String::new(),
            body: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedNotificationTemplate {
    template: NotificationTemplate,
    requested_language_code: String,
    template_fallback_used: bool,
}

#[derive(Debug, Clone)]
pub struct TemplateStore {
    root: PathBuf,
    default_language_code: String,
}

impl TemplateStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            default_language_code: DEFAULT_LANGUAGE_CODE.to_string(),
        }
    }

    pub async fn render(
        &self,
        client: &(impl GenericClient + Sync),
        envelope: &NotificationEnvelope,
        payload: &NotificationPayload,
    ) -> Result<RenderedNotification, String> {
        let requested_language_code =
            requested_language_code(payload).unwrap_or_else(|| self.default_language_code.clone());
        let resolved = self
            .load(
                client,
                payload.template_code.as_str(),
                payload.channel.as_str(),
                Some(requested_language_code.as_str()),
            )
            .await?;
        validate_variables_against_schema(
            &resolved.template.variables_schema_json,
            &payload.variables,
        )?;
        let context = build_render_context(envelope, payload);
        let (title, body, body_fallback_used) =
            render_template_body(&resolved.template, &context, &payload.notification_code)?;

        Ok(RenderedNotification {
            template_code: resolved.template.template_code,
            channel: resolved.template.channel,
            language_code: resolved.template.language_code,
            requested_language_code: resolved.requested_language_code,
            version_no: resolved.template.version_no,
            template_enabled: resolved.template.enabled,
            template_status: resolved.template.status,
            template_fallback_used: resolved.template_fallback_used,
            body_fallback_used,
            variable_schema: resolved.template.variables_schema_json,
            template_metadata: resolved.template.metadata,
            title,
            body,
        })
    }

    async fn load(
        &self,
        client: &(impl GenericClient + Sync),
        template_code: &str,
        channel: &str,
        language_code: Option<&str>,
    ) -> Result<ResolvedNotificationTemplate, String> {
        let requested_language_code = normalize_language_code(language_code)
            .unwrap_or_else(|| self.default_language_code.clone());

        if let Some(template) = self
            .load_from_db(
                client,
                template_code,
                channel,
                requested_language_code.as_str(),
            )
            .await?
        {
            return Ok(ResolvedNotificationTemplate {
                template,
                requested_language_code,
                template_fallback_used: false,
            });
        }

        if requested_language_code != self.default_language_code {
            if let Some(template) = self
                .load_from_db(
                    client,
                    template_code,
                    channel,
                    self.default_language_code.as_str(),
                )
                .await?
            {
                return Ok(ResolvedNotificationTemplate {
                    template,
                    requested_language_code,
                    template_fallback_used: false,
                });
            }
        }

        if template_code != DEFAULT_TEMPLATE_CODE {
            if let Some(template) = self
                .load_from_db(
                    client,
                    DEFAULT_TEMPLATE_CODE,
                    channel,
                    self.default_language_code.as_str(),
                )
                .await?
            {
                return Ok(ResolvedNotificationTemplate {
                    template,
                    requested_language_code,
                    template_fallback_used: true,
                });
            }
        }

        let template = self.load_from_file(template_code)?;
        let template_fallback_used = template.template_code != template_code;
        Ok(ResolvedNotificationTemplate {
            template,
            requested_language_code,
            template_fallback_used,
        })
    }

    async fn load_from_db(
        &self,
        client: &(impl GenericClient + Sync),
        template_code: &str,
        channel: &str,
        language_code: &str,
    ) -> Result<Option<NotificationTemplate>, String> {
        let row = client
            .query_opt(
                "SELECT template_code,
                        language_code,
                        channel,
                        version_no,
                        enabled,
                        status,
                        variables_schema_json,
                        title_template,
                        body_template,
                        fallback_body_template,
                        metadata
                 FROM ops.notification_template
                 WHERE template_code = $1
                   AND channel = $2
                   AND language_code = $3
                   AND enabled = TRUE
                   AND status = 'active'
                 ORDER BY version_no DESC, created_at DESC
                 LIMIT 1",
                &[&template_code, &channel, &language_code],
            )
            .await
            .map_err(|err| format!("load notification template from database failed: {err}"))?;

        Ok(row.map(|row| NotificationTemplate {
            template_code: row.get("template_code"),
            language_code: row.get("language_code"),
            channel: row.get("channel"),
            version_no: row.get("version_no"),
            enabled: row.get("enabled"),
            status: row.get("status"),
            variables_schema_json: row.get("variables_schema_json"),
            title_template: row.get("title_template"),
            body_template: row.get("body_template"),
            fallback_body_template: row.get("fallback_body_template"),
            metadata: row.get("metadata"),
        }))
    }

    fn load_from_file(&self, template_code: &str) -> Result<NotificationTemplate, String> {
        let preferred = self.root.join(format!("{template_code}.json"));
        let fallback = self.root.join(format!("{DEFAULT_TEMPLATE_CODE}.json"));
        let path = if preferred.exists() {
            preferred
        } else {
            fallback
        };
        load_template_from_path(&path)
    }
}

fn requested_language_code(payload: &NotificationPayload) -> Option<String> {
    payload
        .metadata
        .get("language_code")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn normalize_language_code(language_code: Option<&str>) -> Option<String> {
    language_code
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn default_language_code() -> String {
    DEFAULT_LANGUAGE_CODE.to_string()
}

fn default_version_no() -> i32 {
    1
}

fn default_enabled() -> bool {
    true
}

fn default_status() -> String {
    "active".to_string()
}

pub fn empty_json_object() -> Value {
    json!({})
}

fn render_template_body(
    template: &NotificationTemplate,
    context: &Value,
    notification_code: &str,
) -> Result<(String, String, bool), String> {
    let mut renderer = Handlebars::new();
    renderer.set_strict_mode(true);

    let title = renderer
        .render_template(&template.title_template, context)
        .unwrap_or_else(|_| format!("Notification {notification_code}"));
    match renderer.render_template(&template.body_template, context) {
        Ok(body) => Ok((title, body, false)),
        Err(_) => {
            let fallback_body = renderer
                .render_template(&template.fallback_body_template, context)
                .map_err(|err| format!("render notification fallback body failed: {err}"))?;
            Ok((title, fallback_body, true))
        }
    }
}

fn load_template_from_path(path: &Path) -> Result<NotificationTemplate, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("read template {} failed: {err}", path.display()))?;
    serde_json::from_str::<NotificationTemplate>(&raw)
        .map_err(|err| format!("decode template {} failed: {err}", path.display()))
}

fn build_render_context(envelope: &NotificationEnvelope, payload: &NotificationPayload) -> Value {
    json!({
        "notification": {
            "notification_code": payload.notification_code,
            "template_code": payload.template_code,
            "channel": payload.channel,
            "audience_scope": payload.audience_scope,
            "subject_refs": payload.subject_refs,
            "links": payload.links,
        },
        "event": {
            "event_id": envelope.event_id,
            "event_type": envelope.event_type,
            "aggregate_type": envelope.aggregate_type,
            "aggregate_id": envelope.aggregate_id,
            "request_id": envelope.request_id,
            "trace_id": envelope.trace_id,
            "producer_service": envelope.producer_service,
            "occurred_at": envelope.occurred_at,
        },
        "source_event": payload.source_event,
        "recipient": {
            "kind": payload.recipient.kind,
            "id": payload.recipient.id,
            "address": payload.recipient.address,
            "display_name": payload.recipient.display_name,
        },
        "variables": payload.variables,
        "metadata": payload.metadata,
    })
}

fn validate_variables_against_schema(schema: &Value, variables: &Value) -> Result<(), String> {
    if !schema.is_object() || schema.as_object().is_some_and(|schema| schema.is_empty()) {
        return Ok(());
    }
    validate_schema_fragment("variables", schema, variables)
}

fn validate_schema_fragment(path: &str, schema: &Value, value: &Value) -> Result<(), String> {
    if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
        validate_json_type(path, expected_type, value)?;
    }

    if let Some(required_fields) = schema.get("required").and_then(Value::as_array) {
        let object = value
            .as_object()
            .ok_or_else(|| format!("{path} must be an object"))?;
        for field in required_fields {
            let Some(field_name) = field.as_str() else {
                continue;
            };
            if !object.contains_key(field_name) {
                return Err(format!(
                    "{path}.{field_name} is required by template schema"
                ));
            }
        }
    }

    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        let Some(object) = value.as_object() else {
            return Ok(());
        };
        for (field_name, field_schema) in properties {
            if let Some(field_value) = object.get(field_name) {
                validate_schema_fragment(
                    format!("{path}.{field_name}").as_str(),
                    field_schema,
                    field_value,
                )?;
            }
        }
    }

    if let Some(item_schema) = schema.get("items") {
        if let Some(items) = value.as_array() {
            for (index, item) in items.iter().enumerate() {
                validate_schema_fragment(format!("{path}[{index}]").as_str(), item_schema, item)?;
            }
        }
    }

    Ok(())
}

fn validate_json_type(path: &str, expected_type: &str, value: &Value) -> Result<(), String> {
    let matched = match expected_type {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => true,
    };
    if matched {
        Ok(())
    } else {
        Err(format!("{path} must match schema type {expected_type}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{NotificationRecipient, NotificationSourceEvent, SendNotificationRequest};

    fn fixture_store() -> TemplateStore {
        TemplateStore::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates"))
    }

    #[test]
    fn load_file_template_supports_legacy_shape_defaults() {
        let template = fixture_store()
            .load_from_file("NOTIFY_GENERIC_V1")
            .expect("legacy file template should decode");

        assert_eq!(template.language_code, "zh-CN");
        assert_eq!(template.version_no, 1);
        assert!(template.enabled);
        assert_eq!(template.status, "active");
    }

    #[test]
    fn schema_validation_rejects_missing_required_fields() {
        let schema = json!({
            "type": "object",
            "required": ["subject"],
            "properties": {
                "subject": { "type": "string" },
                "message": { "type": "string" }
            }
        });

        let error = validate_variables_against_schema(&schema, &json!({}))
            .expect_err("schema validation should reject missing subject");
        assert!(error.contains("variables.subject is required"));
    }

    #[test]
    fn render_body_uses_fallback_copy_when_message_missing() {
        let template = NotificationTemplate {
            template_code: "NOTIFY_TEST_V1".to_string(),
            language_code: "zh-CN".to_string(),
            channel: "mock-log".to_string(),
            version_no: 1,
            enabled: true,
            status: "active".to_string(),
            variables_schema_json: json!({
                "type": "object",
                "required": ["subject"],
                "properties": {
                    "subject": { "type": "string" },
                    "message": { "type": "string" }
                }
            }),
            title_template: "{{variables.subject}}".to_string(),
            body_template: "{{variables.message}}".to_string(),
            fallback_body_template: "fallback to {{recipient.address}}".to_string(),
            metadata: empty_json_object(),
        };
        let context = json!({
            "recipient": { "address": "buyer@example.test" },
            "variables": { "subject": "Hello" }
        });

        let (title, body, body_fallback_used) =
            render_template_body(&template, &context, "payment.succeeded")
                .expect("render should fall back to fallback body");

        assert_eq!(title, "Hello");
        assert_eq!(body, "fallback to buyer@example.test");
        assert!(body_fallback_used);
    }

    fn live_db_enabled() -> bool {
        std::env::var("NOTIF_TEMPLATE_DB_SMOKE").ok().as_deref() == Some("1")
    }

    #[tokio::test]
    async fn notif003_template_model_db_smoke() {
        if !live_db_enabled() {
            eprintln!("skip notif003_template_model_db_smoke; set NOTIF_TEMPLATE_DB_SMOKE=1");
            return;
        }

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let db = AppDb::connect(
            DbPoolConfig {
                dsn: database_url,
                max_connections: 4,
            }
            .into(),
        )
        .await
        .expect("connect app db");
        let client = db.client().expect("acquire db client");
        let store = fixture_store();

        let count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.notification_template
                 WHERE enabled = TRUE
                   AND status = 'active'",
                &[],
            )
            .await
            .expect("count active templates")
            .get(0);
        assert!(count >= 15, "expected seeded notification templates");

        let envelope = SendNotificationRequest {
            event_id: Some("55555555-5555-5555-5555-555555555555".to_string()),
            aggregate_id: Some("66666666-6666-6666-6666-666666666666".to_string()),
            notification_code: Some("payment.succeeded".to_string()),
            audience_scope: Some("buyer".to_string()),
            template_code: Some("NOTIFY_PAYMENT_SUCCEEDED_V1".to_string()),
            channel: Some("mock-log".to_string()),
            recipient: NotificationRecipient {
                kind: "user".to_string(),
                address: "buyer.preview@example.test".to_string(),
                id: Some("buyer-preview".to_string()),
                display_name: Some("Buyer Preview".to_string()),
            },
            variables: Some(json!({
                "subject": "Payment succeeded"
            })),
            metadata: Some(json!({
                "language_code": "en-US"
            })),
            source_event: Some(NotificationSourceEvent {
                aggregate_type: "billing.billing_event".to_string(),
                aggregate_id: "66666666-6666-6666-6666-666666666666".to_string(),
                event_type: "billing.event.recorded".to_string(),
                event_id: Some("77777777-7777-7777-7777-777777777777".to_string()),
                target_topic: Some("dtp.outbox.domain-events".to_string()),
                occurred_at: None,
            }),
            subject_refs: None,
            links: None,
            idempotency_key: None,
            request_id: Some("req-notif003-preview".to_string()),
            trace_id: Some("trace-notif003-preview".to_string()),
            retry_policy: None,
            simulate_failures: None,
        }
        .into_envelope()
        .expect("build preview envelope");

        let rendered = store
            .render(&client, &envelope, &envelope.payload)
            .await
            .expect("render template from db");

        assert_eq!(rendered.template_code, "NOTIFY_PAYMENT_SUCCEEDED_V1");
        assert_eq!(rendered.channel, "mock-log");
        assert_eq!(rendered.language_code, "zh-CN");
        assert_eq!(rendered.requested_language_code, "en-US");
        assert_eq!(rendered.version_no, 1);
        assert!(rendered.body_fallback_used);
        assert_eq!(rendered.variable_schema["required"][0], "subject");
    }
}
