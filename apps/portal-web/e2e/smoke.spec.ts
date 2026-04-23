import { expect, test } from "@playwright/test";

test("portal home and scaffold pages are reachable", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("门户首页已接入场景导航、推荐位与受控搜索入口。")).toBeVisible();
  await expect(page.getByRole("heading", { name: "标准链路快捷入口", exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "受控搜索入口", exact: true })).toBeVisible();
  await expect(page.getByText("当前主体 / 角色 / 租户 / 作用域")).toBeVisible();

  await page.goto("/search?preview=empty");
  await expect(page.getByText("空态预演")).toBeVisible();

  await page.goto("/trade/orders/new?preview=forbidden");
  await expect(page.getByText("权限态预演")).toBeVisible();
});
