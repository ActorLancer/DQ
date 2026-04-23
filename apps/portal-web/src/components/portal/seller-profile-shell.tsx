"use client";

import type {
  RecommendationsResponse,
  SellerProfileResponse,
} from "@datab/sdk-ts";
import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";
import { useQuery } from "@tanstack/react-query";
import {
  AlertTriangle,
  BadgeCheck,
  Ban,
  Building2,
  CheckCircle2,
  ExternalLink,
  FileSearch,
  LoaderCircle,
  LockKeyhole,
  MessageSquareText,
  PackageSearch,
  ShieldCheck,
  ShoppingCart,
  Sparkles,
} from "lucide-react";
import { motion } from "motion/react";
import type { Route } from "next";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import type { ReactNode } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import {
  certificationLabel,
  formatReputationScore,
  mergeSellerMarketplaceItems,
  sellerRatingMetric,
  sellerRiskDescriptor,
  type SellerMarketplaceItem,
} from "@/lib/seller-profile-view";
import type { PortalSessionPreview } from "@/lib/session";
import { cn, formatList } from "@/lib/utils";

import {
  PreviewStateControls,
  ScaffoldPill,
  getPreviewState,
} from "./state-preview";

const sdk = createBrowserSdk();
const meta = portalRouteMap.seller_profile;
const sellerReadRoles = [
  "platform_admin",
  "tenant_admin",
  "seller_operator",
  "buyer_operator",
  "tenant_audit_readonly",
];
const recommendationReadRoles = [
  "platform_admin",
  "tenant_admin",
  "seller_operator",
  "buyer_operator",
];

type Seller = SellerProfileResponse["data"];
type RecommendationItem = RecommendationsResponse["data"]["items"][number];

type SellerProfileShellProps = {
  orgId: string;
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

export function SellerProfileShell({
  orgId,
  sessionMode,
  initialSubject,
}: SellerProfileShellProps) {
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);
  const canRead =
    sessionMode === "bearer" &&
    hasAnyRole(initialSubject, sellerReadRoles);
  const canRecommend = hasAnyRole(initialSubject, recommendationReadRoles);

  const sellerQuery = useQuery({
    queryKey: ["portal", "seller-profile", orgId],
    enabled: canRead && preview === "ready",
    queryFn: () => sdk.catalog.getSellerProfile({ orgId }),
  });
  const seller = sellerQuery.data?.data;

  const recommendationQuery = useQuery({
    queryKey: ["portal", "seller-profile-featured", orgId, initialSubject?.tenant_id],
    enabled: canRead && canRecommend && preview === "ready",
    queryFn: () =>
      sdk.recommendation.getRecommendations({
        placement_code: "seller_profile_featured",
        subject_scope: initialSubject?.tenant_id ? "organization" : "user",
        subject_org_id: initialSubject?.tenant_id,
        subject_user_id: initialSubject?.user_id,
        context_entity_scope: "seller",
        context_entity_id: orgId,
        limit: 8,
      }),
  });

  return (
    <div className="space-y-6">
      <SellerHeader
        orgId={orgId}
        preview={preview}
        canRead={canRead}
        sessionMode={sessionMode}
        subject={initialSubject}
        seller={seller}
      />

      {preview === "loading" ? (
        <SellerLoadingState />
      ) : preview === "empty" ? (
        <SellerEmptyState orgId={orgId} />
      ) : preview === "error" ? (
        <SellerErrorState
          title="卖方主页读取失败"
          message="CAT_VALIDATION_FAILED: 卖方主页必须承接 platform-core 统一错误码与 request_id。"
          onRetry={() => sellerQuery.refetch()}
        />
      ) : preview === "forbidden" || !canRead ? (
        <SellerPermissionState sessionMode={sessionMode} subject={initialSubject} />
      ) : sellerQuery.isPending ? (
        <SellerLoadingState />
      ) : sellerQuery.isError ? (
        <SellerErrorState
          title="卖方主页请求失败"
          message={describeError(sellerQuery.error)}
          onRetry={() => sellerQuery.refetch()}
        />
      ) : seller ? (
        <SellerContent
          seller={seller}
          recommendations={recommendationQuery.data?.data.items ?? []}
          recommendationPending={recommendationQuery.isPending}
          recommendationError={recommendationQuery.error}
          canRecommend={canRecommend}
        />
      ) : (
        <SellerEmptyState orgId={orgId} />
      )}
    </div>
  );
}

