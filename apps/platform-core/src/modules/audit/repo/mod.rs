use audit_kit::{
    AnchorBatch, AuditEvent, EvidenceItem, EvidenceManifest, EvidenceManifestItem, EvidencePackage,
    LegalHold, ReplayJob, ReplayResult,
};
use db::{Error, GenericClient, Row};
use serde_json::{Map, Value};

use crate::modules::audit::domain::{
    AnchorBatchQuery, AuditTraceQuery, ChainProjectionGapQuery, ConsumerIdempotencyQuery,
    ExternalFactReceiptQuery, FairnessIncidentQuery, OpsAlertQuery, OpsDeadLetterQuery,
    OpsIncidentQuery, OpsLogMirrorQuery, OpsOutboxQuery, OpsSloQuery, TradeMonitorCheckpointQuery,
};
use crate::modules::audit::dto::AuditTraceView;

pub const INSERT_AUDIT_EVENT_SQL: &str = r#"
INSERT INTO audit.audit_event (
  event_schema_version,
  event_class,
  domain_name,
  ref_type,
  ref_id,
  actor_type,
  actor_id,
  actor_org_id,
  session_id,
  trusted_device_id,
  application_id,
  parent_audit_id,
  action_name,
  result_code,
  error_code,
  request_id,
  trace_id,
  source_ip,
  client_fingerprint,
  tx_hash,
  evidence_hash,
  payload_digest,
  auth_assurance_level,
  step_up_challenge_id,
  before_state_digest,
  after_state_digest,
  previous_event_hash,
  event_hash,
  evidence_manifest_id,
  anchor_policy,
  retention_class,
  legal_hold_status,
  sensitivity_level,
  ingested_at,
  event_time,
  metadata
) VALUES (
  $1, $2, $3, $4, $5::text::uuid, $6, $7::text::uuid, $8::text::uuid, $9::text::uuid,
  $10::text::uuid, $11::text::uuid, $12::text::uuid, $13, $14, $15, $16, $17, $18::inet, $19,
  $20, $21, $22, $23, $24::text::uuid, $25, $26, $27, $28, $29::text::uuid, $30, $31, $32,
  $33, COALESCE($34::timestamptz, now()), COALESCE($35::timestamptz, now()), $36::jsonb
)
"#;

pub const INSERT_EVIDENCE_ITEM_SQL: &str = r#"
INSERT INTO audit.evidence_item (
  item_type,
  ref_type,
  ref_id,
  object_uri,
  object_hash,
  content_type,
  size_bytes,
  source_system,
  storage_mode,
  retention_policy_id,
  worm_enabled,
  legal_hold_status,
  created_by,
  metadata
) VALUES (
  $1, $2, $3::text::uuid, $4, $5, $6, $7, $8, $9, $10::text::uuid, $11, $12, $13::text::uuid, $14::jsonb
)
RETURNING
  evidence_item_id::text,
  item_type,
  ref_type,
  ref_id::text,
  object_uri,
  object_hash,
  content_type,
  size_bytes,
  source_system,
  storage_mode,
  retention_policy_id::text,
  worm_enabled,
  legal_hold_status,
  created_by::text,
  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  metadata
"#;

pub const INSERT_EVIDENCE_MANIFEST_SQL: &str = r#"
INSERT INTO audit.evidence_manifest (
  manifest_scope,
  ref_type,
  ref_id,
  manifest_hash,
  item_count,
  storage_uri,
  created_by,
  metadata
) VALUES (
  $1, $2, $3::text::uuid, $4, $5, $6, $7::text::uuid, $8::jsonb
)
RETURNING
  evidence_manifest_id::text,
  manifest_scope,
  ref_type,
  ref_id::text,
  manifest_hash,
  item_count,
  storage_uri,
  created_by::text,
  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  metadata
"#;

pub const INSERT_EVIDENCE_MANIFEST_ITEM_SQL: &str = r#"
INSERT INTO audit.evidence_manifest_item (
  evidence_manifest_id,
  evidence_item_id,
  item_digest,
  ordinal_no
) VALUES (
  $1::text::uuid, $2::text::uuid, $3, $4
)
RETURNING
  evidence_manifest_item_id::text,
  evidence_manifest_id::text,
  evidence_item_id::text,
  item_digest,
  ordinal_no,
  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
"#;

pub const INSERT_EVIDENCE_PACKAGE_SQL: &str = r#"
INSERT INTO audit.evidence_package (
  evidence_package_id,
  package_type,
  ref_type,
  ref_id,
  evidence_manifest_id,
  package_digest,
  storage_uri,
  created_by,
  retention_class,
  masked_level,
  access_mode,
  legal_hold_status
) VALUES (
  $1::text::uuid,
  $2,
  $3,
  $4::text::uuid,
  $5::text::uuid,
  $6,
  $7,
  $8::text::uuid,
  $9,
  $10,
  $11,
  $12
)
RETURNING
  evidence_package_id::text,
  package_type,
  ref_type,
  ref_id::text,
  evidence_manifest_id::text,
  package_digest,
  storage_uri,
  created_by::text,
  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  retention_class,
  legal_hold_status
"#;

