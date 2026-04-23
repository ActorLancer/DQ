"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import type {
  CanceledOrder,
  CreatedOrder,
  OrderDetail,
  OrderLifecycleSnapshots,
  OrderTemplate,
  ProductDetail,
  ProductSku,
  SessionSubject,
} from "@/lib/order-workflow";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  AlertTriangle,
  ArrowRight,
  Ban,
  Boxes,
  CheckCircle2,
  ClipboardCheck,
  FileSearch,
  Fingerprint,
  GitBranch,
  LoaderCircle,
  LockKeyhole,
  PackageCheck,
  ReceiptText,
  RefreshCcw,
  ShieldCheck,
  ShoppingCart,
  Sparkles,
  Waypoints,
} from "lucide-react";
import { motion } from "motion/react";
import type { Route } from "next";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useEffect, useState, type ReactNode } from "react";
import { useForm, useWatch } from "react-hook-form";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import {
  buildCreateOrderRequest,
  canCancelOrder,
  canCreateOrder,
  canReadOrder,
  collectOrderSkuCoverage,
  createOrderIdempotencyKey,
  defaultOrderFormValues,
  deliveryRouteForSku,
  estimateBuyerDeposit,
  findTemplatesForSku,
  formatMoney,
  formatTradeError,
  hasAnyRole,
  orderCreateSchema,
  orderStatusLabel,
  readStandardOrderTemplates,
  readSubjectOrgId,
  requiresScenarioCode,
  resolveTemplateForSku,
  scenarioRole,
  skuOptionLabel,
  unwrapCanceledOrder,
  unwrapCreatedOrder,
  unwrapLifecycle,
  unwrapOrderDetail,
  ORDER_CREATE_ALLOWED_ROLES,
  ORDER_READ_ALLOWED_ROLES,
  type OrderCreateFormValues,
} from "@/lib/order-workflow";
import type { PortalSessionPreview } from "@/lib/session";
import { cn, formatList } from "@/lib/utils";

import {
  PreviewStateControls,
  ScaffoldPill,
  getPreviewState,
} from "./state-preview";

const sdk = createBrowserSdk();
const createMeta = portalRouteMap.order_create;
const detailMeta = portalRouteMap.order_detail;

