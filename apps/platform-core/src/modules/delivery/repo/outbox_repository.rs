use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::ErrorResponse;
use serde_json::{Map, Value, json};

pub(crate) const DELIVERY_RECEIPT_EVENT_TYPE: &str = "delivery.committed";
const DELIVERY_RECEIPT_AGGREGATE_TYPE: &str = "delivery.delivery_record";
const DELIVERY_RECEIPT_TARGET_TOPIC: &str = "dtp.outbox.domain-events";

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_delivery_receipt_outbox_payload(
    delivery_branch: &str,
    order_id: &str,
    delivery_id: &str,
    sku_type: &str,
    actor_role: &str,
    buyer_org_id: &str,
    seller_org_id: &str,
    current_state: &str,
    payment_status: &str,
    delivery_status: &str,
    acceptance_status: &str,
    settlement_status: &str,
    dispute_status: &str,
    receipt_hash: Option<&str>,
    delivery_commit_hash: Option<&str>,
    delivery_type: Option<&str>,
    delivery_route: Option<&str>,
    committed_at: Option<&str>,
    extra: Value,
) -> Value {
    let mut payload = json!({
        "event_name": DELIVERY_RECEIPT_EVENT_TYPE,
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "async_evidence",
        "delivery_branch": delivery_branch,
        "order_id": order_id,
        "delivery_id": delivery_id,
        "sku_type": sku_type,
        "actor_role": actor_role,
        "buyer_org_id": buyer_org_id,
        "seller_org_id": seller_org_id,
        "current_state": current_state,
        "payment_status": payment_status,
        "delivery_status": delivery_status,
        "acceptance_status": acceptance_status,
        "settlement_status": settlement_status,
        "dispute_status": dispute_status,
        "receipt_hash": receipt_hash,
        "delivery_commit_hash": delivery_commit_hash,
        "delivery_type": delivery_type,
        "delivery_route": delivery_route,
        "committed_at": committed_at,
    });

    let object = payload
        .as_object_mut()
        .expect("delivery receipt payload must be object");
    if let Some(extra_object) = extra.as_object() {
        merge_object(object, extra_object);
    } else if !extra.is_null() {
        object.insert("details".to_string(), extra);
    }
    payload
}

pub(crate) async fn write_delivery_receipt_outbox_event(
    client: &(impl GenericClient + Sync),
    delivery_id: &str,
    payload: &Value,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    client
        .query_one(
            "INSERT INTO ops.outbox_event (
               aggregate_type,
               aggregate_id,
               event_type,
               payload,
               status,
               request_id,
               trace_id,
               idempotency_key,
               event_schema_version,
               authority_scope,
               source_of_truth,
               proof_commit_policy,
               target_bus,
               target_topic,
               partition_key,
               ordering_key,
               payload_hash
             ) VALUES (
               $1,
               $2::text::uuid,
               $3,
               $4::jsonb,
               'pending',
               $5,
               $6,
               $7,
               COALESCE($4::jsonb ->> 'event_schema_version', 'v1'),
               COALESCE($4::jsonb ->> 'authority_scope', 'business'),
               COALESCE($4::jsonb ->> 'source_of_truth', 'database'),
               COALESCE($4::jsonb ->> 'proof_commit_policy', 'async_evidence'),
               'kafka',
               $8,
               $2,
               $2,
               encode(digest(($4::jsonb)::text, 'sha256'), 'hex')
             )
             RETURNING outbox_event_id::text",
            &[
                &DELIVERY_RECEIPT_AGGREGATE_TYPE,
                &delivery_id,
                &DELIVERY_RECEIPT_EVENT_TYPE,
                payload,
                &request_id,
                &trace_id,
                &idempotency_key,
                &DELIVERY_RECEIPT_TARGET_TOPIC,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn merge_object(target: &mut Map<String, Value>, extra: &Map<String, Value>) {
    for (key, value) in extra {
        target.insert(key.clone(), value.clone());
    }
}
