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

use crate::AppState;
use crate::modules::audit::repo as audit_repo;
use crate::modules::audit::repo::{AccessAuditInsert, SystemLogInsert};
use crate::modules::recommendation::domain::{
    BehaviorTrackResponse, PatchPlacementRequest, PatchRecommendationRankingProfileRequest,
    PlacementView, RecommendationQuery, RecommendationRankingProfileView,
    RecommendationRebuildRequest, RecommendationRebuildResponse, RecommendationResponse,
    TrackClickRequest, TrackExposureRequest,
};
use crate::modules::recommendation::repo;
use crate::modules::recommendation::service::{
    RecommendationPermission, first_matching_role, is_allowed_roles, needs_step_up,
    permission_from_code,
};

const RECOMMENDATION_QUERY_INVALID_ERROR: &str = "RECOMMENDATION_QUERY_INVALID";
const RECOMMENDATION_BACKEND_UNAVAILABLE_ERROR: &str = "RECOMMENDATION_BACKEND_UNAVAILABLE";
const RECOMMENDATION_RESULT_UNAVAILABLE_ERROR: &str = "RECOMMENDATION_RESULT_UNAVAILABLE";
const RECOMMENDATION_BEHAVIOR_INVALID_ERROR: &str = "RECOMMENDATION_BEHAVIOR_INVALID";
const RECOMMENDATION_BEHAVIOR_REFERENCE_MISSING_ERROR: &str =
    "RECOMMENDATION_BEHAVIOR_REFERENCE_MISSING";
const RECOMMENDATION_BEHAVIOR_BACKEND_UNAVAILABLE_ERROR: &str =
    "RECOMMENDATION_BEHAVIOR_BACKEND_UNAVAILABLE";
const RECOMMENDATION_PLACEMENT_INVALID_ERROR: &str = "RECOMMENDATION_PLACEMENT_INVALID";
const RECOMMENDATION_PLACEMENT_NOT_FOUND_ERROR: &str = "RECOMMENDATION_PLACEMENT_NOT_FOUND";
const RECOMMENDATION_PLACEMENT_BACKEND_UNAVAILABLE_ERROR: &str =
    "RECOMMENDATION_PLACEMENT_BACKEND_UNAVAILABLE";
const RECOMMENDATION_RANKING_INVALID_ERROR: &str = "RECOMMENDATION_RANKING_INVALID";
const RECOMMENDATION_RANKING_NOT_FOUND_ERROR: &str = "RECOMMENDATION_RANKING_NOT_FOUND";
const RECOMMENDATION_RANKING_BACKEND_UNAVAILABLE_ERROR: &str =
    "RECOMMENDATION_RANKING_BACKEND_UNAVAILABLE";
const RECOMMENDATION_REBUILD_INVALID_ERROR: &str = "RECOMMENDATION_REBUILD_INVALID";
const RECOMMENDATION_REBUILD_BACKEND_UNAVAILABLE_ERROR: &str =
    "RECOMMENDATION_REBUILD_BACKEND_UNAVAILABLE";
const RECOMMENDATION_REBUILD_STEP_UP_ACTION: &str = "recommendation.rebuild.execute";

#[derive(Debug, Clone)]
struct StepUpBinding {
    challenge_id: Option<String>,
    token_present: bool,
}

#[derive(Debug, Default, Clone)]
struct RecommendationPermissionChecker;

impl PermissionChecker for RecommendationPermissionChecker {
    fn can(&self, subject: &SessionSubject, permission: &str) -> bool {
        permission_from_code(permission).is_some_and(|resolved_permission| {
            is_allowed_roles(&subject.roles, resolved_permission)
        })
    }
}

