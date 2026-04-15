use kernel::{AppError, AppResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeMode {
    Local,
    Staging,
    Demo,
}

impl RuntimeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuntimeMode::Local => "local",
            RuntimeMode::Staging => "staging",
            RuntimeMode::Demo => "demo",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderMode {
    Mock,
    Real,
}

impl ProviderMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderMode::Mock => "mock",
            ProviderMode::Real => "real",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeConfig {
    pub mode: RuntimeMode,
    pub provider: ProviderMode,
    pub bind_host: String,
    pub bind_port: u16,
    pub service_version: String,
    pub git_sha: String,
    pub migration_version: String,
}

impl RuntimeConfig {
    pub fn from_env() -> AppResult<Self> {
        let mode = match std::env::var("APP_MODE")
            .unwrap_or_else(|_| "local".to_string())
            .as_str()
        {
            "local" => RuntimeMode::Local,
            "staging" => RuntimeMode::Staging,
            "demo" => RuntimeMode::Demo,
            value => {
                return Err(AppError::Config(format!(
                    "APP_MODE must be one of local/staging/demo, got {value}"
                )));
            }
        };

        let provider = match std::env::var("PROVIDER_MODE")
            .unwrap_or_else(|_| "mock".to_string())
            .as_str()
        {
            "mock" => ProviderMode::Mock,
            "real" => ProviderMode::Real,
            value => {
                return Err(AppError::Config(format!(
                    "PROVIDER_MODE must be one of mock/real, got {value}"
                )));
            }
        };

        let bind_host = std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let bind_port = std::env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| AppError::Config(format!("APP_PORT parse error: {e}")))?;
        let service_version =
            std::env::var("APP_VERSION").unwrap_or_else(|_| "0.1.0-dev".to_string());
        let git_sha = std::env::var("GIT_SHA").unwrap_or_else(|_| "unknown".to_string());
        let migration_version =
            std::env::var("MIGRATION_VERSION").unwrap_or_else(|_| "unapplied".to_string());

        Ok(Self {
            mode,
            provider,
            bind_host,
            bind_port,
            service_version,
            git_sha,
            migration_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_local() {
        let cfg = RuntimeConfig::from_env().expect("default config should load");
        assert_eq!(cfg.mode, RuntimeMode::Local);
        assert_eq!(cfg.provider, ProviderMode::Mock);
        assert_eq!(cfg.service_version, "0.1.0-dev");
        assert_eq!(cfg.git_sha, "unknown");
        assert_eq!(cfg.migration_version, "unapplied");
    }
}
