"use client";

import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";
import { useQuery } from "@tanstack/react-query";
import {
  ArrowRight,
  Boxes,
  ClipboardCheck,
  FileSearch,
  Fingerprint,
  GitBranch,
  LoaderCircle,
  LockKeyhole,
  Route as RouteIcon,
  Search,
  ShieldCheck,
  Sparkles,
  Waypoints,
} from "lucide-react";
import { motion } from "motion/react";
import Link from "next/link";
import type { Route } from "next";
import { useSearchParams } from "next/navigation";
import type { ReactNode } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import {
  ORDER_SCENARIO_BLUEPRINTS,
  readStandardOrderTemplates,
} from "@/lib/order-workflow";
import { createBrowserSdk } from "@/lib/platform-sdk";
import type { PortalSessionPreview } from "@/lib/session";
import {
  findStandardDemoGuide,
  frozenStandardScenarios,
  type StandardDemoGuide,
  type StandardScenarioCode,
} from "@/lib/standard-demo";
import { formatList } from "@/lib/utils";

import { PortalRouteScaffold } from "./route-scaffold";
import { getPreviewState, ScaffoldPill } from "./state-preview";

const sdk = createBrowserSdk();

type StandardDemoShellProps = {
  scenarioCode: StandardScenarioCode;
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

export function StandardDemoShell({
  scenarioCode,
  sessionMode,
  initialSubject,
}: StandardDemoShellProps) {
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);
  const guide = findStandardDemoGuide(scenarioCode) ?? findStandardDemoGuide("S1")!;

  const liveEnabled =
    preview === "ready" && sessionMode === "bearer" && Boolean(initialSubject);
  const scenarioQuery = useQuery({
    queryKey: ["portal", "standard-demo", scenarioCode, "scenarios"],
    enabled: liveEnabled,
    queryFn: () => sdk.catalog.getStandardScenarioTemplates(),
  });
  const templateQuery = useQuery({
    queryKey: ["portal", "standard-demo", scenarioCode, "order-templates"],
    enabled: liveEnabled,
    queryFn: () => sdk.trade.listStandardOrderTemplates(),
  });

  const liveScenario = scenarioQuery.data?.data.find(
    (item) => item.scenario_code === guide.scenarioCode,
  );
  const scenario =
    liveScenario ??
    frozenStandardScenarios.find(
      (item) => item.scenario_code === guide.scenarioCode,
    );
  const templates = readStandardOrderTemplates(templateQuery.data?.data);
  const orderTemplate =
    templates.find((item) => item.scenario_code === guide.scenarioCode) ??
    ORDER_SCENARIO_BLUEPRINTS.find(
      (item) => item.scenario_code === guide.scenarioCode,
    );
  const isLiveScenario = Boolean(liveScenario);
  const isLiveTemplate = Boolean(
    templateQuery.data?.data.find(
      (item) => item.scenario_code === guide.scenarioCode,
    ),
  );

  return (
    <PortalRouteScaffold routeKey={guide.routeKey}>
      <div className="space-y-4">
        <motion.section
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
          className="grid gap-4 xl:grid-cols-[minmax(0,1.15fr)_0.85fr]"
        >
          <Card className="overflow-hidden bg-[linear-gradient(135deg,rgba(9,66,77,0.96),rgba(19,105,102,0.88),rgba(232,166,71,0.76))] text-white">
            <div className="space-y-5">
              <div className="flex flex-wrap gap-2">
                <ScaffoldPill>{guide.scenarioCode}</ScaffoldPill>
                <ScaffoldPill>{guide.industryLabel}</ScaffoldPill>
                <ScaffoldPill tone="warning">
                  {isLiveScenario ? "live scenario" : "frozen scenario"}
                </ScaffoldPill>
              </div>
              <div>
                <h1 className="text-3xl font-semibold leading-tight">
                  {guide.scenarioName}演示路径
                </h1>
                <p className="mt-3 max-w-3xl text-sm leading-7 text-white/82">
                  {guide.summary}
                </p>
              </div>
              <div className="grid gap-3 md:grid-cols-3">
                <HeroMetric
                  label="主 SKU"
                  value={guide.primarySku}
                  icon={<Boxes className="size-4" />}
                />
                <HeroMetric
                  label="补充 SKU"
                  value={formatList([...guide.supplementarySkus])}
                  icon={<Waypoints className="size-4" />}
                />
                <HeroMetric
                  label="数据分级"
                  value={guide.dataClassification}
                  icon={<ShieldCheck className="size-4" />}
                />
              </div>
              <div className="flex flex-wrap gap-3">
                <Button asChild variant="secondary">
                  <Link href={searchHref(guide) as Route}>
                    搜索同类商品
                    <Search className="size-4" />
                  </Link>
                </Button>
                <Button asChild variant="ghost">
                  <Link href={orderHref(guide) as Route}>
                    进入标准下单
                    <ArrowRight className="size-4" />
                  </Link>
                </Button>
              </div>
            </div>
          </Card>

          <Card>
            <CardTitle>说明卡片</CardTitle>
            <CardDescription>
              该卡片是首页直达演示路径的正式说明源，不替代搜索、详情、下单和交付页面的真实业务状态。
            </CardDescription>
            <div className="mt-4 grid gap-3">
              <InfoRow label="买方目标" value={guide.buyerOutcome} />
              <InfoRow label="卖方配置要点" value={guide.sellerSignal} />
              <InfoRow label="风险边界" value={guide.riskHint} warning />
              <InfoRow
                label="当前主体"
                value={
                  initialSubject
                    ? `${initialSubject.display_name ?? initialSubject.login_id} / ${formatList(initialSubject.roles)} / ${initialSubject.tenant_id ?? initialSubject.org_id ?? "tenant pending"} / ${initialSubject.auth_context_level}`
                    : `未建立正式主体，当前会话模式 ${sessionMode}`
                }
              />
            </div>
          </Card>
        </motion.section>

        <section className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_420px]">
          <main className="space-y-4">
            <Card>
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <CardTitle>首页直达路线</CardTitle>
                  <CardDescription>
                    路线保持页面说明书的 `首页 -&gt; 搜索 -&gt; 卖方主页 -&gt;
                    产品详情 -&gt; 下单` 主链路，并继续落到交付、验收、账单与审计联查。
                  </CardDescription>
                </div>
                <Badge className="bg-white text-[var(--ink-strong)]">
                  {guide.path}
                </Badge>
              </div>
              <div className="mt-5 grid gap-3">
                {guide.steps.map((step, index) => (
                  <DemoStepCard
                    key={`${guide.scenarioCode}-${step.label}`}
                    index={index}
                    step={step}
                  />
                ))}
              </div>
            </Card>

            <Card>
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <CardTitle>模板与 SKU 映射</CardTitle>
                  <CardDescription>
                    展示合同、验收、退款模板和订单模板，不把 `SHARE_RO / QRY_LITE / RPT_STD`
                    等独立 SKU 并回文件或 API 大类。
                  </CardDescription>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Badge>{guide.primarySku}</Badge>
                  {guide.supplementarySkus.map((sku) => (
                    <Badge key={sku} className="bg-white text-[var(--ink-soft)]">
                      {sku}
                    </Badge>
                  ))}
                </div>
              </div>
              <div className="mt-4 grid gap-3 md:grid-cols-2">
                <InfoRow
                  label="合同模板"
                  value={scenario?.contract_template ?? guide.contractTemplate}
                />
                <InfoRow
                  label="验收模板"
                  value={scenario?.acceptance_template ?? guide.acceptanceTemplate}
                />
                <InfoRow
                  label="退款模板"
                  value={scenario?.refund_template ?? guide.refundTemplate}
                />
                <InfoRow
                  label="订单模板"
                  value={orderTemplate?.template_code ?? "ORDER_TEMPLATE_PENDING"}
                />
                <InfoRow
                  label="主流程"
                  value={String(orderTemplate?.order_draft.primary_flow_code ?? guide.deliveryType)}
                />
                <InfoRow
                  label="流程节点"
                  value={formatList(orderTemplate?.workflow_steps ?? [])}
                />
              </div>
            </Card>

            <Card>
              <CardTitle>交付入口映射</CardTitle>
              <CardDescription>
                演示页只提供受控前端路由入口，真实状态仍由订单、交付、账单和审计 API 返回。
              </CardDescription>
              <div className="mt-4 flex flex-wrap gap-3">
                {guide.deliveryLinks.map((link) => (
                  <Button key={link.href} asChild variant="secondary">
                    <Link href={link.href as Route}>
                      {link.label} {link.sku}
                      <RouteIcon className="size-4" />
                    </Link>
                  </Button>
                ))}
                <Button asChild variant="secondary">
                  <Link href="/delivery/orders/demo-order/acceptance">
                    验收页
                    <ClipboardCheck className="size-4" />
                  </Link>
                </Button>
                <Button asChild variant="secondary">
                  <Link href="/billing">
                    账单页
                    <FileSearch className="size-4" />
                  </Link>
                </Button>
              </div>
            </Card>
          </main>

          <aside className="space-y-4">
            <LiveBindingCard
              sessionMode={sessionMode}
              scenarioQueryState={queryState(
                liveEnabled,
                scenarioQuery.isPending,
                scenarioQuery.isError,
                scenarioQuery.error,
                isLiveScenario,
              )}
              templateQueryState={queryState(
                liveEnabled,
                templateQuery.isPending,
                templateQuery.isError,
                templateQuery.error,
                isLiveTemplate,
              )}
            />
            <Card>
              <CardTitle>API / SDK 绑定</CardTitle>
              <CardDescription>
                本演示页使用 `packages/sdk-ts` 通过受控代理读取正式 API；未登录时只展示冻结说明，不伪造后端返回。
              </CardDescription>
              <div className="mt-4 grid gap-2">
                {[
                  "GET /api/v1/auth/me",
                  "GET /api/v1/catalog/standard-scenarios",
                  "GET /api/v1/orders/standard-templates",
                  "GET /api/v1/recommendations?placement_code=home_featured",
                  "GET /api/v1/catalog/search",
                ].map((binding) => (
                  <div
                    key={binding}
                    className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm text-[var(--ink-soft)]"
                  >
                    {binding}
                  </div>
                ))}
              </div>
            </Card>
            <Card>
              <CardTitle>权限与审计提示</CardTitle>
              <CardDescription>
                演示路径本身是 `portal.home.read` 只读说明页；进入搜索、下单或高风险动作后按目标页面权限执行。
              </CardDescription>
              <div className="mt-4 grid gap-3">
                <InfoRow
                  label="查看权限"
                  value="portal.home.read，全部角色 / 游客可见"
                />
                <InfoRow
                  label="写操作边界"
                  value="本页不发起写操作；下单、交付、验收等写动作必须携带 Idempotency-Key"
                  warning
                />
                <InfoRow
                  label="审计留痕"
                  value="搜索、推荐、详情、下单、交付和账单会在各自页面展示 request_id 与审计提示"
                />
              </div>
            </Card>
          </aside>
        </section>
      </div>
    </PortalRouteScaffold>
  );
}

