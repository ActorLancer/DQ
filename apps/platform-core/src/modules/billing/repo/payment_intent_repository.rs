use crate::modules::billing::db::{map_db_error, parse_intent_row, select_intent_by_idempotency};
use crate::modules::billing::models::{
    CreatePaymentIntentRequest, PaymentIntentDetailView, PaymentIntentView,
    PaymentTransactionSummaryView, PaymentWebhookSummaryView,
};
use axum::Json;
use axum::http::StatusCode;
use db::runtime::DbParam;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

struct OrderPaymentContext {
    buyer_org_id: String,
    seller_org_id: String,
    status: String,
    payment_status: String,
    amount: String,
    currency_code: String,
    fee_preview_snapshot: Value,
    price_snapshot_json: Value,
}

struct ProviderContext {
    provider_type: String,
    settlement_category: String,
    supports_sandbox: bool,
    supports_payin: bool,
    supports_refund: bool,
    supports_webhook: bool,
    supports_multi_currency: bool,
}

pub async fn create_payment_intent(
    client: &Client,
    payload: &CreatePaymentIntentRequest,
    request_id: Option<&str>,
    idempotency_key: &str,
    tenant_scope_id: Option<&str>,
) -> Result<PaymentIntentView, (StatusCode, Json<ErrorResponse>)> {
    if let Some(existing) = select_intent_by_idempotency(client, idempotency_key).await? {
        enforce_intent_scope(tenant_scope_id, &existing)?;
        return Ok(existing);
    }

    let order = load_order_context(client, &payload.order_id).await?;
    enforce_order_scope(tenant_scope_id, &order, payload)?;
    enforce_order_payable(&order)?;

    let payment_amount = parse_positive_amount(
        &payload.payment_amount,
        "payment_amount must be a positive decimal string",
    )?;
    let price_currency_code = payload
        .price_currency_code
        .clone()
        .or_else(|| {
            order
                .price_snapshot_json
                .get("price_currency_code")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "USD".to_string());
    let currency_code = payload
        .currency_code
        .clone()
        .unwrap_or_else(|| order.currency_code.clone());
    let payer_jurisdiction_code = payload
        .payer_jurisdiction_code
        .clone()
        .unwrap_or_else(|| "SG".to_string());
    let payee_jurisdiction_code = payload
        .payee_jurisdiction_code
        .clone()
        .unwrap_or_else(|| "SG".to_string());
    let launch_jurisdiction_code = payload
        .launch_jurisdiction_code
        .clone()
        .unwrap_or_else(|| payer_jurisdiction_code.clone());
    let intent_type = payload
        .intent_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| billing_bad_request("intent_type is required", request_id))?;

    let provider = load_provider_context(client, &payload.provider_key).await?;
    if !provider.supports_payin {
        return Err(billing_bad_request(
            "payment provider does not support payin",
            request_id,
        ));
    }

    ensure_active_jurisdiction(client, &payer_jurisdiction_code, request_id).await?;
    ensure_active_jurisdiction(client, &payee_jurisdiction_code, request_id).await?;
    ensure_active_jurisdiction(client, &launch_jurisdiction_code, request_id).await?;

    let provider_account_id = resolve_provider_account_id(
        client,
        &payload.provider_key,
        payload.provider_account_id.as_deref(),
        payload.payee_subject_type.as_deref(),
        payload.payee_subject_id.as_deref(),
        request_id,
    )
    .await?;

    let corridor_policy_id = resolve_corridor_policy_id(
        client,
        payload.corridor_policy_id.as_deref(),
        &payer_jurisdiction_code,
        &payee_jurisdiction_code,
        &price_currency_code,
        request_id,
    )
    .await?;

    let fee_preview_id = validate_fee_preview(
        client,
        &payload.order_id,
        payload.fee_preview_id.as_deref(),
        &order,
    )
    .await?;

    let capability_snapshot = json!({
        "provider_type": provider.provider_type,
        "settlement_category": provider.settlement_category,
        "supports_sandbox": provider.supports_sandbox,
        "supports_payin": provider.supports_payin,
        "supports_refund": provider.supports_refund,
        "supports_webhook": provider.supports_webhook,
        "supports_multi_currency": provider.supports_multi_currency,
        "provider_account_id": provider_account_id,
        "corridor_policy_id": corridor_policy_id,
        "launch_jurisdiction_code": launch_jurisdiction_code,
        "validated_order_status": order.status,
        "validated_order_payment_status": order.payment_status,
        "validated_order_amount": order.amount,
    });
    let provider_account_id_param = provider_account_id.as_deref();
    let corridor_policy_id_param = corridor_policy_id.as_deref();
    let fee_preview_id_param = fee_preview_id.as_deref();
    let payee_subject_type_param = payload.payee_subject_type.as_deref();
    let payee_subject_id_param = payload.payee_subject_id.as_deref();
    let expire_at_param = payload.expire_at.as_deref();
    let params: [&(dyn DbParam + Sync); 22] = [
        &payload.order_id,
        &intent_type,
        &payload.provider_key,
        &provider_account_id_param,
        &payload.payer_subject_type,
        &payload.payer_subject_id,
        &payee_subject_type_param,
        &payee_subject_id_param,
        &payer_jurisdiction_code,
        &payee_jurisdiction_code,
        &launch_jurisdiction_code,
        &corridor_policy_id_param,
        &fee_preview_id_param,
        &payment_amount,
        &payload.payment_method,
        &currency_code,
        &price_currency_code,
        &request_id,
        &idempotency_key,
        &expire_at_param,
        &capability_snapshot,
        &payload.metadata,
    ];

    let row = client
        .query_one(
            "INSERT INTO payment.payment_intent (
               order_id, intent_type, provider_key, provider_account_id,
               payer_subject_type, payer_subject_id, payee_subject_type, payee_subject_id,
               payer_jurisdiction_code, payee_jurisdiction_code, launch_jurisdiction_code,
               corridor_policy_id, fee_preview_id, amount, payment_method,
               currency_code, price_currency_code, status, request_id, idempotency_key,
               expire_at, capability_snapshot, metadata
             ) VALUES (
               $1::text::uuid, $2, $3, $4::text::uuid,
               $5, $6::text::uuid, $7, $8::text::uuid,
               $9, $10, $11,
               $12::text::uuid, $13::text::uuid, $14::text::numeric, $15,
               $16, $17, 'created', $18, $19,
               $20::timestamptz, $21::jsonb, $22::jsonb
             )
             RETURNING
               payment_intent_id::text,
               order_id::text,
               intent_type,
               provider_key,
               provider_account_id::text,
               payer_subject_type,
               payer_subject_id::text,
               payee_subject_type,
               payee_subject_id::text,
               payer_jurisdiction_code,
               payee_jurisdiction_code,
               launch_jurisdiction_code,
               corridor_policy_id::text,
               fee_preview_id::text,
               amount::text,
               payment_method,
               currency_code,
               price_currency_code,
               status,
               provider_intent_no,
               channel_reference_no,
               idempotency_key,
               request_id,
               to_char(expire_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               capability_snapshot,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &params,
        )
        .await
        .map_err(map_db_error)?;
    parse_intent_row(&row)
}

