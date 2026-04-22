use config::RuntimeMode;
use db::GenericClient;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

use crate::modules::search::domain::{
    AliasSwitchRequest, AliasSwitchResponse, CacheInvalidateRequest, CacheInvalidateResponse,
    PatchRankingProfileRequest, RankingProfileView, ReindexRequest, ReindexResponse, SearchQuery,
    SearchResultItem, SearchSyncQuery, SearchSyncTaskView,
};

type RepoResult<T> = Result<T, String>;
const SEARCH_CACHE_TTL_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCandidate {
    pub entity_scope: String,
    pub entity_id: String,
    pub score: f64,
    pub sort_value: Option<f64>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCandidatePage {
    pub query_scope: String,
    pub backend: String,
    pub total: u64,
    pub hits: Vec<SearchCandidate>,
}

#[derive(Debug, Clone)]
struct AliasBinding {
    alias_binding_id: String,
    entity_scope: String,
    read_alias: String,
    write_alias: String,
    active_index_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateBackend {
    OpenSearch,
    Postgresql,
}

impl CandidateBackend {
    fn as_str(self) -> &'static str {
        match self {
            CandidateBackend::OpenSearch => "opensearch",
            CandidateBackend::Postgresql => "postgresql",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RankingWeights {
    lexical: f64,
    quality: f64,
    reputation: f64,
    freshness: f64,
    trade: f64,
    completeness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedCandidatePage {
    cache_version: i64,
    page: SearchCandidatePage,
}

#[derive(Debug, Clone)]
struct SearchCacheInvalidationPlan {
    entity_scope: Option<String>,
    invalidated_scopes: Vec<String>,
    exact_key: Option<String>,
    delete_patterns: Vec<String>,
    bump_scope_versions: bool,
}

impl RankingWeights {
    fn default_for_scope(entity_scope: &str) -> Self {
        if entity_scope == "seller" {
            Self {
                lexical: 0.40,
                quality: 0.0,
                reputation: 0.25,
                freshness: 0.10,
                trade: 0.15,
                completeness: 0.10,
            }
        } else {
            Self {
                lexical: 0.40,
                quality: 0.20,
                reputation: 0.15,
                freshness: 0.10,
                trade: 0.10,
                completeness: 0.05,
            }
        }
    }

    fn from_json(entity_scope: &str, value: &Value) -> Self {
        let defaults = Self::default_for_scope(entity_scope);
        Self {
            lexical: weight_from_json(
                value,
                &["lexical", "relevance", "relevance_score"],
                defaults.lexical,
            ),
            quality: if entity_scope == "seller" {
                0.0
            } else {
                weight_from_json(value, &["quality", "quality_score"], defaults.quality)
            },
            reputation: weight_from_json(
                value,
                &["reputation", "reputation_score"],
                defaults.reputation,
            ),
            freshness: weight_from_json(
                value,
                &[
                    "freshness",
                    "freshness_score",
                    "updated_at",
                    "updated_at_score",
                ],
                defaults.freshness,
            ),
            trade: weight_from_json(
                value,
                &["trade", "hotness", "hotness_score"],
                defaults.trade,
            ),
            completeness: weight_from_json(
                value,
                &["completeness", "completeness_score"],
                defaults.completeness,
            ),
        }
    }
}

fn weight_from_json(value: &Value, keys: &[&str], default: f64) -> f64 {
    for key in keys {
        let Some(raw) = value.get(*key) else {
            continue;
        };
        if let Some(parsed) = raw.as_f64() {
            return parsed.max(0.0);
        }
        if let Some(parsed) = raw.as_str().and_then(|inner| inner.parse::<f64>().ok()) {
            return parsed.max(0.0);
        }
    }
    default
}

pub async fn search_catalog_candidates(
    client: &(impl GenericClient + Sync),
    runtime_mode: &RuntimeMode,
    query: &SearchQuery,
) -> RepoResult<(SearchCandidatePage, bool)> {
    let backend = candidate_backend_for_runtime(runtime_mode);
    if let Some(cached) = load_candidate_cache(query, backend).await? {
        return Ok((cached, true));
    }

    let page = match backend {
        CandidateBackend::OpenSearch => fetch_candidates_from_opensearch(client, query).await?,
        CandidateBackend::Postgresql => fetch_candidates_from_projection(client, query).await?,
    };

    store_candidate_cache(query, backend, &page).await?;
    Ok((page, false))
}

pub async fn hydrate_search_results(
    client: &(impl GenericClient + Sync),
    candidates: &[SearchCandidate],
) -> RepoResult<Vec<SearchResultItem>> {
    let mut items = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let item = match candidate.entity_scope.as_str() {
            "seller" => fetch_seller_result(client, &candidate.entity_id, candidate.score).await?,
            _ => fetch_product_result(client, &candidate.entity_id, candidate.score).await?,
        };
        if let Some(item) = item {
            items.push(item);
        }
    }
    Ok(items)
}

pub async fn list_sync_tasks(
    client: &(impl GenericClient + Sync),
    query: &SearchSyncQuery,
) -> RepoResult<Vec<SearchSyncTaskView>> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200) as i64;
    let rows = client
        .query(
            "SELECT
               index_sync_task_id::text,
               entity_scope,
               entity_id::text,
               document_version::bigint,
               target_backend,
               target_index,
               source_event_id::text,
               sync_status,
               retry_count,
               last_error_code,
               last_error_message,
               to_char(scheduled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM search.index_sync_task
             WHERE ($1::text IS NULL OR entity_scope = $1)
               AND ($2::text IS NULL OR sync_status = $2)
             ORDER BY scheduled_at DESC, updated_at DESC
             LIMIT $3",
            &[&query.entity_scope, &query.sync_status, &limit],
        )
        .await
        .map_err(|err| format!("list search sync tasks failed: {err}"))?;

    Ok(rows
        .into_iter()
        .map(|row| SearchSyncTaskView {
            index_sync_task_id: row.get(0),
            entity_scope: row.get(1),
            entity_id: row.get(2),
            document_version: row.get(3),
            target_backend: row.get(4),
            target_index: row.get(5),
            source_event_id: row.get(6),
            sync_status: row.get(7),
            retry_count: row.get(8),
            last_error_code: row.get(9),
            last_error_message: row.get(10),
            scheduled_at: row.get(11),
            started_at: row.get(12),
            completed_at: row.get(13),
            updated_at: row.get(14),
        })
        .collect())
}

pub async fn queue_reindex_tasks(
    client: &(impl GenericClient + Sync),
    request: &ReindexRequest,
) -> RepoResult<ReindexResponse> {
    let scope = normalized_scope(&request.entity_scope);
    let mode = request.mode.trim().to_ascii_lowercase();
    if !matches!(scope.as_str(), "product" | "seller" | "all") {
        return Err(format!(
            "unsupported reindex entity_scope: {}",
            request.entity_scope
        ));
    }
    if !matches!(mode.as_str(), "single" | "batch" | "full") {
        return Err(format!("unsupported reindex mode: {}", request.mode));
    }
    if mode == "single" && request.entity_id.is_none() {
        return Err("entity_id is required for mode=single".to_string());
    }

    let mut enqueued_count = 0u64;
    match (scope.as_str(), mode.as_str()) {
        ("product", "single") | ("seller", "single") => {
            let target_index =
                resolve_target_index(client, &scope, request.target_index.as_deref()).await?;
            let doc_version = load_document_version(
                client,
                &scope,
                request
                    .entity_id
                    .as_deref()
                    .expect("single mode entity_id checked"),
            )
            .await?;
            let inserted = client
                .execute(
                    "INSERT INTO search.index_sync_task (
                       entity_scope,
                       entity_id,
                       document_version,
                       target_backend,
                       target_index,
                       sync_status
                     )
                     SELECT
                       $1,
                       $2::text::uuid,
                       $3,
                       'opensearch',
                       $4,
                       'queued'
                     WHERE $5
                        OR NOT EXISTS (
                          SELECT 1
                          FROM search.index_sync_task t
                          WHERE t.entity_scope = $1
                            AND t.entity_id = $2::text::uuid
                            AND t.sync_status IN ('queued', 'processing')
                        )",
                    &[
                        &scope,
                        &request.entity_id,
                        &doc_version,
                        &target_index,
                        &request.force.unwrap_or(false),
                    ],
                )
                .await
                .map_err(|err| format!("queue single reindex task failed: {err}"))?;
            enqueued_count += inserted;
        }
        _ => {
            let scopes: Vec<String> = if scope == "all" {
                vec!["product".to_string(), "seller".to_string()]
            } else {
                vec![scope.clone()]
            };
            for item_scope in scopes {
                let target_index =
                    resolve_target_index(client, &item_scope, request.target_index.as_deref())
                        .await?;
                let inserted = if item_scope == "product" {
                    client
                        .execute(
                            "INSERT INTO search.index_sync_task (
                               entity_scope,
                               entity_id,
                               document_version,
                               target_backend,
                               target_index,
                               sync_status
                             )
                             SELECT
                               'product',
                               product_id,
                               document_version,
                               'opensearch',
                               $1,
                               'queued'
                             FROM search.product_search_document d
                             WHERE $2
                                OR NOT EXISTS (
                                  SELECT 1
                                  FROM search.index_sync_task t
                                  WHERE t.entity_scope = 'product'
                                    AND t.entity_id = d.product_id
                                    AND t.sync_status IN ('queued', 'processing')
                                )",
                            &[&target_index, &request.force.unwrap_or(false)],
                        )
                        .await
                } else {
                    client
                        .execute(
                            "INSERT INTO search.index_sync_task (
                               entity_scope,
                               entity_id,
                               document_version,
                               target_backend,
                               target_index,
                               sync_status
                             )
                             SELECT
                               'seller',
                               org_id,
                               document_version,
                               'opensearch',
                               $1,
                               'queued'
                             FROM search.seller_search_document d
                             WHERE $2
                                OR NOT EXISTS (
                                  SELECT 1
                                  FROM search.index_sync_task t
                                  WHERE t.entity_scope = 'seller'
                                    AND t.entity_id = d.org_id
                                    AND t.sync_status IN ('queued', 'processing')
                                )",
                            &[&target_index, &request.force.unwrap_or(false)],
                        )
                        .await
                }
                .map_err(|err| format!("queue batch/full reindex task failed: {err}"))?;
                enqueued_count += inserted;
            }
        }
    }

    Ok(ReindexResponse {
        entity_scope: scope,
        mode,
        enqueued_count,
        target_backend: "opensearch".to_string(),
        target_index: request.target_index.clone(),
    })
}

