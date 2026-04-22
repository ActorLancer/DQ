use audit_kit::AuditAnnotation;
use axum::{
    Json, Router,
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use config::RuntimeConfig;
use kernel::{AppError, AppResult, ErrorResponse, new_uuid_string};
use prometheus::{Encoder, HistogramOpts, HistogramVec, IntCounterVec, Registry, TextEncoder};
use serde::Serialize;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};
use std::{future::Future, time::Duration, time::Instant, time::SystemTime, time::UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::info;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: T,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyStatus {
    pub name: String,
    pub endpoint: String,
    pub reachable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependenciesReport {
    pub ready: bool,
    pub checks: Vec<DependencyStatus>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TraceLinks {
    pub grafana: String,
    pub loki: String,
    pub tempo: String,
    pub keycloak: String,
    pub minio_console: String,
    pub opensearch: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DevOverviewOutboxItem {
    pub event_id: String,
    pub topic: String,
    pub status: String,
    pub observed_at_utc_ms: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DevOverviewDeadLetterItem {
    pub event_id: String,
    pub topic: String,
    pub reason: String,
    pub observed_at_utc_ms: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DevOverviewChainReceiptItem {
    pub receipt_id: String,
    pub tx_id: String,
    pub status: String,
    pub observed_at_utc_ms: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DevOverview {
    pub run_mode: String,
    pub provider_mode: String,
    pub recent_outbox: Vec<DevOverviewOutboxItem>,
    pub recent_dead_letters: Vec<DevOverviewDeadLetterItem>,
    pub recent_chain_receipts: Vec<DevOverviewChainReceiptItem>,
}

#[derive(Debug, Default)]
struct DevOverviewFeed {
    outbox: VecDeque<DevOverviewOutboxItem>,
    dead_letters: VecDeque<DevOverviewDeadLetterItem>,
    chain_receipts: VecDeque<DevOverviewChainReceiptItem>,
}

const DEV_OVERVIEW_WINDOW: usize = 10;
static DEV_OVERVIEW_FEED: OnceLock<Mutex<DevOverviewFeed>> = OnceLock::new();
static HTTP_METRICS: OnceLock<Arc<HttpMetrics>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub trace_id: String,
    pub tenant_id: String,
    pub idempotency_key: String,
}

pub fn build_router<S>(runtime: RuntimeConfig) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/health/live", get(live_handler))
        .route("/health/ready", get(ready_handler))
        .route("/health/deps", get(deps_handler))
        .route("/metrics", get(metrics_handler))
        .route(
            "/internal/runtime",
            get({
                let runtime = runtime.clone();
                move || {
                    let runtime = runtime.clone();
                    async move { ApiResponse::ok(runtime) }
                }
            }),
        )
        .merge(build_internal_dev_router())
        .layer(middleware::from_fn(request_context_middleware))
}

fn build_internal_dev_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/internal/dev/trace-links", get(trace_links_handler))
        .route("/internal/dev/overview", get(dev_overview_handler))
}

pub async fn live_handler() -> Json<ApiResponse<&'static str>> {
    ApiResponse::ok("ok")
}

pub async fn ready_handler()
-> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(ApiResponse::ok("ready"))
}

pub async fn deps_handler() -> Json<ApiResponse<DependenciesReport>> {
    let checks = check_dependencies().await;
    let ready = checks.iter().all(|c| c.reachable);
    ApiResponse::ok(DependenciesReport { ready, checks })
}

pub async fn trace_links_handler() -> Json<ApiResponse<TraceLinks>> {
    ApiResponse::ok(build_trace_links())
}

pub async fn dev_overview_handler() -> Json<ApiResponse<DevOverview>> {
    ApiResponse::ok(build_dev_overview())
}

async fn metrics_handler() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let encoder = TextEncoder::new();
    let metric_families = http_metrics().registry.gather();
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

async fn check_dependencies() -> Vec<DependencyStatus> {
    let targets = vec![
        dep_target("db", "DB_HOST", "localhost", "DB_PORT", "5432"),
        dep_target("redis", "REDIS_HOST", "localhost", "REDIS_PORT", "6379"),
        dep_target("kafka", "KAFKA_HOST", "localhost", "KAFKA_PORT", "9092"),
        dep_target("minio", "MINIO_HOST", "localhost", "MINIO_PORT", "9000"),
        dep_target(
            "keycloak",
            "KEYCLOAK_HOST",
            "localhost",
            "KEYCLOAK_PORT",
            "8081",
        ),
        dep_target(
            "fabric-adapter",
            "FABRIC_ADAPTER_HOST",
            "localhost",
            "FABRIC_ADAPTER_PORT",
            "10080",
        ),
    ];

    let mut results = Vec::with_capacity(targets.len());
    for (name, endpoint) in targets {
        let reachable = timeout(
            Duration::from_millis(500),
            TcpStream::connect(endpoint.clone()),
        )
        .await
        .map(|r| r.is_ok())
        .unwrap_or(false);
        results.push(DependencyStatus {
            name: name.to_string(),
            endpoint,
            reachable,
        });
    }
    results
}

fn dep_target(
    name: &'static str,
    host_env: &'static str,
    default_host: &'static str,
    port_env: &'static str,
    default_port: &'static str,
) -> (&'static str, String) {
    let host = std::env::var(host_env).unwrap_or_else(|_| default_host.to_string());
    let port = std::env::var(port_env).unwrap_or_else(|_| default_port.to_string());
    (name, format!("{host}:{port}"))
}

fn build_trace_links() -> TraceLinks {
    let host = std::env::var("DEV_LINK_HOST").unwrap_or_else(|_| "localhost".to_string());
    let grafana = std::env::var("GRAFANA_PORT").unwrap_or_else(|_| "3000".to_string());
    let loki = std::env::var("LOKI_PORT").unwrap_or_else(|_| "3100".to_string());
    let tempo = std::env::var("TEMPO_PORT").unwrap_or_else(|_| "3200".to_string());
    let keycloak = std::env::var("KEYCLOAK_PORT").unwrap_or_else(|_| "8081".to_string());
    let minio_console = std::env::var("MINIO_CONSOLE_PORT").unwrap_or_else(|_| "9001".to_string());
    let opensearch = std::env::var("OPENSEARCH_HTTP_PORT").unwrap_or_else(|_| "9200".to_string());

    TraceLinks {
        grafana: format!("http://{host}:{grafana}"),
        loki: format!("http://{host}:{loki}"),
        tempo: format!("http://{host}:{tempo}"),
        keycloak: format!("http://{host}:{keycloak}"),
        minio_console: format!("http://{host}:{minio_console}"),
        opensearch: format!("http://{host}:{opensearch}"),
    }
}

pub fn record_outbox_event(
    event_id: impl Into<String>,
    topic: impl Into<String>,
    status: impl Into<String>,
) {
    let mut feed = dev_overview_feed()
        .lock()
        .expect("dev overview feed lock poisoned");
    push_capped(
        &mut feed.outbox,
        DevOverviewOutboxItem {
            event_id: event_id.into(),
            topic: topic.into(),
            status: status.into(),
            observed_at_utc_ms: now_utc_ms(),
        },
    );
}

pub fn record_dead_letter_event(
    event_id: impl Into<String>,
    topic: impl Into<String>,
    reason: impl Into<String>,
) {
    let mut feed = dev_overview_feed()
        .lock()
        .expect("dev overview feed lock poisoned");
    push_capped(
        &mut feed.dead_letters,
        DevOverviewDeadLetterItem {
            event_id: event_id.into(),
            topic: topic.into(),
            reason: reason.into(),
            observed_at_utc_ms: now_utc_ms(),
        },
    );
}

pub fn record_chain_receipt(
    receipt_id: impl Into<String>,
    tx_id: impl Into<String>,
    status: impl Into<String>,
) {
    let mut feed = dev_overview_feed()
        .lock()
        .expect("dev overview feed lock poisoned");
    push_capped(
        &mut feed.chain_receipts,
        DevOverviewChainReceiptItem {
            receipt_id: receipt_id.into(),
            tx_id: tx_id.into(),
            status: status.into(),
            observed_at_utc_ms: now_utc_ms(),
        },
    );
}

fn build_dev_overview() -> DevOverview {
    let feed = dev_overview_feed()
        .lock()
        .expect("dev overview feed lock poisoned");
    DevOverview {
        run_mode: std::env::var("APP_MODE").unwrap_or_else(|_| "local".to_string()),
        provider_mode: std::env::var("PROVIDER_MODE").unwrap_or_else(|_| "mock".to_string()),
        recent_outbox: feed.outbox.iter().cloned().collect(),
        recent_dead_letters: feed.dead_letters.iter().cloned().collect(),
        recent_chain_receipts: feed.chain_receipts.iter().cloned().collect(),
    }
}

fn dev_overview_feed() -> &'static Mutex<DevOverviewFeed> {
    DEV_OVERVIEW_FEED.get_or_init(|| Mutex::new(DevOverviewFeed::default()))
}

fn push_capped<T>(items: &mut VecDeque<T>, item: T) {
    items.push_front(item);
    while items.len() > DEV_OVERVIEW_WINDOW {
        let _ = items.pop_back();
    }
}

fn now_utc_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

async fn request_context_middleware(mut req: Request, next: Next) -> Response {
    let started_at = Instant::now();
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(new_uuid_string);
    let trace_id = req
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| request_id.clone());
    let tenant_id = req
        .headers()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "public".to_string());
    let idempotency_key = resolve_idempotency_key(req.headers(), &request_id);

    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let metrics_path = normalize_metrics_path(&path);
    req.extensions_mut().insert(RequestContext {
        request_id: request_id.clone(),
        trace_id: trace_id.clone(),
        tenant_id: tenant_id.clone(),
        idempotency_key: idempotency_key.clone(),
    });

    let mut response = next.run(req).await;
    let status = response.status().as_u16().to_string();
    let elapsed_seconds = started_at.elapsed().as_secs_f64();
    http_metrics()
        .requests_total
        .with_label_values(&[method.as_str(), metrics_path.as_str(), status.as_str()])
        .inc();
    http_metrics()
        .request_duration_seconds
        .with_label_values(&[method.as_str(), metrics_path.as_str()])
        .observe(elapsed_seconds);
    set_header(&mut response, "x-request-id", &request_id);
    set_header(&mut response, "x-trace-id", &trace_id);
    set_header(&mut response, "x-tenant-id", &tenant_id);
    set_header(&mut response, "x-idempotency-key", &idempotency_key);

    info!(
        request_id = %request_id,
        trace_id = %trace_id,
        tenant_id = %tenant_id,
        idempotency_key = %idempotency_key,
        method = %method,
        path = %path,
        status = %response.status().as_u16(),
        elapsed_ms = started_at.elapsed().as_millis(),
        "request finished"
    );
    response
}

