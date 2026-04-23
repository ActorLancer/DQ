import { describe, expect, it } from "vitest";

import {
  certificationLabel,
  formatReputationScore,
  formatSellerPrice,
  mergeSellerMarketplaceItems,
  sellerRiskDescriptor,
  type SellerProfile,
  type SellerRecommendationItem,
} from "./seller-profile-view";

const seller: SellerProfile = {
  org_id: "10000000-0000-0000-0000-000000000101",
  org_name: "Luna Seller Org",
  org_type: "tenant_seller",
  status: "active",
  country_code: "CN",
  region_code: "SH",
  industry_tags: ["manufacturing"],
  certification_tags: ["real_name_verified", "compliance:l2"],
  featured_products: [
    {
      product_id: "20000000-0000-0000-0000-000000000309",
      title: "工业设备运行指标 API 订阅",
      subtitle: "首页固定推荐样例 S1",
      category: "manufacturing",
      price_amount: "5999.00000000",
      currency_code: "CNY",
    },
  ],
  rating_summary: {
    rating_count: 12,
    average_rating: 4.7,
    reputation_score: 0.88,
    credit_level: 75,
    risk_level: 1,
  },
  credit_level: 75,
  risk_level: 1,
  reputation_score: "0.88",
  listed_product_count: 11,
  description: "seller",
  search_document_version: 7,
  index_sync_status: "synced",
};

const recommendation: SellerRecommendationItem = {
  recommendation_result_item_id: "30000000-0000-0000-0000-000000000001",
  entity_scope: "product",
  entity_id: "20000000-0000-0000-0000-000000000309",
  title: "重复商品",
  seller_name: "Luna Seller Org",
  price: "5999.00000000",
  currency_code: "CNY",
  final_score: 0.92,
  explanation_codes: ["local:same_seller"],
  recall_sources: ["seller_related"],
  status: "listed",
};

describe("seller profile view helpers", () => {
  it("maps official certification tags without inventing role names", () => {
    expect(certificationLabel("real_name_verified")).toBe("实名认证");
    expect(certificationLabel("compliance:l2")).toBe("合规 L2");
    expect(certificationLabel("custom_tag")).toBe("custom_tag");
  });

  it("formats seller risk and reputation summaries", () => {
    expect(sellerRiskDescriptor(1)).toEqual({
      label: "低风险 L1",
      tone: "success",
    });
    expect(sellerRiskDescriptor(3)).toEqual({
      label: "高风险 L3",
      tone: "danger",
    });
    expect(formatReputationScore("0.88")).toBe("88 / 100");
  });

  it("formats prices from seller projection payloads", () => {
    expect(formatSellerPrice("5999.00000000", "CNY")).toBe("CNY 5,999");
    expect(formatSellerPrice(null, "CNY")).toBe("价格咨询");
  });

  it("merges seller projection and recommendation items without duplicates", () => {
    const merged = mergeSellerMarketplaceItems(seller, [recommendation]);

    expect(merged).toHaveLength(1);
    expect(merged[0]).toMatchObject({
      id: "20000000-0000-0000-0000-000000000309",
      source: "seller_search_document",
      title: "工业设备运行指标 API 订阅",
    });
  });
});
