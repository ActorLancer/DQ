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
const platformCoreBaseUrl = process.env.PLATFORM_CORE_BASE_URL ?? "http://127.0.0.1:8080";
const platformUsername = process.env.WEB_E2E_PLATFORM_USERNAME ?? "local-platform-admin";
const platformPassword = process.env.WEB_E2E_PLATFORM_PASSWORD ?? "LocalPlatformAdmin123!";
const seededOrderId = "30000000-0000-0000-0000-000000000101";
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

test.describe("WEB-018 live console chain", () => {
  test.skip(!liveEnabled, "Set WEB_E2E_LIVE=1 to run platform-core backed E2E.");
  test.describe.configure({ timeout: 120_000 });

  test("logs in with Keycloak and exercises audit, developer and ops linkage", async ({
    page,
    request,
  }) => {
    const restrictedRequests = watchRestrictedBrowserRequests(page);
    const token = await fetchAccessToken(request, platformUsername, platformPassword);

    await verifyLoginDialog(page);
    await installConsoleBearerSession(page, request, token);
    await page.reload();
    await expect(page.getByText("角色 platform_admin")).toBeVisible({ timeout: 20_000 });
    await expect(
      page.getByText("租户 10000000-0000-0000-0000-000000000103"),
    ).toBeVisible();

    await page.goto("/ops/audit/trace");
    await expect(page.getByRole("heading", { name: "审计联查页" })).toBeVisible();
    await page.locator('input[name="lookup_value"]').fill(seededOrderId);
    await page.getByRole("button", { name: "联查" }).click();
    await expect(page.getByText("链状态").first()).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("投影状态").first()).toBeVisible();

    await page.goto("/developer/trace");
    await expect(page.getByText("状态与调用日志联查")).toBeVisible();
    await page.locator('input[name="lookup_value"]').fill(seededOrderId);
    await page.getByRole("button", { name: "联查" }).click();
    await expect(page.getByText("request_id").first()).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("tx_hash").first()).toBeVisible();

    await page.goto("/ops/consistency");
    await expect(page.getByText("双层权威一致性联查")).toBeVisible();
    await page.getByPlaceholder("正式业务对象 ID / UUID").fill(seededOrderId);
    await page.getByRole("button", { name: "联查" }).click();
    await expect(page.getByText("PostgreSQL 真值").first()).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByText("Kafka/outbox 分发").first()).toBeVisible();
    await expect(page.getByText("OpenSearch 读模型").first()).toBeVisible();
    await expect(page.getByText("Redis 缓存").first()).toBeVisible();
    await expect(page.getByText("Fabric 可信确认").first()).toBeVisible();

    await page.goto("/ops/search");
    await expect(page.getByText("搜索同步与推荐重建运维")).toBeVisible();
    await expect(page.getByRole("heading", { name: "推荐重建", exact: true })).toBeVisible();
    await expect(page.getByText("X-Step-Up-Token").first()).toBeVisible();

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

async function verifyLoginDialog(page: Page) {
  await page.goto("/");
  await page.getByRole("button", { name: "登录态占位" }).click();
  await expect(page.getByText("Keycloak / IAM placeholder")).toBeVisible();
  await expect(page.locator("textarea")).toBeVisible();
  await page.getByRole("button", { name: "关闭" }).click();
}

async function installConsoleBearerSession(
  page: Page,
  request: APIRequestContext,
  token: string,
) {
  const response = await request.get(`${platformCoreBaseUrl}/api/v1/auth/me`, {
    headers: buildBearerHeaders(token),
  });
  expect(response.ok()).toBeTruthy();
  const payload = await response.json();
  const subject = payload.data as {
    user_id?: string;
    tenant_id?: string;
    org_id?: string;
    roles?: string[];
  };
  await page.context().addCookies([
    {
      name: "datab_console_session",
      value: Buffer.from(
        JSON.stringify({
          mode: "bearer",
          accessToken: token,
          label: "WEB-018 live console bearer",
          userId: subject.user_id,
          tenantId: subject.tenant_id ?? subject.org_id,
          role: subject.roles?.[0],
        }),
        "utf8",
      ).toString("base64url"),
      url: "http://127.0.0.1:3102",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

function buildBearerHeaders(token: string): Record<string, string> {
  const claims = decodeJwtPayload(token);
  const role = readStringArray(readRecord(claims?.realm_access)?.roles)[0];
  const userId = readString(claims?.user_id);
  const tenantId = readString(claims?.org_id);
  return {
    authorization: `Bearer ${token}`,
    ...(userId ? { "x-user-id": userId } : {}),
    ...(tenantId ? { "x-tenant-id": tenantId } : {}),
    ...(role ? { "x-role": role } : {}),
  };
}

function decodeJwtPayload(token: string): Record<string, unknown> | undefined {
  const [, payload] = token.split(".");
  if (!payload) {
    return undefined;
  }
  try {
    const decoded = JSON.parse(Buffer.from(payload, "base64url").toString("utf8"));
    return readRecord(decoded);
  } catch {
    return undefined;
  }
}

function readRecord(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
}

function readString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function readStringArray(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string")
    : [];
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
