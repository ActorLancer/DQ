use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

#[cfg(test)]
mod tests;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    IamUnauthorized,
    CatValidationFailed,
    TrdStateConflict,
    DlvAccessDenied,
    BilProviderFailed,
    AudEvidenceInvalid,
    OpsCoreConfig,
    OpsCoreStartup,
    OpsCoreShutdown,
    OpsInternal,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::IamUnauthorized => "IAM_UNAUTHORIZED",
            ErrorCode::CatValidationFailed => "CAT_VALIDATION_FAILED",
            ErrorCode::TrdStateConflict => "TRD_STATE_CONFLICT",
            ErrorCode::DlvAccessDenied => "DLV_ACCESS_DENIED",
            ErrorCode::BilProviderFailed => "BIL_PROVIDER_FAILED",
            ErrorCode::AudEvidenceInvalid => "AUD_EVIDENCE_INVALID",
            ErrorCode::OpsCoreConfig => "OPS_CORE_CONFIG",
            ErrorCode::OpsCoreStartup => "OPS_CORE_STARTUP",
            ErrorCode::OpsCoreShutdown => "OPS_CORE_SHUTDOWN",
            ErrorCode::OpsInternal => "OPS_INTERNAL",
        }
    }

    pub fn prefix(self) -> &'static str {
        self.as_str()
            .split('_')
            .next()
            .expect("error code must have prefix")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("startup error: {0}")]
    Startup(String),
    #[error("shutdown error: {0}")]
    Shutdown(String),
}

impl AppError {
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::Config(_) => ErrorCode::OpsCoreConfig,
            AppError::Startup(_) => ErrorCode::OpsCoreStartup,
            AppError::Shutdown(_) => ErrorCode::OpsCoreShutdown,
        }
    }

    pub fn message(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn from_error(error: &AppError, request_id: Option<String>) -> Self {
        Self {
            code: error.code().as_str().to_string(),
            message: error.message(),
            request_id,
        }
    }
}