function SellerHeader({
  orgId,
  preview,
  canRead,
  sessionMode,
  subject,
  seller,
}: {
  orgId: string;
  preview: string;
  canRead: boolean;
  sessionMode: "guest" | "bearer" | "local";
  subject: PortalSessionPreview | null;
  seller?: Seller;
}) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28 }}
      className="grid gap-4 xl:grid-cols-[1.35fr_0.9fr]"
    >
      <Card className="overflow-hidden bg-[radial-gradient(circle_at_88%_16%,rgba(11,132,101,0.18),transparent_28%),linear-gradient(135deg,rgba(255,255,255,0.97),rgba(232,241,238,0.88),rgba(242,234,217,0.7))]">
        <div className="space-y-5">
          <div className="flex flex-wrap gap-2">
            <ScaffoldPill>{meta.group}</ScaffoldPill>
            <ScaffoldPill>{meta.key}</ScaffoldPill>
            <ScaffoldPill tone="warning">preview:{preview}</ScaffoldPill>
            <ScaffoldPill>{canRead ? "Bearer API ready" : "requires portal.seller.read"}</ScaffoldPill>
          </div>
          <div className="max-w-4xl">
            <Badge>Seller Profile</Badge>
            <h1 className="mt-3 text-3xl font-semibold tracking-[-0.045em] text-[var(--ink-strong)] md:text-5xl">
              {seller?.org_name ?? "卖方主页"}
            </h1>
            <CardDescription className="mt-4 text-base">
              主体信息、认证标识、信誉风险、在售商品和咨询入口均绑定
              `platform-core` 正式 API；浏览器端只经过 `/api/platform/**` 受控代理。
            </CardDescription>
          </div>
          <div className="grid gap-3 md:grid-cols-4">
            <HeaderMetric label="org_id" value={orgId} />
            <HeaderMetric label="查看权限" value={meta.viewPermission} />
            <HeaderMetric label="推荐权限" value="portal.recommendation.read" />
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

function SellerContent({
  seller,
  recommendations,
  recommendationPending,
  recommendationError,
  canRecommend,
}: {
  seller: Seller;
  recommendations: RecommendationItem[];
  recommendationPending: boolean;
  recommendationError: unknown;
  canRecommend: boolean;
}) {
  const risk = sellerRiskDescriptor(seller.risk_level);
  const marketplaceItems = mergeSellerMarketplaceItems(seller, recommendations);
  const productItems = marketplaceItems.filter((item) => item.kind === "product");
  const peerSellerItems = marketplaceItems.filter((item) => item.kind === "seller");

  return (
    <div className="grid gap-4 2xl:grid-cols-[minmax(0,1fr)_390px]">
      <main className="space-y-4">
        <SellerSubjectCard seller={seller} risk={risk} />
        <MarketplaceCard
          seller={seller}
          items={productItems}
          pending={recommendationPending}
          error={recommendationError}
          canRecommend={canRecommend}
        />
        <ContactCard seller={seller} />
      </main>
      <aside className="space-y-4">
        <SellerSignalCard seller={seller} risk={risk} />
        <PeerSellerCard
          items={peerSellerItems}
          pending={recommendationPending}
          error={recommendationError}
          canRecommend={canRecommend}
        />
        <AuditBoundaryCard seller={seller} />
      </aside>
    </div>
  );
}

