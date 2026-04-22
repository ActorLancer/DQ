use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use db::{AppDb, DbPoolConfig, GenericClient};
use prometheus::{Encoder, IntCounterVec, IntGauge, Registry, TextEncoder};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

const FAILURE_STAGE_OUTBOX_PUBLISH: &str = "outbox.publish";

#[derive(Debug, Clone)]
struct WorkerConfig {
    database_url: String,
    kafka_brokers: String,
    dead_letter_topic: String,
    app_host: String,
    app_port: u16,
    batch_size: usize,
    poll_interval_ms: u64,
    publish_timeout_ms: u64,
    retry_backoff_ms: u64,
    worker_id: String,
}

#[derive(Debug, Clone)]
struct ClaimedOutboxEvent {
    outbox_event_id: String,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    payload: Value,
    status: String,
    retry_count: i32,
    max_retries: i32,
    request_id: Option<String>,
    trace_id: Option<String>,
    idempotency_key: Option<String>,
    authority_scope: String,
    source_of_truth: String,
    proof_commit_policy: String,
    target_bus: String,
    target_topic: Option<String>,
    partition_key: Option<String>,
    ordering_key: Option<String>,
}

#[derive(Debug, Clone)]
struct PublishOutcome {
    outbox_event_id: String,
    request_id: Option<String>,
    trace_id: Option<String>,
    target_topic: Option<String>,
    result_code: &'static str,
}

#[derive(Clone)]
struct WorkerState {
    cfg: WorkerConfig,
    db: Arc<AppDb>,
    producer: FutureProducer,
    metrics: Arc<WorkerMetrics>,
    ready: Arc<AtomicBool>,
}

struct WorkerMetrics {
    registry: Registry,
    publish_attempts: IntCounterVec,
    cycle_claimed: IntGauge,
    pending_events: IntGauge,
}

#[derive(Debug, Serialize)]
struct HealthPayload {
    status: &'static str,
    worker_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .try_init()
        .ok();

    let cfg = WorkerConfig::from_env();
    let db = Arc::new(
        AppDb::connect(
            DbPoolConfig {
                dsn: cfg.database_url.clone(),
                max_connections: 8,
            }
            .into(),
        )
        .await?,
    );
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &cfg.kafka_brokers)
        .set("message.timeout.ms", cfg.publish_timeout_ms.to_string())
        .create()?;
    let metrics = Arc::new(WorkerMetrics::new()?);
    let ready = Arc::new(AtomicBool::new(true));
    let state = Arc::new(WorkerState {
        cfg,
        db,
        producer,
        metrics,
        ready,
    });

    let http_state = state.clone();
    tokio::spawn(async move {
        if let Err(err) = serve_http(http_state).await {
            error!(error = %err, "outbox-publisher http server stopped");
        }
    });

    info!(
        worker_id = %state.cfg.worker_id,
        kafka_brokers = %state.cfg.kafka_brokers,
        dead_letter_topic = %state.cfg.dead_letter_topic,
        batch_size = state.cfg.batch_size,
        "outbox-publisher started"
    );

    let mut interval = time::interval(Duration::from_millis(state.cfg.poll_interval_ms));
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("outbox-publisher received shutdown signal");
                break;
            }
            _ = interval.tick() => {
                if let Err(err) = run_publish_cycle(&state).await {
                    error!(error = %err, "outbox-publisher publish cycle failed");
                }
            }
        }
    }

    state.ready.store(false, Ordering::Relaxed);
    Ok(())
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
            dead_letter_topic: std::env::var("TOPIC_DEAD_LETTER_EVENTS")
                .unwrap_or_else(|_| "dtp.dead-letter".to_string()),
            app_host: std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            app_port: std::env::var("APP_PORT")
                .ok()
                .and_then(|raw| raw.parse::<u16>().ok())
                .unwrap_or(8098),
            batch_size: std::env::var("OUTBOX_PUBLISHER_BATCH_SIZE")
                .ok()
                .and_then(|raw| raw.parse::<usize>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(20),
            poll_interval_ms: std::env::var("OUTBOX_PUBLISHER_POLL_INTERVAL_MS")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or(500),
            publish_timeout_ms: std::env::var("OUTBOX_PUBLISHER_TIMEOUT_MS")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or(3000),
            retry_backoff_ms: std::env::var("OUTBOX_PUBLISHER_RETRY_BACKOFF_MS")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or(500),
            worker_id: std::env::var("OUTBOX_PUBLISHER_WORKER_ID")
                .unwrap_or_else(|_| "outbox-publisher".to_string()),
        }
    }
}

impl WorkerMetrics {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        let publish_attempts = IntCounterVec::new(
            prometheus::Opts::new(
                "outbox_publisher_publish_attempts_total",
                "Outbox publisher publish attempt results",
            ),
            &["result"],
        )?;
        let cycle_claimed = IntGauge::new(
            "outbox_publisher_cycle_claimed_events",
            "Outbox events claimed in the latest publish cycle",
        )?;
        let pending_events = IntGauge::new(
            "outbox_publisher_pending_events",
            "Current pending outbox events waiting to publish",
        )?;
        registry.register(Box::new(publish_attempts.clone()))?;
        registry.register(Box::new(cycle_claimed.clone()))?;
        registry.register(Box::new(pending_events.clone()))?;
        Ok(Self {
            registry,
            publish_attempts,
            cycle_claimed,
            pending_events,
        })
    }
}

