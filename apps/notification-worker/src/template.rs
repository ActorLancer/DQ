use crate::event::{NotificationEnvelope, NotificationPayload};
use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationTemplate {
    pub template_code: String,
    pub channel: String,
    pub title_template: String,
    pub body_template: String,
    pub fallback_body_template: String,
}

#[derive(Debug, Clone)]
pub struct RenderedNotification {
    pub template_code: String,
    pub channel: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct TemplateStore {
    root: PathBuf,
}

impl TemplateStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn load(&self, template_code: &str) -> Result<NotificationTemplate, String> {
        let preferred = self.root.join(format!("{template_code}.json"));
        let fallback = self.root.join("DEFAULT_NOTIFICATION_V1.json");
        let path = if preferred.exists() {
            preferred
        } else {
            fallback
        };
        load_template_from_path(&path)
    }

    pub fn render(
        &self,
        envelope: &NotificationEnvelope,
        payload: &NotificationPayload,
    ) -> Result<RenderedNotification, String> {
        let template = self.load(payload.template_code.as_str())?;
        let context = build_render_context(envelope, payload);
        let renderer = Handlebars::new();
        let title = renderer
            .render_template(&template.title_template, &context)
            .map_err(|err| format!("render title template failed: {err}"))?;
        let body = renderer
            .render_template(&template.body_template, &context)
            .or_else(|_| renderer.render_template(&template.fallback_body_template, &context))
            .map_err(|err| format!("render body template failed: {err}"))?;

        Ok(RenderedNotification {
            template_code: template.template_code,
            channel: template.channel,
            title,
            body,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{NotificationRecipient, SendNotificationRequest};
    use std::path::PathBuf;

    #[test]
    fn render_uses_default_template_directory() {
        let store = TemplateStore::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates"));
        let envelope = SendNotificationRequest {
            event_id: None,
            aggregate_type: None,
            aggregate_id: Some("e8d5c8c5-71cb-45f6-96f2-717e59a3e8c0".to_string()),
            template_code: "NOTIFY_GENERIC_V1".to_string(),
            channel: Some("mock-log".to_string()),
            recipient: NotificationRecipient {
                kind: "user".to_string(),
                address: "buyer@example.test".to_string(),
                id: None,
                display_name: Some("Buyer".to_string()),
            },
            variables: Some(json!({
                "subject": "Payment succeeded",
                "message": "Order is waiting for delivery"
            })),
            metadata: None,
            idempotency_key: None,
            request_id: Some("req-1".to_string()),
            trace_id: Some("trace-1".to_string()),
            retry_policy: None,
            simulate_failures: None,
        }
        .into_envelope();

        let rendered = store
            .render(&envelope, &envelope.payload)
            .expect("template should render");

        assert_eq!(rendered.template_code, "NOTIFY_GENERIC_V1");
        assert!(rendered.title.contains("Payment succeeded"));
        assert!(rendered.body.contains("Buyer"));
    }
}
