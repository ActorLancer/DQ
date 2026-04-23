import { expect, test, type Page } from "@playwright/test";

const SEEDED_ORDER_ID = "30000000-0000-0000-0000-000000000101";
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

test("console home and scaffold pages are reachable", async ({ page }) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);
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

  await page.goto("/developer");
  await expect(page.getByText("开发者控制面")).toBeVisible();
  await expect(page.getByText("当前主体 / 角色 / 租户 / 作用域").first()).toBeVisible();

  await page.goto("/developer/apps");
  await expect(page.getByText("测试应用与 API Key")).toBeVisible();
  await expect(page.getByText("应用列表")).toBeVisible();
  await expect(page.getByText("Idempotency-Key").first()).toBeVisible();

  await page.goto("/developer/trace");
  await expect(page.getByText("状态与调用日志联查")).toBeVisible();
  await expect(page.getByText("request_id").first()).toBeVisible();

  await page.goto("/developer/assets");
  await expect(page.getByText("Mock 支付与测试资产")).toBeVisible();
  await expect(page.getByText("developer.mock_payment.simulate").first()).toBeVisible();

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
  expect(restrictedRequests).toEqual([]);
});

test("WEB-018 console control-plane flow covers login, audit, ops and developer linkage", async ({
  page,
}) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);

  await page.goto("/");
  await page.getByRole("button", { name: "登录态占位" }).click();
  await expect(page.getByText("Keycloak / IAM placeholder")).toBeVisible();
  await page.getByRole("button", { name: "Local Header" }).click();
  await expect(page.getByPlaceholder("platform.ops@luna.local")).toHaveValue(
    "platform.ops@luna.local",
  );
  await expect(page.getByPlaceholder("platform_admin")).toHaveValue("platform_admin");
  await page.getByRole("button", { name: "关闭" }).click();

  await page.goto("/ops/audit/trace?preview=empty");
  await expect(page.getByText("空态预演")).toBeVisible();

  await page.goto("/ops/audit/trace");
  await expect(page.getByText("审计事件、链回执、外部事实和证据包导出在控制台闭环。")).toBeVisible();
  await expect(page.getByText("request_id").first()).toBeVisible();

  await page.goto(`/developer/trace?order_id=${SEEDED_ORDER_ID}`);
  await expect(page.getByText("状态与调用日志联查")).toBeVisible();
  await expect(page.getByText("request_id").first()).toBeVisible();

  await page.goto("/ops/consistency");
  await expect(page.getByText("双层权威一致性联查")).toBeVisible();
  await expect(page.getByText("PostgreSQL 真值")).toBeVisible();
  await expect(page.getByText("Kafka/outbox 分发")).toBeVisible();
  await expect(page.getByText("OpenSearch 读模型")).toBeVisible();
  await expect(page.getByText("Redis 缓存")).toBeVisible();
  await expect(page.getByText("Fabric 可信确认")).toBeVisible();

  await page.goto("/ops/search");
  await expect(page.getByText("搜索同步与推荐重建运维")).toBeVisible();
  await expect(page.getByText("X-Step-Up-Token").first()).toBeVisible();

  expect(restrictedRequests).toEqual([]);
});
