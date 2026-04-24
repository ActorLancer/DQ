import type {
  AuthMeResponse,
  CancelOrderResponse,
  CreateOrderRequest,
  CreateOrderResponse,
  OrderDetailResponse,
  OrderLifecycleSnapshotsResponse,
  ProductDetailResponse,
  StandardOrderTemplatesResponse,
} from "@datab/sdk-ts";
import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";
import { z } from "zod";

import { standardSkuOptions, type StandardSkuType } from "./seller-products-view";

export type OrderTemplate = StandardOrderTemplatesResponse["data"][number];
export type ProductDetail = ProductDetailResponse["data"];
export type ProductSku = ProductDetail["skus"][number];
export type SessionSubject = AuthMeResponse["data"];
export type CreatedOrder = NonNullable<CreateOrderResponse["data"]>["data"];
export type OrderDetail = NonNullable<OrderDetailResponse["data"]>["data"];
export type OrderLifecycleSnapshots =
  NonNullable<OrderLifecycleSnapshotsResponse["data"]>["data"];
export type CanceledOrder = NonNullable<CancelOrderResponse["data"]>["data"];

export const ORDER_CREATE_ALLOWED_ROLES = [
  "buyer_operator",
  "tenant_admin",
] as const;

export const ORDER_READ_ALLOWED_ROLES = [
  "buyer_operator",
  "seller_operator",
  "tenant_admin",
  "platform_admin",
  "platform_audit_security",
  "platform_risk_settlement",
] as const;

export const ORDER_CANCEL_ALLOWED_ROLES = [
  "buyer_operator",
  "tenant_admin",
  "platform_admin",
  "platform_risk_settlement",
] as const;

export const orderCreateSchema = z
  .object({
    buyer_org_id: z.string().uuid("buyer_org_id 必须是 UUID"),
    product_id: z.string().uuid("product_id 必须是 UUID"),
    sku_id: z.string().uuid("必须选择一个标准 SKU"),
    scenario_code: z.string().trim().optional(),
    quantity: z.number().int().min(1, "数量至少为 1"),
    term_days: z.number().int().min(1, "期限至少 1 天"),
    subscription_cadence: z.enum(["none", "monthly", "weekly", "per_use"]),
    confirm_rights: z.boolean().refine((value) => value, {
      message: "必须确认使用权利、地域与用途边界",
    }),
    confirm_snapshot: z.boolean().refine((value) => value, {
      message: "必须确认 SKU / 价格 / 模板将进入订单快照",
    }),
    confirm_audit: z.boolean().refine((value) => value, {
      message: "必须确认下单动作会写入审计留痕",
    }),
    idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  })
  .superRefine((values, ctx) => {
    if (
      values.subscription_cadence === "none" &&
      values.term_days > 366
    ) {
      ctx.addIssue({
        code: "custom",
        path: ["term_days"],
        message: "非订阅订单期限不应超过 366 天",
      });
    }
  });

export type OrderCreateFormValues = z.infer<typeof orderCreateSchema>;

