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
import { useMutation, useQuery } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import type {
  AuditPackageExportResponse,
  DeveloperTraceResponse,
  ExternalFactsResponse,
  OrderAuditResponse,
  ProjectionGapsResponse,
  TradeMonitorCheckpointsResponse,
  TradeMonitorOverviewResponse,
} from "@datab/sdk-ts";
import {
  AlertTriangle,
  Box,
  Download,
  Fingerprint,
  LoaderCircle,
  LockKeyhole,
  Network,
  PackageCheck,
  RefreshCcw,
  Search,
  ShieldCheck,
} from "lucide-react";
import { motion } from "motion/react";
import {
  startTransition,
  useEffect,
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
  auditLookupFormSchema,
  auditLookupLabels,
  auditPackageExportFormSchema,
  buildAuditTraceQuery,
  buildDeveloperTraceQuery,
  buildExternalFactsQuery,
  buildPackageExportPayload,
  buildProjectionGapsQuery,
  canExportAuditPackage,
  canReadAuditTrace,
  canReadDeveloperTrace,
  canReadExternalFacts,
  canReadOpsTradeMonitor,
  canReadProjectionGaps,
  createAuditIdempotencyKey,
  formatAuditDate,
  formatAuditError,
  normalizeAuditEvents,
  resolveOrderIdFromLookup,
  safePackageExportView,
  subjectDisplayName,
  summarizeAuditGroups,
  type AuditEventGroup,
  type AuditLookupFormValues,
  type AuditPackageExportFormValues,
  type SessionSubject,
  type UnifiedAuditEventRow,
} from "@/lib/audit-trace";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { cn } from "@/lib/utils";

import { ConsoleRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();

const auditGroupLabels: Record<AuditEventGroup | "all", string> = {
  all: "全部",
  order: "订单",
  billing: "账单",
  delivery: "交付",
  dispute: "争议",
  evidence: "证据",
  chain: "链",
  other: "其他",
};

export function AuditTraceShell() {
  const [lookup, setLookup] = useState<AuditLookupFormValues | null>(null);
  const authQuery = useQuery({
    queryKey: ["console", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
  });
  const subject = authQuery.data?.data;
  const canRead = canReadAuditTrace(subject);
  const canUseDeveloperTrace = canReadDeveloperTrace(subject);

  const lookupForm = useForm<AuditLookupFormValues>({
    resolver: zodResolver(auditLookupFormSchema),
    defaultValues: {
      lookup_key: "order_id",
      lookup_value: "",
      page_size: 50,
    },
  });
  const lookupKeyValue = useWatch({
    control: lookupForm.control,
    name: "lookup_key",
  });

  const auditTraceQueryParams = lookup ? buildAuditTraceQuery(lookup) : null;
  const developerTraceQueryParams = lookup ? buildDeveloperTraceQuery(lookup) : null;

  const traceQuery = useQuery({
    queryKey: ["audit", "traces", auditTraceQueryParams],
    queryFn: () => sdk.audit.searchTraces(auditTraceQueryParams ?? {}),
    enabled: Boolean(canRead && auditTraceQueryParams),
  });

  const orderAuditQuery = useQuery({
    queryKey: ["audit", "order", lookup?.lookup_value, lookup?.page_size],
    queryFn: () =>
      sdk.audit.getOrderAudit(
        { id: lookup?.lookup_value ?? "" },
        { page: 1, page_size: lookup?.page_size ?? 50 },
      ),
    enabled: Boolean(canRead && lookup?.lookup_key === "order_id"),
  });

  const developerTraceQuery = useQuery({
    queryKey: ["ops", "developer-trace", developerTraceQueryParams],
    queryFn: () => sdk.ops.getDeveloperTrace(developerTraceQueryParams ?? {}),
    enabled: Boolean(canRead && canUseDeveloperTrace && developerTraceQueryParams),
  });

  const auditRows = normalizeAuditEvents(
    traceQuery.data?.data.items,
    orderAuditQuery.data?.data.traces,
    developerTraceQuery.data?.data.recent_audit_traces,
  );
  const resolvedOrderId = resolveOrderIdFromLookup(
    lookup,
    developerTraceQuery.data,
    orderAuditQuery.data,
    auditRows,
  );
  const externalFactsQueryParams = buildExternalFactsFilter(lookup, resolvedOrderId);
  const projectionGapsQueryParams = buildProjectionGapsFilter(lookup, resolvedOrderId);

  const tradeMonitorQuery = useQuery({
    queryKey: ["ops", "trade-monitor", resolvedOrderId],
    queryFn: () => sdk.ops.getTradeMonitorOverview({ orderId: resolvedOrderId ?? "" }),
    enabled: Boolean(canRead && canReadOpsTradeMonitor(subject) && resolvedOrderId),
  });

  const checkpointQuery = useQuery({
    queryKey: ["ops", "trade-monitor-checkpoints", resolvedOrderId],
    queryFn: () =>
      sdk.ops.listTradeMonitorCheckpoints(
        { orderId: resolvedOrderId ?? "" },
        { page: 1, page_size: 20 },
      ),
    enabled: Boolean(canRead && canReadOpsTradeMonitor(subject) && resolvedOrderId),
  });

  const externalFactsQuery = useQuery({
    queryKey: ["ops", "external-facts", externalFactsQueryParams],
    queryFn: () => sdk.ops.listExternalFacts(externalFactsQueryParams ?? {}),
    enabled: Boolean(canRead && canReadExternalFacts(subject) && externalFactsQueryParams),
  });

  const projectionGapsQuery = useQuery({
    queryKey: ["ops", "projection-gaps", projectionGapsQueryParams],
    queryFn: () => sdk.ops.listProjectionGaps(projectionGapsQueryParams ?? {}),
    enabled: Boolean(canRead && canReadProjectionGaps(subject) && projectionGapsQueryParams),
  });

  const isTraceLoading =
    traceQuery.isPending ||
    orderAuditQuery.isPending ||
    developerTraceQuery.isPending ||
    tradeMonitorQuery.isPending ||
    checkpointQuery.isPending ||
    externalFactsQuery.isPending ||
    projectionGapsQuery.isPending;

  function onLookupSubmit(values: AuditLookupFormValues) {
    startTransition(() => {
      setLookup(values);
    });
  }

  return (
    <ConsoleRouteScaffold routeKey="audit_trace">
      <AuditHero subject={subject} />

      {authQuery.isPending ? (
        <LoadingPanel title="正在解析控制台主体" />
      ) : authQuery.isError ? (
        <ErrorPanel error={authQuery.error} onRetry={() => void authQuery.refetch()} />
      ) : !canRead ? (
        <ForbiddenPanel subject={subject} />
      ) : (
        <div className="space-y-5">
          <Card>
            <div className="grid gap-5 xl:grid-cols-[0.9fr_1.1fr]">
              <div>
                <CardTitle>联查主键</CardTitle>
                <CardDescription className="mt-2">
                  支持页面说明书冻结的 `order_id / request_id / tx_hash / case_id / delivery_id`。
                  `tx_hash` 通过受控 `developer.trace` 解析链回执，不直接访问 Fabric。
                </CardDescription>
              </div>
              <form
                className="grid gap-3 lg:grid-cols-[220px_1fr_120px_auto]"
                onSubmit={lookupForm.handleSubmit(onLookupSubmit)}
              >
                <SelectField
                  label="主键类型"
                  value={lookupKeyValue}
                  onChange={(value) =>
                    lookupForm.setValue("lookup_key", value as AuditLookupFormValues["lookup_key"], {
                      shouldDirty: true,
                    })
                  }
                  options={Object.entries(auditLookupLabels)}
                />
                <div className="space-y-1">
                  <label className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
                    主键值
                  </label>
                  <Input
                    {...lookupForm.register("lookup_value")}
                    placeholder="输入正式 UUID、request_id 或 tx_hash"
                  />
                  <FieldError message={lookupForm.formState.errors.lookup_value?.message} />
                </div>
                <div className="space-y-1">
                  <label className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
                    页大小
                  </label>
                  <Input
                    type="number"
                    min={5}
                    max={100}
                    {...lookupForm.register("page_size", { valueAsNumber: true })}
                  />
                </div>
                <div className="flex items-end">
                  <Button type="submit" className="w-full">
                    <Search className="size-4" />
                    联查
                  </Button>
                </div>
              </form>
            </div>
          </Card>

          {!lookup ? (
            <EmptyPanel
              title="等待联查条件"
              description="输入一个正式主键后，页面会同时读取审计事件、订单审计视图、贸易监控、链回执、外部事实和投影差异。"
            />
          ) : (
            <>
              <LookupSummary
                lookup={lookup}
                resolvedOrderId={resolvedOrderId}
                subject={subject}
                developerTraceAllowed={canUseDeveloperTrace}
              />
              <QueryErrorStrip
                queries={[
                  traceQuery,
                  orderAuditQuery,
                  developerTraceQuery,
                  tradeMonitorQuery,
                  checkpointQuery,
                  externalFactsQuery,
                  projectionGapsQuery,
                ]}
              />
              {isTraceLoading ? <LoadingPanel title="正在读取全链路事实" /> : null}
              <AuditOverviewGrid
                rows={auditRows}
                orderAudit={orderAuditQuery.data}
                tradeMonitor={tradeMonitorQuery.data}
                developerTrace={developerTraceQuery.data}
              />
              <TraceTable rows={auditRows} loading={isTraceLoading} />
              <div className="grid gap-5 xl:grid-cols-2">
                <ChainReceiptPanel
                  developerTrace={developerTraceQuery.data}
                  tradeMonitor={tradeMonitorQuery.data}
                />
                <EvidenceObjectsPanel rows={auditRows} />
                <ExternalFactsPanel
                  response={externalFactsQuery.data}
                  developerTrace={developerTraceQuery.data}
                  tradeMonitor={tradeMonitorQuery.data}
                />
                <ProjectionGapsPanel
                  response={projectionGapsQuery.data}
                  developerTrace={developerTraceQuery.data}
                  tradeMonitor={tradeMonitorQuery.data}
                />
                <CheckpointPanel response={checkpointQuery.data} />
                <AuditExportPanel
                  subject={subject}
                  prefillRefType={lookup.lookup_key === "case_id" ? "dispute_case" : "order"}
                  prefillRefId={lookup.lookup_key === "order_id" || lookup.lookup_key === "case_id" ? lookup.lookup_value : (resolvedOrderId ?? "")}
                />
              </div>
            </>
          )}
        </div>
      )}
    </ConsoleRouteScaffold>
  );
}

export function AuditPackageExportShell() {
  const authQuery = useQuery({
    queryKey: ["console", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
  });
  const subject = authQuery.data?.data;
  const canExport = canExportAuditPackage(subject);

  return (
    <ConsoleRouteScaffold routeKey="audit_package_export">
      <AuditHero subject={subject} compact />
      {authQuery.isPending ? (
        <LoadingPanel title="正在解析证据包导出主体" />
      ) : authQuery.isError ? (
        <ErrorPanel error={authQuery.error} onRetry={() => void authQuery.refetch()} />
      ) : !canExport ? (
        <ForbiddenPanel subject={subject} exportOnly />
      ) : (
        <div className="grid gap-5 xl:grid-cols-[0.9fr_1.1fr]">
          <Card className="bg-[linear-gradient(135deg,rgba(22,52,61,0.96),rgba(25,85,90,0.86),rgba(233,195,120,0.62))] text-white">
            <div className="space-y-4">
              <Badge className="bg-white/15 text-white">audit.package.export</Badge>
              <h1 className="text-3xl font-semibold tracking-tight">
                导出只展示证据包元信息，不暴露真实对象路径。
              </h1>
              <p className="text-sm leading-7 text-white/80">
                控制台会把导出请求发送到 `POST /api/v1/audit/packages/export`，
                透传 `X-Idempotency-Key` 和 step-up 头。后端负责 MinIO 对象、审计事件、
                access_audit 与 system_log 写入。
              </p>
            </div>
          </Card>
          <AuditExportPanel subject={subject} />
        </div>
      )}
    </ConsoleRouteScaffold>
  );
}

function AuditHero({
  subject,
  compact = false,
}: {
  subject?: SessionSubject;
  compact?: boolean;
}) {
  return (
    <div className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
      <Card className="overflow-hidden bg-[radial-gradient(circle_at_12%_8%,rgba(240,196,92,0.26),transparent_34%),linear-gradient(135deg,rgba(19,31,38,0.98),rgba(39,74,83,0.94),rgba(93,106,86,0.82))] text-white">
        <div className="space-y-5">
          <div className="flex flex-wrap gap-2">
            <Badge className="bg-white/15 text-white">WEB-014</Badge>
            <Badge className="bg-white/15 text-white">audit.trace.read</Badge>
            <Badge className="bg-white/15 text-white">platform-core only</Badge>
          </div>
          <div>
            <h1 className="text-3xl font-semibold tracking-tight">
              审计事件、链回执、外部事实和证据包导出在控制台闭环。
            </h1>
            {!compact ? (
              <p className="mt-3 max-w-3xl text-sm leading-7 text-white/78">
                页面只通过 `/api/platform/**` 代理调用 `platform-core`，不会让浏览器直连
                PostgreSQL、Kafka、OpenSearch、Redis、Fabric 或对象存储。链字段展示
                `request_id / tx_hash / proof_commit_state / reconcile_status`，未返回时明确标注。
              </p>
            ) : null}
          </div>
          <IdentitySnapshot subject={subject} inverse />
        </div>
      </Card>
      <Card>
        <CardTitle>审计边界</CardTitle>
        <CardDescription className="mt-2">
          审计联查是只读控制面；证据包导出是高风险写动作，必须带幂等键、step-up 和导出原因。
        </CardDescription>
        <div className="mt-4 grid gap-3">
          <BoundaryRow icon={<Fingerprint className="size-4" />} label="查询键" value="order_id / request_id / tx_hash / case_id / delivery_id" />
          <BoundaryRow icon={<Network className="size-4" />} label="链字段" value="request_id / tx_hash / 链状态 / 投影状态" />
          <BoundaryRow icon={<PackageCheck className="size-4" />} label="证据对象" value="仅显示 package / manifest / digest，不展示 storage_uri" />
          <BoundaryRow icon={<LockKeyhole className="size-4" />} label="写入头" value="X-Idempotency-Key + X-Step-Up-Token / challenge" />
        </div>
      </Card>
    </div>
  );
}

function AuditOverviewGrid({
  rows,
  orderAudit,
  tradeMonitor,
  developerTrace,
}: {
  rows: UnifiedAuditEventRow[];
  orderAudit?: OrderAuditResponse;
  tradeMonitor?: TradeMonitorOverviewResponse;
  developerTrace?: DeveloperTraceResponse;
}) {
  const groups = summarizeAuditGroups(rows);
  const orderStatus =
    orderAudit?.data.status ||
    tradeMonitor?.data.business_state ||
    developerTrace?.data.subject.business_status ||
    "未返回";
  const paymentStatus =
    orderAudit?.data.payment_status ||
    developerTrace?.data.subject.payment_status ||
    "未返回";
  const proofState =
    tradeMonitor?.data.proof_commit_state ||
    developerTrace?.data.subject.proof_commit_state ||
    "未返回";
  const reconcileStatus =
    tradeMonitor?.data.reconcile_status ||
    developerTrace?.data.subject.reconcile_status ||
    "未返回";

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <MetricCard label="审计事件" value={String(rows.length)} hint={`账单 ${groups.billing} / 交付 ${groups.delivery} / 证据 ${groups.evidence}`} />
      <MetricCard label="订单状态" value={orderStatus} hint={`payment_status=${paymentStatus}`} />
      <MetricCard label="链状态" value={proofState} hint={`reconcile_status=${reconcileStatus}`} />
      <MetricCard label="外部事实" value={tradeMonitor?.data.external_fact_status || developerTrace?.data.subject.external_fact_status || "未返回"} hint={`request_id=${tradeMonitor?.data.request_id || developerTrace?.data.subject.request_id || "未返回"}`} />
    </div>
  );
}

function TraceTable({
  rows,
  loading,
}: {
  rows: UnifiedAuditEventRow[];
  loading: boolean;
}) {
  const [groupFilter, setGroupFilter] = useState<AuditEventGroup | "all">("all");
  const [sorting, setSorting] = useState<SortingState>([
    { id: "occurred_at", desc: true },
  ]);
  const parentRef = useRef<HTMLDivElement>(null);
  const filteredRows =
    groupFilter === "all" ? rows : rows.filter((row) => row.group === groupFilter);
  const columns: Array<ColumnDef<UnifiedAuditEventRow>> = [
    {
      accessorKey: "occurred_at",
      header: ({ column }) => (
        <button
          className="text-left font-semibold"
          type="button"
          onClick={column.getToggleSortingHandler()}
        >
          时间 {column.getIsSorted() === "asc" ? "↑" : "↓"}
        </button>
      ),
      cell: ({ row }) => formatAuditDate(row.original.occurred_at),
    },
    {
      accessorKey: "domain_name",
      header: "域 / 动作",
      cell: ({ row }) => (
        <div>
          <div className="font-semibold text-[var(--ink-strong)]">
            {row.original.domain_name}
          </div>
          <div className="mt-1 text-xs text-[var(--ink-soft)]">
            {row.original.action_name}
          </div>
        </div>
      ),
    },
    {
      accessorKey: "ref_type",
      header: "关联对象",
      cell: ({ row }) => (
        <div className="font-mono text-xs text-[var(--ink-soft)]">
          {row.original.ref_type}:{row.original.ref_id || "未返回"}
        </div>
      ),
    },
    {
      accessorKey: "result_code",
      header: "结果 / 错误码",
      cell: ({ row }) => (
        <div className="space-y-1">
          <StatusPill value={row.original.result_code} />
          {row.original.error_code ? (
            <div className="font-mono text-xs text-[var(--danger-ink,#7f1d1d)]">
              {row.original.error_code}
            </div>
          ) : null}
        </div>
      ),
    },
    {
      accessorKey: "request_id",
      header: "request / trace / tx",
      cell: ({ row }) => (
        <div className="space-y-1 font-mono text-xs text-[var(--ink-soft)]">
          <div>req: {row.original.request_id || "未返回"}</div>
          <div>trace: {row.original.trace_id || "未返回"}</div>
          <div>tx: {row.original.tx_hash || "未返回"}</div>
        </div>
      ),
    },
    {
      accessorKey: "evidence_manifest_id",
      header: "证据",
      cell: ({ row }) => (
        <div className="font-mono text-xs text-[var(--ink-soft)]">
          {row.original.evidence_manifest_id || row.original.event_hash || "未绑定"}
        </div>
      ),
    },
  ];
  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable({
    data: filteredRows,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });
  const tableRows = table.getRowModel().rows;
  const rowVirtualizer = useVirtualizer({
    count: tableRows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 78,
    overscan: 8,
  });

  return (
    <Card className="min-w-0">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <CardTitle>审计事件长列表</CardTitle>
          <CardDescription className="mt-2">
            使用 TanStack Table + Virtual；支持按事件组筛选与时间排序，空态和错误态由联查区承接。
          </CardDescription>
        </div>
        <div className="flex flex-wrap gap-2">
          {(Object.keys(auditGroupLabels) as Array<AuditEventGroup | "all">).map((group) => (
            <button
              key={group}
              type="button"
              onClick={() => setGroupFilter(group)}
              className={cn(
                "rounded-full px-3 py-1 text-xs font-semibold transition",
                groupFilter === group
                  ? "bg-[var(--accent-strong)] text-white"
                  : "bg-black/[0.04] text-[var(--ink-soft)] hover:bg-black/[0.08]",
              )}
            >
              {auditGroupLabels[group]}
            </button>
          ))}
        </div>
      </div>

      {loading ? (
        <LoadingPanel title="审计事件加载中" compact />
      ) : tableRows.length === 0 ? (
        <EmptyPanel title="没有匹配的审计事件" description="当前查询未返回事件；可切换查询键或扩大 page_size 后重试。" />
      ) : (
        <div className="mt-5 overflow-x-auto rounded-[24px] border border-black/10 bg-white/70">
          <div className="min-w-[1120px]">
            <div className="grid grid-cols-[170px_180px_180px_140px_260px_1fr] gap-3 border-b border-black/10 bg-black/[0.03] px-4 py-3 text-xs uppercase tracking-[0.14em] text-[var(--ink-subtle)]">
              {table.getHeaderGroups()[0]?.headers.map((header) => (
                <div key={header.id}>
                  {flexRender(header.column.columnDef.header, header.getContext())}
                </div>
              ))}
            </div>
            <div ref={parentRef} className="h-[460px] overflow-auto">
              <div
                className="relative w-full"
                style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
              >
                {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                  const row = tableRows[virtualRow.index];
                  if (!row) {
                    return null;
                  }
                  return (
                    <motion.div
                      key={row.id}
                      initial={{ opacity: 0, y: 8 }}
                      animate={{ opacity: 1, y: 0 }}
                      className="absolute left-0 grid w-full grid-cols-[170px_180px_180px_140px_260px_1fr] gap-3 border-b border-black/[0.06] px-4 py-3 text-sm"
                      style={{ transform: `translateY(${virtualRow.start}px)` }}
                    >
                      {row.getVisibleCells().map((cell) => (
                        <div key={cell.id} className="min-w-0">
                          {flexRender(cell.column.columnDef.cell, cell.getContext())}
                        </div>
                      ))}
                    </motion.div>
                  );
                })}
              </div>
            </div>
          </div>
        </div>
      )}
    </Card>
  );
}

