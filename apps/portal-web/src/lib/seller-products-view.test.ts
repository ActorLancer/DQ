import { describe, expect, it } from "vitest";

import {
  buildCreateProductRequest,
  buildSkuRequest,
  canSubmitProduct,
  defaultCreateProductValues,
  defaultSkuValues,
  standardSkuOptions,
  type SellerProductDetail,
} from "./seller-products-view";

describe("seller product workspace helpers", () => {
  it("keeps the V1 standard SKU truth set explicit", () => {
    expect(standardSkuOptions.map((option) => option.sku_type)).toEqual([
      "FILE_STD",
      "FILE_SUB",
      "SHARE_RO",
      "API_SUB",
      "API_PPU",
      "QRY_LITE",
      "SBX_STD",
      "RPT_STD",
    ]);
    expect(standardSkuOptions.find((option) => option.sku_type === "SHARE_RO")?.trade_mode)
      .toBe("share_grant");
    expect(standardSkuOptions.find((option) => option.sku_type === "QRY_LITE")?.trade_mode)
      .toBe("template_query");
  });

  it("builds create product payloads with metadata and comma-list fields", () => {
    const values = defaultCreateProductValues("10000000-0000-0000-0000-000000000101");
    values.asset_id = "20000000-0000-0000-0000-000000000101";
    values.asset_version_id = "20000000-0000-0000-0000-000000000102";
    values.title = "WEB-007 商品";
    values.allowed_usage = "internal_analysis, risk_control";
    values.allowed_region = "CN, SG";
    values.sample_hash = "sha256-sample";

    expect(buildCreateProductRequest(values)).toMatchObject({
      seller_org_id: "10000000-0000-0000-0000-000000000101",
      allowed_usage: ["internal_analysis", "risk_control"],
      metadata: {
        allowed_region: ["CN", "SG"],
        sample_hash: "sha256-sample",
        task_id: "WEB-007",
      },
    });
  });

  it("does not collapse separate SKU families into broad categories", () => {
    const share = buildSkuRequest(defaultSkuValues("SHARE_RO"));
    const sandbox = buildSkuRequest(defaultSkuValues("SBX_STD"));

    expect(share).toMatchObject({
      sku_type: "SHARE_RO",
      trade_mode: "share_grant",
      delivery_object_kind: "share_grant",
    });
    expect(sandbox).toMatchObject({
      sku_type: "SBX_STD",
      trade_mode: "sandbox_workspace",
      delivery_object_kind: "sandbox_workspace",
    });
  });

  it("blocks submit until a draft has at least one standard SKU", () => {
    const product = {
      status: "draft",
      skus: [],
    } as unknown as SellerProductDetail;
    expect(canSubmitProduct(product).allowed).toBe(false);

    product.skus = [
      {
        sku_id: "30000000-0000-0000-0000-000000000301",
        product_id: "20000000-0000-0000-0000-000000000301",
        sku_code: "FILE_STD-BASIC",
        sku_type: "FILE_STD",
        unit_name: "份",
        billing_mode: "one_time",
        trade_mode: "snapshot_sale",
        acceptance_mode: "manual_accept",
        refund_mode: "manual_refund",
        status: "draft",
        created_at: "2026-04-23T00:00:00Z",
        updated_at: "2026-04-23T00:00:00Z",
      },
    ];

    expect(canSubmitProduct(product).allowed).toBe(true);
  });
});
