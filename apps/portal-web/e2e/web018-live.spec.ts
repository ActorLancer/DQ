import {
  expect,
  test,
  type APIRequestContext,
  type Page,
} from "@playwright/test";

const liveEnabled = process.env.WEB_E2E_LIVE === "1";
const keycloakBaseUrl = process.env.WEB_E2E_KEYCLOAK_URL ?? "http://127.0.0.1:8081";
const keycloakRealm = process.env.WEB_E2E_KEYCLOAK_REALM ?? "platform-local";
const keycloakClientId = process.env.WEB_E2E_KEYCLOAK_CLIENT_ID ?? "portal-web";
const portalUsername = process.env.WEB_E2E_PORTAL_USERNAME ?? "local-platform-admin";
const portalPassword = process.env.WEB_E2E_PORTAL_PASSWORD ?? "LocalPlatformAdmin123!";
const standardProductId = "20000000-0000-0000-0000-000000000309";
const standardSkuId = "20000000-0000-0000-0000-000000000409";
const buyerOrgId = "10000000-0000-0000-0000-000000000102";
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

test.describe("WEB-018 live portal chain", () => {
  test.skip(!liveEnabled, "Set WEB_E2E_LIVE=1 to run platform-core backed E2E.");
  test.describe.configure({ timeout: 120_000 });

  test("logs in with Keycloak and exercises search, product, order, delivery, acceptance and linkage", async ({
    page,
    request,
  }) => {
    const restrictedRequests = watchRestrictedBrowserRequests(page);
    const token = await fetchAccessToken(request, portalUsername, portalPassword);

    await loginWithBearer(page, token);
    await expect(page.getByText("角色 platform_admin")).toBeVisible({ timeout: 20_000 });
    await expect(
      page.getByText("租户 10000000-0000-0000-0000-000000000103"),
    ).toBeVisible();

    await page.goto("/search?q=工业设备运行指标");
    await expect(
      page.getByRole("heading", {
        name: "统一搜索数据商品、服务商品与卖方主体",
        exact: true,
      }),
    ).toBeVisible();
    await expect(page.getByText("工业设备运行指标 API 订阅").first()).toBeVisible({
      timeout: 20_000,
    });

    await page.goto(`/products/${standardProductId}`);
    await expect(page.getByText("工业设备运行指标 API 订阅").first()).toBeVisible({
      timeout: 20_000,
    });
    await expect(page.getByText("API_SUB").first()).toBeVisible();

    const idempotencyKey = `web-018-e2e-order-${Date.now()}`;
    await installPortalLocalBuyerSession(page);
    await page.goto(
      `/trade/orders/new?product_id=${standardProductId}&sku_id=${standardSkuId}&scenario=S1&preview=empty`,
    );
    await expect(page.getByRole("heading", { name: "询单 / 下单页", exact: true })).toBeVisible();
    await expect(page.getByText("五条标准链路下单入口")).toBeVisible();
    const orderPayload = await createOrderThroughPortalProxy(page, idempotencyKey);
    const createdOrderId = readCreatedOrderId(orderPayload);
    expect(createdOrderId).toBeTruthy();
    await installPortalBearerSession(page, token);

    await page.goto(`/trade/orders/${createdOrderId}`);
    await expect(page.getByRole("heading", { name: "订单详情页", exact: true })).toBeVisible();
    await expect(page.getByText("审计与链路信任边界")).toBeVisible();

    await page.goto(`/delivery/orders/${createdOrderId}/api`);
    await expect(page.getByText("订单交付状态")).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("tx_hash").first()).toBeVisible();

    await page.goto(`/delivery/orders/${createdOrderId}/acceptance`);
    await expect(page.getByText("交付结果摘要")).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("生命周期摘要")).toBeVisible();

    await page.goto(`/developer/trace?order_id=${createdOrderId}`);
    await expect(page.getByText("Trace 与调用日志联查")).toBeVisible();
    await expect(page.getByText("request_id").first()).toBeVisible();

    expect(restrictedRequests).toEqual([]);
  });
});

async function fetchAccessToken(
  request: APIRequestContext,
  username: string,
  password: string,
) {
  const response = await request.post(
    `${keycloakBaseUrl}/realms/${keycloakRealm}/protocol/openid-connect/token`,
    {
      form: {
        client_id: keycloakClientId,
        grant_type: "password",
        username,
        password,
      },
    },
  );
  expect(response.ok()).toBeTruthy();
  const payload = await response.json();
  expect(typeof payload.access_token).toBe("string");
  return payload.access_token as string;
}

async function loginWithBearer(page: Page, token: string) {
  await page.goto("/");
  await page.getByRole("button", { name: "登录态占位" }).click();
  await page.locator("textarea").fill(token);
  await page.getByRole("button", { name: "验证并写入会话" }).click();
  await expect(page.getByRole("button", { name: "登录态占位" })).toBeVisible({
    timeout: 20_000,
  });
  await page.reload();
}

async function installPortalLocalBuyerSession(page: Page) {
  await page.context().addCookies([
    {
      name: "datab_portal_session",
      value: Buffer.from(
        JSON.stringify({
          mode: "local",
          loginId: "buyer.operator@luna.local",
          role: "buyer_operator",
          tenantId: buyerOrgId,
        }),
        "utf8",
      ).toString("base64url"),
      url: "http://127.0.0.1:3101",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

async function installPortalBearerSession(page: Page, token: string) {
  await page.context().addCookies([
    {
      name: "datab_portal_session",
      value: Buffer.from(
        JSON.stringify({
          mode: "bearer",
          accessToken: token,
          label: "WEB-018 platform admin bearer",
        }),
        "utf8",
      ).toString("base64url"),
      url: "http://127.0.0.1:3101",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

async function createOrderThroughPortalProxy(page: Page, idempotencyKey: string) {
  return page.evaluate(
    async ({ buyerOrgId, idempotencyKey, standardProductId, standardSkuId }) => {
      const response = await fetch("/api/platform/api/v1/orders", {
        method: "POST",
        headers: {
          "content-type": "application/json",
          "x-idempotency-key": idempotencyKey,
        },
        body: JSON.stringify({
          buyer_org_id: buyerOrgId,
          product_id: standardProductId,
          sku_id: standardSkuId,
          scenario_code: "S1",
        }),
      });
      const payload = await response.json();
      if (!response.ok) {
        throw new Error(JSON.stringify(payload));
      }
      return payload;
    },
    {
      buyerOrgId,
      idempotencyKey,
      standardProductId,
      standardSkuId,
    },
  );
}

function readCreatedOrderId(payload: unknown): string | undefined {
  if (!payload || typeof payload !== "object") {
    return undefined;
  }
  const root = payload as Record<string, unknown>;
  const data = root.data && typeof root.data === "object"
    ? root.data as Record<string, unknown>
    : undefined;
  const nested = data?.data && typeof data.data === "object"
    ? data.data as Record<string, unknown>
    : undefined;
  const orderId = nested?.order_id ?? data?.order_id ?? root.order_id;
  return typeof orderId === "string" ? orderId : undefined;
}

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
