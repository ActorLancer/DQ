import { describe, expect, it, vi } from "vitest";

import {
  buildAcceptOrderRequest,
  buildRejectOrderRequest,
  canOperateAcceptance,
  canReadAcceptance,
  createAcceptanceIdempotencyKey,
  defaultAcceptOrderValues,
  defaultRejectOrderValues,
  isDeliveredForAcceptance,
  isManualAcceptanceSku,
} from "./acceptance-workflow";

describe("acceptance workflow helpers", () => {
  it("builds accept payload from checked verification summary", () => {
    const values = defaultAcceptOrderValues();
    values.note = "hash verified";
    values.verification_summary_json = JSON.stringify({
      hash_match: true,
      contract_template_match: true,
    });

    expect(buildAcceptOrderRequest(values)).toEqual({
      note: "hash verified",
      verification_summary: {
        hash_match: true,
        contract_template_match: true,
      },
    });
  });

  it("requires reject reason and keeps reason code explicit", () => {
    const values = defaultRejectOrderValues();
    values.reason_code = "report_quality_failed";
    values.reason_detail = "sample section mismatched template";
    values.verification_summary_json = JSON.stringify({
      hash_match: true,
      report_section_check: false,
    });

    expect(buildRejectOrderRequest(values)).toEqual({
      reason_code: "report_quality_failed",
      reason_detail: "sample section mismatched template",
      verification_summary: {
        hash_match: true,
        report_section_check: false,
      },
    });
  });

  it("uses formal manual acceptance SKU set", () => {
    expect(isManualAcceptanceSku("FILE_STD")).toBe(true);
    expect(isManualAcceptanceSku("FILE_SUB")).toBe(true);
    expect(isManualAcceptanceSku("RPT_STD")).toBe(true);
    expect(isManualAcceptanceSku("SHARE_RO")).toBe(false);
    expect(isManualAcceptanceSku("QRY_LITE")).toBe(false);
  });

  it("matches backend manual acceptance ready states", () => {
    expect(
      isDeliveredForAcceptance({
        current_state: "delivered",
        price_snapshot: { sku_type: "FILE_STD" },
      } as never),
    ).toBe(true);
    expect(
      isDeliveredForAcceptance({
        current_state: "report_delivered",
        price_snapshot: { sku_type: "RPT_STD" },
      } as never),
    ).toBe(true);
    expect(
      isDeliveredForAcceptance({
        current_state: "delivered",
        price_snapshot: { sku_type: "RPT_STD" },
      } as never),
    ).toBe(false);
  });

  it("checks read and action roles against formal V1 roles", () => {
    expect(canReadAcceptance({ roles: ["tenant_audit_readonly"] } as never)).toBe(true);
    expect(canOperateAcceptance({ roles: ["buyer_operator"] } as never)).toBe(true);
    expect(canOperateAcceptance({ roles: ["seller_operator"] } as never)).toBe(false);
  });

  it("creates WEB-011 idempotency keys", () => {
    vi.stubGlobal("crypto", {
      randomUUID: () => "00000000-0000-4000-8000-000000000011",
    });

    expect(createAcceptanceIdempotencyKey("accept")).toBe(
      "web-011-acceptance-accept-00000000-0000-4000-8000-000000000011",
    );

    vi.unstubAllGlobals();
  });
});
