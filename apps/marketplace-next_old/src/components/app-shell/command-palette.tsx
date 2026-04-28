"use client";

import * as Dialog from "@radix-ui/react-dialog";
import { Search } from "lucide-react";
import { usePathname, useRouter } from "next/navigation";
import { useEffect, useMemo, useState } from "react";

import { marketplaceProducts } from "@/lib/mock-data";
import { cn } from "@/lib/utils";

type CommandItem = {
  id: string;
  label: string;
  href: string;
  subtitle?: string;
};

const staticItems: CommandItem[] = [
  { id: "industry-finance", label: "行业: 金融", href: "/marketplace?industry=finance" },
  { id: "industry-ai", label: "行业: AI训练数据", href: "/marketplace?industry=ai_training" },
  { id: "docs-api", label: "API 文档中心", href: "/docs" },
  { id: "supplier-workspace", label: "供应商后台", href: "/seller" },
  { id: "ops-audit", label: "运营审计联查", href: "/ops#audit" },
] as const;

export function CommandPalette() {
  const router = useRouter();
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setOpen((value) => !value);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const items = useMemo<CommandItem[]>(() => {
    const dynamicProducts: CommandItem[] = marketplaceProducts.map((item) => ({
      id: item.id,
      label: `数据产品: ${item.name}`,
      subtitle: `${item.supplier} · ${item.industry.join(" / ")}`,
      href: `/marketplace/${item.id}`,
    }));
    return [...dynamicProducts, ...staticItems];
  }, []);

  const filtered = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) {
      return items.slice(0, 10);
    }
    return items
      .filter((item) => `${item.label} ${item.subtitle ?? ""}`.toLowerCase().includes(normalized))
      .slice(0, 12);
  }, [items, query]);

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger asChild>
        <button className="inline-flex h-9 min-w-64 items-center justify-between rounded-lg border border-slate-200 bg-white px-3 text-sm text-slate-600 transition hover:border-slate-300 hover:text-slate-900">
          <span className="inline-flex items-center gap-2">
            <Search className="size-4" />
            全局搜索与命令
          </span>
          <span className="rounded border border-slate-200 bg-slate-50 px-1.5 py-0.5 text-xs text-slate-500">
            ⌘K
          </span>
        </button>
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-slate-900/35 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-[12vh] w-[min(760px,90vw)] -translate-x-1/2 rounded-xl border border-slate-200 bg-white p-3 shadow-2xl">
          <div className="mb-3 flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-50 px-3">
            <Search className="size-4 text-slate-500" />
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="搜索数据产品、供应商、行业、字段名、API 文档..."
              className="h-10 w-full border-none bg-transparent text-sm text-slate-900 outline-none"
            />
          </div>
          <div className="max-h-[52vh] space-y-1 overflow-auto">
            {filtered.map((item) => (
              <button
                type="button"
                key={item.id}
                  onClick={() => {
                    setOpen(false);
                    if (item.href !== pathname) {
                      router.push(item.href as never);
                    }
                  }}
                className={cn(
                  "w-full rounded-lg border border-transparent px-3 py-2 text-left transition",
                  "hover:border-slate-200 hover:bg-slate-50",
                )}
              >
                <p className="text-sm font-medium text-slate-800">{item.label}</p>
                {item.subtitle ? (
                  <p className="text-xs text-slate-500">{item.subtitle}</p>
                ) : null}
              </button>
            ))}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
