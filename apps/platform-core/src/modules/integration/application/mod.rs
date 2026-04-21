use crate::modules::integration::events::{
    BuildNotificationRequestInput, NotificationRequestedPayload,
    build_notification_idempotency_key, build_notification_request_payload,
};
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
use db::{Error, GenericClient};

pub struct QueueNotificationRequest<'a> {
    pub aggregate_id: &'a str,
    pub payload: &'a NotificationRequestedPayload,
    pub idempotency_key: &'a str,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

pub async fn queue_notification_request(
    client: &(impl GenericClient + Sync),
    request: QueueNotificationRequest<'_>,
) -> Result<bool, Error> {
    let payload = serde_json::to_value(request.payload).map_err(|err| {
        Error::Bind(format!(
            "encode notification requested payload failed: {err}"
        ))
    })?;
    write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: "notification.dispatch_request",
            aggregate_id: request.aggregate_id,
            event_type: "notification.requested",
            producer_service: "platform-core.integration",
            request_id: request.request_id,
            trace_id: request.trace_id,
            idempotency_key: Some(request.idempotency_key),
            occurred_at: request.payload.source_event.occurred_at.as_deref(),
            business_payload: &payload,
            deduplicate_by_idempotency_key: true,
        },
    )
    .await
}

pub fn prepare_notification_request(
    input: BuildNotificationRequestInput,
) -> (NotificationRequestedPayload, String) {
    let scene = input.scene;
    let audience = input.audience;
    let payload = build_notification_request_payload(input);
    let idempotency_key = build_notification_idempotency_key(
        scene,
        audience,
        &payload.source_event,
        &payload.recipient,
    );
    (payload, idempotency_key)
}