async fn serve_http(state: Arc<WorkerState>) -> Result<(), String> {
    let host = state
        .cfg
        .app_host
        .parse::<IpAddr>()
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let addr = SocketAddr::from((host, state.cfg.app_port));
    let app = Router::new()
        .route("/health/live", get(live_handler))
        .route("/health/ready", get(ready_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|err| format!("bind http listener failed: {err}"))?;
    axum::serve(listener, app)
        .await
        .map_err(|err| format!("serve http failed: {err}"))
}

async fn live_handler(State(state): State<Arc<WorkerState>>) -> Json<HealthPayload> {
    Json(HealthPayload {
        status: "live",
        worker_id: state.cfg.worker_id.clone(),
    })
}

async fn ready_handler(
    State(state): State<Arc<WorkerState>>,
) -> Result<Json<HealthPayload>, Response> {
    if state.ready.load(Ordering::Relaxed) {
        Ok(Json(HealthPayload {
            status: "ready",
            worker_id: state.cfg.worker_id.clone(),
        }))
    } else {
        Err((
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "not_ready"})),
        )
            .into_response())
    }
}

async fn metrics_handler(State(state): State<Arc<WorkerState>>) -> Result<Response, Response> {
    let metric_families = state.metrics.registry.gather();
    let mut buffer = Vec::new();
    TextEncoder::new()
        .encode(&metric_families, &mut buffer)
        .map_err(|err| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("encode metrics failed: {err}")})),
            )
                .into_response()
        })?;
    let body = String::from_utf8(buffer).map_err(|err| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("decode metrics failed: {err}")})),
        )
            .into_response()
    })?;
    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            TextEncoder::new().format_type().to_string(),
        )],
        body,
    )
        .into_response())
}

async fn run_publish_cycle(state: &Arc<WorkerState>) -> Result<(), String> {
    let mut claimed = 0usize;
    for _ in 0..state.cfg.batch_size {
        match process_one_pending_event(state).await? {
            Some(outcome) => {
                claimed += 1;
                info!(
                    worker_id = %state.cfg.worker_id,
                    outbox_event_id = %outcome.outbox_event_id,
                    result_code = %outcome.result_code,
                    target_topic = outcome.target_topic.as_deref().unwrap_or_default(),
                    request_id = outcome.request_id.as_deref().unwrap_or_default(),
                    trace_id = outcome.trace_id.as_deref().unwrap_or_default(),
                    "outbox-publisher processed event"
                );
            }
            None => break,
        }
    }
    state.metrics.cycle_claimed.set(claimed as i64);
    let pending_count = load_pending_event_count(&state.db).await?;
    state.metrics.pending_events.set(pending_count);
    Ok(())
}

async fn process_one_pending_event(
    state: &Arc<WorkerState>,
) -> Result<Option<PublishOutcome>, String> {
    process_pending_event_inner(state, None).await
}

