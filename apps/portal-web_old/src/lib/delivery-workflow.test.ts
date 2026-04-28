import { describe, expect, it, vi } from "vitest";

import {
  DELIVERY_ENTRIES,
  buildCommitDeliveryRequest,
  buildRevisionSubscriptionRequest,
  buildSandboxWorkspaceRequest,
  buildShareGrantRequest,
  buildTemplateGrantRequest,
  buildTemplateRunRequest,
  canExecuteTemplateQueryRun,
  canManageTemplateQueryGrant,
  canOperateDelivery,
  canReadDelivery,
  createDeliveryIdempotencyKey,
  defaultCommitDeliveryValues,
  defaultRevisionSubscriptionValues,
  defaultSandboxWorkspaceValues,
  defaultShareGrantValues,
  defaultTemplateGrantValues,
  defaultTemplateRunValues,
  deliveryEntryForSku,
  templateGrantFormSchema,
  templateRunFormSchema,
  type SessionSubject,
} from "./delivery-workflow";

describe("delivery workflow helpers", () => {
  it("maps all eight V1 SKUs to official delivery entries without collapsing them", () => {
    expect(DELIVERY_ENTRIES.flatMap((entry) => entry.supportedSkus)).toEqual([
      "FILE_STD",
      "FILE_SUB",
      "SHARE_RO",
      "API_SUB",
      "API_PPU",
      "QRY_LITE",
      "SBX_STD",
      "RPT_STD",
    ]);
    expect(deliveryEntryForSku("SHARE_RO")?.kind).toBe("share");
    expect(deliveryEntryForSku("QRY_LITE")?.kind).toBe("template-query");
    expect(deliveryEntryForSku("SBX_STD")?.kind).toBe("sandbox");
    expect(deliveryEntryForSku("RPT_STD")?.kind).toBe("report");
  });

  it("builds branch payloads with only official delivery branch names", () => {
    const file = buildCommitDeliveryRequest("file", {
      ...defaultCommitDeliveryValues("file"),
      confirm_scope: true,
      confirm_audit: true,
    });
    const api = buildCommitDeliveryRequest("api", {
      ...defaultCommitDeliveryValues("api"),
      confirm_scope: true,
      confirm_audit: true,
    });
    const report = buildCommitDeliveryRequest("report", {
      ...defaultCommitDeliveryValues("report"),
      confirm_scope: true,
      confirm_audit: true,
    });

    expect(file.branch).toBe("file");
    expect(api.branch).toBe("api");
    expect(report.branch).toBe("report");
    expect(api.quota_json).toEqual({ monthly_calls: 10000 });
    expect(file.metadata).toMatchObject({ web_task_id: "WEB-010" });
  });

  it("builds idempotent enablement payloads for subscription/share/template/sandbox", () => {
    expect(
      buildRevisionSubscriptionRequest({
        ...defaultRevisionSubscriptionValues(),
        confirm_scope: true,
        confirm_audit: true,
      }),
    ).toMatchObject({
      cadence: "monthly",
      delivery_channel: "file_ticket",
      metadata: { source: "portal-web", web_task_id: "WEB-010" },
    });
    expect(
      buildShareGrantRequest({
        ...defaultShareGrantValues(),
        confirm_scope: true,
        confirm_audit: true,
      }).scope_json,
    ).toEqual({ read_only: true, exportable: false });
    expect(
      buildTemplateGrantRequest({
        ...defaultTemplateGrantValues(),
        query_surface_id: "40000000-0000-0000-0000-000000000901",
        allowed_template_ids:
          "40000000-0000-0000-0000-000000000902, 40000000-0000-0000-0000-000000000903",
        confirm_scope: true,
        confirm_audit: true,
      }).allowed_template_ids,
    ).toEqual([
      "40000000-0000-0000-0000-000000000902",
      "40000000-0000-0000-0000-000000000903",
    ]);
    expect(
      buildSandboxWorkspaceRequest({
        ...defaultSandboxWorkspaceValues(),
        query_surface_id: "40000000-0000-0000-0000-000000000901",
        confirm_scope: true,
        confirm_audit: true,
      }).export_policy_json,
    ).toEqual({ exportable: false, review_required: true });
    expect(
      buildTemplateRunRequest({
        ...defaultTemplateRunValues(),
        template_query_grant_id: "40000000-0000-0000-0000-000000000901",
        query_template_id: "40000000-0000-0000-0000-000000000902",
        approval_ticket_id: "40000000-0000-0000-0000-000000000903",
        confirm_scope: true,
        confirm_audit: true,
      }),
    ).toMatchObject({
      template_query_grant_id: "40000000-0000-0000-0000-000000000901",
      query_template_id: "40000000-0000-0000-0000-000000000902",
      approval_ticket_id: "40000000-0000-0000-0000-000000000903",
      request_payload_json: { city: "Shanghai", radius_km: 3, limit: 2 },
      output_boundary_json: {
        selected_format: "json",
        allowed_formats: ["json"],
        max_rows: 2,
        max_cells: 6,
      },
      execution_metadata_json: { source: "portal-web", entrypoint: "template-query" },
    });
  });

  it("uses official V1 roles for read and action permission checks", () => {
    const seller = {
      roles: ["seller_operator"],
      auth_context_level: "aal1",
      mode: "local_test_user",
    } as SessionSubject;
    const developer = {
      roles: ["tenant_developer"],
      auth_context_level: "aal1",
      mode: "local_test_user",
    } as SessionSubject;

    expect(canReadDelivery(seller)).toBe(true);
    expect(canOperateDelivery(seller, "file")).toBe(true);
    expect(canOperateDelivery(seller, "template-query")).toBe(true);
    expect(canManageTemplateQueryGrant(seller)).toBe(true);
    expect(canExecuteTemplateQueryRun(seller)).toBe(false);
    expect(canOperateDelivery(developer, "sandbox")).toBe(true);
    expect(canExecuteTemplateQueryRun(developer)).toBe(true);
    expect(canManageTemplateQueryGrant(developer)).toBe(false);
    expect(canOperateDelivery(developer, "share")).toBe(false);
  });

  it("generates task-scoped idempotency keys", () => {
    vi.spyOn(crypto, "randomUUID").mockReturnValue("00000000-0000-0000-0000-000000000010");
    expect(createDeliveryIdempotencyKey("file")).toBe(
      "web-010-delivery-file-00000000-0000-0000-0000-000000000010",
    );
  });

  it("accepts PostgreSQL UUID literals used by frozen QRY_LITE fixtures", () => {
    expect(() =>
      templateGrantFormSchema.parse({
        ...defaultTemplateGrantValues(),
        query_surface_id: "62000000-0000-0000-0000-000000000313",
        allowed_template_ids: "63000000-0000-0000-0000-000000000313",
        confirm_scope: true,
        confirm_audit: true,
      }),
    ).not.toThrow();
    expect(() =>
      templateRunFormSchema.parse({
        ...defaultTemplateRunValues(),
        query_template_id: "63000000-0000-0000-0000-000000000313",
        confirm_scope: true,
        confirm_audit: true,
      }),
    ).not.toThrow();
  });
});
