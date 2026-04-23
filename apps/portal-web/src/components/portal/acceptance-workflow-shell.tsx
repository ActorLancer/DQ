"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  ArrowRight,
  Ban,
  CheckCircle2,
  ClipboardCheck,
  FileCheck2,
  Fingerprint,
  GitBranch,
  LoaderCircle,
  ShieldCheck,
  XCircle,
} from "lucide-react";
import { motion } from "motion/react";
import type { Route } from "next";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import type { ReactNode } from "react";
import {
  useForm,
  type FieldErrors,
  type UseFormRegisterReturn,
  type UseFormReturn,
} from "react-hook-form";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  ACCEPTANCE_ACTION_ALLOWED_ROLES,
  MANUAL_ACCEPTANCE_SKUS,
  acceptOrderFormSchema,
  buildAcceptOrderRequest,
  buildRejectOrderRequest,
  canOperateAcceptance,
  canReadAcceptance,
  createAcceptanceIdempotencyKey,
  defaultAcceptOrderValues,
  defaultRejectOrderValues,
  formatAcceptanceError,
  getOrderSkuType,
  isDeliveredForAcceptance,
  isManualAcceptanceSku,
  latestDelivery,
  lifecycleRows,
  readSubjectTenant,
  rejectOrderFormSchema,
  skuDisplayName,
  unwrapAcceptOrder,
  unwrapRejectOrder,
  verificationSummary,
  type AcceptOrderFormValues,
  type AcceptanceDecisionResult,
  type OrderDetail,
  type OrderLifecycleSnapshots,
  type RejectOrderFormValues,
  type RejectionDecisionResult,
  type SessionSubject,
} from "@/lib/acceptance-workflow";
import { unwrapLifecycle, unwrapOrderDetail } from "@/lib/order-workflow";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import type { PortalSessionPreview } from "@/lib/session";
import { cn, formatList } from "@/lib/utils";

import { PreviewStateControls, ScaffoldPill, getPreviewState } from "./state-preview";

const sdk = createBrowserSdk();

