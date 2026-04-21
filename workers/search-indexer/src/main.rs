use db::{AppDb, DbPoolConfig, GenericClient};
use rdkafka::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use redis::AsyncCommands;
use serde_json::Value;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
struct WorkerConfig {
    database_url: String,
    kafka_brokers: String,
    topic_search_sync: String,
    consumer_group: String,
    opensearch_endpoint: String,
    redis_namespace: String,
    redis_url: String,
    reindex_poll_interval_secs: u64,
}

#[derive(Debug, Clone)]
struct IndexedDocument {
    entity_id: String,
    document_version: i64,
    body: Value,
}

#[derive(Debug, Clone)]
struct AliasBinding {
    write_alias: String,
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
    consumer.subscribe(&[&cfg.topic_search_sync])?;

    info!(
        topic = %cfg.topic_search_sync,
        group = %cfg.consumer_group,
        "search-indexer started"
    );

    let mut interval = time::interval(Duration::from_secs(cfg.reindex_poll_interval_secs));
    loop {
        tokio::select! {
            message = consumer.recv() => match message {
                Ok(message) => {
                    if let Err(err) = handle_kafka_message(&db, &cfg, &message).await {
                        error!(error = %err, "search-indexer kafka event handling failed");
                    }
                    if let Err(err) = consumer.commit_message(&message, CommitMode::Async) {
                        warn!(error = %err, "search-indexer commit offset failed");
                    }
                }
                Err(err) => warn!(error = %err, "search-indexer kafka receive failed"),
            },
            _ = interval.tick() => {
                if let Err(err) = process_queued_reindex_tasks(&db, &cfg).await {
                    error!(error = %err, "search-indexer queued reindex sweep failed");
                }
            }
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
            topic_search_sync: std::env::var("TOPIC_SEARCH_SYNC")
                .unwrap_or_else(|_| "dtp.search.sync".to_string()),
            consumer_group: std::env::var("SEARCH_INDEXER_CONSUMER_GROUP")
                .unwrap_or_else(|_| "cg-search-indexer".to_string()),
            opensearch_endpoint: std::env::var("OPENSEARCH_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string()),
            redis_namespace: std::env::var("REDIS_NAMESPACE")
                .unwrap_or_else(|_| "datab:v1".to_string()),
            redis_url: resolve_redis_url(),
            reindex_poll_interval_secs: std::env::var("SEARCH_INDEXER_REINDEX_POLL_INTERVAL_SECS")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or(5),
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
    let envelope: Value = serde_json::from_slice(payload)
        .map_err(|err| format!("decode kafka payload failed: {err}"))?;
    let event_type = envelope["event_type"].as_str().unwrap_or_default();
    let aggregate_type = envelope["aggregate_type"].as_str().unwrap_or_default();
    let aggregate_id = envelope["aggregate_id"].as_str().unwrap_or_default();
    let source_event_id = envelope["event_id"].as_str();

    if event_type != "search.product.changed" && !event_type.starts_with("search.") {
        return Ok(());
    }

    if aggregate_type == "product" && !aggregate_id.is_empty() {
        process_entity(
            db,
            cfg,
            "product",
            aggregate_id,
            source_event_id,
            None,
            None,
        )
        .await?;
        if let Some(seller_org_id) = envelope["payload"]["seller_org_id"].as_str() {
            process_entity(
                db,
                cfg,
                "seller",
                seller_org_id,
                source_event_id,
                None,
                None,
            )
            .await?;
        }
        return Ok(());
    }

    if aggregate_type == "seller" && !aggregate_id.is_empty() {
        process_entity(db, cfg, "seller", aggregate_id, source_event_id, None, None).await?;
    }
    Ok(())
}

async fn process_queued_reindex_tasks(db: &AppDb, cfg: &WorkerConfig) -> Result<(), String> {
    let client = db
        .client()
        .map_err(|err| format!("acquire db client failed: {err}"))?;
    let rows = client
        .query(
            "SELECT
               index_sync_task_id::text,
               entity_scope,
               entity_id::text,
               source_event_id::text,
               target_index
             FROM search.index_sync_task
             WHERE sync_status = 'queued'
             ORDER BY scheduled_at ASC, updated_at ASC
             LIMIT 20",
            &[],
        )
        .await
        .map_err(|err| format!("load queued reindex tasks failed: {err}"))?;

    for row in rows {
        let task_id: String = row.get(0);
        let entity_scope: String = row.get(1);
        let entity_id: String = row.get(2);
        let source_event_id: Option<String> = row.get(3);
        let target_index: Option<String> = row.get(4);

        if let Err(err) = process_entity(
            db,
            cfg,
            &entity_scope,
            &entity_id,
            source_event_id.as_deref(),
            Some(task_id.as_str()),
            target_index.as_deref(),
        )
        .await
        {
            error!(
                task_id = %task_id,
                entity_scope = %entity_scope,
                entity_id = %entity_id,
                error = %err,
                "search-indexer queued task failed"
            );
        }
    }

    Ok(())
}

async fn process_entity(
    db: &AppDb,
    cfg: &WorkerConfig,
    entity_scope: &str,
    entity_id: &str,
    source_event_id: Option<&str>,
    existing_task_id: Option<&str>,
    target_index_override: Option<&str>,
) -> Result<(), String> {
    let client = db
        .client()
        .map_err(|err| format!("acquire db client failed: {err}"))?;
    let document = load_index_document(&client, entity_scope, entity_id).await?;
    let binding = load_alias_binding(&client, entity_scope).await?;
    let target_index = target_index_override.unwrap_or(binding.write_alias.as_str());
    let task_id = mark_task_processing(
        &client,
        entity_scope,
        entity_id,
        document.document_version,
        source_event_id,
        existing_task_id,
        Some(target_index),
    )
    .await?;

    let index_result = put_document(cfg, target_index, &document).await;
    match index_result {
        Ok(()) => {
            mark_projection_success(&client, entity_scope, entity_id).await?;
            invalidate_search_cache(cfg).await?;
            mark_task_completed(&client, &task_id).await?;
            info!(
                entity_scope = %entity_scope,
                entity_id = %entity_id,
                target_index = %target_index,
                "search-indexer synced projection"
            );
            Ok(())
        }
        Err(err) => {
            mark_projection_failed(&client, entity_scope, entity_id, &err).await?;
            mark_task_failed(&client, &task_id, &err).await?;
            Err(err)
        }
    }
}

async fn load_index_document(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
    entity_id: &str,
) -> Result<IndexedDocument, String> {
    let row = if entity_scope == "seller" {
        client
            .query_opt(
                "SELECT
                   org_id::text,
                   document_version::bigint,
                   jsonb_build_object(
                     'entity_scope', 'seller',
                     'id', org_id::text,
                     'name', seller_name,
                     'description', description,
                     'status', 'active',
                     'country_code', country_code,
                     'industry_tags', industry_tags,
                     'reputation_score', reputation_score,
                     'listing_product_count', listing_product_count,
                     'document_version', document_version,
                     'source_updated_at', to_char(source_updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                     'updated_at', to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                   )
                 FROM search.seller_search_document
                 WHERE org_id = $1::text::uuid",
                &[&entity_id],
            )
            .await
    } else {
        client
            .query_opt(
                "SELECT
                   product_id::text,
                   document_version::bigint,
                   jsonb_build_object(
                     'entity_scope', 'product',
                     'id', product_id::text,
                     'seller_id', org_id::text,
                     'name', title,
                     'title', title,
                     'subtitle', subtitle,
                     'description', description,
                     'category', category,
                     'product_type', product_type,
                     'industry', industry,
                     'tags', tags,
                     'use_cases', use_cases,
                     'seller_name', seller_name,
                     'seller_country_code', seller_country_code,
                     'delivery_modes', delivery_modes,
                     'price_amount', price_amount,
                     'currency_code', currency_code,
                     'status', listing_status,
                     'quality_score', quality_score,
                     'seller_reputation_score', seller_reputation_score,
                     'hotness_score', hotness_score,
                     'document_version', document_version,
                     'source_updated_at', to_char(source_updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                     'updated_at', to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                   )
                 FROM search.product_search_document
                 WHERE product_id = $1::text::uuid",
                &[&entity_id],
            )
            .await
    }
    .map_err(|err| format!("load search projection failed: {err}"))?;

    let Some(row) = row else {
        return Err(format!(
            "search projection missing for entity_scope={entity_scope} entity_id={entity_id}"
        ));
    };
    Ok(IndexedDocument {
        entity_id: row.get(0),
        document_version: row.get(1),
        body: row.get(2),
    })
}

async fn load_alias_binding(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
) -> Result<AliasBinding, String> {
    let row = client
        .query_opt(
            "SELECT write_alias
             FROM search.index_alias_binding
             WHERE entity_scope = $1
               AND backend_type = 'opensearch'
             LIMIT 1",
            &[&entity_scope],
        )
        .await
        .map_err(|err| format!("load search.index_alias_binding failed: {err}"))?
        .ok_or_else(|| format!("search.index_alias_binding missing for scope={entity_scope}"))?;
    Ok(AliasBinding {
        write_alias: row.get(0),
    })
}

async fn mark_task_processing(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
    entity_id: &str,
    document_version: i64,
    source_event_id: Option<&str>,
    existing_task_id: Option<&str>,
    target_index: Option<&str>,
) -> Result<String, String> {
    if let Some(task_id) = existing_task_id {
        client
            .execute(
                "UPDATE search.index_sync_task
                 SET sync_status = 'processing',
                     started_at = now(),
                     retry_count = retry_count + 1,
                     updated_at = now(),
                     last_error_code = NULL,
                     last_error_message = NULL
                 WHERE index_sync_task_id = $1::text::uuid",
                &[&task_id],
            )
            .await
            .map_err(|err| format!("update queued search sync task failed: {err}"))?;
        return Ok(task_id.to_string());
    }

    let row = client
        .query_one(
            "INSERT INTO search.index_sync_task (
               entity_scope,
               entity_id,
               document_version,
               target_backend,
               target_index,
               source_event_id,
               sync_status,
               started_at
             ) VALUES (
               $1,
               $2::text::uuid,
               $3,
               'opensearch',
               $4,
               $5::text::uuid,
               'processing',
               now()
             )
             RETURNING index_sync_task_id::text",
            &[
                &entity_scope,
                &entity_id,
                &document_version,
                &target_index,
                &source_event_id,
            ],
        )
        .await
        .map_err(|err| format!("insert search sync task failed: {err}"))?;
    Ok(row.get(0))
}

async fn mark_task_completed(
    client: &(impl GenericClient + Sync),
    task_id: &str,
) -> Result<(), String> {
    client
        .execute(
            "UPDATE search.index_sync_task
             SET sync_status = 'completed',
                 completed_at = now(),
                 updated_at = now(),
                 last_error_code = NULL,
                 last_error_message = NULL
             WHERE index_sync_task_id = $1::text::uuid",
            &[&task_id],
        )
        .await
        .map_err(|err| format!("mark search sync task completed failed: {err}"))?;
    Ok(())
}

async fn mark_task_failed(
    client: &(impl GenericClient + Sync),
    task_id: &str,
    message: &str,
) -> Result<(), String> {
    client
        .execute(
            "UPDATE search.index_sync_task
             SET sync_status = 'failed',
                 completed_at = now(),
                 updated_at = now(),
                 last_error_code = 'SEARCH_INDEX_FAILED',
                 last_error_message = $2
             WHERE index_sync_task_id = $1::text::uuid",
            &[&task_id, &message],
        )
        .await
        .map_err(|err| format!("mark search sync task failed failed: {err}"))?;
    Ok(())
}

async fn mark_projection_success(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
    entity_id: &str,
) -> Result<(), String> {
    if entity_scope == "seller" {
        client
            .execute(
                "UPDATE search.seller_search_document
                 SET index_sync_status = 'indexed',
                     indexed_at = now(),
                     last_index_error = NULL,
                     updated_at = now()
                 WHERE org_id = $1::text::uuid",
                &[&entity_id],
            )
            .await
    } else {
        client
            .execute(
                "UPDATE search.product_search_document
                 SET index_sync_status = 'indexed',
                     indexed_at = now(),
                     last_index_error = NULL,
                     updated_at = now()
                 WHERE product_id = $1::text::uuid",
                &[&entity_id],
            )
            .await
    }
    .map_err(|err| format!("mark search projection success failed: {err}"))?;
    Ok(())
}

async fn mark_projection_failed(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
    entity_id: &str,
    message: &str,
) -> Result<(), String> {
    if entity_scope == "seller" {
        client
            .execute(
                "UPDATE search.seller_search_document
                 SET index_sync_status = 'failed',
                     last_index_error = $2,
                     updated_at = now()
                 WHERE org_id = $1::text::uuid",
                &[&entity_id, &message],
            )
            .await
    } else {
        client
            .execute(
                "UPDATE search.product_search_document
                 SET index_sync_status = 'failed',
                     last_index_error = $2,
                     updated_at = now()
                 WHERE product_id = $1::text::uuid",
                &[&entity_id, &message],
            )
            .await
    }
    .map_err(|err| format!("mark search projection failed failed: {err}"))?;
    Ok(())
}

async fn put_document(
    cfg: &WorkerConfig,
    target_index: &str,
    document: &IndexedDocument,
) -> Result<(), String> {
    let response = reqwest::Client::new()
        .put(format!(
            "{}/{}/_doc/{}?refresh=wait_for",
            cfg.opensearch_endpoint.trim_end_matches('/'),
            target_index,
            document.entity_id
        ))
        .json(&document.body)
        .send()
        .await
        .map_err(|err| format!("opensearch document upsert failed: {err}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unavailable".to_string());
        Err(format!(
            "opensearch document upsert failed: status={status} body={body}"
        ))
    }
}

async fn invalidate_search_cache(cfg: &WorkerConfig) -> Result<(), String> {
    let client = redis::Client::open(cfg.redis_url.as_str())
        .map_err(|err| format!("redis client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis connect failed: {err}"))?;
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(format!("{}:search:catalog:*", cfg.redis_namespace))
        .query_async(&mut connection)
        .await
        .map_err(|err| format!("redis search cache key scan failed: {err}"))?;
    if !keys.is_empty() {
        let _: usize = connection
            .del(keys)
            .await
            .map_err(|err| format!("redis search cache delete failed: {err}"))?;
    }
    Ok(())
}

fn resolve_redis_url() -> String {
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
