import { describe, expect, it, vi } from "vitest";

import {
  buildReviewDecisionPayload,
  canReadReviewWorkbench,
  canWriteReviewDecision,
  countStandardSkuCoverage,
  createReviewIdempotencyKey,
  deriveComplianceSignals,
  reviewDecisionFormSchema,
} from "./review-workbench";

describe("review workbench view model", () => {
  it("uses formal platform reviewer roles and rejects tenant-only operators", () => {
    expect(
      canReadReviewWorkbench({
        mode: "local_test_user",
        roles: ["platform_reviewer"],
        auth_context_level: "aal1",
      }),
    ).toBe(true);
    expect(
      canWriteReviewDecision({
        mode: "local_test_user",
        roles: ["platform_admin"],
        auth_context_level: "aal1",
      }),
    ).toBe(true);
    expect(
      canWriteReviewDecision({
        mode: "local_test_user",
        roles: ["tenant_admin"],
        auth_context_level: "aal1",
      }),
    ).toBe(false);
  });

  it("maps high-risk compliance block to the formal reject API decision", () => {
    const parsed = reviewDecisionFormSchema.parse({
      action: "block",
      action_reason: "跨境导出限制和 L3 风险命中",
      idempotency_key: "web-008:block:demo",
      block_confirmation: "BLOCK",
    });

    expect(buildReviewDecisionPayload(parsed)).toEqual({
      action_name: "reject",
      action_reason: "合规阻断确认：跨境导出限制和 L3 风险命中",
    });
  });

  it("requires explicit manual confirmation for compliance block", () => {
    const parsed = reviewDecisionFormSchema.safeParse({
      action: "block",
      action_reason: "跨境导出限制和 L3 风险命中",
      idempotency_key: "web-008:block:demo",
      block_confirmation: "NO",
    });

    expect(parsed.success).toBe(false);
  });

  it("derives compliance and SKU signals without collapsing standard SKU types", () => {
    const product = {
      product_id: "20000000-0000-0000-0000-000000000801",
      asset_id: "20000000-0000-0000-0000-000000000802",
      asset_version_id: "20000000-0000-0000-0000-000000000803",
      seller_org_id: "10000000-0000-0000-0000-000000000101",
      title: "WEB-008 合规样例",
      category: "industrial_data",
      product_type: "data_product",
      status: "pending_review",
      price_mode: "one_time",
      price: "1000",
      currency_code: "CNY",
      delivery_type: "file_download",
      allowed_usage: ["internal_analysis"],
      searchable_text: null,
      subtitle: null,
      industry: "manufacturing",
      use_cases: ["risk_control"],
      data_classification: "L3",
      quality_score: "A",
      metadata: {
        export_restriction: "cross_border_blocked",
        risk_tags: ["geo_export", "purpose_sensitive"],
      },
      search_document_version: 1,
      index_sync_status: "pending",
      skus: [
        {
          sku_id: "20000000-0000-0000-0000-000000000804",
          product_id: "20000000-0000-0000-0000-000000000801",
          sku_code: "share-ro",
          sku_type: "SHARE_RO",
          trade_mode: "share_readonly",
          billing_mode: "subscription",
          delivery_mode: "share",
          acceptance_mode: "manual",
          refund_mode: "none",
          price: "1000",
          currency_code: "CNY",
          status: "active",
          created_at: "2026-04-23T00:00:00Z",
          updated_at: "2026-04-23T00:00:00Z",
        },
      ],
      created_at: "2026-04-23T00:00:00Z",
      updated_at: "2026-04-23T00:00:00Z",
    };

    const signals = deriveComplianceSignals(product);
    expect(signals.automaticBlockResult).toBe("manual_review");
    expect(signals.highRiskSignals).toContain("L3 分类分级需人工复核");
    expect(countStandardSkuCoverage(product)).toContainEqual({
      skuType: "SHARE_RO",
      present: true,
    });
    expect(countStandardSkuCoverage(product)).toContainEqual({
      skuType: "QRY_LITE",
      present: false,
    });
  });

  it("creates task-scoped idempotency keys", () => {
    vi.spyOn(globalThis.crypto, "randomUUID").mockReturnValue(
      "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
    );

    expect(createReviewIdempotencyKey("products")).toBe(
      "web-008:products:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
    );
  });
});
