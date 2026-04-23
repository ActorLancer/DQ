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
});
