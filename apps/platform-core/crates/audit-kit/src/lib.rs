use async_trait::async_trait;
use kernel::AppResult;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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

fn empty_metadata() -> Value {
    Value::Object(Map::new())
}

fn default_event_schema_version() -> String {
    "v1".to_string()
}

fn default_event_class() -> String {
    "business".to_string()
}

fn default_actor_type() -> String {
    "user".to_string()
}

fn default_anchor_policy() -> String {
    "batched_fabric".to_string()
}

fn default_retention_class() -> String {
    "audit_default".to_string()
}

fn default_legal_hold_status() -> String {
    "none".to_string()
}

fn default_sensitivity_level() -> String {
    "normal".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditContext {
    pub request_id: String,
    pub trace_id: String,
    #[serde(default = "default_actor_type")]
    pub actor_type: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub actor_org_id: Option<String>,
    pub tenant_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub trusted_device_id: Option<String>,
    #[serde(default)]
    pub application_id: Option<String>,
    #[serde(default)]
    pub parent_audit_id: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub client_fingerprint: Option<String>,
    #[serde(default)]
    pub auth_assurance_level: Option<String>,
    #[serde(default)]
    pub step_up_challenge_id: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
}

impl AuditContext {
    pub fn minimal(
        request_id: impl Into<String>,
        trace_id: impl Into<String>,
        actor_id: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Self {
        let tenant_id = tenant_id.into();
        Self {
            request_id: request_id.into(),
            trace_id: trace_id.into(),
            actor_type: default_actor_type(),
            actor_id: Some(actor_id.into()),
            actor_org_id: Some(tenant_id.clone()),
            tenant_id,
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: None,
            step_up_challenge_id: None,
            metadata: empty_metadata(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceItem {
    #[serde(default)]
    pub evidence_item_id: Option<String>,
    pub item_type: String,
    pub ref_type: String,
    #[serde(default)]
    pub ref_id: Option<String>,
    #[serde(default)]
    pub object_uri: Option<String>,
    #[serde(default)]
    pub object_hash: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<i64>,
    #[serde(default)]
    pub source_system: Option<String>,
    #[serde(default)]
    pub storage_mode: Option<String>,
    #[serde(default)]
    pub retention_policy_id: Option<String>,
    #[serde(default)]
    pub worm_enabled: bool,
    #[serde(default = "default_legal_hold_status")]
    pub legal_hold_status: String,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    #[serde(default)]
    pub audit_id: Option<String>,
    #[serde(default = "default_event_schema_version")]
    pub event_schema_version: String,
    #[serde(default = "default_event_class")]
    pub event_class: String,
    pub domain_name: String,
    pub ref_type: String,
    #[serde(default)]
    pub ref_id: Option<String>,
    #[serde(default = "default_actor_type")]
    pub actor_type: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub actor_org_id: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub trusted_device_id: Option<String>,
    #[serde(default)]
    pub application_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub parent_audit_id: Option<String>,
    pub action_name: String,
    pub result_code: String,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub client_fingerprint: Option<String>,
    #[serde(default)]
    pub auth_assurance_level: Option<String>,
    #[serde(default)]
    pub step_up_challenge_id: Option<String>,
    #[serde(default)]
    pub before_state_digest: Option<String>,
    #[serde(default)]
    pub after_state_digest: Option<String>,
    #[serde(default)]
    pub tx_hash: Option<String>,
    #[serde(default)]
    pub previous_event_hash: Option<String>,
    #[serde(default)]
    pub event_hash: Option<String>,
    #[serde(default)]
    pub evidence_hash: Option<String>,
    #[serde(default)]
    pub payload_digest: Option<String>,
    #[serde(default)]
    pub evidence_manifest_id: Option<String>,
    #[serde(default = "default_anchor_policy")]
    pub anchor_policy: String,
    #[serde(default = "default_retention_class")]
    pub retention_class: String,
    #[serde(default = "default_legal_hold_status")]
    pub legal_hold_status: String,
    #[serde(default = "default_sensitivity_level")]
    pub sensitivity_level: String,
    #[serde(default)]
    pub occurred_at: Option<String>,
    #[serde(default)]
    pub ingested_at: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,
}

impl AuditEvent {
    pub fn business(
        domain_name: impl Into<String>,
        ref_type: impl Into<String>,
        ref_id: Option<String>,
        action_name: impl Into<String>,
        result_code: impl Into<String>,
        context: AuditContext,
    ) -> Self {
        Self {
            audit_id: None,
            event_schema_version: default_event_schema_version(),
            event_class: default_event_class(),
            domain_name: domain_name.into(),
            ref_type: ref_type.into(),
            ref_id,
            actor_type: context.actor_type,
            actor_id: context.actor_id,
            actor_org_id: context.actor_org_id,
            tenant_id: Some(context.tenant_id),
            session_id: context.session_id,
            trusted_device_id: context.trusted_device_id,
            application_id: context.application_id,
            request_id: Some(context.request_id),
            trace_id: Some(context.trace_id),
            parent_audit_id: context.parent_audit_id,
            action_name: action_name.into(),
            result_code: result_code.into(),
            error_code: None,
            source_ip: context.source_ip,
            client_fingerprint: context.client_fingerprint,
            auth_assurance_level: context.auth_assurance_level,
            step_up_challenge_id: context.step_up_challenge_id,
            before_state_digest: None,
            after_state_digest: None,
            tx_hash: None,
            previous_event_hash: None,
            event_hash: None,
            evidence_hash: None,
            payload_digest: None,
            evidence_manifest_id: None,
            anchor_policy: default_anchor_policy(),
            retention_class: default_retention_class(),
            legal_hold_status: default_legal_hold_status(),
            sensitivity_level: default_sensitivity_level(),
            occurred_at: None,
            ingested_at: None,
            metadata: context.metadata,
            evidence: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditExportRecord {
    pub export_id: String,
    #[serde(default)]
    pub evidence_manifest_id: Option<String>,
    #[serde(default)]
    pub access_mode: Option<String>,
    pub reason: String,
    pub requested_by: String,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub step_up_challenge_id: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceManifest {
    #[serde(default)]
    pub evidence_manifest_id: Option<String>,
    pub manifest_scope: String,
    pub ref_type: String,
    #[serde(default)]
    pub ref_id: Option<String>,
    pub manifest_hash: String,
    #[serde(default)]
    pub item_count: i32,
    #[serde(default)]
    pub storage_uri: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidencePackage {
    #[serde(default)]
    pub evidence_package_id: Option<String>,
    pub package_type: String,
    pub ref_type: String,
    #[serde(default)]
    pub ref_id: Option<String>,
    #[serde(default)]
    pub evidence_manifest_id: Option<String>,
    #[serde(default)]
    pub package_digest: Option<String>,
    #[serde(default)]
    pub storage_uri: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default = "default_retention_class")]
    pub retention_class: String,
    #[serde(default = "default_legal_hold_status")]
    pub legal_hold_status: String,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayJob {
    #[serde(default)]
    pub replay_job_id: Option<String>,
    pub replay_type: String,
    pub ref_type: String,
    #[serde(default)]
    pub ref_id: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
    pub status: String,
    #[serde(default)]
    pub requested_by: Option<String>,
    #[serde(default)]
    pub step_up_challenge_id: Option<String>,
    #[serde(default)]
    pub request_reason: Option<String>,
    #[serde(default = "empty_metadata")]
    pub options_json: Value,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayResult {
    #[serde(default)]
    pub replay_result_id: Option<String>,
    #[serde(default)]
    pub replay_job_id: Option<String>,
    pub step_name: String,
    pub result_code: String,
    #[serde(default)]
    pub expected_digest: Option<String>,
    #[serde(default)]
    pub actual_digest: Option<String>,
    #[serde(default = "empty_metadata")]
    pub diff_summary: Value,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnchorBatch {
    #[serde(default)]
    pub anchor_batch_id: Option<String>,
    pub batch_scope: String,
    pub chain_id: String,
    #[serde(default)]
    pub record_count: i32,
    pub batch_root: String,
    #[serde(default)]
    pub window_started_at: Option<String>,
    #[serde(default)]
    pub window_ended_at: Option<String>,
    pub status: String,
    #[serde(default)]
    pub chain_anchor_id: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub anchored_at: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegalHold {
    #[serde(default)]
    pub legal_hold_id: Option<String>,
    pub hold_scope_type: String,
    #[serde(default)]
    pub hold_scope_id: Option<String>,
    pub reason_code: String,
    pub status: String,
    #[serde(default)]
    pub retention_policy_id: Option<String>,
    #[serde(default)]
    pub requested_by: Option<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub hold_until: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub released_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetentionPolicy {
    #[serde(default)]
    pub retention_policy_id: Option<String>,
    pub policy_key: String,
    pub scope_type: String,
    #[serde(default)]
    pub scope_id: Option<String>,
    pub retention_class: String,
    #[serde(default)]
    pub hot_days: Option<i32>,
    #[serde(default)]
    pub warm_days: Option<i32>,
    #[serde(default)]
    pub cold_days: Option<i32>,
    #[serde(default)]
    pub delete_after_days: Option<i32>,
    #[serde(default)]
    pub worm_required: bool,
    #[serde(default = "default_legal_hold_status")]
    pub legal_hold_status: String,
    pub status: String,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditAccessRecord {
    #[serde(default)]
    pub access_audit_id: Option<String>,
    #[serde(default)]
    pub accessor_user_id: Option<String>,
    #[serde(default)]
    pub accessor_role_key: Option<String>,
    pub access_mode: String,
    pub target_type: String,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub masked_view: bool,
    #[serde(default)]
    pub breakglass_reason: Option<String>,
    #[serde(default)]
    pub step_up_challenge_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: Value,
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
        let event = AuditEvent::business(
            "trade",
            "order",
            Some("ord-1".to_string()),
            "order.create",
            "success",
            AuditContext::minimal("req-1", "trace-1", "user-1", "tenant-1"),
        );
        writer
            .write_event(event)
            .await
            .expect("write should succeed");
    }

    #[test]
    fn business_event_builder_sets_unified_defaults() {
        let mut context = AuditContext::minimal("req-1", "trace-1", "user-1", "tenant-1");
        context.auth_assurance_level = Some("aal2".to_string());
        context.step_up_challenge_id = Some("step-up-1".to_string());
        context.metadata = serde_json::json!({ "actor_role": "tenant_admin" });

        let event = AuditEvent::business(
            "audit",
            "replay_job",
            Some("job-1".to_string()),
            "audit.replay.request",
            "accepted",
            context,
        );

        assert_eq!(event.event_schema_version, "v1");
        assert_eq!(event.event_class, "business");
        assert_eq!(event.domain_name, "audit");
        assert_eq!(event.ref_type, "replay_job");
        assert_eq!(event.ref_id.as_deref(), Some("job-1"));
        assert_eq!(event.request_id.as_deref(), Some("req-1"));
        assert_eq!(event.trace_id.as_deref(), Some("trace-1"));
        assert_eq!(event.actor_id.as_deref(), Some("user-1"));
        assert_eq!(event.tenant_id.as_deref(), Some("tenant-1"));
        assert_eq!(event.auth_assurance_level.as_deref(), Some("aal2"));
        assert_eq!(event.step_up_challenge_id.as_deref(), Some("step-up-1"));
        assert_eq!(event.anchor_policy, "batched_fabric");
        assert_eq!(event.retention_class, "audit_default");
        assert_eq!(event.legal_hold_status, "none");
        assert_eq!(event.sensitivity_level, "normal");
    }

    #[test]
    fn unified_audit_domain_models_are_serializable() {
        let manifest = EvidenceManifest {
            evidence_manifest_id: Some("manifest-1".to_string()),
            manifest_scope: "audit_export".to_string(),
            ref_type: "order".to_string(),
            ref_id: Some("ord-1".to_string()),
            manifest_hash: "hash-1".to_string(),
            item_count: 2,
            storage_uri: Some("s3://audit/manifest-1.json".to_string()),
            created_by: Some("user-1".to_string()),
            created_at: Some("2026-04-22T09:00:00.000Z".to_string()),
            metadata: serde_json::json!({ "mask_level": "summary" }),
        };
        let replay = ReplayJob {
            replay_job_id: Some("replay-1".to_string()),
            replay_type: "state".to_string(),
            ref_type: "order".to_string(),
            ref_id: Some("ord-1".to_string()),
            dry_run: true,
            status: "pending".to_string(),
            requested_by: Some("user-1".to_string()),
            step_up_challenge_id: Some("step-up-1".to_string()),
            request_reason: Some("investigation".to_string()),
            options_json: serde_json::json!({ "include_evidence": true }),
            created_at: None,
            started_at: None,
            finished_at: None,
            updated_at: None,
        };
        let access = AuditAccessRecord {
            access_audit_id: Some("access-1".to_string()),
            accessor_user_id: Some("user-1".to_string()),
            accessor_role_key: Some("risk_analyst".to_string()),
            access_mode: "export".to_string(),
            target_type: "evidence_manifest".to_string(),
            target_id: Some("manifest-1".to_string()),
            masked_view: false,
            breakglass_reason: Some("regulatory request".to_string()),
            step_up_challenge_id: Some("step-up-1".to_string()),
            request_id: Some("req-1".to_string()),
            trace_id: Some("trace-1".to_string()),
            created_at: Some("2026-04-22T09:01:00.000Z".to_string()),
            metadata: serde_json::json!({ "channel": "api" }),
        };

        let serialized = serde_json::to_value((&manifest, &replay, &access))
            .expect("formal audit models should serialize");
        assert_eq!(serialized[0]["manifest_scope"], "audit_export");
        assert_eq!(serialized[1]["replay_type"], "state");
        assert_eq!(serialized[2]["access_mode"], "export");
    }
}
