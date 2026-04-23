import { PlatformApiError } from "./http";

export interface PlatformErrorCopy {
  code: string;
  title: string;
  description: string;
  known: boolean;
}

export interface NormalizePlatformErrorOptions {
  fallbackCode?: string;
  fallbackDescription?: string;
  includeBackendMessage?: boolean;
}

export interface NormalizedPlatformError extends PlatformErrorCopy {
  status: number | null;
  requestId: string | null;
  backendMessage: string | null;
}

const FORMAL_ERROR_TITLES: Record<string, string> = {
  OK: "请求成功",
  BAD_REQUEST: "请求格式错误",
  UNAUTHORIZED: "当前会话未认证",
  FORBIDDEN: "当前操作不被允许",
  NOT_FOUND: "目标对象不存在",
  CONFLICT: "对象状态冲突",
  TOO_MANY_REQUESTS: "请求过于频繁",
  INTERNAL_ERROR: "平台内部错误",
  SERVICE_UNAVAILABLE: "服务暂不可用",
  IDEMPOTENCY_KEY_REQUIRED: "缺少幂等键",
  IDEMPOTENCY_CONFLICT: "幂等键冲突",
  STEP_UP_REQUIRED: "需要二次认证",
  STEP_UP_EXPIRED: "二次认证已过期",
  AUTH_INVALID_CREDENTIAL: "登录凭证错误",
  AUTH_MFA_REQUIRED: "需要完成 MFA",
  AUTH_SESSION_REVOKED: "会话已撤销",
  AUTH_DEVICE_REVOKED: "设备已撤销",
  AUTH_STEP_UP_REQUIRED: "需要完成 step-up",
  AUTH_STEP_UP_EXPIRED: "step-up 已失效",
  AUTH_SSO_CONNECTION_INVALID: "企业 SSO 配置无效",
  AUTH_FABRIC_IDENTITY_NOT_BOUND: "Fabric 身份未绑定",
  AUTH_CERTIFICATE_REVOKED: "Fabric 证书已吊销",
  AUTH_CERTIFICATE_EXPIRED: "Fabric 证书已过期",
  PRODUCT_SUBMIT_FIELDS_INCOMPLETE: "商品提交字段不完整",
  PRODUCT_STATUS_INVALID: "商品状态不允许当前动作",
  SKU_TYPE_INVALID: "SKU 类型非法",
  SKU_CODE_DUPLICATED: "SKU 编码重复",
  SKU_RIGHTS_INVALID: "SKU 权利集合非法",
  SKU_PRICING_MODE_INVALID: "SKU 定价模式非法",
  SKU_DELIVERY_MODE_INVALID: "SKU 交付模式非法",
  TRADE_MODE_MISMATCH: "交易路径与 SKU 不匹配",
  QUERY_SURFACE_INVALID: "查询面配置非法",
  PRODUCT_VISIBILITY_FORBIDDEN: "当前商品不可见",
  TEMPLATE_NOT_FOUND: "模板不存在",
  TEMPLATE_STATUS_INVALID: "模板状态不可用",
  TEMPLATE_SKU_TYPE_MISMATCH: "模板与 SKU 不兼容",
  TEMPLATE_BIND_FORBIDDEN: "当前主体无权绑定模板",
  POLICY_UPDATE_FORBIDDEN: "当前策略不允许修改",
  API_TEMPLATE_NAME_FORBIDDEN: "命中被冻结的旧 API 模板名",
  ORDER_CREATE_FORBIDDEN: "当前主体或商品不允许下单",
  ORDER_STATE_INVALID: "当前订单状态不允许执行动作",
  CONTRACT_CONFIRM_FORBIDDEN: "当前合同不可确认",
  CONTRACT_STATUS_INVALID: "合同状态非法",
  AUTHORIZATION_NOT_ACTIVE: "授权未生效",
  AUTHORIZATION_REVOKED: "授权已撤销",
  DELIVERY_STATUS_INVALID: "交付状态非法",
  SETTLEMENT_STATUS_INVALID: "结算状态非法",
  DISPUTE_STATUS_INVALID: "争议状态非法",
  LIFECYCLE_SNAPSHOT_NOT_AVAILABLE: "生命周期快照不可用",
  PAYMENT_INTENT_NOT_FOUND: "支付意图不存在",
  PAYMENT_INTENT_STATUS_INVALID: "支付意图状态非法",
  PAYMENT_CORRIDOR_BLOCKED: "支付走廊被阻断",
  PAYMENT_PROVIDER_UNAVAILABLE: "支付通道不可用",
  PAYMENT_SIGNATURE_INVALID: "支付回调验签失败",
  PAYMENT_CALLBACK_DUPLICATED: "支付回调重复",
  PAYMENT_AMOUNT_MISMATCH: "支付金额不匹配",
  REFUND_FORBIDDEN: "当前对象不允许退款",
  PAYOUT_FORBIDDEN: "当前对象不允许打款",
  RECONCILIATION_REQUIRED: "当前对象需先完成对账",
  BILLING_EVENT_INVALID: "账单事件非法",
  DOWNLOAD_TICKET_EXPIRED: "下载票据已过期",
  DOWNLOAD_TICKET_FORBIDDEN: "下载票据无效或无权使用",
  SHARE_GRANT_FORBIDDEN: "只读共享授权不允许",
  SUBSCRIPTION_NOT_ALLOWED: "当前 SKU 不支持订阅",
  TEMPLATE_RUN_FORBIDDEN: "模板执行不允许",
  SANDBOX_WORKSPACE_FORBIDDEN: "查询沙箱开通不允许",
  QUERY_OUTPUT_POLICY_BLOCKED: "查询输出被策略阻断",
  USAGE_LOG_FORBIDDEN: "无权查看使用日志",
  DISPUTE_CREATE_FORBIDDEN: "当前订单不可发起争议",
  DISPUTE_EVIDENCE_INVALID: "争议证据格式或范围非法",
  DISPUTE_RESOLVE_FORBIDDEN: "当前主体不可裁决争议",
  COMPENSATION_FORBIDDEN: "当前对象不允许赔付",
  CASE_ALREADY_CLOSED: "案件已关闭",
  AUDIT_RAW_VIEW_FORBIDDEN: "无权查看原始审计内容",
  AUDIT_UNMASKED_VIEW_REQUIRES_STEP_UP: "查看未脱敏审计内容需 step-up",
  AUDIT_REPLAY_DRY_RUN_ONLY: "当前环境只允许 dry-run 回放",
  AUDIT_LEGAL_HOLD_ACTIVE: "对象处于 legal hold",
  AUDIT_BREAKGLASS_REASON_REQUIRED: "break-glass 缺少理由",
  AUDIT_EXPORT_SCOPE_FORBIDDEN: "审计导出范围不允许",
  OUTBOX_EVENT_NOT_FOUND: "outbox 事件不存在",
  OUTBOX_STATUS_INVALID: "outbox 状态非法",
  DEAD_LETTER_NOT_FOUND: "dead letter 不存在",
  DEAD_LETTER_REPROCESS_FORBIDDEN: "不允许重处理 dead letter",
  RECONCILE_FORBIDDEN: "不允许发起一致性修复",
  RECONCILE_DRY_RUN_REQUIRED: "当前环境必须 dry-run",
  EXTERNAL_FACT_STATUS_INVALID: "外部事实状态非法",
  PROOF_COMMIT_STATUS_INVALID: "证明提交状态非法",
  SEARCH_QUERY_INVALID: "搜索参数非法",
  SEARCH_BACKEND_UNAVAILABLE: "搜索后端不可用",
  SEARCH_RESULT_STALE: "搜索结果已过期或投影落后",
  SEARCH_REINDEX_FORBIDDEN: "不允许发起搜索重建",
  SEARCH_ALIAS_SWITCH_FORBIDDEN: "不允许切换搜索别名",
  SEARCH_CACHE_INVALIDATE_FORBIDDEN: "不允许失效搜索缓存",
  RECOMMENDATION_REQUEST_INVALID: "推荐请求参数非法",
  RECOMMENDATION_CONFIG_FORBIDDEN: "不允许修改推荐配置",
  RECOMMENDATION_REBUILD_FORBIDDEN: "不允许触发推荐重建",
  OPS_LOG_EXPORT_FORBIDDEN: "不允许导出日志",
  OPS_OBSERVABILITY_BACKEND_FORBIDDEN: "不允许修改观测后端",
  OPS_ALERT_RULE_UPDATE_FORBIDDEN: "不允许修改告警规则",
  OPS_INCIDENT_FORCE_CLOSE_FORBIDDEN: "不允许强制关闭事件工单",
  OPS_SLO_UPDATE_FORBIDDEN: "不允许修改 SLO",
  TRADE_MONITOR_FORBIDDEN: "无权查看交易链总览",
  EXTERNAL_FACT_RECEIPT_NOT_FOUND: "外部事实回执不存在",
  FAIRNESS_INCIDENT_NOT_FOUND: "公平性事件不存在",
  FAIRNESS_INCIDENT_RESOLVE_FORBIDDEN: "不允许处理公平性事件",
  CHAIN_PROJECTION_GAP_NOT_FOUND: "链投影缺口不存在",
  CHAIN_PROJECTION_RESOLVE_FORBIDDEN: "不允许关闭链投影缺口",
  PROVIDER_CALLBACK_INVALID: "回调请求非法",
  PROVIDER_SIGNATURE_INVALID: "回调签名非法",
  PROVIDER_EVENT_DUPLICATED: "外部事件重复",
  INTEGRATION_ROUTE_BLOCKED: "外部集成路由被策略阻断",
  INTEGRATION_PROVIDER_UNAVAILABLE: "外部集成不可用",
};

