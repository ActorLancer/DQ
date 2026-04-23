import { expect, test, type BrowserContext, type Page } from "@playwright/test";

const STANDARD_PRODUCT_ID = "20000000-0000-0000-0000-000000000309";
const STANDARD_SKU_ID = "20000000-0000-0000-0000-000000000409";
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

function encodeBase64Url(value: unknown) {
  return Buffer.from(JSON.stringify(value), "utf8").toString("base64url");
}

function createUnsignedJwt(payload: Record<string, unknown>) {
  return [
    encodeBase64Url({ alg: "none", typ: "JWT" }),
    encodeBase64Url(payload),
    "web018-live",
  ].join(".");
}

async function installPortalBearerSession(context: BrowserContext) {
  const token = createUnsignedJwt({
    exp: Math.floor(Date.now() / 1000) + 3600,
    name: "WEB018 Buyer Operator",
    org_id: "10000000-0000-0000-0000-000000000102",
    preferred_username: "web018-buyer-operator",
    realm_access: { roles: ["buyer_operator"] },
    user_id: "10000000-0000-0000-0000-000000000356",
  });
  await context.addCookies([
    {
      name: "datab_portal_session",
      value: encodeBase64Url({
        mode: "bearer",
        accessToken: token,
        label: "WEB-018 fake buyer claims",
      }),
      url: "http://127.0.0.1:3101",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

test("portal home links directly to five standard demo paths", async ({ page }) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);
  const scenarios = [
    ["S1", "工业设备运行指标 API 订阅", "API_SUB", "API_PPU"],
    ["S2", "工业质量与产线日报文件包交付", "FILE_STD", "FILE_SUB"],
    ["S3", "供应链协同查询沙箱", "SBX_STD", "SHARE_RO"],
    ["S4", "零售门店经营分析 API / 报告订阅", "API_SUB", "RPT_STD"],
    ["S5", "商圈/门店选址查询服务", "QRY_LITE", "RPT_STD"],
  ] as const;

  for (const [scenarioCode, scenarioName, primarySku, supplementarySku] of scenarios) {
    await page.goto("/");
    await page.getByRole("link", { name: `查看 ${scenarioCode} 演示路径` }).click();
    await expect(page).toHaveURL(`/demos/${scenarioCode}`);
    await expect(
      page.getByRole("heading", {
        name: `${scenarioName}演示路径`,
        exact: true,
      }),
    ).toBeVisible();
    await expect(page.getByText("场景名 -> 主 SKU / 补充 SKU").first()).toBeVisible();
    await expect(page.getByText(`主 SKU ${primarySku}`).first()).toBeVisible();
    await expect(page.getByText(`补充 SKU ${supplementarySku}`).first()).toBeVisible();
  }

  expect(restrictedRequests).toEqual([]);
});

test("portal core routes render without preview parameters", async ({ context, page }) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);
  await installPortalBearerSession(context);

  await page.goto("/");
  await expect(page.getByText("门户首页已接入场景导航、推荐位与受控搜索入口。")).toBeVisible();

  await page.goto("/search?q=工业设备运行指标");
  await expect(
    page.getByRole("heading", {
      name: "统一搜索数据商品、服务商品与卖方主体",
      exact: true,
    }),
  ).toBeVisible();

  await page.goto(`/trade/orders/new?product_id=${STANDARD_PRODUCT_ID}&sku_id=${STANDARD_SKU_ID}&scenario=S1`);
  await expect(page.getByRole("heading", { name: /询单/ })).toBeVisible();

  await page.goto("/developer");
  await expect(page.getByRole("heading", { name: "开发者工作台", exact: true })).toBeVisible();

  await page.goto(`/developer/trace?order_id=${SEEDED_ORDER_ID}`);
  await expect(page.getByText("Trace 与调用日志联查")).toBeVisible();

  expect(restrictedRequests).toEqual([]);
});
