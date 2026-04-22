use audit_kit::{AuditContext, AuditEvent};
use auth::{
    AuthorizationFacade, AuthorizationRequest, JwtParser, KeycloakClaimsJwtParser, MockJwtParser,
    NoopStepUpGateway, PermissionChecker, SessionSubject, UnifiedAuthorizationFacade,
};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use db::GenericClient;
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse, new_external_readable_id};
use serde_json::{Value, json};
use sqlx::types::Uuid;
use tracing::warn;

use crate::AppState;
use crate::modules::audit::repo as audit_repo;
use crate::modules::audit::repo::{AccessAuditInsert, SystemLogInsert};
use crate::modules::search::domain::{
    AliasSwitchRequest, CacheInvalidateRequest, PatchRankingProfileRequest, RankingProfileView,
    ReindexRequest, ReindexResponse, SearchQuery, SearchResponse, SearchSyncQuery,
    SearchSyncTaskView,
};
use crate::modules::search::repo;
use crate::modules::search::service::{
    SearchPermission, first_matching_role, is_allowed, needs_step_up, permission_from_code,
};

const SEARCH_QUERY_INVALID_ERROR: &str = "SEARCH_QUERY_INVALID";
const SEARCH_BACKEND_UNAVAILABLE_ERROR: &str = "SEARCH_BACKEND_UNAVAILABLE";
const SEARCH_RESULT_STALE_ERROR: &str = "SEARCH_RESULT_STALE";
const SEARCH_REINDEX_STEP_UP_ACTION: &str = "ops.search_reindex.execute";
const SEARCH_ALIAS_STEP_UP_ACTION: &str = "ops.search_alias.manage";
const SEARCH_RANKING_STEP_UP_ACTION: &str = "ops.search_ranking.manage";

#[derive(Debug, Clone)]
struct StepUpBinding {
    challenge_id: Option<String>,
    token_present: bool,
}

#[derive(Debug, Default, Clone)]
struct SearchPermissionChecker;

impl PermissionChecker for SearchPermissionChecker {
    fn can(&self, subject: &SessionSubject, permission: &str) -> bool {
        permission_from_code(permission)
            .is_some_and(|resolved_permission| is_allowed(&subject.roles, resolved_permission))
    }
}

pub(in crate::modules::search) async fn search_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<SearchResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let subject = require_permission(
        &headers,
        SearchPermission::PortalRead,
        "catalog search read",
        &request_id,
    )?;
    validate_search_query(&query, &request_id)?;

    let client = state_client(&state, &request_id)?;
    let (candidate_page, cache_hit) =
        repo::search_catalog_candidates(&client, &state.runtime.mode, &query)
            .await
            .map_err(|message| map_search_error(&request_id, &message))?;
    let items = repo::hydrate_search_results(&client, &candidate_page.hits)
        .await
        .map_err(|message| map_search_error(&request_id, &message))?;

    record_search_lookup_side_effects(
        &client,
        &headers,
        &subject,
        SearchPermission::PortalRead,
        "search_catalog",
        None,
        "GET /api/v1/catalog/search",
        "catalog search lookup executed",
        json!({
            "q": query.q,
            "entity_scope": query.entity_scope,
            "industry": query.industry,
            "tags": query.tags,
            "delivery_mode": query.delivery_mode,
            "price_min": query.price_min,
            "price_max": query.price_max,
            "sort": query.sort,
            "page": query.page.unwrap_or(1).max(1),
            "page_size": query.page_size.unwrap_or(20).clamp(1, 50),
            "backend": candidate_page.backend,
            "cache_hit": cache_hit,
            "total": candidate_page.total,
            "result_count": items.len(),
        }),
    )
    .await?;

    Ok(ApiResponse::ok(SearchResponse {
        entity_scope: candidate_page.query_scope,
        total: candidate_page.total,
        page: query.page.unwrap_or(1).max(1),
        page_size: query.page_size.unwrap_or(20).clamp(1, 50),
        cache_hit,
        backend: candidate_page.backend,
        items,
    }))
}