pub(in crate::modules::recommendation) async fn get_recommendations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<ApiResponse<RecommendationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let trace_id = trace_id(&headers, Some(request_id.clone()));
    let subject = require_permission(
        &headers,
        RecommendationPermission::PortalRead,
        "recommendation read",
        &request_id,
    )?;
    validate_recommendation_query(&query, &request_id)?;

    let client = state_client_with_request_id(&state, &request_id)?;
    let accessor_role_key =
        first_matching_role(&subject.roles, RecommendationPermission::PortalRead)
            .unwrap_or_else(|| "unknown".to_string());
    let response = repo::serve_recommendation(
        &client,
        &state.runtime.mode,
        &query,
        Some(request_id.as_str()),
        Some(trace_id.as_str()),
        accessor_role_key.as_str(),
    )
    .await
    .map_err(|message| map_recommendation_read_error(&request_id, &message))?;

    record_recommendation_lookup_side_effects(
        &client,
        &subject,
        RecommendationPermission::PortalRead,
        "recommendation_result",
        Some(response.recommendation_result_id.clone()),
        "GET /api/v1/recommendations",
        "recommendation lookup executed",
        json!({
            "placement_code": query.placement_code,
            "subject_scope": normalized_subject_scope(&query),
            "subject_org_id": query.subject_org_id,
            "subject_user_id": query.subject_user_id,
            "anonymous_session_key": query.anonymous_session_key,
            "context_entity_scope": query.context_entity_scope,
            "context_entity_id": query.context_entity_id,
            "limit": query.limit.unwrap_or(10).clamp(1, 20),
            "cache_hit": response.cache_hit,
            "item_count": response.items.len(),
            "strategy_version": response.strategy_version,
            "recommendation_request_id": response.recommendation_request_id,
            "recommendation_result_id": response.recommendation_result_id,
        }),
        &request_id,
        &trace_id,
    )
    .await?;

    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn post_track_exposure(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TrackExposureRequest>,
) -> Result<Json<ApiResponse<BehaviorTrackResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let subject = require_permission(
        &headers,
        RecommendationPermission::PortalRead,
        "recommendation exposure track",
        &request_id,
    )?;
    validate_track_exposure_request(&payload, &request_id)?;
    let idempotency_key = required_non_empty_idempotency_key(
        &headers,
        "recommendation exposure track",
        &request_id,
        RECOMMENDATION_BEHAVIOR_INVALID_ERROR,
    )?;
    let effective_trace_id = payload
        .trace_id
        .clone()
        .unwrap_or_else(|| trace_id(&headers, Some(request_id.clone())));
    let client = state_client_with_request_id(&state, &request_id)?;
    let accessor_role_key =
        first_matching_role(&subject.roles, RecommendationPermission::PortalRead)
            .unwrap_or_else(|| "unknown".to_string());
    let response = repo::record_exposure(
        &client,
        &payload,
        Some(request_id.as_str()),
        Some(effective_trace_id.as_str()),
        idempotency_key.as_str(),
        accessor_role_key.as_str(),
    )
    .await
    .map_err(|message| map_recommendation_behavior_error(&request_id, &message))?;

    record_recommendation_behavior_side_effects(
        &client,
        &subject,
        RecommendationPermission::PortalRead,
        "recommendation.exposure.track",
        behavior_result_code(&response),
        "POST /api/v1/recommendations/track/exposure",
        json!({
            "placement_code": payload.placement_code,
            "recommendation_request_id": payload.recommendation_request_id,
            "recommendation_result_id": payload.recommendation_result_id,
            "item_count": payload.items.len(),
            "items": payload.items.iter().map(|item| json!({
                "recommendation_result_item_id": item.recommendation_result_item_id,
                "entity_scope": item.entity_scope,
                "entity_id": item.entity_id,
            })).collect::<Vec<_>>(),
        }),
        &response,
        idempotency_key.as_str(),
        &request_id,
        &effective_trace_id,
    )
    .await?;

    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn post_track_click(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TrackClickRequest>,
) -> Result<Json<ApiResponse<BehaviorTrackResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let subject = require_permission(
        &headers,
        RecommendationPermission::PortalRead,
        "recommendation click track",
        &request_id,
    )?;
    validate_track_click_request(&payload, &request_id)?;
    let idempotency_key = required_non_empty_idempotency_key(
        &headers,
        "recommendation click track",
        &request_id,
        RECOMMENDATION_BEHAVIOR_INVALID_ERROR,
    )?;
    let effective_trace_id = payload
        .trace_id
        .clone()
        .unwrap_or_else(|| trace_id(&headers, Some(request_id.clone())));
    let client = state_client_with_request_id(&state, &request_id)?;
    let accessor_role_key =
        first_matching_role(&subject.roles, RecommendationPermission::PortalRead)
            .unwrap_or_else(|| "unknown".to_string());
    let response = repo::record_click(
        &client,
        &payload,
        Some(request_id.as_str()),
        Some(effective_trace_id.as_str()),
        idempotency_key.as_str(),
        accessor_role_key.as_str(),
    )
    .await
    .map_err(|message| map_recommendation_behavior_error(&request_id, &message))?;

    record_recommendation_behavior_side_effects(
        &client,
        &subject,
        RecommendationPermission::PortalRead,
        "recommendation.click.track",
        behavior_result_code(&response),
        "POST /api/v1/recommendations/track/click",
        json!({
            "recommendation_request_id": payload.recommendation_request_id,
            "recommendation_result_id": payload.recommendation_result_id,
            "recommendation_result_item_id": payload.recommendation_result_item_id,
            "entity_scope": payload.entity_scope,
            "entity_id": payload.entity_id,
        }),
        &response,
        idempotency_key.as_str(),
        &request_id,
        &effective_trace_id,
    )
    .await?;

    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn get_placements(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<PlacementView>>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let trace_id = trace_id(&headers, Some(request_id.clone()));
    let subject = require_permission(
        &headers,
        RecommendationPermission::PlacementRead,
        "recommendation placement read",
        &request_id,
    )?;
    let client = state_client_with_request_id(&state, &request_id)?;
    let response = repo::list_placements(&client)
        .await
        .map_err(|message| map_recommendation_placement_error(&request_id, &message))?;
    record_recommendation_lookup_side_effects(
        &client,
        &subject,
        RecommendationPermission::PlacementRead,
        "recommendation_placement",
        None,
        "GET /api/v1/ops/recommendation/placements",
        "recommendation ops lookup executed",
        json!({
            "placement_count": response.len(),
            "placement_codes": response
                .iter()
                .map(|placement| placement.placement_code.clone())
                .collect::<Vec<_>>(),
        }),
        &request_id,
        &trace_id,
    )
    .await?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn patch_placement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(placement_code): Path<String>,
    Json(payload): Json<PatchPlacementRequest>,
) -> Result<Json<ApiResponse<PlacementView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let trace_id = trace_id(&headers, Some(request_id.clone()));
    let subject = require_permission(
        &headers,
        RecommendationPermission::PlacementManage,
        "recommendation placement manage",
        &request_id,
    )?;
    validate_patch_placement_request(&placement_code, &payload, &request_id)?;
    let idempotency_key = required_non_empty_idempotency_key(
        &headers,
        "recommendation placement manage",
        &request_id,
        RECOMMENDATION_PLACEMENT_INVALID_ERROR,
    )?;
    let actor_user_id = require_actor_user_id(&subject, &request_id)?;
    require_step_up_header(
        &headers,
        "recommendation placement manage",
        &request_id,
        RECOMMENDATION_PLACEMENT_INVALID_ERROR,
    )?;
    let client = state_client_with_request_id(&state, &request_id)?;
    let step_up = require_ops_write_controls(
        &client,
        &headers,
        &subject,
        RecommendationPermission::PlacementManage,
        "recommendation placement manage",
        "recommendation.placement.patch",
        Some("recommendation_placement"),
        None,
        &request_id,
        RECOMMENDATION_PLACEMENT_INVALID_ERROR,
    )
    .await?;
    let accessor_role_key =
        first_matching_role(&subject.roles, RecommendationPermission::PlacementManage)
            .unwrap_or_else(|| "unknown".to_string());
    let response = repo::patch_placement(
        &client,
        &placement_code,
        &payload,
        Some(request_id.as_str()),
        Some(trace_id.as_str()),
        accessor_role_key.as_str(),
    )
    .await
    .map_err(|message| map_recommendation_placement_error(&request_id, &message))?;
    let cache_keys_deleted = repo::invalidate_placement_runtime_cache(&placement_code)
        .await
        .map_err(|message| map_recommendation_placement_error(&request_id, &message))?;
    record_recommendation_write_side_effects(
        &client,
        &subject,
        RecommendationPermission::PlacementManage,
        &step_up,
        "recommendation_placement",
        None,
        "recommendation.placement.patch",
        "updated",
        "PATCH /api/v1/ops/recommendation/placements/{placement_code}",
        json!({
            "placement_code": response.placement_code.clone(),
            "placement_scope": response.placement_scope.clone(),
            "page_context": response.page_context.clone(),
            "default_ranking_profile_key": response.default_ranking_profile_key.clone(),
            "status": response.status.clone(),
            "request_patch": {
                "candidate_policy_json": payload.candidate_policy_json,
                "filter_policy_json": payload.filter_policy_json,
                "default_ranking_profile_key": payload.default_ranking_profile_key,
                "status": payload.status,
                "metadata": payload.metadata,
            },
            "cache_invalidation": {
                "idempotency_key": idempotency_key,
                "cache_keys_deleted": cache_keys_deleted,
            },
        }),
        actor_user_id.as_str(),
        &request_id,
        &trace_id,
    )
    .await?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn get_ranking_profiles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<Vec<RecommendationRankingProfileView>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let request_id = request_id(&headers);
    let _subject = require_permission(
        &headers,
        RecommendationPermission::RankingRead,
        "recommendation ranking profile read",
        &request_id,
    )?;
    let client = state_client_with_request_id(&state, &request_id)?;
    let response = repo::list_ranking_profiles(&client)
        .await
        .map_err(|message| map_recommendation_ranking_error(&request_id, &message))?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn patch_ranking_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchRecommendationRankingProfileRequest>,
) -> Result<Json<ApiResponse<RecommendationRankingProfileView>>, (StatusCode, Json<ErrorResponse>)>
{
    let request_id = request_id(&headers);
    let trace_id = trace_id(&headers, Some(request_id.clone()));
    validate_uuid_with_code(&id, "id", &request_id, RECOMMENDATION_RANKING_INVALID_ERROR)?;
    let subject = require_permission(
        &headers,
        RecommendationPermission::RankingManage,
        "recommendation ranking profile manage",
        &request_id,
    )?;
    let _idempotency_key = required_non_empty_idempotency_key(
        &headers,
        "recommendation ranking profile manage",
        &request_id,
        RECOMMENDATION_RANKING_INVALID_ERROR,
    )?;
    require_step_up_header(
        &headers,
        "recommendation ranking profile manage",
        &request_id,
        RECOMMENDATION_RANKING_INVALID_ERROR,
    )?;
    let client = state_client_with_request_id(&state, &request_id)?;
    let accessor_role_key =
        first_matching_role(&subject.roles, RecommendationPermission::RankingManage)
            .unwrap_or_else(|| "unknown".to_string());
    let response = repo::patch_ranking_profile(
        &client,
        &id,
        &payload,
        Some(request_id.as_str()),
        Some(trace_id.as_str()),
        accessor_role_key.as_str(),
    )
    .await
    .map_err(|message| map_recommendation_ranking_error(&request_id, &message))?;
    Ok(ApiResponse::ok(response))
}

