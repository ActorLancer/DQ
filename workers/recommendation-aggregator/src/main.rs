use db::{AppDb, DbPoolConfig, GenericClient};
use rdkafka::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use redis::AsyncCommands;
use serde_json::Value;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
struct WorkerConfig {
    database_url: String,
    kafka_brokers: String,
    topic_recommend_behavior: String,
    consumer_group: String,
    redis_url: String,
    redis_namespace: String,
    product_write_alias: String,
    seller_write_alias: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .try_init()
        .ok();

    let cfg = WorkerConfig::from_env();
    let db = AppDb::connect(
        DbPoolConfig {
            dsn: cfg.database_url.clone(),
            max_connections: 16,
        }
        .into(),
    )
    .await?;

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &cfg.kafka_brokers)
        .set("group.id", &cfg.consumer_group)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()?;
    consumer.subscribe(&[&cfg.topic_recommend_behavior])?;

    info!(
        topic = %cfg.topic_recommend_behavior,
        group = %cfg.consumer_group,
        "recommendation-aggregator started"
    );

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Err(err) = handle_kafka_message(&db, &cfg, &message).await {
                    error!(error = %err, "recommendation-aggregator event handling failed");
                }
                if let Err(err) = consumer.commit_message(&message, CommitMode::Async) {
                    warn!(error = %err, "recommendation-aggregator commit offset failed");
                }
            }
            Err(err) => warn!(error = %err, "recommendation-aggregator kafka receive failed"),
        }
    }
}

impl WorkerConfig {
    fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string()
            }),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .or_else(|_| std::env::var("KAFKA_BOOTSTRAP_SERVERS"))
                .unwrap_or_else(|_| "127.0.0.1:9092".to_string()),
            topic_recommend_behavior: std::env::var("TOPIC_RECOMMENDATION_BEHAVIOR")
                .unwrap_or_else(|_| "dtp.recommend.behavior".to_string()),
            consumer_group: std::env::var("RECOMMENDATION_AGGREGATOR_CONSUMER_GROUP")
                .unwrap_or_else(|_| "cg-recommendation-aggregator".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://:datab_redis_pass@127.0.0.1:6379/1".to_string()),
            redis_namespace: std::env::var("REDIS_NAMESPACE")
                .unwrap_or_else(|_| "datab:v1".to_string()),
            product_write_alias: std::env::var("INDEX_ALIAS_PRODUCT_SEARCH_WRITE")
                .unwrap_or_else(|_| "product_search_write".to_string()),
            seller_write_alias: std::env::var("INDEX_ALIAS_SELLER_SEARCH_WRITE")
                .unwrap_or_else(|_| "seller_search_write".to_string()),
        }
    }
}

async fn handle_kafka_message(
    db: &AppDb,
    cfg: &WorkerConfig,
    message: &rdkafka::message::BorrowedMessage<'_>,
) -> Result<(), String> {
    let Some(payload) = message.payload() else {
        return Ok(());
    };
    handle_kafka_payload(db, cfg, payload).await
}

async fn handle_kafka_payload(
    db: &AppDb,
    cfg: &WorkerConfig,
    payload: &[u8],
) -> Result<(), String> {
    let envelope: Value = serde_json::from_slice(payload)
        .map_err(|err| format!("decode recommendation kafka payload failed: {err}"))?;
    handle_behavior_envelope(db, cfg, &envelope).await
}

