import type {
  BillingOrderDetailResponse,
  ExecuteCompensationRequest,
  ExecuteCompensationResponse,
  ExecuteRefundRequest,
  ExecuteRefundResponse,
} from "@datab/sdk-ts";
import { PlatformApiError } from "@datab/sdk-ts";
import { z } from "zod";

import type { SessionSubject } from "./acceptance-workflow";

export type BillingOrderDetail =
  NonNullable<BillingOrderDetailResponse["data"]>;
export type RefundExecutionResult =
  NonNullable<ExecuteRefundResponse["data"]>;
export type CompensationExecutionResult =
  NonNullable<ExecuteCompensationResponse["data"]>;

export const BILLING_READ_ALLOWED_ROLES = [
  "buyer_operator",
  "tenant_admin",
  "platform_risk_settlement",
] as const;

export const BILLING_ACTION_ALLOWED_ROLES = [
  "platform_risk_settlement",
] as const;

const uuidLiteralSchema = z
  .string()
  .trim()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    "请输入 PostgreSQL UUID 字面量",
  );

export const billingLookupSchema = z.object({
  order_id: uuidLiteralSchema,
});

const decimalStringSchema = z
  .string()
  .trim()
  .regex(/^\d+(\.\d{1,8})?$/, "金额必须是最多 8 位小数的正数")
  .refine((value) => Number(value) > 0, "金额必须大于 0");

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

const highRiskBaseSchema = {
  order_id: uuidLiteralSchema,
  case_id: uuidLiteralSchema,
  decision_code: z.string().trim().min(2, "decision_code 必填"),
  penalty_code: z.string().trim().optional(),
  amount: decimalStringSchema,
  currency_code: z.string().trim().optional(),
  reason_code: z.string().trim().min(2, "reason_code 必填"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
  idempotency_key: z.string().trim().min(12, "X-Idempotency-Key 必填"),
  metadata_json: jsonRecordSchema,
  confirm_liability: z.boolean().refine(Boolean, {
    message: "必须确认已有责任判定",
  }),
  confirm_step_up: z.boolean().refine(Boolean, {
    message: "必须确认高风险动作已完成 step-up",
  }),
  confirm_audit: z.boolean().refine(Boolean, {
    message: "必须确认动作会写入审计留痕",
  }),
};

function requireStepUp<
  T extends { step_up_token?: string; step_up_challenge_id?: string },
>(value: T, ctx: z.RefinementCtx) {
  if (!value.step_up_token && !value.step_up_challenge_id) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      path: ["step_up_token"],
      message: "X-Step-Up-Token 或 X-Step-Up-Challenge-Id 至少填写一个",
    });
  }
}

export const refundExecutionFormSchema = z
  .object({
    ...highRiskBaseSchema,
    refund_mode: z.string().trim().optional(),
    refund_template: z.string().trim().optional(),
  })
  .superRefine(requireStepUp);

export const compensationExecutionFormSchema = z
  .object({
    ...highRiskBaseSchema,
    compensation_mode: z.string().trim().optional(),
    compensation_template: z.string().trim().optional(),
  })
  .superRefine(requireStepUp);

export type BillingLookupValues = z.infer<typeof billingLookupSchema>;
export type RefundExecutionFormValues = z.infer<typeof refundExecutionFormSchema>;
export type CompensationExecutionFormValues =
  z.infer<typeof compensationExecutionFormSchema>;

export function defaultRefundExecutionValues(
  orderId = "",
  caseId = "",
): RefundExecutionFormValues {
  return {
    order_id: orderId,
    case_id: caseId,
    decision_code: "refund_full",
    penalty_code: "",
    amount: "",
    currency_code: "",
    reason_code: "seller_fault",
    refund_mode: "manual_refund",
    refund_template: "REFUND_FILE_V1",
    step_up_token: "",
    step_up_challenge_id: "",
    idempotency_key: createBillingIdempotencyKey("refund"),
    metadata_json: jsonInputValue({ source: "WEB-012" }),
    confirm_liability: false,
    confirm_step_up: false,
    confirm_audit: false,
  };
}

