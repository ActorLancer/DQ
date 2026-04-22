use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use config::RuntimeMode;
use db::{Client, GenericClient};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::types::Uuid;

use crate::modules::recommendation::domain::{
    BehaviorTrackResponse, PatchPlacementRequest, PatchRecommendationRankingProfileRequest,
    PlacementView, RecommendationItem, RecommendationQuery, RecommendationRankingProfileView,
    RecommendationRebuildRequest, RecommendationRebuildResponse, RecommendationResponse,
    TrackClickRequest, TrackExposureRequest,
};
use crate::modules::search::domain::SearchResultItem;
use crate::modules::search::repo::{SearchCandidate, hydrate_search_results};
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};

type RepoResult<T> = Result<T, String>;

#[derive(Debug, Clone)]
struct PlacementDefinition {
    placement_code: String,
    placement_name: String,
    placement_scope: String,
    page_context: String,
    candidate_policy_json: Value,
    filter_policy_json: Value,
    default_ranking_profile_key: Option<String>,
    status: String,
    metadata: Value,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct RankingProfile {
    recommendation_ranking_profile_id: String,
    profile_key: String,
    placement_scope: String,
    backend_type: String,
    weights_json: Value,
    diversity_policy_json: Value,
    exploration_policy_json: Value,
    explain_codes: Vec<String>,
    status: String,
    stage_from: String,
    metadata: Value,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateBackend {
    OpenSearch,
    PostgresqlLocalMinimal,
}

impl CandidateBackend {
    fn as_str(&self) -> &'static str {
        match self {
            CandidateBackend::OpenSearch => "opensearch",
            CandidateBackend::PostgresqlLocalMinimal => "postgresql_local_minimal",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateSnapshot {
    strategy_version: String,
    candidates: Vec<CandidateSeed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateSeed {
    entity_scope: String,
    entity_id: String,
    raw_score: f64,
    recall_sources: Vec<String>,
    explanation_codes: Vec<String>,
}

#[derive(Debug, Clone)]
struct RecallCandidate {
    entity_scope: String,
    entity_id: String,
    raw_score: f64,
    recall_sources: BTreeSet<String>,
    explanation_codes: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct RankedCandidate {
    entity_scope: String,
    entity_id: String,
    raw_score: f64,
    final_score: f64,
    recall_sources: Vec<String>,
    explanation_codes: Vec<String>,
    search_item: SearchResultItem,
}

#[derive(Debug, Clone)]
struct ContextEntity {
    entity_scope: String,
    entity_id: String,
    seller_org_id: Option<String>,
    category: Option<String>,
    industry: Option<String>,
    tags: Vec<String>,
    product_type: Option<String>,
}

#[derive(Debug, Clone)]
struct PlacementFixedSample {
    sample_order: usize,
    entity_scope: String,
    entity_id: String,
    scenario_code: Option<String>,
}

#[derive(Debug, Clone)]
struct SubjectProfile {
    preferred_categories: Vec<String>,
    preferred_tags: Vec<String>,
    preferred_delivery_modes: Vec<String>,
}

#[derive(Debug, Clone)]
struct RecommendationReference {
    placement_code: String,
    page_context: Option<String>,
    subject_scope: String,
    subject_org_id: Option<String>,
    subject_user_id: Option<String>,
    anonymous_session_key: Option<String>,
    recommendation_request_id: String,
    recommendation_result_id: String,
}

pub async fn serve_recommendation(
    client: &Client,
    runtime_mode: &RuntimeMode,
    query: &RecommendationQuery,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    actor_role: &str,
) -> RepoResult<RecommendationResponse> {
    let tx = client
        .transaction()
        .await
        .map_err(|err| format!("open recommendation transaction failed: {err}"))?;
    let placement = load_active_placement(&tx, &query.placement_code).await?;
    let ranking_profile = load_effective_ranking_profile(&tx, &placement).await?;
    let limit = recommendation_limit(query.limit);
    let candidate_backend = candidate_backend_for_runtime(runtime_mode);
    let cache_key = recommendation_cache_key(query, candidate_backend)?;
    let mut cache_hit = false;
    let mut snapshot = match load_candidate_cache(&cache_key).await? {
        Some(snapshot) => {
            cache_hit = true;
            snapshot
        }
        None => {
            let generated = generate_candidate_snapshot(
                &tx,
                runtime_mode,
                query,
                &placement,
                &ranking_profile,
                limit,
            )
            .await?;
            store_candidate_cache(&cache_key, &generated).await?;
            generated
        }
    };

    let seen_entities = load_seen_entities(query, &placement.placement_code).await?;
    let hydrated = hydrate_candidates(&tx, &snapshot, &seen_entities, limit).await?;
    let ranked = if hydrated.is_empty() {
        snapshot = generate_fallback_snapshot(
            &tx,
            runtime_mode,
            query,
            &placement,
            &ranking_profile,
            limit,
        )
        .await?;
        cache_hit = false;
        hydrate_candidates(&tx, &snapshot, &seen_entities, limit).await?
    } else {
        hydrated
    };
    let final_items = ranked.into_iter().take(limit as usize).collect::<Vec<_>>();
    if final_items.is_empty() {
        return Err(format!(
            "no recommendation candidates available for placement_code={}",
            placement.placement_code
        ));
    }

    let response = persist_recommendation_result(
        &tx,
        query,
        &placement,
        &ranking_profile,
        &snapshot.strategy_version,
        candidate_backend,
        runtime_mode,
        &final_items,
        cache_hit,
        request_id,
        trace_id,
        actor_role,
    )
    .await?;
    tx.commit()
        .await
        .map_err(|err| format!("commit recommendation transaction failed: {err}"))?;
    Ok(response)
}

pub async fn record_exposure(
    client: &Client,
    request: &TrackExposureRequest,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: &str,
    actor_role: &str,
) -> RepoResult<BehaviorTrackResponse> {
    if request.items.is_empty() {
        return Err("recommendation exposure items are required".to_string());
    }
    let tx = client
        .transaction()
        .await
        .map_err(|err| format!("open recommendation exposure transaction failed: {err}"))?;
    let existing = existing_behavior_events(
        &tx,
        &["recommendation_panel_viewed", "recommendation_item_exposed"],
        idempotency_key,
    )
    .await?;
    if !existing.is_empty() {
        tx.rollback()
            .await
            .map_err(|err| format!("rollback duplicate exposure transaction failed: {err}"))?;
        return Ok(BehaviorTrackResponse {
            accepted_count: 0,
            deduplicated_count: existing.len() as u64,
            behavior_event_ids: existing,
            outbox_enqueued_count: 0,
        });
    }

    let reference = load_recommendation_reference(
        &tx,
        &request.recommendation_request_id,
        &request.recommendation_result_id,
    )
    .await?;
    if reference.placement_code != request.placement_code {
        return Err("recommendation placement_code does not match stored result".to_string());
    }

    let mut behavior_event_ids = Vec::with_capacity(request.items.len() + 1);
    let mut outbox_enqueued_count = 0u64;

    let panel_attrs = json!({
        "idempotency_key": idempotency_key,
        "actor_role": actor_role,
        "placement_code": request.placement_code,
        "event_scope": "panel"
    });
    let panel_event_id = insert_behavior_event(
        &tx,
        &reference,
        "recommendation_panel_viewed",
        &request.placement_code,
        "mixed",
        None,
        request_id,
        trace_id,
        &panel_attrs,
    )
    .await?;
    outbox_enqueued_count += enqueue_behavior_outbox(
        &tx,
        &panel_event_id,
        &reference,
        "recommendation_panel_viewed",
        "mixed",
        None,
        request_id,
        trace_id,
        idempotency_key,
        &panel_attrs,
    )
    .await? as u64;
    behavior_event_ids.push(panel_event_id.clone());

    for item in &request.items {
        let item_reference = load_result_item_reference(
            &tx,
            &request.recommendation_result_id,
            item.recommendation_result_item_id.as_deref(),
            &item.entity_scope,
            &item.entity_id,
        )
        .await?;
        let attrs = json!({
            "idempotency_key": idempotency_key,
            "actor_role": actor_role,
            "placement_code": request.placement_code,
            "recommendation_result_item_id": item_reference.recommendation_result_item_id,
            "position_no": item_reference.position_no
        });
        let event_id = insert_behavior_event(
            &tx,
            &reference,
            "recommendation_item_exposed",
            &request.placement_code,
            &item_reference.entity_scope,
            Some(item_reference.entity_id.as_str()),
            request_id,
            trace_id,
            &attrs,
        )
        .await?;
        outbox_enqueued_count += enqueue_behavior_outbox(
            &tx,
            &event_id,
            &reference,
            "recommendation_item_exposed",
            &item_reference.entity_scope,
            Some(item_reference.entity_id.as_str()),
            request_id,
            trace_id,
            idempotency_key,
            &attrs,
        )
        .await? as u64;
        behavior_event_ids.push(event_id);
    }

    tx.commit()
        .await
        .map_err(|err| format!("commit recommendation exposure transaction failed: {err}"))?;
    remember_seen_entities(
        query_subject_cache_ref(&reference),
        &request.placement_code,
        &request.items,
    )
    .await?;
    Ok(BehaviorTrackResponse {
        accepted_count: behavior_event_ids.len() as u64,
        deduplicated_count: 0,
        behavior_event_ids,
        outbox_enqueued_count,
    })
}

pub async fn record_click(
    client: &Client,
    request: &TrackClickRequest,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: &str,
    actor_role: &str,
) -> RepoResult<BehaviorTrackResponse> {
    let tx = client
        .transaction()
        .await
        .map_err(|err| format!("open recommendation click transaction failed: {err}"))?;
    let existing =
        existing_behavior_events(&tx, &["recommendation_item_clicked"], idempotency_key).await?;
    if !existing.is_empty() {
        tx.rollback()
            .await
            .map_err(|err| format!("rollback duplicate click transaction failed: {err}"))?;
        return Ok(BehaviorTrackResponse {
            accepted_count: 0,
            deduplicated_count: existing.len() as u64,
            behavior_event_ids: existing,
            outbox_enqueued_count: 0,
        });
    }

    let reference = load_recommendation_reference(
        &tx,
        &request.recommendation_request_id,
        &request.recommendation_result_id,
    )
    .await?;
    let item_reference = load_result_item_reference(
        &tx,
        &request.recommendation_result_id,
        Some(request.recommendation_result_item_id.as_str()),
        &request.entity_scope,
        &request.entity_id,
    )
    .await?;
    let event_type = if item_reference.entity_scope == "seller" {
        "seller_recommendation_clicked"
    } else {
        "recommendation_item_clicked"
    };
    let attrs = json!({
        "idempotency_key": idempotency_key,
        "actor_role": actor_role,
        "placement_code": reference.placement_code,
        "recommendation_result_item_id": item_reference.recommendation_result_item_id,
        "position_no": item_reference.position_no
    });
    let event_id = insert_behavior_event(
        &tx,
        &reference,
        event_type,
        &reference.placement_code,
        &item_reference.entity_scope,
        Some(item_reference.entity_id.as_str()),
        request_id,
        trace_id,
        &attrs,
    )
    .await?;
    let outbox_enqueued = enqueue_behavior_outbox(
        &tx,
        &event_id,
        &reference,
        event_type,
        &item_reference.entity_scope,
        Some(item_reference.entity_id.as_str()),
        request_id,
        trace_id,
        idempotency_key,
        &attrs,
    )
    .await? as u64;
    tx.commit()
        .await
        .map_err(|err| format!("commit recommendation click transaction failed: {err}"))?;

    Ok(BehaviorTrackResponse {
        accepted_count: 1,
        deduplicated_count: 0,
        behavior_event_ids: vec![event_id],
        outbox_enqueued_count: outbox_enqueued,
    })
}

pub async fn list_placements(
    client: &(impl GenericClient + Sync),
) -> RepoResult<Vec<PlacementView>> {
    let rows = client
        .query(
            "SELECT
               placement_code,
               placement_name,
               placement_scope,
               page_context,
               candidate_policy_json,
               filter_policy_json,
               default_ranking_profile_key,
               status,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM recommend.placement_definition
             ORDER BY placement_code",
            &[],
        )
        .await
        .map_err(|err| format!("list recommendation placements failed: {err}"))?;
    Ok(rows
        .into_iter()
        .map(|row| PlacementView {
            placement_code: row.get(0),
            placement_name: row.get(1),
            placement_scope: row.get(2),
            page_context: row.get(3),
            candidate_policy_json: row.get(4),
            filter_policy_json: row.get(5),
            default_ranking_profile_key: row.get(6),
            status: row.get(7),
            metadata: row.get(8),
            created_at: row.get(9),
            updated_at: row.get(10),
        })
        .collect())
}

pub async fn patch_placement(
    client: &(impl GenericClient + Sync),
    placement_code: &str,
    request: &PatchPlacementRequest,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    actor_role: &str,
) -> RepoResult<PlacementView> {
    let normalized_profile_key = request
        .default_ranking_profile_key
        .as_ref()
        .map(|value| value.trim().to_string());
    let normalized_status = request
        .status
        .as_ref()
        .map(|value| value.trim().to_string());
    if let Some(profile_key) = normalized_profile_key.as_deref() {
        let exists = client
            .query_opt(
                "SELECT 1
                 FROM recommend.ranking_profile
                 WHERE profile_key = $1
                   AND status = 'active'",
                &[&profile_key],
            )
            .await
            .map_err(|err| format!("validate recommendation ranking profile failed: {err}"))?;
        if exists.is_none() {
            return Err(format!(
                "recommendation ranking profile missing: profile_key={profile_key}"
            ));
        }
    }
    let row = client
        .query_opt(
            "UPDATE recommend.placement_definition
             SET candidate_policy_json = COALESCE($2::jsonb, candidate_policy_json),
                 filter_policy_json = COALESCE($3::jsonb, filter_policy_json),
                 default_ranking_profile_key = COALESCE($4::text, default_ranking_profile_key),
                 status = COALESCE($5::text, status),
                 metadata = COALESCE(metadata, '{}'::jsonb)
                   || COALESCE($6::jsonb, '{}'::jsonb)
                   || jsonb_build_object(
                   'last_request_id', $7::text,
                   'last_trace_id', $8::text,
                   'last_actor_role', $9::text
                 ),
                 updated_at = now()
             WHERE placement_code = $1
             RETURNING
               placement_code,
               placement_name,
               placement_scope,
               page_context,
               candidate_policy_json,
               filter_policy_json,
               default_ranking_profile_key,
               status,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &placement_code,
                &request.candidate_policy_json,
                &request.filter_policy_json,
                &normalized_profile_key,
                &normalized_status,
                &request.metadata,
                &request_id,
                &trace_id,
                &actor_role,
            ],
        )
        .await
        .map_err(|err| format!("patch recommendation placement failed: {err}"))?;
    let Some(row) = row else {
        return Err(format!(
            "recommendation placement missing: placement_code={placement_code}"
        ));
    };
    Ok(PlacementView {
        placement_code: row.get(0),
        placement_name: row.get(1),
        placement_scope: row.get(2),
        page_context: row.get(3),
        candidate_policy_json: row.get(4),
        filter_policy_json: row.get(5),
        default_ranking_profile_key: row.get(6),
        status: row.get(7),
        metadata: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    })
}

pub async fn list_ranking_profiles(
    client: &(impl GenericClient + Sync),
) -> RepoResult<Vec<RecommendationRankingProfileView>> {
    let rows = client
        .query(
            "SELECT
               recommendation_ranking_profile_id::text,
               profile_key,
               placement_scope,
               backend_type,
               weights_json,
               diversity_policy_json,
               exploration_policy_json,
               explain_codes,
               status,
               stage_from,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM recommend.ranking_profile
             ORDER BY profile_key",
            &[],
        )
        .await
        .map_err(|err| format!("list recommendation ranking profiles failed: {err}"))?;
    Ok(rows
        .into_iter()
        .map(|row| RecommendationRankingProfileView {
            recommendation_ranking_profile_id: row.get(0),
            profile_key: row.get(1),
            placement_scope: row.get(2),
            backend_type: row.get(3),
            weights_json: row.get(4),
            diversity_policy_json: row.get(5),
            exploration_policy_json: row.get(6),
            explain_codes: row.get(7),
            status: row.get(8),
            stage_from: row.get(9),
            metadata: row.get(10),
            created_at: row.get(11),
            updated_at: row.get(12),
        })
        .collect())
}

pub async fn patch_ranking_profile(
    client: &(impl GenericClient + Sync),
    profile_id: &str,
    request: &PatchRecommendationRankingProfileRequest,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    actor_role: &str,
) -> RepoResult<RecommendationRankingProfileView> {
    let row = client
        .query_opt(
            "UPDATE recommend.ranking_profile
             SET weights_json = COALESCE($2::jsonb, weights_json),
                 diversity_policy_json = COALESCE($3::jsonb, diversity_policy_json),
                 exploration_policy_json = COALESCE($4::jsonb, exploration_policy_json),
                 explain_codes = COALESCE($5::text[], explain_codes),
                 status = COALESCE($6::text, status),
                 metadata = COALESCE(metadata, '{}'::jsonb)
                   || COALESCE($7::jsonb, '{}'::jsonb)
                   || jsonb_build_object(
                   'last_request_id', $8::text,
                   'last_trace_id', $9::text,
                   'last_actor_role', $10::text
                 ),
                 updated_at = now()
             WHERE recommendation_ranking_profile_id = $1::text::uuid
             RETURNING
               recommendation_ranking_profile_id::text,
               profile_key,
               placement_scope,
               backend_type,
               weights_json,
               diversity_policy_json,
               exploration_policy_json,
               explain_codes,
               status,
               stage_from,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &profile_id,
                &request.weights_json,
                &request.diversity_policy_json,
                &request.exploration_policy_json,
                &request.explain_codes,
                &request.status,
                &request.metadata,
                &request_id,
                &trace_id,
                &actor_role,
            ],
        )
        .await
        .map_err(|err| format!("patch recommendation ranking profile failed: {err}"))?;
    let Some(row) = row else {
        return Err(format!(
            "recommendation ranking profile missing: recommendation_ranking_profile_id={profile_id}"
        ));
    };
    Ok(RecommendationRankingProfileView {
        recommendation_ranking_profile_id: row.get(0),
        profile_key: row.get(1),
        placement_scope: row.get(2),
        backend_type: row.get(3),
        weights_json: row.get(4),
        diversity_policy_json: row.get(5),
        exploration_policy_json: row.get(6),
        explain_codes: row.get(7),
        status: row.get(8),
        stage_from: row.get(9),
        metadata: row.get(10),
        created_at: row.get(11),
        updated_at: row.get(12),
    })
}

pub async fn rebuild_runtime(
    client: &Client,
    request: &RecommendationRebuildRequest,
    _request_id: Option<&str>,
    _trace_id: Option<&str>,
    _actor_role: &str,
) -> RepoResult<RecommendationRebuildResponse> {
    let scope = request.scope.trim().to_ascii_lowercase();
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
        return Err(format!(
            "unsupported recommendation rebuild scope: {}",
            request.scope
        ));
    }

    let cache_keys_deleted =
        if request.purge_cache.unwrap_or(true) || scope == "cache" || scope == "all" {
            invalidate_recommendation_cache(request).await?
        } else {
            0
        };

    let mut refreshed_subject_profiles = 0;
    let mut refreshed_cohort_rows = 0;
    let mut refreshed_signal_rows = 0;
    let mut refreshed_similarity_rows = 0;
    let mut refreshed_bundle_rows = 0;
    if matches!(scope.as_str(), "all" | "features" | "subject_profile") {
        refreshed_subject_profiles = rebuild_subject_profiles(client, request).await?;
    }
    if matches!(scope.as_str(), "all" | "features" | "cohort") {
        refreshed_cohort_rows = rebuild_cohort_popularity(client, request).await?;
    }
    if matches!(scope.as_str(), "all" | "features" | "signals") {
        refreshed_signal_rows = rebuild_search_signal_aggregate(client, request).await?;
    }
    if matches!(scope.as_str(), "all" | "features" | "similarity") {
        refreshed_similarity_rows = rebuild_similarity_edges(client, request).await?;
    }
    if matches!(scope.as_str(), "all" | "features" | "bundle") {
        refreshed_bundle_rows = rebuild_bundle_relations(client, request).await?;
    }

    Ok(RecommendationRebuildResponse {
        scope,
        cache_keys_deleted,
        refreshed_subject_profiles,
        refreshed_cohort_rows,
        refreshed_signal_rows,
        refreshed_similarity_rows,
        refreshed_bundle_rows,
    })
}

async fn load_active_placement(
    client: &(impl GenericClient + Sync),
    placement_code: &str,
) -> RepoResult<PlacementDefinition> {
    let row = client
        .query_opt(
            "SELECT
               placement_code,
               placement_name,
               placement_scope,
               page_context,
               candidate_policy_json,
               filter_policy_json,
               default_ranking_profile_key,
               status,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM recommend.placement_definition
             WHERE placement_code = $1
               AND status = 'active'",
            &[&placement_code],
        )
        .await
        .map_err(|err| format!("load recommendation placement failed: {err}"))?;
    let Some(row) = row else {
        return Err(format!(
            "active recommendation placement missing: placement_code={placement_code}"
        ));
    };
    Ok(PlacementDefinition {
        placement_code: row.get(0),
        placement_name: row.get(1),
        placement_scope: row.get(2),
        page_context: row.get(3),
        candidate_policy_json: row.get(4),
        filter_policy_json: row.get(5),
        default_ranking_profile_key: row.get(6),
        status: row.get(7),
        metadata: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    })
}

async fn load_effective_ranking_profile(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
) -> RepoResult<RankingProfile> {
    let profile_key = placement
        .default_ranking_profile_key
        .as_deref()
        .unwrap_or("recommend_v1_default");
    let row = client
        .query_opt(
            "SELECT
               recommendation_ranking_profile_id::text,
               profile_key,
               placement_scope,
               backend_type,
               weights_json,
               diversity_policy_json,
               exploration_policy_json,
               explain_codes,
               status,
               stage_from,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM recommend.ranking_profile
             WHERE profile_key = $1
               AND status = 'active'
             LIMIT 1",
            &[&profile_key],
        )
        .await
        .map_err(|err| format!("load recommendation ranking profile failed: {err}"))?;
    let Some(row) = row else {
        return Err(format!(
            "active recommendation ranking profile missing: profile_key={profile_key}"
        ));
    };
    Ok(RankingProfile {
        recommendation_ranking_profile_id: row.get(0),
        profile_key: row.get(1),
        placement_scope: row.get(2),
        backend_type: row.get(3),
        weights_json: row.get(4),
        diversity_policy_json: row.get(5),
        exploration_policy_json: row.get(6),
        explain_codes: row.get(7),
        status: row.get(8),
        stage_from: row.get(9),
        metadata: row.get(10),
        created_at: row.get(11),
        updated_at: row.get(12),
    })
}

async fn generate_candidate_snapshot(
    client: &(impl GenericClient + Sync),
    runtime_mode: &RuntimeMode,
    query: &RecommendationQuery,
    placement: &PlacementDefinition,
    ranking_profile: &RankingProfile,
    limit: u32,
) -> RepoResult<CandidateSnapshot> {
    if matches!(
        candidate_backend_for_runtime(runtime_mode),
        CandidateBackend::PostgresqlLocalMinimal
    ) {
        return generate_local_candidate_snapshot(client, query, placement, ranking_profile, limit)
            .await;
    }

    let recall_sources = parse_recall_sources(&placement.candidate_policy_json);
    let context = load_context_entity(client, query).await?;
    let subject_profile = load_subject_profile(client, query).await?;
    let mut merged: HashMap<String, RecallCandidate> = HashMap::new();
    let fetch_limit = (limit.max(6) * 4) as usize;

    merge_recall_candidates(&mut merged, fixed_sample_candidates(placement));

    for recall_source in recall_sources {
        let candidates = match recall_source.as_str() {
            "popular" => recall_popular(placement, fetch_limit).await?,
            "new_arrival" => recall_new_arrival(placement, fetch_limit).await?,
            "trusted_seller" => recall_trusted_seller(placement, fetch_limit).await?,
            "similar" => recall_similar(client, placement, context.as_ref(), fetch_limit).await?,
            "cohort" => recall_cohort(client, query, placement, fetch_limit).await?,
            "bundle" => recall_bundle(client, placement, context.as_ref(), fetch_limit).await?,
            "seller_related" => {
                recall_seller_related(client, placement, context.as_ref(), fetch_limit).await?
            }
            "seller_hot" => {
                recall_seller_hot(client, placement, context.as_ref(), fetch_limit).await?
            }
            "seller_quality" => {
                recall_seller_quality(client, placement, context.as_ref(), fetch_limit).await?
            }
            "renewal" => recall_renewal(placement, subject_profile.as_ref(), fetch_limit).await?,
            _ => Vec::new(),
        };
        merge_recall_candidates(&mut merged, candidates);
    }

    if merged.is_empty() {
        merge_recall_candidates(&mut merged, recall_popular(placement, fetch_limit).await?);
        merge_recall_candidates(
            &mut merged,
            recall_trusted_seller(placement, fetch_limit).await?,
        );
    }

    let mut candidates = merged
        .into_values()
        .map(|candidate| CandidateSeed {
            entity_scope: candidate.entity_scope,
            entity_id: candidate.entity_id,
            raw_score: candidate.raw_score,
            recall_sources: candidate.recall_sources.into_iter().collect(),
            explanation_codes: candidate.explanation_codes.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .raw_score
            .partial_cmp(&left.raw_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(fetch_limit);
    Ok(CandidateSnapshot {
        strategy_version: strategy_version(ranking_profile),
        candidates,
    })
}

async fn generate_fallback_snapshot(
    client: &(impl GenericClient + Sync),
    runtime_mode: &RuntimeMode,
    query: &RecommendationQuery,
    placement: &PlacementDefinition,
    ranking_profile: &RankingProfile,
    limit: u32,
) -> RepoResult<CandidateSnapshot> {
    if matches!(
        candidate_backend_for_runtime(runtime_mode),
        CandidateBackend::PostgresqlLocalMinimal
    ) {
        return generate_local_zero_result_snapshot(
            client,
            query,
            placement,
            ranking_profile,
            limit,
        )
        .await;
    }

    let mut merged = HashMap::new();
    merge_recall_candidates(
        &mut merged,
        recall_popular(placement, (limit * 4) as usize).await?,
    );
    if let Some(context) = load_context_entity(client, query).await? {
        merge_recall_candidates(
            &mut merged,
            recall_similar(client, placement, Some(&context), (limit * 4) as usize).await?,
        );
    }
    let mut candidates = merged
        .into_values()
        .map(|candidate| CandidateSeed {
            entity_scope: candidate.entity_scope,
            entity_id: candidate.entity_id,
            raw_score: candidate.raw_score,
            recall_sources: candidate.recall_sources.into_iter().collect(),
            explanation_codes: candidate.explanation_codes.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .raw_score
            .partial_cmp(&left.raw_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate((limit * 4) as usize);
    Ok(CandidateSnapshot {
        strategy_version: strategy_version(ranking_profile),
        candidates,
    })
}

fn candidate_backend_for_runtime(runtime_mode: &RuntimeMode) -> CandidateBackend {
    match runtime_mode {
        RuntimeMode::Local => CandidateBackend::PostgresqlLocalMinimal,
        RuntimeMode::Staging | RuntimeMode::Demo => CandidateBackend::OpenSearch,
    }
}

fn fixed_sample_candidates(placement: &PlacementDefinition) -> Vec<RecallCandidate> {
    parse_fixed_samples(&placement.metadata)
        .into_iter()
        .filter(|sample| placement_allows_scope(placement, &sample.entity_scope))
        .map(|sample| {
            let mut candidate = recall_candidate(
                sample.entity_scope,
                sample.entity_id,
                100.0 - sample.sample_order as f64,
                "placement_sample",
            );
            candidate
                .explanation_codes
                .insert("placement:fixed_sample".to_string());
            if let Some(scenario_code) = sample.scenario_code {
                candidate
                    .explanation_codes
                    .insert(format!("scenario:{scenario_code}"));
            }
            candidate
        })
        .collect()
}

fn parse_fixed_samples(metadata: &Value) -> Vec<PlacementFixedSample> {
    let mut samples = metadata
        .get("fixed_samples")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    let entity_scope = item.get("entity_scope")?.as_str()?.trim();
                    let entity_id = item.get("entity_id")?.as_str()?.trim();
                    if entity_scope.is_empty()
                        || entity_id.is_empty()
                        || !matches!(entity_scope, "product" | "seller")
                        || Uuid::parse_str(entity_id).is_err()
                    {
                        return None;
                    }
                    Some(PlacementFixedSample {
                        sample_order: item
                            .get("sample_order")
                            .and_then(Value::as_u64)
                            .map(|value| value as usize)
                            .unwrap_or(index),
                        entity_scope: entity_scope.to_string(),
                        entity_id: entity_id.to_string(),
                        scenario_code: item
                            .get("scenario_code")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(ToString::to_string),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    samples.sort_by_key(|sample| sample.sample_order);
    samples
}

async fn generate_local_candidate_snapshot(
    client: &(impl GenericClient + Sync),
    query: &RecommendationQuery,
    placement: &PlacementDefinition,
    ranking_profile: &RankingProfile,
    limit: u32,
) -> RepoResult<CandidateSnapshot> {
    let context = load_context_entity(client, query).await?;
    let mut merged: HashMap<String, RecallCandidate> = HashMap::new();
    let fetch_limit = (limit.max(6) * 4) as usize;

    merge_recall_candidates(&mut merged, fixed_sample_candidates(placement));

    if let Some(context) = context.as_ref() {
        merge_recall_candidates(
            &mut merged,
            recall_same_category_projection(client, placement, context, fetch_limit).await?,
        );
        merge_recall_candidates(
            &mut merged,
            recall_same_seller_projection(client, placement, context, fetch_limit).await?,
        );
    }
    merge_recall_candidates(
        &mut merged,
        recall_new_arrival_projection(client, placement, fetch_limit).await?,
    );
    merge_recall_candidates(
        &mut merged,
        recall_popular_projection(client, placement, fetch_limit).await?,
    );

    if placement.placement_code == "search_zero_result_fallback" {
        merge_recall_candidates(
            &mut merged,
            recall_zero_result_projection(client, placement, context.as_ref(), fetch_limit).await?,
        );
    }

    if merged.is_empty() {
        merge_recall_candidates(
            &mut merged,
            recall_zero_result_projection(client, placement, context.as_ref(), fetch_limit).await?,
        );
    }

    candidate_snapshot_from_merged(merged, fetch_limit, ranking_profile)
}

async fn generate_local_zero_result_snapshot(
    client: &(impl GenericClient + Sync),
    query: &RecommendationQuery,
    placement: &PlacementDefinition,
    ranking_profile: &RankingProfile,
    limit: u32,
) -> RepoResult<CandidateSnapshot> {
    let context = load_context_entity(client, query).await?;
    let fetch_limit = (limit.max(6) * 4) as usize;
    let mut merged = HashMap::new();

    merge_recall_candidates(&mut merged, fixed_sample_candidates(placement));
    merge_recall_candidates(
        &mut merged,
        recall_zero_result_projection(client, placement, context.as_ref(), fetch_limit).await?,
    );

    candidate_snapshot_from_merged(merged, fetch_limit, ranking_profile)
}

fn candidate_snapshot_from_merged(
    merged: HashMap<String, RecallCandidate>,
    fetch_limit: usize,
    ranking_profile: &RankingProfile,
) -> RepoResult<CandidateSnapshot> {
    let mut candidates = merged
        .into_values()
        .map(|candidate| CandidateSeed {
            entity_scope: candidate.entity_scope,
            entity_id: candidate.entity_id,
            raw_score: candidate.raw_score,
            recall_sources: candidate.recall_sources.into_iter().collect(),
            explanation_codes: candidate.explanation_codes.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .raw_score
            .partial_cmp(&left.raw_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(fetch_limit);
    Ok(CandidateSnapshot {
        strategy_version: strategy_version(ranking_profile),
        candidates,
    })
}

async fn hydrate_candidates(
    client: &(impl GenericClient + Sync),
    snapshot: &CandidateSnapshot,
    seen_entities: &HashSet<String>,
    limit: u32,
) -> RepoResult<Vec<RankedCandidate>> {
    let mut candidate_lookup = HashMap::new();
    let mut search_candidates = Vec::new();
    for seed in &snapshot.candidates {
        let key = entity_key(&seed.entity_scope, &seed.entity_id);
        if seen_entities.contains(&key) && search_candidates.len() >= limit as usize {
            continue;
        }
        candidate_lookup.insert(key.clone(), seed.clone());
        search_candidates.push(SearchCandidate {
            entity_scope: seed.entity_scope.clone(),
            entity_id: seed.entity_id.clone(),
            score: seed.raw_score,
            sort_value: None,
            updated_at: None,
        });
        if search_candidates.len() >= (limit as usize * 3).max(10) {
            break;
        }
    }
    let hydrated = hydrate_search_results(client, &search_candidates).await?;
    let mut ranked = Vec::new();
    for item in hydrated {
        let key = entity_key(&item.entity_scope, &item.entity_id);
        let Some(seed) = candidate_lookup.get(&key) else {
            continue;
        };
        let final_score = compute_final_score(seed, &item, seen_entities.contains(&key));
        let mut explanation_codes = BTreeSet::new();
        for code in &seed.explanation_codes {
            explanation_codes.insert(code.clone());
        }
        if parse_score(item.quality_score.as_deref()) > 0.0 {
            explanation_codes.insert("rank:quality".to_string());
        }
        if parse_score(item.reputation_score.as_deref()) > 0.0 {
            explanation_codes.insert("rank:reputation".to_string());
        }
        if parse_hotness(&item) > 0.0 {
            explanation_codes.insert("rank:hotness".to_string());
        }
        if seed
            .recall_sources
            .iter()
            .any(|source| source == "new_arrival")
        {
            explanation_codes.insert("rank:freshness".to_string());
        }
        ranked.push(RankedCandidate {
            entity_scope: item.entity_scope.clone(),
            entity_id: item.entity_id.clone(),
            raw_score: seed.raw_score,
            final_score,
            recall_sources: seed.recall_sources.clone(),
            explanation_codes: explanation_codes.into_iter().collect(),
            search_item: item,
        });
    }
    ranked.sort_by(|left, right| {
        right
            .final_score
            .partial_cmp(&left.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .raw_score
                    .partial_cmp(&left.raw_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    Ok(ranked)
}

async fn persist_recommendation_result(
    client: &(impl GenericClient + Sync),
    query: &RecommendationQuery,
    placement: &PlacementDefinition,
    ranking_profile: &RankingProfile,
    strategy_version: &str,
    candidate_backend: CandidateBackend,
    runtime_mode: &RuntimeMode,
    ranked: &[RankedCandidate],
    cache_hit: bool,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    actor_role: &str,
) -> RepoResult<RecommendationResponse> {
    let requested_count = recommendation_limit(query.limit) as i32;
    let subject_scope = normalized_subject_scope(query);
    let subject_ref = subject_ref(query);
    let candidate_source_summary = summarize_candidate_sources(ranked);
    let request_row = client
        .query_one(
            "INSERT INTO recommend.recommendation_request (
               placement_code,
               subject_scope,
               subject_org_id,
               subject_user_id,
               anonymous_session_key,
               page_context,
               context_entity_scope,
               context_entity_id,
               ranking_profile_id,
               filter_json,
               request_attrs,
               candidate_source_summary,
               trace_id,
               request_id,
               status,
               requested_count,
               served_at
             ) VALUES (
               $1,
               $2,
               $3::text::uuid,
               $4::text::uuid,
               $5,
               $6,
               $7,
               $8::text::uuid,
               $9::text::uuid,
               $10::jsonb,
               $11::jsonb,
               $12::jsonb,
               $13,
               $14,
               'served',
               $15,
               now()
             )
             RETURNING recommendation_request_id::text",
            &[
                &placement.placement_code,
                &subject_scope,
                &query.subject_org_id,
                &query.subject_user_id,
                &query.anonymous_session_key,
                &placement.page_context,
                &query.context_entity_scope,
                &query.context_entity_id,
                &ranking_profile.recommendation_ranking_profile_id,
                &placement.filter_policy_json,
                &json!({
                    "placement_name": placement.placement_name,
                    "actor_role": actor_role,
                    "cache_hit": cache_hit,
                    "candidate_backend": candidate_backend.as_str(),
                    "runtime_mode": runtime_mode.as_str()
                }),
                &candidate_source_summary,
                &trace_id,
                &request_id,
                &requested_count,
            ],
        )
        .await
        .map_err(|err| format!("insert recommendation request failed: {err}"))?;
    let recommendation_request_id: String = request_row.get(0);

    let result_row = client
        .query_one(
            "INSERT INTO recommend.recommendation_result (
               recommendation_request_id,
               placement_code,
               ranking_profile_id,
               ranking_profile_version,
               subject_scope,
               subject_ref,
               entity_scope,
               result_status,
               total_candidates,
               returned_count,
               explain_level,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3::text::uuid,
               $4,
               $5,
               $6,
               $7,
               'served',
               $8,
               $9,
               'basic',
               $10::jsonb
             )
             RETURNING recommendation_result_id::text",
            &[
                &recommendation_request_id,
                &placement.placement_code,
                &ranking_profile.recommendation_ranking_profile_id,
                &strategy_version,
                &subject_scope,
                &subject_ref,
                &placement.placement_scope,
                &(ranked.len() as i32),
                &(ranked.len() as i32),
                &json!({
                    "cache_hit": cache_hit,
                    "candidate_sources": candidate_source_summary,
                    "actor_role": actor_role,
                    "candidate_backend": candidate_backend.as_str(),
                    "runtime_mode": runtime_mode.as_str()
                }),
            ],
        )
        .await
        .map_err(|err| format!("insert recommendation result failed: {err}"))?;
    let recommendation_result_id: String = result_row.get(0);

    let mut response_items = Vec::with_capacity(ranked.len());
    for (index, item) in ranked.iter().enumerate() {
        let item_row = client
            .query_one(
                "INSERT INTO recommend.recommendation_result_item (
                   recommendation_result_id,
                   position_no,
                   entity_scope,
                   entity_id,
                   recall_sources,
                   raw_score,
                   final_score,
                   explanation_codes,
                   feature_snapshot,
                   click_status,
                   conversion_status
                 ) VALUES (
                   $1::text::uuid,
                   $2,
                   $3,
                   $4::text::uuid,
                   $5::text[],
                   $6,
                   $7,
                   $8::text[],
                   $9::jsonb,
                   'not_clicked',
                   'none'
                 )
                 RETURNING recommendation_result_item_id::text",
                &[
                    &recommendation_result_id,
                    &((index + 1) as i32),
                    &item.entity_scope,
                    &item.entity_id,
                    &item.recall_sources,
                    &item.raw_score,
                    &item.final_score,
                    &item.explanation_codes,
                    &json!({
                        "strategy_version": strategy_version,
                        "raw_score": item.raw_score,
                        "final_score": item.final_score,
                        "candidate_backend": candidate_backend.as_str(),
                        "runtime_mode": runtime_mode.as_str(),
                        "quality_score": item.search_item.quality_score,
                        "reputation_score": item.search_item.reputation_score,
                        "hotness_score": item.search_item.hotness_score,
                        "listing_product_count": item.search_item.listing_product_count,
                        "recall_sources": item.recall_sources
                    }),
                ],
            )
            .await
            .map_err(|err| format!("insert recommendation result item failed: {err}"))?;
        let recommendation_result_item_id: String = item_row.get(0);
        response_items.push(RecommendationItem {
            recommendation_result_item_id,
            entity_scope: item.entity_scope.clone(),
            entity_id: item.entity_id.clone(),
            title: item.search_item.title.clone(),
            seller_name: item.search_item.seller_name.clone(),
            price: item.search_item.price.clone(),
            currency_code: item.search_item.currency_code.clone(),
            final_score: item.final_score,
            explanation_codes: item.explanation_codes.clone(),
            recall_sources: item.recall_sources.clone(),
            status: item.search_item.status.clone(),
        });
    }

    Ok(RecommendationResponse {
        recommendation_request_id,
        recommendation_result_id,
        placement_code: placement.placement_code.clone(),
        strategy_version: strategy_version.to_string(),
        cache_hit,
        items: response_items,
    })
}

async fn load_context_entity(
    client: &(impl GenericClient + Sync),
    query: &RecommendationQuery,
) -> RepoResult<Option<ContextEntity>> {
    let Some(entity_scope) = query.context_entity_scope.as_deref() else {
        return Ok(None);
    };
    let Some(entity_id) = query.context_entity_id.as_deref() else {
        return Ok(None);
    };

    if entity_scope.eq_ignore_ascii_case("seller") {
        let row = client
            .query_opt(
                "SELECT
                   org_id::text,
                   NULL::text,
                   NULL::text,
                   COALESCE(industry_tags, '{}')::text[]
                 FROM search.seller_search_document
                 WHERE org_id = $1::text::uuid",
                &[&entity_id],
            )
            .await
            .map_err(|err| format!("load recommendation seller context failed: {err}"))?;
        return Ok(row.map(|row| ContextEntity {
            entity_scope: "seller".to_string(),
            entity_id: entity_id.to_string(),
            seller_org_id: Some(row.get(0)),
            category: row.get(1),
            industry: row.get(2),
            tags: row.get(3),
            product_type: None,
        }));
    }

    let row = client
        .query_opt(
            "SELECT
               product_id::text,
               org_id::text,
               category,
               industry,
               COALESCE(tags, '{}')::text[],
               product_type
             FROM search.product_search_document
             WHERE product_id = $1::text::uuid",
            &[&entity_id],
        )
        .await
        .map_err(|err| format!("load recommendation product context failed: {err}"))?;
    Ok(row.map(|row| ContextEntity {
        entity_scope: "product".to_string(),
        entity_id: row.get(0),
        seller_org_id: row.get(1),
        category: row.get(2),
        industry: row.get(3),
        tags: row.get(4),
        product_type: row.get(5),
    }))
}

async fn load_subject_profile(
    client: &(impl GenericClient + Sync),
    query: &RecommendationQuery,
) -> RepoResult<Option<SubjectProfile>> {
    let scope = normalized_subject_scope(query);
    let reference = subject_ref(query);
    if reference == "anonymous" {
        return Ok(None);
    }
    let row = client
        .query_opt(
            "SELECT
               COALESCE(preferred_categories, '{}')::text[],
               COALESCE(preferred_tags, '{}')::text[],
               COALESCE(preferred_delivery_modes, '{}')::text[]
             FROM recommend.subject_profile_snapshot
             WHERE subject_scope = $1
               AND subject_ref = $2",
            &[&scope, &reference],
        )
        .await
        .map_err(|err| format!("load recommendation subject profile failed: {err}"))?;
    Ok(row.map(|row| SubjectProfile {
        preferred_categories: row.get(0),
        preferred_tags: row.get(1),
        preferred_delivery_modes: row.get(2),
    }))
}

#[derive(Debug, Clone, Copy)]
enum ProjectionProductSort {
    Latest,
    Hotness,
    Quality,
}

#[derive(Debug, Clone, Copy)]
struct ProjectionProductFilter<'a> {
    category: Option<&'a str>,
    seller_org_id: Option<&'a str>,
    exclude_product_id: Option<&'a str>,
}

async fn recall_same_category_projection(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    context: &ContextEntity,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    if !placement_supports_products(&placement.placement_scope) {
        return Ok(Vec::new());
    }
    let Some(category) = context.category.as_deref() else {
        return Ok(Vec::new());
    };
    let candidates = fetch_projection_product_candidates(
        client,
        placement,
        ProjectionProductFilter {
            category: Some(category),
            seller_org_id: None,
            exclude_product_id: Some(context.entity_id.as_str()),
        },
        ProjectionProductSort::Quality,
        limit,
        0.9,
        "similar",
    )
    .await?;
    Ok(annotate_candidates(
        candidates,
        &["local:minimal", "local:same_category"],
    ))
}

async fn recall_same_seller_projection(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    context: &ContextEntity,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    if !placement_supports_products(&placement.placement_scope) {
        return Ok(Vec::new());
    }
    let Some(seller_org_id) = context.seller_org_id.as_deref() else {
        return Ok(Vec::new());
    };
    let candidates = fetch_projection_product_candidates(
        client,
        placement,
        ProjectionProductFilter {
            category: None,
            seller_org_id: Some(seller_org_id),
            exclude_product_id: Some(context.entity_id.as_str()),
        },
        ProjectionProductSort::Latest,
        limit,
        0.88,
        "seller_related",
    )
    .await?;
    Ok(annotate_candidates(
        candidates,
        &["local:minimal", "local:same_seller"],
    ))
}

async fn recall_new_arrival_projection(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    if !placement_supports_products(&placement.placement_scope) {
        return Ok(Vec::new());
    }
    let candidates = fetch_projection_product_candidates(
        client,
        placement,
        ProjectionProductFilter {
            category: None,
            seller_org_id: None,
            exclude_product_id: None,
        },
        ProjectionProductSort::Latest,
        limit,
        0.76,
        "new_arrival",
    )
    .await?;
    Ok(annotate_candidates(candidates, &["local:minimal"]))
}

async fn recall_popular_projection(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let mut candidates = Vec::new();
    for entity_scope in placement_entity_scopes(&placement.placement_scope) {
        if entity_scope == "seller" {
            candidates.extend(annotate_candidates(
                fetch_projection_seller_candidates(client, limit, 0.75, "popular").await?,
                &["local:minimal"],
            ));
        } else {
            candidates.extend(annotate_candidates(
                fetch_projection_product_candidates(
                    client,
                    placement,
                    ProjectionProductFilter {
                        category: None,
                        seller_org_id: None,
                        exclude_product_id: None,
                    },
                    ProjectionProductSort::Hotness,
                    limit,
                    0.8,
                    "popular",
                )
                .await?,
                &["local:minimal"],
            ));
        }
    }
    Ok(candidates)
}

async fn recall_zero_result_projection(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    context: Option<&ContextEntity>,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let mut merged = HashMap::new();
    if let Some(context) = context {
        merge_recall_candidates(
            &mut merged,
            annotate_candidates(
                recall_same_category_projection(client, placement, context, limit).await?,
                &["fallback:zero_result"],
            ),
        );
        merge_recall_candidates(
            &mut merged,
            annotate_candidates(
                recall_same_seller_projection(client, placement, context, limit).await?,
                &["fallback:zero_result"],
            ),
        );
    }
    merge_recall_candidates(
        &mut merged,
        annotate_candidates(
            recall_popular_projection(client, placement, limit).await?,
            &["fallback:zero_result"],
        ),
    );
    merge_recall_candidates(
        &mut merged,
        annotate_candidates(
            recall_new_arrival_projection(client, placement, limit).await?,
            &["fallback:zero_result"],
        ),
    );
    Ok(merged.into_values().collect())
}

async fn fetch_projection_product_candidates(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    filter: ProjectionProductFilter<'_>,
    sort: ProjectionProductSort,
    limit: usize,
    base_score: f64,
    source: &str,
) -> RepoResult<Vec<RecallCandidate>> {
    let product_scope_clause = projection_product_scope_clause(&placement.placement_scope);
    let order_clause = projection_product_order_clause(sort);
    let rows = client
        .query(
            &format!(
                "SELECT
                   d.product_id::text,
                   COALESCE(d.quality_score::float8, 0),
                   COALESCE(d.seller_reputation_score::float8, 0),
                   COALESCE(d.hotness_score::float8, 0)
                 FROM search.product_search_document d
                 WHERE d.listing_status = 'listed'
                   AND COALESCE(d.visible_to_search, false)
                   AND ($1::text IS NULL OR d.category = $1)
                   AND ($2::text IS NULL OR d.org_id = $2::text::uuid)
                   AND ($3::text IS NULL OR d.product_id <> $3::text::uuid)
                   {product_scope_clause}
                 ORDER BY {order_clause}
                 LIMIT $4"
            ),
            &[
                &filter.category,
                &filter.seller_org_id,
                &filter.exclude_product_id,
                &(limit.min(50) as i32),
            ],
        )
        .await
        .map_err(|err| format!("load local recommendation product candidates failed: {err}"))?;
    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            recall_candidate(
                "product",
                row.get::<usize, String>(0),
                projection_candidate_score(
                    base_score,
                    index,
                    row.get::<usize, f64>(1),
                    row.get::<usize, f64>(2),
                    row.get::<usize, f64>(3),
                ),
                source,
            )
        })
        .collect())
}

async fn fetch_projection_seller_candidates(
    client: &(impl GenericClient + Sync),
    limit: usize,
    base_score: f64,
    source: &str,
) -> RepoResult<Vec<RecallCandidate>> {
    let rows = client
        .query(
            "SELECT
               ssd.org_id::text,
               COALESCE(ssd.reputation_score::float8, 0),
               GREATEST(COALESCE(ssd.listing_product_count::float8, 0) / 20.0, 0)
             FROM search.seller_search_document ssd
             JOIN core.organization org ON org.org_id = ssd.org_id
             WHERE org.status NOT IN ('suspended', 'frozen')
             ORDER BY COALESCE(ssd.listing_product_count::float8, 0) DESC,
                      COALESCE(ssd.reputation_score::float8, 0) DESC,
                      ssd.source_updated_at DESC
             LIMIT $1",
            &[&(limit.min(50) as i32)],
        )
        .await
        .map_err(|err| format!("load local recommendation seller candidates failed: {err}"))?;
    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            recall_candidate(
                "seller",
                row.get::<usize, String>(0),
                projection_candidate_score(
                    base_score,
                    index,
                    0.0,
                    row.get::<usize, f64>(1),
                    row.get::<usize, f64>(2),
                ),
                source,
            )
        })
        .collect())
}

fn annotate_candidates(
    mut candidates: Vec<RecallCandidate>,
    explanation_codes: &[&str],
) -> Vec<RecallCandidate> {
    for candidate in &mut candidates {
        for code in explanation_codes {
            candidate.explanation_codes.insert((*code).to_string());
        }
    }
    candidates
}

fn projection_candidate_score(
    base_score: f64,
    index: usize,
    quality: f64,
    reputation: f64,
    hotness: f64,
) -> f64 {
    base_score
        + parse_numeric_signal(quality) * 0.08
        + parse_numeric_signal(reputation) * 0.05
        + parse_numeric_signal(hotness) * 0.07
        + score_decay(index) * 0.05
}

fn parse_numeric_signal(value: f64) -> f64 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn projection_product_scope_clause(placement_scope: &str) -> &'static str {
    match placement_scope {
        "service" => "AND d.product_type = 'service'",
        "product" => "AND COALESCE(d.product_type, '') <> 'service'",
        _ => "",
    }
}

fn projection_product_order_clause(sort: ProjectionProductSort) -> &'static str {
    match sort {
        ProjectionProductSort::Latest => {
            "source_updated_at DESC, COALESCE(hotness_score::float8, 0) DESC"
        }
        ProjectionProductSort::Hotness => {
            "COALESCE(hotness_score::float8, 0) DESC, source_updated_at DESC"
        }
        ProjectionProductSort::Quality => {
            "COALESCE(quality_score::float8, 0) DESC, COALESCE(hotness_score::float8, 0) DESC, source_updated_at DESC"
        }
    }
}

async fn recall_popular(
    placement: &PlacementDefinition,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let mut candidates = Vec::new();
    for entity_scope in placement_entity_scopes(&placement.placement_scope) {
        if entity_scope == "seller" {
            candidates.extend(
                fetch_os_candidates(
                    &seller_read_alias(),
                    "seller",
                    recall_body_for_scope(
                        "seller",
                        placement,
                        vec![],
                        vec![],
                        seller_sort("hotness"),
                        limit,
                    ),
                    0.75,
                    "popular",
                )
                .await?,
            );
        } else {
            candidates.extend(
                fetch_os_candidates(
                    &product_read_alias(),
                    "product",
                    recall_body_for_scope(
                        "product",
                        placement,
                        placement_product_filters(placement, None),
                        vec![],
                        product_sort("hotness"),
                        limit,
                    ),
                    0.8,
                    "popular",
                )
                .await?,
            );
        }
    }
    Ok(candidates)
}

async fn recall_new_arrival(
    placement: &PlacementDefinition,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    if !placement_supports_products(&placement.placement_scope) {
        return Ok(Vec::new());
    }
    fetch_os_candidates(
        &product_read_alias(),
        "product",
        recall_body_for_scope(
            "product",
            placement,
            placement_product_filters(placement, None),
            vec![],
            product_sort("latest"),
            limit,
        ),
        0.72,
        "new_arrival",
    )
    .await
}

async fn recall_trusted_seller(
    placement: &PlacementDefinition,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let mut candidates = Vec::new();
    if placement_supports_products(&placement.placement_scope) {
        candidates.extend(
            fetch_os_candidates(
                &product_read_alias(),
                "product",
                recall_body_for_scope(
                    "product",
                    placement,
                    placement_product_filters(placement, None),
                    vec![],
                    product_sort("reputation"),
                    limit,
                ),
                0.78,
                "trusted_seller",
            )
            .await?,
        );
    }
    if placement_supports_sellers(&placement.placement_scope) {
        candidates.extend(
            fetch_os_candidates(
                &seller_read_alias(),
                "seller",
                recall_body_for_scope(
                    "seller",
                    placement,
                    vec![],
                    vec![],
                    seller_sort("reputation"),
                    limit,
                ),
                0.79,
                "trusted_seller",
            )
            .await?,
        );
    }
    Ok(candidates)
}

async fn recall_similar(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    context: Option<&ContextEntity>,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let Some(context) = context else {
        return Ok(Vec::new());
    };
    let similarity_rows = client
        .query(
            "SELECT target_entity_scope, target_entity_id::text, score::double precision
             FROM recommend.entity_similarity
             WHERE source_entity_scope = $1
               AND source_entity_id = $2::text::uuid
             ORDER BY score DESC
             LIMIT $3",
            &[&context.entity_scope, &context.entity_id, &(limit as i32)],
        )
        .await
        .map_err(|err| format!("load recommendation similarity edges failed: {err}"))?;
    if !similarity_rows.is_empty() {
        return Ok(similarity_rows
            .into_iter()
            .map(|row| {
                let entity_scope: String = row.get(0);
                let entity_id: String = row.get(1);
                recall_candidate(
                    entity_scope,
                    entity_id,
                    0.88 + row.get::<usize, f64>(2).min(10.0) / 10.0,
                    "similar",
                )
            })
            .collect());
    }

    if context.entity_scope == "seller" {
        return fetch_os_candidates(
            &seller_read_alias(),
            "seller",
            recall_body_for_scope(
                "seller",
                placement,
                vec![],
                context
                    .tags
                    .iter()
                    .map(|tag| json!({ "term": { "industry_tags.keyword": tag } }))
                    .collect(),
                seller_sort("reputation"),
                limit,
            ),
            0.82,
            "similar",
        )
        .await;
    }

    let mut should = Vec::new();
    if let Some(category) = context.category.as_deref() {
        should.push(json!({ "term": { "category.keyword": category } }));
    }
    if let Some(industry) = context.industry.as_deref() {
        should.push(json!({ "term": { "industry.keyword": industry } }));
    }
    for tag in &context.tags {
        should.push(json!({ "term": { "tags.keyword": tag } }));
    }
    let mut filters = placement_product_filters(placement, Some(context.entity_id.as_str()));
    filters.push(json!({ "exists": { "field": "category" } }));
    fetch_os_candidates(
        &product_read_alias(),
        "product",
        recall_body_for_scope(
            "product",
            placement,
            filters,
            should,
            json!([
              { "_score": { "order": "desc" } },
              { "quality_score": { "order": "desc", "missing": "_last" } }
            ]),
            limit,
        ),
        0.9,
        "similar",
    )
    .await
}

async fn recall_cohort(
    client: &(impl GenericClient + Sync),
    query: &RecommendationQuery,
    placement: &PlacementDefinition,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let cohort_keys = derived_cohort_keys(query);
    let rows = client
        .query(
            "SELECT entity_scope, entity_id::text, hotness_score::double precision
             FROM recommend.cohort_popularity
             WHERE cohort_key = ANY($1::text[])
             ORDER BY hotness_score DESC, updated_at DESC
             LIMIT $2",
            &[&cohort_keys, &(limit as i32)],
        )
        .await
        .map_err(|err| format!("load recommendation cohort popularity failed: {err}"))?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let entity_scope: String = row.get(0);
            if !placement_allows_scope(placement, &entity_scope) {
                return None;
            }
            let entity_id: String = row.get(1);
            Some(recall_candidate(
                entity_scope,
                entity_id,
                0.84 + row.get::<usize, f64>(2).min(10.0) / 10.0,
                "cohort",
            ))
        })
        .collect())
}

async fn recall_bundle(
    client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    context: Option<&ContextEntity>,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let Some(context) = context else {
        return Ok(Vec::new());
    };
    let rows = client
        .query(
            "SELECT target_entity_scope, target_entity_id::text, relation_score::double precision
             FROM recommend.bundle_relation
             WHERE source_entity_scope = $1
               AND source_entity_id = $2::text::uuid
               AND status = 'active'
             ORDER BY relation_score DESC, updated_at DESC
             LIMIT $3",
            &[&context.entity_scope, &context.entity_id, &(limit as i32)],
        )
        .await
        .map_err(|err| format!("load recommendation bundle relations failed: {err}"))?;
    if !rows.is_empty() {
        return Ok(rows
            .into_iter()
            .filter_map(|row| {
                let entity_scope: String = row.get(0);
                if !placement_allows_scope(placement, &entity_scope) {
                    return None;
                }
                let entity_id: String = row.get(1);
                Some(recall_candidate(
                    entity_scope,
                    entity_id,
                    0.87 + row.get::<usize, f64>(2).min(10.0) / 10.0,
                    "bundle",
                ))
            })
            .collect());
    }

    if let Some(seller_org_id) = context.seller_org_id.as_deref() {
        return fetch_os_candidates(
            &product_read_alias(),
            "product",
            recall_body_for_scope(
                "product",
                placement,
                {
                    let mut filters =
                        placement_product_filters(placement, Some(context.entity_id.as_str()));
                    filters.push(json!({ "term": { "seller_id": seller_org_id } }));
                    filters
                },
                vec![],
                product_sort("quality"),
                limit,
            ),
            0.8,
            "bundle",
        )
        .await;
    }
    Ok(Vec::new())
}

async fn recall_seller_related(
    _client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    context: Option<&ContextEntity>,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let Some(context) = context else {
        return Ok(Vec::new());
    };
    let Some(seller_org_id) = context.seller_org_id.as_deref() else {
        return Ok(Vec::new());
    };
    fetch_os_candidates(
        &product_read_alias(),
        "product",
        recall_body_for_scope(
            "product",
            placement,
            {
                let mut filters =
                    placement_product_filters(placement, Some(context.entity_id.as_str()));
                filters.push(json!({ "term": { "seller_id": seller_org_id } }));
                filters
            },
            vec![],
            product_sort("latest"),
            limit,
        ),
        0.82,
        "seller_related",
    )
    .await
}

async fn recall_seller_hot(
    _client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    context: Option<&ContextEntity>,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    if let Some(context) = context {
        if let Some(seller_org_id) = context.seller_org_id.as_deref() {
            return fetch_os_candidates(
                &product_read_alias(),
                "product",
                recall_body_for_scope(
                    "product",
                    placement,
                    {
                        let mut filters =
                            placement_product_filters(placement, Some(context.entity_id.as_str()));
                        filters.push(json!({ "term": { "seller_id": seller_org_id } }));
                        filters
                    },
                    vec![],
                    product_sort("hotness"),
                    limit,
                ),
                0.8,
                "seller_hot",
            )
            .await;
        }
    }
    fetch_os_candidates(
        &seller_read_alias(),
        "seller",
        recall_body_for_scope(
            "seller",
            placement,
            vec![],
            vec![],
            seller_sort("hotness"),
            limit,
        ),
        0.78,
        "seller_hot",
    )
    .await
}

async fn recall_seller_quality(
    _client: &(impl GenericClient + Sync),
    placement: &PlacementDefinition,
    context: Option<&ContextEntity>,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    if let Some(context) = context {
        if let Some(seller_org_id) = context.seller_org_id.as_deref() {
            return fetch_os_candidates(
                &product_read_alias(),
                "product",
                recall_body_for_scope(
                    "product",
                    placement,
                    {
                        let mut filters =
                            placement_product_filters(placement, Some(context.entity_id.as_str()));
                        filters.push(json!({ "term": { "seller_id": seller_org_id } }));
                        filters
                    },
                    vec![],
                    product_sort("quality"),
                    limit,
                ),
                0.79,
                "seller_quality",
            )
            .await;
        }
    }
    fetch_os_candidates(
        &seller_read_alias(),
        "seller",
        recall_body_for_scope(
            "seller",
            placement,
            vec![],
            vec![],
            seller_sort("reputation"),
            limit,
        ),
        0.8,
        "seller_quality",
    )
    .await
}

async fn recall_renewal(
    placement: &PlacementDefinition,
    subject_profile: Option<&SubjectProfile>,
    limit: usize,
) -> RepoResult<Vec<RecallCandidate>> {
    let Some(subject_profile) = subject_profile else {
        return Ok(Vec::new());
    };
    let mut should = Vec::new();
    for category in &subject_profile.preferred_categories {
        should.push(json!({ "term": { "category.keyword": category } }));
    }
    for tag in &subject_profile.preferred_tags {
        should.push(json!({ "term": { "tags.keyword": tag } }));
    }
    let mut filters = placement_product_filters(placement, None);
    if let Some(delivery_mode) = subject_profile.preferred_delivery_modes.first() {
        filters.push(json!({ "term": { "delivery_modes.keyword": delivery_mode } }));
    }
    fetch_os_candidates(
        &product_read_alias(),
        "product",
        recall_body_for_scope(
            "product",
            placement,
            filters,
            should,
            product_sort("latest"),
            limit,
        ),
        0.83,
        "renewal",
    )
    .await
}

fn merge_recall_candidates(
    merged: &mut HashMap<String, RecallCandidate>,
    candidates: Vec<RecallCandidate>,
) {
    for candidate in candidates {
        let key = entity_key(&candidate.entity_scope, &candidate.entity_id);
        merged
            .entry(key)
            .and_modify(|existing| {
                existing.raw_score = existing.raw_score.max(candidate.raw_score);
                existing
                    .recall_sources
                    .extend(candidate.recall_sources.iter().cloned());
                existing
                    .explanation_codes
                    .extend(candidate.explanation_codes.iter().cloned());
            })
            .or_insert(candidate);
    }
}

async fn fetch_os_candidates(
    alias: &str,
    entity_scope: &str,
    mut body: Value,
    base_score: f64,
    source: &str,
) -> RepoResult<Vec<RecallCandidate>> {
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());
    let preference = body
        .get("preference")
        .and_then(Value::as_str)
        .map(str::to_string);
    if let Some(object) = body.as_object_mut() {
        object.remove("preference");
    }
    let request = reqwest::Client::new().post(format!(
        "{}/{alias}/_search",
        endpoint.trim_end_matches('/')
    ));
    let request = if let Some(preference) = preference.as_deref() {
        request.query(&[("preference", preference)])
    } else {
        request
    };
    let response = request
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("opensearch recommendation search request failed: {err}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unavailable".to_string());
        return Err(format!(
            "opensearch recommendation search failed: status={status} body={body}"
        ));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|err| format!("decode recommendation opensearch response failed: {err}"))?;
    let mut candidates = Vec::new();
    if let Some(hits) = payload["hits"]["hits"].as_array() {
        for (index, hit) in hits.iter().enumerate() {
            let Some(source_id) = normalized_hit_entity_id(hit, entity_scope) else {
                continue;
            };
            candidates.push(recall_candidate(
                entity_scope.to_string(),
                source_id,
                base_score + hit["_score"].as_f64().unwrap_or(0.0) + score_decay(index),
                source,
            ));
        }
    }
    Ok(candidates)
}

fn recall_body_for_scope(
    entity_scope: &str,
    placement: &PlacementDefinition,
    filters: Vec<Value>,
    should: Vec<Value>,
    sort: Value,
    limit: usize,
) -> Value {
    let must = if should.is_empty() {
        vec![json!({ "match_all": {} })]
    } else {
        vec![json!({
            "bool": {
                "should": should,
                "minimum_should_match": 1
            }
        })]
    };
    let mut filter_terms = filters;
    if entity_scope == "seller" {
        filter_terms.push(json!({ "term": { "status": "active" } }));
    } else {
        filter_terms.push(json!({ "term": { "status": "listed" } }));
    }
    json!({
        "from": 0,
        "size": limit.min(50),
        "query": {
            "bool": {
                "must": must,
                "filter": filter_terms
            }
        },
        "sort": sort,
        "_source": true,
        "track_total_hits": false,
        "timeout": "3s",
        "preference": placement.page_context
    })
}

fn placement_product_filters(
    placement: &PlacementDefinition,
    exclude_id: Option<&str>,
) -> Vec<Value> {
    let mut filters = Vec::new();
    match placement.placement_scope.as_str() {
        "service" => filters.push(json!({ "term": { "product_type.keyword": "service" } })),
        "product" => filters.push(
            json!({ "bool": { "must_not": [{ "term": { "product_type.keyword": "service" } }] } }),
        ),
        _ => {}
    }
    if let Some(entity_id) = exclude_id {
        filters.push(json!({ "bool": { "must_not": [{ "term": { "id": entity_id } }] } }));
    }
    filters
}

fn product_sort(sort_key: &str) -> Value {
    match sort_key {
        "latest" => json!([{ "updated_at": { "order": "desc", "missing": "_last" } }]),
        "quality" => json!([{ "quality_score": { "order": "desc", "missing": "_last" } }]),
        "reputation" => {
            json!([{ "seller_reputation_score": { "order": "desc", "missing": "_last" } }])
        }
        "hotness" => json!([{ "hotness_score": { "order": "desc", "missing": "_last" } }]),
        _ => json!([
            { "_score": { "order": "desc" } },
            { "updated_at": { "order": "desc", "missing": "_last" } }
        ]),
    }
}

fn seller_sort(sort_key: &str) -> Value {
    match sort_key {
        "reputation" => json!([{ "reputation_score": { "order": "desc", "missing": "_last" } }]),
        "hotness" => json!([{ "listing_product_count": { "order": "desc", "missing": "_last" } }]),
        _ => json!([{ "updated_at": { "order": "desc", "missing": "_last" } }]),
    }
}

fn recall_candidate(
    entity_scope: impl Into<String>,
    entity_id: impl Into<String>,
    raw_score: f64,
    source: &str,
) -> RecallCandidate {
    let mut recall_sources = BTreeSet::new();
    recall_sources.insert(source.to_string());
    let mut explanation_codes = BTreeSet::new();
    explanation_codes.insert(format!("recall:{source}"));
    RecallCandidate {
        entity_scope: entity_scope.into(),
        entity_id: entity_id.into(),
        raw_score,
        recall_sources,
        explanation_codes,
    }
}

fn compute_final_score(seed: &CandidateSeed, item: &SearchResultItem, seen: bool) -> f64 {
    let weights = default_ranking_weights();
    let quality = parse_score(item.quality_score.as_deref());
    let reputation = parse_score(item.reputation_score.as_deref());
    let hotness = parse_hotness(item);
    let freshness = if seed
        .recall_sources
        .iter()
        .any(|source| source == "new_arrival")
    {
        1.0
    } else {
        0.0
    };
    let similarity = if seed
        .recall_sources
        .iter()
        .any(|source| matches!(source.as_str(), "similar" | "bundle"))
    {
        1.0
    } else {
        0.0
    };
    let intent = if seed.recall_sources.iter().any(|source| {
        matches!(
            source.as_str(),
            "cohort" | "renewal" | "seller_related" | "similar"
        )
    }) {
        1.0
    } else {
        0.0
    };
    let bundle = if seed.recall_sources.iter().any(|source| source == "bundle") {
        1.0
    } else {
        0.0
    };
    let repeat_penalty = if seen { 1.0 } else { 0.0 };

    seed.raw_score
        + weights.get("intent").copied().unwrap_or(0.0) * intent
        + weights.get("similarity").copied().unwrap_or(0.0) * similarity
        + weights.get("hotness").copied().unwrap_or(0.0) * hotness
        + weights.get("quality").copied().unwrap_or(0.0) * quality
        + weights.get("reputation").copied().unwrap_or(0.0) * reputation
        + weights.get("freshness").copied().unwrap_or(0.0) * freshness
        + weights.get("bundle").copied().unwrap_or(0.0) * bundle
        - weights.get("repeat_penalty").copied().unwrap_or(0.0) * repeat_penalty
}

fn default_ranking_weights() -> BTreeMap<&'static str, f64> {
    BTreeMap::from([
        ("intent", 0.25),
        ("similarity", 0.20),
        ("hotness", 0.15),
        ("quality", 0.15),
        ("reputation", 0.10),
        ("freshness", 0.05),
        ("bundle", 0.05),
        ("repeat_penalty", 0.05),
    ])
}

