"use client";

import {
  Activity,
  BellRing,
  ChartColumnBig,
  FileSearch,
  Gavel,
  HandCoins,
  ScanSearch,
  ShieldCheck,
  Workflow,
} from "lucide-react";
import { motion } from "motion/react";
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
  const iconMap = {
    console_home: Activity,
    review_subjects: Gavel,
    review_products: Gavel,
    review_compliance: Gavel,
    risk_console: ShieldCheck,
    audit_trace: FileSearch,
    audit_package_export: FileSearch,
    consistency_trace: ScanSearch,
    outbox_dead_letter: Workflow,
    search_ops: ChartColumnBig,
    notification_ops: BellRing,
    developer_home: HandCoins,
    developer_apps: HandCoins,
    developer_trace: HandCoins,
    developer_assets: HandCoins,
  } as const;

  return (
    <Card className="overflow-hidden bg-[linear-gradient(180deg,rgba(255,251,246,0.98),rgba(250,240,231,0.94),rgba(255,255,255,0.92))]">
      <div className="flex flex-col gap-6">
        <div className="flex flex-col gap-3">
          <Badge className="console-glow w-fit">Control Plane</Badge>
          <CardTitle>Operations & Governance Cockpit</CardTitle>
          <CardDescription>
            控制台围绕审核、审计、运维与通知联查设计，提供可追溯、高密度、权限敏感的操作体验。
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
                  const Icon = iconMap[key] ?? Activity;
                  const href = route.path as Route;
                  const active =
                    route.path === "/"
                      ? pathname === "/"
                      : pathname.startsWith(route.path);

                  return (
                    <motion.div
                      key={route.key}
                      initial={{ opacity: 0, x: -8 }}
                      animate={{ opacity: 1, x: 0 }}
                      transition={{ duration: 0.22 }}
                    >
                      <Link
                        href={href}
                        className={cn(
                          "relative flex items-start gap-3 rounded-2xl px-4 py-3.5 text-sm",
                          active
                            ? "bg-[var(--accent-strong)] text-white shadow-[0_20px_42px_-22px_var(--accent-shadow)]"
                            : "bg-black/[0.03] text-[var(--ink-soft)] hover:-translate-y-0.5 hover:bg-black/[0.06] hover:text-[var(--ink-strong)]",
                        )}
                      >
                        <div className={cn("mt-0.5 rounded-xl p-1.5", active ? "bg-white/20 text-white" : "bg-white/65 text-[var(--accent-strong)]")}>
                          <Icon className="size-4" />
                        </div>
                        <div className="min-w-0">
                          <div className="font-medium">{route.title}</div>
                          <div
                            className={cn(
                              "mt-1 text-xs",
                              active ? "text-white/80" : "text-[var(--ink-subtle)]",
                            )}
                          >
                            {route.viewPermission}
                          </div>
                        </div>
                        {active ? (
                          <span className="absolute right-2 top-1/2 h-6 w-1 -translate-y-1/2 rounded-full bg-white/75" />
                        ) : null}
                      </Link>
                    </motion.div>
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
