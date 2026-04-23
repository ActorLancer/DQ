import { PlatformApiError } from "@datab/sdk-ts";
import { describe, expect, it } from "vitest";

import { formatAuditError } from "./audit-trace";
import { formatDeveloperError } from "./developer-workbench";
import { formatOpsError } from "./ops-workbench";
import { formatReviewError } from "./review-workbench";

describe("console error copy", () => {
  it("maps structured console errors to shared titles and descriptions", () => {
    const auditError = formatAuditError(
      new PlatformApiError(
        403,
        {
          code: "AUDIT_UNMASKED_VIEW_REQUIRES_STEP_UP",
          message: "AUDIT_UNMASKED_VIEW_REQUIRES_STEP_UP: challenge required",
          request_id: "req-audit",
        },
        "fallback",
      ),
    );
    const opsError = formatOpsError(
      new PlatformApiError(
        403,
        {
          code: "SEARCH_REINDEX_FORBIDDEN",
          message: "SEARCH_REINDEX_FORBIDDEN: missing permission",
          request_id: "req-ops",
        },
        "fallback",
      ),
    );
    const reviewError = formatReviewError(
      new PlatformApiError(
        400,
        {
          code: "CAT_VALIDATION_FAILED",
          message: "CAT_VALIDATION_FAILED: invalid payload",
          request_id: "req-review",
        },
        "fallback",
      ),
    );

    expect(auditError).toMatchObject({
      title: "查看未脱敏审计内容需 step-up · AUDIT_UNMASKED_VIEW_REQUIRES_STEP_UP",
      requestId: "req-audit",
    });
    expect(auditError.message).toContain("请完成 step-up");

    expect(opsError.title).toBe("不允许发起搜索重建 · SEARCH_REINDEX_FORBIDDEN");
    expect(opsError.message).toContain("当前主体权限");
    expect(opsError.requestId).toBe("req-ops");

    expect(reviewError.title).toBe("目录与商品请求校验失败 · CAT_VALIDATION_FAILED");
    expect(reviewError.message).toContain("请检查请求字段");
    expect(reviewError.requestId).toBe("req-review");
  });

  it("uses internal error copy for client-side exceptions", () => {
    expect(formatDeveloperError(new Error("boom"))).toContain("平台内部错误");
    expect(formatDeveloperError(new Error("boom"))).toContain("INTERNAL_ERROR");
  });
});
