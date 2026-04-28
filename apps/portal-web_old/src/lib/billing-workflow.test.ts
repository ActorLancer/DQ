import { describe, expect, it, vi } from "vitest";

import {
  buildCompensationExecutionRequest,
  buildRefundExecutionRequest,
  canExecuteBillingAdjustments,
  canReadBilling,
  createBillingIdempotencyKey,
  defaultCompensationExecutionValues,
  defaultRefundExecutionValues,
  refundExecutionFormSchema,
} from "./billing-workflow";

describe("billing workflow helpers", () => {
  it("builds refund request and keeps metadata structured", () => {
    const values = defaultRefundExecutionValues(
      "30000000-0000-4000-8000-000000000901",
      "40000000-0000-4000-8000-000000000901",
    );
    values.amount = "10.50000000";
    values.currency_code = "SGD";
    values.step_up_token = "step-up-token";
    values.confirm_liability = true;
    values.confirm_step_up = true;
    values.confirm_audit = true;

    expect(buildRefundExecutionRequest(values)).toMatchObject({
      order_id: "30000000-0000-4000-8000-000000000901",
      case_id: "40000000-0000-4000-8000-000000000901",
      decision_code: "refund_full",
      amount: "10.50000000",
      currency_code: "SGD",
      reason_code: "seller_fault",
      refund_mode: "manual_refund",
      metadata: { source: "WEB-012" },
    });
  });

  it("builds compensation request with manual transfer template", () => {
    const values = defaultCompensationExecutionValues(
      "30000000-0000-4000-8000-000000000901",
      "40000000-0000-4000-8000-000000000901",
    );
    values.amount = "3.00";
    values.step_up_challenge_id = "challenge-1";
    values.confirm_liability = true;
    values.confirm_step_up = true;
    values.confirm_audit = true;

    expect(buildCompensationExecutionRequest(values)).toMatchObject({
      decision_code: "compensation_full",
      amount: "3.00",
      reason_code: "sla_breach",
      compensation_mode: "manual_transfer",
      compensation_template: "COMPENSATION_FILE_V1",
    });
  });

  it("requires step-up evidence for high-risk refund execution", () => {
    const values = defaultRefundExecutionValues(
      "30000000-0000-4000-8000-000000000901",
      "40000000-0000-4000-8000-000000000901",
    );
    values.amount = "1.00";
    values.confirm_liability = true;
    values.confirm_step_up = true;
    values.confirm_audit = true;

    expect(refundExecutionFormSchema.safeParse(values).success).toBe(false);
    values.step_up_token = "step-up-token";
    expect(refundExecutionFormSchema.safeParse(values).success).toBe(true);
  });

  it("uses formal V1 roles for billing read and high-risk actions", () => {
    expect(canReadBilling({ roles: ["buyer_operator"] } as never)).toBe(true);
    expect(canReadBilling({ roles: ["tenant_admin"] } as never)).toBe(true);
    expect(canExecuteBillingAdjustments({ roles: ["platform_risk_settlement"] } as never)).toBe(true);
    expect(canExecuteBillingAdjustments({ roles: ["buyer_operator"] } as never)).toBe(false);
    expect(canReadBilling({ roles: ["tenant_operator"] } as never)).toBe(false);
  });

  it("creates WEB-012 idempotency keys", () => {
    vi.stubGlobal("crypto", {
      randomUUID: () => "00000000-0000-4000-8000-000000000012",
    });

    expect(createBillingIdempotencyKey("refund")).toBe(
      "web-012-billing-refund-00000000-0000-4000-8000-000000000012",
    );

    vi.unstubAllGlobals();
  });
});
