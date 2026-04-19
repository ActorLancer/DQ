use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};

use crate::AppState;
use crate::modules::catalog::domain::{
    BindTemplateRequest, PatchUsagePolicyRequest, TemplateBindingView, UsagePolicyView,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::CatalogPermission;

use super::super::support::*;
use super::super::validators::*;

pub(in crate::modules::catalog) async fn bind_product_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(product_id): Path<String>,
    Json(payload): Json<BindTemplateRequest>,
) -> Result<Json<ApiResponse<TemplateBindingView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::TemplateBind,
        "catalog product template bind",
    )?;
    validate_bind_template_payload(&payload, &headers)?;
    let client = state_client(&state)?;

    let product = PostgresCatalogRepository::get_data_product(&client, &product_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "product does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    enforce_product_scope(
        &headers,
        &product.seller_org_id,
        "catalog product template bind",
    )?;
    let skus = PostgresCatalogRepository::list_product_skus(&client, &product_id)
        .await
        .map_err(map_db_error)?;
    for sku in &skus {
        validate_template_compatibility(
            &client,
            Some(payload.template_id.as_str()),
            &sku.sku_type,
            &headers,
        )
        .await?;
    }
    let tx = client.transaction().await.map_err(map_db_error)?;
    PostgresCatalogRepository::set_product_default_template(&tx, &product_id, &payload.template_id)
        .await
        .map_err(map_db_error)?;
    for sku in &skus {
        PostgresCatalogRepository::bind_template_to_sku(&tx, &sku.sku_id, &payload)
            .await
            .map_err(map_db_error)?;
    }
    write_audit_event(
        &tx,
        "product",
        &product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "template.product.bind",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    let view = PostgresCatalogRepository::build_template_binding_view(
        &tx,
        "product",
        &product_id,
        &payload,
        skus.len() as i32,
    )
    .await
    .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn bind_sku_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sku_id): Path<String>,
    Json(payload): Json<BindTemplateRequest>,
) -> Result<Json<ApiResponse<TemplateBindingView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::TemplateBind,
        "catalog sku template bind",
    )?;
    validate_bind_template_payload(&payload, &headers)?;
    let client = state_client(&state)?;

    let sku = PostgresCatalogRepository::get_product_sku(&client, &sku_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "sku does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    let product = PostgresCatalogRepository::get_data_product(&client, &sku.product_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "product does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    enforce_product_scope(
        &headers,
        &product.seller_org_id,
        "catalog sku template bind",
    )?;
    validate_template_compatibility(
        &client,
        Some(payload.template_id.as_str()),
        &sku.sku_type,
        &headers,
    )
    .await?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    PostgresCatalogRepository::bind_template_to_sku(&tx, &sku_id, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "sku",
        &sku_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "template.sku.bind",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    let view =
        PostgresCatalogRepository::build_template_binding_view(&tx, "sku", &sku_id, &payload, 1)
            .await
            .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn patch_usage_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(policy_id): Path<String>,
    Json(payload): Json<PatchUsagePolicyRequest>,
) -> Result<Json<ApiResponse<UsagePolicyView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::PolicyUpdate,
        "catalog usage policy patch",
    )?;
    validate_patch_usage_policy_payload(&payload, &headers)?;
    let client = state_client(&state)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::patch_usage_policy(&tx, &policy_id, &payload)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "policy does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    write_audit_event(
        &tx,
        "usage_policy",
        &policy_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "template.policy.update",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}
