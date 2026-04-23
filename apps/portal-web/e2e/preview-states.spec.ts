import { expect, test, type BrowserContext, type Page } from "@playwright/test";

test.skip(
  process.env.WEB_E2E_PREVIEW !== "1",
  "Set WEB_E2E_PREVIEW=1 with NEXT_PUBLIC_WEB_ROUTE_PREVIEW=1 to run preview-state E2E.",
);

const STANDARD_PRODUCT_ID = "20000000-0000-0000-0000-000000000309";
const STANDARD_SKU_ID = "20000000-0000-0000-0000-000000000409";
const SEEDED_ORDER_ID = "30000000-0000-0000-0000-000000000101";
const STANDARD_PRODUCT_DETAIL_RESPONSE = {
  success: true,
  data: {
    product_id: STANDARD_PRODUCT_ID,
    asset_id: "20000000-0000-0000-0000-000000000301",
    asset_version_id: "20000000-0000-0000-0000-000000000302",
    seller_org_id: "10000000-0000-0000-0000-000000000101",
    title: "工业设备运行指标 API 订阅",
    category: "industry_iot",
    product_type: "service_product",
    status: "listed",
    description: "设备稼动率、能耗与产线状态以订阅 API 方式提供。",
    price_mode: "fixed",
    price: "1999.00000000",
    currency_code: "CNY",
    delivery_type: "api_subscription",
    allowed_usage: ["internal_analysis"],
    searchable_text: null,
    subtitle: "S1 对应的正式商品详情夹具",
    industry: "industrial_manufacturing",
    use_cases: ["设备稼动率", "能耗监控"],
    data_classification: "P1",
    quality_score: "0.93",
    metadata: {
      sample_summary: "字段级样本已脱敏",
      sample_hash: "sha256:portal-web-021",
      full_hash: "sha256:portal-web-021-full",
      field_summary: "20 字段，分钟级刷新",
      field_names: ["device_id", "uptime_ratio", "energy_consumption"],
      quality_report_id: "report-web-021",
      processing_chain_summary: "iot_ingest -> normalize -> aggregate",
      data_contract_id: "contract-web-021",
    },
    search_document_version: 1,
    index_sync_status: "synced",
    skus: [
      {
        sku_id: STANDARD_SKU_ID,
        sku_code: "SKU-API-SUB-WEB021",
        sku_type: "API_SUB",
        billing_mode: "subscription",
        trade_mode: "api_subscription",
        acceptance_mode: "api_open",
        refund_mode: "billing_adjustment",
        unit_name: "月",
        status: "active",
      },
      {
        sku_id: "20000000-0000-0000-0000-000000000410",
        sku_code: "SKU-API-PPU-WEB021",
        sku_type: "API_PPU",
        billing_mode: "usage",
        trade_mode: "api_pay_per_use",
        acceptance_mode: "api_open",
        refund_mode: "billing_adjustment",
        unit_name: "次",
        status: "active",
      },
    ],
    created_at: "2026-01-01T00:00:00.000Z",
    updated_at: "2026-01-01T00:00:00.000Z",
  },
};
const STANDARD_SCENARIOS_RESPONSE = {
  success: true,
  data: [
    {
      scenario_code: "S1",
      scenario_name: "工业设备运行指标 API 订阅",
      primary_sku: "API_SUB",
      supplementary_skus: ["API_PPU"],
    },
    {
      scenario_code: "S2",
      scenario_name: "工业质量与产线日报文件包交付",
      primary_sku: "FILE_STD",
      supplementary_skus: ["FILE_SUB"],
    },
    {
      scenario_code: "S3",
      scenario_name: "供应链协同查询沙箱",
      primary_sku: "SBX_STD",
      supplementary_skus: ["SHARE_RO"],
    },
    {
      scenario_code: "S4",
      scenario_name: "零售门店经营分析 API / 报告订阅",
      primary_sku: "API_SUB",
      supplementary_skus: ["RPT_STD"],
    },
    {
      scenario_code: "S5",
      scenario_name: "商圈/门店选址查询服务",
      primary_sku: "QRY_LITE",
      supplementary_skus: ["RPT_STD"],
    },
  ],
};
const SELLER_PROFILE_RESPONSE = {
  success: true,
  data: {
    org_id: "10000000-0000-0000-0000-000000000101",
    org_name: "Luna Industrial Data Lab",
    description: "工业制造领域供方夹具。",
    org_type: "enterprise",
    status: "active",
    country_code: "CN",
    region_code: "SH",
    credit_level: "A",
    risk_level: "L2",
    reputation_score: "4.8",
    listed_product_count: 12,
    index_sync_status: "synced",
    search_document_version: 3,
    industry_tags: ["industrial_manufacturing", "iot"],
  },
};
const EMPTY_RECOMMENDATIONS_RESPONSE = {
  success: true,
  data: {
    items: [],
  },
};
const restrictedPorts = new Set([
  "5432",
  "6379",
  "7050",
  "7051",
  "8080",
  "8094",
  "9000",
  "9092",
  "9094",
  "9200",
  "9300",
  "18080",
]);
const restrictedHostFragments = [
  "postgres",
  "kafka",
  "opensearch",
  "redis",
  "fabric",
];

