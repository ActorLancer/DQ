import type {
  ApiUsageLogResponse,
  AuthMeResponse,
  CommitOrderDeliveryRequest,
  CommitOrderDeliveryResponse,
  DownloadTicketResponse,
  ExecuteTemplateRunRequest,
  ExecuteTemplateRunResponse,
  ManageRevisionSubscriptionRequest,
  ManageRevisionSubscriptionResponse,
  ManageSandboxWorkspaceRequest,
  ManageSandboxWorkspaceResponse,
  ManageShareGrantRequest,
  ManageShareGrantResponse,
  ManageTemplateGrantRequest,
  ManageTemplateGrantResponse,
  OrderDetailResponse,
  OrderLifecycleSnapshotsResponse,
  QueryRunsResponse,
  RevisionSubscriptionResponse,
  ShareGrantListResponse,
} from "@datab/sdk-ts";
import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";
import { z } from "zod";

import { standardSkuOptions, type StandardSkuType } from "./seller-products-view";

export type SessionSubject = AuthMeResponse["data"];
export type OrderDetail = NonNullable<OrderDetailResponse["data"]>;
export type OrderLifecycleSnapshots =
  NonNullable<OrderLifecycleSnapshotsResponse["data"]>;
export type CommitDeliveryResult =
  NonNullable<CommitOrderDeliveryResponse["data"]>;
export type DownloadTicket =
  NonNullable<DownloadTicketResponse["data"]>;
export type RevisionSubscription =
  NonNullable<RevisionSubscriptionResponse["data"]>;
export type RevisionSubscriptionMutationResult =
  NonNullable<ManageRevisionSubscriptionResponse["data"]>;
export type ShareGrantList = NonNullable<ShareGrantListResponse["data"]>;
export type ShareGrantResult =
  NonNullable<ManageShareGrantResponse["data"]>;
export type TemplateGrantResult =
  NonNullable<ManageTemplateGrantResponse["data"]>;
export type TemplateRunResult =
  NonNullable<ExecuteTemplateRunResponse["data"]>;
export type SandboxWorkspaceResult =
  NonNullable<ManageSandboxWorkspaceResponse["data"]>;
export type ApiUsageLog = NonNullable<ApiUsageLogResponse["data"]>;
export type QueryRuns = NonNullable<QueryRunsResponse["data"]>;

export type DeliveryRouteKind =
  | "file"
  | "api"
  | "share"
  | "subscription"
  | "template-query"
  | "sandbox"
  | "report";

export type DeliveryEntry = {
  kind: DeliveryRouteKind;
  routeKey:
    | "delivery_file"
    | "delivery_api"
    | "delivery_share"
    | "delivery_subscription"
    | "delivery_template_query"
    | "delivery_sandbox"
    | "delivery_report";
  title: string;
  shortTitle: string;
  pathSuffix: string;
  supportedSkus: StandardSkuType[];
  primaryPermissions: string[];
  apiBindings: string[];
  description: string;
  auditAction: string;
};