pub async fn get_payment_intent_detail(
    client: &Client,
    payment_intent_id: &str,
    tenant_scope_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<Option<PaymentIntentDetailView>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               payment_intent_id::text,
               order_id::text,
               intent_type,
               provider_key,
               provider_account_id::text,
               payer_subject_type,
               payer_subject_id::text,
               payee_subject_type,
               payee_subject_id::text,
               payer_jurisdiction_code,
               payee_jurisdiction_code,
               launch_jurisdiction_code,
               corridor_policy_id::text,
               fee_preview_id::text,
               amount::text,
               payment_method,
               currency_code,
               price_currency_code,
               status,
               provider_intent_no,
               channel_reference_no,
               idempotency_key,
               request_id,
               to_char(expire_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               capability_snapshot,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.payment_intent
             WHERE payment_intent_id = $1::text::uuid",
            &[&payment_intent_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Ok(None);
    };
    let intent = parse_intent_row(&row)?;
    enforce_intent_scope_with_request(tenant_scope_id, &intent, request_id)?;

    let latest_transaction_summary =
        load_latest_transaction_summary(client, payment_intent_id).await?;
    let webhook_summary = load_latest_webhook_summary(client, payment_intent_id).await?;
    Ok(Some(PaymentIntentDetailView {
        payment_intent: intent,
        latest_transaction_summary,
        webhook_summary,
    }))
}

pub async fn cancel_payment_intent(
    client: &Client,
    payment_intent_id: &str,
    tenant_scope_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<PaymentIntentView, (StatusCode, Json<ErrorResponse>)> {
    let detail = get_payment_intent_detail(client, payment_intent_id, tenant_scope_id, request_id)
        .await?
        .ok_or_else(|| billing_not_found(payment_intent_id, request_id))?;
    let current_status = detail.payment_intent.payment_status.as_str();
    if current_status == "canceled" {
        return Ok(detail.payment_intent);
    }
    if matches!(current_status, "succeeded" | "failed" | "expired") {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!(
                    "payment intent cannot be canceled from status {}",
                    detail.payment_intent.payment_status
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    }

    let row = client
        .query_one(
            "UPDATE payment.payment_intent
             SET status = 'canceled', updated_at = now()
             WHERE payment_intent_id = $1::text::uuid
             RETURNING
               payment_intent_id::text,
               order_id::text,
               intent_type,
               provider_key,
               provider_account_id::text,
               payer_subject_type,
               payer_subject_id::text,
               payee_subject_type,
               payee_subject_id::text,
               payer_jurisdiction_code,
               payee_jurisdiction_code,
               launch_jurisdiction_code,
               corridor_policy_id::text,
               fee_preview_id::text,
               amount::text,
               payment_method,
               currency_code,
               price_currency_code,
               status,
               provider_intent_no,
               channel_reference_no,
               idempotency_key,
               request_id,
               to_char(expire_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               capability_snapshot,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&payment_intent_id],
        )
        .await
        .map_err(map_db_error)?;
    parse_intent_row(&row)
}

fn parse_positive_amount(
    value: &str,
    message: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    match value.parse::<f64>() {
        Ok(v) if v > 0.0 => Ok(value.to_string()),
        _ => Err(billing_bad_request(message, None)),
    }
}

async fn load_order_context(
    client: &Client,
    order_id: &str,
) -> Result<OrderPaymentContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               buyer_org_id::text,
               seller_org_id::text,
               status,
               payment_status,
               amount::text,
               currency_code,
               COALESCE(fee_preview_snapshot, '{}'::jsonb),
               COALESCE(price_snapshot_json, '{}'::jsonb)
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: None,
            }),
        ));
    };
    Ok(OrderPaymentContext {
        buyer_org_id: row.get(0),
        seller_org_id: row.get(1),
        status: row.get(2),
        payment_status: row.get(3),
        amount: row.get(4),
        currency_code: row.get(5),
        fee_preview_snapshot: row.get(6),
        price_snapshot_json: row.get(7),
    })
}

