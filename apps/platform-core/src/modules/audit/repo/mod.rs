use audit_kit::AuditEvent;
use serde_json::{Map, Value};

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

fn storage_metadata(event: &AuditEvent) -> Value {
    let mut metadata = match event.metadata.clone() {
        Value::Object(map) => map,
        Value::Null => Map::new(),
        raw => {
            let mut map = Map::new();
            map.insert("raw_metadata".to_string(), raw);
            map
        }
    };

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
