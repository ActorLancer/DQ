"use client";

import type {
  RecommendationsQuery,
  RecommendationsResponse,
  SearchCatalogResponse,
  StandardScenariosResponse,
} from "@datab/sdk-ts";
import { PlatformApiError } from "@datab/sdk-ts";
import { useQuery } from "@tanstack/react-query";
import {
  ArrowRight,
  Compass,
  LockKeyhole,
  Radar,
  Search,
  Sparkles,
  Waypoints,
} from "lucide-react";
import Link from "next/link";
import type { Route } from "next";
import { motion } from "motion/react";
import { useDeferredValue, useState, type ReactNode } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { createBrowserSdk } from "@/lib/platform-sdk";
import type { PortalSessionPreview } from "@/lib/session";

import { PortalRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();
const OFFICIAL_SKU_ORDER = [
  "FILE_STD",
  "FILE_SUB",
  "SHARE_RO",
  "API_SUB",
  "API_PPU",
  "QRY_LITE",
  "SBX_STD",
  "RPT_STD",
] as const;
const INDUSTRY_META = {
  industrial_manufacturing: {
    label: "工业制造",
    description: "覆盖设备指标 API、质量日报文件包和供应链协同查询沙箱。",
  },
  retail: {
    label: "零售经营",
    description: "覆盖门店经营分析 API / 报告订阅与商圈选址查询服务。",
  },
  other: {
    label: "综合场景",
    description: "用于承接未归类场景，仍保持冻结 SKU 与链路口径。",
  },
} as const;
const FALLBACK_SEARCH_PRESETS = [
  { label: "S1 设备指标", keyword: "工业设备运行指标" },
  { label: "S2 质量日报", keyword: "工业质量与产线日报" },
  { label: "S3 供应链查询", keyword: "供应链协同" },
  { label: "S4 门店经营", keyword: "门店经营分析" },
  { label: "S5 选址服务", keyword: "商圈选址" },
] as const;
const FALLBACK_STANDARD_SCENARIOS: ScenarioTemplate[] = [
  {
    scenario_code: "S1",
    scenario_name: "工业设备运行指标 API 订阅",
    primary_sku: "API_SUB",
    supplementary_skus: ["API_PPU"],
    product_template: {
      category: "industry_iot",
      delivery_type: "api_subscription",
      product_type: "service_product",
    },
    metadata_template: {
      data_classification: "P1",
      industry: "industrial_manufacturing",
      use_cases: ["设备稼动率", "能耗监控"],
    },
    contract_template: "CONTRACT_API_SUB_V1",
    acceptance_template: "ACCEPT_API_SUB_V1",
    refund_template: "REFUND_API_SUB_V1",
    review_sample: {
      action_name: "approve",
      action_reason: "api_schema_and_sla_verified",
    },
  },
  {
    scenario_code: "S2",
    scenario_name: "工业质量与产线日报文件包交付",
    primary_sku: "FILE_STD",
    supplementary_skus: ["FILE_SUB"],
    product_template: {
      category: "industrial_quality",
      delivery_type: "file_download",
      product_type: "data_product",
    },
    metadata_template: {
      data_classification: "P1",
      industry: "industrial_manufacturing",
      use_cases: ["质量日报", "产线巡检"],
    },
    contract_template: "CONTRACT_FILE_V1",
    acceptance_template: "ACCEPT_FILE_V1",
    refund_template: "REFUND_FILE_V1",
    review_sample: {
      action_name: "approve",
      action_reason: "file_hash_and_preview_checked",
    },
  },
  {
    scenario_code: "S3",
    scenario_name: "供应链协同查询沙箱",
    primary_sku: "SBX_STD",
    supplementary_skus: ["SHARE_RO"],
    product_template: {
      category: "supply_chain",
      delivery_type: "sandbox",
      product_type: "service_product",
    },
    metadata_template: {
      data_classification: "P2",
      industry: "industrial_manufacturing",
      use_cases: ["履约分析", "库存协同"],
    },
    contract_template: "CONTRACT_SANDBOX_V1",
    acceptance_template: "ACCEPT_SANDBOX_V1",
    refund_template: "REFUND_SANDBOX_V1",
    review_sample: {
      action_name: "approve",
      action_reason: "sandbox_guardrail_verified",
    },
  },
  {
    scenario_code: "S4",
    scenario_name: "零售门店经营分析 API / 报告订阅",
    primary_sku: "API_SUB",
    supplementary_skus: ["RPT_STD"],
    product_template: {
      category: "retail_ops",
      delivery_type: "api_subscription",
      product_type: "service_product",
    },
    metadata_template: {
      data_classification: "P1",
      industry: "retail",
      use_cases: ["客流分析", "销售结构"],
    },
    contract_template: "CONTRACT_API_SUB_V1",
    acceptance_template: "ACCEPT_API_SUB_V1",
    refund_template: "REFUND_API_SUB_V1",
    review_sample: {
      action_name: "approve",
      action_reason: "api_report_combo_check_passed",
    },
  },
  {
    scenario_code: "S5",
    scenario_name: "商圈/门店选址查询服务",
    primary_sku: "QRY_LITE",
    supplementary_skus: ["RPT_STD"],
    product_template: {
      category: "retail_location",
      delivery_type: "query_template",
      product_type: "service_product",
    },
    metadata_template: {
      data_classification: "P1",
      industry: "retail",
      use_cases: ["选址评分", "商圈画像"],
    },
    contract_template: "CONTRACT_QUERY_LITE_V1",
    acceptance_template: "ACCEPT_QUERY_LITE_V1",
    refund_template: "REFUND_QUERY_LITE_V1",
    review_sample: {
      action_name: "approve",
      action_reason: "template_boundary_validated",
    },
  },
];

type HomeShellProps = {
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};
type SessionSubject = PortalSessionPreview;
type ScenarioTemplate = StandardScenariosResponse["data"][number];
type HomeRecommendationItem = RecommendationsResponse["data"]["items"][number];
type SearchPreviewItem = SearchCatalogResponse["data"]["items"][number];
type IndustryGroup = {
  key: keyof typeof INDUSTRY_META;
  label: string;
  description: string;
  scenarios: ScenarioTemplate[];
};

export function HomeShell({ sessionMode, initialSubject }: HomeShellProps) {
  const [searchKeyword, setSearchKeyword] = useState("");
  const deferredSearchKeyword = useDeferredValue(searchKeyword.trim());
  const subject = initialSubject;
  const healthQuery = useQuery({
    queryKey: ["portal", "health-ready"],
    queryFn: () => sdk.ops.healthReady(),
  });
  const scenarioQuery = useQuery({
    queryKey: [
      "portal",
      "standard-scenarios",
      sessionMode,
      subject?.tenant_id ?? subject?.org_id ?? null,
      subject?.roles.join(",") ?? "",
    ],
    enabled: sessionMode === "bearer" && Boolean(subject),
    queryFn: () => sdk.catalog.getStandardScenarioTemplates(),
  });
  const liveScenarios = scenarioQuery.data?.data ?? [];
  const scenarios = liveScenarios.length
    ? liveScenarios
    : FALLBACK_STANDARD_SCENARIOS;
  const usingScenarioFallback = liveScenarios.length === 0;
  const industryGroups = buildIndustryGroups(scenarios);
  const skuCoverage = collectSkuCoverage(scenarios);
  const searchPresets = buildSearchPresets(scenarios);

  const recommendationQuery = useQuery({
    queryKey: [
      "portal",
      "recommendations",
      "home_featured",
      subject?.tenant_id ?? subject?.org_id ?? null,
      subject?.user_id ?? null,
    ],
    enabled: sessionMode === "bearer" && Boolean(subject),
    queryFn: () =>
      sdk.recommendation.getRecommendations(
        buildHomeFeaturedQuery(subject ?? undefined),
      ),
  });
  const searchPreviewQuery = useQuery({
    queryKey: ["portal", "search-preview", deferredSearchKeyword],
    enabled: sessionMode === "bearer" && deferredSearchKeyword.length >= 2,
    queryFn: () =>
      sdk.search.searchCatalog({
        q: deferredSearchKeyword,
        entity_scope: "all",
        page: 1,
        page_size: 4,
      }),
  });

  return (
    <PortalRouteScaffold routeKey="portal_home">
      <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
        <motion.section
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.32 }}
          className="grid gap-4"
        >
          <Card className="overflow-hidden bg-[linear-gradient(130deg,rgba(10,74,105,0.94),rgba(15,92,126,0.84),rgba(8,138,104,0.72))] text-white">
            <div className="grid gap-6 md:grid-cols-[1.1fr_0.9fr]">
              <div className="space-y-4">
                <div className="inline-flex rounded-full bg-white/15 px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-white/80">
                  Portal / Home Featured
                </div>
                <h1 className="text-3xl font-semibold leading-tight">
                  门户首页已接入场景导航、推荐位与受控搜索入口。
                </h1>
                <p className="max-w-2xl text-sm leading-7 text-white/80">
                  首页统一通过 `portal-web -&gt; /api/platform -&gt; platform-core`
                  读取标准链路、推荐位与搜索预览。`guest / local / bearer`
                  三种会话能力被显式区分，避免把需要 Bearer 的正式 API
                  混进本地 header 占位链路。
                </p>
                <div className="flex flex-wrap gap-3">
                  <Button asChild variant="secondary">
                    <Link href={buildSearchHref(searchKeyword || "工业设备运行指标")}>
                      进入搜索入口
                      <ArrowRight className="size-4" />
                    </Link>
                  </Button>
                  <Button asChild variant="ghost">
                    <Link href={("/trade/orders/new" as Route)}>进入标准下单入口</Link>
                  </Button>
                </div>
              </div>
              <div className="grid gap-3">
                <SignalCard
                  icon={<Compass className="size-5" />}
                  title="五条标准链路"
                  description={
                    scenarioQuery.isSuccess
                      ? `已读取 ${scenarioQuery.data.data.length} 条冻结链路模板，顺序保持 S1 -> S5。`
                      : "当前先展示冻结五条标准链路样例；Bearer 会话建立后自动回填 platform-core 标准场景接口。"
                  }
                />
                <SignalCard
                  icon={<Radar className="size-5" />}
                  title="平台连接"
                  description={
                    healthQuery.isSuccess
                      ? "platform-core 已可达，门户可以走受控代理边界。"
                      : "当前未确认 platform-core 就绪，但门户能显式给出联调状态。"
                  }
                />
                <SignalCard
                  icon={<LockKeyhole className="size-5" />}
                  title="推荐 / 搜索会话"
                  description={
                    sessionMode === "bearer"
                      ? "Bearer 会话下允许读取 home_featured 推荐与首页搜索预览。"
                      : sessionMode === "local"
                        ? "Local Header 仅用于 auth/me 联调，首页推荐/搜索预览不会伪造为正式 Bearer 调用。"
                        : "游客态只展示官方场景与标准链路，不展示个性化推荐。"
                  }
                />
              </div>
            </div>
          </Card>

          <Card>
            <div className="flex items-center justify-between gap-3">
              <div>
                <CardTitle>场景导航</CardTitle>
                <CardDescription>
                  以标准场景模板为真值源，把首页入口按行业分组，保持五条标准链路与八个标准 SKU 的官方命名。
                </CardDescription>
              </div>
            </div>
            <div className="mt-4 grid gap-3 md:grid-cols-2">
              {usingScenarioFallback ? (
                <StateCallout
                  tone={scenarioQuery.isError ? "warning" : "neutral"}
                  title={
                    sessionMode === "bearer"
                      ? scenarioQuery.isPending
                        ? "场景模板加载中，先展示冻结入口"
                        : "标准场景暂未回填，已切到冻结入口"
                      : "当前展示冻结标准场景入口"
                  }
                  description={
                    sessionMode === "bearer"
                      ? scenarioQuery.isPending
                        ? "正在通过 /api/v1/catalog/standard-scenarios 读取首页标准场景；在结果返回前先保留冻结五链路入口。"
                        : "当前未从 platform-core 读取到标准场景模板，首页先展示冻结五链路与官方 SKU 映射。"
                      : "未登录或处于本地 header 占位时，首页仍保留官方五链路入口，不把标准场景导航整体锁空。"
                  }
                />
              ) : null}
              {industryGroups.map((group) => (
                <IndustryCard key={group.key} group={group} />
              ))}
            </div>
          </Card>

          <Card id="standard-chains">
            <div className="flex flex-wrap items-end justify-between gap-3">
              <div>
                <CardTitle>标准链路快捷入口</CardTitle>
                <CardDescription>
                  五条链路按冻结顺序直出，不合并 `SHARE_RO / QRY_LITE / RPT_STD`
                  这些独立 SKU。
                </CardDescription>
              </div>
              <div className="flex flex-wrap gap-2">
                {skuCoverage.length ? (
                  skuCoverage.map((sku) => <Badge key={sku}>{sku}</Badge>)
                ) : (
                  <Badge>SKU pending</Badge>
                )}
              </div>
            </div>
            <div className="mt-4 grid gap-3">
              {usingScenarioFallback ? (
                <StateCallout
                  tone={scenarioQuery.isError ? "warning" : "neutral"}
                  title={
                    sessionMode === "bearer"
                      ? scenarioQuery.isPending
                        ? "标准链路加载中，先展示冻结样例"
                        : "标准链路已切到冻结样例"
                      : "标准链路使用冻结官方样例"
                  }
                  description={
                    sessionMode === "bearer"
                      ? scenarioQuery.isPending
                        ? "Bearer 会话正在回填标准场景模板；返回前仍保留五条官方标准链路。"
                        : "标准场景接口当前未返回成功，首页退回冻结五链路样例，但不会隐藏官方链路入口。"
                      : "未登录时仍直出官方五条标准链路，推荐位与搜索预览则继续按 Bearer 正式边界控制。"
                  }
                />
              ) : null}
              {scenarios.map((scenario) => (
                <StandardChainCard key={scenario.scenario_code} scenario={scenario} />
              ))}
            </div>
          </Card>
        </motion.section>

        <div className="grid gap-4">
          <Card>
            <CardTitle>首页联调概览</CardTitle>
            <CardDescription>
              首页当前通过 `/health/ready`、`/api/v1/catalog/standard-scenarios`、
              `GET /api/v1/recommendations` 与 `GET /api/v1/catalog/search`
              校验正式边界。
            </CardDescription>
            <div className="mt-4 grid gap-3">
              <MetricRow
                label="platform-core readiness"
                value={
                  healthQuery.isSuccess
                    ? String(healthQuery.data.data)
                    : healthQuery.isPending
                      ? "checking"
                      : "unavailable"
                }
              />
              <MetricRow
                label="标准场景模板"
                value={
                  scenarioQuery.isSuccess
                    ? `${scenarioQuery.data.data.length} / live`
                    : `${scenarios.length} / frozen`
                }
              />
              <MetricRow
                label="标准 SKU 覆盖"
                value={skuCoverage.length ? `${skuCoverage.length} / 8` : "pending"}
              />
              <MetricRow
                label="受控边界"
                value="portal-web -> /api/platform -> platform-core"
              />
            </div>
          </Card>

          <Card>
            <div className="flex items-start justify-between gap-3">
              <div>
                <CardTitle>推荐位</CardTitle>
                <CardDescription>
                  推荐位固定绑定 `home_featured`。Bearer 会话下走正式推荐 API；
                  推荐服务异常时退化为标准链路样例并保留错误提示。
                </CardDescription>
              </div>
              <Badge className="shrink-0 bg-white text-[var(--ink-strong)]">
                placement: home_featured
              </Badge>
            </div>
            <div className="mt-4 grid gap-3">
              {sessionMode === "guest" ? (
                <StateCallout
                  title="游客态不展示个性化推荐"
                  description="页面说明要求未登录时隐藏个性化内容。当前仍展示官方标准链路，登录 Bearer 后再读取正式推荐位。"
                />
              ) : sessionMode === "local" ? (
                <StateCallout
                  tone="warning"
                  title="Local Header 不替代正式推荐鉴权"
                  description="推荐正式接口要求 Authorization Bearer。当前会话仅用于 auth/me 联调，首页不会伪造推荐结果。"
                />
              ) : !subject ? (
                <StateCallout
                  tone="warning"
                  title="Bearer claims 不完整"
                  description="当前 Token 未能解析出 user_id / org_id / roles，首页无法安全发起推荐读取。"
                />
              ) : recommendationQuery.isPending ? (
                <StateCallout
                  title="推荐位加载中"
                  description="正在通过正式推荐 API 请求首页推荐位。"
                />
              ) : recommendationQuery.isError ? (
                <>
                  <StateCallout
                    tone="warning"
                    title="推荐服务已降级到标准链路样例"
                    description={formatErrorDescription(
                      recommendationQuery.error,
                      "当前保留错误提示，并退回五条标准链路样例，避免首页出现无解释空白。",
                    )}
                  />
                  {scenarios.slice(0, 5).map((scenario) => (
                    <RecommendationFallbackCard
                      key={`fallback-${scenario.scenario_code}`}
                      scenario={scenario}
                    />
                  ))}
                </>
              ) : recommendationQuery.data.data.items.length ? (
                <>
                  <div className="grid gap-3 sm:grid-cols-2">
                    {recommendationQuery.data.data.items.map((item) => (
                      <RecommendationCard key={item.recommendation_result_item_id} item={item} />
                    ))}
                  </div>
                  <MetricRow
                    label="recommendation runtime"
                    value={`${recommendationQuery.data.data.strategy_version} / ${recommendationQuery.data.data.cache_hit ? "cache_hit" : "live_compute"}`}
                  />
                </>
              ) : (
                <StateCallout
                  title="当前推荐位为空"
                  description="推荐读取成功但没有返回可见候选，首页保持标准链路入口可用。"
                />
              )}
            </div>
          </Card>

          <Card>
            <CardTitle>受控搜索入口</CardTitle>
            <CardDescription>
              首页搜索入口直连正式 `catalog/search` API。访客和 Local Header
              会话只保留入口，不会伪造成 Bearer 搜索结果。
            </CardDescription>
            <div className="mt-4 space-y-4">
              <div className="flex flex-col gap-3">
                <Input
                  value={searchKeyword}
                  onChange={(event) => setSearchKeyword(event.target.value)}
                  placeholder="输入行业、SKU、交付方式或官方标准链路关键词"
                />
                <div className="flex flex-wrap gap-2">
                  {searchPresets.map((preset) => (
                    <Button
                      key={preset.label}
                      type="button"
                      size="sm"
                      variant={searchKeyword === preset.keyword ? "default" : "secondary"}
                      onClick={() => setSearchKeyword(preset.keyword)}
                    >
                      {preset.label}
                    </Button>
                  ))}
                </div>
                <div className="flex flex-wrap gap-3">
                  <Button asChild>
                    <Link href={buildSearchHref(searchKeyword)}>
                      前往搜索页
                      <Search className="size-4" />
                    </Link>
                  </Button>
                  <div className="rounded-full bg-black/[0.04] px-4 py-2 text-xs text-[var(--ink-subtle)]">
                    搜索、推荐、详情查看都会经 `request_id` 进入审计访问留痕。
                  </div>
                </div>
              </div>

              {sessionMode === "guest" ? (
                <StateCallout
                  title="游客态仅保留搜索入口"
                  description="正式搜索 API 要求 Bearer。当前仍可带关键词进入搜索页，但首页不直接展示实时搜索结果。"
                />
              ) : sessionMode === "local" ? (
                <StateCallout
                  tone="warning"
                  title="Local Header 不触发正式搜索预览"
                  description="为避免把本地占位误当成正式鉴权，首页搜索预览只在 Bearer 会话下调用 `GET /api/v1/catalog/search`。"
                />
              ) : deferredSearchKeyword.length < 2 ? (
                <StateCallout
                  title="输入至少两个字符即可预览"
                  description="推荐使用官方标准链路关键词、行业词或 SKU 关键词，例如“工业设备运行指标”“供应链协同”“QRY_LITE”。"
                />
              ) : searchPreviewQuery.isPending ? (
                <StateCallout
                  title="搜索预览加载中"
                  description="正在通过正式搜索 API 拉取首页预览结果。"
                />
              ) : searchPreviewQuery.isError ? (
                <StateCallout
                  tone="warning"
                  title="搜索预览暂不可用"
                  description={formatErrorDescription(
                    searchPreviewQuery.error,
                    "搜索页入口保持可用，稍后可直接跳转到完整搜索页继续联调。",
                  )}
                />
              ) : searchPreviewQuery.data.data.items.length ? (
                <div className="space-y-3">
                  <MetricRow
                    label="search runtime"
                    value={`${searchPreviewQuery.data.data.backend} / ${searchPreviewQuery.data.data.cache_hit ? "cache_hit" : "fresh"} / total ${searchPreviewQuery.data.data.total}`}
                  />
                  <div className="grid gap-3">
                    {searchPreviewQuery.data.data.items.map((item) => (
                      <SearchPreviewCard key={item.entity_id} item={item} />
                    ))}
                  </div>
                </div>
              ) : (
                <StateCallout
                  title="搜索结果为空"
                  description="当前关键词没有命中首页预览。可切换上方标准链路预设，或直接进入完整搜索页。"
                />
              )}
            </div>
          </Card>
        </div>
      </div>
    </PortalRouteScaffold>
  );
}