function watchRestrictedBrowserRequests(page: Page) {
  const hits: string[] = [];
  page.on("request", (request) => {
    try {
      const url = new URL(request.url());
      if (
        restrictedPorts.has(url.port) ||
        restrictedHostFragments.some((fragment) =>
          url.hostname.toLowerCase().includes(fragment),
        )
      ) {
        hits.push(request.url());
      }
    } catch {
      hits.push(request.url());
    }
  });
  return hits;
}

function encodeBase64Url(value: unknown) {
  return Buffer.from(JSON.stringify(value), "utf8").toString("base64url");
}

function createUnsignedJwt(payload: Record<string, unknown>) {
  return [
    encodeBase64Url({ alg: "none", typ: "JWT" }),
    encodeBase64Url(payload),
    "web018",
  ].join(".");
}

async function installPortalBearerSession(context: BrowserContext) {
  const token = createUnsignedJwt({
    exp: Math.floor(Date.now() / 1000) + 3600,
    name: "WEB018 Buyer Operator",
    org_id: "10000000-0000-0000-0000-000000000102",
    preferred_username: "web018-buyer-operator",
    realm_access: { roles: ["buyer_operator"] },
    user_id: "10000000-0000-0000-0000-000000000356",
  });
  await context.addCookies([
    {
      name: "datab_portal_session",
      value: encodeBase64Url({
        mode: "bearer",
        accessToken: token,
        label: "WEB-018 fake buyer claims",
      }),
      url: "http://127.0.0.1:3101",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

test("portal home links directly to five standard demo paths", async ({ page }) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);
  const scenarios = [
    ["S1", "工业设备运行指标 API 订阅", "API_SUB", "API_PPU"],
    ["S2", "工业质量与产线日报文件包交付", "FILE_STD", "FILE_SUB"],
    ["S3", "供应链协同查询沙箱", "SBX_STD", "SHARE_RO"],
    ["S4", "零售门店经营分析 API / 报告订阅", "API_SUB", "RPT_STD"],
    ["S5", "商圈/门店选址查询服务", "QRY_LITE", "RPT_STD"],
  ] as const;

  for (const [scenarioCode, scenarioName, primarySku, supplementarySku] of scenarios) {
    await page.goto("/");
    await page
      .getByRole("link", { name: `查看 ${scenarioCode} 演示路径` })
      .click();
    await expect(page).toHaveURL(`/demos/${scenarioCode}`);
    await expect(
      page.getByRole("heading", {
        name: `${scenarioName}演示路径`,
        exact: true,
      }),
    ).toBeVisible();
    await expect(page.getByText("说明卡片", { exact: true })).toBeVisible();
    await expect(
      page.getByText("GET /api/v1/catalog/standard-scenarios").first(),
    ).toBeVisible();
    await expect(
      page.getByText("场景名 -> 主 SKU / 补充 SKU").first(),
    ).toBeVisible();
    await expect(page.getByText(`主 SKU ${primarySku}`).first()).toBeVisible();
    await expect(
      page.getByText(`补充 SKU ${supplementarySku}`).first(),
    ).toBeVisible();
    await expect(page.getByText("Idempotency-Key").first()).toBeVisible();
  }
  expect(restrictedRequests).toEqual([]);
});

test("portal home and scaffold pages are reachable", async ({ page }) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);
  await page.goto("/");
  await expect(page.getByText("门户首页已接入场景导航、推荐位与受控搜索入口。")).toBeVisible();
  await expect(page.getByRole("heading", { name: "标准链路快捷入口", exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "受控搜索入口", exact: true })).toBeVisible();
  await expect(page.getByText("当前主体 / 角色 / 租户 / 作用域")).toBeVisible();

  await page.goto("/search?preview=forbidden");
  await expect(page.getByText("搜索权限态")).toBeVisible();
  await expect(
    page.getByText("需要权限：`portal.search.read` / `portal.search.use`", {
      exact: true,
    }),
  ).toBeVisible();

  await page.goto("/search?preview=empty");
  await expect(page.getByText("没有匹配的搜索结果")).toBeVisible();

  await page.goto("/search?preview=error");
  await expect(page.getByText("SEARCH_BACKEND_UNAVAILABLE")).toBeVisible();

  await page.goto("/products/20000000-0000-0000-0000-000000000309?preview=forbidden");
  await expect(page.getByText("商品详情权限态")).toBeVisible();
  await expect(
    page.getByText("需要权限：`catalog.product.read`；主动作权限：`trade.order.create`", {
      exact: true,
    }),
  ).toBeVisible();

  await page.goto("/products/20000000-0000-0000-0000-000000000309?preview=empty");
  await expect(page.getByText("没有可展示的商品详情")).toBeVisible();

  await page.goto("/sellers/10000000-0000-0000-0000-000000000101?preview=forbidden");
  await expect(page.getByText("卖方主页权限态")).toBeVisible();
  await expect(
    page.getByText("需要权限：`portal.seller.read`；当前会话模式 guest，角色 无。", {
      exact: true,
    }),
  ).toBeVisible();

  await page.goto("/sellers/10000000-0000-0000-0000-000000000101?preview=empty");
  await expect(page.getByText("没有可展示的卖方主页")).toBeVisible();

  await page.goto("/sellers/10000000-0000-0000-0000-000000000101?preview=error");
  await expect(page.getByText("CAT_VALIDATION_FAILED")).toBeVisible();

  await page.goto("/seller/products?preview=forbidden");
  await expect(page.getByText("上架中心权限态")).toBeVisible();
  await expect(
    page.getByText("需要权限：catalog.product.list", { exact: false }),
  ).toBeVisible();

  await page.goto("/seller/products?preview=empty");
  await expect(page.getByText("没有商品草稿")).toBeVisible();
  await expect(page.getByText("POST /api/v1/products", { exact: false })).toBeVisible();

  await page.goto("/seller/products?preview=error");
  await expect(page.getByText("上架中心错误态")).toBeVisible();
  await expect(page.getByText("CAT_VALIDATION_FAILED", { exact: false })).toBeVisible();

  await page.goto("/seller/products/20000000-0000-0000-0000-000000000309/skus?preview=forbidden");
  await expect(page.getByText("上架中心权限态")).toBeVisible();
  await expect(
    page.getByText("catalog.sku.create", { exact: false }).last(),
  ).toBeVisible();

  await page.goto("/trade/orders/new?preview=forbidden");
  await expect(page.getByText("下单权限态")).toBeVisible();
  await expect(page.getByText("五条标准链路下单入口")).toBeVisible();

  await page.goto("/trade/orders/new?preview=empty");
  await expect(page.getByText("请选择商品后下单")).toBeVisible();
  await expect(page.getByText("工业设备运行指标 API 订阅").first()).toBeVisible();

  await page.goto("/trade/orders/new?preview=error");
  await expect(page.getByText("订单创建错误态")).toBeVisible();
  await expect(page.getByText("ORDER_CREATE_FORBIDDEN", { exact: false })).toBeVisible();

  await page.goto("/trade/orders/30000000-0000-0000-0000-000000000901?preview=forbidden");
  await expect(page.getByText("订单详情权限态")).toBeVisible();

  await page.goto("/trade/orders/30000000-0000-0000-0000-000000000901?preview=empty");
  await expect(page.getByText("没有可展示的订单详情")).toBeVisible();

  await page.goto("/trade/orders/30000000-0000-0000-0000-000000000901?preview=error");
  await expect(page.getByText("订单详情错误态")).toBeVisible();
  await expect(page.getByText("TRD_STATE_CONFLICT", { exact: false })).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/file?preview=forbidden");
  await expect(page.getByText("交付中心权限态")).toBeVisible();
  await expect(page.getByText("delivery.file.commit", { exact: false }).first()).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/file?preview=empty");
  await expect(page.getByText("没有可展示的交付数据")).toBeVisible();
  await expect(page.getByRole("link", { name: "文件 FILE_STD" })).toBeVisible();
  await expect(page.getByRole("link", { name: "模板查询 QRY_LITE" })).toBeVisible();
  await expect(page.getByRole("link", { name: "报告 RPT_STD" })).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/api?preview=error");
  await expect(page.getByText("API 开通错误态")).toBeVisible();
  await expect(page.getByText("DELIVERY_STATUS_INVALID", { exact: false })).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/share?preview=forbidden");
  await expect(page.getByText("delivery.share.enable", { exact: false }).first()).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/template-query?preview=empty");
  await expect(page.getByRole("heading", { name: "模板查询授权" })).toBeVisible();
  await expect(page.getByText("POST /api/v1/orders/{id}/template-grants")).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/sandbox?preview=loading");
  await expect(page.getByText("查询沙箱开通加载态")).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/report?preview=forbidden");
  await expect(page.getByText("delivery.report.commit", { exact: false }).first()).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/acceptance?preview=forbidden");
  await expect(page.getByText("验收页权限态")).toBeVisible();
  await expect(page.getByText("delivery.accept.execute", { exact: false }).first()).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/acceptance?preview=empty");
  await expect(page.getByText("没有可展示的验收数据")).toBeVisible();

  await page.goto("/delivery/orders/30000000-0000-0000-0000-000000000901/acceptance?preview=error");
  await expect(page.getByText("验收页错误态")).toBeVisible();
  await expect(page.getByText("DELIVERY_STATUS_INVALID", { exact: false })).toBeVisible();

  await page.goto("/billing?preview=forbidden");
  await expect(page.getByText("账单页面权限态")).toBeVisible();
  await expect(page.getByText("billing.statement.read", { exact: false }).first()).toBeVisible();

  await page.goto("/billing?preview=empty");
  await expect(page.getByText("没有可展示的账单数据")).toBeVisible();

  await page.goto("/billing?preview=error");
  await expect(page.getByText("账单中心错误态")).toBeVisible();
  await expect(page.getByText("BIL_PROVIDER_FAILED", { exact: false })).toBeVisible();

  await page.goto("/billing/refunds?preview=forbidden");
  await expect(page.getByText("退款/赔付按钮权限不足")).toBeVisible();
  await expect(page.getByText("platform_risk_settlement", { exact: false })).toBeVisible();

  await page.goto("/billing/refunds?preview=empty");
  await expect(page.getByText("请输入 order_id 与 case_id 后执行退款/赔付")).toBeVisible();

  await page.goto("/support/cases/new?preview=forbidden");
  await expect(page.getByText("争议页面权限态")).toBeVisible();
  await expect(page.getByText("dispute.case.read", { exact: false }).first()).toBeVisible();

  await page.goto("/support/cases/new?preview=empty");
  await expect(page.getByText("请输入 order_id 创建或跟踪争议")).toBeVisible();

  await page.goto("/support/cases/new?preview=error");
  await expect(page.getByText("争议页错误态")).toBeVisible();
  await expect(page.getByText("DISPUTE_STATUS_INVALID", { exact: false })).toBeVisible();

  await page.goto("/developer");
  await expect(page.getByRole("heading", { name: "开发者工作台", exact: true })).toBeVisible();
  await expect(page.getByText("当前主体 / 角色 / 租户 / 作用域").first()).toBeVisible();

  await page.goto("/developer/apps");
  await expect(page.getByText("应用管理与 API Key")).toBeVisible();
  await expect(page.getByText("应用列表")).toBeVisible();
  await expect(page.getByText("Idempotency-Key").first()).toBeVisible();

  await page.goto("/developer/trace");
  await expect(page.getByText("Trace 与调用日志联查")).toBeVisible();
  await expect(page.getByText("request_id").first()).toBeVisible();

  await page.goto("/developer/assets");
  await expect(page.getByRole("heading", { name: "Mock 支付操作入口", exact: true })).toBeVisible();
  await expect(page.getByText("developer.mock_payment.simulate").first()).toBeVisible();
  expect(restrictedRequests).toEqual([]);
});