function LookupSummary({
  lookup,
  resolvedOrderId,
  subject,
  developerTraceAllowed,
}: {
  lookup: AuditLookupFormValues;
  resolvedOrderId: string | null;
  subject?: SessionSubject;
  developerTraceAllowed: boolean;
}) {
  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <MetricCard label="当前查询键" value={auditLookupLabels[lookup.lookup_key]} hint={lookup.lookup_value} />
      <MetricCard label="解析订单" value={resolvedOrderId ?? "未解析"} hint="用于 trade-monitor / facts / projection 联查" />
      <MetricCard label="当前角色" value={subject?.roles.join(" / ") || "visitor"} hint={`scope=${subject?.auth_context_level ?? "public"}`} />
      <MetricCard label="developer.trace" value={developerTraceAllowed ? "已开放" : "未授权"} hint="tx_hash 解析需要 developer.trace.read" />
    </div>
  );
}

function ChainReceiptPanel({
  developerTrace,
  tradeMonitor,
}: {
  developerTrace?: DeveloperTraceResponse;
  tradeMonitor?: TradeMonitorOverviewResponse;
}) {
  const anchors = dedupeBy(
    [
      developerTrace?.data.matched_chain_anchor,
      ...(developerTrace?.data.recent_chain_anchors ?? []),
    ].filter(isPresent),
    (item) => item.chain_anchor_id || item.tx_hash || item.digest || "",
  );
  const subject = developerTrace?.data.subject;

  return (
    <Card>
      <CardTitle>链回执与投影状态</CardTitle>
      <CardDescription className="mt-2">
        链状态来自 `platform-core` 投影读模型，不由前端直连 Fabric。
      </CardDescription>
      <div className="mt-4 grid gap-3 md:grid-cols-2">
        <FactRow label="request_id" value={tradeMonitor?.data.request_id || subject?.request_id || "未返回"} />
        <FactRow label="tx_hash" value={anchors[0]?.tx_hash || "未返回"} />
        <FactRow label="链状态" value={tradeMonitor?.data.proof_commit_state || subject?.proof_commit_state || anchors[0]?.status || "未返回"} />
        <FactRow label="投影状态" value={tradeMonitor?.data.reconcile_status || subject?.reconcile_status || anchors[0]?.reconcile_status || "未返回"} />
      </div>
      <div className="mt-4 space-y-3">
        {anchors.length ? (
          anchors.map((anchor) => (
            <div key={anchor.chain_anchor_id || anchor.tx_hash || anchor.digest} className="rounded-[22px] bg-black/[0.04] p-4 text-sm">
              <div className="flex flex-wrap gap-2">
                <StatusPill value={anchor.status} />
                <StatusPill value={anchor.reconcile_status} />
              </div>
              <div className="mt-3 grid gap-2 font-mono text-xs text-[var(--ink-soft)] md:grid-cols-2">
                <div>chain_id={anchor.chain_id}</div>
                <div>tx_hash={anchor.tx_hash || "未返回"}</div>
                <div>ref={anchor.ref_type}:{anchor.ref_id || "未返回"}</div>
                <div>anchored_at={formatAuditDate(anchor.anchored_at)}</div>
              </div>
            </div>
          ))
        ) : (
          <EmptyPanel
            compact
            title="未返回链回执明细"
            description="当前角色或查询键未返回 chain_anchor；页面仍展示 trade-monitor 的 proof / reconcile 摘要。"
          />
        )}
      </div>
    </Card>
  );
}

