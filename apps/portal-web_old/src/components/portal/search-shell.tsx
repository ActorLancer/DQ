"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import type { SearchCatalogResponse } from "@datab/sdk-ts";
import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";
import { useQuery } from "@tanstack/react-query";
import {
  AlertTriangle,
  Ban,
  Boxes,
  Building2,
  ChevronLeft,
  ChevronRight,
  DatabaseZap,
  LoaderCircle,
  LockKeyhole,
  Search,
  SlidersHorizontal,
} from "lucide-react";
import type { Route } from "next";
import Link from "next/link";
import { motion } from "motion/react";
import {
  startTransition,
  useDeferredValue,
  useEffect,
  useMemo,
  type ReactNode,
} from "react";
import { useForm, useWatch } from "react-hook-form";
import {
  usePathname,
  useRouter,
  useSearchParams,
} from "next/navigation";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import {
  defaultSearchFormValues,
  entityScopeOptions,
  formValuesToSearchQuery,
  formValuesToUrlSearchParams,
  pageFromSearchParams,
  parseSearchFormValues,
  searchFormSchema,
  sortOptions,
  type SearchFormValues,
} from "@/lib/search-query";
import type { PortalSessionPreview } from "@/lib/session";
import { cn, formatList } from "@/lib/utils";

import {
  PreviewStateControls,
  ScaffoldPill,
  getPreviewState,
  isRoutePreviewEnabled,
} from "./state-preview";

const sdk = createBrowserSdk();
const meta = portalRouteMap.catalog_search;

type SearchShellProps = {
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};
type SearchResultItem = SearchCatalogResponse["data"]["items"][number];
type SearchFacetSummary = SearchCatalogResponse["data"]["facets"];

