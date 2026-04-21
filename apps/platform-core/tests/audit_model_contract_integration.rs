use audit_kit::{AuditContext, AuditEvent};
use platform_core::modules::audit::dto::AuditTraceView;
use platform_core::modules::audit::repo::{AuditEventInsert, INSERT_AUDIT_EVENT_SQL};

#[test]
fn unified_audit_model_crosses_kit_and_platform_core_boundaries() {
    let event = AuditEvent::business(
        "audit",
        "replay_job",
        Some("replay-1".to_string()),
        "audit.replay.request",
        "accepted",
        AuditContext::minimal("req-it-1", "trace-it-1", "user-it-1", "tenant-it-1"),
    );

    let trace = AuditTraceView::from(&event);
    let insert = AuditEventInsert::from(&event);

    assert_eq!(trace.domain_name, "audit");
    assert_eq!(trace.ref_type, "replay_job");
    assert_eq!(trace.request_id.as_deref(), Some("req-it-1"));
    assert_eq!(insert.request_id.as_deref(), Some("req-it-1"));
    assert_eq!(
        insert.metadata["tenant_id"].as_str(),
        Some("tenant-it-1"),
        "tenant_id must remain available to writer/export paths"
    );
    assert!(
        INSERT_AUDIT_EVENT_SQL.contains("evidence_manifest_id"),
        "writer foundation must already expose the hardened audit schema surface"
    );
}
