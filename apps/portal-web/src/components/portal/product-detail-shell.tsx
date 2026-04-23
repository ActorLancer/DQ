"use client";

import type {
  ProductDetailResponse,
  RecommendationsResponse,
  SellerProfileResponse,
} from "@datab/sdk-ts";
import { PlatformApiError } from "@datab/sdk-ts";
import { useQuery } from "@tanstack/react-query";
import {
  AlertTriangle,
  BadgeCheck,
  Ban,
  Boxes,
  Building2,
  CheckCircle2,
  ClipboardCheck,
  FileSearch,
  Fingerprint,
  LoaderCircle,
  LockKeyhole,
  PackageCheck,
  ShieldCheck,
  ShoppingCart,
  Sparkles,
  Tags,
} from "lucide-react";
import { motion } from "motion/react";
import type { Route } from "next";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { type ReactNode } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import {
  getOrderGate,
  metadataDisplayEntries,
  productStatusLabel,
  readMetadataText,
  readMetadataTextArray,
} from "@/lib/product-detail-view";
import type { PortalSessionPreview } from "@/lib/session";
import { cn, formatList } from "@/lib/utils";

import {
  PreviewStateControls,
  ScaffoldPill,
  getPreviewState,
} from "./state-preview";

const sdk = createBrowserSdk();
const meta = portalRouteMap.product_detail;

type Product = ProductDetailResponse["data"];
type Seller = SellerProfileResponse["data"];
type RecommendationItem = RecommendationsResponse["data"]["items"][number];

type ProductDetailShellProps = {
  productId: string;
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

export function ProductDetailShell({
  productId,
  sessionMode,
  initialSubject,
}: ProductDetailShellProps) {
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);
  const canRead = sessionMode === "bearer" && Boolean(initialSubject);
  const canRecommend = Boolean(
    initialSubject?.roles.some((role) =>
      ["platform_admin", "buyer_operator"].includes(role),
    ),
  );
  const productQuery = useQuery({
    queryKey: ["portal", "product-detail", productId],
    enabled: canRead && preview === "ready",
    queryFn: () => sdk.catalog.getProductDetail({ id: productId }),
  });
  const product = productQuery.data?.data;
  const sellerOrgId = product?.seller_org_id;
  const sellerQuery = useQuery({
    queryKey: ["portal", "seller-profile", sellerOrgId],
    enabled: canRead && preview === "ready" && Boolean(sellerOrgId),
    queryFn: () => sdk.catalog.getSellerProfile({ orgId: sellerOrgId ?? "" }),
  });
  const recommendationQuery = useQuery({
    queryKey: ["portal", "product-recommendations", productId, initialSubject?.tenant_id],
    enabled: canRead && canRecommend && preview === "ready" && Boolean(product),
    queryFn: () =>
      sdk.recommendation.getRecommendations({
        placement_code: "product_detail_bundle",
        subject_scope: initialSubject?.tenant_id ? "organization" : "user",
        subject_org_id: initialSubject?.tenant_id,
        subject_user_id: initialSubject?.user_id,
        context_entity_scope: "product",
        context_entity_id: productId,
        limit: 4,
      }),
  });

  return (
    <div className="space-y-6">
      <ProductHeader
        productId={productId}
        preview={preview}
        canRead={canRead}
        sessionMode={sessionMode}
        subject={initialSubject}
        product={product}
      />

      {preview === "loading" ? (
        <ProductLoadingState />
      ) : preview === "empty" ? (
        <ProductEmptyState productId={productId} />
      ) : preview === "error" ? (
        <ProductErrorState
          title="商品详情读取失败"
          message="CAT_VALIDATION_FAILED: 详情页必须承接 platform-core 统一错误码与 request_id。"
          onRetry={() => productQuery.refetch()}
        />
      ) : preview === "forbidden" || !canRead ? (
        <ProductPermissionState sessionMode={sessionMode} subject={initialSubject} />
      ) : productQuery.isPending ? (
        <ProductLoadingState />
      ) : productQuery.isError ? (
        <ProductErrorState
          title="商品详情请求失败"
          message={describeError(productQuery.error)}
          onRetry={() => productQuery.refetch()}
        />
      ) : product ? (
        <ProductContent
          product={product}
          seller={sellerQuery.data?.data}
          sellerError={sellerQuery.error}
          sellerPending={sellerQuery.isPending}
          recommendations={recommendationQuery.data?.data.items ?? []}
          recommendationError={recommendationQuery.error}
          recommendationPending={recommendationQuery.isPending}
          canRecommend={canRecommend}
        />
      ) : (
        <ProductEmptyState productId={productId} />
      )}
    </div>
  );
}

