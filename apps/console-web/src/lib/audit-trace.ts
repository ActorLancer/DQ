import { normalizePlatformError } from "@datab/sdk-ts";
import type {
  AuditPackageExportRequest,
  AuditPackageExportResponse,
  AuditTraceSearchQuery,
  AuditTraceSearchResponse,
  AuthMeResponse,
  DeveloperTraceQuery,
  DeveloperTraceResponse,
  ExternalFactsQuery,
  OrderAuditResponse,
  ProjectionGapsQuery,
} from "@datab/sdk-ts";
import { z } from "zod";

export type SessionSubject = AuthMeResponse["data"];
export type AuditLookupKey =
  | "order_id"
  | "request_id"
  | "tx_hash"
  | "case_id"
  | "delivery_id";

export const auditLookupLabels: Record<AuditLookupKey, string> = {
  order_id: "订单号 order_id",
  request_id: "请求号 request_id",
  tx_hash: "链交易 tx_hash",
  case_id: "争议案件 case_id",
  delivery_id: "交付记录 delivery_id",
};

export const auditLookupFormSchema = z.object({
  lookup_key: z.enum(["order_id", "request_id", "tx_hash", "case_id", "delivery_id"]),
  lookup_value: z.string().trim().min(3, "请输入正式联查主键"),
  page_size: z.number().int().min(5).max(100),
});

export type AuditLookupFormValues = z.infer<typeof auditLookupFormSchema>;

export const auditPackageExportFormSchema = z
  .object({
    ref_type: z.enum(["order", "case", "dispute_case"]),
    ref_id: z.string().uuid("请输入正式 UUID"),
    reason: z.string().trim().min(8, "导出原因至少 8 个字符").max(500),
    masked_level: z.enum(["summary", "masked", "unmasked"]),
    package_type: z.string().trim().min(3).max(64),
    idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
    step_up_token: z.string().trim().optional(),
    step_up_challenge_id: z.string().trim().optional(),
  })
  .superRefine((value, context) => {
    if (!value.step_up_token && !value.step_up_challenge_id) {
      context.addIssue({
        code: "custom",
        path: ["step_up_token"],
        message: "证据包导出属于高风险动作，必须填写 step-up token 或 challenge id",
      });
    }
  });

export type AuditPackageExportFormValues = z.infer<
  typeof auditPackageExportFormSchema
>;

const traceReaderRoles = [
  "tenant_admin",
  "tenant_audit_readonly",
  "platform_admin",
  "platform_audit_security",
  "platform_reviewer",
  "platform_risk_settlement",
  "regulator_readonly",
];

const developerTraceRoles = ["tenant_developer", "platform_audit_security"];

const tradeMonitorRoles = [
  "tenant_admin",
  "tenant_audit_readonly",
  "platform_admin",
  "platform_audit_security",
  "platform_risk_settlement",
];

const externalFactRoles = [
  "platform_admin",
  "platform_audit_security",
  "platform_risk_settlement",
];

const projectionGapRoles = ["platform_admin", "platform_audit_security"];
const packageExportRoles = ["platform_audit_security"];

export function canReadAuditTrace(subject?: SessionSubject) {
  return hasAnyRole(subject, traceReaderRoles);
}

export function canReadDeveloperTrace(subject?: SessionSubject) {
  return hasAnyRole(subject, developerTraceRoles);
}

export function canReadOpsTradeMonitor(subject?: SessionSubject) {
  return hasAnyRole(subject, tradeMonitorRoles);
}

export function canReadExternalFacts(subject?: SessionSubject) {
  return hasAnyRole(subject, externalFactRoles);
}

export function canReadProjectionGaps(subject?: SessionSubject) {
  return hasAnyRole(subject, projectionGapRoles);
}

export function canExportAuditPackage(subject?: SessionSubject) {
  return hasAnyRole(subject, packageExportRoles);
}

