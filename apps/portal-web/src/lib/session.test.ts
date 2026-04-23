import { describe, expect, it } from "vitest";

import {
  buildSessionHeaders,
  inferLocalSessionTenantId,
  readPortalSessionPreview,
  type PortalSession,
} from "./session";

describe("portal session helpers", () => {
  it("injects formal tenant scope for local buyer operator sessions", () => {
    const session: PortalSession = {
      mode: "local",
      loginId: "iam018.buyer.operator@luna.local",
      role: "buyer_operator",
    };

    expect(inferLocalSessionTenantId(session.loginId, session.role)).toBe(
      "10000000-0000-0000-0000-000000000102",
    );
    expect(buildSessionHeaders(session)).toMatchObject({
      "x-login-id": "iam018.buyer.operator@luna.local",
      "x-role": "buyer_operator",
      "x-tenant-id": "10000000-0000-0000-0000-000000000102",
    });
    expect(readPortalSessionPreview(session)?.tenant_id).toBe(
      "10000000-0000-0000-0000-000000000102",
    );
  });

  it("keeps explicit local tenant scope when the dialog provides it", () => {
    const session: PortalSession = {
      mode: "local",
      loginId: "custom.local@luna.local",
      role: "tenant_admin",
      tenantId: "10000000-0000-0000-0000-000000000101",
    };

    expect(buildSessionHeaders(session)).toMatchObject({
      "x-login-id": "custom.local@luna.local",
      "x-role": "tenant_admin",
      "x-tenant-id": "10000000-0000-0000-0000-000000000101",
    });
    expect(readPortalSessionPreview(session)?.org_id).toBe(
      "10000000-0000-0000-0000-000000000101",
    );
  });
});