const COMPATIBILITY_ERROR_TITLES: Record<string, string> = {
  IAM_UNAUTHORIZED: "当前身份未通过认证",
  CAT_VALIDATION_FAILED: "目录与商品请求校验失败",
  TRD_STATE_CONFLICT: "订单状态冲突或业务前置条件不满足",
  DLV_ACCESS_DENIED: "无权访问当前交付对象或交付动作",
  BIL_PROVIDER_FAILED: "支付或账单通道执行失败",
  AUD_EVIDENCE_INVALID: "审计证据对象非法",
  OPS_CORE_CONFIG: "运维核心配置异常",
  OPS_CORE_STARTUP: "运维核心启动失败",
  OPS_CORE_SHUTDOWN: "运维核心关闭失败",
  OPS_INTERNAL: "运维内部错误",
};

const GUIDANCE_BY_CODE: Record<string, string> = {
  AUTH_INVALID_CREDENTIAL: "请检查账号凭证、MFA 状态或重新获取 Keycloak / IAM access token。",
  AUTH_MFA_REQUIRED: "请先完成 MFA，再继续登录或执行高风险动作。",
  AUTH_STEP_UP_REQUIRED: "请先完成 step-up，再重试当前高风险操作。",
  SEARCH_BACKEND_UNAVAILABLE: "请稍后重试；前端只通过 platform-core 受控边界承接搜索结果。",
  SEARCH_RESULT_STALE: "请刷新页面重新查询，避免继续使用过期的投影结果。",
  ORDER_CREATE_FORBIDDEN: "请检查当前主体权限、商品可见性、SKU/场景映射和订单前置条件。",
  DELIVERY_STATUS_INVALID: "请刷新订单详情和生命周期状态，再决定是否继续交付或验收。",
  DOWNLOAD_TICKET_EXPIRED: "请返回交付页重新生成受控下载票据，不要暴露或复用旧对象路径。",
  DOWNLOAD_TICKET_FORBIDDEN: "请确认当前主体、订单作用域和票据状态，再重新申请下载。",
  SHARE_GRANT_FORBIDDEN: "请检查共享范围、订单支付状态和当前主体是否位于正式卖方作用域内。",
  SANDBOX_WORKSPACE_FORBIDDEN: "请检查查询面、执行环境、席位用户和导出策略是否满足正式约束。",
  DISPUTE_EVIDENCE_INVALID: "请核对证据文件、范围和对象归属，避免上传不合规内容。",
  AUDIT_UNMASKED_VIEW_REQUIRES_STEP_UP: "请完成 step-up，并确认 break-glass 理由后再查看未脱敏内容。",
  AUDIT_REPLAY_DRY_RUN_ONLY: "当前环境仅允许 dry-run，请不要把回放当作正式变更执行。",
  RECOMMENDATION_REBUILD_FORBIDDEN: "请检查当前主体权限与 step-up 状态，再决定是否发起推荐重建。",
  TRD_STATE_CONFLICT: "请刷新订单详情和生命周期快照，再结合错误码与 request_id 排查状态冲突。",
  CAT_VALIDATION_FAILED: "请检查请求字段、正式枚举和值域，避免继续依赖裸 message 推断规则。",
  BIL_PROVIDER_FAILED: "请检查支付通道、账单执行条件和重试策略，并以 request_id 回查后端链路。",
  DLV_ACCESS_DENIED: "请检查当前主体角色、租户范围和交付对象权限，前端不得绕过后端直连受限系统。",
};

