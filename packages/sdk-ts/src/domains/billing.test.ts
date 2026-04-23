import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createBillingClient } from "./billing";

describe("billing domain client", () => {
  it("reads billing detail by order id", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ code: 0, message: "ok", data: { order_id: "ord-1" } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createBillingClient(new PlatformClient({
      baseUrl: "http://platform.test",
      fetch: fetchMock,
    }));

    await sdk.getBillingOrder({ order_id: "ord-1" });

    expect(fetchMock).toHaveBeenCalledWith(
      "http://platform.test/api/v1/billing/ord-1",
      expect.objectContaining({ method: "GET" }),
    );
  });

  it("sends idempotency and step-up headers for refund and compensation", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ code: 0, message: "ok", data: {} }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createBillingClient(new PlatformClient({
      baseUrl: "http://platform.test",
      fetch: fetchMock,
    }));

    await sdk.executeRefund(
      {
        order_id: "order-1",
        case_id: "case-1",
        decision_code: "refund_full",
        amount: "10.00",
        reason_code: "seller_fault",
      },
      {
        idempotencyKey: "idem-refund-001",
        stepUpToken: "step-up-token",
      },
    );
    await sdk.executeCompensation(
      {
        order_id: "order-1",
        case_id: "case-1",
        decision_code: "compensation_full",
        amount: "3.00",
        reason_code: "sla_breach",
      },
      {
        idempotencyKey: "idem-comp-001",
        stepUpChallengeId: "challenge-1",
      },
    );

    const refundHeaders = fetchMock.mock.calls[0]?.[1]?.headers as Headers;
    expect(refundHeaders.get("x-idempotency-key")).toBe("idem-refund-001");
    expect(refundHeaders.get("x-step-up-token")).toBe("step-up-token");

    const compensationHeaders = fetchMock.mock.calls[1]?.[1]?.headers as Headers;
    expect(compensationHeaders.get("x-idempotency-key")).toBe("idem-comp-001");
    expect(compensationHeaders.get("x-step-up-challenge-id")).toBe("challenge-1");
  });
});