async fn process_pending_event_inner(
    state: &Arc<WorkerState>,
    selected_outbox_event_id: Option<&str>,
) -> Result<Option<PublishOutcome>, String> {
    let client = state
        .db
        .client()
        .map_err(|err| format!("acquire outbox publisher db client failed: {err}"))?;
    let tx = client
        .transaction()
        .await
        .map_err(|err| format!("open outbox publisher transaction failed: {err}"))?;
    let Some(event) =
        claim_next_outbox_event(&tx, &state.cfg.worker_id, selected_outbox_event_id).await?
    else {
        tx.rollback()
            .await
            .map_err(|err| format!("rollback empty outbox publisher transaction failed: {err}"))?;
        return Ok(None);
    };
    let target_topic = event.target_topic.clone();
    let request_id = event.request_id.clone();
    let trace_id = event.trace_id.clone();
    let attempt_no = event.retry_count + 1;
    let publish_result = publish_main_event(state, &event).await;

    let outcome = match publish_result {
        Ok(()) => {
            insert_publish_attempt(
                &tx,
                &event,
                attempt_no,
                "published",
                None,
                None,
                &json!({
                    "worker_id": state.cfg.worker_id,
                    "partition_key": event.partition_key,
                    "ordering_key": event.ordering_key,
                    "claimed_status": event.status,
                    "idempotency_key": event.idempotency_key,
                    "proof_commit_policy": event.proof_commit_policy,
                }),
            )
            .await?;
            mark_outbox_published(&tx, &event.outbox_event_id).await?;
            record_worker_audit_event(
                &tx,
                "outbox_event",
                &event.outbox_event_id,
                "outbox.publisher.publish",
                "success",
                event.request_id.as_deref(),
                event.trace_id.as_deref(),
                &json!({
                    "worker_id": state.cfg.worker_id,
                    "event_type": event.event_type,
                    "target_topic": event.target_topic,
                    "claimed_status": event.status,
                    "idempotency_key": event.idempotency_key,
                    "proof_commit_policy": event.proof_commit_policy,
                }),
            )
            .await?;
            record_system_log(
                &tx,
                "info",
                event.request_id.as_deref(),
                event.trace_id.as_deref(),
                "outbox event published to kafka",
                &json!({
                    "worker_id": state.cfg.worker_id,
                    "outbox_event_id": event.outbox_event_id,
                    "event_type": event.event_type,
                    "target_topic": event.target_topic,
                    "attempt_no": attempt_no,
                    "status": "published",
                    "claimed_status": event.status,
                    "idempotency_key": event.idempotency_key,
                    "proof_commit_policy": event.proof_commit_policy,
                }),
            )
            .await?;
            state
                .metrics
                .publish_attempts
                .with_label_values(&["published"])
                .inc();
            PublishOutcome {
                outbox_event_id: event.outbox_event_id,
                request_id,
                trace_id,
                target_topic,
                result_code: "published",
            }
        }
        Err(err) => {
            let error_code = publish_error_code(&err);
            let next_retry_count = event.retry_count + 1;
            let exhausted = next_retry_count >= event.max_retries;
            let result_code = if exhausted { "dead_lettered" } else { "failed" };
            let mut metadata = json!({
                "worker_id": state.cfg.worker_id,
                "target_topic": event.target_topic,
                "attempt_no": attempt_no,
                "status": result_code,
            });
            let dead_letter_event_id = if exhausted {
                let dead_letter_event_id =
                    ensure_dead_letter_event(&tx, &event, &err, event.target_topic.as_deref())
                        .await?;
                match publish_dead_letter_message(
                    state,
                    &dead_letter_event_id,
                    &event,
                    &err,
                    attempt_no as u32,
                )
                .await
                {
                    Ok(()) => {
                        metadata["dead_letter_publish"] = Value::String("published".to_string());
                    }
                    Err(dlq_err) => {
                        warn!(
                            outbox_event_id = %event.outbox_event_id,
                            error = %dlq_err,
                            "outbox-publisher failed to publish kafka dead-letter message"
                        );
                        metadata["dead_letter_publish"] = Value::String("failed".to_string());
                        metadata["dead_letter_publish_error"] = Value::String(dlq_err);
                    }
                }
                metadata["dead_letter_event_id"] = Value::String(dead_letter_event_id.clone());
                Some(dead_letter_event_id)
            } else {
                None
            };
            insert_publish_attempt(
                &tx,
                &event,
                attempt_no,
                result_code,
                Some(error_code.as_str()),
                Some(err.as_str()),
                &metadata,
            )
            .await?;
            if exhausted {
                mark_outbox_dead_lettered(
                    &tx,
                    &event.outbox_event_id,
                    next_retry_count,
                    &error_code,
                    &err,
                )
                .await?;
                record_worker_audit_event(
                    &tx,
                    "outbox_event",
                    &event.outbox_event_id,
                    "outbox.publisher.dead_lettered",
                    "dead_lettered",
                    event.request_id.as_deref(),
                    event.trace_id.as_deref(),
                    &json!({
                        "worker_id": state.cfg.worker_id,
                        "event_type": event.event_type,
                        "target_topic": event.target_topic,
                        "dead_letter_event_id": dead_letter_event_id,
                        "error_code": error_code,
                        "claimed_status": event.status,
                        "idempotency_key": event.idempotency_key,
                        "proof_commit_policy": event.proof_commit_policy,
                    }),
                )
                .await?;
                record_system_log(
                    &tx,
                    "error",
                    event.request_id.as_deref(),
                    event.trace_id.as_deref(),
                    "outbox event dead-lettered after publish failure",
                    &json!({
                        "worker_id": state.cfg.worker_id,
                        "outbox_event_id": event.outbox_event_id,
                        "event_type": event.event_type,
                        "target_topic": event.target_topic,
                        "dead_letter_event_id": dead_letter_event_id,
                        "attempt_no": attempt_no,
                        "error_code": error_code,
                        "error_message": err,
                        "claimed_status": event.status,
                        "idempotency_key": event.idempotency_key,
                        "proof_commit_policy": event.proof_commit_policy,
                    }),
                )
                .await?;
            } else {
                let backoff_ms = retry_backoff_ms(state.cfg.retry_backoff_ms, next_retry_count);
                schedule_outbox_retry(
                    &tx,
                    &event.outbox_event_id,
                    next_retry_count,
                    &error_code,
                    &err,
                    backoff_ms,
                )
                .await?;
                record_system_log(
                    &tx,
                    "warn",
                    event.request_id.as_deref(),
                    event.trace_id.as_deref(),
                    "outbox publish failed and scheduled retry",
                    &json!({
                        "worker_id": state.cfg.worker_id,
                        "outbox_event_id": event.outbox_event_id,
                        "event_type": event.event_type,
                        "target_topic": event.target_topic,
                        "attempt_no": attempt_no,
                        "retry_count": next_retry_count,
                        "retry_backoff_ms": backoff_ms,
                        "error_code": error_code,
                        "error_message": err,
                        "claimed_status": event.status,
                        "idempotency_key": event.idempotency_key,
                        "proof_commit_policy": event.proof_commit_policy,
                    }),
                )
                .await?;
            }
            state
                .metrics
                .publish_attempts
                .with_label_values(&[result_code])
                .inc();
            PublishOutcome {
                outbox_event_id: event.outbox_event_id,
                request_id,
                trace_id,
                target_topic,
                result_code,
            }
        }
    };

    tx.commit()
        .await
        .map_err(|err| format!("commit outbox publisher transaction failed: {err}"))?;
    Ok(Some(outcome))
}

