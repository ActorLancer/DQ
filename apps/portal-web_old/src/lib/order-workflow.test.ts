import { describe, expect, it, vi } from "vitest";

import {
  ORDER_SCENARIO_BLUEPRINTS,
  buildCreateOrderRequest,
  canCreateOrder,
  collectOrderSkuCoverage,
  createOrderIdempotencyKey,
  deliveryRouteForSku,
  findTemplatesForSku,
  requiresScenarioCode,
  resolveTemplateForSku,
  scenarioRole,
  unwrapCreatedOrder,
  type SessionSubject,
} from "./order-workflow";

describe("order workflow helpers", () => {
  it("keeps five official order scenarios covering all eight V1 SKUs", () => {
    expect(ORDER_SCENARIO_BLUEPRINTS.map((item) => item.scenario_code)).toEqual([
      "S1",
      "S2",
      "S3",
      "S4",
      "S5",
    ]);
    expect(collectOrderSkuCoverage(ORDER_SCENARIO_BLUEPRINTS)).toEqual([
      "FILE_STD",
      "FILE_SUB",
      "SHARE_RO",
      "API_SUB",
      "API_PPU",
      "QRY_LITE",
      "SBX_STD",
      "RPT_STD",
    ]);
  });

  it("requires explicit scenario_code for ambiguous SKU families", () => {
    expect(requiresScenarioCode(ORDER_SCENARIO_BLUEPRINTS, "API_SUB")).toBe(true);
    expect(findTemplatesForSku(ORDER_SCENARIO_BLUEPRINTS, "API_SUB").map((item) => item.scenario_code))
      .toEqual(["S1", "S4"]);
    expect(resolveTemplateForSku(ORDER_SCENARIO_BLUEPRINTS, "API_SUB")).toBeNull();
    expect(resolveTemplateForSku(ORDER_SCENARIO_BLUEPRINTS, "API_SUB", "S4")?.scenario_name)
      .toBe("零售门店经营分析 API / 报告订阅");
  });

  it("does not collapse supplementary SKUs into broad categories", () => {
    const share = resolveTemplateForSku(ORDER_SCENARIO_BLUEPRINTS, "SHARE_RO", "S3");
    const qry = resolveTemplateForSku(ORDER_SCENARIO_BLUEPRINTS, "QRY_LITE");
    const report = resolveTemplateForSku(ORDER_SCENARIO_BLUEPRINTS, "RPT_STD", "S5");

    expect(share?.primary_sku).toBe("SBX_STD");
    expect(scenarioRole(share!, "SHARE_RO")).toBe("supplementary");
    expect(qry?.primary_sku).toBe("QRY_LITE");
    expect(scenarioRole(qry!, "QRY_LITE")).toBe("primary");
    expect(report?.supplementary_skus).toContain("RPT_STD");
  });

  it("builds create order payload without UI-only form fields", () => {
    expect(
      buildCreateOrderRequest({
        buyer_org_id: "10000000-0000-0000-0000-000000000201",
        product_id: "20000000-0000-0000-0000-000000000901",
        sku_id: "30000000-0000-0000-0000-000000000902",
        scenario_code: "S2",
        quantity: 3,
        term_days: 30,
        subscription_cadence: "monthly",
        confirm_rights: true,
        confirm_snapshot: true,
        confirm_audit: true,
        idempotency_key: "web-009-order-create-test",
      }),
    ).toEqual({
      buyer_org_id: "10000000-0000-0000-0000-000000000201",
      product_id: "20000000-0000-0000-0000-000000000901",
      sku_id: "30000000-0000-0000-0000-000000000902",
      scenario_code: "S2",
    });
  });

  it("unwraps standard trade create envelope shaped by platform-core", () => {
    expect(
      unwrapCreatedOrder({
        code: "OK",
        message: "success",
        request_id: "req-order-create-test",
        data: {
          order_id: "30000000-0000-0000-0000-000000000901",
          buyer_org_id: "10000000-0000-0000-0000-000000000201",
          seller_org_id: "10000000-0000-0000-0000-000000000202",
          product_id: "20000000-0000-0000-0000-000000000901",
          sku_id: "30000000-0000-0000-0000-000000000902",
          current_state: "created",
          payment_status: "unpaid",
          order_amount: "88.80",
          currency_code: "CNY",
          price_snapshot: {
            product_id: "20000000-0000-0000-0000-000000000901",
            sku_id: "30000000-0000-0000-0000-000000000902",
            sku_code: "FILE_STD-BASIC",
            sku_type: "FILE_STD",
            pricing_mode: "one_time",
            unit_price: "88.80",
            currency_code: "CNY",
            billing_mode: "one_time",
            refund_mode: "manual_refund",
            settlement_terms: {
              settlement_basis: "acceptance",
              settlement_mode: "platform_escrow",
            },
            tax_terms: {
              tax_policy: "platform_default",
              tax_code: "VAT",
              tax_inclusive: false,
            },
            captured_at: "2026-04-23T00:00:00Z",
            source: "catalog.product_sku",
          },
          created_at: "2026-04-23T00:00:00Z",
        },
      })?.price_snapshot.sku_type,
    ).toBe("FILE_STD");
  });

  it("uses official roles and delivery routes", () => {
    const subject = {
      roles: ["buyer_operator"],
      auth_context_level: "aal1",
      mode: "local_test_user",
    } as SessionSubject;

    expect(canCreateOrder(subject)).toBe(true);
    expect(deliveryRouteForSku("QRY_LITE", "order-1")).toBe(
      "/delivery/orders/order-1/template-query",
    );
  });

  it("generates task-scoped idempotency keys", () => {
    vi.spyOn(crypto, "randomUUID").mockReturnValue("00000000-0000-0000-0000-000000000009");
    expect(createOrderIdempotencyKey("create")).toBe(
      "web-009-order-create-00000000-0000-0000-0000-000000000009",
    );
  });
});
