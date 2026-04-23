export type ConsoleRouteKey =
  | "console_home"
  | "review_subjects"
  | "review_products"
  | "review_compliance"
  | "risk_console"
  | "audit_trace"
  | "audit_package_export"
  | "consistency_trace"
  | "outbox_dead_letter"
  | "notification_ops"
  | "search_ops"
  | "developer_home"
  | "developer_apps"
  | "developer_trace"
  | "developer_assets";

export interface ConsoleRouteMeta {
  key: ConsoleRouteKey;
  group: string;
  title: string;
  path: string;
  viewPermission: string;
  primaryPermissions: string[];
  description: string;
  apiBindings: string[];
}

export const consoleRouteList: ConsoleRouteMeta[] = [
  {
    key: "console_home",
    group: "控制台总览",
    title: "控制台首页",
    path: "/",
    viewPermission: "ops.observability.read / developer.home.read",
    primaryPermissions: [],
    description: "控制面入口、正式路由导航与最小联调摘要。",
    apiBindings: [
      "/health/ready",
      "/api/v1/auth/me",
      "/api/v1/ops/observability/overview",
      "/api/v1/ops/outbox",
      "/api/v1/audit/traces",
    ],
  },
  {
    key: "review_subjects",
    group: "审核与风控",
    title: "主体审核台",
    path: "/ops/review/subjects",
    viewPermission: "review.subject.read",
    primaryPermissions: ["review.subject.review"],
    description: "主体资料、认证状态与审核动作入口。",
    apiBindings: [
      "GET /api/v1/iam/orgs?status=pending_review",
      "GET /api/v1/iam/orgs/{id}",
      "POST /api/v1/review/subjects/{id}",
    ],
  },
  {
    key: "review_products",
    group: "审核与风控",
    title: "产品审核台",
    path: "/ops/review/products",
    viewPermission: "review.product.read",
    primaryPermissions: ["review.product.review"],
    description: "产品审核队列、审核明细与结论操作入口。",
    apiBindings: [
      "GET /api/v1/products?status=pending_review",
      "GET /api/v1/products/{id}",
      "POST /api/v1/review/products/{id}",
    ],
  },
  {
    key: "review_compliance",
    group: "审核与风控",
    title: "合规审核台",
    path: "/ops/review/compliance",
    viewPermission: "review.compliance.read",
    primaryPermissions: ["review.compliance.review", "review.compliance.block"],
    description: "合规命中、封禁建议与复核入口。",
    apiBindings: [
      "GET /api/v1/products?status=pending_review",
      "GET /api/v1/products/{id}",
      "POST /api/v1/review/compliance/{id}",
    ],
  },
  {
    key: "risk_console",
    group: "审核与风控",
    title: "风控工作台",
    path: "/ops/risk",
    viewPermission: "risk.fairness_incident.read",
    primaryPermissions: ["risk.fairness_incident.handle"],
    description: "风险事件、主体信誉和冻结建议联动入口。",
    apiBindings: [
      "GET /api/v1/ops/fairness-incidents",
      "POST /api/v1/ops/fairness-incidents/{id}/handle",
    ],
  },
  {
    key: "audit_trace",
    group: "审计与监管",
    title: "审计联查页",
    path: "/ops/audit/trace",
    viewPermission: "audit.trace.read",
    primaryPermissions: [],
    description: "按订单号、request_id、tx_hash 等主键联查全链路事实。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "GET /api/v1/audit/traces",
      "GET /api/v1/audit/orders/{id}",
      "GET /api/v1/developer/trace",
      "GET /api/v1/ops/trade-monitor/orders/{orderId}",
      "GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints",
      "GET /api/v1/ops/external-facts",
      "GET /api/v1/ops/projection-gaps",
      "POST /api/v1/audit/packages/export",
    ],
  },
  {
    key: "audit_package_export",
    group: "审计与监管",
    title: "证据包导出页",
    path: "/ops/audit/packages",
    viewPermission: "audit.event.read",
    primaryPermissions: ["audit.package.export"],
    description: "证据包导出、导出留痕与高风险提示入口。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "POST /api/v1/audit/packages/export",
    ],
  },
  {
    key: "consistency_trace",
    group: "审计与监管",
    title: "一致性联查页",
    path: "/ops/consistency",
    viewPermission: "ops.consistency.read",
    primaryPermissions: ["ops.consistency.reconcile"],
    description: "数据库主状态、链状态和外部事实一致性联查入口。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "GET /api/v1/ops/consistency/{refType}/{refId}",
      "POST /api/v1/ops/consistency/reconcile",
    ],
  },
  {
    key: "outbox_dead_letter",
    group: "审计与监管",
    title: "出站事件 / Dead Letter 页",
    path: "/ops/consistency/outbox",
    viewPermission: "ops.outbox.read / ops.dead_letter.read",
    primaryPermissions: ["ops.dead_letter.reprocess"],
    description: "outbox、发布尝试、死信与重处理入口。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "GET /api/v1/ops/outbox",
      "GET /api/v1/ops/dead-letters",
      "POST /api/v1/ops/dead-letters/{id}/reprocess",
      "GET /api/v1/ops/observability/overview",
    ],
  },
  {
    key: "notification_ops",
    group: "审计与监管",
    title: "通知联查页",
    path: "/ops/notifications",
    viewPermission: "ops.outbox.read / ops.dead_letter.read",
    primaryPermissions: ["ops.dead_letter.reprocess"],
    description: "通知发送记录、失败轨迹、模板联查与 dead letter replay 入口。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "POST /api/v1/ops/notifications/audit/search",
      "POST /api/v1/ops/notifications/dead-letters/{dead_letter_event_id}/replay",
      "GET /api/v1/ops/observability/overview",
    ],
  },
  {
    key: "search_ops",
    group: "审计与监管",
    title: "搜索运维页",
    path: "/ops/search",
    viewPermission: "ops.search_sync.read",
    primaryPermissions: [
      "ops.search_reindex.execute",
      "ops.search_alias.manage",
      "ops.search_cache.invalidate",
      "ops.search_ranking.manage",
    ],
    description: "搜索同步、别名、缓存和排序配置入口。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "GET /api/v1/ops/search/sync",
      "POST /api/v1/ops/search/reindex",
      "POST /api/v1/ops/search/aliases/switch",
      "POST /api/v1/ops/search/cache/invalidate",
      "GET /api/v1/ops/search/ranking-profiles",
      "PATCH /api/v1/ops/search/ranking-profiles/{id}",
      "GET /api/v1/ops/recommendation/placements",
      "GET /api/v1/ops/recommendation/ranking-profiles",
      "POST /api/v1/ops/recommendation/rebuild",
    ],
  },
  {
    key: "developer_home",
    group: "开发者通道",
    title: "开发者首页",
    path: "/developer",
    viewPermission: "developer.home.read",
    primaryPermissions: [],
    description: "开发调试导航、网络信息与联调说明入口。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "GET /health/deps",
      "GET /api/v1/apps",
      "GET /api/v1/developer/trace",
    ],
  },
  {
    key: "developer_apps",
    group: "开发者通道",
    title: "测试应用页",
    path: "/developer/apps",
    viewPermission: "developer.app.read",
    primaryPermissions: ["developer.app.create", "developer.app.update"],
    description: "测试应用、API Key 与调用配置入口。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "GET /api/v1/apps",
      "POST /api/v1/apps",
      "PATCH /api/v1/apps/{id}",
      "POST /api/v1/apps/{id}/credentials/rotate",
      "POST /api/v1/apps/{id}/credentials/revoke",
    ],
  },
  {
    key: "developer_trace",
    group: "开发者通道",
    title: "状态联查页",
    path: "/developer/trace",
    viewPermission: "developer.trace.read",
    primaryPermissions: [],
    description: "order / event / tx_hash 级别的开发态 trace 联查入口。",
    apiBindings: ["GET /api/v1/auth/me", "GET /api/v1/developer/trace"],
  },
  {
    key: "developer_assets",
    group: "开发者通道",
    title: "测试资产页",
    path: "/developer/assets",
    viewPermission: "developer.test_asset.read / developer.mock_payment.simulate",
    primaryPermissions: ["developer.mock_payment.simulate"],
    description: "测试钱包、测试账号、Mock 数据和 Mock 支付操作入口。",
    apiBindings: [
      "GET /api/v1/auth/me",
      "POST /api/v1/mock/payments/{id}/simulate-success",
      "POST /api/v1/mock/payments/{id}/simulate-fail",
      "POST /api/v1/mock/payments/{id}/simulate-timeout",
    ],
  },
];

export const consoleRouteMap = Object.fromEntries(
  consoleRouteList.map((route) => [route.key, route]),
) as Record<ConsoleRouteKey, ConsoleRouteMeta>;

export const consoleNavigationGroups: Array<{
  label: string;
  keys: ConsoleRouteKey[];
}> = [
  {
    label: "控制台总览",
    keys: ["console_home"],
  },
  {
    label: "审核与风控",
    keys: [
      "review_subjects",
      "review_products",
      "review_compliance",
      "risk_console",
    ],
  },
  {
    label: "审计与监管",
    keys: [
      "audit_trace",
      "audit_package_export",
      "consistency_trace",
      "outbox_dead_letter",
      "notification_ops",
      "search_ops",
    ],
  },
  {
    label: "开发者通道",
    keys: [
      "developer_home",
      "developer_apps",
      "developer_trace",
      "developer_assets",
    ],
  },
];
