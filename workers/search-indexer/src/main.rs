use db::{AppDb, DbPoolConfig, GenericClient};
use rdkafka::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use redis::AsyncCommands;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

const SERVICE_NAME: &str = "search-indexer";
const FAILURE_STAGE_CONSUMER_HANDLER: &str = "consumer_handler";

#[derive(Debug, Clone)]
struct WorkerConfig {
    database_url: String,
    kafka_brokers: String,
    topic_search_sync: String,
    dead_letter_topic: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessingGate {
    Proceed,
    Duplicate,
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
    let producer = build_producer(&cfg)?;

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
                    match process_kafka_message(&db, &cfg, &producer, &message).await {
                        Ok(result_code) => {
                            if let Err(err) = consumer.commit_message(&message, CommitMode::Async) {
                                warn!(error = %err, result_code, "search-indexer commit offset failed");
                            }
                        }
                        Err(err) => error!(error = %err, "search-indexer kafka event handling failed before safe isolation"),
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
            dead_letter_topic: std::env::var("TOPIC_DEAD_LETTER_EVENTS")
                .unwrap_or_else(|_| "dtp.dead-letter".to_string()),
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

fn build_producer(cfg: &WorkerConfig) -> Result<FutureProducer, rdkafka::error::KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", &cfg.kafka_brokers)
        .create()
}

async fn process_kafka_message(
    db: &AppDb,
    cfg: &WorkerConfig,
    producer: &FutureProducer,
    message: &rdkafka::message::BorrowedMessage<'_>,
) -> Result<&'static str, String> {
    let Some(payload) = message.payload() else {
        return Ok("ignored");
    };
    process_kafka_payload(db, cfg, producer, payload).await
}

async fn process_kafka_payload(
    db: &AppDb,
    cfg: &WorkerConfig,
    producer: &FutureProducer,
    payload: &[u8],
) -> Result<&'static str, String> {
    let envelope: Value = serde_json::from_slice(payload)
        .map_err(|err| format!("decode kafka payload failed: {err}"))?;
    if !is_search_sync_envelope(&envelope) {
        return Ok("ignored");
    }

    let event_id = envelope["event_id"]
        .as_str()
        .ok_or_else(|| "search-indexer envelope missing event_id".to_string())?;
    let aggregate_type = envelope["aggregate_type"].as_str();
    let aggregate_id = envelope["aggregate_id"].as_str();
    let trace_id = envelope["trace_id"].as_str();

    let client = db
        .client()
        .map_err(|err| format!("acquire db client failed: {err}"))?;
    match begin_processing_gate(
        &client,
        event_id,
        aggregate_type,
        aggregate_id,
        trace_id,
        &cfg.topic_search_sync,
    )
    .await?
    {
        ProcessingGate::Duplicate => return Ok("duplicate"),
        ProcessingGate::Proceed => {}
    }

    match process_search_envelope(db, cfg, &envelope).await {
        Ok(()) => {
            update_processing_result(
                &client,
                event_id,
                "processed",
                None,
                json!({
                    "source_topic": cfg.topic_search_sync,
                    "dead_letter_topic": cfg.dead_letter_topic,
                }),
            )
            .await?;
            Ok("processed")
        }
        Err(err) => {
            let dead_letter_event_id =
                ensure_dead_letter_event(&client, &envelope, &err, &cfg.topic_search_sync).await?;
            attach_dead_letter_to_failed_tasks(&client, event_id, &dead_letter_event_id).await?;
            publish_dead_letter_message(
                producer,
                &cfg.topic_search_sync,
                &cfg.dead_letter_topic,
                &cfg.consumer_group,
                &dead_letter_event_id,
                &envelope,
                &err,
            )
            .await?;
            update_processing_result(
                &client,
                event_id,
                "dead_lettered",
                Some(&err),
                json!({
                    "source_topic": cfg.topic_search_sync,
                    "dead_letter_topic": cfg.dead_letter_topic,
                    "dead_letter_event_id": dead_letter_event_id,
                    "consumer_group": cfg.consumer_group,
                }),
            )
            .await?;
            Ok("dead_lettered")
        }
    }
}

fn is_search_sync_envelope(envelope: &Value) -> bool {
    let event_type = envelope["event_type"].as_str().unwrap_or_default();
    event_type == "search.product.changed" || event_type.starts_with("search.")
}