function HeroMetric({
  label,
  value,
  icon,
}: {
  label: string;
  value: string;
  icon: ReactNode;
}) {
  return (
    <div className="rounded-[24px] bg-white/12 p-4 backdrop-blur">
      <div className="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-white/70">
        {icon}
        {label}
      </div>
      <div className="mt-2 text-sm font-semibold text-white">{value}</div>
    </div>
  );
}

function DemoStepCard({
  index,
  step,
}: {
  index: number;
  step: StandardDemoGuide["steps"][number];
}) {
  return (
    <Link
      href={step.href as Route}
      className="grid gap-3 rounded-[24px] bg-black/[0.03] p-4 transition hover:bg-black/[0.06] md:grid-cols-[72px_minmax(0,1fr)_auto]"
    >
      <div className="flex items-center gap-2 text-sm font-semibold text-[var(--ink-strong)]">
        <span className="flex size-9 items-center justify-center rounded-full bg-white text-xs ring-1 ring-black/8">
          {String(index + 1).padStart(2, "0")}
        </span>
      </div>
      <div className="min-w-0">
        <div className="text-sm font-semibold text-[var(--ink-strong)]">
          {step.label}
        </div>
        <div className="mt-1 text-sm leading-6 text-[var(--ink-soft)]">
          {step.description}
        </div>
        <div className="mt-2 text-xs text-[var(--ink-subtle)]">
          权限：{step.permission}
        </div>
      </div>
      <ArrowRight className="size-4 self-center text-[var(--ink-subtle)]" />
    </Link>
  );
}

