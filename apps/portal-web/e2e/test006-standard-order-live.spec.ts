import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import {
  expect,
  test,
  type APIRequestContext,
  type BrowserContext,
  type Page,
} from "@playwright/test";

import {
  standardDemoGuides,
  type StandardDemoGuide,
} from "../src/lib/standard-demo";

const liveEnabled = process.env.WEB_E2E_LIVE === "1";
const keycloakBaseUrl = process.env.WEB_E2E_KEYCLOAK_URL ?? "http://127.0.0.1:8081";
const keycloakRealm = process.env.WEB_E2E_KEYCLOAK_REALM ?? "platform-local";
const keycloakClientId = process.env.WEB_E2E_KEYCLOAK_CLIENT_ID ?? "portal-web";
const platformCoreBaseUrl = process.env.PLATFORM_CORE_BASE_URL ?? "http://127.0.0.1:8094";
const buyerUsername = process.env.WEB_E2E_PORTAL_USERNAME ?? "local-buyer-operator";
const buyerPassword = process.env.WEB_E2E_PORTAL_PASSWORD ?? "LocalBuyerOperator123!";
const traceUsername =
  process.env.WEB_E2E_TRACE_USERNAME ??
  process.env.WEB_E2E_PLATFORM_USERNAME ??
  "local-tenant-developer";
const tracePassword =
  process.env.WEB_E2E_TRACE_PASSWORD ??
  process.env.WEB_E2E_PLATFORM_PASSWORD ??
  "LocalTenantDeveloper123!";
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
const scenarioCases = loadScenarioCases();

