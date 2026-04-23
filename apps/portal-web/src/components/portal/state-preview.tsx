"use client";

import { AlertTriangle, Ban, Box, LoaderCircle } from "lucide-react";
import type { Route } from "next";
import {
  usePathname,
  useRouter,
  useSearchParams,
  type ReadonlyURLSearchParams,
} from "next/navigation";
import type { ReactNode } from "react";
import { startTransition } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

const previewStates = ["ready", "loading", "empty", "error", "forbidden"] as const;

export type PreviewState = (typeof previewStates)[number];

export function PreviewStateControls() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const current = (searchParams.get("preview") as PreviewState | null) ?? "ready";

  return (
    <div className="flex flex-wrap gap-2">
      {previewStates.map((state) => (
        <Button
          key={state}
          variant={current === state ? "default" : "secondary"}
          size="sm"
          onClick={() => {
            const params = new URLSearchParams(searchParams.toString());
            if (state === "ready") {
              params.delete("preview");
            } else {
              params.set("preview", state);
            }
            startTransition(() => {
              router.replace(
                `${pathname}${params.toString() ? `?${params}` : ""}` as Route,
              );
            });
          }}
        >
          {state}
        </Button>
      ))}
    </div>
  );
}

export function StatePreview({ state }: { state: PreviewState }) {
  if (state === "loading") {
    return (
      <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
        <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
          <LoaderCircle className="size-8 animate-spin" />
          <CardTitle>加载态预演</CardTitle>
          <CardDescription>用于核对门户页的加载骨架与骨架高度。</CardDescription>
        </div>
      </Card>
    );
  }

  if (state === "empty") {
    return (
      <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
        <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
          <Box className="size-8" />
          <CardTitle>空态预演</CardTitle>
          <CardDescription>当前页面暂未拿到数据，后续任务会替换为真实业务空态。</CardDescription>
        </div>
      </Card>
    );
  }

  if (state === "error") {
    return (
      <Card className="flex min-h-64 items-center justify-center border-[var(--danger-ring)] bg-[var(--danger-soft)]">
        <div className="flex flex-col items-center gap-3 text-center text-[var(--danger-ink)]">
          <AlertTriangle className="size-8" />
          <CardTitle>错误态预演</CardTitle>
          <CardDescription className="text-[var(--danger-ink)]">
            用于验证错误码映射、重试入口和平台联调失败时的展示承接。
          </CardDescription>
        </div>
      </Card>
    );
  }

  if (state === "forbidden") {
    return (
      <Card className="flex min-h-64 items-center justify-center border-[var(--warning-ring)] bg-[var(--warning-soft)]">
        <div className="flex flex-col items-center gap-3 text-center text-[var(--warning-ink)]">
          <Ban className="size-8" />
          <CardTitle>权限态预演</CardTitle>
          <CardDescription className="text-[var(--warning-ink)]">
            当前主体无权访问该页面或主按钮，必须回退到显式权限提示，而不是静默失败。
          </CardDescription>
        </div>
      </Card>
    );
  }

  return null;
}

export function getPreviewState(searchParams: ReadonlyURLSearchParams): PreviewState {
  const preview = searchParams.get("preview");
  return previewStates.includes(preview as PreviewState)
    ? (preview as PreviewState)
    : "ready";
}

export function ScaffoldPill({
  children,
  tone = "default",
}: {
  children: ReactNode;
  tone?: "default" | "warning";
}) {
  return (
    <span
      className={cn(
        "rounded-full px-3 py-1 text-xs font-medium",
        tone === "warning"
          ? "bg-[var(--warning-soft)] text-[var(--warning-ink)]"
          : "bg-black/[0.04] text-[var(--ink-soft)]",
      )}
    >
      {children}
    </span>
  );
}