pub(in crate::modules::recommendation) async fn post_rebuild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RecommendationRebuildRequest>,
) -> Result<Json<ApiResponse<RecommendationRebuildResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let trace_id = trace_id(&headers, Some(request_id.clone()));
    let subject = require_permission(
        &headers,
        RecommendationPermission::RebuildExecute,
        "recommendation rebuild execute",
        &request_id,
    )?;
    validate_rebuild_request(&payload, &request_id)?;
    let idempotency_key = required_non_empty_idempotency_key(
        &headers,
        "recommendation rebuild execute",
        &request_id,
        RECOMMENDATION_REBUILD_INVALID_ERROR,
    )?;
    let actor_user_id = require_actor_user_id(&subject, &request_id)?;
    require_step_up_header(
        &headers,
        "recommendation rebuild execute",
        &request_id,
        RECOMMENDATION_REBUILD_INVALID_ERROR,
    )?;
    let client = state_client_with_request_id(&state, &request_id)?;
    let step_up = require_ops_write_controls(
        &client,
        &headers,
        &subject,
        RecommendationPermission::RebuildExecute,
        "recommendation rebuild execute",
        RECOMMENDATION_REBUILD_STEP_UP_ACTION,
        Some("recommendation_rebuild"),
        None,
        &request_id,
        RECOMMENDATION_REBUILD_INVALID_ERROR,
    )
    .await?;
    let accessor_role_key =
        first_matching_role(&subject.roles, RecommendationPermission::RebuildExecute)
            .unwrap_or_else(|| "unknown".to_string());
    let response = repo::rebuild_runtime(
        &client,
        &payload,
        Some(request_id.as_str()),
        Some(trace_id.as_str()),
        accessor_role_key.as_str(),
    )
    .await
    .map_err(|message| map_recommendation_rebuild_error(&request_id, &message))?;

    record_recommendation_write_side_effects(
        &client,
        &subject,
        RecommendationPermission::RebuildExecute,
        &step_up,
        "recommendation_rebuild",
        None,
        "recommendation.rebuild.execute",
        "rebuilt",
        "POST /api/v1/ops/recommendation/rebuild",
        json!({
            "scope": response.scope,
            "placement_code": payload.placement_code,
            "subject_scope": normalized_rebuild_subject_scope(&payload),
            "subject_org_id": payload.subject_org_id,
            "subject_user_id": payload.subject_user_id,
            "anonymous_session_key": payload.anonymous_session_key,
            "entity_scope": payload.entity_scope,
            "entity_id": payload.entity_id,
            "purge_cache": payload.purge_cache.unwrap_or(true),
            "idempotency_key": idempotency_key,
            "cache_keys_deleted": response.cache_keys_deleted,
            "refreshed_subject_profiles": response.refreshed_subject_profiles,
            "refreshed_cohort_rows": response.refreshed_cohort_rows,
            "refreshed_signal_rows": response.refreshed_signal_rows,
            "refreshed_similarity_rows": response.refreshed_similarity_rows,
            "refreshed_bundle_rows": response.refreshed_bundle_rows,
        }),
        actor_user_id.as_str(),
        &request_id,
        &trace_id,
    )
    .await?;

    Ok(ApiResponse::ok(response))
}

