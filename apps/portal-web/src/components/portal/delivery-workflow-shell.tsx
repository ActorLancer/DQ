"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  Boxes,
  CheckCircle2,
  ClipboardCheck,
  Download,
  FileArchive,
  GitBranch,
  KeyRound,
  LoaderCircle,
  LockKeyhole,
  RefreshCcw,
  ServerCog,
  Share2,
  ShieldCheck,
  TerminalSquare,
} from "lucide-react";
import { motion } from "motion/react";
import type { Route } from "next";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import type { ReactNode } from "react";
import { useState } from "react";
import { useForm, type Path, type UseFormReturn } from "react-hook-form";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  DELIVERY_ENTRIES,
  buildCommitDeliveryRequest,
  buildRevisionSubscriptionRequest,
  buildSandboxWorkspaceRequest,
  buildShareGrantRequest,
  buildTemplateGrantRequest,
  canOperateDelivery,
  canReadDelivery,
  commitDeliveryFormSchema,
  createDeliveryIdempotencyKey,
  defaultCommitDeliveryValues,
  defaultRevisionSubscriptionValues,
  defaultSandboxWorkspaceValues,
  defaultShareGrantValues,
  defaultTemplateGrantValues,
  deliveryEntryByKind,
  deliveryEntryForSku,
  deliveryRouteForEntry,
  formatDeliveryError,
  getOrderSkuType,
  maskSecret,
  readSubjectTenant,
  revisionSubscriptionFormSchema,
  sandboxWorkspaceFormSchema,
  shareGrantFormSchema,
  skuDisplayName,
  templateGrantFormSchema,
  unwrapApiUsageLog,
  unwrapCommitDelivery,
  unwrapDownloadTicket,
  unwrapQueryRuns,
  unwrapRevisionSubscription,
  unwrapRevisionSubscriptionMutation,
  unwrapSandboxWorkspace,
  unwrapShareGrant,
  unwrapShareGrantList,
  unwrapTemplateGrant,
  type CommitDeliveryFormValues,
  type CommitDeliveryResult,
  type DeliveryEntry,
  type DeliveryRouteKind,
  type DownloadTicket as DownloadTicketData,
  type RevisionSubscriptionFormValues,
  type SandboxWorkspaceFormValues,
  type ShareGrantFormValues,
  type TemplateGrantFormValues,
  type SessionSubject,
} from "@/lib/delivery-workflow";
import { orderStatusLabel, unwrapLifecycle, unwrapOrderDetail } from "@/lib/order-workflow";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import type { PortalSessionPreview } from "@/lib/session";
import { cn, formatList } from "@/lib/utils";

import { PreviewStateControls, ScaffoldPill, getPreviewState } from "./state-preview";

const sdk = createBrowserSdk();