fn parse_score(value: Option<&str>) -> f64 {
    let raw = value.and_then(|raw| raw.parse::<f64>().ok()).unwrap_or(0.0);
    if raw > 1.0 {
        (raw / 100.0).clamp(0.0, 1.0)
    } else {
        raw.clamp(0.0, 1.0)
    }
}

fn parse_hotness(item: &SearchResultItem) -> f64 {
    let hotness = parse_score(item.hotness_score.as_deref());
    if hotness > 0.0 {
        return hotness;
    }
    item.listing_product_count
        .map(|count| (count as f64 / 100.0).clamp(0.0, 1.0))
        .unwrap_or(0.0)
}

fn strategy_version(ranking_profile: &RankingProfile) -> String {
    format!(
        "{}@{}",
        ranking_profile.profile_key, ranking_profile.updated_at
    )
}

fn parse_recall_sources(candidate_policy_json: &Value) -> Vec<String> {
    let sources = candidate_policy_json["recall"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if sources.is_empty() {
        vec!["popular".to_string(), "new_arrival".to_string()]
    } else {
        sources
    }
}

fn placement_entity_scopes(placement_scope: &str) -> Vec<&'static str> {
    match placement_scope {
        "service" | "product" => vec!["product"],
        "seller" => vec!["seller"],
        _ => vec!["product", "seller"],
    }
}

