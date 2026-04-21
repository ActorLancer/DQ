use chrono::{SecondsFormat, Utc};
use kernel::new_uuid_string;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub producer_service: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub request_id: String,
    pub trace_id: String,
    pub idempotency_key: String,
    pub occurred_at: String,
    pub payload: NotificationPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub template_code: String,
    pub channel: String,
    pub recipient: NotificationRecipient,
    #[serde(default = "empty_json_object")]
    pub variables: Value,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
    pub retry_policy: Option<NotificationRetryPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecipient {
    pub kind: String,
    pub address: String,
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryEnvelope {
    pub attempt: u32,
    pub envelope: NotificationEnvelope,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendNotificationRequest {
    pub event_id: Option<String>,
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<String>,
    pub template_code: String,
    pub channel: Option<String>,
    pub recipient: NotificationRecipient,
    #[serde(default)]
    pub variables: Option<Value>,
    #[serde(default)]
    pub metadata: Option<Value>,
    pub idempotency_key: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub retry_policy: Option<NotificationRetryPolicy>,
    pub simulate_failures: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendNotificationResponse {
    pub event_id: String,
    pub event_type: String,
    pub topic: String,
    pub request_id: String,
    pub trace_id: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
}

impl SendNotificationRequest {
    pub fn into_envelope(self) -> NotificationEnvelope {
        let request_id = self.request_id.unwrap_or_else(new_uuid_string);
        let trace_id = self.trace_id.unwrap_or_else(|| request_id.clone());
        let idempotency_key = self.idempotency_key.unwrap_or_else(new_uuid_string);
        let aggregate_id = self.aggregate_id.unwrap_or_else(new_uuid_string);
        let mut metadata = self.metadata.unwrap_or_else(empty_json_object);
        if let Some(simulate_failures) = self.simulate_failures {
            let object = ensure_json_object(&mut metadata);
            object.insert("simulate_failures".to_string(), json!(simulate_failures));
        }

        NotificationEnvelope {
            event_id: self.event_id.unwrap_or_else(new_uuid_string),
            event_type: "notification.requested".to_string(),
            producer_service: "notification-worker.internal".to_string(),
            aggregate_type: self
                .aggregate_type
                .unwrap_or_else(|| "notification.dispatch_request".to_string()),
            aggregate_id,
            request_id,
            trace_id,
            idempotency_key,
            occurred_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            payload: NotificationPayload {
                template_code: self.template_code,
                channel: self.channel.unwrap_or_else(|| "mock-log".to_string()),
                recipient: self.recipient,
                variables: self.variables.unwrap_or_else(empty_json_object),
                metadata,
                retry_policy: self.retry_policy,
            },
        }
    }
}

impl SendNotificationResponse {
    pub fn from_envelope(topic: impl Into<String>, envelope: &NotificationEnvelope) -> Self {
        Self {
            event_id: envelope.event_id.clone(),
            event_type: envelope.event_type.clone(),
            topic: topic.into(),
            request_id: envelope.request_id.clone(),
            trace_id: envelope.trace_id.clone(),
            aggregate_id: envelope.aggregate_id.clone(),
            idempotency_key: envelope.idempotency_key.clone(),
        }
    }
}

pub fn empty_json_object() -> Value {
    json!({})
}

fn ensure_json_object(value: &mut Value) -> &mut serde_json::Map<String, Value> {
    if !value.is_object() {
        *value = json!({});
    }
    value.as_object_mut().expect("json object just initialized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_request_defaults_to_frozen_notification_shape() {
        let envelope = SendNotificationRequest {
            event_id: None,
            aggregate_type: None,
            aggregate_id: None,
            template_code: "NOTIFY_GENERIC_V1".to_string(),
            channel: None,
            recipient: NotificationRecipient {
                kind: "user".to_string(),
                address: "buyer@example.test".to_string(),
                id: None,
                display_name: Some("Buyer".to_string()),
            },
            variables: None,
            metadata: None,
            idempotency_key: None,
            request_id: None,
            trace_id: None,
            retry_policy: None,
            simulate_failures: Some(2),
        }
        .into_envelope();

        assert_eq!(envelope.event_type, "notification.requested");
        assert_eq!(envelope.aggregate_type, "notification.dispatch_request");
        assert_eq!(envelope.payload.channel, "mock-log");
        assert_eq!(envelope.payload.metadata["simulate_failures"], 2);
    }
}
