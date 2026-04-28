"use client";

import { useQuery } from "@tanstack/react-query";

import { createBrowserSdk } from "@/lib/platform-sdk";

import { Badge } from "../ui/badge";
import { Card } from "../ui/card";

const sdk = createBrowserSdk();

export function IdentityStrip({ sessionLabel }: { sessionLabel: string }) {
  const authQuery = useQuery({
    queryKey: ["console", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
  });

  const subject = authQuery.data?.data;
  const stateText = authQuery.isPending
    ? "正在向 platform-core 同步当前控制台会话上下文…"
    : authQuery.isError
      ? "未建立可验证控制台会话，当前为公开浏览态。"
      : `${sessionLabel} 已通过 /api/v1/auth/me 校验。`;

  return (
    <Card className="border-none bg-[linear-gradient(130deg,rgba(165,79,30,0.16),rgba(255,255,255,0.98),rgba(181,133,80,0.12))] p-4">
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
