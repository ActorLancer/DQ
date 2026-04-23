"use client";

import Link from "next/link";
import type { Route } from "next";
import { usePathname } from "next/navigation";

import { Badge } from "@/components/ui/badge";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { primaryNavigation, portalRouteMap } from "@/lib/portal-routes";
import { cn } from "@/lib/utils";

export function PortalNavigation() {
  const pathname = usePathname();

  return (
    <Card className="overflow-hidden bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(245,250,252,0.92))]">
      <div className="flex flex-col gap-5">
        <div className="flex flex-col gap-2">
          <Badge>WEB-001</Badge>
          <CardTitle>Portal Workspace</CardTitle>
          <CardDescription>
            门户已接入正式路由、认证会话与受控 API 边界。
          </CardDescription>
        </div>
        <nav className="grid gap-2">
          {primaryNavigation.map((key) => {
            const route = portalRouteMap[key];
            const href = route.path
              .replace(":orgId", "demo-org")
              .replace(":productId", "demo-product")
              .replace(":assetId", "demo-asset")
              .replace(":orderId", "demo-order") as Route;
            const active =
              route.path === "/"
                ? pathname === "/"
                : pathname.startsWith(route.path.split("/:")[0]);

            return (
              <Link
                key={route.key}
                href={href}
                className={cn(
                  "rounded-2xl px-4 py-3 text-sm transition-colors",
                  active
                    ? "bg-[var(--accent-strong)] text-white"
                    : "bg-black/[0.03] text-[var(--ink-soft)] hover:bg-black/[0.06] hover:text-[var(--ink-strong)]",
                )}
              >
                <div className="font-medium">{route.title}</div>
                <div className={cn("mt-1 text-xs", active ? "text-white/80" : "text-[var(--ink-subtle)]")}>
                  {route.viewPermission}
                </div>
              </Link>
            );
          })}
        </nav>
      </div>
    </Card>
  );
}
