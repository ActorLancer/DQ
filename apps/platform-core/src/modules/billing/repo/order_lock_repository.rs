use crate::modules::billing::db::map_db_error;
use crate::modules::billing::models::{LockOrderRequest, OrderLockView};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::Value;

struct OrderLockContext {
    order_id: String,
    buyer_org_id: String,
    seller_org_id: String,
    status: String,
    payment_status: String,
    amount: String,
    currency_code: String,
    payment_channel_snapshot: Value,
    buyer_locked_at: Option<String>,
}

struct PaymentIntentLockContext {
    order_id: String,
    provider_key: String,
    provider_account_id: Option<String>,
    provider_intent_no: Option<String>,
    channel_reference_no: Option<String>,
    payer_subject_type: String,
    payer_subject_id: String,
    payee_subject_type: Option<String>,
    payee_subject_id: Option<String>,
    payment_amount: String,
    currency_code: String,
    payment_status: String,
}

pub async fn lock_order_payment(
    client: &Client,
    order_id: &str,
    payload: &LockOrderRequest,
    tenant_scope_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<(OrderLockView, bool), (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;

    let order = load_order_for_update(&tx, order_id, request_id).await?;
    enforce_order_scope(tenant_scope_id, &order, request_id)?;

    let intent =
        load_payment_intent_for_update(&tx, &payload.payment_intent_id, request_id).await?;
    enforce_payment_intent_matches_order(&order, &intent, request_id)?;
    enforce_lockable_state(&order, &intent, request_id)?;

    if let Some(existing_payment_intent_id) = order
        .payment_channel_snapshot
        .get("payment_intent_id")
        .and_then(Value::as_str)
    {
        if existing_payment_intent_id == payload.payment_intent_id
            && matches!(order.payment_status.as_str(), "locked" | "paid")
        {
            let view = OrderLockView {
                order_id: order.order_id,
                payment_intent_id: payload.payment_intent_id.clone(),
                order_status: order.status,
                payment_status: order.payment_status,
                buyer_locked_at: order.buyer_locked_at.unwrap_or_default(),
            };
            tx.commit().await.map_err(map_db_error)?;
            return Ok((view, true));
        }
        return Err(billing_conflict(
            "order is already linked to another payment intent",
            request_id,
        ));
    }

    let row = tx
        .query_one(
            "UPDATE trade.order_main
             SET
               payment_status = 'locked',
               buyer_locked_at = COALESCE(buyer_locked_at, now()),
               payment_channel_snapshot = jsonb_strip_nulls(
                 COALESCE(payment_channel_snapshot, '{}'::jsonb) || jsonb_build_object(
                   'payment_intent_id', $2::text,
                   'provider_key', $3::text,
                   'provider_account_id', $4::text,
                   'provider_intent_no', $5::text,
                   'channel_reference_no', $6::text,
                   'payment_amount', $7::text,
                   'currency_code', $8::text,
                   'lock_reason', COALESCE($9::text, 'payment_lock'),
                   'locked_at', to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 )
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
                &intent.provider_key,
                &intent.provider_account_id,
                &intent.provider_intent_no,
                &intent.channel_reference_no,
                &intent.payment_amount,
                &intent.currency_code,
                &payload.lock_reason,
            ],
        )
        .await
        .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;

    Ok((
        OrderLockView {
            order_id: row.get::<_, String>(0),
            payment_intent_id: payload.payment_intent_id.clone(),
            order_status: row.get::<_, String>(1),
            payment_status: row.get::<_, String>(2),
            buyer_locked_at: row.get::<_, String>(3),
        },
        false,
    ))
}

async fn load_order_for_update(
    client: &impl GenericClient,
    order_id: &str,
    request_id: Option<&str>,
) -> Result<OrderLockContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               order_id::text,
               buyer_org_id::text,
               seller_org_id::text,
               status,
               payment_status,
               amount::text,
               currency_code,
               COALESCE(payment_channel_snapshot, '{}'::jsonb),
               to_char(buyer_locked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.order_main
             WHERE order_id = $1::text::uuid
             FOR UPDATE",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(billing_not_found(
            &format!("order not found: {order_id}"),
            request_id,
        ));
    };
    Ok(OrderLockContext {
        order_id: row.get(0),
        buyer_org_id: row.get(1),
        seller_org_id: row.get(2),
        status: row.get(3),
        payment_status: row.get(4),
        amount: row.get(5),
        currency_code: row.get(6),
        payment_channel_snapshot: row.get(7),
        buyer_locked_at: row.get(8),
    })
}

