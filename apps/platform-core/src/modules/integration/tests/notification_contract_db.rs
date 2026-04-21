use crate::modules::integration::application::{
    QueueNotificationRequest, prepare_notification_request, queue_notification_request,
};
use crate::modules::integration::events::{
    BuildNotificationRequestInput, NotificationActionLink, NotificationAudience,
    NotificationRecipient, NotificationScene, NotificationSourceEvent, NotificationSubjectRef,
};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::json;

fn live_db_enabled() -> bool {
    std::env::var("NOTIF_DB_SMOKE").ok().as_deref() == Some("1")
}

#[tokio::test]
async fn notif002_notification_contract_db_smoke() {
    if !live_db_enabled() {
        eprintln!("skip notif002_notification_contract_db_smoke; set NOTIF_DB_SMOKE=1");
        return;
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let (client, connection) = connect(&database_url, NoTls)
        .await
        .expect("connect postgres");
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let seed = seed_requests(&client)
        .await
        .expect("seed notification requests");

    let count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.outbox_event
             WHERE target_topic = 'dtp.notification.dispatch'
               AND aggregate_type = 'notification.dispatch_request'
               AND idempotency_key = ANY($1::text[])",
            &[&seed.idempotency_keys],
        )
        .await
        .expect("count notification outbox rows")
        .get(0);
    assert_eq!(count, NotificationScene::ALL.len() as i64);

    let sample = client
        .query_one(
            "SELECT event_type, target_topic, payload
             FROM ops.outbox_event
             WHERE idempotency_key = $1
             ORDER BY created_at DESC, outbox_event_id DESC
             LIMIT 1",
            &[&seed.sample_key],
        )
        .await
        .expect("query notification outbox");
    let event_type: String = sample.get(0);
    let target_topic: String = sample.get(1);
    let payload: serde_json::Value = sample.get(2);
    assert_eq!(event_type, "notification.requested");
    assert_eq!(target_topic, "dtp.notification.dispatch");
    assert_eq!(
        payload["payload"]["notification_code"].as_str(),
        Some("payment.succeeded")
    );
    assert_eq!(
        payload["payload"]["source_event"]["event_type"].as_str(),
        Some("billing.event.recorded")
    );
    assert_eq!(
        payload["payload"]["subject_refs"][0]["ref_type"].as_str(),
        Some("order")
    );

    cleanup_requests(&client, &seed.idempotency_keys).await;
}

struct SeedState {
    idempotency_keys: Vec<String>,
    sample_key: String,
}

async fn seed_requests(client: &Client) -> Result<SeedState, Error> {
    let aggregate_id = "77777777-7777-7777-7777-777777777777";
    let mut idempotency_keys = Vec::with_capacity(NotificationScene::ALL.len());
    let mut sample_key = String::new();

    for scene in NotificationScene::ALL {
        let source_event = source_event_for_scene(scene, aggregate_id);
        let subject_refs = vec![NotificationSubjectRef {
            ref_type: "order".to_string(),
            ref_id: aggregate_id.to_string(),
        }];
        let links = vec![NotificationActionLink {
            link_code: "order.detail".to_string(),
            href: format!("/orders/{aggregate_id}"),
        }];
        let audience = match scene {
            NotificationScene::PendingDelivery => NotificationAudience::Seller,
            NotificationScene::DisputeEscalated
            | NotificationScene::SettlementFrozen
            | NotificationScene::SettlementResumed => NotificationAudience::Ops,
            _ => NotificationAudience::Buyer,
        };
        let recipient = NotificationRecipient {
            kind: "user".to_string(),
            address: format!("{}@example.test", audience.as_str()),
            id: Some(format!("{}-{}", audience.as_str(), scene.as_str())),
            display_name: Some(audience.as_str().to_ascii_uppercase()),
        };
        let (payload, idempotency_key) =
            prepare_notification_request(BuildNotificationRequestInput {
                scene,
                audience,
                recipient,
                source_event,
                variables: json!({
                    "subject": format!("scene {}", scene.as_str()),
                    "message": format!("notification scene {}", scene.as_str()),
                }),
                metadata: json!({
                    "task_id": "NOTIF-002",
                    "scene": scene.as_str(),
                }),
                retry_policy: None,
                subject_refs,
                links,
                template_code: None,
                channel: None,
            });
        let inserted = queue_notification_request(
            client,
            QueueNotificationRequest {
                aggregate_id,
                payload: &payload,
                idempotency_key: &idempotency_key,
                request_id: Some("req-notif002-db"),
                trace_id: Some("trace-notif002-db"),
            },
        )
        .await?;
        assert!(
            inserted,
            "first insert should succeed for {}",
            scene.as_str()
        );

        let replay_inserted = queue_notification_request(
            client,
            QueueNotificationRequest {
                aggregate_id,
                payload: &payload,
                idempotency_key: &idempotency_key,
                request_id: Some("req-notif002-db"),
                trace_id: Some("trace-notif002-db"),
            },
        )
        .await?;
        assert!(
            !replay_inserted,
            "duplicate insert should be suppressed for {}",
            scene.as_str()
        );

        if scene == NotificationScene::PaymentSucceeded {
            sample_key = idempotency_key.clone();
        }
        idempotency_keys.push(idempotency_key);
    }

    Ok(SeedState {
        idempotency_keys,
        sample_key,
    })
}

fn source_event_for_scene(scene: NotificationScene, aggregate_id: &str) -> NotificationSourceEvent {
    let (aggregate_type, event_type) = match scene {
        NotificationScene::OrderCreated => ("trade.order", "trade.order.created"),
        NotificationScene::PaymentSucceeded
        | NotificationScene::RefundCompleted
        | NotificationScene::CompensationCompleted => {
            ("billing.billing_event", "billing.event.recorded")
        }
        NotificationScene::PaymentFailed => ("payment.payment_intent", "payment.intent_failed"),
        NotificationScene::PendingDelivery => ("trade.order_main", "order.state_changed"),
        NotificationScene::DeliveryCompleted => ("delivery.delivery_record", "delivery.committed"),
        NotificationScene::PendingAcceptance => ("trade.order_main", "order.state_changed"),
        NotificationScene::AcceptancePassed => ("trade.acceptance_record", "acceptance.passed"),
        NotificationScene::AcceptanceRejected => ("trade.acceptance_record", "acceptance.rejected"),
        NotificationScene::DisputeEscalated | NotificationScene::SettlementFrozen => {
            ("support.dispute_case", "dispute.created")
        }
        NotificationScene::SettlementResumed => ("support.dispute_case", "dispute.resolved"),
    };
    NotificationSourceEvent {
        aggregate_type: aggregate_type.to_string(),
        aggregate_id: aggregate_id.to_string(),
        event_type: event_type.to_string(),
        event_id: None,
        target_topic: Some("dtp.outbox.domain-events".to_string()),
        occurred_at: None,
    }
}

async fn cleanup_requests(client: &Client, idempotency_keys: &[String]) {
    let _ = client
        .execute(
            "DELETE FROM ops.outbox_event WHERE idempotency_key = ANY($1::text[])",
            &[&idempotency_keys],
        )
        .await;
}