#[derive(Debug)]
struct HttpMetrics {
    registry: Registry,
    requests_total: IntCounterVec,
    request_duration_seconds: HistogramVec,
}

impl HttpMetrics {
    fn new() -> Result<Self, String> {
        let registry = Registry::new();
        let requests_total = IntCounterVec::new(
            prometheus::Opts::new(
                "platform_core_http_requests_total",
                "Platform core HTTP requests by method, normalized path, and status",
            ),
            &["method", "path", "status"],
        )
        .map_err(|err| format!("build request counter failed: {err}"))?;
        let request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "platform_core_http_request_duration_seconds",
                "Platform core HTTP request latency by method and normalized path",
            )
            .buckets(vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
            &["method", "path"],
        )
        .map_err(|err| format!("build request duration histogram failed: {err}"))?;
        registry
            .register(Box::new(requests_total.clone()))
            .map_err(|err| format!("register request counter failed: {err}"))?;
        registry
            .register(Box::new(request_duration_seconds.clone()))
            .map_err(|err| format!("register request duration histogram failed: {err}"))?;
        Ok(Self {
            registry,
            requests_total,
            request_duration_seconds,
        })
    }
}

fn http_metrics() -> &'static Arc<HttpMetrics> {
    HTTP_METRICS.get_or_init(|| {
        Arc::new(
            HttpMetrics::new()
                .unwrap_or_else(|err| panic!("platform-core http metrics init failed: {err}")),
        )
    })
}