function IndustryCard({ group }: { group: IndustryGroup }) {
  return (
    <div className="rounded-[24px] bg-black/[0.03] p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-sm font-semibold text-[var(--ink-strong)]">{group.label}</div>
          <div className="mt-2 text-sm leading-6 text-[var(--ink-soft)]">
            {group.description}
          </div>
        </div>
        <Badge className="bg-white text-[var(--ink-strong)]">
          {group.scenarios.length} 条场景
        </Badge>
      </div>
      <div className="mt-4 flex flex-wrap gap-2">
        {group.scenarios.map((scenario) => (
          <Link
            key={scenario.scenario_code}
            href={buildSearchHref(readScenarioSearchKeyword(scenario))}
            className="rounded-full bg-white/80 px-3 py-2 text-xs font-semibold text-[var(--ink-strong)] ring-1 ring-black/8 transition hover:bg-white"
          >
            {scenario.scenario_code} · {scenario.scenario_name}
          </Link>
        ))}
      </div>
    </div>
  );
}

function StandardChainCard({ scenario }: { scenario: ScenarioTemplate }) {
  const deliveryType =
    readString(readRecord(scenario.product_template)?.delivery_type) ?? "n/a";

  return (
    <div className="rounded-[24px] bg-black/[0.03] p-4">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
        <div className="space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <Badge>{scenario.scenario_code}</Badge>
            <Badge className="bg-white text-[var(--ink-strong)]">
              {scenario.primary_sku}
            </Badge>
            {scenario.supplementary_skus.map((sku) => (
              <Badge key={sku} className="bg-white text-[var(--ink-soft)]">
                {sku}
              </Badge>
            ))}
          </div>
          <div>
            <div className="text-base font-semibold text-[var(--ink-strong)]">
              {scenario.scenario_name}
            </div>
            <div className="mt-1 text-sm text-[var(--ink-soft)]">
              交付主类型 {deliveryType}，保持冻结合同 / 验收 / 退款模板命名。
            </div>
          </div>
        </div>
        <div className="flex flex-wrap gap-3">
          <Button asChild variant="secondary">
            <Link href={buildSearchHref(readScenarioSearchKeyword(scenario))}>
              搜索同类商品
              <Search className="size-4" />
            </Link>
          </Button>
          <Button asChild>
            <Link href={buildOrderHref(scenario.scenario_code)}>
              进入标准下单
              <Waypoints className="size-4" />
            </Link>
          </Button>
        </div>
      </div>
    </div>
  );
}

