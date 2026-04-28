import type { StandardScenariosResponse } from "@datab/sdk-ts";

export const officialSkuOrder = [
  "FILE_STD",
  "FILE_SUB",
  "SHARE_RO",
  "API_SUB",
  "API_PPU",
  "QRY_LITE",
  "SBX_STD",
  "RPT_STD",
] as const;

export type OfficialSku = (typeof officialSkuOrder)[number];
export type StandardScenarioCode = "S1" | "S2" | "S3" | "S4" | "S5";
export type StandardDemoRouteKey =
  | "standard_demo_s1"
  | "standard_demo_s2"
  | "standard_demo_s3"
  | "standard_demo_s4"
  | "standard_demo_s5";
export type StandardScenarioTemplate = StandardScenariosResponse["data"][number];

export interface StandardDemoGuide {
  scenarioCode: StandardScenarioCode;
  routeKey: StandardDemoRouteKey;
  path: `/demos/${StandardScenarioCode}`;
  scenarioName: string;
  industryCode: "industrial_manufacturing" | "retail";
  industryLabel: string;
  primarySku: OfficialSku;
  supplementarySkus: readonly OfficialSku[];
  deliveryType: string;
  productType: string;
  category: string;
  dataClassification: "P1" | "P2";
  searchKeyword: string;
  summary: string;
  buyerOutcome: string;
  sellerSignal: string;
  riskHint: string;
  contractTemplate: string;
  acceptanceTemplate: string;
  refundTemplate: string;
  reviewActionName: string;
  reviewActionReason: string;
  deliveryLinks: readonly {
    label: string;
    href: string;
    sku: OfficialSku;
  }[];
  steps: readonly {
    label: string;
    href: string;
    permission: string;
    description: string;
  }[];
}