test("WEB-018 portal user flow covers login, search, product, order, delivery, acceptance and linkage", async ({
  context,
  page,
}) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);

  await page.goto("/");
  await page.getByRole("button", { name: "认证会话" }).click();
  await expect(page.getByText("Keycloak / IAM Session")).toBeVisible();
  await page.getByRole("button", { name: "Local Header" }).click();
  await expect(page.getByPlaceholder("buyer.operator@luna.local")).toHaveValue(
    "buyer.operator@luna.local",
  );
  await expect(page.locator('select[name="role"]')).toHaveValue("buyer_operator");
  await expect(page.getByPlaceholder("10000000-0000-0000-0000-000000000102")).toHaveValue(
    "10000000-0000-0000-0000-000000000102",
  );
  await page.getByRole("button", { name: "关闭" }).click();

  await installPortalBearerSession(context);
  await page.goto("/");
  await expect(page.getByText("主体 WEB018 Buyer Operator")).toBeVisible();
  await expect(page.getByText("角色 buyer_operator")).toBeVisible();
  await expect(
    page.getByText("租户 10000000-0000-0000-0000-000000000102"),
  ).toBeVisible();
  await expect(page.getByText("作用域 aal1")).toBeVisible();

  await page.goto("/search?preview=empty&q=工业设备运行指标");
  await expect(page.getByText("没有匹配的搜索结果")).toBeVisible();
  await expect(page.getByText("SEARCH_BACKEND_UNAVAILABLE")).toBeHidden();

  await page.goto(`/products/${STANDARD_PRODUCT_ID}?preview=empty`);
  await expect(page.getByText("没有可展示的商品详情")).toBeVisible();
  await expect(page.getByText(STANDARD_PRODUCT_ID).first()).toBeVisible();

  await page.goto(
    `/trade/orders/new?product_id=${STANDARD_PRODUCT_ID}&sku_id=${STANDARD_SKU_ID}&scenario=S1&preview=empty`,
  );
  await expect(page.getByText("五条标准链路下单入口")).toBeVisible();
  await expect(page.getByText("场景名 -> 主 SKU / 补充 SKU").first()).toBeVisible();
  await expect(page.getByText("工业设备运行指标 API 订阅").first()).toBeVisible();
  await expect(page.getByText("API_SUB + API_PPU").first()).toBeVisible();

  await page.goto(`/trade/orders/${SEEDED_ORDER_ID}?preview=empty`);
  await expect(page.getByText("没有可展示的订单详情")).toBeVisible();

  await page.goto(`/delivery/orders/${SEEDED_ORDER_ID}/file?preview=empty`);
  await expect(page.getByText("没有可展示的交付数据")).toBeVisible();
  await expect(page.getByRole("link", { name: "文件 FILE_STD" })).toBeVisible();

  await page.goto(`/delivery/orders/${SEEDED_ORDER_ID}/acceptance?preview=empty`);
  await expect(page.getByText("没有可展示的验收数据")).toBeVisible();

  await page.goto("/support/cases/new?preview=empty");
  await expect(page.getByText("请输入 order_id 创建或跟踪争议")).toBeVisible();

  await page.goto(`/developer/trace?order_id=${SEEDED_ORDER_ID}`);
  await expect(page.getByText("Trace 与调用日志联查")).toBeVisible();
  await expect(page.getByText("request_id").first()).toBeVisible();

  expect(restrictedRequests).toEqual([]);
});