pub async fn switch_alias_binding(
    client: &(impl GenericClient + Sync),
    request: &AliasSwitchRequest,
) -> RepoResult<AliasSwitchResponse> {
    let binding = load_alias_binding(client, &request.entity_scope).await?;
    ensure_index_exists(&request.next_index_name).await?;
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());
    let payload = json!({
        "actions": [
            {
                "remove": {
                    "index": binding.active_index_name.clone().unwrap_or_else(|| request.next_index_name.clone()),
                    "alias": binding.read_alias
                }
            },
            {
                "remove": {
                    "index": binding.active_index_name.clone().unwrap_or_else(|| request.next_index_name.clone()),
                    "alias": binding.write_alias
                }
            },
            {
                "add": {
                    "index": request.next_index_name,
                    "alias": binding.read_alias
                }
            },
            {
                "add": {
                    "index": request.next_index_name,
                    "alias": binding.write_alias
                }
            }
        ]
    });
    let response = reqwest::Client::new()
        .post(format!("{}/_aliases", endpoint.trim_end_matches('/')))
        .json(&payload)
        .send()
        .await
        .map_err(|err| format!("opensearch alias switch request failed: {err}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unavailable".to_string());
        return Err(format!(
            "opensearch alias switch failed: status={status} body={body}"
        ));
    }

    client
        .execute(
            "UPDATE search.index_alias_binding
             SET active_index_name = $2,
                 updated_at = now()
             WHERE alias_binding_id = $1::text::uuid",
            &[&binding.alias_binding_id, &request.next_index_name],
        )
        .await
        .map_err(|err| format!("update search.index_alias_binding failed: {err}"))?;

    Ok(AliasSwitchResponse {
        entity_scope: binding.entity_scope,
        read_alias: binding.read_alias,
        write_alias: binding.write_alias,
        previous_index_name: binding.active_index_name,
        active_index_name: request.next_index_name.clone(),
    })
}

pub async fn invalidate_search_cache(
    request: &CacheInvalidateRequest,
) -> RepoResult<CacheInvalidateResponse> {
    execute_search_cache_invalidation(build_cache_invalidation_plan(request)).await
}

pub async fn invalidate_scope_cache(entity_scope: &str) -> RepoResult<CacheInvalidateResponse> {
    execute_search_cache_invalidation(SearchCacheInvalidationPlan {
        entity_scope: Some(normalized_scope(entity_scope)),
        invalidated_scopes: related_cache_scopes_for_ops(entity_scope),
        exact_key: None,
        delete_patterns: related_cache_scopes_for_ops(entity_scope)
            .into_iter()
            .map(|scope| search_cache_pattern(&scope))
            .collect(),
        bump_scope_versions: true,
    })
    .await
}

async fn execute_search_cache_invalidation(
    plan: SearchCacheInvalidationPlan,
) -> RepoResult<CacheInvalidateResponse> {
    let redis_url = redis_url();
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|err| format!("redis client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis connect failed: {err}"))?;
    if plan.bump_scope_versions {
        bump_search_cache_versions(&mut connection, &plan.invalidated_scopes).await?;
    }
    let deleted = if let Some(key) = plan.exact_key.as_deref() {
        connection
            .del::<_, usize>(key)
            .await
            .map_err(|err| format!("redis cache delete failed: {err}"))?
    } else {
        delete_search_cache_patterns(&mut connection, &plan.delete_patterns).await?
    };

    Ok(CacheInvalidateResponse {
        entity_scope: plan.entity_scope,
        deleted_keys: deleted,
        invalidated_scopes: plan.invalidated_scopes,
    })
}