function RecommendationCard({ item }: { item: HomeRecommendationItem }) {
  return (
    <Link
      href={buildEntityHref(item.entity_scope, item.entity_id)}
      className="min-w-0 rounded-[24px] bg-black/[0.03] p-4 transition hover:bg-black/[0.05]"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="break-words text-sm font-semibold text-[var(--ink-strong)]">
            {item.title}
          </div>
          <div className="mt-1 text-sm text-[var(--ink-soft)]">
            {item.seller_name ?? "推荐实体"} · {item.entity_scope}
          </div>
        </div>
        <Badge className="bg-white text-[var(--ink-strong)]">
          {item.final_score.toFixed(2)}
        </Badge>
      </div>
      <div className="mt-3 flex flex-wrap gap-2">
        {item.explanation_codes.slice(0, 3).map((code) => (
          <Badge
            key={code}
            className="max-w-full whitespace-normal break-all bg-white text-[var(--ink-soft)]"
          >
            {code}
          </Badge>
        ))}
      </div>
      <div className="mt-3 text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {item.price && item.currency_code
          ? `${item.currency_code} ${item.price}`
          : "价格由详情页继续确认"}
      </div>
    </Link>
  );
}

function RecommendationFallbackCard({ scenario }: { scenario: ScenarioTemplate }) {
  return (
    <div className="rounded-[24px] bg-black/[0.03] p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-sm font-semibold text-[var(--ink-strong)]">
            {scenario.scenario_name}
          </div>
          <div className="mt-1 text-sm text-[var(--ink-soft)]">
            推荐位异常时退回官方标准链路样例，避免首页空白。
          </div>
        </div>
        <Badge className="bg-white text-[var(--ink-strong)]">
          {scenario.primary_sku}
        </Badge>
      </div>
    </div>
  );
}

