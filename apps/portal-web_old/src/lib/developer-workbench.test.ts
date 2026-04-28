import { describe, expect, it } from "vitest";

import {
  buildPatchApplicationPayload,
  buildRotateSecretPayload,
  canManageApplications,
  canReadDeveloper,
  canReadDeveloperTrace,
  subjectDisplayName,
  subjectRoles,
  type SessionSubject,
} from "./developer-workbench";

const tenantAdmin: SessionSubject = {
  mode: "local_test_user",
  user_id: "10000000-0000-0000-0000-000000000302",
  org_id: "10000000-0000-0000-0000-000000000102",
  login_id: "tenant.admin",
  display_name: "Tenant Admin",
  tenant_id: "10000000-0000-0000-0000-000000000102",
  roles: ["tenant_admin"],
  auth_context_level: "aal1",
};

describe("portal developer workbench helpers", () => {
  it("keeps developer permissions aligned to formal roles", () => {
    expect(canReadDeveloper(tenantAdmin)).toBe(true);
    expect(canManageApplications(tenantAdmin)).toBe(true);
    expect(canReadDeveloperTrace(tenantAdmin)).toBe(true);
    expect(canManageApplications({ ...tenantAdmin, roles: ["seller_operator"] })).toBe(false);
  });

  it("builds app update and secret rotation payloads", () => {
    expect(
      buildPatchApplicationPayload({
        app_id: "44444444-4444-4444-4444-444444444444",
        app_name: " Updated App ",
        status: "active",
        idempotency_key: "web-016:patch",
      }),
    ).toEqual({ app_name: "Updated App", status: "active" });
    expect(
      buildRotateSecretPayload({
        app_id: "44444444-4444-4444-4444-444444444444",
        client_secret_hash: "",
        idempotency_key: "web-016:secret",
        step_up_token: "",
        step_up_challenge_id: "",
      }),
    ).toEqual({ client_secret_hash: undefined });
  });

  it("formats subject labels for sensitive pages", () => {
    expect(subjectDisplayName(tenantAdmin)).toBe("Tenant Admin");
    expect(subjectRoles(tenantAdmin)).toBe("tenant_admin");
  });
});