export const standardDemoGuides = [
  {
    scenarioCode: "S1",
    routeKey: "standard_demo_s1",
    path: "/demos/S1",
    scenarioName: "工业设备运行指标 API 订阅",
    industryCode: "industrial_manufacturing",
    industryLabel: "工业制造",
    primarySku: "API_SUB",
    supplementarySkus: ["API_PPU"],
    deliveryType: "api_subscription",
    productType: "service_product",
    category: "industry_iot",
    dataClassification: "P1",
    searchKeyword: "工业设备运行指标 API 订阅",
    summary: "设备稼动率、能耗与产线状态以订阅 API 方式提供，按周期授权、计费和审计。",
    buyerOutcome: "买方从首页进入搜索，查看 API SLA 与样例字段后创建 API_SUB 订单。",
    sellerSignal: "卖方需要在上架中心配置 API_SUB 主 SKU，并可附加 API_PPU 计量能力。",
    riskHint: "API 凭证开通后必须进入调用日志与用量账单，不把 API_PPU 并回 API_SUB。",
    contractTemplate: "CONTRACT_API_SUB_V1",
    acceptanceTemplate: "ACCEPT_API_SUB_V1",
    refundTemplate: "REFUND_API_SUB_V1",
    reviewActionName: "approve",
    reviewActionReason: "api_schema_and_sla_verified",
    deliveryLinks: [
      {
        label: "API 开通",
        href: "/delivery/orders/demo-order/api",
        sku: "API_SUB",
      },
    ],
    steps: [
      {
        label: "首页直达",
        href: "/",
        permission: "portal.home.read",
        description: "首页说明卡片直接进入本演示路径，保持 S1 官方命名。",
      },
      {
        label: "搜索同类商品",
        href: "/search?q=%E5%B7%A5%E4%B8%9A%E8%AE%BE%E5%A4%87%E8%BF%90%E8%A1%8C%E6%8C%87%E6%A0%87&scenario=S1",
        permission: "portal.search.read / portal.search.use",
        description: "搜索页按 API 订阅、行业和标准 SKU 筛选候选商品。",
      },
      {
        label: "产品详情",
        href: "/products/demo-product-S1",
        permission: "catalog.product.read",
        description: "详情页确认字段、SLA、价格快照和 API_SUB / API_PPU SKU。",
      },
      {
        label: "下单预设",
        href: "/trade/orders/new?scenario=S1",
        permission: "trade.order.create",
        description: "订单创建时冻结合同、验收和退款模板，写操作携带 Idempotency-Key。",
      },
      {
        label: "交付与账单",
        href: "/delivery/orders/demo-order/api",
        permission: "delivery.api.enable",
        description: "开通 API 凭证，后续进入用量日志、账单和审计联查。",
      },
    ],
  },
  {
    scenarioCode: "S2",
    routeKey: "standard_demo_s2",
    path: "/demos/S2",
    scenarioName: "工业质量与产线日报文件包交付",
    industryCode: "industrial_manufacturing",
    industryLabel: "工业制造",
    primarySku: "FILE_STD",
    supplementarySkus: ["FILE_SUB"],
    deliveryType: "file_download",
    productType: "data_product",
    category: "industrial_quality",
    dataClassification: "P1",
    searchKeyword: "工业质量与产线日报文件包交付",
    summary: "质量日报、巡检记录和产线摘要以标准文件包交付，文件订阅独立表达。",
    buyerOutcome: "买方确认样例、哈希和下载票据后，进入 FILE_STD 订单和人工验收。",
    sellerSignal: "卖方需要配置 FILE_STD 主 SKU，可为周期性日报增加 FILE_SUB 订阅 SKU。",
    riskHint: "下载类页面只展示受控票据，不暴露真实对象路径或存储桶地址。",
    contractTemplate: "CONTRACT_FILE_V1",
    acceptanceTemplate: "ACCEPT_FILE_V1",
    refundTemplate: "REFUND_FILE_V1",
    reviewActionName: "approve",
    reviewActionReason: "file_hash_and_preview_checked",
    deliveryLinks: [
      {
        label: "文件交付",
        href: "/delivery/orders/demo-order/file",
        sku: "FILE_STD",
      },
      {
        label: "版本订阅",
        href: "/delivery/orders/demo-order/subscription",
        sku: "FILE_SUB",
      },
    ],
    steps: [
      {
        label: "首页直达",
        href: "/",
        permission: "portal.home.read",
        description: "首页说明卡片直接进入本演示路径，展示 FILE_STD / FILE_SUB。",
      },
      {
        label: "搜索同类商品",
        href: "/search?q=%E5%B7%A5%E4%B8%9A%E8%B4%A8%E9%87%8F%E4%B8%8E%E4%BA%A7%E7%BA%BF%E6%97%A5%E6%8A%A5&scenario=S2",
        permission: "portal.search.read / portal.search.use",
        description: "搜索页按文件交付、质量日报和产线关键词筛选。",
      },
      {
        label: "产品详情",
        href: "/products/demo-product-S2",
        permission: "catalog.product.read",
        description: "详情页展示样例预览、文件哈希、大小、更新周期和授权范围。",
      },
      {
        label: "下单预设",
        href: "/trade/orders/new?scenario=S2",
        permission: "trade.order.create",
        description: "订单创建冻结 FILE_STD / FILE_SUB 快照与文件验收模板。",
      },
      {
        label: "交付与验收",
        href: "/delivery/orders/demo-order/file",
        permission: "delivery.file.commit / delivery.accept.execute",
        description: "文件交付后展示下载票据、验真结果和人工验收入口。",
      },
    ],
  },
  {
    scenarioCode: "S3",
    routeKey: "standard_demo_s3",
    path: "/demos/S3",
    scenarioName: "供应链协同查询沙箱",
    industryCode: "industrial_manufacturing",
    industryLabel: "工业制造",
    primarySku: "SBX_STD",
    supplementarySkus: ["SHARE_RO"],
    deliveryType: "sandbox",
    productType: "service_product",
    category: "supply_chain",
    dataClassification: "P2",
    searchKeyword: "供应链协同查询沙箱",
    summary: "买方进入受控查询沙箱查看履约、库存和协同指标，必要时开通只读共享。",
    buyerOutcome: "买方通过 SBX_STD 获得沙箱工作区，不获取原始底层数据副本。",
    sellerSignal: "卖方需要配置沙箱护栏、席位、查询限额和 SHARE_RO 共享边界。",
    riskHint: "SHARE_RO 是独立共享 SKU，必须绑定共享模板族，不得并入文件或 API 大类。",
    contractTemplate: "CONTRACT_SANDBOX_V1",
    acceptanceTemplate: "ACCEPT_SANDBOX_V1",
    refundTemplate: "REFUND_SANDBOX_V1",
    reviewActionName: "approve",
    reviewActionReason: "sandbox_guardrail_verified",
    deliveryLinks: [
      {
        label: "查询沙箱",
        href: "/delivery/orders/demo-order/sandbox",
        sku: "SBX_STD",
      },
      {
        label: "只读共享",
        href: "/delivery/orders/demo-order/share",
        sku: "SHARE_RO",
      },
    ],
    steps: [
      {
        label: "首页直达",
        href: "/",
        permission: "portal.home.read",
        description: "首页说明卡片直接进入本演示路径，突出非副本受控使用。",
      },
      {
        label: "搜索同类商品",
        href: "/search?q=%E4%BE%9B%E5%BA%94%E9%93%BE%E5%8D%8F%E5%90%8C&scenario=S3",
        permission: "portal.search.read / portal.search.use",
        description: "搜索页按供应链协同、沙箱和只读共享能力筛选。",
      },
      {
        label: "产品详情",
        href: "/products/demo-product-S3",
        permission: "catalog.product.read",
        description: "详情页确认沙箱护栏、可查指标、导出限制和共享模板。",
      },
      {
        label: "下单预设",
        href: "/trade/orders/new?scenario=S3",
        permission: "trade.order.create",
        description: "订单创建冻结 SBX_STD 与 SHARE_RO 的独立授权和结算快照。",
      },
      {
        label: "沙箱开通",
        href: "/delivery/orders/demo-order/sandbox",
        permission: "delivery.sandbox.enable",
        description: "交付中心开通沙箱工作区并展示链状态、投影状态和审计提示。",
      },
    ],
  },
  {
    scenarioCode: "S4",
    routeKey: "standard_demo_s4",
    path: "/demos/S4",
    scenarioName: "零售门店经营分析 API / 报告订阅",
    industryCode: "retail",
    industryLabel: "零售经营",
    primarySku: "API_SUB",
    supplementarySkus: ["RPT_STD"],
    deliveryType: "api_subscription",
    productType: "service_product",
    category: "retail_ops",
    dataClassification: "P1",
    searchKeyword: "零售门店经营分析 API / 报告订阅",
    summary: "门店客流、销售结构和经营指标以 API 订阅为主，可叠加标准报告结果产品。",
    buyerOutcome: "买方订阅经营分析 API，并按需接收 RPT_STD 报告交付。",
    sellerSignal: "卖方需要分别配置 API_SUB 与 RPT_STD，不把报告交付混成 API 附件。",
    riskHint: "RPT_STD 必须走结果产品模板族，报告下载仍不得暴露真实对象路径。",
    contractTemplate: "CONTRACT_API_SUB_V1",
    acceptanceTemplate: "ACCEPT_API_SUB_V1",
    refundTemplate: "REFUND_API_SUB_V1",
    reviewActionName: "approve",
    reviewActionReason: "api_report_combo_check_passed",
    deliveryLinks: [
      {
        label: "API 开通",
        href: "/delivery/orders/demo-order/api",
        sku: "API_SUB",
      },
      {
        label: "报告交付",
        href: "/delivery/orders/demo-order/report",
        sku: "RPT_STD",
      },
    ],
    steps: [
      {
        label: "首页直达",
        href: "/",
        permission: "portal.home.read",
        description: "首页说明卡片直接进入本演示路径，展示 API / 报告组合。",
      },
      {
        label: "搜索同类商品",
        href: "/search?q=%E9%97%A8%E5%BA%97%E7%BB%8F%E8%90%A5%E5%88%86%E6%9E%90&scenario=S4",
        permission: "portal.search.read / portal.search.use",
        description: "搜索页按零售经营、API 订阅和标准报告筛选。",
      },
      {
        label: "产品详情",
        href: "/products/demo-product-S4",
        permission: "catalog.product.read",
        description: "详情页展示 API 指标、报告样例、更新周期和组合价格。",
      },
      {
        label: "下单预设",
        href: "/trade/orders/new?scenario=S4",
        permission: "trade.order.create",
        description: "订单创建必须明确 API_SUB 与 RPT_STD 的独立授权快照。",
      },
      {
        label: "API 与报告",
        href: "/delivery/orders/demo-order/api",
        permission: "delivery.api.enable / delivery.report.commit",
        description: "API 开通和报告交付分别进入对应交付页、账单和审计链路。",
      },
    ],
  },
  {
    scenarioCode: "S5",
    routeKey: "standard_demo_s5",
    path: "/demos/S5",
    scenarioName: "商圈/门店选址查询服务",
    industryCode: "retail",
    industryLabel: "零售经营",
    primarySku: "QRY_LITE",
    supplementarySkus: ["RPT_STD"],
    deliveryType: "query_template",
    productType: "service_product",
    category: "retail_location",
    dataClassification: "P1",
    searchKeyword: "商圈/门店选址查询服务",
    summary: "买方使用模板查询获得商圈评分、门店画像和选址建议，可导出标准报告。",
    buyerOutcome: "买方获得 QRY_LITE 模板查询授权，查询结果按边界返回，不暴露底层明细。",
    sellerSignal: "卖方需要配置查询面、模板白名单、参数边界和 RPT_STD 结果报告。",
    riskHint: "QRY_LITE 是独立模板查询 SKU，必须绑定模板查询模板族，不得并入报告大类。",
    contractTemplate: "CONTRACT_QUERY_LITE_V1",
    acceptanceTemplate: "ACCEPT_QUERY_LITE_V1",
    refundTemplate: "REFUND_QUERY_LITE_V1",
    reviewActionName: "approve",
    reviewActionReason: "template_boundary_validated",
    deliveryLinks: [
      {
        label: "模板查询",
        href: "/delivery/orders/demo-order/template-query",
        sku: "QRY_LITE",
      },
      {
        label: "报告交付",
        href: "/delivery/orders/demo-order/report",
        sku: "RPT_STD",
      },
    ],
    steps: [
      {
        label: "首页直达",
        href: "/",
        permission: "portal.home.read",
        description: "首页说明卡片直接进入本演示路径，突出模板查询边界。",
      },
      {
        label: "搜索同类商品",
        href: "/search?q=%E5%95%86%E5%9C%88%E9%80%89%E5%9D%80&scenario=S5",
        permission: "portal.search.read / portal.search.use",
        description: "搜索页按商圈、选址、模板查询和报告能力筛选。",
      },
      {
        label: "产品详情",
        href: "/products/demo-product-S5",
        permission: "catalog.product.read",
        description: "详情页展示模板参数、结果边界、额度、样例报告和导出限制。",
      },
      {
        label: "下单预设",
        href: "/trade/orders/new?scenario=S5",
        permission: "trade.order.create",
        description: "订单创建冻结 QRY_LITE 模板授权与 RPT_STD 报告交付快照。",
      },
      {
        label: "模板查询授权",
        href: "/delivery/orders/demo-order/template-query",
        permission: "delivery.template_query.enable",
        description: "交付中心展示模板白名单、参数边界、运行记录和受控导出。",
      },
    ],
  },
] as const satisfies readonly StandardDemoGuide[];