type AcceptanceWorkflowShellProps = {
  orderId: string;
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

export function AcceptanceWorkflowShell({
  orderId,
  sessionMode,
  initialSubject,
}: AcceptanceWorkflowShellProps) {
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);

  const authQuery = useQuery({
    queryKey: ["portal", "acceptance", orderId, "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest" && preview === "ready",
  });
  const subject = (authQuery.data?.data ?? initialSubject) as SessionSubject | null;
  const canRead = canReadAcceptance(subject);

  const detailQuery = useQuery({
    queryKey: ["portal", "acceptance", orderId, "detail"],
    queryFn: () => sdk.trade.getOrderDetail({ id: orderId }),
    enabled: sessionMode !== "guest" && preview === "ready" && canRead,
  });
  const lifecycleQuery = useQuery({
    queryKey: ["portal", "acceptance", orderId, "lifecycle"],
    queryFn: () => sdk.trade.getOrderLifecycleSnapshots({ id: orderId }),
    enabled: sessionMode !== "guest" && preview === "ready" && canRead,
  });

  const order = unwrapOrderDetail(detailQuery.data);
  const lifecycle = unwrapLifecycle(lifecycleQuery.data);
  const loading = authQuery.isLoading || detailQuery.isLoading || lifecycleQuery.isLoading;
  const error = authQuery.error ?? detailQuery.error ?? lifecycleQuery.error;

  if (preview !== "ready") {
    return <AcceptancePreviewState orderId={orderId} preview={preview} />;
  }

  return (
    <div className="space-y-6">
      <AcceptanceHero
        orderId={orderId}
        subject={subject}
        sessionMode={sessionMode}
      />

      {sessionMode === "guest" || !canRead ? (
        <PermissionPanel subject={subject} sessionMode={sessionMode} />
      ) : loading ? (
        <LoadingPanel />
      ) : error ? (
        <ErrorPanel title="验收页错误态" message={formatAcceptanceError(error)} />
      ) : !order ? (
        <EmptyPanel />
      ) : (
        <>
          <AcceptanceSummary order={order} lifecycle={lifecycle} orderId={orderId} />
          <VerificationPanel order={order} lifecycle={lifecycle} />
          <LifecyclePanel order={order} lifecycle={lifecycle} />
          <AcceptanceActionPanel
            order={order}
            lifecycle={lifecycle}
            orderId={orderId}
            subject={subject}
          />
        </>
      )}
    </div>
  );
}

function AcceptanceHero({
  orderId,
  subject,
  sessionMode,
}: {
  orderId: string;
  subject: SessionSubject | null;
  sessionMode: string;
}) {
  const meta = portalRouteMap.delivery_acceptance;
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      className="relative overflow-hidden rounded-[34px] border border-black/10 bg-[linear-gradient(135deg,#f7f1df_0%,#edf7f2_48%,#e8efff_100%)] p-6 shadow-[0_24px_70px_rgba(31,44,61,0.13)]"
    >
      <div className="absolute -right-12 -top-16 size-56 rounded-full bg-[#f6c75e]/35 blur-3xl" />
      <div className="absolute -bottom-20 left-1/3 size-64 rounded-full bg-[#6bb6a9]/25 blur-3xl" />
      <div className="relative grid gap-6 lg:grid-cols-[1.35fr_0.65fr]">
        <div className="space-y-5">
          <div className="flex flex-wrap gap-2">
            <ScaffoldPill>WEB-011</ScaffoldPill>
            <ScaffoldPill>{meta.viewPermission}</ScaffoldPill>
            {meta.primaryPermissions.map((permission) => (
              <ScaffoldPill key={permission}>{permission}</ScaffoldPill>
            ))}
          </div>
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.32em] text-[var(--accent-strong)]">
              delivery acceptance
            </p>
            <h1 className="mt-3 max-w-4xl text-3xl font-semibold tracking-[-0.05em] text-[var(--ink-strong)] md:text-5xl">
              验收页
            </h1>
            <p className="mt-4 max-w-3xl text-sm leading-7 text-[var(--ink-soft)] md:text-base">
              承接交付验真、合同与模板匹配检查、确认验收、拒收和争议入口。写动作
              统一透传 `X-Idempotency-Key`，并展示审计留痕与后端统一错误码。
            </p>
          </div>
          <PreviewStateControls />
        </div>
        <Card className="relative border-white/70 bg-white/70 backdrop-blur">
          <CardTitle>当前访问上下文</CardTitle>
          <div className="mt-4 grid gap-3 text-sm">
            <InfoRow label="order_id" value={orderId} />
            <InfoRow label="主体" value={subject?.display_name ?? subject?.login_id ?? "未登录"} />
            <InfoRow label="角色" value={formatList(subject?.roles ?? []) || "无"} />
            <InfoRow label="租户/组织" value={readSubjectTenant(subject) || "未返回"} />
            <InfoRow label="作用域" value={subject?.auth_context_level ?? sessionMode} />
          </div>
        </Card>
      </div>
    </motion.section>
  );
}

