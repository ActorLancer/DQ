"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  ArrowRight,
  Ban,
  Banknote,
  CircleDollarSign,
  FileText,
  GitBranch,
  LoaderCircle,
  ReceiptText,
  RotateCcw,
  ShieldAlert,
} from "lucide-react";
import { motion } from "motion/react";
import type { Route } from "next";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { startTransition, useEffect, type ReactNode } from "react";
import {
  useForm,
  type FieldErrors,
  type UseFormRegisterReturn,
  type UseFormReturn,
} from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  BILLING_ACTION_ALLOWED_ROLES,
  BILLING_READ_ALLOWED_ROLES,
  billingLookupSchema,
  billingStatusTiles,
  buildCompensationExecutionRequest,
  buildRefundExecutionRequest,
  canExecuteBillingAdjustments,
  canReadBilling,
  compensationExecutionFormSchema,
  createBillingIdempotencyKey,
  defaultCompensationExecutionValues,
  defaultRefundExecutionValues,
  formatBillingError,
  formatMoney,
  hasRefundOrCompensation,
  latestBillingEvent,
  readBillingSubjectTenant,
  refundExecutionFormSchema,
  unwrapBillingOrder,
  unwrapCompensationExecution,
  unwrapRefundExecution,
  type BillingLookupValues,
  type BillingOrderDetail,
  type CompensationExecutionFormValues,
  type CompensationExecutionResult,
  type RefundExecutionFormValues,
  type RefundExecutionResult,
} from "@/lib/billing-workflow";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import type { PortalSessionPreview } from "@/lib/session";
import { standardSkuOptions } from "@/lib/seller-products-view";
import { cn, formatList } from "@/lib/utils";

import { PreviewStateControls, ScaffoldPill, getPreviewState } from "./state-preview";

const sdk = createBrowserSdk();