fn build_cache_invalidation_plan(request: &CacheInvalidateRequest) -> SearchCacheInvalidationPlan {
    if let Some(query_hash) = request.query_hash.as_deref() {
        let scope = normalized_scope(request.entity_scope.as_deref().unwrap_or("all"));
        return SearchCacheInvalidationPlan {
            entity_scope: Some(scope.clone()),
            invalidated_scopes: vec![scope.clone()],
            exact_key: Some(format!(
                "{}:search:catalog:{}:{}",
                redis_namespace(),
                scope,
                query_hash
            )),
            delete_patterns: Vec::new(),
            bump_scope_versions: false,
        };
    }

    let requested_scope = request.entity_scope.as_deref().map(normalized_scope);
    if request.purge_all.unwrap_or(false) || requested_scope.as_deref().unwrap_or("all") == "all" {
        let scopes = all_search_cache_scopes();
        return SearchCacheInvalidationPlan {
            entity_scope: request.entity_scope.clone(),
            invalidated_scopes: scopes.clone(),
            exact_key: None,
            delete_patterns: scopes
                .into_iter()
                .map(|scope| search_cache_pattern(&scope))
                .collect(),
            bump_scope_versions: true,
        };
    }

    let scope = normalized_scope(request.entity_scope.as_deref().unwrap_or("all"));
    let invalidated_scopes = related_cache_scopes_for_ops(&scope);
    SearchCacheInvalidationPlan {
        entity_scope: Some(scope),
        delete_patterns: invalidated_scopes
            .iter()
            .map(|scope| search_cache_pattern(scope))
            .collect(),
        invalidated_scopes,
        exact_key: None,
        bump_scope_versions: true,
    }
}

pub async fn list_ranking_profiles(
    client: &(impl GenericClient + Sync),
) -> RepoResult<Vec<RankingProfileView>> {
    let rows = client
        .query(
            "SELECT
               ranking_profile_id::text,
               profile_key,
               entity_scope,
               backend_type,
               weights_json,
               filter_policy_json,
               status,
               stage_from,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM search.ranking_profile
             ORDER BY entity_scope, profile_key",
            &[],
        )
        .await
        .map_err(|err| format!("list search ranking profiles failed: {err}"))?;
    Ok(rows
        .into_iter()
        .map(|row| RankingProfileView {
            ranking_profile_id: row.get(0),
            profile_key: row.get(1),
            entity_scope: row.get(2),
            backend_type: row.get(3),
            weights_json: row.get(4),
            filter_policy_json: row.get(5),
            status: row.get(6),
            stage_from: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        })
        .collect())
}

pub async fn patch_ranking_profile(
    client: &(impl GenericClient + Sync),
    id: &str,
    request: &PatchRankingProfileRequest,
) -> RepoResult<RankingProfileView> {
    let row = client
        .query_opt(
            "UPDATE search.ranking_profile
             SET
               weights_json = COALESCE($2::jsonb, weights_json),
               filter_policy_json = COALESCE($3::jsonb, filter_policy_json),
               status = COALESCE($4, status),
               updated_at = now()
             WHERE ranking_profile_id = $1::text::uuid
             RETURNING
               ranking_profile_id::text,
               profile_key,
               entity_scope,
               backend_type,
               weights_json,
               filter_policy_json,
               status,
               stage_from,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &id,
                &request.weights_json,
                &request.filter_policy_json,
                &request.status,
            ],
        )
        .await
        .map_err(|err| format!("update search ranking profile failed: {err}"))?
        .ok_or_else(|| "search ranking profile does not exist".to_string())?;

    Ok(RankingProfileView {
        ranking_profile_id: row.get(0),
        profile_key: row.get(1),
        entity_scope: row.get(2),
        backend_type: row.get(3),
        weights_json: row.get(4),
        filter_policy_json: row.get(5),
        status: row.get(6),
        stage_from: row.get(7),
        created_at: row.get(8),
        updated_at: row.get(9),
    })
}

async fn load_active_ranking_weights(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
) -> RepoResult<RankingWeights> {
    let scope = if entity_scope == "seller" {
        "seller"
    } else {
        "product"
    };
    let row = client
        .query_opt(
            "SELECT weights_json
             FROM search.ranking_profile
             WHERE entity_scope = $1
               AND status = 'active'
             ORDER BY updated_at DESC, created_at DESC
             LIMIT 1",
            &[&scope],
        )
        .await
        .map_err(|err| format!("load active search ranking profile failed: {err}"))?;
    Ok(row
        .map(|row| RankingWeights::from_json(scope, &row.get::<usize, Value>(0)))
        .unwrap_or_else(|| RankingWeights::default_for_scope(scope)))
}

fn candidate_backend_for_runtime(runtime_mode: &RuntimeMode) -> CandidateBackend {
    match runtime_mode {
        RuntimeMode::Staging => CandidateBackend::OpenSearch,
        RuntimeMode::Local | RuntimeMode::Demo => CandidateBackend::Postgresql,
    }
}

async fn fetch_candidates_from_opensearch(
    client: &(impl GenericClient + Sync),
    query: &SearchQuery,
) -> RepoResult<SearchCandidatePage> {
    match normalized_scope(&query.entity_scope).as_str() {
        "product" | "service" => {
            let mut page = fetch_scope_candidates_from_opensearch(
                client,
                product_read_alias(),
                "product",
                query,
            )
            .await?;
            page.query_scope = normalized_scope(&query.entity_scope);
            Ok(page)
        }
        "seller" => {
            fetch_scope_candidates_from_opensearch(client, seller_read_alias(), "seller", query)
                .await
        }
        _ => {
            let product = fetch_scope_candidates_from_opensearch(
                client,
                product_read_alias(),
                "product",
                query,
            )
            .await?;
            let seller = fetch_scope_candidates_from_opensearch(
                client,
                seller_read_alias(),
                "seller",
                query,
            )
            .await?;
            Ok(merge_candidate_pages(query, product, seller))
        }
    }
}

async fn fetch_candidates_from_projection(
    client: &(impl GenericClient + Sync),
    query: &SearchQuery,
) -> RepoResult<SearchCandidatePage> {
    match normalized_scope(&query.entity_scope).as_str() {
        "product" | "service" => {
            let mut page = fetch_scope_candidates_from_projection(client, "product", query).await?;
            page.query_scope = normalized_scope(&query.entity_scope);
            Ok(page)
        }
        "seller" => fetch_scope_candidates_from_projection(client, "seller", query).await,
        _ => {
            let product = fetch_scope_candidates_from_projection(client, "product", query).await?;
            let seller = fetch_scope_candidates_from_projection(client, "seller", query).await?;
            Ok(merge_candidate_pages(query, product, seller))
        }
    }
}

fn merge_candidate_pages(
    query: &SearchQuery,
    product: SearchCandidatePage,
    seller: SearchCandidatePage,
) -> SearchCandidatePage {
    let mut hits = product.hits;
    hits.extend(seller.hits);
    sort_candidates(&mut hits, sort_key(&query.sort));
    let page_size = query.page_size.unwrap_or(20).clamp(1, 50) as usize;
    SearchCandidatePage {
        query_scope: "all".to_string(),
        backend: product.backend,
        total: product.total + seller.total,
        hits: hits.into_iter().take(page_size).collect(),
    }
}

