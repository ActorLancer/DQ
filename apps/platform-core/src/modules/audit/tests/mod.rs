mod api_db;

use crate::modules::audit::api::{AuditPermission, canonical_role_key, is_allowed};
use crate::modules::audit::dto::{
    AuditTraceView, EvidenceItemView, EvidenceManifestItemView, EvidenceManifestView,
};
use crate::modules::audit::repo::{AuditEventInsert, INSERT_AUDIT_EVENT_SQL};
use audit_kit::{AuditContext, AuditEvent, EvidenceItem, EvidenceManifest, EvidenceManifestItem};

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

#[test]
fn evidence_views_preserve_export_replay_legal_hold_and_history_bridge_fields() {
    let evidence_item = EvidenceItem {
        evidence_item_id: Some("item-bridge-1".to_string()),
        item_type: "dispute_attachment".to_string(),
        ref_type: "dispute_case".to_string(),
        ref_id: Some("case-1".to_string()),
        object_uri: Some("s3://audit-evidence/dispute/case-1/item-1.bin".to_string()),
        object_hash: Some("object-hash-1".to_string()),
        content_type: Some("application/octet-stream".to_string()),
        size_bytes: Some(2048),
        source_system: Some("billing.dispute".to_string()),
        storage_mode: Some("minio".to_string()),
        retention_policy_id: Some("retention-90d".to_string()),
        worm_enabled: false,
        legal_hold_status: "active".to_string(),
        created_by: Some("user-1".to_string()),
        created_at: Some("2026-04-22T11:00:00.000Z".to_string()),
        metadata: serde_json::json!({
            "legacy_bridge": {
                "legacy_table": "support.evidence_object",
                "legacy_object_id": "legacy-evidence-1",
            },
            "export_scope": "regulator_batch",
            "replay_scope": "forensic_replay",
        }),
    };
    let evidence_manifest = EvidenceManifest {
        evidence_manifest_id: Some("manifest-bridge-1".to_string()),
        manifest_scope: "audit_export_package".to_string(),
        ref_type: "audit_export".to_string(),
        ref_id: Some("export-1".to_string()),
        manifest_hash: "manifest-hash-1".to_string(),
        item_count: 1,
        storage_uri: Some("s3://audit-evidence/manifests/export-1.json".to_string()),
        created_by: Some("user-1".to_string()),
        created_at: Some("2026-04-22T11:01:00.000Z".to_string()),
        metadata: serde_json::json!({
            "legal_hold_case_id": "legal-hold-1",
            "replay_job_id": "replay-1",
            "export_job_id": "export-1",
        }),
    };
    let manifest_item = EvidenceManifestItem {
        evidence_manifest_item_id: Some("manifest-item-1".to_string()),
        evidence_manifest_id: evidence_manifest.evidence_manifest_id.clone(),
        evidence_item_id: evidence_item.evidence_item_id.clone(),
        item_digest: "object-hash-1".to_string(),
        ordinal_no: 1,
        created_at: Some("2026-04-22T11:01:01.000Z".to_string()),
    };

    let item_view = EvidenceItemView::from(&evidence_item);
    let manifest_view = EvidenceManifestView::from(&evidence_manifest);
    let manifest_item_view = EvidenceManifestItemView::from(&manifest_item);

    assert_eq!(item_view.object_uri, evidence_item.object_uri);
    assert_eq!(item_view.object_hash, evidence_item.object_hash);
    assert_eq!(item_view.source_system.as_deref(), Some("billing.dispute"));
    assert_eq!(item_view.storage_mode.as_deref(), Some("minio"));
    assert_eq!(
        item_view.retention_policy_id.as_deref(),
        Some("retention-90d")
    );
    assert_eq!(item_view.legal_hold_status, "active");
    assert_eq!(
        item_view.metadata["legacy_bridge"]["legacy_table"].as_str(),
        Some("support.evidence_object")
    );
    assert_eq!(
        item_view.metadata["replay_scope"].as_str(),
        Some("forensic_replay")
    );
    assert_eq!(
        manifest_view.metadata["legal_hold_case_id"].as_str(),
        Some("legal-hold-1")
    );
    assert_eq!(
        manifest_view.metadata["replay_job_id"].as_str(),
        Some("replay-1")
    );
    assert_eq!(
        manifest_view.metadata["export_job_id"].as_str(),
        Some("export-1")
    );
    assert_eq!(
        manifest_item_view.evidence_item_id.as_deref(),
        Some("item-bridge-1")
    );
    assert_eq!(manifest_item_view.item_digest, "object-hash-1");
}