fn placement_supports_products(placement_scope: &str) -> bool {
    !matches!(placement_scope, "seller")
}

fn placement_supports_sellers(placement_scope: &str) -> bool {
    matches!(placement_scope, "mixed" | "seller")
}

fn placement_allows_scope(placement: &PlacementDefinition, entity_scope: &str) -> bool {
    match placement.placement_scope.as_str() {
        "mixed" => true,
        "seller" => entity_scope == "seller",
        "service" | "product" => entity_scope == "product",
        _ => true,
    }
}

fn score_decay(index: usize) -> f64 {
    1.0 - (index as f64 * 0.01).min(0.5)
}

fn recommendation_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(10).clamp(1, 20)
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

fn subject_ref(query: &RecommendationQuery) -> String {
    query
        .subject_org_id
        .clone()
        .or_else(|| query.subject_user_id.clone())
        .or_else(|| query.anonymous_session_key.clone())
        .unwrap_or_else(|| "anonymous".to_string())
}

fn derived_cohort_keys(query: &RecommendationQuery) -> Vec<String> {
    let mut keys = vec!["global".to_string()];
    if let Some(subject_org_id) = query.subject_org_id.as_deref() {
        keys.push(format!("org:{subject_org_id}"));
    }
    if let Some(subject_user_id) = query.subject_user_id.as_deref() {
        keys.push(format!("user:{subject_user_id}"));
    }
    keys
}