async fn fetch_scope_candidates_from_opensearch(
    client: &(impl GenericClient + Sync),
    alias: String,
    entity_scope: &str,
    query: &SearchQuery,
) -> RepoResult<SearchCandidatePage> {
    let ranking_weights = load_active_ranking_weights(client, entity_scope).await?;
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 50);
    let offset = ((page - 1) * page_size) as usize;
    let current_sort = sort_key(&query.sort);
    let body = build_search_request_body(
        entity_scope,
        query,
        offset,
        page_size as usize,
        ranking_weights,
    );
    let response = reqwest::Client::new()
        .post(format!(
            "{}/{}/_search",
            endpoint.trim_end_matches('/'),
            alias
        ))
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("opensearch search request failed: {err}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unavailable".to_string());
        return Err(format!(
            "opensearch search failed: status={status} body={body}"
        ));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|err| format!("opensearch search response decode failed: {err}"))?;
    let total = payload["hits"]["total"]["value"]
        .as_u64()
        .or_else(|| payload["hits"]["total"].as_u64())
        .unwrap_or(0);
    let mut hits = Vec::new();
    if let Some(raw_hits) = payload["hits"]["hits"].as_array() {
        for hit in raw_hits {
            let source = &hit["_source"];
            let entity_id = hit["_id"]
                .as_str()
                .or_else(|| source["id"].as_str())
                .unwrap_or_default()
                .to_string();
            if entity_id.is_empty() {
                continue;
            }
            let score = hit["_score"].as_f64().unwrap_or(0.0);
            let sort_value = if current_sort == "composite" {
                Some(score)
            } else {
                candidate_sort_value(source, entity_scope, current_sort)
            };
            let updated_at = source["updated_at"]
                .as_str()
                .or_else(|| source["source_updated_at"].as_str())
                .map(ToString::to_string);
            hits.push(SearchCandidate {
                entity_scope: entity_scope.to_string(),
                entity_id,
                score,
                sort_value,
                updated_at,
            });
        }
    }
    Ok(SearchCandidatePage {
        query_scope: entity_scope.to_string(),
        backend: CandidateBackend::OpenSearch.as_str().to_string(),
        total,
        hits,
    })
}

async fn fetch_scope_candidates_from_projection(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
    query: &SearchQuery,
) -> RepoResult<SearchCandidatePage> {
    let ranking_weights = load_active_ranking_weights(client, entity_scope).await?;
    let query_term = normalized_query_term(query);
    let tags_filter = (!query.tags.is_empty()).then(|| query.tags.clone());
    let delivery_mode = if entity_scope == "seller" {
        None
    } else {
        query.delivery_mode.clone()
    };
    let product_type = if entity_scope == "product"
        && normalized_scope(&query.entity_scope).as_str() == "service"
    {
        Some("service".to_string())
    } else {
        None
    };
    let limit = query.page_size.unwrap_or(20).clamp(1, 50) as i64;
    let offset =
        ((query.page.unwrap_or(1).max(1) - 1) * query.page_size.unwrap_or(20).clamp(1, 50)) as i64;
    let sql = projection_query_sql(entity_scope, sort_key(&query.sort));
    let rows = client
        .query(
            &sql,
            &[
                &query_term,
                &query.industry,
                &tags_filter,
                &delivery_mode,
                &query.price_min,
                &query.price_max,
                &product_type,
                &limit,
                &offset,
                &ranking_weights.lexical,
                &ranking_weights.quality,
                &ranking_weights.reputation,
                &ranking_weights.freshness,
                &ranking_weights.trade,
                &ranking_weights.completeness,
            ],
        )
        .await
        .map_err(|err| format!("postgres search projection query failed: {err}"))?;

    let total = rows
        .first()
        .map(|row| row.get::<usize, i64>(4) as u64)
        .unwrap_or(0);
    let hits = rows
        .into_iter()
        .map(|row| SearchCandidate {
            entity_scope: entity_scope.to_string(),
            entity_id: row.get(0),
            score: row.get::<usize, f64>(1),
            sort_value: row.get(2),
            updated_at: row.get(3),
        })
        .collect();

    Ok(SearchCandidatePage {
        query_scope: entity_scope.to_string(),
        backend: CandidateBackend::Postgresql.as_str().to_string(),
        total,
        hits,
    })
}

fn build_search_request_body(
    entity_scope: &str,
    query: &SearchQuery,
    offset: usize,
    page_size: usize,
    ranking_weights: RankingWeights,
) -> Value {
    let mut filters = Vec::new();
    if let Some(industry) = query.industry.as_deref() {
        let field = if entity_scope == "seller" {
            "industry_tags.keyword"
        } else {
            "industry.keyword"
        };
        filters.push(json!({ "term": { field: industry } }));
    }
    if let Some(delivery_mode) = query.delivery_mode.as_deref() {
        filters.push(json!({ "term": { "delivery_modes.keyword": delivery_mode } }));
    }
    if !query.tags.is_empty() {
        filters.push(json!({ "terms": { "tags.keyword": query.tags } }));
    }
    if let Some(price_min) = query.price_min {
        filters.push(json!({ "range": { "price_amount": { "gte": price_min } } }));
    }
    if let Some(price_max) = query.price_max {
        filters.push(json!({ "range": { "price_amount": { "lte": price_max } } }));
    }
    if entity_scope != "seller" {
        filters.push(json!({ "term": { "visible_to_search": true } }));
    }
    if entity_scope == "product" && normalized_scope(&query.entity_scope) == "service" {
        filters.push(json!({ "term": { "product_type.keyword": "service" } }));
    }

    let must = if let Some(q) = query.q.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        json!([{ "multi_match": {
            "query": q,
            "fields": [
                "name^4",
                "title^4",
                "subtitle^2",
                "description",
                "seller_name^2",
                "industry",
                "industry_tags^2",
                "certification_tags^2",
                "featured_products.title^2",
                "featured_products.subtitle",
                "country_code",
                "region_code",
                "tags",
                "category"
            ],
            "type": "best_fields"
        }}])
    } else {
        json!([{ "match_all": {} }])
    };

    let current_sort = sort_key(&query.sort);
    let base_query = json!({
        "bool": {
            "must": must,
            "filter": filters
        }
    });
    let request_query = if current_sort == "composite" {
        json!({
            "script_score": {
                "query": base_query,
                "script": {
                    "source": composite_score_script(entity_scope),
                    "params": {
                        "lexical_weight": ranking_weights.lexical,
                        "quality_weight": ranking_weights.quality,
                        "reputation_weight": ranking_weights.reputation,
                        "freshness_weight": ranking_weights.freshness,
                        "trade_weight": ranking_weights.trade,
                        "completeness_weight": ranking_weights.completeness,
                        "now_epoch_ms": current_epoch_millis(),
                    }
                }
            }
        })
    } else {
        base_query
    };

    json!({
        "from": offset,
        "size": page_size,
        "query": request_query,
        "sort": sort_descriptor(entity_scope, current_sort)
    })
}