const DOMAIN_LABELS: Array<[RegExp, string]> = [
  [/^(AUTH|IAM)_/, "身份与会话"],
  [/^(CAT|PRODUCT|SKU|TEMPLATE|POLICY|QUERY_SURFACE|TRADE_MODE)_/, "目录与商品"],
  [/^(ORDER|TRD|CONTRACT|AUTHORIZATION|DELIVERY_STATUS|SETTLEMENT_STATUS|LIFECYCLE)_/, "订单与合同"],
  [/^(PAYMENT|BIL|REFUND|PAYOUT|RECONCILIATION|BILLING)_/, "支付与账单"],
  [/^(FILE_DELIVERY|DOWNLOAD|SHARE|SUBSCRIPTION|TEMPLATE_RUN|SANDBOX|QUERY_OUTPUT|USAGE_LOG|DLV)_/, "交付与执行"],
  [/^(DISPUTE|CASE|COMPENSATION)_/, "争议与赔付"],
  [/^(AUD|AUDIT)_/, "审计与回放"],
  [/^(OUTBOX|DEAD_LETTER|RECONCILE|EXTERNAL_FACT|PROOF|CHAIN|FAIRNESS|TRADE_MONITOR)_/, "一致性与联查"],
  [/^(SEARCH|RECOMMENDATION)_/, "搜索与推荐"],
  [/^(OPS)_/, "运维"],
  [/^(PROVIDER|INTEGRATION)_/, "外部集成"],
];