function AcceptanceSummary({
  order,
  lifecycle,
  orderId,
}: {
  order: OrderDetail;
  lifecycle: OrderLifecycleSnapshots | null;
  orderId: string;
}) {
  const skuType = getOrderSkuType(order);
  const delivery = latestDelivery(order, lifecycle);
  const manual = isManualAcceptanceSku(skuType);
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<ClipboardCheck className="size-5" />}
        title="交付结果摘要"
        description="只展示订单详情和 lifecycle API 已返回的字段；对象路径类字段不在验收页外泄。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <InfoTile label="当前状态" value={`${order.current_state} / ${order.acceptance_status}`} />
        <InfoTile label="SKU" value={`${skuType || "未返回"} / ${skuDisplayName(skuType)}`} />
        <InfoTile label="验收模式" value={manual ? "人工验收" : "自动验收或非人工分支"} />
        <InfoTile label="争议状态" value={order.dispute_status} />
      </div>
      <div className="grid gap-3 lg:grid-cols-[1.2fr_0.8fr]">
        <div className="rounded-[26px] border border-black/10 bg-white/70 p-4">
          <div className="mb-3 flex items-center justify-between gap-3">
            <div>
              <div className="text-sm font-semibold text-[var(--ink-strong)]">
                最近交付记录
              </div>
              <div className="text-xs text-[var(--ink-soft)]">
                用于验真、验收和拒收判断，不展示真实对象路径。
              </div>
            </div>
            <Badge className="bg-white/90 text-[var(--ink-soft)]">{delivery?.current_status ?? "未返回"}</Badge>
          </div>
          <div className="grid gap-2 text-sm md:grid-cols-2">
            <InfoRow label="delivery_id" value={delivery?.delivery_id ?? "未返回"} />
            <InfoRow label="delivery_type" value={delivery?.delivery_type ?? "未返回"} />
            <InfoRow label="delivery_route" value={delivery?.delivery_route ?? "未返回"} />
            <InfoRow label="committed_at" value={delivery?.committed_at ?? "未返回"} />
            <InfoRow label="receipt_hash" value={delivery?.receipt_hash ?? "未返回"} />
            <InfoRow label="delivery_commit_hash" value={delivery?.delivery_commit_hash ?? "未返回"} />
          </div>
        </div>
        <div className="rounded-[26px] border border-black/10 bg-[var(--panel-muted)] p-4">
          <div className="text-sm font-semibold text-[var(--ink-strong)]">标准 SKU 承接</div>
          <div className="mt-3 flex flex-wrap gap-2">
            {["FILE_STD", "FILE_SUB", "SHARE_RO", "API_SUB", "API_PPU", "QRY_LITE", "SBX_STD", "RPT_STD"].map((sku) => (
              <span
                key={sku}
                className={cn(
                  "rounded-full px-3 py-1 text-xs font-semibold",
                  sku === skuType
                    ? "bg-[var(--accent-strong)] text-white"
                    : MANUAL_ACCEPTANCE_SKUS.includes(sku as never)
                      ? "bg-[#f8e7b5] text-[#70520f]"
                      : "bg-white/80 text-[var(--ink-soft)]",
                )}
              >
                {sku}
              </span>
            ))}
          </div>
          <p className="mt-4 text-xs leading-6 text-[var(--ink-soft)]">
            本页只对 `FILE_STD / FILE_SUB / RPT_STD` 的人工验收分支展示执行按钮；
            `SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD` 保持独立 SKU 语义，
            不被并回大类。
          </p>
          <Link
            href={`/support/cases/new?order_id=${orderId}` as Route}
            className="mt-4 inline-flex items-center gap-2 text-sm font-semibold text-[var(--accent-strong)]"
          >
            发起争议入口 <ArrowRight className="size-4" />
          </Link>
        </div>
      </div>
    </Card>
  );
}

function VerificationPanel({
  order,
  lifecycle,
}: {
  order: OrderDetail;
  lifecycle: OrderLifecycleSnapshots | null;
}) {
  const summary = verificationSummary(order, lifecycle);
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<Fingerprint className="size-5" />}
        title="验真结果与合同/模板匹配"
        description="页面只呈现可核验输入，买方确认后由 accept/reject API 持久化 verification_summary。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <InfoTile label="delivery_id" value={summary.deliveryId} />
        <InfoTile label="交付状态" value={summary.deliveryStatus} />
        <InfoTile label="receipt_hash" value={summary.receiptHash} />
        <InfoTile label="delivery_commit_hash" value={summary.deliveryCommitHash} />
        <InfoTile label="content_hash" value={summary.contentHash} />
        <InfoTile label="合同状态" value={summary.contractStatus} />
        <InfoTile label="验收模板" value={summary.acceptanceTemplate} />
        <InfoTile label="争议状态" value={summary.disputeStatus} />
      </div>
      <div className="grid gap-3 md:grid-cols-4">
        <TrustItem label="request_id" value="由每次 API 响应/错误返回；成功结果在审计记录中回查" />
        <TrustItem label="tx_hash" value="当前验收 API 未返回 tx_hash，页面显式承接为未返回" />
        <TrustItem label="链状态" value="当前订单详情接口未返回链状态，显示未返回而不伪造" />
        <TrustItem label="投影状态" value="使用订单 / delivery / lifecycle 当前状态作为前端投影承接" />
      </div>
    </Card>
  );
}