async fn process_search_envelope(
    db: &AppDb,
    cfg: &WorkerConfig,
    envelope: &Value,
) -> Result<(), String> {
    let aggregate_type = envelope["aggregate_type"].as_str().unwrap_or_default();
    let aggregate_id = envelope["aggregate_id"].as_str().unwrap_or_default();
    let source_event_id = envelope["event_id"].as_str();

    if aggregate_type == "product" {
        if aggregate_id.is_empty() {
            return Err("search-indexer envelope missing product aggregate_id".to_string());
        }
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

    if aggregate_type == "seller" {
        if aggregate_id.is_empty() {
            return Err("search-indexer envelope missing seller aggregate_id".to_string());
        }
        process_entity(db, cfg, "seller", aggregate_id, source_event_id, None, None).await?;
    }

    Ok(())
}

async fn begin_processing_gate(
    client: &(impl GenericClient + Sync),
    event_id: &str,
    aggregate_type: Option<&str>,
    aggregate_id: Option<&str>,
    trace_id: Option<&str>,
    source_topic: &str,
) -> Result<ProcessingGate, String> {
    let row = client
        .query_opt(
            "SELECT result_code
             FROM ops.consumer_idempotency_record
             WHERE consumer_name = $1
               AND event_id = $2::text::uuid",
            &[&SERVICE_NAME, &event_id],
        )
        .await
        .map_err(|err| format!("load search-indexer idempotency gate failed: {err}"))?;

    if let Some(row) = row {
        let result_code: String = row.get(0);
        if matches!(result_code.as_str(), "processed" | "dead_lettered") {
            return Ok(ProcessingGate::Duplicate);
        }

        let metadata = json!({
            "source_topic": source_topic,
            "trace_id": trace_id,
            "updated_at": now_iso8601(),
        })
        .to_string();
        client
            .execute(
                "UPDATE ops.consumer_idempotency_record
                 SET result_code = 'processing',
                     processed_at = now(),
                     metadata = coalesce(metadata, '{}'::jsonb) || $3::jsonb
                 WHERE consumer_name = $1
                   AND event_id = $2::text::uuid",
                &[&SERVICE_NAME, &event_id, &metadata],
            )
            .await
            .map_err(|err| format!("reset search-indexer idempotency gate failed: {err}"))?;
        return Ok(ProcessingGate::Proceed);
    }

    let metadata = json!({
        "source_topic": source_topic,
        "trace_id": trace_id,
    })
    .to_string();
    client
        .execute(
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
               'processing',
               $6::jsonb
             )",
            &[
                &SERVICE_NAME,
                &event_id,
                &aggregate_type,
                &aggregate_id,
                &trace_id,
                &metadata,
            ],
        )
        .await
        .map_err(|err| format!("insert search-indexer idempotency gate failed: {err}"))?;
    Ok(ProcessingGate::Proceed)
}

async fn update_processing_result(
    client: &(impl GenericClient + Sync),
    event_id: &str,
    result_code: &str,
    error_message: Option<&str>,
    extra_metadata: Value,
) -> Result<(), String> {
    let metadata = json!({
        "updated_at": now_iso8601(),
        "last_error": error_message,
    })
    .as_object()
    .cloned()
    .unwrap_or_default();
    let mut merged = serde_json::Map::new();
    merged.extend(metadata);
    if let Some(extra) = extra_metadata.as_object() {
        for (key, value) in extra {
            merged.insert(key.clone(), value.clone());
        }
    }
    let metadata_json = Value::Object(merged).to_string();

    client
        .execute(
            "UPDATE ops.consumer_idempotency_record
             SET result_code = $3,
                 processed_at = now(),
                 metadata = coalesce(metadata, '{}'::jsonb) || $4::jsonb
             WHERE consumer_name = $1
               AND event_id = $2::text::uuid",
            &[&SERVICE_NAME, &event_id, &result_code, &metadata_json],
        )
        .await
        .map_err(|err| format!("update search-indexer idempotency result failed: {err}"))?;
    Ok(())
}

async fn ensure_dead_letter_event(
    client: &(impl GenericClient + Sync),
    envelope: &Value,
    error_message: &str,
    source_topic: &str,
) -> Result<String, String> {
    let event_id = envelope["event_id"]
        .as_str()
        .ok_or_else(|| "search-indexer dead letter requires event_id".to_string())?;
    let payload = envelope.to_string();
    let aggregate_type = envelope["aggregate_type"].as_str();
    let aggregate_id = envelope["aggregate_id"].as_str();
    let event_type = envelope["event_type"].as_str();
    let request_id = envelope["request_id"].as_str();
    let trace_id = effective_trace_id(envelope);

    let row = client
        .query_one(
            "WITH existing AS (
               SELECT dead_letter_event_id
                 FROM ops.dead_letter_event
                WHERE outbox_event_id = $1::text::uuid
                  AND failure_stage = $10
                ORDER BY created_at DESC, dead_letter_event_id DESC
                LIMIT 1
             ),
             updated AS (
               UPDATE ops.dead_letter_event
                  SET payload = $5::jsonb,
                      failed_reason = $6,
                      last_failed_at = now(),
                      target_topic = $9
                WHERE dead_letter_event_id IN (SELECT dead_letter_event_id FROM existing)
                RETURNING dead_letter_event_id::text
             ),
             inserted AS (
               INSERT INTO ops.dead_letter_event (
                  outbox_event_id,
                  aggregate_type,
                  aggregate_id,
                  event_type,
                  payload,
                  failed_reason,
                  request_id,
                  trace_id,
                  authority_scope,
                  source_of_truth,
                  target_bus,
                  target_topic,
                  failure_stage,
                  last_failed_at
               )
               SELECT
                  $1::text::uuid,
                  $2,
                  $3::text::uuid,
                  $4,
                  $5::jsonb,
                  $6,
                  $7,
                  $8,
                  'business',
                  'database',
                  'kafka',
                  $9,
                  $10,
                  now()
               WHERE NOT EXISTS (SELECT 1 FROM updated)
               RETURNING dead_letter_event_id::text
             )
             SELECT dead_letter_event_id FROM updated
             UNION ALL
             SELECT dead_letter_event_id FROM inserted
             LIMIT 1",
            &[
                &event_id,
                &aggregate_type,
                &aggregate_id,
                &event_type,
                &payload,
                &error_message,
                &request_id,
                &trace_id,
                &source_topic,
                &FAILURE_STAGE_CONSUMER_HANDLER,
            ],
        )
        .await
        .map_err(|err| format!("insert search-indexer dead letter failed: {err}"))?;
    Ok(row.get(0))
}