fn entity_key(entity_scope: &str, entity_id: &str) -> String {
    format!("{entity_scope}:{entity_id}")
}

fn summarize_candidate_sources(ranked: &[RankedCandidate]) -> Value {
    let mut summary = BTreeMap::<String, usize>::new();
    for item in ranked {
        for source in &item.recall_sources {
            *summary.entry(source.clone()).or_default() += 1;
        }
    }
    json!(summary)
}

async fn load_candidate_cache(cache_key: &str) -> RepoResult<Option<CandidateSnapshot>> {
    let client = redis::Client::open(redis_url().as_str())
        .map_err(|err| format!("redis recommendation client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis recommendation connect failed: {err}"))?;
    let value: Option<String> = connection
        .get(cache_key)
        .await
        .map_err(|err| format!("redis recommendation cache get failed: {err}"))?;
    match value {
        Some(serialized) => serde_json::from_str(&serialized)
            .map(Some)
            .map_err(|err| format!("decode recommendation cache failed: {err}")),
        None => Ok(None),
    }
}

async fn store_candidate_cache(cache_key: &str, snapshot: &CandidateSnapshot) -> RepoResult<()> {
    let client = redis::Client::open(redis_url().as_str())
        .map_err(|err| format!("redis recommendation client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis recommendation connect failed: {err}"))?;
    let serialized = serde_json::to_string(snapshot)
        .map_err(|err| format!("encode recommendation cache failed: {err}"))?;
    connection
        .set_ex::<_, _, ()>(cache_key, serialized, 600)
        .await
        .map_err(|err| format!("redis recommendation cache set failed: {err}"))?;
    Ok(())
}

