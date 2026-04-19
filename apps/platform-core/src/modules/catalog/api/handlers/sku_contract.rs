use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

use crate::AppState;
use crate::modules::catalog::domain::{
    CreateDataContractRequest, CreateProductSkuRequest, DataContractView, PatchProductSkuRequest,
    ProductSkuView, default_trade_mode_for_sku_type, is_standard_sku_type,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::{CatalogPermission, is_valid_sku_trade_mode_pair};

use super::super::support::*;
use super::super::validators::*;

pub(in crate::modules::catalog) async fn create_product_sku(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(product_id): Path<String>,
    Json(payload): Json<CreateProductSkuRequest>,
) -> Result<Json<ApiResponse<ProductSkuView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product sku create",
    )?;
    validate_create_sku_payload(&product_id, &payload, &headers)?;
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
    if product.status != "draft" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "only draft product can add sku".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let trade_mode = payload
        .trade_mode
        .clone()
        .or_else(|| default_trade_mode_for_sku_type(&payload.sku_type).map(str::to_string))
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "cannot derive trade_mode from sku_type".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;

    validate_template_compatibility(
        &client,
        payload.template_id.as_deref(),
        &payload.sku_type,
        &headers,
    )
    .await?;

    let mut normalized_payload = payload.clone();
    normalized_payload.trade_mode = Some(trade_mode);

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_product_sku(&tx, &product_id, &normalized_payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "sku",
        &view.sku_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.sku.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.sku.create",
        sku_id = %view.sku_id,
        product_id = %view.product_id,
        "catalog sku created"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn patch_product_sku(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchProductSkuRequest>,
) -> Result<Json<ApiResponse<ProductSkuView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product sku patch",
    )?;
    validate_patch_sku_payload(&payload, &headers)?;
    let client = state_client(&state)?;

    let existing = PostgresCatalogRepository::get_product_sku(&client, &id)
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
    if existing.status != "draft" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "only draft sku can be edited".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let next_sku_type = payload
        .sku_type
        .as_deref()
        .unwrap_or(existing.sku_type.as_str());
    let next_trade_mode = payload
        .trade_mode
        .as_deref()
        .unwrap_or(existing.trade_mode.as_str());

    if !is_standard_sku_type(next_sku_type)
        || !is_valid_sku_trade_mode_pair(next_sku_type, next_trade_mode)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku_type and trade_mode are incompatible".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    validate_template_compatibility(
        &client,
        payload.template_id.as_deref(),
        next_sku_type,
        &headers,
    )
    .await?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::patch_product_sku(&tx, &id, &payload)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    code: ErrorCode::TrdStateConflict.as_str().to_string(),
                    message: "sku is no longer editable".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    write_audit_event(
        &tx,
        "sku",
        &view.sku_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.sku.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.sku.patch",
        sku_id = %view.sku_id,
        "catalog sku patched"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn create_data_contract(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sku_id): Path<String>,
    Json(payload): Json<CreateDataContractRequest>,
) -> Result<Json<ApiResponse<DataContractView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog data contract create",
    )?;
    validate_create_data_contract_payload(&sku_id, &payload, &headers)?;
    let client = state_client(&state)?;

    let existing_sku = PostgresCatalogRepository::get_product_sku(&client, &sku_id)
        .await
        .map_err(map_db_error)?;
    if existing_sku.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_data_contract(&tx, &sku_id, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "data_contract",
        &view.data_contract_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.data_contract.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.data_contract.create",
        data_contract_id = %view.data_contract_id,
        sku_id = %sku_id,
        "catalog data contract created"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn get_data_contract(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sku_id, contract_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<DataContractView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog data contract read",
    )?;
    let client = state_client(&state)?;

    let existing_sku = PostgresCatalogRepository::get_product_sku(&client, &sku_id)
        .await
        .map_err(map_db_error)?;
    if existing_sku.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let view = PostgresCatalogRepository::get_data_contract(&client, &sku_id, &contract_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "data contract does not exist for sku".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    info!(
        action = "catalog.data_contract.read",
        data_contract_id = %view.data_contract_id,
        sku_id = %sku_id,
        "catalog data contract read"
    );
    Ok(ApiResponse::ok(view))
}
