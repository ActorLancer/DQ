"use client";

import Link from "next/link";
import type { Route } from "next";
import { usePathname } from "next/navigation";

import { Badge } from "@/components/ui/badge";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import {
  consoleNavigationGroups,
  consoleRouteMap,
} from "@/lib/console-routes";
import { cn } from "@/lib/utils";

export function ConsoleNavigation() {
  const pathname = usePathname();

  return (
    <Card className="overflow-hidden bg-[linear-gradient(180deg,rgba(255,248,241,0.98),rgba(248,240,231,0.95),rgba(255,255,255,0.94))]">
      <div className="flex flex-col gap-5">
        <div className="flex flex-col gap-2">
          <Badge>WEB-002</Badge>
          <CardTitle>Console Baseline</CardTitle>
          <CardDescription>
            当前只落控制台基线、路由骨架、登录态占位和受控 API 边界。
          </CardDescription>
        </div>
        <nav className="grid gap-4">
          {consoleNavigationGroups.map((group) => (
            <div key={group.label} className="space-y-2">
              <div className="px-1 text-xs font-semibold uppercase tracking-[0.2em] text-[var(--ink-subtle)]">
                {group.label}
              </div>
              <div className="grid gap-2">
                {group.keys.map((key) => {
                  const route = consoleRouteMap[key];
                  const href = route.path as Route;
                  const active =
                    route.path === "/"
                      ? pathname === "/"
                      : pathname.startsWith(route.path);

                  return (
                    <Link
                      key={route.key}
                      href={href}
                      className={cn(
                        "rounded-2xl px-4 py-3 text-sm transition-colors",
                        active
                          ? "bg-[var(--accent-strong)] text-white shadow-[0_18px_40px_-22px_var(--accent-shadow)]"
                          : "bg-black/[0.03] text-[var(--ink-soft)] hover:bg-black/[0.06] hover:text-[var(--ink-strong)]",
                      )}
                    >
                      <div className="font-medium">{route.title}</div>
                      <div
                        className={cn(
                          "mt-1 text-xs",
                          active ? "text-white/80" : "text-[var(--ink-subtle)]",
                        )}
                      >
                        {route.viewPermission}
                      </div>
                    </Link>
                  );
                })}
              </div>
            </div>
          ))}
        </nav>
      </div>
    </Card>
  );
}