async fn publish_dead_letter_message(
    producer: &FutureProducer,
    source_topic: &str,
    dead_letter_topic: &str,
    consumer_group: &str,
    dead_letter_event_id: &str,
    envelope: &Value,
    error_message: &str,
) -> Result<(), String> {
    let payload = build_dead_letter_message(
        dead_letter_event_id,
        source_topic,
        dead_letter_topic,
        consumer_group,
        envelope,
        error_message,
    );
    let raw = serde_json::to_string(&payload)
        .map_err(|err| format!("encode search-indexer dead letter message failed: {err}"))?;
    producer
        .send(
            FutureRecord::to(dead_letter_topic)
                .payload(&raw)
                .key(dead_letter_event_id),
            Timeout::After(Duration::from_secs(3)),
        )
        .await
        .map_err(|(err, _)| format!("publish search-indexer dead letter failed: {err}"))?;
    Ok(())
}

fn build_dead_letter_message(
    dead_letter_event_id: &str,
    source_topic: &str,
    dead_letter_topic: &str,
    consumer_group: &str,
    envelope: &Value,
    error_message: &str,
) -> Value {
    json!({
        "dead_letter_event_id": dead_letter_event_id,
        "source_topic": source_topic,
        "target_topic": dead_letter_topic,
        "consumer_name": SERVICE_NAME,
        "consumer_group": consumer_group,
        "event_id": envelope["event_id"],
        "event_type": envelope["event_type"],
        "aggregate_type": envelope["aggregate_type"],
        "aggregate_id": envelope["aggregate_id"],
        "request_id": envelope["request_id"],
        "trace_id": effective_trace_id(envelope),
        "failure_stage": FAILURE_STAGE_CONSUMER_HANDLER,
        "failure_reason": error_message,
        "reprocess_status": "not_reprocessed",
        "payload": envelope,
    })
}

fn effective_trace_id(envelope: &Value) -> String {
    envelope["trace_id"]
        .as_str()
        .or_else(|| envelope["request_id"].as_str())
        .or_else(|| envelope["event_id"].as_str())
        .unwrap_or("trace-search-indexer-unknown")
        .to_string()
}

fn now_iso8601() -> String {
    format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
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
            invalidate_search_cache(cfg, entity_scope, &document.body).await?;
            resolve_task_exceptions(&client, entity_scope, entity_id).await?;
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
            record_task_exception(
                &client,
                &task_id,
                entity_scope,
                entity_id,
                document.document_version,
                target_index,
                source_event_id,
                "sync_failed",
                "index_write",
                "SEARCH_INDEX_FAILED",
                &err,
            )
            .await?;
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
                     'seller_type', seller_type,
                     'description', description,
                     'status', 'active',
                     'country_code', country_code,
                     'region_code', region_code,
                     'industry_tags', industry_tags,
                     'certification_tags', certification_tags,
                     'featured_products', featured_products,
                     'rating_summary', rating_summary,
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
                     'price_min', price_min,
                     'price_max', price_max,
                     'currency_code', currency_code,
                     'status', listing_status,
                     'review_status', review_status,
                     'visibility_status', visibility_status,
                     'visible_to_search', visible_to_search,
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
                     reconcile_status = 'pending_check',
                     last_error_code = NULL,
                     last_error_message = NULL,
                     dead_letter_event_id = NULL
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
                 reconcile_status = 'clean',
                 last_reconciled_at = now(),
                 last_error_code = NULL,
                 last_error_message = NULL,
                 dead_letter_event_id = NULL
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
                 reconcile_status = 'drift_detected',
                 last_reconciled_at = now(),
                 last_error_code = 'SEARCH_INDEX_FAILED',
                 last_error_message = $2
             WHERE index_sync_task_id = $1::text::uuid",
            &[&task_id, &message],
        )
        .await
        .map_err(|err| format!("mark search sync task failed failed: {err}"))?;
    Ok(())
}

