import { afterEach, describe, expect, it, vi } from "vitest";

import { appendQuery, compilePath, PlatformClient } from "./http";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
});

describe("compilePath", () => {
  it("replaces and encodes required path params", () => {
    expect(
      compilePath("/api/v1/products/{id}/items/{itemId}", {
        id: "demo product",
        itemId: 42,
      }),
    ).toBe("/api/v1/products/demo%20product/items/42");
  });
});

describe("appendQuery", () => {
  it("serializes arrays and skips empty values", () => {
    expect(
      appendQuery("/api/v1/catalog/search", {
        q: "factory",
        tags: ["industrial", "featured"],
        price_min: 0,
        empty: "",
        skip: undefined,
      }),
    ).toBe(
      "/api/v1/catalog/search?q=factory&tags=industrial&tags=featured&price_min=0",
    );
  });

  it("preserves origin for absolute URLs", () => {
    expect(
      appendQuery("http://127.0.0.1:8080/api/v1/catalog/search", {
        q: "工业",
        page: 1,
        page_size: 3,
      }),
    ).toBe(
      "http://127.0.0.1:8080/api/v1/catalog/search?q=%E5%B7%A5%E4%B8%9A&page=1&page_size=3",
    );
  });
});

describe("PlatformClient", () => {
  it("binds global fetch at call time for relative browser URLs", async () => {
    const staleFetch = vi
      .fn<typeof fetch>()
      .mockResolvedValue(new Response(JSON.stringify({ success: false }), {
        status: 200,
        headers: {
          "content-type": "application/json",
        },
      }));
    const activeFetch = vi
      .fn<typeof fetch>()
      .mockResolvedValue(new Response(JSON.stringify({ success: true }), {
        status: 200,
        headers: {
          "content-type": "application/json",
        },
      }));

    globalThis.fetch = staleFetch;
    const client = new PlatformClient({
      baseUrl: "/api/platform",
    });

    globalThis.fetch = activeFetch;

    await expect(
      client.getJson<{ success: boolean }>("/health/ready"),
    ).resolves.toEqual({ success: true });
    expect(staleFetch).not.toHaveBeenCalled();
    expect(activeFetch).toHaveBeenCalledWith(
      "/api/platform/health/ready",
      expect.objectContaining({
        method: "GET",
      }),
    );
  });
});
