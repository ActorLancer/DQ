import { describe, expect, it } from "vitest";

import {
  buildCreateApplicationPayload,
  buildDeveloperTraceQuery,
  buildMockPaymentPayload,
  canManageApplications,
  canReadDeveloperTrace,
  canSimulateMockPayment,
  createDeveloperIdempotencyKey,
  formatDeveloperError,
  statusTone,
  type SessionSubject,
} from "./developer-workbench";

const developerSubject: SessionSubject = {
  mode: "local_test_user",
  user_id: "10000000-0000-0000-0000-000000000352",
  org_id: "10000000-0000-0000-0000-000000000102",
  login_id: "developer.demo",
  display_name: "Developer Demo",
  tenant_id: "10000000-0000-0000-0000-000000000102",
  roles: ["tenant_developer"],
  auth_context_level: "aal1",
};

describe("developer workbench helpers", () => {
  it("maps formal roles to developer permissions", () => {
    expect(canManageApplications(developerSubject)).toBe(true);
    expect(canReadDeveloperTrace(developerSubject)).toBe(true);
    expect(canSimulateMockPayment(developerSubject)).toBe(true);
    expect(canManageApplications({ ...developerSubject, roles: ["seller_operator"] })).toBe(false);
  });

  it("builds SDK payloads without plaintext secret leakage", () => {
    expect(
      buildCreateApplicationPayload({
        org_id: developerSubject.tenant_id ?? "",
        app_name: " WEB-016 App ",
        app_type: "api_client",
        client_id: " web016-client ",
        client_secret_hash: " hash-only ",
        idempotency_key: "web-016:create",
      }),
    ).toEqual({
      org_id: developerSubject.tenant_id,
      app_name: "WEB-016 App",
      app_type: "api_client",
      client_id: "web016-client",
      client_secret_hash: "hash-only",
    });
  });

  it("builds trace and mock payment requests against formal selectors", () => {
    expect(buildDeveloperTraceQuery({ lookup_mode: "tx_hash", lookup_value: " 0xtx " })).toEqual({
      tx_hash: "0xtx",
    });
    expect(
      buildMockPaymentPayload({
        payment_intent_id: "30000000-0000-0000-0000-000000000101",
        scenario: "success",
        delay_seconds: 0,
        duplicate_webhook: false,
        partial_refund_amount: "",
        idempotency_key: "web-016:mock",
        step_up_token: "",
        step_up_challenge_id: "",
      }),
    ).toEqual({
      delay_seconds: 0,
      duplicate_webhook: false,
      partial_refund_amount: undefined,
    });
  });

  it("formats idempotency keys, status tones and error fallbacks", () => {
    expect(createDeveloperIdempotencyKey("app-create")).toMatch(/^web-016:app-create:/);
    expect(statusTone("active")).toBe("ok");
    expect(statusTone("revoked")).toBe("danger");
    expect(formatDeveloperError(new Error("boom"))).toContain("平台内部错误");
    expect(formatDeveloperError(new Error("boom"))).toContain("INTERNAL_ERROR");
  });
});
