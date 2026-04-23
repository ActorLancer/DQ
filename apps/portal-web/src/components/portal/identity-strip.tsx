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

  return (
    <Card className="border-none bg-[linear-gradient(135deg,rgba(15,92,126,0.12),rgba(255,255,255,0.96),rgba(11,132,101,0.10))] p-4">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <div className="text-sm font-medium text-[var(--ink-subtle)]">
            当前主体 / 角色 / 租户 / 作用域
          </div>
          <div className="mt-1 flex flex-wrap items-center gap-2 text-sm text-[var(--ink-soft)]">
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
        <div className="text-xs text-[var(--ink-subtle)]">
          {sessionMode === "bearer" && initialSubject
            ? `${sessionLabel} 已按 Keycloak token claims 建立会话。`
            : authQuery.isPending
              ? "正在向 platform-core 同步当前会话上下文…"
              : authQuery.isError
                ? "未建立可验证会话，当前为公开浏览态。"
                : `${sessionLabel} 已通过 /api/v1/auth/me 校验。`}
        </div>
      </div>
    </Card>
  );
}
