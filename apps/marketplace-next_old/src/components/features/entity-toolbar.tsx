"use client";

import { Download, Filter, Search } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function EntityToolbar() {
  const [density, setDensity] = useState<"comfortable" | "compact">("comfortable");

  return (
    <div className="mb-3 rounded-lg border border-slate-200 bg-slate-50 p-2.5">
      <div className="flex items-center gap-2">
        <div className="relative min-w-[280px] flex-1">
          <Search className="pointer-events-none absolute left-2.5 top-2.5 size-4 text-slate-400" />
          <Input className="h-9 pl-8" placeholder="搜索字段名、标签、描述..." />
        </div>
        <Button variant="secondary" size="sm" className="h-9">
          <Filter className="mr-1 size-3.5" />
          字段过滤
        </Button>
        <div className="inline-flex rounded-lg border border-slate-200 bg-white p-0.5">
          <button
            type="button"
            onClick={() => setDensity("comfortable")}
            className={`rounded-md px-2 py-1 text-xs ${
              density === "comfortable" ? "bg-slate-900 text-white" : "text-slate-600"
            }`}
          >
            Comfortable
          </button>
          <button
            type="button"
            onClick={() => setDensity("compact")}
            className={`rounded-md px-2 py-1 text-xs ${
              density === "compact" ? "bg-slate-900 text-white" : "text-slate-600"
            }`}
          >
            Compact
          </button>
        </div>
        <Button variant="secondary" size="sm" className="h-9">
          <Download className="mr-1 size-3.5" />
          导出当前视图
        </Button>
      </div>
      <div className="mt-2 flex flex-wrap gap-1.5">
        <Badge>Schema 受权限控制</Badge>
        <Badge>Sample 限前100行</Badge>
        <Badge>PII 字段脱敏</Badge>
        <Badge>下载需审批授权</Badge>
      </div>
    </div>
  );
}
