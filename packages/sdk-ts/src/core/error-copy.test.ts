import { describe, expect, it } from "vitest";

import { PlatformApiError } from "./http";
import {
  describePlatformErrorCode,
  formatPlatformErrorForDisplay,
  normalizePlatformError,
} from "./error-copy";

describe("platform error copy", () => {
  it("maps formal dictionary codes to frozen titles", () => {
    expect(describePlatformErrorCode("SEARCH_BACKEND_UNAVAILABLE")).toMatchObject({
      code: "SEARCH_BACKEND_UNAVAILABLE",
      title: "搜索后端不可用",
      known: true,
    });
  });

  it("keeps compatibility codes readable for current backend responses", () => {
    expect(describePlatformErrorCode("TRD_STATE_CONFLICT")).toMatchObject({
      code: "TRD_STATE_CONFLICT",
      title: "订单状态冲突或业务前置条件不满足",
      known: true,
    });
  });

  it("falls back to domain and suffix heuristics for unmapped codes", () => {
    expect(describePlatformErrorCode("FILE_DELIVERY_COMMIT_FORBIDDEN")).toMatchObject({
      code: "FILE_DELIVERY_COMMIT_FORBIDDEN",
      title: "交付与执行当前操作不被允许",
      known: false,
    });
  });

  it("normalizes platform api errors into mapped display copy", () => {
    const normalized = normalizePlatformError(
      new PlatformApiError(
        409,
        {
          code: "ORDER_CREATE_FORBIDDEN",
          message: "ORDER_CREATE_FORBIDDEN: product is not listed",
          request_id: "req-web-019",
        },
        "fallback",
      ),
    );

    expect(normalized).toMatchObject({
      code: "ORDER_CREATE_FORBIDDEN",
      title: "当前主体或商品不允许下单",
      requestId: "req-web-019",
      backendMessage: "product is not listed",
    });
  });

  it("formats display strings with code and request_id", () => {
    const message = formatPlatformErrorForDisplay(
      new PlatformApiError(
        403,
        {
          code: "AUTH_INVALID_CREDENTIAL",
          message: "invalid credential",
          request_id: "req-auth",
        },
        "fallback",
      ),
    );

    expect(message).toContain("登录凭证错误");
    expect(message).toContain("AUTH_INVALID_CREDENTIAL");
    expect(message).toContain("request_id req-auth");
    expect(message).not.toContain("invalid credential");
  });
});