function LiveBindingCard({
  sessionMode,
  scenarioQueryState,
  templateQueryState,
}: {
  sessionMode: "guest" | "bearer" | "local";
  scenarioQueryState: BindingState;
  templateQueryState: BindingState;
}) {
  return (
    <Card>
      <CardTitle>实时契约读取状态</CardTitle>
      <CardDescription>
        Bearer 会话下读取 `platform-core` 正式接口；游客和 Local Header 仅展示冻结演示说明。
      </CardDescription>
      <div className="mt-4 grid gap-3">
        {sessionMode === "guest" ? (
          <StateCallout
            title="游客态展示冻结演示"
            description="未登录时不调用正式推荐、搜索预览或订单模板接口，但首页到演示路径仍可访问。"
          />
        ) : sessionMode === "local" ? (
          <StateCallout
            title="Local Header 不替代 Bearer"
            description="本地 header 只用于 auth/me 联调；演示页不会把本地占位伪造成正式 API 读取。"
            tone="warning"
          />
        ) : null}
        <BindingRow
          icon={<Sparkles className="size-4" />}
          label="标准场景模板"
          state={scenarioQueryState}
        />
        <BindingRow
          icon={<GitBranch className="size-4" />}
          label="标准订单模板"
          state={templateQueryState}
        />
        <div className="rounded-[24px] bg-black/[0.04] p-4 text-sm text-[var(--ink-soft)]">
          浏览器端请求必须保持 `/api/platform/**`，由 Next Route Handler 转发到
          `platform-core`，不得直连 OpenSearch、Redis、Kafka、PostgreSQL 或 Fabric。
        </div>
      </div>
    </Card>
  );
}