async fn require_ops_write_controls(
    client: &db::Client,
    headers: &HeaderMap,
    subject: &SessionSubject,
    permission: RecommendationPermission,
    action: &str,
    expected_step_up_action: &str,
    expected_ref_type: Option<&str>,
    expected_ref_id: Option<&str>,
    request_id: &str,
    invalid_code: &str,
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
        invalid_code,
    )
    .await
}

fn require_permission(
    headers: &HeaderMap,
    permission: RecommendationPermission,
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
    Uuid::parse_str(&subject.user_id).map_err(|_| {
        recommendation_placement_bad_request(
            request_id,
            format!("jwt.sub must be a valid uuid: {}", subject.user_id),
        )
    })?;
    Ok(subject.user_id.clone())
}

fn validate_recommendation_query(
    query: &RecommendationQuery,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if query.placement_code.trim().is_empty() {
        return Err(recommendation_bad_request(
            request_id,
            "placement_code is required".to_string(),
        ));
    }

    if let Some(limit) = query.limit {
        if !(1..=20).contains(&limit) {
            return Err(recommendation_bad_request(
                request_id,
                format!("limit must be between 1 and 20; got `{limit}`"),
            ));
        }
    }

    if let Some(subject_org_id) = query.subject_org_id.as_deref() {
        validate_uuid(subject_org_id, "subject_org_id", request_id)?;
    }
    if let Some(subject_user_id) = query.subject_user_id.as_deref() {
        validate_uuid(subject_user_id, "subject_user_id", request_id)?;
    }

    match normalized_subject_scope(query).as_str() {
        "organization" => {
            if query.subject_org_id.is_none() {
                return Err(recommendation_bad_request(
                    request_id,
                    "subject_org_id is required when subject_scope=organization".to_string(),
                ));
            }
        }
        "user" => {
            if query.subject_user_id.is_none() {
                return Err(recommendation_bad_request(
                    request_id,
                    "subject_user_id is required when subject_scope=user".to_string(),
                ));
            }
        }
        "anonymous" => {
            if query
                .anonymous_session_key
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(recommendation_bad_request(
                    request_id,
                    "anonymous_session_key cannot be empty".to_string(),
                ));
            }
        }
        other => {
            return Err(recommendation_bad_request(
                request_id,
                format!(
                    "subject_scope must be one of: organization, user, anonymous; got `{other}`"
                ),
            ));
        }
    }

    match (
        query.context_entity_scope.as_deref(),
        query.context_entity_id.as_deref(),
    ) {
        (None, None) => {}
        (Some(_), None) => {
            return Err(recommendation_bad_request(
                request_id,
                "context_entity_id is required when context_entity_scope is provided".to_string(),
            ));
        }
        (None, Some(_)) => {
            return Err(recommendation_bad_request(
                request_id,
                "context_entity_scope is required when context_entity_id is provided".to_string(),
            ));
        }
        (Some(scope), Some(entity_id)) => {
            match scope.trim().to_ascii_lowercase().as_str() {
                "product" | "seller" => {}
                other => {
                    return Err(recommendation_bad_request(
                        request_id,
                        format!(
                            "context_entity_scope must be one of: product, seller; got `{other}`"
                        ),
                    ));
                }
            }
            validate_uuid(entity_id, "context_entity_id", request_id)?;
        }
    }

    Ok(())
}

fn validate_track_exposure_request(
    payload: &TrackExposureRequest,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    validate_non_empty(
        &payload.recommendation_request_id,
        "recommendation_request_id",
        request_id,
    )?;
    validate_uuid(
        &payload.recommendation_request_id,
        "recommendation_request_id",
        request_id,
    )?;
    validate_non_empty(
        &payload.recommendation_result_id,
        "recommendation_result_id",
        request_id,
    )?;
    validate_uuid(
        &payload.recommendation_result_id,
        "recommendation_result_id",
        request_id,
    )?;
    validate_non_empty(&payload.placement_code, "placement_code", request_id)?;
    if payload.items.is_empty() {
        return Err(recommendation_behavior_bad_request(
            request_id,
            "items must contain at least one exposure candidate".to_string(),
        ));
    }
    for (index, item) in payload.items.iter().enumerate() {
        let item_field = format!("items[{index}]");
        validate_entity_scope(
            &item.entity_scope,
            format!("{item_field}.entity_scope").as_str(),
            request_id,
        )?;
        validate_non_empty(
            &item.entity_id,
            format!("{item_field}.entity_id").as_str(),
            request_id,
        )?;
        validate_uuid(
            &item.entity_id,
            format!("{item_field}.entity_id").as_str(),
            request_id,
        )?;
        if let Some(result_item_id) = item.recommendation_result_item_id.as_deref() {
            validate_non_empty(
                result_item_id,
                format!("{item_field}.recommendation_result_item_id").as_str(),
                request_id,
            )?;
            validate_uuid(
                result_item_id,
                format!("{item_field}.recommendation_result_item_id").as_str(),
                request_id,
            )?;
        }
    }
    Ok(())
}

