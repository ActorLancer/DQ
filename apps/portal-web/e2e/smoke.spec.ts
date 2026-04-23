import { expect, test } from "@playwright/test";

test("portal home links directly to five standard demo paths", async ({ page }) => {
  const scenarios = [
    ["S1", "工业设备运行指标 API 订阅"],
    ["S2", "工业质量与产线日报文件包交付"],
    ["S3", "供应链协同查询沙箱"],
    ["S4", "零售门店经营分析 API / 报告订阅"],
    ["S5", "商圈/门店选址查询服务"],
  ] as const;

  for (const [scenarioCode, scenarioName] of scenarios) {
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
    await expect(page.getByText("Idempotency-Key").first()).toBeVisible();
  }
});

test("portal home and scaffold pages are reachable", async ({ page }) => {
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
  await expect(page.getByText("DLV_STATE_CONFLICT", { exact: false })).toBeVisible();

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
  await expect(page.getByText("TRD_STATE_CONFLICT", { exact: false })).toBeVisible();

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
});