type BillingShellProps = {
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

type SessionSubject = NonNullable<PortalSessionPreview>;

export function BillingCenterShell({
  sessionMode,
  initialSubject,
}: BillingShellProps) {
  const searchParams = useSearchParams();
  const orderId = searchParams.get("order_id") ?? "";
  const preview = getPreviewState(searchParams);
  const authQuery = useQuery({
    queryKey: ["portal", "billing", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest" && preview === "ready",
  });
  const subject = (authQuery.data?.data ?? initialSubject) as SessionSubject | null;
  const canRead = canReadBilling(subject);
  const billingQuery = useQuery({
    queryKey: ["portal", "billing", orderId],
    queryFn: () => sdk.billing.getBillingOrder({ order_id: orderId }),
    enabled: Boolean(orderId) && sessionMode !== "guest" && preview === "ready" && canRead,
  });
  const detail = unwrapBillingOrder(billingQuery.data);
  const loading = authQuery.isLoading || billingQuery.isLoading;
  const error = authQuery.error ?? billingQuery.error;

  if (preview !== "ready") {
    return <BillingPreviewState preview={preview} subject={subject} orderId={orderId} />;
  }

  return (
    <div className="space-y-6">
      <BillingHero subject={subject} sessionMode={sessionMode} orderId={orderId} />
      <BillingLookupPanel orderId={orderId} />
      {sessionMode === "guest" || !canRead ? (
        <BillingPermissionPanel subject={subject} sessionMode={sessionMode} />
      ) : !orderId ? (
        <BillingEmptyPanel title="请输入 order_id 查询账单" />
      ) : loading ? (
        <BillingLoadingPanel />
      ) : error ? (
        <BillingErrorPanel title="账单中心错误态" message={formatBillingError(error)} />
      ) : !detail ? (
        <BillingEmptyPanel title="没有可展示的账单数据" />
      ) : (
        <>
          <BillingStatusPanel detail={detail} />
          <BillingEventsPanel detail={detail} />
          <SettlementPanel detail={detail} />
          <RefundCompensationStatusPanel detail={detail} />
          <BillingRulePanel detail={detail} />
          <BillingFollowupPanel detail={detail} />
        </>
      )}
    </div>
  );
}

export function BillingRefundCompensationShell({
  sessionMode,
  initialSubject,
}: BillingShellProps) {
  const searchParams = useSearchParams();
  const orderId = searchParams.get("order_id") ?? "";
  const caseId = searchParams.get("case_id") ?? "";
  const preview = getPreviewState(searchParams);
  const queryClient = useQueryClient();
  const authQuery = useQuery({
    queryKey: ["portal", "billing-refunds", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest" && preview === "ready",
  });
  const subject = (authQuery.data?.data ?? initialSubject) as SessionSubject | null;
  const canRead = canReadBilling(subject);
  const canExecute = canExecuteBillingAdjustments(subject);
  const billingQuery = useQuery({
    queryKey: ["portal", "billing", orderId],
    queryFn: () => sdk.billing.getBillingOrder({ order_id: orderId }),
    enabled: Boolean(orderId) && sessionMode !== "guest" && preview === "ready" && canRead,
  });
  const detail = unwrapBillingOrder(billingQuery.data);
  const onChanged = () => {
    void queryClient.invalidateQueries({ queryKey: ["portal", "billing", orderId] });
  };

  if (preview !== "ready") {
    return <BillingRefundPreviewState preview={preview} subject={subject} orderId={orderId} />;
  }

  return (
    <div className="space-y-6">
      <RefundHero subject={subject} sessionMode={sessionMode} orderId={orderId} caseId={caseId} />
      <RefundContextPanel orderId={orderId} caseId={caseId} />
      {sessionMode === "guest" || !canRead ? (
        <BillingPermissionPanel subject={subject} sessionMode={sessionMode} />
      ) : !orderId || !caseId ? (
        <BillingEmptyPanel title="请输入 order_id 与 case_id 后执行退款/赔付" />
      ) : billingQuery.isLoading || authQuery.isLoading ? (
        <BillingLoadingPanel />
      ) : billingQuery.error || authQuery.error ? (
        <BillingErrorPanel
          title="退款/赔付处理页错误态"
          message={formatBillingError(billingQuery.error ?? authQuery.error)}
        />
      ) : (
        <>
          {detail ? <LiabilitySummaryPanel detail={detail} caseId={caseId} /> : null}
          {!canExecute ? (
            <HighRiskPermissionPanel subject={subject} />
          ) : (
            <div className="grid gap-4 xl:grid-cols-2">
              <RefundExecutionForm
                orderId={orderId}
                caseId={caseId}
                currencyCode={detail?.currency_code}
                onChanged={onChanged}
              />
              <CompensationExecutionForm
                orderId={orderId}
                caseId={caseId}
                currencyCode={detail?.currency_code}
                onChanged={onChanged}
              />
            </div>
          )}
        </>
      )}
    </div>
  );
}

function BillingHero({
  subject,
  sessionMode,
  orderId,
}: {
  subject: SessionSubject | null;
  sessionMode: string;
  orderId: string;
}) {
  const meta = portalRouteMap.billing_center;
  return (
    <HeroFrame>
      <div className="space-y-5">
        <div className="flex flex-wrap gap-2">
          <ScaffoldPill>WEB-012</ScaffoldPill>
          <ScaffoldPill>{meta.viewPermission}</ScaffoldPill>
          {meta.primaryPermissions.map((permission) => (
            <ScaffoldPill key={permission}>{permission}</ScaffoldPill>
          ))}
        </div>
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.32em] text-[var(--accent-strong)]">
            billing center
          </p>
          <h1 className="mt-3 text-3xl font-semibold tracking-[-0.05em] text-[var(--ink-strong)] md:text-5xl">
            账单中心
          </h1>
          <p className="mt-4 max-w-3xl text-sm leading-7 text-[var(--ink-soft)] md:text-base">
            联查订单、账单事件、结算结果、支付状态、退款/赔付状态和争议入口。页面只经
            `/api/platform` 调用 `platform-core` 正式 Billing API。
          </p>
        </div>
        <PreviewStateControls />
      </div>
      <SubjectCard subject={subject} sessionMode={sessionMode} orderId={orderId} />
    </HeroFrame>
  );
}

function RefundHero({
  subject,
  sessionMode,
  orderId,
  caseId,
}: {
  subject: SessionSubject | null;
  sessionMode: string;
  orderId: string;
  caseId: string;
}) {
  const meta = portalRouteMap.billing_refund_compensation;
  return (
    <HeroFrame tone="risk">
      <div className="space-y-5">
        <div className="flex flex-wrap gap-2">
          <ScaffoldPill>WEB-012</ScaffoldPill>
          <ScaffoldPill>{meta.viewPermission}</ScaffoldPill>
          {meta.primaryPermissions.map((permission) => (
            <ScaffoldPill key={permission}>{permission}</ScaffoldPill>
          ))}
          <ScaffoldPill tone="warning">step-up required</ScaffoldPill>
        </div>
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.32em] text-[var(--danger-ink)]">
            refund and compensation
          </p>
          <h1 className="mt-3 text-3xl font-semibold tracking-[-0.05em] text-[var(--ink-strong)] md:text-5xl">
            退款/赔付处理页
          </h1>
          <p className="mt-4 max-w-3xl text-sm leading-7 text-[var(--ink-soft)] md:text-base">
            高风险退款与赔付执行必须绑定争议裁决、step-up、幂等键和审计留痕；页面不伪造裁决结论。
          </p>
        </div>
        <PreviewStateControls />
      </div>
      <SubjectCard subject={subject} sessionMode={sessionMode} orderId={orderId} caseId={caseId} />
    </HeroFrame>
  );
}

function HeroFrame({
  children,
  tone = "default",
}: {
  children: ReactNode;
  tone?: "default" | "risk";
}) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      className={cn(
        "relative overflow-hidden rounded-[34px] border border-black/10 p-6 shadow-[0_24px_70px_rgba(31,44,61,0.13)]",
        tone === "risk"
          ? "bg-[linear-gradient(135deg,#fff1e8_0%,#f5ead5_44%,#eaf3f7_100%)]"
          : "bg-[linear-gradient(135deg,#ecf6ee_0%,#f8f2df_48%,#e8efff_100%)]",
      )}
    >
      <div className="absolute -right-16 -top-20 size-64 rounded-full bg-[#f39f6b]/25 blur-3xl" />
      <div className="absolute -bottom-24 left-1/4 size-72 rounded-full bg-[#6bb6a9]/25 blur-3xl" />
      <div className="relative grid gap-6 lg:grid-cols-[1.35fr_0.65fr]">
        {children}
      </div>
    </motion.section>
  );
}