export function defaultCompensationExecutionValues(
  orderId = "",
  caseId = "",
): CompensationExecutionFormValues {
  return {
    order_id: orderId,
    case_id: caseId,
    decision_code: "compensation_full",
    penalty_code: "",
    amount: "",
    currency_code: "",
    reason_code: "sla_breach",
    compensation_mode: "manual_transfer",
    compensation_template: "COMPENSATION_FILE_V1",
    step_up_token: "",
    step_up_challenge_id: "",
    idempotency_key: createBillingIdempotencyKey("compensation"),
    metadata_json: jsonInputValue({ source: "WEB-012" }),
    confirm_liability: false,
    confirm_step_up: false,
    confirm_audit: false,
  };
}

export function buildRefundExecutionRequest(
  values: RefundExecutionFormValues,
): ExecuteRefundRequest {
  return compactObject({
    order_id: values.order_id.trim(),
    case_id: values.case_id.trim(),
    decision_code: values.decision_code.trim(),
    penalty_code: emptyToUndefined(values.penalty_code),
    amount: values.amount.trim(),
    currency_code: emptyToUndefined(values.currency_code),
    reason_code: values.reason_code.trim(),
    refund_mode: emptyToUndefined(values.refund_mode),
    refund_template: emptyToUndefined(values.refund_template),
    metadata: parseJsonRecord(values.metadata_json) ?? {},
  }) as ExecuteRefundRequest;
}

export function buildCompensationExecutionRequest(
  values: CompensationExecutionFormValues,
): ExecuteCompensationRequest {
  return compactObject({
    order_id: values.order_id.trim(),
    case_id: values.case_id.trim(),
    decision_code: values.decision_code.trim(),
    penalty_code: emptyToUndefined(values.penalty_code),
    amount: values.amount.trim(),
    currency_code: emptyToUndefined(values.currency_code),
    reason_code: values.reason_code.trim(),
    compensation_mode: emptyToUndefined(values.compensation_mode),
    compensation_template: emptyToUndefined(values.compensation_template),
    metadata: parseJsonRecord(values.metadata_json) ?? {},
  }) as ExecuteCompensationRequest;
}

export function createBillingIdempotencyKey(
  action: "refund" | "compensation",
): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `web-012-billing-${action}-${crypto.randomUUID()}`;
  }
  return `web-012-billing-${action}-${Date.now()}`;
}

export function unwrapBillingOrder(response: BillingOrderDetailResponse | undefined) {
  return response?.data ?? null;
}

export function unwrapRefundExecution(response: ExecuteRefundResponse | undefined) {
  return response?.data ?? null;
}

export function unwrapCompensationExecution(
  response: ExecuteCompensationResponse | undefined,
) {
  return response?.data ?? null;
}

export function canReadBilling(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, BILLING_READ_ALLOWED_ROLES);
}

export function canExecuteBillingAdjustments(
  subject: SessionSubject | null | undefined,
) {
  return hasAnyRole(subject?.roles, BILLING_ACTION_ALLOWED_ROLES);
}

export function readBillingSubjectTenant(subject: SessionSubject | null | undefined) {
  return subject?.tenant_id ?? subject?.org_id ?? "";
}

export function formatBillingError(error: unknown) {
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
  return "BIL_PROVIDER_FAILED: 账单 API 请求失败";
}

export function formatMoney(amount: string | undefined, currency: string | undefined) {
  if (!amount) {
    return "未返回";
  }
  return `${amount} ${currency ?? ""}`.trim();
}

export function billingStatusTiles(detail: BillingOrderDetail) {
  return [
    ["order_status", detail.order_status],
    ["payment_status", detail.payment_status],
    ["settlement_status", detail.settlement_status],
    ["dispute_status", detail.dispute_status],
    ["deposit_status", "未返回"],
    ["order_amount", formatMoney(detail.order_amount, detail.currency_code)],
  ] as const;
}

export function latestBillingEvent(detail: BillingOrderDetail) {
  return detail.billing_events.at(0) ?? null;
}

export function hasRefundOrCompensation(detail: BillingOrderDetail | null) {
  return Boolean(detail && (detail.refunds.length || detail.compensations.length));
}

function hasAnyRole(
  roles: readonly string[] | undefined,
  allowed: readonly string[],
) {
  return Boolean(roles?.some((role) => allowed.includes(role)));
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
