use super::authorization_header;
use crate::modules::search::api::router;
use crate::modules::search::domain::SearchQuery;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use redis::AsyncCommands;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("SEARCH_DB_SMOKE").ok().as_deref() == Some("1")
}

static SEARCH_ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<String>,
}

impl ScopedEnvVar {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        // SAFETY: test-only mutation guarded by SEARCH_ENV_TEST_LOCK.
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            // SAFETY: test-only mutation guarded by SEARCH_ENV_TEST_LOCK.
            unsafe { std::env::set_var(self.key, previous) };
        } else {
            // SAFETY: test-only mutation guarded by SEARCH_ENV_TEST_LOCK.
            unsafe { std::env::remove_var(self.key) };
        }
    }
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    asset_id: String,
    asset_version_id: String,
    product_id: String,
    operator_user_id: String,
}

#[derive(Debug)]
struct AliasBindingSeed {
    alias_binding_id: String,
    read_alias: String,
    write_alias: String,
    active_index_name: String,
}

#[derive(Debug)]
struct RankingProfileSeed {
    ranking_profile_id: String,
    weights_json: Value,
    filter_policy_json: Value,
    status: String,
}

async fn seed_minimum_graph(client: &Client, suffix: &str) -> Result<SeedIds, Error> {
    let org = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, metadata
             ) VALUES (
               $1::text, 'enterprise', 'active', jsonb_build_object('description', $2::text)
             )
             RETURNING org_id::text",
            &[
                &format!("search-org-{suffix}"),
                &format!("search seller {suffix}"),
            ],
        )
        .await?;
    let org_id: String = org.get(0);

    let asset = client
        .query_one(
            "INSERT INTO catalog.data_asset (
               owner_org_id, title, category, sensitivity_level, status
             ) VALUES (
               $1::text::uuid, $2, 'manufacturing', 'internal', 'draft'
             )
             RETURNING asset_id::text",
            &[&org_id, &format!("search-asset-{suffix}")],
        )
        .await?;
    let asset_id: String = asset.get(0);

    let asset_version = client
        .query_one(
            "INSERT INTO catalog.asset_version (
               asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
               data_size_bytes, origin_region, allowed_region, requires_controlled_execution, trust_boundary_snapshot, status
             ) VALUES (
               $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
               1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
             )
             RETURNING asset_version_id::text",
            &[&asset_id],
        )
        .await?;
    let asset_version_id: String = asset_version.get(0);

    let product = client
        .query_one(
            "INSERT INTO catalog.product (
               asset_id, asset_version_id, seller_org_id, title, category, product_type,
               description, status, price_mode, price, currency_code, delivery_type,
               allowed_usage, searchable_text, metadata
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
               $5, 'listed', 'one_time', 88.00, 'CNY', 'file_download',
               ARRAY['internal_use']::text[], $6,
               jsonb_build_object(
                 'subtitle', $7,
                 'industry', 'industrial_manufacturing',
                 'quality_score', '0.92',
                 'use_cases', jsonb_build_array('质量日报')
               )
             )
             RETURNING product_id::text",
            &[
                &asset_id,
                &asset_version_id,
                &org_id,
                &format!("search-product-{suffix}"),
                &format!("search product {suffix}"),
                &format!("quality search keyword {suffix}"),
                &format!("search subtitle {suffix}"),
            ],
        )
        .await?;
    let product_id: String = product.get(0);

    client
        .execute(
            "SELECT search.refresh_product_search_document_by_id($1::text::uuid)",
            &[&product_id],
        )
        .await?;
    client
        .execute(
            "SELECT search.refresh_seller_search_document_by_id($1::text::uuid)",
            &[&org_id],
        )
        .await?;

    let operator_user_id = seed_operator_user(client, &org_id, suffix).await?;

    Ok(SeedIds {
        org_id,
        asset_id,
        asset_version_id,
        product_id,
        operator_user_id,
    })
}

async fn seed_operator_user(client: &Client, org_id: &str, suffix: &str) -> Result<String, Error> {
    client
        .query_one(
            "INSERT INTO core.user_account (
               org_id, login_id, display_name, user_type, status, mfa_status, attrs
             ) VALUES (
               $1::text::uuid, $2, $3, 'person', 'active', 'verified', jsonb_build_object('seed', $4)
             )
             RETURNING user_id::text",
            &[
                &org_id,
                &format!("search-ops-user-{suffix}"),
                &format!("Search Ops User {suffix}"),
                &suffix,
            ],
        )
        .await
        .map(|row| row.get(0))
}