export function hasAnyRole(subject: SessionSubject | undefined, roles: string[]) {
  const currentRoles = new Set(subject?.roles ?? []);
  return roles.some((role) => currentRoles.has(role));
}

export function subjectDisplayName(subject?: SessionSubject) {
  return subject?.display_name || subject?.login_id || subject?.user_id || "未登录主体";
}

export function buildAuditTraceQuery(
  values: AuditLookupFormValues,
): AuditTraceSearchQuery | null {
  const base = {
    page: 1,
    page_size: values.page_size,
  };
  switch (values.lookup_key) {
    case "order_id":
      return { ...base, order_id: values.lookup_value };
    case "request_id":
      return { ...base, request_id: values.lookup_value };
    case "case_id":
      return { ...base, ref_type: "dispute_case", ref_id: values.lookup_value };
    case "delivery_id":
      return { ...base, ref_type: "delivery", ref_id: values.lookup_value };
    case "tx_hash":
      return null;
  }
}

export function buildDeveloperTraceQuery(
  values: AuditLookupFormValues,
): DeveloperTraceQuery | null {
  if (values.lookup_key === "order_id") {
    return { order_id: values.lookup_value };
  }
  if (values.lookup_key === "tx_hash") {
    return { tx_hash: values.lookup_value };
  }
  return null;
}

export function buildExternalFactsQuery(orderId: string): ExternalFactsQuery {
  return {
    order_id: orderId,
    page: 1,
    page_size: 20,
  };
}

export function buildProjectionGapsQuery(orderId: string): ProjectionGapsQuery {
  return {
    order_id: orderId,
    page: 1,
    page_size: 20,
  };
}

export function buildPackageExportPayload(
  values: AuditPackageExportFormValues,
): AuditPackageExportRequest {
  return {
    ref_type: values.ref_type,
    ref_id: values.ref_id,
    reason: values.reason.trim(),
    masked_level: values.masked_level,
    package_type: values.package_type.trim() || "forensic_export",
  };
}

export function createAuditIdempotencyKey(scope = "audit-export") {
  return `web-014:${scope}:${globalThis.crypto.randomUUID()}`;
}

export type AuditTraceRow =
  | AuditTraceSearchResponse["data"]["items"][number]
  | OrderAuditResponse["data"]["traces"][number]
  | DeveloperTraceResponse["data"]["recent_audit_traces"][number];

export interface UnifiedAuditEventRow {
  key: string;
  audit_id?: string | null;
  domain_name: string;
  event_class?: string | null;
  ref_type: string;
  ref_id?: string | null;
  action_name: string;
  result_code: string;
  error_code?: string | null;
  request_id?: string | null;
  trace_id?: string | null;
  tx_hash?: string | null;
  evidence_manifest_id?: string | null;
  event_hash?: string | null;
  occurred_at?: string | null;
  group: AuditEventGroup;
}

export type AuditEventGroup =
  | "order"
  | "billing"
  | "delivery"
  | "dispute"
  | "evidence"
  | "chain"
  | "other";

export function normalizeAuditEvents(...sources: Array<AuditTraceRow[] | undefined>) {
  const seen = new Set<string>();
  const rows: UnifiedAuditEventRow[] = [];

  for (const source of sources) {
    for (const item of source ?? []) {
      const row = normalizeAuditEvent(item);
      if (seen.has(row.key)) {
        continue;
      }
      seen.add(row.key);
      rows.push(row);
    }
  }

  return rows.sort((left, right) =>
    String(right.occurred_at ?? "").localeCompare(String(left.occurred_at ?? "")),
  );
}