function ProductHeader({
  productId,
  preview,
  canRead,
  sessionMode,
  subject,
  product,
}: {
  productId: string;
  preview: string;
  canRead: boolean;
  sessionMode: "guest" | "bearer" | "local";
  subject: PortalSessionPreview | null;
  product?: Product;
}) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28 }}
      className="grid gap-4 xl:grid-cols-[1.45fr_0.85fr]"
    >
      <Card className="overflow-hidden bg-[radial-gradient(circle_at_88%_18%,rgba(15,107,137,0.18),transparent_26%),linear-gradient(135deg,rgba(255,255,255,0.97),rgba(230,242,236,0.88),rgba(242,236,222,0.7))]">
        <div className="space-y-5">
          <div className="flex flex-wrap gap-2">
            <ScaffoldPill>{meta.group}</ScaffoldPill>
            <ScaffoldPill>{meta.key}</ScaffoldPill>
            <ScaffoldPill tone="warning">preview:{preview}</ScaffoldPill>
            <ScaffoldPill>{canRead ? "Bearer API ready" : "requires Bearer"}</ScaffoldPill>
          </div>
          <div className="max-w-4xl">
            <Badge>Catalog Product Detail</Badge>
            <h1 className="mt-3 text-3xl font-semibold tracking-[-0.045em] text-[var(--ink-strong)] md:text-5xl">
              {product?.title ?? "商品详情页"}
            </h1>
            <CardDescription className="mt-4 text-base">
              元信息、卖方、SKU、价格、样例预览、审计提示与下单入口均绑定
              `platform-core` 正式 API；浏览器端只经过 `/api/platform/**` 受控代理。
            </CardDescription>
          </div>
          <div className="grid gap-3 md:grid-cols-4">
            <HeaderMetric label="product_id" value={productId} />
            <HeaderMetric label="查看权限" value={meta.viewPermission} />
            <HeaderMetric label="主动作权限" value={formatList(meta.primaryPermissions)} />
            <HeaderMetric label="会话模式" value={sessionMode} />
          </div>
          <PreviewStateControls />
        </div>
      </Card>

      <Card>
        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <div className="rounded-2xl bg-[var(--accent-soft)] p-3 text-[var(--accent-strong)]">
              <LockKeyhole className="size-5" />
            </div>
            <div>
              <CardTitle>当前主体访问上下文</CardTitle>
              <CardDescription>敏感页面显式展示主体、角色、租户与作用域。</CardDescription>
            </div>
          </div>
          <div className="grid gap-3">
            <ContextRow label="主体" value={subject?.display_name ?? subject?.login_id ?? subject?.user_id ?? "未建立 Bearer 会话"} />
            <ContextRow label="角色" value={subject?.roles.join(", ") || "无"} />
            <ContextRow label="租户/组织" value={subject?.tenant_id ?? subject?.org_id ?? "无"} />
            <ContextRow label="作用域" value={subject?.auth_context_level ?? "无"} />
          </div>
        </div>
      </Card>
    </motion.section>
  );
}

function ProductContent({
  product,
  seller,
  sellerError,
  sellerPending,
  recommendations,
  recommendationError,
  recommendationPending,
  canRecommend,
}: {
  product: Product;
  seller?: Seller;
  sellerError: unknown;
  sellerPending: boolean;
  recommendations: RecommendationItem[];
  recommendationError: unknown;
  recommendationPending: boolean;
  canRecommend: boolean;
}) {
  return (
    <div className="grid gap-4 2xl:grid-cols-[minmax(0,1fr)_390px]">
      <main className="space-y-4">
        <OverviewCard product={product} />
        <SkuMatrix product={product} />
        <MetadataAndSample product={product} />
        <EvidenceCard product={product} />
        <RecommendationCard
          items={recommendations}
          pending={recommendationPending}
          error={recommendationError}
          canRecommend={canRecommend}
        />
      </main>

      <aside className="space-y-4">
        <OrderCard product={product} />
        <SellerCard seller={seller} pending={sellerPending} error={sellerError} />
        <ContractBoundaryCard product={product} />
      </aside>
    </div>
  );
}