export function SearchShell({ sessionMode, initialSubject }: SearchShellProps) {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);
  const currentPage = pageFromSearchParams(searchParams);
  const formValues = useMemo(
    () => parseSearchFormValues(new URLSearchParams(searchParams.toString())),
    [searchParams],
  );
  const deferredFormValues = useDeferredValue(formValues);
  const query = useMemo(
    () => formValuesToSearchQuery(deferredFormValues, currentPage),
    [deferredFormValues, currentPage],
  );
  const queryKey = useMemo(() => JSON.stringify(query), [query]);
  const canSearch = sessionMode === "bearer" && Boolean(initialSubject);
  const form = useForm<SearchFormValues>({
    resolver: zodResolver(searchFormSchema),
    defaultValues: formValues,
  });
  const entityScopeValue = useWatch({
    control: form.control,
    name: "entity_scope",
  });

  useEffect(() => {
    form.reset(formValues);
  }, [form, formValues]);

  const searchQuery = useQuery({
    queryKey: ["portal", "catalog-search", queryKey],
    enabled: canSearch && preview === "ready",
    queryFn: () => sdk.search.searchCatalog(query),
  });
  const response = searchQuery.data?.data;
  const items = response?.items ?? [];
  const pageSize = response?.page_size ?? formValues.page_size;
  const total = response?.total ?? 0;
  const maxPage = Math.max(1, Math.ceil(total / pageSize));
  const facets = response?.facets;
  const hasFacetSummary = Boolean(
    facets &&
      (facets.seller_org_ids.length > 0 ||
        facets.seller_types.length > 0 ||
        facets.data_classifications.length > 0 ||
        facets.price_modes.length > 0),
  );

  const applyFacet = (
    field: "seller_org_id" | "seller_type" | "data_classification" | "price_mode",
    value: string,
  ) => {
    const currentValues = form.getValues();
    const nextValue = currentValues[field] === value ? "" : value;
    const nextValues: SearchFormValues = {
      ...currentValues,
      [field]: nextValue,
    };
    form.setValue(field, nextValue, { shouldDirty: true });
    const params = formValuesToUrlSearchParams(nextValues, 1);
    startTransition(() => {
      router.push(`${pathname}?${params.toString()}` as Route);
    });
  };

  return (
    <div className="space-y-6">
      <SearchHeader
        preview={preview}
        canSearch={canSearch}
        sessionMode={sessionMode}
        subject={initialSubject}
      />

      <div className="grid gap-4 xl:grid-cols-[360px_1fr]">
        <motion.aside
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.28 }}
          className="space-y-4"
        >
          <Card className="bg-[linear-gradient(160deg,rgba(255,255,255,0.96),rgba(230,241,236,0.86))]">
            <form
              className="space-y-5"
              onSubmit={form.handleSubmit((values: SearchFormValues) => {
                const params = formValuesToUrlSearchParams(values, 1);
                startTransition(() => {
                  router.push(`${pathname}?${params.toString()}` as Route);
                });
              })}
            >
              <div className="flex items-center justify-between gap-3">
                <div>
                  <CardTitle>搜索与筛选</CardTitle>
                  <CardDescription className="mt-1">
                    参数严格绑定 `GET /api/v1/catalog/search` 当前契约。
                  </CardDescription>
                </div>
                <SlidersHorizontal className="size-5 text-[var(--accent-strong)]" />
              </div>

              <Field label="关键词" error={form.formState.errors.q?.message}>
                <div className="relative">
                  <Search className="pointer-events-none absolute left-4 top-1/2 size-4 -translate-y-1/2 text-[var(--ink-subtle)]" />
                  <Input
                    className="pl-10"
                    placeholder="工业设备运行指标 / 选址服务"
                    {...form.register("q")}
                  />
                </div>
              </Field>

              <Field label="搜索对象范围">
                <SegmentedControl
                  value={entityScopeValue ?? defaultSearchFormValues.entity_scope}
                  options={entityScopeOptions}
                  onChange={(value) => form.setValue("entity_scope", value)}
                />
              </Field>

              <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-1">
                <Field label="行业分类" error={form.formState.errors.industry?.message}>
                  <Input placeholder="industrial_manufacturing" {...form.register("industry")} />
                </Field>
                <Field label="交付方式" error={form.formState.errors.delivery_mode?.message}>
                  <Input placeholder="api_subscription / file_download" {...form.register("delivery_mode")} />
                </Field>
              </div>

              <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-1">
                <Field label="供方组织 ID" error={form.formState.errors.seller_org_id?.message}>
                  <Input placeholder="seller_org_id(UUID)" {...form.register("seller_org_id")} />
                </Field>
                <Field label="供方主体类型" error={form.formState.errors.seller_type?.message}>
                  <Input placeholder="enterprise / institution" {...form.register("seller_type")} />
                </Field>
                <Field
                  label="敏感等级"
                  error={form.formState.errors.data_classification?.message}
                >
                  <Input placeholder="L1 / L2 / L3" {...form.register("data_classification")} />
                </Field>
                <Field label="价格模式" error={form.formState.errors.price_mode?.message}>
                  <Input placeholder="fixed / subscription / ppu" {...form.register("price_mode")} />
                </Field>
              </div>

              <Field label="标签" error={form.formState.errors.tags?.message}>
                <Input placeholder="质量, 能耗, 门店" {...form.register("tags")} />
              </Field>

              <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-1">
                <Field label="最低价格" error={form.formState.errors.price_min?.message}>
                  <Input inputMode="decimal" placeholder="0" {...form.register("price_min")} />
                </Field>
                <Field label="最高价格" error={form.formState.errors.price_max?.message}>
                  <Input inputMode="decimal" placeholder="9999" {...form.register("price_max")} />
                </Field>
              </div>

              <Field label="排序">
                <select
                  className="flex h-11 w-full rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]"
                  {...form.register("sort")}
                >
                  {sortOptions.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </Field>

              <Field label="每页数量" error={form.formState.errors.page_size?.message}>
                <select
                  className="flex h-11 w-full rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]"
                  {...form.register("page_size", { valueAsNumber: true })}
                >
                  <option value={12}>12</option>
                  <option value={24}>24</option>
                  <option value={48}>48</option>
                </select>
              </Field>

              <div className="flex flex-wrap gap-2">
                <Button type="submit" disabled={!canSearch}>
                  <Search className="size-4" />
                  执行搜索
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  onClick={() => {
                    form.reset(defaultSearchFormValues);
                    startTransition(() => {
                      router.push(pathname as Route);
                    });
                  }}
                >
                  重置
                </Button>
              </div>
            </form>
          </Card>
        </motion.aside>

        <motion.section
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.04, duration: 0.28 }}
          className="space-y-4"
        >
          {preview === "loading" ? (
            <SearchLoadingState />
          ) : preview === "empty" ? (
            <SearchEmptyState keyword={formValues.q || "preview-empty"} />
          ) : preview === "error" ? (
            <SearchErrorState
              title="搜索后端不可用"
              message="SEARCH_BACKEND_UNAVAILABLE: OpenSearch 不可用时应由 platform-core 切换 PostgreSQL fallback；前端只承接错误码与重试。"
              onRetry={() => searchQuery.refetch()}
            />
          ) : preview === "forbidden" || !canSearch ? (
            <SearchPermissionState sessionMode={sessionMode} subject={initialSubject} />
          ) : searchQuery.isPending ? (
            <SearchLoadingState />
          ) : searchQuery.isError ? (
            <SearchErrorState
              title="搜索请求失败"
              message={describeError(searchQuery.error)}
              onRetry={() => searchQuery.refetch()}
            />
          ) : response && items.length === 0 ? (
            <>
              <SearchStats response={response} queryLabel={describeQuery(formValues)} />
              {hasFacetSummary ? (
                <SearchFacetPanel
                  facets={facets}
                  filters={formValues}
                  onApplyFacet={applyFacet}
                />
              ) : null}
              <SearchEmptyState keyword={formValues.q || "当前筛选条件"} />
            </>
          ) : response ? (
            <>
              <SearchStats response={response} queryLabel={describeQuery(formValues)} />
              {hasFacetSummary ? (
                <SearchFacetPanel
                  facets={facets}
                  filters={formValues}
                  onApplyFacet={applyFacet}
                />
              ) : null}
              <div className="grid gap-4">
                {items.map((item) => (
                  <SearchResultCard key={`${item.entity_scope}-${item.entity_id}`} item={item} />
                ))}
              </div>
              <PaginationControls
                page={response.page}
                maxPage={maxPage}
                buildHref={(page) =>
                  `${pathname}?${formValuesToUrlSearchParams(formValues, page).toString()}` as Route
                }
              />
            </>
          ) : null}
        </motion.section>
      </div>
    </div>
  );
}

