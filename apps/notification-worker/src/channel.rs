use chrono::{SecondsFormat, Utc};
use config::{ProviderMode, RuntimeMode};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use tracing::info;

use crate::event::RetryEnvelope;
use crate::template::RenderedNotification;

#[derive(Debug, Clone)]
pub struct ChannelRegistry {
    runtime_mode: RuntimeMode,
    provider_mode: ProviderMode,
    active: BTreeMap<&'static str, NotificationChannelAdapter>,
    reserved: BTreeSet<&'static str>,
}

#[derive(Debug, Clone)]
enum NotificationChannelAdapter {
    MockLog(MockLogChannelAdapter),
}

#[derive(Debug, Clone)]
struct MockLogChannelAdapter;

#[derive(Debug, Clone, Serialize)]
pub struct ChannelSendResult {
    pub channel: String,
    pub adapter_key: String,
    pub runtime_mode: String,
    pub provider_mode: String,
    pub transport_status: String,
    pub backend_message_id: String,
    pub recipient: String,
    pub attempt: u32,
    pub delivered_at: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ChannelSendRequest<'a> {
    pub retry: &'a RetryEnvelope,
    pub rendered: &'a RenderedNotification,
}

impl ChannelRegistry {
    pub fn new(runtime_mode: RuntimeMode, provider_mode: ProviderMode) -> Self {
        let mut active = BTreeMap::new();
        active.insert(
            "mock-log",
            NotificationChannelAdapter::MockLog(MockLogChannelAdapter),
        );

        let reserved = ["email", "webhook"].into_iter().collect();

        Self {
            runtime_mode,
            provider_mode,
            active,
            reserved,
        }
    }

    pub fn active_channels(&self) -> Vec<&'static str> {
        self.active.keys().copied().collect()
    }

    pub fn reserved_channels(&self) -> Vec<&'static str> {
        self.reserved.iter().copied().collect()
    }

    pub async fn send(&self, request: ChannelSendRequest<'_>) -> Result<ChannelSendResult, String> {
        let channel = request.retry.envelope.payload.channel.as_str();
        let adapter = self.active.get(channel).ok_or_else(|| {
            if self.reserved.contains(channel) {
                format!(
                    "notification channel {channel} is reserved boundary only in {} mode",
                    self.runtime_mode.as_str()
                )
            } else {
                format!("notification channel {channel} is not registered")
            }
        })?;

        adapter
            .send(
                self.runtime_mode.as_str(),
                self.provider_mode.as_str(),
                request,
            )
            .await
    }
}

impl NotificationChannelAdapter {
    async fn send(
        &self,
        runtime_mode: &str,
        provider_mode: &str,
        request: ChannelSendRequest<'_>,
    ) -> Result<ChannelSendResult, String> {
        match self {
            Self::MockLog(adapter) => adapter.send(runtime_mode, provider_mode, request).await,
        }
    }
}

impl MockLogChannelAdapter {
    async fn send(
        &self,
        runtime_mode: &str,
        provider_mode: &str,
        request: ChannelSendRequest<'_>,
    ) -> Result<ChannelSendResult, String> {
        let simulate_failures = request.retry.envelope.payload.metadata["simulate_failures"]
            .as_u64()
            .unwrap_or(0) as u32;
        if request.retry.attempt <= simulate_failures {
            return Err(format!(
                "mock-log forced failure on attempt {} for event {}",
                request.retry.attempt, request.retry.envelope.event_id
            ));
        }

        info!(
            event_id = %request.retry.envelope.event_id,
            template_code = %request.rendered.template_code,
            recipient = %request.retry.envelope.payload.recipient.address,
            title = %request.rendered.title,
            body = %request.rendered.body,
            attempt = request.retry.attempt,
            runtime_mode = runtime_mode,
            provider_mode = provider_mode,
            "notification-worker mock-log delivered"
        );

        Ok(ChannelSendResult {
            channel: "mock-log".to_string(),
            adapter_key: "mock-log-adapter".to_string(),
            runtime_mode: runtime_mode.to_string(),
            provider_mode: provider_mode.to_string(),
            transport_status: "delivered".to_string(),
            backend_message_id: format!("mocklog-{}", request.retry.envelope.event_id),
            recipient: request.retry.envelope.payload.recipient.address.clone(),
            attempt: request.retry.attempt,
            delivered_at: now_iso8601(),
        })
    }
}

impl ChannelSendResult {
    pub fn as_json(&self) -> Value {
        json!({
            "channel": self.channel,
            "adapter_key": self.adapter_key,
            "runtime_mode": self.runtime_mode,
            "provider_mode": self.provider_mode,
            "transport_status": self.transport_status,
            "backend_message_id": self.backend_message_id,
            "attempt": self.attempt,
            "recipient": self.recipient,
            "delivered_at": self.delivered_at,
        })
    }
}