async fn handle_behavior_envelope(
    db: &AppDb,
    cfg: &WorkerConfig,
    envelope: &Value,
) -> Result<(), String> {
    if envelope["event_type"].as_str() != Some("recommend.behavior_recorded") {
        return Ok(());
    }

    let Some(event_id) = envelope["event_id"].as_str() else {
        return Ok(());
    };
    let aggregate_type = envelope["aggregate_type"].as_str();
    let aggregate_id = envelope["aggregate_id"].as_str();
    let trace_id = envelope["trace_id"].as_str();
    let business = &envelope["payload"];
    let behavior_type = business["event_type"].as_str().unwrap_or_default();
    let entity_scope = business["entity_scope"].as_str();
    let entity_id = business["entity_id"].as_str();

    let client = db
        .client()
        .map_err(|err| format!("acquire recommendation aggregator db client failed: {err}"))?;
    let tx = client
        .transaction()
        .await
        .map_err(|err| format!("open recommendation aggregator transaction failed: {err}"))?;
    let processed = register_consumer_idempotency(
        &tx,
        "recommendation-aggregator",
        event_id,
        aggregate_type,
        aggregate_id,
        trace_id,
        behavior_type,
    )
    .await?;
    if !processed {
        tx.rollback()
            .await
            .map_err(|err| format!("rollback duplicated recommendation event failed: {err}"))?;
        return Ok(());
    }

    if let (Some(entity_scope), Some(entity_id)) = (entity_scope, entity_id) {
        refresh_search_signal_aggregate(&tx, entity_scope, entity_id).await?;
        refresh_search_projection_and_queue(&tx, cfg, entity_scope, entity_id).await?;
    }
    if let Some(recommendation_result_id) = business["recommendation_result_id"].as_str() {
        let recommendation_result_item_id =
            business["attrs"]["recommendation_result_item_id"].as_str();
        apply_result_relations(
            &tx,
            recommendation_result_id,
            recommendation_result_item_id,
            entity_scope,
            entity_id,
            behavior_type,
            event_id,
        )
        .await?;
    }

    tx.commit()
        .await
        .map_err(|err| format!("commit recommendation aggregator transaction failed: {err}"))?;
    invalidate_recommendation_cache(cfg, business).await?;
    Ok(())
}

async fn register_consumer_idempotency(
    client: &(impl GenericClient + Sync),
    consumer_name: &str,
    event_id: &str,
    aggregate_type: Option<&str>,
    aggregate_id: Option<&str>,
    trace_id: Option<&str>,
    behavior_type: &str,
) -> Result<bool, String> {
    let row = client
        .query_opt(
            "INSERT INTO ops.consumer_idempotency_record (
               consumer_name,
               event_id,
               aggregate_type,
               aggregate_id,
               trace_id,
               result_code,
               metadata
             ) VALUES (
               $1,
               $2::text::uuid,
               $3,
               $4::text::uuid,
               $5,
               'processed',
               jsonb_build_object('behavior_type', $6)
             )
             ON CONFLICT (consumer_name, event_id) DO NOTHING
             RETURNING consumer_idempotency_record_id::text",
            &[
                &consumer_name,
                &event_id,
                &aggregate_type,
                &aggregate_id,
                &trace_id,
                &behavior_type,
            ],
        )
        .await
        .map_err(|err| format!("register recommendation consumer idempotency failed: {err}"))?;
    Ok(row.is_some())
}

async fn refresh_search_signal_aggregate(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
    entity_id: &str,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO search.search_signal_aggregate (
               entity_scope,
               entity_id,
               exposure_count,
               click_count,
               order_count,
               hotness_score,
               updated_at
             )
             SELECT
               $1,
               $2::text::uuid,
               count(*) FILTER (WHERE event_type IN ('recommendation_panel_viewed', 'recommendation_item_exposed'))::bigint,
               count(*) FILTER (WHERE event_type IN ('recommendation_item_clicked', 'seller_recommendation_clicked'))::bigint,
               count(*) FILTER (WHERE event_type = 'order_submitted')::bigint,
               (
                 count(*) FILTER (WHERE event_type IN ('recommendation_panel_viewed', 'recommendation_item_exposed'))::numeric * 0.20
                 + count(*) FILTER (WHERE event_type IN ('recommendation_item_clicked', 'seller_recommendation_clicked'))::numeric * 1.00
                 + count(*) FILTER (WHERE event_type = 'order_submitted')::numeric * 2.00
               )::numeric(10, 4),
               now()
             FROM recommend.behavior_event
             WHERE entity_scope = $1
               AND entity_id = $2::text::uuid
             ON CONFLICT (entity_scope, entity_id) DO UPDATE
             SET exposure_count = EXCLUDED.exposure_count,
                 click_count = EXCLUDED.click_count,
                 order_count = EXCLUDED.order_count,
                 hotness_score = EXCLUDED.hotness_score,
                 updated_at = now()",
            &[&entity_scope, &entity_id],
        )
        .await
        .map_err(|err| format!("refresh search signal aggregate failed: {err}"))?;
    Ok(())
}

