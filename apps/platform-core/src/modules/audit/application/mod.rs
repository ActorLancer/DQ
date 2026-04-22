use crate::modules::audit::domain::{
    AuditEvent, EvidenceItem, EvidenceManifest, EvidenceManifestItem,
};
use crate::modules::audit::repo::{
    INSERT_EVIDENCE_ITEM_SQL, INSERT_EVIDENCE_MANIFEST_ITEM_SQL, INSERT_EVIDENCE_MANIFEST_SQL,
    insert_audit_event, metadata_object,
};
use db::{Error, GenericClient, Row};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyEvidenceBridge {
    pub legacy_table: String,
    pub legacy_object_id: String,
    pub legacy_parent_type: String,
    pub legacy_parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditWriteCommand {
    pub domain_name: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub actor_org_id: Option<String>,
    pub tenant_id: Option<String>,
    pub action_name: String,
    pub result_code: String,
    pub error_code: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub auth_assurance_level: Option<String>,
    pub step_up_challenge_id: Option<String>,
    pub sensitivity_level: Option<String>,
    pub metadata: Value,
}

pub fn build_audit_event(command: &AuditWriteCommand) -> AuditEvent {
    AuditEvent {
        audit_id: None,
        event_schema_version: "v1".to_string(),
        event_class: "business".to_string(),
        domain_name: command.domain_name.clone(),
        ref_type: command.ref_type.clone(),
        ref_id: command.ref_id.clone(),
        actor_type: command.actor_type.clone(),
        actor_id: command.actor_id.clone(),
        actor_org_id: command.actor_org_id.clone(),
        tenant_id: command.tenant_id.clone(),
        session_id: None,
        trusted_device_id: None,
        application_id: None,
        request_id: command.request_id.clone(),
        trace_id: command.trace_id.clone(),
        parent_audit_id: None,
        action_name: command.action_name.clone(),
        result_code: command.result_code.clone(),
        error_code: command.error_code.clone(),
        source_ip: None,
        client_fingerprint: None,
        auth_assurance_level: command.auth_assurance_level.clone(),
        step_up_challenge_id: command.step_up_challenge_id.clone(),
        before_state_digest: None,
        after_state_digest: None,
        tx_hash: None,
        previous_event_hash: None,
        event_hash: None,
        evidence_hash: None,
        payload_digest: None,
        evidence_manifest_id: None,
        anchor_policy: "batched_fabric".to_string(),
        retention_class: "audit_default".to_string(),
        legal_hold_status: "none".to_string(),
        sensitivity_level: command
            .sensitivity_level
            .clone()
            .unwrap_or_else(|| "normal".to_string()),
        occurred_at: None,
        ingested_at: None,
        metadata: Value::Object(metadata_object(command.metadata.clone())),
        evidence: Vec::new(),
    }
}

pub async fn write_audit_event(
    client: &(impl GenericClient + Sync),
    command: &AuditWriteCommand,
) -> Result<AuditEvent, Error> {
    let event = build_audit_event(command);
    insert_audit_event(client, &event).await?;
    Ok(event)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceWriteCommand {
    pub item_type: String,
    pub ref_type: String,
    pub ref_id: Option<String>,
    pub object_uri: String,
    pub object_hash: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub source_system: String,
    pub storage_mode: String,
    pub retention_policy_id: Option<String>,
    pub worm_enabled: bool,
    pub legal_hold_status: String,
    pub created_by: Option<String>,
    pub metadata: Value,
    pub manifest_scope: String,
    pub manifest_ref_type: String,
    pub manifest_ref_id: Option<String>,
    pub manifest_storage_uri: Option<String>,
    pub manifest_metadata: Value,
    pub legacy_bridge: Option<LegacyEvidenceBridge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceWriteResult {
    pub evidence_item: EvidenceItem,
    pub evidence_manifest: EvidenceManifest,
    pub manifest_items: Vec<EvidenceManifestItem>,
}

pub async fn record_evidence_snapshot(
    client: &(impl GenericClient + Sync),
    command: &EvidenceWriteCommand,
) -> Result<EvidenceWriteResult, Error> {
    let evidence_item = insert_evidence_item(client, command).await?;
    let snapshot_items = load_scope_evidence_items(
        client,
        &command.manifest_ref_type,
        command.manifest_ref_id.as_deref(),
    )
    .await?;
    let manifest_payload = build_manifest_payload(command, &snapshot_items);
    let manifest_hash = sha256_hex(manifest_payload.to_string().as_bytes());
    let evidence_manifest = insert_evidence_manifest(
        client,
        command,
        &manifest_hash,
        snapshot_items.len() as i32,
        manifest_payload,
    )
    .await?;

    let mut manifest_items = Vec::with_capacity(snapshot_items.len());
    for (index, item) in snapshot_items.iter().enumerate() {
        manifest_items.push(
            insert_evidence_manifest_item(
                client,
                evidence_manifest.evidence_manifest_id.as_deref(),
                item.evidence_item_id.as_deref(),
                item.object_hash
                    .as_deref()
                    .unwrap_or_else(|| item.object_uri.as_deref().unwrap_or_default()),
                index as i32 + 1,
            )
            .await?,
        );
    }

    Ok(EvidenceWriteResult {
        evidence_item,
        evidence_manifest,
        manifest_items,
    })
}

pub async fn bridge_support_evidence_object(
    client: &(impl GenericClient + Sync),
    legacy_evidence_id: &str,
    result: &EvidenceWriteResult,
) -> Result<(), Error> {
    client
        .execute(
            "UPDATE support.evidence_object
             SET metadata = COALESCE(metadata, '{}'::jsonb) || $2::jsonb
             WHERE evidence_id = $1::text::uuid",
            &[&legacy_evidence_id, &bridge_metadata(result)],
        )
        .await?;
    Ok(())
}

pub fn bridge_metadata(result: &EvidenceWriteResult) -> Value {
    json!({
        "audit_evidence_item_id": result.evidence_item.evidence_item_id,
        "audit_evidence_manifest_id": result.evidence_manifest.evidence_manifest_id,
        "audit_manifest_hash": result.evidence_manifest.manifest_hash,
    })
}

async fn insert_evidence_item(
    client: &(impl GenericClient + Sync),
    command: &EvidenceWriteCommand,
) -> Result<EvidenceItem, Error> {
    let mut metadata = metadata_object(command.metadata.clone());
    metadata.insert(
        "evidence_writer".to_string(),
        json!({
            "authority_table": "audit.evidence_item",
            "manifest_ref_type": command.manifest_ref_type,
            "manifest_ref_id": command.manifest_ref_id,
        }),
    );
    if let Some(bridge) = &command.legacy_bridge {
        metadata.insert(
            "legacy_bridge".to_string(),
            serde_json::to_value(bridge).expect("legacy bridge should serialize"),
        );
    }

    let row = client
        .query_one(
            INSERT_EVIDENCE_ITEM_SQL,
            &[
                &command.item_type,
                &command.ref_type,
                &command.ref_id,
                &command.object_uri,
                &command.object_hash,
                &command.content_type,
                &command.size_bytes,
                &command.source_system,
                &command.storage_mode,
                &command.retention_policy_id,
                &command.worm_enabled,
                &command.legal_hold_status,
                &command.created_by,
                &Value::Object(metadata),
            ],
        )
        .await?;
    Ok(parse_evidence_item_row(&row))
}

async fn insert_evidence_manifest(
    client: &(impl GenericClient + Sync),
    command: &EvidenceWriteCommand,
    manifest_hash: &str,
    item_count: i32,
    manifest_payload: Value,
) -> Result<EvidenceManifest, Error> {
    let mut metadata = metadata_object(command.manifest_metadata.clone());
    metadata.insert("manifest_snapshot".to_string(), manifest_payload);
    if let Some(bridge) = &command.legacy_bridge {
        metadata.insert(
            "legacy_bridge".to_string(),
            serde_json::to_value(bridge).expect("legacy bridge should serialize"),
        );
    }

    let row = client
        .query_one(
            INSERT_EVIDENCE_MANIFEST_SQL,
            &[
                &command.manifest_scope,
                &command.manifest_ref_type,
                &command.manifest_ref_id,
                &manifest_hash,
                &item_count,
                &command.manifest_storage_uri,
                &command.created_by,
                &Value::Object(metadata),
            ],
        )
        .await?;
    Ok(parse_evidence_manifest_row(&row))
}

async fn insert_evidence_manifest_item(
    client: &(impl GenericClient + Sync),
    evidence_manifest_id: Option<&str>,
    evidence_item_id: Option<&str>,
    item_digest: &str,
    ordinal_no: i32,
) -> Result<EvidenceManifestItem, Error> {
    let row = client
        .query_one(
            INSERT_EVIDENCE_MANIFEST_ITEM_SQL,
            &[
                &evidence_manifest_id,
                &evidence_item_id,
                &item_digest,
                &ordinal_no,
            ],
        )
        .await?;
    Ok(parse_evidence_manifest_item_row(&row))
}

async fn load_scope_evidence_items(
    client: &(impl GenericClient + Sync),
    ref_type: &str,
    ref_id: Option<&str>,
) -> Result<Vec<EvidenceItem>, Error> {
    let rows = if let Some(ref_id) = ref_id {
        client
            .query(
                "SELECT
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
                 FROM audit.evidence_item
                 WHERE ref_type = $1
                   AND ref_id = $2::text::uuid
                 ORDER BY created_at ASC, evidence_item_id ASC",
                &[&ref_type, &ref_id],
            )
            .await?
    } else {
        client
            .query(
                "SELECT
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
                 FROM audit.evidence_item
                 WHERE ref_type = $1
                   AND ref_id IS NULL
                 ORDER BY created_at ASC, evidence_item_id ASC",
                &[&ref_type],
            )
            .await?
    };
    Ok(rows
        .into_iter()
        .map(|row| parse_evidence_item_row(&row))
        .collect())
}

fn build_manifest_payload(command: &EvidenceWriteCommand, items: &[EvidenceItem]) -> Value {
    json!({
        "manifest_scope": command.manifest_scope,
        "ref_type": command.manifest_ref_type,
        "ref_id": command.manifest_ref_id,
        "item_count": items.len(),
        "items": items.iter().enumerate().map(|(index, item)| json!({
            "ordinal_no": index + 1,
            "evidence_item_id": item.evidence_item_id,
            "item_type": item.item_type,
            "object_hash": item.object_hash,
            "object_uri": item.object_uri,
            "content_type": item.content_type,
            "size_bytes": item.size_bytes,
        })).collect::<Vec<_>>(),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn parse_evidence_item_row(row: &Row) -> EvidenceItem {
    EvidenceItem {
        evidence_item_id: row.get(0),
        item_type: row.get(1),
        ref_type: row.get(2),
        ref_id: row.get(3),
        object_uri: row.get(4),
        object_hash: row.get(5),
        content_type: row.get(6),
        size_bytes: row.get(7),
        source_system: row.get(8),
        storage_mode: row.get(9),
        retention_policy_id: row.get(10),
        worm_enabled: row.get(11),
        legal_hold_status: row.get(12),
        created_by: row.get(13),
        created_at: row.get(14),
        metadata: row.get(15),
    }
}

fn parse_evidence_manifest_row(row: &Row) -> EvidenceManifest {
    EvidenceManifest {
        evidence_manifest_id: row.get(0),
        manifest_scope: row.get(1),
        ref_type: row.get(2),
        ref_id: row.get(3),
        manifest_hash: row.get(4),
        item_count: row.get(5),
        storage_uri: row.get(6),
        created_by: row.get(7),
        created_at: row.get(8),
        metadata: row.get(9),
    }
}

fn parse_evidence_manifest_item_row(row: &Row) -> EvidenceManifestItem {
    EvidenceManifestItem {
        evidence_manifest_item_id: row.get(0),
        evidence_manifest_id: row.get(1),
        evidence_item_id: row.get(2),
        item_digest: row.get(3),
        ordinal_no: row.get(4),
        created_at: row.get(5),
    }
}
