use db::GenericClient;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::modules::search::domain::{
    AliasSwitchRequest, AliasSwitchResponse, CacheInvalidateRequest, CacheInvalidateResponse,
    PatchRankingProfileRequest, RankingProfileView, ReindexRequest, ReindexResponse, SearchQuery,
    SearchResultItem, SearchSyncQuery, SearchSyncTaskView,
};

type RepoResult<T> = Result<T, String>;

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

pub async fn search_catalog_candidates(
    query: &SearchQuery,
) -> RepoResult<(SearchCandidatePage, bool)> {
    if let Some(cached) = load_candidate_cache(query).await? {
        return Ok((cached, true));
    }

    let page = match normalized_scope(&query.entity_scope).as_str() {
        "product" | "service" => {
            let mut page = fetch_scope_candidates(product_read_alias(), "product", query).await?;
            page.query_scope = normalized_scope(&query.entity_scope);
            page
        }
        "seller" => fetch_scope_candidates(seller_read_alias(), "seller", query).await?,
        _ => {
            let product = fetch_scope_candidates(product_read_alias(), "product", query).await?;
            let seller = fetch_scope_candidates(seller_read_alias(), "seller", query).await?;
            let mut hits = product.hits;
            hits.extend(seller.hits);
            sort_candidates(&mut hits, sort_key(&query.sort));
            let page_size = query.page_size.unwrap_or(20).clamp(1, 50) as usize;
            SearchCandidatePage {
                query_scope: "all".to_string(),
                total: product.total + seller.total,
                hits: hits.into_iter().take(page_size).collect(),
            }
        }
    };

    store_candidate_cache(query, &page).await?;
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
    let redis_url = redis_url();
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|err| format!("redis client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis connect failed: {err}"))?;

    let deleted = if let Some(query_hash) = request.query_hash.as_deref() {
        let scope = request
            .entity_scope
            .clone()
            .unwrap_or_else(|| "all".to_string());
        let key = format!(
            "{}:search:catalog:{}:{}",
            redis_namespace(),
            scope,
            query_hash
        );
        connection
            .del::<_, usize>(key)
            .await
            .map_err(|err| format!("redis cache delete failed: {err}"))?
    } else {
        let pattern = if request.purge_all.unwrap_or(false) {
            format!("{}:search:catalog:*", redis_namespace())
        } else if let Some(scope) = request.entity_scope.as_deref() {
            format!(
                "{}:search:catalog:{}:*",
                redis_namespace(),
                normalized_scope(scope)
            )
        } else {
            format!("{}:search:catalog:*", redis_namespace())
        };
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut connection)
            .await
            .map_err(|err| format!("redis keys lookup failed: {err}"))?;
        if keys.is_empty() {
            0
        } else {
            connection
                .del::<_, usize>(keys)
                .await
                .map_err(|err| format!("redis cache batch delete failed: {err}"))?
        }
    };

    Ok(CacheInvalidateResponse {
        entity_scope: request.entity_scope.clone(),
        deleted_keys: deleted,
    })
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

async fn fetch_scope_candidates(
    alias: String,
    entity_scope: &str,
    query: &SearchQuery,
) -> RepoResult<SearchCandidatePage> {
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 50);
    let offset = ((page - 1) * page_size) as usize;
    let body = build_search_request_body(entity_scope, query, offset, page_size as usize);
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
            let sort_value = candidate_sort_value(source, entity_scope, sort_key(&query.sort));
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
        total,
        hits,
    })
}

fn build_search_request_body(
    entity_scope: &str,
    query: &SearchQuery,
    offset: usize,
    page_size: usize,
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
    if entity_scope == "product" && normalized_scope(&query.entity_scope) == "service" {
        filters.push(json!({ "term": { "product_type.keyword": "service" } }));
    }

    let must = if let Some(q) = query.q.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        json!([{ "multi_match": {
            "query": q,
            "fields": ["name^4", "title^4", "subtitle^2", "description", "seller_name^2", "industry", "tags", "category"],
            "type": "best_fields"
        }}])
    } else {
        json!([{ "match_all": {} }])
    };

    json!({
        "from": offset,
        "size": page_size,
        "query": {
            "bool": {
                "must": must,
                "filter": filters
            }
        },
        "sort": sort_descriptor(entity_scope, sort_key(&query.sort))
    })
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

async fn load_candidate_cache(query: &SearchQuery) -> RepoResult<Option<SearchCandidatePage>> {
    let redis_url = redis_url();
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|err| format!("redis client init failed: {err}"))?;
    let mut connection = match client.get_multiplexed_async_connection().await {
        Ok(connection) => connection,
        Err(err) => return Err(format!("redis connect failed: {err}")),
    };
    let key = search_cache_key(query)?;
    let value: Option<String> = connection
        .get(key)
        .await
        .map_err(|err| format!("redis cache get failed: {err}"))?;
    match value {
        Some(serialized) => serde_json::from_str(&serialized)
            .map(Some)
            .map_err(|err| format!("decode cached search candidates failed: {err}")),
        None => Ok(None),
    }
}

async fn store_candidate_cache(query: &SearchQuery, page: &SearchCandidatePage) -> RepoResult<()> {
    let redis_url = redis_url();
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|err| format!("redis client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis connect failed: {err}"))?;
    let key = search_cache_key(query)?;
    let serialized = serde_json::to_string(page)
        .map_err(|err| format!("encode cached search candidates failed: {err}"))?;
    connection
        .set_ex::<_, _, ()>(key, serialized, 300)
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
    format!("redis://:{}@{}:{}/0", password, host, port)
}

fn search_cache_key(query: &SearchQuery) -> RepoResult<String> {
    let canonical = serde_json::to_string(query)
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
