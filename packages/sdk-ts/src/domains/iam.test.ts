import { describe, expect, it, vi } from "vitest";

import { PlatformClient } from "../core/http";

import { createIamClient } from "./iam";

describe("iam domain client", () => {
  it("reads organization review queue through the formal IAM API", async () => {
    const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(
        JSON.stringify({
          success: true,
          data: [
            {
              org_id: "33333333-3333-3333-3333-333333333333",
              org_name: "WEB-008 Data Supplier",
              org_type: "seller",
              org_status: "pending_review",
              jurisdiction_code: "CN",
              compliance_level: "L2",
              certification_level: "verified_basic",
              whitelist_refs: [],
              graylist_refs: ["risk-ticket-demo"],
              blacklist_refs: [],
              review_status: "manual_review",
              risk_status: "watch",
              sellable_status: "restricted",
              freeze_reason: null,
              blacklist_active: false,
              created_at: "2026-04-23T00:00:00.000Z",
              updated_at: "2026-04-23T00:00:00.000Z",
            },
          ],
        }),
        {
          status: 200,
          headers: {
            "content-type": "application/json",
          },
        },
      ),
    );
    const client = createIamClient(
      new PlatformClient({ baseUrl: "http://127.0.0.1:8080", fetch: fetchImpl }),
    );

    const result = await client.listOrganizations({
      status: "pending_review",
      org_type: "seller",
    });

    const [input, init] = fetchImpl.mock.calls[0] ?? [];
    expect(String(input)).toBe(
      "http://127.0.0.1:8080/api/v1/iam/orgs?status=pending_review&org_type=seller",
    );
    expect(init?.method).toBe("GET");
    expect(result.data[0]?.review_status).toBe("manual_review");
  });

  it("manages developer applications with idempotency and secret status only", async () => {
    const fetchImpl = vi.fn<typeof fetch>(async () =>
      new Response(
        JSON.stringify({
          success: true,
          data: {
            app_id: "44444444-4444-4444-4444-444444444444",
            org_id: "33333333-3333-3333-3333-333333333333",
            app_name: "WEB-016 Test App",
            app_type: "api_client",
            status: "active",
            client_id: "web016-client",
            client_secret_status: "active",
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
    const client = createIamClient(
      new PlatformClient({ baseUrl: "http://platform.test", fetch: fetchImpl }),
    );

    await client.createApplication(
      {
        org_id: "33333333-3333-3333-3333-333333333333",
        app_name: "WEB-016 Test App",
        app_type: "api_client",
        client_id: "web016-client",
        client_secret_hash: "hash-web016",
      },
      { idempotencyKey: "web-016:create-app" },
    );
    await client.patchApplication(
      { id: "44444444-4444-4444-4444-444444444444" },
      { app_name: "WEB-016 Test App Updated", status: "active" },
      { idempotencyKey: "web-016:patch-app", stepUpChallengeId: "challenge-1" },
    );
    await client.rotateApplicationSecret(
      { id: "44444444-4444-4444-4444-444444444444" },
      { client_secret_hash: "hash-rotated-web016" },
      { idempotencyKey: "web-016:rotate-app-secret" },
    );
    await client.revokeApplicationSecret(
      { id: "44444444-4444-4444-4444-444444444444" },
      { idempotencyKey: "web-016:revoke-app-secret" },
    );

    expect(fetchImpl.mock.calls[0]?.[0]).toBe("http://platform.test/api/v1/apps");
    expect(fetchImpl.mock.calls[1]?.[0]).toBe(
      "http://platform.test/api/v1/apps/44444444-4444-4444-4444-444444444444",
    );
    expect(fetchImpl.mock.calls[2]?.[0]).toBe(
      "http://platform.test/api/v1/apps/44444444-4444-4444-4444-444444444444/credentials/rotate",
    );
    expect(fetchImpl.mock.calls[3]?.[0]).toBe(
      "http://platform.test/api/v1/apps/44444444-4444-4444-4444-444444444444/credentials/revoke",
    );
    expect(new Headers(fetchImpl.mock.calls[0]?.[1]?.headers).get("x-idempotency-key")).toBe(
      "web-016:create-app",
    );
    expect(new Headers(fetchImpl.mock.calls[1]?.[1]?.headers).get("x-step-up-challenge-id")).toBe(
      "challenge-1",
    );
    expect(new Headers(fetchImpl.mock.calls[3]?.[1]?.headers).get("x-idempotency-key")).toBe(
      "web-016:revoke-app-secret",
    );
    expect(fetchImpl.mock.calls[3]?.[1]?.body).toBeUndefined();
  });
});