function LifecyclePanel({
  order,
  lifecycle,
}: {
  order: OrderDetail;
  lifecycle: OrderLifecycleSnapshots | null;
}) {
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<GitBranch className="size-5" />}
        title="生命周期摘要"
        description="按正式 lifecycle 对象展示支付、交付、验收、结算与争议，不新增状态名。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        {lifecycleRows(order, lifecycle).map(([label, status, time], index) => (
          <div key={label} className="rounded-[24px] bg-white/75 p-4">
            <div className="flex items-center gap-3">
              <span className="flex size-8 items-center justify-center rounded-full bg-[var(--accent-soft)] text-xs font-semibold text-[var(--accent-strong)]">
                {index + 1}
              </span>
              <div>
                <div className="text-sm font-semibold text-[var(--ink-strong)]">{label}</div>
                <div className="text-xs text-[var(--ink-soft)]">{status}</div>
              </div>
            </div>
            <div className="mt-3 text-xs text-[var(--ink-subtle)]">{time}</div>
          </div>
        ))}
      </div>
    </Card>
  );
}

function AcceptanceActionPanel({
  order,
  lifecycle,
  orderId,
  subject,
}: {
  order: OrderDetail;
  lifecycle: OrderLifecycleSnapshots | null;
  orderId: string;
  subject: SessionSubject | null;
}) {
  const queryClient = useQueryClient();
  const skuType = getOrderSkuType(order);
  const canOperate = canOperateAcceptance(subject);
  const manual = isManualAcceptanceSku(skuType);
  const delivered = isDeliveredForAcceptance(order);

  const refetch = () => {
    void queryClient.invalidateQueries({
      queryKey: ["portal", "acceptance", orderId],
    });
  };

  if (!canOperate) {
    return <ActionPermissionPanel subject={subject} />;
  }
  if (!manual) {
    return (
      <StateGatePanel
        title="当前 SKU 不是人工验收分支"
        message={`当前 SKU 为 ${skuType || "未返回"}，人工验收只适用于 ${MANUAL_ACCEPTANCE_SKUS.join(" / ")}。自动验收分支需在对应交付页和账单页查看。`}
      />
    );
  }
  if (!delivered) {
    return (
      <StateGatePanel
        title="订单未到人工验收状态"
        message={`按钮级权限要求 FILE_STD / FILE_SUB 为 delivered，RPT_STD 为 report_delivered；当前为 ${order.current_state}，不展示“验收通过”或“拒收”执行按钮。`}
      />
    );
  }

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <AcceptDecisionForm orderId={orderId} onChanged={refetch} />
      <RejectDecisionForm orderId={orderId} lifecycle={lifecycle} onChanged={refetch} />
    </div>
  );
}

