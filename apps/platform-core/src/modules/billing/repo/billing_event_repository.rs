use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::domain::BillingEvent;
use crate::modules::billing::repo::settlement_aggregate_repository::recompute_settlement_for_order;
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct RecordBillingEventRequest {
    pub order_id: String,
    pub event_type: String,
    pub event_source: String,
    pub amount: Option<String>,
    pub currency_code: Option<String>,
    pub units: Option<String>,
    pub occurred_at: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
struct OrderBillingContext {
    buyer_org_id: String,
    seller_org_id: String,
    order_amount: String,
    currency_code: String,
    status: String,
    payment_status: String,
    sku_id: Option<String>,
    sku_type: String,
    pricing_mode: String,
    billing_mode: String,
    settlement_basis: String,
    price_snapshot_json: Value,
}

pub async fn record_billing_event(
    client: &Client,
    payload: &RecordBillingEventRequest,
    tenant_scope_id: Option<&str>,
    actor_role: &str,
    action_name: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(BillingEvent, bool), (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let result = record_billing_event_in_tx(
        &tx,
        payload,
        tenant_scope_id,
        actor_role,
        action_name,
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(result)
}

pub async fn record_billing_event_in_tx(
    client: &(impl GenericClient + Sync),
    payload: &RecordBillingEventRequest,
    tenant_scope_id: Option<&str>,
    actor_role: &str,
    action_name: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(BillingEvent, bool), (StatusCode, Json<ErrorResponse>)> {
    let context = load_order_billing_context(client, &payload.order_id, request_id).await?;
    enforce_order_scope(tenant_scope_id, &context, request_id)?;

    let event_type = normalize_event_type(&payload.event_type, request_id)?;
    let event_source = normalize_event_source(&payload.event_source, request_id)?;
    let idempotency_key = resolve_idempotency_key(
        payload
            .metadata
            .get("idempotency_key")
            .and_then(Value::as_str),
        request_id,
        &payload.order_id,
        &event_type,
        &event_source,
    )?;

    if let Some(existing) = find_existing_event(
        client,
        &payload.order_id,
        &event_type,
        &event_source,
        &idempotency_key,
    )
    .await?
    {
        let replay_action = format!("{action_name}.idempotent_replay");
        write_audit_event(
            client,
            "billing",
            "billing_event",
            &existing.billing_event_id,
            actor_role,
            &replay_action,
            "idempotent_replay",
            request_id,
            trace_id,
        )
        .await?;
        return Ok((existing, true));
    }

    validate_event_semantics(&event_type, &context, &payload.metadata, request_id)?;

    let amount = resolve_amount(&event_type, payload.amount.as_deref(), &context, request_id)?;
    let currency_code = payload
        .currency_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| context.currency_code.clone());
    let units = resolve_units(&event_type, payload.units.as_deref(), request_id)?;
    let merged_metadata = build_metadata(
        &event_type,
        &event_source,
        &idempotency_key,
        &context,
        &payload.metadata,
    );
    let units_param = units.as_deref();
    let occurred_at_param = payload.occurred_at.as_deref();

    let row = client
        .query_one(
            "INSERT INTO billing.billing_event (
               order_id,
               event_type,
               event_source,
               amount,
               currency_code,
               units,
               occurred_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               $4::text::numeric,
               $5,
               $6::text::numeric,
               COALESCE($7::timestamptz, now()),
               $8::jsonb
             )
             RETURNING
               billing_event_id::text,
               order_id::text,
               event_type,
               event_source,
               amount::text,
               currency_code,
               CASE WHEN units IS NULL THEN NULL ELSE units::text END,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata",
            &[
                &payload.order_id,
                &event_type,
                &event_source,
                &amount,
                &currency_code,
                &units_param,
                &occurred_at_param,
                &merged_metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let event = parse_billing_event_row(&row);

    write_billing_event_outbox(
        client,
        &event,
        &idempotency_key,
        request_id,
        trace_id,
        &context,
    )
    .await?;
    write_audit_event(
        client,
        "billing",
        "billing_event",
        &event.billing_event_id,
        actor_role,
        action_name,
        "success",
        request_id,
        trace_id,
    )
    .await?;
    let _ =
        recompute_settlement_for_order(client, &payload.order_id, actor_role, request_id, trace_id)
            .await?;

    Ok((event, false))
}

pub async fn list_billing_events_for_order(
    client: &Client,
    order_id: &str,
    tenant_scope_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<Vec<BillingEvent>, (StatusCode, Json<ErrorResponse>)> {
    let context = load_order_billing_context(client, order_id, request_id).await?;
    enforce_order_scope(tenant_scope_id, &context, request_id)?;
    let rows = client
        .query(
            "SELECT
               billing_event_id::text,
               order_id::text,
               event_type,
               event_source,
               amount::text,
               currency_code,
               CASE WHEN units IS NULL THEN NULL ELSE units::text END,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata
             FROM billing.billing_event
             WHERE order_id = $1::text::uuid
             ORDER BY occurred_at ASC, billing_event_id ASC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    Ok(rows.iter().map(parse_billing_event_row).collect())
}

async fn load_order_billing_context(
    client: &impl GenericClient,
    order_id: &str,
    request_id: Option<&str>,
) -> Result<OrderBillingContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               o.buyer_org_id::text,
               o.seller_org_id::text,
               o.amount::text,
               o.currency_code,
               o.status,
               o.payment_status,
               o.sku_id::text,
               COALESCE(s.sku_type, COALESCE(o.price_snapshot_json ->> 'sku_type', o.price_snapshot_json ->> 'selected_sku_type', 'unknown')),
               COALESCE(o.price_snapshot_json ->> 'pricing_mode', 'unknown'),
               COALESCE(s.billing_mode, COALESCE(o.price_snapshot_json ->> 'billing_mode', 'unknown')),
               COALESCE(o.price_snapshot_json ->> 'settlement_basis', 'unknown'),
               COALESCE(o.price_snapshot_json, '{}'::jsonb)
             FROM trade.order_main o
             LEFT JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(billing_bad_request(
            &format!("order not found: {order_id}"),
            request_id,
            StatusCode::NOT_FOUND,
        ));
    };
    Ok(OrderBillingContext {
        buyer_org_id: row.get(0),
        seller_org_id: row.get(1),
        order_amount: row.get(2),
        currency_code: row.get(3),
        status: row.get(4),
        payment_status: row.get(5),
        sku_id: row.get(6),
        sku_type: row.get(7),
        pricing_mode: row.get(8),
        billing_mode: row.get(9),
        settlement_basis: row.get(10),
        price_snapshot_json: row.get(11),
    })
}

fn parse_billing_event_row(row: &Row) -> BillingEvent {
    BillingEvent {
        billing_event_id: row.get(0),
        order_id: row.get(1),
        event_type: row.get(2),
        event_source: row.get(3),
        amount: row.get(4),
        currency_code: row.get(5),
        units: row.get(6),
        occurred_at: row.get(7),
        metadata: row.get(8),
    }
}

async fn find_existing_event(
    client: &impl GenericClient,
    order_id: &str,
    event_type: &str,
    event_source: &str,
    idempotency_key: &str,
) -> Result<Option<BillingEvent>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               billing_event_id::text,
               order_id::text,
               event_type,
               event_source,
               amount::text,
               currency_code,
               CASE WHEN units IS NULL THEN NULL ELSE units::text END,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata
             FROM billing.billing_event
             WHERE order_id = $1::text::uuid
               AND event_type = $2
               AND event_source = $3
               AND COALESCE(metadata ->> 'idempotency_key', '') = $4
             ORDER BY occurred_at DESC, billing_event_id DESC
             LIMIT 1",
            &[&order_id, &event_type, &event_source, &idempotency_key],
        )
        .await
        .map_err(map_db_error)?;
    Ok(row.as_ref().map(parse_billing_event_row))
}

async fn write_billing_event_outbox(
    client: &impl GenericClient,
    event: &BillingEvent,
    idempotency_key: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    context: &OrderBillingContext,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let payload = json!({
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "pending_fabric_anchor",
        "billing_event_id": event.billing_event_id.clone(),
        "order_id": event.order_id.clone(),
        "event_type": event.event_type.clone(),
        "event_source": event.event_source.clone(),
        "amount": event.amount.clone(),
        "currency_code": event.currency_code.clone(),
        "units": event.units.clone(),
        "occurred_at": event.occurred_at.clone(),
        "sku_id": context.sku_id.clone(),
        "sku_type": context.sku_type.clone(),
        "pricing_mode": context.pricing_mode.clone(),
        "billing_mode": context.billing_mode.clone(),
        "settlement_basis": context.settlement_basis.clone(),
        "metadata": event.metadata.clone()
    });
    write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: "billing.billing_event",
            aggregate_id: &event.billing_event_id,
            event_type: "billing.event.recorded",
            producer_service: "platform-core.billing",
            request_id,
            trace_id,
            idempotency_key: Some(idempotency_key),
            occurred_at: Some(event.occurred_at.as_str()),
            business_payload: &payload,
            deduplicate_by_idempotency_key: false,
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

fn normalize_event_type(
    raw: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = raw.trim().to_ascii_lowercase();
    let canonical = match normalized.as_str() {
        "one_time_charge" | "one_time" | "one-time-charge" => "one_time_charge",
        "recurring_charge" | "periodic_charge" | "subscription_charge" => "recurring_charge",
        "usage_charge" | "metered_charge" | "overage_charge" => "usage_charge",
        "refund" => "refund",
        "refund_adjustment" => "refund_adjustment",
        "compensation" => "compensation",
        "compensation_adjustment" => "compensation_adjustment",
        "manual_settlement" | "manual_payout" => "manual_settlement",
        "manual_adjustment" => "manual_adjustment",
        _ => {
            return Err(billing_bad_request(
                &format!("unsupported billing event_type: {raw}"),
                request_id,
                StatusCode::BAD_REQUEST,
            ));
        }
    };
    Ok(canonical.to_string())
}

fn normalize_event_source(
    raw: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = raw.trim().to_ascii_lowercase().replace('-', "_");
    if normalized.is_empty() {
        return Err(billing_bad_request(
            "billing event_source is required",
            request_id,
            StatusCode::BAD_REQUEST,
        ));
    }
    Ok(normalized)
}

fn resolve_idempotency_key(
    metadata_key: Option<&str>,
    request_id: Option<&str>,
    order_id: &str,
    event_type: &str,
    event_source: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    if let Some(key) = metadata_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(key.to_string());
    }
    if let Some(request_id) = request_id.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(format!(
            "billing_event:{order_id}:{event_type}:{event_source}:{request_id}"
        ));
    }
    Err(billing_bad_request(
        "billing event metadata.idempotency_key or request_id is required",
        request_id,
        StatusCode::BAD_REQUEST,
    ))
}

fn resolve_amount(
    event_type: &str,
    provided_amount: Option<&str>,
    context: &OrderBillingContext,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let candidate = provided_amount
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(match event_type {
            "one_time_charge" | "recurring_charge" => context.order_amount.as_str(),
            _ => "0",
        });
    let parsed = candidate.parse::<f64>().map_err(|_| {
        billing_bad_request(
            "billing event amount must be a decimal string",
            request_id,
            StatusCode::BAD_REQUEST,
        )
    })?;
    let allow_signed = matches!(
        event_type,
        "refund_adjustment" | "compensation_adjustment" | "manual_adjustment"
    );
    if (!allow_signed && parsed <= 0.0) || (allow_signed && parsed == 0.0) {
        return Err(billing_bad_request(
            if allow_signed {
                "adjustment billing event amount must be a non-zero decimal string"
            } else {
                "billing event amount must be a positive decimal string"
            },
            request_id,
            StatusCode::BAD_REQUEST,
        ));
    }
    Ok(candidate.to_string())
}

fn resolve_units(
    event_type: &str,
    provided_units: Option<&str>,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let default_units = match event_type {
        "one_time_charge" => Some("1"),
        _ => None,
    };
    let value = provided_units
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .or(default_units);
    let Some(value) = value else {
        return Ok(None);
    };
    match value.parse::<f64>() {
        Ok(parsed) if parsed > 0.0 => Ok(Some(value.to_string())),
        _ => Err(billing_bad_request(
            "billing event units must be a positive decimal string when provided",
            request_id,
            StatusCode::BAD_REQUEST,
        )),
    }
}

fn validate_event_semantics(
    event_type: &str,
    context: &OrderBillingContext,
    metadata: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let pricing_lower = context.pricing_mode.to_ascii_lowercase();
    let billing_lower = context.billing_mode.to_ascii_lowercase();
    let settlement_lower = context.settlement_basis.to_ascii_lowercase();
    let recurring_capable = billing_lower.contains("subscription")
        || billing_lower.contains("recurring")
        || billing_lower.contains("period")
        || pricing_lower.contains("subscription")
        || pricing_lower.contains("recurring")
        || settlement_lower.contains("period");
    let usage_capable = billing_lower.contains("usage")
        || billing_lower.contains("meter")
        || billing_lower.contains("ppu")
        || pricing_lower.contains("usage")
        || pricing_lower.contains("meter")
        || pricing_lower.contains("ppu")
        || pricing_lower.contains("query");

    match event_type {
        "recurring_charge" if !recurring_capable => Err(billing_bad_request(
            &format!(
                "recurring_charge is not allowed for billing_mode `{}` / pricing_mode `{}`",
                context.billing_mode, context.pricing_mode
            ),
            request_id,
            StatusCode::BAD_REQUEST,
        )),
        "usage_charge" if !usage_capable => Err(billing_bad_request(
            &format!(
                "usage_charge is not allowed for billing_mode `{}` / pricing_mode `{}`",
                context.billing_mode, context.pricing_mode
            ),
            request_id,
            StatusCode::BAD_REQUEST,
        )),
        "refund" | "refund_adjustment" | "compensation" | "compensation_adjustment" => {
            if metadata
                .get("reason_code")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
            {
                Err(billing_bad_request(
                    &format!("{event_type} billing event requires metadata.reason_code"),
                    request_id,
                    StatusCode::BAD_REQUEST,
                ))
            } else {
                Ok(())
            }
        }
        "manual_settlement" | "manual_adjustment" => {
            let direction = metadata
                .get("settlement_direction")
                .and_then(Value::as_str)
                .unwrap_or("");
            if matches!(direction, "payable" | "receivable" | "adjustment") {
                Ok(())
            } else {
                Err(billing_bad_request(
                    "manual_settlement/manual_adjustment requires metadata.settlement_direction = payable | receivable | adjustment",
                    request_id,
                    StatusCode::BAD_REQUEST,
                ))
            }
        }
        _ => Ok(()),
    }?;

    if context.status == "closed" {
        return Err(billing_bad_request(
            "billing event cannot be recorded for closed order",
            request_id,
            StatusCode::CONFLICT,
        ));
    }
    Ok(())
}

fn build_metadata(
    event_type: &str,
    event_source: &str,
    idempotency_key: &str,
    context: &OrderBillingContext,
    metadata: &Value,
) -> Value {
    let mut merged = metadata.clone();
    if !merged.is_object() {
        merged = json!({});
    }
    let canonical_snapshot = json!({
        "sku_id": context.sku_id.clone(),
        "sku_type": context.sku_type.clone(),
        "pricing_mode": context.pricing_mode.clone(),
        "billing_mode": context.billing_mode.clone(),
        "settlement_basis": context.settlement_basis.clone(),
        "order_status": context.status.clone(),
        "payment_status": context.payment_status.clone(),
        "price_snapshot": context.price_snapshot_json.clone(),
    });
    if let Some(object) = merged.as_object_mut() {
        object.insert(
            "idempotency_key".to_string(),
            Value::String(idempotency_key.to_string()),
        );
        object.insert(
            "model_name".to_string(),
            Value::String("BillingEvent".to_string()),
        );
        object.insert(
            "event_type_canonical".to_string(),
            Value::String(event_type.to_string()),
        );
        object.insert(
            "event_source_canonical".to_string(),
            Value::String(event_source.to_string()),
        );
        object.insert(
            "consistency_state".to_string(),
            json!({
                "reconcile_status": "pending",
                "proof_commit_state": "pending_anchor",
                "status_version": 1
            }),
        );
        object.insert("charge_snapshot".to_string(), canonical_snapshot);
    }
    merged
}

fn enforce_order_scope(
    tenant_scope_id: Option<&str>,
    context: &OrderBillingContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(tenant_scope_id) = tenant_scope_id else {
        return Ok(());
    };
    if tenant_scope_id == context.buyer_org_id || tenant_scope_id == context.seller_org_id {
        return Ok(());
    }
    Err(billing_bad_request(
        "tenant scope does not match billing event order",
        request_id,
        StatusCode::FORBIDDEN,
    ))
}

pub fn infer_payment_success_event_type(price_snapshot_json: &Value) -> Option<&'static str> {
    let pricing_mode = price_snapshot_json
        .get("pricing_mode")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let billing_mode = price_snapshot_json
        .get("billing_mode")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let settlement_basis = price_snapshot_json
        .get("settlement_basis")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();

    if billing_mode.contains("usage")
        || billing_mode.contains("meter")
        || billing_mode.contains("ppu")
        || pricing_mode.contains("usage")
        || pricing_mode.contains("meter")
        || pricing_mode.contains("ppu")
        || pricing_mode.contains("query")
    {
        return None;
    }
    if billing_mode.contains("subscription")
        || billing_mode.contains("recurring")
        || billing_mode.contains("period")
        || pricing_mode.contains("subscription")
        || pricing_mode.contains("recurring")
        || settlement_basis.contains("period")
    {
        return Some("recurring_charge");
    }
    Some("one_time_charge")
}

fn billing_bad_request(
    message: &str,
    request_id: Option<&str>,
    status: StatusCode,
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

#[cfg(test)]
mod tests {
    use super::infer_payment_success_event_type;
    use serde_json::json;

    #[test]
    fn payment_success_uses_one_time_for_file_style_snapshot() {
        let event_type = infer_payment_success_event_type(&json!({
            "pricing_mode": "one_time",
            "billing_mode": "one_time",
            "settlement_basis": "one_time"
        }));
        assert_eq!(event_type, Some("one_time_charge"));
    }

    #[test]
    fn payment_success_uses_recurring_for_subscription_snapshot() {
        let event_type = infer_payment_success_event_type(&json!({
            "pricing_mode": "subscription",
            "billing_mode": "subscription_cycle",
            "settlement_basis": "periodic"
        }));
        assert_eq!(event_type, Some("recurring_charge"));
    }

    #[test]
    fn payment_success_skips_usage_only_snapshot() {
        let event_type = infer_payment_success_event_type(&json!({
            "pricing_mode": "usage_metered",
            "billing_mode": "api_ppu",
            "settlement_basis": "usage"
        }));
        assert_eq!(event_type, None);
    }
}
