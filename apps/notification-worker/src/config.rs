use config::{FeatureFlags, ProviderMode, RuntimeConfig, RuntimeMode};
use kernel::{AppError, AppResult};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NotificationWorkerConfig {
    pub runtime: RuntimeConfig,
    pub database_url: String,
    pub redis_url: String,
    pub redis_namespace: String,
    pub kafka_brokers: String,
    pub topic: String,
    pub consumer_group: String,
    pub template_dir: PathBuf,
    pub retry_poll_interval_ms: u64,
    pub retry_backoff_ms: u64,
    pub retry_max_attempts: u32,
}

impl NotificationWorkerConfig {
    pub fn from_env() -> AppResult<Self> {
        let runtime = RuntimeConfig {
            mode: parse_runtime_mode()?,
            provider: parse_provider_mode()?,
            bind_host: std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            bind_port: parse_u16_env("APP_PORT", 8097)?,
            service_version: std::env::var("APP_VERSION")
                .unwrap_or_else(|_| "0.1.0-dev".to_string()),
            git_sha: std::env::var("GIT_SHA").unwrap_or_else(|_| "unknown".to_string()),
            migration_version: std::env::var("MIGRATION_VERSION")
                .unwrap_or_else(|_| "074".to_string()),
            feature_flags: FeatureFlags::from_env()?,
        };

        Ok(Self {
            runtime,
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string()
            }),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://:datab_redis_pass@127.0.0.1:6379/2".to_string()),
            redis_namespace: std::env::var("REDIS_NAMESPACE")
                .unwrap_or_else(|_| "datab:v1".to_string()),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .or_else(|_| std::env::var("KAFKA_BOOTSTRAP_SERVERS"))
                .unwrap_or_else(|_| "127.0.0.1:9094".to_string()),
            topic: std::env::var("NOTIFICATION_TOPIC")
                .unwrap_or_else(|_| "dtp.notification.dispatch".to_string()),
            consumer_group: std::env::var("NOTIFICATION_WORKER_CONSUMER_GROUP")
                .unwrap_or_else(|_| "cg-notification-worker".to_string()),
            template_dir: std::env::var("NOTIFICATION_TEMPLATE_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("apps/notification-worker/templates")),
            retry_poll_interval_ms: parse_u64_env("NOTIFICATION_RETRY_POLL_INTERVAL_MS", 500)?,
            retry_backoff_ms: parse_u64_env("NOTIFICATION_RETRY_BACKOFF_MS", 250)?,
            retry_max_attempts: parse_u32_env("NOTIFICATION_RETRY_MAX_ATTEMPTS", 3)?,
        })
    }
}

fn parse_runtime_mode() -> AppResult<RuntimeMode> {
    match std::env::var("APP_MODE")
        .unwrap_or_else(|_| "local".to_string())
        .as_str()
    {
        "local" => Ok(RuntimeMode::Local),
        "staging" => Ok(RuntimeMode::Staging),
        "demo" => Ok(RuntimeMode::Demo),
        other => Err(AppError::Config(format!(
            "APP_MODE must be one of local/staging/demo, got {other}"
        ))),
    }
}

fn parse_provider_mode() -> AppResult<ProviderMode> {
    match std::env::var("PROVIDER_MODE")
        .unwrap_or_else(|_| "mock".to_string())
        .as_str()
    {
        "mock" => Ok(ProviderMode::Mock),
        "real" => Ok(ProviderMode::Real),
        other => Err(AppError::Config(format!(
            "PROVIDER_MODE must be one of mock/real, got {other}"
        ))),
    }
}

fn parse_u16_env(key: &str, default_value: u16) -> AppResult<u16> {
    std::env::var(key)
        .unwrap_or_else(|_| default_value.to_string())
        .parse::<u16>()
        .map_err(|err| AppError::Config(format!("{key} parse error: {err}")))
}

fn parse_u32_env(key: &str, default_value: u32) -> AppResult<u32> {
    std::env::var(key)
        .unwrap_or_else(|_| default_value.to_string())
        .parse::<u32>()
        .map_err(|err| AppError::Config(format!("{key} parse error: {err}")))
}

fn parse_u64_env(key: &str, default_value: u64) -> AppResult<u64> {
    std::env::var(key)
        .unwrap_or_else(|_| default_value.to_string())
        .parse::<u64>()
        .map_err(|err| AppError::Config(format!("{key} parse error: {err}")))
}
