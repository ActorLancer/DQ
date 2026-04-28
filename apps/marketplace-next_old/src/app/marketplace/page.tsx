import { MarketplaceFilters } from "@/components/features/marketplace-filters";
import { MarketplaceResults } from "@/components/features/marketplace-results";
import { PageHeader } from "@/components/templates/page-header";
import { WorkspaceGrid } from "@/components/templates/workspace-grid";
import { Card } from "@/components/ui/card";
import { listProducts, MarketplaceFilters as QueryFilters } from "@/lib/repository";

function parseParam(searchParams: Record<string, string | string[] | undefined>, key: string) {
  const value = searchParams[key];
  if (!value) {
    return [];
  }
  const raw = Array.isArray(value) ? value.join(",") : value;
  return raw.split(",").filter(Boolean);
}

export default async function MarketplacePage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const resolved = await searchParams;
  const filters: QueryFilters = {
    q: typeof resolved.q === "string" ? resolved.q : undefined,
    industry: parseParam(resolved, "industry"),
    dataType: parseParam(resolved, "dataType"),
    delivery: parseParam(resolved, "delivery"),
    priceMode: parseParam(resolved, "priceMode"),
    update: parseParam(resolved, "update"),
  };
  const products = await listProducts(filters);
  const avgQuality = products.length
    ? Math.round(products.reduce((sum, item) => sum + item.qualityScore, 0) / products.length)
    : 0;

  return (
    <div className="grid h-full grid-rows-[auto_1fr] gap-3 overflow-hidden">
      <PageHeader
        crumbs={[
          { label: "Marketplace" },
          { label: "Explore", current: true },
        ]}
        title="数据市场"
        description="按行业、类型、交付、价格和更新频率组合筛选。URL 状态可直接分享复现。"
        metrics={[
          { label: "Results", value: String(products.length) },
          { label: "Trial", value: String(products.filter((item) => item.trial).length) },
          { label: "API Products", value: String(products.filter((item) => item.dataType === "api").length) },
          { label: "Approval Required", value: String(products.filter((item) => item.pii === "approval_required").length) },
          { label: "Avg Quality", value: String(avgQuality) },
        ]}
      />
      <WorkspaceGrid
        left={<MarketplaceFilters />}
        center={<MarketplaceResults products={products} defaultQuery={filters.q ?? ""} />}
        right={
          <div className="p-3">
            <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">Compare & Save</p>
            <div className="mt-2 space-y-2">
              <Card className="rounded-lg p-3 text-xs text-slate-700">收藏夹 (3)</Card>
              <Card className="rounded-lg p-3 text-xs text-slate-700">对比栏 (2)</Card>
              <Card className="rounded-lg p-3 text-xs text-slate-700">联系供应商工单 (5)</Card>
            </div>
            <Card className="mt-3 rounded-lg p-3 text-xs text-slate-600">
              URL 可分享筛选状态已启用。当前链接包含行业/类型/频率等筛选，复制即可协作复现。
            </Card>
          </div>
        }
      />
    </div>
  );
}
