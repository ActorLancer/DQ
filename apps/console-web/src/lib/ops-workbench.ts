import { PlatformApiError } from "@datab/sdk-ts";
import type {
  AuthMeResponse,
  ConsistencyPath,
  ConsistencyReconcileRequest,
  DeadLettersQuery,
  DeadLetterReprocessRequest,
  OutboxQuery,
  RecommendationPlacementPatchRequest,
  RecommendationRankingProfilePatchRequest,
  RecommendationRebuildRequest,
  SearchAliasSwitchRequest,
  SearchCacheInvalidateRequest,
  SearchRankingProfilePatchRequest,
  SearchReindexRequest,
  SearchSyncQuery,
} from "@datab/sdk-ts";
import { z } from "zod";

export type SessionSubject = AuthMeResponse["data"];

export const consistencyRefTypes = [
  "order",
  "contract",
  "digital_contract",
  "delivery",
  "delivery_record",
  "settlement",
  "settlement_record",
  "payment",
  "payment_intent",
  "refund",
  "refund_intent",
  "payout",
  "payout_instruction",
] as const;

export const consistencyLookupSchema = z.object({
  ref_type: z.enum(consistencyRefTypes),
  ref_id: z.string().trim().min(3, "请输入正式业务对象 ID"),
});

export const consistencyReconcileSchema = consistencyLookupSchema.extend({
  mode: z.enum(["projection_gap", "full"]),
  reason: z.string().trim().min(8, "修复预演原因至少 8 个字符").max(500),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
}).superRefine(requireStepUp("step_up_token"));

export const outboxFilterSchema = z.object({
  outbox_status: z.string().trim().optional(),
  event_type: z.string().trim().optional(),
  target_topic: z.string().trim().optional(),
  request_id: z.string().trim().optional(),
  trace_id: z.string().trim().optional(),
  page_size: z.number().int().min(5).max(100),
});

export const deadLetterFilterSchema = z.object({
  reprocess_status: z.string().trim().optional(),
  failure_stage: z.string().trim().optional(),
  request_id: z.string().trim().optional(),
  trace_id: z.string().trim().optional(),
  page_size: z.number().int().min(5).max(100),
});

export const deadLetterReprocessSchema = z.object({
  dead_letter_event_id: z.string().uuid("请输入正式 dead_letter_event_id"),
  reason: z.string().trim().min(8, "重处理原因至少 8 个字符").max(500),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
}).superRefine(requireStepUp("step_up_token"));

export const searchSyncFilterSchema = z.object({
  entity_scope: z.enum(["all", "product", "seller"]),
  sync_status: z.string().trim().optional(),
  limit: z.number().int().min(1).max(100),
});