export const ORDER_SCENARIO_BLUEPRINTS: OrderTemplate[] = [
  {
    template_code: "ORDER_TEMPLATE_API_SUB_V1",
    scenario_code: "S1",
    scenario_name: "工业设备运行指标 API 订阅",
    industry_code: "industrial_manufacturing",
    primary_sku: "API_SUB",
    supplementary_skus: ["API_PPU"],
    contract_template: "CONTRACT_API_SUB_V1",
    acceptance_template: "ACCEPT_API_SUB_V1",
    refund_template: "REFUND_API_SUB_V1",
    workflow_steps: [
      "listed",
      "contract_confirm",
      "payment_lock_first_cycle",
      "bind_application",
      "issue_api_credential",
      "cycle_billing",
    ],
    order_draft: {
      primary_flow_code: "api_subscription",
      per_sku_snapshot_required: true,
      multi_sku_requires_independent_contract_authorization_settlement: true,
    },
  },
  {
    template_code: "ORDER_TEMPLATE_FILE_STD_V1",
    scenario_code: "S2",
    scenario_name: "工业质量与产线日报文件包交付",
    industry_code: "industrial_manufacturing",
    primary_sku: "FILE_STD",
    supplementary_skus: ["FILE_SUB"],
    contract_template: "CONTRACT_FILE_V1",
    acceptance_template: "ACCEPT_FILE_V1",
    refund_template: "REFUND_FILE_V1",
    workflow_steps: [
      "listed",
      "contract_confirm",
      "payment_lock_full",
      "issue_download_ticket",
      "acceptance",
      "settlement",
    ],
    order_draft: {
      primary_flow_code: "file_snapshot",
      per_sku_snapshot_required: true,
      multi_sku_requires_independent_contract_authorization_settlement: true,
    },
  },
  {
    template_code: "ORDER_TEMPLATE_SBX_STD_V1",
    scenario_code: "S3",
    scenario_name: "供应链协同查询沙箱",
    industry_code: "industrial_manufacturing",
    primary_sku: "SBX_STD",
    supplementary_skus: ["SHARE_RO"],
    contract_template: "CONTRACT_SANDBOX_V1",
    acceptance_template: "ACCEPT_SANDBOX_V1",
    refund_template: "REFUND_SANDBOX_V1",
    workflow_steps: [
      "listed",
      "contract_confirm",
      "payment_lock",
      "enable_sandbox",
      "restricted_query",
      "restricted_export",
      "settlement",
    ],
    order_draft: {
      primary_flow_code: "sandbox_query",
      per_sku_snapshot_required: true,
      multi_sku_requires_independent_contract_authorization_settlement: true,
    },
  },
  {
    template_code: "ORDER_TEMPLATE_API_SUB_RPT_V1",
    scenario_code: "S4",
    scenario_name: "零售门店经营分析 API / 报告订阅",
    industry_code: "retail",
    primary_sku: "API_SUB",
    supplementary_skus: ["RPT_STD"],
    contract_template: "CONTRACT_API_SUB_V1",
    acceptance_template: "ACCEPT_API_SUB_V1",
    refund_template: "REFUND_API_SUB_V1",
    workflow_steps: [
      "listed",
      "contract_confirm",
      "payment_lock_first_cycle",
      "bind_application",
      "issue_api_credential",
      "report_delivery_optional",
      "cycle_billing",
    ],
    order_draft: {
      primary_flow_code: "api_subscription",
      per_sku_snapshot_required: true,
      multi_sku_requires_independent_contract_authorization_settlement: true,
    },
  },
  {
    template_code: "ORDER_TEMPLATE_QRY_LITE_V1",
    scenario_code: "S5",
    scenario_name: "商圈/门店选址查询服务",
    industry_code: "retail",
    primary_sku: "QRY_LITE",
    supplementary_skus: ["RPT_STD"],
    contract_template: "CONTRACT_QUERY_LITE_V1",
    acceptance_template: "ACCEPT_QUERY_LITE_V1",
    refund_template: "REFUND_QUERY_LITE_V1",
    workflow_steps: [
      "listed",
      "contract_confirm",
      "payment_lock",
      "grant_template_query",
      "execute_template_query",
      "restricted_result_export",
      "settlement",
    ],
    order_draft: {
      primary_flow_code: "template_query",
      per_sku_snapshot_required: true,
      multi_sku_requires_independent_contract_authorization_settlement: true,
    },
  },
];

export function readStandardOrderTemplates(
  liveTemplates: OrderTemplate[] | undefined,
): OrderTemplate[] {
  return liveTemplates?.length ? liveTemplates : ORDER_SCENARIO_BLUEPRINTS;
}

export function collectOrderSkuCoverage(templates: OrderTemplate[]): StandardSkuType[] {
  const set = new Set<string>();
  for (const template of templates) {
    set.add(template.primary_sku);
    template.supplementary_skus.forEach((sku) => set.add(sku));
  }
  return standardSkuOptions
    .map((option) => option.sku_type)
    .filter((sku): sku is StandardSkuType => set.has(sku));
}

export function findTemplatesForSku(
  templates: OrderTemplate[],
  skuType: string,
): OrderTemplate[] {
  return templates.filter(
    (template) =>
      template.primary_sku === skuType ||
      template.supplementary_skus.includes(skuType),
  );
}

export function resolveTemplateForSku(
  templates: OrderTemplate[],
  skuType: string,
  scenarioCode?: string,
): OrderTemplate | null {
  const candidates = findTemplatesForSku(templates, skuType);
  if (scenarioCode) {
    return candidates.find((template) => template.scenario_code === scenarioCode) ?? null;
  }
  return candidates.length === 1 ? candidates[0] : null;
}

export function requiresScenarioCode(templates: OrderTemplate[], skuType: string): boolean {
  return findTemplatesForSku(templates, skuType).length > 1;
}

export function scenarioRole(template: OrderTemplate, skuType: string) {
  if (template.primary_sku === skuType) {
    return "primary";
  }
  if (template.supplementary_skus.includes(skuType)) {
    return "supplementary";
  }
  return "unmatched";
}

