import { expect, test } from "@playwright/test";

test("portal home and scaffold pages are reachable", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("门户工程基线已接入正式路由、受控 API 代理和登录态占位。")).toBeVisible();
  await expect(page.getByText("当前主体 / 角色 / 租户 / 作用域")).toBeVisible();

  await page.goto("/search?preview=empty");
  await expect(page.getByText("空态预演")).toBeVisible();

  await page.goto("/trade/orders/new?preview=forbidden");
  await expect(page.getByText("权限态预演")).toBeVisible();
});