fn current_epoch_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn composite_score_script(entity_scope: &str) -> &'static str {
    if entity_scope == "seller" {
        r#"
double lexical = _score <= 0.0 ? 0.0 : (_score / (1.0 + _score));
double ageDays = 0.0;
if (doc.containsKey('updated_at') && doc['updated_at'].size() != 0) {
  ageDays = Math.max(0.0, (params.now_epoch_ms - doc['updated_at'].value.toInstant().toEpochMilli()) / 86400000.0);
}
double freshness = 1.0 / (1.0 + ageDays);
double reputation = (doc.containsKey('reputation_score') && doc['reputation_score'].size() != 0)
  ? doc['reputation_score'].value
  : 0.0;
double listingCount = (doc.containsKey('listing_product_count') && doc['listing_product_count'].size() != 0)
  ? doc['listing_product_count'].value
  : 0.0;
double trade = 1.0 - Math.exp(-listingCount / 10.0);
double completenessSignals = 0.0;
if (doc.containsKey('country_code') && doc['country_code'].size() != 0) completenessSignals += 1.0;
if (doc.containsKey('region_code') && doc['region_code'].size() != 0) completenessSignals += 1.0;
if (doc.containsKey('industry_tags') && doc['industry_tags'].size() != 0) completenessSignals += 1.0;
if (doc.containsKey('certification_tags') && doc['certification_tags'].size() != 0) completenessSignals += 1.0;
if (listingCount > 0.0) completenessSignals += 1.0;
if (reputation > 0.0) completenessSignals += 1.0;
double completeness = completenessSignals / 6.0;
return (lexical * params.lexical_weight)
  + (reputation * params.reputation_weight)
  + (freshness * params.freshness_weight)
  + (trade * params.trade_weight)
  + (completeness * params.completeness_weight);
"#
    } else {
        r#"
double lexical = _score <= 0.0 ? 0.0 : (_score / (1.0 + _score));
double ageDays = 0.0;
if (doc.containsKey('updated_at') && doc['updated_at'].size() != 0) {
  ageDays = Math.max(0.0, (params.now_epoch_ms - doc['updated_at'].value.toInstant().toEpochMilli()) / 86400000.0);
}
double freshness = 1.0 / (1.0 + ageDays);
double quality = (doc.containsKey('quality_score') && doc['quality_score'].size() != 0)
  ? doc['quality_score'].value
  : 0.0;
double reputation = (doc.containsKey('seller_reputation_score') && doc['seller_reputation_score'].size() != 0)
  ? doc['seller_reputation_score'].value
  : 0.0;
double trade = (doc.containsKey('hotness_score') && doc['hotness_score'].size() != 0)
  ? doc['hotness_score'].value
  : 0.0;
double completenessSignals = 0.0;
if (doc.containsKey('industry.keyword') && doc['industry.keyword'].size() != 0) completenessSignals += 1.0;
if (doc.containsKey('tags.keyword') && doc['tags.keyword'].size() != 0) completenessSignals += 1.0;
if (doc.containsKey('delivery_modes.keyword') && doc['delivery_modes.keyword'].size() != 0) completenessSignals += 1.0;
if (doc.containsKey('price_amount') && doc['price_amount'].size() != 0 && doc['price_amount'].value > 0.0) completenessSignals += 1.0;
if (doc.containsKey('currency_code.keyword') && doc['currency_code.keyword'].size() != 0) completenessSignals += 1.0;
if (doc.containsKey('seller_id') && doc['seller_id'].size() != 0) completenessSignals += 1.0;
double completeness = completenessSignals / 6.0;
return (lexical * params.lexical_weight)
  + (quality * params.quality_weight)
  + (reputation * params.reputation_weight)
  + (freshness * params.freshness_weight)
  + (trade * params.trade_weight)
  + (completeness * params.completeness_weight);
"#
    }
}

fn sort_descriptor(entity_scope: &str, sort_key: &str) -> Value {
    match sort_key {
        "latest" => json!([{ "updated_at": { "order": "desc", "missing": "_last" } }]),
        "price_asc" => json!([{ "price_amount": { "order": "asc", "missing": "_last" } }]),
        "price_desc" => json!([{ "price_amount": { "order": "desc", "missing": "_last" } }]),
        "quality" => json!([{ "quality_score": { "order": "desc", "missing": "_last" } }]),
        "reputation" => {
            if entity_scope == "seller" {
                json!([{ "reputation_score": { "order": "desc", "missing": "_last" } }])
            } else {
                json!([{ "seller_reputation_score": { "order": "desc", "missing": "_last" } }])
            }
        }
        "hotness" => {
            if entity_scope == "seller" {
                json!([{ "listing_product_count": { "order": "desc", "missing": "_last" } }])
            } else {
                json!([{ "hotness_score": { "order": "desc", "missing": "_last" } }])
            }
        }
        _ => json!([
            { "_score": { "order": "desc" } },
            { "updated_at": { "order": "desc", "missing": "_last" } }
        ]),
    }
}

fn candidate_sort_value(source: &Value, entity_scope: &str, sort_key: &str) -> Option<f64> {
    match sort_key {
        "price_asc" | "price_desc" => source["price_amount"].as_f64(),
        "quality" => source["quality_score"].as_f64(),
        "reputation" => {
            if entity_scope == "seller" {
                source["reputation_score"].as_f64()
            } else {
                source["seller_reputation_score"].as_f64()
            }
        }
        "hotness" => {
            if entity_scope == "seller" {
                source["listing_product_count"].as_i64().map(|v| v as f64)
            } else {
                source["hotness_score"].as_f64()
            }
        }
        _ => None,
    }
}

fn sort_candidates(candidates: &mut [SearchCandidate], sort_key: &str) {
    candidates.sort_by(|left, right| match sort_key {
        "composite" => right
            .sort_value
            .unwrap_or(right.score)
            .partial_cmp(&left.sort_value.unwrap_or(left.score))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| right.updated_at.cmp(&left.updated_at)),
        "price_asc" => left
            .sort_value
            .partial_cmp(&right.sort_value)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
        "price_desc" | "quality" | "reputation" | "hotness" => right
            .sort_value
            .partial_cmp(&left.sort_value)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
        "latest" => right.updated_at.cmp(&left.updated_at).then_with(|| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        _ => right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.updated_at.cmp(&left.updated_at)),
    });
}

