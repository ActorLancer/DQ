"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import * as Dialog from "@radix-ui/react-dialog";
import { LogIn, ShieldCheck, Trash2 } from "lucide-react";
import { useState, useTransition } from "react";
import { useForm, useWatch } from "react-hook-form";
import { z } from "zod";

import {
  connectPortalSession,
  disconnectPortalSession,
} from "@/actions/session";
import { Button } from "@/components/ui/button";
import { CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { usePortalShellStore } from "@/store/portal-shell-store";

const formSchema = z.discriminatedUnion("mode", [
  z.object({
    mode: z.literal("bearer"),
    accessToken: z.string().min(20, "请输入有效 Bearer Token"),
    label: z.string().max(48).optional(),
  }),
  z.object({
    mode: z.literal("local"),
    loginId: z.string().min(1, "请输入本地测试 login_id"),
    role: z.enum(["tenant_admin", "tenant_operator", "platform_admin"]),
  }),
]);

type FormValues = z.infer<typeof formSchema>;

export function AuthPlaceholderDialog() {
  const { authDialogOpen, setAuthDialogOpen } = usePortalShellStore();
  const [feedback, setFeedback] = useState<string | null>(null);
  const [isPending, startTransition] = useTransition();
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      mode: "bearer",
      accessToken: "",
      label: "Local Keycloak Access Token",
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
              <CardTitle>门户登录态占位</CardTitle>
              <CardDescription>
                当前阶段不在前端实现正式登录页，而是通过已获取的 Bearer Token 或本地测试身份注入 HttpOnly Cookie。
              </CardDescription>
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
                  label: "Local Keycloak Access Token",
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
                  loginId: "buyer.demo",
                  role: "tenant_operator",
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
                const result = await connectPortalSession(values);
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
                  <Input {...form.register("label")} placeholder="Local Keycloak Access Token" />
                </label>
                <label className="block space-y-2">
                  <span className="text-sm font-medium text-[var(--ink-strong)]">Access Token</span>
                  <Textarea
                    {...form.register("accessToken")}
                    placeholder="粘贴 Keycloak / IAM Access Token，用于 /api/v1/auth/me 与门户 Bearer 接口。"
                  />
                  <FormError message={form.getFieldState("accessToken").error?.message} />
                </label>
              </>
            ) : (
              <div className="grid gap-4 md:grid-cols-2">
                <label className="block space-y-2">
                  <span className="text-sm font-medium text-[var(--ink-strong)]">login_id</span>
                  <Input {...form.register("loginId")} placeholder="buyer.demo" />
                  <FormError message={form.getFieldState("loginId").error?.message} />
                </label>
                <label className="block space-y-2">
                  <span className="text-sm font-medium text-[var(--ink-strong)]">role</span>
                  <select
                    className="flex h-11 w-full rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]"
                    {...form.register("role")}
                  >
                    <option value="tenant_operator">tenant_operator</option>
                    <option value="tenant_admin">tenant_admin</option>
                    <option value="platform_admin">platform_admin</option>
                  </select>
                  <FormError message={form.getFieldState("role").error?.message} />
                </label>
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
                    const result = await disconnectPortalSession();
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