function OverviewCard({ product }: { product: Product }) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28 }}
    >
      <Card className="bg-white/90">
        <div className="grid gap-5 xl:grid-cols-[1fr_280px]">
          <div className="space-y-4">
            <div className="flex flex-wrap gap-2">
              <StatusBadge status={product.status} />
              <Badge className="bg-slate-100 text-slate-700">index:{product.index_sync_status}</Badge>
              <Badge className="bg-slate-100 text-slate-700">doc:v{product.search_document_version}</Badge>
              {product.data_classification ? (
                <Badge className="bg-amber-50 text-amber-700">{product.data_classification}</Badge>
              ) : null}
            </div>
            <div>
              <CardTitle className="text-3xl">{product.title}</CardTitle>
              <CardDescription className="mt-2 text-base">
                {product.subtitle || product.description || "当前商品详情接口未返回摘要描述。"}
              </CardDescription>
            </div>
            <div className="flex flex-wrap gap-2">
              <Tag>{product.category}</Tag>
              <Tag>{product.product_type}</Tag>
              <Tag>{product.delivery_type}</Tag>
              {product.industry ? <Tag>{product.industry}</Tag> : null}
              {product.use_cases.map((useCase) => (
                <Tag key={useCase}>{useCase}</Tag>
              ))}
            </div>
          </div>

          <div className="rounded-[26px] bg-[var(--panel-muted)] p-4">
            <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
              价格快照
            </div>
            <div className="mt-3 text-3xl font-semibold text-[var(--ink-strong)]">
              {product.currency_code} {product.price}
            </div>
            <div className="mt-4 grid gap-2">
              <MiniRow label="price_mode" value={product.price_mode} />
              <MiniRow label="delivery_type" value={product.delivery_type} />
              <MiniRow label="quality_score" value={product.quality_score ?? "未返回"} />
            </div>
          </div>
        </div>
      </Card>
    </motion.section>
  );
}

function OrderCard({ product }: { product: Product }) {
  const gate = getOrderGate(product);
  const orderHref = `/trade/orders/new?productId=${product.product_id}` as Route;

  return (
    <Card className="border-[var(--accent-soft)] bg-[linear-gradient(180deg,rgba(255,255,255,0.95),rgba(232,242,236,0.86))]">
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <div className="rounded-2xl bg-[var(--accent-soft)] p-3 text-[var(--accent-strong)]">
            <ShoppingCart className="size-5" />
          </div>
          <div>
            <CardTitle>下单入口</CardTitle>
            <CardDescription>权限：`trade.order.create`；状态条件：商品 `listed`。</CardDescription>
          </div>
        </div>
        <div className="rounded-2xl bg-white/70 p-4 text-sm text-[var(--ink-soft)]">
          {gate.reason}
        </div>
        {gate.enabled ? (
          <Button asChild size="lg" className="w-full">
            <Link href={orderHref}>
              进入下单页
              <ShoppingCart className="size-4" />
            </Link>
          </Button>
        ) : (
          <Button size="lg" className="w-full" disabled>
            暂不可下单
          </Button>
        )}
        <div className="rounded-2xl border border-[var(--warning-ring)] bg-[var(--warning-soft)] px-4 py-3 text-sm text-[var(--warning-ink)]">
          关键动作将写入交易与审计链路；后续写操作必须携带 `Idempotency-Key`。
        </div>
      </div>
    </Card>
  );
}

