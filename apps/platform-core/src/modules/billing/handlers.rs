//! 所有 HTTP handler 函数

use crate::AppState;
use crate::modules::billing::db::{map_db_error, set_webhook_processed_status, write_audit_event};
use crate::modules::billing::domain::PayoutPreference;
use crate::modules::billing::models::{
    BillingPolicyView, LockOrderRequest, OrderLockView, PaymentWebhookRequest,
    PaymentWebhookResultView,
};
use crate::modules::billing::service::{
    BillingPermission, is_allowed, list_corridor_policies, list_jurisdictions,
    list_payout_preferences,
};
use crate::modules::billing::webhook::{
    is_replay_window_valid, map_webhook_target_status, now_utc_ms, parse_webhook_timestamp_ms,
    payment_status_rank, verify_webhook_signature_placeholder,
};
use crate::modules::order::application::apply_payment_result_to_order;
use crate::modules::order::domain::PaymentResultKind;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use db::{Error, GenericClient};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

// ── helpers ──────────────────────────────────────────────────────────────────

pub fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

pub fn require_permission(
    headers: &HeaderMap,
    permission: BillingPermission,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = headers
        .get("x-role")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if is_allowed(role, permission) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("{action} is forbidden for current role"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

pub fn require_step_up_placeholder(
    headers: &HeaderMap,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if header(headers, "x-step-up-token").is_some()
        || header(headers, "x-step-up-challenge-id").is_some()
    {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("x-step-up-token or x-step-up-challenge-id is required for {action}"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

// ── handlers ─────────────────────────────────────────────────────────────────

pub async fn get_billing_policies(
    headers: HeaderMap,
) -> Result<Json<ApiResponse<BillingPolicyView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::JurisdictionRead,
        "billing policy read",
    )?;
    info!(
        action = "billing.policy.read",
        "billing policy placeholder served"
    );
    Ok(ApiResponse::ok(BillingPolicyView {
        jurisdictions: list_jurisdictions(),
        corridor_policies: list_corridor_policies(),
    }))
}

pub async fn get_payout_preferences(
    headers: HeaderMap,
    Path(beneficiary_subject_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<PayoutPreference>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PayoutPreferenceRead,
        "billing payout preference read",
    )?;
    info!(
        action = "billing.payout_preference.read",
        beneficiary_subject_id = %beneficiary_subject_id,
        "billing payout preference placeholder served"
    );
    Ok(ApiResponse::ok(list_payout_preferences(
        &beneficiary_subject_id,
    )))
}

pub async fn create_payment_intent(
    state: State<AppState>,
    headers: HeaderMap,
    payload: Json<crate::modules::billing::models::CreatePaymentIntentRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::billing::models::PaymentIntentView>>,
    (StatusCode, Json<ErrorResponse>),
> {
    crate::modules::billing::payment_intent_handlers::create_payment_intent(state, headers, payload)
        .await
}

pub async fn get_payment_intent(
    state: State<AppState>,
    headers: HeaderMap,
    id: Path<String>,
) -> Result<
    Json<ApiResponse<crate::modules::billing::models::PaymentIntentDetailView>>,
    (StatusCode, Json<ErrorResponse>),
> {
    crate::modules::billing::payment_intent_handlers::get_payment_intent(state, headers, id).await
}

pub async fn cancel_payment_intent(
    state: State<AppState>,
    headers: HeaderMap,
    id: Path<String>,
) -> Result<
    Json<ApiResponse<crate::modules::billing::models::PaymentIntentView>>,
    (StatusCode, Json<ErrorResponse>),
> {
    crate::modules::billing::payment_intent_handlers::cancel_payment_intent(state, headers, id)
        .await
}

pub async fn lock_order_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<LockOrderRequest>,
) -> Result<Json<ApiResponse<OrderLockView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, BillingPermission::OrderLock, "order lock")?;
    let client = state.db.client().map_err(map_db_connect)?;

    let intent_row = client
        .query_opt(
            "SELECT payment_intent_id::text, order_id::text, provider_key
             FROM payment.payment_intent
             WHERE payment_intent_id = $1::text::uuid",
            &[&payload.payment_intent_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(intent_row) = intent_row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("payment intent not found: {}", payload.payment_intent_id),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };

    let intent_order_id: String = intent_row.get(1);
    if intent_order_id != order_id {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!(
                    "payment intent {} does not belong to order {}",
                    payload.payment_intent_id, order_id
                ),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    let provider_key: String = intent_row.get(2);

    let row = client
        .query_opt(
            "UPDATE trade.order_main
             SET
               payment_status = 'locked',
               buyer_locked_at = COALESCE(buyer_locked_at, now()),
               payment_channel_snapshot = payment_channel_snapshot || jsonb_build_object(
                 'payment_intent_id', $2::text,
                 'provider_key', $3::text,
                 'lock_reason', COALESCE($4::text, 'payment_lock'),
                 'locked_at', to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
               ),
               updated_at = now()
             WHERE order_id = $1::text::uuid
             RETURNING
               order_id::text,
               status,
               payment_status,
               to_char(buyer_locked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &order_id,
                &payload.payment_intent_id,
                &provider_key,
                &payload.lock_reason,
            ],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };

    let view = OrderLockView {
        order_id: row.get::<_, String>(0),
        payment_intent_id: payload.payment_intent_id.clone(),
        order_status: row.get::<_, String>(1),
        payment_status: row.get::<_, String>(2),
        buyer_locked_at: row.get::<_, String>(3),
    };
    write_audit_event(
        &client,
        "trade",
        "order",
        &view.order_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "order.payment.lock",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "order.payment.lock",
        order_id = %view.order_id,
        payment_intent_id = %view.payment_intent_id,
        payment_status = %view.payment_status,
        "order lock operation completed"
    );
    Ok(ApiResponse::ok(view))
}

pub async fn handle_payment_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(payload): Json<PaymentWebhookRequest>,
) -> Result<Json<ApiResponse<PaymentWebhookResultView>>, (StatusCode, Json<ErrorResponse>)> {
    let mut client = state.db.client().map_err(map_db_connect)?;

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
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let existing_row = client
        .query_opt(
            "SELECT webhook_event_id::text, signature_verified, payment_intent_id::text
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
        let _ = client
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
            header(&headers, "x-request-id").as_deref(),
            header(&headers, "x-trace-id").as_deref(),
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
            applied_payment_status: None,
        }));
    }

    let signature_verified = verify_webhook_signature_placeholder(
        &provider,
        header(&headers, "x-provider-signature").as_deref(),
        &payload,
    );
    let occurred_at_ms =
        parse_webhook_timestamp_ms(header(&headers, "x-webhook-timestamp").as_deref(), &payload);

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
                &payload.payload,
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
            header(&headers, "x-request-id").as_deref(),
            header(&headers, "x-trace-id").as_deref(),
        )
        .await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "rejected_signature".to_string(),
            duplicate: false,
            signature_verified,
            out_of_order_ignored: false,
            payment_intent_id: payload.payment_intent_id,
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
            header(&headers, "x-request-id").as_deref(),
            header(&headers, "x-trace-id").as_deref(),
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
            header(&headers, "x-request-id").as_deref(),
            header(&headers, "x-trace-id").as_deref(),
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
            applied_payment_status: None,
        }));
    };

    let intent_row = client
        .query_opt(
            "SELECT
               status,
               order_id::text,
               metadata ->> 'webhook_last_occurred_at_ms'
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
            header(&headers, "x-request-id").as_deref(),
            header(&headers, "x-trace-id").as_deref(),
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
            applied_payment_status: None,
        }));
    };

    let current_status: String = intent_row.get(0);
    let order_id: String = intent_row.get(1);
    let last_event_occurred_at_ms = intent_row
        .get::<_, Option<String>>(2)
        .and_then(|v| v.parse::<i64>().ok());
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
            header(&headers, "x-request-id").as_deref(),
            header(&headers, "x-trace-id").as_deref(),
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
            applied_payment_status: None,
        }));
    }

    let target_status =
        map_webhook_target_status(&payload.event_type, payload.provider_status.as_deref());
    let Some(target_status) = target_status else {
        set_webhook_processed_status(&client, &webhook_event_id, "processed_noop").await?;
        return Ok(ApiResponse::ok(PaymentWebhookResultView {
            webhook_event_id,
            provider_key: provider,
            provider_event_id: payload.provider_event_id,
            processed_status: "processed_noop".to_string(),
            duplicate: false,
            signature_verified: true,
            out_of_order_ignored: false,
            payment_intent_id: Some(payment_intent_id),
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
            header(&headers, "x-request-id").as_deref(),
            header(&headers, "x-trace-id").as_deref(),
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
            applied_payment_status: None,
        }));
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
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
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
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
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
        applied_payment_status: Some(applied_status),
    }))
}

pub(crate) fn map_db_connect(err: Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database connection failed: {err}"),
            request_id: None,
        }),
    )
}
