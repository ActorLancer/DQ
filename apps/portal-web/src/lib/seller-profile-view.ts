import type {
  RecommendationsResponse,
  SellerProfileResponse,
} from "@datab/sdk-ts";

export type SellerProfile = SellerProfileResponse["data"];
export type SellerFeaturedProduct = SellerProfile["featured_products"][number];
export type SellerRecommendationItem =
  RecommendationsResponse["data"]["items"][number];

export type SellerMarketplaceItem = {
  id: string;
  title: string;
  subtitle?: string | null;
  href: string;
  kind: "product" | "seller";
  source: "seller_search_document" | "seller_profile_featured";
  category?: string | null;
  priceLabel: string;
  status?: string;
  score?: number;
};

const CERTIFICATION_LABELS: Record<string, string> = {
  "compliance:l2": "合规 L2",
  "certification:enhanced": "增强认证",
  iso27001: "ISO 27001",
  real_name_verified: "实名认证",
  trusted_partner: "可信伙伴",
};

export function certificationLabel(tag: string): string {
  return CERTIFICATION_LABELS[tag] ?? tag;
}

export function sellerRiskDescriptor(riskLevel: number): {
  label: string;
  tone: "success" | "warning" | "danger";
} {
  if (riskLevel <= 1) {
    return { label: `低风险 L${riskLevel}`, tone: "success" };
  }
  if (riskLevel === 2) {
    return { label: "中风险 L2", tone: "warning" };
  }
  return { label: `高风险 L${riskLevel}`, tone: "danger" };
}

export function formatReputationScore(value: string | number | null | undefined): string {
  if (value === null || value === undefined || value === "") {
    return "未返回";
  }
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) {
    return String(value);
  }
  if (numeric >= 0 && numeric <= 1) {
    return `${Math.round(numeric * 100)} / 100`;
  }
  return `${numeric.toFixed(2)} / 100`;
}

export function formatSellerPrice(
  amount: string | number | null | undefined,
  currencyCode: string | null | undefined,
): string {
  if (amount === null || amount === undefined || amount === "") {
    return "价格咨询";
  }
  const numeric = Number(amount);
  const value = Number.isFinite(numeric)
    ? numeric.toLocaleString("zh-CN", {
        maximumFractionDigits: 2,
        minimumFractionDigits: 0,
      })
    : String(amount);
  return `${currencyCode || "CNY"} ${value}`;
}

export function sellerRatingMetric(
  seller: SellerProfile,
  key: keyof SellerProfile["rating_summary"],
): string {
  const value = seller.rating_summary?.[key];
  if (value === null || value === undefined || value === "") {
    return "未返回";
  }
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(2);
  }
  return String(value);
}

export function sellerFeaturedMarketplaceItems(
  seller: SellerProfile,
): SellerMarketplaceItem[] {
  return (seller.featured_products ?? []).map((product) => ({
    id: product.product_id,
    title: product.title,
    subtitle: product.subtitle,
    href: `/products/${product.product_id}`,
    kind: "product",
    source: "seller_search_document",
    category: product.category,
    priceLabel: formatSellerPrice(product.price_amount, product.currency_code),
  }));
}

export function recommendationMarketplaceItems(
  items: SellerRecommendationItem[],
): SellerMarketplaceItem[] {
  return items.map((item) => ({
    id: item.entity_id,
    title: item.title,
    subtitle: item.seller_name,
    href:
      item.entity_scope === "seller"
        ? `/sellers/${item.entity_id}`
        : `/products/${item.entity_id}`,
    kind: item.entity_scope,
    source: "seller_profile_featured",
    priceLabel: formatSellerPrice(item.price, item.currency_code),
    status: item.status,
    score: item.final_score,
  }));
}

export function mergeSellerMarketplaceItems(
  seller: SellerProfile,
  recommendations: SellerRecommendationItem[],
): SellerMarketplaceItem[] {
  const merged = new Map<string, SellerMarketplaceItem>();
  for (const item of sellerFeaturedMarketplaceItems(seller)) {
    merged.set(`${item.kind}:${item.id}`, item);
  }
  for (const item of recommendationMarketplaceItems(recommendations)) {
    if (!merged.has(`${item.kind}:${item.id}`)) {
      merged.set(`${item.kind}:${item.id}`, item);
    }
  }
  return Array.from(merged.values());
}