function SkuMatrix({ product }: { product: Product }) {
  return (
    <Card>
      <div className="space-y-4">
        <SectionTitle icon={<Tags className="size-5" />} title="SKU 与合同策略摘要" description="SKU 真值严格来自 `skus[].sku_type`，不得用 product_type 或 delivery_type 替代。" />
        {product.skus.length ? (
          <div className="grid gap-3">
            {product.skus.map((sku) => (
              <div
                key={sku.sku_id}
                className="grid gap-3 rounded-[24px] border border-black/5 bg-[var(--panel-muted)] p-4 lg:grid-cols-[170px_1fr]"
              >
                <div>
                  <Badge>{sku.sku_type}</Badge>
                  <div className="mt-2 break-all font-mono text-xs text-[var(--ink-subtle)]">
                    {sku.sku_code}
                  </div>
                </div>
                <div className="grid gap-2 md:grid-cols-3">
                  <MiniRow label="billing" value={sku.billing_mode} />
                  <MiniRow label="trade" value={sku.trade_mode} />
                  <MiniRow label="acceptance" value={sku.acceptance_mode} />
                  <MiniRow label="refund" value={sku.refund_mode} />
                  <MiniRow label="unit" value={sku.unit_name ?? "未返回"} />
                  <MiniRow label="status" value={sku.status} />
                </div>
              </div>
            ))}
          </div>
        ) : (
          <InlineState icon={<Boxes className="size-8" />} title="未返回 SKU" description="商品详情接口当前未返回 SKU 记录，页面不以前端套餐编码补造 sku_type。" />
        )}
      </div>
    </Card>
  );
}

function MetadataAndSample({ product }: { product: Product }) {
  const sampleSummary = readMetadataText(product.metadata, "sample_summary");
  const sampleHash =
    readMetadataText(product.metadata, "sample_hash") ??
    readMetadataText(product.metadata, "sample_sha256");
  const fullHash = readMetadataText(product.metadata, "full_hash");
  const fieldSummary = readMetadataText(product.metadata, "field_summary");
  const fieldNames = readMetadataTextArray(product.metadata, "field_names");
  const qualityReportId = readMetadataText(product.metadata, "quality_report_id");
  const processingChain = readMetadataText(product.metadata, "processing_chain_summary");
  const dataContractId = readMetadataText(product.metadata, "data_contract_id");
  const entries = metadataDisplayEntries(product.metadata);

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <Card>
        <div className="space-y-4">
          <SectionTitle icon={<FileSearch className="size-5" />} title="样例预览与字段结构" description="只展示后端返回的摘要/哈希，不暴露真实对象路径。" />
          <div className="grid gap-3">
            <MiniRow label="sample_summary" value={sampleSummary ?? "接口未返回样本摘要"} />
            <MiniRow label="sample_hash" value={sampleHash ?? "接口未返回样本哈希"} />
            <MiniRow label="full_hash" value={fullHash ?? "接口未返回全量哈希"} />
            <MiniRow label="field_summary" value={fieldSummary ?? "接口未返回字段结构摘要"} />
          </div>
          {fieldNames.length ? (
            <div className="flex flex-wrap gap-2">
              {fieldNames.map((field) => (
                <Tag key={field}>{field}</Tag>
              ))}
            </div>
          ) : (
            <CardDescription>字段名列表未在当前详情 schema 中返回。</CardDescription>
          )}
        </div>
      </Card>

      <Card>
        <div className="space-y-4">
          <SectionTitle icon={<ClipboardCheck className="size-5" />} title="质量、加工与契约摘要" description="质量报告、加工责任链和数据契约均从 metadata 读取；缺失时显式展示空态。" />
          <div className="grid gap-3">
            <MiniRow label="quality_report_id" value={qualityReportId ?? "未返回"} />
            <MiniRow label="quality_score" value={product.quality_score ?? "未返回"} />
            <MiniRow label="processing_chain" value={processingChain ?? "未返回"} />
            <MiniRow label="data_contract_id" value={dataContractId ?? "未返回"} />
          </div>
        </div>
      </Card>

      <Card className="xl:col-span-2">
        <div className="space-y-4">
          <SectionTitle icon={<ShieldCheck className="size-5" />} title="元信息详情" description="展示 ProductDetail.metadata 顶层字段；对象路径类字段仅显示脱敏占位。" />
          {entries.length ? (
            <div className="grid gap-3 md:grid-cols-2">
              {entries.map((entry) => (
                <MiniRow
                  key={entry.key}
                  label={entry.key}
                  value={entry.value}
                  muted={entry.hidden}
                />
              ))}
            </div>
          ) : (
            <InlineState icon={<Boxes className="size-8" />} title="元信息为空" description="当前商品没有返回 metadata 顶层字段。" />
          )}
        </div>
      </Card>
    </div>
  );
}