fn normalize_metrics_path(path: &str) -> String {
    let normalized_segments = path
        .split('/')
        .map(|segment| {
            if segment.is_empty() {
                String::new()
            } else if looks_like_dynamic_path_segment(segment) {
                "{id}".to_string()
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>();
    let normalized = normalized_segments.join("/");
    if normalized.is_empty() {
        "/".to_string()
    } else {
        normalized
    }
}

fn looks_like_dynamic_path_segment(segment: &str) -> bool {
    if segment.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if segment.len() >= 8
        && segment
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '-' || c == '_')
        && (segment.contains('-') || segment.contains('_'))
    {
        return true;
    }
    segment.len() >= 24
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn internal_error(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: "OPS_INTERNAL".to_string(),
            message: message.into(),
            request_id: None,
        }),
    )
}

pub fn set_audit_annotation(req: &mut Request, annotation: AuditAnnotation) {
    req.extensions_mut().insert(annotation);
}

pub fn get_audit_annotation(req: &Request) -> Option<&AuditAnnotation> {
    req.extensions().get::<AuditAnnotation>()
}

fn resolve_idempotency_key(headers: &HeaderMap, request_id: &str) -> String {
    headers
        .get("idempotency-key")
        .or_else(|| headers.get("x-idempotency-key"))
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
        .unwrap_or_else(|| request_id.to_string())
}

fn set_header(response: &mut Response, name: &str, value: &str) {
    if let (Ok(header_name), Ok(header_value)) = (
        HeaderName::from_bytes(name.as_bytes()),
        HeaderValue::from_str(value),
    ) {
        response.headers_mut().insert(header_name, header_value);
    }
}

pub async fn serve(
    addr: SocketAddr,
    app: Router,
    shutdown: impl Future<Output = Result<(), std::io::Error>> + Send + 'static,
) -> AppResult<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Startup(format!("bind listener failed: {e}")))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
        })
        .await
        .map_err(|e| AppError::Shutdown(format!("http server stopped with error: {e}")))?;
    Ok(())
}
