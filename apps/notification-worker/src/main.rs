use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use channel::{ChannelRegistry, ChannelSendRequest};
use db::{AppDb, DbPoolConfig, GenericClient};
use http::{ApiResponse, DependenciesReport, DependencyStatus, serve};
use kernel::{AppError, AppResult, ErrorResponse, new_uuid_string};
use prometheus::{Encoder, IntCounterVec, IntGauge, Registry, TextEncoder};
use rdkafka::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use redis::AsyncCommands;
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::time;
use tracing::{error, info, warn};

mod channel;
mod config;
mod event;
mod template;

use crate::config::NotificationWorkerConfig;
use crate::event::{
    NotificationEnvelope, RetryEnvelope, SendNotificationRequest, SendNotificationResponse,
};
use crate::template::{RenderedNotification, TemplateStore};

const SERVICE_NAME: &str = "notification-worker";

#[derive(Clone)]
struct WorkerState {
    cfg: NotificationWorkerConfig,
    db: Arc<AppDb>,
    redis_client: redis::Client,
    producer: FutureProducer,
    templates: TemplateStore,
    metrics: Arc<WorkerMetrics>,
    channels: Arc<ChannelRegistry>,
}

#[derive(Clone)]
struct WorkerMetrics {
    registry: Registry,
    event_results: IntCounterVec,
    send_results: IntCounterVec,
    retry_queue_depth: IntGauge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessSource {
    Kafka,
    RetryQueue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessOutcome {
    Processed,
    Duplicate,
    Retrying,
    DeadLettered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessingGate {
    Proceed,
    Duplicate,
    InFlight,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info,notification_worker=debug")
        .without_time()
        .try_init()
        .ok();

    let cfg = NotificationWorkerConfig::from_env()?;
    let db = Arc::new(
        AppDb::connect(
            DbPoolConfig {
                dsn: cfg.database_url.clone(),
                max_connections: 16,
            }
            .into(),
        )
        .await?,
    );
    let redis_client = redis::Client::open(cfg.redis_url.clone())
        .map_err(|err| AppError::Startup(format!("redis client init failed: {err}")))?;
    let producer = ClientConfig::new()
        .set("bootstrap.servers", &cfg.kafka_brokers)
        .set("message.timeout.ms", "5000")
        .create::<FutureProducer>()
        .map_err(|err| AppError::Startup(format!("kafka producer init failed: {err}")))?;
    let metrics = Arc::new(WorkerMetrics::new()?);
    let channels = Arc::new(ChannelRegistry::new(
        cfg.runtime.mode.clone(),
        cfg.runtime.provider.clone(),
    ));
    let state = Arc::new(WorkerState {
        cfg: cfg.clone(),
        db,
        redis_client,
        producer,
        templates: TemplateStore::new(cfg.template_dir.clone()),
        metrics,
        channels,
    });

    info!(
        topic = %cfg.topic,
        dead_letter_topic = %cfg.dead_letter_topic,
        group = %cfg.consumer_group,
        template_dir = %cfg.template_dir.display(),
        kafka_brokers = %cfg.kafka_brokers,
        active_channels = ?state.channels.active_channels(),
        reserved_channels = ?state.channels.reserved_channels(),
        "notification-worker started"
    );

    let consumer_state = state.clone();
    let retry_state = state.clone();
    let http_state = state.clone();

    let consumer_task = tokio::spawn(async move { run_consumer_loop(consumer_state).await });
    let retry_task = tokio::spawn(async move { run_retry_loop(retry_state).await });
    let http_task = tokio::spawn(async move { run_http_server(http_state).await });

    tokio::select! {
        consumer_result = consumer_task => {
            consumer_result.map_err(|err| AppError::Shutdown(format!("notification-worker consumer join failed: {err}")))??;
        }
        retry_result = retry_task => {
            retry_result.map_err(|err| AppError::Shutdown(format!("notification-worker retry join failed: {err}")))??;
        }
        http_result = http_task => {
            http_result.map_err(|err| AppError::Shutdown(format!("notification-worker http join failed: {err}")))??;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("notification-worker received shutdown signal");
        }
    }

    Ok(())
}

async fn run_http_server(state: Arc<WorkerState>) -> AppResult<()> {
    let ip = state
        .cfg
        .runtime
        .bind_host
        .parse::<IpAddr>()
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let addr = SocketAddr::from((ip, state.cfg.runtime.bind_port));
    let app = Router::new()
        .route("/health/live", get(live_handler))
        .route("/health/ready", get(ready_handler))
        .route("/health/deps", get(deps_handler))
        .route(
            "/internal/notifications/templates/preview",
            post(preview_template_handler),
        )
        .route(
            "/internal/notifications/send",
            post(send_notification_handler),
        )
        .route(
            "/internal/notifications/dead-letters/{dead_letter_event_id}/replay",
            post(replay_dead_letter_handler),
        )
        .route(
            "/internal/notifications/audit/search",
            post(notification_audit_search_handler),
        )
        .route("/metrics", get(metrics_handler))
        .with_state(state);
    serve(addr, app, tokio::signal::ctrl_c()).await
}

async fn run_consumer_loop(state: Arc<WorkerState>) -> AppResult<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &state.cfg.kafka_brokers)
        .set("group.id", &state.cfg.consumer_group)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .map_err(|err| AppError::Startup(format!("kafka consumer init failed: {err}")))?;
    consumer
        .subscribe(&[&state.cfg.topic])
        .map_err(|err| AppError::Startup(format!("kafka subscribe failed: {err}")))?;

    loop {
        match consumer.recv().await {
            Ok(message) => {
                let should_commit = match handle_kafka_message(&state, &message).await {
                    Ok(should_commit) => should_commit,
                    Err(err) => {
                        error!(error = %err, "notification-worker kafka handling failed");
                        false
                    }
                };
                if !should_commit {
                    warn!(
                        "notification-worker skipped offset commit until message is safely isolated"
                    );
                    continue;
                }
                if let Err(err) = consumer.commit_message(&message, CommitMode::Async) {
                    warn!(error = %err, "notification-worker commit offset failed");
                }
            }
            Err(err) => warn!(error = %err, "notification-worker kafka receive failed"),
        }
    }
}

async fn run_retry_loop(state: Arc<WorkerState>) -> AppResult<()> {
    let mut interval = time::interval(Duration::from_millis(state.cfg.retry_poll_interval_ms));
    loop {
        interval.tick().await;
        let due = load_due_retry_jobs(&state).await?;
        if due.is_empty() {
            continue;
        }
        for retry in due {
            if let Err(err) =
                process_retry_envelope(&state, retry.clone(), ProcessSource::RetryQueue).await
            {
                if let Err(restore_err) =
                    restore_retry_job(&state, &retry.envelope.event_id, state.cfg.retry_backoff_ms)
                        .await
                {
                    error!(
                        error = %restore_err,
                        event_id = %retry.envelope.event_id,
                        "notification-worker retry restoration failed"
                    );
                }
                error!(error = %err, "notification-worker retry handling failed");
            }
        }
    }
}

async fn handle_kafka_message(
    state: &Arc<WorkerState>,
    message: &rdkafka::message::BorrowedMessage<'_>,
) -> Result<bool, String> {
    let Some(payload) = message.payload() else {
        return Ok(true);
    };
    let envelope: NotificationEnvelope = serde_json::from_slice(payload)
        .map_err(|err| format!("decode notification payload failed: {err}"))?;
    let retry = RetryEnvelope {
        attempt: 1,
        envelope,
    };
    let _ = process_retry_envelope(state, retry, ProcessSource::Kafka).await?;
    Ok(true)
}

async fn process_retry_envelope(
    state: &Arc<WorkerState>,
    retry: RetryEnvelope,
    source: ProcessSource,
) -> Result<ProcessOutcome, String> {
    if retry.envelope.event_type != "notification.requested" {
        info!(
            event_id = %retry.envelope.event_id,
            event_type = %retry.envelope.event_type,
            "notification-worker ignored non-notification event"
        );
        return Ok(ProcessOutcome::Duplicate);
    }

    let client = state
        .db
        .client()
        .map_err(|err| format!("acquire notification worker db client failed: {err}"))?;
    match begin_processing_gate(&client, &retry, source).await? {
        ProcessingGate::Duplicate | ProcessingGate::InFlight => {
            state
                .metrics
                .event_results
                .with_label_values(&["duplicate"])
                .inc();
            return Ok(ProcessOutcome::Duplicate);
        }
        ProcessingGate::Proceed => {
            set_short_state(
                state,
                &retry.envelope.event_id,
                "processing",
                retry.attempt,
                Some(json!({
                    "source": process_source_name(source),
                })),
            )
            .await?;
        }
    }

    let rendered = match state
        .templates
        .render(&client, &retry.envelope, &retry.envelope.payload)
        .await
    {
        Ok(rendered) => rendered,
        Err(err) => {
            let rendered = RenderedNotification::placeholder(&retry.envelope.payload);
            return handle_failed_attempt(
                state,
                &client,
                retry,
                rendered,
                format!("render notification template failed: {err}"),
            )
            .await;
        }
    };
    match state
        .channels
        .send(ChannelSendRequest {
            retry: &retry,
            rendered: &rendered,
        })
        .await
    {
        Ok(channel_result) => {
            finalize_processing(state, &client, &retry, &rendered, channel_result).await?;
            state
                .metrics
                .event_results
                .with_label_values(&["processed"])
                .inc();
            state
                .metrics
                .send_results
                .with_label_values(&[rendered.channel.as_str(), "success"])
                .inc();
            Ok(ProcessOutcome::Processed)
        }
        Err(err) => {
            state
                .metrics
                .send_results
                .with_label_values(&[retry.envelope.payload.channel.as_str(), "failed"])
                .inc();
            handle_failed_attempt(state, &client, retry, rendered, err).await
        }
    }
}

async fn finalize_processing(
    state: &Arc<WorkerState>,
    client: &(impl GenericClient + Sync),
    retry: &RetryEnvelope,
    rendered: &RenderedNotification,
    channel_result: channel::ChannelSendResult,
) -> Result<(), String> {
    update_processing_result(
        client,
        &retry.envelope.event_id,
        "processed",
        None,
        retry.attempt,
    )
    .await?;
    if let Some(dead_letter_event_id) = replayed_from_dead_letter_id(&retry.envelope) {
        update_dead_letter_reprocess_status(client, dead_letter_event_id, "reprocessed").await?;
    }
    write_system_log(
        client,
        &retry.envelope,
        "info",
        "notification sent via mock-log",
        merge_lookup_metadata(
            &retry.envelope,
            json!({
                "template_code": rendered.template_code,
                "channel": rendered.channel,
                "title": rendered.title,
                "body": rendered.body,
                "attempt": retry.attempt,
                "result": channel_result.as_json(),
            }),
        ),
    )
    .await?;
    write_trace_index(
        client,
        &retry.envelope,
        "notification.dispatch",
        merge_lookup_metadata(
            &retry.envelope,
            json!({
                "template_code": rendered.template_code,
                "channel": rendered.channel,
                "attempt": retry.attempt,
                "status": "processed",
            }),
        ),
    )
    .await?;
    write_audit_event(
        client,
        &retry.envelope,
        "notification.dispatch.sent",
        "success",
        merge_lookup_metadata(
            &retry.envelope,
            json!({
                "template_code": rendered.template_code,
                "channel": rendered.channel,
                "attempt": retry.attempt,
            }),
        ),
    )
    .await?;
    clear_retry_job(state, &retry.envelope.event_id).await?;
    set_short_state(
        state,
        &retry.envelope.event_id,
        "processed",
        retry.attempt,
        Some(json!({
            "template_code": rendered.template_code,
            "channel": rendered.channel,
        })),
    )
    .await?;
    info!(
        event_id = %retry.envelope.event_id,
        template_code = %rendered.template_code,
        channel = %rendered.channel,
        attempt = retry.attempt,
        "notification-worker processed notification"
    );
    Ok(())
}

async fn handle_failed_attempt(
    state: &Arc<WorkerState>,
    client: &(impl GenericClient + Sync),
    retry: RetryEnvelope,
    rendered: RenderedNotification,
    error_message: String,
) -> Result<ProcessOutcome, String> {
    let retry_policy = retry_policy(state, &retry.envelope);
    let max_attempts = retry_policy.max_attempts.max(1);
    if retry.attempt < max_attempts {
        schedule_retry(state, &retry, &error_message, retry_policy.backoff_ms).await?;
        update_processing_result(
            client,
            &retry.envelope.event_id,
            "retrying",
            Some(&error_message),
            retry.attempt,
        )
        .await?;
        write_system_log(
            client,
            &retry.envelope,
            "warn",
            "notification send failed and was queued for retry",
            merge_lookup_metadata(
                &retry.envelope,
                json!({
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "attempt": retry.attempt,
                    "next_attempt": retry.attempt + 1,
                    "error": error_message,
                }),
            ),
        )
        .await?;
        write_trace_index(
            client,
            &retry.envelope,
            "notification.retrying",
            merge_lookup_metadata(
                &retry.envelope,
                json!({
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "attempt": retry.attempt,
                    "status": "retrying",
                }),
            ),
        )
        .await?;
        write_audit_event(
            client,
            &retry.envelope,
            "notification.dispatch.retry_scheduled",
            "failed",
            merge_lookup_metadata(
                &retry.envelope,
                json!({
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "attempt": retry.attempt,
                    "error": error_message,
                }),
            ),
        )
        .await?;
        set_short_state(
            state,
            &retry.envelope.event_id,
            "retrying",
            retry.attempt,
            Some(json!({
                "error": error_message,
                "template_code": rendered.template_code,
            })),
        )
        .await?;
        state
            .metrics
            .event_results
            .with_label_values(&["retrying"])
            .inc();
        Ok(ProcessOutcome::Retrying)
    } else {
        let dead_letter_event_id = ensure_dead_letter_event(
            client,
            &retry.envelope,
            &error_message,
            &state.cfg.dead_letter_topic,
        )
        .await?;
        insert_alert_event(
            client,
            &retry.envelope,
            &error_message,
            &dead_letter_event_id,
            &state.cfg.dead_letter_topic,
        )
        .await?;
        publish_dead_letter_message(
            state,
            &dead_letter_event_id,
            &retry.envelope,
            &error_message,
            retry.attempt,
        )
        .await?;
        update_processing_result(
            client,
            &retry.envelope.event_id,
            "dead_lettered",
            Some(&error_message),
            retry.attempt,
        )
        .await?;
        if let Some(replayed_dead_letter_id) = replayed_from_dead_letter_id(&retry.envelope) {
            update_dead_letter_reprocess_status(
                client,
                replayed_dead_letter_id,
                "reprocess_failed",
            )
            .await?;
        }
        write_system_log(
            client,
            &retry.envelope,
            "error",
            "notification send exhausted retries and moved to dead letter",
            merge_lookup_metadata(
                &retry.envelope,
                json!({
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "attempt": retry.attempt,
                    "error": error_message,
                    "dead_letter_event_id": dead_letter_event_id,
                    "dead_letter_topic": state.cfg.dead_letter_topic.as_str(),
                }),
            ),
        )
        .await?;
        write_trace_index(
            client,
            &retry.envelope,
            "notification.dead_lettered",
            merge_lookup_metadata(
                &retry.envelope,
                json!({
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "attempt": retry.attempt,
                    "status": "dead_lettered",
                    "dead_letter_event_id": dead_letter_event_id,
                }),
            ),
        )
        .await?;
        write_audit_event(
            client,
            &retry.envelope,
            "notification.dispatch.dead_lettered",
            "failed",
            merge_lookup_metadata(
                &retry.envelope,
                json!({
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "attempt": retry.attempt,
                    "error": error_message,
                    "dead_letter_event_id": dead_letter_event_id,
                    "dead_letter_topic": state.cfg.dead_letter_topic.as_str(),
                }),
            ),
        )
        .await?;
        clear_retry_job(state, &retry.envelope.event_id).await?;
        set_short_state(
            state,
            &retry.envelope.event_id,
            "dead_lettered",
            retry.attempt,
            Some(json!({
                "error": error_message,
                "template_code": rendered.template_code,
            })),
        )
        .await?;
        state
            .metrics
            .event_results
            .with_label_values(&["dead_lettered"])
            .inc();
        Ok(ProcessOutcome::DeadLettered)
    }
}

async fn begin_processing_gate(
    client: &(impl GenericClient + Sync),
    retry: &RetryEnvelope,
    source: ProcessSource,
) -> Result<ProcessingGate, String> {
    let row = client
        .query_opt(
            "SELECT result_code
             FROM ops.consumer_idempotency_record
             WHERE consumer_name = $1
               AND event_id = $2::text::uuid",
            &[&SERVICE_NAME, &retry.envelope.event_id],
        )
        .await
        .map_err(|err| format!("load consumer idempotency record failed: {err}"))?;

    if let Some(row) = row {
        let result_code: String = row.get(0);
        match result_code.as_str() {
            "processed" | "dead_lettered" => Ok(ProcessingGate::Duplicate),
            "processing" if source == ProcessSource::RetryQueue && retry.attempt > 1 => {
                update_processing_result(
                    client,
                    &retry.envelope.event_id,
                    "processing",
                    None,
                    retry.attempt,
                )
                .await?;
                Ok(ProcessingGate::Proceed)
            }
            "retrying" if source == ProcessSource::RetryQueue => {
                update_processing_result(
                    client,
                    &retry.envelope.event_id,
                    "processing",
                    None,
                    retry.attempt,
                )
                .await?;
                Ok(ProcessingGate::Proceed)
            }
            "processing" | "retrying" => Ok(ProcessingGate::InFlight),
            _ if source == ProcessSource::RetryQueue => {
                update_processing_result(
                    client,
                    &retry.envelope.event_id,
                    "processing",
                    None,
                    retry.attempt,
                )
                .await?;
                Ok(ProcessingGate::Proceed)
            }
            _ => Ok(ProcessingGate::InFlight),
        }
    } else {
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
                   jsonb_build_object('attempt', $6::int, 'source', $7)
                 )",
                &[
                    &SERVICE_NAME,
                    &retry.envelope.event_id,
                    &retry.envelope.aggregate_type,
                    &retry.envelope.aggregate_id,
                    &retry.envelope.effective_trace_id(),
                    &(retry.attempt as i32),
                    &process_source_name(source),
                ],
            )
            .await
            .map_err(|err| format!("insert consumer idempotency record failed: {err}"))?;
        Ok(ProcessingGate::Proceed)
    }
}