type DeliveryWorkflowShellProps = {
  kind: DeliveryRouteKind;
  orderId: string;
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

export function DeliveryWorkflowShell({
  kind,
  orderId,
  sessionMode,
  initialSubject,
}: DeliveryWorkflowShellProps) {
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);
  const entry = deliveryEntryByKind(kind);

  const authQuery = useQuery({
    queryKey: ["portal", "delivery", orderId, "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest" && preview === "ready",
  });
  const subject = (authQuery.data?.data ?? initialSubject) as SessionSubject | null;
  const canRead = canReadDelivery(subject);
  const canOperate = canOperateDelivery(subject, kind);

  const detailQuery = useQuery({
    queryKey: ["portal", "delivery", orderId, "detail"],
    queryFn: () => sdk.trade.getOrderDetail({ id: orderId }),
    enabled: sessionMode !== "guest" && preview === "ready" && canRead,
  });
  const lifecycleQuery = useQuery({
    queryKey: ["portal", "delivery", orderId, "lifecycle"],
    queryFn: () => sdk.trade.getOrderLifecycleSnapshots({ id: orderId }),
    enabled: sessionMode !== "guest" && preview === "ready" && canRead,
  });
  const shareQuery = useQuery({
    queryKey: ["portal", "delivery", orderId, "share-grants"],
    queryFn: () => sdk.delivery.getShareGrants({ id: orderId }),
    enabled: preview === "ready" && canRead && kind === "share",
    retry: false,
  });
  const subscriptionQuery = useQuery({
    queryKey: ["portal", "delivery", orderId, "subscription"],
    queryFn: () => sdk.delivery.getRevisionSubscription({ id: orderId }),
    enabled: preview === "ready" && canRead && kind === "subscription",
    retry: false,
  });
  const usageQuery = useQuery({
    queryKey: ["portal", "delivery", orderId, "usage-log"],
    queryFn: () => sdk.delivery.getApiUsageLog({ id: orderId }),
    enabled: preview === "ready" && canRead && kind === "api",
    retry: false,
  });
  const queryRunsQuery = useQuery({
    queryKey: ["portal", "delivery", orderId, "query-runs"],
    queryFn: () => sdk.delivery.getQueryRuns({ id: orderId }),
    enabled: preview === "ready" && canRead && kind === "template-query",
    retry: false,
  });

  const order = unwrapOrderDetail(detailQuery.data);
  const lifecycle = unwrapLifecycle(lifecycleQuery.data);
  const skuType = getOrderSkuType(order);
  const expectedEntry = deliveryEntryForSku(skuType);
  const isSkuMatched = !order || entry.supportedSkus.includes(skuType as never);
  const loading =
    authQuery.isLoading ||
    detailQuery.isLoading ||
    lifecycleQuery.isLoading;
  const error =
    authQuery.error ??
    detailQuery.error ??
    lifecycleQuery.error;

  if (preview !== "ready") {
    return <DeliveryPreviewState entry={entry} orderId={orderId} preview={preview} />;
  }

  return (
    <div className="space-y-6">
      <DeliveryHero
        entry={entry}
        orderId={orderId}
        subject={subject}
        sessionMode={sessionMode}
        preview={preview}
      />

      {sessionMode === "guest" || !canRead ? (
        <PermissionPanel entry={entry} subject={subject} sessionMode={sessionMode} />
      ) : loading ? (
        <LoadingPanel entry={entry} />
      ) : error ? (
        <ErrorPanel title={`${entry.title}错误态`} message={formatDeliveryError(error)} />
      ) : !order ? (
        <EmptyPanel entry={entry} />
      ) : (
        <>
          <OrderSummary
            order={order}
            lifecycle={lifecycle}
            entry={entry}
            expectedEntry={expectedEntry}
            isSkuMatched={isSkuMatched}
            orderId={orderId}
          />
          <DeliveryEntryGrid orderId={orderId} activeKind={kind} skuType={skuType} />
          <BranchReadState
            kind={kind}
            shareData={unwrapShareGrantList(shareQuery.data)}
            shareError={shareQuery.error}
            subscriptionData={unwrapRevisionSubscription(subscriptionQuery.data)}
            subscriptionError={subscriptionQuery.error}
            usageData={unwrapApiUsageLog(usageQuery.data)}
            usageError={usageQuery.error}
            queryRunsData={unwrapQueryRuns(queryRunsQuery.data)}
            queryRunsError={queryRunsQuery.error}
          />
          {!canOperate ? (
            <ActionPermissionPanel entry={entry} subject={subject} />
          ) : !isSkuMatched ? (
            <SkuMismatchPanel
              currentEntry={entry}
              expectedEntry={expectedEntry}
              skuType={skuType}
              orderId={orderId}
            />
          ) : (
            <DeliveryActionPanel
              kind={kind}
              orderId={orderId}
              onChanged={() => {
                void detailQuery.refetch();
                void lifecycleQuery.refetch();
                void shareQuery.refetch();
                void subscriptionQuery.refetch();
                void usageQuery.refetch();
                void queryRunsQuery.refetch();
              }}
            />
          )}
        </>
      )}
    </div>
  );
}

function DeliveryHero({
  entry,
  orderId,
  subject,
  sessionMode,
  preview,
}: {
  entry: DeliveryEntry;
  orderId: string;
  subject: SessionSubject | null;
  sessionMode: string;
  preview: string;
}) {
  const meta = portalRouteMap[entry.routeKey];

  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28 }}
      className="grid gap-4 lg:grid-cols-[1.45fr_1fr]"
    >
      <Card className="overflow-hidden bg-[radial-gradient(circle_at_top_left,rgba(29,92,114,0.18),transparent_34%),linear-gradient(135deg,rgba(255,255,255,0.96),rgba(237,247,244,0.88),rgba(246,240,225,0.76))]">
        <div className="space-y-5">
          <div className="flex flex-wrap gap-2">
            <ScaffoldPill>{meta.group}</ScaffoldPill>
            <ScaffoldPill>{meta.key}</ScaffoldPill>
            <ScaffoldPill tone="warning">preview:{preview}</ScaffoldPill>
            <ScaffoldPill>{sessionMode}</ScaffoldPill>
          </div>
          <div>
            <CardTitle className="text-2xl">{entry.title}</CardTitle>
            <CardDescription className="mt-2">{entry.description}</CardDescription>
          </div>
          <div className="grid gap-3 md:grid-cols-4">
            <InfoTile label="订单" value={orderId} />
            <InfoTile label="查看权限" value={meta.viewPermission} />
            <InfoTile label="主操作权限" value={formatList(entry.primaryPermissions)} />
            <InfoTile label="幂等头" value="X-Idempotency-Key" />
          </div>
          <PreviewStateControls />
        </div>
      </Card>

      <Card>
        <div className="space-y-4">
          <CardTitle>当前主体 / 角色 / 租户 / 作用域</CardTitle>
          <IdentityGrid subject={subject} />
          <div className="rounded-[24px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-4 text-sm leading-6 text-[var(--warning-ink)]">
            交付动作会写入审计；下载和对象类页面只展示受控 ticket、Hash 与摘要，不暴露真实对象路径。
          </div>
          <div className="space-y-2">
            <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
              API 绑定
            </div>
            {entry.apiBindings.map((binding) => (
              <div
                key={binding}
                className="rounded-2xl bg-black/[0.04] px-4 py-3 text-xs text-[var(--ink-soft)]"
              >
                {binding}
              </div>
            ))}
          </div>
        </div>
      </Card>
    </motion.section>
  );
}

function IdentityGrid({ subject }: { subject: SessionSubject | null }) {
  return (
    <div className="grid gap-3">
      <InfoTile label="主体" value={subject?.display_name ?? subject?.login_id ?? subject?.user_id ?? "未登录"} />
      <InfoTile label="角色" value={subject?.roles?.join(" / ") ?? "无"} />
      <InfoTile label="租户/组织" value={readSubjectTenant(subject) || "未返回"} />
      <InfoTile label="作用域" value={subject?.auth_context_level ?? "未返回"} />
    </div>
  );
}

function OrderSummary({
  order,
  lifecycle,
  entry,
  expectedEntry,
  isSkuMatched,
  orderId,
}: {
  order: NonNullable<ReturnType<typeof unwrapOrderDetail>>;
  lifecycle: ReturnType<typeof unwrapLifecycle>;
  entry: DeliveryEntry;
  expectedEntry: DeliveryEntry | null;
  isSkuMatched: boolean;
  orderId: string;
}) {
  const lifecycleDelivery = lifecycle?.delivery;
  const relationDelivery = order.relations.deliveries[0];
  const deliveryStatus =
    lifecycleDelivery?.current_status ??
    relationDelivery?.current_status ??
    order.delivery_status;
  const deliveryReceiptHash =
    lifecycleDelivery?.receipt_hash ?? relationDelivery?.receipt_hash ?? null;
  const deliveryCommitHash =
    lifecycleDelivery?.storage_gateway?.integrity.delivery_commit_hash ??
    relationDelivery?.delivery_commit_hash ??
    null;
  const skuType = getOrderSkuType(order);

  return (
    <section className="grid gap-4 lg:grid-cols-[1.2fr_0.8fr]">
      <Card>
        <div className="space-y-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <CardTitle>订单交付状态</CardTitle>
              <CardDescription>
                订单主状态、分层状态和 SKU 真值均来自 `platform-core`，前端不重编排主状态。
              </CardDescription>
            </div>
            <Badge>{skuType || "SKU 未返回"}</Badge>
          </div>
          <div className="grid gap-3 md:grid-cols-4">
            <InfoTile label="主状态" value={orderStatusLabel(order.current_state)} />
            <InfoTile label="支付" value={order.payment_status} />
            <InfoTile label="交付" value={order.delivery_status} />
            <InfoTile label="验收" value={order.acceptance_status} />
          </div>
          <div className="grid gap-3 md:grid-cols-3">
            <InfoTile label="商品" value={order.product_id} />
            <InfoTile label="SKU" value={`${skuDisplayName(skuType)} / ${order.sku_id}`} />
            <InfoTile
              label="官方入口"
              value={expectedEntry ? expectedEntry.title : "未匹配到交付入口"}
            />
          </div>
          {!isSkuMatched ? (
            <div className="rounded-[24px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-4 text-sm text-[var(--warning-ink)]">
              当前页面是 {entry.title}，但订单 SKU {skuType} 应进入{" "}
              {expectedEntry ? (
                <Link
                  className="font-semibold underline"
                  href={deliveryRouteForEntry(expectedEntry, orderId) as Route}
                >
                  {expectedEntry.title}
                </Link>
              ) : (
                "未配置入口"
              )}
              ；页面不会把 SKU 强行并回大类。
            </div>
          ) : null}
        </div>
      </Card>

      <Card>
        <div className="space-y-4">
          <CardTitle>链路与投影承接</CardTitle>
          <div className="grid gap-3">
            <InfoTile label="request_id" value="由每次 platform-core 响应错误或审计记录返回" />
            <InfoTile label="tx_hash" value="当前交付响应未返回 tx_hash，页面显式承接为未返回" />
            <InfoTile
              label="链状态"
              value={deliveryReceiptHash ?? deliveryCommitHash ?? "未返回链摘要"}
            />
            <InfoTile
              label="投影状态"
              value={deliveryStatus}
            />
          </div>
        </div>
      </Card>
    </section>
  );
}