function EvidenceObjectsPanel({ rows }: { rows: UnifiedAuditEventRow[] }) {
  const evidenceRows = rows.filter((row) => row.evidence_manifest_id || row.event_hash);
  return (
    <Card>
      <CardTitle>证据对象</CardTitle>
      <CardDescription className="mt-2">
        仅展示 `evidence_manifest_id / event_hash / audit_id`，不展示对象真实路径。
      </CardDescription>
      <div className="mt-4 space-y-3">
        {evidenceRows.length ? (
          evidenceRows.slice(0, 8).map((row) => (
            <div key={row.key} className="rounded-[22px] bg-black/[0.04] p-4 text-sm">
              <div className="font-semibold text-[var(--ink-strong)]">{row.action_name}</div>
              <div className="mt-2 space-y-1 font-mono text-xs text-[var(--ink-soft)]">
                <div>manifest={row.evidence_manifest_id || "未返回"}</div>
                <div>event_hash={row.event_hash || "未返回"}</div>
                <div>audit_id={row.audit_id || "未返回"}</div>
              </div>
            </div>
          ))
        ) : (
          <EmptyPanel compact title="未绑定证据对象" description="当前审计事件未返回 evidence_manifest_id 或 event_hash。" />
        )}
      </div>
    </Card>
  );
}