export const DELIVERY_ENTRIES: DeliveryEntry[] = [
  {
    kind: "file",
    routeKey: "delivery_file",
    title: "文件交付",
    shortTitle: "文件",
    pathSuffix: "file",
    supportedSkus: ["FILE_STD"],
    primaryPermissions: ["delivery.file.commit", "delivery.file.download"],
    apiBindings: [
      "POST /api/v1/orders/{id}/deliver",
      "GET /api/v1/orders/{id}/download-ticket",
    ],
    description: "托管文件对象、密钥信封、下载票据、Hash 校验与回执。",
    auditAction: "delivery.file.commit",
  },
  {
    kind: "subscription",
    routeKey: "delivery_subscription",
    title: "文件版本订阅",
    shortTitle: "订阅",
    pathSuffix: "subscription",
    supportedSkus: ["FILE_SUB"],
    primaryPermissions: [
      "delivery.subscription.manage",
      "delivery.subscription.read",
    ],
    apiBindings: [
      "POST /api/v1/orders/{id}/subscriptions",
      "GET /api/v1/orders/{id}/subscriptions",
    ],
    description: "FILE_SUB 版本订阅、交付周期、起始版本与更新轨迹。",
    auditAction: "delivery.subscription.manage",
  },
  {
    kind: "share",
    routeKey: "delivery_share",
    title: "只读共享开通",
    shortTitle: "共享",
    pathSuffix: "share",
    supportedSkus: ["SHARE_RO"],
    primaryPermissions: ["delivery.share.enable", "delivery.share.read"],
    apiBindings: [
      "POST /api/v1/orders/{id}/share-grants",
      "GET /api/v1/orders/{id}/share-grants",
    ],
    description: "共享协议、接收方、授权范围、有效期与撤权记录。",
    auditAction: "delivery.share.enable",
  },
  {
    kind: "api",
    routeKey: "delivery_api",
    title: "API 开通",
    shortTitle: "API",
    pathSuffix: "api",
    supportedSkus: ["API_SUB", "API_PPU"],
    primaryPermissions: ["delivery.api.enable"],
    apiBindings: [
      "POST /api/v1/orders/{id}/deliver",
      "GET /api/v1/orders/{id}/usage-log",
    ],
    description: "应用绑定、凭证签发、额度、限流和调用日志摘要。",
    auditAction: "delivery.api.enable",
  },
  {
    kind: "template-query",
    routeKey: "delivery_template_query",
    title: "模板查询授权",
    shortTitle: "模板查询",
    pathSuffix: "template-query",
    supportedSkus: ["QRY_LITE"],
    primaryPermissions: [
      "delivery.template_query.enable",
      "delivery.template_query.use",
    ],
    apiBindings: [
      "POST /api/v1/orders/{id}/template-grants",
      "POST /api/v1/orders/{id}/template-runs",
      "GET /api/v1/orders/{id}/template-runs",
    ],
    description: "白名单模板、参数边界、输出边界、额度与结果入口。",
    auditAction: "delivery.template_query.enable",
  },
  {
    kind: "sandbox",
    routeKey: "delivery_sandbox",
    title: "查询沙箱开通",
    shortTitle: "沙箱",
    pathSuffix: "sandbox",
    supportedSkus: ["SBX_STD"],
    primaryPermissions: ["delivery.sandbox.enable"],
    apiBindings: ["POST /api/v1/orders/{id}/sandbox-workspaces"],
    description: "受控工作区、席位、会话有效期、隔离与导出控制。",
    auditAction: "delivery.sandbox.enable",
  },
  {
    kind: "report",
    routeKey: "delivery_report",
    title: "报告/结果产品交付",
    shortTitle: "报告",
    pathSuffix: "report",
    supportedSkus: ["RPT_STD"],
    primaryPermissions: ["delivery.report.commit"],
    apiBindings: ["POST /api/v1/orders/{id}/deliver"],
    description: "报告包、结果产品、版本、Hash 与验收入口。",
    auditAction: "delivery.report.commit",
  },
];

const uuidLiteralSchema = z
  .string()
  .trim()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    "请输入 PostgreSQL UUID 字面量",
  );

export const DELIVERY_READ_ALLOWED_ROLES = [
  "buyer_operator",
  "seller_operator",
  "tenant_admin",
  "tenant_developer",
  "tenant_audit_readonly",
  "platform_admin",
  "platform_audit_security",
  "platform_risk_settlement",
] as const;

export const TEMPLATE_QUERY_GRANT_ALLOWED_ROLES = [
  "seller_operator",
  "tenant_admin",
  "platform_admin",
] as const;

export const TEMPLATE_QUERY_RUN_ALLOWED_ROLES = [
  "buyer_operator",
  "tenant_developer",
  "tenant_admin",
  "platform_admin",
] as const;

export const DELIVERY_ACTION_ROLES: Record<DeliveryRouteKind, readonly string[]> = {
  file: ["seller_operator", "tenant_admin", "platform_admin"],
  report: ["seller_operator", "tenant_admin", "platform_admin"],
  api: [
    "seller_operator",
    "buyer_operator",
    "tenant_developer",
    "tenant_admin",
    "platform_admin",
  ],
  share: ["seller_operator", "tenant_admin", "platform_admin"],
  subscription: ["seller_operator", "tenant_admin", "platform_admin"],
  "template-query": [
    ...TEMPLATE_QUERY_GRANT_ALLOWED_ROLES,
    ...TEMPLATE_QUERY_RUN_ALLOWED_ROLES,
  ],
  sandbox: [
    "buyer_operator",
    "tenant_developer",
    "tenant_admin",
    "platform_admin",
  ],
};

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