async fn seed_verified_step_up_challenge(
    client: &Client,
    user_id: &str,
    target_action: &str,
    target_ref_type: &str,
    target_ref_id: Option<&str>,
    seed_label: &str,
) -> Result<String, Error> {
    client
        .query_one(
            "INSERT INTO iam.step_up_challenge (
               user_id,
               challenge_type,
               target_action,
               target_ref_type,
               target_ref_id,
               challenge_status,
               expires_at,
               completed_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               'mock_otp',
               $2,
               $3,
               $4::text::uuid,
               'verified',
               now() + interval '10 minutes',
               now(),
               jsonb_build_object('seed', $5)
             )
             RETURNING step_up_challenge_id::text",
            &[
                &user_id,
                &target_action,
                &target_ref_type,
                &target_ref_id,
                &seed_label,
            ],
        )
        .await
        .map(|row| row.get(0))
}

async fn lookup_product_alias_binding(client: &Client) -> Result<AliasBindingSeed, Error> {
    let row = client
        .query_one(
            "SELECT
               alias_binding_id::text,
               read_alias,
               write_alias,
               active_index_name
             FROM search.index_alias_binding
             WHERE entity_scope = 'product'
             LIMIT 1",
            &[],
        )
        .await?;
    Ok(AliasBindingSeed {
        alias_binding_id: row.get(0),
        read_alias: row.get(1),
        write_alias: row.get(2),
        active_index_name: row.get(3),
    })
}

async fn lookup_product_ranking_profile(client: &Client) -> Result<RankingProfileSeed, Error> {
    let row = client
        .query_one(
            "SELECT
               ranking_profile_id::text,
               weights_json,
               filter_policy_json,
               status
             FROM search.ranking_profile
             WHERE entity_scope = 'product'
             ORDER BY created_at
             LIMIT 1",
            &[],
        )
        .await?;
    Ok(RankingProfileSeed {
        ranking_profile_id: row.get(0),
        weights_json: row.get(1),
        filter_policy_json: row.get(2),
        status: row.get(3),
    })
}