function SubjectCard({
  subject,
  sessionMode,
  orderId,
  caseId,
}: {
  subject: SessionSubject | null;
  sessionMode: string;
  orderId: string;
  caseId?: string;
}) {
  return (
    <Card className="relative border-white/70 bg-white/70 backdrop-blur">
      <CardTitle>当前访问上下文</CardTitle>
      <div className="mt-4 grid gap-3 text-sm">
        <InfoRow label="主体" value={subject?.display_name ?? subject?.login_id ?? "未登录"} />
        <InfoRow label="角色" value={formatList(subject?.roles ?? []) || "无"} />
        <InfoRow label="租户/组织" value={readBillingSubjectTenant(subject) || "未返回"} />
        <InfoRow label="作用域" value={subject?.auth_context_level ?? sessionMode} />
        <InfoRow label="order_id" value={orderId || "未输入"} />
        {caseId !== undefined ? <InfoRow label="case_id" value={caseId || "未输入"} /> : null}
      </div>
    </Card>
  );
}

function BillingLookupPanel({ orderId }: { orderId: string }) {
  const router = useRouter();
  const form = useForm<BillingLookupValues>({
    resolver: zodResolver(billingLookupSchema),
    defaultValues: { order_id: orderId },
  });
  useEffect(() => {
    form.reset({ order_id: orderId });
  }, [form, orderId]);

  return (
    <Card>
      <PanelTitle
        icon={<ReceiptText className="size-5" />}
        title="账单查询"
        description="按 order_id 读取 `GET /api/v1/billing/{order_id}`，不从前端直连账务主库。"
      />
      <form
        className="mt-4 flex flex-col gap-3 md:flex-row"
        onSubmit={form.handleSubmit((values) => {
          startTransition(() => {
            router.replace(`/billing?order_id=${values.order_id}` as Route);
          });
        })}
      >
        <Input placeholder="order_id" {...form.register("order_id")} />
        <Button type="submit">查询账单</Button>
      </form>
      <FormErrors errors={form.formState.errors} />
    </Card>
  );
}

function RefundContextPanel({ orderId, caseId }: { orderId: string; caseId: string }) {
  const router = useRouter();
  const form = useForm<{ order_id: string; case_id: string }>({
    defaultValues: { order_id: orderId, case_id: caseId },
  });
  useEffect(() => {
    form.reset({ order_id: orderId, case_id: caseId });
  }, [caseId, form, orderId]);

  return (
    <Card>
      <PanelTitle
        icon={<ShieldAlert className="size-5" />}
        title="责任判定上下文"
        description="退款/赔付只接收已存在争议裁决的 order_id + case_id；完整争议创建由 WEB-013 承接。"
      />
      <form
        className="mt-4 grid gap-3 md:grid-cols-[1fr_1fr_auto]"
        onSubmit={form.handleSubmit((values) => {
          const params = new URLSearchParams();
          params.set("order_id", values.order_id);
          params.set("case_id", values.case_id);
          startTransition(() => {
            router.replace(`/billing/refunds?${params}` as Route);
          });
        })}
      >
        <Input placeholder="order_id" {...form.register("order_id")} />
        <Input placeholder="case_id" {...form.register("case_id")} />
        <Button type="submit">载入裁决</Button>
      </form>
    </Card>
  );
}