function EvidenceCard({ product }: { product: Product }) {
  return (
    <Card>
      <div className="space-y-4">
        <SectionTitle icon={<Fingerprint className="size-5" />} title="审计 / 存证摘要" description="链相关详情字段当前未进入 ProductDetail schema；本页展示可核对的请求对象与投影状态。" />
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <MiniRow label="asset_id" value={product.asset_id} />
          <MiniRow label="asset_version_id" value={product.asset_version_id} />
          <MiniRow label="seller_org_id" value={product.seller_org_id} />
          <MiniRow label="index_sync_status" value={product.index_sync_status} />
          <MiniRow label="search_document_version" value={String(product.search_document_version)} />
          <MiniRow label="created_at" value={product.created_at} />
          <MiniRow label="updated_at" value={product.updated_at} />
          <MiniRow label="request_id" value="由 platform-core 响应 / 审计日志回查" />
        </div>
      </div>
    </Card>
  );
}

function SellerCard({
  seller,
  pending,
  error,
}: {
  seller?: Seller;
  pending: boolean;
  error: unknown;
}) {
  if (pending) {
    return (
      <Card>
        <InlineState icon={<LoaderCircle className="size-8 animate-spin" />} title="正在读取卖方信息" description="GET /api/v1/sellers/{orgId}/profile" />
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="border-[var(--danger-ring)] bg-[var(--danger-soft)]">
        <InlineState icon={<AlertTriangle className="size-8" />} title="卖方信息读取失败" description={describeError(error)} tone="danger" />
      </Card>
    );
  }

  if (!seller) {
    return (
      <Card>
        <InlineState icon={<Building2 className="size-8" />} title="卖方信息未返回" description="商品详情已经返回，但卖方 profile 查询暂无数据。" />
      </Card>
    );
  }

  return (
    <Card>
      <div className="space-y-4">
        <SectionTitle icon={<Building2 className="size-5" />} title="供方信息" description="卖方 profile 来自正式目录接口，不读取底层搜索索引或数据库。" />
        <div>
          <CardTitle>{seller.org_name}</CardTitle>
          <CardDescription className="mt-1">
            {seller.description || "当前卖方 profile 未返回说明。"}
          </CardDescription>
        </div>
        <div className="grid gap-3">
          <MiniRow label="org_id" value={seller.org_id} />
          <MiniRow label="org_type" value={seller.org_type} />
          <MiniRow label="status" value={seller.status} />
          <MiniRow label="country / region" value={[seller.country_code, seller.region_code].filter(Boolean).join(" / ") || "未返回"} />
          <MiniRow label="credit / risk" value={`${seller.credit_level} / ${seller.risk_level}`} />
          <MiniRow label="reputation_score" value={seller.reputation_score} />
          <MiniRow label="listed_product_count" value={String(seller.listed_product_count)} />
          <MiniRow label="seller_index" value={`${seller.index_sync_status} / v${seller.search_document_version}`} />
        </div>
        <div className="flex flex-wrap gap-2">
          {seller.industry_tags.map((tag) => (
            <Tag key={tag}>{tag}</Tag>
          ))}
        </div>
      </div>
    </Card>
  );
}

function ContractBoundaryCard({ product }: { product: Product }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <div className="space-y-3 text-[var(--warning-ink)]">
        <div className="flex items-center gap-3">
          <PackageCheck className="size-5" />
          <CardTitle className="text-[var(--warning-ink)]">权利边界</CardTitle>
        </div>
        <CardDescription className="text-[var(--warning-ink)]">
          `allowed_usage` 为后端返回的正式使用边界；合同、验收、退款、计费摘要优先读取 SKU 策略字段。
        </CardDescription>
        <div className="flex flex-wrap gap-2">
          {product.allowed_usage.length ? (
            product.allowed_usage.map((usage) => (
              <span key={usage} className="rounded-full bg-white/70 px-3 py-1 text-xs font-medium">
                {usage}
              </span>
            ))
          ) : (
            <span className="rounded-full bg-white/70 px-3 py-1 text-xs font-medium">
              未返回 allowed_usage
            </span>
          )}
        </div>
      </div>
    </Card>
  );
}