async fn cleanup_seed_graph(
    client: &Client,
    ids: &SeedIds,
    step_up_ids: &[String],
    ranking_seed: &RankingProfileSeed,
) {
    let _ = client
        .execute(
            "UPDATE search.ranking_profile
             SET weights_json = $2::jsonb,
                 filter_policy_json = $3::jsonb,
                 status = $4,
                 updated_at = now()
             WHERE ranking_profile_id = $1::text::uuid",
            &[
                &ranking_seed.ranking_profile_id,
                &ranking_seed.weights_json,
                &ranking_seed.filter_policy_json,
                &ranking_seed.status,
            ],
        )
        .await;
    for challenge_id in step_up_ids {
        let _ = client
            .execute(
                "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
                &[challenge_id],
            )
            .await;
    }
    let _ = client
        .execute(
            "DELETE FROM search.index_sync_task WHERE entity_id IN ($1::text::uuid, $2::text::uuid)",
            &[&ids.product_id, &ids.org_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
            &[&ids.operator_user_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
            &[&ids.product_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
            &[&ids.asset_version_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
            &[&ids.asset_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await;
}

async fn seed_opensearch_documents(ids: &SeedIds, suffix: &str) -> Result<(), String> {
    let endpoint = opensearch_endpoint();
    let client = reqwest::Client::new();
    let product_doc = json!({
        "entity_scope": "product",
        "id": ids.product_id,
        "seller_id": ids.org_id,
        "name": format!("search-product-{suffix}"),
        "title": format!("search-product-{suffix}"),
        "subtitle": format!("search subtitle {suffix}"),
        "description": format!("quality search keyword {suffix}"),
        "industry": "industrial_manufacturing",
        "status": "listed",
        "review_status": "approved",
        "visibility_status": "visible",
        "visible_to_search": true,
        "price_amount": 88.0,
        "quality_score": 0.92,
        "seller_reputation_score": 0.0,
        "hotness_score": 0.0,
        "updated_at": "2026-04-21T00:00:00.000Z"
    });
    let seller_doc = json!({
        "entity_scope": "seller",
        "id": ids.org_id,
        "name": format!("search-org-{suffix}"),
        "description": format!("search seller {suffix}"),
        "reputation_score": 0.0,
        "listing_product_count": 1,
        "updated_at": "2026-04-21T00:00:00.000Z"
    });

    let product_resp = client
        .put(format!(
            "{}/product_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.product_id
        ))
        .json(&product_doc)
        .send()
        .await
        .map_err(|err| format!("seed product opensearch doc failed: {err}"))?;
    if !product_resp.status().is_success() {
        return Err(format!(
            "seed product opensearch doc status={}",
            product_resp.status()
        ));
    }

    let seller_resp = client
        .put(format!(
            "{}/seller_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.org_id
        ))
        .json(&seller_doc)
        .send()
        .await
        .map_err(|err| format!("seed seller opensearch doc failed: {err}"))?;
    if !seller_resp.status().is_success() {
        return Err(format!(
            "seed seller opensearch doc status={}",
            seller_resp.status()
        ));
    }

    Ok(())
}

async fn cleanup_opensearch_documents(ids: &SeedIds) {
    let endpoint = opensearch_endpoint();
    let client = reqwest::Client::new();
    let _ = client
        .delete(format!(
            "{}/product_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.product_id
        ))
        .send()
        .await;
    let _ = client
        .delete(format!(
            "{}/seller_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.org_id
        ))
        .send()
        .await;
}

async fn ensure_index_exists(index_name: &str) -> Result<(), String> {
    let endpoint = opensearch_endpoint();
    let response = reqwest::Client::new()
        .put(format!("{}/{}", endpoint.trim_end_matches('/'), index_name))
        .json(&json!({
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 0
            }
        }))
        .send()
        .await
        .map_err(|err| format!("create opensearch index failed: {err}"))?;
    if response.status().is_success() {
        return Ok(());
    }
    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "unavailable".to_string());
    if body.contains("resource_already_exists_exception") {
        return Ok(());
    }
    Err(format!("create opensearch index failed: {body}"))
}

async fn delete_index(index_name: &str) {
    let endpoint = opensearch_endpoint();
    let _ = reqwest::Client::new()
        .delete(format!("{}/{}", endpoint.trim_end_matches('/'), index_name))
        .send()
        .await;
}

async fn alias_targets(alias: &str) -> Result<Vec<String>, String> {
    let endpoint = opensearch_endpoint();
    let response = reqwest::Client::new()
        .get(format!(
            "{}/_alias/{}",
            endpoint.trim_end_matches('/'),
            alias
        ))
        .send()
        .await
        .map_err(|err| format!("lookup alias targets failed: {err}"))?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(Vec::new());
    }
    if !response.status().is_success() {
        return Err(format!("lookup alias targets status={}", response.status()));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|err| format!("decode alias targets failed: {err}"))?;
    Ok(payload
        .as_object()
        .map(|items| items.keys().cloned().collect())
        .unwrap_or_default())
}

async fn restore_product_alias_binding(
    client: &Client,
    binding: &AliasBindingSeed,
    temp_index: &str,
) {
    let endpoint = opensearch_endpoint();
    let read_targets = alias_targets(&binding.read_alias).await.unwrap_or_default();
    let write_targets = alias_targets(&binding.write_alias)
        .await
        .unwrap_or_default();
    let mut actions = Vec::new();

    if read_targets.iter().any(|index| index == temp_index) {
        actions.push(json!({
            "remove": {
                "index": temp_index,
                "alias": binding.read_alias,
            }
        }));
    }
    if write_targets.iter().any(|index| index == temp_index) {
        actions.push(json!({
            "remove": {
                "index": temp_index,
                "alias": binding.write_alias,
            }
        }));
    }
    if !read_targets
        .iter()
        .any(|index| index == &binding.active_index_name)
    {
        actions.push(json!({
            "add": {
                "index": binding.active_index_name,
                "alias": binding.read_alias,
            }
        }));
    }
    if !write_targets
        .iter()
        .any(|index| index == &binding.active_index_name)
    {
        actions.push(json!({
            "add": {
                "index": binding.active_index_name,
                "alias": binding.write_alias,
            }
        }));
    }

    if !actions.is_empty() {
        let _ = reqwest::Client::new()
            .post(format!("{}/_aliases", endpoint.trim_end_matches('/')))
            .json(&json!({ "actions": actions }))
            .send()
            .await;
    }

    let _ = client
        .execute(
            "UPDATE search.index_alias_binding
             SET active_index_name = $2,
                 updated_at = now()
             WHERE alias_binding_id = $1::text::uuid",
            &[&binding.alias_binding_id, &binding.active_index_name],
        )
        .await;
}

fn opensearch_endpoint() -> String {
    std::env::var("OPENSEARCH_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9200".to_string())
}

fn redis_url() -> String {
    if let Ok(url) = std::env::var("REDIS_URL") {
        if !url.trim().is_empty() {
            return url;
        }
    }
    "redis://default:datab_redis_pass@127.0.0.1:6379/0".to_string()
}

fn cache_key_for_query(query: &SearchQuery, backend: &str) -> String {
    let canonical = serde_json::to_string(&json!({
        "backend": backend,
        "query": query,
    }))
    .expect("encode search query");
    let digest = format!("{:x}", Sha256::digest(canonical.as_bytes()));
    format!("datab:v1:search:catalog:product:{digest}")
}

async fn response_json(response: axum::response::Response) -> Result<Value, String> {
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .map_err(|err| format!("read response body failed: {err}"))?;
    serde_json::from_slice(&body).map_err(|_err| {
        format!(
            "decode response body failed: status={status} body={}",
            String::from_utf8_lossy(&body)
        )
    })
}

async fn count_audit_events(
    client: &Client,
    request_id: &str,
    action_name: &str,
) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = $2",
            &[&request_id, &action_name],
        )
        .await
        .map(|row| row.get(0))
}

async fn count_access_audit(
    client: &Client,
    request_id: &str,
    target_type: &str,
) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = $1
               AND target_type = $2",
            &[&request_id, &target_type],
        )
        .await
        .map(|row| row.get(0))
}

