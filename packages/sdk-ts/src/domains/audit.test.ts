import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createAuditClient } from "./audit";

describe("audit domain client", () => {
  it("reads order audit traces through the formal path parameter", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: { traces: [] } }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createAuditClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.getOrderAudit(
      { id: "30000000-0000-4000-8000-000000014001" },
      { page: 1, page_size: 20 },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "http://platform.test/api/v1/audit/orders/30000000-0000-4000-8000-000000014001?page=1&page_size=20",
      expect.objectContaining({ method: "GET" }),
    );
  });

  it("exports evidence packages with idempotency and step-up headers", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: {} }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createAuditClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.exportPackage(
      {
        ref_type: "order",
        ref_id: "30000000-0000-4000-8000-000000014001",
        reason: "监管抽查导出 WEB-014 证据包",
        masked_level: "masked",
        package_type: "forensic_export",
      },
      {
        idempotencyKey: "web-014:audit-export:demo",
        stepUpToken: "step-up-token",
      },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "http://platform.test/api/v1/audit/packages/export",
      expect.objectContaining({ method: "POST" }),
    );
    const headers = fetchMock.mock.calls[0]?.[1]?.headers as Headers;
    expect(headers.get("x-idempotency-key")).toBe("web-014:audit-export:demo");
    expect(headers.get("x-step-up-token")).toBe("step-up-token");
    expect(headers.get("content-type")).toBe("application/json");
  });

  it("can send a verified step-up challenge instead of a token", async () => {
    const fetchMock = vi.fn<typeof fetch>(async () =>
      new Response(JSON.stringify({ success: true, data: {} }), {
        headers: { "content-type": "application/json" },
      }),
    );
    const sdk = createAuditClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchMock }),
    );

    await sdk.exportPackage(
      {
        ref_type: "dispute_case",
        ref_id: "40000000-0000-4000-8000-000000014001",
        reason: "争议复核导出证据包",
      },
      {
        idempotencyKey: "web-014:audit-export:challenge",
        stepUpChallengeId: "challenge-001",
      },
    );

    const headers = fetchMock.mock.calls[0]?.[1]?.headers as Headers;
    expect(headers.get("x-step-up-challenge-id")).toBe("challenge-001");
    expect(headers.get("x-step-up-token")).toBeNull();
  });
});
