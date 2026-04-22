use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{EntityId, ErrorCode, ErrorResponse};
use serde_json::json;

use crate::AppState;
use crate::modules::audit::domain::{
    AuditTracePageView, AuditTraceQuery, OrderAuditQuery, OrderAuditView,
};
use crate::modules::audit::repo::{self, AccessAuditInsert, OrderAuditScope, SystemLogInsert};

pub(in crate::modules::audit) async fn get_order_audit_traces(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    Query(query): Query<OrderAuditQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OrderAuditView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&order_id, "order_id", &request_id)?;
    require_permission(
        &headers,
        AuditPermission::TraceRead,
        "audit order trace read",
    )?;

    let client = state_client(&state)?;
    let scope = repo::load_order_audit_scope(&client, &order_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("audit order trace target not found: {order_id}"),
            )
        })?;
    ensure_order_scope(&headers, &scope, &request_id, "audit order trace read")?;

    let pagination = query.pagination();
    let trace_query = AuditTraceQuery {
        order_id: Some(order_id.clone()),
        page: query.page,
        page_size: query.page_size,
        ..Default::default()
    };
    let trace_page = repo::search_audit_traces(
        &client,
        &trace_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_lookup_side_effects(
        &client,
        &headers,
        "order",
        Some(order_id.clone()),
        "GET /api/v1/audit/orders/{id}",
        json!({
            "order_id": order_id,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": trace_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OrderAuditView {
        order_id: scope.order_id,
        buyer_org_id: scope.buyer_org_id,
        seller_org_id: scope.seller_org_id,
        status: scope.status,
        payment_status: scope.payment_status,
        total: trace_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        traces: trace_page.items,
    }))
}

pub(in crate::modules::audit) async fn get_audit_traces(
    State(state): State<AppState>,
    Query(query): Query<AuditTraceQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AuditTracePageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_optional_uuid(query.order_id.as_deref(), "order_id", &request_id)?;
    validate_optional_uuid(query.ref_id.as_deref(), "ref_id", &request_id)?;
    require_permission(&headers, AuditPermission::TraceRead, "audit trace read")?;

    let client = state_client(&state)?;
    ensure_trace_query_scope(&client, &headers, &query, &request_id).await?;

    let pagination = query.pagination();
    let trace_page = repo::search_audit_traces(
        &client,
        &query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_lookup_side_effects(
        &client,
        &headers,
        "audit_trace_query",
        query
            .effective_order_id()
            .map(ToString::to_string)
            .or_else(|| query.ref_id.clone()),
        "GET /api/v1/audit/traces",
        json!({
            "order_id": query.order_id,
            "ref_type": query.ref_type,
            "ref_id": query.ref_id,
            "request_id": query.request_id,
            "trace_id": query.trace_id,
            "action_name": query.action_name,
            "result_code": query.result_code,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": trace_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(AuditTracePageView {
        total: trace_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: trace_page.items,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuditPermission {
    TraceRead,
}

fn is_allowed(role: &str, permission: AuditPermission) -> bool {
    match permission {
        AuditPermission::TraceRead => matches!(
            role,
            "tenant_admin"
                | "tenant_audit_readonly"
                | "platform_admin"
                | "platform_audit_security"
                | "platform_reviewer"
                | "platform_risk_settlement"
                | "audit_admin"
                | "subject_reviewer"
                | "product_reviewer"
                | "compliance_reviewer"
                | "risk_operator"
                | "data_custody_admin"
                | "regulator_readonly"
                | "regulator_observer"
        ),
    }
}

fn is_tenant_scoped_role(role: &str) -> bool {
    matches!(role, "tenant_admin" | "tenant_audit_readonly")
}

fn require_permission(
    headers: &HeaderMap,
    permission: AuditPermission,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = current_role(headers);
    if is_allowed(&role, permission) {
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

fn require_request_id(headers: &HeaderMap) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    header(headers, "x-request-id").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::AudEvidenceInvalid.as_str().to_string(),
                message: "x-request-id is required for audit access".to_string(),
                request_id: None,
            }),
        )
    })
}

async fn ensure_trace_query_scope(
    client: &db::Client,
    headers: &HeaderMap,
    query: &AuditTraceQuery,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = current_role(headers);
    if !is_tenant_scoped_role(&role) {
        return Ok(());
    }

    let Some(order_id) = query.effective_order_id() else {
        return Err(bad_request(
            request_id,
            "tenant-scoped audit trace queries require order_id or ref_type=order + ref_id",
        ));
    };

    let scope = repo::load_order_audit_scope(client, order_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                request_id,
                format!("tenant-scoped audit trace target not found: {order_id}"),
            )
        })?;
    ensure_order_scope(headers, &scope, request_id, "audit trace read")
}

