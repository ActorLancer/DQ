"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import * as Dialog from "@radix-ui/react-dialog";
import { LogIn, ShieldCheck, Trash2 } from "lucide-react";
import { useState, useTransition } from "react";
import { useForm, useWatch } from "react-hook-form";
import { z } from "zod";

import {
  connectConsoleSession,
  disconnectConsoleSession,
} from "@/actions/session";
import { Button } from "@/components/ui/button";
import { CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { useConsoleShellStore } from "@/store/console-shell-store";

const formSchema = z.discriminatedUnion("mode", [
  z.object({
    mode: z.literal("bearer"),
    accessToken: z.string().min(20, "请输入有效 Bearer Token"),
    label: z.string().max(48).optional(),
  }),
  z.object({
    mode: z.literal("local"),
    loginId: z.string().min(1, "请输入本地测试 login_id"),
    role: z.string().min(1, "请输入本地测试 role"),
  }),
]);

type FormValues = z.infer<typeof formSchema>;

export function AuthPlaceholderDialog() {
  const { authDialogOpen, setAuthDialogOpen } = useConsoleShellStore();
  const [feedback, setFeedback] = useState<string | null>(null);
  const [isPending, startTransition] = useTransition();
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      mode: "bearer",
      accessToken: "",
      label: "Console Control Plane Token",
    },
  });

  const mode = useWatch({
    control: form.control,
    name: "mode",
  });

  return (
    <Dialog.Root open={authDialogOpen} onOpenChange={setAuthDialogOpen}>
      <Dialog.Trigger asChild>
        <Button variant="secondary">
          <LogIn className="size-4" />
          登录态占位
        </Button>
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-slate-950/40 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-[min(92vw,760px)] -translate-x-1/2 -translate-y-1/2 rounded-[32px] border border-white/60 bg-white p-6 shadow-2xl">
          <div className="flex items-start justify-between gap-6">
            <div className="space-y-2">
              <div className="inline-flex rounded-full bg-[var(--accent-soft)] px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-[var(--accent-strong)]">
                Keycloak / IAM placeholder
              </div>
              <Dialog.Title asChild>
                <CardTitle>控制台登录态占位</CardTitle>
              </Dialog.Title>
              <Dialog.Description asChild>
                <CardDescription>
                  当前阶段不在控制台实现正式登录页，而是通过已获取的 Bearer Token 或本地测试身份注入 HttpOnly Cookie。
                </CardDescription>
              </Dialog.Description>
            </div>
            <Dialog.Close asChild>
              <Button variant="ghost" size="sm">
                关闭
              </Button>
            </Dialog.Close>
          </div>

          <div className="mt-6 flex gap-3">
            <Button
              type="button"
              variant={mode === "bearer" ? "default" : "secondary"}
              onClick={() =>
                form.reset({
                  mode: "bearer",
                  accessToken: "",
                  label: "Console Control Plane Token",
                })
              }
            >
              Bearer
            </Button>
            <Button
              type="button"
              variant={mode === "local" ? "default" : "secondary"}
              onClick={() =>
                form.reset({
                  mode: "local",
                  loginId: "platform.ops@luna.local",
                  role: "platform_admin",
                })
              }
            >
              Local Header
            </Button>
          </div>

          <form
            className="mt-6 space-y-5"
            onSubmit={form.handleSubmit((values) => {
              setFeedback(null);
              startTransition(async () => {
                const result = await connectConsoleSession(values);
                setFeedback(result.message);
                if (result.ok) {
                  setAuthDialogOpen(false);
                }
              });
            })}
          >
            {mode === "bearer" ? (
              <>
                <label className="block space-y-2">
                  <span className="text-sm font-medium text-[var(--ink-strong)]">会话标签</span>
                  <Input {...form.register("label")} placeholder="Console Control Plane Token" />
                </label>
                <label className="block space-y-2">
                  <span className="text-sm font-medium text-[var(--ink-strong)]">Access Token</span>
                  <Textarea
                    {...form.register("accessToken")}
                    placeholder="粘贴 Keycloak / IAM Access Token，用于 /api/v1/auth/me 与控制台 Bearer 接口。"
                  />
                  <FormError message={form.getFieldState("accessToken").error?.message} />
                </label>
              </>
            ) : (
              <div className="space-y-4">
                <div className="flex flex-wrap gap-2">
                  {[
                    ["platform_reviewer", "平台审核员"],
                    ["platform_admin", "平台管理员"],
                    ["platform_audit_security", "平台审计安全"],
                    ["platform_risk_settlement", "风控结算员"],
                    ["tenant_admin", "租户管理员"],
                  ].map(([role, label]) => (
                    <Button
                      key={role}
                      type="button"
                      size="sm"
                      variant="secondary"
                      onClick={() => form.setValue("role", role)}
                    >
                      {label}
                    </Button>
                  ))}
                </div>
                <div className="grid gap-4 md:grid-cols-2">
                  <label className="block space-y-2">
                    <span className="text-sm font-medium text-[var(--ink-strong)]">login_id</span>
                    <Input {...form.register("loginId")} placeholder="platform.ops@luna.local" />
                    <FormError message={form.getFieldState("loginId").error?.message} />
                  </label>
                  <label className="block space-y-2">
                    <span className="text-sm font-medium text-[var(--ink-strong)]">role</span>
                    <Input {...form.register("role")} placeholder="platform_admin" />
                    <FormError message={form.getFieldState("role").error?.message} />
                  </label>
                </div>
              </div>
            )}

            {feedback ? (
              <div className="rounded-3xl bg-black/[0.04] px-4 py-3 text-sm text-[var(--ink-soft)]">
                {feedback}
              </div>
            ) : null}

            <div className="flex flex-wrap items-center gap-3">
              <Button type="submit" disabled={isPending}>
                <ShieldCheck className="size-4" />
                {isPending ? "验证中…" : "验证并写入会话"}
              </Button>
              <Button
                type="button"
                variant="warning"
                disabled={isPending}
                onClick={() => {
                  startTransition(async () => {
                    const result = await disconnectConsoleSession();
                    setFeedback(result.message);
                  });
                }}
              >
                <Trash2 className="size-4" />
                清空会话
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function FormError({ message }: { message?: string }) {
  if (!message) {
    return null;
  }
  return <div className="text-sm text-[var(--danger-ink)]">{message}</div>;
}
