use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};

use crate::modules::catalog::domain::{ProductDetailView, SellerProfileView};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::CatalogPermission;

use super::super::support::*;

pub(in crate::modules::catalog) async fn get_product_detail(
    headers: HeaderMap,
    Path(product_id): Path<String>,
) -> Result<Json<ApiResponse<ProductDetailView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductRead,
        "catalog product detail read",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

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
    headers: HeaderMap,
    Path(org_id): Path<String>,
) -> Result<Json<ApiResponse<SellerProfileView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::SellerProfileRead,
        "catalog seller profile read",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

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