async fn load_payment_intent_for_update(
    client: &impl GenericClient,
    payment_intent_id: &str,
    request_id: Option<&str>,
) -> Result<PaymentIntentLockContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               order_id::text,
               provider_key,
               provider_account_id::text,
               provider_intent_no,
               channel_reference_no,
               payer_subject_type,
               payer_subject_id::text,
               payee_subject_type,
               payee_subject_id::text,
               amount::text,
               currency_code,
               status
             FROM payment.payment_intent
             WHERE payment_intent_id = $1::text::uuid
             FOR UPDATE",
            &[&payment_intent_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(billing_not_found(
            &format!("payment intent not found: {payment_intent_id}"),
            request_id,
        ));
    };
    Ok(parse_payment_intent_row(row))
}

fn parse_payment_intent_row(row: Row) -> PaymentIntentLockContext {
    PaymentIntentLockContext {
        order_id: row.get(0),
        provider_key: row.get(1),
        provider_account_id: row.get(2),
        provider_intent_no: row.get(3),
        channel_reference_no: row.get(4),
        payer_subject_type: row.get(5),
        payer_subject_id: row.get(6),
        payee_subject_type: row.get(7),
        payee_subject_id: row.get(8),
        payment_amount: row.get(9),
        currency_code: row.get(10),
        payment_status: row.get(11),
    }
}

fn enforce_order_scope(
    tenant_scope_id: Option<&str>,
    order: &OrderLockContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(tenant_scope_id) = tenant_scope_id else {
        return Ok(());
    };
    if tenant_scope_id == order.buyer_org_id {
        return Ok(());
    }
    Err(billing_forbidden(
        "tenant scope does not match buyer organization for order lock",
        request_id,
    ))
}

fn enforce_payment_intent_matches_order(
    order: &OrderLockContext,
    intent: &PaymentIntentLockContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if intent.order_id != order.order_id {
        return Err(billing_conflict(
            "payment intent does not belong to order",
            request_id,
        ));
    }
    if intent.payer_subject_type != "organization" || intent.payer_subject_id != order.buyer_org_id
    {
        return Err(billing_conflict(
            "payment intent payer does not match order buyer",
            request_id,
        ));
    }
    if let (Some(payee_subject_type), Some(payee_subject_id)) =
        (&intent.payee_subject_type, &intent.payee_subject_id)
    {
        if payee_subject_type != "organization" || payee_subject_id != &order.seller_org_id {
            return Err(billing_conflict(
                "payment intent payee does not match order seller",
                request_id,
            ));
        }
    }
    if intent.payment_amount != order.amount || intent.currency_code != order.currency_code {
        return Err(billing_conflict(
            "payment intent amount or currency does not match order snapshot",
            request_id,
        ));
    }
    Ok(())
}

fn enforce_lockable_state(
    order: &OrderLockContext,
    intent: &PaymentIntentLockContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if !matches!(order.status.as_str(), "created" | "contract_effective") {
        return Err(billing_conflict(
            &format!("order is not lockable from status {}", order.status),
            request_id,
        ));
    }
    if !matches!(
        order.payment_status.as_str(),
        "unpaid" | "failed" | "expired" | "locked" | "paid"
    ) {
        return Err(billing_conflict(
            &format!(
                "order is not lockable from payment_status {}",
                order.payment_status
            ),
            request_id,
        ));
    }
    if !matches!(
        intent.payment_status.as_str(),
        "created" | "pending" | "processing" | "succeeded"
    ) {
        return Err(billing_conflict(
            &format!(
                "payment intent is not lockable from status {}",
                intent.payment_status
            ),
            request_id,
        ));
    }
    Ok(())
}

fn billing_not_found(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn billing_conflict(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn billing_forbidden(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
