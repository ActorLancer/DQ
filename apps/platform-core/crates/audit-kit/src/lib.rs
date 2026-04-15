use async_trait::async_trait;
use kernel::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditResultStatus {
    Success,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditAnnotation {
    pub action: String,
    pub risk_level: AuditRiskLevel,
    pub object_type: String,
    pub object_id: String,
    pub result: AuditResultStatus,
}

impl AuditAnnotation {
    pub fn new(
        action: impl Into<String>,
        risk_level: AuditRiskLevel,
        object_type: impl Into<String>,
        object_id: impl Into<String>,
        result: AuditResultStatus,
    ) -> Self {
        Self {
            action: action.into(),
            risk_level,
            object_type: object_type.into(),
            object_id: object_id.into(),
            result,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditContext {
    pub request_id: String,
    pub actor_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceItem {
    pub evidence_type: String,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub action: String,
    pub object_type: String,
    pub object_id: String,
    pub result: String,
    pub context: AuditContext,
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditExportRecord {
    pub export_id: String,
    pub reason: String,
    pub requested_by: String,
}

#[async_trait]
pub trait AuditWriter: Send + Sync {
    async fn write_event(&self, event: AuditEvent) -> AppResult<()>;
    async fn record_export(&self, record: AuditExportRecord) -> AppResult<()>;
}

#[derive(Debug, Default, Clone)]
pub struct NoopAuditWriter;

#[async_trait]
impl AuditWriter for NoopAuditWriter {
    async fn write_event(&self, _event: AuditEvent) -> AppResult<()> {
        Ok(())
    }

    async fn record_export(&self, _record: AuditExportRecord) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annotation_builder_keeps_declared_fields() {
        let annotation = AuditAnnotation::new(
            "order.create",
            AuditRiskLevel::High,
            "order",
            "ord-1",
            AuditResultStatus::Success,
        );
        assert_eq!(annotation.action, "order.create");
        assert_eq!(annotation.risk_level, AuditRiskLevel::High);
        assert_eq!(annotation.object_type, "order");
        assert_eq!(annotation.object_id, "ord-1");
        assert_eq!(annotation.result, AuditResultStatus::Success);
    }

    #[tokio::test]
    async fn noop_writer_accepts_event() {
        let writer = NoopAuditWriter;
        let event = AuditEvent {
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
        };
        writer.write_event(event).await.expect("write should succeed");
    }
}