pub(in crate::modules::search) async fn get_search_sync(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SearchSyncQuery>,
) -> Result<Json<ApiResponse<Vec<SearchSyncTaskView>>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let subject = require_permission(
        &headers,
        SearchPermission::SyncRead,
        "search sync read",
        &request_id,
    )?;
    let client = state_client(&state, &request_id)?;
    let tasks = repo::list_sync_tasks(&client, &query)
        .await
        .map_err(|message| map_search_error(&request_id, &message))?;

    record_search_lookup_side_effects(
        &client,
        &headers,
        &subject,
        SearchPermission::SyncRead,
        "search_sync_query",
        None,
        "GET /api/v1/ops/search/sync",
        "search ops lookup executed",
        json!({
            "entity_scope": query.entity_scope,
            "sync_status": query.sync_status,
            "limit": query.limit,
            "result_count": tasks.len(),
        }),
    )
    .await?;

    Ok(ApiResponse::ok(tasks))
}

pub(in crate::modules::search) async fn post_search_reindex(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReindexRequest>,
) -> Result<Json<ApiResponse<ReindexResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let subject = require_permission(
        &headers,
        SearchPermission::ReindexExecute,
        "search reindex execute",
        &request_id,
    )?;
    let client = state_client(&state, &request_id)?;
    let actor_user_id = require_actor_user_id(&subject, &request_id)?;
    require_idempotency_key(&headers, "search reindex execute", &request_id)?;

    let scope = normalize_reindex_scope(&payload.entity_scope, &request_id)?;
    validate_reindex_entity_id(&payload, &request_id)?;
    let step_up = require_write_controls(
        &client,
        &headers,
        &subject,
        SearchPermission::ReindexExecute,
        "search reindex execute",
        SEARCH_REINDEX_STEP_UP_ACTION,
        if payload.mode.eq_ignore_ascii_case("single") {
            Some(scope.as_str())
        } else {
            Some("search_scope")
        },
        payload.entity_id.as_deref(),
        &request_id,
    )
    .await?;

    let response = repo::queue_reindex_tasks(&client, &payload)
        .await
        .map_err(|message| map_search_error(&request_id, &message))?;

    record_search_write_side_effects(
        &client,
        &headers,
        &subject,
        SearchPermission::ReindexExecute,
        &step_up,
        "search_reindex",
        payload.entity_id.clone(),
        "search.reindex.queue",
        "queued",
        "POST /api/v1/ops/search/reindex",
        json!({
            "entity_scope": scope,
            "entity_id": payload.entity_id,
            "mode": response.mode,
            "force": payload.force.unwrap_or(false),
            "target_backend": response.target_backend,
            "target_index": response.target_index,
            "enqueued_count": response.enqueued_count,
        }),
        actor_user_id.as_str(),
    )
    .await?;

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
    let request_id = request_id(&headers);
    let subject = require_permission(
        &headers,
        SearchPermission::AliasManage,
        "search alias switch",
        &request_id,
    )?;
    let client = state_client(&state, &request_id)?;
    let actor_user_id = require_actor_user_id(&subject, &request_id)?;
    require_idempotency_key(&headers, "search alias switch", &request_id)?;

    let scope = normalize_alias_scope(&payload.entity_scope, &request_id)?;
    let binding_id = repo::get_alias_binding_id(&client, scope.as_str())
        .await
        .map_err(|message| map_search_error(&request_id, &message))?;
    let step_up = require_write_controls(
        &client,
        &headers,
        &subject,
        SearchPermission::AliasManage,
        "search alias switch",
        SEARCH_ALIAS_STEP_UP_ACTION,
        Some("search_scope"),
        None,
        &request_id,
    )
    .await?;

    let response = repo::switch_alias_binding(&client, &payload)
        .await
        .map_err(|message| map_search_error(&request_id, &message))?;
    let cache_invalidation = match repo::invalidate_scope_cache(&response.entity_scope).await {
        Ok(result) => json!({
            "entity_scope": result.entity_scope,
            "deleted_keys": result.deleted_keys,
            "invalidated_scopes": result.invalidated_scopes,
        }),
        Err(err) => {
            warn!(
                error = %err,
                entity_scope = %response.entity_scope,
                "search alias switch cache invalidation failed"
            );
            json!({
                "error": err,
            })
        }
    };

    record_search_write_side_effects(
        &client,
        &headers,
        &subject,
        SearchPermission::AliasManage,
        &step_up,
        "search_alias_binding",
        Some(binding_id.clone()),
        "search.alias.switch",
        "switched",
        "POST /api/v1/ops/search/aliases/switch",
        json!({
            "entity_scope": response.entity_scope,
            "read_alias": response.read_alias,
            "write_alias": response.write_alias,
            "previous_index_name": response.previous_index_name,
            "active_index_name": response.active_index_name,
            "cache_invalidation": cache_invalidation,
        }),
        actor_user_id.as_str(),
    )
    .await?;

    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::search) async fn post_search_cache_invalidate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CacheInvalidateRequest>,
) -> Result<
    Json<ApiResponse<crate::modules::search::domain::CacheInvalidateResponse>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let request_id = request_id(&headers);
    let subject = require_permission(
        &headers,
        SearchPermission::CacheInvalidate,
        "search cache invalidate",
        &request_id,
    )?;
    let client = state_client(&state, &request_id)?;
    let actor_user_id = require_actor_user_id(&subject, &request_id)?;
    require_idempotency_key(&headers, "search cache invalidate", &request_id)?;
    validate_cache_invalidate_request(&payload, &request_id)?;

    let response = repo::invalidate_search_cache(&payload)
        .await
        .map_err(|message| map_search_error(&request_id, &message))?;

    record_search_write_side_effects(
        &client,
        &headers,
        &subject,
        SearchPermission::CacheInvalidate,
        &StepUpBinding {
            challenge_id: None,
            token_present: false,
        },
        "search_cache",
        None,
        "search.cache.invalidate",
        "deleted",
        "POST /api/v1/ops/search/cache/invalidate",
        json!({
            "entity_scope": payload.entity_scope,
            "query_hash": payload.query_hash,
            "purge_all": payload.purge_all.unwrap_or(false),
            "deleted_keys": response.deleted_keys,
            "invalidated_scopes": response.invalidated_scopes,
        }),
        actor_user_id.as_str(),
    )
    .await?;

    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::search) async fn get_ranking_profiles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<RankingProfileView>>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let subject = require_permission(
        &headers,
        SearchPermission::RankingRead,
        "search ranking profile read",
        &request_id,
    )?;
    let client = state_client(&state, &request_id)?;
    let profiles = repo::list_ranking_profiles(&client)
        .await
        .map_err(|message| map_search_error(&request_id, &message))?;

    record_search_lookup_side_effects(
        &client,
        &headers,
        &subject,
        SearchPermission::RankingRead,
        "search_ranking_profile",
        None,
        "GET /api/v1/ops/search/ranking-profiles",
        "search ops lookup executed",
        json!({
            "result_count": profiles.len(),
        }),
    )
    .await?;

    Ok(ApiResponse::ok(profiles))
}

