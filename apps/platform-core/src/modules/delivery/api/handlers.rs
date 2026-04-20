use super::download_middleware::ValidatedDownloadTicket;
use super::support::{DeliveryPermission, header, map_db_connect, require_permission};
use crate::AppState;
use crate::modules::delivery::dto::{
    AcceptOrderRequest, AcceptOrderResponse, ApiUsageLogResponse, CommitOrderDeliveryRequest,
    CommitOrderDeliveryResponse, DownloadFileResponse, DownloadFileResponseData,
    DownloadTicketResponse, ExecuteTemplateRunRequest, ExecuteTemplateRunResponse,
    GetQueryRunsResponse, GetRevisionSubscriptionResponse, GetShareGrantResponse,
    ManageQuerySurfaceRequest, ManageQuerySurfaceResponse, ManageQueryTemplateRequest,
    ManageQueryTemplateResponse, ManageRevisionSubscriptionRequest,
    ManageRevisionSubscriptionResponse, ManageSandboxWorkspaceRequest,
    ManageSandboxWorkspaceResponse, ManageSensitiveExecutionPolicyRequest,
    ManageSensitiveExecutionPolicyResponse, ManageShareGrantRequest, ManageShareGrantResponse,
    ManageTemplateGrantRequest, ManageTemplateGrantResponse, RejectOrderRequest,
    RejectOrderResponse, ReviewResultDisclosureRequest, ReviewResultDisclosureResponse,
};
use crate::modules::delivery::repo::{
    accept_order_delivery, commit_api_delivery, commit_file_delivery, commit_report_delivery,
    consume_download_ticket, execute_template_run, get_api_usage_log, get_query_runs,
    get_revision_subscription, get_share_grants, issue_download_ticket, manage_query_surface,
    manage_query_template, manage_revision_subscription, manage_sandbox_workspace,
    manage_sensitive_execution_policy, manage_share_grant, manage_template_grant,
    reject_order_delivery, review_result_disclosure,
};
use crate::modules::storage::application::fetch_object_bytes;
use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::{HeaderMap, StatusCode};
use base64::Engine;
use http::ApiResponse;
use kernel::ErrorResponse;

pub async fn commit_order_delivery_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CommitOrderDeliveryRequest>,
) -> Result<Json<ApiResponse<CommitOrderDeliveryResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let branch = payload.branch.trim().to_ascii_lowercase();
    match branch.as_str() {
        "file" => require_permission(
            &headers,
            DeliveryPermission::CommitFileDelivery,
            "file delivery commit",
        )?,
        "report" => require_permission(
            &headers,
            DeliveryPermission::CommitReportDelivery,
            "report delivery commit",
        )?,
        "api" => require_permission(
            &headers,
            DeliveryPermission::EnableApiDelivery,
            "api delivery enable",
        )?,
        _ => {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    code: kernel::ErrorCode::TrdStateConflict.as_str().to_string(),
                    message: format!(
                        "DELIVERY_COMMIT_FORBIDDEN: branch `{}` is not supported",
                        payload.branch
                    ),
                    request_id: header(&headers, "x-request-id"),
                }),
            ));
        }
    }

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let idempotency_key = header(&headers, "x-idempotency-key");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let committed = match branch.as_str() {
        "file" => {
            commit_file_delivery(
                &mut client,
                &order_id,
                tenant_id.as_deref(),
                &payload,
                &actor_role,
                request_id.as_deref(),
                trace_id.as_deref(),
                idempotency_key.as_deref(),
            )
            .await?
        }
        "report" => {
            commit_report_delivery(
                &mut client,
                &order_id,
                tenant_id.as_deref(),
                &payload,
                &actor_role,
                request_id.as_deref(),
                trace_id.as_deref(),
                idempotency_key.as_deref(),
            )
            .await?
        }
        "api" => {
            commit_api_delivery(
                &mut client,
                &order_id,
                tenant_id.as_deref(),
                &payload,
                &actor_role,
                request_id.as_deref(),
                trace_id.as_deref(),
                idempotency_key.as_deref(),
            )
            .await?
        }
        _ => unreachable!(),
    };

    Ok(ApiResponse::ok(CommitOrderDeliveryResponse {
        data: committed,
    }))
}

pub async fn accept_order_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<AcceptOrderRequest>,
) -> Result<Json<ApiResponse<AcceptOrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::AcceptDelivery,
        "order acceptance",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let accepted = accept_order_delivery(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(AcceptOrderResponse { data: accepted }))
}

pub async fn reject_order_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<RejectOrderRequest>,
) -> Result<Json<ApiResponse<RejectOrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::RejectDelivery,
        "order rejection",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let rejected = reject_order_delivery(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(RejectOrderResponse { data: rejected }))
}

pub async fn issue_download_ticket_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<DownloadTicketResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::IssueDownloadTicket,
        "download ticket issuance",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let ticket = issue_download_ticket(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(DownloadTicketResponse { data: ticket }))
}

pub async fn get_api_usage_log_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ApiUsageLogResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::ReadApiUsageLog,
        "api usage log read",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let usage_log = get_api_usage_log(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ApiUsageLogResponse { data: usage_log }))
}

pub async fn manage_query_surface_api(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ManageQuerySurfaceRequest>,
) -> Result<Json<ApiResponse<ManageQuerySurfaceResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::ManageQuerySurface,
        "query surface management",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let query_surface = manage_query_surface(
        &mut client,
        &product_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ManageQuerySurfaceResponse {
        data: query_surface,
    }))
}