function AcceptDecisionForm({
  orderId,
  onChanged,
}: {
  orderId: string;
  onChanged: () => void;
}) {
  const form = useForm<AcceptOrderFormValues>({
    resolver: zodResolver(acceptOrderFormSchema),
    defaultValues: defaultAcceptOrderValues(),
  });
  const mutation = useMutation({
    mutationFn: (values: AcceptOrderFormValues) =>
      sdk.delivery.acceptOrder(
        { id: orderId },
        buildAcceptOrderRequest(values),
        { idempotencyKey: values.idempotency_key },
      ),
    onSuccess: () => onChanged(),
  });
  const result = unwrapAcceptOrder(mutation.data);
  return (
    <DecisionCard
      title="确认验收"
      description="Hash、合同和模板检查通过后提交 accept；成功后会进入结算触发桥接链路。"
      icon={<CheckCircle2 className="size-5" />}
      tone="success"
      result={result}
      error={mutation.error}
    >
      <form
        className="space-y-4"
        onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
      >
        <Field label="note">
          <Input placeholder="hash verified and contract matched" {...form.register("note")} />
        </Field>
        <Field label="verification_summary JSON">
          <Textarea rows={5} {...form.register("verification_summary_json")} />
        </Field>
        <CheckField form={form} name="confirm_verification" label="确认已完成 Hash / 合同 / 模板核验" />
        <CheckField form={form} name="confirm_scope" label="确认验收对象来自当前订单与 SKU 快照" />
        <CheckField form={form} name="confirm_audit" label="确认动作会写入 delivery.accept / trade.order.accept 审计" />
        <Field label="X-Idempotency-Key">
          <Input {...form.register("idempotency_key")} />
        </Field>
        <FormErrors errors={form.formState.errors} />
        <div className="flex flex-wrap gap-2">
          <Button type="submit" disabled={mutation.isPending}>
            {mutation.isPending ? <LoaderCircle className="mr-2 size-4 animate-spin" /> : null}
            提交验收通过
          </Button>
          <Button
            type="button"
            variant="secondary"
            onClick={() => form.setValue("idempotency_key", createAcceptanceIdempotencyKey("accept"))}
          >
            重新生成幂等键
          </Button>
        </div>
      </form>
    </DecisionCard>
  );
}

function RejectDecisionForm({
  orderId,
  lifecycle,
  onChanged,
}: {
  orderId: string;
  lifecycle: OrderLifecycleSnapshots | null;
  onChanged: () => void;
}) {
  const form = useForm<RejectOrderFormValues>({
    resolver: zodResolver(rejectOrderFormSchema),
    defaultValues: defaultRejectOrderValues(),
  });
  const mutation = useMutation({
    mutationFn: (values: RejectOrderFormValues) =>
      sdk.delivery.rejectOrder(
        { id: orderId },
        buildRejectOrderRequest(values),
        { idempotencyKey: values.idempotency_key },
      ),
    onSuccess: () => onChanged(),
  });
  const result = unwrapRejectOrder(mutation.data);
  return (
    <DecisionCard
      title="拒收"
      description="拒收必须填写 reason_code 和原因说明；成功后结算会阻断，争议状态打开。"
      icon={<XCircle className="size-5" />}
      tone="danger"
      result={result}
      error={mutation.error}
    >
      <form
        className="space-y-4"
        onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
      >
        <Field label="reason_code">
          <Input placeholder="report_quality_failed" {...form.register("reason_code")} />
        </Field>
        <Field label="reason_detail">
          <Textarea rows={3} placeholder="描述 Hash、样本或模板核验失败原因" {...form.register("reason_detail")} />
        </Field>
        <Field label="verification_summary JSON">
          <Textarea rows={5} {...form.register("verification_summary_json")} />
        </Field>
        <CheckField form={form} name="confirm_verification" label="确认拒收基于已完成的验真和模板检查" />
        <CheckField form={form} name="confirm_scope" label="确认拒收对象来自当前订单与交付记录" />
        <CheckField form={form} name="confirm_audit" label="确认动作会写入 delivery.reject / trade.order.reject 审计" />
        <Field label="X-Idempotency-Key">
          <Input {...form.register("idempotency_key")} />
        </Field>
        <FormErrors errors={form.formState.errors} />
        <div className="rounded-2xl bg-[var(--warning-soft)] p-3 text-xs leading-6 text-[var(--warning-ink)]">
          当前 lifecycle dispute：{lifecycle?.order.dispute.current_status ?? "未返回"}；
          拒收成功后后端应返回 `settlement_status=blocked` 与 `dispute_status=open`。
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="submit"
            variant="warning"
            className="bg-[var(--danger-soft)] text-[var(--danger-ink)] ring-1 ring-[var(--danger-ring)] hover:bg-[var(--danger-soft)]"
            disabled={mutation.isPending}
          >
            {mutation.isPending ? <LoaderCircle className="mr-2 size-4 animate-spin" /> : null}
            提交拒收
          </Button>
          <Button
            type="button"
            variant="secondary"
            onClick={() => form.setValue("idempotency_key", createAcceptanceIdempotencyKey("reject"))}
          >
            重新生成幂等键
          </Button>
        </div>
      </form>
    </DecisionCard>
  );
}

