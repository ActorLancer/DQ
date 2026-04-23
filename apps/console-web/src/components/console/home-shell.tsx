"use client";

import { useQuery } from "@tanstack/react-query";
import {
  ArrowRight,
  Boxes,
  Radar,
  ShieldEllipsis,
} from "lucide-react";
import Link from "next/link";
import type { Route } from "next";
import { motion } from "motion/react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { consoleRouteList } from "@/lib/console-routes";
import { createBrowserSdk } from "@/lib/platform-sdk";

import { ConsoleRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();

export function HomeShell() {
  const healthQuery = useQuery({
    queryKey: ["console", "health-ready"],
    queryFn: () => sdk.ops.healthReady(),
  });
  const observabilityQuery = useQuery({
    queryKey: ["console", "observability-overview"],
    queryFn: () => sdk.ops.getObservabilityOverview(),
  });
  const outboxQuery = useQuery({
    queryKey: ["console", "outbox"],
    queryFn: () => sdk.ops.listOutbox({ page: 1, page_size: 4 }),
  });
  const auditTraceQuery = useQuery({
    queryKey: ["console", "audit-traces"],
    queryFn: () => sdk.audit.searchTraces({ page: 1, page_size: 4 }),
  });

  return (
    <ConsoleRouteScaffold routeKey="console_home">
      <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
        <motion.section
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.32 }}
          className="grid gap-4"
        >
          <Card className="overflow-hidden bg-[linear-gradient(132deg,rgba(36,30,24,0.95),rgba(88,53,32,0.92),rgba(151,78,31,0.84))] text-white">
            <div className="grid gap-6 md:grid-cols-[1.1fr_0.9fr]">
              <div className="space-y-4">
                <div className="inline-flex rounded-full bg-white/15 px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-white/80">
                  Console / V1 Core
                </div>
                <h1 className="text-3xl font-semibold leading-tight">
                  控制台已接入正式路由、受控 API 代理和控制面认证会话。
                </h1>
                <p className="max-w-2xl text-sm leading-7 text-white/80">
                  审核、审计、ops 和开发者控制面统一复用同一套路由元数据、SDK 契约、代理边界与状态体系，保证联查路径和权限语义一致。
                </p>
                <div className="flex flex-wrap gap-3">
                  <Button asChild variant="secondary">
                    <Link href="/ops/audit/trace">
                      进入审计联查页
                      <ArrowRight className="size-4" />
                    </Link>
                  </Button>
                  <Button asChild variant="ghost">
                    <Link href="/developer/trace">查看开发者 trace 页</Link>
                  </Button>
                </div>
              </div>
              <div className="grid gap-3">
                <SignalCard
                  icon={<ShieldEllipsis className="size-5" />}
                  title="会话与权限"
                  description="控制台认证会话、会话 Cookie、敏感主体条和权限预演已经统一。"
                />
                <SignalCard
                  icon={<Radar className="size-5" />}
                  title="平台连接"
                  description={
                    healthQuery.isSuccess
                      ? "platform-core 已可达，控制台可以走受控代理边界。"
                      : "当前未确认 platform-core 就绪，但控制台能显式给出联调状态。"
                  }
                />
                <SignalCard
                  icon={<Boxes className="size-5" />}
                  title="控制面摘要"
                  description="首页先接通 observability / outbox / audit trace 摘要，后续任务再展开到完整联查页面。"
                />
              </div>
            </div>
          </Card>

          <Card>
            <div className="flex items-center justify-between gap-3">
              <div>
                <CardTitle>冻结路由挂载</CardTitle>
                <CardDescription>
                  以下路由已经用官方 `page_key / 路径 / 权限` 注册到控制台，不再依赖 README 或草稿命名。
                </CardDescription>
              </div>
            </div>
            <div className="mt-4 grid gap-3 md:grid-cols-2">
              {consoleRouteList.map((route) => {
                const href = route.path as Route;

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
              `/health/ready`、`/api/v1/ops/observability/overview`、`/api/v1/ops/outbox` 与 `/api/v1/audit/traces` 通过代理校验控制台与 `platform-core` 的正式边界。
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
                label="observability backends"
                value={
                  observabilityQuery.isSuccess
                    ? String(observabilityQuery.data.data.backend_statuses.length)
                    : observabilityQuery.isPending
                      ? "loading"
                      : "locked"
                }
              />
              <MetricRow
                label="outbox events"
                value={
                  outboxQuery.isSuccess
                    ? String(outboxQuery.data.data.total)
                    : outboxQuery.isPending
                      ? "loading"
                      : "locked"
                }
              />
              <MetricRow
                label="audit traces"
                value={
                  auditTraceQuery.isSuccess
                    ? String(auditTraceQuery.data.data.total)
                    : auditTraceQuery.isPending
                      ? "loading"
                      : "locked"
                }
              />
              <MetricRow
                label="受控边界"
                value="console-web -> /api/platform -> platform-core"
              />
            </div>
          </Card>

          <Card>
            <CardTitle>可观测后端快照</CardTitle>
            <CardDescription>
              成功时展示本地 observability 探针结果；无可验证会话时显式给出受控降级提示。
            </CardDescription>
            <div className="mt-4 grid gap-3">
              {observabilityQuery.isSuccess ? (
                observabilityQuery.data.data.backend_statuses.slice(0, 4).map((record) => (
                  <div
                    key={record.backend.observability_backend_id}
                    className="rounded-[24px] bg-black/[0.04] p-4"
                  >
                    <div className="text-sm font-semibold text-[var(--ink-strong)]">
                      {record.backend.backend_key}
                    </div>
                    <div className="mt-1 text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
                      {record.probe_status}
                    </div>
                  </div>
                ))
              ) : (
                <div className="rounded-[24px] bg-black/[0.04] p-4 text-sm text-[var(--ink-soft)]">
                  当前未拿到 `ops.observability.read` 返回值。可以先通过认证会话注入 Bearer 或开发联调身份，再重新校验。
                </div>
              )}
            </div>
          </Card>

          <Card>
            <CardTitle>Outbox / 审计快照</CardTitle>
            <CardDescription>
              首页先把出站事件和审计 trace 摘要挂出来，保证 `console-web` 的真实联调不是摆设。
            </CardDescription>
            <div className="mt-4 grid gap-3">
              {outboxQuery.isSuccess ? (
                outboxQuery.data.data.items.slice(0, 3).map((item) => (
                  <div key={item.outbox_event_id} className="rounded-[24px] bg-black/[0.04] p-4">
                    <div className="text-sm font-semibold text-[var(--ink-strong)]">
                      {item.event_type}
                    </div>
                    <div className="mt-1 text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
                      {item.status} / {item.target_topic}
                    </div>
                  </div>
                ))
              ) : (
                <div className="rounded-[24px] bg-black/[0.04] p-4 text-sm text-[var(--ink-soft)]">
                  当前未拿到 `ops.outbox.read` 返回值，控制台会显式保留受控降级提示。
                </div>
              )}
              {auditTraceQuery.isSuccess ? (
                auditTraceQuery.data.data.items.slice(0, 2).map((trace) => (
                  <div key={`${trace.audit_id ?? trace.request_id ?? trace.action_name}-${trace.action_name}`} className="rounded-[24px] bg-black/[0.04] p-4">
                    <div className="text-sm font-semibold text-[var(--ink-strong)]">
                      {trace.action_name}
                    </div>
                    <div className="mt-1 text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
                      {trace.result_code} / {trace.ref_type}
                    </div>
                  </div>
                ))
              ) : (
                <div className="rounded-[24px] bg-black/[0.04] p-4 text-sm text-[var(--ink-soft)]">
                  当前未拿到 `audit.trace.read` 返回值，后续 `WEB-014` 会在正式审计联查页展开。
                </div>
              )}
            </div>
          </Card>
        </div>
      </div>
    </ConsoleRouteScaffold>
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
