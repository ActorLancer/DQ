import type {
  AcceptOrderRequest,
  AcceptOrderResponse,
  AuthMeResponse,
  RejectOrderRequest,
  RejectOrderResponse,
} from "@datab/sdk-ts";
import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";
import { z } from "zod";

import type { OrderDetail, OrderLifecycleSnapshots } from "./order-workflow";
import { standardSkuOptions, type StandardSkuType } from "./seller-products-view";

export type SessionSubject = AuthMeResponse["data"];
export type { OrderDetail, OrderLifecycleSnapshots } from "./order-workflow";
export type AcceptanceDecisionResult =
  NonNullable<AcceptOrderResponse["data"]>;
export type RejectionDecisionResult =
  NonNullable<RejectOrderResponse["data"]>;

export const MANUAL_ACCEPTANCE_SKUS = [
  "FILE_STD",
  "FILE_SUB",
  "RPT_STD",
] as const satisfies readonly StandardSkuType[];

const MANUAL_ACCEPTANCE_READY_STATES: Record<string, readonly string[]> = {
  FILE_STD: ["delivered"],
  FILE_SUB: ["delivered"],
  RPT_STD: ["report_delivered"],
};

export const ACCEPTANCE_READ_ALLOWED_ROLES = [
  "buyer_operator",
  "seller_operator",
  "tenant_admin",
  "tenant_developer",
  "tenant_audit_readonly",
  "platform_admin",
  "platform_audit_security",
  "platform_risk_settlement",
] as const;

export const ACCEPTANCE_ACTION_ALLOWED_ROLES = [
  "buyer_operator",
] as const;

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
    message: "必须是可解析的 JSON object",
  });

const baseDecisionSchema = {
  verification_summary_json: jsonRecordSchema,
  confirm_verification: z.boolean().refine(Boolean, {
    message: "必须确认已完成 Hash / 合同 / 模板核验",
  }),
  confirm_scope: z.boolean().refine(Boolean, {
    message: "必须确认验收对象来自当前订单和 SKU 快照",
  }),
  confirm_audit: z.boolean().refine(Boolean, {
    message: "必须确认关键动作会写入审计留痕",
  }),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
};

export const acceptOrderFormSchema = z.object({
  note: z.string().trim().optional(),
  ...baseDecisionSchema,
});

export const rejectOrderFormSchema = z.object({
  reason_code: z.string().trim().min(2, "拒收 reason_code 必填"),
  reason_detail: z.string().trim().min(6, "拒收原因至少 6 个字符"),
  ...baseDecisionSchema,
});

export type AcceptOrderFormValues = z.infer<typeof acceptOrderFormSchema>;
export type RejectOrderFormValues = z.infer<typeof rejectOrderFormSchema>;

export function defaultAcceptOrderValues(): AcceptOrderFormValues {
  return {
    note: "",
    verification_summary_json: jsonInputValue({
      hash_match: true,
      contract_template_match: true,
    }),
    confirm_verification: false,
    confirm_scope: false,
    confirm_audit: false,
    idempotency_key: createAcceptanceIdempotencyKey("accept"),
  };
}

export function defaultRejectOrderValues(): RejectOrderFormValues {
  return {
    reason_code: "",
    reason_detail: "",
    verification_summary_json: jsonInputValue({
      hash_match: false,
      contract_template_match: false,
    }),
    confirm_verification: false,
    confirm_scope: false,
    confirm_audit: false,
    idempotency_key: createAcceptanceIdempotencyKey("reject"),
  };
}

export function buildAcceptOrderRequest(
  values: AcceptOrderFormValues,
): AcceptOrderRequest {
  return compactObject({
    note: emptyToUndefined(values.note),
    verification_summary: parseJsonRecord(values.verification_summary_json),
  }) as AcceptOrderRequest;
}

export function buildRejectOrderRequest(
  values: RejectOrderFormValues,
): RejectOrderRequest {
  return compactObject({
    reason_code: values.reason_code.trim(),
    reason_detail: values.reason_detail.trim(),
    verification_summary: parseJsonRecord(values.verification_summary_json),
  }) as RejectOrderRequest;
}

export function createAcceptanceIdempotencyKey(action: "accept" | "reject"): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `web-011-acceptance-${action}-${crypto.randomUUID()}`;
  }
  return `web-011-acceptance-${action}-${Date.now()}`;
}

function unwrapEnvelopeData<T>(
  response:
    | {
        data?: T | null;
      }
    | undefined,
) {
  return response?.data ?? null;
}