function SellerSubjectCard({
  seller,
  risk,
}: {
  seller: Seller;
  risk: ReturnType<typeof sellerRiskDescriptor>;
}) {
  return (
    <Card>
      <div className="grid gap-5 xl:grid-cols-[1fr_320px]">
        <div className="space-y-4">
          <div className="flex flex-wrap items-center gap-2">
            <BadgeCheck className="size-5 text-[var(--accent-strong)]" />
            <CardTitle>主体信息与认证标识</CardTitle>
          </div>
          <p className="text-sm leading-7 text-[var(--ink-soft)]">
            {seller.description ||
              "该卖方暂未返回公开简介，页面保留最小披露信息并继续展示平台校验结果。"}
          </p>
          <div className="grid gap-3 md:grid-cols-3">
            <InfoTile label="组织类型" value={seller.org_type} />
            <InfoTile label="组织状态" value={seller.status} />
            <InfoTile label="地区" value={[seller.country_code, seller.region_code].filter(Boolean).join(" / ") || "未返回"} />
          </div>
          <div className="space-y-2">
            <div className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--ink-subtle)]">
              行业标签
            </div>
            <div className="flex flex-wrap gap-2">
              {seller.industry_tags.length ? (
                seller.industry_tags.map((tag) => (
                  <span key={tag} className="rounded-full bg-[var(--panel-muted)] px-3 py-1 text-xs font-medium text-[var(--ink-soft)]">
                    {tag}
                  </span>
                ))
              ) : (
                <span className="text-sm text-[var(--ink-subtle)]">未返回行业标签</span>
              )}
            </div>
          </div>
        </div>
        <div className="rounded-[24px] border border-black/5 bg-[linear-gradient(160deg,rgba(15,107,137,0.08),rgba(255,255,255,0.86))] p-4">
          <div className="flex items-center gap-2">
            <ShieldCheck className="size-5 text-[var(--accent-strong)]" />
            <div className="text-sm font-semibold text-[var(--ink-strong)]">认证与风险摘要</div>
          </div>
          <div className="mt-4 grid gap-3">
            <MetricLine label="信誉分" value={formatReputationScore(seller.reputation_score)} />
            <MetricLine label="信用等级" value={String(seller.credit_level)} />
            <MetricLine label="风险等级" value={risk.label} tone={risk.tone} />
            <MetricLine label="在售商品" value={`${seller.listed_product_count} 个`} />
          </div>
          <div className="mt-4 flex flex-wrap gap-2">
            {seller.certification_tags.length ? (
              seller.certification_tags.map((tag) => (
                <span key={tag} className="rounded-full bg-white/80 px-3 py-1 text-xs font-semibold text-[var(--accent-strong)] ring-1 ring-[var(--accent-soft)]">
                  {certificationLabel(tag)}
                </span>
              ))
            ) : (
              <span className="text-sm text-[var(--ink-subtle)]">未返回公开认证标签</span>
            )}
          </div>
        </div>
      </div>
    </Card>
  );
}

function MarketplaceCard({
  seller,
  items,
  pending,
  error,
  canRecommend,
}: {
  seller: Seller;
  items: SellerMarketplaceItem[];
  pending: boolean;
  error: unknown;
  canRecommend: boolean;
}) {
  return (
    <Card>
      <div className="mb-4 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <CardTitle>在售商品 / 服务列表</CardTitle>
          <CardDescription>
            数据来自 `seller_search_document.featured_products` 与 `seller_profile_featured` 推荐位，返回前由后端执行 PostgreSQL 可见性校验。
          </CardDescription>
        </div>
        <Button asChild variant="secondary">
          <Link href={`/search?entity_scope=product&q=${encodeURIComponent(seller.org_name)}` as Route}>
            继续筛选该卖方商品
            <ExternalLink className="size-4" />
          </Link>
        </Button>
      </div>

      {!canRecommend ? (
        <InlineNotice
          title="推荐权限未开启"
          message="当前主体缺少 portal.recommendation.read；页面仍展示卖方搜索投影中的主打商品。"
        />
      ) : pending ? (
        <InlineNotice
          icon={<LoaderCircle className="size-4 animate-spin" />}
          title="正在加载推荐位"
          message="读取 seller_profile_featured，用于补充热门商品和同类优质卖方。"
        />
      ) : error ? (
        <InlineNotice
          icon={<AlertTriangle className="size-4" />}
          title="推荐位读取失败"
          message={describeError(error)}
          tone="warning"
        />
      ) : null}

      <div className="mt-4 grid gap-3 lg:grid-cols-2">
        {items.length ? (
          items.map((item) => <MarketplaceItemCard key={`${item.kind}:${item.id}`} item={item} />)
        ) : (
          <div className="rounded-[22px] border border-dashed border-black/10 bg-white/60 p-5 text-sm text-[var(--ink-soft)] lg:col-span-2">
            暂无可展示的在售商品/服务。页面没有使用 mock 填充；请以卖方投影或推荐位返回为准。
          </div>
        )}
      </div>
    </Card>
  );
}

