use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::repo::billing_event_repository::{
    RecordBillingEventRequest, infer_payment_success_event_type, record_billing_event,
};
use crate::modules::order::application::apply_payment_result_to_order;
use crate::modules::order::domain::{
    PaymentResultKind, derive_target_state, payment_status_for_result,
};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentResultSourceKind {
    Webhook,
    Polling,
}

impl PaymentResultSourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Webhook => "webhook",
            Self::Polling => "polling",
        }
    }

    fn event_source(self) -> &'static str {
        match self {
            Self::Webhook => "payment_webhook",
            Self::Polling => "payment_polling",
        }
    }

    fn processed_audit_action(self) -> &'static str {
        match self {
            Self::Webhook => "payment.webhook.processed",
            Self::Polling => "payment.polling.processed",
        }
    }

    fn ignored_audit_action(self) -> &'static str {
        match self {
            Self::Webhook => "payment.webhook.out_of_order_ignored",
            Self::Polling => "payment.polling.out_of_order_ignored",
        }
    }

    fn duplicate_audit_action(self) -> &'static str {
        match self {
            Self::Webhook => "payment.webhook.duplicate",
            Self::Polling => "payment.polling.duplicate",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessPaymentResultRequest {
    pub source_kind: PaymentResultSourceKind,
    pub provider_key: String,
    pub reference_id: String,
    pub payment_intent_id: String,
    pub provider_transaction_no: Option<String>,
    pub transaction_amount: Option<String>,
    pub currency_code: Option<String>,
    pub target_status: String,
    pub occurred_at: Option<String>,
    pub occurred_at_ms: i64,
    pub raw_payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessPaymentResultOutcome {
    pub processed_status: String,
    pub duplicate: bool,
    pub out_of_order_ignored: bool,
    pub payment_transaction_id: Option<String>,
    pub applied_payment_status: Option<String>,
}

#[derive(Debug)]
struct PaymentIntentProcessingContext {
    current_intent_status: String,
    order_id: String,
    last_result_occurred_at_ms: Option<i64>,
    last_result_reference_id: Option<String>,
    last_result_source: Option<String>,
    fallback_amount: String,
    fallback_currency: String,
    price_snapshot_json: Value,
    order_status: String,
    order_payment_status: String,
}

enum OrderSyncDecision {
    ApplyTransition(PaymentResultKind),
    SyncIntentOnly,
    Ignore,
}

pub async fn process_payment_result(
    client: &mut Client,
    request: ProcessPaymentResultRequest,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ProcessPaymentResultOutcome, (StatusCode, Json<ErrorResponse>)> {
    let context = load_processing_context(client, &request.payment_intent_id).await?;
    let source_label = request.source_kind.as_str();

    if context.last_result_source.as_deref() == Some(source_label)
        && context.last_result_reference_id.as_deref() == Some(request.reference_id.as_str())
    {
        write_audit_event(
            &*client,
            "payment",
            "payment_intent",
            &request.payment_intent_id,
            "system",
            request.source_kind.duplicate_audit_action(),
            "duplicate",
            request_id,
            trace_id,
        )
        .await?;
        return Ok(ProcessPaymentResultOutcome {
            processed_status: "duplicate".to_string(),
            duplicate: true,
            out_of_order_ignored: false,
            payment_transaction_id: None,
            applied_payment_status: Some(context.current_intent_status),
        });
    }

    if context
        .last_result_occurred_at_ms
        .map(|last| request.occurred_at_ms < last)
        .unwrap_or(false)
        || crate::modules::billing::webhook::payment_status_rank(&request.target_status)
            < crate::modules::billing::webhook::payment_status_rank(&context.current_intent_status)
    {
        write_audit_event(
            &*client,
            "payment",
            "payment_intent",
            &request.payment_intent_id,
            "system",
            request.source_kind.ignored_audit_action(),
            "ignored",
            request_id,
            trace_id,
        )
        .await?;
        return Ok(ProcessPaymentResultOutcome {
            processed_status: "out_of_order_ignored".to_string(),
            duplicate: false,
            out_of_order_ignored: true,
            payment_transaction_id: None,
            applied_payment_status: None,
        });
    }

    let result_kind = match request.target_status.as_str() {
        "succeeded" => PaymentResultKind::Succeeded,
        "failed" => PaymentResultKind::Failed,
        "expired" => PaymentResultKind::TimedOut,
        _ => {
            return Err(bad_request(
                &format!(
                    "unsupported payment result status: {}",
                    request.target_status
                ),
                request_id,
            ));
        }
    };
    let decision = decide_order_sync(
        &context.order_status,
        &context.order_payment_status,
        result_kind,
    );
    let mut transitioned_order = false;
    match decision {
        OrderSyncDecision::ApplyTransition(result_kind) => {
            let applied = apply_payment_result_to_order(
                client,
                &context.order_id,
                result_kind,
                request_id,
                trace_id,
            )
            .await?;
            if applied.is_none() {
                write_audit_event(
                    &*client,
                    "payment",
                    "payment_intent",
                    &request.payment_intent_id,
                    "system",
                    request.source_kind.ignored_audit_action(),
                    "ignored",
                    request_id,
                    trace_id,
                )
                .await?;
                return Ok(ProcessPaymentResultOutcome {
                    processed_status: "out_of_order_ignored".to_string(),
                    duplicate: false,
                    out_of_order_ignored: true,
                    payment_transaction_id: None,
                    applied_payment_status: None,
                });
            }
            transitioned_order = true;
        }
        OrderSyncDecision::SyncIntentOnly => {}
        OrderSyncDecision::Ignore => {
            write_audit_event(
                &*client,
                "payment",
                "payment_intent",
                &request.payment_intent_id,
                "system",
                request.source_kind.ignored_audit_action(),
                "ignored",
                request_id,
                trace_id,
            )
            .await?;
            return Ok(ProcessPaymentResultOutcome {
                processed_status: "out_of_order_ignored".to_string(),
                duplicate: false,
                out_of_order_ignored: true,
                payment_transaction_id: None,
                applied_payment_status: None,
            });
        }
    }

    let payment_transaction_id = insert_payment_transaction(
        client,
        &request,
        &context.fallback_amount,
        &context.fallback_currency,
    )
    .await?;
    let merged_metadata = build_result_metadata_patch(&request);
    let row = client
        .query_one(
            "UPDATE payment.payment_intent
             SET
               status = $2,
               metadata = metadata || $3::jsonb,
               updated_at = now()
             WHERE payment_intent_id = $1::text::uuid
             RETURNING status",
            &[
                &request.payment_intent_id,
                &request.target_status,
                &merged_metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let applied_status: String = row.get(0);

    if transitioned_order && request.target_status == "succeeded" {
        if let Some(event_type) = infer_payment_success_event_type(&context.price_snapshot_json) {
            let _ = record_billing_event(
                client,
                &RecordBillingEventRequest {
                    order_id: context.order_id.clone(),
                    event_type: event_type.to_string(),
                    event_source: request.source_kind.event_source().to_string(),
                    amount: Some(context.fallback_amount.clone()),
                    currency_code: Some(context.fallback_currency.clone()),
                    units: Some("1".to_string()),
                    occurred_at: request.occurred_at.clone(),
                    metadata: json!({
                        "idempotency_key": format!(
                            "billing_event:{}:{}:{}",
                            request.source_kind.event_source(),
                            request.payment_intent_id,
                            request.reference_id
                        ),
                        "payment_intent_id": request.payment_intent_id,
                        "provider_result_source": source_label,
                        "provider_reference_id": request.reference_id,
                        "provider_transaction_no": request.provider_transaction_no,
                        "provider_status": request.target_status,
                        "raw_payload": request.raw_payload,
                    }),
                },
                None,
                "system",
                "billing.event.generated",
                request_id,
                trace_id,
            )
            .await?;
        }
    }

    write_audit_event(
        &*client,
        "payment",
        "payment_intent",
        &request.payment_intent_id,
        "system",
        request.source_kind.processed_audit_action(),
        "success",
        request_id,
        trace_id,
    )
    .await?;

    Ok(ProcessPaymentResultOutcome {
        processed_status: "processed".to_string(),
        duplicate: false,
        out_of_order_ignored: false,
        payment_transaction_id,
        applied_payment_status: Some(applied_status),
    })
}

async fn load_processing_context(
    client: &Client,
    payment_intent_id: &str,
) -> Result<PaymentIntentProcessingContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               i.status,
               i.order_id::text,
               COALESCE(i.metadata ->> 'payment_result_last_occurred_at_ms', i.metadata ->> 'webhook_last_occurred_at_ms'),
               i.metadata ->> 'payment_result_last_reference_id',
               i.metadata ->> 'payment_result_last_source',
               i.amount::text,
               i.currency_code,
               COALESCE(o.price_snapshot_json, '{}'::jsonb),
               COALESCE(o.status, ''),
               COALESCE(o.payment_status, '')
             FROM payment.payment_intent i
             LEFT JOIN trade.order_main o ON o.order_id = i.order_id
             WHERE i.payment_intent_id = $1::text::uuid",
            &[&payment_intent_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("payment intent not found: {payment_intent_id}"),
                request_id: None,
            }),
        ));
    };
    Ok(PaymentIntentProcessingContext {
        current_intent_status: row.get(0),
        order_id: row.get(1),
        last_result_occurred_at_ms: row
            .get::<_, Option<String>>(2)
            .and_then(|value| value.parse::<i64>().ok()),
        last_result_reference_id: row.get(3),
        last_result_source: row.get(4),
        fallback_amount: row.get(5),
        fallback_currency: row.get(6),
        price_snapshot_json: row.get(7),
        order_status: row.get(8),
        order_payment_status: row.get(9),
    })
}

