use crate::AppState;
use crate::modules::billing::db::write_audit_event;
use crate::modules::billing::handlers::{
    header, map_db_connect, require_permission, require_step_up_placeholder,
};
use crate::modules::billing::models::{
    CreatePaymentIntentRequest, PaymentIntentDetailView, PaymentIntentView,
};
use crate::modules::billing::repo::payment_intent_repository::{
    cancel_payment_intent as cancel_payment_intent_repo,
    create_payment_intent as create_payment_intent_repo, get_payment_intent_detail,
};
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

pub async fn create_payment_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreatePaymentIntentRequest>,
) -> Result<Json<ApiResponse<PaymentIntentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PaymentIntentCreate,
        "payment intent create",
    )?;
    require_step_up_placeholder(&headers, "payment intent create")?;
    let request_id = header(&headers, "x-request-id");
    let idempotency_key = header(&headers, "x-idempotency-key").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: "x-idempotency-key is required for payment intent create".to_string(),
                request_id: request_id.clone(),
            }),
        )
    })?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let tenant_scope_id = tenant_scope_id_for_role(&headers, &role)?;
    let client = state.db.client().map_err(map_db_connect)?;

    let existing =
        crate::modules::billing::db::select_intent_by_idempotency(&client, &idempotency_key)
            .await?;
    let (intent, audit_action) = if let Some(existing) = existing {
        if let Some(ref tenant_scope_id) = tenant_scope_id {
            if existing.payer_subject_id != *tenant_scope_id
                && existing.payee_subject_id.as_deref() != Some(tenant_scope_id)
            {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        code: ErrorCode::IamUnauthorized.as_str().to_string(),
                        message: "tenant scope does not match payment intent".to_string(),
                        request_id: request_id.clone(),
                    }),
                ));
            }
        }
        (existing, "payment.intent.create.idempotent_replay")
    } else {
        (
            create_payment_intent_repo(
                &client,
                &payload,
                request_id.as_deref(),
                &idempotency_key,
                tenant_scope_id.as_deref(),
            )
            .await?,
            "payment.intent.create",
        )
    };

    write_audit_event(
        &client,
        "payment",
        "payment_intent",
        &intent.payment_intent_id,
        role.as_str(),
        audit_action,
        "success",
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = audit_action,
        payment_intent_id = %intent.payment_intent_id,
        provider_key = %intent.provider_key,
        payment_amount = %intent.payment_amount,
        "payment intent create handled"
    );
    Ok(ApiResponse::ok(intent))
}

pub async fn get_payment_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PaymentIntentDetailView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PaymentIntentRead,
        "payment intent read",
    )?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let tenant_scope_id = tenant_scope_id_for_role(&headers, &role)?;
    let client = state.db.client().map_err(map_db_connect)?;
    let detail = get_payment_intent_detail(
        &client,
        &id,
        tenant_scope_id.as_deref(),
        header(&headers, "x-request-id").as_deref(),
    )
    .await?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("payment intent not found: {id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;

    write_audit_event(
        &client,
        "payment",
        "payment_intent",
        &detail.payment_intent.payment_intent_id,
        role.as_str(),
        "payment.intent.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "payment.intent.read",
        payment_intent_id = %detail.payment_intent.payment_intent_id,
        "payment intent queried"
    );
    Ok(ApiResponse::ok(detail))
}

pub async fn cancel_payment_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PaymentIntentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::PaymentIntentCancel,
        "payment intent cancel",
    )?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let tenant_scope_id = tenant_scope_id_for_role(&headers, &role)?;
    let client = state.db.client().map_err(map_db_connect)?;
    let detail = get_payment_intent_detail(
        &client,
        &id,
        tenant_scope_id.as_deref(),
        header(&headers, "x-request-id").as_deref(),
    )
    .await?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::BilProviderFailed.as_str().to_string(),
                message: format!("payment intent not found: {id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;
    let already_canceled = detail.payment_intent.payment_status == "canceled";
    let intent = cancel_payment_intent_repo(
        &client,
        &id,
        tenant_scope_id.as_deref(),
        header(&headers, "x-request-id").as_deref(),
    )
    .await?;
    let audit_action = if already_canceled {
        "payment.intent.cancel.idempotent_replay"
    } else {
        "payment.intent.cancel"
    };
    write_audit_event(
        &client,
        "payment",
        "payment_intent",
        &intent.payment_intent_id,
        role.as_str(),
        audit_action,
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = audit_action,
        payment_intent_id = %intent.payment_intent_id,
        payment_status = %intent.payment_status,
        "payment intent cancel handled"
    );
    Ok(ApiResponse::ok(intent))
}

fn tenant_scope_id_for_role(
    headers: &HeaderMap,
    role: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    if !matches!(role, "tenant_admin" | "tenant_operator") {
        return Ok(None);
    }
    header(headers, "x-tenant-id").map(Some).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "x-tenant-id is required for tenant-scoped payment intent actions"
                    .to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })
}