fn recommendation_cache_key(
    query: &RecommendationQuery,
    candidate_backend: CandidateBackend,
) -> RepoResult<String> {
    let scene = json!({
        "candidate_backend": candidate_backend.as_str(),
        "placement_code": query.placement_code,
        "subject_scope": query.subject_scope,
        "subject_org_id": query.subject_org_id,
        "subject_user_id": query.subject_user_id,
        "anonymous_session_key": query.anonymous_session_key,
        "context_entity_scope": query.context_entity_scope,
        "context_entity_id": query.context_entity_id,
        "limit": recommendation_limit(query.limit)
    });
    let serialized = serde_json::to_vec(&scene)
        .map_err(|err| format!("encode recommendation cache scene failed: {err}"))?;
    let hash = hex_sha256(&serialized);
    Ok(format!(
        "{}:recommend:{}:{}:{}",
        redis_namespace(),
        query
            .subject_org_id
            .clone()
            .unwrap_or_else(|| "public".to_string()),
        query
            .subject_user_id
            .clone()
            .or_else(|| query.anonymous_session_key.clone())
            .unwrap_or_else(|| "anonymous".to_string()),
        hash
    ))
}

async fn load_seen_entities(
    query: &RecommendationQuery,
    placement_code: &str,
) -> RepoResult<HashSet<String>> {
    let subject_key = query_subject_key(query);
    let redis_key = seen_entities_key(&subject_key, placement_code);
    let client = redis::Client::open(redis_url().as_str())
        .map_err(|err| format!("redis seen-set client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis seen-set connect failed: {err}"))?;
    let members: Vec<String> = connection
        .smembers(redis_key)
        .await
        .map_err(|err| format!("redis seen-set load failed: {err}"))?;
    Ok(members.into_iter().collect())
}

