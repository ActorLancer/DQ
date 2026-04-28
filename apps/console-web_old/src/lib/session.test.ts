import { describe, expect, it } from "vitest";

import { buildSessionHeaders } from "./session";

describe("console session headers", () => {
  it("propagates verified local IAM subject headers for high-risk audit actions", () => {
    const headers = new Headers(
      buildSessionHeaders({
        mode: "local",
        loginId: "auditor.admin@luna.local",
        role: "platform_audit_security",
        userId: "10000000-0000-0000-0000-000000000354",
        tenantId: "10000000-0000-0000-0000-000000000103",
      }),
    );

    expect(headers.get("x-login-id")).toBe("auditor.admin@luna.local");
    expect(headers.get("x-role")).toBe("platform_audit_security");
    expect(headers.get("x-user-id")).toBe("10000000-0000-0000-0000-000000000354");
    expect(headers.get("x-tenant-id")).toBe("10000000-0000-0000-0000-000000000103");
  });

  it("keeps bearer token and verified subject mirrors together", () => {
    const headers = new Headers(
      buildSessionHeaders({
        mode: "bearer",
        accessToken: "token",
        userId: "10000000-0000-0000-0000-000000000354",
        tenantId: "10000000-0000-0000-0000-000000000103",
        role: "platform_audit_security",
      }),
    );

    expect(headers.get("authorization")).toBe("Bearer token");
    expect(headers.get("x-user-id")).toBe("10000000-0000-0000-0000-000000000354");
    expect(headers.get("x-role")).toBe("platform_audit_security");
  });
});
