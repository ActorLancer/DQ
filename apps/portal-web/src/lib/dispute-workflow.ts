import type {
  CreateDisputeCaseRequest,
  CreateDisputeCaseResponse,
  ResolveDisputeCaseRequest,
  ResolveDisputeCaseResponse,
  UploadDisputeEvidenceResponse,
} from "@datab/sdk-ts";
import { PlatformApiError } from "@datab/sdk-ts";
import { z } from "zod";

import type { OrderDetail, SessionSubject } from "./order-workflow";
import { hasAnyRole } from "./order-workflow";

export type DisputeCaseResult =
  NonNullable<CreateDisputeCaseResponse["data"]>;
export type DisputeEvidenceResult =
  NonNullable<UploadDisputeEvidenceResponse["data"]>;
export type DisputeResolutionResult =
  NonNullable<ResolveDisputeCaseResponse["data"]>;
export type OrderDisputeSummary =
  OrderDetail["relations"]["disputes"][number];

export const DISPUTE_READ_ALLOWED_ROLES = [
  "buyer_operator",
  "seller_operator",
  "tenant_admin",
  "platform_admin",
  "platform_audit_security",
  "platform_risk_settlement",
] as const;

export const DISPUTE_CREATE_ALLOWED_ROLES = [
  "buyer_operator",
] as const;

export const DISPUTE_RESOLVE_ALLOWED_ROLES = [
  "platform_risk_settlement",
] as const;

export const disputeReasonExamples = [
  "delivery_failed",
  "share_access_interrupted",
  "billing_adjustment_dispute",
  "contract_delivery_mismatch",
  "report_quality_failed",
] as const;

export const disputeEvidenceObjectTypes = [
  "delivery_receipt",
  "download_log",
  "object_hash",
  "key_envelope",
  "contract_snapshot",
  "chat_comment",
] as const;

const uuidLiteralSchema = z
  .string()
  .trim()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    "请输入 PostgreSQL UUID 字面量",
  );

const optionalDecimalStringSchema = z
  .string()
  .trim()
  .optional()
  .refine((value) => {
    if (!value) {
      return true;
    }
    return /^\d+(\.\d{1,8})?$/.test(value) && Number(value) > 0;
  }, "金额必须是最多 8 位小数的正数");

const jsonRecordSchema = z
  .string()
  .trim()
  .optional()
  .refine((value) => {
    if (!value) {
      return true;
    }
    try {
      const parsed = JSON.parse(value) as unknown;
      return Boolean(parsed) && typeof parsed === "object" && !Array.isArray(parsed);
    } catch {
      return false;
    }
  }, {
    message: "metadata 必须是 JSON object",
  });

export const disputeLookupSchema = z.object({
  order_id: uuidLiteralSchema,
  case_id: uuidLiteralSchema.optional().or(z.literal("")),
});

export const createDisputeCaseFormSchema = z.object({
  order_id: uuidLiteralSchema,
  reason_code: z.string().trim().min(2, "reason_code 必填"),
  requested_resolution: z.string().trim().optional(),
  claimed_amount: optionalDecimalStringSchema,
  evidence_scope: z.string().trim().optional(),
  blocking_effect: z.string().trim().optional(),
  metadata_json: jsonRecordSchema,
  idempotency_key: z.string().trim().min(12, "X-Idempotency-Key 必填"),
  confirm_order_scope: z.boolean().refine(Boolean, {
    message: "必须确认争议属于当前订单与租户范围",
  }),
  confirm_audit: z.boolean().refine(Boolean, {
    message: "必须确认创建争议会写入审计留痕",
  }),
});

export const uploadDisputeEvidenceFormSchema = z.object({
  case_id: uuidLiteralSchema,
  object_type: z.string().trim().min(2, "object_type 必填"),
  metadata_json: jsonRecordSchema,
  idempotency_key: z.string().trim().min(12, "X-Idempotency-Key 必填"),
  confirm_no_raw_path: z.boolean().refine(Boolean, {
    message: "必须确认页面不会展示对象存储真实路径",
  }),
  confirm_audit: z.boolean().refine(Boolean, {
    message: "必须确认上传证据会写入证据链和审计留痕",
  }),
});

