use kernel::{Pagination, PaginationQuery};
use serde::{Deserialize, Serialize};

pub use audit_kit::{
    AnchorBatch, AuditAccessRecord, AuditAnnotation, AuditContext, AuditEvent, AuditExportRecord,
    AuditResultStatus, AuditRiskLevel, AuditWriter, EvidenceItem, EvidenceManifest,
    EvidenceManifestItem, EvidencePackage, LegalHold, NoopAuditWriter, ReplayJob, ReplayResult,
    RetentionPolicy,
};

use crate::modules::audit::dto::AuditTraceView;
use crate::modules::audit::dto::{
    ChainProjectionGapView, ConsumerIdempotencyRecordView, DeadLetterEventView,
    EvidenceManifestView, EvidencePackageView, ExternalFactReceiptView, LegalHoldView,
    OutboxEventView, ReplayJobView, ReplayResultView,
};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AuditTraceQuery {
    pub order_id: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub action_name: Option<String>,
    pub result_code: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl AuditTraceQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }

    pub fn effective_order_id(&self) -> Option<&str> {
        self.order_id.as_deref().or_else(|| {
            if self.ref_type.as_deref() == Some("order") {
                self.ref_id.as_deref()
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderAuditQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl OrderAuditQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditTracePageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<AuditTraceView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderAuditView {
    pub order_id: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub status: String,
    pub payment_status: String,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub traces: Vec<AuditTraceView>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AuditPackageExportRequest {
    pub ref_type: String,
    pub ref_id: String,
    pub reason: String,
    pub masked_level: Option<String>,
    pub package_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditPackageExportView {
    pub evidence_package: EvidencePackageView,
    pub evidence_manifest: EvidenceManifestView,
    pub audit_trace_count: i64,
    pub evidence_item_count: i64,
    pub legal_hold_status: String,
    pub step_up_bound: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AuditReplayJobCreateRequest {
    pub replay_type: String,
    pub ref_type: String,
    pub ref_id: String,
    pub reason: String,
    pub dry_run: Option<bool>,
    #[serde(default)]
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditReplayJobDetailView {
    pub replay_job: ReplayJobView,
    pub results: Vec<ReplayResultView>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AuditLegalHoldCreateRequest {
    pub hold_scope_type: String,
    pub hold_scope_id: String,
    pub reason_code: String,
    pub retention_policy_id: Option<String>,
    pub hold_until: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AuditLegalHoldReleaseRequest {
    pub reason: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLegalHoldActionView {
    pub legal_hold: LegalHoldView,
    pub step_up_bound: bool,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct AnchorBatchQuery {
    pub anchor_status: Option<String>,
    pub batch_scope: Option<String>,
    pub chain_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl AnchorBatchQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnchorBatchPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<crate::modules::audit::dto::AnchorBatchView>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AuditAnchorBatchRetryRequest {
    pub reason: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditAnchorBatchRetryView {
    pub anchor_batch: crate::modules::audit::dto::AnchorBatchView,
    pub step_up_bound: bool,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct OpsOutboxQuery {
    pub outbox_status: Option<String>,
    pub event_type: Option<String>,
    pub target_topic: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub aggregate_type: Option<String>,
    pub idempotency_key: Option<String>,
    pub authority_scope: Option<String>,
    pub source_of_truth: Option<String>,
    pub proof_commit_policy: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl OpsOutboxQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsOutboxPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<OutboxEventView>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct OpsDeadLetterQuery {
    pub reprocess_status: Option<String>,
    pub failure_stage: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl OpsDeadLetterQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsDeadLetterPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<DeadLetterEventView>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct ConsumerIdempotencyQuery {
    pub consumer_name: Option<String>,
    pub event_id: Option<String>,
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<String>,
    pub trace_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl ConsumerIdempotencyQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsumerIdempotencyPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<ConsumerIdempotencyRecordView>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct ExternalFactReceiptQuery {
    pub order_id: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub fact_type: Option<String>,
    pub provider_type: Option<String>,
    pub receipt_status: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl ExternalFactReceiptQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalFactReceiptPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<ExternalFactReceiptView>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct ChainProjectionGapQuery {
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<String>,
    pub order_id: Option<String>,
    pub chain_id: Option<String>,
    pub gap_type: Option<String>,
    pub gap_status: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl ChainProjectionGapQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChainProjectionGapPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<ChainProjectionGapView>,
}