async fn update_processing_result(
    client: &(impl GenericClient + Sync),
    event_id: &str,
    result_code: &str,
    error_message: Option<&str>,
    attempt: u32,
) -> Result<(), String> {
    client
        .execute(
            "UPDATE ops.consumer_idempotency_record
             SET result_code = $3,
                 processed_at = now(),
                 metadata = coalesce(metadata, '{}'::jsonb)
                   || jsonb_build_object(
                        'attempt', $4::int,
                        'last_error', $5,
                        'updated_at', now()
                      )
             WHERE consumer_name = $1
               AND event_id = $2::text::uuid",
            &[
                &SERVICE_NAME,
                &event_id,
                &result_code,
                &(attempt as i32),
                &error_message,
            ],
        )
        .await
        .map_err(|err| format!("update consumer idempotency result failed: {err}"))?;
    Ok(())
}

async fn schedule_retry(
    state: &Arc<WorkerState>,
    retry: &RetryEnvelope,
    error_message: &str,
    backoff_ms: u64,
) -> Result<(), String> {
    let next = RetryEnvelope {
        attempt: retry.attempt + 1,
        envelope: retry.envelope.clone(),
    };
    let raw = serde_json::to_string(&next)
        .map_err(|err| format!("encode retry envelope failed: {err}"))?;
    let mut conn = redis_connection(state).await?;
    let payload_key = retry_payload_key(state, &retry.envelope.event_id);
    let queue_key = retry_queue_key(state);
    let due_at = now_utc_ms() + i64::try_from(backoff_ms).unwrap_or(0);
    conn.set_ex::<_, _, ()>(payload_key, raw, 86_400)
        .await
        .map_err(|err| format!("persist retry payload failed: {err}"))?;
    redis::cmd("ZADD")
        .arg(queue_key)
        .arg(due_at)
        .arg(&retry.envelope.event_id)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|err| format!("enqueue retry job failed: {err}"))?;
    update_retry_queue_depth(state, &mut conn).await?;
    warn!(
        event_id = %retry.envelope.event_id,
        error = %error_message,
        current_attempt = retry.attempt,
        next_attempt = next.attempt,
        "notification-worker scheduled retry"
    );
    Ok(())
}