fn validate_track_click_request(
    payload: &TrackClickRequest,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    validate_non_empty(
        &payload.recommendation_request_id,
        "recommendation_request_id",
        request_id,
    )?;
    validate_uuid(
        &payload.recommendation_request_id,
        "recommendation_request_id",
        request_id,
    )?;
    validate_non_empty(
        &payload.recommendation_result_id,
        "recommendation_result_id",
        request_id,
    )?;
    validate_uuid(
        &payload.recommendation_result_id,
        "recommendation_result_id",
        request_id,
    )?;
    validate_non_empty(
        &payload.recommendation_result_item_id,
        "recommendation_result_item_id",
        request_id,
    )?;
    validate_uuid(
        &payload.recommendation_result_item_id,
        "recommendation_result_item_id",
        request_id,
    )?;
    validate_entity_scope(&payload.entity_scope, "entity_scope", request_id)?;
    validate_non_empty(&payload.entity_id, "entity_id", request_id)?;
    validate_uuid(&payload.entity_id, "entity_id", request_id)?;
    Ok(())
}

fn validate_patch_placement_request(
    placement_code: &str,
    payload: &PatchPlacementRequest,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if placement_code.trim().is_empty() {
        return Err(recommendation_placement_bad_request(
            request_id,
            "placement_code is required".to_string(),
        ));
    }
    if payload.candidate_policy_json.is_none()
        && payload.filter_policy_json.is_none()
        && payload.default_ranking_profile_key.is_none()
        && payload.status.is_none()
        && payload.metadata.is_none()
    {
        return Err(recommendation_placement_bad_request(
            request_id,
            "at least one placement field must be provided".to_string(),
        ));
    }
    validate_json_object(
        payload.candidate_policy_json.as_ref(),
        "candidate_policy_json",
        request_id,
        RECOMMENDATION_PLACEMENT_INVALID_ERROR,
    )?;
    validate_json_object(
        payload.filter_policy_json.as_ref(),
        "filter_policy_json",
        request_id,
        RECOMMENDATION_PLACEMENT_INVALID_ERROR,
    )?;
    validate_json_object(
        payload.metadata.as_ref(),
        "metadata",
        request_id,
        RECOMMENDATION_PLACEMENT_INVALID_ERROR,
    )?;
    if let Some(profile_key) = payload.default_ranking_profile_key.as_deref() {
        if profile_key.trim().is_empty() {
            return Err(recommendation_placement_bad_request(
                request_id,
                "default_ranking_profile_key cannot be empty".to_string(),
            ));
        }
    }
    if let Some(status) = payload.status.as_deref() {
        if status.trim().is_empty() {
            return Err(recommendation_placement_bad_request(
                request_id,
                "status cannot be empty".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_rebuild_request(
    payload: &RecommendationRebuildRequest,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let scope = payload.scope.trim().to_ascii_lowercase();
    if !matches!(
        scope.as_str(),
        "all"
            | "cache"
            | "features"
            | "subject_profile"
            | "cohort"
            | "signals"
            | "similarity"
            | "bundle"
    ) {
        return Err(recommendation_rebuild_bad_request(
            request_id,
            format!(
                "scope must be one of: all, cache, features, subject_profile, cohort, signals, similarity, bundle; got `{}`",
                payload.scope
            ),
        ));
    }

    if let Some(placement_code) = payload.placement_code.as_deref() {
        if placement_code.trim().is_empty() {
            return Err(recommendation_rebuild_bad_request(
                request_id,
                "placement_code cannot be empty".to_string(),
            ));
        }
    }

    if let Some(subject_org_id) = payload.subject_org_id.as_deref() {
        validate_uuid_with_code(
            subject_org_id,
            "subject_org_id",
            request_id,
            RECOMMENDATION_REBUILD_INVALID_ERROR,
        )?;
    }
    if let Some(subject_user_id) = payload.subject_user_id.as_deref() {
        validate_uuid_with_code(
            subject_user_id,
            "subject_user_id",
            request_id,
            RECOMMENDATION_REBUILD_INVALID_ERROR,
        )?;
    }
    if let Some(anonymous_session_key) = payload.anonymous_session_key.as_deref() {
        if anonymous_session_key.trim().is_empty() {
            return Err(recommendation_rebuild_bad_request(
                request_id,
                "anonymous_session_key cannot be empty".to_string(),
            ));
        }
    }

    if let Some(subject_scope) = payload.subject_scope.as_deref() {
        match subject_scope.trim().to_ascii_lowercase().as_str() {
            "organization" => {
                if payload.subject_org_id.is_none() {
                    return Err(recommendation_rebuild_bad_request(
                        request_id,
                        "subject_org_id is required when subject_scope=organization".to_string(),
                    ));
                }
            }
            "user" => {
                if payload.subject_user_id.is_none() {
                    return Err(recommendation_rebuild_bad_request(
                        request_id,
                        "subject_user_id is required when subject_scope=user".to_string(),
                    ));
                }
            }
            "anonymous" => {
                if payload.anonymous_session_key.is_none() {
                    return Err(recommendation_rebuild_bad_request(
                        request_id,
                        "anonymous_session_key is required when subject_scope=anonymous"
                            .to_string(),
                    ));
                }
            }
            other => {
                return Err(recommendation_rebuild_bad_request(
                    request_id,
                    format!(
                        "subject_scope must be one of: organization, user, anonymous; got `{other}`"
                    ),
                ));
            }
        }
    }

    match (
        payload.entity_scope.as_deref(),
        payload.entity_id.as_deref(),
    ) {
        (None, None) => {}
        (Some(_), None) => {
            return Err(recommendation_rebuild_bad_request(
                request_id,
                "entity_id is required when entity_scope is provided".to_string(),
            ));
        }
        (None, Some(_)) => {
            return Err(recommendation_rebuild_bad_request(
                request_id,
                "entity_scope is required when entity_id is provided".to_string(),
            ));
        }
        (Some(scope), Some(entity_id)) => {
            match scope.trim().to_ascii_lowercase().as_str() {
                "product" | "seller" => {}
                other => {
                    return Err(recommendation_rebuild_bad_request(
                        request_id,
                        format!("entity_scope must be one of: product, seller; got `{other}`"),
                    ));
                }
            }
            validate_uuid_with_code(
                entity_id,
                "entity_id",
                request_id,
                RECOMMENDATION_REBUILD_INVALID_ERROR,
            )?;
        }
    }

    Ok(())
}

async fn record_recommendation_lookup_side_effects(
    client: &db::Client,
    subject: &SessionSubject,
    permission: RecommendationPermission,
    target_type: &str,
    target_id: Option<String>,
    endpoint: &str,
    log_message: &str,
    filters: Value,
    request_id: &str,
    trace_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
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
            request_id: Some(request_id.to_string()),
            trace_id: Some(trace_id.to_string()),
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
    .map_err(|err| map_db_error_with_request_id(err, request_id))?;

    audit_repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.to_string()),
            trace_id: Some(trace_id.to_string()),
            message_text: format!("{log_message}: {endpoint}"),
            structured_payload: json!({
                "module": "recommendation",
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "access_audit_id": access_audit_id,
                "role": accessor_role_key,
                "filters": filters,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error_with_request_id(err, request_id))?;

    Ok(())
}

async fn record_recommendation_behavior_side_effects(
    client: &db::Client,
    subject: &SessionSubject,
    permission: RecommendationPermission,
    action_name: &str,
    result_code: &str,
    endpoint: &str,
    details: Value,
    response: &BehaviorTrackResponse,
    idempotency_key: &str,
    request_id: &str,
    trace_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let accessor_role_key = first_matching_role(&subject.roles, permission);
    let ref_id = response.behavior_event_ids.first().cloned();
    let mut audit_event = AuditEvent::business(
        "recommendation",
        "recommendation_behavior",
        ref_id.clone(),
        action_name,
        result_code,
        AuditContext {
            auth_assurance_level: Some("aal1".to_string()),
            metadata: json!({
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "tenant_id": subject.tenant_id,
                "roles": subject.roles,
                "idempotency_key": idempotency_key,
                "details": details,
                "behavior_event_ids": response.behavior_event_ids,
                "accepted_count": response.accepted_count,
                "deduplicated_count": response.deduplicated_count,
                "outbox_enqueued_count": response.outbox_enqueued_count,
            }),
            ..AuditContext::minimal(
                request_id.to_string(),
                trace_id.to_string(),
                subject.user_id.clone(),
                subject.tenant_id.clone(),
            )
        },
    );
    audit_event.sensitivity_level = "normal".to_string();
    audit_repo::insert_audit_event(client, &audit_event)
        .await
        .map_err(|err| map_db_error_with_request_id(err, request_id))?;

    let access_audit_id = audit_repo::record_access_audit(
        client,
        &AccessAuditInsert {
            accessor_user_id: Some(subject.user_id.clone()),
            accessor_role_key: accessor_role_key.clone(),
            access_mode: result_code.to_string(),
            target_type: "recommendation_behavior".to_string(),
            target_id: ref_id.clone(),
            masked_view: false,
            breakglass_reason: None,
            step_up_challenge_id: None,
            request_id: Some(request_id.to_string()),
            trace_id: Some(trace_id.to_string()),
            metadata: json!({
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "tenant_id": subject.tenant_id,
                "roles": subject.roles,
                "idempotency_key": idempotency_key,
                "details": details,
                "behavior_event_ids": response.behavior_event_ids,
                "accepted_count": response.accepted_count,
                "deduplicated_count": response.deduplicated_count,
                "outbox_enqueued_count": response.outbox_enqueued_count,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error_with_request_id(err, request_id))?;

    audit_repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.to_string()),
            trace_id: Some(trace_id.to_string()),
            message_text: format!("recommendation behavior tracked: {endpoint}"),
            structured_payload: json!({
                "module": "recommendation",
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "access_audit_id": access_audit_id,
                "role": accessor_role_key,
                "action_name": action_name,
                "result_code": result_code,
                "idempotency_key": idempotency_key,
                "details": details,
                "behavior_event_ids": response.behavior_event_ids,
                "accepted_count": response.accepted_count,
                "deduplicated_count": response.deduplicated_count,
                "outbox_enqueued_count": response.outbox_enqueued_count,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error_with_request_id(err, request_id))?;

    Ok(())
}

async fn record_recommendation_write_side_effects(
    client: &db::Client,
    subject: &SessionSubject,
    permission: RecommendationPermission,
    step_up: &StepUpBinding,
    ref_type: &str,
    ref_id: Option<String>,
    action_name: &str,
    result_code: &str,
    endpoint: &str,
    details: Value,
    actor_user_id: &str,
    request_id: &str,
    trace_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let accessor_role_key = first_matching_role(&subject.roles, permission);
    let mut audit_event = AuditEvent::business(
        "recommendation",
        ref_type,
        ref_id.clone(),
        action_name,
        result_code,
        AuditContext {
            auth_assurance_level: Some(if step_up.challenge_id.is_some() {
                "step_up_required".to_string()
            } else {
                "aal1".to_string()
            }),
            step_up_challenge_id: step_up.challenge_id.clone(),
            metadata: json!({
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "tenant_id": subject.tenant_id,
                "roles": subject.roles,
                "step_up_token_present": step_up.token_present,
                "details": details,
            }),
            ..AuditContext::minimal(
                request_id.to_string(),
                trace_id.to_string(),
                actor_user_id.to_string(),
                subject.tenant_id.clone(),
            )
        },
    );
    audit_event.sensitivity_level = if step_up.challenge_id.is_some() {
        "high".to_string()
    } else {
        "normal".to_string()
    };
    audit_repo::insert_audit_event(client, &audit_event)
        .await
        .map_err(|err| map_db_error_with_request_id(err, request_id))?;

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
            request_id: Some(request_id.to_string()),
            trace_id: Some(trace_id.to_string()),
            metadata: json!({
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "tenant_id": subject.tenant_id,
                "roles": subject.roles,
                "step_up_token_present": step_up.token_present,
                "details": details,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error_with_request_id(err, request_id))?;

    audit_repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.to_string()),
            trace_id: Some(trace_id.to_string()),
            message_text: format!("recommendation ops action executed: {endpoint}"),
            structured_payload: json!({
                "module": "recommendation",
                "endpoint": endpoint,
                "permission_code": permission.permission_code(),
                "access_audit_id": access_audit_id,
                "role": accessor_role_key,
                "action_name": action_name,
                "result_code": result_code,
                "details": details,
            }),
        },
    )
    .await
    .map_err(|err| map_db_error_with_request_id(err, request_id))?;

    Ok(())
}

fn required_non_empty_idempotency_key(
    headers: &HeaderMap,
    action: &str,
    request_id: &str,
    error_code: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let Some(idempotency_key) = header(headers, "x-idempotency-key") else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: error_code.to_string(),
                message: format!("x-idempotency-key is required for {action}"),
                request_id: Some(request_id.to_string()),
            }),
        ));
    };
    if idempotency_key.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: error_code.to_string(),
                message: format!("x-idempotency-key cannot be empty for {action}"),
                request_id: Some(request_id.to_string()),
            }),
        ));
    }
    Ok(idempotency_key)
}

