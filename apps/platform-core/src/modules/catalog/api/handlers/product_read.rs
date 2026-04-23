use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};

use crate::AppState;
use crate::modules::catalog::domain::{
    ListProductsQuery, ProductDetailView, ProductListView, SellerProfileView,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::{CatalogPermission, is_valid_listing_status};

use super::super::support::*;

pub(in crate::modules::catalog) async fn list_products(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<ApiResponse<ProductListView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductList,
        "catalog product list",
    )?;

    let mut seller_org_id = normalize_optional(query.seller_org_id);
    let status = normalize_optional(query.status);
    let q = normalize_optional(query.q);
    if let Some(status) = status.as_deref()
        && !is_valid_listing_status(status)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "status filter is invalid".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let role = header(&headers, "x-role").unwrap_or_default();
    if !role.starts_with("platform_") {
        let tenant_id = header(&headers, "x-tenant-id").ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: "catalog product list requires x-tenant-id".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
        if let Some(seller_org_id) = seller_org_id.as_deref() {
            if seller_org_id != tenant_id {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        code: ErrorCode::IamUnauthorized.as_str().to_string(),
                        message: "catalog product list is forbidden for tenant scope".to_string(),
                        request_id: header(&headers, "x-request-id"),
                    }),
                ));
            }
        } else {
            seller_org_id = Some(tenant_id);
        }
    }

    let page = query.page.unwrap_or(1).clamp(1, 500);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 50);
    let client = state_client(&state)?;
    let (items, total, status_counts) = PostgresCatalogRepository::list_data_products(
        &client,
        seller_org_id.as_deref(),
        status.as_deref(),
        q.as_deref(),
        page,
        page_size,
    )
    .await
    .map_err(map_db_error)?;

    write_audit_event(
        &client,
        "product_list",
        seller_org_id
            .as_deref()
            .unwrap_or("00000000-0000-0000-0000-000000000000"),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product.list",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ProductListView {
        items,
        total,
        page,
        page_size,
        status_counts,
    }))
}

pub(in crate::modules::catalog) async fn get_product_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(product_id): Path<String>,
) -> Result<Json<ApiResponse<ProductDetailView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductRead,
        "catalog product detail read",
    )?;
    let client = state_client(&state)?;

    let mut detail = PostgresCatalogRepository::get_product_detail(&client, &product_id)
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
    if detail.status != "listed" {
        enforce_product_scope(
            &headers,
            &detail.seller_org_id,
            "catalog product detail read",
        )?;
    }
    detail.skus = PostgresCatalogRepository::list_product_skus(&client, &product_id)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &client,
        "product",
        &product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(detail))
}

pub(in crate::modules::catalog) async fn get_seller_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
) -> Result<Json<ApiResponse<SellerProfileView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::SellerProfileRead,
        "catalog seller profile read",
    )?;
    let client = state_client(&state)?;

    let view = PostgresCatalogRepository::get_seller_profile(&client, &org_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "seller does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    write_audit_event(
        &client,
        "seller",
        &org_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.seller.profile.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}
