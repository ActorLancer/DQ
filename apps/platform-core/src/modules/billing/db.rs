use crate::modules::billing::models::PaymentIntentView;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};

pub fn parse_intent_row(row: &Row) -> Result<PaymentIntentView, (StatusCode, Json<ErrorResponse>)> {
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

pub async fn select_intent_by_idempotency(
    client: &Client,
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

pub async fn set_webhook_processed_status(
    client: &Client,
    webhook_event_id: &str,
    status: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let _ = client
        .execute(
            "UPDATE payment.payment_webhook_event
             SET processed_status = $2, processed_at = now()
             WHERE webhook_event_id = $1::text::uuid",
            &[&webhook_event_id, &status],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

pub async fn write_audit_event(
    client: &Client,
    domain_name: &str,
    ref_type: &str,
    ref_id: &str,
    actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id.map(str::to_string);
    let trace_id = trace_id.map(str::to_string);
    client
        .execute(
            "INSERT INTO audit.audit_event (
               domain_name,
               ref_type,
               ref_id,
               actor_type,
               action_name,
               result_code,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1,
               $2,
               $3::text::uuid,
               'role',
               $4,
               $5,
               $6,
               $7,
               jsonb_build_object('actor_role', $8::text)
             )",
            &[
                &domain_name,
                &ref_type,
                &ref_id,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &actor_role,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

pub async fn write_audit_event_without_ref(
    client: &Client,
    domain_name: &str,
    ref_type: &str,
    actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id.map(str::to_string);
    let trace_id = trace_id.map(str::to_string);
    client
        .execute(
            "INSERT INTO audit.audit_event (
               domain_name,
               ref_type,
               ref_id,
               actor_type,
               action_name,
               result_code,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1,
               $2,
               NULL,
               'role',
               $3,
               $4,
               $5,
               $6,
               jsonb_build_object('actor_role', $7::text)
             )",
            &[
                &domain_name,
                &ref_type,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &actor_role,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

pub fn map_db_error(err: Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: format!("billing persistence failed: {err}"),
            request_id: None,
        }),
    )
}