fn require_step_up_header(
    headers: &HeaderMap,
    action: &str,
    request_id: &str,
    error_code: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if header(headers, "x-step-up-token").is_some()
        || header(headers, "x-step-up-challenge-id").is_some()
    {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: error_code.to_string(),
            message: format!("X-Step-Up-Token is required for {action}"),
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
    invalid_code: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    let step_up_token = header(headers, "x-step-up-token");
    let challenge_header = header(headers, "x-step-up-challenge-id");
    let raw_binding = step_up_token.clone().or(challenge_header).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: invalid_code.to_string(),
                message: format!("X-Step-Up-Token is required for {action_label}"),
                request_id: Some(request_id.to_string()),
            }),
        )
    })?;
    validate_uuid_with_code(
        raw_binding.as_str(),
        "X-Step-Up-Token",
        request_id,
        invalid_code,
    )?;

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
        .map_err(|err| map_db_error_with_request_id(err, request_id))?
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
                code: invalid_code.to_string(),
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
                    code: invalid_code.to_string(),
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
                    code: invalid_code.to_string(),
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

fn state_client_with_request_id(
    state: &AppState,
    request_id: &str,
) -> Result<db::Client, (StatusCode, Json<ErrorResponse>)> {
    state
        .db
        .client()
        .map_err(|err| map_db_error_with_request_id(err, request_id))
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
        Box::new(RecommendationPermissionChecker),
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

fn normalized_subject_scope(query: &RecommendationQuery) -> String {
    query.subject_scope.clone().unwrap_or_else(|| {
        if query.subject_user_id.is_some() {
            "user".to_string()
        } else if query.subject_org_id.is_some() {
            "organization".to_string()
        } else {
            "anonymous".to_string()
        }
    })
}

fn normalized_rebuild_subject_scope(payload: &RecommendationRebuildRequest) -> String {
    payload.subject_scope.clone().unwrap_or_else(|| {
        if payload.subject_user_id.is_some() {
            "user".to_string()
        } else if payload.subject_org_id.is_some() {
            "organization".to_string()
        } else {
            "anonymous".to_string()
        }
    })
}

fn validate_uuid(
    raw: &str,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    validate_uuid_with_code(
        raw,
        field_name,
        request_id,
        RECOMMENDATION_QUERY_INVALID_ERROR,
    )?;
    Ok(())
}

fn validate_uuid_with_code(
    raw: &str,
    field_name: &str,
    request_id: &str,
    error_code: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    Uuid::parse_str(raw).map_err(|_| {
        recommendation_bad_request_with_code(
            request_id,
            format!("{field_name} must be a valid uuid: {raw}"),
            error_code,
        )
    })?;
    Ok(())
}

fn validate_non_empty(
    raw: &str,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if raw.trim().is_empty() {
        return Err(recommendation_behavior_bad_request(
            request_id,
            format!("{field_name} is required"),
        ));
    }
    Ok(())
}

fn validate_entity_scope(
    raw: &str,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "product" | "seller" => Ok(()),
        other => Err(recommendation_behavior_bad_request(
            request_id,
            format!("{field_name} must be one of: product, seller; got `{other}`"),
        )),
    }
}

