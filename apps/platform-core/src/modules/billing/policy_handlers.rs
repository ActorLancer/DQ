use crate::AppState;
use crate::modules::billing::db::{write_audit_event, write_audit_event_without_ref};
use crate::modules::billing::handlers::{header, map_db_connect, require_permission};
use crate::modules::billing::models::{
    CreateCorridorPolicyRequest, CreateJurisdictionProfileRequest, CreatePayoutPreferenceRequest,
    ListPayoutPreferenceQuery,
};
use crate::modules::billing::repo::policy_repository::{
    create_default_payout_preference, list_corridors, list_jurisdictions, list_payout_preferences,
    upsert_corridor, upsert_jurisdiction,
};
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

pub async fn get_payment_jurisdictions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<Vec<crate::modules::billing::domain::JurisdictionProfile>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        BillingPermission::JurisdictionRead,
        "payment jurisdiction read",
    )?;
    let client = state.db.client().map_err(map_db_connect)?;
    let rows = list_jurisdictions(&client).await?;
    write_audit_event_without_ref(
        &client,
        "payment",
        "payment_jurisdiction",
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "payment.jurisdiction.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "payment.jurisdiction.read",
        count = rows.len(),
        "payment jurisdictions read"
    );
    Ok(ApiResponse::ok(rows))
}

pub async fn create_payment_jurisdiction(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateJurisdictionProfileRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::billing::domain::JurisdictionProfile>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        BillingPermission::JurisdictionManage,
        "payment jurisdiction manage",
    )?;
    require_step_up_placeholder(&headers, "payment jurisdiction manage")?;
    let client = state.db.client().map_err(map_db_connect)?;
    let view = upsert_jurisdiction(&client, &payload).await?;
    write_audit_event_without_ref(
        &client,
        "payment",
        "payment_jurisdiction",
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "payment.jurisdiction.manage",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "payment.jurisdiction.manage",
        jurisdiction_code = %view.jurisdiction_code,
        "payment jurisdiction upserted"
    );
    Ok(ApiResponse::ok(view))
}

pub async fn get_payment_corridors(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<Vec<crate::modules::billing::domain::CorridorPolicy>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        BillingPermission::CorridorRead,
        "payment corridor read",
    )?;
    let client = state.db.client().map_err(map_db_connect)?;
    let rows = list_corridors(&client).await?;
    write_audit_event_without_ref(
        &client,
        "payment",
        "payment_corridor",
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "payment.corridor.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "payment.corridor.read",
        count = rows.len(),
        "payment corridors read"
    );
    Ok(ApiResponse::ok(rows))
}

pub async fn create_payment_corridor(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateCorridorPolicyRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::billing::domain::CorridorPolicy>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        BillingPermission::CorridorManage,
        "payment corridor manage",
    )?;
    require_step_up_placeholder(&headers, "payment corridor manage")?;
    let client = state.db.client().map_err(map_db_connect)?;
    let view = upsert_corridor(&client, &payload).await?;
    write_audit_event(
        &client,
        "payment",
        "payment_corridor",
        &view.corridor_policy_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "payment.corridor.manage",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "payment.corridor.manage",
        corridor_policy_id = %view.corridor_policy_id,
        "payment corridor upserted"
    );
    Ok(ApiResponse::ok(view))
}

pub async fn list_payout_preferences_v1(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListPayoutPreferenceQuery>,
) -> Result<
    Json<ApiResponse<Vec<crate::modules::billing::domain::PayoutPreference>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        BillingPermission::PayoutPreferenceRead,
        "payment payout preference read",
    )?;
    let (subject_type, subject_id) = resolve_payout_scope(&headers, &query)?;
    let client = state.db.client().map_err(map_db_connect)?;
    let rows =
        list_payout_preferences(&client, subject_type.as_deref(), subject_id.as_deref()).await?;
    write_audit_event_without_ref(
        &client,
        "payment",
        "payment_payout_preference",
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "payment.payout_preference.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "payment.payout_preference.read",
        count = rows.len(),
        "payment payout preferences read"
    );
    Ok(ApiResponse::ok(rows))
}

pub async fn create_payout_preference(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreatePayoutPreferenceRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::billing::domain::PayoutPreference>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        BillingPermission::PayoutPreferenceManage,
        "payment payout preference manage",
    )?;
    enforce_payout_subject_scope(
        &headers,
        &payload.beneficiary_subject_type,
        &payload.beneficiary_subject_id,
    )?;
    let client = state.db.client().map_err(map_db_connect)?;
    let view = create_default_payout_preference(&client, &payload).await?;
    write_audit_event(
        &client,
        "payment",
        "payment_payout_preference",
        &view.payout_preference_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "payment.payout_preference.manage",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "payment.payout_preference.manage",
        payout_preference_id = %view.payout_preference_id,
        beneficiary_subject_id = %view.beneficiary_subject_id,
        "payment payout preference created"
    );
    Ok(ApiResponse::ok(view))
}

fn require_step_up_placeholder(
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

fn resolve_payout_scope(
    headers: &HeaderMap,
    query: &ListPayoutPreferenceQuery,
) -> Result<(Option<String>, Option<String>), (StatusCode, Json<ErrorResponse>)> {
    let role = header(headers, "x-role").unwrap_or_default();
    if role == "tenant_admin" {
        let tenant_id = header(headers, "x-tenant-id").ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: "x-tenant-id is required for tenant payout preference read"
                        .to_string(),
                    request_id: header(headers, "x-request-id"),
                }),
            )
        })?;
        if let Some(requested) = &query.beneficiary_subject_id {
            if requested != &tenant_id {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        code: ErrorCode::IamUnauthorized.as_str().to_string(),
                        message: "tenant scope mismatch for payout preference read".to_string(),
                        request_id: header(headers, "x-request-id"),
                    }),
                ));
            }
        }
        let subject_type = query
            .beneficiary_subject_type
            .clone()
            .unwrap_or_else(|| "organization".to_string());
        return Ok((Some(subject_type), Some(tenant_id)));
    }
    Ok((
        query.beneficiary_subject_type.clone(),
        query.beneficiary_subject_id.clone(),
    ))
}

fn enforce_payout_subject_scope(
    headers: &HeaderMap,
    beneficiary_subject_type: &str,
    beneficiary_subject_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if header(headers, "x-role").as_deref() != Some("tenant_admin") {
        return Ok(());
    }
    let tenant_id = header(headers, "x-tenant-id").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "x-tenant-id is required for tenant payout preference manage".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })?;
    if beneficiary_subject_type != "organization" || beneficiary_subject_id != tenant_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "tenant scope mismatch for payout preference manage".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}