async fn record_task_exception(
    client: &(impl GenericClient + Sync),
    task_id: &str,
    entity_scope: &str,
    entity_id: &str,
    document_version: i64,
    target_index: &str,
    source_event_id: Option<&str>,
    exception_type: &str,
    failure_stage: &str,
    error_code: &str,
    error_message: &str,
) -> Result<(), String> {
    let existing = client
        .query_opt(
            "SELECT index_sync_exception_id::text
             FROM search.index_sync_exception
             WHERE index_sync_task_id = $1::text::uuid
               AND exception_status = 'open'
             ORDER BY detected_at DESC, updated_at DESC
             LIMIT 1",
            &[&task_id],
        )
        .await
        .map_err(|err| format!("load open search sync exception failed: {err}"))?;
    if let Some(row) = existing {
        let exception_id: String = row.get(0);
        client
            .execute(
                "UPDATE search.index_sync_exception
                 SET failure_stage = $2,
                     error_code = $3,
                     error_message = $4,
                     retryable = true,
                     metadata = metadata || jsonb_build_object('updated_by', $5),
                     detected_at = now(),
                     resolved_at = NULL,
                     updated_at = now()
                 WHERE index_sync_exception_id = $1::text::uuid",
                &[
                    &exception_id,
                    &failure_stage,
                    &error_code,
                    &error_message,
                    &SERVICE_NAME,
                ],
            )
            .await
            .map_err(|err| format!("update search sync exception failed: {err}"))?;
        return Ok(());
    }

    client
        .execute(
            "INSERT INTO search.index_sync_exception (
               index_sync_task_id,
               entity_scope,
               entity_id,
               document_version,
               target_backend,
               target_index,
               source_event_id,
               exception_type,
               exception_status,
               failure_stage,
               error_code,
               error_message,
               retryable,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3::text::uuid,
               $4,
               'opensearch',
               $5,
               $6::text::uuid,
               $7,
               'open',
               $8,
               $9,
               $10,
               true,
               jsonb_build_object('recorded_by', $11)
             )",
            &[
                &task_id,
                &entity_scope,
                &entity_id,
                &document_version,
                &target_index,
                &source_event_id,
                &exception_type,
                &failure_stage,
                &error_code,
                &error_message,
                &SERVICE_NAME,
            ],
        )
        .await
        .map_err(|err| format!("insert search sync exception failed: {err}"))?;
    Ok(())
}

async fn resolve_task_exceptions(
    client: &(impl GenericClient + Sync),
    entity_scope: &str,
    entity_id: &str,
) -> Result<(), String> {
    client
        .execute(
            "UPDATE search.index_sync_exception
             SET exception_status = 'resolved',
                 resolved_at = now(),
                 retryable = false,
                 updated_at = now()
             WHERE entity_scope = $1
               AND entity_id = $2::text::uuid
               AND exception_status = 'open'",
            &[&entity_scope, &entity_id],
        )
        .await
        .map_err(|err| format!("resolve search sync exceptions failed: {err}"))?;
    Ok(())
}