function DeliveryEntryGrid({
  orderId,
  activeKind,
  skuType,
}: {
  orderId: string;
  activeKind: DeliveryRouteKind;
  skuType: string;
}) {
  return (
    <Card>
      <div className="space-y-4">
        <CardTitle>标准 SKU 交付入口</CardTitle>
        <CardDescription>
          八个标准 SKU 显式映射到七类交付入口，`SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 不并回文件或 API 大类。
        </CardDescription>
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          {DELIVERY_ENTRIES.map((entry) => {
            const active = entry.kind === activeKind;
            const matched = entry.supportedSkus.includes(skuType as never);
            return (
              <Link
                key={entry.kind}
                href={deliveryRouteForEntry(entry, orderId) as Route}
                className={cn(
                  "rounded-[24px] border p-4 transition hover:-translate-y-0.5 hover:bg-white",
                  active
                    ? "border-[var(--accent-strong)] bg-[var(--accent-soft)]"
                    : "border-black/10 bg-white/70",
                )}
              >
                <div className="flex items-center justify-between gap-3">
                  <div className="font-semibold text-[var(--ink-strong)]">
                    {entry.shortTitle}
                  </div>
                  {matched ? <Badge>当前 SKU</Badge> : null}
                </div>
                <div className="mt-2 text-xs leading-5 text-[var(--ink-soft)]">
                  {entry.supportedSkus.join(" / ")}
                </div>
              </Link>
            );
          })}
        </div>
      </div>
    </Card>
  );
}

function BranchReadState({
  kind,
  shareData,
  shareError,
  subscriptionData,
  subscriptionError,
  usageData,
  usageError,
  queryRunsData,
  queryRunsError,
}: {
  kind: DeliveryRouteKind;
  shareData: ReturnType<typeof unwrapShareGrantList>;
  shareError: unknown;
  subscriptionData: ReturnType<typeof unwrapRevisionSubscription>;
  subscriptionError: unknown;
  usageData: ReturnType<typeof unwrapApiUsageLog>;
  usageError: unknown;
  queryRunsData: ReturnType<typeof unwrapQueryRuns>;
  queryRunsError: unknown;
}) {
  if (kind === "share") {
    return (
      <ReadStateCard
        title="共享授权记录"
        error={shareError}
        empty={!shareData?.grants.length}
        emptyText="当前订单暂无共享授权记录。"
      >
        {shareData?.grants.slice(0, 3).map((grant) => (
          <InfoRow
            key={grant.data_share_grant_id}
            label={`${grant.grant_status} / ${grant.share_protocol}`}
            value={`${grant.recipient_ref} / locator=${maskSecret(grant.access_locator)}`}
          />
        ))}
      </ReadStateCard>
    );
  }

  if (kind === "subscription") {
    return (
      <ReadStateCard
        title="订阅状态"
        error={subscriptionError}
        empty={!subscriptionData}
        emptyText="当前订单暂无版本订阅记录。"
      >
        {subscriptionData ? (
          <>
            <InfoRow label="订阅状态" value={subscriptionData.subscription_status} />
            <InfoRow label="周期/渠道" value={`${subscriptionData.cadence} / ${subscriptionData.delivery_channel}`} />
            <InfoRow label="当前版本" value={String(subscriptionData.current_version_no)} />
          </>
        ) : null}
      </ReadStateCard>
    );
  }

  if (kind === "api") {
    return (
      <ReadStateCard
        title="API 调用摘要"
        error={usageError}
        empty={!usageData}
        emptyText="API 尚未开通或暂无调用日志。"
      >
        {usageData ? (
          <>
            <InfoRow label="应用" value={`${usageData.app.app_name} / ${usageData.app.credential_status}`} />
            <InfoRow label="调用" value={`${usageData.summary.total_calls} total / ${usageData.summary.successful_calls} success`} />
            <InfoRow label="计费用量" value={usageData.summary.total_usage_units} />
          </>
        ) : null}
      </ReadStateCard>
    );
  }

  if (kind === "template-query") {
    return (
      <ReadStateCard
        title="查询运行记录"
        error={queryRunsError}
        empty={!queryRunsData?.query_runs.length}
        emptyText="模板已授权后会在这里展示查询运行与结果摘要。"
      >
        {queryRunsData?.query_runs.slice(0, 3).map((run) => (
          <InfoRow
            key={run.query_run_id}
            label={`${run.query_template_name} / ${run.status}`}
            value={`rows=${run.result_row_count} / result=${maskSecret(run.result_object_uri)}`}
          />
        ))}
      </ReadStateCard>
    );
  }

  return null;
}

function DeliveryActionPanel({
  kind,
  orderId,
  onChanged,
}: {
  kind: DeliveryRouteKind;
  orderId: string;
  onChanged: () => void;
}) {
  if (kind === "file" || kind === "report" || kind === "api") {
    return <CommitDeliveryForm kind={kind} orderId={orderId} onChanged={onChanged} />;
  }
  if (kind === "subscription") {
    return <RevisionSubscriptionForm orderId={orderId} onChanged={onChanged} />;
  }
  if (kind === "share") {
    return <ShareGrantForm orderId={orderId} onChanged={onChanged} />;
  }
  if (kind === "template-query") {
    return <TemplateGrantForm orderId={orderId} onChanged={onChanged} />;
  }
  return <SandboxWorkspaceForm orderId={orderId} onChanged={onChanged} />;
}

function CommitDeliveryForm({
  kind,
  orderId,
  onChanged,
}: {
  kind: "file" | "api" | "report";
  orderId: string;
  onChanged: () => void;
}) {
  const queryClient = useQueryClient();
  const [result, setResult] = useState<CommitDeliveryResult | null>(null);
  const [ticket, setTicket] = useState<DownloadTicketData | null>(null);
  const form = useForm<CommitDeliveryFormValues>({
    resolver: zodResolver(commitDeliveryFormSchema),
    defaultValues: defaultCommitDeliveryValues(kind),
  });
  const mutation = useMutation({
    mutationFn: async (values: CommitDeliveryFormValues) => {
      const response = await sdk.delivery.commitOrderDelivery(
        { id: orderId },
        buildCommitDeliveryRequest(kind, values),
        { idempotencyKey: values.idempotency_key },
      );
      const data = unwrapCommitDelivery(response);
      if (!data) {
        throw new Error("DELIVERY_STATUS_INVALID: commitOrderDelivery 未返回交付数据");
      }
      return data;
    },
    onSuccess: (data) => {
      setResult(data);
      setTicket(null);
      form.setValue("idempotency_key", createDeliveryIdempotencyKey(kind));
      onChanged();
      void queryClient.invalidateQueries({ queryKey: ["portal", "delivery", orderId] });
    },
  });
  const ticketMutation = useMutation({
    mutationFn: async () => {
      const response = await sdk.delivery.issueDownloadTicket({ id: orderId });
      const data = unwrapDownloadTicket(response);
      if (!data) {
        throw new Error("DELIVERY_STATUS_INVALID: download-ticket 未返回票据");
      }
      return data;
    },
    onSuccess: setTicket,
  });

  return (
    <Card>
      <FormHeader
        icon={kind === "api" ? <KeyRound /> : <FileArchive />}
        title={kind === "api" ? "API 开通表单" : kind === "report" ? "报告交付表单" : "文件交付表单"}
        description="提交会经 SDK 调用 `POST /api/v1/orders/{id}/deliver`，并透传 `X-Idempotency-Key`。"
      />
      <form className="mt-5 space-y-5" onSubmit={form.handleSubmit((values) => mutation.mutate(values))}>
        {kind !== "api" ? (
          <div className="grid gap-4 md:grid-cols-2">
            <TextField label="object_uri（受控提交，不在结果区暴露真实路径）" error={form.formState.errors.object_uri?.message}>
              <Input {...form.register("object_uri")} />
            </TextField>
            <TextField label="content_type" error={form.formState.errors.content_type?.message}>
              <Input {...form.register("content_type")} />
            </TextField>
            <TextField label="size_bytes" error={form.formState.errors.size_bytes?.message}>
              <Input {...form.register("size_bytes")} />
            </TextField>
            <TextField label="content_hash" error={form.formState.errors.content_hash?.message}>
              <Input {...form.register("content_hash")} />
            </TextField>
            <TextField label="encryption_algo" error={form.formState.errors.encryption_algo?.message}>
              <Input {...form.register("encryption_algo")} />
            </TextField>
            <TextField label="key_control_mode" error={form.formState.errors.key_control_mode?.message}>
              <Input {...form.register("key_control_mode")} />
            </TextField>
            <TextField label="expire_at" error={form.formState.errors.expire_at?.message}>
              <Input {...form.register("expire_at")} />
            </TextField>
            <TextField label="download_limit" error={form.formState.errors.download_limit?.message}>
              <Input {...form.register("download_limit")} />
            </TextField>
            {kind === "report" ? (
              <TextField label="report_type" error={form.formState.errors.report_type?.message}>
                <Input {...form.register("report_type")} />
              </TextField>
            ) : null}
          </div>
        ) : (
          <div className="grid gap-4 md:grid-cols-2">
            <TextField label="app_id（可选，留空则后端按订单创建/绑定）" error={form.formState.errors.app_id?.message}>
              <Input {...form.register("app_id")} />
            </TextField>
            <TextField label="app_name" error={form.formState.errors.app_name?.message}>
              <Input {...form.register("app_name")} />
            </TextField>
            <TextField label="app_type" error={form.formState.errors.app_type?.message}>
              <Input {...form.register("app_type")} />
            </TextField>
            <TextField label="client_id" error={form.formState.errors.client_id?.message}>
              <Input {...form.register("client_id")} />
            </TextField>
            <JsonField label="quota_json" error={form.formState.errors.quota_json?.message}>
              <Textarea {...form.register("quota_json")} />
            </JsonField>
            <JsonField label="rate_limit_json" error={form.formState.errors.rate_limit_json?.message}>
              <Textarea {...form.register("rate_limit_json")} />
            </JsonField>
            <TextField label="upstream_mode" error={form.formState.errors.upstream_mode?.message}>
              <Input {...form.register("upstream_mode")} />
            </TextField>
          </div>
        )}
        <div className="grid gap-4 md:grid-cols-2">
          <TextField label="delivery_commit_hash" error={form.formState.errors.delivery_commit_hash?.message}>
            <Input {...form.register("delivery_commit_hash")} />
          </TextField>
          <TextField label="receipt_hash" error={form.formState.errors.receipt_hash?.message}>
            <Input {...form.register("receipt_hash")} />
          </TextField>
          <TextField label="X-Idempotency-Key" error={form.formState.errors.idempotency_key?.message}>
            <Input {...form.register("idempotency_key")} />
          </TextField>
          <JsonField label="metadata_json" error={form.formState.errors.metadata_json?.message}>
            <Textarea {...form.register("metadata_json")} />
          </JsonField>
        </div>
        <ConfirmationFields form={form} />
        <FormActions
          isPending={mutation.isPending}
          primaryLabel={kind === "api" ? "开通 API" : "提交交付"}
          onResetKey={() => form.setValue("idempotency_key", createDeliveryIdempotencyKey(kind))}
        />
      </form>
      <MutationMessage error={mutation.error} />
      {result ? <CommitResultPanel result={result} /> : null}
      {kind === "file" && result ? (
        <div className="mt-4">
          <Button
            type="button"
            variant="secondary"
            onClick={() => ticketMutation.mutate()}
            disabled={ticketMutation.isPending}
          >
            {ticketMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <Download className="size-4" />}
            签发下载票据
          </Button>
          <MutationMessage error={ticketMutation.error} />
          {ticket ? <DownloadTicketPanel ticket={ticket} /> : null}
        </div>
      ) : null}
    </Card>
  );
}

function RevisionSubscriptionForm({
  orderId,
  onChanged,
}: {
  orderId: string;
  onChanged: () => void;
}) {
  const [result, setResult] = useState<ReturnType<typeof unwrapRevisionSubscriptionMutation>>(null);
  const form = useForm<RevisionSubscriptionFormValues>({
    resolver: zodResolver(revisionSubscriptionFormSchema),
    defaultValues: defaultRevisionSubscriptionValues(),
  });
  const mutation = useMutation({
    mutationFn: async (values: RevisionSubscriptionFormValues) => {
      const response = await sdk.delivery.manageRevisionSubscription(
        { id: orderId },
        buildRevisionSubscriptionRequest(values),
        { idempotencyKey: values.idempotency_key },
      );
      const data = unwrapRevisionSubscriptionMutation(response);
      if (!data) {
        throw new Error("DELIVERY_STATUS_INVALID: subscriptions 未返回数据");
      }
      return data;
    },
    onSuccess: (data) => {
      setResult(data);
      form.setValue("idempotency_key", createDeliveryIdempotencyKey("subscription"));
      onChanged();
    },
  });

  return (
    <GenericFormCard
      icon={<GitBranch />}
      title="版本订阅管理"
      description="提交会调用 FILE_SUB 订阅写接口，后端记录幂等键与审计 metadata。"
      form={form}
      mutation={mutation}
      result={result}
      onSubmit={(values) => mutation.mutate(values)}
      onResetKey={() => form.setValue("idempotency_key", createDeliveryIdempotencyKey("subscription"))}
    >
      <div className="grid gap-4 md:grid-cols-2">
        <TextField label="cadence" error={form.formState.errors.cadence?.message}>
          <select className="h-10 rounded-full border border-black/10 bg-white px-4 text-sm" {...form.register("cadence")}>
            <option value="weekly">weekly</option>
            <option value="monthly">monthly</option>
            <option value="quarterly">quarterly</option>
            <option value="yearly">yearly</option>
          </select>
        </TextField>
        <TextField label="delivery_channel" error={form.formState.errors.delivery_channel?.message}>
          <select className="h-10 rounded-full border border-black/10 bg-white px-4 text-sm" {...form.register("delivery_channel")}>
            <option value="file_ticket">file_ticket</option>
          </select>
        </TextField>
        <TextField label="start_version_no" error={form.formState.errors.start_version_no?.message}>
          <Input {...form.register("start_version_no")} />
        </TextField>
        <TextField label="last_delivered_version_no" error={form.formState.errors.last_delivered_version_no?.message}>
          <Input {...form.register("last_delivered_version_no")} />
        </TextField>
        <TextField label="next_delivery_at" error={form.formState.errors.next_delivery_at?.message}>
          <Input {...form.register("next_delivery_at")} />
        </TextField>
        <JsonField label="metadata_json" error={form.formState.errors.metadata_json?.message}>
          <Textarea {...form.register("metadata_json")} />
        </JsonField>
      </div>
    </GenericFormCard>
  );
}

function ShareGrantForm({ orderId, onChanged }: { orderId: string; onChanged: () => void }) {
  const [result, setResult] = useState<ReturnType<typeof unwrapShareGrant>>(null);
  const form = useForm<ShareGrantFormValues>({
    resolver: zodResolver(shareGrantFormSchema),
    defaultValues: defaultShareGrantValues(),
  });
  const mutation = useMutation({
    mutationFn: async (values: ShareGrantFormValues) => {
      const response = await sdk.delivery.manageShareGrant(
        { id: orderId },
        buildShareGrantRequest(values),
        { idempotencyKey: values.idempotency_key },
      );
      const data = unwrapShareGrant(response);
      if (!data) {
        throw new Error("DELIVERY_STATUS_INVALID: share-grants 未返回数据");
      }
      return data;
    },
    onSuccess: (data) => {
      setResult(data);
      form.setValue("idempotency_key", createDeliveryIdempotencyKey("share"));
      onChanged();
    },
  });

  return (
    <GenericFormCard
      icon={<Share2 />}
      title="只读共享授权"
      description="授权范围、协议、接收方和有效期来自订单范围；撤权同样走正式 API 和审计。"
      form={form}
      mutation={mutation}
      result={result}
      onSubmit={(values) => mutation.mutate(values)}
      onResetKey={() => form.setValue("idempotency_key", createDeliveryIdempotencyKey("share"))}
    >
      <div className="grid gap-4 md:grid-cols-2">
        <TextField label="operation" error={form.formState.errors.operation?.message}>
          <select className="h-10 rounded-full border border-black/10 bg-white px-4 text-sm" {...form.register("operation")}>
            <option value="grant">grant</option>
            <option value="revoke">revoke</option>
          </select>
        </TextField>
        <TextField label="recipient_ref" error={form.formState.errors.recipient_ref?.message}>
          <Input {...form.register("recipient_ref")} />
        </TextField>
        <TextField label="subscriber_ref" error={form.formState.errors.subscriber_ref?.message}>
          <Input {...form.register("subscriber_ref")} />
        </TextField>
        <TextField label="share_protocol" error={form.formState.errors.share_protocol?.message}>
          <Input {...form.register("share_protocol")} />
        </TextField>
        <TextField label="access_locator（结果区遮蔽显示）" error={form.formState.errors.access_locator?.message}>
          <Input {...form.register("access_locator")} />
        </TextField>
        <TextField label="expires_at" error={form.formState.errors.expires_at?.message}>
          <Input {...form.register("expires_at")} />
        </TextField>
        <TextField label="receipt_hash" error={form.formState.errors.receipt_hash?.message}>
          <Input {...form.register("receipt_hash")} />
        </TextField>
        <TextField label="asset_object_id" error={form.formState.errors.asset_object_id?.message}>
          <Input {...form.register("asset_object_id")} />
        </TextField>
        <JsonField label="scope_json" error={form.formState.errors.scope_json?.message}>
          <Textarea {...form.register("scope_json")} />
        </JsonField>
        <JsonField label="metadata_json" error={form.formState.errors.metadata_json?.message}>
          <Textarea {...form.register("metadata_json")} />
        </JsonField>
      </div>
    </GenericFormCard>
  );
}

function TemplateGrantForm({ orderId, onChanged }: { orderId: string; onChanged: () => void }) {
  const [result, setResult] = useState<ReturnType<typeof unwrapTemplateGrant>>(null);
  const form = useForm<TemplateGrantFormValues>({
    resolver: zodResolver(templateGrantFormSchema),
    defaultValues: defaultTemplateGrantValues(),
  });
  const mutation = useMutation({
    mutationFn: async (values: TemplateGrantFormValues) => {
      const response = await sdk.delivery.manageTemplateGrant(
        { id: orderId },
        buildTemplateGrantRequest(values),
        { idempotencyKey: values.idempotency_key },
      );
      const data = unwrapTemplateGrant(response);
      if (!data) {
        throw new Error("DELIVERY_STATUS_INVALID: template-grants 未返回数据");
      }
      return data;
    },
    onSuccess: (data) => {
      setResult(data);
      form.setValue("idempotency_key", createDeliveryIdempotencyKey("template-query"));
      onChanged();
    },
  });

  return (
    <GenericFormCard
      icon={<TerminalSquare />}
      title="模板查询授权"
      description="授权白名单模板、执行规则、输出边界和 run quota，不直接执行查询。"
      form={form}
      mutation={mutation}
      result={result}
      onSubmit={(values) => mutation.mutate(values)}
      onResetKey={() => form.setValue("idempotency_key", createDeliveryIdempotencyKey("template-query"))}
    >
      <div className="grid gap-4 md:grid-cols-2">
        <TextField label="query_surface_id" error={form.formState.errors.query_surface_id?.message}>
          <Input {...form.register("query_surface_id")} />
        </TextField>
        <TextField label="allowed_template_ids（逗号分隔）" error={form.formState.errors.allowed_template_ids?.message}>
          <Input {...form.register("allowed_template_ids")} />
        </TextField>
        <TextField label="template_query_grant_id" error={form.formState.errors.template_query_grant_id?.message}>
          <Input {...form.register("template_query_grant_id")} />
        </TextField>
        <TextField label="asset_object_id" error={form.formState.errors.asset_object_id?.message}>
          <Input {...form.register("asset_object_id")} />
        </TextField>
        <TextField label="environment_id" error={form.formState.errors.environment_id?.message}>
          <Input {...form.register("environment_id")} />
        </TextField>
        <TextField label="template_type" error={form.formState.errors.template_type?.message}>
          <Input {...form.register("template_type")} />
        </TextField>
        <JsonField label="execution_rule_snapshot" error={form.formState.errors.execution_rule_snapshot?.message}>
          <Textarea {...form.register("execution_rule_snapshot")} />
        </JsonField>
        <JsonField label="output_boundary_json" error={form.formState.errors.output_boundary_json?.message}>
          <Textarea {...form.register("output_boundary_json")} />
        </JsonField>
        <JsonField label="run_quota_json" error={form.formState.errors.run_quota_json?.message}>
          <Textarea {...form.register("run_quota_json")} />
        </JsonField>
      </div>
    </GenericFormCard>
  );
}

function SandboxWorkspaceForm({ orderId, onChanged }: { orderId: string; onChanged: () => void }) {
  const [result, setResult] = useState<ReturnType<typeof unwrapSandboxWorkspace>>(null);
  const form = useForm<SandboxWorkspaceFormValues>({
    resolver: zodResolver(sandboxWorkspaceFormSchema),
    defaultValues: defaultSandboxWorkspaceValues(),
  });
  const mutation = useMutation({
    mutationFn: async (values: SandboxWorkspaceFormValues) => {
      const response = await sdk.delivery.manageSandboxWorkspace(
        { id: orderId },
        buildSandboxWorkspaceRequest(values),
        { idempotencyKey: values.idempotency_key },
      );
      const data = unwrapSandboxWorkspace(response);
      if (!data) {
        throw new Error("DELIVERY_STATUS_INVALID: sandbox-workspaces 未返回数据");
      }
      return data;
    },
    onSuccess: (data) => {
      setResult(data);
      form.setValue("idempotency_key", createDeliveryIdempotencyKey("sandbox"));
      onChanged();
    },
  });

  return (
    <GenericFormCard
      icon={<ServerCog />}
      title="沙箱工作区开通"
      description="开通受控工作区、席位、会话有效期与导出策略，SBX_STD 成功后进入自动验收路径。"
      form={form}
      mutation={mutation}
      result={result}
      onSubmit={(values) => mutation.mutate(values)}
      onResetKey={() => form.setValue("idempotency_key", createDeliveryIdempotencyKey("sandbox"))}
    >
      <div className="grid gap-4 md:grid-cols-2">
        <TextField label="query_surface_id" error={form.formState.errors.query_surface_id?.message}>
          <Input {...form.register("query_surface_id")} />
        </TextField>
        <TextField label="workspace_name" error={form.formState.errors.workspace_name?.message}>
          <Input {...form.register("workspace_name")} />
        </TextField>
        <TextField label="seat_user_id" error={form.formState.errors.seat_user_id?.message}>
          <Input {...form.register("seat_user_id")} />
        </TextField>
        <TextField label="expire_at" error={form.formState.errors.expire_at?.message}>
          <Input {...form.register("expire_at")} />
        </TextField>
        <TextField label="clean_room_mode" error={form.formState.errors.clean_room_mode?.message}>
          <Input {...form.register("clean_room_mode")} />
        </TextField>
        <TextField label="data_residency_mode" error={form.formState.errors.data_residency_mode?.message}>
          <Input {...form.register("data_residency_mode")} />
        </TextField>
        <JsonField label="export_policy_json" error={form.formState.errors.export_policy_json?.message}>
          <Textarea {...form.register("export_policy_json")} />
        </JsonField>
      </div>
    </GenericFormCard>
  );
}

function GenericFormCard<TValues extends { idempotency_key: string; confirm_scope: boolean; confirm_audit: boolean }>({
  icon,
  title,
  description,
  form,
  mutation,
  result,
  children,
  onSubmit,
  onResetKey,
}: {
  icon: ReactNode;
  title: string;
  description: string;
  form: UseFormReturn<TValues>;
  mutation: { isPending: boolean; error: unknown };
  result: unknown;
  children: ReactNode;
  onSubmit: (values: TValues) => void;
  onResetKey: () => void;
}) {
  return (
    <Card>
      <FormHeader icon={icon} title={title} description={description} />
      <form className="mt-5 space-y-5" onSubmit={form.handleSubmit(onSubmit)}>
        {children}
        <TextField label="X-Idempotency-Key" error={form.formState.errors.idempotency_key?.message as string | undefined}>
          <Input {...form.register("idempotency_key" as Path<TValues>)} />
        </TextField>
        <ConfirmationFields form={form} />
        <FormActions
          isPending={mutation.isPending}
          primaryLabel="提交"
          onResetKey={onResetKey}
        />
      </form>
      <MutationMessage error={mutation.error} />
      {result ? <GenericResultPanel result={result} /> : null}
    </Card>
  );
}

function ConfirmationFields<TValues extends { confirm_scope: boolean; confirm_audit: boolean }>({
  form,
}: {
  form: UseFormReturn<TValues>;
}) {
  return (
    <div className="grid gap-3 rounded-[24px] bg-black/[0.04] p-4 text-sm">
      <label className="flex gap-3">
        <input type="checkbox" {...form.register("confirm_scope" as Path<TValues>)} />
        <span>确认交付/授权范围来自订单快照、SKU 真值和后端冻结契约。</span>
      </label>
      {form.formState.errors.confirm_scope?.message ? (
        <p className="text-xs text-[var(--danger-ink)]">
          {form.formState.errors.confirm_scope.message as string}
        </p>
      ) : null}
      <label className="flex gap-3">
        <input type="checkbox" {...form.register("confirm_audit" as Path<TValues>)} />
        <span>确认该关键动作会写入审计留痕，并在错误态回显统一错误码与 request_id。</span>
      </label>
      {form.formState.errors.confirm_audit?.message ? (
        <p className="text-xs text-[var(--danger-ink)]">
          {form.formState.errors.confirm_audit.message as string}
        </p>
      ) : null}
    </div>
  );
}

function FormHeader({
  icon,
  title,
  description,
}: {
  icon: ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="flex items-start gap-4">
      <div className="rounded-2xl bg-[var(--accent-soft)] p-3 text-[var(--accent-strong)] [&_svg]:size-5">
        {icon}
      </div>
      <div>
        <CardTitle>{title}</CardTitle>
        <CardDescription className="mt-1">{description}</CardDescription>
      </div>
    </div>
  );
}

function FormActions({
  isPending,
  primaryLabel,
  onResetKey,
}: {
  isPending: boolean;
  primaryLabel: string;
  onResetKey: () => void;
}) {
  return (
    <div className="flex flex-wrap gap-3">
      <Button type="submit" disabled={isPending}>
        {isPending ? <LoaderCircle className="size-4 animate-spin" /> : <CheckCircle2 className="size-4" />}
        {primaryLabel}
      </Button>
      <Button type="button" variant="secondary" onClick={onResetKey}>
        <RefreshCcw className="size-4" />
        重新生成幂等键
      </Button>
    </div>
  );
}

function TextField({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <label className="grid gap-2 text-sm text-[var(--ink-soft)]">
      <span>{label}</span>
      {children}
      {error ? <span className="text-xs text-[var(--danger-ink)]">{error}</span> : null}
    </label>
  );
}

function JsonField(props: Parameters<typeof TextField>[0]) {
  return <TextField {...props} />;
}

function CommitResultPanel({ result }: { result: CommitDeliveryResult }) {
  return (
    <ResultPanel title="交付提交结果">
      <InfoRow label="delivery_id" value={result.delivery_id} />
      <InfoRow label="branch" value={result.branch} />
      <InfoRow label="状态" value={`${result.current_state} / ${result.delivery_status} / ${result.acceptance_status}`} />
      <InfoRow label="object_id" value={result.object_id ?? "未返回"} />
      <InfoRow label="envelope_id" value={result.envelope_id ?? "未返回"} />
      <InfoRow label="delivery_commit_hash" value={result.delivery_commit_hash ?? "未返回"} />
      <InfoRow label="receipt_hash" value={result.receipt_hash ?? "未返回"} />
      {result.api_key || result.api_key_hint ? (
        <InfoRow label="api_key" value={maskSecret(result.api_key ?? result.api_key_hint)} />
      ) : null}
      {result.app_name ? <InfoRow label="app" value={`${result.app_name} / ${result.credential_status ?? "unknown"}`} /> : null}
      <div className="rounded-[20px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-3 text-xs leading-5 text-[var(--warning-ink)]">
        后端响应中的 bucket/object_key 不在页面展示；下载必须通过受控 ticket 与 gateway。
      </div>
    </ResultPanel>
  );
}

function DownloadTicketPanel({ ticket }: { ticket: DownloadTicketData }) {
  return (
    <ResultPanel title="下载票据状态">
      <InfoRow label="ticket_id" value={ticket.ticket_id} />
      <InfoRow label="download_token" value={maskSecret(ticket.download_token)} />
      <InfoRow label="ticket_status" value={ticket.ticket_status} />
      <InfoRow label="expire_at" value={ticket.expire_at} />
      <InfoRow label="下载次数" value={`${ticket.download_count}/${ticket.download_limit}，剩余 ${ticket.remaining_downloads}`} />
      <InfoRow label="delivery_commit_hash" value={ticket.delivery_commit_hash} />
      <div className="rounded-[20px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-3 text-xs leading-5 text-[var(--warning-ink)]">
        真实对象路径、bucket 和 object_key 已遮蔽；页面只显示 ticket 元数据与 Hash 校验摘要。
      </div>
    </ResultPanel>
  );
}

function GenericResultPanel({ result }: { result: unknown }) {
  const record = result && typeof result === "object" ? result as Record<string, unknown> : {};
  return (
    <ResultPanel title="接口返回摘要">
      {Object.entries(record)
        .filter(([key]) => !["access_locator", "api_key", "object_key", "bucket_name", "result_object_uri"].includes(key))
        .slice(0, 12)
        .map(([key, value]) => (
          <InfoRow key={key} label={key} value={formatValue(value)} />
        ))}
    </ResultPanel>
  );
}

function ResultPanel({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="mt-5 rounded-[28px] border border-[var(--success-ring)] bg-[var(--success-soft)] p-4">
      <div className="mb-3 flex items-center gap-2 font-semibold text-[var(--success-ink)]">
        <ClipboardCheck className="size-4" />
        {title}
      </div>
      <div className="grid gap-2 text-sm">{children}</div>
    </div>
  );
}

function MutationMessage({ error }: { error: unknown }) {
  if (!error) {
    return null;
  }
  return (
    <div className="mt-4 rounded-[24px] border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-4 text-sm text-[var(--danger-ink)]">
      {formatDeliveryError(error)}
    </div>
  );
}

function ReadStateCard({
  title,
  error,
  empty,
  emptyText,
  children,
}: {
  title: string;
  error: unknown;
  empty: boolean;
  emptyText: string;
  children: ReactNode;
}) {
  return (
    <Card>
      <div className="space-y-3">
        <CardTitle>{title}</CardTitle>
        {error ? (
          <CardDescription>{formatDeliveryError(error)}</CardDescription>
        ) : empty ? (
          <CardDescription>{emptyText}</CardDescription>
        ) : (
          <div className="grid gap-2 text-sm">{children}</div>
        )}
      </div>
    </Card>
  );
}

function PermissionPanel({
  entry,
  subject,
  sessionMode,
}: {
  entry: DeliveryEntry;
  subject: SessionSubject | null;
  sessionMode: string;
}) {
  return (
    <StateCard
      icon={<LockKeyhole />}
      title="交付中心权限态"
      message={`需要权限：trade.order.read；主操作权限：${entry.primaryPermissions.join(" / ")}；当前会话 ${sessionMode}，角色 ${subject?.roles?.join(" / ") ?? "无"}`}
      tone="warning"
    />
  );
}

function ActionPermissionPanel({
  entry,
  subject,
}: {
  entry: DeliveryEntry;
  subject: SessionSubject | null;
}) {
  return (
    <StateCard
      icon={<ShieldCheck />}
      title="主按钮权限不足"
      message={`当前角色 ${subject?.roles?.join(" / ") ?? "无"} 只能查看交付状态，不能执行 ${entry.primaryPermissions.join(" / ")}。`}
      tone="warning"
    />
  );
}

function SkuMismatchPanel({
  currentEntry,
  expectedEntry,
  skuType,
  orderId,
}: {
  currentEntry: DeliveryEntry;
  expectedEntry: DeliveryEntry | null;
  skuType: string;
  orderId: string;
}) {
  return (
    <StateCard
      icon={<AlertTriangle />}
      title="SKU 与交付入口不匹配"
      message={`当前 ${currentEntry.title} 不处理 ${skuType}；应进入 ${expectedEntry?.title ?? "未配置入口"}。`}
      tone="warning"
      action={
        expectedEntry ? (
          <Button asChild variant="secondary">
            <Link href={deliveryRouteForEntry(expectedEntry, orderId) as Route}>进入正确入口</Link>
          </Button>
        ) : null
      }
    />
  );
}

function LoadingPanel({ entry }: { entry: DeliveryEntry }) {
  return (
    <StateCard
      icon={<LoaderCircle className="animate-spin" />}
      title={`${entry.title}加载态`}
      message="正在读取 auth/me、订单详情、生命周期快照和分支交付数据。"
    />
  );
}

function ErrorPanel({ title, message }: { title: string; message: string }) {
  return <StateCard icon={<AlertTriangle />} title={title} message={message} tone="danger" />;
}

function EmptyPanel({ entry }: { entry: DeliveryEntry }) {
  return (
    <StateCard
      icon={<Boxes />}
      title="没有可展示的交付数据"
      message={`${entry.title}未读取到订单或交付记录；页面保留空态、重试入口和 API 绑定说明。`}
    />
  );
}

function DeliveryPreviewState({
  entry,
  orderId,
  preview,
}: {
  entry: DeliveryEntry;
  orderId: string;
  preview: string;
}) {
  if (preview === "forbidden") {
    return (
      <div className="space-y-6">
        <DeliveryHero entry={entry} orderId={orderId} subject={null} sessionMode="preview" preview={preview} />
        <PermissionPanel entry={entry} subject={null} sessionMode="preview" />
      </div>
    );
  }
  if (preview === "empty") {
    return (
      <div className="space-y-6">
        <DeliveryHero entry={entry} orderId={orderId} subject={null} sessionMode="preview" preview={preview} />
        <EmptyPanel entry={entry} />
        <DeliveryEntryGrid orderId={orderId} activeKind={entry.kind} skuType="" />
      </div>
    );
  }
  if (preview === "error") {
    return (
      <div className="space-y-6">
        <DeliveryHero entry={entry} orderId={orderId} subject={null} sessionMode="preview" preview={preview} />
        <ErrorPanel
          title={`${entry.title}错误态`}
          message="DELIVERY_STATUS_INVALID: 页面必须承接 platform-core 统一错误码、request_id 与重试入口。"
        />
      </div>
    );
  }
  return (
    <div className="space-y-6">
      <DeliveryHero entry={entry} orderId={orderId} subject={null} sessionMode="preview" preview={preview} />
      <LoadingPanel entry={entry} />
    </div>
  );
}

function StateCard({
  icon,
  title,
  message,
  tone = "default",
  action,
}: {
  icon: ReactNode;
  title: string;
  message: string;
  tone?: "default" | "warning" | "danger";
  action?: ReactNode;
}) {
  return (
    <Card
      className={cn(
        "min-h-64",
        tone === "warning" && "border-[var(--warning-ring)] bg-[var(--warning-soft)]",
        tone === "danger" && "border-[var(--danger-ring)] bg-[var(--danger-soft)]",
      )}
    >
      <div className="flex min-h-52 flex-col items-center justify-center gap-4 text-center">
        <div className="text-[var(--ink-soft)] [&_svg]:size-8">{icon}</div>
        <CardTitle>{title}</CardTitle>
        <CardDescription className={cn(tone === "warning" && "text-[var(--warning-ink)]", tone === "danger" && "text-[var(--danger-ink)]")}>
          {message}
        </CardDescription>
        {action}
      </div>
    </Card>
  );
}

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-[24px] bg-white/70 p-4">
      <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 break-words text-sm font-medium text-[var(--ink-strong)]">
        {value}
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid gap-1 rounded-2xl bg-white/70 px-4 py-3 md:grid-cols-[180px_1fr]">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="break-words text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}

function formatValue(value: unknown) {
  if (value === null || value === undefined) {
    return "未返回";
  }
  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  return JSON.stringify(value);
}
