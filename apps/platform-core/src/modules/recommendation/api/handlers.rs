use audit_kit::{AuditContext, AuditEvent};
use auth::{JwtParser, KeycloakClaimsJwtParser, MockJwtParser, SessionSubject, extract_bearer};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
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
    RecommendationPermission, first_matching_role, is_allowed, is_allowed_roles, needs_step_up,
};

const RECOMMENDATION_QUERY_INVALID_ERROR: &str = "RECOMMENDATION_QUERY_INVALID";
const RECOMMENDATION_BACKEND_UNAVAILABLE_ERROR: &str = "RECOMMENDATION_BACKEND_UNAVAILABLE";
const RECOMMENDATION_RESULT_UNAVAILABLE_ERROR: &str = "RECOMMENDATION_RESULT_UNAVAILABLE";
const RECOMMENDATION_BEHAVIOR_INVALID_ERROR: &str = "RECOMMENDATION_BEHAVIOR_INVALID";
const RECOMMENDATION_BEHAVIOR_REFERENCE_MISSING_ERROR: &str =
    "RECOMMENDATION_BEHAVIOR_REFERENCE_MISSING";
const RECOMMENDATION_BEHAVIOR_BACKEND_UNAVAILABLE_ERROR: &str =
    "RECOMMENDATION_BEHAVIOR_BACKEND_UNAVAILABLE";

pub(in crate::modules::recommendation) async fn get_recommendations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<ApiResponse<RecommendationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = request_id(&headers);
    let trace_id = trace_id(&headers, Some(request_id.clone()));
    let subject = require_read_permission(
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
    let subject = require_read_permission(
        &headers,
        RecommendationPermission::PortalRead,
        "recommendation exposure track",
        &request_id,
    )?;
    validate_track_exposure_request(&payload, &request_id)?;
    let idempotency_key =
        required_idempotency_key(&headers, "recommendation exposure track", &request_id)?;
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
    let subject = require_read_permission(
        &headers,
        RecommendationPermission::PortalRead,
        "recommendation click track",
        &request_id,
    )?;
    validate_track_click_request(&payload, &request_id)?;
    let idempotency_key =
        required_idempotency_key(&headers, "recommendation click track", &request_id)?;
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
    require_placeholder_permission(
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
    require_placeholder_permission(
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
    require_placeholder_permission(
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
) -> Result<Json<ApiResponse<RecommendationRankingProfileView>>, (StatusCode, Json<ErrorResponse>)>
{
    require_placeholder_permission(
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
) -> Result<Json<ApiResponse<RecommendationRebuildResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_placeholder_permission(
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

fn require_read_permission(
    headers: &HeaderMap,
    permission: RecommendationPermission,
    action: &str,
    request_id: &str,
) -> Result<SessionSubject, (StatusCode, Json<ErrorResponse>)> {
    let subject = resolve_subject(headers, request_id)?;
    if is_allowed_roles(&subject.roles, permission) {
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

fn require_placeholder_permission(
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

fn resolve_subject(
    headers: &HeaderMap,
    request_id: &str,
) -> Result<SessionSubject, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_bearer(headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "Authorization: Bearer <access_token> is required".to_string(),
                request_id: Some(request_id.to_string()),
            }),
        )
    })?;
    parser_from_env().parse_subject(&token).map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: err.to_string(),
                request_id: Some(request_id.to_string()),
            }),
        )
    })
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

async fn record_recommendation_lookup_side_effects(
    client: &db::Client,
    subject: &SessionSubject,
    permission: RecommendationPermission,
    target_type: &str,
    target_id: Option<String>,
    endpoint: &str,
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
            message_text: format!("recommendation lookup executed: {endpoint}"),
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

fn required_idempotency_key(
    headers: &HeaderMap,
    action: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let Some(idempotency_key) = header(headers, "x-idempotency-key") else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: RECOMMENDATION_BEHAVIOR_INVALID_ERROR.to_string(),
                message: format!("x-idempotency-key is required for {action}"),
                request_id: Some(request_id.to_string()),
            }),
        ));
    };
    if idempotency_key.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: RECOMMENDATION_BEHAVIOR_INVALID_ERROR.to_string(),
                message: format!("x-idempotency-key cannot be empty for {action}"),
                request_id: Some(request_id.to_string()),
            }),
        ));
    }
    Ok(idempotency_key)
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

fn validate_uuid(
    raw: &str,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    Uuid::parse_str(raw).map_err(|_| {
        recommendation_bad_request(
            request_id,
            format!("{field_name} must be a valid uuid: {raw}"),
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
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: RECOMMENDATION_QUERY_INVALID_ERROR.to_string(),
            message,
            request_id: Some(request_id.to_string()),
        }),
    )
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
