use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};

use crate::AppState;
use crate::modules::recommendation::domain::{
    BehaviorTrackResponse, PatchPlacementRequest, PatchRecommendationRankingProfileRequest,
    PlacementView, RecommendationQuery, RecommendationRankingProfileView,
    RecommendationRebuildRequest, RecommendationRebuildResponse, RecommendationResponse,
    TrackClickRequest, TrackExposureRequest,
};
use crate::modules::recommendation::repo;
use crate::modules::recommendation::service::{
    RecommendationPermission, is_allowed, needs_step_up,
};

pub(in crate::modules::recommendation) async fn get_recommendations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<ApiResponse<RecommendationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        RecommendationPermission::PortalRead,
        "recommendation read",
    )?;
    let client = state_client(&state)?;
    let response = repo::serve_recommendation(
        &client,
        &query,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_recommendation_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn post_track_exposure(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TrackExposureRequest>,
) -> Result<Json<ApiResponse<BehaviorTrackResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        RecommendationPermission::PortalRead,
        "recommendation exposure track",
    )?;
    require_idempotency_key(&headers, "recommendation exposure track")?;
    let header_trace_id = header(&headers, "x-trace-id");
    let effective_trace_id = payload.trace_id.clone().or(header_trace_id);
    let client = state_client(&state)?;
    let response = repo::record_exposure(
        &client,
        &payload,
        header(&headers, "x-request-id").as_deref(),
        effective_trace_id.as_deref(),
        header(&headers, "x-idempotency-key").as_deref().unwrap_or_default(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_recommendation_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn post_track_click(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TrackClickRequest>,
) -> Result<Json<ApiResponse<BehaviorTrackResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        RecommendationPermission::PortalRead,
        "recommendation click track",
    )?;
    require_idempotency_key(&headers, "recommendation click track")?;
    let header_trace_id = header(&headers, "x-trace-id");
    let effective_trace_id = payload.trace_id.clone().or(header_trace_id);
    let client = state_client(&state)?;
    let response = repo::record_click(
        &client,
        &payload,
        header(&headers, "x-request-id").as_deref(),
        effective_trace_id.as_deref(),
        header(&headers, "x-idempotency-key").as_deref().unwrap_or_default(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_recommendation_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn get_placements(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<PlacementView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        RecommendationPermission::PlacementRead,
        "recommendation placement read",
    )?;
    let client = state_client(&state)?;
    let response = repo::list_placements(&client)
        .await
        .map_err(map_recommendation_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn patch_placement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(placement_code): Path<String>,
    Json(payload): Json<PatchPlacementRequest>,
) -> Result<Json<ApiResponse<PlacementView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        RecommendationPermission::PlacementManage,
        "recommendation placement manage",
    )?;
    require_write_controls(
        &headers,
        RecommendationPermission::PlacementManage,
        "recommendation placement manage",
    )?;
    let client = state_client(&state)?;
    let response = repo::patch_placement(
        &client,
        &placement_code,
        &payload,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_recommendation_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn get_ranking_profiles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<Vec<RecommendationRankingProfileView>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        RecommendationPermission::RankingRead,
        "recommendation ranking profile read",
    )?;
    let client = state_client(&state)?;
    let response = repo::list_ranking_profiles(&client)
        .await
        .map_err(map_recommendation_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn patch_ranking_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchRecommendationRankingProfileRequest>,
) -> Result<
    Json<ApiResponse<RecommendationRankingProfileView>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        RecommendationPermission::RankingManage,
        "recommendation ranking profile manage",
    )?;
    require_write_controls(
        &headers,
        RecommendationPermission::RankingManage,
        "recommendation ranking profile manage",
    )?;
    let client = state_client(&state)?;
    let response = repo::patch_ranking_profile(
        &client,
        &id,
        &payload,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_recommendation_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn post_rebuild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RecommendationRebuildRequest>,
) -> Result<
    Json<ApiResponse<RecommendationRebuildResponse>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        RecommendationPermission::RebuildExecute,
        "recommendation rebuild execute",
    )?;
    require_write_controls(
        &headers,
        RecommendationPermission::RebuildExecute,
        "recommendation rebuild execute",
    )?;
    let client = state_client(&state)?;
    let response = repo::rebuild_runtime(
        &client,
        &payload,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_recommendation_error)?;
    Ok(ApiResponse::ok(response))
}

fn require_write_controls(
    headers: &HeaderMap,
    permission: RecommendationPermission,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    require_idempotency_key(headers, action)?;
    if needs_step_up(permission) {
        require_step_up_placeholder(headers, action)?;
    }
    Ok(())
}

fn require_permission(
    headers: &HeaderMap,
    permission: RecommendationPermission,
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

fn require_idempotency_key(
    headers: &HeaderMap,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if header(headers, "x-idempotency-key").is_some() {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("x-idempotency-key is required for {action}"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn require_step_up_placeholder(
    headers: &HeaderMap,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if header(headers, "x-step-up-token").is_some()
        || header(headers, "x-step-up-challenge-id").is_some()
    {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("x-step-up-token or x-step-up-challenge-id is required for {action}"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn state_client(state: &AppState) -> Result<db::Client, (StatusCode, Json<ErrorResponse>)> {
    state.db.client().map_err(map_db_error)
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

fn map_db_error(err: db::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database operation failed: {err}"),
            request_id: None,
        }),
    )
}

fn map_recommendation_error(message: String) -> (StatusCode, Json<ErrorResponse>) {
    let status = if message.contains("opensearch") {
        StatusCode::BAD_GATEWAY
    } else if message.contains("redis") {
        StatusCode::SERVICE_UNAVAILABLE
    } else if message.contains("forbidden")
        || message.contains("missing")
        || message.contains("invalid")
        || message.contains("required")
    {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    (
        status,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message,
            request_id: None,
        }),
    )
}
