import { expect, test, type Page } from "@playwright/test";

test.skip(
  process.env.WEB_E2E_PREVIEW !== "1",
  "Set WEB_E2E_PREVIEW=1 with NEXT_PUBLIC_WEB_ROUTE_PREVIEW=1 to run preview-state E2E.",
);

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

test("console home and scaffold pages are reachable", async ({ page }) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);
  await page.goto("/");
  await expect(
    page.getByText("控制台已接入正式路由、受控 API 代理和控制面认证会话。"),
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

  await page.goto("/ops/notifications");
  await expect(page.getByText("通知联查与补发")).toBeVisible();
  await expect(page.getByText("通知 dead letter replay")).toBeVisible();

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
  await page.getByRole("button", { name: "认证会话" }).click();
  await expect(page.getByText("Keycloak / IAM Session")).toBeVisible();
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

test("WEB-022 notification ops flow stays on platform-core facade", async ({
  page,
}) => {
  const restrictedRequests = watchRestrictedBrowserRequests(page);
  const notificationCalls: string[] = [];

  await page.route("**/api/platform/api/v1/auth/me", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        success: true,
        data: {
          mode: "local_test_user",
          user_id: "10000000-0000-0000-0000-000000000001",
          org_id: "00000000-0000-0000-0000-000000000000",
          login_id: "platform.ops@luna.local",
          display_name: "Platform Ops",
          tenant_id: "00000000-0000-0000-0000-000000000000",
          roles: ["platform_admin", "platform_audit_security"],
          auth_context_level: "aal2",
        },
      }),
    });
  });
  await page.route("**/api/platform/api/v1/ops/observability/overview", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        success: true,
        data: {
          backend_statuses: [
            {
              backend: {
                backend_key: "prometheus_main",
                backend_type: "prometheus",
              },
              probe_status: "up",
            },
          ],
          alert_summary: {
            open_count: 0,
            critical_count: 0,
          },
          recent_incidents: [],
        },
      }),
    });
  });
  await page.route(
    "**/api/platform/api/v1/ops/notifications/audit/search",
    async (route) => {
      notificationCalls.push(route.request().url());
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          success: true,
          data: {
            request_id: "req-web022-search",
            trace_id: "trace-web022-search",
            filters: {
              aggregate_type: "notification.dispatch_request",
              event_type: "notification.requested",
              target_topic: "dtp.notification.dispatch",
              limit: 20,
            },
            total: 1,
            records: [
              {
                event_id: "30000000-0000-4000-8000-000000000201",
                aggregate_id: "30000000-0000-0000-0000-000000000101",
                aggregate_type: "notification.dispatch_request",
                event_type: "notification.requested",
                target_topic: "dtp.notification.dispatch",
                request_id: "req-notif-send",
                trace_id: "trace-notif-send",
                notification_code: "payment.succeeded",
                template_code: "NOTIFY_PAYMENT_SUCCEEDED_V1",
                channel: "mock-log",
                audience_scope: "buyer",
                recipient: {
                  kind: "user",
                  address: "buyer@example.test",
                },
                source_event: {
                  aggregate_type: "billing.billing_event",
                  aggregate_id: "30000000-0000-0000-0000-000000000101",
                  event_type: "billing.event.recorded",
                },
                subject_refs: [
                  {
                    ref_type: "order",
                    ref_id: "30000000-0000-0000-0000-000000000101",
                  },
                ],
                links: [
                  {
                    link_code: "order_detail",
                    href: "/trade/orders/30000000-0000-0000-0000-000000000101",
                  },
                ],
                rendered_variables: {
                  order_id: "30000000-0000-0000-0000-000000000101",
                },
                current_status: "dead_lettered",
                current_attempt: 2,
                title: "Payment success notification",
                body: "Buyer escrow completed and order is pending delivery.",
                channel_result: {
                  provider: "mock-log",
                  status: "retrying",
                },
                retry_timeline: [
                  {
                    status: "failed",
                    message_text: "mock-log send failed",
                    created_at: "2026-04-24T08:00:00Z",
                    attempt: 1,
                    error: "mock-log unavailable",
                    payload: {
                      channel: "mock-log",
                    },
                  },
                  {
                    status: "dead_lettered",
                    message_text: "sent to dead letter",
                    created_at: "2026-04-24T08:05:00Z",
                    attempt: 2,
                    error: "manual replay required",
                    payload: {
                      channel: "mock-log",
                    },
                  },
                ],
                audit_timeline: [
                  {
                    action_name: "notification.dispatch.failed",
                    result_code: "failed",
                    event_time: "2026-04-24T08:00:00Z",
                    metadata: {
                      template_code: "NOTIFY_PAYMENT_SUCCEEDED_V1",
                    },
                  },
                ],
                dead_letter: {
                  dead_letter_event_id: "30000000-0000-4000-8000-000000000301",
                  target_topic: "dtp.notification.dispatch",
                  reprocess_status: "not_reprocessed",
                  failed_reason: "mock-log unavailable",
                  created_at: "2026-04-24T08:05:00Z",
                  last_failed_at: "2026-04-24T08:05:00Z",
                },
              },
            ],
          },
        }),
      });
    },
  );
  await page.route(
    "**/api/platform/api/v1/ops/notifications/dead-letters/*/replay",
    async (route) => {
      notificationCalls.push(route.request().url());
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          success: true,
          data: {
            dead_letter_event_id: "30000000-0000-4000-8000-000000000301",
            original_event_id: "30000000-0000-4000-8000-000000000201",
            replay_event_id: "30000000-0000-4000-8000-000000000401",
            request_id: "req-web022-replay",
            trace_id: "trace-web022-replay",
            dry_run: true,
            status: "dry_run_ready",
            notification_topic: "dtp.notification.dispatch",
            dead_letter_topic: "dtp.notification.dead-letter",
            template_code: "NOTIFY_PAYMENT_SUCCEEDED_V1",
            channel: "mock-log",
            title: "Payment success notification",
            body: "Buyer escrow completed and order is pending delivery.",
          },
        }),
      });
    },
  );

  await page.goto("/ops/notifications");
  await expect(page.getByText("通知联查与补发")).toBeVisible();
  await expect(page.getByText("Platform Ops", { exact: true })).toBeVisible();

  await page.getByLabel("reason").first().fill("trace notification delivery for incident review");
  await page
    .getByLabel("X-Step-Up-Token")
    .first()
    .fill("50000000-0000-0000-0000-000000000001");
  await page.getByRole("button", { name: "联查通知轨迹" }).click();

  await expect(page.getByText("dead letters")).toBeVisible();
  await expect(page.getByText("NOTIFY_PAYMENT_SUCCEEDED_V1").first()).toBeVisible();
  await page
    .getByRole("button", { name: "30000000-0000-4000-8000-000000000301" })
    .click();
  await expect(page.getByLabel("dead_letter_event_id")).toHaveValue(
    "30000000-0000-4000-8000-000000000301",
  );

  await page
    .getByLabel("reason")
    .nth(1)
    .fill("manual replay after incident review");
  await page
    .getByLabel("X-Step-Up-Token")
    .nth(1)
    .fill("60000000-0000-0000-0000-000000000001");
  await page.getByRole("button", { name: "提交 dry-run replay" }).click();

  await expect(page.getByText("通知 replay 结果")).toBeVisible();
  expect(
    notificationCalls.some((url) =>
      url.includes("/api/platform/api/v1/ops/notifications/audit/search"),
    ),
  ).toBe(true);
  expect(
    notificationCalls.some((url) =>
      url.includes(
        "/api/platform/api/v1/ops/notifications/dead-letters/30000000-0000-4000-8000-000000000301/replay",
      ),
    ),
  ).toBe(true);
  expect(
    notificationCalls.every((url) =>
      url.includes("/api/platform/api/v1/ops/notifications"),
    ),
  ).toBe(true);
  expect(restrictedRequests).toEqual([]);
});
