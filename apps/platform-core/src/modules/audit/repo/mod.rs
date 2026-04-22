use audit_kit::{
    AuditEvent, EvidenceItem, EvidenceManifest, EvidenceManifestItem, EvidencePackage, ReplayJob,
    ReplayResult,
};
use db::{Error, GenericClient, Row};
use serde_json::{Map, Value};

use crate::modules::audit::domain::AuditTraceQuery;
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
pub struct ReplayJobDetail {
    pub replay_job: ReplayJob,
    pub results: Vec<ReplayResult>,
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