async fn fetch_product_result(
    client: &(impl GenericClient + Sync),
    product_id: &str,
    score: f64,
) -> RepoResult<Option<SearchResultItem>> {
    let row = client
        .query_opt(
            "SELECT
               p.product_id::text,
               p.title,
               NULLIF(p.metadata ->> 'subtitle', ''),
               p.description,
               p.seller_org_id::text,
               spd.seller_name,
               p.category,
               p.product_type,
               p.status,
               p.price::text,
               p.currency_code,
               ARRAY[p.delivery_type]::text[],
               COALESCE(spd.tags, '{}')::text[],
               COALESCE(spd.seller_industry_tags, '{}')::text[],
               COALESCE(spd.seller_reputation_score, 0)::text,
               COALESCE(spd.quality_score, 0)::text,
               COALESCE(spd.hotness_score, 0)::text,
               COALESCE(spd.document_version, 0)::bigint,
               COALESCE(spd.index_sync_status, 'pending')
             FROM catalog.product p
             JOIN search.product_search_document spd ON spd.product_id = p.product_id
             JOIN core.organization org ON org.org_id = p.seller_org_id
             WHERE p.product_id = $1::text::uuid
               AND p.status = 'listed'
               AND spd.listing_status = 'listed'
               AND org.status = 'active'
               AND lower(COALESCE(p.metadata ->> 'risk_blocked', 'false')) NOT IN ('true', '1')
               AND lower(COALESCE(p.metadata #>> '{risk_flags,block_submit}', 'false')) NOT IN ('true', '1')",
            &[&product_id],
        )
        .await
        .map_err(|err| format!("load product final check failed: {err}"))?;

    Ok(row.map(|row| SearchResultItem {
        entity_scope: "product".to_string(),
        entity_id: row.get(0),
        score,
        title: row.get(1),
        subtitle: row.get(2),
        description: row.get(3),
        seller_org_id: row.get(4),
        seller_name: row.get(5),
        category: row.get(6),
        product_type: row.get(7),
        status: row.get(8),
        price: row.get(9),
        currency_code: row.get(10),
        delivery_modes: row.get(11),
        tags: row.get(12),
        industry_tags: row.get(13),
        country_code: None,
        reputation_score: row.get(14),
        quality_score: row.get(15),
        hotness_score: row.get(16),
        listing_product_count: None,
        document_version: row.get(17),
        index_sync_status: row.get(18),
    }))
}

async fn fetch_seller_result(
    client: &(impl GenericClient + Sync),
    org_id: &str,
    score: f64,
) -> RepoResult<Option<SearchResultItem>> {
    let row = client
        .query_opt(
            "SELECT
               org.org_id::text,
               org.org_name,
               NULL,
               COALESCE(ssd.description, ''),
               NULL,
               org.org_name,
               NULL,
               NULL,
               org.status,
               NULL,
               NULL,
               '{}'::text[],
               '{}'::text[],
               COALESCE(ssd.industry_tags, '{}')::text[],
               COALESCE(ssd.reputation_score, 0)::text,
               NULL,
               NULL,
               COALESCE(ssd.listing_product_count, 0)::bigint,
               COALESCE(ssd.document_version, 0)::bigint,
               COALESCE(ssd.index_sync_status, 'pending'),
               org.country_code
             FROM core.organization org
             JOIN search.seller_search_document ssd ON ssd.org_id = org.org_id
             WHERE org.org_id = $1::text::uuid
               AND org.status NOT IN ('suspended', 'frozen')",
            &[&org_id],
        )
        .await
        .map_err(|err| format!("load seller final check failed: {err}"))?;

    Ok(row.map(|row| SearchResultItem {
        entity_scope: "seller".to_string(),
        entity_id: row.get(0),
        score,
        title: row.get(1),
        subtitle: row.get(2),
        description: row.get(3),
        seller_org_id: row.get(4),
        seller_name: row.get(5),
        category: row.get(6),
        product_type: row.get(7),
        status: row.get(8),
        price: row.get(9),
        currency_code: row.get(10),
        delivery_modes: row.get(11),
        tags: row.get(12),
        industry_tags: row.get(13),
        country_code: row.get(20),
        reputation_score: row.get(14),
        quality_score: row.get(15),
        hotness_score: row.get(16),
        listing_product_count: row.get(17),
        document_version: row.get(18),
        index_sync_status: row.get(19),
    }))
}

fn projection_query_sql(entity_scope: &str, sort_key: &str) -> String {
    let order_by = projection_order_clause(entity_scope, sort_key);
    let from_clause = if entity_scope == "seller" {
        "FROM search.seller_search_document d"
    } else {
        "FROM search.product_search_document d"
    };
    let industry_filter = if entity_scope == "seller" {
        "AND ($2::text IS NULL OR $2 = ANY(d.industry_tags))"
    } else {
        "AND ($2::text IS NULL OR d.industry = $2)"
    };
    let tags_filter = if entity_scope == "seller" {
        "AND ($3::text[] IS NULL OR d.industry_tags && $3::text[])"
    } else {
        "AND ($3::text[] IS NULL OR d.tags && $3::text[])"
    };
    let product_type_filter = if entity_scope == "seller" {
        "AND ($7::text IS NULL OR true)".to_string()
    } else {
        "AND ($7::text IS NULL OR d.product_type = $7)".to_string()
    };
    let delivery_filter = if entity_scope == "seller" {
        "AND ($4::text IS NULL OR true)".to_string()
    } else {
        "AND ($4::text IS NULL OR $4 = ANY(d.delivery_modes))".to_string()
    };
    let price_min_filter = if entity_scope == "seller" {
        "AND ($5::float8 IS NULL OR true)".to_string()
    } else {
        "AND ($5::float8 IS NULL OR COALESCE(d.price_amount::float8, 0) >= $5)".to_string()
    };
    let price_max_filter = if entity_scope == "seller" {
        "AND ($6::float8 IS NULL OR true)".to_string()
    } else {
        "AND ($6::float8 IS NULL OR COALESCE(d.price_amount::float8, 0) <= $6)".to_string()
    };
    let visibility_filter = if entity_scope == "seller" {
        String::new()
    } else {
        "AND COALESCE(d.visible_to_search, false)".to_string()
    };
    let sort_value_expr = projection_sort_value_expr(entity_scope, sort_key);

    format!(
        "WITH params AS (
           SELECT CASE
                    WHEN $1::text IS NULL THEN NULL::tsquery
                    ELSE websearch_to_tsquery('simple', $1)
                  END AS ts_query,
                  $10::float8 AS lexical_weight,
                  $11::float8 AS quality_weight,
                  $12::float8 AS reputation_weight,
                  $13::float8 AS freshness_weight,
                  $14::float8 AS trade_weight,
                  $15::float8 AS completeness_weight
         ),
         scoped AS (
           SELECT
             {entity_id_expr} AS entity_id,
             CASE
               WHEN params.ts_query IS NULL THEN 0::float8
               ELSE ts_rank_cd(d.searchable_tsv, params.ts_query)::float8
             END AS score,
             {sort_value_expr} AS sort_value,
             to_char({updated_at_expr} AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') AS updated_at
           {from_clause}
           CROSS JOIN params
           WHERE (params.ts_query IS NULL OR d.searchable_tsv @@ params.ts_query)
             {industry_filter}
             {tags_filter}
             {delivery_filter}
             {price_min_filter}
             {price_max_filter}
             {visibility_filter}
             {product_type_filter}
         )
         SELECT
           entity_id,
           score,
           sort_value,
           updated_at,
           COUNT(*) OVER()::bigint AS total
         FROM scoped
         ORDER BY {order_by}
         LIMIT $8
         OFFSET $9",
        entity_id_expr = if entity_scope == "seller" {
            "d.org_id::text"
        } else {
            "d.product_id::text"
        },
        sort_value_expr = sort_value_expr,
        updated_at_expr = if entity_scope == "seller" {
            "d.source_updated_at"
        } else {
            "d.source_updated_at"
        },
        from_clause = from_clause,
        industry_filter = industry_filter,
        tags_filter = tags_filter,
        delivery_filter = delivery_filter,
        price_min_filter = price_min_filter,
        price_max_filter = price_max_filter,
        visibility_filter = visibility_filter,
        product_type_filter = product_type_filter,
        order_by = order_by,
    )
}

fn projection_sort_value_expr(entity_scope: &str, sort_key: &str) -> String {
    match sort_key {
        "price_asc" | "price_desc" => "COALESCE(d.price_amount::float8, 0)".to_string(),
        "quality" => "COALESCE(d.quality_score::float8, 0)".to_string(),
        "reputation" if entity_scope == "seller" => {
            "COALESCE(d.reputation_score::float8, 0)".to_string()
        }
        "reputation" => "COALESCE(d.seller_reputation_score::float8, 0)".to_string(),
        "hotness" if entity_scope == "seller" => {
            "COALESCE(d.listing_product_count::float8, 0)".to_string()
        }
        "hotness" => "COALESCE(d.hotness_score::float8, 0)".to_string(),
        _ => projection_composite_sort_value_expr(entity_scope),
    }
}

fn projection_composite_sort_value_expr(entity_scope: &str) -> String {
    let lexical_score = "CASE
               WHEN params.ts_query IS NULL THEN 0::float8
               ELSE ts_rank_cd(d.searchable_tsv, params.ts_query)::float8
             END";
    let freshness_score =
        "1.0 / (1.0 + GREATEST(EXTRACT(EPOCH FROM (now() - d.source_updated_at)) / 86400.0, 0.0))";
    let quality_score = if entity_scope == "seller" {
        "0::float8"
    } else {
        "COALESCE(d.quality_score::float8, 0)"
    };
    let reputation_score = if entity_scope == "seller" {
        "COALESCE(d.reputation_score::float8, 0)"
    } else {
        "COALESCE(d.seller_reputation_score::float8, 0)"
    };
    let trade_score = if entity_scope == "seller" {
        "(1 - exp(-GREATEST(COALESCE(d.listing_product_count::float8, 0), 0) / 10.0))"
    } else {
        "COALESCE(d.hotness_score::float8, 0)"
    };
    let completeness_score = if entity_scope == "seller" {
        "(
            (CASE WHEN NULLIF(d.country_code, '') IS NULL THEN 0 ELSE 1 END)
          + (CASE WHEN NULLIF(d.region_code, '') IS NULL THEN 0 ELSE 1 END)
          + (CASE WHEN cardinality(COALESCE(d.industry_tags, '{}')) = 0 THEN 0 ELSE 1 END)
          + (CASE WHEN cardinality(COALESCE(d.certification_tags, '{}')) = 0 THEN 0 ELSE 1 END)
          + (CASE WHEN COALESCE(d.listing_product_count, 0) > 0 THEN 1 ELSE 0 END)
          + (CASE WHEN COALESCE(d.reputation_score::float8, 0) > 0 THEN 1 ELSE 0 END)
          )::float8 / 6.0"
    } else {
        "(
            (CASE WHEN NULLIF(d.industry, '') IS NULL THEN 0 ELSE 1 END)
          + (CASE WHEN cardinality(COALESCE(d.tags, '{}')) = 0 THEN 0 ELSE 1 END)
          + (CASE WHEN cardinality(COALESCE(d.delivery_modes, '{}')) = 0 THEN 0 ELSE 1 END)
          + (CASE WHEN COALESCE(d.price_amount::float8, 0) > 0 THEN 1 ELSE 0 END)
          + (CASE WHEN NULLIF(d.currency_code, '') IS NULL THEN 0 ELSE 1 END)
          + (CASE WHEN d.org_id IS NULL THEN 0 ELSE 1 END)
          )::float8 / 6.0"
    };

    format!(
        "((CASE
             WHEN {lexical_score} <= 0 THEN 0
             ELSE {lexical_score} / (1 + {lexical_score})
           END) * params.lexical_weight)
         + (({quality_score}) * params.quality_weight)
         + (({reputation_score}) * params.reputation_weight)
         + (({freshness_score}) * params.freshness_weight)
         + (({trade_score}) * params.trade_weight)
         + (({completeness_score}) * params.completeness_weight)",
        lexical_score = lexical_score,
        quality_score = quality_score,
        reputation_score = reputation_score,
        freshness_score = freshness_score,
        trade_score = trade_score,
        completeness_score = completeness_score,
    )
}