function ExternalFactsPanel({
  response,
  developerTrace,
  tradeMonitor,
}: {
  response?: ExternalFactsResponse;
  developerTrace?: DeveloperTraceResponse;
  tradeMonitor?: TradeMonitorOverviewResponse;
}) {
  const facts = dedupeBy(
    [
      ...(response?.data.items ?? []),
      ...(developerTrace?.data.recent_external_facts ?? []),
      ...(tradeMonitor?.data.recent_external_facts ?? []),
    ],
    (item) => item.external_fact_receipt_id || item.receipt_hash || "",
  );

  return (
    <Card>
      <CardTitle>外部事实</CardTitle>
      <CardDescription className="mt-2">
        来自 `ops.external_fact_receipt` 读模型；前端不承担外部事实真相源。
      </CardDescription>
      <div className="mt-4 space-y-3">
        {facts.length ? (
          facts.slice(0, 8).map((fact) => (
            <div key={fact.external_fact_receipt_id || fact.receipt_hash || fact.provider_reference} className="rounded-[22px] bg-black/[0.04] p-4 text-sm">
              <div className="flex flex-wrap gap-2">
                <StatusPill value={fact.fact_type} />
                <StatusPill value={fact.receipt_status} />
              </div>
              <div className="mt-3 grid gap-2 font-mono text-xs text-[var(--ink-soft)] md:grid-cols-2">
                <div>provider={fact.provider_type}/{fact.provider_key || "未返回"}</div>
                <div>request_id={fact.request_id || "未返回"}</div>
                <div>trace_id={fact.trace_id || "未返回"}</div>
                <div>received_at={formatAuditDate(fact.received_at)}</div>
              </div>
            </div>
          ))
        ) : (
          <EmptyPanel compact title="未返回外部事实" description="当前查询没有匹配 external_fact_receipt，或当前角色没有 ops.external_fact.read。" />
        )}
      </div>
    </Card>
  );
}