function MarketplaceItemCard({ item }: { item: SellerMarketplaceItem }) {
  return (
    <Link
      href={item.href as Route}
      className="group block rounded-[22px] border border-black/5 bg-white/78 p-4 transition hover:-translate-y-0.5 hover:bg-white hover:shadow-[0_18px_45px_rgba(18,41,52,0.10)]"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-full bg-[var(--accent-soft)] px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.16em] text-[var(--accent-strong)]">
              {item.kind === "product" ? "product" : "seller"}
            </span>
            <span className="rounded-full bg-[var(--panel-muted)] px-2.5 py-1 text-[10px] font-medium text-[var(--ink-subtle)]">
              {item.source}
            </span>
          </div>
          <div className="mt-3 line-clamp-2 text-base font-semibold tracking-tight text-[var(--ink-strong)]">
            {item.title}
          </div>
          <div className="mt-2 line-clamp-2 text-sm leading-6 text-[var(--ink-soft)]">
            {item.subtitle || item.category || "未返回摘要"}
          </div>
        </div>
        <PackageSearch className="mt-1 size-5 shrink-0 text-[var(--accent-strong)] transition group-hover:scale-110" />
      </div>
      <div className="mt-4 flex flex-wrap items-center gap-2 text-xs text-[var(--ink-subtle)]">
        <span className="font-semibold text-[var(--ink-strong)]">{item.priceLabel}</span>
        {item.status ? <span>状态 {item.status}</span> : null}
        {typeof item.score === "number" ? <span>score {item.score.toFixed(2)}</span> : null}
      </div>
    </Link>
  );
}

function ContactCard({ seller }: { seller: Seller }) {
  const sellerActive = seller.status === "active";
  return (
    <Card className="bg-[linear-gradient(135deg,rgba(255,255,255,0.92),rgba(230,241,236,0.78))]">
      <div className="grid gap-4 lg:grid-cols-[1fr_auto] lg:items-center">
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <MessageSquareText className="size-5 text-[var(--accent-strong)]" />
            <CardTitle>联系方式 / 咨询入口</CardTitle>
          </div>
          <CardDescription>
            V1 不在页面暴露邮箱、电话或真实对象路径；咨询通过平台工单与受控下单入口承接，并保留 request_id 与审计线索。
          </CardDescription>
        </div>
        <div className="flex flex-wrap gap-2">
          {sellerActive ? (
            <>
              <Button asChild>
                <Link href={`/support/cases/new?seller_org_id=${seller.org_id}&reason=pre_sales_consultation` as Route}>
                  发起咨询工单
                </Link>
              </Button>
              <Button asChild variant="secondary">
                <Link href={`/trade/orders/new?seller_org_id=${seller.org_id}` as Route}>
                  <ShoppingCart className="size-4" />
                  进入下单页
                </Link>
              </Button>
            </>
          ) : (
            <Button disabled variant="warning">
              组织状态 {seller.status}，暂停咨询入口
            </Button>
          )}
        </div>
      </div>
    </Card>
  );
}