function BillingStatusPanel({ detail }: { detail: BillingOrderDetail }) {
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<CircleDollarSign className="size-5" />}
        title="支付与结算状态"
        description="状态字段来自 Billing order detail，不新增前端状态名；deposit_status 当前契约未返回，显式标注未返回。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-6">
        {billingStatusTiles(detail).map(([label, value]) => (
          <InfoTile key={label} label={label} value={value} />
        ))}
      </div>
      <div className="grid gap-3 md:grid-cols-4">
        <TrustItem label="request_id" value="由每次 API 请求、错误或审计回查提供" />
        <TrustItem label="tx_hash" value="当前 Billing detail 未返回链交易哈希，显示未返回而不伪造" />
        <TrustItem label="链状态" value={detail.settlement_summary?.proof_commit_state ?? "未返回"} />
        <TrustItem label="投影状态" value={`${detail.payment_status} / ${detail.settlement_status} / ${detail.dispute_status}`} />
      </div>
    </Card>
  );
}

function BillingEventsPanel({ detail }: { detail: BillingOrderDetail }) {
  const latest = latestBillingEvent(detail);
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<FileText className="size-5" />}
        title="账单明细"
        description="账单事件只读展示，不在前端重算最终结算金额。"
      />
      <div className="grid gap-3 md:grid-cols-3">
        <InfoTile label="billing_event_count" value={String(detail.billing_events.length)} />
        <InfoTile label="latest_event_type" value={latest?.event_type ?? "未返回"} />
        <InfoTile label="latest_amount" value={formatMoney(latest?.amount, latest?.currency_code)} />
      </div>
      {detail.billing_events.length ? (
        <div className="overflow-hidden rounded-[26px] border border-black/10 bg-white/70">
          <div className="grid grid-cols-[1.3fr_0.9fr_0.9fr_0.8fr] gap-2 bg-black/[0.04] px-4 py-3 text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
            <span>event</span>
            <span>source</span>
            <span>amount</span>
            <span>occurred_at</span>
          </div>
          <div className="max-h-[360px] overflow-auto">
            {detail.billing_events.map((event) => (
              <div
                key={event.billing_event_id}
                className="grid grid-cols-[1.3fr_0.9fr_0.9fr_0.8fr] gap-2 border-t border-black/5 px-4 py-3 text-sm"
              >
                <div className="min-w-0">
                  <div className="font-semibold text-[var(--ink-strong)]">{event.event_type}</div>
                  <div className="break-all text-xs text-[var(--ink-subtle)]">{event.billing_event_id}</div>
                </div>
                <span className="text-[var(--ink-soft)]">{event.event_source}</span>
                <span className="font-semibold text-[var(--ink-strong)]">
                  {formatMoney(event.amount, event.currency_code)}
                </span>
                <span className="text-xs text-[var(--ink-subtle)]">{event.occurred_at}</span>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <InlineEmpty text="当前订单暂无 billing_event。" />
      )}
    </Card>
  );
}

function SettlementPanel({ detail }: { detail: BillingOrderDetail }) {
  const summary = detail.settlement_summary;
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<Banknote className="size-5" />}
        title="结算结果与保证金占位"
        description="结算摘要来自后端聚合；保证金明细当前未在 Billing detail 中返回，页面显式标注缺省。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <InfoTile label="gross_amount" value={summary?.gross_amount ?? "未返回"} />
        <InfoTile label="platform_fee" value={summary?.platform_commission_amount ?? "未返回"} />
        <InfoTile label="channel_fee" value={summary?.channel_fee_amount ?? "未返回"} />
        <InfoTile label="supplier_receivable" value={summary?.supplier_receivable_amount ?? "未返回"} />
        <InfoTile label="refund_adjustment" value={summary?.refund_adjustment_amount ?? "未返回"} />
        <InfoTile label="compensation_adjustment" value={summary?.compensation_adjustment_amount ?? "未返回"} />
        <InfoTile label="summary_state" value={summary?.summary_state ?? "未返回"} />
        <InfoTile label="deposit_status" value="未返回" />
      </div>
      {detail.settlements.length ? (
        <div className="grid gap-3 lg:grid-cols-2">
          {detail.settlements.map((settlement) => (
            <div key={settlement.settlement_id} className="rounded-[24px] bg-white/75 p-4">
              <InfoRow label="settlement_id" value={settlement.settlement_id} />
              <InfoRow label="status" value={`${settlement.settlement_status} / ${settlement.settlement_mode}`} />
              <InfoRow label="payable" value={formatMoney(settlement.payable_amount, detail.currency_code)} />
              <InfoRow label="net_receivable" value={formatMoney(settlement.net_receivable_amount, detail.currency_code)} />
              <InfoRow label="reason_code" value={settlement.reason_code ?? "无"} />
            </div>
          ))}
        </div>
      ) : (
        <InlineEmpty text="当前订单暂无 settlement_record。" />
      )}
    </Card>
  );
}