type OrderShellProps = {
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

export function OrderCreateShell({
  productId,
  initialScenario,
  initialSkuId,
  sessionMode,
  initialSubject,
}: OrderShellProps & {
  productId?: string;
  initialScenario?: string;
  initialSkuId?: string;
}) {
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);
  const [selectedSkuId, setSelectedSkuId] = useState(initialSkuId ?? "");
  const [selectedScenarioCode, setSelectedScenarioCode] = useState(initialScenario ?? "");
  const [createdOrder, setCreatedOrder] = useState<{
    order: CreatedOrder;
    idempotencyKey: string;
  } | null>(null);

  const authQuery = useQuery({
    queryKey: ["portal", "order-create", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest" && preview === "ready",
  });
  const subject = authQuery.data?.data ?? initialSubject;
  const subjectOrgId = readSubjectOrgId(subject as SessionSubject | null);
  const templatesQuery = useQuery({
    queryKey: ["portal", "order-templates"],
    queryFn: () => sdk.trade.listStandardOrderTemplates(),
    enabled: sessionMode !== "guest" && preview === "ready",
  });
  const templates = readStandardOrderTemplates(templatesQuery.data?.data);
  const productQuery = useQuery({
    queryKey: ["portal", "order-create", "product", productId],
    queryFn: () => sdk.catalog.getProductDetail({ id: productId ?? "" }),
    enabled: Boolean(productId) && sessionMode !== "guest" && preview === "ready",
  });
  const product = productQuery.data?.data;
  const form = useForm<OrderCreateFormValues>({
    resolver: zodResolver(orderCreateSchema),
    defaultValues: defaultOrderFormValues(
      productId,
      subjectOrgId,
      initialSkuId,
      initialScenario,
    ),
  });
  const watchedScenario = useWatch({
    control: form.control,
    name: "scenario_code",
  });
  const effectiveSelectedSkuId =
    selectedSkuId ||
    initialSkuId ||
    product?.skus.find((sku) => ["active", "listed"].includes(sku.status))
      ?.sku_id ||
    "";
  const selectedSku = product?.skus.find(
    (sku) => sku.sku_id === effectiveSelectedSkuId,
  );
  const defaultTemplate = selectedSku
    ? resolveTemplateForSku(templates, selectedSku.sku_type, initialScenario) ??
      resolveTemplateForSku(templates, selectedSku.sku_type)
    : null;
  const effectiveScenarioCode =
    watchedScenario ||
    selectedScenarioCode ||
    initialScenario ||
    defaultTemplate?.scenario_code ||
    "";
  const selectedTemplate = selectedSku
    ? resolveTemplateForSku(
        templates,
        selectedSku.sku_type,
        effectiveScenarioCode || undefined,
      )
    : null;
  const scenarioNeedsChoice = selectedSku
    ? requiresScenarioCode(templates, selectedSku.sku_type)
    : false;
  const canSubmit = canCreateOrder(subject as SessionSubject | null);

  useEffect(() => {
    form.setValue("product_id", product?.product_id ?? productId ?? "");
    form.setValue("buyer_org_id", subjectOrgId);
    form.setValue("sku_id", effectiveSelectedSkuId);
    form.setValue("scenario_code", effectiveScenarioCode || undefined);
  }, [effectiveScenarioCode, effectiveSelectedSkuId, form, product?.product_id, productId, subjectOrgId]);

  const createMutation = useMutation({
    mutationFn: async (values: OrderCreateFormValues) => {
      if (!selectedSku) {
        throw new Error("ORDER_CREATE_FORBIDDEN: 必须选择商品的标准 SKU");
      }
      if (!resolveTemplateForSku(templates, selectedSku.sku_type, values.scenario_code)) {
        throw new Error("ORDER_CREATE_FORBIDDEN: scenario_code 与所选 SKU 不匹配");
      }
      const response = await sdk.trade.createOrder(
        buildCreateOrderRequest(values),
        { idempotencyKey: values.idempotency_key },
      );
      const order = unwrapCreatedOrder(response);
      if (!order) {
        throw new Error("TRD_STATE_CONFLICT: createOrder 未返回订单数据");
      }
      return { order, idempotencyKey: values.idempotency_key };
    },
    onSuccess: (result) => {
      setCreatedOrder(result);
      form.setValue("idempotency_key", createOrderIdempotencyKey("create"));
    },
  });

  const onSubmit = form.handleSubmit((values) => {
    if (scenarioNeedsChoice && !values.scenario_code) {
      form.setError("scenario_code", {
        message: `${selectedSku?.sku_type} 属于多条标准链路，必须选择 scenario_code`,
      });
      return;
    }
    createMutation.mutate(values);
  });

  return (
    <div className="space-y-6">
      <OrderHero
        meta={createMeta}
        preview={preview}
        subject={subject}
        sessionMode={sessionMode}
        title="询单 / 下单页"
        description="按标准链路选择商品 SKU，冻结价格、模板和场景快照后创建订单。"
        icon={<ShoppingCart className="size-6" />}
      />

      <ScenarioMatrix
        templates={templates}
        live={Boolean(templatesQuery.data?.data?.length)}
      />

      {preview === "loading" ? (
        <OrderLoadingState title="下单页加载中" />
      ) : preview === "empty" ? (
        <OrderEntryState templates={templates} />
      ) : preview === "error" ? (
        <OrderErrorState
          title="订单创建错误态"
          message="ORDER_CREATE_FORBIDDEN: 页面必须承接后端统一错误码与 request_id。"
          onRetry={() => {
            productQuery.refetch();
            templatesQuery.refetch();
          }}
        />
      ) : preview === "forbidden" || !canSubmit ? (
        <OrderPermissionState
          title="下单权限态"
          requiredRoles={ORDER_CREATE_ALLOWED_ROLES}
          subject={subject}
          message="需要 catalog.product.read 与 trade.order.create，只有买方运营员或租户管理员可创建订单。"
        />
      ) : !productId ? (
        <OrderEntryState templates={templates} />
      ) : productQuery.isPending || authQuery.isPending ? (
        <OrderLoadingState title="正在读取商品与当前主体" />
      ) : productQuery.isError ? (
        <OrderErrorState
          title="商品快照读取失败"
          message={formatTradeError(productQuery.error)}
          onRetry={() => productQuery.refetch()}
        />
      ) : product ? (
        <div className="grid gap-4 2xl:grid-cols-[minmax(0,1fr)_430px]">
          <main className="space-y-4">
            <ProductSnapshotCard product={product} />
            <SkuScenarioSelector
              product={product}
              templates={templates}
              selectedSkuId={effectiveSelectedSkuId}
              selectedScenarioCode={effectiveScenarioCode}
              onSelectSku={(sku) => {
                setSelectedSkuId(sku.sku_id);
                form.setValue("sku_id", sku.sku_id);
                const nextTemplate =
                  resolveTemplateForSku(templates, sku.sku_type, effectiveScenarioCode) ??
                  resolveTemplateForSku(templates, sku.sku_type);
                const nextScenario = nextTemplate?.scenario_code ?? "";
                setSelectedScenarioCode(nextScenario);
                form.setValue("scenario_code", nextScenario || undefined);
              }}
              onSelectScenario={(scenarioCode) => {
                setSelectedScenarioCode(scenarioCode);
                form.setValue("scenario_code", scenarioCode);
              }}
            />
            <CreateOrderFormCard
              form={form}
              selectedSku={selectedSku}
              selectedTemplate={selectedTemplate}
              watchedScenario={watchedScenario}
              onSubmit={onSubmit}
              disabled={createMutation.isPending}
            />
          </main>
          <aside className="space-y-4">
            <OrderContextCard subject={subject} sessionMode={sessionMode} />
            <OrderCreateRiskCard
              selectedSku={selectedSku}
              selectedTemplate={selectedTemplate}
              scenarioNeedsChoice={scenarioNeedsChoice}
              form={form}
            />
            {createMutation.isError ? (
              <OrderErrorState
                title="订单创建失败"
                message={formatTradeError(createMutation.error)}
                onRetry={() => createMutation.reset()}
              />
            ) : null}
            {createdOrder ? <CreatedOrderCard result={createdOrder} /> : null}
          </aside>
        </div>
      ) : (
        <OrderEntryState templates={templates} />
      )}
    </div>
  );
}

export function OrderDetailShell({
  orderId,
  sessionMode,
  initialSubject,
}: OrderShellProps & {
  orderId: string;
}) {
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);
  const [cancelResult, setCancelResult] = useState<{
    order: CanceledOrder;
    idempotencyKey: string;
  } | null>(null);
  const authQuery = useQuery({
    queryKey: ["portal", "order-detail", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest" && preview === "ready",
  });
  const subject = authQuery.data?.data ?? initialSubject;
  const canRead = canReadOrder(subject as SessionSubject | null);
  const canCancel = canCancelOrder(subject as SessionSubject | null);
  const detailQuery = useQuery({
    queryKey: ["portal", "order-detail", orderId],
    queryFn: () => sdk.trade.getOrderDetail({ id: orderId }),
    enabled: preview === "ready" && canRead,
  });
  const lifecycleQuery = useQuery({
    queryKey: ["portal", "order-lifecycle", orderId],
    queryFn: () => sdk.trade.getOrderLifecycleSnapshots({ id: orderId }),
    enabled: preview === "ready" && canRead,
  });
  const order = unwrapOrderDetail(detailQuery.data);
  const lifecycle = unwrapLifecycle(lifecycleQuery.data);
  const skuType = order?.price_snapshot?.sku_type ?? "UNKNOWN";
  const isCancelable = order
    ? [
        "created",
        "buyer_locked",
        "payment_failed_pending_resolution",
        "payment_timeout_pending_compensation_cancel",
      ].includes(order.current_state)
    : false;
  const cancelMutation = useMutation({
    mutationFn: async () => {
      const idempotencyKey = createOrderIdempotencyKey("cancel");
      const response = await sdk.trade.cancelOrder(
        { id: orderId },
        { idempotencyKey },
      );
      const canceled = unwrapCanceledOrder(response);
      if (!canceled) {
        throw new Error("TRD_STATE_CONFLICT: cancelOrder 未返回取消结果");
      }
      return { order: canceled, idempotencyKey };
    },
    onSuccess: async (result) => {
      setCancelResult(result);
      await Promise.all([detailQuery.refetch(), lifecycleQuery.refetch()]);
    },
  });

  return (
    <div className="space-y-6">
      <OrderHero
        meta={detailMeta}
        preview={preview}
        subject={subject}
        sessionMode={sessionMode}
        title="订单详情页"
        description="汇总订单主链路、分层状态、交付/验收/账单/争议和审计联查入口。"
        icon={<ReceiptText className="size-6" />}
      />

      {preview === "loading" ? (
        <OrderLoadingState title="订单详情加载中" />
      ) : preview === "empty" ? (
        <OrderEmptyState title="没有可展示的订单详情" message={`当前 order_id=${orderId} 没有返回详情。`} />
      ) : preview === "error" ? (
        <OrderErrorState
          title="订单详情错误态"
          message="TRD_STATE_CONFLICT: order detail failed，页面必须展示 request_id 与错误码。"
          onRetry={() => {
            detailQuery.refetch();
            lifecycleQuery.refetch();
          }}
        />
      ) : preview === "forbidden" || !canRead ? (
        <OrderPermissionState
          title="订单详情权限态"
          requiredRoles={ORDER_READ_ALLOWED_ROLES}
          subject={subject}
          message="需要 trade.order.read，且租户必须命中买方或卖方订单范围。"
        />
      ) : detailQuery.isPending || lifecycleQuery.isPending || authQuery.isPending ? (
        <OrderLoadingState title="正在读取订单详情和生命周期快照" />
      ) : detailQuery.isError ? (
        <OrderErrorState
          title="订单详情读取失败"
          message={formatTradeError(detailQuery.error)}
          onRetry={() => detailQuery.refetch()}
        />
      ) : order ? (
        <div className="grid gap-4 2xl:grid-cols-[minmax(0,1fr)_430px]">
          <main className="space-y-4">
            <OrderSummaryCard order={order} />
            <OrderTimelineCard order={order} lifecycle={lifecycle} />
            <OrderRelationsCard order={order} lifecycle={lifecycle} />
            <OrderTrustCard order={order} lifecycle={lifecycle} />
          </main>
          <aside className="space-y-4">
            <OrderContextCard subject={subject} sessionMode={sessionMode} />
            <OrderDetailActionsCard
              order={order}
              skuType={skuType}
              canCancel={canCancel}
              isCancelable={isCancelable}
              cancelPending={cancelMutation.isPending}
              onCancel={() => cancelMutation.mutate()}
            />
            {cancelMutation.isError ? (
              <OrderErrorState
                title="取消订单失败"
                message={formatTradeError(cancelMutation.error)}
                onRetry={() => cancelMutation.reset()}
              />
            ) : null}
            {cancelResult ? (
              <CancelResultCard result={cancelResult} />
            ) : null}
          </aside>
        </div>
      ) : (
        <OrderEmptyState title="没有可展示的订单详情" message={`order_id=${orderId} 未命中正式 API 响应。`} />
      )}
    </div>
  );
}

function OrderHero({
  meta,
  preview,
  subject,
  sessionMode,
  title,
  description,
  icon,
}: {
  meta: typeof createMeta;
  preview: string;
  subject: SessionSubject | PortalSessionPreview | null | undefined;
  sessionMode: "guest" | "bearer" | "local";
  title: string;
  description: string;
  icon: ReactNode;
}) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28 }}
      className="grid gap-4 xl:grid-cols-[1.5fr_0.9fr]"
    >
      <Card className="overflow-hidden bg-[radial-gradient(circle_at_85%_20%,rgba(11,132,101,0.20),transparent_26%),linear-gradient(135deg,rgba(255,255,255,0.98),rgba(230,242,236,0.9),rgba(239,232,214,0.72))]">
        <div className="space-y-5">
          <div className="flex flex-wrap gap-2">
            <ScaffoldPill>{meta.group}</ScaffoldPill>
            <ScaffoldPill>{meta.key}</ScaffoldPill>
            <ScaffoldPill tone="warning">preview:{preview}</ScaffoldPill>
            <ScaffoldPill>{sessionMode}</ScaffoldPill>
          </div>
          <div className="flex gap-4">
            <div className="hidden rounded-[24px] bg-white/75 p-4 text-[var(--accent-strong)] shadow-inner sm:block">
              {icon}
            </div>
            <div>
              <Badge>Trade Order Workflow</Badge>
              <h1 className="mt-3 text-3xl font-semibold tracking-[-0.045em] text-[var(--ink-strong)] md:text-5xl">
                {title}
              </h1>
              <CardDescription className="mt-4 text-base">{description}</CardDescription>
            </div>
          </div>
          <div className="grid gap-3 md:grid-cols-4">
            <InfoTile label="冻结路由" value={meta.path} />
            <InfoTile label="查看权限" value={meta.viewPermission} />
            <InfoTile label="主动作权限" value={formatList(meta.primaryPermissions)} />
            <InfoTile label="API 边界" value="/api/platform -> platform-core" />
          </div>
          <PreviewStateControls />
        </div>
      </Card>
      <OrderContextCard subject={subject} sessionMode={sessionMode} compact />
    </motion.section>
  );
}