async fn load_due_retry_jobs(state: &Arc<WorkerState>) -> Result<Vec<RetryEnvelope>, AppError> {
    let mut conn = redis_connection(state).await.map_err(AppError::Startup)?;
    let queue_key = retry_queue_key(state);
    let event_ids: Vec<String> = redis::cmd("ZRANGEBYSCORE")
        .arg(&queue_key)
        .arg("-inf")
        .arg(now_utc_ms())
        .arg("LIMIT")
        .arg(0)
        .arg(20)
        .query_async(&mut conn)
        .await
        .map_err(|err| AppError::Startup(format!("load retry queue failed: {err}")))?;

    let mut retries = Vec::with_capacity(event_ids.len());
    for event_id in event_ids {
        let payload_key = retry_payload_key(state, &event_id);
        let raw: Option<String> = conn
            .get(&payload_key)
            .await
            .map_err(|err| AppError::Startup(format!("load retry payload failed: {err}")))?;
        redis::cmd("ZREM")
            .arg(&queue_key)
            .arg(&event_id)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|err| AppError::Startup(format!("dequeue retry job failed: {err}")))?;
        if let Some(raw) = raw {
            match serde_json::from_str::<RetryEnvelope>(&raw) {
                Ok(retry) => retries.push(retry),
                Err(err) => {
                    warn!(event_id = %event_id, error = %err, "notification-worker discarded invalid retry payload")
                }
            }
        }
    }
    update_retry_queue_depth(state, &mut conn)
        .await
        .map_err(AppError::Startup)?;
    Ok(retries)
}

async fn restore_retry_job(
    state: &Arc<WorkerState>,
    event_id: &str,
    backoff_ms: u64,
) -> Result<(), String> {
    let mut conn = redis_connection(state).await?;
    redis::cmd("ZADD")
        .arg(retry_queue_key(state))
        .arg(now_utc_ms() + i64::try_from(backoff_ms).unwrap_or(0))
        .arg(event_id)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|err| format!("restore retry queue entry failed: {err}"))?;
    update_retry_queue_depth(state, &mut conn).await?;
    Ok(())
}

async fn clear_retry_job(state: &Arc<WorkerState>, event_id: &str) -> Result<(), String> {
    let mut conn = redis_connection(state).await?;
    let queue_key = retry_queue_key(state);
    let payload_key = retry_payload_key(state, event_id);
    redis::cmd("ZREM")
        .arg(&queue_key)
        .arg(event_id)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|err| format!("remove retry queue entry failed: {err}"))?;
    let _: () = conn
        .del(payload_key)
        .await
        .map_err(|err| format!("remove retry payload failed: {err}"))?;
    update_retry_queue_depth(state, &mut conn).await?;
    Ok(())
}

async fn set_short_state(
    state: &Arc<WorkerState>,
    event_id: &str,
    status: &str,
    attempt: u32,
    extra: Option<Value>,
) -> Result<(), String> {
    let payload = build_short_state_payload(status, attempt, extra);
    let mut conn = redis_connection(state).await?;
    conn.set_ex::<_, _, ()>(
        short_state_key(state, event_id),
        payload.to_string(),
        86_400,
    )
    .await
    .map_err(|err| format!("persist notification short state failed: {err}"))?;
    Ok(())
}

fn build_short_state_payload(status: &str, attempt: u32, extra: Option<Value>) -> Value {
    let mut payload = json!({
        "status": status,
        "attempt": attempt,
        "updated_at": now_iso8601(),
    });
    if let Some(extra) = extra {
        if let Some(object) = payload.as_object_mut() {
            object.insert("details".to_string(), extra);
        }
    }
    payload
}

async fn ensure_dead_letter_event(
    client: &(impl GenericClient + Sync),
    envelope: &NotificationEnvelope,
    error_message: &str,
    target_topic: &str,
) -> Result<String, String> {
    let payload = serde_json::to_string(envelope)
        .map_err(|err| format!("encode dead letter payload failed: {err}"))?;
    let row = client
        .query_one(
            "WITH existing AS (
               SELECT dead_letter_event_id
                 FROM ops.dead_letter_event
                WHERE outbox_event_id = $1::text::uuid
                  AND failure_stage = 'notification.send'
                ORDER BY created_at DESC
                LIMIT 1
             ),
             updated AS (
               UPDATE ops.dead_letter_event
                  SET failed_reason = $6,
                      last_failed_at = now(),
                      target_topic = $9
                WHERE dead_letter_event_id IN (SELECT dead_letter_event_id FROM existing)
                RETURNING dead_letter_event_id
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
                  $9,
                  'notification.send',
                  now()
               WHERE NOT EXISTS (SELECT 1 FROM updated)
               RETURNING dead_letter_event_id
             )
             SELECT dead_letter_event_id::text FROM updated
             UNION ALL
             SELECT dead_letter_event_id::text FROM inserted
             LIMIT 1",
            &[
                &envelope.event_id,
                &envelope.aggregate_type,
                &envelope.aggregate_id,
                &envelope.event_type,
                &payload,
                &error_message,
                &envelope.request_id,
                &envelope.effective_trace_id(),
                &target_topic,
            ],
        )
        .await
        .map_err(|err| format!("insert dead letter event failed: {err}"))?;
    Ok(row.get::<_, String>(0))
}

async fn insert_alert_event(
    client: &(impl GenericClient + Sync),
    envelope: &NotificationEnvelope,
    error_message: &str,
    dead_letter_event_id: &str,
    target_topic: &str,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO ops.alert_event (
               fingerprint,
               alert_type,
               severity,
               title_text,
               summary_text,
               ref_type,
               ref_id,
               request_id,
               trace_id,
               labels_json,
               annotations_json,
               metadata
             )
             SELECT
               $1,
               'notification_dead_letter',
               'high',
               'Notification moved to dead letter',
               $2,
               $3,
               $4::text::uuid,
               $5,
               $6,
               jsonb_build_object('service', $7, 'topic', $8),
               jsonb_build_object('template_code', $9),
               jsonb_build_object('event_id', $10, 'dead_letter_event_id', $11)
             WHERE NOT EXISTS (
               SELECT 1
                 FROM ops.alert_event
                WHERE fingerprint = $1
                  AND alert_type = 'notification_dead_letter'
             )",
            &[
                &format!("notification-worker:{}", envelope.event_id),
                &error_message,
                &envelope.aggregate_type,
                &envelope.aggregate_id,
                &envelope.request_id,
                &envelope.effective_trace_id(),
                &SERVICE_NAME,
                &target_topic,
                &envelope.payload.template_code,
                &envelope.event_id,
                &dead_letter_event_id,
            ],
        )
        .await
        .map_err(|err| format!("insert alert event failed: {err}"))?;
    Ok(())
}

async fn write_system_log(
    client: &(impl GenericClient + Sync),
    envelope: &NotificationEnvelope,
    log_level: &str,
    message_text: &str,
    structured_payload: Value,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO ops.system_log (
               service_name,
               log_level,
               request_id,
               trace_id,
               message_text,
               structured_payload,
               logger_name,
               environment_code,
               backend_type,
               severity_number,
               object_type,
               object_id
             ) VALUES (
               $1,
               $2,
               $3,
               $4,
               $5,
               $6::jsonb,
               $7,
               $8,
               'database_mirror',
               $9,
               'notification_dispatch',
               $10::text::uuid
             )",
            &[
                &SERVICE_NAME,
                &log_level,
                &envelope.request_id,
                &envelope.effective_trace_id(),
                &message_text,
                &structured_payload.to_string(),
                &SERVICE_NAME,
                &state_env_code(),
                &severity_number(log_level),
                &envelope.event_id,
            ],
        )
        .await
        .map_err(|err| format!("insert system log failed: {err}"))?;
    Ok(())
}

async fn write_trace_index(
    client: &(impl GenericClient + Sync),
    envelope: &NotificationEnvelope,
    root_span_name: &str,
    metadata: Value,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO ops.trace_index (
               trace_id,
               root_service_name,
               root_span_name,
               request_id,
               ref_type,
               ref_id,
               object_type,
               object_id,
               status,
               span_count,
               started_at,
               ended_at,
               metadata
             ) VALUES (
               $1,
               $2,
               $3,
               $4,
               $5,
               $6::text::uuid,
               'notification_dispatch',
               $7::text::uuid,
               'ok',
               1,
               now(),
               now(),
               $8::jsonb
             )",
            &[
                &envelope.effective_trace_id(),
                &SERVICE_NAME,
                &root_span_name,
                &envelope.request_id,
                &envelope.aggregate_type,
                &envelope.aggregate_id,
                &envelope.event_id,
                &metadata.to_string(),
            ],
        )
        .await
        .map_err(|err| format!("insert trace index failed: {err}"))?;
    Ok(())
}

