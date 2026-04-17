use crate::modules::billing::domain::{CorridorPolicy, JurisdictionProfile, PayoutPreference};
use crate::modules::billing::service::{
    BillingPermission, is_allowed, list_corridor_policies, list_jurisdictions,
    list_payout_preferences,
};
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use tracing::info;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingPolicyView {
    pub jurisdictions: Vec<JurisdictionProfile>,
    pub corridor_policies: Vec<CorridorPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub order_id: String,
    pub provider_key: String,
    pub payer_subject_type: String,
    pub payer_subject_id: String,
    pub payee_subject_type: Option<String>,
    pub payee_subject_id: Option<String>,
    pub amount: String,
    pub payment_method: String,
    pub currency_code: Option<String>,
    pub price_currency_code: Option<String>,
    pub intent_type: Option<String>,
    pub payer_jurisdiction_code: Option<String>,
    pub payee_jurisdiction_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaymentIntentView {
    pub payment_intent_id: String,
    pub order_id: String,
    pub intent_type: String,
    pub provider_key: String,
    pub payer_subject_type: String,
    pub payer_subject_id: String,
    pub payee_subject_type: Option<String>,
    pub payee_subject_id: Option<String>,
    pub amount: String,
    pub payment_method: String,
    pub currency_code: String,
    pub price_currency_code: String,
    pub status: String,
    pub idempotency_key: Option<String>,
    pub request_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LockOrderRequest {
    pub payment_intent_id: String,
    pub lock_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OrderLockView {
    pub order_id: String,
    pub payment_intent_id: String,
    pub order_status: String,
    pub payment_status: String,
    pub buyer_locked_at: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/billing/policies", get(get_billing_policies))
        .route(
            "/api/v1/billing/payout-preferences/{beneficiary_subject_id}",
            get(get_payout_preferences),
        )
        .route("/api/v1/payments/intents", post(create_payment_intent))
        .route("/api/v1/payments/intents/{id}", get(get_payment_intent))
        .route(
            "/api/v1/payments/intents/{id}/cancel",
            post(cancel_payment_intent),
        )
        .route("/api/v1/orders/{id}/lock", post(lock_order_payment))
}

async fn get_billing_policies(
    headers: HeaderMap,
) -> Result<Json<ApiResponse<BillingPolicyView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::ReadPolicy,
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

async fn get_payout_preferences(
    headers: HeaderMap,
    Path(beneficiary_subject_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<PayoutPreference>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::ReadPolicy,
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

async fn create_payment_intent(
    headers: HeaderMap,
    Json(payload): Json<CreatePaymentIntentRequest>,
) -> Result<Json<ApiResponse<PaymentIntentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PaymentIntentCreate,
        "payment intent create",
    )?;
    let request_id = header(&headers, "x-request-id");
    let idempotency_key = header(&headers, "x-idempotency-key");
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    if let Some(ref key) = idempotency_key {
        if let Some(existing) = select_intent_by_idempotency(&client, key).await? {
            info!(
                action = "payment.intent.create.idempotent_replay",
                payment_intent_id = %existing.payment_intent_id,
                idempotency_key = %key,
                "payment intent idempotent replay"
            );
            return Ok(ApiResponse::ok(existing));
        }
    }

    let row = client
        .query_one(
            "INSERT INTO payment.payment_intent (
               order_id, intent_type, provider_key, payer_subject_type, payer_subject_id,
               payee_subject_type, payee_subject_id, payer_jurisdiction_code, payee_jurisdiction_code,
               launch_jurisdiction_code, amount, payment_method, currency_code, price_currency_code,
               status, request_id, idempotency_key, metadata
             ) VALUES (
               $1::text::uuid, $2, $3, $4, $5::text::uuid, $6, $7::text::uuid, $8, $9, 'SG',
               $10::text::numeric, $11, $12, $13, 'created', $14, $15, '{}'::jsonb
             )
             RETURNING
               payment_intent_id::text,
               order_id::text,
               intent_type,
               provider_key,
               payer_subject_type,
               payer_subject_id::text,
               payee_subject_type,
               payee_subject_id::text,
               amount::text,
               payment_method,
               currency_code,
               price_currency_code,
               status,
               idempotency_key,
               request_id,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &payload.order_id,
                &payload
                    .intent_type
                    .clone()
                    .unwrap_or_else(|| "order_payment".to_string()),
                &payload.provider_key,
                &payload.payer_subject_type,
                &payload.payer_subject_id,
                &payload.payee_subject_type,
                &payload.payee_subject_id,
                &payload
                    .payer_jurisdiction_code
                    .clone()
                    .unwrap_or_else(|| "SG".to_string()),
                &payload
                    .payee_jurisdiction_code
                    .clone()
                    .unwrap_or_else(|| "SG".to_string()),
                &payload.amount,
                &payload.payment_method,
                &payload
                    .currency_code
                    .clone()
                    .unwrap_or_else(|| "SGD".to_string()),
                &payload
                    .price_currency_code
                    .clone()
                    .unwrap_or_else(|| "USD".to_string()),
                &request_id,
                &idempotency_key,
            ],
        )
        .await
        .map_err(map_db_error)?;

    let intent = parse_intent_row(&row)?;
    info!(
        action = "payment.intent.create",
        payment_intent_id = %intent.payment_intent_id,
        provider_key = %intent.provider_key,
        amount = %intent.amount,
        "payment intent created"
    );
    Ok(ApiResponse::ok(intent))
}

async fn get_payment_intent(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PaymentIntentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PaymentIntentRead,
        "payment intent read",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT
               payment_intent_id::text,
               order_id::text,
               intent_type,
               provider_key,
               payer_subject_type,
               payer_subject_id::text,
               payee_subject_type,
               payee_subject_id::text,
               amount::text,
               payment_method,
               currency_code,
               price_currency_code,
               status,
               idempotency_key,
               request_id,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.payment_intent
             WHERE payment_intent_id = $1::text::uuid",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;

    if let Some(row) = row {
        let intent = parse_intent_row(&row)?;
        info!(
            action = "payment.intent.read",
            payment_intent_id = %intent.payment_intent_id,
            "payment intent queried"
        );
        Ok(ApiResponse::ok(intent))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("payment intent not found: {id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ))
    }
}

async fn cancel_payment_intent(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PaymentIntentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PaymentIntentCancel,
        "payment intent cancel",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let status_row = client
        .query_opt(
            "SELECT status FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(status_row) = status_row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("payment intent not found: {id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    let current_status: String = status_row.get(0);
    if matches!(current_status.as_str(), "succeeded" | "failed" | "expired") {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("payment intent cannot be canceled from status {current_status}"),
                request_id: header(&headers, "x-request-id"),
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
               payer_subject_type,
               payer_subject_id::text,
               payee_subject_type,
               payee_subject_id::text,
               amount::text,
               payment_method,
               currency_code,
               price_currency_code,
               status,
               idempotency_key,
               request_id,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let intent = parse_intent_row(&row)?;
    info!(
        action = "payment.intent.cancel",
        payment_intent_id = %intent.payment_intent_id,
        "payment intent canceled"
    );
    Ok(ApiResponse::ok(intent))
}

async fn lock_order_payment(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<LockOrderRequest>,
) -> Result<Json<ApiResponse<OrderLockView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, BillingPermission::OrderLock, "order lock")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

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

    let old_status_row = client
        .query_one(
            "SELECT status FROM trade.order_main WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let current_status: String = old_status_row.get(0);
    let _ = client
        .execute(
            "INSERT INTO trade.order_status_history
             (order_id, old_status, new_status, changed_by_type, reason_code)
             VALUES ($1::text::uuid, $2, $3, 'system', COALESCE($4::text, 'payment_lock'))",
            &[
                &order_id,
                &current_status,
                &current_status,
                &payload.lock_reason,
            ],
        )
        .await
        .map_err(map_db_error)?;

    let view = OrderLockView {
        order_id: row.get::<_, String>(0),
        payment_intent_id: payload.payment_intent_id.clone(),
        order_status: row.get::<_, String>(1),
        payment_status: row.get::<_, String>(2),
        buyer_locked_at: row.get::<_, String>(3),
    };
    info!(
        action = "order.payment.lock",
        order_id = %view.order_id,
        payment_intent_id = %view.payment_intent_id,
        payment_status = %view.payment_status,
        "order lock operation completed"
    );
    Ok(ApiResponse::ok(view))
}

fn parse_intent_row(
    row: &tokio_postgres::Row,
) -> Result<PaymentIntentView, (StatusCode, Json<ErrorResponse>)> {
    Ok(PaymentIntentView {
        payment_intent_id: row.get::<_, String>(0),
        order_id: row.get::<_, String>(1),
        intent_type: row.get::<_, String>(2),
        provider_key: row.get::<_, String>(3),
        payer_subject_type: row.get::<_, String>(4),
        payer_subject_id: row.get::<_, String>(5),
        payee_subject_type: row.get::<_, Option<String>>(6),
        payee_subject_id: row.get::<_, Option<String>>(7),
        amount: row.get::<_, String>(8),
        payment_method: row.get::<_, String>(9),
        currency_code: row.get::<_, String>(10),
        price_currency_code: row.get::<_, String>(11),
        status: row.get::<_, String>(12),
        idempotency_key: row.get::<_, Option<String>>(13),
        request_id: row.get::<_, Option<String>>(14),
        created_at: row.get::<_, String>(15),
        updated_at: row.get::<_, String>(16),
    })
}

async fn select_intent_by_idempotency(
    client: &tokio_postgres::Client,
    idempotency_key: &str,
) -> Result<Option<PaymentIntentView>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               payment_intent_id::text,
               order_id::text,
               intent_type,
               provider_key,
               payer_subject_type,
               payer_subject_id::text,
               payee_subject_type,
               payee_subject_id::text,
               amount::text,
               payment_method,
               currency_code,
               price_currency_code,
               status,
               idempotency_key,
               request_id,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.payment_intent
             WHERE idempotency_key = $1",
            &[&idempotency_key],
        )
        .await
        .map_err(map_db_error)?;
    row.map(|r| parse_intent_row(&r)).transpose()
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

fn require_permission(
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

fn database_dsn() -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    std::env::var("DATABASE_URL").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: ErrorCode::OpsInternal.as_str().to_string(),
                message: "DATABASE_URL is not configured".to_string(),
                request_id: None,
            }),
        )
    })
}

