"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";
import { useMutation, useQuery } from "@tanstack/react-query";
import { AlertTriangle, LoaderCircle, ShieldAlert } from "lucide-react";
import { motion } from "motion/react";
import type { ReactNode } from "react";
import { useForm, useWatch } from "react-hook-form";
import { z } from "zod";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { createBrowserSdk } from "@/lib/platform-sdk";

import { ConsoleRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();

const fairnessQuerySchema = z.object({
  order_id: z.string().trim().optional(),
  incident_type: z.string().trim().optional(),
  severity: z.string().trim().optional(),
  fairness_incident_status: z.string().trim().optional(),
  assigned_role_key: z.string().trim().optional(),
  assigned_user_id: z.string().trim().optional(),
  page_size: z.number().int().min(1).max(50),
});

const fairnessHandleSchema = z
  .object({
    fairness_incident_id: z.string().uuid("fairness_incident_id 必须是 UUID"),
    action: z.enum(["close", "acknowledge", "escalate"]),
    resolution_summary: z.string().trim().min(8, "处理说明至少 8 个字符"),
    auto_action_override: z.string().trim().optional(),
    freeze_settlement: z.boolean(),
    freeze_delivery: z.boolean(),
    create_dispute_suggestion: z.boolean(),
    step_up_token: z.string().trim().optional(),
    step_up_challenge_id: z.string().trim().optional(),
  })
  .superRefine((value, context) => {
    if (!value.step_up_token && !value.step_up_challenge_id) {
      context.addIssue({
        code: "custom",
        path: ["step_up_token"],
        message: "高风险处理必须提供 step_up_token 或 step_up_challenge_id",
      });
    }
  });

const fairnessReadRoles = [
  "platform_admin",
  "platform_audit_security",
  "platform_risk_settlement",
] as const;
const fairnessHandleRoles = ["platform_admin", "platform_risk_settlement"] as const;

export function RiskConsoleShell() {
  const authQuery = useQuery({
    queryKey: ["console", "risk", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
  });
  const subject = authQuery.data?.data;
  const roles = subject?.roles ?? [];
  const canRead = roles.some((role) => fairnessReadRoles.includes(role as (typeof fairnessReadRoles)[number]));
  const canHandle = roles.some((role) => fairnessHandleRoles.includes(role as (typeof fairnessHandleRoles)[number]));

  const queryForm = useForm<z.infer<typeof fairnessQuerySchema>>({
    resolver: zodResolver(fairnessQuerySchema),
    defaultValues: {
      order_id: "",
      incident_type: "",
      severity: "",
      fairness_incident_status: "open",
      assigned_role_key: "platform_risk_settlement",
      assigned_user_id: "",
      page_size: 20,
    },
  });
  const queryValues = useWatch({ control: queryForm.control });
  const incidentsQuery = useQuery({
    queryKey: ["console", "risk", "fairness-incidents", queryValues],
    enabled: canRead,
    queryFn: () =>
      sdk.ops.listFairnessIncidents({
        order_id: emptyToUndefined(queryValues.order_id),
        incident_type: emptyToUndefined(queryValues.incident_type),
        severity: emptyToUndefined(queryValues.severity),
        fairness_incident_status: emptyToUndefined(queryValues.fairness_incident_status),
        assigned_role_key: emptyToUndefined(queryValues.assigned_role_key),
        assigned_user_id: emptyToUndefined(queryValues.assigned_user_id),
        page: 1,
        page_size: queryValues.page_size,
      }),
  });

  const handleForm = useForm<z.infer<typeof fairnessHandleSchema>>({
    resolver: zodResolver(fairnessHandleSchema),
    defaultValues: {
      fairness_incident_id: "",
      action: "close",
      resolution_summary: "manual review confirmed risk and recorded handling trace",
      auto_action_override: "notify_ops",
      freeze_settlement: true,
      freeze_delivery: false,
      create_dispute_suggestion: true,
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const selectedAction = useWatch({ control: handleForm.control, name: "action" });

  const handleMutation = useMutation({
    mutationFn: (values: z.infer<typeof fairnessHandleSchema>) =>
      sdk.ops.handleFairnessIncident(
        { id: values.fairness_incident_id },
        {
          action: values.action,
          resolution_summary: values.resolution_summary,
          auto_action_override: emptyToUndefined(values.auto_action_override),
          freeze_settlement: values.freeze_settlement,
          freeze_delivery: values.freeze_delivery,
          create_dispute_suggestion: values.create_dispute_suggestion,
        },
        {
          stepUpToken: emptyToUndefined(values.step_up_token),
          stepUpChallengeId: emptyToUndefined(values.step_up_challenge_id),
        },
      ),
    onSuccess: () => {
      incidentsQuery.refetch();
    },
  });

  const incidentPage = incidentsQuery.data?.data;
  const incidents = incidentPage?.items ?? [];

  return (
    <ConsoleRouteScaffold routeKey="risk_console">
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.24 }}
        className="space-y-4"
      >
        <Card>
          <CardTitle>当前会话主体</CardTitle>
          <CardDescription>敏感页固定展示主体、角色、租户、作用域。</CardDescription>
          <div className="mt-4 grid gap-2 md:grid-cols-2">
            {[
              ["主体", subject?.display_name ?? subject?.login_id ?? subject?.user_id ?? "guest"],
              ["角色", subject?.roles?.length ? subject.roles.join(" / ") : "guest"],
              ["租户", subject?.tenant_id ?? subject?.org_id ?? "未绑定"],
              ["作用域", subject?.auth_context_level ?? "未返回"],
            ].map(([label, value]) => (
              <div key={`${label}-${value}`} className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm">
                <div className="text-xs uppercase tracking-[0.12em] text-[var(--ink-subtle)]">{label}</div>
                <div className="mt-1 font-medium text-[var(--ink-strong)]">{value}</div>
              </div>
            ))}
          </div>
        </Card>

        {!canRead ? (
          <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
            <CardTitle>权限不足</CardTitle>
            <CardDescription className="text-[var(--warning-ink)]">
              当前角色缺少 `risk.fairness_incident.read`。
            </CardDescription>
          </Card>
        ) : (
          <div className="grid gap-4 xl:grid-cols-[1.1fr_1fr]">
            <Card>
              <CardTitle>风控联查过滤器</CardTitle>
              <CardDescription>
                读取 `GET /api/v1/ops/fairness-incidents`，按订单/严重级别/分配角色过滤。
              </CardDescription>
              <form className="mt-4 grid gap-3 md:grid-cols-2" onSubmit={(event) => event.preventDefault()}>
                <Field label="order_id(optional)" error={queryForm.formState.errors.order_id?.message}>
                  <Input {...queryForm.register("order_id")} />
                </Field>
                <Field label="incident_type(optional)" error={queryForm.formState.errors.incident_type?.message}>
                  <Input {...queryForm.register("incident_type")} />
                </Field>
                <Field label="severity(optional)" error={queryForm.formState.errors.severity?.message}>
                  <Input {...queryForm.register("severity")} />
                </Field>
                <Field
                  label="fairness_incident_status(optional)"
                  error={queryForm.formState.errors.fairness_incident_status?.message}
                >
                  <Input {...queryForm.register("fairness_incident_status")} />
                </Field>
                <Field label="assigned_role_key(optional)" error={queryForm.formState.errors.assigned_role_key?.message}>
                  <Input {...queryForm.register("assigned_role_key")} />
                </Field>
                <Field label="assigned_user_id(optional)" error={queryForm.formState.errors.assigned_user_id?.message}>
                  <Input {...queryForm.register("assigned_user_id")} />
                </Field>
                <Field label="page_size" error={queryForm.formState.errors.page_size?.message}>
                  <Input type="number" {...queryForm.register("page_size", { valueAsNumber: true })} />
                </Field>
              </form>
              {incidentsQuery.isPending ? (
                <LoadingLine text="风控事件加载中" />
              ) : incidentsQuery.isError ? (
                <InlineError error={incidentsQuery.error} fallback="风控事件读取失败" />
              ) : incidents.length === 0 ? (
                <CardDescription className="mt-4">当前条件没有返回 fairness incident。</CardDescription>
              ) : (
                <div className="mt-4 space-y-2">
                  {incidents.map((item) => (
                    <button
                      className="w-full rounded-2xl border border-[var(--line-soft)] bg-white px-4 py-3 text-left hover:bg-black/[0.02]"
                      key={item.fairness_incident_id}
                      onClick={() => {
                        if (item.fairness_incident_id) {
                          handleForm.setValue("fairness_incident_id", item.fairness_incident_id);
                        }
                      }}
                      type="button"
                    >
                      <div className="flex flex-wrap items-center justify-between gap-2">
                        <span className="font-medium text-[var(--ink-strong)]">{item.fairness_incident_id}</span>
                        <Badge className="bg-black/10 text-[var(--ink-soft)]">
                          {item.severity} / {item.fairness_incident_status}
                        </Badge>
                      </div>
                      <div className="mt-1 text-xs text-[var(--ink-soft)]">
                        order={item.order_id} / incident={item.incident_type} / role={item.assigned_role_key}
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </Card>

            <Card>
              <CardTitle>处理高风险事件</CardTitle>
              <CardDescription>
                调用 `POST /api/v1/ops/fairness-incidents/{`{id}`}/handle`，必须携带 step-up。
              </CardDescription>
              <form className="mt-4 grid gap-3" onSubmit={handleForm.handleSubmit((values) => handleMutation.mutate(values))}>
                <Field label="fairness_incident_id" error={handleForm.formState.errors.fairness_incident_id?.message}>
                  <Input {...handleForm.register("fairness_incident_id")} />
                </Field>
                <Field label="action" error={handleForm.formState.errors.action?.message}>
                  <select
                    className="flex h-10 w-full rounded-md border border-[var(--line-soft)] bg-transparent px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-ring)]"
                    value={selectedAction ?? "close"}
                    onChange={(event) =>
                      handleForm.setValue("action", event.currentTarget.value as "close" | "acknowledge" | "escalate", {
                        shouldDirty: true,
                      })
                    }
                  >
                    <option value="close">close</option>
                    <option value="acknowledge">acknowledge</option>
                    <option value="escalate">escalate</option>
                  </select>
                </Field>
                <Field label="resolution_summary" error={handleForm.formState.errors.resolution_summary?.message}>
                  <Textarea rows={3} {...handleForm.register("resolution_summary")} />
                </Field>
                <Field label="auto_action_override(optional)" error={handleForm.formState.errors.auto_action_override?.message}>
                  <Input {...handleForm.register("auto_action_override")} />
                </Field>
                <Field label="X-Step-Up-Token" error={handleForm.formState.errors.step_up_token?.message}>
                  <Input {...handleForm.register("step_up_token")} />
                </Field>
                <Field label="X-Step-Up-Challenge-Id" error={handleForm.formState.errors.step_up_challenge_id?.message}>
                  <Input {...handleForm.register("step_up_challenge_id")} />
                </Field>
                <label className="rounded-2xl bg-black/[0.04] px-4 py-2 text-sm text-[var(--ink-soft)]">
                  <input className="mr-2 align-middle" type="checkbox" {...handleForm.register("freeze_settlement")} />
                  freeze_settlement
                </label>
                <label className="rounded-2xl bg-black/[0.04] px-4 py-2 text-sm text-[var(--ink-soft)]">
                  <input className="mr-2 align-middle" type="checkbox" {...handleForm.register("freeze_delivery")} />
                  freeze_delivery
                </label>
                <label className="rounded-2xl bg-black/[0.04] px-4 py-2 text-sm text-[var(--ink-soft)]">
                  <input className="mr-2 align-middle" type="checkbox" {...handleForm.register("create_dispute_suggestion")} />
                  create_dispute_suggestion
                </label>
                <Button disabled={!canHandle || handleMutation.isPending} type="submit">
                  {handleMutation.isPending ? (
                    <LoaderCircle className="size-4 animate-spin" />
                  ) : (
                    <ShieldAlert className="size-4" />
                  )}
                  提交风险处理
                </Button>
              </form>
              {!canHandle ? (
                <CardDescription className="mt-3 text-[var(--warning-ink)]">
                  当前角色缺少 `risk.fairness_incident.handle`。
                </CardDescription>
              ) : null}
              {handleMutation.isError ? (
                <InlineError error={handleMutation.error} fallback="风险事件处理失败" />
              ) : null}
              {handleMutation.data ? (
                <pre className="mt-3 max-h-56 overflow-auto rounded-2xl bg-black/[0.06] p-3 text-xs text-[var(--ink-soft)]">
                  {JSON.stringify(handleMutation.data.data ?? handleMutation.data, null, 2)}
                </pre>
              ) : null}
            </Card>
          </div>
        )}
      </motion.div>
    </ConsoleRouteScaffold>
  );
}

function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <label>
      <span className="mb-1 block text-xs text-[var(--ink-soft)]">{label}</span>
      {children}
      {error ? <span className="mt-1 block text-xs text-[var(--danger-ink)]">{error}</span> : null}
    </label>
  );
}

function LoadingLine({ text }: { text: string }) {
  return (
    <div className="mt-4 flex items-center gap-2 text-sm text-[var(--ink-soft)]">
      <LoaderCircle className="size-4 animate-spin" />
      {text}
    </div>
  );
}

function InlineError({ error, fallback }: { error: unknown; fallback: string }) {
  const description = formatPlatformErrorForDisplay(error, {
    fallbackCode: "OPS_RISK_FAILED",
    fallbackDescription: fallback,
  });
  return (
    <div className="mt-3 rounded-2xl border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-3 text-xs text-[var(--danger-ink)]">
      <div className="flex items-center gap-2 font-medium">
        <AlertTriangle className="size-4" />
        {description}
      </div>
    </div>
  );
}

function emptyToUndefined(value: string | undefined) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}
