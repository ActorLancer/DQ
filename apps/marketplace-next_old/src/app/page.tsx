import Link from "next/link";
import { ArrowRight, ShieldCheck, Sparkles } from "lucide-react";

import { ProductCard } from "@/components/features/product-card";
import { PageHeader } from "@/components/templates/page-header";
import { WorkspaceGrid } from "@/components/templates/workspace-grid";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { industryOptions, marketplaceProducts } from "@/lib/mock-data";

export default function HomePage() {
  return (
    <div className="grid h-full grid-rows-[auto_1fr] gap-3 overflow-hidden">
      <PageHeader
        crumbs={[
          { label: "Marketplace" },
          { label: "Home", current: true },
        ]}
        title="可信数据市场"
        description="对齐交易、审计、合规、授权与链上投影能力，面向企业级数据产品采购与分发。"
        metrics={[
          { label: "推荐产品", value: "42" },
          { label: "优质供应商", value: "18" },
          { label: "免费试用", value: "9" },
          { label: "链上留痕率", value: "99.9%" },
          { label: "API SLA", value: "99.95%" },
        ]}
        actions={
          <div className="flex gap-2">
            <Input placeholder="搜索数据产品、供应商、字段名..." className="w-80" />
            <Link
              href="/marketplace"
              className="inline-flex h-10 items-center rounded-lg bg-slate-900 px-4 text-sm font-medium text-white"
            >
              搜索
            </Link>
          </div>
        }
      />
      <WorkspaceGrid
        center={
          <div className="h-full overflow-auto p-4">
            <p className="mb-2 text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">
              热门行业分类
            </p>
            <div className="grid grid-cols-3 gap-2">
              {industryOptions.map((item) => (
                <Link
                  key={item.key}
                  href={`/marketplace?industry=${item.key}`}
                  className="rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
                >
                  {item.label}
                </Link>
              ))}
            </div>
            <div className="mt-4">
              <div className="mb-2 flex items-center justify-between">
                <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">推荐数据产品</p>
                <Link href="/marketplace" className="text-xs text-slate-600 hover:text-slate-900">
                  全部查看 <ArrowRight className="inline size-3.5" />
                </Link>
              </div>
              <div className="grid grid-cols-2 gap-3">
                {marketplaceProducts.slice(0, 4).map((item) => (
                  <ProductCard key={item.id} product={item} />
                ))}
              </div>
            </div>
          </div>
        }
        right={
          <div className="flex h-full flex-col gap-3 overflow-auto p-3">
            <Card className="rounded-lg p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">优质数据供应商</p>
              <div className="mt-3 space-y-2">
                {["TrustLedger Data Lab", "TransitScope Analytics", "MedGraph Consortium"].map((supplier) => (
                  <div key={supplier} className="rounded-lg border border-slate-200 p-3">
                    <p className="text-sm font-medium text-slate-900">{supplier}</p>
                    <p className="mt-1 text-xs text-slate-500">认证级别 A · 审计链路完备 · API SLA 99.9%</p>
                  </div>
                ))}
              </div>
            </Card>
            <Card className="rounded-lg p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">
                新上架 / 热门订阅 / 免费试用
              </p>
              <div className="mt-2 flex flex-wrap gap-2">
                <Badge>新上架 16</Badge>
                <Badge>热门订阅 42</Badge>
                <Badge>免费试用 9</Badge>
              </div>
            </Card>
            <Card className="rounded-lg p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">平台可信能力</p>
              <div className="mt-3 space-y-2">
                {[
                  ["合规", "多级合规标签 + 审批链 + 留痕审计"],
                  ["脱敏", "按字段策略动态脱敏，下载水印不可移除"],
                  ["授权", "租户/角色/作用域细粒度控制 + Step-up"],
                  ["审计", "request_id + tx_hash + 链状态 + 投影状态联查"],
                  ["API交付", "统一 SDK + OpenAPI 契约对齐 + 幂等写入"],
                ].map(([title, detail]) => (
                  <div key={title} className="rounded-lg border border-slate-200 bg-slate-50 p-3">
                    <p className="text-sm font-medium text-slate-900">
                      {title} <Sparkles className="inline size-3.5 text-slate-400" />
                    </p>
                    <p className="mt-1 text-xs text-slate-600">{detail}</p>
                  </div>
                ))}
              </div>
              <div className="mt-3 rounded-lg border border-slate-200 bg-white p-3 text-xs text-slate-500">
                <ShieldCheck className="mr-1 inline size-3.5" />
                关键动作均附带审计提示，支付/重放类动作要求二次确认。
              </div>
            </Card>
          </div>
        }
      />
    </div>
  );
}