function RecommendationCard({
  items,
  pending,
  error,
  canRecommend,
}: {
  items: RecommendationItem[];
  pending: boolean;
  error: unknown;
  canRecommend: boolean;
}) {
  return (
    <Card>
      <div className="space-y-4">
        <SectionTitle icon={<Sparkles className="size-5" />} title="相似商品 / 配套服务推荐" description="真实调用 `GET /api/v1/recommendations` 的 product_detail_bundle placement。" />
        {!canRecommend ? (
          <InlineState icon={<Ban className="size-8" />} title="推荐权限态" description="需要 `portal.recommendation.read`，当前主体不满足推荐读取角色。" />
        ) : pending ? (
          <InlineState icon={<LoaderCircle className="size-8 animate-spin" />} title="正在读取推荐" description="等待 recommendation 运行态返回。" />
        ) : error ? (
          <InlineState icon={<AlertTriangle className="size-8" />} title="推荐读取失败" description={describeError(error)} tone="danger" />
        ) : items.length ? (
          <div className="grid gap-3 md:grid-cols-2">
            {items.map((item) => {
              const href =
                item.entity_scope === "product"
                  ? (`/products/${item.entity_id}` as Route)
                  : (`/sellers/${item.entity_id}` as Route);
              return (
                <Link
                  key={item.recommendation_result_item_id}
                  href={href}
                  className="rounded-[24px] border border-black/5 bg-[var(--panel-muted)] p-4 transition hover:bg-white"
                >
                  <div className="flex flex-wrap gap-2">
                    <Badge>{item.entity_scope}</Badge>
                    <Badge className="bg-slate-100 text-slate-700">score:{item.final_score.toFixed(2)}</Badge>
                  </div>
                  <CardTitle className="mt-3 text-base">{item.title}</CardTitle>
                  <CardDescription className="mt-1">
                    {item.seller_name ?? "未返回卖方"} / {item.status}
                  </CardDescription>
                </Link>
              );
            })}
          </div>
        ) : (
          <InlineState icon={<Boxes className="size-8" />} title="暂无推荐项" description="推荐接口返回空列表，页面不使用 mock 推荐。" />
        )}
      </div>
    </Card>
  );
}

function ProductPermissionState({
  sessionMode,
  subject,
}: {
  sessionMode: "guest" | "bearer" | "local";
  subject: PortalSessionPreview | null;
}) {
  const message =
    sessionMode === "local"
      ? "本地 Header 登录占位不能作为正式商品详情 Bearer 鉴权。"
      : sessionMode === "bearer" && !subject
        ? "Bearer Token 缺少 user_id / org_id / roles claims，无法建立商品详情访问上下文。"
        : "请先通过 Keycloak / IAM Bearer 会话登录后再查看正式商品详情。";

  return (
    <Card className="flex min-h-[420px] items-center justify-center border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <div className="max-w-xl text-center text-[var(--warning-ink)]">
        <Ban className="mx-auto size-10" />
        <CardTitle className="mt-4 text-[var(--warning-ink)]">商品详情权限态</CardTitle>
        <CardDescription className="mt-3 text-[var(--warning-ink)]">{message}</CardDescription>
        <div className="mt-4 rounded-2xl bg-white/65 px-4 py-3 text-sm">
          需要权限：`catalog.product.read`；主动作权限：`trade.order.create`
        </div>
      </div>
    </Card>
  );
}

function ProductLoadingState() {
  return (
    <Card className="min-h-[420px] bg-white/80">
      <div className="flex h-full min-h-[360px] flex-col justify-center gap-5">
        <div className="flex items-center gap-3 text-[var(--accent-strong)]">
          <LoaderCircle className="size-6 animate-spin" />
          <CardTitle>正在读取商品详情与卖方信息</CardTitle>
        </div>
        <div className="grid gap-3">
          {[0, 1, 2].map((index) => (
            <div
              key={index}
              className="h-28 animate-pulse rounded-[24px] bg-[linear-gradient(90deg,rgba(16,39,51,0.05),rgba(16,39,51,0.1),rgba(16,39,51,0.05))]"
            />
          ))}
        </div>
      </div>
    </Card>
  );
}

