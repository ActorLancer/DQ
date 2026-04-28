"use client";

import { usePathname, useRouter, useSearchParams } from "next/navigation";

import { industryOptions } from "@/lib/mock-data";

const filterGroups = [
  {
    key: "dataType",
    label: "数据类型",
    options: [
      { key: "api", label: "API" },
      { key: "file", label: "文件" },
      { key: "share", label: "数据共享" },
      { key: "report", label: "报告" },
      { key: "model", label: "模型" },
    ],
  },
  {
    key: "delivery",
    label: "交付方式",
    options: [
      { key: "api", label: "API" },
      { key: "file_download", label: "文件下载" },
      { key: "share_link", label: "共享链接" },
    ],
  },
  {
    key: "priceMode",
    label: "价格模式",
    options: [
      { key: "subscription", label: "订阅" },
      { key: "ppu", label: "按量付费" },
      { key: "one_off", label: "一次性" },
    ],
  },
  {
    key: "update",
    label: "更新频率",
    options: [
      { key: "daily", label: "每日" },
      { key: "weekly", label: "每周" },
      { key: "monthly", label: "每月" },
      { key: "realtime", label: "实时" },
    ],
  },
] as const;

function parseList(value: string | null) {
  return value ? value.split(",").filter(Boolean) : [];
}

export function MarketplaceFilters() {
  const searchParams = useSearchParams();
  const pathname = usePathname();
  const router = useRouter();

  const updateParam = (key: string, values: string[]) => {
    const params = new URLSearchParams(searchParams.toString());
    if (values.length) {
      params.set(key, values.join(","));
    } else {
      params.delete(key);
    }
    router.replace(`${pathname}?${params.toString()}` as never);
  };

  return (
    <aside className="h-full overflow-auto rounded-2xl border border-slate-200 bg-white p-3">
      <p className="mb-3 text-xs font-semibold uppercase tracking-[0.12em] text-slate-500">筛选条件</p>
      <section className="mb-4">
        <p className="mb-2 text-xs font-medium text-slate-500">行业</p>
        <div className="grid grid-cols-2 gap-2">
          {industryOptions.map((option) => {
            const selected = parseList(searchParams.get("industry")).includes(option.key);
            return (
              <button
                key={option.key}
                type="button"
                onClick={() => {
                  const current = parseList(searchParams.get("industry"));
                  const next = selected
                    ? current.filter((item) => item !== option.key)
                    : [...current, option.key];
                  updateParam("industry", next);
                }}
                className={`rounded-lg border px-2 py-1.5 text-xs transition ${
                  selected
                    ? "border-slate-900 bg-slate-900 text-white"
                    : "border-slate-200 bg-white text-slate-600 hover:border-slate-300"
                }`}
              >
                {option.label}
              </button>
            );
          })}
        </div>
      </section>
      {filterGroups.map((group) => (
        <section key={group.key} className="mb-4">
          <p className="mb-2 text-xs font-medium text-slate-500">{group.label}</p>
          <div className="space-y-1.5">
            {group.options.map((option) => {
              const selected = parseList(searchParams.get(group.key)).includes(option.key);
              return (
                <button
                  key={option.key}
                  type="button"
                  onClick={() => {
                    const current = parseList(searchParams.get(group.key));
                    const next = selected
                      ? current.filter((item) => item !== option.key)
                      : [...current, option.key];
                    updateParam(group.key, next);
                  }}
                  className={`w-full rounded-lg border px-2 py-1.5 text-left text-xs transition ${
                    selected
                      ? "border-slate-900 bg-slate-900 text-white"
                      : "border-slate-200 bg-white text-slate-600 hover:border-slate-300"
                  }`}
                >
                  {option.label}
                </button>
              );
            })}
          </div>
        </section>
      ))}
    </aside>
  );
}
