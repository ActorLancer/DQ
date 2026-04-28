"use client";

import { ChevronDown } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { ProductField } from "@/lib/types";

export function SchemaTable({ fields }: { fields: ProductField[] }) {
  const [expanded, setExpanded] = useState<string | null>(null);

  return (
    <div className="rounded-xl border border-slate-200">
      <div className="grid grid-cols-[1.3fr_0.9fr_2fr_1fr_0.8fr_0.8fr] border-b border-slate-200 bg-slate-50 px-3 py-2 text-xs font-medium text-slate-600">
        <span>字段名</span>
        <span>类型</span>
        <span>描述</span>
        <span>示例</span>
        <span>敏感</span>
        <span>质量</span>
      </div>
      <div className="max-h-72 overflow-auto">
        {fields.map((field) => {
          const open = expanded === field.name;
          return (
            <div key={field.name} className="border-b border-slate-100 last:border-none">
              <button
                type="button"
                onClick={() => setExpanded((value) => (value === field.name ? null : field.name))}
                className="grid w-full grid-cols-[1.3fr_0.9fr_2fr_1fr_0.8fr_0.8fr] items-center px-3 py-2 text-left text-xs text-slate-700 hover:bg-slate-50"
              >
                <span className="inline-flex items-center gap-1 font-medium">
                  <ChevronDown className={`size-3 transition ${open ? "rotate-180" : ""}`} />
                  {field.name}
                </span>
                <span>{field.type}</span>
                <span className="truncate">{field.description}</span>
                <span>{field.sample}</span>
                <span>{field.sensitive ? <Badge>Yes</Badge> : "No"}</span>
                <span>{field.quality}</span>
              </button>
              {open ? (
                <div className="bg-slate-50 px-4 py-2 text-xs text-slate-600">
                  数据质量说明: 字段覆盖率 99.2%，缺失补偿策略为窗口期插值，PII 字段按租户策略脱敏。
                </div>
              ) : null}
            </div>
          );
        })}
      </div>
    </div>
  );
}
