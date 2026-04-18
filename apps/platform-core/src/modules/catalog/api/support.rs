use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use kernel::{ErrorCode, ErrorResponse, new_external_readable_id};
use tokio_postgres::{Client, GenericClient, NoTls};

use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::{CatalogPermission, is_allowed};

pub(in crate::modules::catalog::api) async fn validate_template_compatibility(
    client: &impl GenericClient,
    template_id: Option<&str>,
    sku_type: &str,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(template_id) = template_id else {
        return Ok(());
    };
    let row = client
        .query_opt(
            "SELECT template_name, applicable_sku_types::text[]
             FROM contract.template_definition
             WHERE template_id = $1::text::uuid
               AND status = 'active'",
            &[&template_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "template does not exist or is inactive".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    };
    let template_name: String = row.get(0);
    let applicable_sku_types: Vec<String> = row.get(1);
    if !applicable_sku_types.iter().any(|v| v == sku_type) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "template is not compatible with sku_type".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let template_name_upper = template_name.to_uppercase();
    let uses_legacy_api_name = matches!(
        template_name_upper.as_str(),
        "CONTRACT_API_V1" | "LICENSE_API_USE_V1" | "ACCEPT_API_V1" | "REFUND_API_V1"
    );
    if uses_legacy_api_name {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "legacy generic API templates are not allowed".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if sku_type == "API_SUB" && !template_name_upper.contains("API_SUB") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "API_SUB can only bind API_SUB template family".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if sku_type == "API_PPU" && !template_name_upper.contains("API_PPU") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "API_PPU can only bind API_PPU template family".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) async fn ensure_product_submit_ready(
    client: &impl GenericClient,
    product_id: &str,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let has_profile = PostgresCatalogRepository::product_has_metadata_profile(client, product_id)
        .await
        .map_err(map_db_error)?;
    if !has_profile {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "product metadata profile is required before submit".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let has_skus = PostgresCatalogRepository::product_has_skus(client, product_id)
        .await
        .map_err(map_db_error)?;
    if !has_skus {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "at least one sku is required before submit".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let templates_complete =
        PostgresCatalogRepository::product_all_skus_have_template(client, product_id)
            .await
            .map_err(map_db_error)?;
    if !templates_complete {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "all skus must bind template before submit".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let risk_blocked = PostgresCatalogRepository::product_is_risk_blocked(client, product_id)
        .await
        .map_err(map_db_error)?;
    if risk_blocked {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "product is blocked by risk policy".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn decision_to_review_status(
    action_name: &str,
) -> &'static str {
    if action_name == "approve" {
        "approved"
    } else {
        "rejected"
    }
}

pub(in crate::modules::catalog::api) fn enforce_product_scope(
    headers: &HeaderMap,
    seller_org_id: &str,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = header(headers, "x-role").unwrap_or_default();
    if role.starts_with("platform_") {
        return Ok(());
    }
    let tenant_id = header(headers, "x-tenant-id").ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: format!("{action} requires x-tenant-id"),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })?;
    if tenant_id == seller_org_id {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("{action} is forbidden for tenant scope"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

pub(in crate::modules::catalog::api) fn enforce_subject_scope(
    headers: &HeaderMap,
    subject_id: &str,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = header(headers, "x-role").unwrap_or_default();
    if role.starts_with("platform_") {
        return Ok(());
    }
    let tenant_id = header(headers, "x-tenant-id").ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: format!("{action} requires x-tenant-id"),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })?;
    if tenant_id == subject_id {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("{action} is forbidden for tenant scope"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

pub(in crate::modules::catalog::api) async fn write_outbox_event(
    client: &(impl GenericClient + Sync),
    headers: &HeaderMap,
    aggregate_type: &str,
    aggregate_id: &str,
    event_type: &str,
    target_topic: &str,
    payload: &serde_json::Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(headers, "x-request-id");
    let trace_id = header(headers, "x-trace-id");
    let idempotency_key = header(headers, "x-idempotency-key");
    client
        .query_one(
            "INSERT INTO ops.outbox_event (
               aggregate_type, aggregate_id, event_type, payload,
               request_id, trace_id, idempotency_key, target_topic, status
             ) VALUES (
               $1, $2::text::uuid, $3, $4::jsonb,
               $5, $6, $7, $8, 'pending'
             )
             RETURNING outbox_event_id::text",
            &[
                &aggregate_type,
                &aggregate_id,
                &event_type,
                payload,
                &request_id,
                &trace_id,
                &idempotency_key,
                &target_topic,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

pub(in crate::modules::catalog::api) async fn write_audit_event(
    client: &(impl GenericClient + Sync),
    ref_type: &str,
    ref_id: &str,
    actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    client
        .query_one(
            "INSERT INTO audit.audit_event (
               domain_name, ref_type, ref_id, actor_type, actor_id, action_name, result_code,
               request_id, trace_id, metadata
             ) VALUES (
               'catalog', $1, $2::text::uuid, 'role', NULL, $3, $4, $5, $6, $7::jsonb
             )
             RETURNING audit_id::text",
            &[
                &ref_type,
                &ref_id,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &serde_json::json!({
                    "actor_role": actor_role,
                    "event_id": new_external_readable_id("cat"),
                }),
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

pub(in crate::modules::catalog::api) fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

pub(in crate::modules::catalog::api) fn require_permission(
    headers: &HeaderMap,
    permission: CatalogPermission,
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

pub(in crate::modules::catalog::api) fn require_any_permission(
    headers: &HeaderMap,
    permissions: &[CatalogPermission],
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = headers
        .get("x-role")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if permissions
        .iter()
        .any(|permission| is_allowed(role, *permission))
    {
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

pub(in crate::modules::catalog::api) fn database_dsn()
-> Result<String, (StatusCode, Json<ErrorResponse>)> {
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

pub(in crate::modules::catalog::api) async fn connect_db(
    dsn: &str,
) -> Result<
    (
        Client,
        tokio_postgres::Connection<tokio_postgres::Socket, tokio_postgres::tls::NoTlsStream>,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    tokio_postgres::connect(dsn, NoTls)
        .await
        .map_err(map_db_error)
}

pub(in crate::modules::catalog::api) fn map_db_error(
    err: tokio_postgres::Error,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database operation failed: {err}"),
            request_id: None,
        }),
    )
}