pub(in crate::modules::search) async fn patch_ranking_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchRankingProfileRequest>,
) -> Result<Json<ApiResponse<RankingProfileView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    validate_uuid(&id, "id", &request_id)?;
    let subject = require_permission(
        &headers,
        SearchPermission::RankingManage,
        "search ranking profile manage",
        &request_id,
    )?;
    let client = state_client(&state, &request_id)?;
    let actor_user_id = require_actor_user_id(&subject, &request_id)?;
    require_idempotency_key(&headers, "search ranking profile manage", &request_id)?;
    let step_up = require_write_controls(
        &client,
        &headers,
        &subject,
        SearchPermission::RankingManage,
        "search ranking profile manage",
        SEARCH_RANKING_STEP_UP_ACTION,
        Some("ranking_profile"),
        Some(id.as_str()),
        &request_id,
    )
    .await?;

    let profile = repo::patch_ranking_profile(&client, &id, &payload)
        .await
        .map_err(|message| map_search_error(&request_id, &message))?;
    let cache_invalidation = match repo::invalidate_scope_cache(&profile.entity_scope).await {
        Ok(result) => json!({
            "entity_scope": result.entity_scope,
            "deleted_keys": result.deleted_keys,
            "invalidated_scopes": result.invalidated_scopes,
        }),
        Err(err) => {
            warn!(
                error = %err,
                ranking_profile_id = %id,
                entity_scope = %profile.entity_scope,
                "search ranking patch cache invalidation failed"
            );
            json!({
                "error": err,
            })
        }
    };

    record_search_write_side_effects(
        &client,
        &headers,
        &subject,
        SearchPermission::RankingManage,
        &step_up,
        "search_ranking_profile",
        Some(id.clone()),
        "search.ranking_profile.patch",
        "updated",
        "PATCH /api/v1/ops/search/ranking-profiles/{id}",
        json!({
            "ranking_profile_id": id,
            "profile_key": profile.profile_key,
            "entity_scope": profile.entity_scope,
            "backend_type": profile.backend_type,
            "weights_json": payload.weights_json,
            "filter_policy_json": payload.filter_policy_json,
            "status": payload.status,
            "cache_invalidation": cache_invalidation,
        }),
        actor_user_id.as_str(),
    )
    .await?;

    Ok(ApiResponse::ok(profile))
}