function RefundCompensationStatusPanel({ detail }: { detail: BillingOrderDetail }) {
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<RotateCcw className="size-5" />}
        title="退款/赔付状态"
        description="退款与赔付记录来自后端只读聚合；执行入口在退款/赔付处理页并要求 step-up。"
      />
      <div className="grid gap-4 xl:grid-cols-2">
        <StatusList
          title="退款记录"
          empty="暂无退款记录"
          items={detail.refunds.map((refund) => ({
            id: refund.refund_id,
            status: refund.current_status,
            amount: formatMoney(refund.amount, refund.currency_code),
            time: refund.executed_at ?? refund.updated_at,
          }))}
        />
        <StatusList
          title="赔付记录"
          empty="暂无赔付记录"
          items={detail.compensations.map((compensation) => ({
            id: compensation.compensation_id,
            status: compensation.current_status,
            amount: formatMoney(compensation.amount, compensation.currency_code),
            time: compensation.executed_at ?? compensation.updated_at,
          }))}
        />
      </div>
      <div className="flex flex-wrap gap-2">
        <Button asChild variant="secondary">
          <Link href={`/billing/refunds?order_id=${detail.order_id}` as Route}>
            进入退款/赔付处理页
          </Link>
        </Button>
        <Button asChild variant="warning">
          <Link href={`/support/cases/new?order_id=${detail.order_id}` as Route}>
            发起争议入口
          </Link>
        </Button>
      </div>
    </Card>
  );
}

function BillingRulePanel({ detail }: { detail: BillingOrderDetail }) {
  const skuType = detail.sku_billing_basis?.sku_type ?? detail.api_billing_basis?.sku_type ?? "";
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<GitBranch className="size-5" />}
        title="SKU 计费规则与发票/税务占位"
        description="显式支持八个标准 SKU，不把 SHARE_RO / QRY_LITE / RPT_STD 并回大类。"
      />
      <div className="flex flex-wrap gap-2">
        {standardSkuOptions.map((option) => (
          <span
            key={option.sku_type}
            className={cn(
              "rounded-full px-3 py-1 text-xs font-semibold",
              option.sku_type === skuType
                ? "bg-[var(--accent-strong)] text-white"
                : "bg-white/80 text-[var(--ink-soft)]",
            )}
          >
            {option.sku_type}
          </span>
        ))}
      </div>
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <InfoTile label="sku_type" value={skuType || "未返回"} />
        <InfoTile label="billing_trigger" value={detail.sku_billing_basis?.billing_trigger ?? "未返回"} />
        <InfoTile label="settlement_cycle" value={detail.sku_billing_basis?.settlement_cycle ?? "未返回"} />
        <InfoTile label="refund_entry" value={detail.sku_billing_basis?.refund_entry ?? "未返回"} />
        <InfoTile label="refund_mode" value={detail.sku_billing_basis?.refund_mode ?? "未返回"} />
        <InfoTile label="compensation_entry" value={detail.sku_billing_basis?.compensation_entry ?? "未返回"} />
        <InfoTile label="tax_status" value={detail.tax_placeholder.tax_engine_status} />
        <InfoTile label="invoice_status" value={detail.invoice_placeholder.latest_invoice_status ?? "未返回"} />
      </div>
    </Card>
  );
}

function BillingFollowupPanel({ detail }: { detail: BillingOrderDetail }) {
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<ArrowRight className="size-5" />}
        title="联查入口"
        description="账单页只负责账务联查；争议创建、证据上传和审计联查由后续正式页面承接。"
      />
      <div className="grid gap-3 md:grid-cols-3">
        <FollowLink href={`/trade/orders/${detail.order_id}` as Route} label="订单详情" />
        <FollowLink href={`/support/cases/new?order_id=${detail.order_id}` as Route} label="发起争议" />
        <FollowLink href={`/billing/refunds?order_id=${detail.order_id}` as Route} label="退款/赔付" />
      </div>
    </Card>
  );
}

