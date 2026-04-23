import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createRecommendationClient } from "./recommendation";

describe("recommendation domain client", () => {
  it("reads recommendation ops placements and ranking profiles", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: [] }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createRecommendationClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.listPlacements();
    await sdk.listRankingProfiles();

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://platform.test/api/v1/ops/recommendation/placements",
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      "http://platform.test/api/v1/ops/recommendation/ranking-profiles",
    );
  });

  it("patches recommendation placement and ranking profile with write controls", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: {} }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createRecommendationClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.patchPlacement(
      { placement_code: "home_featured" },
      {
        status: "active",
        metadata: {},
      },
      {
        idempotencyKey: "idem-placement",
        stepUpToken: "10000000-0000-0000-0000-000000000001",
      },
    );
    await sdk.patchRankingProfile(
      { id: "20000000-0000-0000-0000-000000000001" },
      {
        status: "active",
        explain_codes: ["ops:web015"],
      },
      {
        idempotencyKey: "idem-rec-ranking",
        stepUpChallengeId: "20000000-0000-0000-0000-000000000002",
      },
    );

    const placementHeaders = new Headers(fetchMock.mock.calls[0]?.[1]?.headers);
    const rankingHeaders = new Headers(fetchMock.mock.calls[1]?.[1]?.headers);
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://platform.test/api/v1/ops/recommendation/placements/home_featured",
    );
    expect(placementHeaders.get("x-idempotency-key")).toBe("idem-placement");
    expect(placementHeaders.get("x-step-up-token")).toBe(
      "10000000-0000-0000-0000-000000000001",
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      "http://platform.test/api/v1/ops/recommendation/ranking-profiles/20000000-0000-0000-0000-000000000001",
    );
    expect(rankingHeaders.get("x-idempotency-key")).toBe("idem-rec-ranking");
    expect(rankingHeaders.get("x-step-up-challenge-id")).toBe(
      "20000000-0000-0000-0000-000000000002",
    );
  });

  it("submits recommendation rebuild with idempotency and step-up", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: { scope: "all" } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createRecommendationClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.rebuild(
      {
        scope: "all",
        placement_code: "home_featured",
        purge_cache: true,
      },
      {
        idempotencyKey: "idem-rebuild",
        stepUpToken: "30000000-0000-0000-0000-000000000001",
      },
    );

    const headers = new Headers(fetchMock.mock.calls[0]?.[1]?.headers);
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://platform.test/api/v1/ops/recommendation/rebuild",
    );
    expect(headers.get("x-idempotency-key")).toBe("idem-rebuild");
    expect(headers.get("x-step-up-token")).toBe(
      "30000000-0000-0000-0000-000000000001",
    );
  });
});
