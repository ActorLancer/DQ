import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createCatalogClient } from "./catalog";

describe("catalog domain client", () => {
  it("sends Idempotency-Key for product draft writes", async () => {
    const fetchImpl = vi
      .fn<typeof fetch>()
      .mockResolvedValue(
        new Response(
          JSON.stringify({
            success: true,
            data: {
              product_id: "20000000-0000-0000-0000-000000000701",
              asset_id: "20000000-0000-0000-0000-000000000702",
              asset_version_id: "20000000-0000-0000-0000-000000000703",
              seller_org_id: "10000000-0000-0000-0000-000000000101",
              title: "WEB-007 商品",
              category: "industrial_data",
              product_type: "data_product",
              status: "draft",
              price_mode: "one_time",
              price: "1000",
              currency_code: "CNY",
              delivery_type: "file_download",
              created_at: "2026-04-23T00:00:00Z",
              updated_at: "2026-04-23T00:00:00Z",
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
    const client = createCatalogClient(
      new PlatformClient({ baseUrl: "http://127.0.0.1:8080", fetch: fetchImpl }),
    );

    await client.createProductDraft(
      {
        asset_id: "20000000-0000-0000-0000-000000000702",
        asset_version_id: "20000000-0000-0000-0000-000000000703",
        seller_org_id: "10000000-0000-0000-0000-000000000101",
        title: "WEB-007 商品",
        category: "industrial_data",
        product_type: "data_product",
        delivery_type: "file_download",
        allowed_usage: ["internal_analysis"],
        use_cases: ["risk_control"],
        metadata: {
          task_id: "WEB-007",
        },
      },
      { idempotencyKey: "web-007-test-key" },
    );

    const [, init] = fetchImpl.mock.calls[0] ?? [];
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("x-idempotency-key")).toBe(
      "web-007-test-key",
    );
  });
});