function SearchPreviewCard({ item }: { item: SearchPreviewItem }) {
  return (
    <Link
      href={buildEntityHref(item.entity_scope === "seller" ? "seller" : "product", item.entity_id)}
      className="min-w-0 rounded-[24px] bg-black/[0.03] p-4 transition hover:bg-black/[0.05]"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="break-words text-sm font-semibold text-[var(--ink-strong)]">
            {item.title}
          </div>
          <div className="mt-1 text-sm text-[var(--ink-soft)]">
            {item.seller_name ?? item.subtitle ?? "搜索预览结果"}
          </div>
        </div>
        <Badge className="bg-white text-[var(--ink-strong)]">
          {item.entity_scope}
        </Badge>
      </div>
      <div className="mt-3 flex flex-wrap gap-2">
        {item.delivery_modes.slice(0, 2).map((mode) => (
          <Badge
            key={mode}
            className="max-w-full whitespace-normal break-all bg-white text-[var(--ink-soft)]"
          >
            {mode}
          </Badge>
        ))}
        {item.tags.slice(0, 2).map((tag) => (
          <Badge
            key={tag}
            className="max-w-full whitespace-normal break-all bg-white text-[var(--ink-soft)]"
          >
            {tag}
          </Badge>
        ))}
      </div>
    </Link>
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
      <div className="flex items-center gap-2 text-sm font-semibold">
        <Sparkles className="size-4" />
        {title}
      </div>
      <div className="mt-2 text-sm leading-6">{description}</div>
    </div>
  );
}