export const resolveDisputeCaseFormSchema = z
  .object({
    case_id: uuidLiteralSchema,
    decision_type: z.string().trim().optional(),
    decision_code: z.string().trim().min(2, "decision_code 必填"),
    liability_type: z.string().trim().optional(),
    penalty_code: z.string().trim().optional(),
    decision_text: z.string().trim().optional(),
    metadata_json: jsonRecordSchema,
    step_up_token: z.string().trim().optional(),
    step_up_challenge_id: z.string().trim().optional(),
    idempotency_key: z.string().trim().min(12, "X-Idempotency-Key 必填"),
    confirm_sod: z.boolean().refine(Boolean, {
      message: "必须确认平台侧职责分离和人工裁决要求",
    }),
    confirm_step_up: z.boolean().refine(Boolean, {
      message: "必须确认高风险裁决已完成 step-up",
    }),
    confirm_audit: z.boolean().refine(Boolean, {
      message: "必须确认裁决会写入审计留痕",
    }),
  })
  .superRefine((value, ctx) => {
    if (!value.step_up_token && !value.step_up_challenge_id) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ["step_up_token"],
        message: "X-Step-Up-Token 或 X-Step-Up-Challenge-Id 至少填写一个",
      });
    }
  });

export type DisputeLookupValues = z.infer<typeof disputeLookupSchema>;
export type CreateDisputeCaseFormValues =
  z.infer<typeof createDisputeCaseFormSchema>;
export type UploadDisputeEvidenceFormValues =
  z.infer<typeof uploadDisputeEvidenceFormSchema>;
export type ResolveDisputeCaseFormValues =
  z.infer<typeof resolveDisputeCaseFormSchema>;

export function defaultCreateDisputeCaseValues(
  orderId = "",
): CreateDisputeCaseFormValues {
  return {
    order_id: orderId,
    reason_code: "delivery_failed",
    requested_resolution: "refund_full",
    claimed_amount: "",
    evidence_scope: "delivery_receipt,download_log,object_hash",
    blocking_effect: "settlement_freeze",
    metadata_json: jsonInputValue({ source: "WEB-013" }),
    idempotency_key: createDisputeIdempotencyKey("case"),
    confirm_order_scope: false,
    confirm_audit: false,
  };
}

export function defaultUploadDisputeEvidenceValues(
  caseId = "",
): UploadDisputeEvidenceFormValues {
  return {
    case_id: caseId,
    object_type: "delivery_receipt",
    metadata_json: jsonInputValue({
      source: "WEB-013",
      evidence_channel: "portal_upload",
    }),
    idempotency_key: createDisputeIdempotencyKey("evidence"),
    confirm_no_raw_path: false,
    confirm_audit: false,
  };
}

export function defaultResolveDisputeCaseValues(
  caseId = "",
): ResolveDisputeCaseFormValues {
  return {
    case_id: caseId,
    decision_type: "manual_resolution",
    decision_code: "refund_full",
    liability_type: "seller",
    penalty_code: "seller_full_refund",
    decision_text: "",
    metadata_json: jsonInputValue({ source: "WEB-013" }),
    step_up_token: "",
    step_up_challenge_id: "",
    idempotency_key: createDisputeIdempotencyKey("resolve"),
    confirm_sod: false,
    confirm_step_up: false,
    confirm_audit: false,
  };
}

export function buildCreateDisputeCaseRequest(
  values: CreateDisputeCaseFormValues,
): CreateDisputeCaseRequest {
  return compactObject({
    order_id: values.order_id.trim(),
    reason_code: values.reason_code.trim(),
    requested_resolution: emptyToUndefined(values.requested_resolution),
    claimed_amount: emptyToUndefined(values.claimed_amount),
    evidence_scope: emptyToUndefined(values.evidence_scope),
    blocking_effect: emptyToUndefined(values.blocking_effect),
    metadata: parseJsonRecord(values.metadata_json) ?? {},
  }) as CreateDisputeCaseRequest;
}

