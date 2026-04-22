use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::repo::billing_event_repository::{
    RecordBillingEventRequest, record_billing_event_in_tx,
};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct BillingBridgeProcessResult {
    pub order_id: String,
    pub processed_count: usize,
    pub ignored_count: usize,
    pub replayed_count: usize,
    pub processed_outbox_event_ids: Vec<String>,
    pub processed_billing_event_ids: Vec<String>,
    pub ignored_outbox_event_ids: Vec<String>,
}

#[derive(Debug)]
struct PendingBridgeEvent {
    outbox_event_id: String,
    payload: Value,
    publish_attempt_id: String,
    publish_attempt_no: i32,
}

#[derive(Debug)]
enum BridgeDecision {
    Record {
        event_type: &'static str,
        event_source: &'static str,
        amount: Option<String>,
        units: Option<String>,
        metadata: Value,
    },
    Ignore {
        reason_code: &'static str,
    },
}

pub async fn process_billing_bridge_events_for_order(
    client: &mut Client,
    order_id: &str,
    outbox_event_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<BillingBridgeProcessResult, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    ensure_order_exists(&tx, order_id, request_id).await?;
    let events = load_published_bridge_events(&tx, order_id, outbox_event_id).await?;

    let mut result = BillingBridgeProcessResult {
        order_id: order_id.to_string(),
        processed_count: 0,
        ignored_count: 0,
        replayed_count: 0,
        processed_outbox_event_ids: Vec::new(),
        processed_billing_event_ids: Vec::new(),
        ignored_outbox_event_ids: Vec::new(),
    };

    for event in events {
        match derive_bridge_decision(
            order_id,
            &event.outbox_event_id,
            &event.publish_attempt_id,
            event.publish_attempt_no,
            &event.payload,
        ) {
            Ok(BridgeDecision::Record {
                event_type,
                event_source,
                amount,
                units,
                metadata,
            }) => {
                let (billing_event, replayed) = record_billing_event_in_tx(
                    &tx,
                    &RecordBillingEventRequest {
                        order_id: order_id.to_string(),
                        event_type: event_type.to_string(),
                        event_source: event_source.to_string(),
                        amount,
                        currency_code: None,
                        units,
                        occurred_at: None,
                        metadata,
                    },
                    None,
                    actor_role,
                    "billing.bridge.materialize",
                    request_id,
                    trace_id,
                )
                .await?;
                write_audit_event(
                    &tx,
                    "billing",
                    "billing_event",
                    &billing_event.billing_event_id,
                    actor_role,
                    "billing.bridge.process",
                    if replayed {
                        "idempotent_replay"
                    } else {
                        "success"
                    },
                    request_id,
                    trace_id,
                )
                .await?;
                result.processed_count += 1;
                if replayed {
                    result.replayed_count += 1;
                }
                result
                    .processed_outbox_event_ids
                    .push(event.outbox_event_id);
                result
                    .processed_billing_event_ids
                    .push(billing_event.billing_event_id);
            }
            Ok(BridgeDecision::Ignore { reason_code }) => {
                write_audit_event(
                    &tx,
                    "billing",
                    "outbox_event",
                    &event.outbox_event_id,
                    actor_role,
                    "billing.bridge.ignored",
                    reason_code,
                    request_id,
                    trace_id,
                )
                .await?;
                result.ignored_count += 1;
                result.ignored_outbox_event_ids.push(event.outbox_event_id);
            }
            Err(err) => return Err(err),
        }
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(result)
}

async fn ensure_order_exists(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let exists = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM trade.order_main WHERE order_id = $1::text::uuid)",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?
        .get::<_, bool>(0);
    if exists {
        return Ok(());
    }
    Err(billing_bridge_error(
        StatusCode::NOT_FOUND,
        &format!("order not found: {order_id}"),
        request_id,
    ))
}

async fn load_published_bridge_events(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    outbox_event_id: Option<&str>,
) -> Result<Vec<PendingBridgeEvent>, (StatusCode, Json<ErrorResponse>)> {
    let rows = if let Some(outbox_event_id) = outbox_event_id {
        client
            .query(
                "SELECT
                    oe.outbox_event_id::text,
                    oe.payload,
                    latest_attempt.outbox_publish_attempt_id::text,
                    latest_attempt.attempt_no
                 FROM ops.outbox_event oe
                 JOIN LATERAL (
                   SELECT outbox_publish_attempt_id, attempt_no, result_code
                     FROM ops.outbox_publish_attempt
                    WHERE outbox_event_id = oe.outbox_event_id
                    ORDER BY attempt_no DESC, attempted_at DESC, outbox_publish_attempt_id DESC
                    LIMIT 1
                 ) latest_attempt
                   ON latest_attempt.result_code = 'published'
                 WHERE oe.aggregate_type = 'trade.order_main'
                   AND oe.aggregate_id = $1::text::uuid
                   AND oe.event_type = 'billing.trigger.bridge'
                   AND oe.status = 'published'
                   AND oe.published_at IS NOT NULL
                   AND oe.target_topic = 'dtp.outbox.domain-events'
                   AND oe.outbox_event_id = $2::text::uuid
                 ORDER BY oe.published_at ASC, oe.created_at ASC, oe.outbox_event_id ASC",
                &[&order_id, &outbox_event_id],
            )
            .await
            .map_err(map_db_error)?
    } else {
        client
            .query(
                "SELECT
                    oe.outbox_event_id::text,
                    oe.payload,
                    latest_attempt.outbox_publish_attempt_id::text,
                    latest_attempt.attempt_no
                 FROM ops.outbox_event oe
                 JOIN LATERAL (
                   SELECT outbox_publish_attempt_id, attempt_no, result_code
                     FROM ops.outbox_publish_attempt
                    WHERE outbox_event_id = oe.outbox_event_id
                    ORDER BY attempt_no DESC, attempted_at DESC, outbox_publish_attempt_id DESC
                    LIMIT 1
                 ) latest_attempt
                   ON latest_attempt.result_code = 'published'
                 WHERE oe.aggregate_type = 'trade.order_main'
                   AND oe.aggregate_id = $1::text::uuid
                   AND oe.event_type = 'billing.trigger.bridge'
                   AND oe.status = 'published'
                   AND oe.published_at IS NOT NULL
                   AND oe.target_topic = 'dtp.outbox.domain-events'
                 ORDER BY oe.published_at ASC, oe.created_at ASC, oe.outbox_event_id ASC",
                &[&order_id],
            )
            .await
            .map_err(map_db_error)?
    };

    Ok(rows
        .into_iter()
        .map(|row| PendingBridgeEvent {
            outbox_event_id: row.get(0),
            payload: row.get(1),
            publish_attempt_id: row.get(2),
            publish_attempt_no: row.get(3),
        })
        .collect())
}

