use auth::{
    AuthorizationFacade, MockJwtParser, NoopStepUpGateway, RolePermissionChecker,
    UnifiedAuthorizationFacade,
};
use config::{ProviderMode, RuntimeConfig, RuntimeMode};
use db::{
    AppDb, DbPool, DbPoolConfig, NoopBusinessMutationWriter, OrderRepository,
    OrderRepositoryBackend, TxTemplate, build_order_repository,
};
use http::{
    build_router, live_handler, record_chain_receipt, record_outbox_event, serve,
    with_http_observability,
};
use kernel::{
    AppError, AppLauncher, AppResult, DomainEventEnvelope, InProcessEventBus, Module,
    ModuleContext, UtcTimestampMs, new_external_readable_id, validate_error_code_document,
};
use provider_kit::{
    FabricWriterProvider, KycProvider, NotificationProvider, PaymentProvider, ProviderBackend,
    SigningProvider, build_fabric_writer_provider, build_kyc_provider, build_notification_provider,
    build_payment_provider, build_signing_provider,
};
use rdkafka::consumer::{BaseConsumer, Consumer};
use reqwest::StatusCode;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::info;

mod app;
pub mod modules;
mod shared;

use crate::modules::search::domain::{
    PRODUCT_SEARCH_READ_ALIAS, PRODUCT_SEARCH_WRITE_ALIAS, SEARCH_SYNC_JOBS_INDEX,
    SELLER_SEARCH_READ_ALIAS, SELLER_SEARCH_WRITE_ALIAS,
};

#[derive(Clone)]
pub struct AppState {
    pub runtime: RuntimeConfig,
    pub db: Arc<AppDb>,
}

#[cfg(test)]
pub fn stub_test_app_state() -> AppState {
    AppState {
        runtime: RuntimeConfig::from_env().expect("test runtime config should load"),
        db: Arc::new(AppDb::Mysql(db::MySqlDbRuntime {
            dsn: "mysql://reserved-for-tests".to_string(),
            max_connections: 1,
        })),
    }
}

#[cfg(test)]
pub async fn live_test_app_state() -> AppState {
    let dsn = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string());
    let db = AppDb::connect(
        DbPoolConfig {
            dsn,
            max_connections: 16,
        }
        .into(),
    )
    .await
    .expect("live test database should connect");

    AppState {
        runtime: RuntimeConfig::from_env().expect("test runtime config should load"),
        db: Arc::new(db),
    }
}

#[cfg(test)]
pub fn with_stub_test_state(router: axum::Router<AppState>) -> axum::Router {
    with_http_observability(router.with_state(stub_test_app_state()))
}

#[cfg(test)]
pub async fn with_live_test_state(router: axum::Router<AppState>) -> axum::Router {
    with_http_observability(router.with_state(live_test_app_state().await))
}

#[cfg(test)]
fn write_test_artifact(env_key: &str, file_name: &str, artifact: &serde_json::Value) {
    let Ok(dir) = std::env::var(env_key) else {
        return;
    };
    let artifact_dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&artifact_dir).expect("test artifact dir should exist");
    let artifact_path = artifact_dir.join(file_name);
    let payload = serde_json::to_vec_pretty(artifact).expect("test artifact json");
    std::fs::write(artifact_path, payload).expect("test artifact should write");
}

#[cfg(test)]
pub fn write_test024_artifact(file_name: &str, artifact: &serde_json::Value) {
    write_test_artifact("TEST024_ARTIFACT_DIR", file_name, artifact);
}

#[cfg(test)]
pub fn write_test025_artifact(file_name: &str, artifact: &serde_json::Value) {
    write_test_artifact("TEST025_ARTIFACT_DIR", file_name, artifact);
}

#[cfg(test)]
pub fn write_test026_artifact(file_name: &str, artifact: &serde_json::Value) {
    write_test_artifact("TEST026_ARTIFACT_DIR", file_name, artifact);
}

#[cfg(test)]
pub fn write_test027_artifact(file_name: &str, artifact: &serde_json::Value) {
    write_test_artifact("TEST027_ARTIFACT_DIR", file_name, artifact);
}

