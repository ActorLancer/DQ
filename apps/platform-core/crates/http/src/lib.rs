use audit_kit::AuditAnnotation;
use axum::{
    Json, Router,
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use config::RuntimeConfig;
use kernel::{AppError, AppResult, ErrorResponse, new_uuid_string};
use serde::Serialize;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Mutex, OnceLock};
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

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub trace_id: String,
    pub tenant_id: String,
    pub idempotency_key: String,
}

pub fn build_router(runtime: RuntimeConfig) -> Router {
    Router::new()
        .route("/health/live", get(live_handler))
        .route("/health/ready", get(ready_handler))
        .route("/health/deps", get(deps_handler))
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

fn build_internal_dev_router() -> Router {
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
    req.extensions_mut().insert(RequestContext {
        request_id: request_id.clone(),
        trace_id: trace_id.clone(),
        tenant_id: tenant_id.clone(),
        idempotency_key: idempotency_key.clone(),
    });

    let mut response = next.run(req).await;
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
