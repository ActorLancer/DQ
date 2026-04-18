use audit_kit::NoopAuditWriter;
use auth::{
    AuthorizationFacade, MockJwtParser, NoopStepUpGateway, RolePermissionChecker,
    UnifiedAuthorizationFacade,
};
use config::{ProviderMode, RuntimeConfig};
use db::{
    DbPool, DbPoolConfig, NoopBusinessMutationWriter, OrderRepository, OrderRepositoryBackend,
    TxTemplate, build_order_repository,
};
use http::{build_router, live_handler, record_chain_receipt, record_outbox_event, serve};
use kernel::{
    AppError, AppLauncher, AppResult, DomainEventEnvelope, InProcessEventBus, Module,
    ModuleContext, UtcTimestampMs, new_external_readable_id, validate_error_code_document,
};
use outbox_kit::NoopOutboxWriter;
use provider_kit::{
    FabricWriterProvider, KycProvider, NotificationProvider, PaymentProvider, ProviderBackend,
    SigningProvider, build_fabric_writer_provider, build_kyc_provider, build_notification_provider,
    build_payment_provider, build_signing_provider,
};
use rdkafka::consumer::{BaseConsumer, Consumer};
use reqwest::StatusCode;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::info;

mod app;
mod modules;
mod shared;

struct CoreModule {
    provider_backend: ProviderBackend,
}

#[async_trait::async_trait]
impl Module for CoreModule {
    fn name(&self) -> &'static str {
        "platform-core"
    }

    async fn start(&self, ctx: &ModuleContext) -> AppResult<()> {
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://local:local@localhost:5432/platform".to_string());
        let pool = DbPool::connect(DbPoolConfig {
            dsn,
            max_connections: 16,
        })?;
        let order_repo_backend = OrderRepositoryBackend::from_env()?;
        let order_repository = build_order_repository(&pool, order_repo_backend);

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
        ctx.container.insert(NoopAuditWriter).await;
        ctx.container.insert(NoopOutboxWriter).await;
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
        let outbox_topic =
            std::env::var("TOPIC_OUTBOX_EVENTS").unwrap_or_else(|_| "outbox.events".to_string());
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

    let required_topics = [
        ("TOPIC_OUTBOX_EVENTS", "outbox.events"),
        ("TOPIC_SEARCH_SYNC", "search.sync"),
        ("TOPIC_AUDIT_ANCHOR", "audit.anchor"),
        ("TOPIC_BILLING_EVENTS", "billing.events"),
        ("TOPIC_RECOMMENDATION_BEHAVIOR", "recommendation.behavior"),
        ("TOPIC_DEAD_LETTER_EVENTS", "dead-letter.events"),
    ];
    let mut resolved_topics = Vec::with_capacity(required_topics.len());
    for (key, default_value) in required_topics {
        resolved_topics.push(read_required_with_default(key, default_value)?);
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
        ("INDEX_ALIAS_CATALOG_PRODUCTS", "catalog_products_v1"),
        ("INDEX_ALIAS_SELLER_PROFILES", "seller_profiles_v1"),
        ("INDEX_ALIAS_SEARCH_SYNC_JOBS", "search_sync_jobs_v1"),
    ];
    let mut resolved_aliases = Vec::with_capacity(required_aliases.len());
    for (key, default_value) in required_aliases {
        resolved_aliases.push(read_required_with_default(key, default_value)?);
    }

    verify_kafka_topics_exist(&resolved_topics)?;
    verify_minio_buckets_exist(&resolved_buckets).await?;
    verify_opensearch_aliases_exist(&resolved_aliases).await?;

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
        .unwrap_or_else(|_| "localhost:9092".to_string());
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
        std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
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

    let router = build_router(cfg.clone())
        .route("/healthz", axum::routing::get(live_handler))
        .merge(modules::billing::api::router())
        .merge(modules::catalog::api::router())
        .merge(modules::iam::api::router())
        .merge(modules::order::api::router());

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