export const frozenStandardScenarios: StandardScenarioTemplate[] =
  standardDemoGuides.map((guide) => ({
    scenario_code: guide.scenarioCode,
    scenario_name: guide.scenarioName,
    primary_sku: guide.primarySku,
    supplementary_skus: [...guide.supplementarySkus],
    product_template: {
      category: guide.category,
      delivery_type: guide.deliveryType,
      product_type: guide.productType,
    },
    metadata_template: {
      data_classification: guide.dataClassification,
      industry: guide.industryCode,
      use_cases: [guide.searchKeyword, guide.buyerOutcome],
    },
    contract_template: guide.contractTemplate,
    acceptance_template: guide.acceptanceTemplate,
    refund_template: guide.refundTemplate,
    review_sample: {
      action_name: guide.reviewActionName,
      action_reason: guide.reviewActionReason,
    },
  }));

export function findStandardDemoGuide(
  scenarioCode: string,
): StandardDemoGuide | undefined {
  const normalized = scenarioCode.toUpperCase();
  return standardDemoGuides.find((guide) => guide.scenarioCode === normalized);
}

export function collectStandardDemoSkuCoverage(): OfficialSku[] {
  const seen = new Set<string>();
  for (const guide of standardDemoGuides) {
    seen.add(guide.primarySku);
    guide.supplementarySkus.forEach((sku) => seen.add(sku));
  }
  return officialSkuOrder.filter((sku) => seen.has(sku));
}

export function buildStandardDemoHref(scenarioCode: string) {
  return findStandardDemoGuide(scenarioCode)?.path ?? "/demos/S1";
}