function SearchHeader({
  preview,
  canSearch,
  sessionMode,
  subject,
}: {
  preview: string;
  canSearch: boolean;
  sessionMode: "guest" | "bearer" | "local";
  subject: PortalSessionPreview | null;
}) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28 }}
      className="grid gap-4 xl:grid-cols-[1.4fr_0.9fr]"
    >
      <Card className="overflow-hidden bg-[linear-gradient(135deg,rgba(255,255,255,0.96),rgba(219,239,235,0.86),rgba(230,242,247,0.82))]">
        <div className="space-y-5">
          <div className="flex flex-wrap gap-2">
            <ScaffoldPill>{meta.group}</ScaffoldPill>
            <ScaffoldPill>{meta.key}</ScaffoldPill>
            {isRoutePreviewEnabled() ? (
              <ScaffoldPill tone="warning">preview:{preview}</ScaffoldPill>
            ) : null}
            <ScaffoldPill>{canSearch ? "Bearer API ready" : "requires Bearer"}</ScaffoldPill>
          </div>
          <div className="max-w-3xl">
            <Badge>Catalog Search</Badge>
            <h1 className="mt-3 text-3xl font-semibold tracking-[-0.04em] text-[var(--ink-strong)] md:text-5xl">
              统一搜索数据商品、服务商品与卖方主体
            </h1>
            <CardDescription className="mt-4 text-base">
              关键词、精确筛选、排序、分页与结果状态均通过 `platform-core`
              正式搜索接口返回；浏览器端只访问 `/api/platform/**` 受控代理。
            </CardDescription>
          </div>
          <div className="grid gap-3 md:grid-cols-3">
            <HeaderMetric label="查看权限" value={meta.viewPermission} />
            <HeaderMetric label="执行权限" value={formatList(meta.primaryPermissions)} />
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
              <CardTitle>当前主体搜索上下文</CardTitle>
              <CardDescription>
                敏感页面必须显示主体、角色、租户、作用域；身份条同时保留全局展示。
              </CardDescription>
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

function SearchStats({
  response,
  queryLabel,
}: {
  response: SearchCatalogResponse["data"];
  queryLabel: string;
}) {
  return (
    <Card className="bg-white/90">
      <div className="grid gap-4 lg:grid-cols-[1fr_auto] lg:items-center">
        <div>
          <div className="flex flex-wrap gap-2">
            <Badge>{response.backend}</Badge>
            <Badge className={response.cache_hit ? "bg-emerald-50 text-emerald-700" : "bg-slate-100 text-slate-700"}>
              cache:{String(response.cache_hit)}
            </Badge>
            <Badge className="bg-slate-100 text-slate-700">scope:{response.entity_scope}</Badge>
          </div>
          <CardTitle className="mt-3">搜索结果统计</CardTitle>
          <CardDescription className="mt-1">
            {queryLabel}，当前第 {response.page} 页，每页 {response.page_size} 条。
          </CardDescription>
        </div>
        <div className="grid grid-cols-3 gap-3 text-center">
          <HeaderMetric label="total" value={String(response.total)} />
          <HeaderMetric label="page" value={String(response.page)} />
          <HeaderMetric label="items" value={String(response.items.length)} />
        </div>
      </div>
    </Card>
  );
}