function SellerSignalCard({
  seller,
  risk,
}: {
  seller: Seller;
  risk: ReturnType<typeof sellerRiskDescriptor>;
}) {
  return (
    <Card>
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Sparkles className="size-5 text-[var(--accent-strong)]" />
          <CardTitle>最近成交与争议摘要</CardTitle>
        </div>
        <CardDescription>
          来自卖方搜索投影 `rating_summary`，前端只展示后端聚合结果，不自行计算账务或争议真值。
        </CardDescription>
        <div className="grid gap-3">
          <InfoTile label="评分数量" value={sellerRatingMetric(seller, "rating_count")} />
          <InfoTile label="平均评分" value={sellerRatingMetric(seller, "average_rating")} />
          <InfoTile label="信誉快照" value={formatReputationScore(sellerRatingMetric(seller, "reputation_score"))} />
          <InfoTile label="风险摘要" value={risk.label} />
          <InfoTile label="快照时间" value={sellerRatingMetric(seller, "effective_at")} />
        </div>
      </div>
    </Card>
  );
}

function PeerSellerCard({
  items,
  pending,
  error,
  canRecommend,
}: {
  items: SellerMarketplaceItem[];
  pending: boolean;
  error: unknown;
  canRecommend: boolean;
}) {
  return (
    <Card>
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Building2 className="size-5 text-[var(--accent-strong)]" />
          <CardTitle>同类优质卖方</CardTitle>
        </div>
        <CardDescription>
          仅展示 `seller_profile_featured` 返回的 seller 候选；无返回时保留空态。
        </CardDescription>
        {!canRecommend ? (
          <InlineNotice title="无推荐读取权限" message="需要 portal.recommendation.read。" />
        ) : pending ? (
          <InlineNotice icon={<LoaderCircle className="size-4 animate-spin" />} title="加载中" message="正在读取推荐位。" />
        ) : error ? (
          <InlineNotice icon={<AlertTriangle className="size-4" />} title="同类卖方读取失败" message={describeError(error)} tone="warning" />
        ) : items.length ? (
          <div className="space-y-2">
            {items.map((item) => (
              <Link key={item.id} href={item.href as Route} className="block rounded-2xl bg-[var(--panel-muted)] p-3 text-sm font-medium text-[var(--ink-strong)] hover:bg-[var(--accent-soft)]">
                {item.title}
              </Link>
            ))}
          </div>
        ) : (
          <div className="rounded-2xl border border-dashed border-black/10 bg-white/55 p-4 text-sm text-[var(--ink-soft)]">
            暂无同类优质卖方推荐。
          </div>
        )}
      </div>
    </Card>
  );
}

function AuditBoundaryCard({ seller }: { seller: Seller }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <div className="space-y-3">
        <div className="flex items-center gap-2 text-[var(--warning-ink)]">
          <FileSearch className="size-5" />
          <CardTitle className="text-[var(--warning-ink)]">审计与投影边界</CardTitle>
        </div>
        <CardDescription className="text-[var(--warning-ink)]">
          卖方 profile 读取会在 `platform-core` 侧留下 `catalog.seller.profile.read` 审计；推荐读取会写入推荐访问审计。前端不承担搜索索引、推荐、账务或审计真相源。
        </CardDescription>
        <div className="grid gap-2 text-xs text-[var(--warning-ink)]">
          <MetricLine label="search_document_version" value={String(seller.search_document_version)} />
          <MetricLine label="index_sync_status" value={seller.index_sync_status} />
          <MetricLine label="API" value="/api/v1/sellers/{orgId}/profile" />
          <MetricLine label="推荐位" value="seller_profile_featured" />
        </div>
      </div>
    </Card>
  );
}

function SellerLoadingState() {
  return (
    <Card className="border-dashed">
      <div className="flex flex-col items-center justify-center gap-3 py-12 text-center">
        <LoaderCircle className="size-9 animate-spin text-[var(--accent-strong)]" />
        <CardTitle>正在加载卖方主页</CardTitle>
        <CardDescription>读取卖方 profile、推荐位与搜索投影摘要。</CardDescription>
      </div>
    </Card>
  );
}

function SellerEmptyState({ orgId }: { orgId: string }) {
  return (
    <Card>
      <div className="flex flex-col items-center justify-center gap-3 py-12 text-center">
        <PackageSearch className="size-10 text-[var(--ink-subtle)]" />
        <CardTitle>没有可展示的卖方主页</CardTitle>
        <CardDescription>
          org_id={orgId} 未返回公开 profile，页面没有使用 mock 数据填充。
        </CardDescription>
      </div>
    </Card>
  );
}

