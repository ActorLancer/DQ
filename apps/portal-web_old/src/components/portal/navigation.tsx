"use client";

import {
  Boxes,
  Compass,
  CreditCard,
  FileCheck2,
  PackageSearch,
  ReceiptText,
  Waypoints,
  Wrench,
} from "lucide-react";
import { motion } from "motion/react";
import Link from "next/link";
import type { Route } from "next";
import { usePathname } from "next/navigation";

import { Badge } from "@/components/ui/badge";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { primaryNavigation, portalRouteMap } from "@/lib/portal-routes";
import { cn } from "@/lib/utils";

export function PortalNavigation() {
  const pathname = usePathname();
  const iconMap = {
    portal_home: Compass,
    catalog_search: PackageSearch,
    seller_product_center: Boxes,
    order_create: CreditCard,
    order_detail: ReceiptText,
    delivery_acceptance: FileCheck2,
    billing_center: Waypoints,
    developer_home: Wrench,
  } as const;

  return (
    <Card className="overflow-hidden bg-[linear-gradient(180deg,rgba(255,255,255,0.98),rgba(236,248,252,0.92),rgba(233,246,242,0.88))]">
      <div className="flex flex-col gap-6">
        <div className="flex flex-col gap-3">
          <Badge className="portal-glow w-fit">Marketplace Portal</Badge>
          <CardTitle>Buyer & Seller Experience Hub</CardTitle>
          <CardDescription>
            交易门户采用“发现-决策-下单-交付-验收”主路径，统一承接数据产品与服务商品。
          </CardDescription>
        </div>
        <nav className="grid gap-2.5">
          {primaryNavigation.map((key) => {
            const route = portalRouteMap[key];
            const Icon = iconMap[key] ?? Compass;
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
                      ? "bg-[var(--accent-strong)] text-white shadow-[0_20px_40px_-24px_var(--accent-shadow)]"
                      : "bg-black/[0.03] text-[var(--ink-soft)] hover:-translate-y-0.5 hover:bg-black/[0.06] hover:text-[var(--ink-strong)]",
                  )}
                >
                  <div className={cn("mt-0.5 rounded-xl p-1.5", active ? "bg-white/20 text-white" : "bg-white/65 text-[var(--accent-strong)]")}>
                    <Icon className="size-4" />
                  </div>
                  <div className="min-w-0">
                    <div className="font-medium">{route.title}</div>
                    <div className={cn("mt-1 text-xs", active ? "text-white/80" : "text-[var(--ink-subtle)]")}>
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
        </nav>
      </div>
    </Card>
  );
}