fn projection_order_clause(entity_scope: &str, sort_key: &str) -> String {
    match sort_key {
        "latest" => "updated_at DESC, score DESC, entity_id ASC".to_string(),
        "price_asc" => {
            "sort_value ASC NULLS LAST, score DESC, updated_at DESC, entity_id ASC".to_string()
        }
        "price_desc" | "quality" | "reputation" | "hotness" => {
            "sort_value DESC NULLS LAST, score DESC, updated_at DESC, entity_id ASC".to_string()
        }
        _ if entity_scope == "seller" => {
            "sort_value DESC NULLS LAST, score DESC, updated_at DESC, entity_id ASC".to_string()
        }
        _ => "sort_value DESC NULLS LAST, score DESC, updated_at DESC, entity_id ASC".to_string(),
    }
}

fn normalized_query_term(query: &SearchQuery) -> Option<String> {
    query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

async fn load_candidate_cache(
    query: &SearchQuery,
    backend: CandidateBackend,
) -> RepoResult<Option<SearchCandidatePage>> {
    let redis_url = redis_url();
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|err| format!("redis client init failed: {err}"))?;
    let mut connection = match client.get_multiplexed_async_connection().await {
        Ok(connection) => connection,
        Err(err) => return Err(format!("redis connect failed: {err}")),
    };
    let key = search_cache_key(query, backend)?;
    let query_scope = normalized_scope(&query.entity_scope);
    let expected_version = load_search_cache_version(&mut connection, &query_scope).await?;
    let value: Option<String> = connection
        .get(&key)
        .await
        .map_err(|err| format!("redis cache get failed: {err}"))?;
    match value {
        Some(serialized) => {
            if let Ok(cached) = serde_json::from_str::<CachedCandidatePage>(&serialized) {
                if cached.cache_version == expected_version {
                    return Ok(Some(cached.page));
                }
            } else if let Ok(page) = serde_json::from_str::<SearchCandidatePage>(&serialized) {
                if expected_version == 0 {
                    return Ok(Some(page));
                }
            } else {
                return Err(
                    "decode cached search candidates failed: payload format mismatch".to_string(),
                );
            }
            let _ = connection.del::<_, usize>(&key).await;
            Ok(None)
        }
        None => Ok(None),
    }
}

async fn store_candidate_cache(
    query: &SearchQuery,
    backend: CandidateBackend,
    page: &SearchCandidatePage,
) -> RepoResult<()> {
    let redis_url = redis_url();
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|err| format!("redis client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis connect failed: {err}"))?;
    let key = search_cache_key(query, backend)?;
    let query_scope = normalized_scope(&query.entity_scope);
    let cache_version = load_search_cache_version(&mut connection, &query_scope).await?;
    let serialized = serde_json::to_string(&CachedCandidatePage {
        cache_version,
        page: page.clone(),
    })
    .map_err(|err| format!("encode cached search candidates failed: {err}"))?;
    connection
        .set_ex::<_, _, ()>(key, serialized, SEARCH_CACHE_TTL_SECS)
        .await
        .map_err(|err| format!("redis cache set failed: {err}"))?;
    Ok(())
}

async fn ensure_index_exists(index_name: &str) -> RepoResult<()> {
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());
    let response = reqwest::Client::new()
        .get(format!("{}/{}", endpoint.trim_end_matches('/'), index_name))
        .send()
        .await
        .map_err(|err| format!("opensearch index probe failed: {err}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "opensearch target index missing: {} (status={})",
            index_name,
            response.status()
        ))
    }
}