async fn remember_seen_entities(
    subject_key: String,
    placement_code: &str,
    items: &[crate::modules::recommendation::domain::ExposureItemInput],
) -> RepoResult<()> {
    let redis_key = seen_entities_key(&subject_key, placement_code);
    let client = redis::Client::open(redis_url().as_str())
        .map_err(|err| format!("redis seen-set client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis seen-set connect failed: {err}"))?;
    let values = items
        .iter()
        .map(|item| entity_key(&item.entity_scope, &item.entity_id))
        .collect::<Vec<_>>();
    if !values.is_empty() {
        connection
            .sadd::<_, _, usize>(&redis_key, values)
            .await
            .map_err(|err| format!("redis seen-set sadd failed: {err}"))?;
    }
    connection
        .expire::<_, ()>(&redis_key, 86_400)
        .await
        .map_err(|err| format!("redis seen-set expire failed: {err}"))?;
    Ok(())
}

async fn existing_behavior_events(
    client: &(impl GenericClient + Sync),
    event_types: &[&str],
    idempotency_key: &str,
) -> RepoResult<Vec<String>> {
    let types = event_types
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    let rows = client
        .query(
            "SELECT behavior_event_id::text
             FROM recommend.behavior_event
             WHERE event_type = ANY($1::text[])
               AND attrs ->> 'idempotency_key' = $2
             ORDER BY created_at ASC",
            &[&types, &idempotency_key],
        )
        .await
        .map_err(|err| format!("lookup recommendation behavior idempotency failed: {err}"))?;
    Ok(rows.into_iter().map(|row| row.get(0)).collect())
}