const requiredJsonRecordSchema = z
  .string()
  .trim()
  .min(2, "必须填写 JSON object")
  .refine((value) => {
    try {
      const parsed = JSON.parse(value) as unknown;
      return Boolean(parsed) && typeof parsed === "object" && !Array.isArray(parsed);
    } catch {
      return false;
    }
  }, {
    message: "必须是可解析的 JSON object",
  });

const positiveIntString = z
  .string()
  .trim()
  .optional()
  .refine((value) => !value || /^\d+$/.test(value), {
    message: "必须是正整数",
  });

const confirmActionSchema = {
  confirm_scope: z.boolean().refine(Boolean, {
    message: "必须确认授权/交付边界来自订单与 SKU 快照",
  }),
  confirm_audit: z.boolean().refine(Boolean, {
    message: "必须确认关键动作会写入审计留痕",
  }),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
};

export const commitDeliveryFormSchema = z.object({
  object_uri: z.string().trim().optional(),
  content_type: z.string().trim().optional(),
  size_bytes: positiveIntString,
  content_hash: z.string().trim().optional(),
  encryption_algo: z.string().trim().optional(),
  key_cipher: z.string().trim().optional(),
  key_control_mode: z.string().trim().optional(),
  expire_at: z.string().trim().optional(),
  download_limit: positiveIntString,
  delivery_commit_hash: z.string().trim().min(8, "delivery_commit_hash 必填"),
  receipt_hash: z.string().trim().min(8, "receipt_hash 必填"),
  report_type: z.string().trim().optional(),
  app_id: z.string().trim().optional(),
  app_name: z.string().trim().optional(),
  app_type: z.string().trim().optional(),
  client_id: z.string().trim().optional(),
  quota_json: jsonRecordSchema,
  rate_limit_json: jsonRecordSchema,
  upstream_mode: z.string().trim().optional(),
  metadata_json: jsonRecordSchema,
  ...confirmActionSchema,
});

export const revisionSubscriptionFormSchema = z.object({
  cadence: z.enum(["weekly", "monthly", "quarterly", "yearly"]),
  delivery_channel: z.enum(["file_ticket"]),
  start_version_no: positiveIntString,
  last_delivered_version_no: positiveIntString,
  next_delivery_at: z.string().trim().optional(),
  metadata_json: jsonRecordSchema,
  ...confirmActionSchema,
});

export const shareGrantFormSchema = z.object({
  operation: z.enum(["grant", "revoke"]),
  asset_object_id: z.string().trim().optional(),
  recipient_ref: z.string().trim().min(1, "recipient_ref 必填"),
  subscriber_ref: z.string().trim().optional(),
  share_protocol: z.string().trim().min(1, "share_protocol 必填"),
  access_locator: z.string().trim().optional(),
  scope_json: jsonRecordSchema,
  expires_at: z.string().trim().optional(),
  receipt_hash: z.string().trim().min(8, "receipt_hash 必填"),
  metadata_json: jsonRecordSchema,
  ...confirmActionSchema,
});

export const templateGrantFormSchema = z.object({
  template_query_grant_id: z.string().trim().optional(),
  query_surface_id: uuidLiteralSchema,
  asset_object_id: z.string().trim().optional(),
  environment_id: z.string().trim().optional(),
  template_type: z.string().trim().optional(),
  allowed_template_ids: z.string().trim().min(1, "至少填写一个模板 UUID"),
  execution_rule_snapshot: jsonRecordSchema,
  output_boundary_json: jsonRecordSchema,
  run_quota_json: jsonRecordSchema,
  ...confirmActionSchema,
});

