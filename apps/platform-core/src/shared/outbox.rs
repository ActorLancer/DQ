use db::{Error, GenericClient};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
struct EventRoutePolicy {
    authority_scope: String,
    proof_commit_policy: String,
    target_bus: String,
    target_topic: String,
    partition_key_template: Option<String>,
    ordering_scope: Option<String>,
}

pub(crate) struct CanonicalOutboxWrite<'a> {
    pub aggregate_type: &'a str,
    pub aggregate_id: &'a str,
    pub event_type: &'a str,
    pub producer_service: &'a str,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
    pub idempotency_key: Option<&'a str>,
    pub occurred_at: Option<&'a str>,
    pub business_payload: &'a Value,
    pub deduplicate_by_idempotency_key: bool,
}

pub(crate) async fn write_canonical_outbox_event(
    client: &(impl GenericClient + Sync),
    write: CanonicalOutboxWrite<'_>,
) -> Result<bool, Error> {
    let route = load_event_route_policy(client, write.aggregate_type, write.event_type).await?;
    let business_payload = normalize_business_payload(write.business_payload);
    let partition_key = resolve_partition_key(
        route.partition_key_template.as_deref(),
        write.aggregate_id,
        write.request_id,
        &business_payload,
    );
    let ordering_key = resolve_ordering_key(
        route.ordering_scope.as_deref(),
        write.aggregate_id,
        write.request_id,
        &partition_key,
        &business_payload,
    );
    let row = client
        .query_opt(
            "WITH normalized AS (
               SELECT
                 gen_random_uuid() AS outbox_event_id,
                 to_char(
                   timezone('UTC', now()),
                   'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'
                 ) AS occurred_at_text,
                 CASE
                   WHEN jsonb_typeof($8::jsonb) = 'object' THEN $8::jsonb
                   ELSE jsonb_build_object('data', $8::jsonb)
                 END AS business_payload
             ),
             envelope AS (
               SELECT
                 outbox_event_id,
                 (
                   (business_payload
                     - 'event_name'
                     - 'event_id'
                     - 'event_type'
                     - 'event_version'
                     - 'occurred_at'
                     - 'producer_service'
                     - 'aggregate_type'
                     - 'aggregate_id'
                     - 'request_id'
                     - 'trace_id'
                     - 'idempotency_key'
                     - 'payload'
                     - 'event_schema_version'
                     - 'authority_scope'
                     - 'source_of_truth'
                     - 'proof_commit_policy')
                   || jsonb_build_object(
                     'event_id', outbox_event_id::text,
                     'event_type', $3,
                     'event_version', 1,
                     'occurred_at', COALESCE($7, occurred_at_text),
                     'producer_service', $4,
                     'aggregate_type', $1,
                     'aggregate_id', $2,
                     'request_id', $5,
                     'trace_id', $6,
                     'idempotency_key', $9,
                     'event_schema_version', 'v1',
                     'authority_scope', $10,
                     'source_of_truth', 'database',
                     'proof_commit_policy', $11,
                     'payload', business_payload
                   )
                 ) AS payload
               FROM normalized
             )
             INSERT INTO ops.outbox_event (
               outbox_event_id,
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
             )
             SELECT
               envelope.outbox_event_id,
               $1,
               $2::text::uuid,
               $3,
               envelope.payload,
               'pending',
               $5,
               $6,
               $9,
               'v1',
               $10,
               'database',
               $11,
               $12,
               $13,
               $14,
               $15,
               encode(digest((envelope.payload)::text, 'sha256'), 'hex')
             FROM envelope
             WHERE NOT $16
                OR $9 IS NULL
                OR NOT EXISTS (
                  SELECT 1
                  FROM ops.outbox_event
                  WHERE idempotency_key = $9
                )
             RETURNING outbox_event_id::text",
            &[
                &write.aggregate_type,
                &write.aggregate_id,
                &write.event_type,
                &write.producer_service,
                &write.request_id,
                &write.trace_id,
                &write.occurred_at,
                &business_payload,
                &write.idempotency_key,
                &route.authority_scope,
                &route.proof_commit_policy,
                &route.target_bus,
                &route.target_topic,
                &partition_key,
                &ordering_key,
                &write.deduplicate_by_idempotency_key,
            ],
        )
        .await?;
    Ok(row.is_some())
}

async fn load_event_route_policy(
    client: &(impl GenericClient + Sync),
    aggregate_type: &str,
    event_type: &str,
) -> Result<EventRoutePolicy, Error> {
    let row = client
        .query_opt(
            "SELECT
               authority_scope,
               proof_commit_policy,
               target_bus,
               target_topic,
               partition_key_template,
               ordering_scope
             FROM ops.event_route_policy
             WHERE aggregate_type = $1
               AND event_type = $2
               AND status = 'active'
             ORDER BY updated_at DESC, created_at DESC
             LIMIT 1",
            &[&aggregate_type, &event_type],
        )
        .await?;
    let Some(row) = row else {
        return Err(Error::Bind(format!(
            "missing ops.event_route_policy for aggregate_type={aggregate_type} event_type={event_type}"
        )));
    };
    Ok(EventRoutePolicy {
        authority_scope: row.get(0),
        proof_commit_policy: row.get(1),
        target_bus: row.get(2),
        target_topic: row.get(3),
        partition_key_template: row.get(4),
        ordering_scope: row.get(5),
    })
}

fn normalize_business_payload(payload: &Value) -> Value {
    if payload.is_object() {
        payload.clone()
    } else {
        json!({
            "data": payload.clone()
        })
    }
}

fn resolve_partition_key(
    template: Option<&str>,
    aggregate_id: &str,
    request_id: Option<&str>,
    payload: &Value,
) -> String {
    resolve_template_value(template, aggregate_id, request_id, payload)
        .unwrap_or_else(|| aggregate_id.to_string())
}

fn resolve_ordering_key(
    scope: Option<&str>,
    aggregate_id: &str,
    request_id: Option<&str>,
    partition_key: &str,
    payload: &Value,
) -> String {
    match scope.map(str::trim).filter(|scope| !scope.is_empty()) {
        Some(scope) if scope.eq_ignore_ascii_case("aggregate_id") => aggregate_id.to_string(),
        Some(scope) if scope.eq_ignore_ascii_case("request_id") => request_id
            .map(str::to_string)
            .unwrap_or_else(|| partition_key.to_string()),
        Some(scope) if scope.eq_ignore_ascii_case("partition_key") => partition_key.to_string(),
        Some(scope) => {
            extract_payload_string(payload, scope).unwrap_or_else(|| partition_key.to_string())
        }
        None => partition_key.to_string(),
    }
}

fn resolve_template_value(
    template: Option<&str>,
    aggregate_id: &str,
    request_id: Option<&str>,
    payload: &Value,
) -> Option<String> {
    match template
        .map(str::trim)
        .filter(|template| !template.is_empty())
    {
        Some(template) if template.eq_ignore_ascii_case("aggregate_id") => {
            Some(aggregate_id.to_string())
        }
        Some(template) if template.eq_ignore_ascii_case("request_id") => {
            request_id.map(str::to_string)
        }
        Some(template) => extract_payload_string(payload, template),
        None => None,
    }
}

fn extract_payload_string(payload: &Value, key: &str) -> Option<String> {
    payload.get(key).and_then(value_to_string)
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(raw) => Some(raw.clone()),
        Value::Number(raw) => Some(raw.to_string()),
        Value::Bool(raw) => Some(raw.to_string()),
        _ => None,
    }
}