async fn claim_next_outbox_event(
    client: &(impl GenericClient + Sync),
    worker_id: &str,
    selected_outbox_event_id: Option<&str>,
) -> Result<Option<ClaimedOutboxEvent>, String> {
    let row = client
        .query_opt(
            "WITH next_event AS (
               SELECT outbox_event_id
                 FROM ops.outbox_event
                WHERE status = 'pending'
                  AND available_at <= now()
                  AND ($2::text IS NULL OR outbox_event_id = $2::text::uuid)
                ORDER BY available_at ASC, created_at ASC, outbox_event_id ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
             )
             UPDATE ops.outbox_event oe
                SET lock_owner = $1,
                    locked_at = now()
               FROM next_event
              WHERE oe.outbox_event_id = next_event.outbox_event_id
          RETURNING
               oe.outbox_event_id::text,
               oe.aggregate_type,
               oe.aggregate_id::text,
               oe.event_type,
               oe.payload,
               oe.status,
               oe.retry_count,
               oe.max_retries,
               oe.request_id,
               oe.trace_id,
               oe.idempotency_key,
               oe.authority_scope,
               oe.source_of_truth,
               oe.proof_commit_policy,
               oe.target_bus,
               oe.target_topic,
               oe.partition_key,
               oe.ordering_key",
            &[&worker_id, &selected_outbox_event_id],
        )
        .await
        .map_err(|err| format!("claim next outbox event failed: {err}"))?;
    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(ClaimedOutboxEvent {
        outbox_event_id: row.get(0),
        aggregate_type: row.get(1),
        aggregate_id: row.get(2),
        event_type: row.get(3),
        payload: row.get(4),
        status: row.get(5),
        retry_count: row.get(6),
        max_retries: row.get(7),
        request_id: row.get(8),
        trace_id: row.get(9),
        idempotency_key: row.get(10),
        authority_scope: row.get(11),
        source_of_truth: row.get(12),
        proof_commit_policy: row.get(13),
        target_bus: row.get(14),
        target_topic: row.get(15),
        partition_key: row.get(16),
        ordering_key: row.get(17),
    }))
}

async fn publish_main_event(
    state: &Arc<WorkerState>,
    event: &ClaimedOutboxEvent,
) -> Result<(), String> {
    if !event.target_bus.eq_ignore_ascii_case("kafka") {
        return Err(format!(
            "unsupported target_bus={} for outbox_event_id={}",
            event.target_bus, event.outbox_event_id
        ));
    }
    let Some(target_topic) = event.target_topic.as_deref() else {
        return Err(format!(
            "missing target_topic for outbox_event_id={}",
            event.outbox_event_id
        ));
    };
    let raw = serde_json::to_string(&event.payload)
        .map_err(|err| format!("encode outbox payload failed: {err}"))?;
    let key = event
        .partition_key
        .as_deref()
        .or(event.ordering_key.as_deref())
        .unwrap_or(event.outbox_event_id.as_str());
    state
        .producer
        .send(
            FutureRecord::to(target_topic).payload(&raw).key(key),
            Duration::from_millis(state.cfg.publish_timeout_ms),
        )
        .await
        .map_err(|(err, _)| format!("publish to kafka failed: {err}"))?;
    Ok(())
}

async fn insert_publish_attempt(
    client: &(impl GenericClient + Sync),
    event: &ClaimedOutboxEvent,
    attempt_no: i32,
    result_code: &str,
    error_code: Option<&str>,
    error_message: Option<&str>,
    metadata: &Value,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO ops.outbox_publish_attempt (
               outbox_event_id,
               worker_id,
               target_bus,
               target_topic,
               attempt_no,
               result_code,
               error_code,
               error_message,
               attempted_at,
               completed_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               $4,
               $5,
               $6,
               $7,
               $8,
               now(),
               now(),
               $9::jsonb
             )",
            &[
                &event.outbox_event_id,
                &metadata
                    .get("worker_id")
                    .and_then(Value::as_str)
                    .or(Some("outbox-publisher")),
                &event.target_bus,
                &event.target_topic,
                &attempt_no,
                &result_code,
                &error_code,
                &error_message,
                metadata,
            ],
        )
        .await
        .map_err(|err| format!("insert outbox publish attempt failed: {err}"))?;
    Ok(())
}