async fn require_write_controls(
    client: &db::Client,
    headers: &HeaderMap,
    subject: &SessionSubject,
    permission: SearchPermission,
    action: &str,
    expected_step_up_action: &str,
    expected_ref_type: Option<&str>,
    expected_ref_id: Option<&str>,
    request_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    if !needs_step_up(permission) {
        return Ok(StepUpBinding {
            challenge_id: None,
            token_present: false,
        });
    }
    require_step_up_binding(
        client,
        headers,
        subject,
        expected_step_up_action,
        expected_ref_type,
        expected_ref_id,
        action,
        request_id,
    )
    .await
}

fn require_permission(
    headers: &HeaderMap,
    permission: SearchPermission,
    action: &str,
    request_id: &str,
) -> Result<SessionSubject, (StatusCode, Json<ErrorResponse>)> {
    let subject = resolve_subject(headers, request_id)?;
    let facade = authorization_facade();
    let decision = facade
        .evaluate(
            &subject,
            &AuthorizationRequest {
                permission: permission.permission_code().to_string(),
                require_step_up: false,
            },
        )
        .map_err(|err| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: err.to_string(),
                    request_id: Some(request_id.to_string()),
                }),
            )
        })?;
    if decision.allowed {
        return Ok(subject);
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: permission.forbidden_code().to_string(),
            message: format!(
                "{action} is forbidden for current roles; required permission `{}`",
                permission.permission_code()
            ),
            request_id: Some(request_id.to_string()),
        }),
    ))
}

fn resolve_subject(
    headers: &HeaderMap,
    request_id: &str,
) -> Result<SessionSubject, (StatusCode, Json<ErrorResponse>)> {
    authorization_facade()
        .resolve_subject(headers)
        .map_err(|err| unauthorized_subject_error(err, request_id))
}

fn require_actor_user_id(
    subject: &SessionSubject,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    validate_uuid(&subject.user_id, "jwt.sub", request_id)?;
    Ok(subject.user_id.clone())
}

fn require_idempotency_key(
    headers: &HeaderMap,
    action: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if header(headers, "x-idempotency-key").is_some() {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: SEARCH_QUERY_INVALID_ERROR.to_string(),
            message: format!("X-Idempotency-Key is required for {action}"),
            request_id: Some(request_id.to_string()),
        }),
    ))
}