async fn refresh_search_projection_and_queue(
    client: &(impl GenericClient + Sync),
    cfg: &WorkerConfig,
    entity_scope: &str,
    entity_id: &str,
) -> Result<(), String> {
    let target_index = if entity_scope == "seller" {
        client
            .execute(
                "SELECT search.refresh_seller_search_document_by_id($1::text::uuid)",
                &[&entity_id],
            )
            .await
            .map_err(|err| format!("refresh seller search projection failed: {err}"))?;
        cfg.seller_write_alias.as_str()
    } else {
        client
            .execute(
                "SELECT search.refresh_product_search_document_by_id($1::text::uuid)",
                &[&entity_id],
            )
            .await
            .map_err(|err| format!("refresh product search projection failed: {err}"))?;
        cfg.product_write_alias.as_str()
    };
    let version_row = if entity_scope == "seller" {
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
    .map_err(|err| format!("load refreshed search document version failed: {err}"))?;
    let document_version: i64 = version_row.get(0);

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
               $1,
               $2::text::uuid,
               $3,
               'opensearch',
               $4,
               'queued'
             WHERE NOT EXISTS (
               SELECT 1
               FROM search.index_sync_task
               WHERE entity_scope = $1
                 AND entity_id = $2::text::uuid
                 AND sync_status IN ('queued', 'processing')
             )",
            &[&entity_scope, &entity_id, &document_version, &target_index],
        )
        .await
        .map_err(|err| {
            format!("queue search sync task from recommendation behavior failed: {err}")
        })?;
    Ok(())
}

async fn apply_result_relations(
    client: &(impl GenericClient + Sync),
    recommendation_result_id: &str,
    recommendation_result_item_id: Option<&str>,
    entity_scope: Option<&str>,
    entity_id: Option<&str>,
    behavior_type: &str,
    event_id: &str,
) -> Result<(), String> {
    let (Some(entity_scope), Some(entity_id)) = (entity_scope, entity_id) else {
        return Ok(());
    };
    if matches!(
        behavior_type,
        "recommendation_item_clicked" | "seller_recommendation_clicked"
    ) {
        client
            .execute(
                "UPDATE recommend.recommendation_result_item
                 SET click_status = 'clicked'
                 WHERE recommendation_result_id = $1::text::uuid
                   AND ($2::text IS NULL OR recommendation_result_item_id = $2::text::uuid)
                   AND entity_scope = $3
                   AND entity_id = $4::text::uuid",
                &[
                    &recommendation_result_id,
                    &recommendation_result_item_id,
                    &entity_scope,
                    &entity_id,
                ],
            )
            .await
            .map_err(|err| format!("mark recommendation result item clicked failed: {err}"))?;
    }

    let increment = match behavior_type {
        "recommendation_item_clicked" | "seller_recommendation_clicked" => 1.0,
        "recommendation_item_exposed" => 0.2,
        _ => 0.05,
    };
    let rows = client
        .query(
            "SELECT entity_scope, entity_id::text
             FROM recommend.recommendation_result_item
             WHERE recommendation_result_id = $1::text::uuid
               AND (entity_scope, entity_id) <> ($2, $3::text::uuid)",
            &[&recommendation_result_id, &entity_scope, &entity_id],
        )
        .await
        .map_err(|err| format!("load related recommendation result items failed: {err}"))?;

    for row in rows {
        let other_scope: String = row.get(0);
        let other_id: String = row.get(1);
        upsert_similarity_edge(
            client,
            entity_scope,
            entity_id,
            &other_scope,
            &other_id,
            increment,
            event_id,
            recommendation_result_id,
        )
        .await?;
        upsert_similarity_edge(
            client,
            &other_scope,
            &other_id,
            entity_scope,
            entity_id,
            increment,
            event_id,
            recommendation_result_id,
        )
        .await?;
        upsert_bundle_relation(
            client,
            entity_scope,
            entity_id,
            &other_scope,
            &other_id,
            increment,
            event_id,
            recommendation_result_id,
        )
        .await?;
        upsert_bundle_relation(
            client,
            &other_scope,
            &other_id,
            entity_scope,
            entity_id,
            increment,
            event_id,
            recommendation_result_id,
        )
        .await?;
    }
    Ok(())
}