type BindingState =
  | { kind: "disabled"; label: string }
  | { kind: "loading"; label: string }
  | { kind: "error"; label: string }
  | { kind: "empty"; label: string }
  | { kind: "live"; label: string };

function queryState(
  enabled: boolean,
  pending: boolean,
  error: boolean,
  errorValue: unknown,
  hasLiveValue: boolean,
): BindingState {
  if (!enabled) {
    return { kind: "disabled", label: "frozen fallback" };
  }
  if (pending) {
    return { kind: "loading", label: "loading" };
  }
  if (error) {
    return { kind: "error", label: formatError(errorValue) };
  }
  if (!hasLiveValue) {
    return { kind: "empty", label: "empty -> frozen fallback" };
  }
  return { kind: "live", label: "platform-core live" };
}

function BindingRow({
  icon,
  label,
  state,
}: {
  icon: ReactNode;
  label: string;
  state: BindingState;
}) {
  const tone =
    state.kind === "error"
      ? "border-[var(--warning-ring)] bg-[var(--warning-soft)] text-[var(--warning-ink)]"
      : state.kind === "live"
        ? "border-emerald-200 bg-emerald-50 text-emerald-900"
        : "border-black/8 bg-black/[0.03] text-[var(--ink-soft)]";

  return (
    <div className={`rounded-[24px] border p-4 ${tone}`}>
      <div className="flex items-center gap-2 text-sm font-semibold">
        {state.kind === "loading" ? (
          <LoaderCircle className="size-4 animate-spin" />
        ) : (
          icon
        )}
        {label}
      </div>
      <div className="mt-2 break-words text-sm">{state.label}</div>
    </div>
  );
}

function InfoRow({
  label,
  value,
  warning = false,
}: {
  label: string;
  value: string;
  warning?: boolean;
}) {
  return (
    <div
      className={
        warning
          ? "rounded-[24px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-4 text-[var(--warning-ink)]"
          : "rounded-[24px] bg-black/[0.04] p-4"
      }
    >
      <div className="flex items-center gap-2 text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {warning ? <LockKeyhole className="size-4" /> : <Fingerprint className="size-4" />}
        {label}
      </div>
      <div className="mt-2 text-sm leading-6 text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}

function StateCallout({
  title,
  description,
  tone = "neutral",
}: {
  title: string;
  description: string;
  tone?: "neutral" | "warning";
}) {
  return (
    <div
      className={
        tone === "warning"
          ? "rounded-[24px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-4 text-[var(--warning-ink)]"
          : "rounded-[24px] bg-black/[0.04] p-4 text-[var(--ink-soft)]"
      }
    >
      <div className="text-sm font-semibold">{title}</div>
      <div className="mt-2 text-sm leading-6">{description}</div>
    </div>
  );
}

function searchHref(guide: StandardDemoGuide) {
  const params = new URLSearchParams({
    q: guide.searchKeyword,
    scenario: guide.scenarioCode,
  });
  return `/search?${params.toString()}`;
}

function orderHref(guide: StandardDemoGuide) {
  return `/trade/orders/new?scenario=${encodeURIComponent(guide.scenarioCode)}`;
}

function formatError(error: unknown) {
  return formatPlatformErrorForDisplay(error, {
    fallbackCode: "SERVICE_UNAVAILABLE",
    fallbackDescription: "演示接口暂不可用，页面已退回冻结演示说明。",
  });
}
