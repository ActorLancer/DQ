import { mkdir, writeFile } from "node:fs/promises";
import { dirname } from "node:path";

import {
  expect,
  test,
  type APIRequestContext,
  type BrowserContext,
  type Page,
} from "@playwright/test";

const liveEnabled = process.env.WEB_E2E_LIVE === "1";
const keycloakBaseUrl = process.env.WEB_E2E_KEYCLOAK_URL ?? "http://127.0.0.1:8081";
const keycloakRealm = process.env.WEB_E2E_KEYCLOAK_REALM ?? "platform-local";
const keycloakClientId = process.env.WEB_E2E_KEYCLOAK_CLIENT_ID ?? "portal-web";
const platformCoreBaseUrl = process.env.PLATFORM_CORE_BASE_URL ?? "http://127.0.0.1:8094";
const orderId = process.env.WEB_E2E_SHARE_ORDER_ID ?? "";
const assetObjectId = process.env.WEB_E2E_SHARE_ASSET_OBJECT_ID ?? "";
const sellerUsername = process.env.WEB_E2E_SELLER_USERNAME ?? "local-seller-operator";
const sellerPassword =
  process.env.WEB_E2E_SELLER_PASSWORD ?? "LocalSellerOperator123!";
const buyerUsername = process.env.WEB_E2E_BUYER_USERNAME ?? "local-buyer-operator";
const buyerPassword =
  process.env.WEB_E2E_BUYER_PASSWORD ?? "LocalBuyerOperator123!";
const artifactFile = process.env.TEST025_PORTAL_ARTIFACT_FILE ?? "";
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