export const templateRunFormSchema = z.object({
  template_query_grant_id: z.string().trim().optional(),
  query_template_id: uuidLiteralSchema,
  requester_user_id: z.string().trim().optional(),
  request_payload_json: requiredJsonRecordSchema,
  output_boundary_json: jsonRecordSchema,
  masked_level: z.enum(["masked", "summary", "restricted"]).optional(),
  export_scope: z.enum(["none", "summary", "restricted_object"]).optional(),
  approval_ticket_id: z.string().trim().optional(),
  execution_metadata_json: jsonRecordSchema,
  ...confirmActionSchema,
});

export const sandboxWorkspaceFormSchema = z.object({
  query_surface_id: uuidLiteralSchema,
  workspace_name: z.string().trim().min(2, "workspace_name 至少 2 个字符"),
  seat_user_id: z.string().trim().optional(),
  expire_at: z.string().trim().optional(),
  export_policy_json: jsonRecordSchema,
  clean_room_mode: z.string().trim().optional(),
  data_residency_mode: z.string().trim().optional(),
  ...confirmActionSchema,
});

export type CommitDeliveryFormValues = z.infer<
  typeof commitDeliveryFormSchema
>;
export type RevisionSubscriptionFormValues = z.infer<
  typeof revisionSubscriptionFormSchema
>;
export type ShareGrantFormValues = z.infer<typeof shareGrantFormSchema>;
export type TemplateGrantFormValues = z.infer<typeof templateGrantFormSchema>;
export type TemplateRunFormValues = z.infer<typeof templateRunFormSchema>;
export type SandboxWorkspaceFormValues = z.infer<
  typeof sandboxWorkspaceFormSchema
>;

export function buildCommitDeliveryRequest(
  kind: "file" | "api" | "report",
  values: CommitDeliveryFormValues,
): CommitOrderDeliveryRequest {
  const branch = kind === "report" ? "report" : kind;
  return compactObject({
    branch,
    object_uri: emptyToUndefined(values.object_uri),
    content_type: emptyToUndefined(values.content_type),
    size_bytes: parseOptionalInt(values.size_bytes),
    content_hash: emptyToUndefined(values.content_hash),
    encryption_algo: emptyToUndefined(values.encryption_algo),
    key_cipher: emptyToUndefined(values.key_cipher),
    key_control_mode: emptyToUndefined(values.key_control_mode),
    expire_at: emptyToUndefined(values.expire_at),
    download_limit: parseOptionalInt(values.download_limit),
    delivery_commit_hash: values.delivery_commit_hash.trim(),
    receipt_hash: values.receipt_hash.trim(),
    report_type: emptyToUndefined(values.report_type),
    app_id: emptyToUndefined(values.app_id),
    app_name: emptyToUndefined(values.app_name),
    app_type: emptyToUndefined(values.app_type),
    client_id: emptyToUndefined(values.client_id),
    quota_json: parseJsonRecord(values.quota_json),
    rate_limit_json: parseJsonRecord(values.rate_limit_json),
    upstream_mode: emptyToUndefined(values.upstream_mode),
    metadata: compactObject({
      ...parseJsonRecord(values.metadata_json),
      web_task_id: "WEB-010",
      delivery_entry: kind,
    }),
  }) as CommitOrderDeliveryRequest;
}

export function buildRevisionSubscriptionRequest(
  values: RevisionSubscriptionFormValues,
): ManageRevisionSubscriptionRequest {
  return compactObject({
    cadence: values.cadence,
    delivery_channel: values.delivery_channel,
    start_version_no: parseOptionalInt(values.start_version_no),
    last_delivered_version_no: parseOptionalInt(values.last_delivered_version_no),
    next_delivery_at: emptyToUndefined(values.next_delivery_at),
    metadata: compactObject({
      ...parseJsonRecord(values.metadata_json),
      web_task_id: "WEB-010",
    }),
  }) as ManageRevisionSubscriptionRequest;
}