function ScenarioMatrix({
  templates,
  live,
}: {
  templates: OrderTemplate[];
  live: boolean;
}) {
  const coverage = collectOrderSkuCoverage(templates);
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<Waypoints className="size-5" />}
        title="五条标准链路下单入口"
        description="官方链路、主 SKU、补充 SKU、合同模板、验收模板与退款模板按冻结表展示。"
      />
      <div className="flex flex-wrap gap-2">
        <Badge className="bg-white/70 text-[var(--ink-strong)]">
          {live ? "standard-templates live" : "standard-templates fallback"}
        </Badge>
        {coverage.map((sku) => (
          <Badge key={sku} className="bg-[var(--accent-soft)] text-[var(--accent-strong)]">
            {sku}
          </Badge>
        ))}
      </div>
      <div className="grid gap-3 lg:grid-cols-5">
        {templates.map((template) => (
          <div key={template.scenario_code} className="rounded-[24px] bg-black/[0.035] p-4">
            <div className="flex items-center justify-between gap-2">
              <Badge>{template.scenario_code}</Badge>
              <span className="text-xs text-[var(--ink-subtle)]">{template.industry_code}</span>
            </div>
            <div className="mt-3 text-sm font-semibold text-[var(--ink-strong)]">
              {template.scenario_name}
            </div>
            <div className="mt-3 flex flex-wrap gap-2">
              <ScenarioSkuBadge sku={template.primary_sku} role="主 SKU" />
              {template.supplementary_skus.map((sku) => (
                <ScenarioSkuBadge key={sku} sku={sku} role="补充 SKU" />
              ))}
            </div>
            <div className="mt-3 space-y-1 text-xs leading-5 text-[var(--ink-soft)]">
              <div>合同 {template.contract_template}</div>
              <div>验收 {template.acceptance_template}</div>
              <div>退款 {template.refund_template}</div>
            </div>
            <Button asChild variant="secondary" size="sm" className="mt-4">
              <Link
                href={`/trade/orders/new?scenario=${template.scenario_code}` as Route}
              >
                使用该链路 <ArrowRight className="size-3" />
              </Link>
            </Button>
          </div>
        ))}
      </div>
    </Card>
  );
}

