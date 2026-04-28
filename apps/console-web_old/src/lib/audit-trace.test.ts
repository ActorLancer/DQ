import { describe, expect, it, vi } from "vitest";

import {
  auditLookupFormSchema,
  auditPackageExportFormSchema,
  buildAuditTraceQuery,
  buildDeveloperTraceQuery,
  buildPackageExportPayload,
  canExportAuditPackage,
  canReadAuditTrace,
  canReadDeveloperTrace,
  classifyAuditEvent,
  createAuditIdempotencyKey,
  normalizeAuditEvents,
  safePackageExportView,
} from "./audit-trace";

describe("audit trace view model", () => {
  it("uses formal V1 audit roles and does not accept seller operators", () => {
    expect(
      canReadAuditTrace({
        mode: "local_test_user",
        roles: ["tenant_audit_readonly"],
        auth_context_level: "aal1",
      }),
    ).toBe(true);
    expect(
      canExportAuditPackage({
        mode: "local_test_user",
        roles: ["platform_audit_security"],
        auth_context_level: "aal2",
      }),
    ).toBe(true);
    expect(
      canReadAuditTrace({
        mode: "local_test_user",
        roles: ["seller_operator"],
        auth_context_level: "aal1",
      }),
    ).toBe(false);
  });

  it("maps each lookup key to formal audit or developer trace filters", () => {
    const orderLookup = auditLookupFormSchema.parse({
      lookup_key: "order_id",
      lookup_value: "30000000-0000-4000-8000-000000014001",
      page_size: 20,
    });
    expect(buildAuditTraceQuery(orderLookup)).toEqual({
      order_id: "30000000-0000-4000-8000-000000014001",
      page: 1,
      page_size: 20,
    });
    expect(buildDeveloperTraceQuery(orderLookup)).toEqual({
      order_id: "30000000-0000-4000-8000-000000014001",
    });

    const caseLookup = auditLookupFormSchema.parse({
      lookup_key: "case_id",
      lookup_value: "40000000-0000-4000-8000-000000014001",
      page_size: 50,
    });
    expect(buildAuditTraceQuery(caseLookup)).toMatchObject({
      ref_type: "dispute_case",
      ref_id: "40000000-0000-4000-8000-000000014001",
    });

    const txLookup = auditLookupFormSchema.parse({
      lookup_key: "tx_hash",
      lookup_value: "0xabc014",
      page_size: 50,
    });
    expect(buildAuditTraceQuery(txLookup)).toBeNull();
    expect(buildDeveloperTraceQuery(txLookup)).toEqual({ tx_hash: "0xabc014" });
    expect(canReadDeveloperTrace({ roles: ["platform_audit_security"], mode: "local_test_user", auth_context_level: "aal1" })).toBe(true);
  });

  it("requires step-up token or challenge for evidence package export", () => {
    const denied = auditPackageExportFormSchema.safeParse({
      ref_type: "order",
      ref_id: "30000000-0000-4000-8000-000000014001",
      reason: "监管抽查导出正式证据包",
      masked_level: "masked",
      package_type: "forensic_export",
      idempotency_key: "web-014:audit-export:demo",
    });
    expect(denied.success).toBe(false);

    const parsed = auditPackageExportFormSchema.parse({
      ref_type: "order",
      ref_id: "30000000-0000-4000-8000-000000014001",
      reason: "监管抽查导出正式证据包",
      masked_level: "masked",
      package_type: "forensic_export",
      idempotency_key: "web-014:audit-export:demo",
      step_up_token: "step-up-token",
    });
    expect(buildPackageExportPayload(parsed)).toEqual({
      ref_type: "order",
      ref_id: "30000000-0000-4000-8000-000000014001",
      reason: "监管抽查导出正式证据包",
      masked_level: "masked",
      package_type: "forensic_export",
    });
  });

  it("normalizes audit rows and keeps formal event grouping", () => {
    const rows = normalizeAuditEvents([
      {
        audit_id: "audit-1",
        event_schema_version: "v1",
        domain_name: "billing",
        event_class: "domain",
        ref_type: "order",
        ref_id: "order-1",
        action_name: "billing.refund.execute",
        result_code: "ok",
        occurred_at: "2026-04-23T00:00:00.000Z",
      },
      {
        audit_id: "audit-1",
        event_schema_version: "v1",
        domain_name: "billing",
        event_class: "domain",
        ref_type: "order",
        ref_id: "order-1",
        action_name: "billing.refund.execute",
        result_code: "ok",
        occurred_at: "2026-04-23T00:00:00.000Z",
      },
    ]);

    expect(rows).toHaveLength(1);
    expect(rows[0]?.group).toBe("billing");
    expect(
      classifyAuditEvent({
        domain_name: "delivery",
        action_name: "delivery.receipt.record",
        ref_type: "delivery_record",
      }),
    ).toBe("delivery");
  });

  it("hides storage uri fields from package export results", () => {
    const safe = safePackageExportView({
      code: "OK",
      message: "success",
      request_id: "req-web-014",
      data: {
        audit_trace_count: 2,
        evidence_item_count: 3,
        legal_hold_status: "none",
        step_up_bound: true,
        evidence_package: {
          evidence_package_id: "pkg-1",
          package_type: "forensic_export",
          ref_type: "order",
          ref_id: "order-1",
          package_digest: "sha256:pkg",
          retention_class: "regulatory",
          legal_hold_status: "none",
          storage_uri: "s3://audit-bucket/private/pkg.json",
        },
        evidence_manifest: {
          evidence_manifest_id: "manifest-1",
          manifest_scope: "order",
          ref_type: "order",
          ref_id: "order-1",
          manifest_hash: "sha256:manifest",
          item_count: 3,
          storage_uri: "s3://audit-bucket/private/manifest.json",
        },
      },
    });

    expect(JSON.stringify(safe)).not.toContain("s3://");
    expect(safe?.hidden_fields).toContain("evidence_package.storage_uri");
  });

  it("creates task-scoped idempotency keys", () => {
    vi.spyOn(globalThis.crypto, "randomUUID").mockReturnValue(
      "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
    );

    expect(createAuditIdempotencyKey()).toBe(
      "web-014:audit-export:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
    );
  });
});
