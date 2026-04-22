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
    EvidenceManifestView, EvidencePackageView, ReplayJobView, ReplayResultView,
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
