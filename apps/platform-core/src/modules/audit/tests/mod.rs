use crate::modules::audit::dto::AuditTraceView;
use crate::modules::audit::repo::{AuditEventInsert, INSERT_AUDIT_EVENT_SQL};
use audit_kit::{AuditContext, AuditEvent, EvidenceItem};

fn sample_event() -> AuditEvent {
    let mut context = AuditContext::minimal("req-aud-1", "trace-aud-1", "user-1", "tenant-1");
    context.actor_org_id = Some("org-1".to_string());
    context.auth_assurance_level = Some("aal2".to_string());
    context.step_up_challenge_id = Some("step-up-1".to_string());
    context.metadata = serde_json::json!({
        "actor_role": "tenant_admin",
        "request_channel": "api",
    });

    let mut event = AuditEvent::business(
        "audit",
        "evidence_manifest",
        Some("manifest-1".to_string()),
        "audit.export.request",
        "accepted",
        context,
    );
    event.error_code = Some("AUDIT_EXPORT_SCOPE_FORBIDDEN".to_string());
    event.tx_hash = Some("tx-1".to_string());
    event.event_hash = Some("hash-1".to_string());
    event.evidence_manifest_id = Some("manifest-1".to_string());
    event.occurred_at = Some("2026-04-22T10:00:00.000Z".to_string());
    event.evidence = vec![EvidenceItem {
        evidence_item_id: Some("item-1".to_string()),
        item_type: "payment_webhook_raw".to_string(),
        ref_type: "payment_intent".to_string(),
        ref_id: Some("pi-1".to_string()),
        object_uri: Some("s3://audit/raw/pi-1.json".to_string()),
        object_hash: Some("obj-hash-1".to_string()),
        content_type: Some("application/json".to_string()),
        size_bytes: Some(512),
        source_system: Some("mock-payment-provider".to_string()),
        storage_mode: Some("minio".to_string()),
        retention_policy_id: None,
        worm_enabled: false,
        legal_hold_status: "none".to_string(),
        created_by: Some("user-1".to_string()),
        created_at: Some("2026-04-22T10:00:01.000Z".to_string()),
        metadata: serde_json::json!({ "bucket": "audit-evidence" }),
    }];
    event
}

#[test]
fn audit_event_maps_to_writer_foundation_with_tenant_and_evidence() {
    let event = sample_event();
    let insert = AuditEventInsert::from(&event);

    assert!(INSERT_AUDIT_EVENT_SQL.contains("INSERT INTO audit.audit_event"));
    assert_eq!(insert.domain_name, "audit");
    assert_eq!(insert.ref_type, "evidence_manifest");
    assert_eq!(insert.ref_id.as_deref(), Some("manifest-1"));
    assert_eq!(insert.request_id.as_deref(), Some("req-aud-1"));
    assert_eq!(insert.trace_id.as_deref(), Some("trace-aud-1"));
    assert_eq!(
        insert.metadata["tenant_id"].as_str(),
        Some("tenant-1"),
        "tenant_id must survive mapping even though DB stores it in metadata"
    );
    assert_eq!(
        insert.metadata["evidence_items"][0]["object_uri"].as_str(),
        Some("s3://audit/raw/pi-1.json")
    );
}

#[test]
fn audit_trace_view_keeps_lookup_and_export_fields_aligned() {
    let event = sample_event();
    let view = AuditTraceView::from(&event);

    assert_eq!(view.domain_name, "audit");
    assert_eq!(view.ref_type, "evidence_manifest");
    assert_eq!(view.ref_id.as_deref(), Some("manifest-1"));
    assert_eq!(view.actor_id.as_deref(), Some("user-1"));
    assert_eq!(view.actor_org_id.as_deref(), Some("org-1"));
    assert_eq!(view.tenant_id.as_deref(), Some("tenant-1"));
    assert_eq!(view.request_id.as_deref(), Some("req-aud-1"));
    assert_eq!(view.trace_id.as_deref(), Some("trace-aud-1"));
    assert_eq!(view.tx_hash.as_deref(), Some("tx-1"));
    assert_eq!(view.evidence_manifest_id.as_deref(), Some("manifest-1"));
    assert_eq!(view.event_hash.as_deref(), Some("hash-1"));
    assert_eq!(
        view.occurred_at.as_deref(),
        Some("2026-04-22T10:00:00.000Z")
    );
}