export function describePlatformErrorCode(code?: string | null): PlatformErrorCopy {
  const normalizedCode = normalizeCode(code);
  const title =
    FORMAL_ERROR_TITLES[normalizedCode] ??
    COMPATIBILITY_ERROR_TITLES[normalizedCode] ??
    inferFallbackTitle(normalizedCode);
  const description =
    GUIDANCE_BY_CODE[normalizedCode] ??
    inferFallbackDescription(normalizedCode);

  return {
    code: normalizedCode,
    title,
    description,
    known:
      normalizedCode in FORMAL_ERROR_TITLES ||
      normalizedCode in COMPATIBILITY_ERROR_TITLES,
  };
}

export function normalizePlatformError(
  error: unknown,
  options: NormalizePlatformErrorOptions = {},
): NormalizedPlatformError {
  if (error instanceof PlatformApiError) {
    const copy = describePlatformErrorCode(
      error.code && error.code !== "UNKNOWN"
        ? error.code
        : options.fallbackCode,
    );
    return {
      ...copy,
      status: error.status,
      requestId: error.requestId ?? null,
      backendMessage: sanitizeBackendMessage(error.code, error.message),
    };
  }

  if (error instanceof Error) {
    const copy = describePlatformErrorCode(options.fallbackCode ?? "INTERNAL_ERROR");
    return {
      ...copy,
      description: options.fallbackDescription ?? copy.description,
      status: null,
      requestId: null,
      backendMessage: error.message || null,
    };
  }

  const copy = describePlatformErrorCode(options.fallbackCode ?? "INTERNAL_ERROR");
  return {
    ...copy,
    description: options.fallbackDescription ?? copy.description,
    status: null,
    requestId: null,
    backendMessage: null,
  };
}

export function formatPlatformErrorForDisplay(
  error: unknown,
  options: NormalizePlatformErrorOptions = {},
): string {
  const normalized = normalizePlatformError(error, options);
  const segments = [
    `${normalized.title}。${normalized.description}`,
    `错误码 ${normalized.code}`,
  ];

  if (normalized.requestId) {
    segments.push(`request_id ${normalized.requestId}`);
  }

  if (options.includeBackendMessage && normalized.backendMessage) {
    segments.push(`后端提示 ${normalized.backendMessage}`);
  }

  return segments.join(" / ");
}

function normalizeCode(code?: string | null): string {
  const normalized = code?.trim().toUpperCase();
  return normalized ? normalized : "INTERNAL_ERROR";
}

function inferFallbackTitle(code: string): string {
  const domain = inferDomain(code);
  const suffix = inferSuffixTitle(code);
  return domain ? `${domain}${suffix}` : suffix;
}