export function buildShareGrantRequest(
  values: ShareGrantFormValues,
): ManageShareGrantRequest {
  return compactObject({
    operation: values.operation,
    asset_object_id: emptyToUndefined(values.asset_object_id),
    recipient_ref: values.recipient_ref.trim(),
    subscriber_ref: emptyToUndefined(values.subscriber_ref),
    share_protocol: values.share_protocol.trim(),
    access_locator: emptyToUndefined(values.access_locator),
    scope_json: parseJsonRecord(values.scope_json),
    expires_at: emptyToUndefined(values.expires_at),
    receipt_hash: values.receipt_hash.trim(),
    metadata: compactObject({
      ...parseJsonRecord(values.metadata_json),
      web_task_id: "WEB-010",
    }),
  }) as ManageShareGrantRequest;
}

export function buildTemplateGrantRequest(
  values: TemplateGrantFormValues,
): ManageTemplateGrantRequest {
  return compactObject({
    template_query_grant_id: emptyToUndefined(values.template_query_grant_id),
    query_surface_id: values.query_surface_id.trim(),
    asset_object_id: emptyToUndefined(values.asset_object_id),
    environment_id: emptyToUndefined(values.environment_id),
    template_type: emptyToUndefined(values.template_type),
    allowed_template_ids: values.allowed_template_ids
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean),
    execution_rule_snapshot: parseJsonRecord(values.execution_rule_snapshot),
    output_boundary_json: parseJsonRecord(values.output_boundary_json),
    run_quota_json: parseJsonRecord(values.run_quota_json),
  }) as ManageTemplateGrantRequest;
}

export function buildTemplateRunRequest(
  values: TemplateRunFormValues,
): ExecuteTemplateRunRequest {
  return compactObject({
    template_query_grant_id: emptyToUndefined(values.template_query_grant_id),
    query_template_id: values.query_template_id.trim(),
    requester_user_id: emptyToUndefined(values.requester_user_id),
    request_payload_json: parseJsonRecord(values.request_payload_json) ?? {},
    output_boundary_json: parseJsonRecord(values.output_boundary_json),
    masked_level: values.masked_level,
    export_scope: values.export_scope,
    approval_ticket_id: emptyToUndefined(values.approval_ticket_id),
    execution_metadata_json: parseJsonRecord(values.execution_metadata_json),
  }) as ExecuteTemplateRunRequest;
}

export function buildSandboxWorkspaceRequest(
  values: SandboxWorkspaceFormValues,
): ManageSandboxWorkspaceRequest {
  return compactObject({
    query_surface_id: values.query_surface_id.trim(),
    workspace_name: values.workspace_name.trim(),
    seat_user_id: emptyToUndefined(values.seat_user_id),
    expire_at: emptyToUndefined(values.expire_at),
    export_policy_json: parseJsonRecord(values.export_policy_json),
    clean_room_mode: emptyToUndefined(values.clean_room_mode),
    data_residency_mode: emptyToUndefined(values.data_residency_mode),
  }) as ManageSandboxWorkspaceRequest;
}

export function defaultCommitDeliveryValues(
  kind: "file" | "api" | "report",
): CommitDeliveryFormValues {
  const now = Date.now();
  return {
    object_uri: kind === "api" ? "" : `s3://delivery-controlled/web-010-${kind}.bin`,
    content_type:
      kind === "report"
        ? "application/pdf"
        : kind === "file"
          ? "text/csv"
          : "",
    size_bytes: kind === "api" ? "" : "1024",
    content_hash: kind === "api" ? "" : `sha256:web010-${kind}-content`,
    encryption_algo: kind === "api" ? "" : "AES-256-GCM",
    key_cipher: kind === "api" ? "" : "kms-envelope",
    key_control_mode: kind === "api" ? "" : "platform_envelope",
    expire_at: new Date(now + 7 * 24 * 60 * 60 * 1000).toISOString(),
    download_limit: kind === "api" ? "" : "3",
    delivery_commit_hash: `sha256:web010-${kind}-commit`,
    receipt_hash: `sha256:web010-${kind}-receipt`,
    report_type: kind === "report" ? "standard_result_report" : "",
    app_id: "",
    app_name: kind === "api" ? "WEB-010 Buyer Application" : "",
    app_type: kind === "api" ? "buyer_client" : "",
    client_id: "",
    quota_json: kind === "api" ? jsonInputValue({ monthly_calls: 10000 }) : "",
    rate_limit_json: kind === "api" ? jsonInputValue({ rpm: 60 }) : "",
    upstream_mode: kind === "api" ? "managed_gateway" : "",
    metadata_json: jsonInputValue({ source: "portal-web" }),
    confirm_scope: false,
    confirm_audit: false,
    idempotency_key: createDeliveryIdempotencyKey(kind),
  };
}