struct CoreModule {
    provider_backend: ProviderBackend,
}

#[derive(Deserialize)]
struct KafkaTopicCatalog {
    topics: Vec<KafkaTopicDefinition>,
}

#[derive(Deserialize)]
struct KafkaTopicDefinition {
    env_key: String,
    name: String,
    required_in_startup_check: bool,
}

#[async_trait::async_trait]
impl Module for CoreModule {
    fn name(&self) -> &'static str {
        "platform-core"
    }

    async fn start(&self, ctx: &ModuleContext) -> AppResult<()> {
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://local:local@127.0.0.1:5432/platform".to_string());
        let app_db = Arc::new(
            AppDb::connect(
                DbPoolConfig {
                    dsn: dsn.clone(),
                    max_connections: 16,
                }
                .into(),
            )
            .await?,
        );
        let pool = DbPool::connect(DbPoolConfig {
            dsn,
            max_connections: 16,
        })?;
        let order_repo_backend = OrderRepositoryBackend::from_env()?;
        let order_repository = build_order_repository(&pool, order_repo_backend);

        ctx.container.insert(app_db.clone()).await;
        ctx.container.insert(pool).await;
        ctx.container.insert(TxTemplate).await;
        ctx.container.insert(NoopBusinessMutationWriter).await;
        ctx.container
            .insert::<Arc<dyn OrderRepository>>(order_repository)
            .await;
        ctx.container.insert(RolePermissionChecker).await;
        ctx.container.insert(NoopStepUpGateway).await;
        ctx.container.insert(MockJwtParser).await;
        ctx.container
            .insert::<Arc<dyn AuthorizationFacade>>(Arc::new(UnifiedAuthorizationFacade::new(
                Box::new(MockJwtParser),
                Box::new(RolePermissionChecker),
                Box::new(NoopStepUpGateway),
            )))
            .await;
        ctx.container
            .insert::<Arc<dyn KycProvider>>(build_kyc_provider(self.provider_backend))
            .await;
        ctx.container
            .insert::<Arc<dyn SigningProvider>>(build_signing_provider(self.provider_backend))
            .await;
        ctx.container
            .insert::<Arc<dyn PaymentProvider>>(build_payment_provider(self.provider_backend))
            .await;
        ctx.container
            .insert::<Arc<dyn NotificationProvider>>(build_notification_provider(
                self.provider_backend,
            ))
            .await;
        ctx.container
            .insert::<Arc<dyn FabricWriterProvider>>(build_fabric_writer_provider(
                self.provider_backend,
            ))
            .await;
        let event_bus = Arc::new(InProcessEventBus::new(128));
        ctx.container
            .insert::<Arc<InProcessEventBus>>(event_bus.clone())
            .await;
        event_bus.publish(DomainEventEnvelope {
            event_name: "platform_core.module.started".to_string(),
            aggregate_type: "platform_core".to_string(),
            aggregate_id: "core-module".to_string(),
            payload_json: "{\"module\":\"platform-core\"}".to_string(),
            occurred_at_utc_ms: UtcTimestampMs::now().0,
        })?;
        let outbox_topic = resolve_kafka_topic_by_env_key("TOPIC_OUTBOX_EVENTS")?;
        record_outbox_event(new_external_readable_id("evt"), outbox_topic, "queued");
        if std::env::var("FF_CHAIN_ANCHORING")
            .unwrap_or_else(|_| "false".to_string())
            .eq_ignore_ascii_case("true")
        {
            record_chain_receipt(
                new_external_readable_id("receipt"),
                "bootstrap-anchor",
                "pending",
            );
        }
        verify_provider_bindings(ctx).await?;
        Ok(())
    }
}

