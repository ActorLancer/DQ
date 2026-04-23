import { expect, test } from "@playwright/test";

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
});