fn now_iso8601() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{
        NotificationEnvelope, NotificationRecipient, NotificationRetryPolicy,
        NotificationSourceEvent, RetryEnvelope,
    };
    use crate::template::RenderedNotification;
    use serde_json::json;

    fn sample_retry(channel: &str, simulate_failures: u32) -> RetryEnvelope {
        RetryEnvelope {
            attempt: 1,
            envelope: NotificationEnvelope {
                event_id: "11111111-1111-1111-1111-111111111111".to_string(),
                event_type: "notification.requested".to_string(),
                producer_service: "test".to_string(),
                aggregate_type: "notification.dispatch_request".to_string(),
                aggregate_id: "22222222-2222-2222-2222-222222222222".to_string(),
                request_id: "req-1".to_string(),
                trace_id: "trace-1".to_string(),
                idempotency_key: "idem-1".to_string(),
                occurred_at: now_iso8601(),
                payload: notification_contract::build_notification_request_payload(
                    notification_contract::BuildNotificationRequestInput {
                        scene: notification_contract::NotificationScene::PaymentSucceeded,
                        audience: notification_contract::NotificationAudience::Buyer,
                        recipient: NotificationRecipient {
                            kind: "user".to_string(),
                            address: "buyer@example.test".to_string(),
                            id: Some("33333333-3333-3333-3333-333333333333".to_string()),
                            display_name: Some("Buyer".to_string()),
                        },
                        source_event: NotificationSourceEvent {
                            aggregate_type: "billing.billing_event".to_string(),
                            aggregate_id: "44444444-4444-4444-4444-444444444444".to_string(),
                            event_type: "billing.event.recorded".to_string(),
                            event_id: Some("55555555-5555-5555-5555-555555555555".to_string()),
                            target_topic: Some("dtp.notification.dispatch".to_string()),
                            occurred_at: None,
                        },
                        variables: json!({"subject":"x"}),
                        metadata: json!({"simulate_failures": simulate_failures}),
                        retry_policy: Some(NotificationRetryPolicy {
                            max_attempts: 2,
                            backoff_ms: 1000,
                        }),
                        subject_refs: Vec::new(),
                        links: Vec::new(),
                        template_code: Some("NOTIFY_PAYMENT_SUCCEEDED_V1".to_string()),
                        channel: Some(channel.to_string()),
                    },
                ),
            },
        }
    }

    fn sample_rendered(channel: &str) -> RenderedNotification {
        RenderedNotification {
            template_code: "NOTIFY_PAYMENT_SUCCEEDED_V1".to_string(),
            channel: channel.to_string(),
            language_code: "zh-CN".to_string(),
            requested_language_code: "zh-CN".to_string(),
            version_no: 2,
            template_enabled: true,
            template_status: "active".to_string(),
            template_fallback_used: false,
            body_fallback_used: false,
            variable_schema: json!({}),
            template_metadata: json!({}),
            title: "ok".to_string(),
            body: "ok".to_string(),
        }
    }

    #[tokio::test]
    async fn registry_enables_mock_log_and_reserves_email_webhook() {
        let registry = ChannelRegistry::new(RuntimeMode::Local, ProviderMode::Mock);

        assert_eq!(registry.active_channels(), vec!["mock-log"]);
        assert_eq!(registry.reserved_channels(), vec!["email", "webhook"]);

        let result = registry
            .send(ChannelSendRequest {
                retry: &sample_retry("mock-log", 0),
                rendered: &sample_rendered("mock-log"),
            })
            .await
            .expect("mock-log should be active");

        assert_eq!(result.channel, "mock-log");
        assert_eq!(result.adapter_key, "mock-log-adapter");
        assert_eq!(result.transport_status, "delivered");
    }

    #[tokio::test]
    async fn reserved_channel_returns_boundary_error() {
        let registry = ChannelRegistry::new(RuntimeMode::Local, ProviderMode::Mock);

        let error = registry
            .send(ChannelSendRequest {
                retry: &sample_retry("email", 0),
                rendered: &sample_rendered("email"),
            })
            .await
            .expect_err("email should stay boundary-only in local mode");

        assert!(error.contains("reserved boundary only"));
    }

    #[tokio::test]
    async fn mock_log_adapter_supports_failure_injection() {
        let registry = ChannelRegistry::new(RuntimeMode::Local, ProviderMode::Mock);

        let error = registry
            .send(ChannelSendRequest {
                retry: &sample_retry("mock-log", 1),
                rendered: &sample_rendered("mock-log"),
            })
            .await
            .expect_err("failure injection should still work");

        assert!(error.contains("mock-log forced failure"));
    }
}