async fn verify_provider_bindings(ctx: &ModuleContext) -> AppResult<()> {
    if ctx.container.get::<Arc<dyn KycProvider>>().await.is_none() {
        return Err(AppError::Startup("KYC provider not bound".to_string()));
    }
    if ctx
        .container
        .get::<Arc<dyn SigningProvider>>()
        .await
        .is_none()
    {
        return Err(AppError::Startup("Signing provider not bound".to_string()));
    }
    if ctx
        .container
        .get::<Arc<dyn PaymentProvider>>()
        .await
        .is_none()
    {
        return Err(AppError::Startup("Payment provider not bound".to_string()));
    }
    if ctx
        .container
        .get::<Arc<dyn NotificationProvider>>()
        .await
        .is_none()
    {
        return Err(AppError::Startup(
            "Notification provider not bound".to_string(),
        ));
    }
    if ctx
        .container
        .get::<Arc<dyn FabricWriterProvider>>()
        .await
        .is_none()
    {
        return Err(AppError::Startup(
            "Fabric writer provider not bound".to_string(),
        ));
    }
    Ok(())
}

fn read_required_with_default(env_key: &str, default_value: &str) -> AppResult<String> {
    let value = std::env::var(env_key).unwrap_or_else(|_| default_value.to_string());
    if value.trim().is_empty() {
        return Err(AppError::Startup(format!(
            "required startup config is empty: {env_key}"
        )));
    }
    Ok(value)
}

fn load_kafka_topic_catalog() -> AppResult<KafkaTopicCatalog> {
    serde_json::from_str(include_str!("../../../infra/kafka/topics.v1.json"))
        .map_err(|e| AppError::Startup(format!("kafka topic catalog parse failed: {e}")))
}

fn resolve_kafka_topic_by_env_key(env_key: &str) -> AppResult<String> {
    let topic_catalog = load_kafka_topic_catalog()?;
    let topic = topic_catalog
        .topics
        .iter()
        .find(|topic| topic.env_key == env_key)
        .ok_or_else(|| {
            AppError::Startup(format!(
                "kafka topic env key missing from catalog: {env_key}"
            ))
        })?;
    read_required_with_default(&topic.env_key, &topic.name)
}

async fn startup_self_check(cfg: &RuntimeConfig) -> AppResult<()> {
    if cfg.bind_port == 0 {
        return Err(AppError::Startup(
            "bind_port must be greater than zero".to_string(),
        ));
    }
    if matches!(cfg.provider, ProviderMode::Real) && !cfg.feature_flags.enable_real_provider {
        return Err(AppError::Startup(
            "provider mode is real but FF_REAL_PROVIDER is disabled".to_string(),
        ));
    }

    let _check_id = new_external_readable_id("boot");
    let _checked_at = UtcTimestampMs::now();

    let topic_catalog = load_kafka_topic_catalog()?;
    let required_topics: Vec<_> = topic_catalog
        .topics
        .into_iter()
        .filter(|topic| topic.required_in_startup_check)
        .collect();
    let mut resolved_topics = Vec::with_capacity(required_topics.len());
    for topic in required_topics {
        resolved_topics.push(read_required_with_default(&topic.env_key, &topic.name)?);
    }

    let required_buckets = [
        ("BUCKET_RAW_DATA", "raw-data"),
        ("BUCKET_PREVIEW_ARTIFACTS", "preview-artifacts"),
        ("BUCKET_DELIVERY_OBJECTS", "delivery-objects"),
        ("BUCKET_REPORT_RESULTS", "report-results"),
        ("BUCKET_EVIDENCE_PACKAGES", "evidence-packages"),
        ("BUCKET_MODEL_ARTIFACTS", "model-artifacts"),
    ];
    let mut resolved_buckets = Vec::with_capacity(required_buckets.len());
    for (key, default_value) in required_buckets {
        resolved_buckets.push(read_required_with_default(key, default_value)?);
    }

    let required_aliases = [
        ("INDEX_ALIAS_PRODUCT_SEARCH_READ", PRODUCT_SEARCH_READ_ALIAS),
        (
            "INDEX_ALIAS_PRODUCT_SEARCH_WRITE",
            PRODUCT_SEARCH_WRITE_ALIAS,
        ),
        ("INDEX_ALIAS_SELLER_SEARCH_READ", SELLER_SEARCH_READ_ALIAS),
        ("INDEX_ALIAS_SELLER_SEARCH_WRITE", SELLER_SEARCH_WRITE_ALIAS),
    ];
    let mut resolved_aliases = Vec::with_capacity(required_aliases.len());
    for (key, default_value) in required_aliases {
        resolved_aliases.push(read_required_with_default(key, default_value)?);
    }
    let required_indices = [("INDEX_NAME_SEARCH_SYNC_JOBS", SEARCH_SYNC_JOBS_INDEX)];
    let mut resolved_indices = Vec::with_capacity(required_indices.len());
    for (key, default_value) in required_indices {
        resolved_indices.push(read_required_with_default(key, default_value)?);
    }

    verify_kafka_topics_exist(&resolved_topics)?;
    verify_minio_buckets_exist(&resolved_buckets).await?;
    if requires_opensearch_startup_checks(&cfg.mode) {
        verify_opensearch_aliases_exist(&resolved_aliases).await?;
        verify_opensearch_indices_exist(&resolved_indices).await?;
    }

    info!(
        check_id = %new_external_readable_id("boot"),
        checked_at_utc_ms = UtcTimestampMs::now().0,
        mode = %cfg.mode.as_str(),
        provider = %cfg.provider.as_str(),
        ff_demo_features = %cfg.feature_flags.enable_demo_features,
        ff_chain_anchoring = %cfg.feature_flags.enable_chain_anchoring,
        ff_real_provider = %cfg.feature_flags.enable_real_provider,
        ff_sensitive_experiments = %cfg.feature_flags.enable_sensitive_experiments,
        "startup self-check passed"
    );
    Ok(())
}