async fn upsert_similarity_edge(
    client: &(impl GenericClient + Sync),
    source_scope: &str,
    source_id: &str,
    target_scope: &str,
    target_id: &str,
    increment: f64,
    event_id: &str,
    recommendation_result_id: &str,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO recommend.entity_similarity (
               source_entity_scope,
               source_entity_id,
               target_entity_scope,
               target_entity_id,
               similarity_type,
               score,
               evidence_json,
               version_no,
               updated_at
             ) VALUES (
               $1,
               $2::text::uuid,
               $3,
               $4::text::uuid,
               'co_recommended',
               $5,
               jsonb_build_object(
                 'last_behavior_event_id', $6,
                 'last_recommendation_result_id', $7
               ),
               1,
               now()
             )
             ON CONFLICT (
               source_entity_scope,
               source_entity_id,
               target_entity_scope,
               target_entity_id,
               similarity_type
             ) DO UPDATE
             SET score = recommend.entity_similarity.score + EXCLUDED.score,
                 evidence_json = recommend.entity_similarity.evidence_json
                   || jsonb_build_object(
                     'last_behavior_event_id', $6,
                     'last_recommendation_result_id', $7
                   ),
                 version_no = recommend.entity_similarity.version_no + 1,
                 updated_at = now()",
            &[
                &source_scope,
                &source_id,
                &target_scope,
                &target_id,
                &increment,
                &event_id,
                &recommendation_result_id,
            ],
        )
        .await
        .map_err(|err| format!("upsert recommendation similarity edge failed: {err}"))?;
    Ok(())
}

async fn upsert_bundle_relation(
    client: &(impl GenericClient + Sync),
    source_scope: &str,
    source_id: &str,
    target_scope: &str,
    target_id: &str,
    increment: f64,
    event_id: &str,
    recommendation_result_id: &str,
) -> Result<(), String> {
    client
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
             ) VALUES (
               $1,
               $2::text::uuid,
               $3,
               $4::text::uuid,
               'co_recommended',
               $5,
               'active',
               jsonb_build_object(
                 'last_behavior_event_id', $6,
                 'last_recommendation_result_id', $7
               )
             )
             ON CONFLICT (
               source_entity_scope,
               source_entity_id,
               target_entity_scope,
               target_entity_id,
               relation_type
             ) DO UPDATE
             SET relation_score = recommend.bundle_relation.relation_score + EXCLUDED.relation_score,
                 status = 'active',
                 metadata = recommend.bundle_relation.metadata
                   || jsonb_build_object(
                     'last_behavior_event_id', $6,
                     'last_recommendation_result_id', $7
                   ),
                 updated_at = now()",
            &[
                &source_scope,
                &source_id,
                &target_scope,
                &target_id,
                &increment,
                &event_id,
                &recommendation_result_id,
            ],
        )
        .await
        .map_err(|err| format!("upsert recommendation bundle relation failed: {err}"))?;
    Ok(())
}