fn enforce_order_scope(
    tenant_scope_id: Option<&str>,
    order: &OrderPaymentContext,
    payload: &CreatePaymentIntentRequest,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(tenant_scope_id) = tenant_scope_id else {
        return Ok(());
    };

    if tenant_scope_id != order.buyer_org_id && tenant_scope_id != order.seller_org_id {
        return Err(billing_forbidden(
            "tenant scope does not match order participants",
        ));
    }

    let payer_matches =
        payload.payer_subject_type == "organization" && payload.payer_subject_id == tenant_scope_id;
    let payee_matches = payload
        .payee_subject_type
        .as_deref()
        .map(|subject_type| subject_type == "organization")
        .unwrap_or(false)
        && payload.payee_subject_id.as_deref() == Some(tenant_scope_id);

    if !payer_matches && !payee_matches {
        return Err(billing_forbidden(
            "tenant scope does not match payment intent subjects",
        ));
    }

    Ok(())
}

fn enforce_order_payable(
    order: &OrderPaymentContext,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let status_allowed = matches!(order.status.as_str(), "created" | "contract_effective");
    let payment_allowed = matches!(
        order.payment_status.as_str(),
        "unpaid" | "failed" | "expired"
    );
    if status_allowed && payment_allowed {
        return Ok(());
    }
    Err((
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: format!(
                "order is not payable from status {} / payment_status {}",
                order.status, order.payment_status
            ),
            request_id: None,
        }),
    ))
}

