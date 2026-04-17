use crate::modules::order::domain::{PaymentResultKind, derive_target_state};
use axum::Json;
use axum::http::StatusCode;
use kernel::{ErrorCode, ErrorResponse};

pub async fn apply_payment_result_to_order(
    client: &tokio_postgres::Client,
    order_id: &str,
    result: PaymentResultKind,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT status, payment_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let current_status: String = row.get(0);
    let target = derive_target_state(&current_status, result);
    let Some((next_order_status, next_payment_status, reason_code)) = target else {
        write_order_audit(
            client,
            order_id,
            "order.payment.result.ignored",
            "ignored",
            request_id,
            trace_id,
            Some(current_status.as_str()),
            None,
        )
        .await?;
        return Ok(None);
    };

    let _ = client
        .execute(
            "UPDATE trade.order_main
             SET
               status = $2,
               payment_status = $3,
               last_reason_code = $4,
               updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[
                &order_id,
                &next_order_status,
                &next_payment_status,
                &reason_code,
            ],
        )
        .await
        .map_err(map_db_error)?;

    write_order_audit(
        client,
        order_id,
        "order.payment.result.applied",
        "success",
        request_id,
        trace_id,
        Some(current_status.as_str()),
        Some(next_order_status),
    )
    .await?;
    Ok(Some(next_order_status.to_string()))
}

async fn write_order_audit(
    client: &tokio_postgres::Client,
    order_id: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    previous_status: Option<&str>,
    next_status: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let metadata = serde_json::json!({
        "actor_role": "system",
        "previous_status": previous_status,
        "next_status": next_status
    });
    let _ = client
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
               'trade',
               'order',
               $1::text::uuid,
               'system',
               $2,
               $3,
               $4,
               $5,
               $6::jsonb
             )",
            &[
                &order_id,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn map_db_error(err: tokio_postgres::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("order payment orchestration failed: {err}"),
            request_id: None,
        }),
    )
}
