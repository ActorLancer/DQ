use audit_kit::{AuditEvent, EvidenceItem, EvidenceManifest, EvidenceManifestItem};
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