pub async fn execute_template_run_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ExecuteTemplateRunRequest>,
) -> Result<Json<ApiResponse<ExecuteTemplateRunResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::UseTemplateQuery,
        "template query execution",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let requester_user_id = header(&headers, "x-user-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let query_run = execute_template_run(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        requester_user_id.as_deref(),
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ExecuteTemplateRunResponse {
        data: query_run,
    }))
}

pub async fn get_query_runs_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<GetQueryRunsResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::UseTemplateQuery,
        "template query run read",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let query_runs = get_query_runs(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(GetQueryRunsResponse { data: query_runs }))
}

pub async fn review_result_disclosure_api(
    State(state): State<AppState>,
    Path(query_run_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ReviewResultDisclosureRequest>,
) -> Result<Json<ApiResponse<ReviewResultDisclosureResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::ReviewResultDisclosure,
        "result disclosure review",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let reviewer_user_id = header(&headers, "x-user-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let review = review_result_disclosure(
        &mut client,
        &query_run_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        reviewer_user_id.as_deref(),
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ReviewResultDisclosureResponse {
        data: review,
    }))
}

pub async fn manage_query_template_api(
    State(state): State<AppState>,
    Path(query_surface_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ManageQueryTemplateRequest>,
) -> Result<Json<ApiResponse<ManageQueryTemplateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::ManageQueryTemplate,
        "query template management",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let query_template = manage_query_template(
        &mut client,
        &query_surface_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ManageQueryTemplateResponse {
        data: query_template,
    }))
}

pub async fn manage_sandbox_workspace_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ManageSandboxWorkspaceRequest>,
) -> Result<Json<ApiResponse<ManageSandboxWorkspaceResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::EnableSandboxWorkspace,
        "sandbox workspace enable",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let idempotency_key = header(&headers, "x-idempotency-key");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let workspace = manage_sandbox_workspace(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
        idempotency_key.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ManageSandboxWorkspaceResponse {
        data: workspace,
    }))
}

pub async fn manage_sensitive_execution_policy_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ManageSensitiveExecutionPolicyRequest>,
) -> Result<
    Json<ApiResponse<ManageSensitiveExecutionPolicyResponse>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        DeliveryPermission::ManageSensitiveExecutionPolicy,
        "sensitive execution policy management",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let policy = manage_sensitive_execution_policy(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ManageSensitiveExecutionPolicyResponse {
        data: policy,
    }))
}

pub async fn download_file_api(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(validated): Extension<ValidatedDownloadTicket>,
) -> Result<Json<ApiResponse<DownloadFileResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let client_fingerprint = header(&headers, "x-client-fingerprint");
    let source_ip = header(&headers, "x-forwarded-for")
        .or_else(|| header(&headers, "x-real-ip"))
        .map(|value| {
            value
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .filter(|value| !value.is_empty());

    let fetched_object = fetch_object_bytes(
        &validated.cache_payload.bucket_name,
        &validated.cache_payload.object_key,
    )
    .await?;

    let mut client = state.db.client().map_err(map_db_connect)?;
    let access = consume_download_ticket(
        &mut client,
        &validated.cache_payload,
        &actor_role,
        &validated.raw_token,
        client_fingerprint.as_deref(),
        source_ip.as_deref(),
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    let response = DownloadFileResponseData {
        order_id: access.order_id,
        delivery_id: access.delivery_id,
        ticket_id: access.ticket_id,
        receipt_id: access.receipt_id,
        receipt_hash: access.receipt_hash,
        downloaded_at: access.downloaded_at,
        ticket_status: access.ticket_status,
        download_limit: access.download_limit,
        download_count: access.download_count,
        remaining_downloads: access.remaining_downloads,
        bucket_name: access.bucket_name,
        object_key: access.object_key,
        content_type: access.content_type.or(fetched_object.content_type),
        content_hash: access.content_hash,
        delivery_commit_hash: access.delivery_commit_hash,
        key_envelope: access.key_envelope,
        object_base64: base64::engine::general_purpose::STANDARD.encode(&fetched_object.bytes),
    };

    Ok(ApiResponse::ok(DownloadFileResponse { data: response }))
}

pub async fn manage_revision_subscription_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ManageRevisionSubscriptionRequest>,
) -> Result<Json<ApiResponse<ManageRevisionSubscriptionResponse>>, (StatusCode, Json<ErrorResponse>)>
{
    require_permission(
        &headers,
        DeliveryPermission::ManageRevisionSubscription,
        "revision subscription management",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let subscription = manage_revision_subscription(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ManageRevisionSubscriptionResponse {
        data: subscription,
    }))
}

pub async fn get_revision_subscription_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<GetRevisionSubscriptionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::ReadRevisionSubscription,
        "revision subscription read",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let subscription = get_revision_subscription(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(GetRevisionSubscriptionResponse {
        data: subscription,
    }))
}

pub async fn manage_share_grant_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ManageShareGrantRequest>,
) -> Result<Json<ApiResponse<ManageShareGrantResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::ManageShareGrant,
        "share grant management",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let idempotency_key = header(&headers, "x-idempotency-key");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let share_grant = manage_share_grant(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
        idempotency_key.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ManageShareGrantResponse {
        data: share_grant,
    }))
}

pub async fn manage_template_grant_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ManageTemplateGrantRequest>,
) -> Result<Json<ApiResponse<ManageTemplateGrantResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::EnableTemplateQuery,
        "template query grant management",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let idempotency_key = header(&headers, "x-idempotency-key");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let template_grant = manage_template_grant(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
        idempotency_key.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(ManageTemplateGrantResponse {
        data: template_grant,
    }))
}

pub async fn get_share_grants_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<GetShareGrantResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::ReadShareGrant,
        "share grant read",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let grants = get_share_grants(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(GetShareGrantResponse { data: grants }))
}