function LiabilitySummaryPanel({
  detail,
  caseId,
}: {
  detail: BillingOrderDetail;
  caseId: string;
}) {
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<ShieldAlert className="size-5" />}
        title="责任判定摘要"
        description="当前页不创建裁决，只把 case_id 与账单聚合作为退款/赔付执行输入。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <InfoTile label="case_id" value={caseId} />
        <InfoTile label="order_id" value={detail.order_id} />
        <InfoTile label="payment_status" value={detail.payment_status} />
        <InfoTile label="settlement_status" value={detail.settlement_status} />
        <InfoTile label="refund_records" value={String(detail.refunds.length)} />
        <InfoTile label="compensation_records" value={String(detail.compensations.length)} />
        <InfoTile label="refund_adjustment" value={detail.settlement_summary?.refund_adjustment_amount ?? "未返回"} />
        <InfoTile label="compensation_adjustment" value={detail.settlement_summary?.compensation_adjustment_amount ?? "未返回"} />
      </div>
      {!hasRefundOrCompensation(detail) ? (
        <div className="rounded-[24px] bg-[var(--warning-soft)] p-4 text-sm text-[var(--warning-ink)]">
          当前账单聚合未返回退款/赔付记录。只有已有责任判定并完成 step-up 后，平台风控结算员才能执行写操作。
        </div>
      ) : null}
    </Card>
  );
}

function RefundExecutionForm({
  orderId,
  caseId,
  currencyCode,
  onChanged,
}: {
  orderId: string;
  caseId: string;
  currencyCode?: string;
  onChanged: () => void;
}) {
  const form = useForm<RefundExecutionFormValues>({
    resolver: zodResolver(refundExecutionFormSchema),
    defaultValues: defaultRefundExecutionValues(orderId, caseId),
  });
  useEffect(() => {
    const next = defaultRefundExecutionValues(orderId, caseId);
    next.currency_code = currencyCode ?? "";
    form.reset(next);
  }, [caseId, currencyCode, form, orderId]);
  const mutation = useMutation({
    mutationFn: (values: RefundExecutionFormValues) =>
      sdk.billing.executeRefund(buildRefundExecutionRequest(values), {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token || undefined,
        stepUpChallengeId: values.step_up_challenge_id || undefined,
      }),
    onSuccess: () => onChanged(),
  });
  const result = unwrapRefundExecution(mutation.data);

  return (
    <ExecutionCard
      title="执行退款"
      description="调用 POST /api/v1/refunds；需要 billing.refund.execute、step-up、幂等键和责任判定。"
      result={result}
      error={mutation.error}
      kind="refund"
    >
      <ExecutionFormFields
        form={form}
        idempotencyAction="refund"
        onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
        pending={mutation.isPending}
      >
        <Field label="refund_mode">
          <Input {...form.register("refund_mode")} />
        </Field>
        <Field label="refund_template">
          <Input {...form.register("refund_template")} />
        </Field>
      </ExecutionFormFields>
    </ExecutionCard>
  );
}

function CompensationExecutionForm({
  orderId,
  caseId,
  currencyCode,
  onChanged,
}: {
  orderId: string;
  caseId: string;
  currencyCode?: string;
  onChanged: () => void;
}) {
  const form = useForm<CompensationExecutionFormValues>({
    resolver: zodResolver(compensationExecutionFormSchema),
    defaultValues: defaultCompensationExecutionValues(orderId, caseId),
  });
  useEffect(() => {
    const next = defaultCompensationExecutionValues(orderId, caseId);
    next.currency_code = currencyCode ?? "";
    form.reset(next);
  }, [caseId, currencyCode, form, orderId]);
  const mutation = useMutation({
    mutationFn: (values: CompensationExecutionFormValues) =>
      sdk.billing.executeCompensation(buildCompensationExecutionRequest(values), {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token || undefined,
        stepUpChallengeId: values.step_up_challenge_id || undefined,
      }),
    onSuccess: () => onChanged(),
  });
  const result = unwrapCompensationExecution(mutation.data);

  return (
    <ExecutionCard
      title="执行赔付"
      description="调用 POST /api/v1/compensations；需要 billing.compensation.execute、step-up、幂等键和责任判定。"
      result={result}
      error={mutation.error}
      kind="compensation"
    >
      <ExecutionFormFields
        form={form}
        idempotencyAction="compensation"
        onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
        pending={mutation.isPending}
      >
        <Field label="compensation_mode">
          <Input {...form.register("compensation_mode")} />
        </Field>
        <Field label="compensation_template">
          <Input {...form.register("compensation_template")} />
        </Field>
      </ExecutionFormFields>
    </ExecutionCard>
  );
}

function ExecutionCard({
  title,
  description,
  result,
  error,
  kind,
  children,
}: {
  title: string;
  description: string;
  result: RefundExecutionResult | CompensationExecutionResult | null;
  error: unknown;
  kind: "refund" | "compensation";
  children: ReactNode;
}) {
  return (
    <Card className="space-y-4 border-[var(--danger-ring)]">
      <PanelTitle
        icon={<ShieldAlert className="size-5" />}
        title={title}
        description={description}
      />
      {children}
      {error ? (
        <div className="rounded-2xl border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-3 text-sm text-[var(--danger-ink)]">
          {formatBillingError(error)}
        </div>
      ) : null}
      {result ? <ExecutionResult result={result} kind={kind} /> : null}
    </Card>
  );
}