export function defaultRevisionSubscriptionValues(): RevisionSubscriptionFormValues {
  return {
    cadence: "monthly",
    delivery_channel: "file_ticket",
    start_version_no: "1",
    last_delivered_version_no: "",
    next_delivery_at: "",
    metadata_json: jsonInputValue({ source: "portal-web" }),
    confirm_scope: false,
    confirm_audit: false,
    idempotency_key: createDeliveryIdempotencyKey("subscription"),
  };
}

export function defaultShareGrantValues(): ShareGrantFormValues {
  return {
    operation: "grant",
    asset_object_id: "",
    recipient_ref: "buyer.demo",
    subscriber_ref: "",
    share_protocol: "presigned_read",
    access_locator: "share://controlled/web-010",
    scope_json: jsonInputValue({ read_only: true, exportable: false }),
    expires_at: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(),
    receipt_hash: "sha256:web010-share-receipt",
    metadata_json: jsonInputValue({ source: "portal-web" }),
    confirm_scope: false,
    confirm_audit: false,
    idempotency_key: createDeliveryIdempotencyKey("share"),
  };
}

export function defaultTemplateGrantValues(): TemplateGrantFormValues {
  return {
    template_query_grant_id: "",
    query_surface_id: "",
    asset_object_id: "",
    environment_id: "",
    template_type: "sql_template",
    allowed_template_ids: "",
    execution_rule_snapshot: jsonInputValue({ allowed_parameters: "schema-bound" }),
    output_boundary_json: jsonInputValue({ exportable: false, masking: "standard" }),
    run_quota_json: jsonInputValue({ daily_runs: 20 }),
    confirm_scope: false,
    confirm_audit: false,
    idempotency_key: createDeliveryIdempotencyKey("template-query"),
  };
}

export function defaultTemplateRunValues(): TemplateRunFormValues {
  return {
    template_query_grant_id: "",
    query_template_id: "",
    requester_user_id: "",
    request_payload_json: jsonInputValue({
      city: "Shanghai",
      radius_km: 3,
      limit: 2,
    }),
    output_boundary_json: jsonInputValue({
      selected_format: "json",
      allowed_formats: ["json"],
      max_rows: 2,
      max_cells: 6,
    }),
    masked_level: "masked",
    export_scope: "none",
    approval_ticket_id: "",
    execution_metadata_json: jsonInputValue({ source: "portal-web", entrypoint: "template-query" }),
    confirm_scope: false,
    confirm_audit: false,
    idempotency_key: createDeliveryIdempotencyKey("template-run"),
  };
}

export function defaultSandboxWorkspaceValues(): SandboxWorkspaceFormValues {
  return {
    query_surface_id: "",
    workspace_name: "WEB-010 Sandbox",
    seat_user_id: "",
    expire_at: new Date(Date.now() + 14 * 24 * 60 * 60 * 1000).toISOString(),
    export_policy_json: jsonInputValue({ exportable: false, review_required: true }),
    clean_room_mode: "lite",
    data_residency_mode: "seller_self_hosted",
    confirm_scope: false,
    confirm_audit: false,
    idempotency_key: createDeliveryIdempotencyKey("sandbox"),
  };
}

export function deliveryEntryByKind(kind: DeliveryRouteKind) {
  return DELIVERY_ENTRIES.find((entry) => entry.kind === kind) ?? DELIVERY_ENTRIES[0];
}

export function deliveryEntryForSku(skuType: string | undefined) {
  return (
    DELIVERY_ENTRIES.find((entry) =>
      entry.supportedSkus.includes(skuType as StandardSkuType),
    ) ?? null
  );
}

export function deliveryRouteForEntry(entry: DeliveryEntry, orderId: string) {
  return `/delivery/orders/${orderId}/${entry.pathSuffix}`;
}

