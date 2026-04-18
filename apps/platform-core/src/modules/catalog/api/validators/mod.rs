use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use kernel::{ErrorCode, ErrorResponse};
use tokio_postgres::GenericClient;

use crate::modules::catalog::domain::{
    BindTemplateRequest, CreateAssetFieldDefinitionRequest, CreateAssetObjectRequest,
    CreateAssetProcessingJobRequest, CreateAssetQualityReportRequest, CreateDataContractRequest,
    CreateDataProductRequest, CreateExtractionJobRequest, CreateFormatDetectionRequest,
    CreatePreviewArtifactRequest, CreateProductSkuRequest, CreateRawIngestBatchRequest,
    CreateRawObjectManifestRequest, PatchAssetReleasePolicyRequest, PatchDataProductRequest,
    PatchProductSkuRequest, PatchUsagePolicyRequest, PutProductMetadataProfileRequest,
    ReviewDecisionRequest, SubmitProductRequest, SuspendProductRequest,
    default_trade_mode_for_sku_type, is_standard_sku_type,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::is_valid_sku_trade_mode_pair;

use super::support::header;

pub(in crate::modules::catalog::api) fn validate_create_product_payload(
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

pub(in crate::modules::catalog::api) fn validate_patch_product_payload(
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
        || payload.subtitle.is_some()
        || payload.industry.is_some()
        || payload.use_cases.is_some()
        || payload.data_classification.is_some()
        || payload.quality_score.is_some()
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

pub(in crate::modules::catalog::api) fn validate_put_product_metadata_profile_payload(
    product_id_from_path: &str,
    payload: &PutProductMetadataProfileRequest,
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
    if payload.metadata_version_no.is_some_and(|v| v != 1) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "V1 only supports metadata_version_no=1".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .status
        .as_deref()
        .is_some_and(|v| v.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "status must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_submit_product_payload(
    payload: &SubmitProductRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if payload
        .submission_note
        .as_deref()
        .is_some_and(|note| note.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "submission_note must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_review_decision_payload(
    payload: &ReviewDecisionRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if !matches!(payload.action_name.as_str(), "approve" | "reject") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "action_name must be approve or reject".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .action_reason
        .as_deref()
        .is_some_and(|reason| reason.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "action_reason must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_suspend_payload(
    payload: &SuspendProductRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if !matches!(payload.suspend_mode.as_str(), "delist" | "freeze") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "suspend_mode must be delist or freeze".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .reason
        .as_deref()
        .is_some_and(|reason| reason.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "reason must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_create_sku_payload(
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

pub(in crate::modules::catalog::api) fn validate_patch_sku_payload(
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

pub(in crate::modules::catalog::api) fn validate_bind_template_payload(
    payload: &BindTemplateRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if payload.template_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "template_id is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .binding_type
        .as_deref()
        .is_some_and(|v| !matches!(v, "contract" | "acceptance" | "refund" | "license"))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "binding_type is invalid".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_patch_usage_policy_payload(
    payload: &PatchUsagePolicyRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let has_change = payload.policy_name.is_some()
        || payload.subject_constraints.is_some()
        || payload.usage_constraints.is_some()
        || payload.time_constraints.is_some()
        || payload.region_constraints.is_some()
        || payload.output_constraints.is_some()
        || payload.exportable.is_some()
        || payload.status.is_some();
    if !has_change {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "at least one policy patch field is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .status
        .as_deref()
        .is_some_and(|status| !matches!(status, "draft" | "active" | "disabled" | "archived"))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "policy status is invalid".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_create_raw_ingest_batch_payload(
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

pub(in crate::modules::catalog::api) fn validate_create_raw_object_manifest_payload(
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

pub(in crate::modules::catalog::api) fn validate_detect_format_payload(
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

pub(in crate::modules::catalog::api) fn validate_create_extraction_job_payload(
    raw_object_manifest_id_from_path: &str,
    payload: &CreateExtractionJobRequest,
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
    if payload.job_type.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "job_type is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_create_preview_artifact_payload(
    asset_version_id_from_path: &str,
    payload: &CreatePreviewArtifactRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(asset_version_id_from_body) = payload.asset_version_id.as_deref()
        && asset_version_id_from_body != asset_version_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset_version_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.preview_type.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "preview_type is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_create_asset_field_definition_payload(
    asset_version_id_from_path: &str,
    payload: &CreateAssetFieldDefinitionRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(asset_version_id_from_body) = payload.asset_version_id.as_deref()
        && asset_version_id_from_body != asset_version_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset_version_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.field_name.trim().is_empty()
        || payload.field_path.trim().is_empty()
        || payload.field_type.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "field_name, field_path and field_type are required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_create_asset_object_payload(
    asset_version_id_from_path: &str,
    payload: &CreateAssetObjectRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(asset_version_id_from_body) = payload.asset_version_id.as_deref()
        && asset_version_id_from_body != asset_version_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset_version_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.object_kind.trim().is_empty()
        || payload.object_name.trim().is_empty()
        || payload.object_uri.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "object_kind, object_name and object_uri are required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if !matches!(
        payload.object_kind.as_str(),
        "raw_object" | "preview_object" | "delivery_object" | "report_object" | "result_object"
    ) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message:
                    "object_kind must be one of raw_object|preview_object|delivery_object|report_object|result_object"
                        .to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_patch_asset_release_policy_payload(
    payload: &PatchAssetReleasePolicyRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let has_change = payload.release_mode.is_some()
        || payload.is_revision_subscribable.is_some()
        || payload.update_frequency.is_some()
        || payload.release_notes_json.is_object();
    if !has_change {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "at least one release policy field is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .release_mode
        .as_deref()
        .is_some_and(|value| !matches!(value, "snapshot" | "revision"))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "release_mode must be snapshot or revision".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .update_frequency
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "update_frequency must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_create_asset_quality_report_payload(
    asset_version_id_from_path: &str,
    payload: &CreateAssetQualityReportRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(asset_version_id_from_body) = payload.asset_version_id.as_deref()
        && asset_version_id_from_body != asset_version_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset_version_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.report_no.is_some_and(|v| v <= 0) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "report_no must be > 0".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    for rate in [
        payload.missing_rate,
        payload.duplicate_rate,
        payload.anomaly_rate,
    ] {
        if rate.is_some_and(|v| !(0.0..=1.0).contains(&v)) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "quality rate fields must be within [0, 1]".to_string(),
                    request_id: header(headers, "x-request-id"),
                }),
            ));
        }
    }
    if payload
        .status
        .as_deref()
        .is_some_and(|v| v.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "status must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_create_asset_processing_job_payload(
    asset_version_id_from_path: &str,
    payload: &CreateAssetProcessingJobRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(asset_version_id_from_body) = payload.asset_version_id.as_deref()
        && asset_version_id_from_body != asset_version_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "asset_version_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.processing_mode.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "processing_mode is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.input_sources.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "at least one input source is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    for source in &payload.input_sources {
        if source.input_asset_version_id.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "input_asset_version_id must not be empty".to_string(),
                    request_id: header(headers, "x-request-id"),
                }),
            ));
        }
        if source
            .input_role
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "input_role must not be empty".to_string(),
                    request_id: header(headers, "x-request-id"),
                }),
            ));
        }
    }
    if payload
        .status
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "status must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

pub(in crate::modules::catalog::api) fn validate_create_data_contract_payload(
    sku_id_from_path: &str,
    payload: &CreateDataContractRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(sku_id_from_body) = payload.sku_id.as_deref()
        && sku_id_from_body != sku_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.contract_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "contract_name is required".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload.version_no.is_some_and(|version| version <= 0) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "version_no must be > 0".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if payload
        .status
        .as_deref()
        .is_some_and(|status| status.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "status must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}
