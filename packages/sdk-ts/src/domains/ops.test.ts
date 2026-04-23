import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createOpsClient } from "./ops";

describe("ops domain client", () => {
  it("reads the trade monitor overview for one order", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: { order_id: "order-1" } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createOpsClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.getTradeMonitorOverview({ orderId: "order-1" });

    expect(fetchMock).toHaveBeenCalledWith(
      "http://platform.test/api/v1/ops/trade-monitor/orders/order-1",
      expect.objectContaining({ method: "GET" }),
    );
  });

  it("queries external facts and projection gaps with formal filters", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: { items: [] } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createOpsClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.listExternalFacts({
      order_id: "order-1",
      receipt_status: "confirmed",
      page: 1,
      page_size: 20,
    });
    await sdk.listProjectionGaps({
      order_id: "order-1",
      gap_status: "open",
      page: 1,
      page_size: 20,
    });

    expect(String(fetchMock.mock.calls[0]?.[0])).toBe(
      "http://platform.test/api/v1/ops/external-facts?order_id=order-1&receipt_status=confirmed&page=1&page_size=20",
    );
    expect(String(fetchMock.mock.calls[1]?.[0])).toBe(
      "http://platform.test/api/v1/ops/projection-gaps?order_id=order-1&gap_status=open&page=1&page_size=20",
    );
  });

  it("looks up consistency by formal ref path", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: {} }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createOpsClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.getConsistency({ refType: "order", refId: "order-1" });

    expect(fetchMock).toHaveBeenCalledWith(
      "http://platform.test/api/v1/ops/consistency/order/order-1",
      expect.objectContaining({ method: "GET" }),
    );
  });

  it("submits dry-run consistency reconcile with idempotency and step-up headers", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: { status: "dry_run_ready" } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createOpsClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.reconcileConsistency(
      {
        ref_type: "order",
        ref_id: "10000000-0000-0000-0000-000000000001",
        mode: "full",
        dry_run: true,
        reason: "preview consistency repair",
      },
      {
        idempotencyKey: "idem-consistency-1",
        stepUpChallengeId: "20000000-0000-0000-0000-000000000001",
      },
    );

    const headers = new Headers(fetchMock.mock.calls[0]?.[1]?.headers);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://platform.test/api/v1/ops/consistency/reconcile",
      expect.objectContaining({ method: "POST" }),
    );
    expect(headers.get("x-idempotency-key")).toBe("idem-consistency-1");
    expect(headers.get("x-step-up-challenge-id")).toBe(
      "20000000-0000-0000-0000-000000000001",
    );
  });

  it("queries and dry-run reprocesses dead letters through ops contract paths", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: { items: [] } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createOpsClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.listDeadLetters({
      reprocess_status: "not_reprocessed",
      page: 1,
      page_size: 20,
    });
    await sdk.reprocessDeadLetter(
      { id: "30000000-0000-0000-0000-000000000001" },
      {
        reason: "preview search-indexer reprocess",
        dry_run: true,
        metadata: { source: "web-015" },
      },
      {
        idempotencyKey: "idem-dead-letter-1",
        stepUpToken: "40000000-0000-0000-0000-000000000001",
      },
    );

    expect(String(fetchMock.mock.calls[0]?.[0])).toBe(
      "http://platform.test/api/v1/ops/dead-letters?reprocess_status=not_reprocessed&page=1&page_size=20",
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      "http://platform.test/api/v1/ops/dead-letters/30000000-0000-0000-0000-000000000001/reprocess",
    );
    const headers = new Headers(fetchMock.mock.calls[1]?.[1]?.headers);
    expect(headers.get("x-idempotency-key")).toBe("idem-dead-letter-1");
    expect(headers.get("x-step-up-token")).toBe(
      "40000000-0000-0000-0000-000000000001",
    );
  });

  it("queries notification audit through platform-core facade with step-up headers", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: { total: 1, records: [] } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createOpsClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.searchNotificationAudit(
      {
        order_id: "30000000-0000-0000-0000-000000000001",
        aggregate_type: "notification.dispatch_request",
        event_type: "notification.requested",
        target_topic: "dtp.notification.dispatch",
        notification_code: "payment.succeeded",
        template_code: "NOTIFY_PAYMENT_SUCCEEDED_V1",
        limit: 20,
        reason: "trace notification delivery",
      },
      {
        stepUpChallengeId: "50000000-0000-0000-0000-000000000001",
      },
    );

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://platform.test/api/v1/ops/notifications/audit/search",
    );
    const headers = new Headers(fetchMock.mock.calls[0]?.[1]?.headers);
    expect(headers.get("x-step-up-challenge-id")).toBe(
      "50000000-0000-0000-0000-000000000001",
    );
  });

  it("replays notification dead letters through platform-core facade paths", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(
        JSON.stringify({ success: true, data: { status: "reprocess_requested" } }),
        {
          headers: { "content-type": "application/json" },
        },
      ),
    );
    const sdk = createOpsClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.replayNotificationDeadLetter(
      { dead_letter_event_id: "60000000-0000-0000-0000-000000000001" },
      {
        dry_run: true,
        reason: "manual replay after incident review",
      },
      {
        idempotencyKey: "idem-notification-replay-1",
        stepUpToken: "70000000-0000-0000-0000-000000000001",
      },
    );

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://platform.test/api/v1/ops/notifications/dead-letters/60000000-0000-0000-0000-000000000001/replay",
    );
    const headers = new Headers(fetchMock.mock.calls[0]?.[1]?.headers);
    expect(headers.get("x-idempotency-key")).toBe("idem-notification-replay-1");
    expect(headers.get("x-step-up-token")).toBe(
      "70000000-0000-0000-0000-000000000001",
    );
  });
});