#[test]
fn audit_permission_matrix_matches_core_roles_and_distinct_points() {
    assert_eq!(
        AuditPermission::DeveloperTraceRead.permission_code(),
        "developer.trace.read"
    );
    assert_eq!(
        AuditPermission::OpsConsistencyReconcile.permission_code(),
        "ops.consistency.reconcile"
    );
    assert_eq!(
        AuditPermission::AnchorManage.permission_code(),
        "audit.anchor.manage"
    );
    assert!(!AuditPermission::DeveloperTraceRead.requires_step_up());
    assert!(AuditPermission::OpsConsistencyReconcile.requires_step_up());
    assert!(AuditPermission::AnchorManage.requires_step_up());

    assert!(is_allowed(
        "tenant_developer",
        AuditPermission::DeveloperTraceRead
    ));
    assert!(is_allowed(
        "platform_audit_security",
        AuditPermission::DeveloperTraceRead
    ));
    assert!(!is_allowed(
        "platform_admin",
        AuditPermission::DeveloperTraceRead
    ));
    assert!(is_allowed(
        "platform_admin",
        AuditPermission::OpsConsistencyReconcile
    ));
    assert!(is_allowed(
        "platform_audit_security",
        AuditPermission::OpsConsistencyReconcile
    ));
    assert!(!is_allowed(
        "tenant_admin",
        AuditPermission::OpsConsistencyReconcile
    ));
    assert!(is_allowed("platform_admin", AuditPermission::OpsOutboxRead));
    assert!(is_allowed(
        "platform_audit_security",
        AuditPermission::OpsDeadLetterReprocess
    ));
    assert!(is_allowed("platform_admin", AuditPermission::AnchorManage));
    assert!(!is_allowed(
        "platform_admin",
        AuditPermission::PackageExport
    ));
    assert!(is_allowed(
        "platform_audit_security",
        AuditPermission::PackageExport
    ));
}

#[test]
fn legacy_audit_role_aliases_only_survive_as_compatibility_mapping() {
    assert_eq!(canonical_role_key("developer_admin"), "tenant_developer");
    assert_eq!(canonical_role_key("audit_admin"), "platform_audit_security");
    assert_eq!(
        canonical_role_key("consistency_operator"),
        "platform_audit_security"
    );
    assert_eq!(
        canonical_role_key("platform_auditor"),
        "platform_audit_security"
    );
    assert_eq!(
        canonical_role_key("node_ops_admin"),
        "platform_audit_security"
    );
    assert_eq!(canonical_role_key("subject_reviewer"), "platform_reviewer");
    assert_eq!(
        canonical_role_key("risk_operator"),
        "platform_risk_settlement"
    );
    assert_eq!(
        canonical_role_key("regulator_observer"),
        "regulator_readonly"
    );

    assert!(is_allowed(
        "developer_admin",
        AuditPermission::DeveloperTraceRead
    ));
    assert!(is_allowed("audit_admin", AuditPermission::PackageExport));
    assert!(is_allowed(
        "consistency_operator",
        AuditPermission::OpsConsistencyRead
    ));
    assert!(is_allowed("node_ops_admin", AuditPermission::OpsOutboxRead));
    assert!(!is_allowed(
        "developer_admin",
        AuditPermission::PackageExport
    ));
}