function ProductSnapshotCard({ product }: { product: ProductDetail }) {
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<PackageCheck className="size-5" />}
        title="商品快照卡片"
        description="订单创建将冻结 product_id、sku_id、价格、币种、模板和场景快照。"
      />
      <div className="grid gap-3 md:grid-cols-4">
        <InfoTile label="商品" value={product.title} />
        <InfoTile label="product_id" value={product.product_id} />
        <InfoTile label="状态" value={product.status} />
        <InfoTile label="价格" value={formatMoney(product.price, product.currency_code)} />
      </div>
      <div className="grid gap-3 md:grid-cols-3">
        <InfoTile label="卖方组织" value={product.seller_org_id} />
        <InfoTile label="交付方式" value={product.delivery_type} />
        <InfoTile label="索引状态" value={product.index_sync_status} />
      </div>
    </Card>
  );
}

function SkuScenarioSelector({
  product,
  templates,
  selectedSkuId,
  selectedScenarioCode,
  onSelectSku,
  onSelectScenario,
}: {
  product: ProductDetail;
  templates: OrderTemplate[];
  selectedSkuId: string;
  selectedScenarioCode: string;
  onSelectSku: (sku: ProductSku) => void;
  onSelectScenario: (scenarioCode: string) => void;
}) {
  const selectedSku = product.skus.find((sku) => sku.sku_id === selectedSkuId);
  const matchingTemplates = selectedSku
    ? findTemplatesForSku(templates, selectedSku.sku_type)
    : [];
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<GitBranch className="size-5" />}
        title="SKU 与标准链路选择"
        description="SKU 类型是下单事实源；场景只用于选择官方模板与快照口径。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        {product.skus.map((sku) => {
          const skuTemplates = findTemplatesForSku(templates, sku.sku_type);
          const selected = sku.sku_id === selectedSkuId;
          return (
            <button
              key={sku.sku_id}
              type="button"
              onClick={() => onSelectSku(sku)}
              className={cn(
                "rounded-[24px] border p-4 text-left transition",
                selected
                  ? "border-[var(--accent-strong)] bg-[var(--accent-soft)]"
                  : "border-black/10 bg-white/75 hover:border-[var(--accent-strong)]",
              )}
            >
              <div className="flex flex-wrap items-center gap-2">
                <Badge>{sku.sku_type}</Badge>
                <span className="text-xs text-[var(--ink-subtle)]">{sku.status}</span>
              </div>
              <div className="mt-2 text-sm font-semibold text-[var(--ink-strong)]">
                {sku.sku_code}
              </div>
              <div className="mt-1 text-xs text-[var(--ink-soft)]">
                {skuOptionLabel(sku.sku_type)} / {sku.billing_mode} / {sku.acceptance_mode}
              </div>
              <div className="mt-3 flex flex-wrap gap-1">
                {skuTemplates.map((template) => (
                  <span key={template.scenario_code} className="rounded-full bg-white/70 px-2 py-1 text-[11px] text-[var(--ink-soft)]">
                    {template.scenario_code} {scenarioRole(template, sku.sku_type)}
                  </span>
                ))}
              </div>
            </button>
          );
        })}
      </div>
      {selectedSku ? (
        <div className="rounded-[24px] border border-black/10 bg-white/70 p-4">
          <div className="text-sm font-semibold text-[var(--ink-strong)]">
            {selectedSku.sku_type} 匹配的官方链路
          </div>
          <div className="mt-3 flex flex-wrap gap-2">
            {matchingTemplates.map((template) => (
              <Button
                key={template.scenario_code}
                type="button"
                size="sm"
                variant={selectedScenarioCode === template.scenario_code ? "default" : "secondary"}
                onClick={() => onSelectScenario(template.scenario_code)}
              >
                {template.scenario_code} {template.scenario_name}
              </Button>
            ))}
          </div>
          {requiresScenarioCode(templates, selectedSku.sku_type) ? (
            <CardDescription className="mt-3 text-[var(--warning-ink)]">
              {selectedSku.sku_type} 属于多条标准链路，创建订单必须显式传递 `scenario_code`。
            </CardDescription>
          ) : null}
        </div>
      ) : null}
    </Card>
  );
}