async fn mark_outbox_published(
    client: &(impl GenericClient + Sync),
    outbox_event_id: &str,
) -> Result<(), String> {
    client
        .execute(
            "UPDATE ops.outbox_event
                SET status = 'published',
                    published_at = now(),
                    lock_owner = NULL,
                    locked_at = NULL,
                    last_error_code = NULL,
                    last_error_message = NULL
              WHERE outbox_event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await
        .map_err(|err| format!("mark outbox published failed: {err}"))?;
    Ok(())
}

async fn schedule_outbox_retry(
    client: &(impl GenericClient + Sync),
    outbox_event_id: &str,
    retry_count: i32,
    error_code: &str,
    error_message: &str,
    backoff_ms: u64,
) -> Result<(), String> {
    let backoff_ms = backoff_ms as i64;
    client
        .execute(
            "UPDATE ops.outbox_event
                SET status = 'pending',
                    retry_count = $2,
                    available_at = now() + ($3::bigint * interval '1 millisecond'),
                    lock_owner = NULL,
                    locked_at = NULL,
                    last_error_code = $4,
                    last_error_message = $5
              WHERE outbox_event_id = $1::text::uuid",
            &[
                &outbox_event_id,
                &retry_count,
                &backoff_ms,
                &error_code,
                &error_message,
            ],
        )
        .await
        .map_err(|err| format!("schedule outbox retry failed: {err}"))?;
    Ok(())
}

async fn mark_outbox_dead_lettered(
    client: &(impl GenericClient + Sync),
    outbox_event_id: &str,
    retry_count: i32,
    error_code: &str,
    error_message: &str,
) -> Result<(), String> {
    client
        .execute(
            "UPDATE ops.outbox_event
                SET status = 'dead_lettered',
                    retry_count = $2,
                    dead_lettered_at = now(),
                    lock_owner = NULL,
                    locked_at = NULL,
                    last_error_code = $3,
                    last_error_message = $4
              WHERE outbox_event_id = $1::text::uuid",
            &[&outbox_event_id, &retry_count, &error_code, &error_message],
        )
        .await
        .map_err(|err| format!("mark outbox dead_lettered failed: {err}"))?;
    Ok(())
}

async fn ensure_dead_letter_event(
    client: &(impl GenericClient + Sync),
    event: &ClaimedOutboxEvent,
    error_message: &str,
    target_topic: Option<&str>,
) -> Result<String, String> {
    let row = client
        .query_one(
            "WITH existing AS (
               SELECT dead_letter_event_id
                 FROM ops.dead_letter_event
                WHERE outbox_event_id = $1::text::uuid
                  AND failure_stage = $2
                ORDER BY created_at DESC, dead_letter_event_id DESC
                LIMIT 1
             ),
             updated AS (
               UPDATE ops.dead_letter_event
                  SET failed_reason = $3,
                      last_failed_at = now(),
                      request_id = $4,
                      trace_id = $5,
                      authority_scope = $6,
                      source_of_truth = $7,
                      target_bus = $8,
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
                 first_failed_at,
                 last_failed_at,
                 reprocess_status
               )
               SELECT
                 $1::text::uuid,
                 $10,
                 $11::text::uuid,
                 $12,
                 $13::jsonb,
                 $3,
                 $4,
                 $5,
                 $6,
                 $7,
                 $8,
                 $9,
                 $2,
                 now(),
                 now(),
                 'not_reprocessed'
               WHERE NOT EXISTS (SELECT 1 FROM existing)
               RETURNING dead_letter_event_id::text
             )
             SELECT dead_letter_event_id FROM updated
             UNION ALL
             SELECT dead_letter_event_id FROM inserted
             LIMIT 1",
            &[
                &event.outbox_event_id,
                &FAILURE_STAGE_OUTBOX_PUBLISH,
                &error_message,
                &event.request_id,
                &event.trace_id,
                &event.authority_scope,
                &event.source_of_truth,
                &event.target_bus,
                &target_topic,
                &event.aggregate_type,
                &event.aggregate_id,
                &event.event_type,
                &event.payload,
            ],
        )
        .await
        .map_err(|err| format!("ensure dead letter event failed: {err}"))?;
    Ok(row.get(0))
}

async fn publish_dead_letter_message(
    state: &Arc<WorkerState>,
    dead_letter_event_id: &str,
    event: &ClaimedOutboxEvent,
    error_message: &str,
    attempt: u32,
) -> Result<(), String> {
    let payload = json!({
        "dead_letter_event_id": dead_letter_event_id,
        "source_topic": event.target_topic,
        "target_topic": state.cfg.dead_letter_topic,
        "event_id": event.outbox_event_id,
        "event_type": event.event_type,
        "aggregate_type": event.aggregate_type,
        "aggregate_id": event.aggregate_id,
        "request_id": event.request_id,
        "trace_id": event.trace_id,
        "failure_stage": FAILURE_STAGE_OUTBOX_PUBLISH,
        "failure_reason": error_message,
        "reprocess_status": "not_reprocessed",
        "attempt": attempt,
        "payload": event.payload,
    });
    let raw = serde_json::to_string(&payload)
        .map_err(|err| format!("encode outbox dead-letter payload failed: {err}"))?;
    state
        .producer
        .send(
            FutureRecord::to(&state.cfg.dead_letter_topic)
                .payload(&raw)
                .key(dead_letter_event_id),
            Duration::from_millis(state.cfg.publish_timeout_ms),
        )
        .await
        .map_err(|(err, _)| format!("publish kafka dead-letter failed: {err}"))?;
    Ok(())
}