pub const INSERT_REPLAY_JOB_SQL: &str = r#"
INSERT INTO audit.replay_job (
  replay_job_id,
  replay_type,
  ref_type,
  ref_id,
  dry_run,
  status,
  requested_by,
  step_up_challenge_id,
  request_reason,
  options_json,
  started_at,
  finished_at
) VALUES (
  $1::text::uuid,
  $2,
  $3,
  $4::text::uuid,
  $5,
  $6,
  $7::text::uuid,
  $8::text::uuid,
  $9,
  $10::jsonb,
  $11::timestamptz,
  $12::timestamptz
)
RETURNING
  replay_job_id::text,
  replay_type,
  ref_type,
  ref_id::text,
  dry_run,
  status,
  requested_by::text,
  step_up_challenge_id::text,
  request_reason,
  options_json,
  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  to_char(finished_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
"#;

pub const INSERT_REPLAY_RESULT_SQL: &str = r#"
INSERT INTO audit.replay_result (
  replay_job_id,
  step_name,
  result_code,
  expected_digest,
  actual_digest,
  diff_summary
) VALUES (
  $1::text::uuid,
  $2,
  $3,
  $4,
  $5,
  $6::jsonb
)
RETURNING
  replay_result_id::text,
  replay_job_id::text,
  step_name,
  result_code,
  expected_digest,
  actual_digest,
  diff_summary,
  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
"#;

pub const INSERT_LEGAL_HOLD_SQL: &str = r#"
INSERT INTO audit.legal_hold (
  legal_hold_id,
  hold_scope_type,
  hold_scope_id,
  reason_code,
  status,
  retention_policy_id,
  requested_by,
  approved_by,
  hold_until,
  metadata
) VALUES (
  $1::text::uuid,
  $2,
  $3::text::uuid,
  $4,
  $5,
  $6::text::uuid,
  $7::text::uuid,
  $8::text::uuid,
  $9::timestamptz,
  $10::jsonb
)
RETURNING
  legal_hold_id::text,
  hold_scope_type,
  hold_scope_id::text,
  reason_code,
  status,
  retention_policy_id::text,
  requested_by::text,
  approved_by::text,
  to_char(hold_until AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  to_char(released_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
  metadata
"#;

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventInsert {
    pub event_schema_version: String,
    pub event_class: String,
    pub domain_name: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub actor_org_id: Option<String>,
    pub session_id: Option<String>,
    pub trusted_device_id: Option<String>,
    pub application_id: Option<String>,
    pub parent_audit_id: Option<String>,
    pub action_name: String,
    pub result_code: String,
    pub error_code: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub source_ip: Option<String>,
    pub client_fingerprint: Option<String>,
    pub tx_hash: Option<String>,
    pub evidence_hash: Option<String>,
    pub payload_digest: Option<String>,
    pub auth_assurance_level: Option<String>,
    pub step_up_challenge_id: Option<String>,
    pub before_state_digest: Option<String>,
    pub after_state_digest: Option<String>,
    pub previous_event_hash: Option<String>,
    pub event_hash: Option<String>,
    pub evidence_manifest_id: Option<String>,
    pub anchor_policy: String,
    pub retention_class: String,
    pub legal_hold_status: String,
    pub sensitivity_level: String,
    pub ingested_at: Option<String>,
    pub event_time: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceItemInsert {
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
    pub worm_enabled: bool,
    pub legal_hold_status: String,
    pub created_by: Option<String>,
    pub metadata: Value,
}

impl From<&EvidenceItem> for EvidenceItemInsert {
    fn from(item: &EvidenceItem) -> Self {
        Self {
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
            worm_enabled: item.worm_enabled,
            legal_hold_status: item.legal_hold_status.clone(),
            created_by: item.created_by.clone(),
            metadata: metadata_value(item.metadata.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceManifestInsert {
    pub manifest_scope: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub manifest_hash: String,
    pub item_count: i32,
    pub storage_uri: Option<String>,
    pub created_by: Option<String>,
    pub metadata: Value,
}

impl From<&EvidenceManifest> for EvidenceManifestInsert {
    fn from(manifest: &EvidenceManifest) -> Self {
        Self {
            manifest_scope: manifest.manifest_scope.clone(),
            ref_type: manifest.ref_type.clone(),
            ref_id: manifest.ref_id.clone(),
            manifest_hash: manifest.manifest_hash.clone(),
            item_count: manifest.item_count,
            storage_uri: manifest.storage_uri.clone(),
            created_by: manifest.created_by.clone(),
            metadata: metadata_value(manifest.metadata.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceManifestItemInsert {
    pub evidence_manifest_id: Option<String>,
    pub evidence_item_id: Option<String>,
    pub item_digest: String,
    pub ordinal_no: i32,
}

impl From<&EvidenceManifestItem> for EvidenceManifestItemInsert {
    fn from(item: &EvidenceManifestItem) -> Self {
        Self {
            evidence_manifest_id: item.evidence_manifest_id.clone(),
            evidence_item_id: item.evidence_item_id.clone(),
            item_digest: item.item_digest.clone(),
            ordinal_no: item.ordinal_no,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderAuditScope {
    pub order_id: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub status: String,
    pub payment_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditTracePage {
    pub total: i64,
    pub items: Vec<AuditTraceView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccessAuditInsert {
    pub accessor_user_id: Option<String>,
    pub accessor_role_key: Option<String>,
    pub access_mode: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub masked_view: bool,
    pub breakglass_reason: Option<String>,
    pub step_up_challenge_id: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePackageInsert {
    pub evidence_package_id: Option<String>,
    pub package_type: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub evidence_manifest_id: Option<String>,
    pub package_digest: Option<String>,
    pub storage_uri: Option<String>,
    pub created_by: Option<String>,
    pub retention_class: String,
    pub masked_level: String,
    pub access_mode: String,
    pub legal_hold_status: String,
    pub metadata: Value,
}

impl From<&EvidencePackage> for EvidencePackageInsert {
    fn from(package: &EvidencePackage) -> Self {
        let masked_level = package
            .metadata
            .get("masked_level")
            .and_then(|value| value.as_str())
            .unwrap_or("summary")
            .to_string();
        let access_mode = package
            .metadata
            .get("access_mode")
            .and_then(|value| value.as_str())
            .unwrap_or("export")
            .to_string();
        Self {
            evidence_package_id: package.evidence_package_id.clone(),
            package_type: package.package_type.clone(),
            ref_type: package.ref_type.clone(),
            ref_id: package.ref_id.clone(),
            evidence_manifest_id: package.evidence_manifest_id.clone(),
            package_digest: package.package_digest.clone(),
            storage_uri: package.storage_uri.clone(),
            created_by: package.created_by.clone(),
            retention_class: package.retention_class.clone(),
            masked_level,
            access_mode,
            legal_hold_status: package.legal_hold_status.clone(),
            metadata: metadata_value(package.metadata.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayJobInsert {
    pub replay_job_id: Option<String>,
    pub replay_type: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub dry_run: bool,
    pub status: String,
    pub requested_by: Option<String>,
    pub step_up_challenge_id: Option<String>,
    pub request_reason: Option<String>,
    pub options_json: Value,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

impl From<&ReplayJob> for ReplayJobInsert {
    fn from(job: &ReplayJob) -> Self {
        Self {
            replay_job_id: job.replay_job_id.clone(),
            replay_type: job.replay_type.clone(),
            ref_type: job.ref_type.clone(),
            ref_id: job.ref_id.clone(),
            dry_run: job.dry_run,
            status: job.status.clone(),
            requested_by: job.requested_by.clone(),
            step_up_challenge_id: job.step_up_challenge_id.clone(),
            request_reason: job.request_reason.clone(),
            options_json: metadata_value(job.options_json.clone()),
            started_at: job.started_at.clone(),
            finished_at: job.finished_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayResultInsert {
    pub replay_job_id: Option<String>,
    pub step_name: String,
    pub result_code: String,
    pub expected_digest: Option<String>,
    pub actual_digest: Option<String>,
    pub diff_summary: Value,
}

impl From<&ReplayResult> for ReplayResultInsert {
    fn from(result: &ReplayResult) -> Self {
        Self {
            replay_job_id: result.replay_job_id.clone(),
            step_name: result.step_name.clone(),
            result_code: result.result_code.clone(),
            expected_digest: result.expected_digest.clone(),
            actual_digest: result.actual_digest.clone(),
            diff_summary: metadata_value(result.diff_summary.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegalHoldInsert {
    pub legal_hold_id: Option<String>,
    pub hold_scope_type: String,
    pub hold_scope_id: Option<String>,
    pub reason_code: String,
    pub status: String,
    pub retention_policy_id: Option<String>,
    pub requested_by: Option<String>,
    pub approved_by: Option<String>,
    pub hold_until: Option<String>,
    pub metadata: Value,
}

impl From<&LegalHold> for LegalHoldInsert {
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
            metadata: metadata_value(hold.metadata.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayJobDetail {
    pub replay_job: ReplayJob,
    pub results: Vec<ReplayResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnchorBatchPage {
    pub total: i64,
    pub items: Vec<AnchorBatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutboxPublishAttemptRecord {
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
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutboxEventRecord {
    pub outbox_event_id: Option<String>,
    pub aggregate_type: String,
    pub aggregate_id: Option<String>,
    pub event_type: String,
    pub payload: Value,
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
    pub latest_publish_attempt: Option<OutboxPublishAttemptRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutboxEventPage {
    pub total: i64,
    pub items: Vec<OutboxEventRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsumerIdempotencyRecord {
    pub consumer_idempotency_record_id: Option<String>,
    pub consumer_name: String,
    pub event_id: String,
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<String>,
    pub trace_id: Option<String>,
    pub result_code: String,
    pub processed_at: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsumerIdempotencyPage {
    pub total: i64,
    pub items: Vec<ConsumerIdempotencyRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeadLetterEventRecord {
    pub dead_letter_event_id: Option<String>,
    pub outbox_event_id: Option<String>,
    pub aggregate_type: String,
    pub aggregate_id: Option<String>,
    pub event_type: String,
    pub payload: Value,
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
    pub consumer_idempotency_records: Vec<ConsumerIdempotencyRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeadLetterEventPage {
    pub total: i64,
    pub items: Vec<DeadLetterEventRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalFactReceiptRecord {
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
    pub receipt_payload: Value,
    pub receipt_hash: Option<String>,
    pub occurred_at: Option<String>,
    pub received_at: Option<String>,
    pub confirmed_at: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalFactReceiptPage {
    pub total: i64,
    pub items: Vec<ExternalFactReceiptRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainProjectionGapRecord {
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
    pub resolution_summary: Value,
    pub metadata: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainProjectionGapPage {
    pub total: i64,
    pub items: Vec<ChainProjectionGapRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TradeLifecycleCheckpointRecord {
    pub trade_lifecycle_checkpoint_id: Option<String>,
    pub monitoring_policy_profile_id: Option<String>,
    pub order_id: Option<String>,
    pub ref_domain: String,
    pub ref_type: String,
    pub ref_id: String,
    pub checkpoint_code: String,
    pub lifecycle_stage: String,
    pub checkpoint_status: String,
    pub expected_by: Option<String>,
    pub occurred_at: Option<String>,
    pub source_type: String,
    pub source_ref_type: Option<String>,
    pub source_ref_id: Option<String>,
    pub related_tx_hash: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TradeLifecycleCheckpointPage {
    pub total: i64,
    pub items: Vec<TradeLifecycleCheckpointRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FairnessIncidentRecord {
    pub fairness_incident_id: Option<String>,
    pub order_id: Option<String>,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub incident_type: String,
    pub severity: String,
    pub lifecycle_stage: String,
    pub detected_by_type: String,
    pub source_checkpoint_id: Option<String>,
    pub source_receipt_id: Option<String>,
    pub fairness_incident_status: String,
    pub auto_action_code: Option<String>,
    pub assigned_role_key: Option<String>,
    pub assigned_user_id: Option<String>,
    pub resolution_summary: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: Value,
    pub created_at: Option<String>,
    pub closed_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FairnessIncidentPage {
    pub total: i64,
    pub items: Vec<FairnessIncidentRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilityBackendRecord {
    pub observability_backend_id: Option<String>,
    pub backend_key: String,
    pub backend_type: String,
    pub endpoint_uri: Option<String>,
    pub auth_mode: String,
    pub enabled: bool,
    pub stage_from: String,
    pub capability_json: Value,
    pub metadata: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemLogMirrorRecord {
    pub system_log_id: Option<String>,
    pub service_name: String,
    pub logger_name: Option<String>,
    pub log_level: String,
    pub severity_number: Option<i32>,
    pub environment_code: String,
    pub host_name: Option<String>,
    pub node_name: Option<String>,
    pub pod_name: Option<String>,
    pub backend_type: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub message_text: String,
    pub structured_payload: Value,
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub masked_status: String,
    pub retention_class: String,
    pub legal_hold_status: String,
    pub resource_attrs: Value,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemLogMirrorPage {
    pub total: i64,
    pub items: Vec<SystemLogMirrorRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceIndexRecord {
    pub trace_index_id: Option<String>,
    pub trace_id: String,
    pub traceparent: Option<String>,
    pub backend_key: Option<String>,
    pub root_service_name: Option<String>,
    pub root_span_name: Option<String>,
    pub request_id: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub status: String,
    pub span_count: Option<i32>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub metadata: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEventRecord {
    pub alert_event_id: Option<String>,
    pub alert_rule_id: Option<String>,
    pub source_backend_key: Option<String>,
    pub fingerprint: String,
    pub alert_type: String,
    pub severity: String,
    pub title_text: String,
    pub summary_text: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub labels_json: Value,
    pub annotations_json: Value,
    pub status: String,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<String>,
    pub fired_at: Option<String>,
    pub resolved_at: Option<String>,
    pub metadata: Value,
    pub incident_ticket_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEventPage {
    pub total: i64,
    pub items: Vec<AlertEventRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncidentTicketRecord {
    pub incident_ticket_id: Option<String>,
    pub incident_key: String,
    pub source_alert_event_id: Option<String>,
    pub severity: String,
    pub title_text: String,
    pub summary_text: Option<String>,
    pub status: String,
    pub owner_role_key: Option<String>,
    pub owner_user_id: Option<String>,
    pub runbook_uri: Option<String>,
    pub impact_summary: Option<String>,
    pub root_cause_summary: Option<String>,
    pub latest_event_type: Option<String>,
    pub latest_event_note: Option<String>,
    pub metadata: Value,
    pub started_at: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncidentTicketPage {
    pub total: i64,
    pub items: Vec<IncidentTicketRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRecord {
    pub slo_definition_id: Option<String>,
    pub slo_key: String,
    pub service_name: String,
    pub indicator_type: String,
    pub objective_value: String,
    pub window_code: String,
    pub source_backend_key: Option<String>,
    pub alert_rule_id: Option<String>,
    pub status: String,
    pub current_value: Option<String>,
    pub budget_remaining: Option<String>,
    pub snapshot_status: Option<String>,
    pub window_started_at: Option<String>,
    pub window_ended_at: Option<String>,
    pub metadata: Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPage {
    pub total: i64,
    pub items: Vec<SloRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainAnchorRecord {
    pub chain_anchor_id: Option<String>,
    pub chain_id: String,
    pub anchor_type: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub digest: String,
    pub tx_hash: Option<String>,
    pub status: String,
    pub anchored_at: Option<String>,
    pub created_at: Option<String>,
    pub authority_model: String,
    pub reconcile_status: String,
    pub last_reconciled_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsistencySubjectRecord {
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
    pub snapshot: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemLogInsert {
    pub service_name: String,
    pub log_level: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub message_text: String,
    pub structured_payload: Value,
}

impl From<&AuditEvent> for AuditEventInsert {
    fn from(event: &AuditEvent) -> Self {
        Self {
            event_schema_version: event.event_schema_version.clone(),
            event_class: event.event_class.clone(),
            domain_name: event.domain_name.clone(),
            ref_type: event.ref_type.clone(),
            ref_id: event.ref_id.clone(),
            actor_type: event.actor_type.clone(),
            actor_id: event.actor_id.clone(),
            actor_org_id: event.actor_org_id.clone(),
            session_id: event.session_id.clone(),
            trusted_device_id: event.trusted_device_id.clone(),
            application_id: event.application_id.clone(),
            parent_audit_id: event.parent_audit_id.clone(),
            action_name: event.action_name.clone(),
            result_code: event.result_code.clone(),
            error_code: event.error_code.clone(),
            request_id: event.request_id.clone(),
            trace_id: event.trace_id.clone(),
            source_ip: event.source_ip.clone(),
            client_fingerprint: event.client_fingerprint.clone(),
            tx_hash: event.tx_hash.clone(),
            evidence_hash: event.evidence_hash.clone(),
            payload_digest: event.payload_digest.clone(),
            auth_assurance_level: event.auth_assurance_level.clone(),
            step_up_challenge_id: event.step_up_challenge_id.clone(),
            before_state_digest: event.before_state_digest.clone(),
            after_state_digest: event.after_state_digest.clone(),
            previous_event_hash: event.previous_event_hash.clone(),
            event_hash: event.event_hash.clone(),
            evidence_manifest_id: event.evidence_manifest_id.clone(),
            anchor_policy: event.anchor_policy.clone(),
            retention_class: event.retention_class.clone(),
            legal_hold_status: event.legal_hold_status.clone(),
            sensitivity_level: event.sensitivity_level.clone(),
            ingested_at: event.ingested_at.clone(),
            event_time: event.occurred_at.clone(),
            metadata: storage_metadata(event),
        }
    }
}

pub fn metadata_object(raw: Value) -> Map<String, Value> {
    match raw {
        Value::Object(map) => map,
        Value::Null => Map::new(),
        raw => {
            let mut map = Map::new();
            map.insert("raw_metadata".to_string(), raw);
            map
        }
    }
}

pub fn metadata_value(raw: Value) -> Value {
    Value::Object(metadata_object(raw))
}

pub async fn insert_audit_event(
    client: &(impl GenericClient + Sync),
    event: &AuditEvent,
) -> Result<(), Error> {
    let insert = AuditEventInsert::from(event);
    client
        .execute(
            INSERT_AUDIT_EVENT_SQL,
            &[
                &insert.event_schema_version,
                &insert.event_class,
                &insert.domain_name,
                &insert.ref_type,
                &insert.ref_id,
                &insert.actor_type,
                &insert.actor_id,
                &insert.actor_org_id,
                &insert.session_id,
                &insert.trusted_device_id,
                &insert.application_id,
                &insert.parent_audit_id,
                &insert.action_name,
                &insert.result_code,
                &insert.error_code,
                &insert.request_id,
                &insert.trace_id,
                &insert.source_ip,
                &insert.client_fingerprint,
                &insert.tx_hash,
                &insert.evidence_hash,
                &insert.payload_digest,
                &insert.auth_assurance_level,
                &insert.step_up_challenge_id,
                &insert.before_state_digest,
                &insert.after_state_digest,
                &insert.previous_event_hash,
                &insert.event_hash,
                &insert.evidence_manifest_id,
                &insert.anchor_policy,
                &insert.retention_class,
                &insert.legal_hold_status,
                &insert.sensitivity_level,
                &insert.ingested_at,
                &insert.event_time,
                &insert.metadata,
            ],
        )
        .await?;
    Ok(())
}

pub async fn insert_evidence_package(
    client: &(impl GenericClient + Sync),
    package: &EvidencePackage,
) -> Result<EvidencePackage, Error> {
    let insert = EvidencePackageInsert::from(package);
    let row = client
        .query_one(
            INSERT_EVIDENCE_PACKAGE_SQL,
            &[
                &insert.evidence_package_id,
                &insert.package_type,
                &insert.ref_type,
                &insert.ref_id,
                &insert.evidence_manifest_id,
                &insert.package_digest,
                &insert.storage_uri,
                &insert.created_by,
                &insert.retention_class,
                &insert.masked_level,
                &insert.access_mode,
                &insert.legal_hold_status,
            ],
        )
        .await?;
    let mut stored = parse_evidence_package_row(&row);
    stored.metadata = insert.metadata;
    Ok(stored)
}

pub async fn insert_replay_job(
    client: &(impl GenericClient + Sync),
    replay_job: &ReplayJob,
) -> Result<ReplayJob, Error> {
    let insert = ReplayJobInsert::from(replay_job);
    let row = client
        .query_one(
            INSERT_REPLAY_JOB_SQL,
            &[
                &insert.replay_job_id,
                &insert.replay_type,
                &insert.ref_type,
                &insert.ref_id,
                &insert.dry_run,
                &insert.status,
                &insert.requested_by,
                &insert.step_up_challenge_id,
                &insert.request_reason,
                &insert.options_json,
                &insert.started_at,
                &insert.finished_at,
            ],
        )
        .await?;
    Ok(parse_replay_job_row(&row))
}

pub async fn insert_replay_result(
    client: &(impl GenericClient + Sync),
    replay_result: &ReplayResult,
) -> Result<ReplayResult, Error> {
    let insert = ReplayResultInsert::from(replay_result);
    let row = client
        .query_one(
            INSERT_REPLAY_RESULT_SQL,
            &[
                &insert.replay_job_id,
                &insert.step_name,
                &insert.result_code,
                &insert.expected_digest,
                &insert.actual_digest,
                &insert.diff_summary,
            ],
        )
        .await?;
    Ok(parse_replay_result_row(&row))
}

pub async fn insert_legal_hold(
    client: &(impl GenericClient + Sync),
    legal_hold: &LegalHold,
) -> Result<LegalHold, Error> {
    let insert = LegalHoldInsert::from(legal_hold);
    let row = client
        .query_one(
            INSERT_LEGAL_HOLD_SQL,
            &[
                &insert.legal_hold_id,
                &insert.hold_scope_type,
                &insert.hold_scope_id,
                &insert.reason_code,
                &insert.status,
                &insert.retention_policy_id,
                &insert.requested_by,
                &insert.approved_by,
                &insert.hold_until,
                &insert.metadata,
            ],
        )
        .await?;
    Ok(parse_legal_hold_row(&row))
}

pub async fn load_legal_hold(
    client: &(impl GenericClient + Sync),
    legal_hold_id: &str,
) -> Result<Option<LegalHold>, Error> {
    let row = client
        .query_opt(
            "SELECT
               legal_hold_id::text,
               hold_scope_type,
               hold_scope_id::text,
               reason_code,
               status,
               retention_policy_id::text,
               requested_by::text,
               approved_by::text,
               to_char(hold_until AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(released_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata
             FROM audit.legal_hold
             WHERE legal_hold_id = $1::text::uuid",
            &[&legal_hold_id],
        )
        .await?;
    Ok(row.map(|row| parse_legal_hold_row(&row)))
}

pub async fn load_active_legal_hold_for_scope(
    client: &(impl GenericClient + Sync),
    hold_scope_type: &str,
    hold_scope_id: &str,
) -> Result<Option<LegalHold>, Error> {
    let row = client
        .query_opt(
            "SELECT
               legal_hold_id::text,
               hold_scope_type,
               hold_scope_id::text,
               reason_code,
               status,
               retention_policy_id::text,
               requested_by::text,
               approved_by::text,
               to_char(hold_until AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(released_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata
             FROM audit.legal_hold
             WHERE hold_scope_type = $1
               AND hold_scope_id = $2::text::uuid
               AND status = 'active'
             ORDER BY created_at DESC, legal_hold_id DESC
             LIMIT 1",
            &[&hold_scope_type, &hold_scope_id],
        )
        .await?;
    Ok(row.map(|row| parse_legal_hold_row(&row)))
}

pub async fn release_legal_hold(
    client: &(impl GenericClient + Sync),
    legal_hold_id: &str,
    approved_by: Option<&str>,
    released_at: &str,
    metadata_patch: &Value,
) -> Result<Option<LegalHold>, Error> {
    let row = client
        .query_opt(
            "UPDATE audit.legal_hold
             SET status = 'released',
                 approved_by = COALESCE($2::text::uuid, approved_by),
                 released_at = COALESCE(released_at, $3::timestamptz),
                 metadata = COALESCE(metadata, '{}'::jsonb) || $4::jsonb
             WHERE legal_hold_id = $1::text::uuid
               AND status = 'active'
             RETURNING
               legal_hold_id::text,
               hold_scope_type,
               hold_scope_id::text,
               reason_code,
               status,
               retention_policy_id::text,
               requested_by::text,
               approved_by::text,
               to_char(hold_until AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(released_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata",
            &[&legal_hold_id, &approved_by, &released_at, metadata_patch],
        )
        .await?;
    Ok(row.map(|row| parse_legal_hold_row(&row)))
}

pub async fn load_anchor_batch(
    client: &(impl GenericClient + Sync),
    anchor_batch_id: &str,
) -> Result<Option<AnchorBatch>, Error> {
    let row = client
        .query_opt(
            "SELECT
               ab.anchor_batch_id::text,
               ab.batch_scope,
               ab.chain_id,
               ab.record_count,
               ab.batch_root,
               to_char(ab.window_started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ab.window_ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ab.status,
               ab.chain_anchor_id::text,
               ab.created_by::text,
               to_char(ab.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ab.anchored_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ab.metadata,
               to_char(ab.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ca.tx_hash,
               ca.status,
               ca.authority_model,
               ca.reconcile_status,
               to_char(ca.last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM audit.anchor_batch ab
             LEFT JOIN chain.chain_anchor ca
               ON ca.chain_anchor_id = ab.chain_anchor_id
             WHERE ab.anchor_batch_id = $1::text::uuid",
            &[&anchor_batch_id],
        )
        .await?;
    Ok(row.map(|row| parse_anchor_batch_row(&row)))
}

pub async fn search_anchor_batches(
    client: &(impl GenericClient + Sync),
    query: &AnchorBatchQuery,
    limit: i64,
    offset: i64,
) -> Result<AnchorBatchPage, Error> {
    let rows = client
        .query(
            "SELECT
               ab.anchor_batch_id::text,
               ab.batch_scope,
               ab.chain_id,
               ab.record_count,
               ab.batch_root,
               to_char(ab.window_started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ab.window_ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ab.status,
               ab.chain_anchor_id::text,
               ab.created_by::text,
               to_char(ab.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ab.anchored_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ab.metadata,
               to_char(ab.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ca.tx_hash,
               ca.status,
               ca.authority_model,
               ca.reconcile_status,
               to_char(ca.last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM audit.anchor_batch ab
             LEFT JOIN chain.chain_anchor ca
               ON ca.chain_anchor_id = ab.chain_anchor_id
             WHERE ($1::text IS NULL OR ab.status = $1)
               AND ($2::text IS NULL OR ab.batch_scope = $2)
               AND ($3::text IS NULL OR ab.chain_id = $3)
             ORDER BY ab.created_at DESC, ab.anchor_batch_id DESC
             LIMIT $4
             OFFSET $5",
            &[
                &query.anchor_status,
                &query.batch_scope,
                &query.chain_id,
                &limit,
                &offset,
            ],
        )
        .await?;
    let total = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.anchor_batch ab
             WHERE ($1::text IS NULL OR ab.status = $1)
               AND ($2::text IS NULL OR ab.batch_scope = $2)
               AND ($3::text IS NULL OR ab.chain_id = $3)",
            &[&query.anchor_status, &query.batch_scope, &query.chain_id],
        )
        .await?
        .get(0);
    Ok(AnchorBatchPage {
        total,
        items: rows.iter().map(parse_anchor_batch_row).collect(),
    })
}

pub async fn mark_anchor_batch_retry_requested(
    client: &(impl GenericClient + Sync),
    anchor_batch_id: &str,
    metadata_patch: &Value,
    retried_at: &str,
) -> Result<bool, Error> {
    let updated = client
        .execute(
            "UPDATE audit.anchor_batch
             SET status = 'retry_requested',
                 metadata = COALESCE(metadata, '{}'::jsonb) || $2::jsonb,
                 updated_at = COALESCE($3::timestamptz, now())
             WHERE anchor_batch_id = $1::text::uuid
               AND status = 'failed'",
            &[&anchor_batch_id, metadata_patch, &retried_at],
        )
        .await?;
    Ok(updated > 0)
}

pub async fn load_replay_job_detail(
    client: &(impl GenericClient + Sync),
    replay_job_id: &str,
) -> Result<Option<ReplayJobDetail>, Error> {
    let replay_job_row = client
        .query_opt(
            "SELECT
               replay_job_id::text,
               replay_type,
               ref_type,
               ref_id::text,
               dry_run,
               status,
               requested_by::text,
               step_up_challenge_id::text,
               request_reason,
               options_json,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(finished_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM audit.replay_job
             WHERE replay_job_id = $1::text::uuid",
            &[&replay_job_id],
        )
        .await?;

    let Some(replay_job_row) = replay_job_row else {
        return Ok(None);
    };

    let results = client
        .query(
            "SELECT
               replay_result_id::text,
               replay_job_id::text,
               step_name,
               result_code,
               expected_digest,
               actual_digest,
               diff_summary,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM audit.replay_result
             WHERE replay_job_id = $1::text::uuid
             ORDER BY created_at ASC, replay_result_id ASC",
            &[&replay_job_id],
        )
        .await?
        .iter()
        .map(parse_replay_result_row)
        .collect();

    Ok(Some(ReplayJobDetail {
        replay_job: parse_replay_job_row(&replay_job_row),
        results,
    }))
}

pub async fn load_order_audit_scope(
    client: &(impl GenericClient + Sync),
    order_id: &str,
) -> Result<Option<OrderAuditScope>, Error> {
    let row = client
        .query_opt(
            "SELECT
               order_id::text,
               buyer_org_id::text,
               seller_org_id::text,
               status,
               payment_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await?;
    Ok(row.map(|row| OrderAuditScope {
        order_id: row.get(0),
        buyer_org_id: row.get(1),
        seller_org_id: row.get(2),
        status: row.get(3),
        payment_status: row.get(4),
    }))
}

pub async fn load_outbox_event_by_id(
    client: &(impl GenericClient + Sync),
    outbox_event_id: &str,
) -> Result<Option<OutboxEventRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               oe.outbox_event_id::text,
               oe.aggregate_type,
               oe.aggregate_id::text,
               oe.event_type,
               oe.payload,
               oe.status,
               oe.retry_count,
               oe.max_retries,
               to_char(oe.available_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(oe.published_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(oe.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               oe.request_id,
               oe.trace_id,
               oe.idempotency_key,
               oe.authority_scope,
               oe.source_of_truth,
               oe.proof_commit_policy,
               oe.target_bus,
               oe.target_topic,
               oe.partition_key,
               oe.ordering_key,
               oe.payload_hash,
               oe.last_error_code,
               oe.last_error_message,
               opa.outbox_publish_attempt_id::text,
               opa.worker_id,
               opa.target_bus,
               opa.target_topic,
               opa.attempt_no,
               opa.result_code,
               opa.error_code,
               opa.error_message,
               to_char(opa.attempted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(opa.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               opa.metadata
             FROM ops.outbox_event oe
             LEFT JOIN LATERAL (
               SELECT
                 outbox_publish_attempt_id,
                 worker_id,
                 target_bus,
                 target_topic,
                 attempt_no,
                 result_code,
                 error_code,
                 error_message,
                 attempted_at,
                 completed_at,
                 metadata
               FROM ops.outbox_publish_attempt
               WHERE outbox_event_id = oe.outbox_event_id
               ORDER BY attempt_no DESC, attempted_at DESC, outbox_publish_attempt_id DESC
               LIMIT 1
             ) opa ON true
             WHERE oe.outbox_event_id = $1::text::uuid",
            &[&outbox_event_id],
        )
        .await?;
    Ok(row.map(|row| parse_outbox_event_row(&row)))
}

pub async fn load_latest_dead_letter_by_outbox_event_id(
    client: &(impl GenericClient + Sync),
    outbox_event_id: &str,
) -> Result<Option<DeadLetterEventRecord>, Error> {
    let rows = client
        .query(
            "SELECT
               dead_letter_event_id::text,
               outbox_event_id::text,
               aggregate_type,
               aggregate_id::text,
               event_type,
               payload,
               failed_reason,
               request_id,
               trace_id,
               authority_scope,
               source_of_truth,
               target_bus,
               target_topic,
               failure_stage,
               to_char(first_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               reprocess_status,
               to_char(reprocessed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.dead_letter_event
             WHERE outbox_event_id = $1::text::uuid
             ORDER BY created_at DESC, dead_letter_event_id DESC
             LIMIT 1",
            &[&outbox_event_id],
        )
        .await?;
    let Some(row) = rows.first() else {
        return Ok(None);
    };
    let idempotency_map =
        load_consumer_idempotency_for_event_ids(client, &[outbox_event_id.to_string()]).await?;
    let consumer_records = idempotency_map
        .get(outbox_event_id)
        .cloned()
        .unwrap_or_default();
    Ok(Some(parse_dead_letter_row(row, consumer_records)))
}

pub async fn load_audit_trace_by_id(
    client: &(impl GenericClient + Sync),
    audit_id: &str,
) -> Result<Option<AuditTraceView>, Error> {
    let row = client
        .query_opt(
            "SELECT
               audit_id::text,
               event_schema_version,
               event_class,
               domain_name,
               ref_type,
               ref_id::text,
               actor_id::text,
               actor_org_id::text,
               action_name,
               result_code,
               error_code,
               request_id,
               trace_id,
               tx_hash,
               evidence_manifest_id::text,
               event_hash,
               to_char(event_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM audit.audit_event
             WHERE audit_id = $1::text::uuid",
            &[&audit_id],
        )
        .await?;
    Ok(row.map(|row| parse_audit_trace_row(&row)))
}

pub async fn load_latest_audit_trace_by_tx_hash(
    client: &(impl GenericClient + Sync),
    tx_hash: &str,
) -> Result<Option<AuditTraceView>, Error> {
    let row = client
        .query_opt(
            "SELECT
               audit_id::text,
               event_schema_version,
               event_class,
               domain_name,
               ref_type,
               ref_id::text,
               actor_id::text,
               actor_org_id::text,
               action_name,
               result_code,
               error_code,
               request_id,
               trace_id,
               tx_hash,
               evidence_manifest_id::text,
               event_hash,
               to_char(event_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM audit.audit_event
             WHERE tx_hash = $1
             ORDER BY event_time DESC, audit_id DESC
             LIMIT 1",
            &[&tx_hash],
        )
        .await?;
    Ok(row.map(|row| parse_audit_trace_row(&row)))
}

pub async fn load_chain_anchor_by_tx_hash(
    client: &(impl GenericClient + Sync),
    tx_hash: &str,
) -> Result<Option<ChainAnchorRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               chain_anchor_id::text,
               chain_id,
               anchor_type,
               ref_type,
               ref_id::text,
               digest,
               tx_hash,
               status,
               to_char(anchored_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               authority_model,
               reconcile_status,
               to_char(last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM chain.chain_anchor
             WHERE tx_hash = $1
             ORDER BY created_at DESC, chain_anchor_id DESC
             LIMIT 1",
            &[&tx_hash],
        )
        .await?;
    Ok(row.map(|row| parse_chain_anchor_row(&row)))
}

pub async fn load_latest_chain_projection_gap_by_tx_hash(
    client: &(impl GenericClient + Sync),
    tx_hash: &str,
) -> Result<Option<ChainProjectionGapRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               chain_projection_gap_id::text,
               aggregate_type,
               aggregate_id::text,
               order_id::text,
               chain_id,
               source_event_type,
               expected_tx_id,
               projected_tx_hash,
               gap_type,
               gap_status,
               to_char(first_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               outbox_event_id::text,
               anchor_id::text,
               resolution_summary,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.chain_projection_gap
             WHERE projected_tx_hash = $1
             ORDER BY created_at DESC, chain_projection_gap_id DESC
             LIMIT 1",
            &[&tx_hash],
        )
        .await?;
    Ok(row.map(|row| parse_chain_projection_gap_row(&row)))
}

pub async fn load_latest_trade_lifecycle_checkpoint_by_tx_hash(
    client: &(impl GenericClient + Sync),
    tx_hash: &str,
) -> Result<Option<TradeLifecycleCheckpointRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               trade_lifecycle_checkpoint_id::text,
               monitoring_policy_profile_id::text,
               order_id::text,
               ref_domain,
               ref_type,
               ref_id::text,
               checkpoint_code,
               lifecycle_stage,
               checkpoint_status,
               to_char(expected_by AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               source_type,
               source_ref_type,
               source_ref_id::text,
               related_tx_hash,
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.trade_lifecycle_checkpoint
             WHERE related_tx_hash = $1
             ORDER BY COALESCE(occurred_at, expected_by, created_at) DESC,
                      created_at DESC,
                      trade_lifecycle_checkpoint_id DESC
             LIMIT 1",
            &[&tx_hash],
        )
        .await?;
    Ok(row.map(|row| parse_trade_lifecycle_checkpoint_row(&row)))
}

pub async fn search_audit_traces(
    client: &(impl GenericClient + Sync),
    query: &AuditTraceQuery,
    page_size: i64,
    offset: i64,
) -> Result<AuditTracePage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE ($1::text IS NULL OR (ref_type = 'order' AND ref_id = $1::text::uuid))
               AND ($2::text IS NULL OR ref_type = $2)
               AND ($3::text IS NULL OR ref_id = $3::text::uuid)
               AND ($4::text IS NULL OR request_id = $4)
               AND ($5::text IS NULL OR trace_id = $5)
               AND ($6::text IS NULL OR action_name = $6)
               AND ($7::text IS NULL OR result_code = $7)",
            &[
                &query.order_id,
                &query.ref_type,
                &query.ref_id,
                &query.request_id,
                &query.trace_id,
                &query.action_name,
                &query.result_code,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               audit_id::text,
               event_schema_version,
               event_class,
               domain_name,
               ref_type,
               ref_id::text,
               actor_id::text,
               actor_org_id::text,
               action_name,
               result_code,
               error_code,
               request_id,
               trace_id,
               tx_hash,
               evidence_manifest_id::text,
               event_hash,
               to_char(event_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM audit.audit_event
             WHERE ($1::text IS NULL OR (ref_type = 'order' AND ref_id = $1::text::uuid))
               AND ($2::text IS NULL OR ref_type = $2)
               AND ($3::text IS NULL OR ref_id = $3::text::uuid)
               AND ($4::text IS NULL OR request_id = $4)
               AND ($5::text IS NULL OR trace_id = $5)
               AND ($6::text IS NULL OR action_name = $6)
               AND ($7::text IS NULL OR result_code = $7)
             ORDER BY event_time DESC, audit_id DESC
             LIMIT $8
             OFFSET $9",
            &[
                &query.order_id,
                &query.ref_type,
                &query.ref_id,
                &query.request_id,
                &query.trace_id,
                &query.action_name,
                &query.result_code,
                &page_size,
                &offset,
            ],
        )
        .await?;

    Ok(AuditTracePage {
        total,
        items: rows.iter().map(parse_audit_trace_row).collect(),
    })
}

pub async fn search_outbox_events(
    client: &(impl GenericClient + Sync),
    query: &OpsOutboxQuery,
    limit: i64,
    offset: i64,
) -> Result<OutboxEventPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.outbox_event oe
             WHERE ($1::text IS NULL OR oe.status = $1)
               AND ($2::text IS NULL OR oe.event_type = $2)
               AND ($3::text IS NULL OR oe.target_topic = $3)
               AND ($4::text IS NULL OR oe.request_id = $4)
               AND ($5::text IS NULL OR oe.trace_id = $5)
               AND ($6::text IS NULL OR oe.aggregate_type = $6)
               AND ($7::text IS NULL OR oe.idempotency_key = $7)
               AND ($8::text IS NULL OR oe.authority_scope = $8)
               AND ($9::text IS NULL OR oe.source_of_truth = $9)
               AND ($10::text IS NULL OR oe.proof_commit_policy = $10)",
            &[
                &query.outbox_status,
                &query.event_type,
                &query.target_topic,
                &query.request_id,
                &query.trace_id,
                &query.aggregate_type,
                &query.idempotency_key,
                &query.authority_scope,
                &query.source_of_truth,
                &query.proof_commit_policy,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               oe.outbox_event_id::text,
               oe.aggregate_type,
               oe.aggregate_id::text,
               oe.event_type,
               oe.payload,
               oe.status,
               oe.retry_count,
               oe.max_retries,
               to_char(oe.available_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(oe.published_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(oe.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               oe.request_id,
               oe.trace_id,
               oe.idempotency_key,
               oe.authority_scope,
               oe.source_of_truth,
               oe.proof_commit_policy,
               oe.target_bus,
               oe.target_topic,
               oe.partition_key,
               oe.ordering_key,
               oe.payload_hash,
               oe.last_error_code,
               oe.last_error_message,
               opa.outbox_publish_attempt_id::text,
               opa.worker_id,
               opa.target_bus,
               opa.target_topic,
               opa.attempt_no,
               opa.result_code,
               opa.error_code,
               opa.error_message,
               to_char(opa.attempted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(opa.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               opa.metadata
             FROM ops.outbox_event oe
             LEFT JOIN LATERAL (
               SELECT
                 outbox_publish_attempt_id,
                 worker_id,
                 target_bus,
                 target_topic,
                 attempt_no,
                 result_code,
                 error_code,
                 error_message,
                 attempted_at,
                 completed_at,
                 metadata
               FROM ops.outbox_publish_attempt
               WHERE outbox_event_id = oe.outbox_event_id
               ORDER BY attempt_no DESC, attempted_at DESC, outbox_publish_attempt_id DESC
               LIMIT 1
             ) opa ON true
             WHERE ($1::text IS NULL OR oe.status = $1)
               AND ($2::text IS NULL OR oe.event_type = $2)
               AND ($3::text IS NULL OR oe.target_topic = $3)
               AND ($4::text IS NULL OR oe.request_id = $4)
               AND ($5::text IS NULL OR oe.trace_id = $5)
               AND ($6::text IS NULL OR oe.aggregate_type = $6)
               AND ($7::text IS NULL OR oe.idempotency_key = $7)
               AND ($8::text IS NULL OR oe.authority_scope = $8)
               AND ($9::text IS NULL OR oe.source_of_truth = $9)
               AND ($10::text IS NULL OR oe.proof_commit_policy = $10)
             ORDER BY oe.created_at DESC, oe.outbox_event_id DESC
             LIMIT $11
             OFFSET $12",
            &[
                &query.outbox_status,
                &query.event_type,
                &query.target_topic,
                &query.request_id,
                &query.trace_id,
                &query.aggregate_type,
                &query.idempotency_key,
                &query.authority_scope,
                &query.source_of_truth,
                &query.proof_commit_policy,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(OutboxEventPage {
        total,
        items: rows.iter().map(parse_outbox_event_row).collect(),
    })
}

pub async fn search_dead_letters(
    client: &(impl GenericClient + Sync),
    query: &OpsDeadLetterQuery,
    limit: i64,
    offset: i64,
) -> Result<DeadLetterEventPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.dead_letter_event dl
             WHERE ($1::text IS NULL OR dl.reprocess_status = $1)
               AND ($2::text IS NULL OR dl.failure_stage = $2)
               AND ($3::text IS NULL OR dl.request_id = $3)
               AND ($4::text IS NULL OR dl.trace_id = $4)",
            &[
                &query.reprocess_status,
                &query.failure_stage,
                &query.request_id,
                &query.trace_id,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               dead_letter_event_id::text,
               outbox_event_id::text,
               aggregate_type,
               aggregate_id::text,
               event_type,
               payload,
               failed_reason,
               request_id,
               trace_id,
               authority_scope,
               source_of_truth,
               target_bus,
               target_topic,
               failure_stage,
               to_char(first_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               reprocess_status,
               to_char(reprocessed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.dead_letter_event dl
             WHERE ($1::text IS NULL OR dl.reprocess_status = $1)
               AND ($2::text IS NULL OR dl.failure_stage = $2)
               AND ($3::text IS NULL OR dl.request_id = $3)
               AND ($4::text IS NULL OR dl.trace_id = $4)
             ORDER BY dl.created_at DESC, dl.dead_letter_event_id DESC
             LIMIT $5
             OFFSET $6",
            &[
                &query.reprocess_status,
                &query.failure_stage,
                &query.request_id,
                &query.trace_id,
                &limit,
                &offset,
            ],
        )
        .await?;

    let event_ids: Vec<String> = rows
        .iter()
        .filter_map(|row| row.get::<_, Option<String>>(1))
        .collect();
    let idempotency_map = load_consumer_idempotency_for_event_ids(client, &event_ids).await?;

    Ok(DeadLetterEventPage {
        total,
        items: rows
            .iter()
            .map(|row| {
                let outbox_event_id = row.get::<_, Option<String>>(1);
                let records = outbox_event_id
                    .as_ref()
                    .and_then(|event_id| idempotency_map.get(event_id))
                    .cloned()
                    .unwrap_or_default();
                parse_dead_letter_row(row, records)
            })
            .collect(),
    })
}

pub async fn load_dead_letter_event(
    client: &(impl GenericClient + Sync),
    dead_letter_event_id: &str,
) -> Result<Option<DeadLetterEventRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               dead_letter_event_id::text,
               outbox_event_id::text,
               aggregate_type,
               aggregate_id::text,
               event_type,
               payload,
               failed_reason,
               request_id,
               trace_id,
               authority_scope,
               source_of_truth,
               target_bus,
               target_topic,
               failure_stage,
               to_char(first_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               reprocess_status,
               to_char(reprocessed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.dead_letter_event
             WHERE dead_letter_event_id = $1::text::uuid",
            &[&dead_letter_event_id],
        )
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };

    let event_ids = row
        .get::<_, Option<String>>(1)
        .into_iter()
        .collect::<Vec<String>>();
    let idempotency_map = load_consumer_idempotency_for_event_ids(client, &event_ids).await?;
    let consumer_records = row
        .get::<_, Option<String>>(1)
        .as_ref()
        .and_then(|event_id| idempotency_map.get(event_id))
        .cloned()
        .unwrap_or_default();

    Ok(Some(parse_dead_letter_row(&row, consumer_records)))
}

pub async fn search_consumer_idempotency_records(
    client: &(impl GenericClient + Sync),
    query: &ConsumerIdempotencyQuery,
    limit: i64,
    offset: i64,
) -> Result<ConsumerIdempotencyPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.consumer_idempotency_record cir
             WHERE ($1::text IS NULL OR cir.consumer_name = $1)
               AND ($2::text IS NULL OR cir.event_id = $2::text::uuid)
               AND ($3::text IS NULL OR cir.aggregate_type = $3)
               AND ($4::text IS NULL OR cir.aggregate_id = $4::text::uuid)
               AND ($5::text IS NULL OR cir.trace_id = $5)",
            &[
                &query.consumer_name,
                &query.event_id,
                &query.aggregate_type,
                &query.aggregate_id,
                &query.trace_id,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               consumer_idempotency_record_id::text,
               consumer_name,
               event_id::text,
               aggregate_type,
               aggregate_id::text,
               trace_id,
               result_code,
               to_char(processed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata
             FROM ops.consumer_idempotency_record cir
             WHERE ($1::text IS NULL OR cir.consumer_name = $1)
               AND ($2::text IS NULL OR cir.event_id = $2::text::uuid)
               AND ($3::text IS NULL OR cir.aggregate_type = $3)
               AND ($4::text IS NULL OR cir.aggregate_id = $4::text::uuid)
               AND ($5::text IS NULL OR cir.trace_id = $5)
             ORDER BY cir.processed_at DESC, cir.consumer_idempotency_record_id DESC
             LIMIT $6
             OFFSET $7",
            &[
                &query.consumer_name,
                &query.event_id,
                &query.aggregate_type,
                &query.aggregate_id,
                &query.trace_id,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(ConsumerIdempotencyPage {
        total,
        items: rows.iter().map(parse_consumer_idempotency_row).collect(),
    })
}

pub async fn search_external_fact_receipts(
    client: &(impl GenericClient + Sync),
    query: &ExternalFactReceiptQuery,
    limit: i64,
    offset: i64,
) -> Result<ExternalFactReceiptPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.external_fact_receipt efr
             WHERE ($1::text IS NULL OR efr.order_id = $1::text::uuid)
               AND ($2::text IS NULL OR efr.ref_type = $2)
               AND ($3::text IS NULL OR efr.ref_id = $3::text::uuid)
               AND ($4::text IS NULL OR efr.fact_type = $4)
               AND ($5::text IS NULL OR efr.provider_type = $5)
               AND ($6::text IS NULL OR efr.receipt_status = $6)
               AND ($7::text IS NULL OR efr.request_id = $7)
               AND ($8::text IS NULL OR efr.trace_id = $8)
               AND ($9::text IS NULL OR COALESCE(efr.received_at, efr.occurred_at, efr.created_at) >= $9::timestamptz)
               AND ($10::text IS NULL OR COALESCE(efr.received_at, efr.occurred_at, efr.created_at) <= $10::timestamptz)",
            &[
                &query.order_id,
                &query.ref_type,
                &query.ref_id,
                &query.fact_type,
                &query.provider_type,
                &query.receipt_status,
                &query.request_id,
                &query.trace_id,
                &query.from,
                &query.to,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               external_fact_receipt_id::text,
               order_id::text,
               ref_domain,
               ref_type,
               ref_id::text,
               fact_type,
               provider_type,
               provider_key,
               provider_reference,
               receipt_status,
               receipt_payload,
               receipt_hash,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(received_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(confirmed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.external_fact_receipt efr
             WHERE ($1::text IS NULL OR efr.order_id = $1::text::uuid)
               AND ($2::text IS NULL OR efr.ref_type = $2)
               AND ($3::text IS NULL OR efr.ref_id = $3::text::uuid)
               AND ($4::text IS NULL OR efr.fact_type = $4)
               AND ($5::text IS NULL OR efr.provider_type = $5)
               AND ($6::text IS NULL OR efr.receipt_status = $6)
               AND ($7::text IS NULL OR efr.request_id = $7)
               AND ($8::text IS NULL OR efr.trace_id = $8)
               AND ($9::text IS NULL OR COALESCE(efr.received_at, efr.occurred_at, efr.created_at) >= $9::timestamptz)
               AND ($10::text IS NULL OR COALESCE(efr.received_at, efr.occurred_at, efr.created_at) <= $10::timestamptz)
             ORDER BY efr.received_at DESC, efr.external_fact_receipt_id DESC
             LIMIT $11
             OFFSET $12",
            &[
                &query.order_id,
                &query.ref_type,
                &query.ref_id,
                &query.fact_type,
                &query.provider_type,
                &query.receipt_status,
                &query.request_id,
                &query.trace_id,
                &query.from,
                &query.to,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(ExternalFactReceiptPage {
        total,
        items: rows.iter().map(parse_external_fact_receipt_row).collect(),
    })
}

pub async fn load_external_fact_receipt(
    client: &(impl GenericClient + Sync),
    external_fact_receipt_id: &str,
) -> Result<Option<ExternalFactReceiptRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               external_fact_receipt_id::text,
               order_id::text,
               ref_domain,
               ref_type,
               ref_id::text,
               fact_type,
               provider_type,
               provider_key,
               provider_reference,
               receipt_status,
               receipt_payload,
               receipt_hash,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(received_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(confirmed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.external_fact_receipt
             WHERE external_fact_receipt_id = $1::text::uuid",
            &[&external_fact_receipt_id],
        )
        .await?;
    Ok(row.map(|row| parse_external_fact_receipt_row(&row)))
}

pub async fn confirm_external_fact_receipt(
    client: &(impl GenericClient + Sync),
    external_fact_receipt_id: &str,
    confirm_result: &str,
    confirmed_at: &str,
    metadata_patch: &Value,
) -> Result<Option<ExternalFactReceiptRecord>, Error> {
    let row = client
        .query_opt(
            "UPDATE ops.external_fact_receipt
             SET receipt_status = $2,
                 confirmed_at = $3::timestamptz,
                 metadata = COALESCE(metadata, '{}'::jsonb) || $4::jsonb,
                 updated_at = now()
             WHERE external_fact_receipt_id = $1::text::uuid
               AND receipt_status = 'pending'
             RETURNING
               external_fact_receipt_id::text,
               order_id::text,
               ref_domain,
               ref_type,
               ref_id::text,
               fact_type,
               provider_type,
               provider_key,
               provider_reference,
               receipt_status,
               receipt_payload,
               receipt_hash,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(received_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(confirmed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &external_fact_receipt_id,
                &confirm_result,
                &confirmed_at,
                metadata_patch,
            ],
        )
        .await?;
    Ok(row.map(|row| parse_external_fact_receipt_row(&row)))
}

pub async fn search_chain_projection_gaps(
    client: &(impl GenericClient + Sync),
    query: &ChainProjectionGapQuery,
    limit: i64,
    offset: i64,
) -> Result<ChainProjectionGapPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.chain_projection_gap cpg
             WHERE ($1::text IS NULL OR cpg.aggregate_type = $1)
               AND ($2::text IS NULL OR cpg.aggregate_id = $2::text::uuid)
               AND ($3::text IS NULL OR cpg.order_id = $3::text::uuid)
               AND ($4::text IS NULL OR cpg.chain_id = $4)
               AND ($5::text IS NULL OR cpg.gap_type = $5)
               AND ($6::text IS NULL OR cpg.gap_status = $6)
               AND ($7::text IS NULL OR cpg.request_id = $7)
               AND ($8::text IS NULL OR cpg.trace_id = $8)",
            &[
                &query.aggregate_type,
                &query.aggregate_id,
                &query.order_id,
                &query.chain_id,
                &query.gap_type,
                &query.gap_status,
                &query.request_id,
                &query.trace_id,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               chain_projection_gap_id::text,
               aggregate_type,
               aggregate_id::text,
               order_id::text,
               chain_id,
               source_event_type,
               expected_tx_id,
               projected_tx_hash,
               gap_type,
               gap_status,
               to_char(first_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               outbox_event_id::text,
               anchor_id::text,
               resolution_summary,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.chain_projection_gap cpg
             WHERE ($1::text IS NULL OR cpg.aggregate_type = $1)
               AND ($2::text IS NULL OR cpg.aggregate_id = $2::text::uuid)
               AND ($3::text IS NULL OR cpg.order_id = $3::text::uuid)
               AND ($4::text IS NULL OR cpg.chain_id = $4)
               AND ($5::text IS NULL OR cpg.gap_type = $5)
               AND ($6::text IS NULL OR cpg.gap_status = $6)
               AND ($7::text IS NULL OR cpg.request_id = $7)
               AND ($8::text IS NULL OR cpg.trace_id = $8)
             ORDER BY cpg.created_at DESC, cpg.chain_projection_gap_id DESC
             LIMIT $9
             OFFSET $10",
            &[
                &query.aggregate_type,
                &query.aggregate_id,
                &query.order_id,
                &query.chain_id,
                &query.gap_type,
                &query.gap_status,
                &query.request_id,
                &query.trace_id,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(ChainProjectionGapPage {
        total,
        items: rows.iter().map(parse_chain_projection_gap_row).collect(),
    })
}

pub async fn load_chain_projection_gap(
    client: &(impl GenericClient + Sync),
    chain_projection_gap_id: &str,
) -> Result<Option<ChainProjectionGapRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               chain_projection_gap_id::text,
               aggregate_type,
               aggregate_id::text,
               order_id::text,
               chain_id,
               source_event_type,
               expected_tx_id,
               projected_tx_hash,
               gap_type,
               gap_status,
               to_char(first_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               outbox_event_id::text,
               anchor_id::text,
               resolution_summary,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.chain_projection_gap
             WHERE chain_projection_gap_id = $1::text::uuid",
            &[&chain_projection_gap_id],
        )
        .await?;
    Ok(row.map(|row| parse_chain_projection_gap_row(&row)))
}

pub async fn resolve_chain_projection_gap(
    client: &(impl GenericClient + Sync),
    chain_projection_gap_id: &str,
    resolved_at: &str,
    request_id: &str,
    trace_id: &str,
    resolution_summary_patch: &Value,
    metadata_patch: &Value,
) -> Result<Option<ChainProjectionGapRecord>, Error> {
    let row = client
        .query_opt(
            "UPDATE ops.chain_projection_gap
             SET gap_status = 'resolved',
                 resolved_at = $2::timestamptz,
                 request_id = $3,
                 trace_id = $4,
                 resolution_summary = COALESCE(resolution_summary, '{}'::jsonb) || $5::jsonb,
                 metadata = COALESCE(metadata, '{}'::jsonb) || $6::jsonb,
                 updated_at = now()
             WHERE chain_projection_gap_id = $1::text::uuid
               AND gap_status <> 'resolved'
             RETURNING
               chain_projection_gap_id::text,
               aggregate_type,
               aggregate_id::text,
               order_id::text,
               chain_id,
               source_event_type,
               expected_tx_id,
               projected_tx_hash,
               gap_type,
               gap_status,
               to_char(first_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               outbox_event_id::text,
               anchor_id::text,
               resolution_summary,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &chain_projection_gap_id,
                &resolved_at,
                &request_id,
                &trace_id,
                resolution_summary_patch,
                metadata_patch,
            ],
        )
        .await?;
    Ok(row.map(|row| parse_chain_projection_gap_row(&row)))
}

pub async fn search_trade_lifecycle_checkpoints_by_order(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    query: &TradeMonitorCheckpointQuery,
    limit: i64,
    offset: i64,
) -> Result<TradeLifecycleCheckpointPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.trade_lifecycle_checkpoint tlc
             WHERE tlc.order_id = $1::text::uuid
               AND ($2::text IS NULL OR tlc.checkpoint_code = $2)
               AND ($3::text IS NULL OR tlc.checkpoint_status = $3)
               AND ($4::text IS NULL OR tlc.lifecycle_stage = $4)
               AND ($5::text IS NULL OR COALESCE(tlc.occurred_at, tlc.expected_by, tlc.created_at) >= $5::timestamptz)
               AND ($6::text IS NULL OR COALESCE(tlc.occurred_at, tlc.expected_by, tlc.created_at) <= $6::timestamptz)",
            &[
                &order_id,
                &query.checkpoint_code,
                &query.checkpoint_status,
                &query.lifecycle_stage,
                &query.from,
                &query.to,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               trade_lifecycle_checkpoint_id::text,
               monitoring_policy_profile_id::text,
               order_id::text,
               ref_domain,
               ref_type,
               ref_id::text,
               checkpoint_code,
               lifecycle_stage,
               checkpoint_status,
               to_char(expected_by AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               source_type,
               source_ref_type,
               source_ref_id::text,
               related_tx_hash,
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.trade_lifecycle_checkpoint tlc
             WHERE tlc.order_id = $1::text::uuid
               AND ($2::text IS NULL OR tlc.checkpoint_code = $2)
               AND ($3::text IS NULL OR tlc.checkpoint_status = $3)
               AND ($4::text IS NULL OR tlc.lifecycle_stage = $4)
               AND ($5::text IS NULL OR COALESCE(tlc.occurred_at, tlc.expected_by, tlc.created_at) >= $5::timestamptz)
               AND ($6::text IS NULL OR COALESCE(tlc.occurred_at, tlc.expected_by, tlc.created_at) <= $6::timestamptz)
             ORDER BY COALESCE(tlc.occurred_at, tlc.expected_by, tlc.created_at) DESC,
                      tlc.created_at DESC,
                      tlc.trade_lifecycle_checkpoint_id DESC
             LIMIT $7
             OFFSET $8",
            &[
                &order_id,
                &query.checkpoint_code,
                &query.checkpoint_status,
                &query.lifecycle_stage,
                &query.from,
                &query.to,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(TradeLifecycleCheckpointPage {
        total,
        items: rows
            .iter()
            .map(parse_trade_lifecycle_checkpoint_row)
            .collect(),
    })
}

pub async fn search_recent_fairness_incidents_for_order(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    limit: i64,
) -> Result<FairnessIncidentPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM risk.fairness_incident fi
             WHERE fi.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               fairness_incident_id::text,
               order_id::text,
               ref_type,
               ref_id::text,
               incident_type,
               severity,
               lifecycle_stage,
               detected_by_type,
               source_checkpoint_id::text,
               source_receipt_id::text,
               status,
               auto_action_code,
               assigned_role_key,
               assigned_user_id::text,
               resolution_summary,
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(closed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM risk.fairness_incident fi
             WHERE fi.order_id = $1::text::uuid
             ORDER BY fi.created_at DESC, fi.fairness_incident_id DESC
             LIMIT $2",
            &[&order_id, &limit],
        )
        .await?;

    Ok(FairnessIncidentPage {
        total,
        items: rows.iter().map(parse_fairness_incident_row).collect(),
    })
}

pub async fn search_fairness_incidents(
    client: &(impl GenericClient + Sync),
    query: &FairnessIncidentQuery,
    limit: i64,
    offset: i64,
) -> Result<FairnessIncidentPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM risk.fairness_incident fi
             WHERE ($1::text IS NULL OR fi.order_id = $1::text::uuid)
               AND ($2::text IS NULL OR fi.incident_type = $2)
               AND ($3::text IS NULL OR fi.severity = $3)
               AND ($4::text IS NULL OR fi.status = $4)
               AND ($5::text IS NULL OR fi.assigned_role_key = $5)
               AND ($6::text IS NULL OR fi.assigned_user_id = $6::text::uuid)
               AND ($7::text IS NULL OR fi.request_id = $7)
               AND ($8::text IS NULL OR fi.trace_id = $8)",
            &[
                &query.order_id,
                &query.incident_type,
                &query.severity,
                &query.fairness_incident_status,
                &query.assigned_role_key,
                &query.assigned_user_id,
                &query.request_id,
                &query.trace_id,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               fairness_incident_id::text,
               order_id::text,
               ref_type,
               ref_id::text,
               incident_type,
               severity,
               lifecycle_stage,
               detected_by_type,
               source_checkpoint_id::text,
               source_receipt_id::text,
               status,
               auto_action_code,
               assigned_role_key,
               assigned_user_id::text,
               resolution_summary,
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(closed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM risk.fairness_incident fi
             WHERE ($1::text IS NULL OR fi.order_id = $1::text::uuid)
               AND ($2::text IS NULL OR fi.incident_type = $2)
               AND ($3::text IS NULL OR fi.severity = $3)
               AND ($4::text IS NULL OR fi.status = $4)
               AND ($5::text IS NULL OR fi.assigned_role_key = $5)
               AND ($6::text IS NULL OR fi.assigned_user_id = $6::text::uuid)
               AND ($7::text IS NULL OR fi.request_id = $7)
               AND ($8::text IS NULL OR fi.trace_id = $8)
             ORDER BY fi.created_at DESC, fi.fairness_incident_id DESC
             LIMIT $9 OFFSET $10",
            &[
                &query.order_id,
                &query.incident_type,
                &query.severity,
                &query.fairness_incident_status,
                &query.assigned_role_key,
                &query.assigned_user_id,
                &query.request_id,
                &query.trace_id,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(FairnessIncidentPage {
        total,
        items: rows.iter().map(parse_fairness_incident_row).collect(),
    })
}

pub async fn load_fairness_incident(
    client: &(impl GenericClient + Sync),
    fairness_incident_id: &str,
) -> Result<Option<FairnessIncidentRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               fairness_incident_id::text,
               order_id::text,
               ref_type,
               ref_id::text,
               incident_type,
               severity,
               lifecycle_stage,
               detected_by_type,
               source_checkpoint_id::text,
               source_receipt_id::text,
               status,
               auto_action_code,
               assigned_role_key,
               assigned_user_id::text,
               resolution_summary,
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(closed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM risk.fairness_incident
             WHERE fairness_incident_id = $1::text::uuid",
            &[&fairness_incident_id],
        )
        .await?;

    Ok(row.map(|row| parse_fairness_incident_row(&row)))
}

pub async fn handle_fairness_incident(
    client: &(impl GenericClient + Sync),
    fairness_incident_id: &str,
    next_status: &str,
    resolution_summary: &str,
    auto_action_code: Option<&str>,
    closed_at: Option<&str>,
    request_id: &str,
    trace_id: &str,
    metadata_patch: &Value,
) -> Result<Option<FairnessIncidentRecord>, Error> {
    let row = client
        .query_opt(
            "UPDATE risk.fairness_incident
             SET status = $2,
                 resolution_summary = $3,
                 auto_action_code = COALESCE($4, auto_action_code),
                 closed_at = CASE
                   WHEN $5::text IS NULL THEN closed_at
                   ELSE $5::text::timestamptz
                 END,
                 request_id = $6,
                 trace_id = $7,
                 metadata = metadata || $8::jsonb,
                 updated_at = now()
             WHERE fairness_incident_id = $1::text::uuid
               AND status = 'open'
             RETURNING
               fairness_incident_id::text,
               order_id::text,
               ref_type,
               ref_id::text,
               incident_type,
               severity,
               lifecycle_stage,
               detected_by_type,
               source_checkpoint_id::text,
               source_receipt_id::text,
               status,
               auto_action_code,
               assigned_role_key,
               assigned_user_id::text,
               resolution_summary,
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(closed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &fairness_incident_id,
                &next_status,
                &resolution_summary,
                &auto_action_code,
                &closed_at,
                &request_id,
                &trace_id,
                metadata_patch,
            ],
        )
        .await?;

    Ok(row.map(|row| parse_fairness_incident_row(&row)))
}

pub async fn search_observability_backends(
    client: &(impl GenericClient + Sync),
) -> Result<Vec<ObservabilityBackendRecord>, Error> {
    let rows = client
        .query(
            "SELECT
               observability_backend_id::text,
               backend_key,
               backend_type,
               endpoint_uri,
               auth_mode,
               enabled,
               stage_from,
               capability_json,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.observability_backend
             ORDER BY backend_type, backend_key",
            &[],
        )
        .await?;
    Ok(rows.iter().map(parse_observability_backend_row).collect())
}

pub async fn search_system_log_mirrors(
    client: &(impl GenericClient + Sync),
    query: &OpsLogMirrorQuery,
    limit: i64,
    offset: i64,
) -> Result<SystemLogMirrorPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log sl
             WHERE ($1::text IS NULL OR sl.service_name = $1)
               AND ($2::text IS NULL OR sl.log_level = $2)
               AND ($3::text IS NULL OR sl.request_id = $3)
               AND ($4::text IS NULL OR sl.trace_id = $4)
               AND ($5::text IS NULL OR sl.object_type = $5)
               AND ($6::text IS NULL OR sl.object_id = $6::text::uuid)
               AND ($7::text IS NULL OR sl.created_at >= $7::timestamptz)
               AND ($8::text IS NULL OR sl.created_at <= $8::timestamptz)
               AND (
                 $9::text IS NULL
                 OR sl.message_text ILIKE '%' || $9 || '%'
                 OR COALESCE(sl.logger_name, '') ILIKE '%' || $9 || '%'
                 OR sl.structured_payload::text ILIKE '%' || $9 || '%'
               )",
            &[
                &query.service_name,
                &query.log_level,
                &query.request_id,
                &query.trace_id,
                &query.object_type,
                &query.object_id,
                &query.from,
                &query.to,
                &query.query,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               sl.system_log_id::text,
               sl.service_name,
               sl.logger_name,
               sl.log_level,
               sl.severity_number,
               sl.environment_code,
               sl.host_name,
               sl.node_name,
               sl.pod_name,
               sl.backend_type,
               sl.request_id,
               sl.trace_id,
               sl.message_text,
               sl.structured_payload,
               sl.object_type,
               sl.object_id::text,
               sl.masked_status,
               sl.retention_class,
               sl.legal_hold_status,
               sl.resource_attrs,
               to_char(sl.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.system_log sl
             WHERE ($1::text IS NULL OR sl.service_name = $1)
               AND ($2::text IS NULL OR sl.log_level = $2)
               AND ($3::text IS NULL OR sl.request_id = $3)
               AND ($4::text IS NULL OR sl.trace_id = $4)
               AND ($5::text IS NULL OR sl.object_type = $5)
               AND ($6::text IS NULL OR sl.object_id = $6::text::uuid)
               AND ($7::text IS NULL OR sl.created_at >= $7::timestamptz)
               AND ($8::text IS NULL OR sl.created_at <= $8::timestamptz)
               AND (
                 $9::text IS NULL
                 OR sl.message_text ILIKE '%' || $9 || '%'
                 OR COALESCE(sl.logger_name, '') ILIKE '%' || $9 || '%'
                 OR sl.structured_payload::text ILIKE '%' || $9 || '%'
               )
             ORDER BY sl.created_at DESC, sl.system_log_id DESC
             LIMIT $10 OFFSET $11",
            &[
                &query.service_name,
                &query.log_level,
                &query.request_id,
                &query.trace_id,
                &query.object_type,
                &query.object_id,
                &query.from,
                &query.to,
                &query.query,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(SystemLogMirrorPage {
        total,
        items: rows.iter().map(parse_system_log_mirror_row).collect(),
    })
}

pub async fn load_trace_index_by_trace_id(
    client: &(impl GenericClient + Sync),
    trace_id: &str,
) -> Result<Option<TraceIndexRecord>, Error> {
    let row = client
        .query_opt(
            "SELECT
               trace_index_id::text,
               trace_id,
               traceparent,
               backend_key,
               root_service_name,
               root_span_name,
               request_id,
               ref_type,
               ref_id::text,
               object_type,
               object_id::text,
               status,
               span_count,
               to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.trace_index
             WHERE trace_id = $1
             ORDER BY COALESCE(ended_at, started_at, created_at) DESC, trace_index_id DESC
             LIMIT 1",
            &[&trace_id],
        )
        .await?;
    Ok(row.map(|row| parse_trace_index_row(&row)))
}

pub async fn count_system_logs_by_trace_id(
    client: &(impl GenericClient + Sync),
    trace_id: &str,
) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE trace_id = $1",
            &[&trace_id],
        )
        .await
        .map(|row| row.get(0))
}

pub async fn count_alert_events_by_trace_id(
    client: &(impl GenericClient + Sync),
    trace_id: &str,
) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.alert_event
             WHERE trace_id = $1",
            &[&trace_id],
        )
        .await
        .map(|row| row.get(0))
}

pub async fn search_alert_events(
    client: &(impl GenericClient + Sync),
    query: &OpsAlertQuery,
    limit: i64,
    offset: i64,
) -> Result<AlertEventPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.alert_event ae
             WHERE ($1::text IS NULL OR ae.status = $1)
               AND ($2::text IS NULL OR ae.severity = $2)
               AND ($3::text IS NULL OR ae.source_backend_key = $3)
               AND ($4::text IS NULL OR ae.alert_type = $4)",
            &[
                &query.alert_status,
                &query.severity,
                &query.source_backend_key,
                &query.alert_type,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               ae.alert_event_id::text,
               ae.alert_rule_id::text,
               ae.source_backend_key,
               ae.fingerprint,
               ae.alert_type,
               ae.severity,
               ae.title_text,
               ae.summary_text,
               ae.ref_type,
               ae.ref_id::text,
               ae.request_id,
               ae.trace_id,
               ae.labels_json,
               ae.annotations_json,
               ae.status,
               ae.acknowledged_by::text,
               to_char(ae.acknowledged_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ae.fired_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ae.resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ae.metadata,
               it.incident_ticket_id::text,
               to_char(ae.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ae.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.alert_event ae
             LEFT JOIN ops.incident_ticket it
               ON it.source_alert_event_id = ae.alert_event_id
             WHERE ($1::text IS NULL OR ae.status = $1)
               AND ($2::text IS NULL OR ae.severity = $2)
               AND ($3::text IS NULL OR ae.source_backend_key = $3)
               AND ($4::text IS NULL OR ae.alert_type = $4)
             ORDER BY ae.fired_at DESC, ae.alert_event_id DESC
             LIMIT $5 OFFSET $6",
            &[
                &query.alert_status,
                &query.severity,
                &query.source_backend_key,
                &query.alert_type,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(AlertEventPage {
        total,
        items: rows.iter().map(parse_alert_event_row).collect(),
    })
}

pub async fn search_incident_tickets(
    client: &(impl GenericClient + Sync),
    query: &OpsIncidentQuery,
    limit: i64,
    offset: i64,
) -> Result<IncidentTicketPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.incident_ticket it
             WHERE ($1::text IS NULL OR it.status = $1)
               AND ($2::text IS NULL OR it.severity = $2)
               AND ($3::text IS NULL OR it.owner_role_key = $3)",
            &[
                &query.incident_status,
                &query.severity,
                &query.owner_role_key,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               it.incident_ticket_id::text,
               it.incident_key,
               it.source_alert_event_id::text,
               it.severity,
               it.title_text,
               it.summary_text,
               it.status,
               it.owner_role_key,
               it.owner_user_id::text,
               it.runbook_uri,
               it.metadata ->> 'impact_summary',
               it.metadata ->> 'root_cause_summary',
               ie.event_type,
               ie.note_text,
               it.metadata,
               to_char(it.started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(it.resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(it.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(it.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.incident_ticket it
             LEFT JOIN LATERAL (
               SELECT event_type, note_text
               FROM ops.incident_event
               WHERE incident_ticket_id = it.incident_ticket_id
               ORDER BY created_at DESC, incident_event_id DESC
               LIMIT 1
             ) ie ON true
             WHERE ($1::text IS NULL OR it.status = $1)
               AND ($2::text IS NULL OR it.severity = $2)
               AND ($3::text IS NULL OR it.owner_role_key = $3)
             ORDER BY it.started_at DESC, it.incident_ticket_id DESC
             LIMIT $4 OFFSET $5",
            &[
                &query.incident_status,
                &query.severity,
                &query.owner_role_key,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(IncidentTicketPage {
        total,
        items: rows.iter().map(parse_incident_ticket_row).collect(),
    })
}

pub async fn search_slos(
    client: &(impl GenericClient + Sync),
    query: &OpsSloQuery,
    limit: i64,
    offset: i64,
) -> Result<SloPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.slo_definition sd
             WHERE ($1::text IS NULL OR sd.service_name = $1)
               AND ($2::text IS NULL OR sd.source_backend_key = $2)
               AND ($3::text IS NULL OR sd.status = $3)",
            &[
                &query.service_name,
                &query.source_backend_key,
                &query.status,
            ],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               sd.slo_definition_id::text,
               sd.slo_key,
               sd.service_name,
               sd.indicator_type,
               sd.objective_value::text,
               sd.window_code,
               sd.source_backend_key,
               sd.alert_rule_id::text,
               sd.status,
               ss.measured_value::text,
               ss.error_budget_remaining::text,
               ss.status,
               to_char(ss.window_started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ss.window_ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               sd.metadata,
               to_char(sd.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(sd.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.slo_definition sd
             LEFT JOIN LATERAL (
               SELECT
                 measured_value,
                 error_budget_remaining,
                 status,
                 window_started_at,
                 window_ended_at
               FROM ops.slo_snapshot
               WHERE slo_definition_id = sd.slo_definition_id
               ORDER BY window_ended_at DESC, slo_snapshot_id DESC
               LIMIT 1
             ) ss ON true
             WHERE ($1::text IS NULL OR sd.service_name = $1)
               AND ($2::text IS NULL OR sd.source_backend_key = $2)
               AND ($3::text IS NULL OR sd.status = $3)
             ORDER BY sd.service_name, sd.slo_key
             LIMIT $4 OFFSET $5",
            &[
                &query.service_name,
                &query.source_backend_key,
                &query.status,
                &limit,
                &offset,
            ],
        )
        .await?;

    Ok(SloPage {
        total,
        items: rows.iter().map(parse_slo_row).collect(),
    })
}

pub async fn count_open_fairness_incidents_for_order(
    client: &(impl GenericClient + Sync),
    order_id: &str,
) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM risk.fairness_incident fi
             WHERE fi.order_id = $1::text::uuid
               AND fi.status <> 'closed'",
            &[&order_id],
        )
        .await
        .map(|row| row.get(0))
}

pub async fn load_consistency_subject(
    client: &(impl GenericClient + Sync),
    ref_type: &str,
    ref_id: &str,
) -> Result<Option<ConsistencySubjectRecord>, Error> {
    let row = match ref_type {
        "order" => {
            client
                .query_opt(
                    "SELECT
                       order_id::text,
                       order_id::text,
                       status,
                       authority_model,
                       business_state_version,
                       proof_commit_state,
                       proof_commit_policy,
                       external_fact_status,
                       reconcile_status,
                       to_char(last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       jsonb_build_object(
                         'buyer_org_id', buyer_org_id::text,
                         'seller_org_id', seller_org_id::text,
                         'payment_status', payment_status,
                         'delivery_status', delivery_status,
                         'acceptance_status', acceptance_status,
                         'settlement_status', settlement_status,
                         'dispute_status', dispute_status,
                         'amount', amount,
                         'currency_code', currency_code,
                         'chain_tx_create', chain_tx_create,
                         'chain_tx_settle', chain_tx_settle
                       )
                     FROM trade.order_main
                     WHERE order_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await?
        }
        "digital_contract" => {
            client
                .query_opt(
                    "SELECT
                       contract_id::text,
                       order_id::text,
                       status,
                       authority_model,
                       business_state_version,
                       proof_commit_state,
                       proof_commit_policy,
                       external_fact_status,
                       reconcile_status,
                       to_char(last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       jsonb_build_object(
                         'order_id', order_id::text,
                         'data_contract_id', data_contract_id::text,
                         'data_contract_digest', data_contract_digest,
                         'contract_digest', contract_digest,
                         'signed_at', to_char(signed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                       )
                     FROM contract.digital_contract
                     WHERE contract_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await?
        }
        "delivery_record" => {
            client
                .query_opt(
                    "SELECT
                       delivery_id::text,
                       order_id::text,
                       status,
                       authority_model,
                       business_state_version,
                       proof_commit_state,
                       proof_commit_policy,
                       external_fact_status,
                       reconcile_status,
                       to_char(last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       jsonb_build_object(
                         'order_id', order_id::text,
                         'delivery_type', delivery_type,
                         'delivery_route', delivery_route,
                         'executor_type', executor_type,
                         'delivery_commit_hash', delivery_commit_hash,
                         'receipt_hash', receipt_hash,
                         'disclosure_review_status', disclosure_review_status
                       )
                     FROM delivery.delivery_record
                     WHERE delivery_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await?
        }
        "settlement_record" => {
            client
                .query_opt(
                    "SELECT
                       settlement_id::text,
                       order_id::text,
                       settlement_status,
                       authority_model,
                       business_state_version,
                       proof_commit_state,
                       proof_commit_policy,
                       external_fact_status,
                       reconcile_status,
                       to_char(last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       jsonb_build_object(
                         'order_id', order_id::text,
                         'settlement_type', settlement_type,
                         'settlement_mode', settlement_mode,
                         'payable_amount', payable_amount,
                         'net_receivable_amount', net_receivable_amount,
                         'refund_amount', refund_amount,
                         'compensation_amount', compensation_amount,
                         'settled_at', to_char(settled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                       )
                     FROM billing.settlement_record
                     WHERE settlement_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await?
        }
        "payment_intent" => {
            client
                .query_opt(
                    "SELECT
                       payment_intent_id::text,
                       order_id::text,
                       status,
                       authority_model,
                       business_state_version,
                       proof_commit_state,
                       proof_commit_policy,
                       external_fact_status,
                       reconcile_status,
                       to_char(last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       jsonb_build_object(
                         'order_id', order_id::text,
                         'provider_key', provider_key,
                         'provider_intent_no', provider_intent_no,
                         'channel_reference_no', channel_reference_no,
                         'amount', amount,
                         'currency_code', currency_code
                       )
                     FROM payment.payment_intent
                     WHERE payment_intent_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await?
        }
        "refund_intent" => {
            client
                .query_opt(
                    "SELECT
                       refund_intent_id::text,
                       pi.order_id::text,
                       ri.status,
                       ri.authority_model,
                       ri.business_state_version,
                       ri.proof_commit_state,
                       ri.proof_commit_policy,
                       ri.external_fact_status,
                       ri.reconcile_status,
                       to_char(ri.last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       jsonb_build_object(
                         'payment_intent_id', ri.payment_intent_id::text,
                         'provider_key', ri.provider_key,
                         'provider_refund_no', ri.provider_refund_no,
                         'reason_code', ri.reason_code,
                         'amount', ri.amount,
                         'currency_code', ri.currency_code
                       )
                     FROM payment.refund_intent ri
                     LEFT JOIN payment.payment_intent pi
                       ON pi.payment_intent_id = ri.payment_intent_id
                     WHERE ri.refund_intent_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await?
        }
        "payout_instruction" => {
            client
                .query_opt(
                    "SELECT
                       payout_instruction_id::text,
                       sr.order_id::text,
                       pi.status,
                       pi.authority_model,
                       pi.business_state_version,
                       pi.proof_commit_state,
                       pi.proof_commit_policy,
                       pi.external_fact_status,
                       pi.reconcile_status,
                       to_char(pi.last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       jsonb_build_object(
                         'settlement_id', pi.settlement_id::text,
                         'provider_key', pi.provider_key,
                         'provider_payout_no', pi.provider_payout_no,
                         'payout_mode', pi.payout_mode,
                         'amount', pi.amount,
                         'currency_code', pi.currency_code
                       )
                     FROM payment.payout_instruction pi
                     LEFT JOIN billing.settlement_record sr
                       ON sr.settlement_id = pi.settlement_id
                     WHERE pi.payout_instruction_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await?
        }
        _ => None,
    };

    Ok(row.map(|row| ConsistencySubjectRecord {
        ref_type: ref_type.to_string(),
        ref_id: row.get(0),
        order_id: row.get(1),
        business_status: row.get(2),
        authority_model: row.get(3),
        business_state_version: row.get(4),
        proof_commit_state: row.get(5),
        proof_commit_policy: row.get(6),
        external_fact_status: row.get(7),
        reconcile_status: row.get(8),
        last_reconciled_at: row.get(9),
        snapshot: row.get(10),
    }))
}

pub async fn search_recent_outbox_events_for_aggregates(
    client: &(impl GenericClient + Sync),
    aggregate_types: &[String],
    aggregate_id: &str,
    limit: i64,
) -> Result<Vec<OutboxEventRecord>, Error> {
    let rows = client
        .query(
            "SELECT
               oe.outbox_event_id::text,
               oe.aggregate_type,
               oe.aggregate_id::text,
               oe.event_type,
               oe.payload,
               oe.status,
               oe.retry_count,
               oe.max_retries,
               to_char(oe.available_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(oe.published_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(oe.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               oe.request_id,
               oe.trace_id,
               oe.idempotency_key,
               oe.authority_scope,
               oe.source_of_truth,
               oe.proof_commit_policy,
               oe.target_bus,
               oe.target_topic,
               oe.partition_key,
               oe.ordering_key,
               oe.payload_hash,
               oe.last_error_code,
               oe.last_error_message,
               opa.outbox_publish_attempt_id::text,
               opa.worker_id,
               opa.target_bus,
               opa.target_topic,
               opa.attempt_no,
               opa.result_code,
               opa.error_code,
               opa.error_message,
               to_char(opa.attempted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(opa.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               opa.metadata
             FROM ops.outbox_event oe
             LEFT JOIN LATERAL (
               SELECT
                 outbox_publish_attempt_id,
                 worker_id,
                 target_bus,
                 target_topic,
                 attempt_no,
                 result_code,
                 error_code,
                 error_message,
                 attempted_at,
                 completed_at,
                 metadata
               FROM ops.outbox_publish_attempt
               WHERE outbox_event_id = oe.outbox_event_id
               ORDER BY attempt_no DESC, attempted_at DESC, outbox_publish_attempt_id DESC
               LIMIT 1
             ) opa ON true
             WHERE oe.aggregate_type = ANY($1::text[])
               AND oe.aggregate_id = $2::text::uuid
             ORDER BY oe.created_at DESC, oe.outbox_event_id DESC
             LIMIT $3",
            &[&aggregate_types, &aggregate_id, &limit],
        )
        .await?;

    Ok(rows.iter().map(parse_outbox_event_row).collect())
}

pub async fn search_recent_dead_letters_for_aggregates(
    client: &(impl GenericClient + Sync),
    aggregate_types: &[String],
    aggregate_id: &str,
    limit: i64,
) -> Result<Vec<DeadLetterEventRecord>, Error> {
    let rows = client
        .query(
            "SELECT
               dead_letter_event_id::text,
               outbox_event_id::text,
               aggregate_type,
               aggregate_id::text,
               event_type,
               payload,
               failed_reason,
               request_id,
               trace_id,
               authority_scope,
               source_of_truth,
               target_bus,
               target_topic,
               failure_stage,
               to_char(first_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               reprocess_status,
               to_char(reprocessed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.dead_letter_event
             WHERE aggregate_type = ANY($1::text[])
               AND aggregate_id = $2::text::uuid
             ORDER BY created_at DESC, dead_letter_event_id DESC
             LIMIT $3",
            &[&aggregate_types, &aggregate_id, &limit],
        )
        .await?;

    let outbox_event_ids: Vec<String> = rows
        .iter()
        .filter_map(|row| row.get::<_, Option<String>>(1))
        .collect();
    let idempotency_by_event =
        load_consumer_idempotency_for_event_ids(client, &outbox_event_ids).await?;

    Ok(rows
        .iter()
        .map(|row| {
            let outbox_event_id = row.get::<_, Option<String>>(1);
            let records = outbox_event_id
                .as_ref()
                .and_then(|event_id| idempotency_by_event.get(event_id))
                .cloned()
                .unwrap_or_default();
            parse_dead_letter_row(row, records)
        })
        .collect())
}

pub async fn search_recent_external_fact_receipts_for_refs(
    client: &(impl GenericClient + Sync),
    ref_types: &[String],
    ref_id: &str,
    order_id: Option<&str>,
    limit: i64,
) -> Result<ExternalFactReceiptPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.external_fact_receipt efr
             WHERE (efr.ref_type = ANY($1::text[]) AND efr.ref_id = $2::text::uuid)
                OR ($3::text IS NOT NULL AND efr.order_id = $3::text::uuid)",
            &[&ref_types, &ref_id, &order_id],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               external_fact_receipt_id::text,
               order_id::text,
               ref_domain,
               ref_type,
               ref_id::text,
               fact_type,
               provider_type,
               provider_key,
               provider_reference,
               receipt_status,
               receipt_payload,
               receipt_hash,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(received_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(confirmed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.external_fact_receipt efr
             WHERE (efr.ref_type = ANY($1::text[]) AND efr.ref_id = $2::text::uuid)
                OR ($3::text IS NOT NULL AND efr.order_id = $3::text::uuid)
             ORDER BY efr.received_at DESC, efr.external_fact_receipt_id DESC
             LIMIT $4",
            &[&ref_types, &ref_id, &order_id, &limit],
        )
        .await?;

    Ok(ExternalFactReceiptPage {
        total,
        items: rows.iter().map(parse_external_fact_receipt_row).collect(),
    })
}

pub async fn count_external_fact_receipts_by_status_for_refs(
    client: &(impl GenericClient + Sync),
    ref_types: &[String],
    ref_id: &str,
    order_id: Option<&str>,
) -> Result<Value, Error> {
    let rows = client
        .query(
            "SELECT receipt_status, COUNT(*)::bigint
             FROM ops.external_fact_receipt efr
             WHERE (efr.ref_type = ANY($1::text[]) AND efr.ref_id = $2::text::uuid)
                OR ($3::text IS NOT NULL AND efr.order_id = $3::text::uuid)
             GROUP BY receipt_status
             ORDER BY receipt_status ASC",
            &[&ref_types, &ref_id, &order_id],
        )
        .await?;

    let mut counts = Map::new();
    for row in rows {
        counts.insert(row.get::<_, String>(0), Value::from(row.get::<_, i64>(1)));
    }
    Ok(Value::Object(counts))
}

pub async fn search_recent_chain_projection_gaps_for_aggregates(
    client: &(impl GenericClient + Sync),
    aggregate_types: &[String],
    aggregate_id: &str,
    order_id: Option<&str>,
    limit: i64,
) -> Result<ChainProjectionGapPage, Error> {
    let total: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.chain_projection_gap cpg
             WHERE (cpg.aggregate_type = ANY($1::text[]) AND cpg.aggregate_id = $2::text::uuid)
                OR ($3::text IS NOT NULL AND cpg.order_id = $3::text::uuid)",
            &[&aggregate_types, &aggregate_id, &order_id],
        )
        .await?
        .get(0);

    let rows = client
        .query(
            "SELECT
               chain_projection_gap_id::text,
               aggregate_type,
               aggregate_id::text,
               order_id::text,
               chain_id,
               source_event_type,
               expected_tx_id,
               projected_tx_hash,
               gap_type,
               gap_status,
               to_char(first_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(last_detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               request_id,
               trace_id,
               outbox_event_id::text,
               anchor_id::text,
               resolution_summary,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM ops.chain_projection_gap cpg
             WHERE (cpg.aggregate_type = ANY($1::text[]) AND cpg.aggregate_id = $2::text::uuid)
                OR ($3::text IS NOT NULL AND cpg.order_id = $3::text::uuid)
             ORDER BY cpg.created_at DESC, cpg.chain_projection_gap_id DESC
             LIMIT $4",
            &[&aggregate_types, &aggregate_id, &order_id, &limit],
        )
        .await?;

    Ok(ChainProjectionGapPage {
        total,
        items: rows.iter().map(parse_chain_projection_gap_row).collect(),
    })
}

pub async fn count_chain_projection_gaps_by_status_for_aggregates(
    client: &(impl GenericClient + Sync),
    aggregate_types: &[String],
    aggregate_id: &str,
    order_id: Option<&str>,
) -> Result<Value, Error> {
    let rows = client
        .query(
            "SELECT gap_status, COUNT(*)::bigint
             FROM ops.chain_projection_gap cpg
             WHERE (cpg.aggregate_type = ANY($1::text[]) AND cpg.aggregate_id = $2::text::uuid)
                OR ($3::text IS NOT NULL AND cpg.order_id = $3::text::uuid)
             GROUP BY gap_status
             ORDER BY gap_status ASC",
            &[&aggregate_types, &aggregate_id, &order_id],
        )
        .await?;

    let mut counts = Map::new();
    for row in rows {
        counts.insert(row.get::<_, String>(0), Value::from(row.get::<_, i64>(1)));
    }
    Ok(Value::Object(counts))
}

pub async fn search_recent_chain_anchors_for_refs(
    client: &(impl GenericClient + Sync),
    ref_types: &[String],
    ref_id: &str,
    limit: i64,
) -> Result<Vec<ChainAnchorRecord>, Error> {
    let rows = client
        .query(
            "SELECT
               chain_anchor_id::text,
               chain_id,
               anchor_type,
               ref_type,
               ref_id::text,
               digest,
               tx_hash,
               status,
               to_char(anchored_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               authority_model,
               reconcile_status,
               to_char(last_reconciled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM chain.chain_anchor
             WHERE ref_type = ANY($1::text[])
               AND ref_id = $2::text::uuid
             ORDER BY created_at DESC, chain_anchor_id DESC
             LIMIT $3",
            &[&ref_types, &ref_id, &limit],
        )
        .await?;

    Ok(rows.iter().map(parse_chain_anchor_row).collect())
}

pub async fn search_recent_audit_traces_for_refs(
    client: &(impl GenericClient + Sync),
    ref_types: &[String],
    ref_id: &str,
    limit: i64,
) -> Result<Vec<AuditTraceView>, Error> {
    let rows = client
        .query(
            "SELECT
               audit_id::text,
               event_schema_version,
               event_class,
               domain_name,
               ref_type,
               ref_id::text,
               actor_id::text,
               actor_org_id::text,
               action_name,
               result_code,
               error_code,
               request_id,
               trace_id,
               tx_hash,
               evidence_manifest_id::text,
               event_hash,
               to_char(event_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM audit.audit_event
             WHERE ref_type = ANY($1::text[])
               AND ref_id = $2::text::uuid
             ORDER BY event_time DESC, audit_id DESC
             LIMIT $3",
            &[&ref_types, &ref_id, &limit],
        )
        .await?;

    Ok(rows.iter().map(parse_audit_trace_row).collect())
}

async fn load_consumer_idempotency_for_event_ids(
    client: &(impl GenericClient + Sync),
    event_ids: &[String],
) -> Result<std::collections::HashMap<String, Vec<ConsumerIdempotencyRecord>>, Error> {
    let mut records_by_event = std::collections::HashMap::new();
    if event_ids.is_empty() {
        return Ok(records_by_event);
    }

    let rows = client
        .query(
            "SELECT
               consumer_idempotency_record_id::text,
               consumer_name,
               event_id::text,
               aggregate_type,
               aggregate_id::text,
               trace_id,
               result_code,
               to_char(processed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata
             FROM ops.consumer_idempotency_record
             WHERE event_id::text = ANY($1::text[])
             ORDER BY processed_at DESC, consumer_idempotency_record_id DESC",
            &[&event_ids],
        )
        .await?;

    for row in rows {
        let record = parse_consumer_idempotency_row(&row);
        records_by_event
            .entry(record.event_id.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    Ok(records_by_event)
}

pub async fn record_access_audit(
    client: &(impl GenericClient + Sync),
    access: &AccessAuditInsert,
) -> Result<String, Error> {
    let row = client
        .query_one(
            "INSERT INTO audit.access_audit (
               accessor_user_id,
               accessor_role_key,
               access_mode,
               target_type,
               target_id,
               masked_view,
               breakglass_reason,
               step_up_challenge_id,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               $4,
               $5::text::uuid,
               $6,
               $7,
               $8::text::uuid,
               $9,
               $10,
               $11::jsonb
             )
             RETURNING access_audit_id::text",
            &[
                &access.accessor_user_id,
                &access.accessor_role_key,
                &access.access_mode,
                &access.target_type,
                &access.target_id,
                &access.masked_view,
                &access.breakglass_reason,
                &access.step_up_challenge_id,
                &access.request_id,
                &access.trace_id,
                &access.metadata,
            ],
        )
        .await?;
    Ok(row.get(0))
}

pub async fn record_system_log(
    client: &(impl GenericClient + Sync),
    log: &SystemLogInsert,
) -> Result<(), Error> {
    client
        .execute(
            "INSERT INTO ops.system_log (
               service_name,
               log_level,
               request_id,
               trace_id,
               message_text,
               structured_payload
             ) VALUES (
               $1,
               $2,
               $3,
               $4,
               $5,
               $6::jsonb
             )",
            &[
                &log.service_name,
                &log.log_level,
                &log.request_id,
                &log.trace_id,
                &log.message_text,
                &log.structured_payload,
            ],
        )
        .await?;
    Ok(())
}

fn parse_evidence_package_row(row: &Row) -> EvidencePackage {
    EvidencePackage {
        evidence_package_id: row.get(0),
        package_type: row.get(1),
        ref_type: row.get(2),
        ref_id: row.get(3),
        evidence_manifest_id: row.get(4),
        package_digest: row.get(5),
        storage_uri: row.get(6),
        created_by: row.get(7),
        created_at: row.get(8),
        retention_class: row.get(9),
        legal_hold_status: row.get(10),
        metadata: Value::Object(Map::new()),
    }
}

fn parse_replay_job_row(row: &Row) -> ReplayJob {
    ReplayJob {
        replay_job_id: row.get(0),
        replay_type: row.get(1),
        ref_type: row.get(2),
        ref_id: row.get(3),
        dry_run: row.get(4),
        status: row.get(5),
        requested_by: row.get(6),
        step_up_challenge_id: row.get(7),
        request_reason: row.get(8),
        options_json: row.get(9),
        created_at: row.get(10),
        started_at: row.get(11),
        finished_at: row.get(12),
        updated_at: row.get(13),
    }
}

fn parse_replay_result_row(row: &Row) -> ReplayResult {
    ReplayResult {
        replay_result_id: row.get(0),
        replay_job_id: row.get(1),
        step_name: row.get(2),
        result_code: row.get(3),
        expected_digest: row.get(4),
        actual_digest: row.get(5),
        diff_summary: row.get(6),
        created_at: row.get(7),
    }
}

fn parse_legal_hold_row(row: &Row) -> LegalHold {
    LegalHold {
        legal_hold_id: row.get(0),
        hold_scope_type: row.get(1),
        hold_scope_id: row.get(2),
        reason_code: row.get(3),
        status: row.get(4),
        retention_policy_id: row.get(5),
        requested_by: row.get(6),
        approved_by: row.get(7),
        hold_until: row.get(8),
        created_at: row.get(9),
        released_at: row.get(10),
        updated_at: row.get(11),
        metadata: row.get(12),
    }
}

fn parse_anchor_batch_row(row: &Row) -> AnchorBatch {
    let mut metadata = metadata_object(row.get(12));
    if let Some(tx_hash) = row.get::<_, Option<String>>(14) {
        metadata.insert("tx_hash".to_string(), Value::String(tx_hash));
    }
    if let Some(chain_anchor_status) = row.get::<_, Option<String>>(15) {
        metadata.insert(
            "chain_anchor_status".to_string(),
            Value::String(chain_anchor_status),
        );
    }
    if let Some(authority_model) = row.get::<_, Option<String>>(16) {
        metadata.insert(
            "authority_model".to_string(),
            Value::String(authority_model),
        );
    }
    if let Some(reconcile_status) = row.get::<_, Option<String>>(17) {
        metadata.insert(
            "reconcile_status".to_string(),
            Value::String(reconcile_status),
        );
    }
    if let Some(last_reconciled_at) = row.get::<_, Option<String>>(18) {
        metadata.insert(
            "last_reconciled_at".to_string(),
            Value::String(last_reconciled_at),
        );
    }

    AnchorBatch {
        anchor_batch_id: row.get(0),
        batch_scope: row.get(1),
        chain_id: row.get(2),
        record_count: row.get(3),
        batch_root: row.get(4),
        window_started_at: row.get(5),
        window_ended_at: row.get(6),
        status: row.get(7),
        chain_anchor_id: row.get(8),
        created_by: row.get(9),
        created_at: row.get(10),
        anchored_at: row.get(11),
        metadata: Value::Object(metadata),
        updated_at: row.get(13),
    }
}

fn storage_metadata(event: &AuditEvent) -> Value {
    let mut metadata = metadata_object(event.metadata.clone());

    if let Some(tenant_id) = &event.tenant_id {
        metadata
            .entry("tenant_id".to_string())
            .or_insert_with(|| Value::String(tenant_id.clone()));
    }

    if !event.evidence.is_empty() {
        metadata
            .entry("evidence_items".to_string())
            .or_insert_with(|| {
                Value::Array(
                    event
                        .evidence
                        .iter()
                        .map(|item| {
                            serde_json::json!({
                                "item_type": item.item_type,
                                "ref_type": item.ref_type,
                                "ref_id": item.ref_id,
                                "object_uri": item.object_uri,
                                "object_hash": item.object_hash,
                            })
                        })
                        .collect(),
                )
            });
    }

    Value::Object(metadata)
}

fn parse_audit_trace_row(row: &Row) -> AuditTraceView {
    AuditTraceView {
        audit_id: row.get(0),
        event_schema_version: row.get(1),
        event_class: row.get(2),
        domain_name: row.get(3),
        ref_type: row.get(4),
        ref_id: row.get(5),
        actor_id: row.get(6),
        actor_org_id: row.get(7),
        tenant_id: None,
        action_name: row.get(8),
        result_code: row.get(9),
        error_code: row.get(10),
        request_id: row.get(11),
        trace_id: row.get(12),
        tx_hash: row.get(13),
        evidence_manifest_id: row.get(14),
        event_hash: row.get(15),
        occurred_at: row.get(16),
    }
}

fn parse_outbox_publish_attempt_from_row(
    row: &Row,
    start: usize,
) -> Option<OutboxPublishAttemptRecord> {
    let outbox_publish_attempt_id = row.get::<_, Option<String>>(start);
    outbox_publish_attempt_id.as_ref()?;
    Some(OutboxPublishAttemptRecord {
        outbox_publish_attempt_id,
        worker_id: row.get(start + 1),
        target_bus: row.get(start + 2),
        target_topic: row.get(start + 3),
        attempt_no: row.get(start + 4),
        result_code: row.get(start + 5),
        error_code: row.get(start + 6),
        error_message: row.get(start + 7),
        attempted_at: row.get(start + 8),
        completed_at: row.get(start + 9),
        metadata: row.get(start + 10),
    })
}

fn parse_outbox_event_row(row: &Row) -> OutboxEventRecord {
    OutboxEventRecord {
        outbox_event_id: row.get(0),
        aggregate_type: row.get(1),
        aggregate_id: row.get(2),
        event_type: row.get(3),
        payload: row.get(4),
        status: row.get(5),
        retry_count: row.get(6),
        max_retries: row.get(7),
        available_at: row.get(8),
        published_at: row.get(9),
        created_at: row.get(10),
        request_id: row.get(11),
        trace_id: row.get(12),
        idempotency_key: row.get(13),
        authority_scope: row.get(14),
        source_of_truth: row.get(15),
        proof_commit_policy: row.get(16),
        target_bus: row.get(17),
        target_topic: row.get(18),
        partition_key: row.get(19),
        ordering_key: row.get(20),
        payload_hash: row.get(21),
        last_error_code: row.get(22),
        last_error_message: row.get(23),
        latest_publish_attempt: parse_outbox_publish_attempt_from_row(row, 24),
    }
}

fn parse_consumer_idempotency_row(row: &Row) -> ConsumerIdempotencyRecord {
    ConsumerIdempotencyRecord {
        consumer_idempotency_record_id: row.get(0),
        consumer_name: row.get(1),
        event_id: row.get(2),
        aggregate_type: row.get(3),
        aggregate_id: row.get(4),
        trace_id: row.get(5),
        result_code: row.get(6),
        processed_at: row.get(7),
        metadata: row.get(8),
    }
}

fn parse_dead_letter_row(
    row: &Row,
    consumer_idempotency_records: Vec<ConsumerIdempotencyRecord>,
) -> DeadLetterEventRecord {
    DeadLetterEventRecord {
        dead_letter_event_id: row.get(0),
        outbox_event_id: row.get(1),
        aggregate_type: row.get(2),
        aggregate_id: row.get(3),
        event_type: row.get(4),
        payload: row.get(5),
        failed_reason: row.get(6),
        request_id: row.get(7),
        trace_id: row.get(8),
        authority_scope: row.get(9),
        source_of_truth: row.get(10),
        target_bus: row.get(11),
        target_topic: row.get(12),
        failure_stage: row.get(13),
        first_failed_at: row.get(14),
        last_failed_at: row.get(15),
        reprocess_status: row.get(16),
        reprocessed_at: row.get(17),
        created_at: row.get(18),
        consumer_idempotency_records,
    }
}

fn parse_external_fact_receipt_row(row: &Row) -> ExternalFactReceiptRecord {
    ExternalFactReceiptRecord {
        external_fact_receipt_id: row.get(0),
        order_id: row.get(1),
        ref_domain: row.get(2),
        ref_type: row.get(3),
        ref_id: row.get(4),
        fact_type: row.get(5),
        provider_type: row.get(6),
        provider_key: row.get(7),
        provider_reference: row.get(8),
        receipt_status: row.get(9),
        receipt_payload: row.get(10),
        receipt_hash: row.get(11),
        occurred_at: row.get(12),
        received_at: row.get(13),
        confirmed_at: row.get(14),
        request_id: row.get(15),
        trace_id: row.get(16),
        metadata: row.get(17),
        created_at: row.get(18),
        updated_at: row.get(19),
    }
}

fn parse_chain_projection_gap_row(row: &Row) -> ChainProjectionGapRecord {
    ChainProjectionGapRecord {
        chain_projection_gap_id: row.get(0),
        aggregate_type: row.get(1),
        aggregate_id: row.get(2),
        order_id: row.get(3),
        chain_id: row.get(4),
        source_event_type: row.get(5),
        expected_tx_id: row.get(6),
        projected_tx_hash: row.get(7),
        gap_type: row.get(8),
        gap_status: row.get(9),
        first_detected_at: row.get(10),
        last_detected_at: row.get(11),
        resolved_at: row.get(12),
        request_id: row.get(13),
        trace_id: row.get(14),
        outbox_event_id: row.get(15),
        anchor_id: row.get(16),
        resolution_summary: row.get(17),
        metadata: row.get(18),
        created_at: row.get(19),
        updated_at: row.get(20),
    }
}

fn parse_trade_lifecycle_checkpoint_row(row: &Row) -> TradeLifecycleCheckpointRecord {
    TradeLifecycleCheckpointRecord {
        trade_lifecycle_checkpoint_id: row.get(0),
        monitoring_policy_profile_id: row.get(1),
        order_id: row.get(2),
        ref_domain: row.get(3),
        ref_type: row.get(4),
        ref_id: row.get(5),
        checkpoint_code: row.get(6),
        lifecycle_stage: row.get(7),
        checkpoint_status: row.get(8),
        expected_by: row.get(9),
        occurred_at: row.get(10),
        source_type: row.get(11),
        source_ref_type: row.get(12),
        source_ref_id: row.get(13),
        related_tx_hash: row.get(14),
        request_id: row.get(15),
        trace_id: row.get(16),
        metadata: row.get(17),
        created_at: row.get(18),
        updated_at: row.get(19),
    }
}

fn parse_fairness_incident_row(row: &Row) -> FairnessIncidentRecord {
    FairnessIncidentRecord {
        fairness_incident_id: row.get(0),
        order_id: row.get(1),
        ref_type: row.get(2),
        ref_id: row.get(3),
        incident_type: row.get(4),
        severity: row.get(5),
        lifecycle_stage: row.get(6),
        detected_by_type: row.get(7),
        source_checkpoint_id: row.get(8),
        source_receipt_id: row.get(9),
        fairness_incident_status: row.get(10),
        auto_action_code: row.get(11),
        assigned_role_key: row.get(12),
        assigned_user_id: row.get(13),
        resolution_summary: row.get(14),
        request_id: row.get(15),
        trace_id: row.get(16),
        metadata: row.get(17),
        created_at: row.get(18),
        closed_at: row.get(19),
        updated_at: row.get(20),
    }
}

fn parse_observability_backend_row(row: &Row) -> ObservabilityBackendRecord {
    ObservabilityBackendRecord {
        observability_backend_id: row.get(0),
        backend_key: row.get(1),
        backend_type: row.get(2),
        endpoint_uri: row.get(3),
        auth_mode: row.get(4),
        enabled: row.get(5),
        stage_from: row.get(6),
        capability_json: row.get(7),
        metadata: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    }
}

fn parse_system_log_mirror_row(row: &Row) -> SystemLogMirrorRecord {
    SystemLogMirrorRecord {
        system_log_id: row.get(0),
        service_name: row.get(1),
        logger_name: row.get(2),
        log_level: row.get(3),
        severity_number: row.get(4),
        environment_code: row.get(5),
        host_name: row.get(6),
        node_name: row.get(7),
        pod_name: row.get(8),
        backend_type: row.get(9),
        request_id: row.get(10),
        trace_id: row.get(11),
        message_text: row.get(12),
        structured_payload: row.get(13),
        object_type: row.get(14),
        object_id: row.get(15),
        masked_status: row.get(16),
        retention_class: row.get(17),
        legal_hold_status: row.get(18),
        resource_attrs: row.get(19),
        created_at: row.get(20),
    }
}

fn parse_trace_index_row(row: &Row) -> TraceIndexRecord {
    TraceIndexRecord {
        trace_index_id: row.get(0),
        trace_id: row.get(1),
        traceparent: row.get(2),
        backend_key: row.get(3),
        root_service_name: row.get(4),
        root_span_name: row.get(5),
        request_id: row.get(6),
        ref_type: row.get(7),
        ref_id: row.get(8),
        object_type: row.get(9),
        object_id: row.get(10),
        status: row.get(11),
        span_count: row.get(12),
        started_at: row.get(13),
        ended_at: row.get(14),
        metadata: row.get(15),
        created_at: row.get(16),
        updated_at: row.get(17),
    }
}

fn parse_alert_event_row(row: &Row) -> AlertEventRecord {
    AlertEventRecord {
        alert_event_id: row.get(0),
        alert_rule_id: row.get(1),
        source_backend_key: row.get(2),
        fingerprint: row.get(3),
        alert_type: row.get(4),
        severity: row.get(5),
        title_text: row.get(6),
        summary_text: row.get(7),
        ref_type: row.get(8),
        ref_id: row.get(9),
        request_id: row.get(10),
        trace_id: row.get(11),
        labels_json: row.get(12),
        annotations_json: row.get(13),
        status: row.get(14),
        acknowledged_by: row.get(15),
        acknowledged_at: row.get(16),
        fired_at: row.get(17),
        resolved_at: row.get(18),
        metadata: row.get(19),
        incident_ticket_id: row.get(20),
        created_at: row.get(21),
        updated_at: row.get(22),
    }
}

fn parse_incident_ticket_row(row: &Row) -> IncidentTicketRecord {
    IncidentTicketRecord {
        incident_ticket_id: row.get(0),
        incident_key: row.get(1),
        source_alert_event_id: row.get(2),
        severity: row.get(3),
        title_text: row.get(4),
        summary_text: row.get(5),
        status: row.get(6),
        owner_role_key: row.get(7),
        owner_user_id: row.get(8),
        runbook_uri: row.get(9),
        impact_summary: row.get(10),
        root_cause_summary: row.get(11),
        latest_event_type: row.get(12),
        latest_event_note: row.get(13),
        metadata: row.get(14),
        started_at: row.get(15),
        resolved_at: row.get(16),
        created_at: row.get(17),
        updated_at: row.get(18),
    }
}

fn parse_slo_row(row: &Row) -> SloRecord {
    SloRecord {
        slo_definition_id: row.get(0),
        slo_key: row.get(1),
        service_name: row.get(2),
        indicator_type: row.get(3),
        objective_value: row.get(4),
        window_code: row.get(5),
        source_backend_key: row.get(6),
        alert_rule_id: row.get(7),
        status: row.get(8),
        current_value: row.get(9),
        budget_remaining: row.get(10),
        snapshot_status: row.get(11),
        window_started_at: row.get(12),
        window_ended_at: row.get(13),
        metadata: row.get(14),
        created_at: row.get(15),
        updated_at: row.get(16),
    }
}

fn parse_chain_anchor_row(row: &Row) -> ChainAnchorRecord {
    ChainAnchorRecord {
        chain_anchor_id: row.get(0),
        chain_id: row.get(1),
        anchor_type: row.get(2),
        ref_type: row.get(3),
        ref_id: row.get(4),
        digest: row.get(5),
        tx_hash: row.get(6),
        status: row.get(7),
        anchored_at: row.get(8),
        created_at: row.get(9),
        authority_model: row.get(10),
        reconcile_status: row.get(11),
        last_reconciled_at: row.get(12),
    }
}
