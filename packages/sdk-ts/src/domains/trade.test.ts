import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createTradeClient } from "./trade";

describe("trade domain client", () => {
  it("sends Idempotency-Key for order creation", async () => {
    const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(
        JSON.stringify({
          data: {
            data: {
              order_id: "30000000-0000-0000-0000-000000000901",
              buyer_org_id: "10000000-0000-0000-0000-000000000201",
              seller_org_id: "10000000-0000-0000-0000-000000000202",
              product_id: "20000000-0000-0000-0000-000000000901",
              sku_id: "30000000-0000-0000-0000-000000000902",
              status: "created",
              payment_status: "unpaid",
              amount: "88.80",
              currency_code: "CNY",
              price_snapshot: {
                product_id: "20000000-0000-0000-0000-000000000901",
                sku_id: "30000000-0000-0000-0000-000000000902",
                sku_code: "FILE_STD-BASIC",
                sku_type: "FILE_STD",
                pricing_mode: "one_time",
                unit_price: "88.80",
                currency_code: "CNY",
                billing_mode: "one_time",
                refund_mode: "manual_refund",
                settlement_terms: {
                  settlement_basis: "acceptance",
                  settlement_mode: "platform_escrow",
                },
                tax_terms: {
                  tax_policy: "platform_default",
                  tax_code: "VAT",
                  tax_inclusive: false,
                },
                captured_at: "2026-04-23T00:00:00Z",
                source: "catalog.product_sku",
              },
              created_at: "2026-04-23T00:00:00Z",
            },
          },
        }),
        {
          status: 200,
          headers: {
            "content-type": "application/json",
          },
        },
      ),
    );
    const client = createTradeClient(
      new PlatformClient({ baseUrl: "http://127.0.0.1:8080", fetch: fetchImpl }),
    );

    await client.createOrder(
      {
        buyer_org_id: "10000000-0000-0000-0000-000000000201",
        product_id: "20000000-0000-0000-0000-000000000901",
        sku_id: "30000000-0000-0000-0000-000000000902",
        scenario_code: "S2",
      },
      { idempotencyKey: "web-009-order-create-key" },
    );

    const [input, init] = fetchImpl.mock.calls[0] ?? [];
    expect(String(input)).toBe("http://127.0.0.1:8080/api/v1/orders");
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("x-idempotency-key")).toBe(
      "web-009-order-create-key",
    );
  });

  it("sends Idempotency-Key for order cancel actions", async () => {
    const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(
        JSON.stringify({
          data: {
            data: {
              order_id: "30000000-0000-0000-0000-000000000901",
              previous_state: "created",
              current_state: "closed",
              refund_branch: "none",
              transitioned_at: "2026-04-23T00:00:00Z",
            },
          },
        }),
        {
          status: 200,
          headers: {
            "content-type": "application/json",
          },
        },
      ),
    );
    const client = createTradeClient(
      new PlatformClient({ baseUrl: "http://127.0.0.1:8080", fetch: fetchImpl }),
    );

    await client.cancelOrder(
      { id: "30000000-0000-0000-0000-000000000901" },
      { idempotencyKey: "web-009-order-cancel-key" },
    );

    const [input, init] = fetchImpl.mock.calls[0] ?? [];
    expect(String(input)).toContain(
      "/api/v1/orders/30000000-0000-0000-0000-000000000901/cancel",
    );
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("x-idempotency-key")).toBe(
      "web-009-order-cancel-key",
    );
  });
});