test.describe("TEST-006 standard order E2E", () => {
  test.skip(!liveEnabled, "Set WEB_E2E_LIVE=1 to run TEST-006 live scenario E2E.");
  test.describe.configure({ timeout: 120_000 });

  for (const scenarioCase of scenarioCases) {
    test(`${scenarioCase.guide.scenarioCode} ${scenarioCase.guide.scenarioName}`, async ({
      context,
      page,
      request,
    }) => {
      const restrictedRequests = watchRestrictedBrowserRequests(page);
      const buyerToken = await fetchAccessToken(request, buyerUsername, buyerPassword);
      const traceToken = await fetchAccessToken(
        request,
        traceUsername,
        tracePassword,
      );

      await installPortalBearerSession(context, buyerToken);

      await test.step("门户认证与官方演示路径", async () => {
        await page.goto("/");
        await expect(page.getByText("角色 buyer_operator")).toBeVisible({
          timeout: 20_000,
        });
        await expect(
          page.getByText(`租户 ${scenarioCase.scenario.participants.buyer_org_id}`),
        ).toBeVisible();

        await page.goto(scenarioCase.guide.path);
        await expect(
          page.getByRole("heading", {
            name: `${scenarioCase.guide.scenarioName}演示路径`,
            exact: true,
          }),
        ).toBeVisible();
        await expect(
          page.getByText(`主 SKU ${scenarioCase.guide.primarySku}`).first(),
        ).toBeVisible();
        for (const sku of scenarioCase.guide.supplementarySkus) {
          await expect(page.getByText(`补充 SKU ${sku}`).first()).toBeVisible();
        }
        await expect(
          page.getByText(scenarioCase.orderBlueprint.template_codes.contract_template).first(),
        ).toBeVisible();
        await expect(
          page.getByText(scenarioCase.orderBlueprint.template_codes.acceptance_template).first(),
        ).toBeVisible();
        await expect(
          page.getByText(scenarioCase.orderBlueprint.template_codes.refund_template).first(),
        ).toBeVisible();
      });

      await test.step("真实搜索与商品详情", async () => {
        await page.goto(
          `/search?q=${encodeURIComponent(scenarioCase.guide.searchKeyword)}&scenario=${encodeURIComponent(scenarioCase.guide.scenarioCode)}`,
        );
        await expect(
          page.getByRole("heading", {
            name: "统一搜索数据商品、服务商品与卖方主体",
            exact: true,
          }),
        ).toBeVisible();
        await expect(
          page.getByText(scenarioCase.guide.scenarioName).first(),
        ).toBeVisible({ timeout: 20_000 });

        await page.goto(`/products/${scenarioCase.orderBlueprint.product_id}`);
        await expect(
          page.getByText(scenarioCase.guide.scenarioName).first(),
        ).toBeVisible({ timeout: 20_000 });
        await expect(
          page.getByText(scenarioCase.guide.primarySku).first(),
        ).toBeVisible();
      });

      await test.step("下单页承接冻结场景与模板", async () => {
        await page.goto(
          `/trade/orders/new?product_id=${scenarioCase.orderBlueprint.product_id}&sku_id=${scenarioCase.orderBlueprint.sku_id}&scenario=${scenarioCase.guide.scenarioCode}`,
        );
        await expect(
          page.getByRole("heading", { name: "询单 / 下单页", exact: true }),
        ).toBeVisible();
        await expect(page.getByText("五条标准链路下单入口")).toBeVisible();
        await expect(
          page.getByText(scenarioCase.guide.scenarioName).first(),
        ).toBeVisible();
        await expect(
          page.getByText(scenarioCase.guide.primarySku).first(),
        ).toBeVisible();
        await expect(
          page.getByText(scenarioCase.orderBlueprint.template_codes.contract_template).first(),
        ).toBeVisible();
        await expect(
          page.getByText(scenarioCase.orderBlueprint.template_codes.acceptance_template).first(),
        ).toBeVisible();
        await expect(
          page.getByText(scenarioCase.orderBlueprint.template_codes.refund_template).first(),
        ).toBeVisible();
        await expect(page.getByText("X-Idempotency-Key").first()).toBeVisible();
      });

      await test.step("后端 order detail / lifecycle / developer trace 回查", async () => {
        const buyerHeaders = buildBearerHeaders(
          buyerToken,
          `test006-order-detail-${scenarioCase.guide.scenarioCode}`,
        );
        const orderDetail = await fetchPlatformJson(
          request,
          `/api/v1/orders/${scenarioCase.orderId}`,
          buyerHeaders,
        );
        const orderDetailText = JSON.stringify(orderDetail);
        expect(orderDetailText).toContain("\"code\":\"OK\"");
        expect(orderDetailText).toContain("\"request_id\":\"");
        expect(orderDetailText).toContain(scenarioCase.orderId);
        expect(orderDetailText).toContain(scenarioCase.guide.scenarioCode);
        expect(orderDetailText).toContain(scenarioCase.orderBlueprint.current_state);
        expect(orderDetailText).toContain(scenarioCase.orderBlueprint.payment_status);
        expect(orderDetailText).toContain(scenarioCase.orderBlueprint.delivery_status);
        expect(orderDetailText).toContain(scenarioCase.guide.primarySku);
        expect(orderDetailText).toContain(
          scenarioCase.orderBlueprint.template_codes.contract_template,
        );

        const lifecycle = await fetchPlatformJson(
          request,
          `/api/v1/orders/${scenarioCase.orderId}/lifecycle-snapshots`,
          buildBearerHeaders(
            buyerToken,
            `test006-lifecycle-${scenarioCase.guide.scenarioCode}`,
          ),
        );
        const lifecycleText = JSON.stringify(lifecycle);
        expect(lifecycleText).toContain("\"code\":\"OK\"");
        const lifecycleData = readRecord(readRecord(lifecycle)?.data);
        const lifecycleOrder = readRecord(lifecycleData?.order);
        const lifecyclePayment = readRecord(lifecycleOrder?.payment);
        const lifecycleAcceptance = readRecord(lifecycleOrder?.acceptance);
        const lifecycleSettlement = readRecord(lifecycleOrder?.settlement);
        const lifecycleDelivery = readRecord(lifecycleData?.delivery);
        expect(readString(lifecycleOrder?.current_state)).toBe(
          scenarioCase.orderBlueprint.current_state,
        );
        expect(readString(lifecyclePayment?.current_status)).toBe(
          scenarioCase.orderBlueprint.payment_status,
        );
        expect(readString(lifecycleAcceptance?.current_status)).toBe(
          scenarioCase.orderBlueprint.acceptance_status,
        );
        expect(readString(lifecycleSettlement?.current_status)).toBe(
          scenarioCase.orderBlueprint.settlement_status,
        );
        expect(readString(lifecycleDelivery?.current_status)).toBe("committed");

        const developerTrace = await fetchPlatformJson(
          request,
          `/api/v1/developer/trace?order_id=${scenarioCase.orderId}`,
          buildBearerHeaders(
            traceToken,
            `test006-developer-trace-${scenarioCase.guide.scenarioCode}`,
          ),
        );
        const developerTraceText = JSON.stringify(developerTrace);
        expect(developerTraceText).toContain("\"code\":\"OK\"");
        expect(developerTraceText).toContain("\"request_id\":\"");
        expect(developerTraceText).toContain(scenarioCase.orderId);
      });

      await test.step("订单详情、交付页与验收页", async () => {
        await page.goto(`/trade/orders/${scenarioCase.orderId}`);
        await expect(
          page.getByRole("heading", { name: "订单详情页", exact: true }),
        ).toBeVisible();
        await expect(page.getByText("场景与 SKU 快照")).toBeVisible();
        await expect(page.getByText("审计与链路信任边界")).toBeVisible();
        await expect(
          page.getByText(scenarioCase.orderBlueprint.current_state).first(),
        ).toBeVisible();
        await expect(
          page.getByText(scenarioCase.guide.scenarioCode).first(),
        ).toBeVisible();

        await page.goto(`/delivery/orders/${scenarioCase.orderId}/${scenarioCase.deliveryPathSuffix}`);
        await expect(page.getByText("订单交付状态")).toBeVisible();
        if (scenarioCase.deliveryPermissionWarning) {
          await expect(
            page.getByRole("heading", { name: scenarioCase.deliveryPermissionWarning, exact: true }),
          ).toBeVisible();
        } else {
          await expect(
            page.getByText(scenarioCase.deliveryExpectation).first(),
          ).toBeVisible();
        }
        await expect(
          page.getByText(scenarioCase.guide.primarySku).first(),
        ).toBeVisible();

        await page.goto(`/delivery/orders/${scenarioCase.orderId}/acceptance`);
        await expect(
          page.getByRole("heading", { name: "验收页", exact: true }),
        ).toBeVisible();
        await expect(page.getByText("交付结果摘要")).toBeVisible();
        await expect(page.getByText("生命周期摘要")).toBeVisible();
      });

      expect(restrictedRequests).toEqual([]);
    });
  }
});

