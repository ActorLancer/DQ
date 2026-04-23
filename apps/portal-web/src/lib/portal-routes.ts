export type PortalRouteKey =
  | "portal_home"
  | "catalog_search"
  | "seller_profile"
  | "product_detail"
  | "seller_product_center"
  | "seller_product_edit"
  | "asset_raw_ingest_center"
  | "seller_metadata_contract"
  | "seller_sku_config"
  | "seller_template_bind"
  | "seller_query_surface"
  | "seller_share_modes"
  | "order_create"
  | "order_contract_confirm"
  | "order_payment_lock"
  | "order_detail"
  | "delivery_file"
  | "delivery_share"
  | "delivery_subscription"
  | "delivery_api"
  | "delivery_template_query"
  | "delivery_sandbox"
  | "delivery_query_runs"
  | "delivery_report"
  | "delivery_acceptance"
  | "billing_center"
  | "billing_refund_compensation"
  | "dispute_create"
  | "developer_home"
  | "developer_apps"
  | "developer_trace"
  | "developer_assets";

export interface PortalRouteMeta {
  key: PortalRouteKey;
  group: string;
  title: string;
  path: string;
  viewPermission: string;
  primaryPermissions: string[];
  description: string;
  apiBindings: string[];
}

export const portalRouteList: PortalRouteMeta[] = [
  {
    key: "portal_home",
    group: "门户与目录",
    title: "首页",
    path: "/",
    viewPermission: "portal.home.read",
    primaryPermissions: [],
    description: "门户首页、场景导航、推荐位、搜索入口与标准链路快捷入口。",
    apiBindings: [
      "/health/ready",
      "/api/v1/auth/me",
      "/api/v1/catalog/standard-scenarios",
      "/api/v1/recommendations",
      "/api/v1/catalog/search",
      "/api/v1/orders/standard-templates",
    ],
  },
  {
    key: "catalog_search",
    group: "门户与目录",
    title: "搜索页",
    path: "/search",
    viewPermission: "portal.search.read",
    primaryPermissions: ["portal.search.use"],
    description: "统一搜索、筛选、排序和结果卡片入口。",
    apiBindings: ["/api/v1/catalog/search"],
  },
  {
    key: "seller_profile",
    group: "门户与目录",
    title: "卖方主页",
    path: "/sellers/:orgId",
    viewPermission: "portal.seller.read",
    primaryPermissions: [],
    description: "卖方主体信息、认证标识、信誉风险、在售商品与咨询入口。",
    apiBindings: [
      "/api/v1/sellers/{orgId}/profile",
      "/api/v1/recommendations?placement_code=seller_profile_featured",
    ],
  },
  {
    key: "product_detail",
    group: "门户与目录",
    title: "产品详情页",
    path: "/products/:productId",
    viewPermission: "catalog.product.read",
    primaryPermissions: ["trade.order.create"],
    description: "产品详情、SKU、价格、样例预览和下单入口。",
    apiBindings: ["/api/v1/products/{id}"],
  },
  {
    key: "seller_product_center",
    group: "供方侧",
    title: "上架中心",
    path: "/seller/products",
    viewPermission: "catalog.product.list",
    primaryPermissions: ["catalog.product.create"],
    description: "卖方商品草稿、状态与上架入口。",
    apiBindings: [
      "GET /api/v1/products",
      "POST /api/v1/products",
      "GET /api/v1/products/{id}",
    ],
  },
  {
    key: "seller_product_edit",
    group: "供方侧",
    title: "产品编辑页",
    path: "/seller/products/:productId/edit",
    viewPermission: "catalog.product.read",
    primaryPermissions: ["catalog.product.update"],
    description: "产品基础信息、元字段与状态编辑入口。",
    apiBindings: [
      "GET /api/v1/products/{id}",
      "PATCH /api/v1/products/{id}",
    ],
  },
  {
    key: "asset_raw_ingest_center",
    group: "供方侧",
    title: "原始接入与加工页",
    path: "/seller/assets/:assetId/raw-ingest",
    viewPermission: "catalog.product.read",
    primaryPermissions: [
      "catalog.raw_ingest.manage",
      "catalog.extraction.manage",
      "catalog.preview.manage",
    ],
    description: "原始对象接入、抽取、预览与加工入口。",
    apiBindings: [
      "/api/v1/assets/{assetId}/raw-ingest-batches",
      "/api/v1/raw-ingest-batches/{id}/manifests",
    ],
  },
  {
    key: "seller_metadata_contract",
    group: "供方侧",
    title: "元信息与数据契约页",
    path: "/seller/products/:productId/metadata-contracts",
    viewPermission: "catalog.product.read",
    primaryPermissions: [
      "catalog.metadata.edit",
      "catalog.quality_report.manage",
      "catalog.processing.manage",
      "contract.data_contract.manage",
    ],
    description: "元信息、质量报告、数据契约与加工配置入口。",
    apiBindings: [
      "GET /api/v1/products/{id}",
      "/api/v1/products/{id}/metadata-profile",
      "/api/v1/assets/{versionId}/quality-reports",
      "/api/v1/skus/{id}/data-contracts",
    ],
  },
  {
    key: "seller_sku_config",
    group: "供方侧",
    title: "SKU 配置页",
    path: "/seller/products/:productId/skus",
    viewPermission: "catalog.sku.read",
    primaryPermissions: ["catalog.sku.create", "catalog.sku.update"],
    description: "标准 SKU 与交付路径配置。",
    apiBindings: [
      "GET /api/v1/products/{id}",
      "POST /api/v1/products/{id}/skus",
      "PATCH /api/v1/skus/{id}",
    ],
  },
  {
    key: "seller_template_bind",
    group: "供方侧",
    title: "模板绑定页",
    path: "/seller/products/:productId/templates",
    viewPermission: "template.contract.read",
    primaryPermissions: ["template.contract.bind", "template.policy.bind"],
    description: "合同模板、验收模板与策略模板绑定。",
    apiBindings: [
      "GET /api/v1/products/{id}",
      "POST /api/v1/products/{id}/bind-template",
      "POST /api/v1/skus/{id}/bind-template",
      "/api/v1/policies/{id}",
    ],
  },
  {
    key: "seller_query_surface",
    group: "供方侧",
    title: "查询面与模板配置页",
    path: "/seller/products/:productId/query-surfaces",
    viewPermission: "catalog.product.read",
    primaryPermissions: [
      "delivery.query_surface.manage",
      "delivery.query_template.manage",
    ],
    description: "查询面、模板与查询能力配置入口。",
    apiBindings: [
      "/api/v1/products/{productId}/query-surfaces",
      "/api/v1/query-surfaces/{id}/templates",
    ],
  },
  {
    key: "seller_share_modes",
    group: "供方侧",
    title: "共享与版本配置页",
    path: "/seller/products/:productId/share-modes",
    viewPermission: "catalog.product.read",
    primaryPermissions: [
      "catalog.asset_object.manage",
      "catalog.asset_release.manage",
    ],
    description: "共享、版本和发布策略入口。",
    apiBindings: [
      "/api/v1/assets/{versionId}/objects",
      "/api/v1/assets/{assetId}/release-policy",
    ],
  },
  {
    key: "order_create",
    group: "交易侧",
    title: "询单/下单页",
    path: "/trade/orders/new",
    viewPermission: "catalog.product.read",
    primaryPermissions: ["trade.order.create"],
    description: "询单、冻结模板与标准链路下单入口。",
    apiBindings: [
      "/api/v1/auth/me",
      "/api/v1/products/{id}",
      "/api/v1/orders/standard-templates",
      "POST /api/v1/orders",
    ],
  },
  {
    key: "order_contract_confirm",
    group: "交易侧",
    title: "合同确认页",
    path: "/trade/orders/:orderId/contract",
    viewPermission: "template.contract.read",
    primaryPermissions: ["trade.contract.confirm"],
    description: "合同模板确认与签署入口。",
    apiBindings: ["/api/v1/orders/{id}/contract-confirm"],
  },
  {
    key: "order_payment_lock",
    group: "交易侧",
    title: "支付锁定页",
    path: "/trade/orders/:orderId/payment-lock",
    viewPermission: "trade.order.read",
    primaryPermissions: ["billing.deposit.lock"],
    description: "支付意图、锁资与支付结果入口。",
    apiBindings: [
      "/api/v1/payments/intents",
      "/api/v1/orders/{id}/lock",
      "/api/v1/orders/{id}",
    ],
  },
  {
    key: "order_detail",
    group: "交易侧",
    title: "订单详情页",
    path: "/trade/orders/:orderId",
    viewPermission: "trade.order.read",
    primaryPermissions: ["trade.order.cancel"],
    description: "订单主状态、分层状态和生命周期详情。",
    apiBindings: [
      "/api/v1/auth/me",
      "/api/v1/orders/{id}",
      "/api/v1/orders/{id}/lifecycle-snapshots",
      "POST /api/v1/orders/{id}/cancel",
    ],
  },
  {
    key: "delivery_file",
    group: "交付与验收",
    title: "文件交付页",
    path: "/delivery/orders/:orderId/file",
    viewPermission: "trade.order.read",
    primaryPermissions: ["delivery.file.commit", "delivery.file.download"],
    description: "文件交付、下载票据与回执入口。",
    apiBindings: [
      "/api/v1/auth/me",
      "GET /api/v1/orders/{id}",
      "GET /api/v1/orders/{id}/lifecycle-snapshots",
      "POST /api/v1/orders/{id}/deliver",
      "GET /api/v1/orders/{id}/download-ticket",
    ],
  },
  {
    key: "delivery_share",
    group: "交付与验收",
    title: "共享开通页",
    path: "/delivery/orders/:orderId/share",
    viewPermission: "trade.order.read",
    primaryPermissions: ["delivery.share.enable", "delivery.share.read"],
    description: "只读共享开通与撤权入口。",
    apiBindings: [
      "/api/v1/auth/me",
      "GET /api/v1/orders/{id}",
      "GET /api/v1/orders/{id}/lifecycle-snapshots",
      "POST /api/v1/orders/{id}/share-grants",
      "GET /api/v1/orders/{id}/share-grants",
    ],
  },
  {
    key: "delivery_subscription",
    group: "交付与验收",
    title: "版本订阅页",
    path: "/delivery/orders/:orderId/subscription",
    viewPermission: "trade.order.read",
    primaryPermissions: [
      "delivery.subscription.manage",
      "delivery.subscription.read",
    ],
    description: "订阅版本、周期交付与更新轨迹入口。",
    apiBindings: [
      "/api/v1/auth/me",
      "GET /api/v1/orders/{id}",
      "GET /api/v1/orders/{id}/lifecycle-snapshots",
      "POST /api/v1/orders/{id}/subscriptions",
      "GET /api/v1/orders/{id}/subscriptions",
    ],
  },
  {
    key: "delivery_api",
    group: "交付与验收",
    title: "API 开通页",
    path: "/delivery/orders/:orderId/api",
    viewPermission: "trade.order.read",
    primaryPermissions: ["delivery.api.enable", "delivery.app.bind"],
    description: "API 凭证、应用绑定和调用入口。",
    apiBindings: [
      "/api/v1/auth/me",
      "GET /api/v1/orders/{id}",
      "GET /api/v1/orders/{id}/lifecycle-snapshots",
      "POST /api/v1/orders/{id}/deliver",
      "GET /api/v1/orders/{id}/usage-log",
    ],
  },
  {
    key: "delivery_template_query",
    group: "交付与验收",
    title: "模板查询开通页",
    path: "/delivery/orders/:orderId/template-query",
    viewPermission: "trade.order.read",
    primaryPermissions: ["delivery.template_query.enable"],
    description: "模板授权与执行入口。",
    apiBindings: [
      "/api/v1/auth/me",
      "GET /api/v1/orders/{id}",
      "GET /api/v1/orders/{id}/lifecycle-snapshots",
      "POST /api/v1/orders/{id}/template-grants",
      "GET /api/v1/orders/{id}/template-runs",
    ],
  },
  {
    key: "delivery_sandbox",
    group: "交付与验收",
    title: "查询沙箱开通页",
    path: "/delivery/orders/:orderId/sandbox",
    viewPermission: "trade.order.read",
    primaryPermissions: ["delivery.sandbox.enable"],
    description: "沙箱工作区、席位和生命周期入口。",
    apiBindings: [
      "/api/v1/auth/me",
      "GET /api/v1/orders/{id}",
      "GET /api/v1/orders/{id}/lifecycle-snapshots",
      "POST /api/v1/orders/{id}/sandbox-workspaces",
    ],
  },
  {
    key: "delivery_query_runs",
    group: "交付与验收",
    title: "查询运行与结果记录页",
    path: "/delivery/orders/:orderId/query-runs",
    viewPermission: "trade.order.read",
    primaryPermissions: ["delivery.query_run.read"],
    description: "查询执行记录、结果与审计入口。",
    apiBindings: ["/api/v1/orders/{id}/template-runs"],
  },
  {
    key: "delivery_report",
    group: "交付与验收",
    title: "报告交付页",
    path: "/delivery/orders/:orderId/report",
    viewPermission: "trade.order.read",
    primaryPermissions: ["delivery.report.commit"],
    description: "结果产品与报告包交付入口。",
    apiBindings: [
      "/api/v1/auth/me",
      "GET /api/v1/orders/{id}",
      "GET /api/v1/orders/{id}/lifecycle-snapshots",
      "POST /api/v1/orders/{id}/deliver",
    ],
  },
  {
    key: "delivery_acceptance",
    group: "交付与验收",
    title: "验收页",
    path: "/delivery/orders/:orderId/acceptance",
    viewPermission: "trade.order.read",
    primaryPermissions: ["delivery.accept.execute", "delivery.reject.execute"],
    description: "通过、拒收与生命周期摘要入口。",
    apiBindings: ["/api/v1/orders/{id}/accept", "/api/v1/orders/{id}/reject"],
  },
  {
    key: "billing_center",
    group: "账单与售后",
    title: "账单中心",
    path: "/billing",
    viewPermission: "billing.statement.read",
    primaryPermissions: ["billing.invoice.request"],
    description: "账单、支付、退款与赔付概览。",
    apiBindings: ["/api/v1/billing/{order_id}"],
  },
  {
    key: "billing_refund_compensation",
    group: "账单与售后",
    title: "退款/赔付处理页",
    path: "/billing/refunds",
    viewPermission: "billing.statement.read",
    primaryPermissions: [
      "billing.refund.execute",
      "billing.compensation.execute",
    ],
    description: "退款、赔付与二次认证入口。",
    apiBindings: ["/api/v1/refunds", "/api/v1/compensations"],
  },
  {
    key: "dispute_create",
    group: "账单与售后",
    title: "争议提交页",
    path: "/support/cases/new",
    viewPermission: "dispute.case.read",
    primaryPermissions: ["dispute.case.create", "dispute.evidence.upload"],
    description: "争议创建、证据上传与裁决入口。",
    apiBindings: ["/api/v1/cases", "/api/v1/cases/{id}/evidence"],
  },
  {
    key: "developer_home",
    group: "开发者通道",
    title: "开发者首页",
    path: "/developer",
    viewPermission: "developer.home.read",
    primaryPermissions: [],
    description: "开发者工作台、应用和调试导航。",
    apiBindings: ["/api/v1/developer/trace"],
  },
  {
    key: "developer_apps",
    group: "开发者通道",
    title: "测试应用页",
    path: "/developer/apps",
    viewPermission: "developer.app.read",
    primaryPermissions: ["developer.app.create", "developer.app.update"],
    description: "应用、API Key 与调用配置入口。",
    apiBindings: ["/api/v1/apps", "/api/v1/apps/{id}"],
  },
  {
    key: "developer_trace",
    group: "开发者通道",
    title: "状态联查页",
    path: "/developer/trace",
    viewPermission: "developer.trace.read",
    primaryPermissions: [],
    description: "订单、事件、链回执和观测联查入口。",
    apiBindings: ["/api/v1/developer/trace"],
  },
  {
    key: "developer_assets",
    group: "开发者通道",
    title: "测试资产页",
    path: "/developer/assets",
    viewPermission: "developer.test_asset.read",
    primaryPermissions: [],
    description: "测试资产、样例和开发调试资源入口。",
    apiBindings: ["/internal/dev/overview"],
  },
];

export const portalRouteMap = Object.fromEntries(
  portalRouteList.map((route) => [route.key, route]),
) as Record<PortalRouteKey, PortalRouteMeta>;

export const primaryNavigation = [
  "portal_home",
  "catalog_search",
  "seller_product_center",
  "order_create",
  "order_detail",
  "delivery_acceptance",
  "billing_center",
  "developer_home",
] satisfies PortalRouteKey[];