async fn load_alias_binding(
    client: &(impl GenericClient + Sync),
    scope: &str,
) -> RepoResult<AliasBinding> {
    let normalized = normalized_scope(scope);
    if !matches!(normalized.as_str(), "product" | "seller") {
        return Err(format!("unsupported alias entity_scope: {scope}"));
    }
    let row = client
        .query_opt(
            "SELECT
               alias_binding_id::text,
               entity_scope,
               read_alias,
               write_alias,
               active_index_name
             FROM search.index_alias_binding
             WHERE entity_scope = $1
               AND backend_type = 'opensearch'
             LIMIT 1",
            &[&normalized],
        )
        .await
        .map_err(|err| format!("load search.index_alias_binding failed: {err}"))?
        .ok_or_else(|| format!("search.index_alias_binding missing for scope={normalized}"))?;
    Ok(AliasBinding {
        alias_binding_id: row.get(0),
        entity_scope: row.get(1),
        read_alias: row.get(2),
        write_alias: row.get(3),
        active_index_name: row.get(4),
    })
}

pub async fn get_alias_binding_id(
    client: &(impl GenericClient + Sync),
    scope: &str,
) -> RepoResult<String> {
    load_alias_binding(client, scope)
        .await
        .map(|binding| binding.alias_binding_id)
}

async fn resolve_target_index(
    client: &(impl GenericClient + Sync),
    scope: &str,
    requested: Option<&str>,
) -> RepoResult<Option<String>> {
    if let Some(requested) = requested {
        return Ok(Some(requested.to_string()));
    }
    if scope == "all" {
        return Ok(None);
    }
    let binding = load_alias_binding(client, scope).await?;
    Ok(binding.active_index_name)
}

async fn load_document_version(
    client: &(impl GenericClient + Sync),
    scope: &str,
    entity_id: &str,
) -> RepoResult<i64> {
    let row = if scope == "seller" {
        client
            .query_one(
                "SELECT document_version::bigint
                 FROM search.seller_search_document
                 WHERE org_id = $1::text::uuid",
                &[&entity_id],
            )
            .await
    } else {
        client
            .query_one(
                "SELECT document_version::bigint
                 FROM search.product_search_document
                 WHERE product_id = $1::text::uuid",
                &[&entity_id],
            )
            .await
    }
    .map_err(|err| format!("load search projection document version failed: {err}"))?;
    Ok(row.get(0))
}

fn product_read_alias() -> String {
    std::env::var("INDEX_ALIAS_PRODUCT_SEARCH_READ")
        .unwrap_or_else(|_| "product_search_read".to_string())
}

fn seller_read_alias() -> String {
    std::env::var("INDEX_ALIAS_SELLER_SEARCH_READ")
        .unwrap_or_else(|_| "seller_search_read".to_string())
}

fn redis_namespace() -> String {
    std::env::var("REDIS_NAMESPACE").unwrap_or_else(|_| "datab:v1".to_string())
}

fn redis_url() -> String {
    if let Ok(url) = std::env::var("REDIS_URL") {
        if !url.trim().is_empty() {
            return url;
        }
    }
    let host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
    let password =
        std::env::var("REDIS_PASSWORD").unwrap_or_else(|_| "datab_redis_pass".to_string());
    format!("redis://default:{}@{}:{}/0", password, host, port)
}

fn all_search_cache_scopes() -> Vec<String> {
    vec![
        "all".to_string(),
        "product".to_string(),
        "service".to_string(),
        "seller".to_string(),
    ]
}

fn related_cache_scopes_for_ops(scope: &str) -> Vec<String> {
    match normalized_scope(scope).as_str() {
        "seller" => vec!["seller".to_string(), "all".to_string()],
        "product" | "service" => vec![
            "product".to_string(),
            "service".to_string(),
            "all".to_string(),
        ],
        _ => all_search_cache_scopes(),
    }
}

fn search_cache_pattern(scope: &str) -> String {
    format!(
        "{}:search:catalog:{}:*",
        redis_namespace(),
        normalized_scope(scope)
    )
}

fn search_cache_version_key(scope: &str) -> String {
    format!(
        "{}:search:catalog:version:{}",
        redis_namespace(),
        normalized_scope(scope)
    )
}

async fn load_search_cache_version(
    connection: &mut redis::aio::MultiplexedConnection,
    scope: &str,
) -> RepoResult<i64> {
    let version = connection
        .get::<_, Option<i64>>(search_cache_version_key(scope))
        .await
        .map_err(|err| format!("redis cache version lookup failed: {err}"))?;
    Ok(version.unwrap_or(0))
}

async fn bump_search_cache_versions(
    connection: &mut redis::aio::MultiplexedConnection,
    scopes: &[String],
) -> RepoResult<()> {
    let mut unique_scopes = BTreeSet::new();
    unique_scopes.extend(scopes.iter().cloned());
    for scope in unique_scopes {
        connection
            .incr::<_, _, i64>(search_cache_version_key(&scope), 1)
            .await
            .map_err(|err| format!("redis cache version bump failed: {err}"))?;
    }
    Ok(())
}

async fn delete_search_cache_patterns(
    connection: &mut redis::aio::MultiplexedConnection,
    patterns: &[String],
) -> RepoResult<usize> {
    let mut keys = BTreeSet::new();
    for pattern in patterns {
        keys.extend(scan_search_cache_keys(connection, pattern).await?);
    }
    if keys.is_empty() {
        return Ok(0);
    }
    connection
        .del::<_, usize>(keys.into_iter().collect::<Vec<_>>())
        .await
        .map_err(|err| format!("redis cache batch delete failed: {err}"))
}

async fn scan_search_cache_keys(
    connection: &mut redis::aio::MultiplexedConnection,
    pattern: &str,
) -> RepoResult<Vec<String>> {
    let mut cursor = 0u64;
    let mut keys = Vec::new();
    loop {
        let (next_cursor, batch): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(200)
            .query_async(connection)
            .await
            .map_err(|err| format!("redis search cache scan failed: {err}"))?;
        keys.extend(batch);
        if next_cursor == 0 {
            break;
        }
        cursor = next_cursor;
    }
    Ok(keys)
}

fn search_cache_key(query: &SearchQuery, backend: CandidateBackend) -> RepoResult<String> {
    let canonical = serde_json::to_string(&json!({
        "backend": backend.as_str(),
        "query": query,
    }))
    .map_err(|err| format!("encode search cache fingerprint failed: {err}"))?;
    let digest = format!("{:x}", Sha256::digest(canonical.as_bytes()));
    Ok(format!(
        "{}:search:catalog:{}:{}",
        redis_namespace(),
        normalized_scope(&query.entity_scope),
        digest
    ))
}

fn normalized_scope(scope: &str) -> String {
    match scope.trim().to_ascii_lowercase().as_str() {
        "product" => "product".to_string(),
        "seller" => "seller".to_string(),
        "service" => "service".to_string(),
        _ => "all".to_string(),
    }
}

fn sort_key(value: &str) -> &str {
    match value.trim().to_ascii_lowercase().as_str() {
        "latest" => "latest",
        "price_asc" => "price_asc",
        "price_desc" => "price_desc",
        "quality" => "quality",
        "reputation" => "reputation",
        "hotness" => "hotness",
        _ => "composite",
    }
}