fn derive_bridge_decision(
    order_id: &str,
    outbox_event_id: &str,
    publish_attempt_id: &str,
    publish_attempt_no: i32,
    payload: &Value,
) -> Result<BridgeDecision, (StatusCode, Json<ErrorResponse>)> {
    let sku_type = payload
        .get("sku_type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let trigger_stage = payload
        .get("trigger_stage")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let trigger_action = payload
        .get("trigger_action")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let delivery_branch = payload
        .get("delivery_branch")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let matrix = payload
        .get("billing_trigger_matrix")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let decision = match sku_type {
        "FILE_STD" | "FILE_SUB" | "RPT_STD"
            if trigger_stage == "acceptance_passed" || trigger_action == "delivery.accept" =>
        {
            BridgeDecision::Record {
                event_type: if sku_type == "FILE_SUB" {
                    "recurring_charge"
                } else {
                    "one_time_charge"
                },
                event_source: "delivery_acceptance",
                amount: None,
                units: Some("1".to_string()),
                metadata: bridge_metadata(
                    order_id,
                    outbox_event_id,
                    sku_type,
                    trigger_stage,
                    trigger_action,
                    delivery_branch,
                    matrix,
                    payload,
                    "billing_bridge_delivery_acceptance",
                    publish_attempt_id,
                    publish_attempt_no,
                ),
            }
        }
        "QRY_LITE" if trigger_stage == "delivery_committed" => BridgeDecision::Record {
            event_type: "one_time_charge",
            event_source: "query_run_success",
            amount: None,
            units: Some("1".to_string()),
            metadata: bridge_metadata(
                order_id,
                outbox_event_id,
                sku_type,
                trigger_stage,
                trigger_action,
                delivery_branch,
                matrix,
                payload,
                "billing_bridge_query_run_success",
                publish_attempt_id,
                publish_attempt_no,
            ),
        },
        "SBX_STD" if trigger_stage == "delivery_committed" => BridgeDecision::Record {
            event_type: "recurring_charge",
            event_source: "sandbox_workspace_enable",
            amount: None,
            units: Some("1".to_string()),
            metadata: bridge_metadata(
                order_id,
                outbox_event_id,
                sku_type,
                trigger_stage,
                trigger_action,
                delivery_branch,
                matrix,
                payload,
                "billing_bridge_sandbox_workspace_enable",
                publish_attempt_id,
                publish_attempt_no,
            ),
        },
        "SHARE_RO" if trigger_stage == "delivery_committed" => BridgeDecision::Record {
            event_type: "one_time_charge",
            event_source: "share_grant_effective",
            amount: None,
            units: Some("1".to_string()),
            metadata: bridge_metadata(
                order_id,
                outbox_event_id,
                sku_type,
                trigger_stage,
                trigger_action,
                delivery_branch,
                matrix,
                payload,
                "billing_bridge_share_grant_effective",
                publish_attempt_id,
                publish_attempt_no,
            ),
        },
        "API_SUB" => BridgeDecision::Ignore {
            reason_code: "api_sub_cycle_billed_by_explicit_transition",
        },
        "API_PPU" => BridgeDecision::Ignore {
            reason_code: "api_ppu_usage_billed_by_usage_meter",
        },
        "FILE_STD" | "FILE_SUB" | "RPT_STD" => BridgeDecision::Ignore {
            reason_code: "manual_acceptance_required_before_charge",
        },
        _ => BridgeDecision::Ignore {
            reason_code: "unsupported_bridge_mapping",
        },
    };

    Ok(decision)
}

fn bridge_metadata(
    order_id: &str,
    outbox_event_id: &str,
    sku_type: &str,
    trigger_stage: &str,
    trigger_action: &str,
    delivery_branch: &str,
    matrix: Value,
    payload: &Value,
    reason_code: &str,
    publish_attempt_id: &str,
    publish_attempt_no: i32,
) -> Value {
    json!({
        "idempotency_key": format!("billing_bridge:{outbox_event_id}:{sku_type}:{trigger_stage}:{trigger_action}"),
        "reason_code": reason_code,
        "bridge_outbox_event_id": outbox_event_id,
        "bridge_publish_attempt_id": publish_attempt_id,
        "bridge_publish_attempt_no": publish_attempt_no,
        "bridge_order_id": order_id,
        "bridge_sku_type": sku_type,
        "bridge_trigger_stage": trigger_stage,
        "bridge_trigger_action": trigger_action,
        "bridge_delivery_branch": delivery_branch,
        "billing_trigger_matrix": matrix,
        "bridge_payload_snapshot": payload,
    })
}

fn billing_bridge_error(
    status: StatusCode,
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
