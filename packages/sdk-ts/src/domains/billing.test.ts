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

  it("creates dispute cases with idempotency headers", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ code: 0, message: "ok", data: { case_id: "case-1" } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createBillingClient(new PlatformClient({
      baseUrl: "http://platform.test",
      fetch: fetchMock,
    }));

    await sdk.createDisputeCase(
      {
        order_id: "30000000-0000-4000-8000-000000000901",
        reason_code: "delivery_failed",
        requested_resolution: "refund_full",
        metadata: { source: "WEB-013" },
      },
      { idempotencyKey: "idem-case-001" },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "http://platform.test/api/v1/cases",
      expect.objectContaining({ method: "POST" }),
    );
    const headers = fetchMock.mock.calls[0]?.[1]?.headers as Headers;
    expect(headers.get("x-idempotency-key")).toBe("idem-case-001");
    expect(headers.get("content-type")).toBe("application/json");
  });

  it("uploads dispute evidence as multipart without forcing json content type", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ code: 0, message: "ok", data: { evidence_id: "ev-1" } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createBillingClient(new PlatformClient({
      baseUrl: "http://platform.test",
      fetch: fetchMock,
    }));
    const formData = new FormData();
    formData.set("object_type", "delivery_receipt");
    formData.set("metadata_json", JSON.stringify({ source: "WEB-013" }));
    formData.set("file", new Blob(["receipt"]), "receipt.txt");

    await sdk.uploadDisputeEvidence(
      { id: "40000000-0000-4000-8000-000000000901" },
      formData,
      { idempotencyKey: "idem-evidence-001" },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "http://platform.test/api/v1/cases/40000000-0000-4000-8000-000000000901/evidence",
      expect.objectContaining({ method: "POST", body: formData }),
    );
    const headers = fetchMock.mock.calls[0]?.[1]?.headers as Headers;
    expect(headers.get("x-idempotency-key")).toBe("idem-evidence-001");
    expect(headers.get("content-type")).toBeNull();
  });

  it("resolves dispute cases with step-up and idempotency headers", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ code: 0, message: "ok", data: { case_id: "case-1" } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createBillingClient(new PlatformClient({
      baseUrl: "http://platform.test",
      fetch: fetchMock,
    }));

    await sdk.resolveDisputeCase(
      { id: "40000000-0000-4000-8000-000000000901" },
      {
        decision_type: "manual_resolution",
        decision_code: "refund_full",
        penalty_code: "seller_full_refund",
        decision_text: "Delivery evidence failed hash verification.",
      },
      {
        idempotencyKey: "idem-resolve-001",
        stepUpToken: "step-up-token",
      },
    );

    const headers = fetchMock.mock.calls[0]?.[1]?.headers as Headers;
    expect(headers.get("x-idempotency-key")).toBe("idem-resolve-001");
    expect(headers.get("x-step-up-token")).toBe("step-up-token");
  });

  it("binds developer mock payment simulations to formal endpoints", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(
        JSON.stringify({
          code: 0,
          message: "ok",
          data: {
            mock_payment_case_id: "case-1",
            payment_intent_id: "30000000-0000-0000-0000-000000000101",
            scenario_type: "success",
            provider_key: "mock_payment_provider",
            provider_kind: "mock",
            provider_event_id: "evt-1",
            provider_status: "succeeded",
            webhook_processed_status: "processed",
            duplicate_webhook: false,
          },
        }),
        {
          headers: { "content-type": "application/json" },
        },
      ),
    );
    const sdk = createBillingClient(new PlatformClient({
      baseUrl: "http://platform.test",
      fetch: fetchMock,
    }));
    const path = { id: "30000000-0000-0000-0000-000000000101" };

    await sdk.simulateMockPaymentSuccess(
      path,
      { delay_seconds: 0, duplicate_webhook: false, partial_refund_amount: null },
      { idempotencyKey: "web-016:mock-payment-success" },
    );
    await sdk.simulateMockPaymentFail(
      path,
      { delay_seconds: 1, duplicate_webhook: true, partial_refund_amount: null },
      { idempotencyKey: "web-016:mock-payment-fail" },
    );
    await sdk.simulateMockPaymentTimeout(
      path,
      { delay_seconds: 2, duplicate_webhook: false, partial_refund_amount: null },
      { idempotencyKey: "web-016:mock-payment-timeout" },
    );

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://platform.test/api/v1/mock/payments/30000000-0000-0000-0000-000000000101/simulate-success",
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      "http://platform.test/api/v1/mock/payments/30000000-0000-0000-0000-000000000101/simulate-fail",
    );
    expect(fetchMock.mock.calls[2]?.[0]).toBe(
      "http://platform.test/api/v1/mock/payments/30000000-0000-0000-0000-000000000101/simulate-timeout",
    );
    expect(new Headers(fetchMock.mock.calls[0]?.[1]?.headers).get("x-idempotency-key")).toBe(
      "web-016:mock-payment-success",
    );
    expect(new Headers(fetchMock.mock.calls[2]?.[1]?.headers).get("x-idempotency-key")).toBe(
      "web-016:mock-payment-timeout",
    );
  });
});
