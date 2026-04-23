import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createSearchClient } from "./search";

describe("search domain client", () => {
  it("queries search sync status with formal ops filters", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: [] }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createSearchClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.listSearchSync({
      entity_scope: "product",
      sync_status: "queued",
      limit: 20,
    });

    expect(String(fetchMock.mock.calls[0]?.[0])).toBe(
      "http://platform.test/api/v1/ops/search/sync?entity_scope=product&sync_status=queued&limit=20",
    );
  });

  it("sends idempotency and step-up headers for high-risk search ops writes", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: {} }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createSearchClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.reindex(
      {
        entity_scope: "product",
        mode: "full",
        force: true,
      },
      {
        idempotencyKey: "idem-search-reindex",
        stepUpToken: "10000000-0000-0000-0000-000000000001",
      },
    );
    await sdk.switchAlias(
      {
        entity_scope: "seller",
        next_index_name: "datab-seller-v2",
      },
      {
        idempotencyKey: "idem-search-alias",
        stepUpChallengeId: "10000000-0000-0000-0000-000000000002",
      },
    );

    const reindexHeaders = new Headers(fetchMock.mock.calls[0]?.[1]?.headers);
    const aliasHeaders = new Headers(fetchMock.mock.calls[1]?.[1]?.headers);
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://platform.test/api/v1/ops/search/reindex",
    );
    expect(reindexHeaders.get("x-idempotency-key")).toBe("idem-search-reindex");
    expect(reindexHeaders.get("x-step-up-token")).toBe(
      "10000000-0000-0000-0000-000000000001",
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      "http://platform.test/api/v1/ops/search/aliases/switch",
    );
    expect(aliasHeaders.get("x-idempotency-key")).toBe("idem-search-alias");
    expect(aliasHeaders.get("x-step-up-challenge-id")).toBe(
      "10000000-0000-0000-0000-000000000002",
    );
  });

  it("supports cache invalidation and ranking profile patch ops", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: {} }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createSearchClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.invalidateCache(
      {
        entity_scope: "all",
        purge_all: true,
      },
      { idempotencyKey: "idem-cache" },
    );
    await sdk.patchRankingProfile(
      { id: "20000000-0000-0000-0000-000000000001" },
      {
        status: "active",
      },
      {
        idempotencyKey: "idem-ranking",
        stepUpToken: "20000000-0000-0000-0000-000000000002",
      },
    );

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://platform.test/api/v1/ops/search/cache/invalidate",
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      "http://platform.test/api/v1/ops/search/ranking-profiles/20000000-0000-0000-0000-000000000001",
    );
    expect(new Headers(fetchMock.mock.calls[0]?.[1]?.headers).get("x-idempotency-key")).toBe(
      "idem-cache",
    );
    expect(new Headers(fetchMock.mock.calls[1]?.[1]?.headers).get("x-step-up-token")).toBe(
      "20000000-0000-0000-0000-000000000002",
    );
  });
});