function DecisionCard({
  title,
  description,
  icon,
  tone,
  result,
  error,
  children,
}: {
  title: string;
  description: string;
  icon: ReactNode;
  tone: "success" | "danger";
  result: AcceptanceDecisionResult | RejectionDecisionResult | null;
  error: unknown;
  children: ReactNode;
}) {
  return (
    <Card className={cn("space-y-4", tone === "danger" && "border-[var(--danger-ring)]")}>
      <PanelTitle icon={icon} title={title} description={description} />
      {children}
      {error ? (
        <div className="rounded-2xl border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-3 text-sm text-[var(--danger-ink)]">
          {formatAcceptanceError(error)}
        </div>
      ) : null}
      {result ? <DecisionResult result={result} /> : null}
    </Card>
  );
}

function DecisionResult({
  result,
}: {
  result: AcceptanceDecisionResult | RejectionDecisionResult;
}) {
  return (
    <div className="rounded-2xl border border-black/10 bg-white/80 p-4">
      <div className="mb-3 text-sm font-semibold text-[var(--ink-strong)]">后端处理结果</div>
      <div className="grid gap-2 text-sm md:grid-cols-2">
        <InfoRow label="action" value={result.action} />
        <InfoRow label="operation" value={result.operation ?? "未返回"} />
        <InfoRow label="previous_state" value={result.previous_state} />
        <InfoRow label="current_state" value={result.current_state} />
        <InfoRow label="acceptance_status" value={result.acceptance_status} />
        <InfoRow label="settlement_status" value={result.settlement_status} />
        <InfoRow label="dispute_status" value={result.dispute_status} />
        <InfoRow label="reason_code" value={result.reason_code} />
        <InfoRow label="request_id" value="由 API 响应错误或审计回查提供" />
        <InfoRow label="processed_at" value={result.processed_at} />
      </div>
    </div>
  );
}

function PermissionPanel({
  subject,
  sessionMode,
}: {
  subject: SessionSubject | null;
  sessionMode: string;
}) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <PanelTitle
        icon={<Ban className="size-5" />}
        title="验收页权限态"
        description={`需要权限：trade.order.read；主按钮权限：delivery.accept.execute / delivery.reject.execute；当前会话模式 ${sessionMode}，角色 ${formatList(subject?.roles ?? []) || "无"}。`}
      />
    </Card>
  );
}

function ActionPermissionPanel({ subject }: { subject: SessionSubject | null }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <PanelTitle
        icon={<ShieldCheck className="size-5" />}
        title="主按钮权限不足"
        description={`确认验收和拒收需要 ${formatList([...ACCEPTANCE_ACTION_ALLOWED_ROLES])} 对应的 delivery.accept.execute / delivery.reject.execute；当前角色 ${formatList(subject?.roles ?? []) || "无"}。`}
      />
    </Card>
  );
}

function StateGatePanel({ title, message }: { title: string; message: string }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <PanelTitle icon={<ShieldCheck className="size-5" />} title={title} description={message} />
    </Card>
  );
}

function LoadingPanel() {
  return (
    <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <LoaderCircle className="size-8 animate-spin" />
        <CardTitle>验收页加载态</CardTitle>
        <CardDescription>正在读取当前主体、订单详情和 lifecycle 快照。</CardDescription>
      </div>
    </Card>
  );
}

