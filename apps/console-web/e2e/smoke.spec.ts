import { expect, test } from "@playwright/test";

test("console home and scaffold pages are reachable", async ({ page }) => {
  await page.goto("/");
  await expect(
    page.getByText("控制台工程基线已接入正式路由、受控 API 代理和控制面登录态占位。"),
  ).toBeVisible();
  await expect(page.getByText("当前主体 / 角色 / 租户 / 作用域")).toBeVisible();

  await page.goto("/ops/audit/trace?preview=empty");
  await expect(page.getByText("空态预演")).toBeVisible();

  await page.goto("/ops/audit/trace");
  await expect(
    page.getByText("审计事件、链回执、外部事实和证据包导出在控制台闭环。"),
  ).toBeVisible();
  await expect(page.getByText("WEB-014")).toBeVisible();

  await page.goto("/ops/audit/packages?preview=forbidden");
  await expect(page.getByText("权限态预演")).toBeVisible();

  await page.goto("/developer/apps?preview=forbidden");
  await expect(page.getByText("权限态预演")).toBeVisible();

  await page.goto("/ops/review/subjects?preview=empty");
  await expect(page.getByText("空态预演")).toBeVisible();

  await page.goto("/ops/review/products");
  await expect(
    page.getByText("审核队列、详情、权限和决策写入在同一工作台闭环。"),
  ).toBeVisible();
  await expect(
    page.getByText("GET /api/v1/products?status=pending_review"),
  ).toBeVisible();

  await page.goto("/ops/consistency");
  await expect(page.getByText("双层权威一致性联查")).toBeVisible();
  await expect(page.getByText("dry-run 一致性修复预演")).toBeVisible();

  await page.goto("/ops/consistency/outbox");
  await expect(page.getByText("Outbox / Dead Letter 控制台")).toBeVisible();
  await expect(page.getByText("Dead Letter dry-run 重处理")).toBeVisible();

  await page.goto("/ops/search");
  await expect(page.getByText("搜索同步与推荐重建运维")).toBeVisible();
  await expect(page.getByRole("heading", { name: "推荐重建", exact: true })).toBeVisible();
});