function SignalCard({
  icon,
  title,
  description,
}: {
  icon: ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="rounded-[24px] bg-white/10 p-4 text-white backdrop-blur">
      <div className="flex items-center gap-3">
        <div className="rounded-full bg-white/12 p-2">{icon}</div>
        <div className="text-sm font-semibold">{title}</div>
      </div>
      <div className="mt-3 text-sm leading-6 text-white/75">{description}</div>
    </div>
  );
}

function MetricRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[24px] bg-black/[0.04] p-4">
      <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 text-sm font-semibold text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}

function buildHomeFeaturedQuery(subject?: SessionSubject): RecommendationsQuery {
  const orgId = subject?.tenant_id ?? subject?.org_id;
  if (orgId) {
    return {
      placement_code: "home_featured",
      subject_scope: "organization",
      subject_org_id: orgId,
      limit: 5,
    };
  }

  if (subject?.user_id) {
    return {
      placement_code: "home_featured",
      subject_scope: "user",
      subject_user_id: subject.user_id,
      limit: 5,
    };
  }

  return {
    placement_code: "home_featured",
    limit: 5,
  };
}

function collectSkuCoverage(scenarios: ScenarioTemplate[]) {
  const seen = new Set<string>();
  for (const scenario of scenarios) {
    seen.add(scenario.primary_sku);
    for (const sku of scenario.supplementary_skus) {
      seen.add(sku);
    }
  }

  return OFFICIAL_SKU_ORDER.filter((sku) => seen.has(sku));
}

