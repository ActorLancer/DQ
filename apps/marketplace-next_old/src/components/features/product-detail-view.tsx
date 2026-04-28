"use client";

import { CheckCircle2, Loader2 } from "lucide-react";
import { startTransition, useEffect, useState } from "react";

import { AccessStepperDrawer } from "@/components/features/access-stepper-drawer";
import { EntityToolbar } from "@/components/features/entity-toolbar";
import { LineageGraph } from "@/components/features/lineage-graph";
import { SamplePreview } from "@/components/features/sample-preview";
import { SchemaTable } from "@/components/features/schema-table";
import { StickyTabs } from "@/components/features/sticky-tabs";
import { PageHeader } from "@/components/templates/page-header";
import { WorkspaceGrid } from "@/components/templates/workspace-grid";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { DataProduct, ProductField, ProductSampleRow } from "@/lib/types";
import { formatPrice } from "@/lib/utils";

export function ProductDetailView({
  product,
  fields,
  rows,
}: {
  product: DataProduct;
  fields: ProductField[];
  rows: ProductSampleRow[];
}) {
  const [tab, setTab] = useState<"overview" | "schema" | "sample" | "pricing" | "docs" | "reviews">(
    "overview",
  );
  const [railCollapsed, setRailCollapsed] = useState(false);
  const [actionState, setActionState] = useState<"idle" | "submitting" | "submitted">("idle");

  const triggerAction = () => {
    if (actionState === "submitting") {
      return;
    }
    setActionState("submitting");
    window.setTimeout(() => {
      setActionState("submitted");
      window.setTimeout(() => setActionState("idle"), 1800);
    }, 900);
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return;
      }
      if (event.key === "1") startTransition(() => setTab("overview"));
      if (event.key === "2") startTransition(() => setTab("schema"));
      if (event.key === "3") startTransition(() => setTab("sample"));
      if (event.key === "4") startTransition(() => setTab("pricing"));
      if (event.key === "5") startTransition(() => setTab("docs"));
      if (event.key === "6") startTransition(() => setTab("reviews"));
      if (event.key.toLowerCase() === "r") setRailCollapsed((prev) => !prev);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  return (
    <div className="grid h-full grid-rows-[auto_1fr] gap-3 overflow-hidden">
      <PageHeader
        crumbs={[
          { label: "Marketplace" },
          { label: "Product", current: true },
        ]}
        title={product.name}
        description={`${product.supplier} · ${product.description}`}
        metrics={[
          { label: "Fields", value: String(product.fieldCount) },
          { label: "Samples", value: String(product.sampleRows) },
          { label: "Quality", value: String(product.qualityScore) },
          { label: "Update", value: product.updateFrequency },
          { label: "Coverage", value: product.coverage },
        ]}
        actions={
          <div className="flex items-center gap-2">
            <div className="rounded-md border border-slate-200 bg-slate-50 px-2 py-1 text-[11px] text-slate-600">
              快捷键: 1-6 切换 Tab, R 折叠操作栏
            </div>
            <Button variant="secondary" size="sm" onClick={() => setRailCollapsed((prev) => !prev)}>
              {railCollapsed ? "展开操作栏" : "折叠操作栏"}
            </Button>
          </div>
        }
      />
      <WorkspaceGrid
        center={
          <div className="h-full overflow-auto p-3">
            <div className="mb-2 flex flex-wrap gap-1.5">
              {product.tags.map((tag) => (
                <Badge key={tag}>{tag}</Badge>
              ))}
            </div>
            <EntityToolbar />
            <StickyTabs value={tab} onChange={setTab} />

            {tab === "overview" ? (
              <div className="space-y-3">
                <Card className="grid grid-cols-2 gap-3 rounded-lg p-3 text-sm">
                  <div>
                    <p className="text-xs text-slate-500">覆盖范围</p>
                    <p className="font-medium text-slate-900">{product.coverage}</p>
                  </div>
                  <div>
                    <p className="text-xs text-slate-500">更新频率</p>
                    <p className="font-medium text-slate-900">{product.updateFrequency}</p>
                  </div>
                  <div>
                    <p className="text-xs text-slate-500">质量指标</p>
                    <p className="font-medium text-slate-900">{product.qualityScore} / 100</p>
                  </div>
                  <div>
                    <p className="text-xs text-slate-500">授权协议</p>
                    <p className="font-medium text-slate-900">DTP Commercial License v1</p>
                  </div>
                </Card>
                <LineageGraph />
              </div>
            ) : null}

            {tab === "schema" ? <SchemaTable fields={fields} /> : null}
            {tab === "sample" ? <SamplePreview rows={rows} /> : null}
            {tab === "pricing" ? (
              <Card className="rounded-lg p-4 text-sm text-slate-700">
                <p className="text-sm font-semibold text-slate-900">价格方案</p>
                <p className="mt-1">标准版: {formatPrice(product.priceCents, product.priceMode)}</p>
                <p className="mt-1">企业版: 专属配额、专线交付、优先审批，按合同报价。</p>
              </Card>
            ) : null}
            {tab === "docs" ? (
              <Card className="rounded-lg p-4 text-sm text-slate-700">
                <p className="text-sm font-semibold text-slate-900">API 文档 / 下载说明</p>
                <p className="mt-1">REST: `/api/v1/catalog/products/&#123;productId&#125;`</p>
                <p className="mt-1">鉴权: Bearer + Scope + Step-up（高风险）</p>
                <p className="mt-1">下载对象路径已隐藏，需通过受控下载票据。</p>
              </Card>
            ) : null}
            {tab === "reviews" ? (
              <Card className="rounded-lg p-4 text-sm text-slate-700">
                <p className="text-sm font-semibold text-slate-900">评价 / 案例</p>
                <p className="mt-1">某全国银行：贷前风险误报率下降 18%。</p>
                <p className="mt-1">某支付机构：接口响应 P95 稳定在 120ms。</p>
              </Card>
            ) : null}
          </div>
        }
        right={
          railCollapsed ? undefined : (
          <div className="h-full overflow-auto p-3">
            <div className="sticky top-0 space-y-3">
              <Card className="rounded-lg p-3">
                <p className="text-xs uppercase tracking-[0.12em] text-slate-500">购买 / 申请</p>
                <p className="mt-1 text-xl font-semibold text-slate-900">
                  {formatPrice(product.priceCents, product.priceMode)}
                </p>
                <p className="mt-1 text-xs text-slate-500">试用、审批、联系供应商始终可见</p>
                <div className="mt-3 space-y-2">
                  <AccessStepperDrawer />
                  <Button variant="secondary" className="w-full" onClick={triggerAction}>
                    {actionState === "submitting" ? (
                      <>
                        <Loader2 className="mr-1 size-3.5 animate-spin" />
                        提交中
                      </>
                    ) : (
                      "免费试用"
                    )}
                  </Button>
                  <Button variant="ghost" className="w-full border border-slate-200">
                    联系供应商
                  </Button>
                </div>
                {actionState === "submitted" ? (
                  <div className="mt-2 rounded-lg border border-emerald-200 bg-emerald-50 px-2 py-1.5 text-xs text-emerald-700">
                    <CheckCircle2 className="mr-1 inline size-3.5" />
                    已提交试用申请，审批流已创建。
                  </div>
                ) : null}
              </Card>
              <Card className="rounded-lg p-3 text-xs text-slate-600">
                <p>request_id: req-91ab20d</p>
                <p>tx_hash: 0x29af...8821</p>
                <p>链状态: confirmed</p>
                <p>投影状态: synced</p>
              </Card>
            </div>
          </div>
          )
        }
      />
    </div>
  );
}