async fn load_provider_context(
    client: &Client,
    provider_key: &str,
) -> Result<ProviderContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT provider_type, settlement_category, supports_sandbox, supports_payin,
                    supports_refund, supports_webhook, supports_multi_currency, status
             FROM payment.provider
             WHERE provider_key = $1",
            &[&provider_key],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(billing_bad_request("payment provider does not exist", None));
    };
    let status: String = row.get(7);
    if status != "active" {
        return Err(billing_bad_request("payment provider is not active", None));
    }
    Ok(ProviderContext {
        provider_type: row.get(0),
        settlement_category: row.get(1),
        supports_sandbox: row.get(2),
        supports_payin: row.get(3),
        supports_refund: row.get(4),
        supports_webhook: row.get(5),
        supports_multi_currency: row.get(6),
    })
}

async fn ensure_active_jurisdiction(
    client: &Client,
    jurisdiction_code: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT status FROM payment.jurisdiction_profile WHERE jurisdiction_code = $1",
            &[&jurisdiction_code],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(billing_bad_request(
            &format!("jurisdiction does not exist: {jurisdiction_code}"),
            request_id,
        ));
    };
    let status: String = row.get(0);
    if status != "active" {
        return Err(billing_bad_request(
            &format!("jurisdiction is not active: {jurisdiction_code}"),
            request_id,
        ));
    }
    Ok(())
}

async fn resolve_provider_account_id(
    client: &Client,
    provider_key: &str,
    provider_account_id: Option<&str>,
    payee_subject_type: Option<&str>,
    payee_subject_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    if let Some(provider_account_id) = provider_account_id {
        let params: [&(dyn DbParam + Sync); 2] = [&provider_account_id, &provider_key];
        let row = client
            .query_opt(
                "SELECT provider_account_id::text
                 FROM payment.provider_account
                 WHERE provider_account_id = $1::text::uuid
                   AND provider_key = $2
                   AND status = 'active'",
                &params,
            )
            .await
            .map_err(map_db_error)?;
        return row
            .map(|row| row.get::<_, String>(0))
            .ok_or_else(|| {
                billing_bad_request(
                    "provider_account_id does not exist or does not belong to provider",
                    request_id,
                )
            })
            .map(Some);
    }

    let params: [&(dyn DbParam + Sync); 3] =
        [&provider_key, &payee_subject_type, &payee_subject_id];
    let row = client
        .query_opt(
            "SELECT provider_account_id::text
             FROM payment.provider_account
             WHERE provider_key = $1
               AND status = 'active'
               AND ($2::text IS NULL OR settlement_subject_type = $2)
               AND ($3::text::uuid IS NULL OR settlement_subject_id = $3::text::uuid)
             ORDER BY created_at DESC
             LIMIT 1",
            &params,
        )
        .await
        .map_err(map_db_error)?;

    if let Some(row) = row {
        return Ok(Some(row.get(0)));
    }
    if provider_key == "mock_payment" {
        return Ok(None);
    }
    Err(billing_bad_request(
        "provider account is required for the selected payment provider",
        request_id,
    ))
}

async fn resolve_corridor_policy_id(
    client: &Client,
    corridor_policy_id: Option<&str>,
    payer_jurisdiction_code: &str,
    payee_jurisdiction_code: &str,
    price_currency_code: &str,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let params: [&(dyn DbParam + Sync); 4] = [
        &corridor_policy_id,
        &payer_jurisdiction_code,
        &payee_jurisdiction_code,
        &price_currency_code,
    ];
    let query = if corridor_policy_id.is_some() {
        "SELECT corridor_policy_id::text
         FROM payment.corridor_policy
         WHERE corridor_policy_id = $1::text::uuid
           AND status = 'active'
           AND COALESCE((policy_snapshot ->> 'real_payment_enabled')::boolean, false) = true"
    } else {
        "SELECT corridor_policy_id::text
         FROM payment.corridor_policy
         WHERE payer_jurisdiction_code = $2
           AND payee_jurisdiction_code = $3
           AND product_scope = 'general'
           AND price_currency_code = $4
           AND status = 'active'
           AND COALESCE((policy_snapshot ->> 'real_payment_enabled')::boolean, false) = true
           AND (effective_from IS NULL OR effective_from <= now())
           AND (effective_to IS NULL OR effective_to > now())
         ORDER BY effective_from DESC NULLS LAST, created_at DESC
         LIMIT 1"
    };

    let row = client
        .query_opt(query, &params)
        .await
        .map_err(map_db_error)?;

    row.map(|row| row.get::<_, String>(0))
        .ok_or_else(|| {
            billing_bad_request("payment corridor is missing or not enabled", request_id)
        })
        .map(Some)
}