test("portal product detail renders standard scenario mapping from platform api", async ({
  context,
  page,
}) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);

  await installPortalBearerSession(context);
  await page.route("**/api/platform/api/v1/products/*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(STANDARD_PRODUCT_DETAIL_RESPONSE),
    });
  });
  await page.route("**/api/platform/api/v1/catalog/standard-scenarios", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(STANDARD_SCENARIOS_RESPONSE),
    });
  });
  await page.route("**/api/platform/api/v1/sellers/*/profile", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(SELLER_PROFILE_RESPONSE),
    });
  });
  await page.route("**/api/platform/api/v1/recommendations*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(EMPTY_RECOMMENDATIONS_RESPONSE),
    });
  });

  await page.goto(`/products/${STANDARD_PRODUCT_ID}`);

  await expect(page.getByText("商品可承接的标准链路")).toBeVisible();
  await expect(
    page.getByText("场景名 -> 主 SKU / 补充 SKU").first(),
  ).toBeVisible();
  await expect(page.getByText("standard-scenarios live")).toBeVisible();
  await expect(page.getByText("工业设备运行指标 API 订阅").first()).toBeVisible();
  await expect(
    page.getByText("当前商品已完整命中该标准链路的主 SKU / 补充 SKU，可携带 scenario_code 进入下单。"),
  ).toBeVisible();
  await expect(
    page.getByText("当前商品已命中主 SKU，但仍缺少部分补充 SKU；下单时仍以实际 SKU 快照为准。"),
  ).toBeVisible();
  await expect(page.getByText("待补充 SKU").first()).toBeVisible();
  await expect(page.getByText("RPT_STD").first()).toBeVisible();

  expect(restrictedRequests).toEqual([]);
});