async fn require_step_up_binding(
    client: &db::Client,
    headers: &HeaderMap,
    subject: &SessionSubject,
    expected_action: &str,
    expected_ref_type: Option<&str>,
    expected_ref_id: Option<&str>,
    action_label: &str,
    request_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    let step_up_token = header(headers, "x-step-up-token");
    let challenge_header = header(headers, "x-step-up-challenge-id");
    let raw_binding = step_up_token.clone().or(challenge_header).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: SEARCH_QUERY_INVALID_ERROR.to_string(),
                message: format!("X-Step-Up-Token is required for {action_label}"),
                request_id: Some(request_id.to_string()),
            }),
        )
    })?;
    validate_uuid(raw_binding.as_str(), "X-Step-Up-Token", request_id)?;

    let row = client
        .query_opt(
            "SELECT step_up_challenge_id::text,
                    user_id::text,
                    challenge_status,
                    target_action,
                    COALESCE(target_ref_type, ''),
                    COALESCE(target_ref_id::text, '')
             FROM iam.step_up_challenge
             WHERE step_up_challenge_id = $1::text::uuid",
            &[&raw_binding],
        )
        .await
        .map_err(|err| map_db_error(err, request_id))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: format!("step-up challenge not found: {raw_binding}"),
                    request_id: Some(request_id.to_string()),
                }),
            )
        })?;

    let challenge_id: String = row.get(0);
    let challenge_user_id: String = row.get(1);
    let challenge_status: String = row.get(2);
    let target_action: String = row.get(3);
    let target_ref_type: String = row.get(4);
    let target_ref_id: String = row.get(5);

    if challenge_user_id != subject.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "step-up challenge does not belong to current actor".to_string(),
                request_id: Some(request_id.to_string()),
            }),
        ));
    }
    if challenge_status != "verified" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: format!("verified step-up challenge is required for {action_label}"),
                request_id: Some(request_id.to_string()),
            }),
        ));
    }
    if target_action != expected_action {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: SEARCH_QUERY_INVALID_ERROR.to_string(),
                message: format!("step-up challenge target_action must be {expected_action}"),
                request_id: Some(request_id.to_string()),
            }),
        ));
    }
    if let Some(expected_ref_type) = expected_ref_type {
        if !target_ref_type.is_empty() && target_ref_type != expected_ref_type {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: SEARCH_QUERY_INVALID_ERROR.to_string(),
                    message: format!(
                        "step-up challenge target_ref_type must be `{expected_ref_type}`"
                    ),
                    request_id: Some(request_id.to_string()),
                }),
            ));
        }
    }
    if let Some(expected_ref_id) = expected_ref_id {
        if !target_ref_id.is_empty() && target_ref_id != expected_ref_id {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: SEARCH_QUERY_INVALID_ERROR.to_string(),
                    message: format!("step-up challenge target_ref_id must be `{expected_ref_id}`"),
                    request_id: Some(request_id.to_string()),
                }),
            ));
        }
    }

    Ok(StepUpBinding {
        challenge_id: Some(challenge_id),
        token_present: step_up_token.is_some(),
    })
}