fn ensure_order_scope(
    headers: &HeaderMap,
    scope: &OrderAuditScope,
    request_id: &str,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = current_role(headers);
    if !is_tenant_scoped_role(&role) {
        return Ok(());
    }

    let tenant_id = header(headers, "x-tenant-id")
        .ok_or_else(|| bad_request(request_id, "x-tenant-id is required for tenant audit scope"))?;
    if tenant_id == scope.buyer_org_id || tenant_id == scope.seller_org_id {
        return Ok(());
    }

    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("{action} is forbidden outside tenant order scope"),
            request_id: Some(request_id.to_string()),
        }),
    ))
}

async fn record_lookup_side_effects(
    client: &db::Client,
    headers: &HeaderMap,
    target_type: &str,
    target_id: Option<String>,
    endpoint: &str,
    filters: serde_json::Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(headers, "x-request-id");
    let trace_id = header(headers, "x-trace-id");
    let filters_for_access = filters.clone();
    let role = current_role(headers);
    let access_audit_id = repo::record_access_audit(
        client,
        &AccessAuditInsert {
            accessor_user_id: parse_uuid_header(headers, "x-user-id"),
            accessor_role_key: Some(role.clone()),
            access_mode: "masked".to_string(),
            target_type: target_type.to_string(),
            target_id,
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: parse_uuid_header(headers, "x-step-up-challenge-id"),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
            metadata: json!({
                "endpoint": endpoint,
                "filters": filters_for_access,
                "step_up_token_present": header(headers, "x-step-up-token").is_some(),
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id,
            trace_id,
            message_text: format!("audit lookup executed: {endpoint}"),
            structured_payload: json!({
                "module": "audit",
                "endpoint": endpoint,
                "access_audit_id": access_audit_id,
                "role": role,
                "filters": filters,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

fn state_client(state: &AppState) -> Result<db::Client, (StatusCode, Json<ErrorResponse>)> {
    state.db.client().map_err(map_db_error)
}

fn map_db_error(err: db::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("audit persistence failed: {err}"),
            request_id: None,
        }),
    )
}

fn bad_request(request_id: &str, message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::AudEvidenceInvalid.as_str().to_string(),
            message: message.into(),
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn not_found(request_id: &str, message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::AudEvidenceInvalid.as_str().to_string(),
            message: message.into(),
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn validate_uuid(
    raw: &str,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    validate_optional_uuid(Some(raw), field_name, request_id)
}

fn validate_optional_uuid(
    raw: Option<&str>,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(raw) = raw {
        EntityId::parse(raw).map_err(|_| {
            bad_request(
                request_id,
                format!("{field_name} must be a valid uuid: {raw}"),
            )
        })?;
    }
    Ok(())
}

fn current_role(headers: &HeaderMap) -> String {
    header(headers, "x-role").unwrap_or_else(|| "unknown".to_string())
}

fn parse_uuid_header(headers: &HeaderMap, key: &str) -> Option<String> {
    header(headers, key).and_then(|value| {
        if EntityId::parse(&value).is_ok() {
            Some(value)
        } else {
            None
        }
    })
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}
