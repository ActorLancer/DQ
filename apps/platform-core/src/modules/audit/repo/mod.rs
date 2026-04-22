use audit_kit::{AuditEvent, EvidenceItem, EvidenceManifest, EvidenceManifestItem};
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
  $33, $34::timestamptz, $35::timestamptz, $36::jsonb
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