async fn load_recommendation_reference(
    client: &(impl GenericClient + Sync),
    recommendation_request_id: &str,
    recommendation_result_id: &str,
) -> RepoResult<RecommendationReference> {
    let row = client
        .query_opt(
            "SELECT
               req.placement_code,
               req.page_context,
               req.subject_scope,
               req.subject_org_id::text,
               req.subject_user_id::text,
               req.anonymous_session_key,
               req.recommendation_request_id::text,
               res.recommendation_result_id::text
             FROM recommend.recommendation_request req
             JOIN recommend.recommendation_result res
               ON res.recommendation_request_id = req.recommendation_request_id
             WHERE req.recommendation_request_id = $1::text::uuid
               AND res.recommendation_result_id = $2::text::uuid",
            &[&recommendation_request_id, &recommendation_result_id],
        )
        .await
        .map_err(|err| format!("load recommendation reference failed: {err}"))?;
    let Some(row) = row else {
        return Err("recommendation request/result reference missing".to_string());
    };
    Ok(RecommendationReference {
        placement_code: row.get(0),
        page_context: row.get(1),
        subject_scope: row.get(2),
        subject_org_id: row.get(3),
        subject_user_id: row.get(4),
        anonymous_session_key: row.get(5),
        recommendation_request_id: row.get(6),
        recommendation_result_id: row.get(7),
    })
}

#[derive(Debug, Clone)]
struct ResultItemReference {
    recommendation_result_item_id: String,
    entity_scope: String,
    entity_id: String,
    position_no: i32,
}

async fn load_result_item_reference(
    client: &(impl GenericClient + Sync),
    recommendation_result_id: &str,
    recommendation_result_item_id: Option<&str>,
    entity_scope: &str,
    entity_id: &str,
) -> RepoResult<ResultItemReference> {
    let row = client
        .query_opt(
            "SELECT
               recommendation_result_item_id::text,
               entity_scope,
               entity_id::text,
               position_no
             FROM recommend.recommendation_result_item
             WHERE recommendation_result_id = $1::text::uuid
               AND ($2::text IS NULL OR recommendation_result_item_id = $2::text::uuid)
               AND entity_scope = $3
               AND entity_id = $4::text::uuid
             LIMIT 1",
            &[
                &recommendation_result_id,
                &recommendation_result_item_id,
                &entity_scope,
                &entity_id,
            ],
        )
        .await
        .map_err(|err| format!("load recommendation result item reference failed: {err}"))?;
    let Some(row) = row else {
        return Err("recommendation result item reference missing".to_string());
    };
    Ok(ResultItemReference {
        recommendation_result_item_id: row.get(0),
        entity_scope: row.get(1),
        entity_id: row.get(2),
        position_no: row.get(3),
    })
}

async fn insert_behavior_event(
    client: &(impl GenericClient + Sync),
    reference: &RecommendationReference,
    event_type: &str,
    placement_code: &str,
    entity_scope: &str,
    entity_id: Option<&str>,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    attrs: &Value,
) -> RepoResult<String> {
    let row = client
        .query_one(
            "INSERT INTO recommend.behavior_event (
               subject_scope,
               subject_org_id,
               subject_user_id,
               anonymous_session_key,
               event_type,
               placement_code,
               entity_scope,
               entity_id,
               page_context,
               recommendation_request_id,
               recommendation_result_id,
               request_id,
               trace_id,
               attrs
             ) VALUES (
               $1,
               $2::text::uuid,
               $3::text::uuid,
               $4,
               $5,
               $6,
               $7,
               $8::text::uuid,
               $9,
               $10::text::uuid,
               $11::text::uuid,
               $12,
               $13,
               $14::jsonb
             )
             RETURNING behavior_event_id::text",
            &[
                &reference.subject_scope,
                &reference.subject_org_id,
                &reference.subject_user_id,
                &reference.anonymous_session_key,
                &event_type,
                &placement_code,
                &entity_scope,
                &entity_id,
                &reference.page_context,
                &reference.recommendation_request_id,
                &reference.recommendation_result_id,
                &request_id,
                &trace_id,
                attrs,
            ],
        )
        .await
        .map_err(|err| format!("insert recommendation behavior event failed: {err}"))?;
    Ok(row.get(0))
}

async fn enqueue_behavior_outbox(
    client: &(impl GenericClient + Sync),
    behavior_event_id: &str,
    reference: &RecommendationReference,
    event_type: &str,
    entity_scope: &str,
    entity_id: Option<&str>,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: &str,
    attrs: &Value,
) -> RepoResult<bool> {
    let payload = json!({
        "behavior_event_id": behavior_event_id,
        "subject_scope": reference.subject_scope,
        "subject_org_id": reference.subject_org_id,
        "subject_user_id": reference.subject_user_id,
        "anonymous_session_key": reference.anonymous_session_key,
        "placement_code": reference.placement_code,
        "event_type": event_type,
        "entity_scope": entity_scope,
        "entity_id": entity_id,
        "page_context": reference.page_context,
        "recommendation_request_id": reference.recommendation_request_id,
        "recommendation_result_id": reference.recommendation_result_id,
        "request_id": request_id,
        "trace_id": trace_id,
        "idempotency_key": idempotency_key,
        "attrs": attrs
    });
    write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: "recommend.behavior_event",
            aggregate_id: behavior_event_id,
            event_type: "recommend.behavior_recorded",
            producer_service: "platform-core.recommendation",
            request_id,
            trace_id,
            idempotency_key: Some(idempotency_key),
            occurred_at: None,
            business_payload: &payload,
            deduplicate_by_idempotency_key: false,
        },
    )
    .await
    .map_err(|err| format!("enqueue recommendation behavior outbox failed: {err}"))
}

fn query_subject_key(query: &RecommendationQuery) -> String {
    query
        .subject_user_id
        .clone()
        .or_else(|| query.subject_org_id.clone())
        .or_else(|| query.anonymous_session_key.clone())
        .unwrap_or_else(|| "anonymous".to_string())
}

fn query_subject_cache_ref(reference: &RecommendationReference) -> String {
    reference
        .subject_user_id
        .clone()
        .or_else(|| reference.subject_org_id.clone())
        .or_else(|| reference.anonymous_session_key.clone())
        .unwrap_or_else(|| "anonymous".to_string())
}

fn seen_entities_key(subject_key: &str, placement_code: &str) -> String {
    format!(
        "{}:recommend:seen:{}:{}",
        redis_namespace(),
        subject_key,
        placement_code
    )
}

pub async fn invalidate_placement_runtime_cache(placement_code: &str) -> RepoResult<usize> {
    let client = redis::Client::open(redis_url().as_str())
        .map_err(|err| format!("redis recommendation client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis recommendation connect failed: {err}"))?;
    let mut keys: Vec<String> = connection
        .keys(format!("{}:recommend:*", redis_namespace()))
        .await
        .map_err(|err| format!("redis recommendation keys lookup failed: {err}"))?;
    let placement_seen_keys: Vec<String> = connection
        .keys(format!(
            "{}:recommend:seen:*:{}",
            redis_namespace(),
            placement_code
        ))
        .await
        .map_err(|err| format!("redis recommendation seen-set lookup failed: {err}"))?;
    keys.extend(placement_seen_keys);
    keys.sort();
    keys.dedup();
    let mut deleted = 0usize;
    for key in keys {
        deleted += connection
            .del::<_, usize>(key)
            .await
            .map_err(|err| format!("redis recommendation delete failed: {err}"))?;
    }
    Ok(deleted)
}

async fn invalidate_recommendation_cache(
    request: &RecommendationRebuildRequest,
) -> RepoResult<usize> {
    let client = redis::Client::open(redis_url().as_str())
        .map_err(|err| format!("redis recommendation client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis recommendation connect failed: {err}"))?;
    let mut keys: Vec<String> = connection
        .keys(recommendation_cache_pattern(request))
        .await
        .map_err(|err| format!("redis recommendation keys lookup failed: {err}"))?;
    let seen_keys: Vec<String> = connection
        .keys(recommendation_seen_pattern(request))
        .await
        .map_err(|err| format!("redis recommendation seen-set lookup failed: {err}"))?;
    keys.extend(seen_keys);
    keys.sort();
    keys.dedup();
    let mut deleted = 0usize;
    for key in keys {
        deleted += connection
            .del::<_, usize>(key)
            .await
            .map_err(|err| format!("redis recommendation delete failed: {err}"))?;
    }
    Ok(deleted)
}

fn recommendation_cache_pattern(request: &RecommendationRebuildRequest) -> String {
    let tenant = request.subject_org_id.clone().unwrap_or_else(|| {
        if request.subject_user_id.is_some() || request.anonymous_session_key.is_some() {
            "public".to_string()
        } else {
            "*".to_string()
        }
    });
    let actor = request
        .subject_user_id
        .clone()
        .or_else(|| request.anonymous_session_key.clone())
        .unwrap_or_else(|| "*".to_string());
    format!("{}:recommend:{tenant}:{actor}:*", redis_namespace())
}

fn recommendation_seen_pattern(request: &RecommendationRebuildRequest) -> String {
    let subject_key = request
        .subject_user_id
        .clone()
        .or_else(|| request.subject_org_id.clone())
        .or_else(|| request.anonymous_session_key.clone())
        .unwrap_or_else(|| "*".to_string());
    let placement_code = request
        .placement_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("*");
    format!(
        "{}:recommend:seen:{subject_key}:{placement_code}",
        redis_namespace()
    )
}