function SellerErrorState({
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
      <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-[var(--danger-ink)]">
            <AlertTriangle className="size-5" />
            <CardTitle className="text-[var(--danger-ink)]">{title}</CardTitle>
          </div>
          <CardDescription className="text-[var(--danger-ink)]">{message}</CardDescription>
        </div>
        <Button type="button" variant="secondary" onClick={onRetry}>
          重试
        </Button>
      </div>
    </Card>
  );
}

function SellerPermissionState({
  sessionMode,
  subject,
}: {
  sessionMode: "guest" | "bearer" | "local";
  subject: PortalSessionPreview | null;
}) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <div className="flex flex-col gap-3 md:flex-row md:items-center">
        <Ban className="size-8 text-[var(--warning-ink)]" />
        <div className="space-y-2">
          <CardTitle className="text-[var(--warning-ink)]">卖方主页权限态</CardTitle>
          <CardDescription className="text-[var(--warning-ink)]">
            需要权限：`portal.seller.read`；当前会话模式 {sessionMode}，角色 {formatList(subject?.roles ?? [])}。
          </CardDescription>
        </div>
      </div>
    </Card>
  );
}

function HeaderMetric({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="rounded-2xl bg-white/70 p-3 ring-1 ring-black/5">
      <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-1 truncate font-mono text-xs text-[var(--ink-strong)]">
        {value}
      </div>
    </div>
  );
}

function ContextRow({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex items-start justify-between gap-3 rounded-2xl bg-[var(--panel-muted)] px-3 py-2 text-sm">
      <span className="shrink-0 text-[var(--ink-subtle)]">{label}</span>
      <span className="break-all text-right font-medium text-[var(--ink-strong)]">
        {value}
      </span>
    </div>
  );
}

function InfoTile({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="rounded-2xl bg-[var(--panel-muted)] p-3">
      <div className="text-xs text-[var(--ink-subtle)]">{label}</div>
      <div className="mt-1 break-words font-semibold text-[var(--ink-strong)]">
        {value || "未返回"}
      </div>
    </div>
  );
}

function MetricLine({
  label,
  value,
  tone,
}: {
  label: string;
  value: ReactNode;
  tone?: "success" | "warning" | "danger";
}) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-2xl bg-white/65 px-3 py-2">
      <span className="text-xs text-[var(--ink-subtle)]">{label}</span>
      <span
        className={cn(
          "break-all text-right text-sm font-semibold text-[var(--ink-strong)]",
          tone === "success" && "text-[var(--accent-strong)]",
          tone === "warning" && "text-[var(--warning-ink)]",
          tone === "danger" && "text-[var(--danger-ink)]",
        )}
      >
        {value}
      </span>
    </div>
  );
}

function InlineNotice({
  title,
  message,
  icon = <CheckCircle2 className="size-4" />,
  tone = "default",
}: {
  title: string;
  message: string;
  icon?: ReactNode;
  tone?: "default" | "warning";
}) {
  return (
    <div
      className={cn(
        "flex gap-3 rounded-[20px] border p-4 text-sm",
        tone === "warning"
          ? "border-[var(--warning-ring)] bg-[var(--warning-soft)] text-[var(--warning-ink)]"
          : "border-black/5 bg-[var(--panel-muted)] text-[var(--ink-soft)]",
      )}
    >
      <span className="mt-0.5 shrink-0">{icon}</span>
      <span>
        <span className="font-semibold">{title}</span>
        <span className="ml-2">{message}</span>
      </span>
    </div>
  );
}

function describeError(error: unknown): string {
  return formatPlatformErrorForDisplay(error, {
    fallbackCode: "INTERNAL_ERROR",
    fallbackDescription: "请结合错误码和 request_id 回查卖方主体、商品聚合和权限范围。",
  });
}

function hasAnyRole(
  subject: PortalSessionPreview | null,
  roles: string[],
): boolean {
  return Boolean(subject?.roles.some((role) => roles.includes(role)));
}