export function normalizeAuditEvent(item: AuditTraceRow): UnifiedAuditEventRow {
  const row = {
    audit_id: item.audit_id ?? null,
    domain_name: item.domain_name ?? "unknown",
    event_class: item.event_class ?? null,
    ref_type: item.ref_type ?? "unknown",
    ref_id: item.ref_id ?? null,
    action_name: item.action_name ?? "unknown",
    result_code: item.result_code ?? "unknown",
    error_code: item.error_code ?? null,
    request_id: item.request_id ?? null,
    trace_id: item.trace_id ?? null,
    tx_hash: item.tx_hash ?? null,
    evidence_manifest_id: item.evidence_manifest_id ?? null,
    event_hash: item.event_hash ?? null,
    occurred_at: item.occurred_at ?? null,
  };
  const key =
    row.audit_id ||
    row.event_hash ||
    [
      row.domain_name,
      row.ref_type,
      row.ref_id,
      row.action_name,
      row.request_id,
      row.trace_id,
      row.occurred_at,
    ].join(":");

  return {
    ...row,
    key,
    group: classifyAuditEvent(row),
  };
}

export function classifyAuditEvent(
  row: Pick<UnifiedAuditEventRow, "domain_name" | "action_name" | "ref_type">,
): AuditEventGroup {
  const text = `${row.domain_name} ${row.action_name} ${row.ref_type}`.toLowerCase();
  if (text.includes("billing") || text.includes("payment") || text.includes("refund") || text.includes("settlement")) {
    return "billing";
  }
  if (text.includes("delivery") || text.includes("acceptance") || text.includes("download")) {
    return "delivery";
  }
  if (text.includes("dispute") || text.includes("case")) {
    return "dispute";
  }
  if (text.includes("evidence") || text.includes("manifest") || text.includes("package")) {
    return "evidence";
  }
  if (text.includes("chain") || text.includes("anchor") || text.includes("proof")) {
    return "chain";
  }
  if (text.includes("order") || text.includes("trade")) {
    return "order";
  }
  return "other";
}

export function summarizeAuditGroups(rows: UnifiedAuditEventRow[]) {
  return rows.reduce<Record<AuditEventGroup, number>>(
    (acc, row) => {
      acc[row.group] += 1;
      return acc;
    },
    {
      order: 0,
      billing: 0,
      delivery: 0,
      dispute: 0,
      evidence: 0,
      chain: 0,
      other: 0,
    },
  );
}

export function resolveOrderIdFromLookup(
  values: AuditLookupFormValues | null,
  developerTrace?: DeveloperTraceResponse,
  orderAudit?: OrderAuditResponse,
  traceRows: UnifiedAuditEventRow[] = [],
) {
  if (values?.lookup_key === "order_id") {
    return values.lookup_value;
  }
  const developerOrderId = developerTrace?.data.subject.resolved_order_id;
  if (developerOrderId) {
    return developerOrderId;
  }
  const orderAuditId = orderAudit?.data.order_id;
  if (orderAuditId) {
    return orderAuditId;
  }
  return traceRows.find((row) => row.ref_type === "order" && row.ref_id)?.ref_id ?? null;
}

export function safePackageExportView(response?: AuditPackageExportResponse) {
  if (!response?.data) {
    return null;
  }
  const { evidence_package, evidence_manifest, ...rest } = response.data;
  const safePackage = { ...evidence_package };
  const safeManifest = { ...evidence_manifest };
  delete safePackage.storage_uri;
  delete safeManifest.storage_uri;

  return {
    ...rest,
    evidence_package: safePackage,
    evidence_manifest: safeManifest,
    hidden_fields: ["evidence_package.storage_uri", "evidence_manifest.storage_uri"],
  };
}

export function formatAuditError(error: unknown) {
  const normalized = normalizePlatformError(error, {
    fallbackCode: "INTERNAL_ERROR",
    fallbackDescription: "请结合错误码和 request_id 回查审计 trace、evidence package 和 ops 联查结果。",
  });

  return {
    title: `${normalized.title} · ${normalized.code}`,
    message: normalized.description,
    requestId: normalized.requestId ?? "未返回",
  };
}

export function formatAuditDate(value?: string | null) {
  if (!value) {
    return "未返回";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("zh-CN", {
    dateStyle: "medium",
    timeStyle: "medium",
    hour12: false,
  }).format(date);
}