function ExecutionFormFields<TValues extends RefundExecutionFormValues | CompensationExecutionFormValues>({
  form,
  idempotencyAction,
  onSubmit,
  pending,
  children,
}: {
  form: UseFormReturn<TValues>;
  idempotencyAction: "refund" | "compensation";
  onSubmit: () => void;
  pending: boolean;
  children: ReactNode;
}) {
  return (
    <form className="space-y-4" onSubmit={onSubmit}>
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="order_id">
          <Input {...form.register("order_id" as never)} />
        </Field>
        <Field label="case_id">
          <Input {...form.register("case_id" as never)} />
        </Field>
        <Field label="decision_code">
          <Input {...form.register("decision_code" as never)} />
        </Field>
        <Field label="penalty_code">
          <Input {...form.register("penalty_code" as never)} />
        </Field>
        <Field label="amount">
          <Input {...form.register("amount" as never)} />
        </Field>
        <Field label="currency_code">
          <Input {...form.register("currency_code" as never)} />
        </Field>
        <Field label="reason_code">
          <Input {...form.register("reason_code" as never)} />
        </Field>
        {children}
      </div>
      <Field label="metadata JSON">
        <Textarea rows={4} {...form.register("metadata_json" as never)} />
      </Field>
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="X-Step-Up-Token">
          <Input {...form.register("step_up_token" as never)} />
        </Field>
        <Field label="X-Step-Up-Challenge-Id">
          <Input {...form.register("step_up_challenge_id" as never)} />
        </Field>
      </div>
      <CheckField form={form} name="confirm_liability" label="确认已有责任判定或裁决结果" />
      <CheckField form={form} name="confirm_step_up" label="确认高风险动作已完成 step-up" />
      <CheckField form={form} name="confirm_audit" label="确认动作会写入 billing.*.execute 审计" />
      <Field label="X-Idempotency-Key">
        <Input {...form.register("idempotency_key" as never)} />
      </Field>
      <FormErrors errors={form.formState.errors} />
      <div className="rounded-2xl bg-[var(--warning-soft)] p-3 text-xs leading-6 text-[var(--warning-ink)]">
        高风险动作要求二次认证、幂等提交、人工确认和审计强留痕；重复点击期间按钮会禁用。
      </div>
      <div className="flex flex-wrap gap-2">
        <Button type="submit" disabled={pending}>
          {pending ? <LoaderCircle className="mr-2 size-4 animate-spin" /> : null}
          提交执行
        </Button>
        <Button
          type="button"
          variant="secondary"
          onClick={() =>
            form.setValue(
              "idempotency_key" as never,
              createBillingIdempotencyKey(idempotencyAction) as never,
            )
          }
        >
          重新生成幂等键
        </Button>
      </div>
    </form>
  );
}