function ProjectionGapsPanel({
  response,
  developerTrace,
  tradeMonitor,
}: {
  response?: ProjectionGapsResponse;
  developerTrace?: DeveloperTraceResponse;
  tradeMonitor?: TradeMonitorOverviewResponse;
}) {
  const gaps = dedupeBy(
    [
      ...(response?.data.items ?? []),
      developerTrace?.data.matched_projection_gap,
      ...(developerTrace?.data.recent_projection_gaps ?? []),
      ...(tradeMonitor?.data.recent_projection_gaps ?? []),
    ].filter(isPresent),
    (item) => item.chain_projection_gap_id || item.projected_tx_hash || "",
  );

  return (
    <Card>
      <CardTitle>投影差异</CardTitle>
      <CardDescription className="mt-2">
        展示链投影差异读模型，前端只读 `platform-core` 的 ops API。
      </CardDescription>
      <div className="mt-4 space-y-3">
        {gaps.length ? (
          gaps.slice(0, 8).map((gap) => (
            <div key={gap.chain_projection_gap_id || gap.projected_tx_hash || gap.expected_tx_id} className="rounded-[22px] bg-black/[0.04] p-4 text-sm">
              <div className="flex flex-wrap gap-2">
                <StatusPill value={gap.gap_type} />
                <StatusPill value={gap.gap_status} />
              </div>
              <div className="mt-3 grid gap-2 font-mono text-xs text-[var(--ink-soft)] md:grid-cols-2">
                <div>chain_id={gap.chain_id}</div>
                <div>projected_tx_hash={gap.projected_tx_hash || "未返回"}</div>
                <div>request_id={gap.request_id || "未返回"}</div>
                <div>trace_id={gap.trace_id || "未返回"}</div>
              </div>
            </div>
          ))
        ) : (
          <EmptyPanel compact title="无投影差异" description="当前查询未返回 open projection gap；若角色不足，页面不绕过 ops API。" />
        )}
      </div>
    </Card>
  );
}

function CheckpointPanel({ response }: { response?: TradeMonitorCheckpointsResponse }) {
  const checkpoints = response?.data.items ?? [];
  return (
    <Card>
      <CardTitle>交付 / 账单 / 生命周期记录</CardTitle>
      <CardDescription className="mt-2">
        `ops.trade_lifecycle_checkpoint` 用于辅助展示交付、验收、账单等事件顺序。
      </CardDescription>
      <div className="mt-4 space-y-3">
        {checkpoints.length ? (
          checkpoints.slice(0, 10).map((checkpoint) => (
            <div key={checkpoint.trade_lifecycle_checkpoint_id || `${checkpoint.ref_id}:${checkpoint.checkpoint_code}`} className="rounded-[22px] bg-black/[0.04] p-4 text-sm">
              <div className="flex flex-wrap gap-2">
                <StatusPill value={checkpoint.lifecycle_stage} />
                <StatusPill value={checkpoint.checkpoint_status} />
              </div>
              <div className="mt-3 grid gap-2 font-mono text-xs text-[var(--ink-soft)] md:grid-cols-2">
                <div>checkpoint={checkpoint.checkpoint_code}</div>
                <div>related_tx_hash={checkpoint.related_tx_hash || "未返回"}</div>
                <div>request_id={checkpoint.request_id || "未返回"}</div>
                <div>occurred_at={formatAuditDate(checkpoint.occurred_at)}</div>
              </div>
            </div>
          ))
        ) : (
          <EmptyPanel compact title="未返回生命周期 checkpoint" description="订单未解析或当前角色没有 ops.trade_monitor.read。" />
        )}
      </div>
    </Card>
  );
}

