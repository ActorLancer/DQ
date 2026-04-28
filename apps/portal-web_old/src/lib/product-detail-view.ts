import type { ProductDetailResponse } from "@datab/sdk-ts";

export type ProductDetail = ProductDetailResponse["data"];

const HIDDEN_METADATA_KEY_PATTERN =
  /(object_uri|object_path|storage_uri|bucket|internal_path|download_url|raw_uri)/i;

export function hasSampleEvidence(product: ProductDetail): boolean {
  return Boolean(
    readMetadataText(product.metadata, "sample_summary") ||
      readMetadataText(product.metadata, "sample_hash") ||
      readMetadataText(product.metadata, "sample_sha256") ||
      readMetadataText(product.metadata, "full_hash"),
  );
}

export function getOrderGate(product: ProductDetail): {
  enabled: boolean;
  reason: string;
} {
  if (["suspended", "retired", "frozen", "delisted"].includes(product.status)) {
    return {
      enabled: false,
      reason: `商品状态为 ${product.status}，页面必须禁止下单。`,
    };
  }

  if (product.status !== "listed") {
    return {
      enabled: false,
      reason: `商品仍处于 ${product.status}，只有 listed 状态允许进入下单。`,
    };
  }

  if (!hasSampleEvidence(product)) {
    return {
      enabled: false,
      reason: "当前 API 未返回样本摘要或样本哈希，按页面说明禁止上架态下单展示。",
    };
  }

  return {
    enabled: true,
    reason: "商品 listed 且存在样本摘要/哈希，可进入下单。",
  };
}

export function readMetadataText(
  metadata: ProductDetail["metadata"],
  key: string,
): string | undefined {
  const value = metadata[key];
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

export function readMetadataTextArray(
  metadata: ProductDetail["metadata"],
  key: string,
): string[] {
  const value = metadata[key];
  if (Array.isArray(value)) {
    return value.filter((item): item is string => typeof item === "string");
  }
  if (typeof value === "string" && value.trim()) {
    return value
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
  }
  return [];
}

export function metadataDisplayEntries(
  metadata: ProductDetail["metadata"],
  limit = 12,
): { key: string; value: string; hidden: boolean }[] {
  return Object.entries(metadata)
    .slice(0, limit)
    .map(([key, value]) => {
      if (HIDDEN_METADATA_KEY_PATTERN.test(key)) {
        return {
          key,
          value: "已隐藏受控对象路径",
          hidden: true,
        };
      }
      return {
        key,
        value: formatMetadataValue(value),
        hidden: false,
      };
    });
}

export function productStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    draft: "草稿",
    pending_review: "待审核",
    listed: "已上架",
    delisted: "已下架",
    frozen: "已冻结",
    suspended: "已暂停",
    retired: "已退役",
  };
  return labels[status] ?? status;
}

function formatMetadataValue(value: unknown): string {
  if (value === null || value === undefined) {
    return "未返回";
  }
  if (typeof value === "string") {
    return value || "未返回";
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (Array.isArray(value)) {
    return value.map((item) => formatMetadataValue(item)).join(", ") || "[]";
  }
  return JSON.stringify(value);
}