function CreateOrderFormCard({
  form,
  selectedSku,
  selectedTemplate,
  watchedScenario,
  onSubmit,
  disabled,
}: {
  form: ReturnType<typeof useForm<OrderCreateFormValues>>;
  selectedSku?: ProductSku;
  selectedTemplate: OrderTemplate | null;
  watchedScenario?: string;
  onSubmit: () => void;
  disabled: boolean;
}) {
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<ClipboardCheck className="size-5" />}
        title="创建订单"
        description="前端只提交正式 CreateOrderRequest 字段；数量/期限用于页面确认，不写入未冻结接口。"
      />
      <form className="space-y-4" onSubmit={onSubmit}>
        <input type="hidden" {...form.register("product_id")} />
        <input type="hidden" {...form.register("sku_id")} />
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="buyer_org_id" error={form.formState.errors.buyer_org_id?.message}>
            <Input {...form.register("buyer_org_id")} placeholder="当前买方组织 UUID" />
          </Field>
          <Field label="scenario_code" error={form.formState.errors.scenario_code?.message}>
            <Input {...form.register("scenario_code")} placeholder="S1 / S2 / S3 / S4 / S5" />
          </Field>
        </div>
        <div className="grid gap-3 md:grid-cols-3">
          <Field label="数量" error={form.formState.errors.quantity?.message}>
            <Input type="number" min={1} {...form.register("quantity", { valueAsNumber: true })} />
          </Field>
          <Field label="期限（天）" error={form.formState.errors.term_days?.message}>
            <Input type="number" min={1} {...form.register("term_days", { valueAsNumber: true })} />
          </Field>
          <Field label="订阅/计费配置" error={form.formState.errors.subscription_cadence?.message}>
            <select
              className="h-11 w-full rounded-2xl border border-black/10 bg-white/90 px-4 text-sm"
              {...form.register("subscription_cadence")}
            >
              <option value="none">none</option>
              <option value="monthly">monthly</option>
              <option value="weekly">weekly</option>
              <option value="per_use">per_use</option>
            </select>
          </Field>
        </div>
        <Field label="X-Idempotency-Key" error={form.formState.errors.idempotency_key?.message}>
          <Input {...form.register("idempotency_key")} />
        </Field>
        <div className="grid gap-3 md:grid-cols-3">
          <CheckField form={form} name="confirm_rights" label="确认权利、地域和用途边界" />
          <CheckField form={form} name="confirm_snapshot" label="确认 SKU/价格/模板进入快照" />
          <CheckField form={form} name="confirm_audit" label="确认下单动作写入审计" />
        </div>
        <div className="rounded-[24px] bg-black/[0.035] p-4 text-sm text-[var(--ink-soft)]">
          当前选择：{selectedSku?.sku_type ?? "未选择 SKU"} / {watchedScenario || "未选择场景"} /{" "}
          {selectedTemplate?.template_code ?? "模板未匹配"}
        </div>
        <Button type="submit" disabled={disabled || !selectedSku || !selectedTemplate}>
          {disabled ? "正在创建订单..." : "创建订单"}
        </Button>
      </form>
    </Card>
  );
}

