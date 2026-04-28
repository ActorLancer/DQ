"use client";

import { useQuery } from "@tanstack/react-query";

import { createBrowserSdk } from "@/lib/platform-sdk";
import type { PortalSessionPreview, PortalSession } from "@/lib/session";

import { Badge } from "../ui/badge";
import { Card } from "../ui/card";

const sdk = createBrowserSdk();

export function IdentityStrip({
  sessionLabel,
  sessionMode,
  initialSubject,
}: {
  sessionLabel: string;
  sessionMode: PortalSession["mode"];
  initialSubject: PortalSessionPreview | null;
}) {
  const authQuery = useQuery({
    queryKey: ["portal", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode === "local",
  });

  const subject = authQuery.data?.data ?? initialSubject;
  const stateText =
    sessionMode === "bearer" && initialSubject
      ? `${sessionLabel} 已按 Keycloak token claims 建立会话。`
      : authQuery.isPending
        ? "正在向 platform-core 同步当前会话上下文…"
        : authQuery.isError
          ? "未建立可验证会话，当前为公开浏览态。"
          : `${sessionLabel} 已通过 /api/v1/auth/me 校验。`;

  return (
    <Card className="border-none bg-[linear-gradient(130deg,rgba(3,111,147,0.14),rgba(255,255,255,0.98),rgba(18,170,143,0.12))] p-4">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div className="min-w-0">
          <div className="text-sm font-semibold text-[var(--ink-subtle)]">
            当前主体 / 角色 / 租户 / 作用域
          </div>
          <div className="mt-2 flex flex-wrap items-center gap-2 text-sm text-[var(--ink-soft)]">
            <Badge className="bg-white/70 text-[var(--ink-strong)]">
              主体 {subject?.display_name ?? subject?.login_id ?? "游客"}
            </Badge>
            <Badge className="bg-white/70 text-[var(--ink-strong)]">
              角色 {subject?.roles?.join(" / ") || "visitor"}
            </Badge>
            <Badge className="bg-white/70 text-[var(--ink-strong)]">
              租户 {subject?.tenant_id ?? subject?.org_id ?? "public"}
            </Badge>
            <Badge className="bg-white/70 text-[var(--ink-strong)]">
              作用域 {subject?.auth_context_level ?? "public"}
            </Badge>
          </div>
        </div>
        <div className="flex items-center gap-2 text-xs text-[var(--ink-subtle)]">
          <span
            className="inline-block size-2 rounded-full bg-[var(--accent-strong)]"
            aria-hidden
          />
          {stateText}
        </div>
      </div>
    </Card>
  );
}
