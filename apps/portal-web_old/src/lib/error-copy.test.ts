import { PlatformApiError } from "@datab/sdk-ts";
import { describe, expect, it } from "vitest";

import { formatAcceptanceError } from "./acceptance-workflow";
import { formatBillingError } from "./billing-workflow";
import { formatDeliveryError } from "./delivery-workflow";
import { formatDeveloperError } from "./developer-workbench";
import { formatDisputeError } from "./dispute-workflow";
import { formatTradeError } from "./order-workflow";

describe("portal error copy", () => {
  it("maps formal and compatibility codes into shared display text", () => {
    const tradeError = formatTradeError(
      new PlatformApiError(
        409,
        {
          code: "TRD_STATE_CONFLICT",
          message: "TRD_STATE_CONFLICT: state mismatch",
          request_id: "req-trade",
        },
        "fallback",
      ),
    );
    const deliveryError = formatDeliveryError(
      new PlatformApiError(
        403,
        {
          code: "DOWNLOAD_TICKET_FORBIDDEN",
          message: "DOWNLOAD_TICKET_FORBIDDEN: scope mismatch",
          request_id: "req-delivery",
        },
        "fallback",
      ),
    );

    expect(tradeError).toContain("订单状态冲突或业务前置条件不满足");
    expect(tradeError).toContain("TRD_STATE_CONFLICT");
    expect(tradeError).toContain("request_id req-trade");

    expect(deliveryError).toContain("下载票据无效或无权使用");
    expect(deliveryError).toContain("DOWNLOAD_TICKET_FORBIDDEN");
    expect(deliveryError).toContain("request_id req-delivery");
  });

  it("keeps other portal workflows aligned to the same copy source", () => {
    const billingError = formatBillingError(new Error("boom"));
    const acceptanceError = formatAcceptanceError(new Error("boom"));
    const disputeError = formatDisputeError(new Error("boom"));
    const developerError = formatDeveloperError(new Error("boom"));

    expect(billingError).toContain("支付或账单通道执行失败");
    expect(billingError).toContain("BIL_PROVIDER_FAILED");
    expect(acceptanceError).toContain("交付状态非法");
    expect(acceptanceError).toContain("DELIVERY_STATUS_INVALID");
    expect(disputeError).toContain("争议状态非法");
    expect(disputeError).toContain("DISPUTE_STATUS_INVALID");
    expect(developerError).toContain("平台内部错误");
    expect(developerError).toContain("INTERNAL_ERROR");
  });
});
