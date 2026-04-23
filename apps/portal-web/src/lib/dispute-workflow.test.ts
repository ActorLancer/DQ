import { describe, expect, it, vi } from "vitest";

import {
  buildCreateDisputeCaseRequest,
  buildDisputeEvidenceFormData,
  buildResolveDisputeCaseRequest,
  canCreateDispute,
  canReadDispute,
  canResolveDispute,
  createDisputeIdempotencyKey,
  defaultCreateDisputeCaseValues,
  defaultResolveDisputeCaseValues,
  defaultUploadDisputeEvidenceValues,
  hiddenObjectPathNotice,
  resolveDisputeCaseFormSchema,
  selectActiveDisputeCase,
} from "./dispute-workflow";

describe("dispute workflow helpers", () => {
  it("builds create case payload with structured metadata", () => {
    const values = defaultCreateDisputeCaseValues(
      "30000000-0000-4000-8000-000000000901",
    );
    values.reason_code = "delivery_failed";
    values.claimed_amount = "9.50000000";
    values.metadata_json = JSON.stringify({ source: "WEB-013", channel: "portal" });

    expect(buildCreateDisputeCaseRequest(values)).toEqual({
      order_id: "30000000-0000-4000-8000-000000000901",
      reason_code: "delivery_failed",
      requested_resolution: "refund_full",
      claimed_amount: "9.50000000",
      evidence_scope: "delivery_receipt,download_log,object_hash",
      blocking_effect: "settlement_freeze",
      metadata: { source: "WEB-013", channel: "portal" },
    });
  });

  it("builds multipart evidence form without object path exposure", async () => {
    const values = defaultUploadDisputeEvidenceValues(
      "40000000-0000-4000-8000-000000000901",
    );
    values.object_type = "delivery_receipt";
    const data = buildDisputeEvidenceFormData(
      values,
      new Blob(["receipt"]),
      "receipt.txt",
    );

    expect(data.get("object_type")).toBe("delivery_receipt");
    expect(data.get("metadata_json")).toContain("portal_upload");
    const file = data.get("file");
    expect(file).toBeInstanceOf(Blob);
    expect(hiddenObjectPathNotice({
      evidence_id: "ev-1",
      case_id: values.case_id,
      object_type: values.object_type,
      object_uri: "cases/hidden/path",
      object_hash: "sha256:abc",
      metadata: {},
      created_at: "2026-04-23T00:00:00Z",
      idempotent_replay: false,
    })).toBe("evidence_hash=sha256:abc");
  });

  it("requires step-up token or challenge for resolve", () => {
    const values = defaultResolveDisputeCaseValues(
      "40000000-0000-4000-8000-000000000901",
    );
    values.confirm_sod = true;
    values.confirm_step_up = true;
    values.confirm_audit = true;

    expect(resolveDisputeCaseFormSchema.safeParse(values).success).toBe(false);
    values.step_up_token = "step-up-token";
    expect(resolveDisputeCaseFormSchema.safeParse(values).success).toBe(true);
    expect(buildResolveDisputeCaseRequest(values)).toMatchObject({
      decision_type: "manual_resolution",
      decision_code: "refund_full",
      liability_type: "seller",
      penalty_code: "seller_full_refund",
    });
  });

  it("uses formal V1 roles for dispute actions", () => {
    expect(canReadDispute({ roles: ["buyer_operator"] } as never)).toBe(true);
    expect(canCreateDispute({ roles: ["buyer_operator"] } as never)).toBe(true);
    expect(canResolveDispute({ roles: ["platform_risk_settlement"] } as never)).toBe(true);
    expect(canCreateDispute({ roles: ["tenant_admin"] } as never)).toBe(false);
    expect(canReadDispute({ roles: ["tenant_operator"] } as never)).toBe(false);
  });

  it("selects the latest order dispute relation unless case_id is explicit", () => {
    const order = {
      relations: {
        disputes: [
          {
            case_id: "40000000-0000-4000-8000-000000000001",
            updated_at: "2026-04-22T00:00:00Z",
          },
          {
            case_id: "40000000-0000-4000-8000-000000000002",
            updated_at: "2026-04-23T00:00:00Z",
          },
        ],
      },
    } as never;

    expect(selectActiveDisputeCase(order)?.case_id).toBe(
      "40000000-0000-4000-8000-000000000002",
    );
    expect(
      selectActiveDisputeCase(order, "40000000-0000-4000-8000-000000000001")
        ?.case_id,
    ).toBe("40000000-0000-4000-8000-000000000001");
  });

  it("creates WEB-013 idempotency keys", () => {
    vi.stubGlobal("crypto", {
      randomUUID: () => "00000000-0000-4000-8000-000000000013",
    });

    expect(createDisputeIdempotencyKey("case")).toBe(
      "web-013-dispute-case-00000000-0000-4000-8000-000000000013",
    );

    vi.unstubAllGlobals();
  });
});