fn verify_kafka_topics_exist(required_topics: &[String]) -> AppResult<()> {
    let brokers = std::env::var("KAFKA_BROKERS")
        .or_else(|_| std::env::var("KAFKA_BOOTSTRAP_SERVERS"))
        .unwrap_or_else(|_| "127.0.0.1:9094".to_string());
    let consumer: BaseConsumer = rdkafka::ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("group.id", "platform-core-startup-self-check")
        .set("enable.partition.eof", "false")
        .create()
        .map_err(|e| AppError::Startup(format!("kafka client init failed: {e}")))?;
    let metadata = consumer
        .fetch_metadata(None, std::time::Duration::from_secs(3))
        .map_err(|e| AppError::Startup(format!("kafka metadata fetch failed: {e}")))?;
    for topic in required_topics {
        let exists = metadata.topics().iter().any(|t| t.name() == topic.as_str());
        if !exists {
            return Err(AppError::Startup(format!(
                "required kafka topic missing: {topic}"
            )));
        }
    }
    Ok(())
}

async fn verify_minio_buckets_exist(required_buckets: &[String]) -> AppResult<()> {
    let endpoint =
        std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9000".to_string());
    let client = reqwest::Client::new();
    for bucket in required_buckets {
        let url = format!("{}/{}", endpoint.trim_end_matches('/'), bucket);
        let resp = client.head(&url).send().await.map_err(|e| {
            AppError::Startup(format!("minio bucket probe failed for {bucket}: {e}"))
        })?;
        if !(resp.status() == StatusCode::OK || resp.status() == StatusCode::FORBIDDEN) {
            return Err(AppError::Startup(format!(
                "required minio bucket missing or unreachable: {bucket} (status={})",
                resp.status()
            )));
        }
    }
    Ok(())
}

async fn verify_opensearch_aliases_exist(required_aliases: &[String]) -> AppResult<()> {
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9200".to_string());
    let client = reqwest::Client::new();
    for alias in required_aliases {
        let url = format!("{}/_alias/{}", endpoint.trim_end_matches('/'), alias);
        let resp = client.get(&url).send().await.map_err(|e| {
            AppError::Startup(format!("opensearch alias probe failed for {alias}: {e}"))
        })?;
        if resp.status() != StatusCode::OK {
            return Err(AppError::Startup(format!(
                "required opensearch alias missing: {alias} (status={})",
                resp.status()
            )));
        }
    }
    Ok(())
}