function OrderCreateRiskCard({
  selectedSku,
  selectedTemplate,
  scenarioNeedsChoice,
  form,
}: {
  selectedSku?: ProductSku;
  selectedTemplate: OrderTemplate | null;
  scenarioNeedsChoice: boolean;
  form: ReturnType<typeof useForm<OrderCreateFormValues>>;
}) {
  const idempotencyKey = useWatch({
    control: form.control,
    name: "idempotency_key",
  });
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<ShieldCheck className="size-5" />}
        title="风险提示与审计留痕"
        description="订单创建会冻结价格快照并写入 trade.order.create 审计。"
      />
      <InfoTile label="所选 SKU" value={selectedSku?.sku_type ?? "未选择"} />
      <InfoTile label="场景消歧" value={scenarioNeedsChoice ? "必须显式选择 scenario_code" : "唯一映射"} />
      <InfoTile label="模板匹配" value={selectedTemplate?.template_code ?? "未匹配"} />
      <InfoTile label="幂等键" value={idempotencyKey || "提交时生成"} />
      <CardDescription className="text-[var(--warning-ink)]">
        页面不会直连 PostgreSQL / Kafka / OpenSearch / Redis / Fabric；所有创建请求经 `/api/platform/api/v1/orders`。
      </CardDescription>
    </Card>
  );
}

function CreatedOrderCard({
  result,
}: {
  result: { order: CreatedOrder; idempotencyKey: string };
}) {
  const scenario = result.order.price_snapshot.scenario_snapshot;
  return (
    <Card className="space-y-4 border-[var(--accent-strong)]">
      <PanelTitle
        icon={<CheckCircle2 className="size-5" />}
        title="订单创建成功"
        description="关键输出：order_id / buyer_deposit / price_snapshot。"
      />
      <InfoTile label="order_id" value={result.order.order_id} />
      <InfoTile label="buyer_deposit 估算" value={estimateBuyerDeposit(result.order.amount, result.order.currency_code)} />
      <InfoTile label="Idempotency-Key" value={result.idempotencyKey} />
      <InfoTile label="price_snapshot.sku_type" value={result.order.price_snapshot.sku_type} />
      <InfoTile label="scenario_snapshot" value={scenario ? `${scenario.scenario_code} / ${scenario.selected_sku_role}` : "未返回"} />
      <Button asChild>
        <Link href={`/trade/orders/${result.order.order_id}` as Route}>
          查看订单详情 <ArrowRight className="size-4" />
        </Link>
      </Button>
    </Card>
  );
}

function OrderSummaryCard({ order }: { order: OrderDetail }) {
  const scenario = order.price_snapshot?.scenario_snapshot;
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<ReceiptText className="size-5" />}
        title="订单基本信息"
        description="订单主状态和分层状态均来自 platform-core 订单详情接口。"
      />
      <div className="grid gap-3 md:grid-cols-4">
        <InfoTile label="order_id" value={order.order_id} />
        <InfoTile label="current_state" value={`${order.current_state} / ${orderStatusLabel(order.current_state)}`} />
        <InfoTile label="payment_status" value={order.payment_status} />
        <InfoTile label="金额" value={formatMoney(order.amount, order.currency_code)} />
      </div>
      <div className="grid gap-3 md:grid-cols-4">
        <InfoTile label="buyer_org_id" value={order.buyer_org_id} />
        <InfoTile label="seller_org_id" value={order.seller_org_id} />
        <InfoTile label="product_id" value={order.product_id} />
        <InfoTile label="sku_id" value={order.sku_id} />
      </div>
      <div className="rounded-[24px] bg-black/[0.035] p-4">
        <div className="text-sm font-semibold text-[var(--ink-strong)]">场景与 SKU 快照</div>
        <div className="mt-3 grid gap-3 md:grid-cols-4">
          <InfoTile label="sku_type" value={order.price_snapshot?.sku_type ?? "未返回"} />
          <InfoTile label="scenario_code" value={scenario?.scenario_code ?? "未返回"} />
          <InfoTile label="selected_sku_role" value={scenario?.selected_sku_role ?? "未返回"} />
          <InfoTile label="per_sku_snapshot_required" value={String(scenario?.per_sku_snapshot_required ?? "未返回")} />
        </div>
      </div>
    </Card>
  );
}

function OrderTimelineCard({
  order,
  lifecycle,
}: {
  order: OrderDetail;
  lifecycle: OrderLifecycleSnapshots | null;
}) {
  const rows = [
    ["主状态", order.current_state, order.updated_at],
    ["支付", lifecycle?.order.payment.current_status ?? order.payment_status, lifecycle?.order.payment.buyer_locked_at ?? "未返回"],
    ["交付", lifecycle?.delivery?.current_status ?? order.delivery_status, lifecycle?.delivery?.updated_at ?? "未返回"],
    ["验收", lifecycle?.order.acceptance.current_status ?? order.acceptance_status, lifecycle?.order.acceptance.accepted_at ?? "未返回"],
    ["结算", lifecycle?.order.settlement.current_status ?? order.settlement_status, lifecycle?.order.settlement.settled_at ?? "未返回"],
    ["争议", lifecycle?.order.dispute.current_status ?? order.dispute_status, lifecycle?.order.dispute.last_reason_code ?? "无"],
  ] as const;
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<GitBranch className="size-5" />}
        title="状态时间线"
        description="不发明新状态名，直接展示订单主状态和生命周期对象状态。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        {rows.map(([label, status, time], index) => (
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