async fn write_audit_event(
    client: &(impl GenericClient + Sync),
    envelope: &NotificationEnvelope,
    action_name: &str,
    result_code: &str,
    metadata: Value,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO audit.audit_event (
               domain_name,
               ref_type,
               ref_id,
               actor_type,
               action_name,
               result_code,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               'notification',
               $1,
               $2::text::uuid,
               'service',
               $3,
               $4,
               $5,
               $6,
               $7::jsonb
             )",
            &[
                &envelope.aggregate_type,
                &envelope.aggregate_id,
                &action_name,
                &result_code,
                &envelope.request_id,
                &envelope.effective_trace_id(),
                &metadata.to_string(),
            ],
        )
        .await
        .map_err(|err| format!("insert notification audit event failed: {err}"))?;
    Ok(())
}

async fn write_notification_lookup_audit_event(
    client: &(impl GenericClient + Sync),
    request: &NotificationAuditLookupRequest,
    request_id: &str,
    trace_id: &str,
    result_count: usize,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO audit.audit_event (
               domain_name,
               ref_type,
               actor_type,
               action_name,
               result_code,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               'notification',
               'notification_lookup',
               'service',
               'notification.dispatch.lookup',
               'success',
               $1,
               $2,
               $3::jsonb
             )",
            &[
                &request_id,
                &trace_id,
                &json!({
                    "order_id": request.order_id,
                    "case_id": request.case_id,
                    "template_code": request.template_code,
                    "notification_code": request.notification_code,
                    "event_id": request.event_id,
                    "limit": lookup_limit(request.limit),
                    "reason": request.reason,
                    "step_up_ticket": request.step_up_ticket,
                    "result_count": result_count,
                })
                .to_string(),
            ],
        )
        .await
        .map_err(|err| format!("insert notification lookup audit event failed: {err}"))?;
    Ok(())
}

async fn load_notification_audit_event_ids(
    client: &(impl GenericClient + Sync),
    request: &NotificationAuditLookupRequest,
    limit: i64,
) -> Result<Vec<String>, String> {
    let rows = client
        .query(
            "SELECT object_id::text
               FROM ops.system_log
              WHERE service_name = $1
                AND object_type = 'notification_dispatch'
                AND ($2::text IS NULL OR object_id = $2::text::uuid)
                AND (
                      $3::text IS NULL
                      OR structured_payload->>'template_code' = $3
                      OR EXISTS (
                           SELECT 1
                             FROM ops.outbox_event outbox_event
                            WHERE outbox_event.payload->>'event_id' = ops.system_log.object_id::text
                              AND outbox_event.payload->'payload'->>'template_code' = $3
                      )
                    )
                AND (
                      $4::text IS NULL
                      OR structured_payload->>'notification_code' = $4
                      OR EXISTS (
                           SELECT 1
                             FROM ops.outbox_event outbox_event
                            WHERE outbox_event.payload->>'event_id' = ops.system_log.object_id::text
                              AND outbox_event.payload->'payload'->>'notification_code' = $4
                      )
                    )
                AND (
                      $5::text IS NULL
                      OR EXISTS (
                           SELECT 1
                             FROM jsonb_array_elements(coalesce(structured_payload->'subject_refs', '[]'::jsonb)) subject_ref
                            WHERE subject_ref->>'ref_type' = 'order'
                              AND subject_ref->>'ref_id' = $5
                      )
                      OR EXISTS (
                           SELECT 1
                             FROM ops.outbox_event outbox_event,
                                  jsonb_array_elements(coalesce(outbox_event.payload->'payload'->'subject_refs', '[]'::jsonb)) subject_ref
                            WHERE outbox_event.payload->>'event_id' = ops.system_log.object_id::text
                              AND subject_ref->>'ref_type' = 'order'
                              AND subject_ref->>'ref_id' = $5
                      )
                    )
                AND (
                      $6::text IS NULL
                      OR EXISTS (
                           SELECT 1
                             FROM jsonb_array_elements(coalesce(structured_payload->'subject_refs', '[]'::jsonb)) subject_ref
                            WHERE subject_ref->>'ref_type' = 'dispute_case'
                              AND subject_ref->>'ref_id' = $6
                      )
                      OR EXISTS (
                           SELECT 1
                             FROM ops.outbox_event outbox_event,
                                  jsonb_array_elements(coalesce(outbox_event.payload->'payload'->'subject_refs', '[]'::jsonb)) subject_ref
                            WHERE outbox_event.payload->>'event_id' = ops.system_log.object_id::text
                              AND subject_ref->>'ref_type' = 'dispute_case'
                              AND subject_ref->>'ref_id' = $6
                      )
                    )
              GROUP BY object_id
              ORDER BY MAX(created_at) DESC
              LIMIT $7",
            &[
                &SERVICE_NAME,
                &request.event_id,
                &request.template_code,
                &request.notification_code,
                &request.order_id,
                &request.case_id,
                &limit,
            ],
        )
        .await
        .map_err(|err| format!("load notification audit event ids failed: {err}"))?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<_, String>(0))
        .collect())
}