export function defaultOrderFormValues(
  productId: string | undefined,
  buyerOrgId: string | undefined,
  skuId: string | undefined,
  scenarioCode: string | undefined,
): OrderCreateFormValues {
  return {
    buyer_org_id: buyerOrgId ?? "",
    product_id: productId ?? "",
    sku_id: skuId ?? "",
    scenario_code: scenarioCode,
    quantity: 1,
    term_days: 30,
    subscription_cadence: "none",
    confirm_rights: false,
    confirm_snapshot: false,
    confirm_audit: false,
    idempotency_key: createOrderIdempotencyKey("create"),
  };
}

export function buildCreateOrderRequest(
  values: OrderCreateFormValues,
): CreateOrderRequest {
  return {
    buyer_org_id: values.buyer_org_id,
    product_id: values.product_id,
    sku_id: values.sku_id,
    scenario_code: emptyToUndefined(values.scenario_code),
  };
}

export function createOrderIdempotencyKey(action: "create" | "cancel"): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `web-009-order-${action}-${crypto.randomUUID()}`;
  }
  return `web-009-order-${action}-${Date.now()}`;
}

function unwrapEnvelopeData<T>(
  response:
    | {
        data?: T | { data?: T | null } | null;
      }
    | undefined,
) {
  const payload = response?.data;
  if (
    payload &&
    typeof payload === "object" &&
    "data" in payload &&
    (payload as { data?: T | null }).data !== undefined
  ) {
    return (payload as { data?: T | null }).data ?? null;
  }
  return (payload as T | null | undefined) ?? null;
}

export function unwrapCreatedOrder(response: CreateOrderResponse | undefined) {
  return unwrapEnvelopeData<CreatedOrder>(response);
}

export function unwrapOrderDetail(response: OrderDetailResponse | undefined) {
  return unwrapEnvelopeData<OrderDetail>(response);
}

export function unwrapLifecycle(response: OrderLifecycleSnapshotsResponse | undefined) {
  return unwrapEnvelopeData<OrderLifecycleSnapshots>(response);
}

export function unwrapCanceledOrder(response: CancelOrderResponse | undefined) {
  return unwrapEnvelopeData<CanceledOrder>(response);
}

export function readSubjectOrgId(subject: SessionSubject | null | undefined) {
  return subject?.tenant_id ?? subject?.org_id ?? "";
}

export function hasAnyRole(
  roles: readonly string[] | undefined,
  allowed: readonly string[],
) {
  return Boolean(roles?.some((role) => allowed.includes(role)));
}

export function canCreateOrder(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, ORDER_CREATE_ALLOWED_ROLES);
}

export function canReadOrder(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, ORDER_READ_ALLOWED_ROLES);
}

export function canCancelOrder(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, ORDER_CANCEL_ALLOWED_ROLES);
}

export function deliveryRouteForSku(skuType: string, orderId: string) {
  const suffixBySku: Record<string, string> = {
    FILE_STD: "file",
    FILE_SUB: "subscription",
    SHARE_RO: "share",
    API_SUB: "api",
    API_PPU: "api",
    QRY_LITE: "template-query",
    SBX_STD: "sandbox",
    RPT_STD: "report",
  };
  const suffix = suffixBySku[skuType] ?? "file";
  return `/delivery/orders/${orderId}/${suffix}`;
}

export function orderStatusLabel(status: string) {
  const labels: Record<string, string> = {
    created: "已创建",
    contract_pending: "合同待确认",
    contract_effective: "合同已生效",
    buyer_locked: "买方资金已锁定",
    seller_delivering: "卖方交付中",
    delivered: "已交付",
    accepted: "已验收",
    settled: "已结算",
    closed: "已关闭",
    dispute_opened: "争议中",
    payment_failed_pending_resolution: "支付失败待处理",
    payment_timeout_pending_compensation_cancel: "支付超时待补偿/取消",
  };
  return labels[status] ?? status;
}

export function formatTradeError(error: unknown) {
  return formatPlatformErrorForDisplay(error, {
    fallbackCode: "TRD_STATE_CONFLICT",
    fallbackDescription: "请刷新订单详情和生命周期快照，再结合错误码与 request_id 排查状态冲突。",
  });
}

export function formatMoney(amount: string | undefined, currencyCode: string | undefined) {
  return `${amount ?? "0"} ${currencyCode ?? "CNY"}`;
}

export function estimateBuyerDeposit(
  amount: string | undefined,
  currencyCode: string | undefined,
) {
  return formatMoney(amount, currencyCode);
}

export function skuOptionLabel(skuType: string) {
  return (
    standardSkuOptions.find((option) => option.sku_type === skuType)?.label ??
    skuType
  );
}

function emptyToUndefined(value: string | undefined) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}