function OrderRelationsCard({
  order,
  lifecycle,
}: {
  order: OrderDetail;
  lifecycle: OrderLifecycleSnapshots | null;
}) {
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<FileSearch className="size-5" />}
        title="交付、验收、账单与争议摘要"
        description="详情页只展示正式 relation 与 lifecycle 字段，后续操作进入对应页面。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <InfoTile label="合同" value={order.relations.contract?.contract_status ?? lifecycle?.contract?.contract_status ?? "未返回"} />
        <InfoTile label="授权数量" value={String(order.relations.authorizations.length)} />
        <InfoTile label="交付数量" value={String(order.relations.deliveries.length)} />
        <InfoTile label="争议数量" value={String(order.relations.disputes.length)} />
      </div>
      <div className="grid gap-3 md:grid-cols-2">
        <RelationList title="账单事件" items={order.relations.billing.billing_events.map((item) => `${item.event_type} / ${item.amount} ${item.currency_code}`)} />
        <RelationList title="交付记录" items={order.relations.deliveries.map((item) => `${item.delivery_type} / ${item.current_status} / receipt=${item.receipt_hash ?? "未返回"}`)} />
      </div>
    </Card>
  );
}

function OrderTrustCard({
  order,
  lifecycle,
}: {
  order: OrderDetail;
  lifecycle: OrderLifecycleSnapshots | null;
}) {
  const txHash =
    order.relations.deliveries.find((item) => item.delivery_commit_hash)?.delivery_commit_hash ??
    order.relations.deliveries.find((item) => item.receipt_hash)?.receipt_hash ??
    lifecycle?.delivery?.receipt_hash ??
    "当前订单详情接口未返回";
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<Fingerprint className="size-5" />}
        title="审计与链路信任边界"
        description="链相关字段未由当前接口返回时显式标注未返回，不伪造 tx_hash 或投影状态。"
      />
      <div className="grid gap-3 md:grid-cols-4">
        <InfoTile label="request_id" value="由每次 API 响应/错误返回；详情对象未单列" />
        <InfoTile label="tx_hash / receipt_hash" value={txHash} />
        <InfoTile label="链状态" value={txHash === "当前订单详情接口未返回" ? "未返回" : "receipt_returned"} />
        <InfoTile label="投影状态" value="请通过审计/ops 联查页核验" />
      </div>
      <div className="flex flex-wrap gap-2">
        <Button asChild variant="secondary">
          <Link href={`/developer/trace?order_id=${order.order_id}` as Route}>
            开发者联查
          </Link>
        </Button>
        <Button asChild variant="secondary">
          <Link href={`/ops/audit/trace?order_id=${order.order_id}` as Route}>
            审计联查
          </Link>
        </Button>
      </div>
    </Card>
  );
}

function OrderDetailActionsCard({
  order,
  skuType,
  canCancel,
  isCancelable,
  cancelPending,
  onCancel,
}: {
  order: OrderDetail;
  skuType: string;
  canCancel: boolean;
  isCancelable: boolean;
  cancelPending: boolean;
  onCancel: () => void;
}) {
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<LockKeyhole className="size-5" />}
        title="主动作与下游入口"
        description="取消订单为 trade.order.cancel，交付/验收/账单/争议进入后续正式页面。"
      />
      <Button
        type="button"
        variant="warning"
        disabled={!canCancel || !isCancelable || cancelPending}
        onClick={onCancel}
      >
        {cancelPending ? "正在取消..." : "取消订单"}
      </Button>
      <CardDescription>
        {!canCancel
          ? "当前主体无 trade.order.cancel 权限。"
          : !isCancelable
            ? `当前状态 ${order.current_state} 不在可取消态。`
            : "提交时自动携带 X-Idempotency-Key，并写 trade.order.cancel 审计。"}
      </CardDescription>
      <div className="grid gap-2">
        <ActionLink href={`/trade/orders/${order.order_id}/payment-lock`} label="支付锁定" />
        <ActionLink href={deliveryRouteForSku(skuType, order.order_id)} label={`交付入口 / ${skuType}`} />
        <ActionLink href={`/delivery/orders/${order.order_id}/acceptance`} label="验收页" />
        <ActionLink href="/billing" label="账单中心" />
        <ActionLink href="/support/cases/new" label="争议提交" />
      </div>
    </Card>
  );
}

function CancelResultCard({
  result,
}: {
  result: { order: CanceledOrder; idempotencyKey: string };
}) {
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<RefreshCcw className="size-5" />}
        title="取消动作已返回"
        description="取消结果来自正式 cancelOrder 响应。"
      />
      <InfoTile label="previous_state" value={result.order.previous_state} />
      <InfoTile label="current_state" value={result.order.current_state} />
      <InfoTile label="refund_branch" value={result.order.refund_branch} />
      <InfoTile label="Idempotency-Key" value={result.idempotencyKey} />
    </Card>
  );
}

function OrderEntryState({ templates }: { templates: OrderTemplate[] }) {
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<Sparkles className="size-5" />}
        title="请选择商品后下单"
        description="当前未携带 productId。请从首页、搜索页或商品详情进入，下方保留五条官方链路入口。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-5">
        {templates.map((template) => (
          <Link
            key={template.scenario_code}
            href={`/search?q=${encodeURIComponent(template.scenario_name)}` as Route}
            className="rounded-[24px] bg-black/[0.035] p-4 transition hover:bg-[var(--accent-soft)]"
          >
            <Badge>{template.scenario_code}</Badge>
            <div className="mt-3 text-sm font-semibold text-[var(--ink-strong)]">
              {template.scenario_name}
            </div>
            <div className="mt-2 text-xs text-[var(--ink-soft)]">
              {template.primary_sku} + {template.supplementary_skus.join(" / ")}
            </div>
          </Link>
        ))}
      </div>
    </Card>
  );
}