async fn load_notification_audit_record(
    client: &(impl GenericClient + Sync),
    event_id: &str,
) -> Result<NotificationAuditLookupRecord, String> {
    let log_rows = client
        .query(
            "SELECT request_id,
                    trace_id,
                    message_text,
                    created_at::text,
                    structured_payload
               FROM ops.system_log
              WHERE service_name = $1
                AND object_type = 'notification_dispatch'
                AND object_id = $2::text::uuid
              ORDER BY created_at ASC, system_log_id ASC",
            &[&SERVICE_NAME, &event_id],
        )
        .await
        .map_err(|err| format!("load notification system logs failed: {err}"))?;
    if log_rows.is_empty() {
        return Err(format!(
            "notification audit record not found for event {event_id}"
        ));
    }

    let mut base_payload = log_rows
        .iter()
        .map(|row| row.get::<_, Value>(4))
        .find(|payload| payload["notification_code"].is_string())
        .unwrap_or_else(|| json!({}));
    if !base_payload["notification_code"].is_string() {
        if let Some(payload) = load_notification_lookup_payload(client, event_id).await? {
            base_payload = payload;
        }
    }
    let request_id = log_rows
        .last()
        .map(|row| row.get::<_, String>(0))
        .unwrap_or_default();
    let trace_id = log_rows
        .last()
        .map(|row| row.get::<_, String>(1))
        .unwrap_or_default();

    let timeline = log_rows
        .iter()
        .map(|row| {
            let message_text: String = row.get(2);
            let payload: Value = row.get(4);
            NotificationDispatchTimelineEntry {
                status: timeline_status(&message_text),
                message_text,
                created_at: row.get(3),
                attempt: payload["attempt"].as_i64(),
                error: payload["error"].as_str().map(ToOwned::to_owned),
                payload,
            }
        })
        .collect::<Vec<_>>();

    let latest_payload = timeline
        .last()
        .map(|entry| entry.payload.clone())
        .unwrap_or_else(|| json!({}));
    let sent_payload = timeline
        .iter()
        .rev()
        .find(|entry| entry.message_text == "notification sent via mock-log")
        .map(|entry| entry.payload.clone());

    let consumer_row = client
        .query_opt(
            "SELECT result_code, metadata
               FROM ops.consumer_idempotency_record
              WHERE consumer_name = $1
                AND event_id = $2::text::uuid",
            &[&SERVICE_NAME, &event_id],
        )
        .await
        .map_err(|err| format!("load consumer idempotency record failed: {err}"))?;
    let (current_status, current_attempt) = consumer_row
        .map(|row| {
            let metadata: Value = row.get(1);
            (
                row.get::<_, String>(0),
                metadata["attempt"]
                    .as_i64()
                    .or_else(|| latest_payload["attempt"].as_i64()),
            )
        })
        .unwrap_or_else(|| {
            (
                timeline
                    .last()
                    .map(|entry| entry.status.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                latest_payload["attempt"].as_i64(),
            )
        });

    let dead_letter = client
        .query_opt(
            "SELECT dead_letter_event_id::text,
                    target_topic,
                    reprocess_status,
                    failed_reason,
                    created_at::text,
                    COALESCE(last_failed_at::text, first_failed_at::text)
               FROM ops.dead_letter_event
              WHERE payload->>'event_id' = $1
              ORDER BY created_at DESC
              LIMIT 1",
            &[&event_id],
        )
        .await
        .map_err(|err| format!("load dead letter record failed: {err}"))?
        .map(|row| NotificationDeadLetterRecord {
            dead_letter_event_id: row.get(0),
            target_topic: row.get(1),
            reprocess_status: row.get(2),
            failed_reason: row.get(3),
            created_at: row.get(4),
            last_failed_at: row.get(5),
        });

    let audit_timeline = client
        .query(
            "SELECT action_name,
                    result_code,
                    event_time::text,
                    metadata
               FROM audit.audit_event
              WHERE domain_name = 'notification'
                AND trace_id = $1
              ORDER BY event_time ASC, audit_id ASC",
            &[&trace_id],
        )
        .await
        .map_err(|err| format!("load notification audit timeline failed: {err}"))?
        .into_iter()
        .map(|row| NotificationAuditTimelineEntry {
            action_name: row.get(0),
            result_code: row.get(1),
            event_time: row.get(2),
            metadata: row.get(3),
        })
        .collect::<Vec<_>>();

    let channel_result = sent_payload
        .as_ref()
        .and_then(|payload| payload.get("result").cloned());

    Ok(NotificationAuditLookupRecord {
        event_id: event_id.to_string(),
        aggregate_id: base_payload["aggregate_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        request_id,
        trace_id,
        notification_code: base_payload["notification_code"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        template_code: base_payload["template_code"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        channel: base_payload["channel"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        audience_scope: base_payload["audience_scope"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        recipient: base_payload
            .get("recipient")
            .cloned()
            .unwrap_or_else(|| json!({})),
        source_event: base_payload
            .get("source_event")
            .cloned()
            .unwrap_or_else(|| json!({})),
        subject_refs: base_payload
            .get("subject_refs")
            .cloned()
            .unwrap_or_else(|| json!([])),
        links: base_payload
            .get("links")
            .cloned()
            .unwrap_or_else(|| json!([])),
        rendered_variables: base_payload
            .get("variables")
            .cloned()
            .unwrap_or_else(|| json!({})),
        current_status,
        current_attempt,
        title: sent_payload
            .as_ref()
            .and_then(|payload| payload["title"].as_str())
            .map(ToOwned::to_owned),
        body: sent_payload
            .as_ref()
            .and_then(|payload| payload["body"].as_str())
            .map(ToOwned::to_owned),
        channel_result,
        retry_timeline: timeline,
        audit_timeline,
        dead_letter,
    })
}

fn timeline_status(message_text: &str) -> String {
    match message_text {
        "notification sent via mock-log" => "processed",
        "notification send failed and was queued for retry" => "retry_scheduled",
        "notification send exhausted retries and moved to dead letter" => "dead_lettered",
        "notification dead letter replay requested" => "reprocess_requested",
        "notification dead letter replay dry-run prepared" => "reprocess_previewed",
        _ => "info",
    }
    .to_string()
}

async fn load_notification_lookup_payload(
    client: &(impl GenericClient + Sync),
    event_id: &str,
) -> Result<Option<Value>, String> {
    if let Some(row) = client
        .query_opt(
            "SELECT payload
               FROM ops.outbox_event
              WHERE payload->>'event_id' = $1
              ORDER BY created_at DESC
              LIMIT 1",
            &[&event_id],
        )
        .await
        .map_err(|err| format!("load outbox notification envelope failed: {err}"))?
    {
        let envelope: Value = row.get(0);
        if let Ok(envelope) = serde_json::from_value::<NotificationEnvelope>(envelope) {
            return Ok(Some(notification_lookup_metadata(&envelope)));
        }
    }
    if let Some(row) = client
        .query_opt(
            "SELECT payload
               FROM ops.dead_letter_event
              WHERE payload->>'event_id' = $1
              ORDER BY created_at DESC
              LIMIT 1",
            &[&event_id],
        )
        .await
        .map_err(|err| format!("load dead letter notification envelope failed: {err}"))?
    {
        let envelope: Value = row.get(0);
        if let Ok(envelope) = serde_json::from_value::<NotificationEnvelope>(envelope) {
            return Ok(Some(notification_lookup_metadata(&envelope)));
        }
    }
    Ok(None)
}

async fn send_notification_handler(
    State(state): State<Arc<WorkerState>>,
    Json(request): Json<SendNotificationRequest>,
) -> Result<Json<ApiResponse<SendNotificationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let envelope = request.into_envelope().map_err(internal_error)?;
    publish_envelope(&state, &envelope)
        .await
        .map_err(internal_error)?;
    Ok(ApiResponse::ok(SendNotificationResponse::from_envelope(
        &state.cfg.topic,
        &envelope,
    )))
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PreviewTemplateRequest {
    #[serde(flatten)]
    request: SendNotificationRequest,
    #[serde(default)]
    language_code: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct PreviewTemplateResponse {
    event_id: String,
    aggregate_id: String,
    request_id: String,
    trace_id: String,
    template_code: String,
    channel: String,
    language_code: String,
    requested_language_code: String,
    version_no: i32,
    template_enabled: bool,
    template_status: String,
    template_fallback_used: bool,
    body_fallback_used: bool,
    variable_schema: Value,
    template_metadata: Value,
    title: String,
    body: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ReplayDeadLetterRequest {
    #[serde(default = "default_true")]
    dry_run: bool,
    reason: String,
    step_up_ticket: String,
    request_id: Option<String>,
    trace_id: Option<String>,
}

#[derive(Debug, Clone)]
struct DeadLetterReplayTarget {
    dead_letter_event_id: String,
    original_event_id: String,
    reprocess_status: String,
    envelope: NotificationEnvelope,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ReplayDeadLetterResponse {
    dead_letter_event_id: String,
    original_event_id: String,
    replay_event_id: Option<String>,
    request_id: String,
    trace_id: String,
    dry_run: bool,
    status: String,
    notification_topic: String,
    dead_letter_topic: String,
    template_code: String,
    channel: String,
    title: String,
    body: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NotificationAuditLookupRequest {
    #[serde(default)]
    order_id: Option<String>,
    #[serde(default)]
    case_id: Option<String>,
    #[serde(default)]
    template_code: Option<String>,
    #[serde(default)]
    notification_code: Option<String>,
    #[serde(default)]
    event_id: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
    reason: String,
    step_up_ticket: String,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    trace_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct NotificationAuditLookupResponse {
    request_id: String,
    trace_id: String,
    filters: NotificationAuditLookupFilters,
    total: usize,
    records: Vec<NotificationAuditLookupRecord>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct NotificationAuditLookupFilters {
    order_id: Option<String>,
    case_id: Option<String>,
    template_code: Option<String>,
    notification_code: Option<String>,
    event_id: Option<String>,
    limit: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct NotificationAuditLookupRecord {
    event_id: String,
    aggregate_id: String,
    request_id: String,
    trace_id: String,
    notification_code: String,
    template_code: String,
    channel: String,
    audience_scope: String,
    recipient: Value,
    source_event: Value,
    subject_refs: Value,
    links: Value,
    rendered_variables: Value,
    current_status: String,
    current_attempt: Option<i64>,
    title: Option<String>,
    body: Option<String>,
    channel_result: Option<Value>,
    retry_timeline: Vec<NotificationDispatchTimelineEntry>,
    audit_timeline: Vec<NotificationAuditTimelineEntry>,
    dead_letter: Option<NotificationDeadLetterRecord>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct NotificationDispatchTimelineEntry {
    status: String,
    message_text: String,
    created_at: String,
    attempt: Option<i64>,
    error: Option<String>,
    payload: Value,
}

#[derive(Debug, Clone, serde::Serialize)]
struct NotificationAuditTimelineEntry {
    action_name: String,
    result_code: String,
    event_time: String,
    metadata: Value,
}

#[derive(Debug, Clone, serde::Serialize)]
struct NotificationDeadLetterRecord {
    dead_letter_event_id: String,
    target_topic: String,
    reprocess_status: String,
    failed_reason: Option<String>,
    created_at: String,
    last_failed_at: Option<String>,
}

async fn preview_template_handler(
    State(state): State<Arc<WorkerState>>,
    Json(request): Json<PreviewTemplateRequest>,
) -> Result<Json<ApiResponse<PreviewTemplateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let mut envelope = request.request.into_envelope().map_err(internal_error)?;
    if let Some(language_code) = request.language_code {
        if !envelope.payload.metadata.is_object() {
            envelope.payload.metadata = json!({});
        }
        if let Some(object) = envelope.payload.metadata.as_object_mut() {
            object.insert("language_code".to_string(), Value::String(language_code));
        }
    }
    let client = state.db.client().map_err(|err| {
        internal_error(format!(
            "acquire notification worker db client failed: {err}"
        ))
    })?;
    let rendered = state
        .templates
        .render(&client, &envelope, &envelope.payload)
        .await
        .map_err(internal_error)?;
    let effective_trace_id = envelope.effective_trace_id().to_string();

    Ok(ApiResponse::ok(PreviewTemplateResponse {
        event_id: envelope.event_id,
        aggregate_id: envelope.aggregate_id,
        request_id: envelope.request_id,
        trace_id: effective_trace_id,
        template_code: rendered.template_code,
        channel: rendered.channel,
        language_code: rendered.language_code,
        requested_language_code: rendered.requested_language_code,
        version_no: rendered.version_no,
        template_enabled: rendered.template_enabled,
        template_status: rendered.template_status,
        template_fallback_used: rendered.template_fallback_used,
        body_fallback_used: rendered.body_fallback_used,
        variable_schema: rendered.variable_schema,
        template_metadata: rendered.template_metadata,
        title: rendered.title,
        body: rendered.body,
    }))
}

async fn replay_dead_letter_handler(
    Path(dead_letter_event_id): Path<String>,
    State(state): State<Arc<WorkerState>>,
    Json(request): Json<ReplayDeadLetterRequest>,
) -> Result<Json<ApiResponse<ReplayDeadLetterResponse>>, (StatusCode, Json<ErrorResponse>)> {
    validate_replay_request(&request)?;
    let client = state.db.client().map_err(|err| {
        internal_error(format!(
            "acquire notification worker db client failed: {err}"
        ))
    })?;
    let target = load_dead_letter_replay_target(&client, &dead_letter_event_id).await?;
    if !request.dry_run
        && matches!(
            target.reprocess_status.as_str(),
            "reprocess_requested" | "reprocessed"
        )
    {
        return Err(conflict_error(format!(
            "dead letter {} is already in status {}",
            target.dead_letter_event_id, target.reprocess_status
        )));
    }

    let replay_envelope = build_replay_envelope(&target, &request);
    let rendered = state
        .templates
        .render(&client, &replay_envelope, &replay_envelope.payload)
        .await
        .map_err(internal_error)?;

    if request.dry_run {
        write_system_log(
            &client,
            &replay_envelope,
            "info",
            "notification dead letter replay dry-run prepared",
            merge_lookup_metadata(
                &replay_envelope,
                json!({
                    "dead_letter_event_id": target.dead_letter_event_id,
                    "original_event_id": target.original_event_id,
                    "reason": request.reason,
                    "step_up_ticket": request.step_up_ticket,
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "dry_run": true,
                }),
            ),
        )
        .await
        .map_err(internal_error)?;
        write_trace_index(
            &client,
            &replay_envelope,
            "notification.reprocess.dry_run",
            merge_lookup_metadata(
                &replay_envelope,
                json!({
                    "dead_letter_event_id": target.dead_letter_event_id,
                    "original_event_id": target.original_event_id,
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "status": "dry_run_ready",
                }),
            ),
        )
        .await
        .map_err(internal_error)?;
        write_audit_event(
            &client,
            &replay_envelope,
            "notification.dispatch.reprocess.dry_run",
            "previewed",
            merge_lookup_metadata(
                &replay_envelope,
                json!({
                    "dead_letter_event_id": target.dead_letter_event_id,
                    "original_event_id": target.original_event_id,
                    "reason": request.reason,
                    "step_up_ticket": request.step_up_ticket,
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "dry_run": true,
                }),
            ),
        )
        .await
        .map_err(internal_error)?;
        let replay_request_id = replay_envelope.request_id.clone();
        let replay_trace_id = replay_envelope.effective_trace_id().to_string();
        return Ok(ApiResponse::ok(ReplayDeadLetterResponse {
            dead_letter_event_id: target.dead_letter_event_id,
            original_event_id: target.original_event_id,
            replay_event_id: Some(replay_envelope.event_id),
            request_id: replay_request_id,
            trace_id: replay_trace_id,
            dry_run: true,
            status: "dry_run_ready".to_string(),
            notification_topic: state.cfg.topic.clone(),
            dead_letter_topic: state.cfg.dead_letter_topic.clone(),
            template_code: rendered.template_code,
            channel: rendered.channel,
            title: rendered.title,
            body: rendered.body,
        }));
    }

    update_dead_letter_reprocess_status(
        &client,
        &target.dead_letter_event_id,
        "reprocess_requested",
    )
    .await
    .map_err(internal_error)?;
    if let Err(err) = publish_envelope(&state, &replay_envelope).await {
        let _ = update_dead_letter_reprocess_status(
            &client,
            &target.dead_letter_event_id,
            "reprocess_failed",
        )
        .await;
        let _ = write_audit_event(
            &client,
            &replay_envelope,
            "notification.dispatch.reprocess.failed",
            "failed",
            merge_lookup_metadata(
                &replay_envelope,
                json!({
                    "dead_letter_event_id": target.dead_letter_event_id,
                    "original_event_id": target.original_event_id,
                    "reason": request.reason,
                    "step_up_ticket": request.step_up_ticket,
                    "template_code": rendered.template_code,
                    "channel": rendered.channel,
                    "error": err,
                }),
            ),
        )
        .await;
        return Err(internal_error(err));
    }
    write_system_log(
        &client,
        &replay_envelope,
        "warn",
        "notification dead letter replay requested",
        merge_lookup_metadata(
            &replay_envelope,
            json!({
                "dead_letter_event_id": target.dead_letter_event_id,
                "original_event_id": target.original_event_id,
                "reason": request.reason,
                "step_up_ticket": request.step_up_ticket,
                "template_code": rendered.template_code,
                "channel": rendered.channel,
                "dry_run": false,
            }),
        ),
    )
    .await
    .map_err(internal_error)?;
    write_trace_index(
        &client,
        &replay_envelope,
        "notification.reprocess.requested",
        merge_lookup_metadata(
            &replay_envelope,
            json!({
                "dead_letter_event_id": target.dead_letter_event_id,
                "original_event_id": target.original_event_id,
                "template_code": rendered.template_code,
                "channel": rendered.channel,
                "status": "reprocess_requested",
            }),
        ),
    )
    .await
    .map_err(internal_error)?;
    write_audit_event(
        &client,
        &replay_envelope,
        "notification.dispatch.reprocess.requested",
        "accepted",
        merge_lookup_metadata(
            &replay_envelope,
            json!({
                "dead_letter_event_id": target.dead_letter_event_id,
                "original_event_id": target.original_event_id,
                "reason": request.reason,
                "step_up_ticket": request.step_up_ticket,
                "template_code": rendered.template_code,
                "channel": rendered.channel,
                "dry_run": false,
            }),
        ),
    )
    .await
    .map_err(internal_error)?;

    let replay_request_id = replay_envelope.request_id.clone();
    let replay_trace_id = replay_envelope.effective_trace_id().to_string();
    Ok(ApiResponse::ok(ReplayDeadLetterResponse {
        dead_letter_event_id: target.dead_letter_event_id,
        original_event_id: target.original_event_id,
        replay_event_id: Some(replay_envelope.event_id),
        request_id: replay_request_id,
        trace_id: replay_trace_id,
        dry_run: false,
        status: "reprocess_requested".to_string(),
        notification_topic: state.cfg.topic.clone(),
        dead_letter_topic: state.cfg.dead_letter_topic.clone(),
        template_code: rendered.template_code,
        channel: rendered.channel,
        title: rendered.title,
        body: rendered.body,
    }))
}

async fn notification_audit_search_handler(
    State(state): State<Arc<WorkerState>>,
    Json(request): Json<NotificationAuditLookupRequest>,
) -> Result<Json<ApiResponse<NotificationAuditLookupResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request = normalize_lookup_request(request);
    validate_lookup_request(&request)?;
    let client = state.db.client().map_err(|err| {
        internal_error(format!(
            "acquire notification worker db client failed: {err}"
        ))
    })?;
    let request_id = request.request_id.clone().unwrap_or_else(new_uuid_string);
    let trace_id = request
        .trace_id
        .clone()
        .unwrap_or_else(|| request_id.clone());
    let limit = lookup_limit(request.limit);
    let event_ids = load_notification_audit_event_ids(&client, &request, limit)
        .await
        .map_err(internal_error)?;
    let mut records = Vec::with_capacity(event_ids.len());
    for event_id in event_ids {
        records.push(
            load_notification_audit_record(&client, &event_id)
                .await
                .map_err(internal_error)?,
        );
    }
    write_notification_lookup_audit_event(&client, &request, &request_id, &trace_id, records.len())
        .await
        .map_err(internal_error)?;

    Ok(ApiResponse::ok(NotificationAuditLookupResponse {
        request_id,
        trace_id,
        filters: NotificationAuditLookupFilters {
            order_id: request.order_id,
            case_id: request.case_id,
            template_code: request.template_code,
            notification_code: request.notification_code,
            event_id: request.event_id,
            limit,
        },
        total: records.len(),
        records,
    }))
}

async fn live_handler() -> Json<ApiResponse<&'static str>> {
    ApiResponse::ok("ok")
}

async fn ready_handler() -> Json<ApiResponse<&'static str>> {
    ApiResponse::ok("ready")
}

async fn deps_handler(
    State(state): State<Arc<WorkerState>>,
) -> Json<ApiResponse<DependenciesReport>> {
    let checks = check_dependencies(&state.cfg).await;
    let ready = checks.iter().all(|check| check.reachable);
    ApiResponse::ok(DependenciesReport { ready, checks })
}

async fn metrics_handler(
    State(state): State<Arc<WorkerState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let encoder = TextEncoder::new();
    let metric_families = state.metrics.registry.gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|err| internal_error(format!("encode prometheus metrics failed: {err}")))?;
    let body = String::from_utf8(buffer)
        .map_err(|err| internal_error(format!("metrics utf8 decode failed: {err}")))?;
    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_str(encoder.format_type())
                .unwrap_or_else(|_| HeaderValue::from_static("text/plain")),
        )],
        body,
    ))
}

fn default_true() -> bool {
    true
}

fn validate_replay_request(
    request: &ReplayDeadLetterRequest,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if request.reason.trim().is_empty() {
        return Err(bad_request_error("replay reason must not be empty"));
    }
    if request.step_up_ticket.trim().is_empty() {
        return Err(bad_request_error("step_up_ticket must not be empty"));
    }
    Ok(())
}

fn validate_lookup_request(
    request: &NotificationAuditLookupRequest,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if request.reason.trim().is_empty() {
        return Err(bad_request_error("lookup reason must not be empty"));
    }
    if request.step_up_ticket.trim().is_empty() {
        return Err(bad_request_error("step_up_ticket must not be empty"));
    }
    if request
        .order_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .is_none()
        && request
            .case_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .is_none()
        && request
            .template_code
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .is_none()
        && request
            .notification_code
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .is_none()
        && request
            .event_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .is_none()
    {
        return Err(bad_request_error(
            "at least one of order_id, case_id, template_code, notification_code, or event_id is required",
        ));
    }
    Ok(())
}

fn normalize_lookup_request(
    request: NotificationAuditLookupRequest,
) -> NotificationAuditLookupRequest {
    NotificationAuditLookupRequest {
        order_id: normalize_filter_value(request.order_id),
        case_id: normalize_filter_value(request.case_id),
        template_code: normalize_filter_value(request.template_code),
        notification_code: normalize_filter_value(request.notification_code),
        event_id: normalize_filter_value(request.event_id),
        limit: request.limit,
        reason: request.reason.trim().to_string(),
        step_up_ticket: request.step_up_ticket.trim().to_string(),
        request_id: normalize_filter_value(request.request_id),
        trace_id: normalize_filter_value(request.trace_id),
    }
}

fn normalize_filter_value(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn lookup_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(20).clamp(1, 50)
}

fn notification_lookup_metadata(envelope: &NotificationEnvelope) -> Value {
    json!({
        "event_id": envelope.event_id,
        "aggregate_id": envelope.aggregate_id,
        "request_id": envelope.request_id,
        "trace_id": envelope.effective_trace_id(),
        "idempotency_key": envelope.idempotency_key,
        "notification_code": envelope.payload.notification_code,
        "template_code": envelope.payload.template_code,
        "channel": envelope.payload.channel,
        "audience_scope": envelope.payload.audience_scope,
        "recipient": envelope.payload.recipient,
        "source_event": envelope.payload.source_event,
        "subject_refs": envelope.payload.subject_refs,
        "links": envelope.payload.links,
        "variables": envelope.payload.variables,
    })
}

fn merge_lookup_metadata(envelope: &NotificationEnvelope, extra: Value) -> Value {
    merge_json_objects(notification_lookup_metadata(envelope), extra)
}

fn merge_json_objects(mut base: Value, extra: Value) -> Value {
    if !base.is_object() {
        base = json!({});
    }
    let Some(base_object) = base.as_object_mut() else {
        return extra;
    };
    if let Some(extra_object) = extra.as_object() {
        for (key, value) in extra_object {
            base_object.insert(key.clone(), value.clone());
        }
    }
    base
}

async fn load_dead_letter_replay_target(
    client: &(impl GenericClient + Sync),
    dead_letter_event_id: &str,
) -> Result<DeadLetterReplayTarget, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT dead_letter_event_id::text,
                    outbox_event_id::text,
                    reprocess_status,
                    failure_stage,
                    payload
               FROM ops.dead_letter_event
              WHERE dead_letter_event_id = $1::text::uuid",
            &[&dead_letter_event_id],
        )
        .await
        .map_err(|err| internal_error(format!("load dead letter event failed: {err}")))?;
    let Some(row) = row else {
        return Err(not_found_error(format!(
            "dead letter {} not found",
            dead_letter_event_id
        )));
    };
    let failure_stage: String = row.get(3);
    if failure_stage != "notification.send" {
        return Err(bad_request_error(format!(
            "dead letter {} failure stage {} is not replayable by notification-worker",
            dead_letter_event_id, failure_stage
        )));
    }
    let payload: Value = row.get(4);
    let envelope: NotificationEnvelope = serde_json::from_value(payload).map_err(|err| {
        internal_error(format!(
            "decode dead letter {} payload failed: {err}",
            dead_letter_event_id
        ))
    })?;
    if envelope.event_type != "notification.requested" {
        return Err(bad_request_error(format!(
            "dead letter {} does not contain a notification.requested envelope",
            dead_letter_event_id
        )));
    }
    Ok(DeadLetterReplayTarget {
        dead_letter_event_id: row.get(0),
        original_event_id: row.get(1),
        reprocess_status: row.get(2),
        envelope,
    })
}

fn build_replay_envelope(
    target: &DeadLetterReplayTarget,
    request: &ReplayDeadLetterRequest,
) -> NotificationEnvelope {
    let request_id = request.request_id.clone().unwrap_or_else(new_uuid_string);
    let trace_id = request
        .trace_id
        .clone()
        .unwrap_or_else(|| request_id.clone());
    let mut envelope = target.envelope.clone();
    envelope.event_id = new_uuid_string();
    envelope.request_id = request_id;
    envelope.trace_id = trace_id;
    envelope.occurred_at = now_iso8601();
    envelope.producer_service = "notification-worker.replay".to_string();
    if !envelope.payload.metadata.is_object() {
        envelope.payload.metadata = json!({});
    }
    if let Some(metadata) = envelope.payload.metadata.as_object_mut() {
        metadata.remove("simulate_failures");
        metadata.insert(
            "replayed_from_dead_letter_id".to_string(),
            Value::String(target.dead_letter_event_id.clone()),
        );
        metadata.insert(
            "replayed_from_event_id".to_string(),
            Value::String(target.original_event_id.clone()),
        );
        metadata.insert(
            "replay_reason".to_string(),
            Value::String(request.reason.clone()),
        );
        metadata.insert(
            "replay_step_up_ticket".to_string(),
            Value::String(request.step_up_ticket.clone()),
        );
        metadata.insert(
            "replay_requested_at".to_string(),
            Value::String(envelope.occurred_at.clone()),
        );
        metadata.insert(
            "replay_request_id".to_string(),
            Value::String(envelope.request_id.clone()),
        );
    }
    envelope
}

fn replayed_from_dead_letter_id(envelope: &NotificationEnvelope) -> Option<&str> {
    envelope.payload.metadata["replayed_from_dead_letter_id"].as_str()
}

async fn update_dead_letter_reprocess_status(
    client: &(impl GenericClient + Sync),
    dead_letter_event_id: &str,
    status: &str,
) -> Result<(), String> {
    client
        .execute(
            "UPDATE ops.dead_letter_event
                SET reprocess_status = $2,
                    reprocessed_at = now()
              WHERE dead_letter_event_id = $1::text::uuid",
            &[&dead_letter_event_id, &status],
        )
        .await
        .map_err(|err| format!("update dead letter reprocess status failed: {err}"))?;
    Ok(())
}

async fn publish_envelope(
    state: &Arc<WorkerState>,
    envelope: &NotificationEnvelope,
) -> Result<(), String> {
    let payload = serde_json::to_string(envelope)
        .map_err(|err| format!("encode notification envelope failed: {err}"))?;
    state
        .producer
        .send(
            FutureRecord::to(&state.cfg.topic)
                .payload(&payload)
                .key(&envelope.aggregate_id),
            Timeout::After(Duration::from_secs(3)),
        )
        .await
        .map_err(|(err, _)| format!("publish notification envelope failed: {err}"))?;
    Ok(())
}

async fn publish_dead_letter_message(
    state: &Arc<WorkerState>,
    dead_letter_event_id: &str,
    envelope: &NotificationEnvelope,
    error_message: &str,
    attempt: u32,
) -> Result<(), String> {
    let payload = build_dead_letter_message(
        dead_letter_event_id,
        &state.cfg.topic,
        &state.cfg.dead_letter_topic,
        envelope,
        error_message,
        attempt,
    );
    let raw = serde_json::to_string(&payload)
        .map_err(|err| format!("encode dead letter message failed: {err}"))?;
    state
        .producer
        .send(
            FutureRecord::to(&state.cfg.dead_letter_topic)
                .payload(&raw)
                .key(dead_letter_event_id),
            Timeout::After(Duration::from_secs(3)),
        )
        .await
        .map_err(|(err, _)| format!("publish dead letter payload failed: {err}"))?;
    Ok(())
}

fn build_dead_letter_message(
    dead_letter_event_id: &str,
    source_topic: &str,
    dead_letter_topic: &str,
    envelope: &NotificationEnvelope,
    error_message: &str,
    attempt: u32,
) -> Value {
    json!({
        "dead_letter_event_id": dead_letter_event_id,
        "source_topic": source_topic,
        "target_topic": dead_letter_topic,
        "event_id": envelope.event_id,
        "event_type": envelope.event_type,
        "aggregate_type": envelope.aggregate_type,
        "aggregate_id": envelope.aggregate_id,
        "request_id": envelope.request_id,
        "trace_id": envelope.effective_trace_id(),
        "failure_stage": "notification.send",
        "failure_reason": error_message,
        "reprocess_status": "not_reprocessed",
        "attempt": attempt,
        "payload": envelope,
    })
}

impl WorkerMetrics {
    fn new() -> AppResult<Self> {
        let registry = Registry::new();
        let event_results = IntCounterVec::new(
            prometheus::Opts::new(
                "notification_worker_events_total",
                "Notification worker event processing results",
            ),
            &["result"],
        )
        .map_err(|err| AppError::Startup(format!("build event counter failed: {err}")))?;
        let send_results = IntCounterVec::new(
            prometheus::Opts::new(
                "notification_worker_send_total",
                "Notification worker channel send results",
            ),
            &["channel", "result"],
        )
        .map_err(|err| AppError::Startup(format!("build send counter failed: {err}")))?;
        let retry_queue_depth = IntGauge::new(
            "notification_worker_retry_queue_depth",
            "Current retry queue depth in Redis",
        )
        .map_err(|err| AppError::Startup(format!("build retry queue gauge failed: {err}")))?;

        registry
            .register(Box::new(event_results.clone()))
            .map_err(|err| AppError::Startup(format!("register event counter failed: {err}")))?;
        registry
            .register(Box::new(send_results.clone()))
            .map_err(|err| AppError::Startup(format!("register send counter failed: {err}")))?;
        registry
            .register(Box::new(retry_queue_depth.clone()))
            .map_err(|err| AppError::Startup(format!("register retry gauge failed: {err}")))?;

        Ok(Self {
            registry,
            event_results,
            send_results,
            retry_queue_depth,
        })
    }
}

fn retry_policy(
    state: &Arc<WorkerState>,
    envelope: &NotificationEnvelope,
) -> event::NotificationRetryPolicy {
    envelope
        .payload
        .retry_policy
        .clone()
        .unwrap_or(event::NotificationRetryPolicy {
            max_attempts: state.cfg.retry_max_attempts,
            backoff_ms: state.cfg.retry_backoff_ms,
        })
}

async fn redis_connection(
    state: &Arc<WorkerState>,
) -> Result<redis::aio::MultiplexedConnection, String> {
    state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("connect redis failed: {err}"))
}

async fn update_retry_queue_depth(
    state: &Arc<WorkerState>,
    conn: &mut redis::aio::MultiplexedConnection,
) -> Result<(), String> {
    let queue_depth: usize = redis::cmd("ZCARD")
        .arg(retry_queue_key(state))
        .query_async(conn)
        .await
        .map_err(|err| format!("query retry queue depth failed: {err}"))?;
    state.metrics.retry_queue_depth.set(queue_depth as i64);
    Ok(())
}

fn retry_queue_key(state: &Arc<WorkerState>) -> String {
    format!("{}:notification:retry-queue", state.cfg.redis_namespace)
}

fn retry_payload_key(state: &Arc<WorkerState>, event_id: &str) -> String {
    format!(
        "{}:notification:retry-payload:{event_id}",
        state.cfg.redis_namespace
    )
}

fn short_state_key(state: &Arc<WorkerState>, event_id: &str) -> String {
    format!(
        "{}:notification:state:{event_id}",
        state.cfg.redis_namespace
    )
}

fn process_source_name(source: ProcessSource) -> &'static str {
    match source {
        ProcessSource::Kafka => "kafka",
        ProcessSource::RetryQueue => "retry_queue",
    }
}

fn severity_number(log_level: &str) -> i32 {
    match log_level {
        "error" => 17,
        "warn" => 13,
        "info" => 9,
        _ => 5,
    }
}

fn state_env_code() -> String {
    std::env::var("APP_MODE").unwrap_or_else(|_| "local".to_string())
}

fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn now_utc_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn internal_error(message: impl ToString) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: "notification_worker_error".to_string(),
            message: message.to_string(),
            request_id: None,
        }),
    )
}