async fn record_system_log(
    client: &(impl GenericClient + Sync),
    log_level: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    message_text: &str,
    structured_payload: &Value,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO ops.system_log (
               service_name,
               log_level,
               request_id,
               trace_id,
               message_text,
               structured_payload
             ) VALUES (
               'outbox-publisher',
               $1,
               $2,
               $3,
               $4,
               $5::jsonb
             )",
            &[
                &log_level,
                &request_id,
                &trace_id,
                &message_text,
                structured_payload,
            ],
        )
        .await
        .map_err(|err| format!("insert outbox publisher system log failed: {err}"))?;
    Ok(())
}

async fn record_worker_audit_event(
    client: &(impl GenericClient + Sync),
    ref_type: &str,
    ref_id: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    metadata: &Value,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO audit.audit_event (
               event_schema_version,
               event_class,
               domain_name,
               ref_type,
               ref_id,
               actor_type,
               action_name,
               result_code,
               request_id,
               trace_id,
               auth_assurance_level,
               ingested_at,
               event_time,
               metadata
             ) VALUES (
               'v1',
               'ops',
               'ops',
               $1,
               $2::text::uuid,
               'system',
               $3,
               $4,
               $5,
               $6,
               'service',
               now(),
               now(),
               $7::jsonb
             )",
            &[
                &ref_type,
                &ref_id,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                metadata,
            ],
        )
        .await
        .map_err(|err| format!("insert outbox publisher audit event failed: {err}"))?;
    Ok(())
}

async fn load_pending_event_count(db: &AppDb) -> Result<i64, String> {
    let client = db
        .client()
        .map_err(|err| format!("acquire db client for pending count failed: {err}"))?;
    let row = client
        .query_one(
            "SELECT count(*)::bigint
               FROM ops.outbox_event
              WHERE status = 'pending'
                AND available_at <= now()",
            &[],
        )
        .await
        .map_err(|err| format!("load pending outbox event count failed: {err}"))?;
    Ok(row.get::<_, i64>(0))
}

fn retry_backoff_ms(base_ms: u64, retry_count: i32) -> u64 {
    let exponent = retry_count.saturating_sub(1).clamp(0, 6) as u32;
    base_ms.saturating_mul(1u64 << exponent)
}

