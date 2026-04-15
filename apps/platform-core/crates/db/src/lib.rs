use async_trait::async_trait;
use audit_kit::{AuditEvent, AuditWriter};
use kernel::{AppError, AppResult};
use outbox_kit::{EventEnvelope, OutboxWriter};
use std::sync::Arc;

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

#[derive(Debug, Default, Clone)]
pub struct NoopBusinessMutationWriter;

#[async_trait]
impl BusinessMutationWriter for NoopBusinessMutationWriter {
    async fn apply_mutation(&self, _mutation: BusinessMutation) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use audit_kit::AuditContext;
    use outbox_kit::{NoopOutboxWriter, PublishStatus, RetryPolicy};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use audit_kit::NoopAuditWriter;

    #[tokio::test]
    async fn tx_template_executes() {
        let tx = TxTemplate;
        let value = tx
            .execute_in_tx(|| Ok::<_, AppError>(42))
            .await
            .expect("tx should execute");
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn tx_bundle_commits() {
        let tx = TxTemplate;
        let phase = tx
            .execute_business_audit_outbox(
                Arc::new(NoopBusinessMutationWriter),
                Arc::new(NoopAuditWriter),
                Arc::new(NoopOutboxWriter),
                sample_bundle(),
            )
            .await
            .expect("bundle should commit");
        assert_eq!(phase, TxPhase::Committed);
    }

    #[derive(Default)]
    struct FailingMutationWriter;

    #[async_trait]
    impl BusinessMutationWriter for FailingMutationWriter {
        async fn apply_mutation(&self, _mutation: BusinessMutation) -> AppResult<()> {
            Err(AppError::Config("business write failed".to_string()))
        }
    }

    #[derive(Default)]
    struct CountingLifecycle {
        begin: AtomicUsize,
        commit: AtomicUsize,
        rollback: AtomicUsize,
    }

    #[async_trait]
    impl TxLifecycleHook for CountingLifecycle {
        async fn on_begin(&self) -> AppResult<()> {
            self.begin.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
        async fn on_commit(&self) -> AppResult<()> {
            self.commit.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
        async fn on_rollback(&self) -> AppResult<()> {
            self.rollback.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    #[tokio::test]
    async fn tx_bundle_rolls_back_on_failure() {
        let tx = TxTemplate;
        let lifecycle = Arc::new(CountingLifecycle::default());
        let result = tx
            .execute_with_lifecycle(
                Arc::new(FailingMutationWriter),
                Arc::new(NoopAuditWriter),
                Arc::new(NoopOutboxWriter),
                lifecycle.clone(),
                sample_bundle(),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(lifecycle.begin.load(Ordering::Relaxed), 1);
        assert_eq!(lifecycle.commit.load(Ordering::Relaxed), 0);
        assert_eq!(lifecycle.rollback.load(Ordering::Relaxed), 1);
    }

    fn sample_bundle() -> TransactionBundle {
        TransactionBundle {
            business_mutations: vec![BusinessMutation {
                aggregate_type: "order".to_string(),
                aggregate_id: "ord-1".to_string(),
                operation: "create".to_string(),
                payload_json: "{}".to_string(),
            }],
            audit_events: vec![AuditEvent {
                action: "order.create".to_string(),
                object_type: "order".to_string(),
                object_id: "ord-1".to_string(),
                result: "success".to_string(),
                context: AuditContext {
                    request_id: "req-1".to_string(),
                    actor_id: "user-1".to_string(),
                    tenant_id: "tenant-1".to_string(),
                },
                evidence: vec![],
            }],
            outbox_events: vec![EventEnvelope {
                event_id: "evt-1".to_string(),
                topic: "trade.order.created".to_string(),
                aggregate_type: "order".to_string(),
                aggregate_id: "ord-1".to_string(),
                payload_json: "{}".to_string(),
                idempotency_key: "idem-1".to_string(),
                status: PublishStatus::Pending,
                retry_policy: RetryPolicy::default(),
            }],
        }
    }
}