export const searchReindexSchema = z.object({
  entity_scope: z.enum(["product", "seller", "all"]),
  entity_id: z.string().uuid("single 模式需要正式 UUID").optional().or(z.literal("")),
  mode: z.enum(["single", "batch", "full"]),
  force: z.boolean(),
  target_index: z.string().trim().optional(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
}).superRefine((value, context) => {
  if (value.mode === "single" && !value.entity_id) {
    context.addIssue({
      code: "custom",
      path: ["entity_id"],
      message: "single reindex 必须填写 entity_id",
    });
  }
  requireStepUp("step_up_token")(value, context);
});

export const aliasSwitchSchema = z.object({
  entity_scope: z.enum(["product", "seller"]),
  next_index_name: z.string().trim().min(3, "请输入新物理索引名"),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
}).superRefine(requireStepUp("step_up_token"));

export const cacheInvalidateSchema = z.object({
  entity_scope: z.enum(["all", "product", "service", "seller"]),
  query_hash: z.string().trim().optional(),
  purge_all: z.boolean(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
});

export const searchRankingPatchSchema = z.object({
  ranking_profile_id: z.string().uuid("请选择正式 ranking_profile_id"),
  status: z.string().trim().optional(),
  weights_json: z.string().trim().optional(),
  filter_policy_json: z.string().trim().optional(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
}).superRefine(requireStepUp("step_up_token"));

export const recommendationRebuildSchema = z.object({
  scope: z.enum([
    "all",
    "cache",
    "features",
    "subject_profile",
    "cohort",
    "signals",
    "similarity",
    "bundle",
  ]),
  placement_code: z.string().trim().optional(),
  entity_scope: z.enum(["all", "product", "seller"]),
  entity_id: z.string().uuid("请输入正式实体 UUID").optional().or(z.literal("")),
  purge_cache: z.boolean(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
}).superRefine((value, context) => {
  if (value.entity_scope !== "all" && !value.entity_id) {
    context.addIssue({
      code: "custom",
      path: ["entity_id"],
      message: "选择 product / seller 重建时必须填写 entity_id",
    });
  }
  if (value.entity_scope === "all" && value.entity_id) {
    context.addIssue({
      code: "custom",
      path: ["entity_scope"],
      message: "填写 entity_id 时必须选择 product 或 seller",
    });
  }
  requireStepUp("step_up_token")(value, context);
});

export const recommendationPlacementPatchSchema = z.object({
  placement_code: z.string().trim().min(3, "请选择推荐位"),
  status: z.string().trim().optional(),
  default_ranking_profile_key: z.string().trim().optional(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
}).superRefine(requireStepUp("step_up_token"));

export const recommendationRankingPatchSchema = z.object({
  ranking_profile_id: z.string().uuid("请选择正式 recommendation_ranking_profile_id"),
  status: z.string().trim().optional(),
  explain_codes: z.string().trim().optional(),
  idempotency_key: z.string().trim().min(12, "幂等键不能为空"),
  step_up_token: z.string().trim().optional(),
  step_up_challenge_id: z.string().trim().optional(),
}).superRefine(requireStepUp("step_up_token"));

export type ConsistencyLookupFormValues = z.infer<typeof consistencyLookupSchema>;
export type ConsistencyReconcileFormValues = z.infer<typeof consistencyReconcileSchema>;
export type OutboxFilterFormValues = z.infer<typeof outboxFilterSchema>;
export type DeadLetterFilterFormValues = z.infer<typeof deadLetterFilterSchema>;
export type DeadLetterReprocessFormValues = z.infer<typeof deadLetterReprocessSchema>;
export type SearchSyncFilterFormValues = z.infer<typeof searchSyncFilterSchema>;
export type SearchReindexFormValues = z.infer<typeof searchReindexSchema>;
export type AliasSwitchFormValues = z.infer<typeof aliasSwitchSchema>;
export type CacheInvalidateFormValues = z.infer<typeof cacheInvalidateSchema>;
export type SearchRankingPatchFormValues = z.infer<typeof searchRankingPatchSchema>;
export type RecommendationRebuildFormValues = z.infer<typeof recommendationRebuildSchema>;
export type RecommendationPlacementPatchFormValues = z.infer<typeof recommendationPlacementPatchSchema>;
export type RecommendationRankingPatchFormValues = z.infer<typeof recommendationRankingPatchSchema>;

const opsReadRoles = ["platform_admin", "platform_audit_security"];
const searchManageRoles = ["platform_admin"];
const searchCacheRoles = ["platform_admin", "platform_audit_security"];
const recommendationReadRoles = ["platform_admin"];
const recommendationManageRoles = ["platform_admin"];

export function canReadConsistency(subject?: SessionSubject) {
  return hasAnyRole(subject, opsReadRoles);
}

export function canReconcileConsistency(subject?: SessionSubject) {
  return hasAnyRole(subject, opsReadRoles);
}

export function canReadOutbox(subject?: SessionSubject) {
  return hasAnyRole(subject, opsReadRoles);
}

export function canReprocessDeadLetter(subject?: SessionSubject) {
  return hasAnyRole(subject, opsReadRoles);
}

export function canReadSearchOps(subject?: SessionSubject) {
  return hasAnyRole(subject, searchCacheRoles);
}

export function canManageSearchOps(subject?: SessionSubject) {
  return hasAnyRole(subject, searchManageRoles);
}

export function canInvalidateSearchCache(subject?: SessionSubject) {
  return hasAnyRole(subject, searchCacheRoles);
}

export function canReadRecommendationOps(subject?: SessionSubject) {
  return hasAnyRole(subject, recommendationReadRoles);
}

export function canManageRecommendationOps(subject?: SessionSubject) {
  return hasAnyRole(subject, recommendationManageRoles);
}

export function canReadObservability(subject?: SessionSubject) {
  return hasAnyRole(subject, opsReadRoles);
}

export function hasAnyRole(subject: SessionSubject | undefined, roles: string[]) {
  const currentRoles = new Set(subject?.roles ?? []);
  return roles.some((role) => currentRoles.has(role));
}

export function subjectDisplayName(subject?: SessionSubject) {
  return subject?.display_name || subject?.login_id || subject?.user_id || "未登录主体";
}

export function createOpsIdempotencyKey(scope: string) {
  return `web-015:${scope}:${globalThis.crypto.randomUUID()}`;
}

export function buildConsistencyPath(
  values: ConsistencyLookupFormValues,
): ConsistencyPath {
  return {
    refType: values.ref_type,
    refId: values.ref_id.trim(),
  };
}

export function buildConsistencyReconcilePayload(
  values: ConsistencyReconcileFormValues,
): ConsistencyReconcileRequest {
  return {
    ref_type: values.ref_type,
    ref_id: values.ref_id.trim(),
    mode: values.mode,
    dry_run: true,
    reason: values.reason.trim(),
  };
}

export function buildOutboxQuery(values: OutboxFilterFormValues): OutboxQuery {
  return compactQuery({
    outbox_status: values.outbox_status,
    event_type: values.event_type,
    target_topic: values.target_topic,
    request_id: values.request_id,
    trace_id: values.trace_id,
    page: 1,
    page_size: values.page_size,
  });
}

export function buildDeadLettersQuery(
  values: DeadLetterFilterFormValues,
): DeadLettersQuery {
  return compactQuery({
    reprocess_status: values.reprocess_status,
    failure_stage: values.failure_stage,
    request_id: values.request_id,
    trace_id: values.trace_id,
    page: 1,
    page_size: values.page_size,
  });
}

export function buildDeadLetterReprocessPayload(
  values: DeadLetterReprocessFormValues,
): DeadLetterReprocessRequest {
  return {
    reason: values.reason.trim(),
    dry_run: true,
    metadata: {
      source: "console-web",
      task_id: "WEB-015",
    },
  };
}

export function buildSearchSyncQuery(
  values: SearchSyncFilterFormValues,
): SearchSyncQuery {
  return compactQuery({
    entity_scope: values.entity_scope === "all" ? undefined : values.entity_scope,
    sync_status: values.sync_status,
    limit: values.limit,
  });
}

export function buildSearchReindexPayload(
  values: SearchReindexFormValues,
): SearchReindexRequest {
  return compactBody({
    entity_scope: values.entity_scope,
    entity_id: values.entity_id || undefined,
    mode: values.mode,
    force: values.force,
    target_index: values.target_index,
  });
}

export function buildAliasSwitchPayload(
  values: AliasSwitchFormValues,
): SearchAliasSwitchRequest {
  return {
    entity_scope: values.entity_scope,
    next_index_name: values.next_index_name.trim(),
  };
}

export function buildCacheInvalidatePayload(
  values: CacheInvalidateFormValues,
): SearchCacheInvalidateRequest {
  return compactBody({
    entity_scope: values.entity_scope,
    query_hash: values.query_hash,
    purge_all: values.purge_all,
  });
}

export function buildSearchRankingPatchPayload(
  values: SearchRankingPatchFormValues,
): SearchRankingProfilePatchRequest {
  return compactBody({
    status: values.status,
    weights_json: parseJsonObject(values.weights_json),
    filter_policy_json: parseJsonObject(values.filter_policy_json),
  });
}

export function buildRecommendationRebuildPayload(
  values: RecommendationRebuildFormValues,
): RecommendationRebuildRequest {
  return compactBody({
    scope: values.scope,
    placement_code: values.placement_code,
    entity_scope: values.entity_scope === "all" ? undefined : values.entity_scope,
    entity_id: values.entity_id || undefined,
    purge_cache: values.purge_cache,
  });
}

export function buildRecommendationPlacementPatchPayload(
  values: RecommendationPlacementPatchFormValues,
): RecommendationPlacementPatchRequest {
  return compactBody({
    status: values.status,
    default_ranking_profile_key: values.default_ranking_profile_key,
  });
}

export function buildRecommendationRankingPatchPayload(
  values: RecommendationRankingPatchFormValues,
): RecommendationRankingProfilePatchRequest {
  return compactBody({
    status: values.status,
    explain_codes: splitCsv(values.explain_codes),
  });
}

export function formatOpsError(error: unknown) {
  if (error instanceof PlatformApiError) {
    return {
      title: `${error.code} / HTTP ${error.status}`,
      message: error.message,
      requestId: error.requestId ?? "未返回",
    };
  }
  if (error instanceof Error) {
    return {
      title: "CLIENT_ERROR",
      message: error.message,
      requestId: "未生成",
    };
  }
  return {
    title: "UNKNOWN_ERROR",
    message: "未知错误",
    requestId: "未生成",
  };
}

export function parseJsonObject(input?: string): Record<string, never> | undefined {
  const normalized = input?.trim();
  if (!normalized) {
    return undefined;
  }
  const parsed = JSON.parse(normalized) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("JSON 必须是对象");
  }
  return parsed as Record<string, never>;
}

export function statusTone(status?: string | null) {
  const normalized = (status ?? "").toLowerCase();
  if (
    normalized.includes("ready") ||
    normalized.includes("matched") ||
    normalized.includes("clean") ||
    normalized.includes("published") ||
    normalized.includes("up") ||
    normalized.includes("active") ||
    normalized.includes("confirmed")
  ) {
    return "ok";
  }
  if (
    normalized.includes("failed") ||
    normalized.includes("dead") ||
    normalized.includes("open") ||
    normalized.includes("mismatch") ||
    normalized.includes("down") ||
    normalized.includes("critical")
  ) {
    return "danger";
  }
  return "warn";
}

function requireStepUp(path: string) {
  return (
    value: { step_up_token?: string; step_up_challenge_id?: string },
    context: z.RefinementCtx,
  ) => {
    if (!value.step_up_token && !value.step_up_challenge_id) {
      context.addIssue({
        code: "custom",
        path: [path],
        message: "高风险 ops 动作必须填写 step-up token 或 challenge id",
      });
    }
  };
}

function compactQuery<T extends Record<string, unknown>>(value: T) {
  return Object.fromEntries(
    Object.entries(value).filter(
      ([, item]) => item !== undefined && item !== null && item !== "",
    ),
  ) as T;
}

function compactBody<T extends Record<string, unknown>>(value: T) {
  return Object.fromEntries(
    Object.entries(value).filter(
      ([, item]) => item !== undefined && item !== null && item !== "",
    ),
  ) as T;
}

function splitCsv(input?: string) {
  const normalized = input?.trim();
  if (!normalized) {
    return undefined;
  }
  return normalized
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}