export function getOrderSkuType(order: OrderDetail | null | undefined) {
  return order?.price_snapshot?.sku_type ?? "";
}

export function hasAnyRole(
  roles: readonly string[] | undefined,
  allowed: readonly string[],
) {
  return Boolean(roles?.some((role) => allowed.includes(role)));
}

export function canReadDelivery(subject: SessionSubject | null | undefined) {
  return hasAnyRole(subject?.roles, DELIVERY_READ_ALLOWED_ROLES);
}

export function canOperateDelivery(
  subject: SessionSubject | null | undefined,
  kind: DeliveryRouteKind,
) {
  return hasAnyRole(subject?.roles, DELIVERY_ACTION_ROLES[kind]);
}

export function canManageTemplateQueryGrant(
  subject: SessionSubject | null | undefined,
) {
  return hasAnyRole(subject?.roles, TEMPLATE_QUERY_GRANT_ALLOWED_ROLES);
}

export function canExecuteTemplateQueryRun(
  subject: SessionSubject | null | undefined,
) {
  return hasAnyRole(subject?.roles, TEMPLATE_QUERY_RUN_ALLOWED_ROLES);
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

export function createDeliveryIdempotencyKey(kind: string): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `web-010-delivery-${kind}-${crypto.randomUUID()}`;
  }
  return `web-010-delivery-${kind}-${Date.now()}`;
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

export function unwrapCommitDelivery(
  response: CommitOrderDeliveryResponse | undefined,
) {
  return unwrapEnvelopeData<CommitDeliveryResult>(response);
}

export function unwrapDownloadTicket(response: DownloadTicketResponse | undefined) {
  return unwrapEnvelopeData<DownloadTicket>(response);
}

export function unwrapRevisionSubscription(
  response: RevisionSubscriptionResponse | undefined,
) {
  return unwrapEnvelopeData<RevisionSubscription>(response);
}

export function unwrapRevisionSubscriptionMutation(
  response: ManageRevisionSubscriptionResponse | undefined,
) {
  return unwrapEnvelopeData<RevisionSubscriptionMutationResult>(response);
}

export function unwrapShareGrantList(response: ShareGrantListResponse | undefined) {
  return unwrapEnvelopeData<ShareGrantList>(response);
}

export function unwrapShareGrant(response: ManageShareGrantResponse | undefined) {
  return unwrapEnvelopeData<ShareGrantResult>(response);
}

export function unwrapTemplateGrant(
  response: ManageTemplateGrantResponse | undefined,
) {
  return unwrapEnvelopeData<TemplateGrantResult>(response);
}

export function unwrapTemplateRun(response: ExecuteTemplateRunResponse | undefined) {
  return unwrapEnvelopeData<TemplateRunResult>(response);
}

export function unwrapSandboxWorkspace(
  response: ManageSandboxWorkspaceResponse | undefined,
) {
  return unwrapEnvelopeData<SandboxWorkspaceResult>(response);
}

export function unwrapApiUsageLog(response: ApiUsageLogResponse | undefined) {
  return unwrapEnvelopeData<ApiUsageLog>(response);
}

export function unwrapQueryRuns(response: QueryRunsResponse | undefined) {
  return unwrapEnvelopeData<QueryRuns>(response);
}

export function formatDeliveryError(error: unknown) {
  return formatPlatformErrorForDisplay(error, {
    fallbackCode: "DELIVERY_STATUS_INVALID",
    fallbackDescription: "请刷新订单详情和生命周期状态，再决定是否继续交付或验收。",
  });
}

export function maskSecret(value: string | null | undefined) {
  if (!value) {
    return "未返回";
  }
  if (value.length <= 10) {
    return `${value.slice(0, 2)}***`;
  }
  return `${value.slice(0, 6)}...${value.slice(-4)}`;
}

export function jsonInputValue(value: unknown) {
  return JSON.stringify(value ?? {}, null, 2);
}

function parseOptionalInt(value: string | undefined) {
  const trimmed = value?.trim();
  return trimmed ? Number.parseInt(trimmed, 10) : undefined;
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
