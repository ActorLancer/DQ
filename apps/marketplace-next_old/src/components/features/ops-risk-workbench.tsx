"use client";

import * as Dialog from "@radix-ui/react-dialog";
import { Search } from "lucide-react";
import { useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { OpsRiskEvent } from "@/lib/types";

export function OpsRiskWorkbench({ events }: { events: OpsRiskEvent[] }) {
  const [keyword, setKeyword] = useState("");
  const [level, setLevel] = useState<"all" | "high" | "medium" | "low">("all");
  const [active, setActive] = useState<OpsRiskEvent | null>(null);

  const filtered = useMemo(() => {
    return events.filter((event) => {
      if (level !== "all" && event.riskLevel !== level) {
        return false;
      }
      const text = `${event.type} ${event.subject} ${event.requestId} ${event.txHash}`.toLowerCase();
      return text.includes(keyword.trim().toLowerCase());
    });
  }, [events, keyword, level]);

  return (
    <>
      <div className="p-4">
        <div className="mb-3 rounded-lg border border-slate-200 bg-slate-50 p-2.5">
          <div className="flex items-center gap-2">
            <div className="relative min-w-[260px] flex-1">
              <Search className="pointer-events-none absolute left-2.5 top-2.5 size-4 text-slate-400" />
              <Input
                value={keyword}
                onChange={(event) => setKeyword(event.target.value)}
                placeholder="检索 request_id / subject / type"
                className="h-9 pl-8"
              />
            </div>
            <div className="inline-flex rounded-lg border border-slate-200 bg-white p-0.5">
              {(["all", "high", "medium", "low"] as const).map((item) => (
                <button
                  type="button"
                  key={item}
                  onClick={() => setLevel(item)}
                  className={`rounded-md px-2 py-1 text-xs ${
                    level === item ? "bg-slate-900 text-white" : "text-slate-600"
                  }`}
                >
                  {item.toUpperCase()}
                </button>
              ))}
            </div>
            <Button size="sm" variant="secondary">
              保存筛选视图
            </Button>
          </div>
        </div>

        <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">风控与审计日志</p>
        <table className="mt-3 w-full text-sm">
          <thead className="sticky top-0 bg-white">
            <tr className="border-b border-slate-200 text-left text-xs text-slate-500">
              <th className="pb-2">风险</th>
              <th className="pb-2">类型</th>
              <th className="pb-2">主体</th>
              <th className="pb-2">request_id</th>
              <th className="pb-2">tx_hash</th>
              <th className="pb-2">链/投影</th>
              <th className="pb-2">动作</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((event) => (
              <tr key={event.id} className="border-b border-slate-100 text-slate-700">
                <td className="py-2">
                  <Badge
                    className={
                      event.riskLevel === "high"
                        ? "border-rose-200 bg-rose-50 text-rose-600"
                        : event.riskLevel === "medium"
                          ? "border-amber-200 bg-amber-50 text-amber-700"
                          : "border-emerald-200 bg-emerald-50 text-emerald-700"
                    }
                  >
                    {event.riskLevel}
                  </Badge>
                </td>
                <td>{event.type}</td>
                <td>{event.subject}</td>
                <td>{event.requestId}</td>
                <td>{event.txHash}</td>
                <td>
                  {event.chainStatus}/{event.projectionStatus}
                </td>
                <td>
                  <button
                    type="button"
                    onClick={() => setActive(event)}
                    className="rounded-md border border-slate-200 px-2 py-1 text-xs"
                  >
                    Drill-down
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <Dialog.Root open={Boolean(active)} onOpenChange={(open) => !open && setActive(null)}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 bg-slate-900/40 backdrop-blur-sm" />
          <Dialog.Content className="fixed left-1/2 top-1/2 w-[min(840px,92vw)] -translate-x-1/2 -translate-y-1/2 rounded-xl border border-slate-200 bg-white p-4 shadow-2xl">
            <Dialog.Title className="text-base font-semibold text-slate-900">风险事件联查详情</Dialog.Title>
            {active ? (
              <div className="mt-3 grid grid-cols-2 gap-3 text-sm text-slate-700">
                <div className="rounded-lg border border-slate-200 p-3">
                  <p className="text-xs text-slate-500">request_id</p>
                  <p className="font-medium">{active.requestId}</p>
                </div>
                <div className="rounded-lg border border-slate-200 p-3">
                  <p className="text-xs text-slate-500">tx_hash</p>
                  <p className="font-medium">{active.txHash}</p>
                </div>
                <div className="rounded-lg border border-slate-200 p-3">
                  <p className="text-xs text-slate-500">subject</p>
                  <p className="font-medium">{active.subject}</p>
                </div>
                <div className="rounded-lg border border-slate-200 p-3">
                  <p className="text-xs text-slate-500">type</p>
                  <p className="font-medium">{active.type}</p>
                </div>
                <div className="rounded-lg border border-slate-200 p-3">
                  <p className="text-xs text-slate-500">chain_state</p>
                  <p className="font-medium">{active.chainStatus}</p>
                </div>
                <div className="rounded-lg border border-slate-200 p-3">
                  <p className="text-xs text-slate-500">projection_state</p>
                  <p className="font-medium">{active.projectionStatus}</p>
                </div>
              </div>
            ) : null}
            <div className="mt-4 flex justify-end">
              <Button variant="secondary" onClick={() => setActive(null)}>
                关闭
              </Button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </>
  );
}
