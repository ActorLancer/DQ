use crate::AppState;
use crate::modules::billing::db::{map_db_error, set_webhook_processed_status, write_audit_event};
use crate::modules::billing::handlers::{header, map_db_connect};
use crate::modules::billing::models::{PaymentWebhookRequest, PaymentWebhookResultView};
use crate::modules::billing::webhook::{
    is_replay_window_valid, map_webhook_target_status, map_webhook_transaction_shape, now_utc_ms,
    parse_webhook_timestamp_ms, payment_status_rank, verify_webhook_signature_placeholder,
};
use crate::modules::order::application::apply_payment_result_to_order;
use crate::modules::order::domain::PaymentResultKind;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use db::GenericClient;
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};

pub async fn handle_payment_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(payload): Json<PaymentWebhookRequest>,
) -> Result<Json<ApiResponse<PaymentWebhookResultView>>, (StatusCode, Json<ErrorResponse>)> {
    let mut client = state.db.client().map_err(map_db_connect)?;
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let provider_exists = client
        .query_opt(
            "SELECT provider_key FROM payment.provider WHERE provider_key = $1",
            &[&provider],
        )
        .await
        .map_err(map_db_error)?;
    if provider_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("provider not found: {provider}"),
                request_id,
            }),
        ));
    }

    let existing_row = client
        .query_opt(
            "SELECT webhook_event_id::text, signature_verified, payment_intent_id::text, payment_transaction_id::text
             FROM payment.payment_webhook_event
             WHERE provider_key = $1 AND provider_event_id = $2",
            &[&provider, &payload.provider_event_id],
        )
        .await
        .map_err(map_db_error)?;
    if let Some(existing) = existing_row {
        let webhook_event_id: String = existing.get(0);
        let signature_verified: bool = existing.get(1);
        let payment_intent_id: Option<String> = existing.get(2);
        let payment_transaction_id: Option<String> = existing.get(3);
        client
            .execute(
                "UPDATE payment.payment_webhook_event
                 SET duplicate_flag = true, processed_status = 'duplicate', processed_at = now()
                 WHERE webhook_event_id = $1::text::uuid",
                &[&webhook_event_id],
            )
            .await
            .map_err(map_db_error)?;
        write_audit_event(
            &client,
            "payment",
            "payment_webhook_event",
            &webhook_event_id,
            "provider_callback",
            "payment.webhook.duplicate",
            "duplicate",
            request_id.as_deref(),
            trace_id.as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "duplicate".to_string(),
            duplicate: true,
            signature_verified,
            out_of_order_ignored: false,
            payment_intent_id,
            payment_transaction_id,
            applied_payment_status: None,
        }));
    }

    let occurred_at_ms =
        parse_webhook_timestamp_ms(header(&headers, "x-webhook-timestamp").as_deref(), &payload)
            .or(normalize_occured_at_text_ms(
                &client,
                payload.occurred_at.as_deref(),
                request_id.as_deref(),
            )
            .await?);
    let signature_verified = verify_webhook_signature_placeholder(
        &provider,
        header(&headers, "x-provider-signature").as_deref(),
        &payload,
    );

    let webhook_insert = client
        .query_one(
            "INSERT INTO payment.payment_webhook_event (
               provider_key,
               provider_event_id,
               event_type,
               signature_verified,
               payment_intent_id,
               payload,
               processed_status,
               duplicate_flag
             ) VALUES (
               $1, $2, $3, $4, $5::text::uuid, $6::jsonb, 'pending', false
             )
             RETURNING webhook_event_id::text",
            &[
                &provider,
                &payload.provider_event_id,
                &payload.event_type,
                &signature_verified,
                &payload.payment_intent_id,
                &payload.raw_payload,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let webhook_event_id: String = webhook_insert.get(0);

    if !signature_verified {
        set_webhook_processed_status(&client, &webhook_event_id, "rejected_signature").await?;
        write_audit_event(
            &client,
            "payment",
            "payment_webhook_event",
            &webhook_event_id,
            "provider_callback",
            "payment.webhook.rejected_signature",
            "rejected",
            request_id.as_deref(),
            trace_id.as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "rejected_signature".to_string(),
            duplicate: false,
            signature_verified: false,
            out_of_order_ignored: false,
            payment_intent_id: payload.payment_intent_id,
            payment_transaction_id: None,
            applied_payment_status: None,
        }));
    }

    if !is_replay_window_valid(occurred_at_ms.unwrap_or_else(now_utc_ms)) {
        set_webhook_processed_status(&client, &webhook_event_id, "rejected_replay").await?;
        write_audit_event(
            &client,
            "payment",
            "payment_webhook_event",
            &webhook_event_id,
            "provider_callback",
            "payment.webhook.rejected_replay",
            "rejected",
            request_id.as_deref(),
            trace_id.as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "rejected_replay".to_string(),
            duplicate: false,
            signature_verified: true,
            out_of_order_ignored: false,
            payment_intent_id: payload.payment_intent_id,
            payment_transaction_id: None,
            applied_payment_status: None,
        }));
    }

    let Some(payment_intent_id) = payload.payment_intent_id.clone() else {
        set_webhook_processed_status(&client, &webhook_event_id, "processed_noop").await?;
        write_audit_event(
            &client,
            "payment",
            "payment_webhook_event",
            &webhook_event_id,
            "provider_callback",
            "payment.webhook.processed_noop",
            "success",
            request_id.as_deref(),
            trace_id.as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "processed_noop".to_string(),
            duplicate: false,
            signature_verified: true,
            out_of_order_ignored: false,
            payment_intent_id: None,
            payment_transaction_id: None,
            applied_payment_status: None,
        }));
    };

    let intent_row = client
        .query_opt(
            "SELECT
               status,
               order_id::text,
               metadata ->> 'webhook_last_occurred_at_ms',
               amount::text,
               currency_code
             FROM payment.payment_intent
             WHERE payment_intent_id = $1::text::uuid",
            &[&payment_intent_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(intent_row) = intent_row else {
        set_webhook_processed_status(&client, &webhook_event_id, "intent_not_found").await?;
        write_audit_event(
            &client,
            "payment",
            "payment_webhook_event",
            &webhook_event_id,
            "provider_callback",
            "payment.webhook.intent_not_found",
            "failed",
            request_id.as_deref(),
            trace_id.as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "intent_not_found".to_string(),
            duplicate: false,
            signature_verified: true,
            out_of_order_ignored: false,
            payment_intent_id: Some(payment_intent_id),
            payment_transaction_id: None,
            applied_payment_status: None,
        }));
    };

    let current_status: String = intent_row.get(0);
    let order_id: String = intent_row.get(1);
    let last_event_occurred_at_ms = intent_row
        .get::<_, Option<String>>(2)
        .and_then(|v| v.parse::<i64>().ok());
    let fallback_amount: String = intent_row.get(3);
    let fallback_currency: String = intent_row.get(4);
    let incoming_occurred_at_ms = occurred_at_ms.unwrap_or_else(now_utc_ms);
    if last_event_occurred_at_ms
        .map(|last| incoming_occurred_at_ms < last)
        .unwrap_or(false)
    {
        set_webhook_processed_status(&client, &webhook_event_id, "out_of_order_ignored").await?;
        write_audit_event(
            &client,
            "payment",
            "payment_intent",
            &payment_intent_id,
            "provider_callback",
            "payment.webhook.out_of_order_ignored",
            "ignored",
            request_id.as_deref(),
            trace_id.as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "out_of_order_ignored".to_string(),
            duplicate: false,
            signature_verified: true,
            out_of_order_ignored: true,
            payment_intent_id: Some(payment_intent_id),
            payment_transaction_id: None,
            applied_payment_status: None,
        }));
    }

    let target_status =
        map_webhook_target_status(&payload.event_type, payload.provider_status.as_deref());
    let Some(target_status) = target_status else {
        set_webhook_processed_status(&client, &webhook_event_id, "processed_noop").await?;
        write_audit_event(
            &client,
            "payment",
            "payment_webhook_event",
            &webhook_event_id,
            "provider_callback",
            "payment.webhook.processed_noop",
            "success",
            request_id.as_deref(),
            trace_id.as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "processed_noop".to_string(),
            duplicate: false,
            signature_verified: true,
            out_of_order_ignored: false,
            payment_intent_id: Some(payment_intent_id),
            payment_transaction_id: None,
            applied_payment_status: None,
        }));
    };

    if payment_status_rank(target_status) < payment_status_rank(&current_status) {
        set_webhook_processed_status(&client, &webhook_event_id, "out_of_order_ignored").await?;
        write_audit_event(
            &client,
            "payment",
            "payment_intent",
            &payment_intent_id,
            "provider_callback",
            "payment.webhook.out_of_order_ignored",
            "ignored",
            request_id.as_deref(),
            trace_id.as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "out_of_order_ignored".to_string(),
            duplicate: false,
            signature_verified: true,
            out_of_order_ignored: true,
            payment_intent_id: Some(payment_intent_id),
            payment_transaction_id: None,
            applied_payment_status: None,
        }));
    }

    let payment_transaction_id = insert_payment_transaction(
        &client,
        &payment_intent_id,
        &payload,
        target_status,
        &fallback_amount,
        &fallback_currency,
        incoming_occurred_at_ms,
    )
    .await?;
    if let Some(payment_transaction_id) = payment_transaction_id.as_deref() {
        client
            .execute(
                "UPDATE payment.payment_webhook_event
                 SET payment_transaction_id = $2::text::uuid
                 WHERE webhook_event_id = $1::text::uuid",
                &[&webhook_event_id, &payment_transaction_id],
            )
            .await
            .map_err(map_db_error)?;
    }

    let row = client
        .query_one(
            "UPDATE payment.payment_intent
             SET
               status = $2,
               metadata = metadata || jsonb_build_object(
                 'webhook_last_event_id', $3::text,
                 'webhook_last_occurred_at_ms', $4::bigint,
                 'webhook_last_event_type', $5::text,
                 'webhook_last_provider_status', COALESCE($6::text, '')
               ),
               updated_at = now()
             WHERE payment_intent_id = $1::text::uuid
             RETURNING status",
            &[
                &payment_intent_id,
                &target_status,
                &payload.provider_event_id,
                &incoming_occurred_at_ms,
                &payload.event_type,
                &payload.provider_status,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let applied_status: String = row.get(0);
    let order_result_kind = match target_status {
        "succeeded" => PaymentResultKind::Succeeded,
        "failed" => PaymentResultKind::Failed,
        "expired" => PaymentResultKind::TimedOut,
        _ => PaymentResultKind::Failed,
    };
    let _ = apply_payment_result_to_order(
        &mut client,
        &order_id,
        order_result_kind,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;
    set_webhook_processed_status(&client, &webhook_event_id, "processed").await?;
    write_audit_event(
        &client,
        "payment",
        "payment_intent",
        &payment_intent_id,
        "provider_callback",
        "payment.webhook.processed",
        "success",
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(PaymentWebhookResultView {
        webhook_event_id,
        provider_key: provider,
        provider_event_id: payload.provider_event_id,
        processed_status: "processed".to_string(),
        duplicate: false,
        signature_verified: true,
        out_of_order_ignored: false,
        payment_intent_id: Some(payment_intent_id),
        payment_transaction_id,
        applied_payment_status: Some(applied_status),
    }))
}

async fn normalize_occured_at_text_ms(
    client: &impl GenericClient,
    occurred_at: Option<&str>,
    request_id: Option<&str>,
) -> Result<Option<i64>, (StatusCode, Json<ErrorResponse>)> {
    let Some(occurred_at) = occurred_at.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let row = client
        .query_one(
            "SELECT floor(extract(epoch from $1::timestamptz) * 1000)::bigint",
            &[&occurred_at],
        )
        .await
        .map_err(|_| bad_request("occurred_at must be a valid RFC3339 timestamp", request_id))?;
    Ok(Some(row.get(0)))
}

async fn insert_payment_transaction(
    client: &impl GenericClient,
    payment_intent_id: &str,
    payload: &PaymentWebhookRequest,
    target_status: &str,
    fallback_amount: &str,
    fallback_currency: &str,
    occurred_at_ms: i64,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let Some((transaction_type, direction)) = map_webhook_transaction_shape(&payload.event_type)
    else {
        return Ok(None);
    };
    let amount = payload
        .transaction_amount
        .as_deref()
        .unwrap_or(fallback_amount);
    let currency_code = payload
        .currency_code
        .as_deref()
        .unwrap_or(fallback_currency);
    let provider_transaction_no = payload
        .provider_transaction_no
        .as_deref()
        .unwrap_or(payload.provider_event_id.as_str());

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
               $2,
               $3,
               $4,
               $5,
               $6::text::numeric,
               $7,
               0,
               CASE WHEN $5 = 'succeeded' THEN $6::text::numeric ELSE 0 END,
               to_timestamp($8::double precision / 1000.0),
               $9::jsonb
             )
             RETURNING payment_transaction_id::text",
            &[
                &payment_intent_id,
                &transaction_type,
                &direction,
                &provider_transaction_no,
                &target_status,
                &amount,
                &currency_code,
                &occurred_at_ms,
                &payload.raw_payload,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(Some(row.get(0)))
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