fn bad_request_error(message: impl ToString) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: "notification_worker_bad_request".to_string(),
            message: message.to_string(),
            request_id: None,
        }),
    )
}

fn not_found_error(message: impl ToString) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: "notification_worker_not_found".to_string(),
            message: message.to_string(),
            request_id: None,
        }),
    )
}

fn conflict_error(message: impl ToString) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: "notification_worker_conflict".to_string(),
            message: message.to_string(),
            request_id: None,
        }),
    )
}

async fn check_dependencies(cfg: &NotificationWorkerConfig) -> Vec<DependencyStatus> {
    let targets = vec![
        dependency_target(
            "db",
            resolve_host_port("DB_HOST", "127.0.0.1", "DB_PORT", "5432"),
        ),
        dependency_target(
            "redis",
            resolve_host_port("REDIS_HOST", "127.0.0.1", "REDIS_PORT", "6379"),
        ),
        dependency_target(
            "kafka",
            resolve_kafka_target(&cfg.kafka_brokers)
                .unwrap_or_else(|| "127.0.0.1:9094".to_string()),
        ),
        dependency_target(
            "keycloak",
            resolve_host_port("KEYCLOAK_HOST", "127.0.0.1", "KEYCLOAK_PORT", "8081"),
        ),
    ];

    let mut checks = Vec::with_capacity(targets.len());
    for (name, endpoint) in targets {
        let reachable = time::timeout(
            Duration::from_millis(500),
            TcpStream::connect(endpoint.clone()),
        )
        .await
        .map(|result| result.is_ok())
        .unwrap_or(false);
        checks.push(DependencyStatus {
            name: name.to_string(),
            endpoint,
            reachable,
        });
    }
    checks
}

