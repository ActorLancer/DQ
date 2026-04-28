"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import {
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type SortingState,
} from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import type {
  ConsistencyResponse,
  DeadLettersResponse,
  NotificationAuditSearchResponse,
  ObservabilityOverviewResponse,
  OutboxResponse,
  RecommendationPlacementsResponse,
  RecommendationRankingProfilesResponse,
  SearchRankingProfilesResponse,
  SearchSyncResponse,
} from "@datab/sdk-ts";
import {
  Activity,
  AlertTriangle,
  BellRing,
  Boxes,
  DatabaseZap,
  GitCompareArrows,
  LoaderCircle,
  LockKeyhole,
  RadioTower,
  RefreshCcw,
  Search,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { motion } from "motion/react";
import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ComponentProps,
  type ReactNode,
} from "react";
import { useForm, useWatch } from "react-hook-form";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  aliasSwitchSchema,
  buildAliasSwitchPayload,
  buildCacheInvalidatePayload,
  buildConsistencyPath,
  buildConsistencyReconcilePayload,
  buildDeadLetterReprocessPayload,
  buildDeadLettersQuery,
  buildNotificationAuditSearchPayload,
  buildNotificationReplayPayload,
  buildOutboxQuery,
  buildRecommendationPlacementPatchPayload,
  buildRecommendationRankingPatchPayload,
  buildRecommendationRebuildPayload,
  buildSearchRankingPatchPayload,
  buildSearchReindexPayload,
  buildSearchSyncQuery,
  cacheInvalidateSchema,
  canInvalidateSearchCache,
  canManageRecommendationOps,
  canManageSearchOps,
  canReadConsistency,
  canReadNotificationOps,
  canReadObservability,
  canReadOutbox,
  canReadRecommendationOps,
  canReadSearchOps,
  canReplayNotificationOps,
  canReconcileConsistency,
  canReprocessDeadLetter,
  consistencyLookupSchema,
  consistencyRefTypes,
  consistencyReconcileSchema,
  createOpsIdempotencyKey,
  deadLetterFilterSchema,
  deadLetterReprocessSchema,
  formatOpsError,
  notificationAuditSearchSchema,
  notificationReplaySchema,
  outboxFilterSchema,
  recommendationPlacementPatchSchema,
  recommendationRankingPatchSchema,
  recommendationRebuildSchema,
  searchReindexSchema,
  searchRankingPatchSchema,
  searchSyncFilterSchema,
  statusTone,
  subjectDisplayName,
  type AliasSwitchFormValues,
  type CacheInvalidateFormValues,
  type ConsistencyLookupFormValues,
  type ConsistencyReconcileFormValues,
  type DeadLetterFilterFormValues,
  type DeadLetterReprocessFormValues,
  type NotificationAuditSearchFormValues,
  type NotificationReplayFormValues,
  type OutboxFilterFormValues,
  type RecommendationPlacementPatchFormValues,
  type RecommendationRankingPatchFormValues,
  type RecommendationRebuildFormValues,
  type SearchRankingPatchFormValues,
  type SearchReindexFormValues,
  type SearchSyncFilterFormValues,
  type SessionSubject,
} from "@/lib/ops-workbench";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { cn } from "@/lib/utils";

import { ConsoleRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();

type ConsistencyData = ConsistencyResponse["data"];
type OutboxRow = OutboxResponse["data"]["items"][number];
type DeadLetterRow = DeadLettersResponse["data"]["items"][number];
type NotificationAuditData = NotificationAuditSearchResponse["data"];
type NotificationAuditRow = NotificationAuditData["records"][number];
type SearchSyncRow = SearchSyncResponse["data"][number];
type SearchRankingRow = SearchRankingProfilesResponse["data"][number];
type PlacementRow = RecommendationPlacementsResponse["data"][number];
type RecommendationRankingRow = RecommendationRankingProfilesResponse["data"][number];
type ObservabilityData = ObservabilityOverviewResponse["data"];

export function ConsistencyOpsShell() {
  const [lookup, setLookup] = useState<ConsistencyLookupFormValues | null>(null);
  const [reconcileResult, setReconcileResult] = useState<unknown>(null);
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canRead = canReadConsistency(subject);
  const canReconcile = canReconcileConsistency(subject);

  const lookupForm = useForm<ConsistencyLookupFormValues>({
    resolver: zodResolver(consistencyLookupSchema),
    defaultValues: {
      ref_type: "order",
      ref_id: "",
    },
  });

  const reconcileForm = useForm<ConsistencyReconcileFormValues>({
    resolver: zodResolver(consistencyReconcileSchema),
    defaultValues: {
      ref_type: "order",
      ref_id: "",
      mode: "projection_gap",
      reason: "",
      idempotency_key: createOpsIdempotencyKey("consistency-reconcile"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });

  useEffect(() => {
    if (!lookup) {
      return;
    }
    reconcileForm.reset({
      ...reconcileForm.getValues(),
      ref_type: lookup.ref_type,
      ref_id: lookup.ref_id,
      idempotency_key: createOpsIdempotencyKey("consistency-reconcile"),
    });
  }, [lookup, reconcileForm]);

  const consistencyQuery = useQuery({
    queryKey: ["console", "ops", "consistency", lookup],
    queryFn: () => sdk.ops.getConsistency(buildConsistencyPath(lookup!)),
    enabled: Boolean(canRead && lookup),
  });

  const reconcileMutation = useMutation({
    mutationFn: (values: ConsistencyReconcileFormValues) =>
      sdk.ops.reconcileConsistency(buildConsistencyReconcilePayload(values), {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token,
        stepUpChallengeId: values.step_up_challenge_id,
      }),
    onSuccess: (response) => {
      setReconcileResult(response.data);
      reconcileForm.setValue(
        "idempotency_key",
        createOpsIdempotencyKey("consistency-reconcile"),
      );
    },
  });

  return (
    <ConsoleRouteScaffold routeKey="consistency_trace">
      <OpsHero
        title="双层权威一致性联查"
        description="围绕业务主状态、链证明、外部事实、outbox、dead letter 与审计事件做单对象联查；修复入口只执行 dry-run 预演。"
        icon={<GitCompareArrows className="size-5" />}
        badges={["ops.consistency.read", "ops.consistency.reconcile", "dry-run only"]}
      />

      <div className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <BoundaryCard />
      </div>

      <Card>
        <div className="grid gap-4 lg:grid-cols-[1fr_auto] lg:items-end">
          <div>
            <CardTitle>联查主键</CardTitle>
            <CardDescription className="mt-2">
              正式路径：`GET /api/v1/ops/consistency/{`{refType}`}/{`{refId}`}`。
            </CardDescription>
          </div>
          <Badge className={canRead ? badgeClass("ok") : badgeClass("warn")}>
            {canRead ? "可联查" : "权限态"}
          </Badge>
        </div>
        <form
          className="mt-5 grid gap-4 md:grid-cols-[180px_1fr_auto]"
          onSubmit={lookupForm.handleSubmit((values) => setLookup(values))}
        >
          <SelectField
            label="ref_type"
            value={useWatch({ control: lookupForm.control, name: "ref_type" })}
            onChange={(value) =>
              lookupForm.setValue("ref_type", value as ConsistencyLookupFormValues["ref_type"], {
                shouldDirty: true,
              })
            }
            options={consistencyRefTypes.map((refType) => [refType, refType])}
          />
          <TextInputField
            label="ref_id"
            placeholder="正式业务对象 ID / UUID"
            error={lookupForm.formState.errors.ref_id?.message}
            {...lookupForm.register("ref_id")}
          />
          <Button disabled={!canRead || consistencyQuery.isFetching} type="submit">
            {consistencyQuery.isFetching ? <LoaderCircle className="size-4 animate-spin" /> : <Search className="size-4" />}
            联查
          </Button>
        </form>
      </Card>

      {!canRead ? (
        <PermissionNotice
          title="当前主体缺少一致性联查权限"
          required="ops.consistency.read"
        />
      ) : null}

      {consistencyQuery.isPending && lookup ? <LoadingState label="正在联查一致性状态" /> : null}
      {consistencyQuery.isError ? <ErrorState error={consistencyQuery.error} /> : null}
      {consistencyQuery.isSuccess ? (
        <ConsistencyResult data={consistencyQuery.data.data} />
      ) : lookup ? null : (
        <EmptyState
          title="等待输入联查对象"
          description="输入订单、交付、结算、支付或退款等正式对象 ID 后，页面会通过 platform-core 汇总链下主状态与链上证明状态。"
        />
      )}

      <Card>
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <CardTitle>dry-run 一致性修复预演</CardTitle>
            <CardDescription className="mt-2">
              高风险动作：前端生成 `X-Idempotency-Key`，同时透传 step-up；后端只返回修复建议，不直接篡改业务事实。
            </CardDescription>
          </div>
          <Badge className={canReconcile ? badgeClass("warn") : badgeClass("danger")}>
            {canReconcile ? "需要 step-up" : "无执行权限"}
          </Badge>
        </div>
        <form
          className="mt-5 grid gap-4"
          onSubmit={reconcileForm.handleSubmit((values) => reconcileMutation.mutate(values))}
        >
          <div className="grid gap-4 md:grid-cols-[160px_1fr_160px]">
            <SelectField
              label="ref_type"
              value={useWatch({ control: reconcileForm.control, name: "ref_type" })}
              onChange={(value) =>
                reconcileForm.setValue(
                  "ref_type",
                  value as ConsistencyReconcileFormValues["ref_type"],
                  { shouldDirty: true },
                )
              }
              options={consistencyRefTypes.map((refType) => [refType, refType])}
            />
            <TextInputField
              label="ref_id"
              error={reconcileForm.formState.errors.ref_id?.message}
              {...reconcileForm.register("ref_id")}
            />
            <SelectField
              label="mode"
              value={useWatch({ control: reconcileForm.control, name: "mode" })}
              onChange={(value) =>
                reconcileForm.setValue(
                  "mode",
                  value as ConsistencyReconcileFormValues["mode"],
                  { shouldDirty: true },
                )
              }
              options={[
                ["projection_gap", "projection_gap"],
                ["full", "full"],
              ]}
            />
          </div>
          <TextareaField
            label="reason"
            placeholder="说明为什么需要执行 dry-run 修复预演"
            error={reconcileForm.formState.errors.reason?.message}
            {...reconcileForm.register("reason")}
          />
          <div className="grid gap-4 lg:grid-cols-3">
            <TextInputField
              label="X-Idempotency-Key"
              error={reconcileForm.formState.errors.idempotency_key?.message}
              {...reconcileForm.register("idempotency_key")}
            />
            <TextInputField
              label="X-Step-Up-Token"
              error={reconcileForm.formState.errors.step_up_token?.message}
              {...reconcileForm.register("step_up_token")}
            />
            <TextInputField
              label="X-Step-Up-Challenge-Id"
              {...reconcileForm.register("step_up_challenge_id")}
            />
          </div>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <AuditHint />
            <Button disabled={!canReconcile || reconcileMutation.isPending} type="submit" variant="warning">
              {reconcileMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <LockKeyhole className="size-4" />}
              提交 dry-run
            </Button>
          </div>
        </form>
        {reconcileMutation.isError ? <ErrorState error={reconcileMutation.error} compact /> : null}
        {reconcileResult ? <ResultBlock title="修复预演结果" value={reconcileResult} /> : null}
      </Card>
    </ConsoleRouteScaffold>
  );
}

export function OutboxDeadLetterShell() {
  const [outboxFilters, setOutboxFilters] = useState<OutboxFilterFormValues>({
    page_size: 50,
  });
  const [deadLetterFilters, setDeadLetterFilters] = useState<DeadLetterFilterFormValues>({
    reprocess_status: "not_reprocessed",
    page_size: 50,
  });
  const [reprocessResult, setReprocessResult] = useState<unknown>(null);
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canRead = canReadOutbox(subject);
  const canReprocess = canReprocessDeadLetter(subject);

  const outboxForm = useForm<OutboxFilterFormValues>({
    resolver: zodResolver(outboxFilterSchema),
    defaultValues: outboxFilters,
  });
  const deadLetterForm = useForm<DeadLetterFilterFormValues>({
    resolver: zodResolver(deadLetterFilterSchema),
    defaultValues: deadLetterFilters,
  });
  const reprocessForm = useForm<DeadLetterReprocessFormValues>({
    resolver: zodResolver(deadLetterReprocessSchema),
    defaultValues: {
      dead_letter_event_id: "",
      reason: "",
      idempotency_key: createOpsIdempotencyKey("dead-letter-reprocess"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });

  const outboxQuery = useQuery({
    queryKey: ["console", "ops", "outbox", outboxFilters],
    queryFn: () => sdk.ops.listOutbox(buildOutboxQuery(outboxFilters)),
    enabled: canRead,
  });
  const deadLettersQuery = useQuery({
    queryKey: ["console", "ops", "dead-letters", deadLetterFilters],
    queryFn: () => sdk.ops.listDeadLetters(buildDeadLettersQuery(deadLetterFilters)),
    enabled: canRead,
  });

  const reprocessMutation = useMutation({
    mutationFn: (values: DeadLetterReprocessFormValues) =>
      sdk.ops.reprocessDeadLetter(
        { id: values.dead_letter_event_id },
        buildDeadLetterReprocessPayload(values),
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: values.step_up_token,
          stepUpChallengeId: values.step_up_challenge_id,
        },
      ),
    onSuccess: (response) => {
      setReprocessResult(response.data);
      reprocessForm.setValue(
        "idempotency_key",
        createOpsIdempotencyKey("dead-letter-reprocess"),
      );
    },
  });

  const outboxColumns = useMemo<ColumnDef<OutboxRow>[]>(
    () => [
      { header: "event", accessorKey: "event_type" },
      { header: "status", accessorKey: "status" },
      { header: "topic", accessorKey: "target_topic" },
      { header: "retry", accessorFn: (row) => `${row.retry_count}/${row.max_retries}` },
      { header: "request_id", accessorKey: "request_id" },
      { header: "error", accessorKey: "last_error_code" },
    ],
    [],
  );
  const deadLetterColumns = useMemo<ColumnDef<DeadLetterRow>[]>(
    () => [
      {
        header: "dead_letter_event_id",
        accessorKey: "dead_letter_event_id",
        cell: ({ row }) => (
          <button
            className="text-left font-mono text-xs text-[var(--accent-strong)] underline-offset-4 hover:underline"
            type="button"
            onClick={() =>
              reprocessForm.setValue(
                "dead_letter_event_id",
                row.original.dead_letter_event_id ?? "",
                { shouldDirty: true },
              )
            }
          >
            {row.original.dead_letter_event_id}
          </button>
        ),
      },
      { header: "event", accessorKey: "event_type" },
      { header: "status", accessorKey: "reprocess_status" },
      { header: "stage", accessorKey: "failure_stage" },
      { header: "topic", accessorKey: "target_topic" },
      { header: "request_id", accessorKey: "request_id" },
    ],
    [reprocessForm],
  );

  return (
    <ConsoleRouteScaffold routeKey="outbox_dead_letter">
      <OpsHero
        title="Outbox / Dead Letter 控制台"
        description="从 PostgreSQL 正式读模型查看 canonical outbox、发布尝试、consumer 幂等记录和 SEARCHREC dead letter 隔离结果。"
        icon={<Boxes className="size-5" />}
        badges={["ops.outbox.read", "ops.dead_letter.read", "ops.dead_letter.reprocess"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <ObservabilityOverviewPanel subject={subject} />
      </div>

      {!canRead ? (
        <PermissionNotice
          title="当前主体缺少 outbox / dead letter 查看权限"
          required="ops.outbox.read / ops.dead_letter.read"
        />
      ) : null}

      <div className="grid gap-4 xl:grid-cols-2">
        <Card className="min-w-0">
          <CardTitle>Outbox 筛选</CardTitle>
          <form
            className="mt-4 grid gap-3 md:grid-cols-2"
            onSubmit={outboxForm.handleSubmit((values) => setOutboxFilters(values))}
          >
            <TextInputField label="outbox_status" {...outboxForm.register("outbox_status")} />
            <TextInputField label="event_type" {...outboxForm.register("event_type")} />
            <TextInputField label="target_topic" {...outboxForm.register("target_topic")} />
            <TextInputField label="request_id" {...outboxForm.register("request_id")} />
            <TextInputField label="trace_id" {...outboxForm.register("trace_id")} />
            <TextInputField
              label="page_size"
              type="number"
              {...outboxForm.register("page_size", { valueAsNumber: true })}
            />
            <Button className="md:col-span-2" disabled={!canRead || outboxQuery.isFetching} type="submit">
              <RefreshCcw className="size-4" />
              刷新 outbox
            </Button>
          </form>
          <QueryPanel query={outboxQuery} title="Outbox 列表">
            {(response) => (
              <VirtualTable
                columns={outboxColumns}
                data={response.data.items}
                emptyLabel="当前筛选条件下没有 outbox 事件"
              />
            )}
          </QueryPanel>
        </Card>

        <Card className="min-w-0">
          <CardTitle>Dead Letter 筛选</CardTitle>
          <form
            className="mt-4 grid gap-3 md:grid-cols-2"
            onSubmit={deadLetterForm.handleSubmit((values) => setDeadLetterFilters(values))}
          >
            <TextInputField label="reprocess_status" {...deadLetterForm.register("reprocess_status")} />
            <TextInputField label="failure_stage" {...deadLetterForm.register("failure_stage")} />
            <TextInputField label="request_id" {...deadLetterForm.register("request_id")} />
            <TextInputField label="trace_id" {...deadLetterForm.register("trace_id")} />
            <TextInputField
              label="page_size"
              type="number"
              {...deadLetterForm.register("page_size", { valueAsNumber: true })}
            />
            <Button className="md:col-span-2" disabled={!canRead || deadLettersQuery.isFetching} type="submit">
              <RefreshCcw className="size-4" />
              刷新 dead letter
            </Button>
          </form>
          <QueryPanel query={deadLettersQuery} title="Dead Letter 列表">
            {(response) => (
              <VirtualTable
                columns={deadLetterColumns}
                data={response.data.items}
                emptyLabel="当前筛选条件下没有 dead letter"
              />
            )}
          </QueryPanel>
        </Card>
      </div>

      <Card>
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <CardTitle>Dead Letter dry-run 重处理</CardTitle>
            <CardDescription className="mt-2">
              仅支持 SEARCHREC consumer failure 的 dry-run 预演。点击列表中的 `dead_letter_event_id` 可回填目标。
            </CardDescription>
          </div>
          <Badge className={canReprocess ? badgeClass("warn") : badgeClass("danger")}>
            {canReprocess ? "需要 step-up" : "无执行权限"}
          </Badge>
        </div>
        <form
          className="mt-5 grid gap-4"
          onSubmit={reprocessForm.handleSubmit((values) => reprocessMutation.mutate(values))}
        >
          <div className="grid gap-4 lg:grid-cols-[1fr_1fr]">
            <TextInputField
              label="dead_letter_event_id"
              error={reprocessForm.formState.errors.dead_letter_event_id?.message}
              {...reprocessForm.register("dead_letter_event_id")}
            />
            <TextInputField
              label="X-Idempotency-Key"
              error={reprocessForm.formState.errors.idempotency_key?.message}
              {...reprocessForm.register("idempotency_key")}
            />
          </div>
          <TextareaField
            label="reason"
            error={reprocessForm.formState.errors.reason?.message}
            {...reprocessForm.register("reason")}
          />
          <div className="grid gap-4 lg:grid-cols-2">
            <TextInputField
              label="X-Step-Up-Token"
              error={reprocessForm.formState.errors.step_up_token?.message}
              {...reprocessForm.register("step_up_token")}
            />
            <TextInputField
              label="X-Step-Up-Challenge-Id"
              {...reprocessForm.register("step_up_challenge_id")}
            />
          </div>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <AuditHint />
            <Button disabled={!canReprocess || reprocessMutation.isPending} type="submit" variant="warning">
              {reprocessMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <LockKeyhole className="size-4" />}
              dry-run 重处理
            </Button>
          </div>
        </form>
        {reprocessMutation.isError ? <ErrorState error={reprocessMutation.error} compact /> : null}
        {reprocessResult ? <ResultBlock title="重处理预演结果" value={reprocessResult} /> : null}
      </Card>
    </ConsoleRouteScaffold>
  );
}

export function NotificationOpsShell() {
  const queryClient = useQueryClient();
  const [submittedSearch, setSubmittedSearch] =
    useState<NotificationAuditSearchFormValues | null>(null);
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);
  const [replayResult, setReplayResult] = useState<unknown>(null);
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canRead = canReadNotificationOps(subject);
  const canReplay = canReplayNotificationOps(subject);

  const searchForm = useForm<NotificationAuditSearchFormValues>({
    resolver: zodResolver(notificationAuditSearchSchema),
    defaultValues: {
      order_id: "",
      case_id: "",
      aggregate_type: "notification.dispatch_request",
      event_type: "notification.requested",
      target_topic: "dtp.notification.dispatch",
      template_code: "",
      notification_code: "",
      event_id: "",
      limit: 20,
      reason: "",
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const replayForm = useForm<NotificationReplayFormValues>({
    resolver: zodResolver(notificationReplaySchema),
    defaultValues: {
      dead_letter_event_id: "",
      dry_run: true,
      reason: "",
      idempotency_key: createOpsIdempotencyKey("notification-replay"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });

  const notificationQuery = useQuery({
    queryKey: ["console", "ops", "notifications", submittedSearch],
    queryFn: () =>
      sdk.ops.searchNotificationAudit(
        buildNotificationAuditSearchPayload(submittedSearch!),
        {
          stepUpToken: submittedSearch!.step_up_token,
          stepUpChallengeId: submittedSearch!.step_up_challenge_id,
        },
      ),
    enabled: Boolean(canRead && submittedSearch),
  });

  const replayMutation = useMutation({
    mutationFn: (values: NotificationReplayFormValues) =>
      sdk.ops.replayNotificationDeadLetter(
        { dead_letter_event_id: values.dead_letter_event_id },
        buildNotificationReplayPayload(values),
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: values.step_up_token,
          stepUpChallengeId: values.step_up_challenge_id,
        },
      ),
    onSuccess: (response) => {
      setReplayResult(response.data);
      replayForm.setValue(
        "idempotency_key",
        createOpsIdempotencyKey("notification-replay"),
      );
      void queryClient.invalidateQueries({
        queryKey: ["console", "ops", "notifications"],
      });
    },
  });

  const notificationRows = useMemo(
    () => notificationQuery.data?.data.records ?? [],
    [notificationQuery.data],
  );
  const selectedRecord = useMemo(() => {
    if (!notificationRows.length) {
      return null;
    }
    return (
      notificationRows.find((row) => row.event_id === selectedEventId) ??
      notificationRows[0]
    );
  }, [notificationRows, selectedEventId]);
  const notificationSummary = useMemo(() => {
    const templates = new Set(notificationRows.map((row) => row.template_code));
    const withDeadLetter = notificationRows.filter((row) => row.dead_letter);
    const retrying = notificationRows.filter(
      (row) => row.retry_timeline.length > 1 || (row.current_attempt ?? 1) > 1,
    );
    return {
      total: notificationQuery.data?.data.total ?? 0,
      templateCount: templates.size,
      deadLetterCount: withDeadLetter.length,
      retryCount: retrying.length,
    };
  }, [notificationQuery.data, notificationRows]);

  const notificationColumns = useMemo<ColumnDef<NotificationAuditRow>[]>(
    () => [
      {
        header: "event_id",
        accessorKey: "event_id",
        cell: ({ row }) => (
          <button
            className="text-left font-mono text-xs text-[var(--accent-strong)] underline-offset-4 hover:underline"
            type="button"
            onClick={() => setSelectedEventId(row.original.event_id)}
          >
            {row.original.event_id}
          </button>
        ),
      },
      { header: "notification_code", accessorKey: "notification_code" },
      { header: "template_code", accessorKey: "template_code" },
      {
        header: "status",
        accessorKey: "current_status",
        cell: ({ row }) => (
          <Badge className={badgeClass(statusTone(row.original.current_status))}>
            {row.original.current_status}
          </Badge>
        ),
      },
      { header: "attempt", accessorKey: "current_attempt" },
      { header: "channel", accessorKey: "channel" },
      {
        header: "dead_letter",
        accessorFn: (row) => row.dead_letter?.dead_letter_event_id ?? "none",
        cell: ({ row }) =>
          row.original.dead_letter ? (
            <button
              className="text-left font-mono text-xs text-[var(--accent-strong)] underline-offset-4 hover:underline"
              type="button"
              onClick={() => {
                replayForm.setValue(
                  "dead_letter_event_id",
                  row.original.dead_letter?.dead_letter_event_id ?? "",
                  { shouldDirty: true },
                );
                setSelectedEventId(row.original.event_id);
              }}
            >
              {row.original.dead_letter.dead_letter_event_id}
            </button>
          ) : (
            <span className="text-[var(--ink-subtle)]">none</span>
          ),
      },
      { header: "request_id", accessorKey: "request_id" },
    ],
    [replayForm],
  );

  const replayMode = useWatch({
    control: replayForm.control,
    name: "dry_run",
  });

  return (
    <ConsoleRouteScaffold routeKey="notification_ops">
      <OpsHero
        title="通知联查与补发"
        description="通过 platform-core facade 联查通知发送、失败、重试和模板渲染结果；dead letter replay 仍由后端受控转发到 notification-worker。"
        icon={<BellRing className="size-5" />}
        badges={["ops.outbox.read", "ops.dead_letter.read", "ops.dead_letter.reprocess"]}
      />

      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <ObservabilityOverviewPanel subject={subject} />
      </div>

      {!canRead ? (
        <PermissionNotice
          title="当前主体缺少通知联查权限"
          required="ops.outbox.read / ops.dead_letter.read"
        />
      ) : null}

      <Card>
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <CardTitle>通知联查条件</CardTitle>
            <CardDescription className="mt-2">
              正式入口：`POST /api/v1/ops/notifications/audit/search`。至少填写一个正式筛选条件，并显式附带 step-up 上下文。
            </CardDescription>
          </div>
          <Badge className={canRead ? badgeClass("warn") : badgeClass("danger")}>
            {canRead ? "需要 step-up" : "权限态"}
          </Badge>
        </div>
        <form
          className="mt-5 grid gap-4"
          onSubmit={searchForm.handleSubmit((values) => {
            setSubmittedSearch(values);
            setSelectedEventId(null);
          })}
        >
          <div className="grid gap-4 lg:grid-cols-3">
            <TextInputField
              label="order_id"
              error={searchForm.formState.errors.order_id?.message}
              {...searchForm.register("order_id")}
            />
            <TextInputField
              label="case_id"
              error={searchForm.formState.errors.case_id?.message}
              {...searchForm.register("case_id")}
            />
            <TextInputField
              label="event_id"
              error={searchForm.formState.errors.event_id?.message}
              {...searchForm.register("event_id")}
            />
            <TextInputField label="notification_code" {...searchForm.register("notification_code")} />
            <TextInputField label="template_code" {...searchForm.register("template_code")} />
            <TextInputField label="target_topic" {...searchForm.register("target_topic")} />
            <TextInputField label="aggregate_type" {...searchForm.register("aggregate_type")} />
            <TextInputField label="event_type" {...searchForm.register("event_type")} />
            <TextInputField
              label="limit"
              type="number"
              {...searchForm.register("limit", { valueAsNumber: true })}
            />
          </div>
          <TextareaField
            label="reason"
            placeholder="说明为何需要联查通知发送与补发轨迹"
            error={searchForm.formState.errors.reason?.message}
            {...searchForm.register("reason")}
          />
          <div className="grid gap-4 lg:grid-cols-2">
            <TextInputField
              label="X-Step-Up-Token"
              error={searchForm.formState.errors.step_up_token?.message}
              {...searchForm.register("step_up_token")}
            />
            <TextInputField
              label="X-Step-Up-Challenge-Id"
              {...searchForm.register("step_up_challenge_id")}
            />
          </div>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <AuditHint />
            <Button disabled={!canRead || notificationQuery.isFetching} type="submit">
              {notificationQuery.isFetching ? (
                <LoaderCircle className="size-4 animate-spin" />
              ) : (
                <Search className="size-4" />
              )}
              联查通知轨迹
            </Button>
          </div>
        </form>
      </Card>

      <QueryPanel query={notificationQuery} title="通知联查结果">
        {(response) => (
          <div className="grid gap-4">
            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
              <StatCard
                label="records"
                value={notificationSummary.total}
                hint={`request_id: ${response.data.request_id}`}
              />
              <StatCard
                label="templates"
                value={notificationSummary.templateCount}
                hint={`trace_id: ${response.data.trace_id}`}
              />
              <StatCard
                label="dead letters"
                value={notificationSummary.deadLetterCount}
                hint="可回填 replay 目标"
              />
              <StatCard
                label="retried"
                value={notificationSummary.retryCount}
                hint="重试或多 attempt 记录"
              />
            </div>
            <VirtualTable
              columns={notificationColumns}
              data={response.data.records}
              emptyLabel="当前筛选条件下没有通知发送记录"
            />
            {selectedRecord ? (
              <div className="grid gap-4 xl:grid-cols-2">
                <ResultBlock title="当前通知记录" value={selectedRecord} />
                <ResultBlock
                  title="当前记录时间线"
                  value={{
                    retry_timeline: selectedRecord.retry_timeline,
                    audit_timeline: selectedRecord.audit_timeline,
                    dead_letter: selectedRecord.dead_letter,
                  }}
                />
              </div>
            ) : null}
          </div>
        )}
      </QueryPanel>

      <Card>
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <CardTitle>通知 dead letter replay</CardTitle>
            <CardDescription className="mt-2">
              正式入口：`POST /api/v1/ops/notifications/dead-letters/{`{dead_letter_event_id}`}/replay`。浏览器只打 platform-core，由后端再转发到 worker 内部控制面。
            </CardDescription>
          </div>
          <Badge className={canReplay ? badgeClass("warn") : badgeClass("danger")}>
            {canReplay ? "高风险动作" : "无执行权限"}
          </Badge>
        </div>
        <form
          className="mt-5 grid gap-4"
          onSubmit={replayForm.handleSubmit((values) => replayMutation.mutate(values))}
        >
          <div className="grid gap-4 lg:grid-cols-[1fr_180px]">
            <TextInputField
              label="dead_letter_event_id"
              error={replayForm.formState.errors.dead_letter_event_id?.message}
              {...replayForm.register("dead_letter_event_id")}
            />
            <SelectField
              label="mode"
              value={replayMode ? "true" : "false"}
              onChange={(value) =>
                replayForm.setValue("dry_run", value === "true", { shouldDirty: true })
              }
              options={[
                ["true", "dry_run"],
                ["false", "replay"],
              ]}
            />
          </div>
          <TextareaField
            label="reason"
            placeholder="说明为何需要 dry-run 或正式 replay"
            error={replayForm.formState.errors.reason?.message}
            {...replayForm.register("reason")}
          />
          <WriteHeaders form={replayForm} />
          <SubmitWriteButton
            disabled={!canReplay}
            pending={replayMutation.isPending}
            label={replayMode ? "提交 dry-run replay" : "提交正式 replay"}
          />
        </form>
        {replayMutation.isError ? <ErrorState error={replayMutation.error} compact /> : null}
        {replayResult ? <ResultBlock title="通知 replay 结果" value={replayResult} /> : null}
      </Card>
    </ConsoleRouteScaffold>
  );
}

export function SearchOpsShell() {
  const queryClient = useQueryClient();
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canReadSearch = canReadSearchOps(subject);
  const canManageSearch = canManageSearchOps(subject);
  const canInvalidateCache = canInvalidateSearchCache(subject);
  const canReadRecommendation = canReadRecommendationOps(subject);
  const canManageRecommendation = canManageRecommendationOps(subject);
  const [syncFilters, setSyncFilters] = useState<SearchSyncFilterFormValues>({
    entity_scope: "all",
    limit: 30,
  });
  const [lastAction, setLastAction] = useState<{ title: string; value: unknown } | null>(null);

  const syncForm = useForm<SearchSyncFilterFormValues>({
    resolver: zodResolver(searchSyncFilterSchema),
    defaultValues: syncFilters,
  });
  const reindexForm = useForm<SearchReindexFormValues>({
    resolver: zodResolver(searchReindexSchema),
    defaultValues: {
      entity_scope: "product",
      entity_id: "",
      mode: "batch",
      force: false,
      target_index: "",
      idempotency_key: createOpsIdempotencyKey("search-reindex"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const aliasForm = useForm<AliasSwitchFormValues>({
    resolver: zodResolver(aliasSwitchSchema),
    defaultValues: {
      entity_scope: "product",
      next_index_name: "",
      idempotency_key: createOpsIdempotencyKey("search-alias"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const cacheForm = useForm<CacheInvalidateFormValues>({
    resolver: zodResolver(cacheInvalidateSchema),
    defaultValues: {
      entity_scope: "all",
      query_hash: "",
      purge_all: false,
      idempotency_key: createOpsIdempotencyKey("search-cache"),
    },
  });
  const rankingForm = useForm<SearchRankingPatchFormValues>({
    resolver: zodResolver(searchRankingPatchSchema),
    defaultValues: {
      ranking_profile_id: "",
      status: "",
      weights_json: "",
      filter_policy_json: "",
      idempotency_key: createOpsIdempotencyKey("search-ranking"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const rebuildForm = useForm<RecommendationRebuildFormValues>({
    resolver: zodResolver(recommendationRebuildSchema),
    defaultValues: {
      scope: "all",
      placement_code: "home_featured",
      entity_scope: "all",
      entity_id: "",
      purge_cache: true,
      idempotency_key: createOpsIdempotencyKey("recommendation-rebuild"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const placementForm = useForm<RecommendationPlacementPatchFormValues>({
    resolver: zodResolver(recommendationPlacementPatchSchema),
    defaultValues: {
      placement_code: "home_featured",
      status: "",
      default_ranking_profile_key: "",
      idempotency_key: createOpsIdempotencyKey("recommendation-placement"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const recommendationRankingForm = useForm<RecommendationRankingPatchFormValues>({
    resolver: zodResolver(recommendationRankingPatchSchema),
    defaultValues: {
      ranking_profile_id: "",
      status: "",
      explain_codes: "",
      idempotency_key: createOpsIdempotencyKey("recommendation-ranking"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });

  const syncQuery = useQuery({
    queryKey: ["console", "ops", "search-sync", syncFilters],
    queryFn: () => sdk.search.listSearchSync(buildSearchSyncQuery(syncFilters)),
    enabled: canReadSearch,
  });
  const searchRankingQuery = useQuery({
    queryKey: ["console", "ops", "search-ranking-profiles"],
    queryFn: () => sdk.search.listRankingProfiles(),
    enabled: canReadSearch,
  });
  const placementsQuery = useQuery({
    queryKey: ["console", "ops", "recommendation-placements"],
    queryFn: () => sdk.recommendation.listPlacements(),
    enabled: canReadRecommendation,
  });
  const recommendationRankingQuery = useQuery({
    queryKey: ["console", "ops", "recommendation-ranking-profiles"],
    queryFn: () => sdk.recommendation.listRankingProfiles(),
    enabled: canReadRecommendation,
  });

  const onWriteSuccess = (title: string, value: unknown) => {
    setLastAction({ title, value });
    void queryClient.invalidateQueries({ queryKey: ["console", "ops"] });
  };

  const reindexMutation = useMutation({
    mutationFn: (values: SearchReindexFormValues) =>
      sdk.search.reindex(buildSearchReindexPayload(values), {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token,
        stepUpChallengeId: values.step_up_challenge_id,
      }),
    onSuccess: (response) => {
      onWriteSuccess("搜索 Reindex 已入队", response.data);
      reindexForm.setValue("idempotency_key", createOpsIdempotencyKey("search-reindex"));
    },
  });
  const aliasMutation = useMutation({
    mutationFn: (values: AliasSwitchFormValues) =>
      sdk.search.switchAlias(buildAliasSwitchPayload(values), {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token,
        stepUpChallengeId: values.step_up_challenge_id,
      }),
    onSuccess: (response) => {
      onWriteSuccess("搜索 Alias 已切换", response.data);
      aliasForm.setValue("idempotency_key", createOpsIdempotencyKey("search-alias"));
    },
  });
  const cacheMutation = useMutation({
    mutationFn: (values: CacheInvalidateFormValues) =>
      sdk.search.invalidateCache(buildCacheInvalidatePayload(values), {
        idempotencyKey: values.idempotency_key,
      }),
    onSuccess: (response) => {
      onWriteSuccess("搜索缓存已失效", response.data);
      cacheForm.setValue("idempotency_key", createOpsIdempotencyKey("search-cache"));
    },
  });
  const rankingMutation = useMutation({
    mutationFn: (values: SearchRankingPatchFormValues) =>
      sdk.search.patchRankingProfile(
        { id: values.ranking_profile_id },
        buildSearchRankingPatchPayload(values),
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: values.step_up_token,
          stepUpChallengeId: values.step_up_challenge_id,
        },
      ),
    onSuccess: (response) => {
      onWriteSuccess("搜索排序配置已更新", response.data);
      rankingForm.setValue("idempotency_key", createOpsIdempotencyKey("search-ranking"));
    },
  });
  const rebuildMutation = useMutation({
    mutationFn: (values: RecommendationRebuildFormValues) =>
      sdk.recommendation.rebuild(buildRecommendationRebuildPayload(values), {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token,
        stepUpChallengeId: values.step_up_challenge_id,
      }),
    onSuccess: (response) => {
      onWriteSuccess("推荐重建已执行", response.data);
      rebuildForm.setValue("idempotency_key", createOpsIdempotencyKey("recommendation-rebuild"));
    },
  });
  const placementMutation = useMutation({
    mutationFn: (values: RecommendationPlacementPatchFormValues) =>
      sdk.recommendation.patchPlacement(
        { placement_code: values.placement_code },
        buildRecommendationPlacementPatchPayload(values),
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: values.step_up_token,
          stepUpChallengeId: values.step_up_challenge_id,
        },
      ),
    onSuccess: (response) => {
      onWriteSuccess("推荐位已更新", response.data);
      placementForm.setValue("idempotency_key", createOpsIdempotencyKey("recommendation-placement"));
    },
  });
  const recommendationRankingMutation = useMutation({
    mutationFn: (values: RecommendationRankingPatchFormValues) =>
      sdk.recommendation.patchRankingProfile(
        { id: values.ranking_profile_id },
        buildRecommendationRankingPatchPayload(values),
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: values.step_up_token,
          stepUpChallengeId: values.step_up_challenge_id,
        },
      ),
    onSuccess: (response) => {
      onWriteSuccess("推荐排序配置已更新", response.data);
      recommendationRankingForm.setValue(
        "idempotency_key",
        createOpsIdempotencyKey("recommendation-ranking"),
      );
    },
  });

  const syncColumns = useMemo<ColumnDef<SearchSyncRow>[]>(
    () => [
      { header: "entity", accessorFn: (row) => `${row.entity_scope}/${row.entity_id}` },
      { header: "sync_status", accessorKey: "sync_status" },
      { header: "doc_version", accessorKey: "document_version" },
      { header: "reconcile", accessorKey: "reconcile_status" },
      { header: "target_index", accessorKey: "target_index" },
      { header: "exception", accessorKey: "latest_exception_error_code" },
    ],
    [],
  );
  const searchRankingColumns = useMemo<ColumnDef<SearchRankingRow>[]>(
    () => [
      { header: "profile_key", accessorKey: "profile_key" },
      { header: "scope", accessorKey: "entity_scope" },
      { header: "status", accessorKey: "status" },
      { header: "active index", accessorKey: "backend_type" },
      { header: "updated_at", accessorKey: "updated_at" },
      {
        header: "select",
        id: "select",
        cell: ({ row }) => (
          <button
            className="text-[var(--accent-strong)] underline-offset-4 hover:underline"
            type="button"
            onClick={() => rankingForm.setValue("ranking_profile_id", row.original.ranking_profile_id)}
          >
            填入
          </button>
        ),
      },
    ],
    [rankingForm],
  );
  const placementColumns = useMemo<ColumnDef<PlacementRow>[]>(
    () => [
      { header: "placement_code", accessorKey: "placement_code" },
      { header: "scope", accessorKey: "placement_scope" },
      { header: "status", accessorKey: "status" },
      { header: "default profile", accessorKey: "default_ranking_profile_key" },
      {
        header: "select",
        id: "select",
        cell: ({ row }) => (
          <button
            className="text-[var(--accent-strong)] underline-offset-4 hover:underline"
            type="button"
            onClick={() => placementForm.setValue("placement_code", row.original.placement_code)}
          >
            填入
          </button>
        ),
      },
    ],
    [placementForm],
  );
  const recommendationRankingColumns = useMemo<ColumnDef<RecommendationRankingRow>[]>(
    () => [
      { header: "profile_key", accessorKey: "profile_key" },
      { header: "scope", accessorKey: "placement_scope" },
      { header: "status", accessorKey: "status" },
      { header: "stage", accessorKey: "stage_from" },
      {
        header: "select",
        id: "select",
        cell: ({ row }) => (
          <button
            className="text-[var(--accent-strong)] underline-offset-4 hover:underline"
            type="button"
            onClick={() =>
              recommendationRankingForm.setValue(
                "ranking_profile_id",
                row.original.recommendation_ranking_profile_id,
              )
            }
          >
            填入
          </button>
        ),
      },
    ],
    [recommendationRankingForm],
  );

  return (
    <ConsoleRouteScaffold routeKey="search_ops">
      <OpsHero
        title="搜索同步与推荐重建运维"
        description="把 OpenSearch / Redis / Kafka 的运行状态收敛到 platform-core 正式 ops API，页面不直连任何受限中间件。"
        icon={<DatabaseZap className="size-5" />}
        badges={["ops.search_sync.read", "ops.search_reindex.execute", "ops.recommend_rebuild.execute"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <BoundaryCard />
      </div>

      {!canReadSearch ? (
        <PermissionNotice
          title="当前主体缺少搜索运维读取权限"
          required="ops.search_sync.read / ops.search_ranking.read"
        />
      ) : null}

      <Card className="min-w-0">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <CardTitle>搜索同步任务</CardTitle>
            <CardDescription className="mt-2">
              正式读取 `search.index_sync_task`、文档版本、异常摘要、投影状态和 active index。
            </CardDescription>
          </div>
          <Badge className={canReadSearch ? badgeClass("ok") : badgeClass("warn")}>
            GET /api/v1/ops/search/sync
          </Badge>
        </div>
        <form
          className="mt-5 grid gap-4 md:grid-cols-[180px_1fr_140px_auto]"
          onSubmit={syncForm.handleSubmit((values) => setSyncFilters(values))}
        >
          <SelectField
            label="entity_scope"
            value={useWatch({ control: syncForm.control, name: "entity_scope" })}
            onChange={(value) =>
              syncForm.setValue("entity_scope", value as SearchSyncFilterFormValues["entity_scope"], {
                shouldDirty: true,
              })
            }
            options={[
              ["all", "all"],
              ["product", "product"],
              ["seller", "seller"],
            ]}
          />
          <TextInputField label="sync_status" {...syncForm.register("sync_status")} />
          <TextInputField
            label="limit"
            type="number"
            {...syncForm.register("limit", { valueAsNumber: true })}
          />
          <Button disabled={!canReadSearch || syncQuery.isFetching} type="submit">
            <RefreshCcw className="size-4" />
            刷新
          </Button>
        </form>
        <QueryPanel query={syncQuery} title="同步状态列表">
          {(response) => (
            <VirtualTable
              columns={syncColumns}
              data={response.data}
              emptyLabel="当前筛选条件下没有同步任务"
            />
          )}
        </QueryPanel>
      </Card>

      <div className="grid gap-4 xl:grid-cols-2">
        <SearchWriteCard
          title="发起 Reindex"
          description="写入 search.index_sync_task，要求 X-Idempotency-Key 与 step-up。"
          allowed={canManageSearch}
          mutation={reindexMutation}
        >
          <form className="grid gap-3" onSubmit={reindexForm.handleSubmit((values) => reindexMutation.mutate(values))}>
            <div className="grid gap-3 md:grid-cols-3">
              <SelectField
                label="entity_scope"
                value={useWatch({ control: reindexForm.control, name: "entity_scope" })}
                onChange={(value) =>
                  reindexForm.setValue("entity_scope", value as SearchReindexFormValues["entity_scope"], {
                    shouldDirty: true,
                  })
                }
                options={[
                  ["product", "product"],
                  ["seller", "seller"],
                  ["all", "all"],
                ]}
              />
              <SelectField
                label="mode"
                value={useWatch({ control: reindexForm.control, name: "mode" })}
                onChange={(value) =>
                  reindexForm.setValue("mode", value as SearchReindexFormValues["mode"], {
                    shouldDirty: true,
                  })
                }
                options={[
                  ["single", "single"],
                  ["batch", "batch"],
                  ["full", "full"],
                ]}
              />
              <label className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm">
                <input
                  className="mr-2"
                  type="checkbox"
                  {...reindexForm.register("force")}
                />
                force
              </label>
            </div>
            <TextInputField label="entity_id" error={reindexForm.formState.errors.entity_id?.message} {...reindexForm.register("entity_id")} />
            <TextInputField label="target_index" {...reindexForm.register("target_index")} />
            <WriteHeaders form={reindexForm} />
            <SubmitWriteButton disabled={!canManageSearch} pending={reindexMutation.isPending} label="提交 Reindex" />
          </form>
        </SearchWriteCard>

        <SearchWriteCard
          title="切换 Alias"
          description="切换 read/write alias 到新物理索引，要求 step-up。"
          allowed={canManageSearch}
          mutation={aliasMutation}
        >
          <form className="grid gap-3" onSubmit={aliasForm.handleSubmit((values) => aliasMutation.mutate(values))}>
            <SelectField
              label="entity_scope"
              value={useWatch({ control: aliasForm.control, name: "entity_scope" })}
              onChange={(value) =>
                aliasForm.setValue("entity_scope", value as AliasSwitchFormValues["entity_scope"], {
                  shouldDirty: true,
                })
              }
              options={[
                ["product", "product"],
                ["seller", "seller"],
              ]}
            />
            <TextInputField label="next_index_name" error={aliasForm.formState.errors.next_index_name?.message} {...aliasForm.register("next_index_name")} />
            <WriteHeaders form={aliasForm} />
            <SubmitWriteButton disabled={!canManageSearch} pending={aliasMutation.isPending} label="切换 Alias" />
          </form>
        </SearchWriteCard>

        <SearchWriteCard
          title="失效搜索缓存"
          description="仅要求 X-Idempotency-Key，不伪造 step-up。"
          allowed={canInvalidateCache}
          mutation={cacheMutation}
        >
          <form className="grid gap-3" onSubmit={cacheForm.handleSubmit((values) => cacheMutation.mutate(values))}>
            <SelectField
              label="entity_scope"
              value={useWatch({ control: cacheForm.control, name: "entity_scope" })}
              onChange={(value) =>
                cacheForm.setValue("entity_scope", value as CacheInvalidateFormValues["entity_scope"], {
                  shouldDirty: true,
                })
              }
              options={[
                ["all", "all"],
                ["product", "product"],
                ["service", "service"],
                ["seller", "seller"],
              ]}
            />
            <label className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm">
              <input className="mr-2" type="checkbox" {...cacheForm.register("purge_all")} />
              purge_all
            </label>
            <TextInputField label="query_hash" {...cacheForm.register("query_hash")} />
            <TextInputField label="X-Idempotency-Key" error={cacheForm.formState.errors.idempotency_key?.message} {...cacheForm.register("idempotency_key")} />
            <SubmitWriteButton disabled={!canInvalidateCache} pending={cacheMutation.isPending} label="失效缓存" />
          </form>
        </SearchWriteCard>

        <SearchWriteCard
          title="推荐重建"
          description="触发 recommendation cache / features 重建，要求 X-Idempotency-Key 与 step-up。"
          allowed={canManageRecommendation}
          mutation={rebuildMutation}
        >
          <form className="grid gap-3" onSubmit={rebuildForm.handleSubmit((values) => rebuildMutation.mutate(values))}>
            <div className="grid gap-3 md:grid-cols-2">
              <SelectField
                label="scope"
                value={useWatch({ control: rebuildForm.control, name: "scope" })}
                onChange={(value) =>
                  rebuildForm.setValue("scope", value as RecommendationRebuildFormValues["scope"], {
                    shouldDirty: true,
                  })
                }
                options={["all", "cache", "features", "subject_profile", "cohort", "signals", "similarity", "bundle"].map((item) => [item, item])}
              />
              <SelectField
                label="entity_scope"
                value={useWatch({ control: rebuildForm.control, name: "entity_scope" })}
                onChange={(value) =>
                  rebuildForm.setValue("entity_scope", value as RecommendationRebuildFormValues["entity_scope"], {
                    shouldDirty: true,
                  })
                }
                options={[
                  ["all", "all"],
                  ["product", "product"],
                  ["seller", "seller"],
                ]}
              />
            </div>
            <TextInputField label="placement_code" {...rebuildForm.register("placement_code")} />
            <TextInputField label="entity_id" error={rebuildForm.formState.errors.entity_id?.message} {...rebuildForm.register("entity_id")} />
            <label className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm">
              <input className="mr-2" type="checkbox" {...rebuildForm.register("purge_cache")} />
              purge_cache
            </label>
            <WriteHeaders form={rebuildForm} />
            <SubmitWriteButton disabled={!canManageRecommendation} pending={rebuildMutation.isPending} label="执行推荐重建" />
          </form>
        </SearchWriteCard>
      </div>

      <div className="grid gap-4 xl:grid-cols-2">
        <Card className="min-w-0">
          <CardTitle>搜索排序配置</CardTitle>
          <QueryPanel query={searchRankingQuery} title="search.ranking_profile">
            {(response) => (
              <VirtualTable
                columns={searchRankingColumns}
                data={response.data}
                emptyLabel="暂无搜索排序配置"
              />
            )}
          </QueryPanel>
          <form className="mt-5 grid gap-3" onSubmit={rankingForm.handleSubmit((values) => rankingMutation.mutate(values))}>
            <TextInputField label="ranking_profile_id" error={rankingForm.formState.errors.ranking_profile_id?.message} {...rankingForm.register("ranking_profile_id")} />
            <TextInputField label="status" {...rankingForm.register("status")} />
            <TextareaField label="weights_json" placeholder='{"quality":0.7}' {...rankingForm.register("weights_json")} />
            <TextareaField label="filter_policy_json" placeholder='{"visible":true}' {...rankingForm.register("filter_policy_json")} />
            <WriteHeaders form={rankingForm} />
            <SubmitWriteButton disabled={!canManageSearch} pending={rankingMutation.isPending} label="更新搜索排序" />
            {rankingMutation.isError ? <ErrorState error={rankingMutation.error} compact /> : null}
          </form>
        </Card>

        <Card className="min-w-0">
          <CardTitle>推荐位与推荐排序</CardTitle>
          {!canReadRecommendation ? (
            <PermissionNotice title="当前主体缺少推荐运维读取权限" required="ops.recommendation.read" compact />
          ) : null}
          <QueryPanel query={placementsQuery} title="recommend.placement_definition">
            {(response) => (
              <VirtualTable
                columns={placementColumns}
                data={response.data}
                emptyLabel="暂无推荐位"
              />
            )}
          </QueryPanel>
          <QueryPanel query={recommendationRankingQuery} title="recommend.ranking_profile">
            {(response) => (
              <VirtualTable
                columns={recommendationRankingColumns}
                data={response.data}
                emptyLabel="暂无推荐排序配置"
              />
            )}
          </QueryPanel>
        </Card>
      </div>

      <div className="grid gap-4 xl:grid-cols-2">
        <Card>
          <CardTitle>推荐位更新</CardTitle>
          <form className="mt-4 grid gap-3" onSubmit={placementForm.handleSubmit((values) => placementMutation.mutate(values))}>
            <TextInputField label="placement_code" error={placementForm.formState.errors.placement_code?.message} {...placementForm.register("placement_code")} />
            <TextInputField label="status" {...placementForm.register("status")} />
            <TextInputField label="default_ranking_profile_key" {...placementForm.register("default_ranking_profile_key")} />
            <WriteHeaders form={placementForm} />
            <SubmitWriteButton disabled={!canManageRecommendation} pending={placementMutation.isPending} label="更新推荐位" />
            {placementMutation.isError ? <ErrorState error={placementMutation.error} compact /> : null}
          </form>
        </Card>
        <Card>
          <CardTitle>推荐排序更新</CardTitle>
          <form className="mt-4 grid gap-3" onSubmit={recommendationRankingForm.handleSubmit((values) => recommendationRankingMutation.mutate(values))}>
            <TextInputField label="ranking_profile_id" error={recommendationRankingForm.formState.errors.ranking_profile_id?.message} {...recommendationRankingForm.register("ranking_profile_id")} />
            <TextInputField label="status" {...recommendationRankingForm.register("status")} />
            <TextInputField label="explain_codes CSV" {...recommendationRankingForm.register("explain_codes")} />
            <WriteHeaders form={recommendationRankingForm} />
            <SubmitWriteButton disabled={!canManageRecommendation} pending={recommendationRankingMutation.isPending} label="更新推荐排序" />
            {recommendationRankingMutation.isError ? <ErrorState error={recommendationRankingMutation.error} compact /> : null}
          </form>
        </Card>
      </div>

      {lastAction ? <ResultBlock title={lastAction.title} value={lastAction.value} /> : null}
    </ConsoleRouteScaffold>
  );
}

function ConsistencyResult({ data }: { data: ConsistencyData }) {
  const proofAnchor = data.proof_state.latest_chain_anchor;
  const projectionGap = data.proof_state.latest_projection_gap;
  const externalReceipt = data.external_fact_state.latest_receipt;

  return (
    <div className="space-y-4">
      <div className="grid gap-4 xl:grid-cols-3">
        <StatusCard
          title="业务主状态"
          items={[
            ["ref", `${data.ref_type}/${data.ref_id}`],
            ["business_status", data.business_state.business_status],
            ["authority_model", data.business_state.authority_model],
            ["reconcile_status", data.business_state.reconcile_status],
          ]}
        />
        <StatusCard
          title="链证明状态"
          items={[
            ["proof_commit_state", data.proof_state.proof_commit_state],
            ["proof_commit_policy", data.proof_state.proof_commit_policy],
            ["tx_hash", proofAnchor?.tx_hash ?? "未返回"],
            ["projection_gap", projectionGap?.gap_status ?? "未返回"],
          ]}
        />
        <StatusCard
          title="外部事实状态"
          items={[
            ["summary_status", data.external_fact_state.summary_status],
            ["total_receipts", String(data.external_fact_state.total_receipts)],
            ["latest_fact_type", externalReceipt?.fact_type ?? "未返回"],
            ["receipt_status", externalReceipt?.receipt_status ?? "未返回"],
          ]}
        />
      </div>
      <div className="grid gap-4 xl:grid-cols-3">
        <MiniList
          title="recent outbox"
          items={data.recent_outbox_events.map((item) => ({
            key: item.outbox_event_id ?? item.request_id ?? item.event_type,
            title: item.event_type,
            description: `${item.status} / ${item.target_topic ?? "topic 未返回"}`,
          }))}
        />
        <MiniList
          title="recent dead letters"
          items={data.recent_dead_letters.map((item) => ({
            key: item.dead_letter_event_id ?? item.request_id ?? item.event_type,
            title: item.event_type,
            description: `${item.reprocess_status} / ${item.failure_stage}`,
          }))}
        />
        <MiniList
          title="recent audit traces"
          items={data.recent_audit_traces.map((item, index) => ({
            key: item.audit_id ?? `${item.action_name}-${index}`,
            title: item.action_name,
            description: `${item.result_code} / ${item.request_id ?? "request_id 未返回"}`,
          }))}
        />
      </div>
    </div>
  );
}

function ObservabilityOverviewPanel({ subject }: { subject?: SessionSubject }) {
  const canRead = canReadObservability(subject);
  const query = useQuery({
    queryKey: ["console", "ops", "observability-overview"],
    queryFn: () => sdk.ops.getObservabilityOverview(),
    enabled: canRead,
  });

  if (!canRead) {
    return (
      <PermissionNotice
        title="观测总览入口权限不足"
        required="ops.observability.read"
        compact
      />
    );
  }

  return (
    <Card>
      <div className="flex items-center justify-between gap-3">
        <div>
          <CardTitle>观测总览入口</CardTitle>
          <CardDescription className="mt-2">
            后端探针和事件镜像来自 `GET /api/v1/ops/observability/overview`。
          </CardDescription>
        </div>
        <RadioTower className="size-6 text-[var(--accent-strong)]" />
      </div>
      <QueryPanel query={query} title="observability">
        {(response) => <ObservabilitySummary data={response.data} />}
      </QueryPanel>
    </Card>
  );
}

function ObservabilitySummary({ data }: { data: ObservabilityData }) {
  return (
    <div className="mt-4 grid gap-3 md:grid-cols-4">
      <MetricTile label="backends" value={String(data.backend_statuses.length)} />
      <MetricTile label="open alerts" value={String(data.alert_summary.open_count)} />
      <MetricTile label="critical" value={String(data.alert_summary.critical_count)} />
      <MetricTile label="incidents" value={String(data.recent_incidents.length)} />
      <div className="md:col-span-4 grid gap-2">
        {data.backend_statuses.slice(0, 4).map((item) => (
          <div
            className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-black/[0.04] px-4 py-3 text-sm"
            key={item.backend.backend_key}
          >
            <span className="font-medium text-[var(--ink-strong)]">
              {item.backend.backend_key} / {item.backend.backend_type}
            </span>
            <Badge className={badgeClass(statusTone(item.probe_status))}>
              {item.probe_status}
            </Badge>
          </div>
        ))}
      </div>
    </div>
  );
}

function QueryPanel<T>({
  query,
  title,
  children,
}: {
  query: {
    isPending: boolean;
    isFetching: boolean;
    isError: boolean;
    isSuccess: boolean;
    error: unknown;
    data?: T;
  };
  title: string;
  children: (data: T) => ReactNode;
}) {
  if (query.isPending || query.isFetching) {
    return <LoadingState label={`${title} 加载中`} compact />;
  }
  if (query.isError) {
    return <ErrorState error={query.error} compact />;
  }
  if (query.isSuccess && query.data) {
    return <>{children(query.data)}</>;
  }
  return <EmptyState title={`${title} 未查询`} description="等待权限、筛选条件或后端返回。" compact />;
}

function VirtualTable<T>({
  columns,
  data,
  emptyLabel,
}: {
  columns: ColumnDef<T>[];
  data: T[];
  emptyLabel: string;
}) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const parentRef = useRef<HTMLDivElement>(null);
  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable({
    data,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });
  const rows = table.getRowModel().rows;
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 62,
    overscan: 8,
  });
  const template = `repeat(${columns.length}, minmax(150px, 1fr))`;

  if (!data.length) {
    return <EmptyState title={emptyLabel} description="后端返回了正式空集合。" compact />;
  }

  return (
    <div className="mt-4 min-w-0 overflow-hidden rounded-[24px] border border-black/10 bg-white/70">
      <div
        className="grid min-w-[760px] border-b border-black/10 bg-black/[0.04] text-xs font-semibold uppercase tracking-[0.14em] text-[var(--ink-subtle)]"
        style={{ gridTemplateColumns: template }}
      >
        {table.getHeaderGroups()[0]?.headers.map((header) => (
          <button
            className="px-4 py-3 text-left"
            key={header.id}
            type="button"
            onClick={header.column.getToggleSortingHandler()}
          >
            {flexRender(header.column.columnDef.header, header.getContext())}
            {header.column.getIsSorted() ? ` ${header.column.getIsSorted()}` : ""}
          </button>
        ))}
      </div>
      <div className="max-w-full overflow-x-auto">
        <div className="relative h-[360px] min-w-[760px] overflow-auto" ref={parentRef}>
          <div className="relative" style={{ height: virtualizer.getTotalSize() }}>
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const row = rows[virtualRow.index];
              return (
                <div
                  className="absolute left-0 grid w-full border-b border-black/5 text-sm text-[var(--ink-soft)]"
                  key={row.id}
                  style={{
                    gridTemplateColumns: template,
                    transform: `translateY(${virtualRow.start}px)`,
                  }}
                >
                  {row.getVisibleCells().map((cell) => (
                    <div className="truncate px-4 py-3" key={cell.id}>
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </div>
                  ))}
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

function SearchWriteCard({
  title,
  description,
  allowed,
  mutation,
  children,
}: {
  title: string;
  description: string;
  allowed: boolean;
  mutation: { isError: boolean; error: unknown };
  children: ReactNode;
}) {
  return (
    <Card>
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <CardTitle>{title}</CardTitle>
          <CardDescription className="mt-2">{description}</CardDescription>
        </div>
        <Badge className={allowed ? badgeClass("warn") : badgeClass("danger")}>
          {allowed ? "可执行" : "权限态"}
        </Badge>
      </div>
      <div className="mt-5">{children}</div>
      {mutation.isError ? <ErrorState error={mutation.error} compact /> : null}
    </Card>
  );
}

function WriteHeaders({
  form,
}: {
  form: {
    register: (name: "idempotency_key" | "step_up_token" | "step_up_challenge_id") => Record<string, unknown>;
    formState: { errors: Record<string, { message?: string } | undefined> };
  };
}) {
  return (
    <div className="grid gap-3 lg:grid-cols-3">
      <TextInputField
        label="X-Idempotency-Key"
        error={form.formState.errors.idempotency_key?.message}
        {...form.register("idempotency_key")}
      />
      <TextInputField
        label="X-Step-Up-Token"
        error={form.formState.errors.step_up_token?.message}
        {...form.register("step_up_token")}
      />
      <TextInputField
        label="X-Step-Up-Challenge-Id"
        {...form.register("step_up_challenge_id")}
      />
    </div>
  );
}

function SubmitWriteButton({
  disabled,
  pending,
  label,
}: {
  disabled: boolean;
  pending: boolean;
  label: string;
}) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <AuditHint />
      <Button disabled={disabled || pending} type="submit" variant="warning">
        {pending ? <LoaderCircle className="size-4 animate-spin" /> : <LockKeyhole className="size-4" />}
        {label}
      </Button>
    </div>
  );
}

function useAuthMe() {
  return useQuery({
    queryKey: ["console", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
  });
}

function OpsHero({
  title,
  description,
  icon,
  badges,
}: {
  title: string;
  description: string;
  icon: ReactNode;
  badges: string[];
}) {
  return (
    <motion.section
      animate={{ opacity: 1, y: 0 }}
      className="overflow-hidden rounded-[32px] border border-white/70 bg-[radial-gradient(circle_at_top_left,rgba(30,136,158,0.25),transparent_34%),linear-gradient(135deg,#102a43,#184e5c_48%,#ecf8f2)] p-6 text-white shadow-[0_28px_80px_rgba(13,37,48,0.22)]"
      initial={{ opacity: 0, y: 18 }}
      transition={{ duration: 0.28 }}
    >
      <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
        <div className="max-w-3xl">
          <div className="mb-4 inline-flex items-center gap-2 rounded-full bg-white/12 px-4 py-2 text-sm">
            {icon}
            WEB-015 ops control plane
          </div>
          <h1 className="text-3xl font-semibold tracking-tight md:text-5xl">{title}</h1>
          <p className="mt-3 max-w-2xl text-sm leading-6 text-white/76">{description}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          {badges.map((badge) => (
            <Badge className="bg-white/15 text-white" key={badge}>
              {badge}
            </Badge>
          ))}
        </div>
      </div>
    </motion.section>
  );
}

function SubjectCard({
  subject,
  loading,
}: {
  subject?: SessionSubject;
  loading: boolean;
}) {
  return (
    <Card>
      <div className="flex items-start justify-between gap-3">
        <div>
          <CardTitle>当前主体 / 角色 / 租户 / 作用域</CardTitle>
          <CardDescription className="mt-2">
            敏感 ops 页面必须显示当前 IAM 上下文，并只通过受控代理访问 `platform-core`。
          </CardDescription>
        </div>
        <ShieldCheck className="size-6 text-[var(--accent-strong)]" />
      </div>
      {loading ? (
        <LoadingState label="正在读取 auth/me" compact />
      ) : (
        <div className="mt-4 grid gap-3 md:grid-cols-2">
          <InfoTile label="主体" value={subjectDisplayName(subject)} />
          <InfoTile label="角色" value={(subject?.roles ?? []).join(" / ") || "未登录"} />
          <InfoTile label="租户" value={subject?.tenant_id ?? subject?.org_id ?? "未返回"} />
          <InfoTile label="作用域" value={subject?.auth_context_level ?? "未返回"} />
        </div>
      )}
    </Card>
  );
}

function BoundaryCard() {
  return (
    <Card>
      <CardTitle>前端边界</CardTitle>
      <CardDescription className="mt-2">
        页面只访问 `/api/platform/**`，由控制台代理转发到 `platform-core`；Kafka、PostgreSQL、OpenSearch、Redis、Fabric 只作为后端受控依赖出现。
      </CardDescription>
      <div className="mt-4 grid gap-2">
        {["PostgreSQL 真值", "Kafka/outbox 分发", "OpenSearch 读模型", "Redis 缓存", "Fabric 可信确认"].map((item) => (
          <div className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm text-[var(--ink-soft)]" key={item}>
            {item}
          </div>
        ))}
      </div>
    </Card>
  );
}

function StatusCard({
  title,
  items,
}: {
  title: string;
  items: Array<[string, string]>;
}) {
  return (
    <Card>
      <CardTitle>{title}</CardTitle>
      <div className="mt-4 grid gap-2">
        {items.map(([label, value]) => (
          <div className="flex items-start justify-between gap-3 rounded-2xl bg-black/[0.04] px-4 py-3 text-sm" key={label}>
            <span className="text-[var(--ink-subtle)]">{label}</span>
            <span className="max-w-[60%] truncate font-mono text-[var(--ink-strong)]">{value}</span>
          </div>
        ))}
      </div>
    </Card>
  );
}

function MiniList({
  title,
  items,
}: {
  title: string;
  items: Array<{ key: string; title: string; description: string }>;
}) {
  return (
    <Card>
      <CardTitle>{title}</CardTitle>
      <div className="mt-4 grid gap-2">
        {items.length ? (
          items.slice(0, 6).map((item) => (
            <div className="rounded-2xl bg-black/[0.04] px-4 py-3" key={item.key}>
              <div className="truncate text-sm font-medium text-[var(--ink-strong)]">{item.title}</div>
              <div className="mt-1 truncate text-xs text-[var(--ink-soft)]">{item.description}</div>
            </div>
          ))
        ) : (
          <EmptyState title="暂无记录" description="后端返回了正式空数组。" compact />
        )}
      </div>
    </Card>
  );
}

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl bg-black/[0.04] px-4 py-3">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</div>
      <div className="mt-2 truncate font-mono text-sm text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}

function MetricTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl bg-black/[0.04] p-4">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</div>
      <div className="mt-2 text-2xl font-semibold text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}

function ErrorState({ error, compact = false }: { error: unknown; compact?: boolean }) {
  const detail = formatOpsError(error);
  return (
    <Card className={cn("border-red-200 bg-red-50/85", compact ? "mt-4 p-4" : undefined)}>
      <div className="flex items-start gap-3">
        <AlertTriangle className="mt-1 size-5 text-red-700" />
        <div>
          <CardTitle className="text-base text-red-950">{detail.title}</CardTitle>
          <CardDescription className="mt-1 text-red-800">{detail.message}</CardDescription>
          <div className="mt-2 font-mono text-xs text-red-700">request_id: {detail.requestId}</div>
        </div>
      </div>
    </Card>
  );
}

function LoadingState({ label, compact = false }: { label: string; compact?: boolean }) {
  return (
    <div
      className={cn(
        "mt-4 flex items-center gap-3 rounded-[24px] bg-white/70 px-4 py-3 text-sm text-[var(--ink-soft)]",
        compact ? "py-3" : "py-6",
      )}
    >
      <LoaderCircle className="size-4 animate-spin" />
      {label}
    </div>
  );
}

function EmptyState({
  title,
  description,
  compact = false,
}: {
  title: string;
  description: string;
  compact?: boolean;
}) {
  return (
    <div className={cn("mt-4 rounded-[24px] bg-black/[0.04] p-5", compact ? "p-4" : undefined)}>
      <div className="font-medium text-[var(--ink-strong)]">{title}</div>
      <p className="mt-1 text-sm leading-6 text-[var(--ink-soft)]">{description}</p>
    </div>
  );
}

function PermissionNotice({
  title,
  required,
  compact = false,
}: {
  title: string;
  required: string;
  compact?: boolean;
}) {
  return (
    <Card className={cn("border-amber-200 bg-amber-50/85", compact ? "p-4" : undefined)}>
      <div className="flex items-start gap-3">
        <LockKeyhole className="mt-1 size-5 text-amber-700" />
        <div>
          <CardTitle className="text-base text-amber-950">{title}</CardTitle>
          <CardDescription className="mt-1 text-amber-800">
            需要权限：`{required}`。页面保留权限态，不发起高风险写动作。
          </CardDescription>
        </div>
      </div>
    </Card>
  );
}

function ResultBlock({ title, value }: { title: string; value: unknown }) {
  return (
    <Card className="mt-4 border-emerald-200 bg-emerald-50/80">
      <div className="flex items-start gap-3">
        <Sparkles className="mt-1 size-5 text-emerald-700" />
        <div className="min-w-0 flex-1">
          <CardTitle className="text-base text-emerald-950">{title}</CardTitle>
          <pre className="mt-3 max-h-[320px] overflow-auto rounded-2xl bg-white/80 p-4 text-xs text-emerald-950">
            {JSON.stringify(value, null, 2)}
          </pre>
        </div>
      </div>
    </Card>
  );
}

function StatCard({
  label,
  value,
  hint,
}: {
  label: string;
  value: number | string;
  hint: string;
}) {
  return (
    <Card className="gap-2 border-black/10 bg-white/80">
      <div className="text-xs uppercase tracking-[0.14em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="text-3xl font-semibold text-[var(--ink-strong)]">{value}</div>
      <CardDescription>{hint}</CardDescription>
    </Card>
  );
}

function AuditHint() {
  return (
    <div className="inline-flex items-center gap-2 rounded-full bg-amber-50 px-4 py-2 text-xs font-medium text-amber-800 ring-1 ring-amber-200">
      <Activity className="size-4" />
      写动作会留下 audit.audit_event / access_audit / ops.system_log
    </div>
  );
}

function SelectField({
  label,
  value,
  onChange,
  options,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: Array<[string, string]>;
}) {
  return (
    <label className="grid gap-2 text-sm">
      <span className="font-medium text-[var(--ink-soft)]">{label}</span>
      <select
        className="h-11 rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]"
        onChange={(event) => onChange(event.target.value)}
        value={value}
      >
        {options.map(([optionValue, optionLabel]) => (
          <option key={optionValue} value={optionValue}>
            {optionLabel}
          </option>
        ))}
      </select>
    </label>
  );
}

function TextInputField({
  label,
  error,
  ...props
}: ComponentProps<typeof Input> & { label: string; error?: string }) {
  return (
    <label className="grid gap-2 text-sm">
      <span className="font-medium text-[var(--ink-soft)]">{label}</span>
      <Input {...props} />
      {error ? <span className="text-xs text-red-700">{error}</span> : null}
    </label>
  );
}

function TextareaField({
  label,
  error,
  ...props
}: ComponentProps<typeof Textarea> & { label: string; error?: string }) {
  return (
    <label className="grid gap-2 text-sm">
      <span className="font-medium text-[var(--ink-soft)]">{label}</span>
      <Textarea {...props} />
      {error ? <span className="text-xs text-red-700">{error}</span> : null}
    </label>
  );
}

function badgeClass(tone: ReturnType<typeof statusTone>) {
  if (tone === "ok") {
    return "bg-emerald-100 text-emerald-800";
  }
  if (tone === "danger") {
    return "bg-red-100 text-red-800";
  }
  return "bg-amber-100 text-amber-800";
}