fn decide_order_sync(
    current_order_status: &str,
    current_order_payment_status: &str,
    result_kind: PaymentResultKind,
) -> OrderSyncDecision {
    if derive_target_state(current_order_status, result_kind).is_some() {
        return OrderSyncDecision::ApplyTransition(result_kind);
    }
    if current_order_payment_status == payment_status_for_result(result_kind) {
        return OrderSyncDecision::SyncIntentOnly;
    }
    OrderSyncDecision::Ignore
}

async fn insert_payment_transaction(
    client: &Client,
    request: &ProcessPaymentResultRequest,
    fallback_amount: &str,
    fallback_currency: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let provider_transaction_no = request
        .provider_transaction_no
        .as_deref()
        .unwrap_or(request.reference_id.as_str());
    if let Some(existing) = client
        .query_opt(
            "SELECT payment_transaction_id::text
             FROM payment.payment_transaction
             WHERE payment_intent_id = $1::text::uuid
               AND transaction_type = 'payin'
               AND provider_transaction_no = $2
             ORDER BY occurred_at DESC, payment_transaction_id DESC
             LIMIT 1",
            &[&request.payment_intent_id, &provider_transaction_no],
        )
        .await
        .map_err(map_db_error)?
    {
        return Ok(Some(existing.get(0)));
    }

    let amount = request
        .transaction_amount
        .as_deref()
        .unwrap_or(fallback_amount);
    let currency_code = request
        .currency_code
        .as_deref()
        .unwrap_or(fallback_currency);
    let row = client
        .query_one(
            "INSERT INTO payment.payment_transaction (
               payment_intent_id,
               transaction_type,
               direction,
               provider_transaction_no,
               provider_status,
               amount,
               currency_code,
               channel_fee_amount,
               settled_amount,
               occurred_at,
               raw_payload
             ) VALUES (
               $1::text::uuid,
               'payin',
               'inbound',
               $2,
               $3,
               $4::text::numeric,
               $5,
               0,
               CASE WHEN $3 = 'succeeded' THEN $4::text::numeric ELSE 0 END,
               to_timestamp($6::double precision / 1000.0),
               $7::jsonb
             )
             RETURNING payment_transaction_id::text",
            &[
                &request.payment_intent_id,
                &provider_transaction_no,
                &request.target_status,
                &amount,
                &currency_code,
                &request.occurred_at_ms,
                &request.raw_payload,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(Some(row.get(0)))
}

fn build_result_metadata_patch(request: &ProcessPaymentResultRequest) -> Value {
    let source_label = request.source_kind.as_str();
    let provider_transaction_no = request
        .provider_transaction_no
        .clone()
        .unwrap_or_else(|| request.reference_id.clone());
    let mut patch = json!({
        "payment_result_last_source": source_label,
        "payment_result_last_reference_id": request.reference_id,
        "payment_result_last_occurred_at_ms": request.occurred_at_ms,
        "payment_result_last_provider_status": request.target_status,
        "payment_result_last_transaction_no": provider_transaction_no,
    });
    if let Some(map) = patch.as_object_mut() {
        match request.source_kind {
            PaymentResultSourceKind::Webhook => {
                map.insert(
                    "webhook_last_event_id".to_string(),
                    Value::String(request.reference_id.clone()),
                );
                map.insert(
                    "webhook_last_occurred_at_ms".to_string(),
                    Value::String(request.occurred_at_ms.to_string()),
                );
                map.insert(
                    "webhook_last_provider_status".to_string(),
                    Value::String(request.target_status.clone()),
                );
            }
            PaymentResultSourceKind::Polling => {
                map.insert(
                    "polling_last_result_id".to_string(),
                    Value::String(request.reference_id.clone()),
                );
                map.insert(
                    "polling_last_occurred_at_ms".to_string(),
                    Value::String(request.occurred_at_ms.to_string()),
                );
                map.insert(
                    "polling_last_provider_status".to_string(),
                    Value::String(request.target_status.clone()),
                );
            }
        }
    }
    patch
}

fn bad_request(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