type ScenarioFixture = {
  scenario_code: string;
  scenario_name: string;
  participants: {
    buyer_org_id: string;
  };
  primary_order_blueprint_id: string;
};

type OrderBlueprint = {
  order_blueprint_id: string;
  scenario_code: string;
  scenario_role: string;
  product_id: string;
  sku_id: string;
  current_state: string;
  payment_status: string;
  delivery_status: string;
  acceptance_status: string;
  settlement_status: string;
  dispute_status: string;
  template_codes: {
    contract_template: string;
    acceptance_template: string;
    refund_template: string;
  };
};

type ScenarioCase = {
  guide: StandardDemoGuide;
  scenario: ScenarioFixture;
  orderBlueprint: OrderBlueprint;
  orderId: string;
  deliveryPathSuffix: string;
  deliveryExpectation: string;
  deliveryPermissionWarning: string | null;
};

function loadScenarioCases(): ScenarioCase[] {
  const repoRoot = resolve(process.cwd(), "../..");
  const scenariosFixture = JSON.parse(
    readFileSync(resolve(repoRoot, "fixtures/demo/scenarios.json"), "utf8"),
  ) as { scenarios: ScenarioFixture[] };
  const ordersFixture = JSON.parse(
    readFileSync(resolve(repoRoot, "fixtures/demo/orders.json"), "utf8"),
  ) as { order_blueprints: OrderBlueprint[] };

  const scenariosByCode = new Map(
    scenariosFixture.scenarios.map((scenario) => [scenario.scenario_code, scenario]),
  );
  const ordersById = new Map(
    ordersFixture.order_blueprints.map((order) => [order.order_blueprint_id, order]),
  );

  return standardDemoGuides.map((guide) => {
    const scenario = scenariosByCode.get(guide.scenarioCode);
    if (!scenario) {
      throw new Error(`missing scenario fixture for ${guide.scenarioCode}`);
    }
    const orderBlueprint = ordersById.get(scenario.primary_order_blueprint_id);
    if (!orderBlueprint) {
      throw new Error(`missing order blueprint ${scenario.primary_order_blueprint_id}`);
    }

    const primaryDeliveryLink =
      guide.deliveryLinks.find((link) => link.sku === guide.primarySku) ??
      guide.deliveryLinks[0];
    if (!primaryDeliveryLink) {
      throw new Error(`missing delivery link for ${guide.scenarioCode}`);
    }
    const deliveryPathSuffix = primaryDeliveryLink.href.split("/").pop();
    if (!deliveryPathSuffix) {
      throw new Error(`cannot resolve delivery path for ${guide.scenarioCode}`);
    }

    return {
      guide,
      scenario,
      orderBlueprint,
      orderId: orderBlueprint.order_blueprint_id,
      deliveryPathSuffix,
      deliveryExpectation: deliveryExpectation(deliveryPathSuffix),
      deliveryPermissionWarning: deliveryPermissionWarning(deliveryPathSuffix),
    };
  });
}