async fn rebuild_subject_profiles(
    client: &Client,
    request: &RecommendationRebuildRequest,
) -> RepoResult<u64> {
    let tx = client.transaction().await.map_err(|err| {
        format!("open recommendation subject-profile rebuild transaction failed: {err}")
    })?;
    tx.execute("DELETE FROM recommend.subject_profile_snapshot", &[])
        .await
        .map_err(|err| format!("clear recommendation subject profiles failed: {err}"))?;
    let inserted = tx
        .execute(
            "INSERT INTO recommend.subject_profile_snapshot (
               subject_scope,
               subject_ref,
               org_id,
               user_id,
               profile_version,
               preferred_categories,
               preferred_tags,
               preferred_delivery_modes,
               feature_snapshot,
               last_behavior_at
             )
             SELECT
               be.subject_scope,
               COALESCE(be.subject_org_id::text, be.subject_user_id::text, be.anonymous_session_key),
               max(be.subject_org_id::text)::uuid,
               max(be.subject_user_id::text)::uuid,
               GREATEST(count(*)::bigint, 1),
               array_remove(array_agg(DISTINCT NULLIF(be.attrs ->> 'category', '')), NULL),
               array_remove(array_agg(DISTINCT NULLIF(tag.value, '')), NULL),
               array_remove(
                 array_agg(DISTINCT NULLIF(COALESCE(be.attrs ->> 'delivery_mode', be.attrs ->> 'delivery_type'), '')),
                 NULL
               ),
               jsonb_build_object(
                 'last_event_type', (array_agg(be.event_type ORDER BY be.occurred_at DESC))[1],
                 'last_entity_scope', (array_agg(be.entity_scope ORDER BY be.occurred_at DESC))[1],
                 'last_entity_id', (array_agg(be.entity_id::text ORDER BY be.occurred_at DESC))[1],
                 'last_placement_code', (array_agg(be.placement_code ORDER BY be.occurred_at DESC))[1]
               ),
               max(be.occurred_at)
             FROM recommend.behavior_event be
             LEFT JOIN LATERAL jsonb_array_elements_text(
               CASE
                 WHEN jsonb_typeof(be.attrs -> 'tags') = 'array' THEN be.attrs -> 'tags'
                 ELSE '[]'::jsonb
               END
             ) AS tag(value) ON true
             WHERE COALESCE(be.subject_org_id::text, be.subject_user_id::text, be.anonymous_session_key) IS NOT NULL
             GROUP BY be.subject_scope, COALESCE(be.subject_org_id::text, be.subject_user_id::text, be.anonymous_session_key)",
            &[],
        )
        .await
        .map_err(|err| format!("rebuild recommendation subject profiles failed: {err}"))?;
    tx.commit().await.map_err(|err| {
        format!("commit recommendation subject-profile rebuild transaction failed: {err}")
    })?;
    let _ = request;
    Ok(inserted)
}

async fn rebuild_cohort_popularity(
    client: &Client,
    request: &RecommendationRebuildRequest,
) -> RepoResult<u64> {
    let tx = client
        .transaction()
        .await
        .map_err(|err| format!("open recommendation cohort rebuild transaction failed: {err}"))?;
    tx.execute("DELETE FROM recommend.cohort_popularity", &[])
        .await
        .map_err(|err| format!("clear recommendation cohort popularity failed: {err}"))?;
    let inserted = tx
        .execute(
            "INSERT INTO recommend.cohort_popularity (
               cohort_key,
               entity_scope,
               entity_id,
               exposure_count,
               click_count,
               order_count,
               payment_count,
               acceptance_count,
               hotness_score
             )
             SELECT
               COALESCE(
                 be.attrs ->> 'cohort_key',
                 CASE
                   WHEN be.subject_org_id IS NOT NULL THEN 'org:' || be.subject_org_id::text
                   WHEN be.subject_user_id IS NOT NULL THEN 'user:' || be.subject_user_id::text
                   ELSE 'global'
                 END
               ) AS cohort_key,
               be.entity_scope,
               be.entity_id,
               count(*) FILTER (WHERE be.event_type IN ('recommendation_panel_viewed', 'recommendation_item_exposed'))::bigint,
               count(*) FILTER (WHERE be.event_type IN ('recommendation_item_clicked', 'seller_recommendation_clicked'))::bigint,
               count(*) FILTER (WHERE be.event_type = 'order_submitted')::bigint,
               count(*) FILTER (WHERE be.event_type = 'payment_succeeded')::bigint,
               count(*) FILTER (WHERE be.event_type = 'delivery_accepted')::bigint,
               (
                 count(*) FILTER (WHERE be.event_type IN ('recommendation_panel_viewed', 'recommendation_item_exposed'))::numeric * 0.20
                 + count(*) FILTER (WHERE be.event_type IN ('recommendation_item_clicked', 'seller_recommendation_clicked'))::numeric * 1.00
                 + count(*) FILTER (WHERE be.event_type = 'order_submitted')::numeric * 2.00
                 + count(*) FILTER (WHERE be.event_type = 'payment_succeeded')::numeric * 3.00
                 + count(*) FILTER (WHERE be.event_type = 'delivery_accepted')::numeric * 3.50
               )::numeric(12, 6)
             FROM recommend.behavior_event be
             WHERE be.entity_id IS NOT NULL
             GROUP BY 1, 2, 3",
            &[],
        )
        .await
        .map_err(|err| format!("rebuild recommendation cohort popularity failed: {err}"))?;
    tx.commit()
        .await
        .map_err(|err| format!("commit recommendation cohort rebuild transaction failed: {err}"))?;
    let _ = request;
    Ok(inserted)
}

async fn rebuild_search_signal_aggregate(
    client: &Client,
    request: &RecommendationRebuildRequest,
) -> RepoResult<u64> {
    let tx = client
        .transaction()
        .await
        .map_err(|err| format!("open recommendation signal rebuild transaction failed: {err}"))?;
    tx.execute(
        "DELETE FROM search.search_signal_aggregate
         WHERE entity_scope IN ('product', 'seller')",
        &[],
    )
    .await
    .map_err(|err| format!("clear search signal aggregate failed: {err}"))?;
    let inserted = tx
        .execute(
            "INSERT INTO search.search_signal_aggregate (
               entity_scope,
               entity_id,
               exposure_count,
               click_count,
               order_count,
               hotness_score
             )
             SELECT
               be.entity_scope,
               be.entity_id,
               count(*) FILTER (WHERE be.event_type IN ('recommendation_panel_viewed', 'recommendation_item_exposed'))::bigint,
               count(*) FILTER (WHERE be.event_type IN ('recommendation_item_clicked', 'seller_recommendation_clicked'))::bigint,
               count(*) FILTER (WHERE be.event_type = 'order_submitted')::bigint,
               (
                 count(*) FILTER (WHERE be.event_type IN ('recommendation_panel_viewed', 'recommendation_item_exposed'))::numeric * 0.20
                 + count(*) FILTER (WHERE be.event_type IN ('recommendation_item_clicked', 'seller_recommendation_clicked'))::numeric * 1.00
                 + count(*) FILTER (WHERE be.event_type = 'order_submitted')::numeric * 2.00
               )::numeric(10, 4)
             FROM recommend.behavior_event be
             WHERE be.entity_id IS NOT NULL
               AND be.entity_scope IN ('product', 'seller')
             GROUP BY be.entity_scope, be.entity_id",
            &[],
        )
        .await
        .map_err(|err| format!("rebuild recommendation search signals failed: {err}"))?;
    tx.commit()
        .await
        .map_err(|err| format!("commit recommendation signal rebuild transaction failed: {err}"))?;
    let _ = request;
    Ok(inserted)
}

async fn rebuild_similarity_edges(
    client: &Client,
    request: &RecommendationRebuildRequest,
) -> RepoResult<u64> {
    let tx = client.transaction().await.map_err(|err| {
        format!("open recommendation similarity rebuild transaction failed: {err}")
    })?;
    tx.execute("DELETE FROM recommend.entity_similarity", &[])
        .await
        .map_err(|err| format!("clear recommendation entity similarity failed: {err}"))?;
    let inserted = tx
        .execute(
            "INSERT INTO recommend.entity_similarity (
               source_entity_scope,
               source_entity_id,
               target_entity_scope,
               target_entity_id,
               similarity_type,
               score,
               evidence_json,
               version_no
             )
             SELECT
               left_item.entity_scope,
               left_item.entity_id,
               right_item.entity_scope,
               right_item.entity_id,
               'co_recommended',
               count(*)::numeric(12, 6),
               jsonb_build_object('rebuild_source', 'recommendation_result_item'),
               1
             FROM recommend.recommendation_result_item left_item
             JOIN recommend.recommendation_result_item right_item
               ON right_item.recommendation_result_id = left_item.recommendation_result_id
              AND (right_item.entity_scope, right_item.entity_id) <> (left_item.entity_scope, left_item.entity_id)
             GROUP BY 1, 2, 3, 4",
            &[],
        )
        .await
        .map_err(|err| format!("rebuild recommendation similarity edges failed: {err}"))?;
    tx.commit().await.map_err(|err| {
        format!("commit recommendation similarity rebuild transaction failed: {err}")
    })?;
    let _ = request;
    Ok(inserted)
}

async fn rebuild_bundle_relations(
    client: &Client,
    request: &RecommendationRebuildRequest,
) -> RepoResult<u64> {
    let tx = client
        .transaction()
        .await
        .map_err(|err| format!("open recommendation bundle rebuild transaction failed: {err}"))?;
    tx.execute("DELETE FROM recommend.bundle_relation", &[])
        .await
        .map_err(|err| format!("clear recommendation bundle relations failed: {err}"))?;
    let inserted = tx
        .execute(
            "INSERT INTO recommend.bundle_relation (
               source_entity_scope,
               source_entity_id,
               target_entity_scope,
               target_entity_id,
               relation_type,
               relation_score,
               status,
               metadata
             )
             SELECT
               left_item.entity_scope,
               left_item.entity_id,
               right_item.entity_scope,
               right_item.entity_id,
               'co_recommended',
               count(*)::numeric(12, 6),
               'active',
               jsonb_build_object('rebuild_source', 'recommendation_result_item')
             FROM recommend.recommendation_result_item left_item
             JOIN recommend.recommendation_result_item right_item
               ON right_item.recommendation_result_id = left_item.recommendation_result_id
              AND (right_item.entity_scope, right_item.entity_id) <> (left_item.entity_scope, left_item.entity_id)
             GROUP BY 1, 2, 3, 4",
            &[],
        )
        .await
        .map_err(|err| format!("rebuild recommendation bundle relations failed: {err}"))?;
    tx.commit()
        .await
        .map_err(|err| format!("commit recommendation bundle rebuild transaction failed: {err}"))?;
    let _ = request;
    Ok(inserted)
}

fn redis_url() -> String {
    std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://default:datab_redis_pass@127.0.0.1:6379/1".to_string())
}

fn redis_namespace() -> String {
    std::env::var("REDIS_NAMESPACE").unwrap_or_else(|_| "datab:v1".to_string())
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn product_read_alias() -> String {
    std::env::var("INDEX_ALIAS_PRODUCT_SEARCH_READ")
        .unwrap_or_else(|_| "product_search_read".to_string())
}

fn seller_read_alias() -> String {
    std::env::var("INDEX_ALIAS_SELLER_SEARCH_READ")
        .unwrap_or_else(|_| "seller_search_read".to_string())
}

fn normalized_hit_entity_id(hit: &Value, entity_scope: &str) -> Option<String> {
    let source = &hit["_source"];
    let candidates = if entity_scope == "seller" {
        [
            source.get("org_id").and_then(Value::as_str),
            source.get("id").and_then(Value::as_str),
            hit.get("_id").and_then(Value::as_str),
        ]
    } else {
        [
            source.get("product_id").and_then(Value::as_str),
            source.get("id").and_then(Value::as_str),
            hit.get("_id").and_then(Value::as_str),
        ]
    };
    candidates
        .into_iter()
        .flatten()
        .find(|candidate| Uuid::parse_str(candidate).is_ok())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::normalized_hit_entity_id;
    use serde_json::json;

    #[test]
    fn normalized_hit_entity_id_prefers_valid_source_uuid() {
        let hit = json!({
            "_id": "demo-product-001",
            "_source": {
                "id": "33ca2004-a11c-4af5-874b-606c7b88413a"
            }
        });
        assert_eq!(
            normalized_hit_entity_id(&hit, "product").as_deref(),
            Some("33ca2004-a11c-4af5-874b-606c7b88413a")
        );
    }

    #[test]
    fn normalized_hit_entity_id_rejects_non_uuid_documents() {
        let hit = json!({
            "_id": "demo-seller-001",
            "_source": {
                "id": "demo-seller-001"
            }
        });
        assert!(normalized_hit_entity_id(&hit, "seller").is_none());
    }
}