function SearchFacetPanel({
  facets,
  filters,
  onApplyFacet,
}: {
  facets: SearchFacetSummary | undefined;
  filters: SearchFormValues;
  onApplyFacet: (
    field: "seller_org_id" | "seller_type" | "data_classification" | "price_mode",
    value: string,
  ) => void;
}) {
  if (!facets) {
    return null;
  }

  return (
    <Card className="bg-white/90">
      <div className="space-y-4">
        <div>
          <CardTitle>Facet 聚合统计</CardTitle>
          <CardDescription className="mt-1">
            聚合来源为 `platform-core` 返回，点击任意项可回填筛选条件。
          </CardDescription>
        </div>
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <FacetGroup
            title="供方组织"
            buckets={facets.seller_org_ids}
            activeValue={filters.seller_org_id}
            onSelect={(value) => onApplyFacet("seller_org_id", value)}
          />
          <FacetGroup
            title="主体类型"
            buckets={facets.seller_types}
            activeValue={filters.seller_type}
            onSelect={(value) => onApplyFacet("seller_type", value)}
          />
          <FacetGroup
            title="敏感等级"
            buckets={facets.data_classifications}
            activeValue={filters.data_classification}
            onSelect={(value) => onApplyFacet("data_classification", value)}
          />
          <FacetGroup
            title="价格模式"
            buckets={facets.price_modes}
            activeValue={filters.price_mode}
            onSelect={(value) => onApplyFacet("price_mode", value)}
          />
        </div>
      </div>
    </Card>
  );
}

function SearchResultCard({ item }: { item: SearchResultItem }) {
  const isProduct = item.entity_scope === "product";
  const canOrder = isProduct && item.status === "listed";
  const detailHref = isProduct
    ? (`/products/${item.entity_id}` as Route)
    : (`/sellers/${item.entity_id}` as Route);
  const orderHref = `/trade/orders/new?productId=${item.entity_id}` as Route;

  return (
    <Card className="group overflow-hidden transition hover:-translate-y-0.5 hover:shadow-[0_28px_70px_rgba(18,41,52,0.13)]">
      <div className="grid gap-4 xl:grid-cols-[1fr_260px]">
        <div className="space-y-4">
          <div className="flex flex-wrap items-center gap-2">
            <Badge className={isProduct ? "" : "bg-amber-50 text-amber-700"}>
              {item.entity_scope}
            </Badge>
            <span className="rounded-full bg-black/[0.04] px-3 py-1 text-xs font-medium text-[var(--ink-soft)]">
              status:{item.status}
            </span>
            <span className="rounded-full bg-black/[0.04] px-3 py-1 text-xs font-medium text-[var(--ink-soft)]">
              index:{item.index_sync_status}
            </span>
            <span className="rounded-full bg-black/[0.04] px-3 py-1 text-xs font-medium text-[var(--ink-soft)]">
              v{item.document_version}
            </span>
          </div>
          <div>
            <Link href={detailHref}>
              <CardTitle className="text-2xl transition group-hover:text-[var(--accent-strong)]">
                {item.title}
              </CardTitle>
            </Link>
            <CardDescription className="mt-2">
              {item.subtitle || item.description || "当前结果未返回摘要描述。"}
            </CardDescription>
          </div>
          <div className="flex flex-wrap gap-2">
            {item.industry_tags.map((tag) => (
              <Tag key={`industry-${tag}`}>{tag}</Tag>
            ))}
            {item.tags.map((tag) => (
              <Tag key={`tag-${tag}`}>{tag}</Tag>
            ))}
            {item.delivery_modes.map((mode) => (
              <Tag key={`delivery-${mode}`}>{mode}</Tag>
            ))}
            {item.seller_type ? <Tag>seller_type:{item.seller_type}</Tag> : null}
            {item.data_classification ? <Tag>classification:{item.data_classification}</Tag> : null}
            {item.price_mode ? <Tag>price_mode:{item.price_mode}</Tag> : null}
          </div>
          <div className="grid gap-3 text-sm text-[var(--ink-soft)] md:grid-cols-2">
            <InlineInfo icon={<Building2 className="size-4" />} label="卖方" value={item.seller_name ?? "未返回"} />
            <InlineInfo icon={<DatabaseZap className="size-4" />} label="类型" value={item.product_type ?? item.category ?? "未返回"} />
          </div>
        </div>

        <div className="rounded-[24px] bg-[var(--panel-muted)] p-4">
          <div className="grid grid-cols-2 gap-3">
            <MiniScore label="score" value={formatScore(item.score)} />
            <MiniScore label="quality" value={item.quality_score ?? "-"} />
            <MiniScore label="reputation" value={item.reputation_score ?? "-"} />
            <MiniScore label="hotness" value={item.hotness_score ?? "-"} />
          </div>
          <div className="mt-4 rounded-2xl bg-white/70 p-4">
            <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
              价格
            </div>
            <div className="mt-2 text-xl font-semibold text-[var(--ink-strong)]">
              {formatPrice(item)}
            </div>
          </div>
          <div className="mt-4 flex flex-col gap-2">
            <Button asChild>
              <Link href={detailHref}>查看详情</Link>
            </Button>
            {canOrder ? (
              <Button asChild variant="secondary">
                <Link href={orderHref}>进入下单</Link>
              </Button>
            ) : (
              <div className="rounded-2xl border border-[var(--warning-ring)] bg-[var(--warning-soft)] px-4 py-3 text-sm text-[var(--warning-ink)]">
                {isProduct
                  ? "结果项状态需刷新或不可购买，已隐藏下单入口。"
                  : "卖方结果不直接展示购买入口，请进入卖方主页查看商品。"}
              </div>
            )}
          </div>
        </div>
      </div>
    </Card>
  );
}