function AuditExportPanel({
  subject,
  prefillRefType = "order",
  prefillRefId = "",
}: {
  subject?: SessionSubject;
  prefillRefType?: "order" | "case" | "dispute_case";
  prefillRefId?: string;
}) {
  const canExport = canExportAuditPackage(subject);
  const [lastResponse, setLastResponse] = useState<AuditPackageExportResponse | null>(null);
  const form = useForm<AuditPackageExportFormValues>({
    resolver: zodResolver(auditPackageExportFormSchema),
    defaultValues: {
      ref_type: prefillRefType,
      ref_id: prefillRefId,
      reason: "",
      masked_level: "masked",
      package_type: "forensic_export",
      idempotency_key: createAuditIdempotencyKey(),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const exportRefType = useWatch({
    control: form.control,
    name: "ref_type",
  });
  const exportMaskedLevel = useWatch({
    control: form.control,
    name: "masked_level",
  });

  useEffect(() => {
    if (!prefillRefId) {
      return;
    }
    form.reset({
      ...form.getValues(),
      ref_type: prefillRefType,
      ref_id: prefillRefId,
      idempotency_key: createAuditIdempotencyKey(),
    });
  }, [form, prefillRefId, prefillRefType]);

  const mutation = useMutation({
    mutationFn: (values: AuditPackageExportFormValues) =>
      sdk.audit.exportPackage(buildPackageExportPayload(values), {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token,
        stepUpChallengeId: values.step_up_challenge_id,
      }),
    onSuccess: (response) => {
      setLastResponse(response);
      form.setValue("idempotency_key", createAuditIdempotencyKey());
    },
  });

  const safeView = safePackageExportView(lastResponse ?? undefined);

  return (
    <Card className="min-w-0">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <CardTitle>证据包导出</CardTitle>
          <CardDescription className="mt-2">
            高风险动作：需要 `audit.package.export`、导出原因、幂等键和 step-up。导出结果隐藏 `storage_uri`。
          </CardDescription>
        </div>
        <Badge className={canExport ? "bg-emerald-100 text-emerald-800" : "bg-amber-100 text-amber-800"}>
          {canExport ? "可导出" : "无导出权限"}
        </Badge>
      </div>

      <form
        className="mt-5 grid gap-4"
        onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
      >
        <div className="grid gap-4 md:grid-cols-[160px_1fr]">
          <SelectField
            label="ref_type"
            value={exportRefType}
            onChange={(value) =>
              form.setValue("ref_type", value as AuditPackageExportFormValues["ref_type"], {
                shouldDirty: true,
              })
            }
            options={[
              ["order", "order"],
              ["case", "case"],
              ["dispute_case", "dispute_case"],
            ]}
          />
          <TextInputField
            label="ref_id"
            placeholder="正式 UUID"
            error={form.formState.errors.ref_id?.message}
            {...form.register("ref_id")}
          />
        </div>
        <div className="grid gap-4 md:grid-cols-3">
          <SelectField
            label="masked_level"
            value={exportMaskedLevel}
            onChange={(value) =>
              form.setValue("masked_level", value as AuditPackageExportFormValues["masked_level"], {
                shouldDirty: true,
              })
            }
            options={[
              ["summary", "summary"],
              ["masked", "masked"],
              ["unmasked", "unmasked"],
            ]}
          />
          <TextInputField
            label="package_type"
            error={form.formState.errors.package_type?.message}
            {...form.register("package_type")}
          />
          <TextInputField
            label="X-Idempotency-Key"
            error={form.formState.errors.idempotency_key?.message}
            {...form.register("idempotency_key")}
          />
        </div>
        <div className="grid gap-4 md:grid-cols-2">
          <TextInputField
            label="X-Step-Up-Token"
            placeholder="step-up-token"
            error={form.formState.errors.step_up_token?.message}
            {...form.register("step_up_token")}
          />
          <TextInputField
            label="X-Step-Up-Challenge-Id"
            placeholder="challenge id"
            error={form.formState.errors.step_up_challenge_id?.message}
            {...form.register("step_up_challenge_id")}
          />
        </div>
        <div className="space-y-1">
          <label className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
            导出原因
          </label>
          <Textarea
            rows={4}
            placeholder="例如：监管抽查 / 争议复核 / 司法保全导出"
            {...form.register("reason")}
          />
          <FieldError message={form.formState.errors.reason?.message} />
        </div>
        <div className="rounded-[24px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-4 text-sm text-[var(--warning-ink)]">
          导出动作会追加审计事件、访问审计和 system_log；前端不会展示 `storage_uri` 或对象真实路径。
        </div>
        {mutation.isError ? <ErrorPanel error={mutation.error} compact /> : null}
        <div className="flex flex-wrap gap-3">
          <Button type="submit" disabled={!canExport || mutation.isPending}>
            {mutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <Download className="size-4" />}
            导出证据包
          </Button>
          <Button
            type="button"
            variant="secondary"
            onClick={() => form.setValue("idempotency_key", createAuditIdempotencyKey())}
          >
            <RefreshCcw className="size-4" />
            换幂等键
          </Button>
        </div>
      </form>

      {safeView ? (
        <div className="mt-5 rounded-[24px] bg-black/[0.04] p-4">
          <div className="flex flex-wrap gap-2">
            <StatusPill value={safeView.legal_hold_status} />
            <StatusPill value={safeView.step_up_bound ? "step_up_bound" : "step_up_missing"} />
          </div>
          <div className="mt-4 grid gap-2 font-mono text-xs text-[var(--ink-soft)] md:grid-cols-2">
            <div>package_id={safeView.evidence_package.evidence_package_id || "未返回"}</div>
            <div>manifest_id={safeView.evidence_manifest.evidence_manifest_id || "未返回"}</div>
            <div>package_digest={safeView.evidence_package.package_digest || "未返回"}</div>
            <div>manifest_hash={safeView.evidence_manifest.manifest_hash || "未返回"}</div>
            <div>audit_trace_count={safeView.audit_trace_count}</div>
            <div>evidence_item_count={safeView.evidence_item_count}</div>
          </div>
          <div className="mt-3 text-xs text-[var(--ink-subtle)]">
            已隐藏字段：{safeView.hidden_fields.join(" / ")}
          </div>
        </div>
      ) : null}
    </Card>
  );
}

function QueryErrorStrip({ queries }: { queries: Array<{ isError: boolean; error: unknown }> }) {
  const errors = queries.filter((query) => query.isError).map((query) => query.error);
  if (!errors.length) {
    return null;
  }
  return (
    <div className="grid gap-3 md:grid-cols-2">
      {errors.map((error, index) => (
        <ErrorPanel key={index} error={error} compact />
      ))}
    </div>
  );
}

function IdentitySnapshot({
  subject,
  inverse = false,
}: {
  subject?: SessionSubject;
  inverse?: boolean;
}) {
  const itemClass = inverse ? "bg-white/12 text-white" : "bg-black/[0.04]";
  return (
    <div className="grid gap-3 md:grid-cols-4">
      <HeroMetric label="当前主体" value={subjectDisplayName(subject)} inverse={inverse} className={itemClass} />
      <HeroMetric label="当前角色" value={subject?.roles.join(" / ") || "visitor"} inverse={inverse} className={itemClass} />
      <HeroMetric label="租户/组织" value={subject?.tenant_id ?? subject?.org_id ?? "public"} inverse={inverse} className={itemClass} />
      <HeroMetric label="认证上下文" value={subject?.auth_context_level ?? "public"} inverse={inverse} className={itemClass} />
    </div>
  );
}

function MetricCard({
  label,
  value,
  hint,
}: {
  label: string;
  value: string;
  hint?: string;
}) {
  return (
    <Card className="min-w-0">
      <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 truncate text-xl font-semibold text-[var(--ink-strong)]">
        {value}
      </div>
      {hint ? (
        <div className="mt-2 truncate font-mono text-xs text-[var(--ink-soft)]">
          {hint}
        </div>
      ) : null}
    </Card>
  );
}

function HeroMetric({
  label,
  value,
  inverse = false,
  className,
}: {
  label: string;
  value: string;
  inverse?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("rounded-[22px] p-4", className)}>
      <div className={cn("text-xs uppercase tracking-[0.18em]", inverse ? "text-white/60" : "text-[var(--ink-subtle)]")}>
        {label}
      </div>
      <div className={cn("mt-2 truncate text-sm font-semibold", inverse ? "text-white" : "text-[var(--ink-strong)]")}>
        {value}
      </div>
    </div>
  );
}

function BoundaryRow({
  icon,
  label,
  value,
}: {
  icon: ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-start gap-3 rounded-[22px] bg-black/[0.04] p-4">
      <div className="mt-0.5 text-[var(--accent-strong)]">{icon}</div>
      <div>
        <div className="text-sm font-semibold text-[var(--ink-strong)]">{label}</div>
        <div className="mt-1 text-sm text-[var(--ink-soft)]">{value}</div>
      </div>
    </div>
  );
}

function FactRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[20px] bg-black/[0.04] p-3">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 break-all font-mono text-xs text-[var(--ink-strong)]">
        {value}
      </div>
    </div>
  );
}