export function buildDisputeEvidenceFormData(
  values: UploadDisputeEvidenceFormValues,
  file: Blob,
  fileName = "dispute-evidence.txt",
): FormData {
  const formData = new FormData();
  formData.set("object_type", values.object_type.trim());
  const metadata = values.metadata_json?.trim();
  if (metadata) {
    formData.set("metadata_json", metadata);
  }
  formData.set("file", file, fileName);
  return formData;
}

export function buildResolveDisputeCaseRequest(
  values: ResolveDisputeCaseFormValues,
): ResolveDisputeCaseRequest {
  return compactObject({
    decision_type: emptyToUndefined(values.decision_type),
    decision_code: values.decision_code.trim(),
    liability_type: emptyToUndefined(values.liability_type),
    penalty_code: emptyToUndefined(values.penalty_code),
    decision_text: emptyToUndefined(values.decision_text),
    metadata: parseJsonRecord(values.metadata_json) ?? {},
  }) as ResolveDisputeCaseRequest;
}

export function createDisputeIdempotencyKey(
  action: "case" | "evidence" | "resolve",
): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `web-013-dispute-${action}-${crypto.randomUUID()}`;
  }
  return `web-013-dispute-${action}-${Date.now()}`;
}

export function unwrapCreatedDisputeCase(
  response: CreateDisputeCaseResponse | undefined,
) {
  return response?.data ?? null;
}

export function unwrapDisputeEvidence(
  response: UploadDisputeEvidenceResponse | undefined,
) {
  return response?.data ?? null;
}

export function unwrapDisputeResolution(
  response: ResolveDisputeCaseResponse | undefined,
) {
  return response?.data ?? null;
}

export function canReadDispute(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, DISPUTE_READ_ALLOWED_ROLES);
}

export function canCreateDispute(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, DISPUTE_CREATE_ALLOWED_ROLES);
}

export function canResolveDispute(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, DISPUTE_RESOLVE_ALLOWED_ROLES);
}

export function readDisputeSubjectTenant(
  subject: SessionSubject | null | undefined,
) {
  return subject?.tenant_id ?? subject?.org_id ?? "";
}

export function selectActiveDisputeCase(
  order: OrderDetail | null | undefined,
  explicitCaseId?: string,
) {
  const disputes = order?.relations.disputes ?? [];
  if (explicitCaseId) {
    return disputes.find((item) => item.case_id === explicitCaseId) ?? null;
  }
  return [...disputes].sort((left, right) =>
    right.updated_at.localeCompare(left.updated_at),
  )[0] ?? null;
}

export function formatDisputeError(error: unknown) {
  if (error instanceof PlatformApiError) {
    return [
      error.code || "UNKNOWN",
      error.message,
      error.requestId ? `request_id=${error.requestId}` : "",
    ]
      .filter(Boolean)
      .join(" / ");
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "DISPUTE_STATUS_INVALID: 争议 API 请求失败";
}

export function hiddenObjectPathNotice(result: DisputeEvidenceResult | null) {
  if (!result) {
    return "未上传";
  }
  return result.object_hash
    ? `evidence_hash=${result.object_hash}`
    : "后端未返回 evidence_hash；对象路径已按前端边界隐藏";
}

function jsonInputValue(value: unknown) {
  return JSON.stringify(value ?? {}, null, 2);
}

function emptyToUndefined(value: string | undefined) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function parseJsonRecord(value: string | undefined) {
  const trimmed = value?.trim();
  if (!trimmed) {
    return undefined;
  }
  return JSON.parse(trimmed) as Record<string, unknown>;
}

function compactObject(value: Record<string, unknown>) {
  return Object.fromEntries(
    Object.entries(value).filter(([, entryValue]) => entryValue !== undefined),
  );
}
