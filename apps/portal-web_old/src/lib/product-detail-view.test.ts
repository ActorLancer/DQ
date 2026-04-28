import { describe, expect, it } from "vitest";

import {
  getOrderGate,
  hasSampleEvidence,
  metadataDisplayEntries,
  type ProductDetail,
} from "./product-detail-view";

const baseProduct: ProductDetail = {
  product_id: "20000000-0000-0000-0000-000000000309",
  asset_id: "20000000-0000-0000-0000-000000000301",
  asset_version_id: "20000000-0000-0000-0000-000000000302",
  seller_org_id: "10000000-0000-0000-0000-000000000101",
  title: "工业设备运行指标 API 订阅",
  category: "industry_iot",
  product_type: "service_product",
  status: "listed",
  description: "demo",
  price_mode: "fixed",
  price: "1999.00000000",
  currency_code: "CNY",
  delivery_type: "api_subscription",
  allowed_usage: ["internal_analysis"],
  searchable_text: null,
  subtitle: null,
  industry: "industrial_manufacturing",
  use_cases: ["设备稼动率"],
  data_classification: "P1",
  quality_score: "0.93",
  metadata: {
    sample_summary: "字段级样本已脱敏",
    object_uri: "s3://private/raw.csv",
  },
  search_document_version: 1,
  index_sync_status: "synced",
  skus: [],
  created_at: "2026-01-01T00:00:00.000Z",
  updated_at: "2026-01-01T00:00:00.000Z",
};

describe("product detail view helpers", () => {
  it("allows order entry only when listed product carries sample evidence", () => {
    expect(getOrderGate(baseProduct)).toMatchObject({ enabled: true });
    expect(
      getOrderGate({
        ...baseProduct,
        metadata: {},
      }),
    ).toMatchObject({
      enabled: false,
      reason: expect.stringContaining("样本"),
    });
  });

  it("blocks terminal or non-sale statuses", () => {
    expect(getOrderGate({ ...baseProduct, status: "retired" })).toMatchObject({
      enabled: false,
      reason: expect.stringContaining("retired"),
    });
    expect(getOrderGate({ ...baseProduct, status: "pending_review" })).toMatchObject({
      enabled: false,
      reason: expect.stringContaining("pending_review"),
    });
  });

  it("does not expose object paths in display metadata", () => {
    expect(hasSampleEvidence(baseProduct)).toBe(true);
    expect(metadataDisplayEntries(baseProduct.metadata)).toContainEqual({
      key: "object_uri",
      value: "已隐藏受控对象路径",
      hidden: true,
    });
  });
});