function buildIndustryGroups(scenarios: ScenarioTemplate[]): IndustryGroup[] {
  const grouped = new Map<keyof typeof INDUSTRY_META, ScenarioTemplate[]>();

  for (const scenario of scenarios) {
    const industry =
      readString(readRecord(scenario.metadata_template)?.industry) ?? "other";
    const key = industry in INDUSTRY_META
      ? (industry as keyof typeof INDUSTRY_META)
      : "other";
    const current = grouped.get(key) ?? [];
    current.push(scenario);
    grouped.set(key, current);
  }

  return (Object.keys(INDUSTRY_META) as Array<keyof typeof INDUSTRY_META>)
    .filter((key) => (grouped.get(key) ?? []).length > 0)
    .map((key) => ({
      key,
      label: INDUSTRY_META[key].label,
      description: INDUSTRY_META[key].description,
      scenarios: grouped.get(key) ?? [],
    }));
}

function buildSearchPresets(scenarios: ScenarioTemplate[]) {
  if (!scenarios.length) {
    return [...FALLBACK_SEARCH_PRESETS];
  }

  return scenarios.map((scenario, index) => ({
    label:
      FALLBACK_SEARCH_PRESETS[index]?.label ??
      `${scenario.scenario_code} ${scenario.primary_sku}`,
    keyword: readScenarioSearchKeyword(scenario),
  }));
}

function readScenarioSearchKeyword(scenario: ScenarioTemplate) {
  const useCases = readStringArray(readRecord(scenario.metadata_template)?.use_cases);
  return scenario.scenario_name || useCases[0] || scenario.primary_sku;
}

function readRecord(value: unknown): Record<string, unknown> | undefined {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return undefined;
}

function readString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function readStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string");
}

function buildSearchHref(keyword?: string) {
  const trimmed = keyword?.trim();
  return (
    trimmed
      ? `/search?q=${encodeURIComponent(trimmed)}`
      : "/search"
  ) as Route;
}

function buildOrderHref(scenarioCode: string) {
  return `/trade/orders/new?scenario=${encodeURIComponent(scenarioCode)}` as Route;
}

function buildEntityHref(entityScope: string, entityId: string) {
  return (entityScope === "seller"
    ? `/sellers/${entityId}`
    : `/products/${entityId}`) as Route;
}

function formatErrorDescription(error: unknown, fallback: string) {
  if (error instanceof PlatformApiError) {
    const requestId = error.requestId ? ` / request_id ${error.requestId}` : "";
    return `${error.code}: ${error.message}${requestId}`;
  }

  return fallback;
}
