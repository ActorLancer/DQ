use async_trait::async_trait;
use kernel::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PublishStatus {
    Pending,
    Publishing,
    Published,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 8,
            backoff_ms: 1_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelope {
    pub event_id: String,
    pub topic: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub payload_json: String,
    pub idempotency_key: String,
    pub status: PublishStatus,
    pub retry_policy: RetryPolicy,
}

#[async_trait]
pub trait OutboxWriter: Send + Sync {
    async fn append(&self, event: EventEnvelope) -> AppResult<()>;
    async fn mark_status(&self, event_id: &str, status: PublishStatus) -> AppResult<()>;
}

#[derive(Debug, Default, Clone)]
pub struct NoopOutboxWriter;

#[async_trait]
impl OutboxWriter for NoopOutboxWriter {
    async fn append(&self, _event: EventEnvelope) -> AppResult<()> {
        Ok(())
    }

    async fn mark_status(&self, _event_id: &str, _status: PublishStatus) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_retry_policy_is_positive() {
        let policy = RetryPolicy::default();
        assert!(policy.max_attempts > 0);
        assert!(policy.backoff_ms > 0);
    }
}