fn recommendation_bad_request(
    request_id: &str,
    message: String,
) -> (StatusCode, Json<ErrorResponse>) {
    recommendation_bad_request_with_code(request_id, message, RECOMMENDATION_QUERY_INVALID_ERROR)
}

fn recommendation_bad_request_with_code(
    request_id: &str,
    message: String,
    code: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: code.to_string(),
            message,
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn recommendation_placement_bad_request(
    request_id: &str,
    message: String,
) -> (StatusCode, Json<ErrorResponse>) {
    recommendation_bad_request_with_code(
        request_id,
        message,
        RECOMMENDATION_PLACEMENT_INVALID_ERROR,
    )
}

fn recommendation_rebuild_bad_request(
    request_id: &str,
    message: String,
) -> (StatusCode, Json<ErrorResponse>) {
    recommendation_bad_request_with_code(request_id, message, RECOMMENDATION_REBUILD_INVALID_ERROR)
}

fn recommendation_behavior_bad_request(
    request_id: &str,
    message: String,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: RECOMMENDATION_BEHAVIOR_INVALID_ERROR.to_string(),
            message,
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn validate_json_object(
    value: Option<&Value>,
    field_name: &str,
    request_id: &str,
    error_code: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(value) = value {
        if !value.is_object() {
            return Err(recommendation_bad_request_with_code(
                request_id,
                format!("{field_name} must be a JSON object"),
                error_code,
            ));
        }
    }
    Ok(())
}

fn request_id(headers: &HeaderMap) -> String {
    header(headers, "x-request-id").unwrap_or_else(|| new_external_readable_id("recommend"))
}

fn trace_id(headers: &HeaderMap, request_id: Option<String>) -> String {
    header(headers, "x-trace-id").unwrap_or_else(|| {
        request_id.unwrap_or_else(|| new_external_readable_id("recommend-trace"))
    })
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

fn map_db_error_with_request_id(
    err: db::Error,
    request_id: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database operation failed: {err}"),
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn map_recommendation_behavior_error(
    request_id: &str,
    message: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    let lower = message.to_ascii_lowercase();
    let (status, code) = if lower.contains("required")
        || lower.contains("valid uuid")
        || lower.contains("must be")
        || lower.contains("does not match")
    {
        (
            StatusCode::BAD_REQUEST,
            RECOMMENDATION_BEHAVIOR_INVALID_ERROR,
        )
    } else if lower.contains("reference missing") {
        (
            StatusCode::NOT_FOUND,
            RECOMMENDATION_BEHAVIOR_REFERENCE_MISSING_ERROR,
        )
    } else if lower.contains("opensearch") || lower.contains("redis") {
        (
            StatusCode::BAD_GATEWAY,
            RECOMMENDATION_BEHAVIOR_BACKEND_UNAVAILABLE_ERROR,
        )
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

fn behavior_result_code(response: &BehaviorTrackResponse) -> &'static str {
    if response.deduplicated_count > 0 && response.accepted_count == 0 {
        "deduplicated"
    } else {
        "accepted"
    }
}

fn map_recommendation_placement_error(
    request_id: &str,
    message: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    let lower = message.to_ascii_lowercase();
    let (status, code) = if lower.contains("required")
        || lower.contains("must be")
        || lower.contains("at least one")
        || lower.contains("valid uuid")
        || lower.contains("step-up challenge target_")
    {
        (
            StatusCode::BAD_REQUEST,
            RECOMMENDATION_PLACEMENT_INVALID_ERROR,
        )
    } else if lower.contains("placement missing") || lower.contains("ranking profile missing") {
        (
            StatusCode::NOT_FOUND,
            RECOMMENDATION_PLACEMENT_NOT_FOUND_ERROR,
        )
    } else if lower.contains("redis") || lower.contains("opensearch") {
        (
            StatusCode::BAD_GATEWAY,
            RECOMMENDATION_PLACEMENT_BACKEND_UNAVAILABLE_ERROR,
        )
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

fn map_recommendation_rebuild_error(
    request_id: &str,
    message: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    let lower = message.to_ascii_lowercase();
    let (status, code) = if lower.contains("required")
        || lower.contains("must be")
        || lower.contains("valid uuid")
        || lower.contains("unsupported")
        || lower.contains("step-up challenge target_")
    {
        (
            StatusCode::BAD_REQUEST,
            RECOMMENDATION_REBUILD_INVALID_ERROR,
        )
    } else if lower.contains("redis") || lower.contains("opensearch") {
        (
            StatusCode::BAD_GATEWAY,
            RECOMMENDATION_REBUILD_BACKEND_UNAVAILABLE_ERROR,
        )
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

fn map_recommendation_ranking_error(
    request_id: &str,
    message: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    let lower = message.to_ascii_lowercase();
    let (status, code) = if lower.contains("required")
        || lower.contains("must be")
        || lower.contains("valid uuid")
        || lower.contains("step-up challenge target_")
    {
        (
            StatusCode::BAD_REQUEST,
            RECOMMENDATION_RANKING_INVALID_ERROR,
        )
    } else if lower.contains("recommendation ranking profile missing") {
        (
            StatusCode::NOT_FOUND,
            RECOMMENDATION_RANKING_NOT_FOUND_ERROR,
        )
    } else if lower.contains("redis") || lower.contains("opensearch") {
        (
            StatusCode::BAD_GATEWAY,
            RECOMMENDATION_RANKING_BACKEND_UNAVAILABLE_ERROR,
        )
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

fn map_recommendation_read_error(
    request_id: &str,
    message: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    let lower = message.to_ascii_lowercase();
    let (status, code) = if lower.contains("required")
        || lower.contains("valid uuid")
        || lower.contains("must be")
        || lower.contains("unsupported")
    {
        (StatusCode::BAD_REQUEST, RECOMMENDATION_QUERY_INVALID_ERROR)
    } else if lower.contains("placement missing") || lower.contains("profile missing") {
        (StatusCode::NOT_FOUND, RECOMMENDATION_QUERY_INVALID_ERROR)
    } else if lower.contains("no recommendation candidates available") {
        (
            StatusCode::NOT_FOUND,
            RECOMMENDATION_RESULT_UNAVAILABLE_ERROR,
        )
    } else if lower.contains("opensearch") || lower.contains("redis") {
        (
            StatusCode::BAD_GATEWAY,
            RECOMMENDATION_BACKEND_UNAVAILABLE_ERROR,
        )
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