function CheckField<TValues extends RefundExecutionFormValues | CompensationExecutionFormValues>({
  form,
  name,
  label,
}: {
  form: UseFormReturn<TValues>;
  name: "confirm_liability" | "confirm_step_up" | "confirm_audit";
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

function ExecutionResult({
  result,
  kind,
}: {
  result: RefundExecutionResult | CompensationExecutionResult;
  kind: "refund" | "compensation";
}) {
  const id =
    kind === "refund"
      ? (result as RefundExecutionResult).refund_id
      : (result as CompensationExecutionResult).compensation_id;
  return (
    <div className="rounded-2xl border border-black/10 bg-white/80 p-4">
      <div className="mb-3 text-sm font-semibold text-[var(--ink-strong)]">后端处理结果</div>
      <div className="grid gap-2 text-sm md:grid-cols-2">
        <InfoRow label={`${kind}_id`} value={id} />
        <InfoRow label="order_id" value={result.order_id} />
        <InfoRow label="case_id" value={result.case_id} />
        <InfoRow label="decision_code" value={result.decision_code} />
        <InfoRow label="amount" value={formatMoney(result.amount, result.currency_code)} />
        <InfoRow label="current_status" value={result.current_status} />
        <InfoRow label="provider_key" value={result.provider_key} />
        <InfoRow label="step_up_bound" value={String(result.step_up_bound)} />
        <InfoRow label="idempotent_replay" value={String(result.idempotent_replay)} />
        <InfoRow label="updated_at" value={result.updated_at} />
      </div>
    </div>
  );
}

function BillingPermissionPanel({
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
        title="账单页面权限态"
        description={`需要 billing.statement.read；正式前端角色 ${formatList([...BILLING_READ_ALLOWED_ROLES])}。当前会话 ${sessionMode}，角色 ${formatList(subject?.roles ?? []) || "无"}。`}
      />
    </Card>
  );
}

function HighRiskPermissionPanel({ subject }: { subject: SessionSubject | null }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <PanelTitle
        icon={<ShieldAlert className="size-5" />}
        title="退款/赔付按钮权限不足"
        description={`执行退款/赔付只展示给 ${formatList([...BILLING_ACTION_ALLOWED_ROLES])}；当前角色 ${formatList(subject?.roles ?? []) || "无"}。`}
      />
    </Card>
  );
}

function BillingPreviewState({
  preview,
  subject,
  orderId,
}: {
  preview: string;
  subject: SessionSubject | null;
  orderId: string;
}) {
  return (
    <div className="space-y-6">
      <BillingHero subject={subject} sessionMode="preview" orderId={orderId} />
      {preview === "loading" ? <BillingLoadingPanel /> : null}
      {preview === "empty" ? <BillingEmptyPanel title="没有可展示的账单数据" /> : null}
      {preview === "error" ? (
        <BillingErrorPanel
          title="账单中心错误态"
          message="BIL_PROVIDER_FAILED: 页面必须承接统一错误码与 request_id。"
        />
      ) : null}
      {preview === "forbidden" ? (
        <BillingPermissionPanel subject={subject} sessionMode="guest" />
      ) : null}
    </div>
  );
}

function BillingRefundPreviewState({
  preview,
  subject,
  orderId,
}: {
  preview: string;
  subject: SessionSubject | null;
  orderId: string;
}) {
  return (
    <div className="space-y-6">
      <RefundHero subject={subject} sessionMode="preview" orderId={orderId} caseId="" />
      {preview === "loading" ? <BillingLoadingPanel /> : null}
      {preview === "empty" ? <BillingEmptyPanel title="请输入 order_id 与 case_id 后执行退款/赔付" /> : null}
      {preview === "error" ? (
        <BillingErrorPanel
          title="退款/赔付处理页错误态"
          message="BIL_PROVIDER_FAILED: 高风险写接口必须回显统一错误码。"
        />
      ) : null}
      {preview === "forbidden" ? (
        <HighRiskPermissionPanel subject={subject} />
      ) : null}
    </div>
  );
}

function BillingLoadingPanel() {
  return (
    <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <LoaderCircle className="size-8 animate-spin" />
        <CardTitle>账单页面加载态</CardTitle>
        <CardDescription>正在读取当前主体与 Billing 聚合。</CardDescription>
      </div>
    </Card>
  );
}

function BillingEmptyPanel({ title }: { title: string }) {
  return (
    <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <ReceiptText className="size-8" />
        <CardTitle>{title}</CardTitle>
        <CardDescription>页面不会用 mock 账单充当真实完成证据。</CardDescription>
      </div>
    </Card>
  );
}

function BillingErrorPanel({ title, message }: { title: string; message: string }) {
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

function StatusList({
  title,
  items,
  empty,
}: {
  title: string;
  items: Array<{ id: string; status: string; amount: string; time: string | null | undefined }>;
  empty: string;
}) {
  return (
    <div className="rounded-[26px] bg-white/75 p-4">
      <div className="mb-3 text-sm font-semibold text-[var(--ink-strong)]">{title}</div>
      {items.length ? (
        <div className="space-y-2">
          {items.map((item) => (
            <div key={item.id} className="rounded-2xl bg-black/[0.04] p-3 text-sm">
              <InfoRow label="id" value={item.id} />
              <InfoRow label="status" value={item.status} />
              <InfoRow label="amount" value={item.amount} />
              <InfoRow label="time" value={item.time ?? "未返回"} />
            </div>
          ))}
        </div>
      ) : (
        <InlineEmpty text={empty} />
      )}
    </div>
  );
}

function FollowLink({ href, label }: { href: Route; label: string }) {
  return (
    <Link
      href={href}
      className="flex items-center justify-between rounded-[24px] bg-white/75 px-4 py-3 text-sm font-semibold text-[var(--accent-strong)] ring-1 ring-black/10"
    >
      {label}
      <ArrowRight className="size-4" />
    </Link>
  );
}

function InlineEmpty({ text }: { text: string }) {
  return (
    <div className="rounded-[24px] bg-[var(--panel-muted)] p-4 text-sm text-[var(--ink-soft)]">
      {text}
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

function FormErrors({
  errors,
}: {
  errors: FieldErrors;
}) {
  const messages = Object.values(errors)
    .map((error) => error?.message)
    .filter(Boolean);
  if (!messages.length) {
    return null;
  }
  return (
    <div className="mt-3 rounded-2xl border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-3 text-sm text-[var(--danger-ink)]">
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
