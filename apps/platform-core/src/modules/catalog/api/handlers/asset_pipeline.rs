use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

use crate::modules::catalog::domain::{
    AssetFieldDefinitionView, AssetObjectView, AssetProcessingJobView, AssetQualityReportView,
    AssetReleasePolicyView, CreateAssetFieldDefinitionRequest, CreateAssetObjectRequest,
    CreateAssetProcessingJobRequest, CreateAssetQualityReportRequest, CreateExtractionJobRequest,
    CreateFormatDetectionRequest, CreatePreviewArtifactRequest, CreateRawIngestBatchRequest,
    CreateRawObjectManifestRequest, ExtractionJobView, FormatDetectionResultView,
    PatchAssetReleasePolicyRequest, PreviewArtifactView, RawIngestBatchView, RawObjectManifestView,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::CatalogPermission;

use super::super::support::*;
use super::super::validators::*;

pub(in crate::modules::catalog) async fn create_raw_ingest_batch(
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

pub(in crate::modules::catalog) async fn create_raw_object_manifest(
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

pub(in crate::modules::catalog) async fn detect_raw_object_format(
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

pub(in crate::modules::catalog) async fn create_extraction_job(
    headers: HeaderMap,
    Path(raw_object_manifest_id): Path<String>,
    Json(payload): Json<CreateExtractionJobRequest>,
) -> Result<Json<ApiResponse<ExtractionJobView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::RawIngestWrite,
        "catalog extraction job create",
    )?;
    validate_create_extraction_job_payload(&raw_object_manifest_id, &payload, &headers)?;
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
    let view =
        PostgresCatalogRepository::create_extraction_job(&tx, &raw_object_manifest_id, &payload)
            .await
            .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "extraction_job",
        &view.extraction_job_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.extraction_job.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.extraction_job.create",
        extraction_job_id = %view.extraction_job_id,
        raw_object_manifest_id = %raw_object_manifest_id,
        "catalog extraction job created"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn create_preview_artifact(
    headers: HeaderMap,
    Path(asset_version_id): Path<String>,
    Json(payload): Json<CreatePreviewArtifactRequest>,
) -> Result<Json<ApiResponse<PreviewArtifactView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::RawIngestWrite,
        "catalog preview artifact create",
    )?;
    validate_create_preview_artifact_payload(&asset_version_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing_version = PostgresCatalogRepository::get_asset_version(&client, &asset_version_id)
        .await
        .map_err(map_db_error)?;
    if existing_version.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset version does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    if let Some(raw_object_manifest_id) = payload.raw_object_manifest_id.as_deref() {
        let existing_manifest =
            PostgresCatalogRepository::get_raw_object_manifest(&client, raw_object_manifest_id)
                .await
                .map_err(map_db_error)?;
        if existing_manifest.is_none() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "raw object manifest does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            ));
        }
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_preview_artifact(&tx, &asset_version_id, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "preview_artifact",
        &view.preview_artifact_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.preview_artifact.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.preview_artifact.create",
        preview_artifact_id = %view.preview_artifact_id,
        asset_version_id = %asset_version_id,
        "catalog preview artifact created"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn create_asset_object(
    headers: HeaderMap,
    Path(asset_version_id): Path<String>,
    Json(payload): Json<CreateAssetObjectRequest>,
) -> Result<Json<ApiResponse<AssetObjectView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::RawIngestWrite,
        "catalog asset object create",
    )?;
    validate_create_asset_object_payload(&asset_version_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing_version = PostgresCatalogRepository::get_asset_version(&client, &asset_version_id)
        .await
        .map_err(map_db_error)?;
    if existing_version.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset version does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_asset_object(&tx, &asset_version_id, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "asset_object",
        &view.asset_object_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.asset_object.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.asset_object.create",
        asset_object_id = %view.asset_object_id,
        asset_version_id = %asset_version_id,
        "catalog asset object created"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn patch_asset_release_policy(
    headers: HeaderMap,
    Path(asset_id): Path<String>,
    Json(payload): Json<PatchAssetReleasePolicyRequest>,
) -> Result<Json<ApiResponse<AssetReleasePolicyView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog asset release policy patch",
    )?;
    validate_patch_asset_release_policy_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing_asset = PostgresCatalogRepository::get_data_resource(&client, &asset_id)
        .await
        .map_err(map_db_error)?;
    if existing_asset.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::patch_asset_release_policy(&tx, &asset_id, &payload)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "asset versions do not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    write_audit_event(
        &tx,
        "asset",
        &asset_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.asset.release_policy.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.asset.release_policy.patch",
        asset_id = %asset_id,
        applied_version_count = %view.applied_version_count,
        "catalog asset release policy patched"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn create_asset_field_definition(
    headers: HeaderMap,
    Path(asset_version_id): Path<String>,
    Json(payload): Json<CreateAssetFieldDefinitionRequest>,
) -> Result<Json<ApiResponse<AssetFieldDefinitionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog asset field definition create",
    )?;
    validate_create_asset_field_definition_payload(&asset_version_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing_version = PostgresCatalogRepository::get_asset_version(&client, &asset_version_id)
        .await
        .map_err(map_db_error)?;
    if existing_version.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset version does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view =
        PostgresCatalogRepository::create_asset_field_definition(&tx, &asset_version_id, &payload)
            .await
            .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "asset_field_definition",
        &view.field_definition_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.asset_field_definition.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.asset_field_definition.create",
        field_definition_id = %view.field_definition_id,
        asset_version_id = %asset_version_id,
        "catalog asset field definition created"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn create_asset_quality_report(
    headers: HeaderMap,
    Path(asset_version_id): Path<String>,
    Json(payload): Json<CreateAssetQualityReportRequest>,
) -> Result<Json<ApiResponse<AssetQualityReportView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog asset quality report create",
    )?;
    validate_create_asset_quality_report_payload(&asset_version_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing_version = PostgresCatalogRepository::get_asset_version(&client, &asset_version_id)
        .await
        .map_err(map_db_error)?;
    if existing_version.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset version does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view =
        PostgresCatalogRepository::create_asset_quality_report(&tx, &asset_version_id, &payload)
            .await
            .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "asset_quality_report",
        &view.quality_report_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.asset_quality_report.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.asset_quality_report.create",
        quality_report_id = %view.quality_report_id,
        asset_version_id = %asset_version_id,
        "catalog asset quality report created"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn create_asset_processing_job(
    headers: HeaderMap,
    Path(asset_version_id): Path<String>,
    Json(payload): Json<CreateAssetProcessingJobRequest>,
) -> Result<Json<ApiResponse<AssetProcessingJobView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog asset processing job create",
    )?;
    validate_create_asset_processing_job_payload(&asset_version_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing_version = PostgresCatalogRepository::get_asset_version(&client, &asset_version_id)
        .await
        .map_err(map_db_error)?;
    if existing_version.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset version does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    for input_source in &payload.input_sources {
        let input_asset_version = PostgresCatalogRepository::get_asset_version(
            &client,
            &input_source.input_asset_version_id,
        )
        .await
        .map_err(map_db_error)?;
        if input_asset_version.is_none() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "input asset version does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            ));
        }
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view =
        PostgresCatalogRepository::create_asset_processing_job(&tx, &asset_version_id, &payload)
            .await
            .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "asset_processing_job",
        &view.processing_job_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.asset_processing_job.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.asset_processing_job.create",
        processing_job_id = %view.processing_job_id,
        output_asset_version_id = %asset_version_id,
        "catalog asset processing job created"
    );
    Ok(ApiResponse::ok(view))
}