async fn invalidate_recommendation_cache(
    cfg: &WorkerConfig,
    payload: &Value,
) -> Result<(), String> {
    let tenant = payload["subject_org_id"].as_str().unwrap_or("public");
    let actor = payload["subject_user_id"]
        .as_str()
        .or_else(|| payload["anonymous_session_key"].as_str())
        .unwrap_or("anonymous");
    let pattern = format!("{}:recommend:{tenant}:{actor}:*", cfg.redis_namespace);
    let client = redis::Client::open(cfg.redis_url.as_str())
        .map_err(|err| format!("redis recommendation cache client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis recommendation cache connect failed: {err}"))?;
    let keys: Vec<String> = connection
        .keys(pattern)
        .await
        .map_err(|err| format!("redis recommendation cache lookup failed: {err}"))?;
    if !keys.is_empty() {
        connection
            .del::<_, usize>(keys)
            .await
            .map_err(|err| format!("redis recommendation cache delete failed: {err}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::{Client, Error, NoTls, connect};
    use serde_json::json;

    #[derive(Debug)]
    struct SeedGraph {
        org_id: String,
        asset_ids: Vec<String>,
        asset_version_ids: Vec<String>,
        product_ids: Vec<String>,
        recommendation_request_id: String,
        recommendation_result_id: String,
        recommendation_result_item_ids: Vec<String>,
        behavior_event_id: String,
        outbox_event_id: String,
        cache_key: String,
    }

    fn live_db_enabled() -> bool {
        std::env::var("RECOMMEND_DB_SMOKE").ok().as_deref() == Some("1")
    }

    fn database_url() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string()
        })
    }

    fn redis_url() -> String {
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://:datab_redis_pass@127.0.0.1:6379/1".to_string())
    }

    fn worker_config() -> WorkerConfig {
        WorkerConfig {
            database_url: database_url(),
            kafka_brokers: "127.0.0.1:9094".to_string(),
            topic_recommend_behavior: "dtp.recommend.behavior".to_string(),
            consumer_group: "cg-recommendation-aggregator-test".to_string(),
            redis_url: redis_url(),
            redis_namespace: "datab:v1".to_string(),
            product_write_alias: "product_search_write".to_string(),
            seller_write_alias: "seller_search_write".to_string(),
        }
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, Error> {
        let ids_row = client
            .query_one(
                "SELECT gen_random_uuid()::text, gen_random_uuid()::text",
                &[],
            )
            .await?;
        let behavior_event_id: String = ids_row.get(0);
        let outbox_event_id: String = ids_row.get(1);

        let org = client
            .query_one(
                "INSERT INTO core.organization (
                   org_name, org_type, status, country_code, metadata
                 ) VALUES (
                   $1::text, 'enterprise', 'active', 'CN', jsonb_build_object('description', $2::text)
                 )
                 RETURNING org_id::text",
                &[
                    &format!("recommend-worker-org-{suffix}"),
                    &format!("recommend worker seller {suffix}"),
                ],
            )
            .await?;
        let org_id: String = org.get(0);

        let mut asset_ids = Vec::new();
        let mut asset_version_ids = Vec::new();
        let mut product_ids = Vec::new();
        for index in 0..2 {
            let asset = client
                .query_one(
                    "INSERT INTO catalog.data_asset (
                       owner_org_id, title, category, sensitivity_level, status
                     ) VALUES (
                       $1::text::uuid, $2, 'manufacturing', 'internal', 'draft'
                     )
                     RETURNING asset_id::text",
                    &[&org_id, &format!("recommend-worker-asset-{suffix}-{index}")],
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
                       $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing',
                       CASE WHEN $5::int = 0 THEN 'data_product' ELSE 'service' END,
                       $6, 'listed', 'one_time', $7, 'CNY',
                       CASE WHEN $5::int = 0 THEN 'file_download' ELSE 'api' END,
                       ARRAY['internal_use']::text[], $8,
                       jsonb_build_object(
                         'subtitle', $9,
                         'industry', 'industrial_manufacturing',
                         'quality_score', CASE WHEN $5::int = 0 THEN '0.92' ELSE '0.88' END
                       )
                     )
                     RETURNING product_id::text",
                    &[
                        &asset_id,
                        &asset_version_id,
                        &org_id,
                        &format!("recommend-worker-product-{suffix}-{index}"),
                        &(index as i32),
                        &format!("recommend worker product {suffix} {index}"),
                        &(98.0f64 + index as f64),
                        &format!("recommend worker keyword {suffix} {index}"),
                        &format!("recommend worker subtitle {suffix} {index}"),
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
            asset_ids.push(asset_id);
            asset_version_ids.push(asset_version_id);
            product_ids.push(product_id);
        }
        client
            .execute(
                "SELECT search.refresh_seller_search_document_by_id($1::text::uuid)",
                &[&org_id],
            )
            .await?;

        let recommendation_request = client
            .query_one(
                "INSERT INTO recommend.recommendation_request (
                   placement_code,
                   subject_scope,
                   subject_org_id,
                   page_context,
                   filter_json,
                   request_attrs,
                   candidate_source_summary,
                   status,
                   requested_count,
                   served_at
                 ) VALUES (
                   'home_featured',
                   'organization',
                   $1::text::uuid,
                   'home',
                   '{}'::jsonb,
                   jsonb_build_object('source', 'worker-smoke'),
                   jsonb_build_object('popular', 2),
                   'served',
                   2,
                   now()
                 )
                 RETURNING recommendation_request_id::text",
                &[&org_id],
            )
            .await?;
        let recommendation_request_id: String = recommendation_request.get(0);

        let recommendation_result = client
            .query_one(
                "INSERT INTO recommend.recommendation_result (
                   recommendation_request_id,
                   placement_code,
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
                   'home_featured',
                   'organization',
                   $2,
                   'mixed',
                   'served',
                   2,
                   2,
                   'basic',
                   jsonb_build_object('source', 'worker-smoke')
                 )
                 RETURNING recommendation_result_id::text",
                &[&recommendation_request_id, &org_id],
            )
            .await?;
        let recommendation_result_id: String = recommendation_result.get(0);

        let mut recommendation_result_item_ids = Vec::new();
        for (index, product_id) in product_ids.iter().enumerate() {
            let result_item = client
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
                       feature_snapshot
                     ) VALUES (
                       $1::text::uuid,
                       $2,
                       'product',
                       $3::text::uuid,
                       ARRAY['popular']::text[],
                       $4,
                       $5,
                       ARRAY['rank:quality']::text[],
                       jsonb_build_object('source', 'worker-smoke')
                     )
                     RETURNING recommendation_result_item_id::text",
                    &[
                        &recommendation_result_id,
                        &((index + 1) as i32),
                        product_id,
                        &(0.9f64 - (index as f64 * 0.1)),
                        &(1.1f64 - (index as f64 * 0.1)),
                    ],
                )
                .await?;
            recommendation_result_item_ids.push(result_item.get(0));
        }

        client
            .execute(
                "INSERT INTO recommend.behavior_event (
                   behavior_event_id,
                   subject_scope,
                   subject_org_id,
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
                   $1::text::uuid,
                   'organization',
                   $2::text::uuid,
                   'recommendation_item_clicked',
                   'home_featured',
                   'product',
                   $3::text::uuid,
                   'home',
                   $4::text::uuid,
                   $5::text::uuid,
                   $6,
                   $7,
                   jsonb_build_object(
                     'recommendation_result_item_id', $8::text::uuid,
                     'idempotency_key', $9
                   )
                 )",
                &[
                    &behavior_event_id,
                    &org_id,
                    &product_ids[0],
                    &recommendation_request_id,
                    &recommendation_result_id,
                    &format!("worker-req-{suffix}"),
                    &format!("worker-trace-{suffix}"),
                    &recommendation_result_item_ids[0],
                    &format!("worker-idempotency-{suffix}"),
                ],
            )
            .await?;

        let cache_key = format!("datab:v1:recommend:{org_id}:anonymous:smoke");
        Ok(SeedGraph {
            org_id,
            asset_ids,
            asset_version_ids,
            product_ids,
            recommendation_request_id,
            recommendation_result_id,
            recommendation_result_item_ids,
            behavior_event_id,
            outbox_event_id,
            cache_key,
        })
    }

    async fn cleanup_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM ops.consumer_idempotency_record
                 WHERE consumer_name = 'recommendation-aggregator'
                   AND event_id = $1::text::uuid",
                &[&seed.outbox_event_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM search.index_sync_task
                 WHERE entity_id = ANY($1::text[]::uuid[])",
                &[&seed.product_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM search.search_signal_aggregate
                 WHERE entity_scope = 'product'
                   AND entity_id = ANY($1::text[]::uuid[])",
                &[&seed.product_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM recommend.entity_similarity
                 WHERE source_entity_id = ANY($1::text[]::uuid[])
                    OR target_entity_id = ANY($1::text[]::uuid[])",
                &[&seed.product_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM recommend.bundle_relation
                 WHERE source_entity_id = ANY($1::text[]::uuid[])
                    OR target_entity_id = ANY($1::text[]::uuid[])",
                &[&seed.product_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM recommend.behavior_event
                 WHERE behavior_event_id = $1::text::uuid",
                &[&seed.behavior_event_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM recommend.recommendation_request
                 WHERE recommendation_request_id = $1::text::uuid",
                &[&seed.recommendation_request_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM recommend.subject_profile_snapshot
                 WHERE org_id = $1::text::uuid",
                &[&seed.org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM recommend.cohort_popularity
                 WHERE cohort_key = $1",
                &[&format!("org:{}", seed.org_id)],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM search.product_search_document
                 WHERE product_id = ANY($1::text[]::uuid[])",
                &[&seed.product_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM search.seller_search_document
                 WHERE org_id = $1::text::uuid",
                &[&seed.org_id],
            )
            .await;
        for product_id in &seed.product_ids {
            let _ = client
                .execute(
                    "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                    &[product_id],
                )
                .await;
        }
        for asset_version_id in &seed.asset_version_ids {
            let _ = client
                .execute(
                    "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                    &[asset_version_id],
                )
                .await;
        }
        for asset_id in &seed.asset_ids {
            let _ = client
                .execute(
                    "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                    &[asset_id],
                )
                .await;
        }
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seed.org_id],
            )
            .await;
    }

    async fn seed_cache(cfg: &WorkerConfig, key: &str) {
        let client = redis::Client::open(cfg.redis_url.as_str()).expect("redis client");
        let mut connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("redis connect");
        connection
            .set_ex::<_, _, ()>(key, "{\"seed\":true}", 600)
            .await
            .expect("seed recommendation cache");
    }

    async fn cache_exists(cfg: &WorkerConfig, key: &str) -> bool {
        let client = redis::Client::open(cfg.redis_url.as_str()).expect("redis client");
        let mut connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("redis connect");
        let value: Option<String> = connection.get(key).await.expect("get cache");
        value.is_some()
    }

    #[tokio::test]
    async fn recommendation_aggregator_db_smoke() {
        if !live_db_enabled() {
            return;
        }
        let cfg = worker_config();
        let db = AppDb::connect(
            DbPoolConfig {
                dsn: cfg.database_url.clone(),
                max_connections: 4,
            }
            .into(),
        )
        .await
        .expect("connect app db");
        let (client, connection) = connect(&cfg.database_url, NoTls)
            .await
            .expect("connect database");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_millis()
        );
        let seed = seed_graph(&client, &suffix).await.expect("seed graph");
        seed_cache(&cfg, &seed.cache_key).await;
        assert!(cache_exists(&cfg, &seed.cache_key).await);

        let envelope = json!({
            "event_id": seed.outbox_event_id,
            "event_type": "recommend.behavior_recorded",
            "aggregate_type": "recommend.behavior_event",
            "aggregate_id": seed.behavior_event_id,
            "trace_id": format!("worker-trace-{suffix}"),
            "payload": {
                "event_type": "recommendation_item_clicked",
                "placement_code": "home_featured",
                "subject_scope": "organization",
                "subject_org_id": seed.org_id,
                "recommendation_request_id": seed.recommendation_request_id,
                "recommendation_result_id": seed.recommendation_result_id,
                "entity_scope": "product",
                "entity_id": seed.product_ids[0],
                "attrs": {
                    "recommendation_result_item_id": seed.recommendation_result_item_ids[0]
                }
            }
        });

        handle_behavior_envelope(&db, &cfg, &envelope)
            .await
            .expect("handle recommendation envelope");

        let click_status: String = client
            .query_one(
                "SELECT click_status
                 FROM recommend.recommendation_result_item
                 WHERE recommendation_result_item_id = $1::text::uuid",
                &[&seed.recommendation_result_item_ids[0]],
            )
            .await
            .expect("load click status")
            .get(0);
        assert_eq!(click_status, "clicked");

        let aggregate_row = client
            .query_one(
                "SELECT exposure_count, click_count, hotness_score::text
                 FROM search.search_signal_aggregate
                 WHERE entity_scope = 'product'
                   AND entity_id = $1::text::uuid",
                &[&seed.product_ids[0]],
            )
            .await
            .expect("load search signal aggregate");
        let exposure_count: i64 = aggregate_row.get(0);
        let click_count: i64 = aggregate_row.get(1);
        let hotness_score: String = aggregate_row.get(2);
        assert_eq!(exposure_count, 0);
        assert_eq!(click_count, 1);
        assert_eq!(hotness_score, "1.0000");

        let sync_task_count: i64 = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM search.index_sync_task
                 WHERE entity_scope = 'product'
                   AND entity_id = $1::text::uuid
                   AND sync_status = 'queued'",
                &[&seed.product_ids[0]],
            )
            .await
            .expect("load sync task count")
            .get(0);
        assert!(sync_task_count >= 1);

        let similarity_score: String = client
            .query_one(
                "SELECT score::text
                 FROM recommend.entity_similarity
                 WHERE source_entity_scope = 'product'
                   AND source_entity_id = $1::text::uuid
                   AND target_entity_scope = 'product'
                   AND target_entity_id = $2::text::uuid
                   AND similarity_type = 'co_recommended'",
                &[&seed.product_ids[0], &seed.product_ids[1]],
            )
            .await
            .expect("load similarity score")
            .get(0);
        assert_eq!(similarity_score, "1.000000");

        let bundle_score: String = client
            .query_one(
                "SELECT relation_score::text
                 FROM recommend.bundle_relation
                 WHERE source_entity_scope = 'product'
                   AND source_entity_id = $1::text::uuid
                   AND target_entity_scope = 'product'
                   AND target_entity_id = $2::text::uuid
                   AND relation_type = 'co_recommended'",
                &[&seed.product_ids[0], &seed.product_ids[1]],
            )
            .await
            .expect("load bundle score")
            .get(0);
        assert_eq!(bundle_score, "1.000000");

        let idempotency_count: i64 = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM ops.consumer_idempotency_record
                 WHERE consumer_name = 'recommendation-aggregator'
                   AND event_id = $1::text::uuid",
                &[&seed.outbox_event_id],
            )
            .await
            .expect("load consumer idempotency count")
            .get(0);
        assert_eq!(idempotency_count, 1);
        assert!(!cache_exists(&cfg, &seed.cache_key).await);

        handle_behavior_envelope(&db, &cfg, &envelope)
            .await
            .expect("re-handle duplicated recommendation envelope");

        let duplicated_similarity_score: String = client
            .query_one(
                "SELECT score::text
                 FROM recommend.entity_similarity
                 WHERE source_entity_scope = 'product'
                   AND source_entity_id = $1::text::uuid
                   AND target_entity_scope = 'product'
                   AND target_entity_id = $2::text::uuid
                   AND similarity_type = 'co_recommended'",
                &[&seed.product_ids[0], &seed.product_ids[1]],
            )
            .await
            .expect("load duplicated similarity score")
            .get(0);
        assert_eq!(duplicated_similarity_score, "1.000000");

        cleanup_graph(&client, &seed).await;
    }
}
