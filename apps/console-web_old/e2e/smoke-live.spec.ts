import { expect, test, type Page } from "@playwright/test";

const SEEDED_ORDER_ID = "30000000-0000-0000-0000-000000000101";
const restrictedPorts = new Set([
  "5432",
  "6379",
  "7050",
  "7051",
  "8080",
  "8094",
  "8097",
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
  "notification-worker",
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

test("console core routes render without preview parameters", async ({ page }) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);

  await page.goto("/");
  await expect(
    page.getByText("控制台已接入正式路由、受控 API 代理和控制面认证会话。"),
  ).toBeVisible();
  await expect(page.getByText("当前主体 / 角色 / 租户 / 作用域")).toBeVisible();

  await page.goto("/ops/audit/trace");
  await expect(
    page.getByText("审计事件、链回执、外部事实和证据包导出在控制台闭环。"),
  ).toBeVisible();

  await page.goto(`/developer/trace?order_id=${SEEDED_ORDER_ID}`);
  await expect(page.getByText("状态与调用日志联查")).toBeVisible();

  await page.goto("/ops/consistency");
  await expect(page.getByText("双层权威一致性联查")).toBeVisible();

  await page.goto("/ops/notifications");
  await expect(page.getByText("通知联查与补发")).toBeVisible();

  await page.goto("/ops/search");
  await expect(page.getByText("搜索同步与推荐重建运维")).toBeVisible();
  await expect(page.getByRole("heading", { name: "推荐重建", exact: true })).toBeVisible();

  expect(restrictedRequests).toEqual([]);
});
