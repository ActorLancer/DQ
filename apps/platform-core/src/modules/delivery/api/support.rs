use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use db::Error;
use kernel::{ErrorCode, ErrorResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryPermission {
    CommitFileDelivery,
    EnableApiDelivery,
    ReadApiUsageLog,
    ManageQuerySurface,
    ManageQueryTemplate,
    EnableTemplateQuery,
    UseTemplateQuery,
    IssueDownloadTicket,
    DownloadFile,
    ManageRevisionSubscription,
    ReadRevisionSubscription,
    ManageShareGrant,
    ReadShareGrant,
}

pub fn is_allowed(role: &str, permission: DeliveryPermission) -> bool {
    match permission {
        DeliveryPermission::CommitFileDelivery => matches!(
            role,
            "seller_operator" | "tenant_admin" | "platform_admin" | "platform_risk_settlement"
        ),
        DeliveryPermission::EnableApiDelivery => matches!(
            role,
            "seller_operator"
                | "tenant_developer"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        DeliveryPermission::ReadApiUsageLog => matches!(
            role,
            "buyer_operator"
                | "procurement_manager"
                | "tenant_developer"
                | "tenant_audit_readonly"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        DeliveryPermission::ManageQuerySurface => matches!(
            role,
            "seller_operator" | "tenant_admin" | "platform_admin" | "platform_risk_settlement"
        ),
        DeliveryPermission::ManageQueryTemplate => matches!(
            role,
            "seller_operator" | "tenant_admin" | "platform_admin" | "platform_risk_settlement"
        ),
        DeliveryPermission::EnableTemplateQuery => matches!(
            role,
            "seller_operator"
                | "tenant_developer"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        DeliveryPermission::UseTemplateQuery => matches!(
            role,
            "buyer_operator"
                | "tenant_developer"
                | "business_analyst"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        DeliveryPermission::IssueDownloadTicket | DeliveryPermission::DownloadFile => matches!(
            role,
            "buyer_operator" | "tenant_admin" | "platform_admin" | "platform_risk_settlement"
        ),
        DeliveryPermission::ManageRevisionSubscription => matches!(
            role,
            "seller_operator"
                | "seller_storage_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        DeliveryPermission::ReadRevisionSubscription => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "seller_storage_operator"
                | "procurement_manager"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        DeliveryPermission::ManageShareGrant => matches!(
            role,
            "seller_operator"
                | "seller_storage_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        DeliveryPermission::ReadShareGrant => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "seller_storage_operator"
                | "tenant_audit_readonly"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
    }
}

pub fn require_permission(
    headers: &HeaderMap,
    permission: DeliveryPermission,
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

pub fn map_db_connect(err: Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database connection failed: {err}"),
            request_id: None,
        }),
    )
}

pub fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}