function StatusPill({ value }: { value: string }) {
  const tone =
    value.includes("fail") || value.includes("error") || value.includes("open")
      ? "bg-red-100 text-red-800"
      : value.includes("pending") || value.includes("missing")
        ? "bg-amber-100 text-amber-800"
        : "bg-emerald-100 text-emerald-800";
  return (
    <span className={cn("inline-flex rounded-full px-3 py-1 text-xs font-semibold", tone)}>
      {value}
    </span>
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
    <div className="space-y-1">
      <label className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </label>
      <select
        className="h-11 w-full rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        {options.map(([optionValue, optionLabel]) => (
          <option key={optionValue} value={optionValue}>
            {optionLabel}
          </option>
        ))}
      </select>
    </div>
  );
}

function TextInputField({
  label,
  error,
  ...props
}: InputPropsWithLabel) {
  return (
    <div className="space-y-1">
      <label className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </label>
      <Input {...props} />
      <FieldError message={error} />
    </div>
  );
}

type InputPropsWithLabel = ComponentProps<typeof Input> & {
  label: string;
  error?: string;
};

function FieldError({ message }: { message?: string }) {
  return message ? (
    <div className="text-xs font-medium text-red-700">{message}</div>
  ) : null;
}

function LoadingPanel({
  title,
  compact = false,
}: {
  title: string;
  compact?: boolean;
}) {
  return (
    <Card className={cn("flex items-center gap-3", compact && "p-4")}>
      <LoaderCircle className="size-5 animate-spin text-[var(--accent-strong)]" />
      <div>
        <CardTitle className="text-base">{title}</CardTitle>
        <CardDescription>正在通过受控 API 读取 `platform-core`。</CardDescription>
      </div>
    </Card>
  );
}

function EmptyPanel({
  title,
  description,
  compact = false,
}: {
  title: string;
  description: string;
  compact?: boolean;
}) {
  return (
    <Card className={cn("border-dashed", compact && "p-4")}>
      <div className="flex items-start gap-3">
        <Box className="mt-1 size-5 text-[var(--ink-subtle)]" />
        <div>
          <CardTitle className="text-base">{title}</CardTitle>
          <CardDescription className="mt-1">{description}</CardDescription>
        </div>
      </div>
    </Card>
  );
}

function ErrorPanel({
  error,
  onRetry,
  compact = false,
}: {
  error: unknown;
  onRetry?: () => void;
  compact?: boolean;
}) {
  const formatted = formatAuditError(error);
  return (
    <Card className={cn("border-red-200 bg-red-50/80", compact && "p-4")}>
      <div className="flex items-start gap-3">
        <AlertTriangle className="mt-1 size-5 text-red-700" />
        <div className="min-w-0 flex-1">
          <CardTitle className="text-base text-red-950">{formatted.title}</CardTitle>
          <CardDescription className="mt-1 text-red-800">
            {formatted.message}
          </CardDescription>
          <div className="mt-2 font-mono text-xs text-red-700">
            request_id={formatted.requestId}
          </div>
          {onRetry ? (
            <Button className="mt-3" size="sm" variant="secondary" onClick={onRetry}>
              <RefreshCcw className="size-4" />
              重试
            </Button>
          ) : null}
        </div>
      </div>
    </Card>
  );
}

function ForbiddenPanel({
  subject,
  exportOnly = false,
}: {
  subject?: SessionSubject;
  exportOnly?: boolean;
}) {
  return (
    <Card className="border-amber-200 bg-amber-50/80">
      <div className="flex items-start gap-3">
        <ShieldCheck className="mt-1 size-5 text-amber-700" />
        <div>
          <CardTitle className="text-amber-950">当前主体权限不足</CardTitle>
          <CardDescription className="mt-2 text-amber-800">
            {exportOnly
              ? "证据包导出需要 platform_audit_security 与 audit.package.export。"
              : "审计联查需要 audit.trace.read；页面不会绕过权限或改用前端直连读模型。"}
          </CardDescription>
          <div className="mt-3 grid gap-2 font-mono text-xs text-amber-900 md:grid-cols-4">
            <div>subject={subjectDisplayName(subject)}</div>
            <div>roles={subject?.roles.join("/") || "visitor"}</div>
            <div>tenant={subject?.tenant_id ?? subject?.org_id ?? "public"}</div>
            <div>scope={subject?.auth_context_level ?? "public"}</div>
          </div>
        </div>
      </div>
    </Card>
  );
}

function buildExternalFactsFilter(
  lookup: AuditLookupFormValues | null,
  orderId: string | null,
) {
  if (orderId) {
    return buildExternalFactsQuery(orderId);
  }
  if (!lookup) {
    return null;
  }
  if (lookup.lookup_key === "request_id") {
    return { request_id: lookup.lookup_value, page: 1, page_size: 20 };
  }
  if (lookup.lookup_key === "case_id") {
    return { ref_type: "dispute_case", ref_id: lookup.lookup_value, page: 1, page_size: 20 };
  }
  if (lookup.lookup_key === "delivery_id") {
    return { ref_type: "delivery", ref_id: lookup.lookup_value, page: 1, page_size: 20 };
  }
  return null;
}

function buildProjectionGapsFilter(
  lookup: AuditLookupFormValues | null,
  orderId: string | null,
) {
  if (orderId) {
    return buildProjectionGapsQuery(orderId);
  }
  if (lookup?.lookup_key === "request_id") {
    return { request_id: lookup.lookup_value, page: 1, page_size: 20 };
  }
  return null;
}

function dedupeBy<T>(items: T[], getKey: (item: T) => string) {
  const seen = new Set<string>();
  const result: T[] = [];
  for (const item of items) {
    const key = getKey(item);
    if (!key || seen.has(key)) {
      continue;
    }
    seen.add(key);
    result.push(item);
  }
  return result;
}

function isPresent<T>(value: T | null | undefined): value is T {
  return value !== null && value !== undefined;
}
