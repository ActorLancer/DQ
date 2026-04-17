use crate::modules::catalog::domain::{
    CreateDataProductRequest, CreateFormatDetectionRequest, CreateProductSkuRequest,
    CreateRawIngestBatchRequest, CreateRawObjectManifestRequest, DataProductView,
    FormatDetectionResultView, PatchDataProductRequest, PatchProductSkuRequest, ProductSkuView,
    RawIngestBatchView, RawObjectManifestView, default_trade_mode_for_sku_type,
    is_standard_sku_type,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::{
    CatalogPermission, is_allowed, is_valid_sku_trade_mode_pair,
};
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{patch, post};
use axum::{Json, Router};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse, new_external_readable_id};
use tokio_postgres::{Client, GenericClient, NoTls};
use tracing::info;

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/products", post(create_product_draft))
        .route("/api/v1/products/{id}", patch(patch_product_draft))
        .route("/api/v1/products/{id}/skus", post(create_product_sku))
        .route("/api/v1/skus/{id}", patch(patch_product_sku))
        .route(
            "/api/v1/assets/{assetId}/raw-ingest-batches",
            post(create_raw_ingest_batch),
        )
        .route(
            "/api/v1/raw-ingest-batches/{id}/manifests",
            post(create_raw_object_manifest),
        )
        .route(
            "/api/v1/raw-object-manifests/{id}/detect-format",
            post(detect_raw_object_format),
        )
}

async fn create_product_draft(
    headers: HeaderMap,
    Json(payload): Json<CreateDataProductRequest>,
) -> Result<Json<ApiResponse<DataProductView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product draft create",
    )?;
    validate_create_product_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_data_product(&tx, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "product",
        &view.product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.product.create",
        product_id = %view.product_id,
        "catalog product draft created"
    );
    Ok(ApiResponse::ok(view))
}

async fn patch_product_draft(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchDataProductRequest>,
) -> Result<Json<ApiResponse<DataProductView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product draft patch",
    )?;
    validate_patch_product_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing = PostgresCatalogRepository::get_data_product(&client, &id)
        .await
        .map_err(map_db_error)?;
    let existing = match existing {
        Some(v) => v,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "product does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            ));
        }
    };
    if existing.status != "draft" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "only draft product can be edited".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::patch_data_product(&tx, &id, &payload)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    code: ErrorCode::TrdStateConflict.as_str().to_string(),
                    message: "product is no longer editable".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    write_audit_event(
        &tx,
        "product",
        &view.product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.product.patch",
        product_id = %view.product_id,
        "catalog product draft patched"
    );
    Ok(ApiResponse::ok(view))
}

async fn create_product_sku(
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
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

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

async fn patch_product_sku(
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
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

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

async fn create_raw_ingest_batch(
    headers: HeaderMap,
    Path(asset_id): Path<String>,
    Json(payload): Json<CreateRawIngestBatchRequest>,
) -> Result<Json<ApiResponse<RawIngestBatchView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::RawIngestWrite,
        "catalog raw ingest batch create",
    )?;
    validate_create_raw_ingest_batch_payload(&asset_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_raw_ingest_batch(
        &tx,
        &asset_id,
        &payload,
        header(&headers, "x-user-id").as_deref(),
    )
    .await
    .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "raw_ingest_batch",
        &view.raw_ingest_batch_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.raw_ingest_batch.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.raw_ingest_batch.create",
        raw_ingest_batch_id = %view.raw_ingest_batch_id,
        asset_id = %asset_id,
        "catalog raw ingest batch created"
    );
    Ok(ApiResponse::ok(view))
}

async fn create_raw_object_manifest(
    headers: HeaderMap,
    Path(raw_ingest_batch_id): Path<String>,
    Json(payload): Json<CreateRawObjectManifestRequest>,
) -> Result<Json<ApiResponse<RawObjectManifestView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::RawIngestWrite,
        "catalog raw object manifest create",
    )?;
    validate_create_raw_object_manifest_payload(&raw_ingest_batch_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing_batch =
        PostgresCatalogRepository::get_raw_ingest_batch(&client, &raw_ingest_batch_id)
            .await
            .map_err(map_db_error)?;
    if existing_batch.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "raw ingest batch does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view =
        PostgresCatalogRepository::create_raw_object_manifest(&tx, &raw_ingest_batch_id, &payload)
            .await
            .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "raw_object_manifest",
        &view.raw_object_manifest_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.raw_object_manifest.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.raw_object_manifest.create",
        raw_object_manifest_id = %view.raw_object_manifest_id,
        raw_ingest_batch_id = %raw_ingest_batch_id,
        "catalog raw object manifest created"
    );
    Ok(ApiResponse::ok(view))
}