async fn connect_db(
    dsn: &str,
) -> Result<
    (
        tokio_postgres::Client,
        tokio_postgres::Connection<tokio_postgres::Socket, tokio_postgres::tls::NoTlsStream>,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    tokio_postgres::connect(dsn, NoTls).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: ErrorCode::OpsInternal.as_str().to_string(),
                message: format!("database connection failed: {err}"),
                request_id: None,
            }),
        )
    })
}

fn map_db_error(err: tokio_postgres::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: format!("payment intent persistence failed: {err}"),
            request_id: None,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn rejects_policy_request_without_role() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/billing/policies")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_payment_intent_without_permission() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments/intents")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"order_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","provider_key":"mock_payment","payer_subject_type":"organization","payer_subject_id":"0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56","amount":"10.00","payment_method":"wallet"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_cancel_payment_intent_for_tenant_operator() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments/intents/0e4f4f8f-26e2-4d0f-89a6-8e57421cbf56/cancel")
                    .method("POST")
                    .header("x-role", "tenant_operator")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_lock_order_for_tenant_operator() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/orders/30000000-0000-0000-0000-000000000101/lock")
                    .method("POST")
                    .header("x-role", "tenant_operator")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"payment_intent_id":"4f4b3a2e-508b-4902-ba35-97aa905b3772"}"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