fn dependency_target(name: &'static str, endpoint: String) -> (&'static str, String) {
    (name, endpoint)
}

fn resolve_host_port(
    host_env: &'static str,
    default_host: &'static str,
    port_env: &'static str,
    default_port: &'static str,
) -> String {
    let host = std::env::var(host_env).unwrap_or_else(|_| default_host.to_string());
    let port = std::env::var(port_env).unwrap_or_else(|_| default_port.to_string());
    format!("{host}:{port}")
}

fn resolve_kafka_target(raw: &str) -> Option<String> {
    raw.split(',').next().map(str::trim).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::NotificationRecipient;

    #[test]
    fn build_short_state_payload_keeps_optional_details() {
        let payload = build_short_state_payload(
            "processing",
            2,
            Some(json!({
                "source": "retry_queue",
            })),
        );

        assert_eq!(payload["status"], "processing");
        assert_eq!(payload["attempt"], 2);
        assert_eq!(payload["details"]["source"], "retry_queue");
        assert!(payload["updated_at"].as_str().is_some());
    }

    #[test]
    fn resolve_kafka_target_uses_first_broker() {
        assert_eq!(
            resolve_kafka_target("127.0.0.1:9094,127.0.0.1:9095"),
            Some("127.0.0.1:9094".to_string())
        );
        assert_eq!(resolve_kafka_target(""), None);
    }

    fn sample_dead_letter_target() -> DeadLetterReplayTarget {
        let envelope = SendNotificationRequest {
            event_id: Some("11111111-1111-1111-1111-111111111111".to_string()),
            aggregate_id: Some("22222222-2222-2222-2222-222222222222".to_string()),
            notification_code: Some("payment.succeeded".to_string()),
            audience_scope: Some("buyer".to_string()),
            template_code: None,
            channel: None,
            recipient: NotificationRecipient {
                kind: "user".to_string(),
                address: "buyer@example.test".to_string(),
                id: Some("33333333-3333-3333-3333-333333333333".to_string()),
                display_name: Some("Buyer".to_string()),
            },
            variables: Some(json!({"orderId":"ORD-1"})),
            metadata: Some(json!({"simulate_failures": 2})),
            source_event: None,
            subject_refs: None,
            links: None,
            idempotency_key: Some("notif-idem-1".to_string()),
            request_id: Some("req-original".to_string()),
            trace_id: Some("trace-original".to_string()),
            retry_policy: None,
            simulate_failures: None,
        }
        .into_envelope()
        .expect("sample envelope");

        DeadLetterReplayTarget {
            dead_letter_event_id: "44444444-4444-4444-4444-444444444444".to_string(),
            original_event_id: envelope.event_id.clone(),
            reprocess_status: "not_reprocessed".to_string(),
            envelope,
        }
    }

    #[test]
    fn replay_envelope_preserves_idempotency_and_marks_lineage() {
        let target = sample_dead_letter_target();
        let request = ReplayDeadLetterRequest {
            dry_run: false,
            reason: "manual replay after provider recovery".to_string(),
            step_up_ticket: "step-up-local-1".to_string(),
            request_id: Some("req-replay".to_string()),
            trace_id: Some("trace-replay".to_string()),
        };

        let replay = build_replay_envelope(&target, &request);

        assert_ne!(replay.event_id, target.original_event_id);
        assert_eq!(replay.aggregate_id, target.envelope.aggregate_id);
        assert_eq!(replay.idempotency_key, target.envelope.idempotency_key);
        assert_eq!(
            replay.payload.metadata["replayed_from_dead_letter_id"],
            target.dead_letter_event_id
        );
        assert_eq!(
            replay.payload.metadata["replayed_from_event_id"],
            target.original_event_id
        );
        assert_eq!(replay.payload.metadata["replay_reason"], request.reason);
        assert_eq!(
            replay.payload.metadata["replay_step_up_ticket"],
            request.step_up_ticket
        );
        assert!(replay.payload.metadata.get("simulate_failures").is_none());
        assert_eq!(replay.request_id, "req-replay");
        assert_eq!(replay.trace_id, "trace-replay");
        assert_eq!(replay.producer_service, "notification-worker.replay");
    }

    #[test]
    fn dead_letter_message_contains_dual_dlq_fields() {
        let target = sample_dead_letter_target();
        let payload = build_dead_letter_message(
            &target.dead_letter_event_id,
            "dtp.notification.dispatch",
            "dtp.dead-letter",
            &target.envelope,
            "mock-log forced failure",
            3,
        );

        assert_eq!(payload["dead_letter_event_id"], target.dead_letter_event_id);
        assert_eq!(payload["source_topic"], "dtp.notification.dispatch");
        assert_eq!(payload["target_topic"], "dtp.dead-letter");
        assert_eq!(payload["failure_stage"], "notification.send");
        assert_eq!(payload["failure_reason"], "mock-log forced failure");
        assert_eq!(payload["attempt"], 3);
        assert_eq!(payload["payload"]["event_id"], target.original_event_id);
    }

    #[test]
    fn replay_request_requires_reason_and_step_up_ticket() {
        let invalid_reason = ReplayDeadLetterRequest {
            dry_run: true,
            reason: "   ".to_string(),
            step_up_ticket: "step-up".to_string(),
            request_id: None,
            trace_id: None,
        };
        assert!(validate_replay_request(&invalid_reason).is_err());

        let invalid_step_up = ReplayDeadLetterRequest {
            dry_run: true,
            reason: "manual replay".to_string(),
            step_up_ticket: " ".to_string(),
            request_id: None,
            trace_id: None,
        };
        assert!(validate_replay_request(&invalid_step_up).is_err());
    }

    #[test]
    fn lookup_request_requires_filter_reason_and_step_up_ticket() {
        let no_filter = NotificationAuditLookupRequest {
            order_id: None,
            case_id: None,
            template_code: None,
            notification_code: None,
            event_id: None,
            limit: None,
            reason: "trace notification history".to_string(),
            step_up_ticket: "step-up-local-1".to_string(),
            request_id: None,
            trace_id: None,
        };
        assert!(validate_lookup_request(&no_filter).is_err());

        let invalid_reason = NotificationAuditLookupRequest {
            order_id: Some("order-1".to_string()),
            case_id: None,
            template_code: None,
            notification_code: None,
            event_id: None,
            limit: None,
            reason: "   ".to_string(),
            step_up_ticket: "step-up-local-1".to_string(),
            request_id: None,
            trace_id: None,
        };
        assert!(validate_lookup_request(&invalid_reason).is_err());

        let invalid_step_up = NotificationAuditLookupRequest {
            order_id: Some("order-1".to_string()),
            case_id: None,
            template_code: None,
            notification_code: None,
            event_id: None,
            limit: None,
            reason: "trace notification history".to_string(),
            step_up_ticket: "   ".to_string(),
            request_id: None,
            trace_id: None,
        };
        assert!(validate_lookup_request(&invalid_step_up).is_err());
    }

    #[test]
    fn lookup_metadata_keeps_subject_refs_and_variables() {
        let target = sample_dead_letter_target();
        let payload = merge_lookup_metadata(
            &target.envelope,
            json!({
                "template_code": "NOTIFY_PAYMENT_SUCCEEDED_V1",
                "attempt": 2,
            }),
        );

        assert_eq!(payload["event_id"], target.original_event_id);
        assert_eq!(payload["notification_code"], "payment.succeeded");
        assert_eq!(payload["attempt"], 2);
        assert!(payload["subject_refs"].is_array());
        assert_eq!(payload["variables"]["orderId"], "ORD-1");
    }
}
