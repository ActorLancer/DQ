import { describe, expect, it } from "vitest";

import {
  aliasSwitchSchema,
  buildAliasSwitchPayload,
  buildConsistencyPath,
  buildNotificationAuditSearchPayload,
  buildNotificationReplayPayload,
  buildRecommendationRebuildPayload,
  recommendationRebuildSchema,
  buildSearchReindexPayload,
  canManageSearchOps,
  canReadConsistency,
  canReadNotificationOps,
  consistencyReconcileSchema,
  createOpsIdempotencyKey,
  deadLetterReprocessSchema,
  notificationAuditSearchSchema,
  notificationReplaySchema,
  parseJsonObject,
  searchReindexSchema,
  statusTone,
  type SessionSubject,
} from "./ops-workbench";

const platformAdmin = {
  user_id: "10000000-0000-0000-0000-000000000001",
  login_id: "ops.admin@luna.local",
  display_name: "Ops Admin",
  tenant_id: "00000000-0000-0000-0000-000000000000",
  org_id: "00000000-0000-0000-0000-000000000000",
  mode: "local_test_user",
  roles: ["platform_admin"],
  auth_context_level: "aal1",
} satisfies SessionSubject;

describe("ops workbench view model", () => {
  it("maps consistency lookup to formal path params", () => {
    expect(
      buildConsistencyPath({
        ref_type: "order",
        ref_id: "order-1",
      }),
    ).toEqual({
      refType: "order",
      refId: "order-1",
    });
  });

  it("requires step-up on high-risk dry-run forms", () => {
    expect(
      consistencyReconcileSchema.safeParse({
        ref_type: "order",
        ref_id: "10000000-0000-4000-8000-000000000001",
        mode: "full",
        reason: "preview repair",
        idempotency_key: "web-015-consistency",
      }).success,
    ).toBe(false);
    expect(
      deadLetterReprocessSchema.safeParse({
        dead_letter_event_id: "10000000-0000-4000-8000-000000000002",
        reason: "preview reprocess",
        idempotency_key: "web-015-dead-letter",
        step_up_token: "10000000-0000-4000-8000-000000000003",
      }).success,
    ).toBe(true);
  });

  it("builds search and recommendation write payloads without UI-only fields", () => {
    const reindex = searchReindexSchema.parse({
      entity_scope: "product",
      mode: "full",
      force: true,
      target_index: "datab-product-v2",
      idempotency_key: "web-015-reindex",
      step_up_token: "10000000-0000-4000-8000-000000000004",
    });
    const alias = aliasSwitchSchema.parse({
      entity_scope: "seller",
      next_index_name: "datab-seller-v2",
      idempotency_key: "web-015-alias",
      step_up_token: "10000000-0000-4000-8000-000000000005",
    });

    expect(buildSearchReindexPayload(reindex)).toEqual({
      entity_scope: "product",
      mode: "full",
      force: true,
      target_index: "datab-product-v2",
    });
    expect(buildAliasSwitchPayload(alias)).toEqual({
      entity_scope: "seller",
      next_index_name: "datab-seller-v2",
    });
    expect(
      buildRecommendationRebuildPayload({
        scope: "all",
        placement_code: "home_featured",
        entity_scope: "all",
        entity_id: "",
        purge_cache: true,
        idempotency_key: "web-015-rebuild",
        step_up_token: "10000000-0000-4000-8000-000000000006",
      }),
    ).toEqual({
      scope: "all",
      placement_code: "home_featured",
        purge_cache: true,
      });
  });

  it("builds notification linkage payloads from formal filters and replay form values", () => {
    const search = notificationAuditSearchSchema.parse({
      order_id: "10000000-0000-4000-8000-000000000010",
      case_id: "",
      aggregate_type: "notification.dispatch_request",
      event_type: "notification.requested",
      target_topic: "dtp.notification.dispatch",
      template_code: "NOTIFY_PAYMENT_SUCCEEDED_V1",
      notification_code: "payment.succeeded",
      event_id: "",
      limit: 20,
      reason: "trace notification delivery",
      step_up_challenge_id: "10000000-0000-4000-8000-000000000011",
    });
    const replay = notificationReplaySchema.parse({
      dead_letter_event_id: "10000000-0000-4000-8000-000000000012",
      dry_run: false,
      reason: "manual replay after review",
      idempotency_key: "web-022-notification-replay",
      step_up_token: "10000000-0000-4000-8000-000000000013",
    });

    expect(buildNotificationAuditSearchPayload(search)).toEqual({
      order_id: "10000000-0000-4000-8000-000000000010",
      aggregate_type: "notification.dispatch_request",
      event_type: "notification.requested",
      target_topic: "dtp.notification.dispatch",
      template_code: "NOTIFY_PAYMENT_SUCCEEDED_V1",
      notification_code: "payment.succeeded",
      limit: 20,
      reason: "trace notification delivery",
    });
    expect(buildNotificationReplayPayload(replay)).toEqual({
      dry_run: false,
      reason: "manual replay after review",
    });
  });

  it("keeps role gates and status tones aligned to V1 ops semantics", () => {
    expect(canReadConsistency(platformAdmin)).toBe(true);
    expect(canReadNotificationOps(platformAdmin)).toBe(true);
    expect(canManageSearchOps(platformAdmin)).toBe(true);
    expect(canManageSearchOps({ ...platformAdmin, roles: ["platform_audit_security"] })).toBe(
      false,
    );
    expect(statusTone("published")).toBe("ok");
    expect(statusTone("dead_lettered")).toBe("danger");
    expect(createOpsIdempotencyKey("search")).toMatch(/^web-015:search:/);
  });

  it("parses object JSON fields used by ranking forms", () => {
    expect(parseJsonObject('{"quality":0.7}')).toEqual({ quality: 0.7 });
    expect(() => parseJsonObject("[1,2]")).toThrow("JSON 必须是对象");
  });

  it("validates recommendation rebuild entity scope before submit", () => {
    expect(
      recommendationRebuildSchema.safeParse({
        scope: "cache",
        placement_code: "home_featured",
        entity_scope: "product",
        entity_id: "",
        purge_cache: true,
        idempotency_key: "web-015-rebuild",
        step_up_token: "10000000-0000-4000-8000-000000000007",
      }).success,
    ).toBe(false);
    expect(
      recommendationRebuildSchema.safeParse({
        scope: "cache",
        placement_code: "home_featured",
        entity_scope: "product",
        entity_id: "10000000-0000-4000-8000-000000000008",
        purge_cache: true,
        idempotency_key: "web-015-rebuild",
        step_up_token: "10000000-0000-4000-8000-000000000009",
      }).success,
    ).toBe(true);
  });

  it("requires step-up and at least one selector for notification linkage search", () => {
    expect(
      notificationAuditSearchSchema.safeParse({
        order_id: "",
        case_id: "",
        aggregate_type: "",
        event_type: "",
        target_topic: "",
        template_code: "",
        notification_code: "",
        event_id: "",
        limit: 20,
        reason: "trace notification delivery",
      }).success,
    ).toBe(false);
    expect(
      notificationReplaySchema.safeParse({
        dead_letter_event_id: "10000000-0000-4000-8000-000000000014",
        dry_run: true,
        reason: "manual replay after review",
        idempotency_key: "web-022-notification-replay",
      }).success,
    ).toBe(false);
  });
});
