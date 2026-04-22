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
    AlertEventView, ChainProjectionGapView, ConsumerIdempotencyRecordView, DeadLetterEventView,
    EvidenceManifestView, EvidencePackageView, ExternalFactReceiptView, FairnessIncidentView,
    IncidentTicketView, LegalHoldView, ObservabilityBackendView, OutboxEventView, ReplayJobView,
    ReplayResultView, SloView, SystemLogMirrorView, TraceIndexView, TradeLifecycleCheckpointView,
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OpsDeadLetterReprocessRequest {
    pub reason: String,
    pub dry_run: Option<bool>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsDeadLetterReprocessView {
    pub dead_letter: DeadLetterEventView,
    pub dry_run: bool,
    pub step_up_bound: bool,
    pub status: String,
    pub consumer_names: Vec<String>,
    pub consumer_groups: Vec<String>,
    pub replay_target_topic: String,
    pub replay_plan: serde_json::Value,
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
    pub from: Option<String>,
    pub to: Option<String>,
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OpsExternalFactConfirmRequest {
    pub confirm_result: String,
    pub reason: String,
    pub operator_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsExternalFactConfirmView {
    pub external_fact_receipt: ExternalFactReceiptView,
    pub confirm_result: String,
    pub step_up_bound: bool,
    pub status: String,
    pub rule_evaluation_status: String,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct FairnessIncidentQuery {
    pub order_id: Option<String>,
    pub incident_type: Option<String>,
    pub severity: Option<String>,
    pub fairness_incident_status: Option<String>,
    pub assigned_role_key: Option<String>,
    pub assigned_user_id: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl FairnessIncidentQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FairnessIncidentPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<FairnessIncidentView>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OpsFairnessIncidentHandleRequest {
    pub action: String,
    pub resolution_summary: String,
    pub auto_action_override: Option<String>,
    pub freeze_settlement: Option<bool>,
    pub freeze_delivery: Option<bool>,
    pub create_dispute_suggestion: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsFairnessIncidentHandleView {
    pub fairness_incident: FairnessIncidentView,
    pub action: String,
    pub step_up_bound: bool,
    pub status: String,
    pub action_plan_status: String,
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OpsProjectionGapResolveRequest {
    pub dry_run: Option<bool>,
    pub resolution_mode: Option<String>,
    pub reason: String,
    pub expected_state_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsProjectionGapResolveView {
    pub projection_gap: ChainProjectionGapView,
    pub resolution_mode: String,
    pub reason: String,
    pub expected_state_digest: Option<String>,
    pub state_digest: String,
    pub step_up_bound: bool,
    pub dry_run: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsConsistencyBusinessStateView {
    pub ref_type: String,
    pub ref_id: String,
    pub order_id: Option<String>,
    pub business_status: String,
    pub authority_model: String,
    pub business_state_version: i64,
    pub proof_commit_state: String,
    pub proof_commit_policy: String,
    pub external_fact_status: String,
    pub reconcile_status: String,
    pub last_reconciled_at: Option<String>,
    pub snapshot: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsConsistencyProofStateView {
    pub proof_commit_state: String,
    pub proof_commit_policy: String,
    pub latest_chain_anchor: Option<serde_json::Value>,
    pub projection_gap_status_breakdown: serde_json::Value,
    pub open_projection_gap_count: i64,
    pub latest_projection_gap: Option<ChainProjectionGapView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsConsistencyExternalFactStateView {
    pub summary_status: String,
    pub total_receipts: i64,
    pub receipt_status_breakdown: serde_json::Value,
    pub latest_receipt: Option<ExternalFactReceiptView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsConsistencyView {
    pub ref_type: String,
    pub ref_id: String,
    pub business_state: OpsConsistencyBusinessStateView,
    pub proof_state: OpsConsistencyProofStateView,
    pub external_fact_state: OpsConsistencyExternalFactStateView,
    pub recent_outbox_events: Vec<OutboxEventView>,
    pub recent_dead_letters: Vec<DeadLetterEventView>,
    pub recent_audit_traces: Vec<AuditTraceView>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OpsConsistencyReconcileRequest {
    pub ref_type: String,
    pub ref_id: String,
    pub mode: Option<String>,
    pub dry_run: Option<bool>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsConsistencyRepairRecommendationView {
    pub code: String,
    pub summary: String,
    pub priority: String,
    pub recommended_action: String,
    pub target_topic: Option<String>,
    pub related_gap_id: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsConsistencyReconcileView {
    pub ref_type: String,
    pub ref_id: String,
    pub mode: String,
    pub dry_run: bool,
    pub step_up_bound: bool,
    pub status: String,
    pub reconcile_target_topic: String,
    pub recommendation_count: i64,
    pub subject_snapshot: serde_json::Value,
    pub projection_gap_status_breakdown: serde_json::Value,
    pub related_projection_gaps: Vec<ChainProjectionGapView>,
    pub recommendations: Vec<OpsConsistencyRepairRecommendationView>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct TradeMonitorCheckpointQuery {
    pub checkpoint_code: Option<String>,
    pub checkpoint_status: Option<String>,
    pub lifecycle_stage: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl TradeMonitorCheckpointQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeMonitorOverviewView {
    pub order_id: String,
    pub request_id: String,
    pub trace_id: String,
    pub business_state: String,
    pub current_checkpoint_code: String,
    pub current_checkpoint_status: String,
    pub proof_commit_state: String,
    pub external_fact_status: String,
    pub reconcile_status: String,
    pub open_fairness_incident_count: i64,
    pub last_external_fact_at: Option<String>,
    pub last_chain_confirmed_at: Option<String>,
    pub last_observed_at: String,
    pub recent_checkpoints: Vec<TradeLifecycleCheckpointView>,
    pub recent_external_facts: Vec<ExternalFactReceiptView>,
    pub recent_fairness_incidents: Vec<FairnessIncidentView>,
    pub recent_projection_gaps: Vec<ChainProjectionGapView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeMonitorCheckpointPageView {
    pub order_id: String,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<TradeLifecycleCheckpointView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservabilityBackendStatusView {
    pub backend: ObservabilityBackendView,
    pub probe_status: String,
    pub checked_at: Option<String>,
    pub local_probe_url: Option<String>,
    pub http_status: Option<u16>,
    pub detail_url: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsAlertSummaryView {
    pub open_count: i64,
    pub acknowledged_count: i64,
    pub critical_count: i64,
    pub high_count: i64,
    pub latest_fired_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsServiceHealthView {
    pub service_name: String,
    pub status: String,
    pub metric_name: String,
    pub backend_key: String,
    pub observed_value: Option<f64>,
    pub checked_at: Option<String>,
    pub detail_url: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsSloSummaryView {
    pub total: i64,
    pub ok_count: i64,
    pub degraded_count: i64,
    pub breached_count: i64,
    pub items: Vec<SloView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsObservabilityOverviewView {
    pub backend_statuses: Vec<ObservabilityBackendStatusView>,
    pub alert_summary: OpsAlertSummaryView,
    pub key_services: Vec<OpsServiceHealthView>,
    pub slo_summary: OpsSloSummaryView,
    pub recent_incidents: Vec<IncidentTicketView>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct OpsLogMirrorQuery {
    pub service_name: Option<String>,
    pub log_level: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub query: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl OpsLogMirrorQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsLogMirrorPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<SystemLogMirrorView>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OpsLogExportRequest {
    pub reason: String,
    pub service_name: Option<String>,
    pub log_level: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsLogExportView {
    pub export_id: String,
    pub bucket_name: String,
    pub object_key: String,
    pub object_uri: String,
    pub object_hash: String,
    pub exported_count: i64,
    pub step_up_bound: bool,
    pub content_type: String,
    pub request_id: String,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsTraceLookupView {
    pub trace: TraceIndexView,
    pub related_log_count: i64,
    pub related_alert_count: i64,
    pub backend_status: Option<ObservabilityBackendStatusView>,
    pub tempo_link: Option<String>,
    pub grafana_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct OpsAlertQuery {
    pub alert_status: Option<String>,
    pub severity: Option<String>,
    pub source_backend_key: Option<String>,
    pub alert_type: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl OpsAlertQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsAlertPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<AlertEventView>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct OpsIncidentQuery {
    pub incident_status: Option<String>,
    pub severity: Option<String>,
    pub owner_role_key: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl OpsIncidentQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsIncidentPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<IncidentTicketView>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct OpsSloQuery {
    pub service_name: Option<String>,
    pub source_backend_key: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl OpsSloQuery {
    pub fn pagination(&self) -> Pagination {
        Pagination::from_query(Some(PaginationQuery {
            page: self.page,
            page_size: self.page_size,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsSloPageView {
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<SloView>,
}
