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
});