async fn attach_dead_letter_to_failed_tasks(
    client: &(impl GenericClient + Sync),
    source_event_id: &str,
    dead_letter_event_id: &str,
) -> Result<(), String> {
    client
        .execute(
            "UPDATE search.index_sync_task
             SET dead_letter_event_id = $2::text::uuid,
                 reconcile_status = 'drift_detected',
                 last_reconciled_at = now(),
                 updated_at = now()
             WHERE source_event_id = $1::text::uuid
               AND sync_status = 'failed'",
            &[&source_event_id, &dead_letter_event_id],
        )
        .await
        .map_err(|err| format!("attach dead letter to search sync task failed: {err}"))?;
    client
        .execute(
            "UPDATE search.index_sync_exception
             SET dead_letter_event_id = $2::text::uuid,
                 failure_stage = COALESCE(failure_stage, $3),
                 updated_at = now()
             WHERE source_event_id = $1::text::uuid
               AND exception_status = 'open'",
            &[
                &source_event_id,
                &dead_letter_event_id,
                &FAILURE_STAGE_CONSUMER_HANDLER,
            ],
        )
        .await
        .map_err(|err| format!("attach dead letter to search sync exception failed: {err}"))?;
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

async fn invalidate_search_cache(
    cfg: &WorkerConfig,
    entity_scope: &str,
    document: &Value,
) -> Result<(), String> {
    let client = redis::Client::open(cfg.redis_url.as_str())
        .map_err(|err| format!("redis client init failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("redis connect failed: {err}"))?;
    let scopes = related_cache_scopes_for_entity(entity_scope, document);
    bump_search_cache_versions(&mut connection, cfg, &scopes).await?;
    let mut keys = BTreeSet::new();
    for scope in &scopes {
        keys.extend(
            scan_search_cache_keys(
                &mut connection,
                &format!("{}:search:catalog:{}:*", cfg.redis_namespace, scope),
            )
            .await?,
        );
    }
    if !keys.is_empty() {
        let _: usize = connection
            .del(keys.into_iter().collect::<Vec<_>>())
            .await
            .map_err(|err| format!("redis search cache delete failed: {err}"))?;
    }
    Ok(())
}

fn related_cache_scopes_for_entity(entity_scope: &str, document: &Value) -> Vec<String> {
    match entity_scope {
        "seller" => vec!["seller".to_string(), "all".to_string()],
        _ => {
            let mut scopes = vec!["product".to_string(), "all".to_string()];
            if document["product_type"].as_str() == Some("service") {
                scopes.insert(1, "service".to_string());
            }
            scopes
        }
    }
}

fn search_cache_version_key(cfg: &WorkerConfig, scope: &str) -> String {
    format!("{}:search:catalog:version:{}", cfg.redis_namespace, scope)
}

async fn bump_search_cache_versions(
    connection: &mut redis::aio::MultiplexedConnection,
    cfg: &WorkerConfig,
    scopes: &[String],
) -> Result<(), String> {
    let mut unique_scopes = BTreeSet::new();
    unique_scopes.extend(scopes.iter().cloned());
    for scope in unique_scopes {
        connection
            .incr::<_, _, i64>(search_cache_version_key(cfg, &scope), 1)
            .await
            .map_err(|err| format!("redis search cache version bump failed: {err}"))?;
    }
    Ok(())
}

async fn scan_search_cache_keys(
    connection: &mut redis::aio::MultiplexedConnection,
    pattern: &str,
) -> Result<Vec<String>, String> {
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
            .map_err(|err| format!("redis search cache key scan failed: {err}"))?;
        keys.extend(batch);
        if next_cursor == 0 {
            break;
        }
        cursor = next_cursor;
    }
    Ok(keys)
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
    format!("redis://default:{}@{}:{}/0", password, host, port)
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::{Client, Error, NoTls, connect};
    use tokio::time::timeout;

    #[derive(Debug)]
    struct SeedGraph {
        org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        cache_key: String,
    }

    fn live_db_enabled() -> bool {
        std::env::var("SEARCHREC_WORKER_DB_SMOKE").ok().as_deref() == Some("1")
            || std::env::var("SEARCH_DB_SMOKE").ok().as_deref() == Some("1")
    }

    fn database_url() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string()
        })
    }

    fn kafka_brokers() -> String {
        std::env::var("KAFKA_BROKERS")
            .or_else(|_| std::env::var("KAFKA_BOOTSTRAP_SERVERS"))
            .unwrap_or_else(|_| "127.0.0.1:9094".to_string())
    }

    fn redis_url() -> String {
        resolve_redis_url()
    }

    fn opensearch_endpoint() -> String {
        std::env::var("OPENSEARCH_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9200".to_string())
    }

    fn worker_config() -> WorkerConfig {
        WorkerConfig {
            database_url: database_url(),
            kafka_brokers: kafka_brokers(),
            topic_search_sync: "dtp.search.sync".to_string(),
            dead_letter_topic: "dtp.dead-letter".to_string(),
            consumer_group: "cg-search-indexer-test".to_string(),
            opensearch_endpoint: opensearch_endpoint(),
            redis_namespace: "datab:v1".to_string(),
            redis_url: redis_url(),
            reindex_poll_interval_secs: 5,
        }
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, Error> {
        let org_row = client
            .query_one(
                "INSERT INTO core.organization (
                   org_name,
                   org_type,
                   status,
                   real_name_status,
                   compliance_level,
                   country_code,
                   region_code,
                   industry_tags,
                   metadata
                 ) VALUES (
                   $1,
                   'enterprise',
                   'active',
                   'verified',
                   'L2',
                   'CN',
                   'SH',
                   ARRAY['industrial_manufacturing']::text[],
                   jsonb_build_object(
                     'source', 'search-indexer-smoke',
                     'certification_level', 'enhanced',
                     'certification_tags', jsonb_build_array('iso27001')
                   )
                 )
                 RETURNING org_id::text",
                &[&format!("search-indexer-org-{suffix}")],
            )
            .await?;
        let org_id: String = org_row.get(0);

        let asset_row = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft'
                 )
                 RETURNING asset_id::text",
                &[&org_id, &format!("search-indexer-asset-{suffix}")],
            )
            .await?;
        let asset_id: String = asset_row.get(0);

        let asset_version_row = client
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
        let asset_version_id: String = asset_version_row.get(0);

        let product_row = client
            .query_one(
                "INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3::text::uuid,
                   $4,
                   'manufacturing',
                   'data_product',
                   $5,
                   'listed',
                   'one_time',
                   128.0,
                   'CNY',
                   'file_download',
                   ARRAY['internal_use']::text[],
                   $6,
                   jsonb_build_object(
                     'subtitle', $7,
                     'industry', 'industrial_manufacturing',
                     'quality_score', '0.92'
                   )
                 )
                 RETURNING product_id::text",
                &[
                    &asset_id,
                    &asset_version_id,
                    &org_id,
                    &format!("search-indexer-product-{suffix}"),
                    &format!("search-indexer product {suffix}"),
                    &format!("search indexer keyword {suffix}"),
                    &format!("search indexer subtitle {suffix}"),
                ],
            )
            .await?;
        let product_id: String = product_row.get(0);

        client
            .execute(
                "INSERT INTO risk.reputation_snapshot (
                   subject_type,
                   subject_id,
                   score,
                   risk_level,
                   credit_level,
                   effective_at,
                   metadata
                 ) VALUES (
                   'organization',
                   $1::text::uuid,
                   0.91,
                   1,
                   4,
                   now(),
                   jsonb_build_object(
                     'rating_count', 9,
                     'average_rating', 4.6,
                     'last_rating_at', '2026-04-22T00:00:00.000Z'
                   )
                 )",
                &[&org_id],
            )
            .await?;

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

        Ok(SeedGraph {
            org_id: org_id.clone(),
            asset_id,
            asset_version_id,
            product_id,
            cache_key: format!("datab:v1:search:catalog:product:search-indexer-{suffix}"),
        })
    }

    async fn seed_outbox_event(
        client: &Client,
        event_id: &str,
        seed: &SeedGraph,
        request_id: &str,
        trace_id: &str,
    ) -> Result<(), Error> {
        client
            .execute(
                "INSERT INTO ops.outbox_event (
                   outbox_event_id,
                   aggregate_type,
                   aggregate_id,
                   event_type,
                   payload,
                   status,
                   request_id,
                   trace_id,
                   authority_scope,
                   source_of_truth,
                   proof_commit_policy,
                   target_bus,
                   target_topic,
                   partition_key,
                   ordering_key
                 ) VALUES (
                   $1::text::uuid,
                   'product',
                   $2::text::uuid,
                   'search.product.changed',
                   jsonb_build_object(
                     'seller_org_id', $3::text::uuid,
                     'source', 'search-indexer-smoke'
                   ),
                   'published',
                   $4,
                   $5,
                   'business',
                   'database',
                   'async_evidence',
                   'kafka',
                   'dtp.search.sync',
                   $2,
                   $2
                 )",
                &[
                    &event_id,
                    &seed.product_id,
                    &seed.org_id,
                    &request_id,
                    &trace_id,
                ],
            )
            .await?;
        Ok(())
    }

    async fn cleanup_event_artifacts(client: &Client, event_id: &str) {
        let _ = client
            .execute(
                "DELETE FROM search.index_sync_exception
                 WHERE source_event_id = $1::text::uuid",
                &[&event_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM ops.consumer_idempotency_record
                 WHERE consumer_name = $1
                   AND event_id = $2::text::uuid",
                &[&SERVICE_NAME, &event_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM ops.dead_letter_event
                 WHERE outbox_event_id = $1::text::uuid
                   AND failure_stage = $2",
                &[&event_id, &FAILURE_STAGE_CONSUMER_HANDLER],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM search.index_sync_task
                 WHERE source_event_id = $1::text::uuid",
                &[&event_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event
                 WHERE outbox_event_id = $1::text::uuid",
                &[&event_id],
            )
            .await;
    }

    async fn cleanup_graph(client: &Client, cfg: &WorkerConfig, seed: &SeedGraph) {
        let _ = delete_opensearch_document(cfg, "product", &seed.product_id).await;
        let _ = delete_opensearch_document(cfg, "seller", &seed.org_id).await;
        let _ = client
            .execute(
                "DELETE FROM search.index_sync_exception
                 WHERE entity_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.product_id, &seed.org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM search.index_sync_task
                 WHERE entity_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.product_id, &seed.org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM search.product_search_document WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM search.seller_search_document WHERE org_id = $1::text::uuid",
                &[&seed.org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM risk.reputation_snapshot
                 WHERE subject_type = 'organization'
                   AND subject_id = $1::text::uuid",
                &[&seed.org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&seed.asset_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seed.org_id],
            )
            .await;
    }

    async fn lookup_write_alias(client: &Client, entity_scope: &str) -> String {
        client
            .query_one(
                "SELECT write_alias
                 FROM search.index_alias_binding
                 WHERE entity_scope = $1
                   AND backend_type = 'opensearch'
                 LIMIT 1",
                &[&entity_scope],
            )
            .await
            .expect("load write alias")
            .get(0)
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
            .expect("seed search cache");
    }

    async fn cache_exists(cfg: &WorkerConfig, key: &str) -> bool {
        let client = redis::Client::open(cfg.redis_url.as_str()).expect("redis client");
        let mut connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("redis connect");
        let value: Option<String> = connection.get(key).await.expect("get search cache");
        value.is_some()
    }

    async fn cache_version(cfg: &WorkerConfig, scope: &str) -> i64 {
        let client = redis::Client::open(cfg.redis_url.as_str()).expect("redis client");
        let mut connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("redis connect");
        connection
            .get::<_, Option<i64>>(search_cache_version_key(cfg, scope))
            .await
            .expect("get search cache version")
            .unwrap_or(0)
    }

    fn build_consumer(group_id: &str, topic: &str) -> StreamConsumer {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &kafka_brokers())
            .set("group.id", group_id)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("create kafka consumer");
        consumer.subscribe(&[topic]).expect("subscribe topic");
        consumer
    }

    async fn wait_for_dead_letter_message(
        consumer: &StreamConsumer,
        expected_event_id: &str,
    ) -> Value {
        timeout(Duration::from_secs(10), async {
            loop {
                let message = consumer.recv().await.expect("recv kafka message");
                let Some(payload) = message.payload() else {
                    continue;
                };
                let decoded: Value =
                    serde_json::from_slice(payload).expect("decode dead letter message");
                if decoded["event_id"].as_str() == Some(expected_event_id) {
                    return decoded;
                }
            }
        })
        .await
        .expect("wait for dead letter message")
    }

    async fn fetch_indexed_document(
        cfg: &WorkerConfig,
        entity_scope: &str,
        entity_id: &str,
    ) -> Value {
        let alias = if entity_scope == "seller" {
            "seller"
        } else {
            "product"
        };
        let write_alias = {
            let (client, connection) = connect(&cfg.database_url, NoTls)
                .await
                .expect("connect database");
            tokio::spawn(async move {
                let _ = connection.await;
            });
            lookup_write_alias(&client, alias).await
        };
        reqwest::Client::new()
            .get(format!(
                "{}/{}/_doc/{}",
                cfg.opensearch_endpoint.trim_end_matches('/'),
                write_alias,
                entity_id
            ))
            .send()
            .await
            .expect("fetch indexed document")
            .json::<Value>()
            .await
            .expect("decode indexed document")
    }

    async fn delete_opensearch_document(
        cfg: &WorkerConfig,
        entity_scope: &str,
        entity_id: &str,
    ) -> Result<(), String> {
        let (client, connection) = connect(&cfg.database_url, NoTls)
            .await
            .map_err(|err| format!("connect database for opensearch cleanup failed: {err}"))?;
        tokio::spawn(async move {
            let _ = connection.await;
        });
        let write_alias = lookup_write_alias(&client, entity_scope).await;
        let response = reqwest::Client::new()
            .delete(format!(
                "{}/{}/_doc/{}?refresh=wait_for",
                cfg.opensearch_endpoint.trim_end_matches('/'),
                write_alias,
                entity_id
            ))
            .send()
            .await
            .map_err(|err| format!("delete opensearch document failed: {err}"))?;
        let status = response.status();
        if status.is_success() || status.as_u16() == 404 {
            Ok(())
        } else {
            Err(format!(
                "delete opensearch document failed with status {status}"
            ))
        }
    }

    async fn dead_letter_row(client: &Client, event_id: &str) -> (String, String, String) {
        let row = client
            .query_one(
                "SELECT dead_letter_event_id::text, target_topic, reprocess_status
                 FROM ops.dead_letter_event
                 WHERE outbox_event_id = $1::text::uuid
                   AND failure_stage = $2",
                &[&event_id, &FAILURE_STAGE_CONSUMER_HANDLER],
            )
            .await
            .expect("load dead letter row");
        (row.get(0), row.get(1), row.get(2))
    }

    #[tokio::test]
    async fn search_indexer_db_smoke() {
        if !live_db_enabled() {
            return;
        }

        let cfg = worker_config();
        let producer = build_producer(&cfg).expect("create kafka producer");
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
        let product_version_before = cache_version(&cfg, "product").await;
        let service_version_before = cache_version(&cfg, "service").await;
        let all_version_before = cache_version(&cfg, "all").await;

        let success_event_id: String = client
            .query_one("SELECT gen_random_uuid()::text", &[])
            .await
            .expect("generate success event id")
            .get(0);
        let success_request_id = format!("req-search-indexer-success-{suffix}");
        let success_trace_id = format!("trace-search-indexer-success-{suffix}");
        seed_outbox_event(
            &client,
            &success_event_id,
            &seed,
            &success_request_id,
            &success_trace_id,
        )
        .await
        .expect("seed success outbox event");
        let success_envelope = json!({
            "event_id": success_event_id,
            "event_type": "search.product.changed",
            "aggregate_type": "product",
            "aggregate_id": seed.product_id,
            "request_id": success_request_id,
            "trace_id": success_trace_id,
            "payload": {
                "seller_org_id": seed.org_id
            }
        });
        let success_result = process_kafka_payload(
            &db,
            &cfg,
            &producer,
            success_envelope.to_string().as_bytes(),
        )
        .await
        .expect("process search sync envelope");
        assert_eq!(success_result, "processed");

        let indexed_document = fetch_indexed_document(&cfg, "product", &seed.product_id).await;
        assert_eq!(
            indexed_document["_source"]["id"].as_str(),
            Some(seed.product_id.as_str())
        );
        assert_eq!(
            indexed_document["_source"]["review_status"].as_str(),
            Some("approved")
        );
        assert_eq!(
            indexed_document["_source"]["visibility_status"].as_str(),
            Some("visible")
        );
        assert_eq!(
            indexed_document["_source"]["visible_to_search"].as_bool(),
            Some(true)
        );
        let seller_document = fetch_indexed_document(&cfg, "seller", &seed.org_id).await;
        assert_eq!(
            seller_document["_source"]["id"].as_str(),
            Some(seed.org_id.as_str())
        );
        assert_eq!(
            seller_document["_source"]["certification_tags"][0].as_str(),
            Some("certification:enhanced")
        );
        assert_eq!(
            seller_document["_source"]["featured_products"][0]["title"].as_str(),
            Some(format!("search-indexer-product-{suffix}").as_str())
        );
        assert_eq!(
            seller_document["_source"]["rating_summary"]["average_rating"].as_f64(),
            Some(4.6)
        );
        assert_eq!(
            seller_document["_source"]["rating_summary"]["rating_count"].as_i64(),
            Some(9)
        );
        assert!(!cache_exists(&cfg, &seed.cache_key).await);
        assert!(cache_version(&cfg, "product").await > product_version_before);
        assert_eq!(cache_version(&cfg, "service").await, service_version_before);
        assert!(cache_version(&cfg, "all").await > all_version_before);

        let success_idempotency = client
            .query_one(
                "SELECT result_code
                 FROM ops.consumer_idempotency_record
                 WHERE consumer_name = $1
                   AND event_id = $2::text::uuid",
                &[&SERVICE_NAME, &success_event_id],
            )
            .await
            .expect("load success idempotency row");
        assert_eq!(success_idempotency.get::<_, String>(0), "processed");
        let resolved_exception_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM search.index_sync_exception
                 WHERE entity_scope = 'product'
                   AND entity_id = $1::text::uuid
                   AND exception_status = 'open'",
                &[&seed.product_id],
            )
            .await
            .expect("count open exceptions after success")
            .get(0);
        assert_eq!(resolved_exception_count, 0);

        let duplicate_result = process_kafka_payload(
            &db,
            &cfg,
            &producer,
            success_envelope.to_string().as_bytes(),
        )
        .await
        .expect("process duplicated search sync envelope");
        assert_eq!(duplicate_result, "duplicate");

        let failure_event_id: String = client
            .query_one("SELECT gen_random_uuid()::text", &[])
            .await
            .expect("generate failure event id")
            .get(0);
        let failure_request_id = format!("req-search-indexer-failure-{suffix}");
        let failure_trace_id = format!("trace-search-indexer-failure-{suffix}");
        seed_outbox_event(
            &client,
            &failure_event_id,
            &seed,
            &failure_request_id,
            &failure_trace_id,
        )
        .await
        .expect("seed failure outbox event");
        let failure_consumer = build_consumer(
            &format!("cg-search-indexer-dlq-{suffix}"),
            &cfg.dead_letter_topic,
        );
        let failure_cfg = WorkerConfig {
            opensearch_endpoint: "http://127.0.0.1:1".to_string(),
            ..cfg.clone()
        };
        seed_cache(&failure_cfg, &seed.cache_key).await;
        let failure_envelope = json!({
            "event_id": failure_event_id,
            "event_type": "search.product.changed",
            "aggregate_type": "product",
            "aggregate_id": seed.product_id,
            "request_id": failure_request_id,
            "trace_id": failure_trace_id,
            "payload": {
                "seller_org_id": seed.org_id
            }
        });
        let failure_result = process_kafka_payload(
            &db,
            &failure_cfg,
            &producer,
            failure_envelope.to_string().as_bytes(),
        )
        .await
        .expect("process failing search sync envelope");
        assert_eq!(failure_result, "dead_lettered");

        let dead_letter_message =
            wait_for_dead_letter_message(&failure_consumer, &failure_event_id).await;
        let (dead_letter_event_id, target_topic, reprocess_status) =
            dead_letter_row(&client, &failure_event_id).await;
        assert_eq!(target_topic, cfg.topic_search_sync);
        assert_eq!(reprocess_status, "not_reprocessed");
        assert_eq!(
            dead_letter_message["dead_letter_event_id"].as_str(),
            Some(dead_letter_event_id.as_str())
        );
        assert_eq!(
            dead_letter_message["source_topic"].as_str(),
            Some(cfg.topic_search_sync.as_str())
        );
        assert_eq!(
            dead_letter_message["target_topic"].as_str(),
            Some(cfg.dead_letter_topic.as_str())
        );
        assert_eq!(
            dead_letter_message["consumer_name"].as_str(),
            Some(SERVICE_NAME)
        );

        let failure_idempotency = client
            .query_one(
                "SELECT result_code
                 FROM ops.consumer_idempotency_record
                 WHERE consumer_name = $1
                   AND event_id = $2::text::uuid",
                &[&SERVICE_NAME, &failure_event_id],
            )
            .await
            .expect("load failure idempotency row");
        assert_eq!(failure_idempotency.get::<_, String>(0), "dead_lettered");

        let failed_task_count: i64 = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM search.index_sync_task
                 WHERE source_event_id = $1::text::uuid
                   AND sync_status = 'failed'",
                &[&failure_event_id],
            )
            .await
            .expect("count failed search sync tasks")
            .get(0);
        assert!(failed_task_count >= 1);
        let failure_exception = client
            .query_one(
                "SELECT
                   exception_type,
                   exception_status,
                   error_code,
                   retryable,
                   dead_letter_event_id::text
                 FROM search.index_sync_exception
                 WHERE source_event_id = $1::text::uuid
                 ORDER BY detected_at DESC, updated_at DESC
                 LIMIT 1",
                &[&failure_event_id],
            )
            .await
            .expect("load search sync exception");
        assert_eq!(failure_exception.get::<_, String>(0), "sync_failed");
        assert_eq!(failure_exception.get::<_, String>(1), "open");
        assert_eq!(failure_exception.get::<_, String>(2), "SEARCH_INDEX_FAILED");
        assert!(failure_exception.get::<_, bool>(3));
        assert_eq!(
            failure_exception.get::<_, Option<String>>(4).as_deref(),
            Some(dead_letter_event_id.as_str())
        );
        let failed_task_state = client
            .query_one(
                "SELECT
                   reconcile_status,
                   dead_letter_event_id::text
                 FROM search.index_sync_task
                 WHERE source_event_id = $1::text::uuid
                 ORDER BY updated_at DESC
                 LIMIT 1",
                &[&failure_event_id],
            )
            .await
            .expect("load failed sync task state");
        assert_eq!(failed_task_state.get::<_, String>(0), "drift_detected");
        assert_eq!(
            failed_task_state.get::<_, Option<String>>(1).as_deref(),
            Some(dead_letter_event_id.as_str())
        );

        cleanup_event_artifacts(&client, &success_event_id).await;
        cleanup_event_artifacts(&client, &failure_event_id).await;
        cleanup_graph(&client, &cfg, &seed).await;
    }
}