function FacetGroup({
  title,
  buckets,
  activeValue,
  onSelect,
}: {
  title: string;
  buckets: SearchFacetSummary["seller_org_ids"];
  activeValue: string;
  onSelect: (value: string) => void;
}) {
  return (
    <div className="rounded-2xl border border-black/10 bg-white p-3">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {title}
      </div>
      {buckets.length === 0 ? (
        <div className="mt-2 text-xs text-[var(--ink-subtle)]">无聚合项</div>
      ) : (
        <div className="mt-3 flex flex-wrap gap-2">
          {buckets.map((bucket) => {
            const isActive = activeValue === bucket.value;
            return (
              <button
                key={`${title}-${bucket.value}`}
                type="button"
                className={cn(
                  "rounded-full border px-3 py-1 text-xs font-medium transition",
                  isActive
                    ? "border-[var(--accent-strong)] bg-[var(--accent-soft)] text-[var(--accent-strong)]"
                    : "border-black/10 bg-black/[0.03] text-[var(--ink-soft)] hover:bg-black/[0.06]",
                )}
                onClick={() => onSelect(bucket.value)}
              >
                {bucket.value} ({bucket.count})
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

function SearchPermissionState({
  sessionMode,
  subject,
}: {
  sessionMode: "guest" | "bearer" | "local";
  subject: PortalSessionPreview | null;
}) {
  const message =
    sessionMode === "local"
      ? "本地 Header 联调身份只能用于身份条和部分本地校验；正式搜索接口要求 Bearer Token 与 portal.search.read。"
      : sessionMode === "bearer" && !subject
        ? "Bearer Token 缺少可识别主体 claims，无法建立搜索访问上下文。"
        : "请先通过 Keycloak / IAM Bearer 会话登录后再执行正式搜索。";

  return (
    <Card className="flex min-h-[420px] items-center justify-center border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <div className="max-w-xl text-center text-[var(--warning-ink)]">
        <Ban className="mx-auto size-10" />
        <CardTitle className="mt-4 text-[var(--warning-ink)]">搜索权限态</CardTitle>
        <CardDescription className="mt-3 text-[var(--warning-ink)]">
          {message}
        </CardDescription>
        <div className="mt-4 rounded-2xl bg-white/65 px-4 py-3 text-sm">
          需要权限：`portal.search.read` / `portal.search.use`
        </div>
      </div>
    </Card>
  );
}

function SearchLoadingState() {
  return (
    <Card className="min-h-[420px] bg-white/80">
      <div className="flex h-full min-h-[360px] flex-col justify-center gap-5">
        <div className="flex items-center gap-3 text-[var(--accent-strong)]">
          <LoaderCircle className="size-6 animate-spin" />
          <CardTitle>正在读取正式搜索结果</CardTitle>
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

function SearchEmptyState({ keyword }: { keyword: string }) {
  return (
    <Card className="flex min-h-[420px] items-center justify-center bg-[var(--panel-muted)]">
      <div className="max-w-xl text-center">
        <Boxes className="mx-auto size-10 text-[var(--ink-subtle)]" />
        <CardTitle className="mt-4">没有匹配的搜索结果</CardTitle>
        <CardDescription className="mt-3">
          当前关键词或筛选条件 `{keyword}` 未返回商品/服务/卖方主体。可尝试清空价格区间、减少标签或切换对象范围。
        </CardDescription>
      </div>
    </Card>
  );
}

function SearchErrorState({
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
        <CardDescription className="mt-3 text-[var(--danger-ink)]">
          {message}
        </CardDescription>
        <Button className="mt-5" variant="secondary" onClick={onRetry}>
          重试
        </Button>
      </div>
    </Card>
  );
}

function PaginationControls({
  page,
  maxPage,
  buildHref,
}: {
  page: number;
  maxPage: number;
  buildHref: (page: number) => Route;
}) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 rounded-[28px] bg-white/75 p-4">
      <div className="text-sm text-[var(--ink-soft)]">
        第 {page} / {maxPage} 页
      </div>
      <div className="flex gap-2">
        <Button asChild variant="secondary" disabled={page <= 1}>
          <Link href={buildHref(Math.max(1, page - 1))}>
            <ChevronLeft className="size-4" />
            上一页
          </Link>
        </Button>
        <Button asChild variant="secondary" disabled={page >= maxPage}>
          <Link href={buildHref(Math.min(maxPage, page + 1))}>
            下一页
            <ChevronRight className="size-4" />
          </Link>
        </Button>
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
      <span className="text-sm font-medium text-[var(--ink-strong)]">{label}</span>
      {children}
      {error ? <span className="text-sm text-[var(--danger-ink)]">{error}</span> : null}
    </label>
  );
}

function SegmentedControl<TValue extends string>({
  value,
  options,
  onChange,
}: {
  value: TValue;
  options: readonly { value: TValue; label: string }[];
  onChange: (value: TValue) => void;
}) {
  return (
    <div className="grid gap-2 sm:grid-cols-2">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          className={cn(
            "rounded-2xl border px-3 py-2 text-left text-sm transition",
            value === option.value
              ? "border-[var(--accent-strong)] bg-[var(--accent-soft)] text-[var(--accent-strong)]"
              : "border-black/10 bg-white/80 text-[var(--ink-soft)] hover:bg-white",
          )}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
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

function InlineInfo({
  icon,
  label,
  value,
}: {
  icon: ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-2 rounded-2xl bg-black/[0.035] px-3 py-2">
      {icon}
      <span className="text-[var(--ink-subtle)]">{label}</span>
      <span className="font-medium text-[var(--ink-strong)]">{value}</span>
    </div>
  );
}

function MiniScore({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl bg-white/70 p-3">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-1 text-lg font-semibold text-[var(--ink-strong)]">
        {value}
      </div>
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

function describeQuery(values: SearchFormValues): string {
  const parts = [
    values.q ? `关键词 ${values.q}` : "全部关键词",
    `范围 ${values.entity_scope}`,
    values.industry ? `行业 ${values.industry}` : null,
    values.seller_org_id ? `供方 ${values.seller_org_id}` : null,
    values.seller_type ? `主体类型 ${values.seller_type}` : null,
    values.data_classification ? `敏感等级 ${values.data_classification}` : null,
    values.price_mode ? `价格模式 ${values.price_mode}` : null,
    values.tags ? `标签 ${values.tags}` : null,
    values.delivery_mode ? `交付 ${values.delivery_mode}` : null,
    `排序 ${values.sort}`,
  ].filter(Boolean);
  return parts.join(" / ");
}

function describeError(error: unknown): string {
  return formatPlatformErrorForDisplay(error, {
    fallbackCode: "SEARCH_BACKEND_UNAVAILABLE",
    fallbackDescription: "请稍后重试；前端只通过 platform-core 受控边界承接搜索结果。",
  });
}

function formatScore(value: number): string {
  return Number.isFinite(value) ? value.toFixed(2) : "-";
}

function formatPrice(item: SearchResultItem): string {
  if (!item.price) {
    return "未返回";
  }
  return `${item.currency_code ?? ""} ${item.price}`.trim();
}