async fn record_search_lookup_side_effects(
    client: &db::Client,
    headers: &HeaderMap,
    subject: &SessionSubject,
    permission: SearchPermission,
    target_type: &str,
    target_id: Option<String>,
    endpoint: &str,
    log_message: &str,
    filters: Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(headers, "x-request-id");
    let trace_id = trace_id(headers, request_id.clone());
    let accessor_role_key = first_matching_role(&subject.roles, permission);
    let access_audit_id = audit_repo::record_access_audit(
        client,
        &AccessAuditInsert {
            accessor_user_id: Some(subject.user_id.clone()),
            accessor_role_key: accessor_role_key.clone(),
            access_mode: "masked".to_string(),
            target_type: target_type.to_string(),
            target_id,
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: None,
            request_id: request_id.clone(),
            trace_id: Some(trace_id.clone()),
            metadata: json!({
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "tenant_id": subject.tenant_id,
                "roles": subject.roles,
                "filters": filters,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error(err, request_id.as_deref().unwrap_or("search-lookup")))?;

    audit_repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: request_id.clone(),
            trace_id: Some(trace_id),
            message_text: format!("{log_message}: {endpoint}"),
            structured_payload: json!({
                "module": "search",
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "access_audit_id": access_audit_id,
                "role": accessor_role_key,
                "filters": filters,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error(err, request_id.as_deref().unwrap_or("search-lookup")))?;

    Ok(())
}

fn validate_search_query(
    query: &SearchQuery,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    match query.entity_scope.trim().to_ascii_lowercase().as_str() {
        "all" | "product" | "service" | "seller" => {}
        other => {
            return Err(search_bad_request(
                request_id,
                format!(
                    "entity_scope must be one of: all, product, service, seller; got `{other}`"
                ),
            ));
        }
    }

    match query.sort.trim().to_ascii_lowercase().as_str() {
        "composite" | "latest" | "price_asc" | "price_desc" | "quality" | "reputation"
        | "hotness" => {}
        other => {
            return Err(search_bad_request(
                request_id,
                format!(
                    "sort must be one of: composite, latest, price_asc, price_desc, quality, reputation, hotness; got `{other}`"
                ),
            ));
        }
    }

    if let (Some(price_min), Some(price_max)) = (query.price_min, query.price_max) {
        if price_min > price_max {
            return Err(search_bad_request(
                request_id,
                format!(
                    "price_min must be less than or equal to price_max; got `{price_min}` > `{price_max}`"
                ),
            ));
        }
    }

    Ok(())
}

async fn record_search_write_side_effects(
    client: &db::Client,
    headers: &HeaderMap,
    subject: &SessionSubject,
    permission: SearchPermission,
    step_up: &StepUpBinding,
    ref_type: &str,
    ref_id: Option<String>,
    action_name: &str,
    result_code: &str,
    endpoint: &str,
    metadata: Value,
    actor_user_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(headers);
    let trace_id = trace_id(headers, Some(request_id.clone()));
    let accessor_role_key = first_matching_role(&subject.roles, permission);

    let mut event = AuditEvent::business(
        "search",
        ref_type,
        ref_id.clone(),
        action_name,
        result_code,
        AuditContext {
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: None,
            tenant_id: subject.tenant_id.clone(),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some(if step_up.challenge_id.is_some() {
                "step_up_required".to_string()
            } else {
                "aal1".to_string()
            }),
            step_up_challenge_id: step_up.challenge_id.clone(),
            metadata: json!({
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "roles": subject.roles,
                "step_up_token_present": step_up.token_present,
                "details": metadata,
            }),
        },
    );
    event.sensitivity_level = if step_up.challenge_id.is_some() {
        "high".to_string()
    } else {
        "normal".to_string()
    };
    audit_repo::insert_audit_event(client, &event)
        .await
        .map_err(|err| map_db_error(err, &request_id))?;

    let access_audit_id = audit_repo::record_access_audit(
        client,
        &AccessAuditInsert {
            accessor_user_id: Some(actor_user_id.to_string()),
            accessor_role_key: accessor_role_key.clone(),
            access_mode: result_code.to_string(),
            target_type: ref_type.to_string(),
            target_id: ref_id.clone(),
            masked_view: false,
            breakglass_reason: None,
            step_up_challenge_id: step_up.challenge_id.clone(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            metadata: json!({
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "tenant_id": subject.tenant_id,
                "roles": subject.roles,
                "step_up_token_present": step_up.token_present,
                "details": metadata,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error(err, &request_id))?;

    audit_repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id),
            message_text: format!("search ops action executed: {endpoint}"),
            structured_payload: json!({
                "module": "search",
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "access_audit_id": access_audit_id,
                "role": accessor_role_key,
                "action_name": action_name,
                "result_code": result_code,
                "details": metadata,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error(err, &request_id))?;

    Ok(())
}

fn validate_cache_invalidate_request(
    payload: &CacheInvalidateRequest,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(scope) = payload.entity_scope.as_deref() {
        match scope.trim().to_ascii_lowercase().as_str() {
            "all" | "product" | "service" | "seller" => {}
            other => {
                return Err(search_bad_request(
                    request_id,
                    format!(
                        "entity_scope must be one of: all, product, service, seller; got `{other}`"
                    ),
                ));
            }
        }
    }
    if payload.query_hash.is_some() && payload.entity_scope.is_none() {
        return Err(search_bad_request(
            request_id,
            "entity_scope is required when query_hash is provided".to_string(),
        ));
    }
    if payload.query_hash.is_some() && payload.purge_all.unwrap_or(false) {
        return Err(search_bad_request(
            request_id,
            "query_hash cannot be combined with purge_all".to_string(),
        ));
    }
    Ok(())
}

fn normalize_reindex_scope(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "product" => Ok("product".to_string()),
        "seller" => Ok("seller".to_string()),
        "all" => Ok("all".to_string()),
        other => Err(search_bad_request(
            request_id,
            format!("entity_scope must be one of: product, seller, all; got `{other}`"),
        )),
    }
}

fn normalize_alias_scope(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "product" => Ok("product".to_string()),
        "seller" => Ok("seller".to_string()),
        other => Err(search_bad_request(
            request_id,
            format!("entity_scope must be one of: product, seller; got `{other}`"),
        )),
    }
}

fn validate_reindex_entity_id(
    payload: &ReindexRequest,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let mode = payload.mode.trim().to_ascii_lowercase();
    match mode.as_str() {
        "single" => {
            let entity_id = payload.entity_id.as_deref().ok_or_else(|| {
                search_bad_request(
                    request_id,
                    "entity_id is required for mode=single".to_string(),
                )
            })?;
            validate_uuid(entity_id, "entity_id", request_id)
        }
        "batch" | "full" => Ok(()),
        other => Err(search_bad_request(
            request_id,
            format!("mode must be one of: single, batch, full; got `{other}`"),
        )),
    }
}

fn validate_uuid(
    raw: &str,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    Uuid::parse_str(raw).map_err(|_| {
        search_bad_request(
            request_id,
            format!("{field_name} must be a valid uuid: {raw}"),
        )
    })?;
    Ok(())
}

fn state_client(
    state: &AppState,
    request_id: &str,
) -> Result<db::Client, (StatusCode, Json<ErrorResponse>)> {
    state
        .db
        .client()
        .map_err(|err| map_db_error(err, request_id))
}

fn parser_from_env() -> Box<dyn JwtParser> {
    match std::env::var("IAM_JWT_PARSER")
        .unwrap_or_else(|_| "keycloak_claims".to_string())
        .as_str()
    {
        "mock" => Box::new(MockJwtParser),
        _ => Box::new(KeycloakClaimsJwtParser),
    }
}

fn authorization_facade() -> UnifiedAuthorizationFacade {
    UnifiedAuthorizationFacade::new(
        parser_from_env(),
        Box::new(SearchPermissionChecker),
        Box::new(NoopStepUpGateway),
    )
}

fn unauthorized_subject_error(
    err: kernel::AppError,
    request_id: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    let message = err.to_string();
    let normalized_message = if message.contains("missing bearer token") {
        "Authorization: Bearer <access_token> is required".to_string()
    } else {
        message
    };
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: normalized_message,
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn request_id(headers: &HeaderMap) -> String {
    header(headers, "x-request-id").unwrap_or_else(|| new_external_readable_id("search"))
}

fn trace_id(headers: &HeaderMap, request_id: Option<String>) -> String {
    header(headers, "x-trace-id")
        .unwrap_or_else(|| request_id.unwrap_or_else(|| new_external_readable_id("search-trace")))
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

fn map_db_error(err: db::Error, request_id: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database operation failed: {err}"),
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn map_search_error(request_id: &str, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    let lower = message.to_ascii_lowercase();
    let (status, code) = if lower.contains("unsupported")
        || lower.contains("required")
        || lower.contains("valid uuid")
    {
        (StatusCode::BAD_REQUEST, SEARCH_QUERY_INVALID_ERROR)
    } else if lower.contains("does not exist") || lower.contains("not found") {
        (StatusCode::NOT_FOUND, SEARCH_QUERY_INVALID_ERROR)
    } else if lower.contains("stale") {
        (StatusCode::CONFLICT, SEARCH_RESULT_STALE_ERROR)
    } else if lower.contains("opensearch") || lower.contains("redis") {
        (StatusCode::BAD_GATEWAY, SEARCH_BACKEND_UNAVAILABLE_ERROR)
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::OpsInternal.as_str(),
        )
    };

    (
        status,
        Json(ErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn search_bad_request(request_id: &str, message: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: SEARCH_QUERY_INVALID_ERROR.to_string(),
            message,
            request_id: Some(request_id.to_string()),
        }),
    )
}
