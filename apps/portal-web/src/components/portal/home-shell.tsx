"use client";

import { useQuery } from "@tanstack/react-query";
import { ArrowRight, Radar, ShieldEllipsis } from "lucide-react";
import Link from "next/link";
import type { Route } from "next";
import { motion } from "motion/react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { portalRouteList } from "@/lib/portal-routes";
import { createBrowserSdk } from "@/lib/platform-sdk";

import { PortalRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();

export function HomeShell() {
  const healthQuery = useQuery({
    queryKey: ["portal", "health-ready"],
    queryFn: () => sdk.ops.healthReady(),
  });
  const scenarioQuery = useQuery({
    queryKey: ["portal", "standard-scenarios"],
    queryFn: () => sdk.catalog.getStandardScenarioTemplates(),
  });

  return (
    <PortalRouteScaffold routeKey="portal_home">
      <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
        <motion.section
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.32 }}
          className="grid gap-4"
        >
          <Card className="overflow-hidden bg-[linear-gradient(130deg,rgba(10,74,105,0.94),rgba(15,92,126,0.84),rgba(8,138,104,0.72))] text-white">
            <div className="grid gap-6 md:grid-cols-[1.1fr_0.9fr]">
              <div className="space-y-4">
                <div className="inline-flex rounded-full bg-white/15 px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-white/80">
                  Portal / V1 Core
                </div>
                <h1 className="text-3xl font-semibold leading-tight">
                  门户工程基线已接入正式路由、受控 API 代理和登录态占位。
                </h1>
                <p className="max-w-2xl text-sm leading-7 text-white/80">
                  `WEB-001` 当前不把首页业务做满，而是先把门户布局、官方页面键、SDK、代理边界和可验证状态体系建起来，避免后续页面各写各的。
                </p>
                <div className="flex flex-wrap gap-3">
                  <Button asChild variant="secondary">
                    <Link href="/search">
                      进入搜索骨架
                      <ArrowRight className="size-4" />
                    </Link>
                  </Button>
                  <Button asChild variant="ghost">
                    <Link href="/trade/orders/new">查看下单骨架</Link>
                  </Button>
                </div>
              </div>
              <div className="grid gap-3">
                <SignalCard
                  icon={<ShieldEllipsis className="size-5" />}
                  title="会话与权限"
                  description="登录态占位、会话 Cookie、敏感页面主体条已经统一。"
                />
                <SignalCard
                  icon={<Radar className="size-5" />}
                  title="平台连接"
                  description={
                    healthQuery.isSuccess
                      ? "platform-core 已可达，门户可以走受控代理边界。"
                      : "当前未确认 platform-core 就绪，但门户能显式给出联调状态。"
                  }
                />
              </div>
            </div>
          </Card>

          <Card>
            <div className="flex items-center justify-between gap-3">
              <div>
                <CardTitle>冻结路由挂载</CardTitle>
                <CardDescription>
                  以下路由已经用官方 `page_key / 路径 / 权限` 注册到门户，不再依赖 README 或草稿命名。
                </CardDescription>
              </div>
            </div>
            <div className="mt-4 grid gap-3 md:grid-cols-2">
              {portalRouteList.slice(0, 12).map((route) => {
                const href = route.path
                  .replace(":orgId", "demo-org")
                  .replace(":productId", "demo-product")
                  .replace(":assetId", "demo-asset")
                  .replace(":orderId", "demo-order") as Route;

                return (
                  <Link
                    key={route.key}
                    href={href}
                    className="rounded-[24px] bg-black/[0.03] p-4 transition hover:bg-black/[0.06]"
                  >
                    <div className="text-sm font-semibold text-[var(--ink-strong)]">
                      {route.title}
                    </div>
                    <div className="mt-1 text-xs text-[var(--ink-subtle)]">
                      {route.viewPermission}
                    </div>
                    <div className="mt-3 text-sm text-[var(--ink-soft)]">
                      {route.description}
                    </div>
                  </Link>
                );
              })}
            </div>
          </Card>
        </motion.section>

        <div className="grid gap-4">
          <Card>
            <CardTitle>平台联调概览</CardTitle>
            <CardDescription>
              `/health/ready` 和 `/api/v1/catalog/standard-scenarios` 通过代理校验门户与 `platform-core` 的正式边界。
            </CardDescription>
            <div className="mt-4 grid gap-3">
              <MetricRow
                label="platform-core readiness"
                value={
                  healthQuery.isSuccess
                    ? String(healthQuery.data.data)
                    : healthQuery.isPending
                      ? "checking"
                      : "unavailable"
                }
              />
              <MetricRow
                label="标准场景模板"
                value={
                  scenarioQuery.isSuccess
                    ? String(scenarioQuery.data.data.length)
                    : scenarioQuery.isPending
                      ? "loading"
                      : "locked"
                }
              />
              <MetricRow
                label="受控边界"
                value="portal-web -> /api/platform -> platform-core"
              />
            </div>
          </Card>

          <Card>
            <CardTitle>标准场景快照</CardTitle>
            <CardDescription>
              成功时展示冻结场景名与主 SKU；无可验证会话时显式给出受控降级提示。
            </CardDescription>
            <div className="mt-4 grid gap-3">
              {scenarioQuery.isSuccess ? (
                scenarioQuery.data.data.map((scenario) => (
                  <div
                    key={scenario.scenario_code}
                    className="rounded-[24px] bg-black/[0.04] p-4"
                  >
                    <div className="text-sm font-semibold text-[var(--ink-strong)]">
                      {scenario.scenario_name}
                    </div>
                    <div className="mt-1 text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
                      {scenario.primary_sku}
                    </div>
                  </div>
                ))
              ) : (
                <div className="rounded-[24px] bg-black/[0.04] p-4 text-sm text-[var(--ink-soft)]">
                  当前未拿到 `catalog.standard.scenarios.read` 返回值。可以先通过登录态占位注入 Bearer 或本地测试身份，再重新校验。
                </div>
              )}
            </div>
          </Card>
        </div>
      </div>
    </PortalRouteScaffold>
  );
}

function SignalCard({
  icon,
  title,
  description,
}: {
  icon: ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="rounded-[24px] bg-white/10 p-4 text-white backdrop-blur">
      <div className="flex items-center gap-3">
        <div className="rounded-full bg-white/12 p-2">{icon}</div>
        <div className="text-sm font-semibold">{title}</div>
      </div>
      <div className="mt-3 text-sm leading-6 text-white/75">{description}</div>
    </div>
  );
}

function MetricRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[24px] bg-black/[0.04] p-4">
      <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 text-sm font-semibold text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}