async fn detect_raw_object_format(
    headers: HeaderMap,
    Path(raw_object_manifest_id): Path<String>,
    Json(payload): Json<CreateFormatDetectionRequest>,
) -> Result<Json<ApiResponse<FormatDetectionResultView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::RawIngestWrite,
        "catalog format detection create",
    )?;
    validate_detect_format_payload(&raw_object_manifest_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing =
        PostgresCatalogRepository::get_raw_object_manifest(&client, &raw_object_manifest_id)
            .await
            .map_err(map_db_error)?;
    if existing.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "raw object manifest does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_format_detection_result(
        &tx,
        &raw_object_manifest_id,
        &payload,
    )
    .await
    .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "format_detection_result",
        &view.format_detection_result_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.format_detection_result.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.format_detection_result.create",
        format_detection_result_id = %view.format_detection_result_id,
        raw_object_manifest_id = %raw_object_manifest_id,
        "catalog format detection result created"
    );
    Ok(ApiResponse::ok(view))
}

fn validate_create_product_payload(
    payload: &CreateDataProductRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let invalid = payload.asset_id.trim().is_empty()
        || payload.asset_version_id.trim().is_empty()
        || payload.seller_org_id.trim().is_empty()
        || payload.title.trim().is_empty()
        || payload.category.trim().is_empty()
        || payload.product_type.trim().is_empty()
        || payload.delivery_type.trim().is_empty();
    if !invalid {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "required product fields must not be empty".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn validate_patch_product_payload(
    payload: &PatchDataProductRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let has_change = payload.title.is_some()
        || payload.category.is_some()
        || payload.product_type.is_some()
        || payload.description.is_some()
        || payload.price_mode.is_some()
        || payload.price.is_some()
        || payload.currency_code.is_some()
        || payload.delivery_type.is_some()
        || payload.searchable_text.is_some()
        || payload.status.is_some();
    if has_change {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "at least one patch field is required".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn validate_create_sku_payload(
    product_id_from_path: &str,
    payload: &CreateProductSkuRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(product_id_from_body) = payload.product_id.as_deref()
        && product_id_from_body != product_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "product_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let invalid = payload.sku_code.trim().is_empty()
        || payload.sku_type.trim().is_empty()
        || payload.billing_mode.trim().is_empty()
        || payload.acceptance_mode.trim().is_empty()
        || payload.refund_mode.trim().is_empty();
    if invalid {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "required sku fields must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if !is_standard_sku_type(&payload.sku_type) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku_type is not in V1 standard truth set".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let trade_mode = payload
        .trade_mode
        .as_deref()
        .or_else(|| default_trade_mode_for_sku_type(&payload.sku_type));
    let Some(trade_mode) = trade_mode else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "trade_mode is required or derivable".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    };
    if !is_valid_sku_trade_mode_pair(&payload.sku_type, trade_mode) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku_type and trade_mode are incompatible".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

fn validate_patch_sku_payload(
    payload: &PatchProductSkuRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let has_change = payload.sku_code.is_some()
        || payload.sku_type.is_some()
        || payload.unit_name.is_some()
        || payload.billing_mode.is_some()
        || payload.trade_mode.is_some()
        || payload.delivery_object_kind.is_some()
        || payload.subscription_cadence.is_some()
        || payload.share_protocol.is_some()
        || payload.result_form.is_some()
        || payload.template_id.is_some()
        || payload.acceptance_mode.is_some()
        || payload.refund_mode.is_some()
        || payload.status.is_some();
    if has_change {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "at least one sku patch field is required".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn validate_create_raw_ingest_batch_payload(
    asset_id_from_path: &str,
    payload: &CreateRawIngestBatchRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(asset_id_from_body) = payload.asset_id.as_deref()
        && asset_id_from_body != asset_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let invalid =
        payload.owner_org_id.trim().is_empty() || payload.ingest_source_type.trim().is_empty();
    if !invalid {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "required raw ingest batch fields must not be empty".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn validate_create_raw_object_manifest_payload(
    raw_ingest_batch_id_from_path: &str,
    payload: &CreateRawObjectManifestRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(raw_ingest_batch_id_from_body) = payload.raw_ingest_batch_id.as_deref()
        && raw_ingest_batch_id_from_body != raw_ingest_batch_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "raw_ingest_batch_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.object_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "object_name is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.byte_size.is_some_and(|v| v < 0) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "byte_size must be >= 0".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

fn validate_detect_format_payload(
    raw_object_manifest_id_from_path: &str,
    payload: &CreateFormatDetectionRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(raw_object_manifest_id_from_body) = payload.raw_object_manifest_id.as_deref()
        && raw_object_manifest_id_from_body != raw_object_manifest_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "raw_object_manifest_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.detected_object_family.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "detected_object_family is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .classification_confidence
        .is_some_and(|v| !(0.0..=1.0).contains(&v))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "classification_confidence must be within [0, 1]".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

async fn validate_template_compatibility(
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
            "SELECT applicable_sku_types::text[]
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
    let applicable_sku_types: Vec<String> = row.get(0);
    if applicable_sku_types.iter().any(|v| v == sku_type) {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "template is not compatible with sku_type".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

async fn write_audit_event(
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

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

fn require_permission(
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

fn database_dsn() -> Result<String, (StatusCode, Json<ErrorResponse>)> {
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

async fn connect_db(
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

fn map_db_error(err: tokio_postgres::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database operation failed: {err}"),
            request_id: None,
        }),
    )
}
