"use client";

import { Download, LayoutGrid, List, Save } from "lucide-react";
import { useMemo, useState } from "react";

import { ProductCard } from "@/components/features/product-card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { DataProduct } from "@/lib/types";
import { formatPrice } from "@/lib/utils";

export function MarketplaceResults({
  products,
  defaultQuery,
}: {
  products: DataProduct[];
  defaultQuery: string;
}) {
  const [viewMode, setViewMode] = useState<"cards" | "table">("cards");
  const [selected, setSelected] = useState<string[]>([]);

  const selectedProducts = useMemo(
    () => products.filter((item) => selected.includes(item.id)),
    [products, selected],
  );

  const allChecked = products.length > 0 && selected.length === products.length;

  return (
    <div className="flex h-full flex-col p-3">
      <div className="mb-3 flex items-center justify-between gap-3">
        <Input defaultValue={defaultQuery} readOnly placeholder="搜索数据产品..." className="max-w-xl" />
        <div className="flex items-center gap-2">
          <div className="inline-flex rounded-lg border border-slate-200 bg-slate-50 p-0.5">
            <button
              type="button"
              onClick={() => setViewMode("cards")}
              className={`rounded-md px-2 py-1 text-xs ${
                viewMode === "cards" ? "bg-slate-900 text-white" : "text-slate-600"
              }`}
            >
              <LayoutGrid className="mr-1 inline size-3.5" />
              Cards
            </button>
            <button
              type="button"
              onClick={() => setViewMode("table")}
              className={`rounded-md px-2 py-1 text-xs ${
                viewMode === "table" ? "bg-slate-900 text-white" : "text-slate-600"
              }`}
            >
              <List className="mr-1 inline size-3.5" />
              Table
            </button>
          </div>
          <div className="flex gap-1 rounded-lg border border-slate-200 bg-slate-50 p-1 text-xs">
            {["相关度", "最新", "热门", "价格", "评分"].map((sort) => (
              <span key={sort} className="rounded-md px-2 py-1 text-slate-600">
                {sort}
              </span>
            ))}
          </div>
        </div>
      </div>

      {selected.length > 0 ? (
        <div className="mb-2 flex items-center justify-between rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs">
          <span>已选择 {selected.length} 项</span>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="secondary">
              <Save className="mr-1 size-3.5" />
              保存视图
            </Button>
            <Button size="sm" variant="secondary">
              <Download className="mr-1 size-3.5" />
              导出比较清单
            </Button>
            <button
              type="button"
              onClick={() => setSelected([])}
              className="rounded-md border border-slate-200 px-2 py-1 text-slate-600"
            >
              清空
            </button>
          </div>
        </div>
      ) : null}

      {viewMode === "cards" ? (
        <div className="grid h-full grid-cols-3 gap-3 overflow-auto pr-1">
          {products.map((item) => (
            <div key={item.id} className="relative">
              <label className="absolute left-2 top-2 z-10 inline-flex items-center rounded border border-slate-200 bg-white px-1.5 py-0.5 text-[11px]">
                <input
                  type="checkbox"
                  className="mr-1"
                  checked={selected.includes(item.id)}
                  onChange={(event) => {
                    if (event.target.checked) {
                      setSelected((prev) => [...prev, item.id]);
                    } else {
                      setSelected((prev) => prev.filter((id) => id !== item.id));
                    }
                  }}
                />
                Compare
              </label>
              <ProductCard product={item} />
            </div>
          ))}
        </div>
      ) : (
        <div className="h-full overflow-auto rounded-lg border border-slate-200">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-white">
              <tr className="border-b border-slate-200 text-left text-xs text-slate-500">
                <th className="px-3 py-2">
                  <input
                    type="checkbox"
                    checked={allChecked}
                    onChange={(event) => {
                      setSelected(event.target.checked ? products.map((item) => item.id) : []);
                    }}
                  />
                </th>
                <th className="px-3 py-2">产品</th>
                <th className="px-3 py-2">供应商</th>
                <th className="px-3 py-2">类型</th>
                <th className="px-3 py-2">更新</th>
                <th className="px-3 py-2">质量</th>
                <th className="px-3 py-2">价格</th>
              </tr>
            </thead>
            <tbody>
              {products.map((item) => (
                <tr key={item.id} className="border-b border-slate-100 text-slate-700">
                  <td className="px-3 py-2">
                    <input
                      type="checkbox"
                      checked={selected.includes(item.id)}
                      onChange={(event) => {
                        if (event.target.checked) {
                          setSelected((prev) => [...prev, item.id]);
                        } else {
                          setSelected((prev) => prev.filter((id) => id !== item.id));
                        }
                      }}
                    />
                  </td>
                  <td className="px-3 py-2 font-medium">{item.name}</td>
                  <td className="px-3 py-2">{item.supplier}</td>
                  <td className="px-3 py-2 uppercase">{item.dataType}</td>
                  <td className="px-3 py-2">{item.updateFrequency}</td>
                  <td className="px-3 py-2">{item.qualityScore}</td>
                  <td className="px-3 py-2">{formatPrice(item.priceCents, item.priceMode)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {selectedProducts.length > 0 ? (
        <div className="mt-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600">
          对比摘要: {selectedProducts.map((item) => item.name).join(" / ")}
        </div>
      ) : null}
    </div>
  );
}
