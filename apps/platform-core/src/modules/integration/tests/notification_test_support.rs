use db::{Client, GenericClient};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

pub fn test027_live_chain_enabled() -> bool {
    std::env::var("TEST027_LIVE_CHAIN").ok().as_deref() == Some("1")
}

pub async fn wait_for_mock_log_chain_if_enabled(
    client: &Client,
    request_id: &str,
    expected_notification_codes: &[&str],
) -> Option<Value> {
    if !test027_live_chain_enabled() {
        return None;
    }
    Some(wait_for_mock_log_chain(client, request_id, expected_notification_codes).await)
}

pub async fn wait_for_mock_log_chain(
    client: &Client,
    request_id: &str,
    expected_notification_codes: &[&str],
) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    let expected_codes = expected_notification_codes
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();

    loop {
        let outbox_rows = client
            .query(
                "SELECT
                   outbox_event_id::text,
                   status,
                   payload
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND target_topic = 'dtp.notification.dispatch'
                 ORDER BY created_at ASC, outbox_event_id ASC",
                &[&request_id],
            )
            .await
            .expect("load TEST-027 outbox rows");
        let system_log_rows = client
            .query(
                "SELECT structured_payload
                 FROM ops.system_log
                 WHERE request_id = $1
                   AND object_type = 'notification_dispatch'
                   AND message_text = 'notification sent via mock-log'
                 ORDER BY created_at ASC",
                &[&request_id],
            )
            .await
            .expect("load TEST-027 mock-log records");
        let audit_rows = client
            .query(
                "SELECT
                   audit_id::text,
                   result_code,
                   metadata
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'notification.dispatch.sent'
                 ORDER BY event_time ASC, audit_id ASC",
                &[&request_id],
            )
            .await
            .expect("load TEST-027 notification dispatch audit");

        let outbox_statuses = outbox_rows
            .iter()
            .map(|row| row.get::<_, String>(1))
            .collect::<Vec<_>>();
        let published_count = outbox_statuses
            .iter()
            .filter(|status| status.as_str() == "published")
            .count();
        let outbox_codes = outbox_rows
            .iter()
            .filter_map(|row| {
                let payload = row.get::<_, Value>(2);
                payload["payload"]["notification_code"]
                    .as_str()
                    .map(ToString::to_string)
            })
            .collect::<Vec<_>>();
        let system_log_payloads = system_log_rows
            .iter()
            .map(|row| row.get::<_, Value>(0))
            .collect::<Vec<_>>();
        let log_codes = system_log_payloads
            .iter()
            .filter_map(|payload| {
                payload["notification_code"]
                    .as_str()
                    .map(ToString::to_string)
            })
            .collect::<Vec<_>>();
        let audit_metadata = audit_rows
            .iter()
            .map(|row| row.get::<_, Value>(2))
            .collect::<Vec<_>>();
        let audit_codes = audit_metadata
            .iter()
            .filter_map(|payload| {
                payload["notification_code"]
                    .as_str()
                    .map(ToString::to_string)
            })
            .collect::<Vec<_>>();

        let ready = published_count == expected_codes.len()
            && system_log_rows.len() == expected_codes.len()
            && audit_rows.len() == expected_codes.len()
            && code_multiset(&outbox_codes) == code_multiset(&expected_codes)
            && code_multiset(&log_codes) == code_multiset(&expected_codes)
            && code_multiset(&audit_codes) == code_multiset(&expected_codes);

        if ready {
            let outbox_event_ids = outbox_rows
                .iter()
                .map(|row| row.get::<_, String>(0))
                .collect::<Vec<_>>();
            let outbox_status_counts =
                outbox_statuses
                    .iter()
                    .fold(BTreeMap::new(), |mut acc, status| {
                        *acc.entry(status.clone()).or_insert(0_u32) += 1;
                        acc
                    });
            let target_topics = system_log_payloads
                .iter()
                .filter_map(|payload| payload["target_topic"].as_str().map(ToString::to_string))
                .collect::<Vec<_>>();
            let channels = system_log_payloads
                .iter()
                .filter_map(|payload| payload["channel"].as_str().map(ToString::to_string))
                .collect::<Vec<_>>();
            let audit_ids = audit_rows
                .iter()
                .map(|row| row.get::<_, String>(0))
                .collect::<Vec<_>>();
            let audit_result_codes = audit_rows
                .iter()
                .map(|row| row.get::<_, String>(1))
                .collect::<Vec<_>>();
            return json!({
                "request_id": request_id,
                "expected_notification_codes": expected_codes,
                "outbox": {
                    "count": outbox_rows.len(),
                    "published_count": published_count,
                    "status_counts": outbox_status_counts,
                    "notification_codes": outbox_codes,
                    "event_ids": outbox_event_ids,
                },
                "mock_log": {
                    "count": system_log_rows.len(),
                    "notification_codes": log_codes,
                    "channels": channels,
                    "target_topics": target_topics,
                },
                "audit": {
                    "count": audit_rows.len(),
                    "notification_codes": audit_codes,
                    "audit_ids": audit_ids,
                    "result_codes": audit_result_codes,
                },
            });
        }

        assert!(
            Instant::now() < deadline,
            "timed out waiting for TEST-027 live notification chain request_id={request_id}",
        );
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

fn code_multiset(codes: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for code in codes {
        *counts.entry(code.clone()).or_insert(0) += 1;
    }
    counts
}
