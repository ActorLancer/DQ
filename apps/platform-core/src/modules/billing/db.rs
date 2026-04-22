use crate::modules::audit::application::{
    AuditWriteCommand, write_audit_event as write_unified_audit_event,
};
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
        provider_account_id: row.get::<_, Option<String>>(4),
        payer_subject_type: row.get::<_, String>(5),
        payer_subject_id: row.get::<_, String>(6),
        payee_subject_type: row.get::<_, Option<String>>(7),
        payee_subject_id: row.get::<_, Option<String>>(8),
        payer_jurisdiction_code: row.get::<_, Option<String>>(9),
        payee_jurisdiction_code: row.get::<_, Option<String>>(10),
        launch_jurisdiction_code: row.get::<_, String>(11),
        corridor_policy_id: row.get::<_, Option<String>>(12),
        fee_preview_id: row.get::<_, Option<String>>(13),
        payment_amount: row.get::<_, String>(14),
        payment_method: row.get::<_, String>(15),
        currency_code: row.get::<_, String>(16),
        price_currency_code: row.get::<_, String>(17),
        payment_status: row.get::<_, String>(18),
        provider_intent_no: row.get::<_, Option<String>>(19),
        channel_reference_no: row.get::<_, Option<String>>(20),
        idempotency_key: row.get::<_, Option<String>>(21),
        request_id: row.get::<_, Option<String>>(22),
        expire_at: row.get::<_, Option<String>>(23),
        capability_snapshot: row.get::<_, serde_json::Value>(24),
        metadata: row.get::<_, serde_json::Value>(25),
        created_at: row.get::<_, String>(26),
        updated_at: row.get::<_, String>(27),
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
    client: &(impl GenericClient + Sync),
    domain_name: &str,
    ref_type: &str,
    ref_id: &str,
    actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    write_unified_audit_event(
        client,
        &AuditWriteCommand {
            domain_name: domain_name.to_string(),
            ref_type: ref_type.to_string(),
            ref_id: Some(ref_id.to_string()),
            actor_type: "role".to_string(),
            actor_id: None,
            actor_org_id: None,
            tenant_id: None,
            action_name: action_name.to_string(),
            result_code: result_code.to_string(),
            error_code: None,
            request_id: request_id.map(str::to_string),
            trace_id: trace_id.map(str::to_string),
            auth_assurance_level: None,
            step_up_challenge_id: None,
            sensitivity_level: None,
            metadata: serde_json::json!({
                "actor_role": actor_role,
                "writer": "audit.application.write_audit_event",
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

pub async fn write_audit_event_without_ref(
    client: &(impl GenericClient + Sync),
    domain_name: &str,
    ref_type: &str,
    actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    write_unified_audit_event(
        client,
        &AuditWriteCommand {
            domain_name: domain_name.to_string(),
            ref_type: ref_type.to_string(),
            ref_id: None,
            actor_type: "role".to_string(),
            actor_id: None,
            actor_org_id: None,
            tenant_id: None,
            action_name: action_name.to_string(),
            result_code: result_code.to_string(),
            error_code: None,
            request_id: request_id.map(str::to_string),
            trace_id: trace_id.map(str::to_string),
            auth_assurance_level: None,
            step_up_challenge_id: None,
            sensitivity_level: None,
            metadata: serde_json::json!({
                "actor_role": actor_role,
                "writer": "audit.application.write_audit_event",
            }),
        },
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