export function unwrapAcceptOrder(response: AcceptOrderResponse | undefined) {
  return unwrapEnvelopeData<AcceptanceDecisionResult>(response);
}

export function unwrapRejectOrder(response: RejectOrderResponse | undefined) {
  return unwrapEnvelopeData<RejectionDecisionResult>(response);
}

export function canReadAcceptance(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, ACCEPTANCE_READ_ALLOWED_ROLES);
}

export function canOperateAcceptance(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, ACCEPTANCE_ACTION_ALLOWED_ROLES);
}

export function getOrderSkuType(order: OrderDetail | null | undefined) {
  return order?.price_snapshot?.sku_type ?? "";
}

export function isManualAcceptanceSku(skuType: string | undefined) {
  return (MANUAL_ACCEPTANCE_SKUS as readonly string[]).includes(skuType ?? "");
}

export function isDeliveredForAcceptance(order: OrderDetail | null | undefined) {
  const skuType = getOrderSkuType(order);
  return Boolean(
    order?.current_state &&
      MANUAL_ACCEPTANCE_READY_STATES[skuType]?.includes(order.current_state),
  );
}

export function readSubjectTenant(subject: SessionSubject | null | undefined) {
  return subject?.tenant_id ?? subject?.org_id ?? "";
}

export function skuDisplayName(skuType: string | undefined) {
  return (
    standardSkuOptions.find((option) => option.sku_type === skuType)?.label ??
    skuType ??
    "未知 SKU"
  );
}

export function formatAcceptanceError(error: unknown) {
  return formatPlatformErrorForDisplay(error, {
    fallbackCode: "DELIVERY_STATUS_INVALID",
    fallbackDescription: "请刷新订单详情和交付状态后，再决定是否继续执行验收动作。",
  });
}

export function lifecycleRows(
  order: OrderDetail,
  lifecycle: OrderLifecycleSnapshots | null,
) {
  return [
    ["订单", lifecycle?.order.current_state ?? order.current_state, order.updated_at],
    [
      "支付",
      lifecycle?.order.payment.current_status ?? order.payment_status,
      lifecycle?.order.payment.buyer_locked_at ?? "未返回",
    ],
    [
      "交付",
      lifecycle?.delivery?.current_status ?? order.delivery_status,
      lifecycle?.delivery?.committed_at ?? "未返回",
    ],
    [
      "验收",
      lifecycle?.order.acceptance.current_status ?? order.acceptance_status,
      lifecycle?.order.acceptance.accepted_at ?? "未返回",
    ],
    [
      "结算",
      lifecycle?.order.settlement.current_status ?? order.settlement_status,
      lifecycle?.order.settlement.settled_at ?? "未返回",
    ],
    [
      "争议",
      lifecycle?.order.dispute.current_status ?? order.dispute_status,
      lifecycle?.order.dispute.last_reason_code ?? "无",
    ],
  ] as const;
}

export function latestDelivery(order: OrderDetail, lifecycle: OrderLifecycleSnapshots | null) {
  return (
    order.relations.deliveries.at(0) ??
    (lifecycle?.delivery
      ? {
          delivery_id: lifecycle.delivery.delivery_id,
          delivery_type: lifecycle.delivery.delivery_type,
          delivery_route: lifecycle.delivery.delivery_route,
          current_status: lifecycle.delivery.current_status,
          delivery_commit_hash: null,
          receipt_hash: lifecycle.delivery.receipt_hash,
          committed_at: lifecycle.delivery.committed_at,
          expires_at: lifecycle.delivery.expires_at,
          storage_gateway: lifecycle.delivery.storage_gateway,
          updated_at: lifecycle.delivery.updated_at,
        }
      : null)
  );
}

export function verificationSummary(order: OrderDetail, lifecycle: OrderLifecycleSnapshots | null) {
  const delivery = latestDelivery(order, lifecycle);
  const integrity = delivery?.storage_gateway?.integrity;
  return {
    deliveryId: delivery?.delivery_id ?? "未返回",
    deliveryType: delivery?.delivery_type ?? "未返回",
    deliveryStatus: delivery?.current_status ?? "未返回",
    receiptHash: delivery?.receipt_hash ?? integrity?.receipt_hash ?? "未返回",
    deliveryCommitHash:
      delivery?.delivery_commit_hash ?? integrity?.delivery_commit_hash ?? "未返回",
    contentHash: integrity?.content_hash ?? "未返回",
    contractStatus: order.relations.contract?.contract_status ?? "未返回",
    acceptanceTemplate:
      order.price_snapshot?.scenario_snapshot?.acceptance_template ?? "未返回",
    disputeStatus: order.dispute_status,
  };
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
