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
const orderId = process.env.WEB_E2E_QRY_LITE_ORDER_ID ?? "";
const querySurfaceId = process.env.WEB_E2E_QRY_LITE_QUERY_SURFACE_ID ?? "";
const assetObjectId = process.env.WEB_E2E_QRY_LITE_ASSET_OBJECT_ID ?? "";
const queryTemplateId = process.env.WEB_E2E_QRY_LITE_QUERY_TEMPLATE_ID ?? "";
const approvalTicketId = process.env.WEB_E2E_QRY_LITE_APPROVAL_TICKET_ID ?? "";
const caseId = process.env.WEB_E2E_QRY_LITE_CASE_ID ?? "";
const buyerUserId = process.env.WEB_E2E_QRY_LITE_BUYER_USER_ID ?? "";
const orderAmount = process.env.WEB_E2E_QRY_LITE_ORDER_AMOUNT ?? "";
const sellerUsername = process.env.WEB_E2E_SELLER_USERNAME ?? "local-seller-operator";
const sellerPassword =
  process.env.WEB_E2E_SELLER_PASSWORD ?? "LocalSellerOperator123!";
const buyerUsername = process.env.WEB_E2E_BUYER_USERNAME ?? "local-buyer-operator";
const buyerPassword =
  process.env.WEB_E2E_BUYER_PASSWORD ?? "LocalBuyerOperator123!";
const riskUsername = process.env.WEB_E2E_RISK_USERNAME ?? "local-risk-settlement";
const riskPassword =
  process.env.WEB_E2E_RISK_PASSWORD ?? "LocalRiskSettlement123!";
const artifactFile = process.env.TEST026_PORTAL_ARTIFACT_FILE ?? "";
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

