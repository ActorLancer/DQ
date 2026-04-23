import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createDeliveryClient } from "./delivery";

const orderId = "30000000-0000-0000-0000-000000000910";

function okResponse() {
  return new Response(JSON.stringify({ success: true, data: { data: {} } }), {
    status: 200,
    headers: {
      "content-type": "application/json",
    },
  });
}

describe("delivery domain client", () => {
  it("sends Idempotency-Key for delivery commit writes", async () => {
    const fetchImpl = vi
      .fn<typeof fetch>()
      .mockImplementation(() => Promise.resolve(okResponse()));
    const client = createDeliveryClient(
      new PlatformClient({ baseUrl: "http://127.0.0.1:8080", fetch: fetchImpl }),
    );

    await client.commitOrderDelivery(
      { id: orderId },
      {
        branch: "file",
        object_uri: "s3://delivery-objects/web-010.csv",
        delivery_commit_hash: "sha256:web010-delivery",
        receipt_hash: "sha256:web010-receipt",
      },
      { idempotencyKey: "web-010-delivery-commit-key" },
    );

    const [input, init] = fetchImpl.mock.calls[0] ?? [];
    expect(String(input)).toContain(`/api/v1/orders/${orderId}/deliver`);
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("x-idempotency-key")).toBe(
      "web-010-delivery-commit-key",
    );
  });

  it("sends Idempotency-Key for non-file delivery enablement writes", async () => {
    const fetchImpl = vi
      .fn<typeof fetch>()
      .mockImplementation(() => Promise.resolve(okResponse()));
    const client = createDeliveryClient(
      new PlatformClient({ baseUrl: "http://127.0.0.1:8080", fetch: fetchImpl }),
    );

    await client.manageRevisionSubscription(
      { id: orderId },
      { cadence: "monthly", delivery_channel: "file_ticket", start_version_no: 1 },
      { idempotencyKey: "web-010-subscription-key" },
    );
    await client.manageShareGrant(
      { id: orderId },
      {
        operation: "grant",
        recipient_ref: "buyer.demo",
        share_protocol: "presigned_read",
        access_locator: "share://controlled/web-010",
        expires_at: "2026-05-23T00:00:00.000Z",
        receipt_hash: "sha256:web010-share",
      },
      { idempotencyKey: "web-010-share-key" },
    );
    await client.manageTemplateGrant(
      { id: orderId },
      {
        query_surface_id: "40000000-0000-0000-0000-000000000901",
        allowed_template_ids: ["40000000-0000-0000-0000-000000000902"],
      },
      { idempotencyKey: "web-010-template-key" },
    );
    await client.manageSandboxWorkspace(
      { id: orderId },
      {
        query_surface_id: "40000000-0000-0000-0000-000000000901",
        workspace_name: "WEB-010 Sandbox",
      },
      { idempotencyKey: "web-010-sandbox-key" },
    );

    const calls = fetchImpl.mock.calls.map(([input, init]) => ({
      input: String(input),
      key: new Headers(init?.headers).get("x-idempotency-key"),
    }));
    expect(calls).toEqual([
      {
        input: `http://127.0.0.1:8080/api/v1/orders/${orderId}/subscriptions`,
        key: "web-010-subscription-key",
      },
      {
        input: `http://127.0.0.1:8080/api/v1/orders/${orderId}/share-grants`,
        key: "web-010-share-key",
      },
      {
        input: `http://127.0.0.1:8080/api/v1/orders/${orderId}/template-grants`,
        key: "web-010-template-key",
      },
      {
        input: `http://127.0.0.1:8080/api/v1/orders/${orderId}/sandbox-workspaces`,
        key: "web-010-sandbox-key",
      },
    ]);
  });

  it("keeps read APIs free of write idempotency headers", async () => {
    const fetchImpl = vi
      .fn<typeof fetch>()
      .mockImplementation(() => Promise.resolve(okResponse()));
    const client = createDeliveryClient(
      new PlatformClient({ baseUrl: "http://127.0.0.1:8080", fetch: fetchImpl }),
    );

    await client.issueDownloadTicket({ id: orderId });
    await client.getShareGrants({ id: orderId });
    await client.getApiUsageLog({ id: orderId });

    for (const [, init] of fetchImpl.mock.calls) {
      expect(init?.method).toBe("GET");
      expect(new Headers(init?.headers).get("x-idempotency-key")).toBeNull();
    }
  });
});
