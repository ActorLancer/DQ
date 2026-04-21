use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationScene {
    OrderCreated,
    PaymentSucceeded,
    PaymentFailed,
    PendingDelivery,
    DeliveryCompleted,
    PendingAcceptance,
    AcceptancePassed,
    AcceptanceRejected,
    DisputeEscalated,
    RefundCompleted,
    CompensationCompleted,
    SettlementFrozen,
    SettlementResumed,
}

impl NotificationScene {
    pub const ALL: [Self; 13] = [
        Self::OrderCreated,
        Self::PaymentSucceeded,
        Self::PaymentFailed,
        Self::PendingDelivery,
        Self::DeliveryCompleted,
        Self::PendingAcceptance,
        Self::AcceptancePassed,
        Self::AcceptanceRejected,
        Self::DisputeEscalated,
        Self::RefundCompleted,
        Self::CompensationCompleted,
        Self::SettlementFrozen,
        Self::SettlementResumed,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OrderCreated => "order.created",
            Self::PaymentSucceeded => "payment.succeeded",
            Self::PaymentFailed => "payment.failed",
            Self::PendingDelivery => "order.pending_delivery",
            Self::DeliveryCompleted => "delivery.completed",
            Self::PendingAcceptance => "order.pending_acceptance",
            Self::AcceptancePassed => "acceptance.passed",
            Self::AcceptanceRejected => "acceptance.rejected",
            Self::DisputeEscalated => "dispute.escalated",
            Self::RefundCompleted => "refund.completed",
            Self::CompensationCompleted => "compensation.completed",
            Self::SettlementFrozen => "settlement.frozen",
            Self::SettlementResumed => "settlement.resumed",
        }
    }

    pub fn default_template_code(self) -> &'static str {
        match self {
            Self::OrderCreated => "NOTIFY_ORDER_CREATED_V1",
            Self::PaymentSucceeded => "NOTIFY_PAYMENT_SUCCEEDED_V1",
            Self::PaymentFailed => "NOTIFY_PAYMENT_FAILED_V1",
            Self::PendingDelivery => "NOTIFY_PENDING_DELIVERY_V1",
            Self::DeliveryCompleted => "NOTIFY_DELIVERY_COMPLETED_V1",
            Self::PendingAcceptance => "NOTIFY_PENDING_ACCEPTANCE_V1",
            Self::AcceptancePassed => "NOTIFY_ACCEPTANCE_PASSED_V1",
            Self::AcceptanceRejected => "NOTIFY_ACCEPTANCE_REJECTED_V1",
            Self::DisputeEscalated => "NOTIFY_DISPUTE_ESCALATED_V1",
            Self::RefundCompleted => "NOTIFY_REFUND_COMPLETED_V1",
            Self::CompensationCompleted => "NOTIFY_COMPENSATION_COMPLETED_V1",
            Self::SettlementFrozen => "NOTIFY_SETTLEMENT_FROZEN_V1",
            Self::SettlementResumed => "NOTIFY_SETTLEMENT_RESUMED_V1",
        }
    }
}

impl FromStr for NotificationScene {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "order.created" => Ok(Self::OrderCreated),
            "payment.succeeded" => Ok(Self::PaymentSucceeded),
            "payment.failed" => Ok(Self::PaymentFailed),
            "order.pending_delivery" => Ok(Self::PendingDelivery),
            "delivery.completed" => Ok(Self::DeliveryCompleted),
            "order.pending_acceptance" => Ok(Self::PendingAcceptance),
            "acceptance.passed" => Ok(Self::AcceptancePassed),
            "acceptance.rejected" => Ok(Self::AcceptanceRejected),
            "dispute.escalated" => Ok(Self::DisputeEscalated),
            "refund.completed" => Ok(Self::RefundCompleted),
            "compensation.completed" => Ok(Self::CompensationCompleted),
            "settlement.frozen" => Ok(Self::SettlementFrozen),
            "settlement.resumed" => Ok(Self::SettlementResumed),
            other => Err(format!("unsupported notification scene: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationAudience {
    Buyer,
    Seller,
    Ops,
}

impl NotificationAudience {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buyer => "buyer",
            Self::Seller => "seller",
            Self::Ops => "ops",
        }
    }
}

