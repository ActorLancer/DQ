use audit_kit::{
    AnchorBatch, AuditEvent, EvidenceItem, EvidenceManifest, EvidenceManifestItem, EvidencePackage,
    LegalHold, ReplayJob, ReplayResult,
};
use serde::{Deserialize, Serialize};

use crate::modules::audit::repo::{
    ChainProjectionGapRecord, ConsumerIdempotencyRecord, DeadLetterEventRecord,
    ExternalFactReceiptRecord, OutboxEventRecord, OutboxPublishAttemptRecord,
};

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
pub struct OutboxPublishAttemptView {
    pub outbox_publish_attempt_id: Option<String>,
    pub worker_id: Option<String>,
    pub target_bus: String,
    pub target_topic: Option<String>,
    pub attempt_no: i32,
    pub result_code: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub attempted_at: Option<String>,
    pub completed_at: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsumerIdempotencyRecordView {
    pub consumer_idempotency_record_id: Option<String>,
    pub consumer_name: String,
    pub event_id: String,
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<String>,
    pub trace_id: Option<String>,
    pub result_code: String,
    pub processed_at: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutboxEventView {
    pub outbox_event_id: Option<String>,
    pub aggregate_type: String,
    pub aggregate_id: Option<String>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub available_at: Option<String>,
    pub published_at: Option<String>,
    pub created_at: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub authority_scope: String,
    pub source_of_truth: String,
    pub proof_commit_policy: String,
    pub target_bus: String,
    pub target_topic: Option<String>,
    pub partition_key: Option<String>,
    pub ordering_key: Option<String>,
    pub payload_hash: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub latest_publish_attempt: Option<OutboxPublishAttemptView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeadLetterEventView {
    pub dead_letter_event_id: Option<String>,
    pub outbox_event_id: Option<String>,
    pub aggregate_type: String,
    pub aggregate_id: Option<String>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub failed_reason: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub authority_scope: String,
    pub source_of_truth: String,
    pub target_bus: String,
    pub target_topic: Option<String>,
    pub failure_stage: Option<String>,
    pub first_failed_at: Option<String>,
    pub last_failed_at: Option<String>,
    pub reprocess_status: String,
    pub reprocessed_at: Option<String>,
    pub created_at: Option<String>,
    pub consumer_idempotency_records: Vec<ConsumerIdempotencyRecordView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalFactReceiptView {
    pub external_fact_receipt_id: Option<String>,
    pub order_id: Option<String>,
    pub ref_domain: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub fact_type: String,
    pub provider_type: String,
    pub provider_key: Option<String>,
    pub provider_reference: Option<String>,
    pub receipt_status: String,
    pub receipt_payload: serde_json::Value,
    pub receipt_hash: Option<String>,
    pub occurred_at: Option<String>,
    pub received_at: Option<String>,
    pub confirmed_at: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChainProjectionGapView {
    pub chain_projection_gap_id: Option<String>,
    pub aggregate_type: String,
    pub aggregate_id: Option<String>,
    pub order_id: Option<String>,
    pub chain_id: String,
    pub source_event_type: Option<String>,
    pub expected_tx_id: Option<String>,
    pub projected_tx_hash: Option<String>,
    pub gap_type: String,
    pub gap_status: String,
    pub first_detected_at: Option<String>,
    pub last_detected_at: Option<String>,
    pub resolved_at: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub outbox_event_id: Option<String>,
    pub anchor_id: Option<String>,
    pub resolution_summary: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<&OutboxPublishAttemptRecord> for OutboxPublishAttemptView {
    fn from(attempt: &OutboxPublishAttemptRecord) -> Self {
        Self {
            outbox_publish_attempt_id: attempt.outbox_publish_attempt_id.clone(),
            worker_id: attempt.worker_id.clone(),
            target_bus: attempt.target_bus.clone(),
            target_topic: attempt.target_topic.clone(),
            attempt_no: attempt.attempt_no,
            result_code: attempt.result_code.clone(),
            error_code: attempt.error_code.clone(),
            error_message: attempt.error_message.clone(),
            attempted_at: attempt.attempted_at.clone(),
            completed_at: attempt.completed_at.clone(),
            metadata: attempt.metadata.clone(),
        }
    }
}

impl From<&ConsumerIdempotencyRecord> for ConsumerIdempotencyRecordView {
    fn from(record: &ConsumerIdempotencyRecord) -> Self {
        Self {
            consumer_idempotency_record_id: record.consumer_idempotency_record_id.clone(),
            consumer_name: record.consumer_name.clone(),
            event_id: record.event_id.clone(),
            aggregate_type: record.aggregate_type.clone(),
            aggregate_id: record.aggregate_id.clone(),
            trace_id: record.trace_id.clone(),
            result_code: record.result_code.clone(),
            processed_at: record.processed_at.clone(),
            metadata: record.metadata.clone(),
        }
    }
}

impl From<&OutboxEventRecord> for OutboxEventView {
    fn from(record: &OutboxEventRecord) -> Self {
        Self {
            outbox_event_id: record.outbox_event_id.clone(),
            aggregate_type: record.aggregate_type.clone(),
            aggregate_id: record.aggregate_id.clone(),
            event_type: record.event_type.clone(),
            payload: record.payload.clone(),
            status: record.status.clone(),
            retry_count: record.retry_count,
            max_retries: record.max_retries,
            available_at: record.available_at.clone(),
            published_at: record.published_at.clone(),
            created_at: record.created_at.clone(),
            request_id: record.request_id.clone(),
            trace_id: record.trace_id.clone(),
            idempotency_key: record.idempotency_key.clone(),
            authority_scope: record.authority_scope.clone(),
            source_of_truth: record.source_of_truth.clone(),
            proof_commit_policy: record.proof_commit_policy.clone(),
            target_bus: record.target_bus.clone(),
            target_topic: record.target_topic.clone(),
            partition_key: record.partition_key.clone(),
            ordering_key: record.ordering_key.clone(),
            payload_hash: record.payload_hash.clone(),
            last_error_code: record.last_error_code.clone(),
            last_error_message: record.last_error_message.clone(),
            latest_publish_attempt: record
                .latest_publish_attempt
                .as_ref()
                .map(OutboxPublishAttemptView::from),
        }
    }
}

impl From<&DeadLetterEventRecord> for DeadLetterEventView {
    fn from(record: &DeadLetterEventRecord) -> Self {
        Self {
            dead_letter_event_id: record.dead_letter_event_id.clone(),
            outbox_event_id: record.outbox_event_id.clone(),
            aggregate_type: record.aggregate_type.clone(),
            aggregate_id: record.aggregate_id.clone(),
            event_type: record.event_type.clone(),
            payload: record.payload.clone(),
            failed_reason: record.failed_reason.clone(),
            request_id: record.request_id.clone(),
            trace_id: record.trace_id.clone(),
            authority_scope: record.authority_scope.clone(),
            source_of_truth: record.source_of_truth.clone(),
            target_bus: record.target_bus.clone(),
            target_topic: record.target_topic.clone(),
            failure_stage: record.failure_stage.clone(),
            first_failed_at: record.first_failed_at.clone(),
            last_failed_at: record.last_failed_at.clone(),
            reprocess_status: record.reprocess_status.clone(),
            reprocessed_at: record.reprocessed_at.clone(),
            created_at: record.created_at.clone(),
            consumer_idempotency_records: record
                .consumer_idempotency_records
                .iter()
                .map(ConsumerIdempotencyRecordView::from)
                .collect(),
        }
    }
}

impl From<&ExternalFactReceiptRecord> for ExternalFactReceiptView {
    fn from(record: &ExternalFactReceiptRecord) -> Self {
        Self {
            external_fact_receipt_id: record.external_fact_receipt_id.clone(),
            order_id: record.order_id.clone(),
            ref_domain: record.ref_domain.clone(),
            ref_type: record.ref_type.clone(),
            ref_id: record.ref_id.clone(),
            fact_type: record.fact_type.clone(),
            provider_type: record.provider_type.clone(),
            provider_key: record.provider_key.clone(),
            provider_reference: record.provider_reference.clone(),
            receipt_status: record.receipt_status.clone(),
            receipt_payload: record.receipt_payload.clone(),
            receipt_hash: record.receipt_hash.clone(),
            occurred_at: record.occurred_at.clone(),
            received_at: record.received_at.clone(),
            confirmed_at: record.confirmed_at.clone(),
            request_id: record.request_id.clone(),
            trace_id: record.trace_id.clone(),
            metadata: record.metadata.clone(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}

impl From<&ChainProjectionGapRecord> for ChainProjectionGapView {
    fn from(record: &ChainProjectionGapRecord) -> Self {
        Self {
            chain_projection_gap_id: record.chain_projection_gap_id.clone(),
            aggregate_type: record.aggregate_type.clone(),
            aggregate_id: record.aggregate_id.clone(),
            order_id: record.order_id.clone(),
            chain_id: record.chain_id.clone(),
            source_event_type: record.source_event_type.clone(),
            expected_tx_id: record.expected_tx_id.clone(),
            projected_tx_hash: record.projected_tx_hash.clone(),
            gap_type: record.gap_type.clone(),
            gap_status: record.gap_status.clone(),
            first_detected_at: record.first_detected_at.clone(),
            last_detected_at: record.last_detected_at.clone(),
            resolved_at: record.resolved_at.clone(),
            request_id: record.request_id.clone(),
            trace_id: record.trace_id.clone(),
            outbox_event_id: record.outbox_event_id.clone(),
            anchor_id: record.anchor_id.clone(),
            resolution_summary: record.resolution_summary.clone(),
            metadata: record.metadata.clone(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
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