async fn verify_opensearch_indices_exist(required_indices: &[String]) -> AppResult<()> {
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9200".to_string());
    let client = reqwest::Client::new();
    for index_name in required_indices {
        let url = format!("{}/{}", endpoint.trim_end_matches('/'), index_name);
        let resp = client.get(&url).send().await.map_err(|e| {
            AppError::Startup(format!(
                "opensearch index probe failed for {index_name}: {e}"
            ))
        })?;
        if resp.status() != StatusCode::OK {
            return Err(AppError::Startup(format!(
                "required opensearch index missing: {index_name} (status={})",
                resp.status()
            )));
        }
    }
    Ok(())
}

fn requires_opensearch_startup_checks(mode: &RuntimeMode) -> bool {
    matches!(mode, RuntimeMode::Staging)
}

pub async fn run() -> AppResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .try_init()
        .ok();

    validate_error_code_document(include_str!("../../../docs/01-architecture/error-codes.md"))?;

    let cfg = RuntimeConfig::from_env()?;
    startup_self_check(&cfg).await?;
    let addr = SocketAddr::new(
        cfg.bind_host
            .parse::<IpAddr>()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        cfg.bind_port,
    );
    let state = AppState {
        runtime: cfg.clone(),
        db: Arc::new(
            AppDb::connect(
                DbPoolConfig {
                    dsn: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                        "postgres://local:local@127.0.0.1:5432/platform".to_string()
                    }),
                    max_connections: 16,
                }
                .into(),
            )
            .await?,
        ),
    };

    let router = with_http_observability(
        build_router::<AppState>(cfg.clone())
            .route("/healthz", axum::routing::get(live_handler))
            .merge(modules::audit::api::router())
            .merge(modules::billing::api::router())
            .merge(modules::catalog::api::router())
            .merge(modules::delivery::api::router())
            .merge(modules::iam::api::router())
            .merge(modules::order::api::router())
            .merge(modules::recommendation::api::router())
            .merge(modules::search::api::router())
            .with_state(state.clone()),
    );

    let mut launcher = AppLauncher::new("platform-core");
    let provider_backend = match cfg.provider {
        ProviderMode::Mock => ProviderBackend::Mock,
        ProviderMode::Real => ProviderBackend::Real,
    };
    launcher
        .registry_mut()
        .register(CoreModule { provider_backend });

    info!(
        "platform-core starting: mode={}, provider={}, addr={}",
        cfg.mode.as_str(),
        cfg.provider.as_str(),
        addr
    );

    launcher
        .run(|| async move { serve(addr, router, tokio::signal::ctrl_c()).await })
        .await
}

#[cfg(test)]
mod tests {
    use super::{requires_opensearch_startup_checks, startup_self_check};
    use config::{FeatureFlags, ProviderMode, RuntimeConfig, RuntimeMode};

    #[test]
    fn opensearch_startup_checks_are_only_required_in_staging() {
        assert!(!requires_opensearch_startup_checks(&RuntimeMode::Local));
        assert!(!requires_opensearch_startup_checks(&RuntimeMode::Demo));
        assert!(requires_opensearch_startup_checks(&RuntimeMode::Staging));
    }

    #[tokio::test]
    async fn startup_self_check_rejects_real_provider_without_flag() {
        let err = startup_self_check(&runtime_config(ProviderMode::Real, false))
            .await
            .expect_err("real provider without FF_REAL_PROVIDER should fail");
        assert!(
            err.to_string()
                .contains("provider mode is real but FF_REAL_PROVIDER is disabled")
        );
    }

    #[tokio::test]
    async fn startup_self_check_accepts_real_provider_with_flag() {
        startup_self_check(&runtime_config(ProviderMode::Real, true))
            .await
            .expect("real provider with FF_REAL_PROVIDER should pass startup self check");
    }

    fn runtime_config(provider: ProviderMode, enable_real_provider: bool) -> RuntimeConfig {
        RuntimeConfig {
            mode: RuntimeMode::Local,
            provider,
            bind_host: "127.0.0.1".to_string(),
            bind_port: 18094,
            service_version: "test".to_string(),
            git_sha: "test".to_string(),
            migration_version: "test".to_string(),
            feature_flags: FeatureFlags {
                enable_demo_features: true,
                enable_chain_anchoring: false,
                enable_real_provider,
                enable_sensitive_experiments: false,
            },
        }
    }
}