test.describe("TEST-026 QRY_LITE live portal E2E", () => {
  test.skip(!liveEnabled, "Set WEB_E2E_LIVE=1 to run TEST-026 live portal E2E.");
  test.skip(
    !orderId ||
      !querySurfaceId ||
      !assetObjectId ||
      !queryTemplateId ||
      !approvalTicketId ||
      !caseId ||
      !buyerUserId ||
      !orderAmount,
    "Set WEB_E2E_QRY_LITE_* env vars before running TEST-026 live portal E2E.",
  );
  test.describe.configure({ timeout: 150_000 });

  test("seller grant, buyer run/read, risk refund stay on formal portal/API boundary", async ({
    context,
    page,
    request,
  }) => {
    const restrictedRequests = watchRestrictedBrowserRequests(page);
    const sellerToken = await fetchAccessToken(request, sellerUsername, sellerPassword);
    const buyerToken = await fetchAccessToken(request, buyerUsername, buyerPassword);
    const riskToken = await fetchAccessToken(request, riskUsername, riskPassword);
    const runSuffix = `${Date.now()}`;
    const grantExecutionRule = {
      entrypoint: "portal-live",
      task_id: "TEST-026",
      allowed_parameters: "schema-bound",
    };
    const grantOutputBoundary = {
      allowed_formats: ["json"],
      max_rows: 5,
      max_cells: 15,
    };
    const grantQuota = {
      max_runs: 5,
      daily_limit: 2,
      monthly_limit: 8,
    };

    await installPortalBearerSession(context, sellerToken, "TEST-026 seller bearer");

    await page.goto("/");
    await expect(
      page.getByText("角色 seller_operator", { exact: true }),
    ).toBeVisible({ timeout: 20_000 });
    await page.goto(`/delivery/orders/${orderId}/template-query`);
    await expect(page.getByText("订单交付状态")).toBeVisible({ timeout: 20_000 });
    await expect(
      page.getByRole("heading", { name: "模板查询授权" }).first(),
    ).toBeVisible();
    await expect(page.locator('input[name="query_surface_id"]')).toBeVisible();

    const grantResponsePromise = page.waitForResponse((response) =>
      response.request().method() === "POST" &&
      response.url().includes(`/api/platform/api/v1/orders/${orderId}/template-grants`)
    );

    await page.locator('input[name="query_surface_id"]').fill(querySurfaceId);
    await page.locator('input[name="allowed_template_ids"]').fill(queryTemplateId);
    await page.locator('input[name="asset_object_id"]').fill(assetObjectId);
    await page.locator('textarea[name="execution_rule_snapshot"]').fill(
      JSON.stringify(grantExecutionRule),
    );
    await page.locator('textarea[name="output_boundary_json"]').fill(
      JSON.stringify(grantOutputBoundary),
    );
    await page.locator('textarea[name="run_quota_json"]').fill(
      JSON.stringify(grantQuota),
    );
    await page.locator('input[name="confirm_scope"]').check();
    await page.locator('input[name="confirm_audit"]').check();
    await page.getByRole("button", { name: "提交" }).click();

    const grantResponse = await grantResponsePromise;
    expect(grantResponse.ok()).toBeTruthy();
    const grantPayload = await grantResponse.json();
    const grantData = unwrapData(grantPayload);
    const templateQueryGrantId = readString(
      readRecord(grantData)?.template_query_grant_id,
    );
    expect(templateQueryGrantId).toBeTruthy();
    expect(readString(readRecord(grantData)?.grant_status)).toBe("active");
    expect(readString(readRecord(grantData)?.current_state)).toBe("template_authorized");
    await expect(page.getByText("接口返回摘要")).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("template_authorized").first()).toBeVisible();

    await installPortalBearerSession(context, buyerToken, "TEST-026 buyer bearer");
    await page.goto(`/delivery/orders/${orderId}/query-runs`);
    await expect(
      page.getByText("角色 buyer_operator", { exact: true }),
    ).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("执行模板查询")).toBeVisible();

    const runResponsePromise = page.waitForResponse((response) =>
      response.request().method() === "POST" &&
      response.url().includes(`/api/platform/api/v1/orders/${orderId}/template-runs`)
    );

    await page.locator('input[name="query_template_id"]').fill(queryTemplateId);
    await page
      .locator('input[name="template_query_grant_id"]')
      .fill(templateQueryGrantId ?? "");
    await page.locator('input[name="requester_user_id"]').fill(buyerUserId);
    await page.locator('input[name="approval_ticket_id"]').fill(approvalTicketId);
    await page.locator('textarea[name="request_payload_json"]').fill(
      JSON.stringify({ city: "Shanghai", radius_km: 3, limit: 2 }),
    );
    await page.locator('textarea[name="output_boundary_json"]').fill(
      JSON.stringify({
        selected_format: "json",
        allowed_formats: ["json"],
        max_rows: 2,
        max_cells: 6,
      }),
    );
    await page.locator('textarea[name="execution_metadata_json"]').fill(
      JSON.stringify({ task_id: "TEST-026", entrypoint: "portal-live" }),
    );
    await page.locator('input[name="confirm_scope"]').check();
    await page.locator('input[name="confirm_audit"]').check();
    await page.getByRole("button", { name: "提交" }).click();

    const runResponse = await runResponsePromise;
    expect(runResponse.ok()).toBeTruthy();
    const runPayload = await runResponse.json();
    const runData = unwrapData(runPayload);
    const queryRunId = readString(readRecord(runData)?.query_run_id);
    expect(queryRunId).toBeTruthy();
    expect(readString(readRecord(runData)?.status)).toBe("completed");
    expect(readString(readRecord(runData)?.current_state)).toBe("query_executed");
    expect(readString(readRecord(runData)?.approval_ticket_id)).toBe(approvalTicketId);
    expect(readRecord(runData)?.result_row_count).toBe(2);
    await expect(page.getByText("completed").first()).toBeVisible({ timeout: 20_000 });

    const queryRunsPageResponse = page.waitForResponse((response) =>
      response.request().method() === "GET" &&
      response.url().includes(`/api/platform/api/v1/orders/${orderId}/template-runs`)
    );
    await page.goto(`/delivery/orders/${orderId}/query-runs`);
    const pageRunsResponse = await queryRunsPageResponse;
    expect(pageRunsResponse.ok()).toBeTruthy();
    await expect(page.getByText("查询运行记录")).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("completed").first()).toBeVisible();

    const queryRunsPayload = await fetchPlatformJson(
      request,
      `/api/v1/orders/${orderId}/template-runs`,
      buildBearerHeaders(buyerToken, `test026-query-runs-${runSuffix}`),
    );
    const queryRunsData = unwrapData(queryRunsPayload);
    const queryRuns = readArray(readRecord(queryRunsData)?.query_runs);
    const targetRun = queryRuns.find(
      (entry) => readString(readRecord(entry)?.query_run_id) === queryRunId,
    );
    expect(Boolean(targetRun)).toBeTruthy();
    expect(readString(readRecord(targetRun)?.status)).toBe("completed");
    expect(readRecord(targetRun)?.result_row_count).toBe(2);
    await expect(page.getByText("tpl_demo_s5_location_score_v1").first()).toBeVisible();
    await expect(page.getByText("rows=2").first()).toBeVisible();

    const orderAfterRun = await fetchPlatformJson(
      request,
      `/api/v1/orders/${orderId}`,
      buildBearerHeaders(buyerToken, `test026-order-run-${runSuffix}`),
    );
    const orderAfterRunData = unwrapData(orderAfterRun);
    expect(readString(readRecord(orderAfterRunData)?.current_state)).toBe("query_executed");
    expect(readString(readRecord(orderAfterRunData)?.delivery_status)).toBe("delivered");
    expect(readString(readRecord(orderAfterRunData)?.acceptance_status)).toBe("accepted");

    await installPortalBearerSession(context, riskToken, "TEST-026 risk bearer");
    await page.goto(`/billing/refunds?order_id=${orderId}&case_id=${caseId}`);
    await expect(
      page.getByText("角色 platform_risk_settlement", { exact: true }),
    ).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("执行退款")).toBeVisible();

    const refundResponsePromise = page.waitForResponse((response) =>
      response.request().method() === "POST" &&
      response.url().includes("/api/platform/api/v1/refunds")
    );

    const refundForm = page
      .locator("form")
      .filter({ has: page.locator('input[name="refund_mode"]') })
      .first();
    await refundForm.locator('input[name="amount"]').fill(orderAmount);
    await refundForm.locator('input[name="step_up_token"]').fill("test026-step-up-token");
    await refundForm.locator('input[name="confirm_liability"]').check();
    await refundForm.locator('input[name="confirm_step_up"]').check();
    await refundForm.locator('input[name="confirm_audit"]').check();
    await refundForm.getByRole("button", { name: "提交执行" }).click();

    const refundResponse = await refundResponsePromise;
    expect(refundResponse.ok()).toBeTruthy();
    const refundPayload = await refundResponse.json();
    const refundData = unwrapData(refundPayload);
    expect(readString(readRecord(refundData)?.current_status)).toBe("succeeded");
    expect(readString(readRecord(refundData)?.decision_code)).toBe("refund_full");
    expect(readRecord(refundData)?.step_up_bound).toBe(true);
    expect(readRecord(refundData)?.idempotent_replay).toBe(false);
    await expect(page.getByText("后端处理结果")).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText("succeeded").first()).toBeVisible();

    const billingAfterRefund = await fetchPlatformJson(
      request,
      `/api/v1/billing/${orderId}`,
      buildBearerHeaders(riskToken, `test026-billing-${runSuffix}`),
    );
    const billingData = unwrapData(billingAfterRefund);
    const refunds = readArray(readRecord(billingData)?.refunds);
    expect(refunds.length).toBe(1);
    expect(
      Number(
        readString(
          readRecord(readRecord(billingData)?.settlement_summary)?.refund_adjustment_amount,
        ),
      ),
    ).toBe(Number(orderAmount));

    const orderAfterRefund = await fetchPlatformJson(
      request,
      `/api/v1/orders/${orderId}`,
      buildBearerHeaders(riskToken, `test026-order-refund-${runSuffix}`),
    );
    const orderAfterRefundData = unwrapData(orderAfterRefund);
    expect(readString(readRecord(orderAfterRefundData)?.payment_status)).toBe("refunded");
    expect(readString(readRecord(orderAfterRefundData)?.dispute_status)).toBe("resolved");

    await writeArtifact({
      test_id: "test026-qry-lite-live",
      order_id: orderId,
      case_id: caseId,
      query_surface_id: querySurfaceId,
      asset_object_id: assetObjectId,
      query_template_id: queryTemplateId,
      approval_ticket_id: approvalTicketId,
      buyer_user_id: buyerUserId,
      order_amount: orderAmount,
      portal_proxy_requests: {
        grant_request_id: readString(readRecord(grantPayload)?.request_id),
        run_request_id: readString(readRecord(runPayload)?.request_id),
        query_runs_read_request_id: readString(readRecord(queryRunsPayload)?.request_id),
        refund_request_id: readString(readRecord(refundPayload)?.request_id),
        billing_read_request_id: readString(readRecord(billingAfterRefund)?.request_id),
      },
      grant_response: {
        template_query_grant_id: readString(readRecord(grantData)?.template_query_grant_id),
        grant_status: readString(readRecord(grantData)?.grant_status),
        current_state: readString(readRecord(grantData)?.current_state),
        delivery_status: readString(readRecord(grantData)?.delivery_status),
        allowed_template_ids: readArray(readRecord(grantData)?.allowed_template_ids),
      },
      run_response: {
        query_run_id: readString(readRecord(runData)?.query_run_id),
        status: readString(readRecord(runData)?.status),
        current_state: readString(readRecord(runData)?.current_state),
        result_row_count: readRecord(runData)?.result_row_count,
        result_object_id: readString(readRecord(runData)?.result_object_id),
        bucket_name: readString(readRecord(runData)?.bucket_name),
        object_key: readString(readRecord(runData)?.object_key),
        approval_ticket_id: readString(readRecord(runData)?.approval_ticket_id),
      },
      query_runs_read_response: {
        query_run_count: queryRuns.length,
        target_status: readString(readRecord(targetRun)?.status),
        target_result_row_count: readRecord(targetRun)?.result_row_count,
        target_policy_hits: readArray(readRecord(targetRun)?.policy_hits),
        target_query_template_name: readString(readRecord(targetRun)?.query_template_name),
      },
      order_after_run: {
        current_state: readString(readRecord(orderAfterRunData)?.current_state),
        delivery_status: readString(readRecord(orderAfterRunData)?.delivery_status),
        acceptance_status: readString(readRecord(orderAfterRunData)?.acceptance_status),
      },
      refund_response: {
        refund_id: readString(readRecord(refundData)?.refund_id),
        current_status: readString(readRecord(refundData)?.current_status),
        decision_code: readString(readRecord(refundData)?.decision_code),
        provider_key: readString(readRecord(refundData)?.provider_key),
        step_up_bound: readRecord(refundData)?.step_up_bound,
        idempotent_replay: readRecord(refundData)?.idempotent_replay,
      },
      billing_after_refund: {
        refund_count: refunds.length,
        refund_adjustment_amount: readString(
          readRecord(readRecord(billingData)?.settlement_summary)?.refund_adjustment_amount,
        ),
        summary_state: readString(
          readRecord(readRecord(billingData)?.settlement_summary)?.summary_state,
        ),
      },
      order_after_refund: {
        current_state: readString(readRecord(orderAfterRefundData)?.current_state),
        payment_status: readString(readRecord(orderAfterRefundData)?.payment_status),
        dispute_status: readString(readRecord(orderAfterRefundData)?.dispute_status),
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
