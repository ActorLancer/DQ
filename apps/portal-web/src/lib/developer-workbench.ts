import { PlatformApiError } from "@datab/sdk-ts";
import type {
  AuthMeResponse,
  CreateApplicationRequest,
  DeveloperTraceQuery,
  MockPaymentSimulationRequest,
  PatchApplicationRequest,
  RotateApplicationSecretRequest,
} from "@datab/sdk-ts";
import { z } from "zod";

export type SessionSubject = AuthMeResponse["data"];

export const applicationStatuses = ["active", "suspended", "revoked"] as const;
export const applicationTypes = ["api_client", "webhook_client", "batch_job"] as const;
export const traceLookupModes = ["order_id", "event_id", "tx_hash"] as const;
export const mockPaymentScenarios = ["success", "fail", "timeout"] as const;

export const applicationCreateSchema = z.object({
  org_id: z.string().uuid("请输入正式租户 / 组织 UUID"),
  app_name: z.string().trim().min(2, "应用名称至少 2 个字符").max(80),
  app_type: z.enum(applicationTypes),
  client_id: z.string().trim().min(3, "client_id 至少 3 个字符").max(96),
  client_secret_hash: z.string().trim().optional(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
});

export const applicationPatchSchema = z.object({
  app_id: z.string().uuid("请选择正式 app_id"),
  app_name: z.string().trim().min(2, "应用名称至少 2 个字符").max(80).optional(),
  status: z.enum(applicationStatuses),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
});

export const applicationSecretSchema = z.object({
  app_id: z.string().uuid("请选择正式 app_id"),
  client_secret_hash: z.string().trim().optional(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
});

export const developerTraceSchema = z.object({
  lookup_mode: z.enum(traceLookupModes),
  lookup_value: z.string().trim().min(6, "请输入正式 order_id / event_id / tx_hash"),
});

export const mockPaymentSchema = z.object({
  payment_intent_id: z.string().uuid("请输入正式 payment_intent_id"),
  scenario: z.enum(mockPaymentScenarios),
  delay_seconds: z.number().int().min(0).max(30),
  duplicate_webhook: z.boolean(),
  partial_refund_amount: z.string().trim().optional(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
});

export type ApplicationCreateFormValues = z.infer<typeof applicationCreateSchema>;
export type ApplicationPatchFormValues = z.infer<typeof applicationPatchSchema>;
export type ApplicationSecretFormValues = z.infer<typeof applicationSecretSchema>;
export type DeveloperTraceFormValues = z.infer<typeof developerTraceSchema>;
export type MockPaymentFormValues = z.infer<typeof mockPaymentSchema>;

const developerReadRoles = [
  "tenant_developer",
  "tenant_admin",
  "platform_admin",
  "platform_audit_security",
];
const developerWriteRoles = ["tenant_developer", "tenant_admin", "platform_admin"];
const mockPaymentRoles = [
  "tenant_developer",
  "tenant_admin",
  "buyer_operator",
  "platform_admin",
];

export function canReadDeveloper(subject?: SessionSubject) {
  return hasAnyRole(subject, developerReadRoles);
}

export function canManageApplications(subject?: SessionSubject) {
  return hasAnyRole(subject, developerWriteRoles);
}

export function canReadDeveloperTrace(subject?: SessionSubject) {
  return hasAnyRole(subject, developerReadRoles);
}

export function canSimulateMockPayment(subject?: SessionSubject) {
  return hasAnyRole(subject, mockPaymentRoles);
}

export function buildCreateApplicationPayload(
  values: ApplicationCreateFormValues,
): CreateApplicationRequest {
  return {
    org_id: values.org_id,
    app_name: values.app_name.trim(),
    app_type: values.app_type,
    client_id: values.client_id.trim(),
    client_secret_hash: normalizeOptional(values.client_secret_hash),
  };
}

export function buildPatchApplicationPayload(
  values: ApplicationPatchFormValues,
): PatchApplicationRequest {
  return {
    app_name: normalizeOptional(values.app_name),
    status: values.status,
  };
}

export function buildRotateSecretPayload(
  values: ApplicationSecretFormValues,
): RotateApplicationSecretRequest {
  return {
    client_secret_hash: normalizeOptional(values.client_secret_hash),
  };
}

export function buildDeveloperTraceQuery(
  values: DeveloperTraceFormValues,
): DeveloperTraceQuery {
  return {
    [values.lookup_mode]: values.lookup_value.trim(),
  } as DeveloperTraceQuery;
}

export function buildMockPaymentPayload(
  values: MockPaymentFormValues,
): MockPaymentSimulationRequest {
  return {
    delay_seconds: values.delay_seconds,
    duplicate_webhook: values.duplicate_webhook,
    partial_refund_amount: normalizeOptional(values.partial_refund_amount),
  };
}

export function createDeveloperIdempotencyKey(action: string) {
  return `web-016:${action}:${Date.now().toString(36)}`;
}

export function subjectDisplayName(subject?: SessionSubject) {
  if (!subject) {
    return "未解析";
  }
  return (
    subject.display_name ??
    subject.login_id ??
    subject.user_id ??
    subject.tenant_id ??
    subject.mode
  );
}

export function subjectRoles(subject?: SessionSubject) {
  return subject?.roles?.length ? subject.roles.join(" / ") : "无角色";
}

export function formatDeveloperError(error: unknown) {
  if (error instanceof PlatformApiError) {
    return `${error.code} · ${error.message}${error.requestId ? ` · request_id=${error.requestId}` : ""}`;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "未知错误";
}

export function statusTone(status?: string | null) {
  if (!status) {
    return "muted";
  }
  if (["active", "success", "succeeded", "processed", "clean", "confirmed"].includes(status)) {
    return "ok";
  }
  if (["revoked", "failed", "timeout", "blocked", "drift_detected"].includes(status)) {
    return "danger";
  }
  if (["missing", "pending", "pending_lock", "pending_check", "created"].includes(status)) {
    return "warn";
  }
  return "muted";
}

function hasAnyRole(subject: SessionSubject | undefined, roles: readonly string[]) {
  return Boolean(subject?.roles?.some((role) => roles.includes(role)));
}

function normalizeOptional(value?: string | null) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}
