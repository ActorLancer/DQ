use audit_kit::{
    AnchorBatch, AuditEvent, EvidenceItem, EvidenceManifest, EvidenceManifestItem, EvidencePackage,
    LegalHold, ReplayJob, ReplayResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditTraceView {
    pub audit_id: Option<String>,
    pub event_schema_version: String,
    pub event_class: String,
    pub domain_name: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub actor_id: Option<String>,
    pub actor_org_id: Option<String>,
    pub tenant_id: Option<String>,
    pub action_name: String,
    pub result_code: String,
    pub error_code: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub tx_hash: Option<String>,
    pub evidence_manifest_id: Option<String>,
    pub event_hash: Option<String>,
    pub occurred_at: Option<String>,
}

impl From<&AuditEvent> for AuditTraceView {
    fn from(event: &AuditEvent) -> Self {
        Self {
            audit_id: event.audit_id.clone(),
            event_schema_version: event.event_schema_version.clone(),
            event_class: event.event_class.clone(),
            domain_name: event.domain_name.clone(),
            ref_type: event.ref_type.clone(),
            ref_id: event.ref_id.clone(),
            actor_id: event.actor_id.clone(),
            actor_org_id: event.actor_org_id.clone(),
            tenant_id: event.tenant_id.clone(),
            action_name: event.action_name.clone(),
            result_code: event.result_code.clone(),
            error_code: event.error_code.clone(),
            request_id: event.request_id.clone(),
            trace_id: event.trace_id.clone(),
            tx_hash: event.tx_hash.clone(),
            evidence_manifest_id: event.evidence_manifest_id.clone(),
            event_hash: event.event_hash.clone(),
            occurred_at: event.occurred_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceItemView {
    pub evidence_item_id: Option<String>,
    pub item_type: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub object_uri: Option<String>,
    pub object_hash: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub source_system: Option<String>,
    pub storage_mode: Option<String>,
    pub retention_policy_id: Option<String>,
    pub legal_hold_status: String,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub metadata: serde_json::Value,
}

impl From<&EvidenceItem> for EvidenceItemView {
    fn from(item: &EvidenceItem) -> Self {
        Self {
            evidence_item_id: item.evidence_item_id.clone(),
            item_type: item.item_type.clone(),
            ref_type: item.ref_type.clone(),
            ref_id: item.ref_id.clone(),
            object_uri: item.object_uri.clone(),
            object_hash: item.object_hash.clone(),
            content_type: item.content_type.clone(),
            size_bytes: item.size_bytes,
            source_system: item.source_system.clone(),
            storage_mode: item.storage_mode.clone(),
            retention_policy_id: item.retention_policy_id.clone(),
            legal_hold_status: item.legal_hold_status.clone(),
            created_by: item.created_by.clone(),
            created_at: item.created_at.clone(),
            metadata: item.metadata.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceManifestView {
    pub evidence_manifest_id: Option<String>,
    pub manifest_scope: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub manifest_hash: String,
    pub item_count: i32,
    pub storage_uri: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub metadata: serde_json::Value,
}

impl From<&EvidenceManifest> for EvidenceManifestView {
    fn from(manifest: &EvidenceManifest) -> Self {
        Self {
            evidence_manifest_id: manifest.evidence_manifest_id.clone(),
            manifest_scope: manifest.manifest_scope.clone(),
            ref_type: manifest.ref_type.clone(),
            ref_id: manifest.ref_id.clone(),
            manifest_hash: manifest.manifest_hash.clone(),
            item_count: manifest.item_count,
            storage_uri: manifest.storage_uri.clone(),
            created_by: manifest.created_by.clone(),
            created_at: manifest.created_at.clone(),
            metadata: manifest.metadata.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceManifestItemView {
    pub evidence_manifest_item_id: Option<String>,
    pub evidence_manifest_id: Option<String>,
    pub evidence_item_id: Option<String>,
    pub item_digest: String,
    pub ordinal_no: i32,
    pub created_at: Option<String>,
}

impl From<&EvidenceManifestItem> for EvidenceManifestItemView {
    fn from(item: &EvidenceManifestItem) -> Self {
        Self {
            evidence_manifest_item_id: item.evidence_manifest_item_id.clone(),
            evidence_manifest_id: item.evidence_manifest_id.clone(),
            evidence_item_id: item.evidence_item_id.clone(),
            item_digest: item.item_digest.clone(),
            ordinal_no: item.ordinal_no,
            created_at: item.created_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidencePackageView {
    pub evidence_package_id: Option<String>,
    pub package_type: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub evidence_manifest_id: Option<String>,
    pub package_digest: Option<String>,
    pub storage_uri: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub retention_class: String,
    pub legal_hold_status: String,
    pub metadata: serde_json::Value,
}

impl From<&EvidencePackage> for EvidencePackageView {
    fn from(package: &EvidencePackage) -> Self {
        Self {
            evidence_package_id: package.evidence_package_id.clone(),
            package_type: package.package_type.clone(),
            ref_type: package.ref_type.clone(),
            ref_id: package.ref_id.clone(),
            evidence_manifest_id: package.evidence_manifest_id.clone(),
            package_digest: package.package_digest.clone(),
            storage_uri: package.storage_uri.clone(),
            created_by: package.created_by.clone(),
            created_at: package.created_at.clone(),
            retention_class: package.retention_class.clone(),
            legal_hold_status: package.legal_hold_status.clone(),
            metadata: package.metadata.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayJobView {
    pub replay_job_id: Option<String>,
    pub replay_type: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub dry_run: bool,
    pub replay_status: String,
    pub created_by: Option<String>,
    pub request_reason: Option<String>,
    pub step_up_challenge_id: Option<String>,
    pub created_at: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub updated_at: Option<String>,
    pub options: serde_json::Value,
}

impl From<&ReplayJob> for ReplayJobView {
    fn from(job: &ReplayJob) -> Self {
        Self {
            replay_job_id: job.replay_job_id.clone(),
            replay_type: job.replay_type.clone(),
            ref_type: job.ref_type.clone(),
            ref_id: job.ref_id.clone(),
            dry_run: job.dry_run,
            replay_status: job.status.clone(),
            created_by: job.requested_by.clone(),
            request_reason: job.request_reason.clone(),
            step_up_challenge_id: job.step_up_challenge_id.clone(),
            created_at: job.created_at.clone(),
            started_at: job.started_at.clone(),
            finished_at: job.finished_at.clone(),
            updated_at: job.updated_at.clone(),
            options: job.options_json.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnchorBatchView {
    pub anchor_batch_id: Option<String>,
    pub batch_scope: String,
    pub record_count: i32,
    pub batch_root: String,
    pub chain_id: String,
    pub anchor_status: String,
    pub tx_hash: Option<String>,
    pub anchored_at: Option<String>,
    pub window_started_at: Option<String>,
    pub window_ended_at: Option<String>,
    pub chain_anchor_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub metadata: serde_json::Value,
}

impl From<&AnchorBatch> for AnchorBatchView {
    fn from(batch: &AnchorBatch) -> Self {
        Self {
            anchor_batch_id: batch.anchor_batch_id.clone(),
            batch_scope: batch.batch_scope.clone(),
            record_count: batch.record_count,
            batch_root: batch.batch_root.clone(),
            chain_id: batch.chain_id.clone(),
            anchor_status: batch.status.clone(),
            tx_hash: batch
                .metadata
                .get("tx_hash")
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            anchored_at: batch.anchored_at.clone(),
            window_started_at: batch.window_started_at.clone(),
            window_ended_at: batch.window_ended_at.clone(),
            chain_anchor_id: batch.chain_anchor_id.clone(),
            created_by: batch.created_by.clone(),
            created_at: batch.created_at.clone(),
            updated_at: batch.updated_at.clone(),
            metadata: batch.metadata.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayResultView {
    pub replay_result_id: Option<String>,
    pub replay_job_id: Option<String>,
    pub step_name: String,
    pub result_code: String,
    pub expected_digest: Option<String>,
    pub actual_digest: Option<String>,
    pub diff_summary: serde_json::Value,
    pub created_at: Option<String>,
}

impl From<&ReplayResult> for ReplayResultView {
    fn from(result: &ReplayResult) -> Self {
        Self {
            replay_result_id: result.replay_result_id.clone(),
            replay_job_id: result.replay_job_id.clone(),
            step_name: result.step_name.clone(),
            result_code: result.result_code.clone(),
            expected_digest: result.expected_digest.clone(),
            actual_digest: result.actual_digest.clone(),
            diff_summary: result.diff_summary.clone(),
            created_at: result.created_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegalHoldView {
    pub legal_hold_id: Option<String>,
    pub hold_scope_type: String,
    pub hold_scope_id: Option<String>,
    pub reason_code: String,
    pub status: String,
    pub retention_policy_id: Option<String>,
    pub requested_by: Option<String>,
    pub approved_by: Option<String>,
    pub hold_until: Option<String>,
    pub created_at: Option<String>,
    pub released_at: Option<String>,
    pub updated_at: Option<String>,
    pub metadata: serde_json::Value,
}

impl From<&LegalHold> for LegalHoldView {
    fn from(hold: &LegalHold) -> Self {
        Self {
            legal_hold_id: hold.legal_hold_id.clone(),
            hold_scope_type: hold.hold_scope_type.clone(),
            hold_scope_id: hold.hold_scope_id.clone(),
            reason_code: hold.reason_code.clone(),
            status: hold.status.clone(),
            retention_policy_id: hold.retention_policy_id.clone(),
            requested_by: hold.requested_by.clone(),
            approved_by: hold.approved_by.clone(),
            hold_until: hold.hold_until.clone(),
            created_at: hold.created_at.clone(),
            released_at: hold.released_at.clone(),
            updated_at: hold.updated_at.clone(),
            metadata: hold.metadata.clone(),
        }
    }
}