function inferFallbackDescription(code: string): string {
  if (/_STEP_UP_REQUIRED$/.test(code) || /_REQUIRES_STEP_UP$/.test(code) || code === "STEP_UP_REQUIRED") {
    return "请先完成 step-up，再重试当前高风险动作。";
  }
  if (/_STEP_UP_EXPIRED$/.test(code) || code === "STEP_UP_EXPIRED") {
    return "step-up 已失效，请重新发起二次认证后再继续。";
  }
  if (/_UNAUTHORIZED$/.test(code)) {
    return "请重新登录或刷新当前 Bearer / 会话 Cookie 后重试。";
  }
  if (/_FORBIDDEN$/.test(code)) {
    return "请检查当前主体权限、租户范围和对象状态，再决定是否继续操作。";
  }
  if (/_NOT_FOUND$/.test(code)) {
    return "请确认对象 ID、路由参数和当前租户范围，避免联查不存在的正式对象。";
  }
  if (/_INVALID$/.test(code) || /_FAILED$/.test(code)) {
    return "请检查请求参数、当前对象状态和平台前置条件，再结合 request_id 排查。";
  }
  if (/_CONFLICT$/.test(code) || /_STATUS_INVALID$/.test(code) || /_NOT_ACTIVE$/.test(code)) {
    return "请刷新页面获取最新状态，再结合错误码与 request_id 排查状态冲突。";
  }
  if (/_EXPIRED$/.test(code)) {
    return "请重新获取新的凭据、票据或临时令牌后再重试。";
  }
  if (/_UNAVAILABLE$/.test(code) || /_BLOCKED$/.test(code)) {
    return "请稍后重试；如持续失败，请根据 request_id 回查后端和运维链路。";
  }
  if (/_DUPLICATED$/.test(code) || /_CONFLICT$/.test(code)) {
    return "请确认是否发生重复提交或重复回调，并优先使用既有结果。";
  }
  if (/_REQUIRED$/.test(code)) {
    return "请补齐必填字段、幂等键或前置审批信息后再提交。";
  }
  if (/_REVOKED$/.test(code)) {
    return "相关授权、凭据或证书已撤销，请重新建立正式授权链路。";
  }
  if (/_DRY_RUN_ONLY$/.test(code)) {
    return "当前环境只允许 dry-run，请不要把该操作当作正式变更执行。";
  }
  if (/_STALE$/.test(code)) {
    return "当前结果已过期，请刷新页面重新查询。";
  }
  if (/_MISMATCH$/.test(code)) {
    return "请求与当前对象快照不一致，请检查 SKU、模板、金额或主体范围。";
  }
  return "请结合错误码和 request_id 回查 platform-core 审计、日志和业务对象状态。";
}

function inferDomain(code: string): string {
  for (const [pattern, label] of DOMAIN_LABELS) {
    if (pattern.test(code)) {
      return label;
    }
  }
  return "";
}

function inferSuffixTitle(code: string): string {
  if (/_STEP_UP_REQUIRED$/.test(code) || /_REQUIRES_STEP_UP$/.test(code) || code === "STEP_UP_REQUIRED") {
    return "需要二次认证";
  }
  if (/_STEP_UP_EXPIRED$/.test(code) || code === "STEP_UP_EXPIRED") {
    return "二次认证已过期";
  }
  if (/_UNAUTHORIZED$/.test(code)) {
    return "未通过认证";
  }
  if (/_FORBIDDEN$/.test(code)) {
    return "当前操作不被允许";
  }
  if (/_NOT_FOUND$/.test(code)) {
    return "对象不存在";
  }
  if (/_INVALID$/.test(code)) {
    return "参数或状态非法";
  }
  if (/_CONFLICT$/.test(code) || /_STATUS_INVALID$/.test(code) || /_NOT_ACTIVE$/.test(code)) {
    return "状态冲突";
  }
  if (/_EXPIRED$/.test(code)) {
    return "凭据或票据已过期";
  }
  if (/_UNAVAILABLE$/.test(code) || /_BLOCKED$/.test(code)) {
    return "服务暂不可用";
  }
  if (/_DUPLICATED$/.test(code)) {
    return "请求重复";
  }
  if (/_REQUIRED$/.test(code)) {
    return "缺少必要信息";
  }
  if (/_REVOKED$/.test(code)) {
    return "授权已撤销";
  }
  if (/_STALE$/.test(code)) {
    return "结果已过期";
  }
  return "业务错误";
}

function sanitizeBackendMessage(code: string, message: string): string | null {
  const normalized = message.trim();
  if (!normalized) {
    return null;
  }

  if (normalized === code) {
    return null;
  }

  if (normalized.startsWith(`${code}:`)) {
    return normalized.slice(code.length + 1).trim() || null;
  }

  return normalized;
}
