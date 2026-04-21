use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};

use crate::AppState;
use crate::modules::search::domain::{
    AliasSwitchRequest, CacheInvalidateRequest, PatchRankingProfileRequest, ReindexRequest,
    SearchQuery, SearchResponse, SearchSyncQuery,
};
use crate::modules::search::repo;
use crate::modules::search::service::{SearchPermission, is_allowed, needs_step_up};

pub(in crate::modules::search) async fn search_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<SearchResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        SearchPermission::PortalRead,
        "catalog search read",
    )?;
    let client = state_client(&state)?;
    let (candidate_page, cache_hit) = repo::search_catalog_candidates(&query)
        .await
        .map_err(map_search_error)?;
    let items = repo::hydrate_search_results(&client, &candidate_page.hits)
        .await
        .map_err(map_search_error)?;
    Ok(ApiResponse::ok(SearchResponse {
        entity_scope: candidate_page.query_scope,
        total: candidate_page.total,
        page: query.page.unwrap_or(1).max(1),
        page_size: query.page_size.unwrap_or(20).clamp(1, 50),
        cache_hit,
        backend: "opensearch".to_string(),
        items,
    }))
}

pub(in crate::modules::search) async fn get_search_sync(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SearchSyncQuery>,
) -> Result<
    Json<ApiResponse<Vec<crate::modules::search::domain::SearchSyncTaskView>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(&headers, SearchPermission::SyncRead, "search sync read")?;
    let client = state_client(&state)?;
    let tasks = repo::list_sync_tasks(&client, &query)
        .await
        .map_err(map_search_error)?;
    Ok(ApiResponse::ok(tasks))
}

pub(in crate::modules::search) async fn post_search_reindex(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReindexRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::search::domain::ReindexResponse>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        SearchPermission::ReindexExecute,
        "search reindex execute",
    )?;
    require_write_controls(
        &headers,
        SearchPermission::ReindexExecute,
        "search reindex execute",
    )?;
    let client = state_client(&state)?;
    let response = repo::queue_reindex_tasks(
        &client,
        &payload,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_search_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::search) async fn post_search_alias_switch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AliasSwitchRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::search::domain::AliasSwitchResponse>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        SearchPermission::AliasManage,
        "search alias switch",
    )?;
    require_write_controls(
        &headers,
        SearchPermission::AliasManage,
        "search alias switch",
    )?;
    let client = state_client(&state)?;
    let response = repo::switch_alias_binding(
        &client,
        &payload,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_search_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::search) async fn post_search_cache_invalidate(
    headers: HeaderMap,
    Json(payload): Json<CacheInvalidateRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::search::domain::CacheInvalidateResponse>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        SearchPermission::CacheInvalidate,
        "search cache invalidate",
    )?;
    require_write_controls(
        &headers,
        SearchPermission::CacheInvalidate,
        "search cache invalidate",
    )?;
    let response = repo::invalidate_search_cache(&payload)
        .await
        .map_err(map_search_error)?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::search) async fn get_ranking_profiles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<Vec<crate::modules::search::domain::RankingProfileView>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        SearchPermission::RankingRead,
        "search ranking profile read",
    )?;
    let client = state_client(&state)?;
    let profiles = repo::list_ranking_profiles(&client)
        .await
        .map_err(map_search_error)?;
    Ok(ApiResponse::ok(profiles))
}

pub(in crate::modules::search) async fn patch_ranking_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchRankingProfileRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::search::domain::RankingProfileView>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        SearchPermission::RankingManage,
        "search ranking profile manage",
    )?;
    require_write_controls(
        &headers,
        SearchPermission::RankingManage,
        "search ranking profile manage",
    )?;
    let client = state_client(&state)?;
    let profile = repo::patch_ranking_profile(
        &client,
        &id,
        &payload,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
    )
    .await
    .map_err(map_search_error)?;
    Ok(ApiResponse::ok(profile))
}

fn require_write_controls(
    headers: &HeaderMap,
    permission: SearchPermission,
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
    permission: SearchPermission,
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

fn map_search_error(message: String) -> (StatusCode, Json<ErrorResponse>) {
    let status = if message.contains("opensearch") {
        StatusCode::BAD_GATEWAY
    } else if message.contains("redis") {
        StatusCode::SERVICE_UNAVAILABLE
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