test.describe("TEST-025 SHARE_RO live portal E2E", () => {
  test.skip(!liveEnabled, "Set WEB_E2E_LIVE=1 to run TEST-025 live portal E2E.");
  test.skip(!orderId || !assetObjectId, "Set WEB_E2E_SHARE_ORDER_ID and WEB_E2E_SHARE_ASSET_OBJECT_ID.");
  test.describe.configure({ timeout: 120_000 });

  test("seller grant/revoke and buyer read flow stays on formal portal/API boundary", async ({
    context,
    page,
    request,
  }) => {
    const restrictedRequests = watchRestrictedBrowserRequests(page);
    const sellerToken = await fetchAccessToken(request, sellerUsername, sellerPassword);
    const buyerToken = await fetchAccessToken(request, buyerUsername, buyerPassword);
    const runSuffix = `${Date.now()}`;
    const recipientRef = `warehouse://buyer/test025/${runSuffix}`;
    const subscriberRef = `sub-test025-${runSuffix}`;
    const grantReceiptHash = `sha256:test025-share-grant-${runSuffix}`;
    const revokeReceiptHash = `sha256:test025-share-revoke-${runSuffix}`;
    const accessLocator = `share://seller/test025/${runSuffix}/dataset`;

    await installPortalBearerSession(context, sellerToken, "TEST-025 seller bearer");

    await page.goto("/");
    await expect(
      page.getByText("角色 seller_operator", { exact: true }),
    ).toBeVisible({ timeout: 20_000 });
    await page.goto(`/delivery/orders/${orderId}/share`);
    await expect(page.getByText("订单交付状态")).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("只读共享授权")).toBeVisible();
    await expect(page.getByText("共享协议、接收方、授权范围、有效期与撤权记录。")).toBeVisible();

    const grantResponsePromise = page.waitForResponse((response) =>
      response.request().method() === "POST" &&
      response.url().includes(`/api/platform/api/v1/orders/${orderId}/share-grants`)
    );

    await page.locator('input[name="recipient_ref"]').fill(recipientRef);
    await page.locator('input[name="subscriber_ref"]').fill(subscriberRef);
    await page.locator('input[name="share_protocol"]').fill("share_grant");
    await page.locator('input[name="access_locator"]').fill(accessLocator);
    await page.locator('input[name="expires_at"]').fill("2026-07-01T00:00:00Z");
    await page.locator('input[name="receipt_hash"]').fill(grantReceiptHash);
    await page.locator('input[name="asset_object_id"]').fill(assetObjectId);
    await page.locator('textarea[name="scope_json"]').fill(
      JSON.stringify({ schema: "analytics", tables: ["orders", "inventory"] }),
    );
    await page.locator('textarea[name="metadata_json"]').fill(
      JSON.stringify({ task_id: "TEST-025", source: "portal-live" }),
    );
    await page.locator('input[name="confirm_scope"]').check();
    await page.locator('input[name="confirm_audit"]').check();
    await page.getByRole("button", { name: "提交" }).click();

    const grantResponse = await grantResponsePromise;
    expect(grantResponse.ok()).toBeTruthy();
    const grantPayload = await grantResponse.json();
    const grantData = unwrapData(grantPayload);
    expect(readString(readRecord(grantData)?.grant_status)).toBe("active");
    expect(readString(readRecord(grantData)?.current_state)).toBe("share_granted");
    await expect(page.getByText("接口返回摘要")).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("share_granted").first()).toBeVisible();

    await installPortalBearerSession(context, buyerToken, "TEST-025 buyer bearer");
    const buyerReadResponsePromise = page.waitForResponse((response) =>
      response.request().method() === "GET" &&
      response.url().includes(`/api/platform/api/v1/orders/${orderId}/share-grants`)
    );
    await page.goto(`/delivery/orders/${orderId}/share`);
    const buyerReadResponse = await buyerReadResponsePromise;
    expect(buyerReadResponse.ok()).toBeTruthy();
    const buyerReadPayload = await fetchPlatformJson(
      request,
      `/api/v1/orders/${orderId}/share-grants`,
      buildBearerHeaders(buyerToken, `test025-share-read-${runSuffix}`),
    );
    const buyerReadData = unwrapData(buyerReadPayload);
    const buyerGrants = readArray(readRecord(buyerReadData)?.grants);
    const buyerTargetGrant = buyerGrants.find(
      (entry) => readString(readRecord(entry)?.recipient_ref) === recipientRef,
    );
    expect(
      Boolean(buyerTargetGrant),
    ).toBeTruthy();
    await expect(
      page.getByText("角色 buyer_operator", { exact: true }),
    ).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText(recipientRef).first()).toBeVisible();
    await expect(page.getByText("active / share_grant").first()).toBeVisible();

    await installPortalBearerSession(context, sellerToken, "TEST-025 seller bearer");
    await page.goto(`/delivery/orders/${orderId}/share`);
    const revokeResponsePromise = page.waitForResponse((response) =>
      response.request().method() === "POST" &&
      response.url().includes(`/api/platform/api/v1/orders/${orderId}/share-grants`)
    );
    await page.locator('select[name="operation"]').selectOption("revoke");
    await page.locator('input[name="recipient_ref"]').fill(recipientRef);
    await page.locator('input[name="subscriber_ref"]').fill(subscriberRef);
    await page.locator('input[name="share_protocol"]').fill("share_grant");
    await page.locator('input[name="access_locator"]').fill(accessLocator);
    await page.locator('input[name="receipt_hash"]').fill(revokeReceiptHash);
    await page.locator('input[name="asset_object_id"]').fill(assetObjectId);
    await page.locator('textarea[name="scope_json"]').fill(
      JSON.stringify({ schema: "analytics", tables: ["orders", "inventory"] }),
    );
    await page.locator('textarea[name="metadata_json"]').fill(
      JSON.stringify({ task_id: "TEST-025", source: "portal-live", operation: "revoke" }),
    );
    await page.locator('input[name="confirm_scope"]').check();
    await page.locator('input[name="confirm_audit"]').check();
    await page.getByRole("button", { name: "提交" }).click();

    const revokeResponse = await revokeResponsePromise;
    expect(revokeResponse.ok()).toBeTruthy();
    const revokePayload = await revokeResponse.json();
    const revokeData = unwrapData(revokePayload);
    expect(readString(readRecord(revokeData)?.grant_status)).toBe("revoked");
    expect(readString(readRecord(revokeData)?.current_state)).toBe("revoked");
    await expect(page.getByText("revoked").first()).toBeVisible({ timeout: 20_000 });

    const orderAfterRevoke = await fetchPlatformJson(
      request,
      `/api/v1/orders/${orderId}`,
      buildBearerHeaders(sellerToken, `test025-order-${runSuffix}`),
    );
    const orderData = unwrapData(orderAfterRevoke);
    expect(readString(readRecord(orderData)?.order_id)).toBe(orderId);
    expect(readString(readRecord(orderData)?.current_state)).toBe("revoked");
    expect(readString(readRecord(orderData)?.delivery_status)).toBe("closed");
    expect(readString(readRecord(orderData)?.payment_status)).toBe("paid");

    const shareAfterRevoke = await fetchPlatformJson(
      request,
      `/api/v1/orders/${orderId}/share-grants`,
      buildBearerHeaders(buyerToken, `test025-share-grants-${runSuffix}`),
    );
    const shareAfterRevokeData = unwrapData(shareAfterRevoke);
    const grantsAfterRevoke = readArray(readRecord(shareAfterRevokeData)?.grants);
    const revokeTargetGrant = grantsAfterRevoke.find(
      (entry) => readString(readRecord(entry)?.recipient_ref) === recipientRef,
    );
    expect(readString(readRecord(revokeTargetGrant)?.grant_status)).toBe("revoked");
    expect(readString(readRecord(revokeTargetGrant)?.receipt_hash)).toBe(revokeReceiptHash);

    await writeArtifact({
      test_id: "test025-share-ro-live",
      order_id: orderId,
      asset_object_id: assetObjectId,
      recipient_ref: recipientRef,
      subscriber_ref: subscriberRef,
      access_locator: accessLocator,
      grant_receipt_hash: grantReceiptHash,
      revoke_receipt_hash: revokeReceiptHash,
      seller_username: sellerUsername,
      buyer_username: buyerUsername,
      portal_proxy_requests: {
        grant_request_id: readString(readRecord(grantPayload)?.request_id),
        buyer_read_request_id: readString(readRecord(buyerReadPayload)?.request_id),
        revoke_request_id: readString(readRecord(revokePayload)?.request_id),
      },
      grant_response: {
        grant_status: readString(readRecord(grantData)?.grant_status),
        current_state: readString(readRecord(grantData)?.current_state),
        delivery_status: readString(readRecord(grantData)?.delivery_status),
        operation: readString(readRecord(grantData)?.operation),
      },
      buyer_read_response: {
        grant_count: buyerGrants.length,
        target_grant_status: readString(readRecord(buyerTargetGrant)?.grant_status),
      },
      revoke_response: {
        grant_status: readString(readRecord(revokeData)?.grant_status),
        current_state: readString(readRecord(revokeData)?.current_state),
        operation: readString(readRecord(revokeData)?.operation),
      },
      order_after_revoke: {
        current_state: readString(readRecord(orderData)?.current_state),
        delivery_status: readString(readRecord(orderData)?.delivery_status),
        payment_status: readString(readRecord(orderData)?.payment_status),
      },
      share_after_revoke: {
        grant_count: grantsAfterRevoke.length,
        target_grant_status: readString(readRecord(revokeTargetGrant)?.grant_status),
        target_receipt_hash: readString(readRecord(revokeTargetGrant)?.receipt_hash),
      },
      restricted_requests: restrictedRequests,
    });

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

async function installPortalBearerSession(
  context: BrowserContext,
  token: string,
  label: string,
) {
  await context.addCookies([
    {
      name: "datab_portal_session",
      value: Buffer.from(
        JSON.stringify({
          mode: "bearer",
          accessToken: token,
          label,
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

function unwrapData(payload: unknown) {
  const record = readRecord(payload);
  const data = readRecord(record?.data);
  return readRecord(data?.data) ?? data ?? record;
}

function decodeJwtPayload(token: string): Record<string, unknown> | undefined {
  const [, payload] = token.split(".");
  if (!payload) {
    return undefined;
  }
  try {
    return readRecord(JSON.parse(Buffer.from(payload, "base64url").toString("utf8")));
  } catch {
    return undefined;
  }
}

function readRecord(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

function readArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : [];
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

async function writeArtifact(payload: unknown) {
  if (!artifactFile) {
    return;
  }
  await mkdir(dirname(artifactFile), { recursive: true });
  await writeFile(artifactFile, JSON.stringify(payload, null, 2));
}