async fn validate_fee_preview(
    client: &Client,
    order_id: &str,
    fee_preview_id: Option<&str>,
    order: &OrderPaymentContext,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    if let Some(fee_preview_id) = fee_preview_id {
        let row = client
            .query_opt(
                "SELECT fee_preview_id::text
                 FROM billing.fee_preview
                 WHERE fee_preview_id = $1::text::uuid
                   AND order_id = $2::text::uuid",
                &[&fee_preview_id, &order_id],
            )
            .await
            .map_err(map_db_error)?;
        return row
            .map(|row| row.get::<_, String>(0))
            .ok_or_else(|| billing_bad_request("fee preview does not exist for order", None))
            .map(Some);
    }

    if order.fee_preview_snapshot.is_null()
        || order.fee_preview_snapshot == json!({})
        || order
            .fee_preview_snapshot
            .get("pricing_mode")
            .and_then(Value::as_str)
            .is_none()
    {
        return Err(billing_bad_request(
            "order fee preview snapshot is missing or invalid",
            None,
        ));
    }
    Ok(None)
}

async fn load_latest_transaction_summary(
    client: &Client,
    payment_intent_id: &str,
) -> Result<Option<PaymentTransactionSummaryView>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               payment_transaction_id::text,
               transaction_type,
               provider_transaction_no,
               provider_status,
               amount::text,
               currency_code,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.payment_transaction
             WHERE payment_intent_id = $1::text::uuid
             ORDER BY occurred_at DESC, created_at DESC
             LIMIT 1",
            &[&payment_intent_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(row.map(|row| PaymentTransactionSummaryView {
        payment_transaction_id: row.get(0),
        transaction_type: row.get(1),
        provider_transaction_no: row.get(2),
        provider_status: row.get(3),
        transaction_amount: row.get(4),
        currency_code: row.get(5),
        occurred_at: row.get(6),
    }))
}

async fn load_latest_webhook_summary(
    client: &Client,
    payment_intent_id: &str,
) -> Result<Option<PaymentWebhookSummaryView>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               webhook_event_id::text,
               provider_event_id,
               event_type,
               processed_status,
               duplicate_flag,
               signature_verified,
               to_char(received_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(processed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.payment_webhook_event
             WHERE payment_intent_id = $1::text::uuid
             ORDER BY received_at DESC
             LIMIT 1",
            &[&payment_intent_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(row.map(|row| PaymentWebhookSummaryView {
        webhook_event_id: row.get(0),
        provider_event_id: row.get(1),
        event_type: row.get(2),
        processed_status: row.get(3),
        duplicate_flag: row.get(4),
        signature_verified: row.get(5),
        received_at: row.get(6),
        processed_at: row.get(7),
    }))
}

fn enforce_intent_scope(
    tenant_scope_id: Option<&str>,
    intent: &PaymentIntentView,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    enforce_intent_scope_with_request(tenant_scope_id, intent, None)
}

fn enforce_intent_scope_with_request(
    tenant_scope_id: Option<&str>,
    intent: &PaymentIntentView,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(tenant_scope_id) = tenant_scope_id else {
        return Ok(());
    };
    if tenant_scope_id == intent.payer_subject_id
        || intent.payee_subject_id.as_deref() == Some(tenant_scope_id)
    {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "tenant scope does not match payment intent".to_string(),
            request_id: request_id.map(str::to_string),
        }),
    ))
}

fn billing_bad_request(
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn billing_forbidden(message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: message.to_string(),
            request_id: None,
        }),
    )
}

fn billing_not_found(
    payment_intent_id: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: format!("payment intent not found: {payment_intent_id}"),
            request_id: request_id.map(str::to_string),
        }),
    )
}