function OrderContextCard({
  subject,
  sessionMode,
  compact = false,
}: {
  subject: SessionSubject | PortalSessionPreview | null | undefined;
  sessionMode: "guest" | "bearer" | "local";
  compact?: boolean;
}) {
  return (
    <Card className={cn("space-y-4", compact && "bg-white/78")}>
      <PanelTitle
        icon={<LockKeyhole className="size-5" />}
        title="当前主体访问上下文"
        description="敏感交易页面必须展示主体、角色、租户和作用域。"
      />
      <div className="grid gap-3">
        <InfoTile label="主体" value={subject?.display_name ?? subject?.login_id ?? subject?.user_id ?? "游客"} />
        <InfoTile label="角色" value={subject?.roles?.join(" / ") || "visitor"} />
        <InfoTile label="租户/组织" value={subject?.tenant_id ?? subject?.org_id ?? "public"} />
        <InfoTile label="作用域" value={subject?.auth_context_level ?? "public"} />
        <InfoTile label="会话模式" value={sessionMode} />
      </div>
    </Card>
  );
}

function OrderPermissionState({
  title,
  requiredRoles,
  subject,
  message,
}: {
  title: string;
  requiredRoles: readonly string[];
  subject: SessionSubject | PortalSessionPreview | null | undefined;
  message: string;
}) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--warning-ink)]">
        <Ban className="size-8" />
        <CardTitle>{title}</CardTitle>
        <CardDescription className="text-[var(--warning-ink)]">{message}</CardDescription>
        <div className="text-sm">
          需要角色：{requiredRoles.join(" / ")}；当前角色：
          {subject?.roles?.join(" / ") || "visitor"}；匹配结果：
          {hasAnyRole(subject?.roles, requiredRoles) ? "已匹配" : "未匹配"}
        </div>
      </div>
    </Card>
  );
}

function OrderLoadingState({ title }: { title: string }) {
  return (
    <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <LoaderCircle className="size-8 animate-spin" />
        <CardTitle>{title}</CardTitle>
        <CardDescription>正在通过 `/api/platform/**` 读取 platform-core 正式 API。</CardDescription>
      </div>
    </Card>
  );
}

function OrderEmptyState({ title, message }: { title: string; message: string }) {
  return (
    <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <Boxes className="size-8" />
        <CardTitle>{title}</CardTitle>
        <CardDescription>{message}</CardDescription>
      </div>
    </Card>
  );
}

function OrderErrorState({
  title,
  message,
  onRetry,
}: {
  title: string;
  message: string;
  onRetry: () => void;
}) {
  return (
    <Card className="border-[var(--danger-ring)] bg-[var(--danger-soft)]">
      <div className="flex flex-col items-start gap-3 text-[var(--danger-ink)]">
        <AlertTriangle className="size-8" />
        <CardTitle className="text-[var(--danger-ink)]">{title}</CardTitle>
        <CardDescription className="text-[var(--danger-ink)]">{message}</CardDescription>
        <Button type="button" variant="warning" onClick={onRetry}>
          重试
        </Button>
      </div>
    </Card>
  );
}

function ScenarioSkuBadge({ sku, role }: { sku: string; role: string }) {
  return (
    <span className="rounded-full bg-white/75 px-2.5 py-1 text-[11px] font-semibold text-[var(--ink-soft)]">
      {role} {sku}
    </span>
  );
}

function RelationList({ title, items }: { title: string; items: string[] }) {
  return (
    <div className="rounded-[24px] bg-black/[0.035] p-4">
      <div className="text-sm font-semibold text-[var(--ink-strong)]">{title}</div>
      <div className="mt-3 space-y-2">
        {items.length ? (
          items.map((item) => (
            <div key={item} className="rounded-2xl bg-white/70 px-3 py-2 text-xs text-[var(--ink-soft)]">
              {item}
            </div>
          ))
        ) : (
          <div className="text-xs text-[var(--ink-subtle)]">当前接口未返回记录。</div>
        )}
      </div>
    </div>
  );
}

function ActionLink({ href, label }: { href: string; label: string }) {
  return (
    <Button asChild variant="secondary" size="sm" className="justify-start">
      <Link href={href as Route}>{label}</Link>
    </Button>
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
      <div className="rounded-2xl bg-[var(--accent-soft)] p-3 text-[var(--accent-strong)]">
        {icon}
      </div>
      <div>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </div>
    </div>
  );
}

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-[22px] bg-white/72 p-4">
      <div className="text-[11px] uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 break-words text-sm font-medium text-[var(--ink-strong)]">
        {value}
      </div>
    </div>
  );
}

function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <label className="block space-y-2">
      <span className="text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </span>
      {children}
      {error ? <span className="text-xs text-[var(--danger-ink)]">{error}</span> : null}
    </label>
  );
}

function CheckField({
  form,
  name,
  label,
}: {
  form: ReturnType<typeof useForm<OrderCreateFormValues>>;
  name: "confirm_rights" | "confirm_snapshot" | "confirm_audit";
  label: string;
}) {
  const error = form.formState.errors[name]?.message;
  return (
    <label className="rounded-[22px] bg-black/[0.035] p-4 text-sm text-[var(--ink-soft)]">
      <input type="checkbox" className="mr-2" {...form.register(name)} />
      {label}
      {error ? <span className="mt-2 block text-xs text-[var(--danger-ink)]">{error}</span> : null}
    </label>
  );
}