function deliveryExpectation(pathSuffix: string) {
  switch (pathSuffix) {
    case "api":
      return "API 开通表单";
    case "file":
      return "文件交付表单";
    case "subscription":
      return "版本订阅管理";
    case "share":
      return "只读共享授权";
    case "sandbox":
      return "沙箱工作区开通";
    case "template-query":
      return "模板查询授权";
    case "report":
      return "报告交付表单";
    default:
      throw new Error(`unsupported delivery path suffix: ${pathSuffix}`);
  }
}

function deliveryPermissionWarning(pathSuffix: string) {
  switch (pathSuffix) {
    case "file":
      return "主按钮权限不足";
    default:
      return null;
  }
}

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

async function installPortalBearerSession(context: BrowserContext, token: string) {
  await context.addCookies([
    {
      name: "datab_portal_session",
      value: Buffer.from(
        JSON.stringify({
          mode: "bearer",
          accessToken: token,
          label: "TEST-006 live buyer bearer",
        }),
        "utf8",
      ).toString("base64url"),
      url: "http://127.0.0.1:3101",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

function buildBearerHeaders(token: string, requestId: string): Record<string, string> {
  const claims = decodeJwtPayload(token);
  const role = readStringArray(readRecord(claims?.realm_access)?.roles)[0];
  const userId = readString(claims?.user_id);
  const tenantId = readString(claims?.org_id);

  return {
    authorization: `Bearer ${token}`,
    "x-request-id": requestId,
    ...(userId ? { "x-user-id": userId } : {}),
    ...(tenantId ? { "x-tenant-id": tenantId } : {}),
    ...(role ? { "x-role": role } : {}),
  };
}

async function fetchPlatformJson(
  request: APIRequestContext,
  pathName: string,
  headers: Record<string, string>,
) {
  const response = await request.get(`${platformCoreBaseUrl}${pathName}`, { headers });
  const body = await response.text();
  expect(response.ok(), `${pathName} failed: ${body}`).toBeTruthy();
  return JSON.parse(body) as unknown;
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
    ? (value as Record<string, unknown>)
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
