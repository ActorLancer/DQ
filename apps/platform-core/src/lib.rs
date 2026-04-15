use audit_kit::NoopAuditWriter;
use auth::{NoopStepUpGateway, RolePermissionChecker};
use config::{ProviderMode, RuntimeConfig};
use db::{DbPool, DbPoolConfig, TxTemplate};
use http::{ApiResponse, build_router, live_handler, serve};
use kernel::{AppLauncher, AppResult, Module, ModuleContext};
use outbox_kit::NoopOutboxWriter;
use provider_kit::{
    FabricWriterProvider, KycProvider, NotificationProvider, PaymentProvider, ProviderBackend,
    SigningProvider, build_fabric_writer_provider, build_kyc_provider,
    build_notification_provider, build_payment_provider, build_signing_provider,
};
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

        ctx.container.insert(pool).await;
        ctx.container.insert(TxTemplate).await;
        ctx.container.insert(RolePermissionChecker).await;
        ctx.container.insert(NoopStepUpGateway).await;
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
        Ok(())
    }
}

pub async fn run() -> AppResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .try_init()
        .ok();

    let cfg = RuntimeConfig::from_env()?;
    let addr = SocketAddr::new(
        cfg.bind_host
            .parse::<IpAddr>()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        cfg.bind_port,
    );

    let router = build_router()
        .route("/healthz", axum::routing::get(live_handler))
        .route(
            "/internal/runtime",
            axum::routing::get({
                let runtime = cfg.clone();
                move || {
                    let runtime = runtime.clone();
                    async move { ApiResponse::ok(runtime) }
                }
            }),
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