async fn latest_access_step_up(client: &Client, request_id: &str) -> Result<Option<String>, Error> {
    client
        .query_opt(
            "SELECT step_up_challenge_id::text
             FROM audit.access_audit
             WHERE request_id = $1
             ORDER BY created_at DESC
             LIMIT 1",
            &[&request_id],
        )
        .await
        .map(|row| row.and_then(|row| row.get::<usize, Option<String>>(0)))
}

async fn count_system_logs(client: &Client, request_id: &str, message: &str) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = $2",
            &[&request_id, &message],
        )
        .await
        .map(|row| row.get(0))
}

#[tokio::test(flavor = "current_thread")]
async fn search_api_and_ops_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let _env_lock = SEARCH_ENV_TEST_LOCK.lock().expect("lock search env");
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let ids = seed_minimum_graph(&client, &suffix)
        .await
        .expect("seed minimum graph");
    let ranking_seed = lookup_product_ranking_profile(&client)
        .await
        .expect("load ranking profile");
    let alias_binding = lookup_product_alias_binding(&client)
        .await
        .expect("load product alias binding");
    let temp_index = format!("product_search_v1_aud022_{suffix}");
    let _app_mode = ScopedEnvVar::set("APP_MODE", "staging");
    let app = crate::with_live_test_state(router()).await;
    let admin_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["platform_admin"]);
    let buyer_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["buyer_operator"]);

    let req_search_1 = format!("req-search-catalog-1-{suffix}");
    let req_search_2 = format!("req-search-catalog-2-{suffix}");
    let req_cache = format!("req-search-cache-{suffix}");
    let req_reindex_missing_idem = format!("req-search-reindex-missing-idem-{suffix}");
    let req_reindex = format!("req-search-reindex-{suffix}");
    let req_sync = format!("req-search-sync-{suffix}");
    let req_rank_list = format!("req-search-ranking-list-{suffix}");
    let req_rank_patch = format!("req-search-ranking-patch-{suffix}");
    let req_alias = format!("req-search-alias-{suffix}");

    let query = SearchQuery {
        q: Some(suffix.clone()),
        entity_scope: "product".to_string(),
        industry: None,
        tags: Vec::new(),
        delivery_mode: None,
        price_min: None,
        price_max: None,
        sort: "composite".to_string(),
        page: Some(1),
        page_size: Some(10),
    };
    let cache_key = cache_key_for_query(&query, "opensearch");
    let updated_weights = json!({
        "quality_score": 0.78,
        "hotness_score": 0.12,
        "freshness_score": 0.10
    });
    let updated_filter_policy = json!({
        "blocked_statuses": ["delisted"],
        "industry_whitelist": ["industrial_manufacturing"]
    });

    let mut step_up_ids = Vec::new();
    let outcome: Result<(), String> = async {
        seed_opensearch_documents(&ids, &suffix).await?;
        ensure_index_exists(&temp_index).await?;

        let missing_idempotency = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/ops/search/reindex")
                    .header("authorization", &admin_auth)
                    .header("content-type", "application/json")
                    .header("x-request-id", &req_reindex_missing_idem)
                    .body(Body::from(
                        json!({
                            "entity_scope": "product",
                            "entity_id": ids.product_id,
                            "mode": "single",
                            "force": true
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build missing idempotency request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call missing idempotency request failed: {err}"))?;
        assert_eq!(missing_idempotency.status(), StatusCode::BAD_REQUEST);
        let missing_json = response_json(missing_idempotency).await?;
        if missing_json["code"].as_str() != Some("SEARCH_QUERY_INVALID") {
            return Err(format!(
                "missing idempotency error mismatch: {}",
                missing_json
            ));
        }

        let search_resp_1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/catalog/search?q={}&entity_scope=product&page=1&page_size=10",
                        suffix
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &req_search_1)
                    .body(Body::empty())
                    .map_err(|err| format!("build first search request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call first search endpoint failed: {err}"))?;
        assert_eq!(search_resp_1.status(), StatusCode::OK);
        let search_json_1 = response_json(search_resp_1).await?;
        let search_items = search_json_1["data"]["items"]
            .as_array()
            .ok_or_else(|| "search items missing".to_string())?;
        if !search_items
            .iter()
            .any(|item| item["entity_id"].as_str() == Some(ids.product_id.as_str()))
        {
            return Err("search result missing seeded product".to_string());
        }
        if search_json_1["data"]["cache_hit"].as_bool() != Some(false) {
            return Err(format!("first search should be cache miss: {search_json_1}"));
        }
        if search_json_1["data"]["backend"].as_str() != Some("opensearch") {
            return Err(format!(
                "staging search should use opensearch backend: {search_json_1}"
            ));
        }

        let redis_client =
            redis::Client::open(redis_url()).map_err(|err| format!("init redis client failed: {err}"))?;
        let mut redis_conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("connect redis failed: {err}"))?;
        let cached_value: Option<String> = redis_conn
            .get(&cache_key)
            .await
            .map_err(|err| format!("load cached search key failed: {err}"))?;
        if cached_value.is_none() {
            return Err(format!("search cache key missing after first search: {cache_key}"));
        }

        let search_resp_2 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/catalog/search?q={}&entity_scope=product&page=1&page_size=10",
                        suffix
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &req_search_2)
                    .body(Body::empty())
                    .map_err(|err| format!("build second search request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call second search endpoint failed: {err}"))?;
        assert_eq!(search_resp_2.status(), StatusCode::OK);
        let search_json_2 = response_json(search_resp_2).await?;
        if search_json_2["data"]["cache_hit"].as_bool() != Some(true) {
            return Err(format!("second search should be cache hit: {search_json_2}"));
        }
        if search_json_2["data"]["backend"].as_str() != Some("opensearch") {
            return Err(format!(
                "staging search should keep opensearch backend: {search_json_2}"
            ));
        }

        let cache_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/ops/search/cache/invalidate")
                    .header("authorization", &admin_auth)
                    .header("content-type", "application/json")
                    .header("x-request-id", &req_cache)
                    .header("x-idempotency-key", format!("idem-search-cache-{suffix}"))
                    .body(Body::from(
                        json!({
                            "entity_scope": "product"
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build cache invalidate request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call cache invalidate endpoint failed: {err}"))?;
        assert_eq!(cache_resp.status(), StatusCode::OK);
        let cache_json = response_json(cache_resp).await?;
        if cache_json["data"]["deleted_keys"]
            .as_u64()
            .unwrap_or_default()
            < 1
        {
            return Err(format!("cache invalidate deleted_keys mismatch: {cache_json}"));
        }
        let deleted_cache: Option<String> = redis_conn
            .get(&cache_key)
            .await
            .map_err(|err| format!("load cache key after delete failed: {err}"))?;
        if deleted_cache.is_some() {
            return Err(format!("search cache key still exists after invalidation: {cache_key}"));
        }

        let reindex_step_up = seed_verified_step_up_challenge(
            &client,
            &ids.operator_user_id,
            "ops.search_reindex.execute",
            "product",
            Some(ids.product_id.as_str()),
            &format!("aud022-reindex-{suffix}"),
        )
        .await
        .map_err(|err| format!("seed reindex step-up failed: {err}"))?;
        step_up_ids.push(reindex_step_up.clone());
        let reindex_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/ops/search/reindex")
                    .header("authorization", &admin_auth)
                    .header("content-type", "application/json")
                    .header("x-request-id", &req_reindex)
                    .header("x-idempotency-key", format!("idem-search-reindex-{suffix}"))
                    .header("x-step-up-token", &reindex_step_up)
                    .body(Body::from(
                        json!({
                            "entity_scope": "product",
                            "entity_id": ids.product_id,
                            "mode": "single",
                            "force": true
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build reindex request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call reindex endpoint failed: {err}"))?;
        assert_eq!(reindex_resp.status(), StatusCode::OK);
        let reindex_json = response_json(reindex_resp).await?;
        if reindex_json["data"]["enqueued_count"].as_u64() != Some(1) {
            return Err(format!("reindex enqueued_count mismatch: {reindex_json}"));
        }

        let queued = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM search.index_sync_task
                 WHERE entity_scope = 'product'
                   AND entity_id = $1::text::uuid
                   AND sync_status = 'queued'",
                &[&ids.product_id],
            )
            .await
            .map_err(|err| format!("query queued reindex task failed: {err}"))?;
        let queued_count: i64 = queued.get(0);
        if queued_count < 1 {
            return Err("queued reindex task missing".to_string());
        }
        if count_audit_events(&client, &req_reindex, "search.reindex.queue")
            .await
            .map_err(|err| format!("count reindex audit events failed: {err}"))?
            != 1
        {
            return Err("reindex audit event missing".to_string());
        }
        if count_access_audit(&client, &req_reindex, "search_reindex")
            .await
            .map_err(|err| format!("count reindex access audit failed: {err}"))?
            != 1
        {
            return Err("reindex access audit missing".to_string());
        }
        if latest_access_step_up(&client, &req_reindex)
            .await
            .map_err(|err| format!("load reindex step-up binding failed: {err}"))?
            .as_deref()
            != Some(reindex_step_up.as_str())
        {
            return Err("reindex step-up binding mismatch".to_string());
        }
        if count_system_logs(
            &client,
            &req_reindex,
            "search ops action executed: POST /api/v1/ops/search/reindex",
        )
        .await
        .map_err(|err| format!("count reindex system log failed: {err}"))?
            != 1
        {
            return Err("reindex system log missing".to_string());
        }

        let sync_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/ops/search/sync?entity_scope=product&sync_status=queued&limit=20")
                    .header("authorization", &admin_auth)
                    .header("x-request-id", &req_sync)
                    .body(Body::empty())
                    .map_err(|err| format!("build sync request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call sync endpoint failed: {err}"))?;
        assert_eq!(sync_resp.status(), StatusCode::OK);
        let sync_json = response_json(sync_resp).await?;
        let sync_items = sync_json["data"]
            .as_array()
            .ok_or_else(|| "sync response data missing".to_string())?;
        if !sync_items
            .iter()
            .any(|item| item["entity_id"].as_str() == Some(ids.product_id.as_str()))
        {
            return Err(format!("sync task list missing queued product task: {sync_json}"));
        }
        if count_access_audit(&client, &req_sync, "search_sync_query")
            .await
            .map_err(|err| format!("count sync access audit failed: {err}"))?
            != 1
        {
            return Err("search sync access audit missing".to_string());
        }
        if count_system_logs(
            &client,
            &req_sync,
            "search ops lookup executed: GET /api/v1/ops/search/sync",
        )
        .await
        .map_err(|err| format!("count sync system log failed: {err}"))?
            != 1
        {
            return Err("search sync system log missing".to_string());
        }

        let ranking_list_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/ops/search/ranking-profiles")
                    .header("authorization", &admin_auth)
                    .header("x-request-id", &req_rank_list)
                    .body(Body::empty())
                    .map_err(|err| format!("build ranking list request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call ranking list endpoint failed: {err}"))?;
        assert_eq!(ranking_list_resp.status(), StatusCode::OK);
        let ranking_list_json = response_json(ranking_list_resp).await?;
        let ranking_items = ranking_list_json["data"]
            .as_array()
            .ok_or_else(|| "ranking profile list missing".to_string())?;
        if !ranking_items
            .iter()
            .any(|item| item["ranking_profile_id"].as_str() == Some(ranking_seed.ranking_profile_id.as_str()))
        {
            return Err(format!(
                "ranking profile list missing target profile: {ranking_list_json}"
            ));
        }
        if count_access_audit(&client, &req_rank_list, "search_ranking_profile")
            .await
            .map_err(|err| format!("count ranking list access audit failed: {err}"))?
            != 1
        {
            return Err("ranking list access audit missing".to_string());
        }
        if count_system_logs(
            &client,
            &req_rank_list,
            "search ops lookup executed: GET /api/v1/ops/search/ranking-profiles",
        )
        .await
        .map_err(|err| format!("count ranking list system log failed: {err}"))?
            != 1
        {
            return Err("ranking list system log missing".to_string());
        }

        let ranking_step_up = seed_verified_step_up_challenge(
            &client,
            &ids.operator_user_id,
            "ops.search_ranking.manage",
            "ranking_profile",
            Some(ranking_seed.ranking_profile_id.as_str()),
            &format!("aud022-ranking-{suffix}"),
        )
        .await
        .map_err(|err| format!("seed ranking step-up failed: {err}"))?;
        step_up_ids.push(ranking_step_up.clone());
        let ranking_patch_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!(
                        "/api/v1/ops/search/ranking-profiles/{}",
                        ranking_seed.ranking_profile_id
                    ))
                    .header("authorization", &admin_auth)
                    .header("content-type", "application/json")
                    .header("x-request-id", &req_rank_patch)
                    .header("x-idempotency-key", format!("idem-search-ranking-{suffix}"))
                    .header("x-step-up-token", &ranking_step_up)
                    .body(Body::from(
                        json!({
                            "weights_json": updated_weights,
                            "filter_policy_json": updated_filter_policy,
                            "status": "active"
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build ranking patch request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call ranking patch endpoint failed: {err}"))?;
        assert_eq!(ranking_patch_resp.status(), StatusCode::OK);
        let ranking_patch_json = response_json(ranking_patch_resp).await?;
        if ranking_patch_json["data"]["weights_json"] != updated_weights {
            return Err(format!(
                "ranking profile weights mismatch after patch: {ranking_patch_json}"
            ));
        }
        let ranking_row = client
            .query_one(
                "SELECT weights_json, filter_policy_json, status
                 FROM search.ranking_profile
                 WHERE ranking_profile_id = $1::text::uuid",
                &[&ranking_seed.ranking_profile_id],
            )
            .await
            .map_err(|err| format!("load ranking profile after patch failed: {err}"))?;
        let patched_weights: Value = ranking_row.get(0);
        let patched_filter_policy: Value = ranking_row.get(1);
        let patched_status: String = ranking_row.get(2);
        if patched_weights != updated_weights
            || patched_filter_policy != updated_filter_policy
            || patched_status != "active"
        {
            return Err("ranking profile database state mismatch after patch".to_string());
        }
        if count_audit_events(&client, &req_rank_patch, "search.ranking_profile.patch")
            .await
            .map_err(|err| format!("count ranking patch audit events failed: {err}"))?
            != 1
        {
            return Err("ranking patch audit event missing".to_string());
        }
        if count_access_audit(&client, &req_rank_patch, "search_ranking_profile")
            .await
            .map_err(|err| format!("count ranking patch access audit failed: {err}"))?
            != 1
        {
            return Err("ranking patch access audit missing".to_string());
        }
        if latest_access_step_up(&client, &req_rank_patch)
            .await
            .map_err(|err| format!("load ranking patch step-up binding failed: {err}"))?
            .as_deref()
            != Some(ranking_step_up.as_str())
        {
            return Err("ranking patch step-up binding mismatch".to_string());
        }
        if count_system_logs(
            &client,
            &req_rank_patch,
            "search ops action executed: PATCH /api/v1/ops/search/ranking-profiles/{id}",
        )
        .await
        .map_err(|err| format!("count ranking patch system log failed: {err}"))?
            != 1
        {
            return Err("ranking patch system log missing".to_string());
        }

        let alias_step_up = seed_verified_step_up_challenge(
            &client,
            &ids.operator_user_id,
            "ops.search_alias.manage",
            "search_scope",
            None,
            &format!("aud022-alias-{suffix}"),
        )
        .await
        .map_err(|err| format!("seed alias step-up failed: {err}"))?;
        step_up_ids.push(alias_step_up.clone());
        let alias_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/ops/search/aliases/switch")
                    .header("authorization", &admin_auth)
                    .header("content-type", "application/json")
                    .header("x-request-id", &req_alias)
                    .header("x-idempotency-key", format!("idem-search-alias-{suffix}"))
                    .header("x-step-up-token", &alias_step_up)
                    .body(Body::from(
                        json!({
                            "entity_scope": "product",
                            "next_index_name": temp_index
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build alias switch request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call alias switch endpoint failed: {err}"))?;
        assert_eq!(alias_resp.status(), StatusCode::OK);
        let alias_json = response_json(alias_resp).await?;
        if alias_json["data"]["active_index_name"].as_str() != Some(temp_index.as_str()) {
            return Err(format!("alias switch response mismatch: {alias_json}"));
        }
        let alias_binding_row = client
            .query_one(
                "SELECT active_index_name
                 FROM search.index_alias_binding
                 WHERE alias_binding_id = $1::text::uuid",
                &[&alias_binding.alias_binding_id],
            )
            .await
            .map_err(|err| format!("load alias binding after switch failed: {err}"))?;
        let active_index_name: String = alias_binding_row.get(0);
        if active_index_name != temp_index {
            return Err(format!(
                "alias binding active_index_name mismatch: expected {temp_index}, got {active_index_name}"
            ));
        }
        let read_targets = alias_targets(&alias_binding.read_alias).await?;
        let write_targets = alias_targets(&alias_binding.write_alias).await?;
        if !read_targets.iter().any(|index| index == &temp_index) {
            return Err(format!(
                "read alias target mismatch after switch: alias={} targets={read_targets:?}",
                alias_binding.read_alias
            ));
        }
        if !write_targets.iter().any(|index| index == &temp_index) {
            return Err(format!(
                "write alias target mismatch after switch: alias={} targets={write_targets:?}",
                alias_binding.write_alias
            ));
        }
        if count_audit_events(&client, &req_alias, "search.alias.switch")
            .await
            .map_err(|err| format!("count alias switch audit events failed: {err}"))?
            != 1
        {
            return Err("alias switch audit event missing".to_string());
        }
        if count_access_audit(&client, &req_alias, "search_alias_binding")
            .await
            .map_err(|err| format!("count alias switch access audit failed: {err}"))?
            != 1
        {
            return Err("alias switch access audit missing".to_string());
        }
        if latest_access_step_up(&client, &req_alias)
            .await
            .map_err(|err| format!("load alias switch step-up binding failed: {err}"))?
            .as_deref()
            != Some(alias_step_up.as_str())
        {
            return Err("alias switch step-up binding mismatch".to_string());
        }
        if count_system_logs(
            &client,
            &req_alias,
            "search ops action executed: POST /api/v1/ops/search/aliases/switch",
        )
        .await
        .map_err(|err| format!("count alias switch system log failed: {err}"))?
            != 1
        {
            return Err("alias switch system log missing".to_string());
        }

        if count_audit_events(&client, &req_cache, "search.cache.invalidate")
            .await
            .map_err(|err| format!("count cache invalidate audit events failed: {err}"))?
            != 1
        {
            return Err("cache invalidate audit event missing".to_string());
        }
        if count_access_audit(&client, &req_cache, "search_cache")
            .await
            .map_err(|err| format!("count cache invalidate access audit failed: {err}"))?
            != 1
        {
            return Err("cache invalidate access audit missing".to_string());
        }
        if latest_access_step_up(&client, &req_cache)
            .await
            .map_err(|err| format!("load cache invalidate step-up binding failed: {err}"))?
            .is_some()
        {
            return Err("cache invalidate should not carry step-up binding".to_string());
        }
        if count_system_logs(
            &client,
            &req_cache,
            "search ops action executed: POST /api/v1/ops/search/cache/invalidate",
        )
        .await
        .map_err(|err| format!("count cache invalidate system log failed: {err}"))?
            != 1
        {
            return Err("cache invalidate system log missing".to_string());
        }

        Ok(())
    }
    .await;

    restore_product_alias_binding(&client, &alias_binding, &temp_index).await;
    delete_index(&temp_index).await;
    cleanup_opensearch_documents(&ids).await;
    cleanup_seed_graph(&client, &ids, &step_up_ids, &ranking_seed).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn search_catalog_pg_fallback_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let _env_lock = SEARCH_ENV_TEST_LOCK.lock().expect("lock search env");
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let ids = seed_minimum_graph(&client, &suffix)
        .await
        .expect("seed minimum graph");
    let ranking_seed = lookup_product_ranking_profile(&client)
        .await
        .expect("load ranking profile");
    let _app_mode = ScopedEnvVar::set("APP_MODE", "local");
    let _opensearch_endpoint = ScopedEnvVar::set("OPENSEARCH_ENDPOINT", "http://127.0.0.1:1");
    let app = crate::with_live_test_state(router()).await;
    let buyer_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["buyer_operator"]);

    let req_search_1 = format!("req-search-pg-fallback-1-{suffix}");
    let req_search_2 = format!("req-search-pg-fallback-2-{suffix}");
    let query = SearchQuery {
        q: Some(suffix.clone()),
        entity_scope: "product".to_string(),
        industry: None,
        tags: Vec::new(),
        delivery_mode: None,
        price_min: None,
        price_max: None,
        sort: "composite".to_string(),
        page: Some(1),
        page_size: Some(10),
    };
    let cache_key = cache_key_for_query(&query, "postgresql");

    let outcome: Result<(), String> = async {
        let search_resp_1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/catalog/search?q={}&entity_scope=product&page=1&page_size=10",
                        suffix
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &req_search_1)
                    .body(Body::empty())
                    .map_err(|err| format!("build first pg fallback search request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call first pg fallback search endpoint failed: {err}"))?;
        assert_eq!(search_resp_1.status(), StatusCode::OK);
        let search_json_1 = response_json(search_resp_1).await?;
        let search_items = search_json_1["data"]["items"]
            .as_array()
            .ok_or_else(|| "pg fallback search items missing".to_string())?;
        if !search_items
            .iter()
            .any(|item| item["entity_id"].as_str() == Some(ids.product_id.as_str()))
        {
            return Err(format!(
                "pg fallback search result missing seeded product: {search_json_1}"
            ));
        }
        if search_json_1["data"]["backend"].as_str() != Some("postgresql") {
            return Err(format!(
                "local search should use postgresql fallback backend: {search_json_1}"
            ));
        }
        if search_json_1["data"]["cache_hit"].as_bool() != Some(false) {
            return Err(format!(
                "first pg fallback search should be cache miss: {search_json_1}"
            ));
        }

        let redis_client = redis::Client::open(redis_url())
            .map_err(|err| format!("init redis client failed: {err}"))?;
        let mut redis_conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| format!("connect redis failed: {err}"))?;
        let cached_value: Option<String> = redis_conn
            .get(&cache_key)
            .await
            .map_err(|err| format!("load pg fallback cached search key failed: {err}"))?;
        if cached_value.is_none() {
            return Err(format!(
                "pg fallback search cache key missing after first search: {cache_key}"
            ));
        }

        let search_resp_2 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/catalog/search?q={}&entity_scope=product&page=1&page_size=10",
                        suffix
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &req_search_2)
                    .body(Body::empty())
                    .map_err(|err| format!("build second pg fallback search request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call second pg fallback search endpoint failed: {err}"))?;
        assert_eq!(search_resp_2.status(), StatusCode::OK);
        let search_json_2 = response_json(search_resp_2).await?;
        if search_json_2["data"]["backend"].as_str() != Some("postgresql") {
            return Err(format!(
                "second local search should keep postgresql fallback backend: {search_json_2}"
            ));
        }
        if search_json_2["data"]["cache_hit"].as_bool() != Some(true) {
            return Err(format!(
                "second pg fallback search should be cache hit: {search_json_2}"
            ));
        }

        Ok(())
    }
    .await;

    let redis_client = redis::Client::open(redis_url()).expect("init redis client");
    if let Ok(mut redis_conn) = redis_client.get_multiplexed_async_connection().await {
        let _ = redis_conn.del::<_, usize>(&cache_key).await;
    }
    cleanup_seed_graph(&client, &ids, &[], &ranking_seed).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}