fn publish_error_code(err: &str) -> String {
    if err.contains("unsupported target_bus") {
        "UNSUPPORTED_TARGET_BUS".to_string()
    } else if err.contains("missing target_topic") {
        "MISSING_TARGET_TOPIC".to_string()
    } else if err.contains("UnknownTopic") || err.contains("UnknownTopicOrPartition") {
        "UNKNOWN_TARGET_TOPIC".to_string()
    } else if err.contains("Message production error") || err.contains("publish to kafka failed") {
        "KAFKA_PUBLISH_FAILED".to_string()
    } else {
        "OUTBOX_PUBLISH_FAILED".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::{Client, NoTls, connect};
    use rdkafka::consumer::{Consumer, StreamConsumer};
    use rdkafka::message::Message;
    use tokio::time::timeout;

    #[derive(Debug)]
    struct SeededOutboxEvent {
        outbox_event_id: String,
        request_id: String,
        target_topic: String,
    }

    fn live_db_enabled() -> bool {
        std::env::var("AUD_DB_SMOKE").ok().as_deref() == Some("1")
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

    async fn connect_db() -> (AppDb, Client) {
        let database_url = database_url();
        let app_db = AppDb::connect(
            DbPoolConfig {
                dsn: database_url.clone(),
                max_connections: 4,
            }
            .into(),
        )
        .await
        .expect("connect app db");
        let (client, connection) = connect(&database_url, NoTls).await.expect("connect raw db");
        tokio::spawn(async move {
            let _ = connection.await;
        });
        (app_db, client)
    }

    fn smoke_config(worker_id: &str) -> WorkerConfig {
        WorkerConfig {
            database_url: database_url(),
            kafka_brokers: kafka_brokers(),
            dead_letter_topic: "dtp.dead-letter".to_string(),
            app_host: "127.0.0.1".to_string(),
            app_port: 18098,
            batch_size: 10,
            poll_interval_ms: 100,
            publish_timeout_ms: 3000,
            retry_backoff_ms: 100,
            worker_id: worker_id.to_string(),
        }
    }

    async fn build_state(cfg: WorkerConfig) -> Arc<WorkerState> {
        let db = Arc::new(
            AppDb::connect(
                DbPoolConfig {
                    dsn: cfg.database_url.clone(),
                    max_connections: 4,
                }
                .into(),
            )
            .await
            .expect("connect app db"),
        );
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &cfg.kafka_brokers)
            .set("message.timeout.ms", cfg.publish_timeout_ms.to_string())
            .create()
            .expect("create kafka producer");
        Arc::new(WorkerState {
            cfg,
            db,
            producer,
            metrics: Arc::new(WorkerMetrics::new().expect("metrics")),
            ready: Arc::new(AtomicBool::new(true)),
        })
    }

    async fn seed_outbox_event(
        client: &Client,
        suffix: &str,
        target_topic: &str,
        max_retries: i32,
    ) -> SeededOutboxEvent {
        let ids = client
            .query_one(
                "SELECT gen_random_uuid()::text, gen_random_uuid()::text",
                &[],
            )
            .await
            .expect("generate ids");
        let aggregate_id: String = ids.get(0);
        let outbox_event_id: String = ids.get(1);
        let request_id = format!("req-aud009-{suffix}");
        let trace_id = format!("trace-aud009-{suffix}");
        let payload = json!({
            "event_id": outbox_event_id,
            "event_type": "catalog.product.changed",
            "event_version": 1,
            "occurred_at": "2026-04-22T00:00:00.000Z",
            "producer_service": "platform-core.catalog",
            "aggregate_type": "catalog.product",
            "aggregate_id": aggregate_id,
            "request_id": request_id,
            "trace_id": trace_id,
            "idempotency_key": format!("idem-aud009-{suffix}"),
            "event_schema_version": "v1",
            "authority_scope": "business",
            "source_of_truth": "database",
            "proof_commit_policy": "async_evidence",
            "payload": {
                "sku_type": "FILE_STD",
                "title": format!("aud009-{suffix}")
            }
        });
        client
            .execute(
                "INSERT INTO ops.outbox_event (
                   outbox_event_id,
                   aggregate_type,
                   aggregate_id,
                   event_type,
                   payload,
                   status,
                   retry_count,
                   available_at,
                   request_id,
                   trace_id,
                   idempotency_key,
                   event_schema_version,
                   authority_scope,
                   source_of_truth,
                   proof_commit_policy,
                   target_bus,
                   target_topic,
                   partition_key,
                   ordering_key,
                   payload_hash,
                   max_retries
                 ) VALUES (
                   $1::text::uuid,
                   'catalog.product',
                   $2::text::uuid,
                   'catalog.product.changed',
                   $3::jsonb,
                   'pending',
                   0,
                   now(),
                   $4,
                   $5,
                   $6,
                   'v1',
                   'business',
                   'database',
                   'async_evidence',
                   'kafka',
                   $7,
                   $2,
                   $2,
                   md5($3::text),
                   $8
                 )",
                &[
                    &outbox_event_id,
                    &aggregate_id,
                    &payload,
                    &request_id,
                    &trace_id,
                    &format!("idem-aud009-{suffix}"),
                    &target_topic,
                    &max_retries,
                ],
            )
            .await
            .expect("insert outbox event");
        SeededOutboxEvent {
            outbox_event_id,
            request_id,
            target_topic: target_topic.to_string(),
        }
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

    async fn wait_for_message(consumer: &StreamConsumer, expected_event_id: &str) -> Value {
        timeout(Duration::from_secs(10), async {
            loop {
                let message = consumer.recv().await.expect("recv kafka message");
                let Some(payload) = message.payload() else {
                    continue;
                };
                let decoded: Value = serde_json::from_slice(payload).expect("decode kafka message");
                if decoded["event_id"].as_str() == Some(expected_event_id)
                    || decoded["dead_letter_event_id"].as_str().is_some()
                        && decoded["event_id"].as_str() == Some(expected_event_id)
                {
                    return decoded;
                }
            }
        })
        .await
        .expect("wait for kafka message")
    }

    async fn cleanup_seed(client: &Client, seed: &SeededOutboxEvent) {
        let _ = client
            .execute(
                "DELETE FROM ops.dead_letter_event WHERE outbox_event_id = $1::text::uuid",
                &[&seed.outbox_event_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_publish_attempt WHERE outbox_event_id = $1::text::uuid",
                &[&seed.outbox_event_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event WHERE outbox_event_id = $1::text::uuid",
                &[&seed.outbox_event_id],
            )
            .await;
    }

    #[tokio::test]
    async fn outbox_publisher_db_smoke() {
        if !live_db_enabled() {
            return;
        }
        let (_app_db, client) = connect_db().await;
        let suffix = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_millis()
        );
        let success_seed = seed_outbox_event(
            &client,
            &format!("{suffix}-ok"),
            "dtp.outbox.domain-events",
            3,
        )
        .await;
        let failure_seed =
            seed_outbox_event(&client, &format!("{suffix}-fail"), "dtp.missing.topic", 1).await;

        let success_consumer = build_consumer(
            &format!("cg-aud009-success-{suffix}"),
            &success_seed.target_topic,
        );
        let dead_letter_consumer =
            build_consumer(&format!("cg-aud009-dlq-{suffix}"), "dtp.dead-letter");

        let state = build_state(smoke_config(&format!("outbox-publisher-smoke-{suffix}"))).await;

        let published = process_pending_event_inner(&state, Some(&success_seed.outbox_event_id))
            .await
            .expect("process success event")
            .expect("success outcome");
        assert_eq!(published.result_code, "published");
        let published_message =
            wait_for_message(&success_consumer, &success_seed.outbox_event_id).await;
        assert_eq!(published_message["event_id"], success_seed.outbox_event_id);
        assert_eq!(published_message["event_type"], "catalog.product.changed");

        let dead_lettered =
            process_pending_event_inner(&state, Some(&failure_seed.outbox_event_id))
                .await
                .expect("process failure event")
                .expect("failure outcome");
        assert_eq!(dead_lettered.result_code, "dead_lettered");
        let dead_letter_message =
            wait_for_message(&dead_letter_consumer, &failure_seed.outbox_event_id).await;
        assert_eq!(
            dead_letter_message["event_id"],
            failure_seed.outbox_event_id
        );
        assert_eq!(
            dead_letter_message["failure_stage"],
            FAILURE_STAGE_OUTBOX_PUBLISH
        );
        assert_eq!(dead_letter_message["target_topic"], "dtp.dead-letter");

        let success_row = client
            .query_one(
                "SELECT status, published_at IS NOT NULL
                   FROM ops.outbox_event
                  WHERE outbox_event_id = $1::text::uuid",
                &[&success_seed.outbox_event_id],
            )
            .await
            .expect("load published outbox row");
        assert_eq!(success_row.get::<_, String>(0), "published");
        assert!(success_row.get::<_, bool>(1));

        let success_attempt_row = client
            .query_one(
                "SELECT result_code, target_topic
                   FROM ops.outbox_publish_attempt
                  WHERE outbox_event_id = $1::text::uuid
                  ORDER BY attempt_no DESC, attempted_at DESC
                  LIMIT 1",
                &[&success_seed.outbox_event_id],
            )
            .await
            .expect("load success publish attempt");
        assert_eq!(success_attempt_row.get::<_, String>(0), "published");
        assert_eq!(
            success_attempt_row.get::<_, Option<String>>(1).as_deref(),
            Some("dtp.outbox.domain-events")
        );

        let failure_row = client
            .query_one(
                "SELECT status, retry_count, dead_lettered_at IS NOT NULL
                   FROM ops.outbox_event
                  WHERE outbox_event_id = $1::text::uuid",
                &[&failure_seed.outbox_event_id],
            )
            .await
            .expect("load dead-lettered outbox row");
        assert_eq!(failure_row.get::<_, String>(0), "dead_lettered");
        assert_eq!(failure_row.get::<_, i32>(1), 1);
        assert!(failure_row.get::<_, bool>(2));

        let dead_letter_row = client
            .query_one(
                "SELECT failure_stage, reprocess_status, target_topic
                   FROM ops.dead_letter_event
                  WHERE outbox_event_id = $1::text::uuid",
                &[&failure_seed.outbox_event_id],
            )
            .await
            .expect("load dead letter event");
        assert_eq!(
            dead_letter_row.get::<_, Option<String>>(0).as_deref(),
            Some(FAILURE_STAGE_OUTBOX_PUBLISH)
        );
        assert_eq!(dead_letter_row.get::<_, String>(1), "not_reprocessed");
        assert_eq!(
            dead_letter_row.get::<_, Option<String>>(2).as_deref(),
            Some("dtp.missing.topic")
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT count(*)::bigint
                   FROM audit.audit_event
                  WHERE request_id = ANY($1::text[])
                    AND action_name IN ('outbox.publisher.publish', 'outbox.publisher.dead_lettered')",
                &[&vec![success_seed.request_id.clone(), failure_seed.request_id.clone()]],
            )
            .await
            .expect("count audit events")
            .get(0);
        assert_eq!(audit_count, 2);

        let system_log_count: i64 = client
            .query_one(
                "SELECT count(*)::bigint
                   FROM ops.system_log
                  WHERE request_id = ANY($1::text[])
                    AND service_name = 'outbox-publisher'",
                &[&vec![
                    success_seed.request_id.clone(),
                    failure_seed.request_id.clone(),
                ]],
            )
            .await
            .expect("count system logs")
            .get(0);
        assert!(system_log_count >= 2);

        cleanup_seed(&client, &success_seed).await;
        cleanup_seed(&client, &failure_seed).await;
    }

    #[test]
    fn retry_backoff_exponential_and_capped() {
        assert_eq!(retry_backoff_ms(500, 1), 500);
        assert_eq!(retry_backoff_ms(500, 2), 1000);
        assert_eq!(retry_backoff_ms(500, 3), 2000);
        assert_eq!(retry_backoff_ms(500, 8), 32000);
    }

    #[test]
    fn publish_error_codes_are_stable() {
        assert_eq!(
            publish_error_code("unsupported target_bus=filesystem"),
            "UNSUPPORTED_TARGET_BUS"
        );
        assert_eq!(
            publish_error_code("missing target_topic for outbox_event_id=1"),
            "MISSING_TARGET_TOPIC"
        );
        assert_eq!(
            publish_error_code(
                "publish to kafka failed: Message production error: UnknownTopicOrPartition"
            ),
            "UNKNOWN_TARGET_TOPIC"
        );
        assert_eq!(
            publish_error_code("publish to kafka failed: Message production error"),
            "KAFKA_PUBLISH_FAILED"
        );
    }
}
