import { normalizePlatformError } from "@datab/sdk-ts";
import type {
  AuthMeResponse,
  OrganizationListResponse,
  ProductDetailResponse,
  ProductListResponse,
  ReviewDecisionRequest,
} from "@datab/sdk-ts";
import { z } from "zod";

export type ReviewWorkbenchKind = "subjects" | "products" | "compliance";
export type SessionSubject = AuthMeResponse["data"];
export type OrganizationReviewRow = OrganizationListResponse["data"][number];
export type ProductReviewRow = ProductListResponse["data"]["items"][number];
export type ProductReviewDetail = ProductDetailResponse["data"];

export const reviewWorkbenchTitles: Record<ReviewWorkbenchKind, string> = {
  subjects: "主体审核台",
  products: "产品审核台",
  compliance: "合规审核台",
};

export const reviewAuditActions: Record<ReviewWorkbenchKind, string> = {
  subjects: "catalog.review.subject",
  products: "catalog.review.product",
  compliance: "catalog.review.compliance",
};

export const standardSkuTypes = [
  "FILE_STD",
  "FILE_SUB",
  "SHARE_RO",
  "API_SUB",
  "API_PPU",
  "QRY_LITE",
  "SBX_STD",
  "RPT_STD",
] as const;

export const reviewDecisionFormSchema = z
  .object({
    action: z.enum(["approve", "reject", "block"]),
    action_reason: z
      .string()
      .trim()
      .min(6, "审核原因至少 6 个字符")
      .max(600, "审核原因不能超过 600 个字符"),
    idempotency_key: z
      .string()
      .trim()
      .min(10, "缺少 X-Idempotency-Key"),
    step_up_token: z.string().trim().optional(),
    step_up_challenge_id: z.string().trim().optional(),
    block_confirmation: z.string().trim().optional(),
  })
  .superRefine((value, ctx) => {
    if (value.action !== "block") {
      return;
    }
    if (value.block_confirmation !== "BLOCK") {
      ctx.addIssue({
        code: "custom",
        path: ["block_confirmation"],
        message: "合规阻断必须输入 BLOCK 完成人工确认",
      });
    }
  });

export type ReviewDecisionFormValues = z.infer<typeof reviewDecisionFormSchema>;

export function createReviewIdempotencyKey(kind: ReviewWorkbenchKind) {
  const entropy =
    typeof globalThis.crypto?.randomUUID === "function"
      ? globalThis.crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `web-008:${kind}:${entropy}`;
}

export function canReadReviewWorkbench(subject?: SessionSubject | null) {
  return hasAnyRole(subject, ["platform_admin", "platform_reviewer"]);
}

export function canWriteReviewDecision(subject?: SessionSubject | null) {
  return hasAnyRole(subject, ["platform_admin", "platform_reviewer"]);
}

export function hasAnyRole(
  subject: SessionSubject | null | undefined,
  allowedRoles: string[],
) {
  const roles = new Set(subject?.roles ?? []);
  return allowedRoles.some((role) => roles.has(role));
}

export function buildReviewDecisionPayload(
  values: ReviewDecisionFormValues,
): ReviewDecisionRequest {
  return {
    action_name: values.action === "approve" ? "approve" : "reject",
    action_reason:
      values.action === "block"
        ? `合规阻断确认：${values.action_reason.trim()}`
        : values.action_reason.trim(),
  };
}

export function deriveComplianceSignals(product?: ProductReviewDetail | null) {
  const metadata = asRecord(product?.metadata);
  const declaredPurpose =
    readString(metadata, "declared_usage_purpose") ??
    readString(metadata, "usage_purpose") ??
    product?.use_cases.join(" / ") ??
    "未声明";
  const region =
    readString(metadata, "region_code") ??
    readString(metadata, "jurisdiction_code") ??
    "未声明";
  const exportPolicy =
    readString(metadata, "export_restriction") ??
    readString(metadata, "download_policy") ??
    "未声明";
  const riskTags = readStringArray(metadata, "risk_tags");
  const highRiskSignals = [
    product?.data_classification?.includes("L3") ? "L3 分类分级需人工复核" : null,
    exportPolicy.includes("cross_border") ? "跨境导出限制命中" : null,
    riskTags.length ? `风险标签：${riskTags.join(" / ")}` : null,
    product?.skus.some((sku) => sku.sku_type === "SHARE_RO")
      ? "SHARE_RO 授权类 SKU 需关注撤销与账单联动"
      : null,
  ].filter(Boolean) as string[];

  return {
    dataClassification: product?.data_classification ?? "未声明",
    declaredPurpose,
    region,
    exportPolicy,
    highRiskSignals,
    automaticBlockResult: highRiskSignals.length ? "manual_review" : "pass",
  };
}

export function formatReviewError(error: unknown) {
  const normalized = normalizePlatformError(error, {
    fallbackCode: "INTERNAL_ERROR",
    fallbackDescription: "请结合错误码和 request_id 回查审核队列、权限配置和对象当前状态。",
  });

  return {
    title: `${normalized.title} · ${normalized.code}`,
    message: normalized.description,
    requestId: normalized.requestId ?? "未返回 request_id",
  };
}

export function labelReviewStatus(status?: string | null) {
  const labels: Record<string, string> = {
    pending_review: "待人工审核",
    manual_review: "人工复核",
    approved: "已通过",
    rejected: "已驳回",
    restricted: "受限",
    draft: "草稿",
    listed: "已上架",
    frozen: "冻结",
    delisted: "已下架",
    watch: "观察",
  };
  return status ? (labels[status] ?? status) : "未声明";
}

export function formatDateTime(value?: string | null) {
  if (!value) {
    return "未返回";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("zh-CN", {
    dateStyle: "medium",
    timeStyle: "short",
    timeZone: "Asia/Shanghai",
  }).format(date);
}

export function countStandardSkuCoverage(product?: ProductReviewDetail | null) {
  const skuTypes = new Set(product?.skus.map((sku) => sku.sku_type) ?? []);
  return standardSkuTypes.map((skuType) => ({
    skuType,
    present: skuTypes.has(skuType),
  }));
}

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function readString(record: Record<string, unknown>, key: string) {
  const value = record[key];
  return typeof value === "string" && value.trim() ? value : null;
}

function readStringArray(record: Record<string, unknown>, key: string) {
  const value = record[key];
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter(
    (item): item is string => typeof item === "string" && item.trim().length > 0,
  );
}