impl FromStr for NotificationAudience {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "buyer" => Ok(Self::Buyer),
            "seller" => Ok(Self::Seller),
            "ops" => Ok(Self::Ops),
            other => Err(format!("unsupported notification audience: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationRetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationRecipient {
    pub kind: String,
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationSourceEvent {
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_topic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationSubjectRef {
    pub ref_type: String,
    pub ref_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationActionLink {
    pub link_code: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationRequestedPayload {
    pub notification_code: String,
    pub template_code: String,
    pub channel: String,
    pub audience_scope: String,
    pub recipient: NotificationRecipient,
    pub source_event: NotificationSourceEvent,
    #[serde(default = "empty_json_object")]
    pub variables: Value,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<NotificationRetryPolicy>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subject_refs: Vec<NotificationSubjectRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<NotificationActionLink>,
}

#[derive(Debug, Clone)]
pub struct BuildNotificationRequestInput {
    pub scene: NotificationScene,
    pub audience: NotificationAudience,
    pub recipient: NotificationRecipient,
    pub source_event: NotificationSourceEvent,
    pub variables: Value,
    pub metadata: Value,
    pub retry_policy: Option<NotificationRetryPolicy>,
    pub subject_refs: Vec<NotificationSubjectRef>,
    pub links: Vec<NotificationActionLink>,
    pub template_code: Option<String>,
    pub channel: Option<String>,
}

pub fn build_notification_request_payload(
    input: BuildNotificationRequestInput,
) -> NotificationRequestedPayload {
    NotificationRequestedPayload {
        notification_code: input.scene.as_str().to_string(),
        template_code: input
            .template_code
            .unwrap_or_else(|| input.scene.default_template_code().to_string()),
        channel: input.channel.unwrap_or_else(|| "mock-log".to_string()),
        audience_scope: input.audience.as_str().to_string(),
        recipient: input.recipient,
        source_event: input.source_event,
        variables: normalize_object(input.variables),
        metadata: normalize_object(input.metadata),
        retry_policy: input.retry_policy,
        subject_refs: input.subject_refs,
        links: input.links,
    }
}

pub fn build_notification_idempotency_key(
    scene: NotificationScene,
    audience: NotificationAudience,
    source_event: &NotificationSourceEvent,
    recipient: &NotificationRecipient,
) -> String {
    let recipient_ref = recipient
        .id
        .as_deref()
        .unwrap_or(recipient.address.as_str());
    format!(
        "notification:{}:{}:{}:{}:{}:{}",
        canonical_key_fragment(scene.as_str()),
        canonical_key_fragment(audience.as_str()),
        canonical_key_fragment(source_event.aggregate_type.as_str()),
        canonical_key_fragment(source_event.aggregate_id.as_str()),
        canonical_key_fragment(source_event.event_type.as_str()),
        canonical_key_fragment(recipient_ref),
    )
}

pub fn empty_json_object() -> Value {
    json!({})
}

fn normalize_object(value: Value) -> Value {
    if value.is_object() {
        value
    } else {
        empty_json_object()
    }
}

fn canonical_key_fragment(raw: &str) -> String {
    raw.trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_catalog_matches_notif002_frozen_scope() {
        let scenes = NotificationScene::ALL.map(NotificationScene::as_str);
        assert_eq!(
            scenes,
            [
                "order.created",
                "payment.succeeded",
                "payment.failed",
                "order.pending_delivery",
                "delivery.completed",
                "order.pending_acceptance",
                "acceptance.passed",
                "acceptance.rejected",
                "dispute.escalated",
                "refund.completed",
                "compensation.completed",
                "settlement.frozen",
                "settlement.resumed",
            ]
        );
    }

    #[test]
    fn builds_stable_idempotency_key() {
        let key = build_notification_idempotency_key(
            NotificationScene::PendingDelivery,
            NotificationAudience::Seller,
            &NotificationSourceEvent {
                aggregate_type: "trade.order".to_string(),
                aggregate_id: "11111111-1111-1111-1111-111111111111".to_string(),
                event_type: "trade.order.created".to_string(),
                event_id: None,
                target_topic: None,
                occurred_at: None,
            },
            &NotificationRecipient {
                kind: "user".to_string(),
                address: "seller@example.test".to_string(),
                id: Some("seller-user".to_string()),
                display_name: None,
            },
        );

        assert_eq!(
            key,
            "notification:order.pending_delivery:seller:trade.order:11111111-1111-1111-1111-111111111111:trade.order.created:seller-user"
        );
    }

    #[test]
    fn build_payload_defaults_template_and_channel() {
        let payload = build_notification_request_payload(BuildNotificationRequestInput {
            scene: NotificationScene::PaymentSucceeded,
            audience: NotificationAudience::Buyer,
            recipient: NotificationRecipient {
                kind: "user".to_string(),
                address: "buyer@example.test".to_string(),
                id: None,
                display_name: Some("Buyer".to_string()),
            },
            source_event: NotificationSourceEvent {
                aggregate_type: "billing.billing_event".to_string(),
                aggregate_id: "22222222-2222-2222-2222-222222222222".to_string(),
                event_type: "billing.event.recorded".to_string(),
                event_id: None,
                target_topic: Some("dtp.outbox.domain-events".to_string()),
                occurred_at: None,
            },
            variables: json!({"subject":"paid"}),
            metadata: json!({"order_id":"33333333-3333-3333-3333-333333333333"}),
            retry_policy: Some(NotificationRetryPolicy {
                max_attempts: 3,
                backoff_ms: 250,
            }),
            subject_refs: vec![NotificationSubjectRef {
                ref_type: "order".to_string(),
                ref_id: "33333333-3333-3333-3333-333333333333".to_string(),
            }],
            links: vec![],
            template_code: None,
            channel: None,
        });

        assert_eq!(payload.notification_code, "payment.succeeded");
        assert_eq!(payload.template_code, "NOTIFY_PAYMENT_SUCCEEDED_V1");
        assert_eq!(payload.channel, "mock-log");
        assert_eq!(payload.audience_scope, "buyer");
        assert_eq!(payload.subject_refs.len(), 1);
    }
}
