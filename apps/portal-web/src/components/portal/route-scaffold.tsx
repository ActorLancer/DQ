"use client";

import { motion } from "motion/react";
import { useSearchParams } from "next/navigation";
import type { ReactNode } from "react";

import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { portalRouteMap, type PortalRouteKey } from "@/lib/portal-routes";
import { formatList } from "@/lib/utils";

import {
  PreviewStateControls,
  ScaffoldPill,
  StatePreview,
  getPreviewState,
  isRoutePreviewEnabled,
} from "./state-preview";

export function PortalRouteScaffold({
  routeKey,
  params,
  children,
}: {
  routeKey: PortalRouteKey;
  params?: Record<string, string>;
  children: ReactNode;
}) {
  const meta = portalRouteMap[routeKey];
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);

  return (
    <div className="space-y-6">
      <motion.section
        initial={{ opacity: 0, y: 18 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.28 }}
        className="grid gap-4 lg:grid-cols-[1.45fr_1fr]"
      >
        <Card className="bg-[linear-gradient(135deg,rgba(255,255,255,0.96),rgba(231,244,247,0.82),rgba(240,248,244,0.86))]">
          <div className="space-y-4">
            <div className="flex flex-wrap gap-2">
              <ScaffoldPill>{meta.group}</ScaffoldPill>
              <ScaffoldPill>{meta.key}</ScaffoldPill>
              {isRoutePreviewEnabled() ? (
                <ScaffoldPill tone="warning">preview:{preview}</ScaffoldPill>
              ) : null}
            </div>
            <div>
              <CardTitle className="text-2xl">{meta.title}</CardTitle>
              <CardDescription className="mt-2">{meta.description}</CardDescription>
            </div>
            <div className="grid gap-3 md:grid-cols-3">
              <InfoItem label="冻结路由" value={meta.path} />
              <InfoItem label="查看权限" value={meta.viewPermission} />
              <InfoItem
                label="主操作权限"
                value={formatList(meta.primaryPermissions)}
              />
            </div>
            {params ? (
              <div className="rounded-[24px] bg-black/[0.04] p-4 text-sm text-[var(--ink-soft)]">
                当前示例参数：{" "}
                {Object.entries(params)
                  .map(([key, value]) => `${key}=${value}`)
                  .join(" / ")}
              </div>
            ) : null}
            {isRoutePreviewEnabled() ? <PreviewStateControls /> : null}
          </div>
        </Card>

        <Card>
          <div className="space-y-4">
            <CardTitle>API 绑定规划</CardTitle>
            <CardDescription>
              当前页面后续只允许通过 <code>portal-web -&gt; platform-core</code>{" "}
              正式 API 链路读取或写入状态。
            </CardDescription>
            <div className="space-y-2">
              {meta.apiBindings.map((apiPath) => (
                <div
                  key={apiPath}
                  className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm text-[var(--ink-soft)]"
                >
                  {apiPath}
                </div>
              ))}
            </div>
            <div className="rounded-[24px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-4 text-sm text-[var(--warning-ink)]">
              关键动作必须携带 `X-Idempotency-Key`；高风险动作还要透传 `X-Step-Up-Token` 或等价链路。
            </div>
          </div>
        </Card>
      </motion.section>

      {preview === "ready" ? (
        <motion.section
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.06, duration: 0.28 }}
          className="space-y-4"
        >
          {children}
        </motion.section>
      ) : (
        <StatePreview state={preview} />
      )}
    </div>
  );
}

function InfoItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[24px] bg-white/70 p-4">
      <div className="text-xs uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 text-sm font-medium text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}
