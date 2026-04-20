use crate::modules::billing::db::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct MockPaymentIntentContext {
    pub payment_intent_id: String,
    pub order_id: String,
    pub provider_key: String,
    pub payer_subject_id: String,
    pub payee_subject_id: Option<String>,
    pub payment_status: String,
}

pub async fn load_mock_payment_intent_context(
    client: &Client,
    payment_intent_id: &str,
    tenant_scope_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<MockPaymentIntentContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT payment_intent_id::text, order_id::text, provider_key,
                    payer_subject_type, payer_subject_id::text, payee_subject_type, payee_subject_id::text, status
             FROM payment.payment_intent
             WHERE payment_intent_id = $1::text::uuid",
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
    let payer_subject_type: String = row.get(3);
    let payee_subject_type: Option<String> = row.get(5);
    let payer_subject_id: String = row.get(4);
    let payee_subject_id: Option<String> = row.get(6);
    if payer_subject_type != "organization" {
        return Err(billing_conflict(
            "mock payment simulate only supports organization payer",
            request_id,
        ));
    }
    if let Some(tenant_scope_id) = tenant_scope_id {
        let payee_matches_org = payee_subject_type.as_deref() == Some("organization")
            && payee_subject_id.as_deref() == Some(tenant_scope_id);
        if payer_subject_id != tenant_scope_id && !payee_matches_org {
            return Err(billing_forbidden(
                "tenant scope does not match payment intent subjects",
                request_id,
            ));
        }
    }
    let provider_key: String = row.get(2);
    if provider_key != "mock_payment" {
        return Err(billing_conflict(
            "mock simulate only supports provider `mock_payment`",
            request_id,
        ));
    }
    Ok(MockPaymentIntentContext {
        payment_intent_id: row.get(0),
        order_id: row.get(1),
        provider_key,
        payer_subject_id,
        payee_subject_id,
        payment_status: row.get(7),
    })
}

pub async fn create_mock_payment_case(
    client: &Client,
    payment_intent_id: &str,
    provider_key: &str,
    scenario_type: &str,
    delay_seconds: i32,
    duplicate_webhook: bool,
    partial_refund_amount: Option<&str>,
    payload: &Value,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_one(
            "INSERT INTO developer.mock_payment_case (
               payment_intent_id,
               provider_key,
               scenario_type,
               delay_seconds,
               duplicate_webhook,
               partial_refund_amount,
               payload,
               status
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               $4,
               $5,
               $6::text::numeric,
               $7::jsonb,
               'pending'
             )
             RETURNING mock_payment_case_id::text",
            &[
                &payment_intent_id,
                &provider_key,
                &scenario_type,
                &delay_seconds,
                &duplicate_webhook,
                &partial_refund_amount,
                payload,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(row.get(0))
}

pub async fn update_mock_payment_case_result(
    client: &Client,
    mock_payment_case_id: &str,
    status: &str,
    payload: &Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    client
        .execute(
            "UPDATE developer.mock_payment_case
             SET status = $2,
                 payload = payload || $3::jsonb,
                 executed_at = CASE WHEN $2 = 'executed' THEN now() ELSE executed_at END,
                 updated_at = now()
             WHERE mock_payment_case_id = $1::text::uuid",
            &[&mock_payment_case_id, &status, payload],
        )
        .await
        .map_err(map_db_error)?;
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
