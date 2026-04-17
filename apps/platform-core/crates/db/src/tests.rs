#[cfg(test)]
mod tests {
    use crate::*;
    use audit_kit::AuditContext;
    use audit_kit::NoopAuditWriter;
    use outbox_kit::{NoopOutboxWriter, PublishStatus, RetryPolicy};
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    #[tokio::test]
    async fn rollback_fixture_reports_rollback_state() {
        let tx = TxTemplate;
        let result = run_transaction_rollback_fixture(&tx, sample_bundle())
            .await
            .expect("fixture should report rollback");
        assert!(result.began);
        assert!(!result.committed);
        assert!(result.rolled_back);
    }

    #[test]
    fn test_db_fixture_provides_pool_and_template() {
        let fixture = TestDbFixture::from_env().expect("fixture should build");
        assert!(!fixture.pool.dsn.is_empty());
        assert!(fixture.pool.max_connections >= 1);
    }

    #[tokio::test]
    async fn in_memory_order_repository_supports_rule_tests() {
        let repo = InMemoryOrderRepository::default();
        repo.upsert(OrderRecord {
            order_id: "ord-2".to_string(),
            tenant_id: "t-1".to_string(),
            status: "draft".to_string(),
            amount_minor: 100,
        })
        .await
        .expect("insert ord-2");
        repo.upsert(OrderRecord {
            order_id: "ord-1".to_string(),
            tenant_id: "t-1".to_string(),
            status: "paid".to_string(),
            amount_minor: 200,
        })
        .await
        .expect("insert ord-1");

        let found = repo.find_by_id("ord-1").await.expect("find by id");
        assert_eq!(found.expect("order exists").status, "paid");

        let tenant_orders = repo.list_by_tenant("t-1").await.expect("list by tenant");
        assert_eq!(tenant_orders.len(), 2);
        assert_eq!(tenant_orders[0].order_id, "ord-1");
        assert_eq!(tenant_orders[1].order_id, "ord-2");
    }

    #[test]
    fn repository_backend_defaults_to_in_memory() {
        // SAFETY: test mutation and cleanup are paired in this scope.
        unsafe { std::env::remove_var("ORDER_REPOSITORY_BACKEND") };
        let backend = OrderRepositoryBackend::from_env().expect("backend from env");
        assert_eq!(backend, OrderRepositoryBackend::InMemory);
    }

    #[test]
    fn repository_backend_parses_postgres() {
        // SAFETY: test mutation and cleanup are paired in this scope.
        unsafe { std::env::set_var("ORDER_REPOSITORY_BACKEND", "postgres") };
        let backend = OrderRepositoryBackend::from_env().expect("backend from env");
        // SAFETY: cleanup paired with set_var above.
        unsafe { std::env::remove_var("ORDER_REPOSITORY_BACKEND") };
        assert_eq!(backend, OrderRepositoryBackend::Postgres);
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
