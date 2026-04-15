use async_trait::async_trait;
use kernel::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct DbPoolConfig {
    pub dsn: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct DbPool {
    pub dsn: String,
    pub max_connections: u32,
}

impl DbPool {
    pub fn connect(cfg: DbPoolConfig) -> AppResult<Self> {
        if cfg.dsn.trim().is_empty() {
            return Err(AppError::Config("database dsn cannot be empty".to_string()));
        }
        Ok(Self {
            dsn: cfg.dsn,
            max_connections: cfg.max_connections.max(1),
        })
    }
}

#[async_trait]
pub trait MigrationRunner: Send + Sync {
    async fn run_migrations(&self, _pool: &DbPool) -> AppResult<()>;
}

#[async_trait]
pub trait ReadOnlyExecutor: Send + Sync {
    async fn execute_read_only<T, F>(&self, op: F) -> AppResult<T>
    where
        T: Send + 'static,
        F: FnOnce() -> AppResult<T> + Send + 'static;
}

#[async_trait]
pub trait WriteExecutor: Send + Sync {
    async fn execute_in_tx<T, F>(&self, op: F) -> AppResult<T>
    where
        T: Send + 'static,
        F: FnOnce() -> AppResult<T> + Send + 'static;
}

#[derive(Debug, Default, Clone)]
pub struct TxTemplate;

#[async_trait]
impl ReadOnlyExecutor for TxTemplate {
    async fn execute_read_only<T, F>(&self, op: F) -> AppResult<T>
    where
        T: Send + 'static,
        F: FnOnce() -> AppResult<T> + Send + 'static,
    {
        op()
    }
}

#[async_trait]
impl WriteExecutor for TxTemplate {
    async fn execute_in_tx<T, F>(&self, op: F) -> AppResult<T>
    where
        T: Send + 'static,
        F: FnOnce() -> AppResult<T> + Send + 'static,
    {
        op()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tx_template_executes() {
        let tx = TxTemplate;
        let value = tx
            .execute_in_tx(|| Ok::<_, AppError>(42))
            .await
            .expect("tx should execute");
        assert_eq!(value, 42);
    }
}