pub fn validate_error_code_document(doc: &str) -> AppResult<()> {
    let required_prefixes = ["IAM_", "CAT_", "TRD_", "DLV_", "BIL_", "AUD_", "OPS_"];
    for prefix in required_prefixes {
        if !doc.contains(prefix) {
            return Err(AppError::Config(format!(
                "error-codes.md missing prefix section: {prefix}"
            )));
        }
    }

    for code in [
        ErrorCode::IamUnauthorized,
        ErrorCode::CatValidationFailed,
        ErrorCode::TrdStateConflict,
        ErrorCode::DlvAccessDenied,
        ErrorCode::BilProviderFailed,
        ErrorCode::AudEvidenceInvalid,
        ErrorCode::OpsCoreConfig,
        ErrorCode::OpsCoreStartup,
        ErrorCode::OpsCoreShutdown,
        ErrorCode::OpsInternal,
    ] {
        let prefix = format!("{}_", code.prefix());
        if !doc.contains(&prefix) {
            return Err(AppError::Config(format!(
                "error-codes.md missing prefix for code {}",
                code.as_str()
            )));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtcTimestampMs(pub i64);

impl UtcTimestampMs {
    pub fn now() -> Self {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        Self(ms)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(raw: &str) -> AppResult<Self> {
        Uuid::parse_str(raw)
            .map(Self)
            .map_err(|e| AppError::Config(format!("invalid uuid entity id: {e}")))
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub fn new_uuid_string() -> String {
    Uuid::new_v4().to_string()
}

pub fn new_external_readable_id(prefix: &str) -> String {
    let safe_prefix = prefix.trim().to_ascii_uppercase();
    let safe_prefix = if safe_prefix.is_empty() {
        "OBJ".to_string()
    } else {
        safe_prefix
    };
    let ts_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let suffix = &Uuid::new_v4().simple().to_string()[..8];
    format!("{safe_prefix}-{ts_seconds}-{suffix}")
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
}

impl Pagination {
    pub fn from_query(query: Option<PaginationQuery>) -> Self {
        let page = query.as_ref().and_then(|q| q.page).unwrap_or(1).max(1);
        let page_size = query
            .as_ref()
            .and_then(|q| q.page_size)
            .unwrap_or(20)
            .clamp(1, 200);
        Self { page, page_size }
    }

    pub fn offset(&self) -> u64 {
        ((self.page - 1) as u64) * self.page_size as u64
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct FilterQuery {
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListQuery {
    pub pagination: Pagination,
    pub filter: FilterQuery,
}

impl ListQuery {
    pub fn new(pagination: Option<PaginationQuery>, filter: Option<FilterQuery>) -> Self {
        Self {
            pagination: Pagination::from_query(pagination),
            filter: filter.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainEventEnvelope {
    pub event_name: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub payload_json: String,
    pub occurred_at_utc_ms: i64,
}

#[derive(Clone)]
pub struct InProcessEventBus {
    sender: broadcast::Sender<DomainEventEnvelope>,
}

impl InProcessEventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(1));
        Self { sender }
    }

    pub fn publish(&self, event: DomainEventEnvelope) -> AppResult<()> {
        let _ = self.sender.send(event);
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEventEnvelope> {
        self.sender.subscribe()
    }
}

impl Default for InProcessEventBus {
    fn default() -> Self {
        Self::new(128)
    }
}

#[derive(Clone, Default)]
pub struct ServiceContainer {
    services: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl ServiceContainer {
    pub async fn insert<T>(&self, value: T)
    where
        T: Send + Sync + 'static,
    {
        self.services
            .write()
            .await
            .insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub async fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .read()
            .await
            .get(&TypeId::of::<T>())
            .and_then(|item| item.clone().downcast::<T>().ok())
    }
}

#[derive(Clone)]
pub struct ModuleContext {
    pub app_name: &'static str,
    pub container: ServiceContainer,
}

#[async_trait]
pub trait Module: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self, _ctx: &ModuleContext) -> AppResult<()> {
        Ok(())
    }
    async fn shutdown(&self, _ctx: &ModuleContext) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct ModuleRegistry {
    modules: Vec<Arc<dyn Module>>,
}

impl ModuleRegistry {
    pub fn register<M>(&mut self, module: M)
    where
        M: Module + 'static,
    {
        self.modules.push(Arc::new(module));
    }

    pub fn modules(&self) -> &[Arc<dyn Module>] {
        &self.modules
    }
}

type Hook = Arc<dyn Fn() + Send + Sync>;

#[derive(Default, Clone)]
pub struct LifecycleHooks {
    pub before_start: Vec<Hook>,
    pub before_shutdown: Vec<Hook>,
}

impl LifecycleHooks {
    pub fn on_before_start<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.before_start.push(Arc::new(callback));
    }

    pub fn on_before_shutdown<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.before_shutdown.push(Arc::new(callback));
    }
}

pub struct AppLauncher {
    pub app_name: &'static str,
    pub registry: ModuleRegistry,
    pub container: ServiceContainer,
    pub hooks: LifecycleHooks,
}

impl AppLauncher {
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name,
            registry: ModuleRegistry::default(),
            container: ServiceContainer::default(),
            hooks: LifecycleHooks::default(),
        }
    }

    pub fn registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.registry
    }

    pub fn hooks_mut(&mut self) -> &mut LifecycleHooks {
        &mut self.hooks
    }

    pub async fn run<F, Fut>(&self, serve: F) -> AppResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = AppResult<()>>,
    {
        for hook in &self.hooks.before_start {
            hook();
        }

        let ctx = ModuleContext {
            app_name: self.app_name,
            container: self.container.clone(),
        };

        for module in self.registry.modules() {
            module.start(&ctx).await?;
        }

        serve().await?;

        for hook in &self.hooks.before_shutdown {
            hook();
        }

        for module in self.registry.modules().iter().rev() {
            module.shutdown(&ctx).await?;
        }
        Ok(())
    }
}
