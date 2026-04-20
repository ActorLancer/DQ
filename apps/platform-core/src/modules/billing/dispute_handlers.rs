use crate::AppState;
use crate::modules::billing::handlers::{
    header, map_db_connect, require_permission, require_step_up_placeholder,
};
use crate::modules::billing::models::{
    CreateDisputeCaseRequest, DisputeCaseView, DisputeEvidenceView, DisputeResolutionView,
    ResolveDisputeCaseRequest, UploadDisputeEvidenceRequest,
};
use crate::modules::billing::repo::dispute_repository::{
    create_dispute_case as create_dispute_case_repo,
    resolve_dispute_case as resolve_dispute_case_repo,
    upload_dispute_evidence as upload_dispute_evidence_repo,
};
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

pub async fn create_dispute_case(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDisputeCaseRequest>,
) -> Result<Json<ApiResponse<DisputeCaseView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::DisputeCaseCreate,
        "dispute case create",
    )?;
    let request_id = header(&headers, "x-request-id");
    let role = header(&headers, "x-role").unwrap_or_default();
    let tenant_scope_id = buyer_tenant_scope_id(&headers, "dispute case create")?;
    let client = state.db.client().map_err(map_db_connect)?;
    let dispute_case = create_dispute_case_repo(
        &client,
        &payload,
        tenant_scope_id.as_str(),
        header(&headers, "x-user-id").as_deref(),
        role.as_str(),
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "dispute.case.create",
        case_id = %dispute_case.case_id,
        order_id = %dispute_case.order_id,
        reason_code = %dispute_case.reason_code,
        "dispute case created"
    );
    Ok(ApiResponse::ok(dispute_case))
}

pub async fn upload_dispute_evidence(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(case_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<DisputeEvidenceView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::DisputeEvidenceUpload,
        "dispute evidence upload",
    )?;
    let request_id = header(&headers, "x-request-id");
    let role = header(&headers, "x-role").unwrap_or_default();
    let tenant_scope_id = buyer_tenant_scope_id(&headers, "dispute evidence upload")?;

    let mut object_type: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut metadata = serde_json::Value::Object(Default::default());

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| multipart_error(&err, request_id.as_deref()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "object_type" => {
                object_type =
                    Some(non_empty(field.text().await.map_err(|err| {
                        multipart_error(&err, request_id.as_deref())
                    })?));
            }
            "metadata_json" => {
                let raw = field
                    .text()
                    .await
                    .map_err(|err| multipart_error(&err, request_id.as_deref()))?;
                if !raw.trim().is_empty() {
                    metadata = serde_json::from_str(&raw).map_err(|err| {
                        bad_request(
                            &format!("metadata_json must be a valid JSON object: {err}"),
                            request_id.as_deref(),
                        )
                    })?;
                    if !metadata.is_object() {
                        return Err(bad_request(
                            "metadata_json must be a JSON object",
                            request_id.as_deref(),
                        ));
                    }
                }
            }
            "file" => {
                file_name = field.file_name().map(str::to_string);
                content_type = field.content_type().map(str::to_string);
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|err| multipart_error(&err, request_id.as_deref()))?
                        .to_vec(),
                );
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let payload = UploadDisputeEvidenceRequest {
        object_type: required_field("object_type", object_type, request_id.as_deref())?,
        file_name: required_field("file", file_name, request_id.as_deref())?,
        content_type,
        file_bytes: file_bytes
            .filter(|bytes| !bytes.is_empty())
            .ok_or_else(|| {
                bad_request(
                    "file is required for dispute evidence upload",
                    request_id.as_deref(),
                )
            })?,
        metadata,
    };

    let client = state.db.client().map_err(map_db_connect)?;
    let evidence = upload_dispute_evidence_repo(
        &client,
        case_id.as_str(),
        &payload,
        tenant_scope_id.as_str(),
        header(&headers, "x-user-id").as_deref(),
        role.as_str(),
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = if evidence.idempotent_replay {
            "dispute.evidence.upload.idempotent_replay"
        } else {
            "dispute.evidence.upload"
        },
        evidence_id = %evidence.evidence_id,
        case_id = %evidence.case_id,
        object_type = %evidence.object_type,
        "dispute evidence uploaded"
    );
    Ok(ApiResponse::ok(evidence))
}

pub async fn resolve_dispute_case(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(case_id): Path<String>,
    Json(payload): Json<ResolveDisputeCaseRequest>,
) -> Result<Json<ApiResponse<DisputeResolutionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::DisputeCaseResolve,
        "dispute case resolve",
    )?;
    require_step_up_placeholder(&headers, "dispute case resolve")?;
    let request_id = header(&headers, "x-request-id");
    let actor_user_id = header(&headers, "x-user-id").ok_or_else(|| {
        bad_request(
            "x-user-id is required for dispute case resolve",
            request_id.as_deref(),
        )
    })?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let client = state.db.client().map_err(map_db_connect)?;
    let resolution = resolve_dispute_case_repo(
        &client,
        case_id.as_str(),
        &payload,
        actor_user_id.as_str(),
        role.as_str(),
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = if resolution.idempotent_replay {
            "dispute.case.resolve.idempotent_replay"
        } else {
            "dispute.case.resolve"
        },
        case_id = %resolution.case_id,
        decision_id = %resolution.decision_id,
        decision_code = %resolution.decision_code,
        "dispute case resolved"
    );
    Ok(ApiResponse::ok(resolution))
}

fn buyer_tenant_scope_id(
    headers: &HeaderMap,
    action: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    header(headers, "x-tenant-id").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: format!("x-tenant-id is required for {action}"),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })
}

fn required_field(
    field_name: &str,
    value: Option<String>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    value
        .map(non_empty)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            bad_request(
                &format!("{field_name} is required for dispute evidence upload"),
                request_id,
            )
        })
}

fn non_empty(value: String) -> String {
    value.trim().to_string()
}

fn multipart_error(
    err: &impl std::fmt::Display,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    bad_request(
        &format!("dispute evidence multipart parse failed: {err}"),
        request_id,
    )
}

fn bad_request(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