function EmptyPanel() {
  return (
    <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <FileCheck2 className="size-8" />
        <CardTitle>没有可展示的验收数据</CardTitle>
        <CardDescription>订单详情或生命周期快照未返回，不能展示验收动作。</CardDescription>
      </div>
    </Card>
  );
}

function ErrorPanel({ title, message }: { title: string; message: string }) {
  return (
    <Card className="flex min-h-64 items-center justify-center border-[var(--danger-ring)] bg-[var(--danger-soft)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--danger-ink)]">
        <AlertTriangle className="size-8" />
        <CardTitle>{title}</CardTitle>
        <CardDescription className="text-[var(--danger-ink)]">{message}</CardDescription>
      </div>
    </Card>
  );
}

function AcceptancePreviewState({
  orderId,
  preview,
}: {
  orderId: string;
  preview: string;
}) {
  const common = (
    <AcceptanceHero orderId={orderId} subject={null} sessionMode="preview" />
  );
  if (preview === "loading") {
    return <div className="space-y-6">{common}<LoadingPanel /></div>;
  }
  if (preview === "empty") {
    return <div className="space-y-6">{common}<EmptyPanel /></div>;
  }
  if (preview === "error") {
    return (
      <div className="space-y-6">
        {common}
        <ErrorPanel title="验收页错误态" message="DELIVERY_STATUS_INVALID: 页面必须承接统一错误码与 request_id。" />
      </div>
    );
  }
  return (
    <div className="space-y-6">
      {common}
      <PermissionPanel subject={null} sessionMode="guest" />
    </div>
  );
}

function PanelTitle({
  icon,
  title,
  description,
}: {
  icon: ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="flex items-start gap-3">
      <span className="flex size-10 shrink-0 items-center justify-center rounded-2xl bg-[var(--accent-soft)] text-[var(--accent-strong)]">
        {icon}
      </span>
      <div>
        <CardTitle>{title}</CardTitle>
        <CardDescription className="mt-1">{description}</CardDescription>
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="grid gap-2 text-sm font-semibold text-[var(--ink-strong)]">
      {label}
      {children}
    </label>
  );
}

function CheckField({
  form,
  name,
  label,
}: {
  form: UseFormReturn<AcceptOrderFormValues> | UseFormReturn<RejectOrderFormValues>;
  name: "confirm_verification" | "confirm_scope" | "confirm_audit";
  label: string;
}) {
  const register = form.register as unknown as (
    field: typeof name,
  ) => UseFormRegisterReturn<typeof name>;
  return (
    <label className="flex items-start gap-2 rounded-2xl bg-[var(--panel-muted)] p-3 text-sm text-[var(--ink-soft)]">
      <input type="checkbox" className="mt-1" {...register(name)} />
      <span>{label}</span>
    </label>
  );
}

function FormErrors({
  errors,
}: {
  errors: FieldErrors<AcceptOrderFormValues | RejectOrderFormValues>;
}) {
  const messages = Object.values(errors)
    .map((error) => error?.message)
    .filter(Boolean);
  if (!messages.length) {
    return null;
  }
  return (
    <div className="rounded-2xl border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-3 text-sm text-[var(--danger-ink)]">
      {messages.join(" / ")}
    </div>
  );
}

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[24px] bg-white/75 p-4">
      <div className="text-xs font-medium uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 break-all text-sm font-semibold text-[var(--ink-strong)]">
        {value || "未返回"}
      </div>
    </div>
  );
}

function TrustItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[22px] border border-black/10 bg-[var(--panel-muted)] p-4 text-sm">
      <div className="font-semibold text-[var(--ink-strong)]">{label}</div>
      <div className="mt-2 leading-6 text-[var(--ink-soft)]">{value}</div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-start justify-between gap-3 rounded-2xl bg-white/60 px-3 py-2">
      <span className="shrink-0 text-xs font-medium uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </span>
      <span className="break-all text-right text-sm font-semibold text-[var(--ink-strong)]">
        {value || "未返回"}
      </span>
    </div>
  );
}