function ProductEmptyState({ productId }: { productId: string }) {
  return (
    <Card className="flex min-h-[420px] items-center justify-center bg-[var(--panel-muted)]">
      <div className="max-w-xl text-center">
        <Boxes className="mx-auto size-10 text-[var(--ink-subtle)]" />
        <CardTitle className="mt-4">没有可展示的商品详情</CardTitle>
        <CardDescription className="mt-3">
          `{productId}` 未返回商品详情，或当前主体不可见。页面不使用 fallback/mock 商品数据。
        </CardDescription>
      </div>
    </Card>
  );
}

function ProductErrorState({
  title,
  message,
  onRetry,
}: {
  title: string;
  message: string;
  onRetry: () => void;
}) {
  return (
    <Card className="flex min-h-[420px] items-center justify-center border-[var(--danger-ring)] bg-[var(--danger-soft)]">
      <div className="max-w-2xl text-center text-[var(--danger-ink)]">
        <AlertTriangle className="mx-auto size-10" />
        <CardTitle className="mt-4 text-[var(--danger-ink)]">{title}</CardTitle>
        <CardDescription className="mt-3 text-[var(--danger-ink)]">{message}</CardDescription>
        <Button className="mt-5" variant="secondary" onClick={onRetry}>
          重试
        </Button>
      </div>
    </Card>
  );
}

function SectionTitle({
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
        <CardDescription className="mt-1">{description}</CardDescription>
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const isListed = status === "listed";
  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 rounded-full px-3 py-1 text-xs font-semibold",
        isListed
          ? "bg-emerald-50 text-emerald-700"
          : "bg-[var(--warning-soft)] text-[var(--warning-ink)]",
      )}
    >
      {isListed ? <CheckCircle2 className="size-3.5" /> : <BadgeCheck className="size-3.5" />}
      {productStatusLabel(status)} / {status}
    </span>
  );
}

function HeaderMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[22px] bg-white/70 p-4">
      <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 break-words text-sm font-semibold text-[var(--ink-strong)]">
        {value}
      </div>
    </div>
  );
}

function ContextRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-2xl bg-black/[0.04] px-4 py-3 text-sm">
      <span className="text-[var(--ink-subtle)]">{label}</span>
      <span className="break-all text-right font-medium text-[var(--ink-strong)]">
        {value}
      </span>
    </div>
  );
}

function MiniRow({
  label,
  value,
  muted = false,
}: {
  label: string;
  value: string;
  muted?: boolean;
}) {
  return (
    <div className="rounded-2xl bg-white/70 px-4 py-3">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div
        className={cn(
          "mt-1 break-words text-sm font-semibold text-[var(--ink-strong)]",
          muted && "text-[var(--warning-ink)]",
        )}
      >
        {value}
      </div>
    </div>
  );
}

function InlineState({
  icon,
  title,
  description,
  tone = "default",
}: {
  icon: ReactNode;
  title: string;
  description: string;
  tone?: "default" | "danger";
}) {
  return (
    <div
      className={cn(
        "flex min-h-40 flex-col items-center justify-center gap-3 rounded-[24px] bg-[var(--panel-muted)] p-6 text-center text-[var(--ink-soft)]",
        tone === "danger" && "text-[var(--danger-ink)]",
      )}
    >
      {icon}
      <CardTitle className={tone === "danger" ? "text-[var(--danger-ink)]" : undefined}>
        {title}
      </CardTitle>
      <CardDescription className={tone === "danger" ? "text-[var(--danger-ink)]" : undefined}>
        {description}
      </CardDescription>
    </div>
  );
}

function Tag({ children }: { children: ReactNode }) {
  return (
    <span className="rounded-full bg-[var(--accent-soft)] px-3 py-1 text-xs font-medium text-[var(--accent-strong)]">
      {children}
    </span>
  );
}

function describeError(error: unknown): string {
  if (error instanceof PlatformApiError) {
    return `${error.code}: ${error.message}${
      error.requestId ? ` / request_id=${error.requestId}` : ""
    }`;
  }
  return error instanceof Error ? error.message : "未知商品详情错误";
}
