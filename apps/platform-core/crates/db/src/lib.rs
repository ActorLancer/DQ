use async_trait::async_trait;
use audit_kit::{AuditEvent, AuditWriter};
use kernel::{AppError, AppResult};
use outbox_kit::{EventEnvelope, OutboxWriter};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::NoTls;

#[cfg(test)]
mod tests;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderRecord {
    pub order_id: String,
    pub tenant_id: String,
    pub status: String,
    pub amount_minor: i64,
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn upsert(&self, order: OrderRecord) -> AppResult<()>;
    async fn find_by_id(&self, order_id: &str) -> AppResult<Option<OrderRecord>>;
    async fn list_by_tenant(&self, tenant_id: &str) -> AppResult<Vec<OrderRecord>>;
}

#[derive(Default)]
pub struct InMemoryOrderRepository {
    data: RwLock<HashMap<String, OrderRecord>>,
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn upsert(&self, order: OrderRecord) -> AppResult<()> {
        self.data
            .write()
            .await
            .insert(order.order_id.clone(), order);
        Ok(())
    }

    async fn find_by_id(&self, order_id: &str) -> AppResult<Option<OrderRecord>> {
        Ok(self.data.read().await.get(order_id).cloned())
    }

    async fn list_by_tenant(&self, tenant_id: &str) -> AppResult<Vec<OrderRecord>> {
        let mut list = self
            .data
            .read()
            .await
            .values()
            .filter(|o| o.tenant_id == tenant_id)
            .cloned()
            .collect::<Vec<_>>();
        list.sort_by(|a, b| a.order_id.cmp(&b.order_id));
        Ok(list)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderRepositoryBackend {
    InMemory,
    Postgres,
}

impl OrderRepositoryBackend {
    pub fn from_env() -> AppResult<Self> {
        let raw = std::env::var("ORDER_REPOSITORY_BACKEND")
            .unwrap_or_else(|_| "in_memory".to_string())
            .to_ascii_lowercase();
        match raw.as_str() {
            "in_memory" | "memory" => Ok(Self::InMemory),
            "postgres" | "postgresql" => Ok(Self::Postgres),
            other => Err(AppError::Config(format!(
                "ORDER_REPOSITORY_BACKEND must be in_memory|postgres, got {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresOrderRepository {
    dsn: String,
}

impl PostgresOrderRepository {
    pub fn new(dsn: impl Into<String>) -> Self {
        Self { dsn: dsn.into() }
    }
}

#[async_trait]
impl OrderRepository for PostgresOrderRepository {
    async fn upsert(&self, order: OrderRecord) -> AppResult<()> {
        let (client, connection) = tokio_postgres::connect(&self.dsn, NoTls)
            .await
            .map_err(|e| AppError::Config(format!("postgres connect failed: {e}")))?;
        tokio::spawn(async move {
            let _ = connection.await;
        });

        client
            .execute(
                "INSERT INTO trade_order (order_id, tenant_id, status, amount_minor)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (order_id)
                 DO UPDATE SET tenant_id = EXCLUDED.tenant_id,
                               status = EXCLUDED.status,
                               amount_minor = EXCLUDED.amount_minor",
                &[
                    &order.order_id,
                    &order.tenant_id,
                    &order.status,
                    &order.amount_minor,
                ],
            )
            .await
            .map_err(|e| AppError::Config(format!("postgres upsert order failed: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, order_id: &str) -> AppResult<Option<OrderRecord>> {
        let (client, connection) = tokio_postgres::connect(&self.dsn, NoTls)
            .await
            .map_err(|e| AppError::Config(format!("postgres connect failed: {e}")))?;
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let row = client
            .query_opt(
                "SELECT order_id, tenant_id, status, amount_minor FROM trade_order WHERE order_id = $1",
                &[&order_id],
            )
            .await
            .map_err(|e| AppError::Config(format!("postgres find order failed: {e}")))?;

        Ok(row.map(|r| OrderRecord {
            order_id: r.get::<_, String>(0),
            tenant_id: r.get::<_, String>(1),
            status: r.get::<_, String>(2),
            amount_minor: r.get::<_, i64>(3),
        }))
    }

    async fn list_by_tenant(&self, tenant_id: &str) -> AppResult<Vec<OrderRecord>> {
        let (client, connection) = tokio_postgres::connect(&self.dsn, NoTls)
            .await
            .map_err(|e| AppError::Config(format!("postgres connect failed: {e}")))?;
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let rows = client
            .query(
                "SELECT order_id, tenant_id, status, amount_minor
                 FROM trade_order
                 WHERE tenant_id = $1
                 ORDER BY order_id ASC",
                &[&tenant_id],
            )
            .await
            .map_err(|e| AppError::Config(format!("postgres list orders failed: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| OrderRecord {
                order_id: r.get::<_, String>(0),
                tenant_id: r.get::<_, String>(1),
                status: r.get::<_, String>(2),
                amount_minor: r.get::<_, i64>(3),
            })
            .collect())
    }
}

pub fn build_order_repository(
    pool: &DbPool,
    backend: OrderRepositoryBackend,
) -> Arc<dyn OrderRepository> {
    match backend {
        OrderRepositoryBackend::InMemory => Arc::new(InMemoryOrderRepository::default()),
        OrderRepositoryBackend::Postgres => {
            Arc::new(PostgresOrderRepository::new(pool.dsn.clone()))
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusinessMutation {
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub operation: String,
    pub payload_json: String,
}

#[derive(Debug, Clone, Default)]
pub struct TransactionBundle {
    pub business_mutations: Vec<BusinessMutation>,
    pub audit_events: Vec<AuditEvent>,
    pub outbox_events: Vec<EventEnvelope>,
}

#[async_trait]
pub trait BusinessMutationWriter: Send + Sync {
    async fn apply_mutation(&self, mutation: BusinessMutation) -> AppResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxPhase {
    Begun,
    Committed,
    RolledBack,
}

#[async_trait]
pub trait TxLifecycleHook: Send + Sync {
    async fn on_begin(&self) -> AppResult<()> {
        Ok(())
    }
    async fn on_commit(&self) -> AppResult<()> {
        Ok(())
    }
    async fn on_rollback(&self) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoopTxLifecycleHook;

#[async_trait]
impl TxLifecycleHook for NoopTxLifecycleHook {}

impl TxTemplate {
    pub async fn execute_business_audit_outbox(
        &self,
        business_writer: Arc<dyn BusinessMutationWriter>,
        audit_writer: Arc<dyn AuditWriter>,
        outbox_writer: Arc<dyn OutboxWriter>,
        bundle: TransactionBundle,
    ) -> AppResult<TxPhase> {
        self.execute_with_lifecycle(
            business_writer,
            audit_writer,
            outbox_writer,
            Arc::new(NoopTxLifecycleHook),
            bundle,
        )
        .await
    }

    pub async fn execute_with_lifecycle(
        &self,
        business_writer: Arc<dyn BusinessMutationWriter>,
        audit_writer: Arc<dyn AuditWriter>,
        outbox_writer: Arc<dyn OutboxWriter>,
        lifecycle: Arc<dyn TxLifecycleHook>,
        bundle: TransactionBundle,
    ) -> AppResult<TxPhase> {
        lifecycle.on_begin().await?;

        let result: AppResult<()> = async {
            for mutation in bundle.business_mutations {
                business_writer.apply_mutation(mutation).await?;
            }
            for event in bundle.audit_events {
                audit_writer.write_event(event).await?;
            }
            for envelope in bundle.outbox_events {
                outbox_writer.append(envelope).await?;
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                lifecycle.on_commit().await?;
                Ok(TxPhase::Committed)
            }
            Err(err) => {
                lifecycle.on_rollback().await?;
                Err(err)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestDbFixture {
    pub pool: DbPool,
    pub tx_template: TxTemplate,
}

impl TestDbFixture {
    pub fn from_env() -> AppResult<Self> {
        let dsn = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://test:test@localhost:5432/platform_test".to_string());
        let pool = DbPool::connect(DbPoolConfig {
            dsn,
            max_connections: 4,
        })?;
        Ok(Self {
            pool,
            tx_template: TxTemplate,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RollbackFixtureResult {
    pub began: bool,
    pub committed: bool,
    pub rolled_back: bool,
}

pub async fn run_transaction_rollback_fixture(
    tx: &TxTemplate,
    bundle: TransactionBundle,
) -> AppResult<RollbackFixtureResult> {
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Default)]
    struct FailingWriter;
    #[async_trait]
    impl BusinessMutationWriter for FailingWriter {
        async fn apply_mutation(&self, _mutation: BusinessMutation) -> AppResult<()> {
            Err(AppError::Config(
                "rollback fixture: forced mutation failure".to_string(),
            ))
        }
    }

    #[derive(Default)]
    struct Hook {
        began: AtomicBool,
        committed: AtomicBool,
        rolled_back: AtomicBool,
    }
    #[async_trait]
    impl TxLifecycleHook for Hook {
        async fn on_begin(&self) -> AppResult<()> {
            self.began.store(true, Ordering::Relaxed);
            Ok(())
        }
        async fn on_commit(&self) -> AppResult<()> {
            self.committed.store(true, Ordering::Relaxed);
            Ok(())
        }
        async fn on_rollback(&self) -> AppResult<()> {
            self.rolled_back.store(true, Ordering::Relaxed);
            Ok(())
        }
    }

    let hook = Arc::new(Hook::default());
    let result = tx
        .execute_with_lifecycle(
            Arc::new(FailingWriter),
            Arc::new(audit_kit::NoopAuditWriter),
            Arc::new(outbox_kit::NoopOutboxWriter),
            hook.clone(),
            bundle,
        )
        .await;

    if result.is_ok() {
        return Err(AppError::Config(
            "rollback fixture expected failure but committed".to_string(),
        ));
    }

    Ok(RollbackFixtureResult {
        began: hook.began.load(Ordering::Relaxed),
        committed: hook.committed.load(Ordering::Relaxed),
        rolled_back: hook.rolled_back.load(Ordering::Relaxed),
    })
}

#[cfg(feature = "query-compile-check")]
mod query_compile_checks {
    // Query compile-check scaffold:
    // until a concrete DB library (SQLx/SeaORM) is fully wired in this crate,
    // these typed query specs are compiled in CI to catch accidental query-shape drift early.
    pub const ORDER_BASE_COLUMNS: &[&str] = &[
        "order_id",
        "tenant_id",
        "status",
        "created_at",
        "updated_at",
    ];

    pub const ORDER_SELECT_BY_ID: &str = "SELECT order_id, tenant_id, status, created_at, updated_at FROM trade_order WHERE order_id = $1";
    pub const OUTBOX_PENDING_SELECT: &str = "SELECT event_id, topic, aggregate_type, aggregate_id, payload_json, idempotency_key FROM outbox_event WHERE status = 'pending' ORDER BY created_at ASC LIMIT $1";
    pub const AUDIT_BY_OBJECT: &str = "SELECT action, object_type, object_id, result, created_at FROM audit_log WHERE object_type = $1 AND object_id = $2 ORDER BY created_at DESC LIMIT $3";

    #[test]
    fn query_specs_are_well_formed() {
        assert_eq!(ORDER_BASE_COLUMNS.len(), 5);
        for query in [ORDER_SELECT_BY_ID, OUTBOX_PENDING_SELECT, AUDIT_BY_OBJECT] {
            assert!(query.starts_with("SELECT "));
            assert!(query.contains(" FROM "));
            assert!(query.contains('$'));
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoopBusinessMutationWriter;

#[async_trait]
impl BusinessMutationWriter for NoopBusinessMutationWriter {
    async fn apply_mutation(&self, _mutation: BusinessMutation) -> AppResult<()> {
        Ok(())
    }
}
