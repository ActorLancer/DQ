"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  Ban,
  FileCheck2,
  FileUp,
  Gavel,
  GitBranch,
  LoaderCircle,
  Scale,
  ShieldAlert,
} from "lucide-react";
import { motion } from "motion/react";
import type { Route } from "next";
import { useRouter, useSearchParams } from "next/navigation";
import { startTransition, useEffect, useState, type ReactNode } from "react";
import {
  useForm,
  type FieldErrors,
  type UseFormRegisterReturn,
} from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  DISPUTE_CREATE_ALLOWED_ROLES,
  DISPUTE_READ_ALLOWED_ROLES,
  DISPUTE_RESOLVE_ALLOWED_ROLES,
  buildCreateDisputeCaseRequest,
  buildDisputeEvidenceFormData,
  buildResolveDisputeCaseRequest,
  canCreateDispute,
  canReadDispute,
  canResolveDispute,
  createDisputeCaseFormSchema,
  createDisputeIdempotencyKey,
  defaultCreateDisputeCaseValues,
  defaultResolveDisputeCaseValues,
  defaultUploadDisputeEvidenceValues,
  disputeEvidenceObjectTypes,
  disputeLookupSchema,
  disputeReasonExamples,
  formatDisputeError,
  hiddenObjectPathNotice,
  readDisputeSubjectTenant,
  resolveDisputeCaseFormSchema,
  selectActiveDisputeCase,
  unwrapCreatedDisputeCase,
  unwrapDisputeEvidence,
  unwrapDisputeResolution,
  uploadDisputeEvidenceFormSchema,
  type CreateDisputeCaseFormValues,
  type DisputeCaseResult,
  type DisputeEvidenceResult,
  type DisputeLookupValues,
  type DisputeResolutionResult,
  type OrderDisputeSummary,
  type ResolveDisputeCaseFormValues,
  type UploadDisputeEvidenceFormValues,
} from "@/lib/dispute-workflow";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import type { PortalSessionPreview } from "@/lib/session";
import {
  formatMoney,
  formatTradeError,
  orderStatusLabel,
  skuOptionLabel,
  unwrapOrderDetail,
  type OrderDetail,
} from "@/lib/order-workflow";
import { cn, formatList } from "@/lib/utils";

import { PreviewStateControls, ScaffoldPill, getPreviewState } from "./state-preview";

const sdk = createBrowserSdk();

type DisputeShellProps = {
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

type SessionSubject = NonNullable<PortalSessionPreview>;

export function DisputeWorkflowShell({
  sessionMode,
  initialSubject,
}: DisputeShellProps) {
  const searchParams = useSearchParams();
  const orderId = searchParams.get("order_id") ?? "";
  const caseId = searchParams.get("case_id") ?? "";
  const preview = getPreviewState(searchParams);
  const queryClient = useQueryClient();
  const [createdCase, setCreatedCase] = useState<DisputeCaseResult | null>(null);

  const authQuery = useQuery({
    queryKey: ["portal", "dispute", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest" && preview === "ready",
  });
  const subject = (authQuery.data?.data ?? initialSubject) as SessionSubject | null;
  const canRead = canReadDispute(subject);
  const canCreate = canCreateDispute(subject);
  const canResolve = canResolveDispute(subject);
  const orderQuery = useQuery({
    queryKey: ["portal", "dispute", "order", orderId],
    queryFn: () => sdk.trade.getOrderDetail({ id: orderId }),
    enabled: Boolean(orderId) && sessionMode !== "guest" && preview === "ready" && canRead,
  });
  const order = unwrapOrderDetail(orderQuery.data);
  const relationCase = selectActiveDisputeCase(order, caseId);
  const activeCase = relationCase ?? createdCase;
  const loading = authQuery.isLoading || orderQuery.isLoading;
  const error = authQuery.error ?? orderQuery.error;
  const refreshOrder = () => {
    void queryClient.invalidateQueries({ queryKey: ["portal", "dispute", "order", orderId] });
  };

  if (preview !== "ready") {
    return (
      <div className="space-y-6">
        <DisputeHero
          subject={subject}
          sessionMode={sessionMode}
          orderId={orderId}
          caseId={caseId}
        />
        <DisputePreviewState preview={preview} subject={subject} orderId={orderId} />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <DisputeHero
        subject={subject}
        sessionMode={sessionMode}
        orderId={orderId}
        caseId={caseId || activeCase?.case_id}
      />
      <DisputeLookupPanel orderId={orderId} caseId={caseId} />
      {sessionMode === "guest" || !canRead ? (
        <DisputePermissionPanel subject={subject} sessionMode={sessionMode} />
      ) : !orderId ? (
        <DisputeEmptyPanel title="请输入 order_id 创建或跟踪争议" />
      ) : loading ? (
        <DisputeLoadingPanel />
      ) : error ? (
        <DisputeErrorPanel
          title="争议页错误态"
          message={formatTradeError(error) || formatDisputeError(error)}
        />
      ) : !order ? (
        <DisputeEmptyPanel title="没有可展示的订单争议数据" />
      ) : (
        <>
          <DisputeOrderPanel order={order} activeCase={activeCase} />
          <CaseTimelinePanel order={order} createdCase={createdCase} activeCase={activeCase} />
          <div className="grid gap-4 xl:grid-cols-2">
            <CreateDisputeCaseForm
              orderId={order.order_id}
              canCreate={canCreate}
              onCreated={(nextCase) => {
                setCreatedCase(nextCase);
                refreshOrder();
              }}
            />
            <UploadDisputeEvidenceForm
              key={(activeCase?.case_id ?? caseId) || "empty-case"}
              activeCaseId={activeCase?.case_id ?? caseId}
              canUpload={canCreate}
              onUploaded={refreshOrder}
            />
          </div>
          <ResolveDisputeCaseForm
            activeCaseId={activeCase?.case_id ?? caseId}
            canResolve={canResolve}
            onResolved={refreshOrder}
          />
          <DisputeTrustPanel order={order} activeCase={activeCase} />
        </>
      )}
    </div>
  );
}

function DisputeHero({
  subject,
  sessionMode,
  orderId,
  caseId,
}: {
  subject: SessionSubject | null;
  sessionMode: string;
  orderId: string;
  caseId?: string;
}) {
  const meta = portalRouteMap.dispute_create;
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      className="relative overflow-hidden rounded-[34px] border border-black/10 bg-[linear-gradient(135deg,#f7efe1_0%,#e9f4ef_46%,#edf1fb_100%)] p-6 shadow-[0_24px_70px_rgba(31,44,61,0.13)]"
    >
      <div className="absolute -right-20 -top-24 size-72 rounded-full bg-[#c6764f]/20 blur-3xl" />
      <div className="absolute -bottom-24 left-1/4 size-72 rounded-full bg-[#4e8f83]/20 blur-3xl" />
      <div className="relative grid gap-6 lg:grid-cols-[1.35fr_0.65fr]">
        <div className="space-y-5">
          <div className="flex flex-wrap gap-2">
            <ScaffoldPill>WEB-013</ScaffoldPill>
            <ScaffoldPill>{meta.viewPermission}</ScaffoldPill>
            {meta.primaryPermissions.map((permission) => (
              <ScaffoldPill key={permission}>{permission}</ScaffoldPill>
            ))}
            <ScaffoldPill>dispute.case.resolve</ScaffoldPill>
          </div>
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.32em] text-[var(--accent-strong)]">
              dispute workflow
            </p>
            <h1 className="mt-3 text-3xl font-semibold tracking-[-0.05em] text-[var(--ink-strong)] md:text-5xl">
              争议提交与裁决跟踪
            </h1>
            <p className="mt-4 max-w-3xl text-sm leading-7 text-[var(--ink-soft)] md:text-base">
              创建案件、上传证据并查看裁决结果。浏览器只访问 `/api/platform`，证据页只展示
              `evidence_hash` 与审计线索，不暴露对象存储真实路径。
            </p>
          </div>
          <PreviewStateControls />
        </div>
        <SubjectCard
          subject={subject}
          sessionMode={sessionMode}
          orderId={orderId}
          caseId={caseId}
        />
      </div>
    </motion.section>
  );
}

function SubjectCard({
  subject,
  sessionMode,
  orderId,
  caseId,
}: {
  subject: SessionSubject | null;
  sessionMode: string;
  orderId: string;
  caseId?: string;
}) {
  return (
    <Card className="relative border-white/70 bg-white/70 backdrop-blur">
      <CardTitle>当前访问上下文</CardTitle>
      <div className="mt-4 grid gap-3 text-sm">
        <InfoRow label="主体" value={subject?.display_name ?? subject?.login_id ?? "未登录"} />
        <InfoRow label="角色" value={formatList(subject?.roles ?? []) || "无"} />
        <InfoRow label="租户/组织" value={readDisputeSubjectTenant(subject) || "未返回"} />
        <InfoRow label="作用域" value={subject?.auth_context_level ?? sessionMode} />
        <InfoRow label="order_id" value={orderId || "未输入"} />
        <InfoRow label="case_id" value={caseId || "未输入"} />
      </div>
    </Card>
  );
}

function DisputeLookupPanel({ orderId, caseId }: { orderId: string; caseId: string }) {
  const router = useRouter();
  const form = useForm<DisputeLookupValues>({
    resolver: zodResolver(disputeLookupSchema),
    defaultValues: { order_id: orderId, case_id: caseId },
  });
  useEffect(() => {
    form.reset({ order_id: orderId, case_id: caseId });
  }, [caseId, form, orderId]);

  return (
    <Card>
      <PanelTitle
        icon={<Scale className="size-5" />}
        title="案件上下文查询"
        description="按 order_id 读取订单争议关系；case_id 可选，用于聚焦指定案件。"
      />
      <form
        className="mt-4 grid gap-3 md:grid-cols-[1fr_1fr_auto]"
        onSubmit={form.handleSubmit((values) => {
          const params = new URLSearchParams({ order_id: values.order_id });
          if (values.case_id) {
            params.set("case_id", values.case_id);
          }
          startTransition(() => {
            router.replace(`/support/cases/new?${params}` as Route);
          });
        })}
      >
        <Input placeholder="order_id" {...form.register("order_id")} />
        <Input placeholder="case_id 可选" {...form.register("case_id")} />
        <Button type="submit">查询争议</Button>
      </form>
      <FormErrors errors={form.formState.errors} />
    </Card>
  );
}

function DisputeOrderPanel({
  order,
  activeCase,
}: {
  order: OrderDetail;
  activeCase: OrderDisputeSummary | DisputeCaseResult | null;
}) {
  const skuType = order.price_snapshot?.sku_type ?? "未返回";
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<GitBranch className="size-5" />}
        title="订单与争议状态"
        description="从 GET /api/v1/orders/{id} 读取 relations.disputes，不使用前端本地状态充当案件真相源。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <InfoTile label="order_id" value={order.order_id} />
        <InfoTile label="current_state" value={orderStatusLabel(order.current_state)} />
        <InfoTile label="sku_type" value={skuOptionLabel(skuType)} />
        <InfoTile label="amount" value={formatMoney(order.amount, order.currency_code)} />
        <InfoTile label="payment_status" value={order.payment_status} />
        <InfoTile label="settlement_status" value={order.settlement_status} />
        <InfoTile label="dispute_status" value={order.dispute_status} />
        <InfoTile label="active_case" value={activeCase?.case_id ?? "未返回"} />
      </div>
    </Card>
  );
}

function CaseTimelinePanel({
  order,
  createdCase,
  activeCase,
}: {
  order: OrderDetail;
  createdCase: DisputeCaseResult | null;
  activeCase: OrderDisputeSummary | DisputeCaseResult | null;
}) {
  const disputes = order.relations.disputes;
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<FileCheck2 className="size-5" />}
        title="案件时间线与裁决摘要"
        description="展示正式 case_id、reason_code、evidence_count、decision_code 与 penalty_code。"
      />
      {disputes.length ? (
        <div className="grid gap-3 lg:grid-cols-2">
          {disputes.map((item) => (
            <CaseCard
              key={item.case_id}
              item={item}
              active={item.case_id === activeCase?.case_id}
            />
          ))}
        </div>
      ) : createdCase ? (
        <div className="rounded-[24px] bg-[var(--warning-soft)] p-4 text-sm leading-6 text-[var(--warning-ink)]">
          已创建案件 `{createdCase.case_id}`，订单读模型尚未刷新到 relations.disputes；页面会继续以
          platform-core 返回为准。
        </div>
      ) : (
        <InlineEmpty text="当前订单没有争议案件。买方可在下方创建，页面不会使用 mock 案件填充时间线。" />
      )}
    </Card>
  );
}

function CaseCard({
  item,
  active,
}: {
  item: OrderDisputeSummary;
  active: boolean;
}) {
  return (
    <div
      className={cn(
        "rounded-[24px] border bg-white/75 p-4 text-sm",
        active ? "border-[var(--accent-strong)]" : "border-black/10",
      )}
    >
      <InfoRow label="case_id" value={item.case_id} />
      <InfoRow label="reason_code" value={item.reason_code} />
      <InfoRow label="current_status" value={item.current_status} />
      <InfoRow label="evidence_count" value={String(item.evidence_count)} />
      <InfoRow label="decision_code" value={item.decision_code ?? "未返回"} />
      <InfoRow label="penalty_code" value={item.penalty_code ?? "未返回"} />
      <InfoRow label="opened_at" value={item.opened_at} />
      <InfoRow label="updated_at" value={item.updated_at} />
    </div>
  );
}

function CreateDisputeCaseForm({
  orderId,
  canCreate,
  onCreated,
}: {
  orderId: string;
  canCreate: boolean;
  onCreated: (value: DisputeCaseResult) => void;
}) {
  const form = useForm<CreateDisputeCaseFormValues>({
    resolver: zodResolver(createDisputeCaseFormSchema),
    defaultValues: defaultCreateDisputeCaseValues(orderId),
  });
  useEffect(() => {
    form.reset(defaultCreateDisputeCaseValues(orderId));
  }, [form, orderId]);
  const mutation = useMutation({
    mutationFn: (values: CreateDisputeCaseFormValues) =>
      sdk.billing.createDisputeCase(buildCreateDisputeCaseRequest(values), {
        idempotencyKey: values.idempotency_key,
      }),
    onSuccess: (response) => {
      const nextCase = unwrapCreatedDisputeCase(response);
      if (nextCase) {
        onCreated(nextCase);
      }
    },
  });
  const result = unwrapCreatedDisputeCase(mutation.data);

  return (
    <ActionCard
      title="创建争议案件"
      description="调用 POST /api/v1/cases；仅买方争议权限角色可执行，提交携带 X-Idempotency-Key。"
      icon={<Scale className="size-5" />}
      disabled={!canCreate}
      permissionText={`需要角色：${DISPUTE_CREATE_ALLOWED_ROLES.join(" / ")}`}
    >
      <form className="space-y-4" onSubmit={form.handleSubmit((values) => mutation.mutate(values))}>
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="order_id">
            <Input {...form.register("order_id")} />
          </Field>
          <Field label="reason_code">
            <select className={selectClassName} {...form.register("reason_code")}>
              {disputeReasonExamples.map((reason) => (
                <option key={reason} value={reason}>{reason}</option>
              ))}
            </select>
          </Field>
          <Field label="requested_resolution">
            <Input {...form.register("requested_resolution")} />
          </Field>
          <Field label="claimed_amount">
            <Input {...form.register("claimed_amount")} />
          </Field>
          <Field label="evidence_scope">
            <Input {...form.register("evidence_scope")} />
          </Field>
          <Field label="blocking_effect">
            <Input {...form.register("blocking_effect")} />
          </Field>
        </div>
        <Field label="metadata JSON">
          <Textarea rows={4} {...form.register("metadata_json")} />
        </Field>
        <CheckRow label="确认争议属于当前订单与租户范围" inputProps={form.register("confirm_order_scope")} />
        <CheckRow label="确认创建案件会写入 dispute.case.create 审计" inputProps={form.register("confirm_audit")} />
        <IdempotencyField
          valueProps={form.register("idempotency_key")}
          action="case"
          onRegenerate={() => form.setValue("idempotency_key", createDisputeIdempotencyKey("case"))}
        />
        <FormErrors errors={form.formState.errors} />
        <ActionFooter pending={mutation.isPending} disabled={!canCreate} label="创建案件" />
      </form>
      {mutation.error ? <InlineError message={formatDisputeError(mutation.error)} /> : null}
      {result ? <CreatedCaseResult result={result} /> : null}
    </ActionCard>
  );
}

function UploadDisputeEvidenceForm({
  activeCaseId,
  canUpload,
  onUploaded,
}: {
  activeCaseId: string;
  canUpload: boolean;
  onUploaded: () => void;
}) {
  const [file, setFile] = useState<File | null>(null);
  const [fileError, setFileError] = useState("");
  const form = useForm<UploadDisputeEvidenceFormValues>({
    resolver: zodResolver(uploadDisputeEvidenceFormSchema),
    defaultValues: defaultUploadDisputeEvidenceValues(activeCaseId),
  });
  useEffect(() => {
    form.reset(defaultUploadDisputeEvidenceValues(activeCaseId));
  }, [activeCaseId, form]);
  const mutation = useMutation({
    mutationFn: (values: UploadDisputeEvidenceFormValues) => {
      if (!file) {
        throw new Error("DISPUTE_EVIDENCE_INVALID: 必须选择证据文件");
      }
      return sdk.billing.uploadDisputeEvidence(
        { id: values.case_id },
        buildDisputeEvidenceFormData(values, file, file.name),
        { idempotencyKey: values.idempotency_key },
      );
    },
    onSuccess: () => onUploaded(),
  });
  const result = unwrapDisputeEvidence(mutation.data);
  const disabled = !canUpload || !activeCaseId;

  return (
    <ActionCard
      title="上传证据"
      description="调用 POST /api/v1/cases/{id}/evidence；multipart 上传后只展示 evidence_hash。"
      icon={<FileUp className="size-5" />}
      disabled={disabled}
      permissionText={
        activeCaseId
          ? `需要角色：${DISPUTE_CREATE_ALLOWED_ROLES.join(" / ")}`
          : "请先创建或选择 case_id"
      }
    >
      <form
        className="space-y-4"
        onSubmit={form.handleSubmit((values) => {
          setFileError("");
          if (!file) {
            setFileError("DISPUTE_EVIDENCE_INVALID: 必须选择证据文件");
            return;
          }
          mutation.mutate(values);
        })}
      >
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="case_id">
            <Input {...form.register("case_id")} />
          </Field>
          <Field label="object_type">
            <select className={selectClassName} {...form.register("object_type")}>
              {disputeEvidenceObjectTypes.map((type) => (
                <option key={type} value={type}>{type}</option>
              ))}
            </select>
          </Field>
        </div>
        <Field label="证据文件">
          <Input
            type="file"
            onChange={(event) => {
              setFile(event.target.files?.[0] ?? null);
              setFileError("");
            }}
          />
        </Field>
        <Field label="metadata JSON">
          <Textarea rows={4} {...form.register("metadata_json")} />
        </Field>
        <CheckRow label="确认页面不会展示 object_uri / 对象真实路径" inputProps={form.register("confirm_no_raw_path")} />
        <CheckRow label="确认上传证据会写入证据链和审计留痕" inputProps={form.register("confirm_audit")} />
        <IdempotencyField
          valueProps={form.register("idempotency_key")}
          action="evidence"
          onRegenerate={() => form.setValue("idempotency_key", createDisputeIdempotencyKey("evidence"))}
        />
        <FormErrors errors={form.formState.errors} />
        {fileError ? <InlineError message={fileError} /> : null}
        <ActionFooter pending={mutation.isPending} disabled={disabled} label="上传证据" />
      </form>
      {mutation.error ? <InlineError message={formatDisputeError(mutation.error)} /> : null}
      {result ? <EvidenceResult result={result} /> : null}
    </ActionCard>
  );
}

function ResolveDisputeCaseForm({
  activeCaseId,
  canResolve,
  onResolved,
}: {
  activeCaseId: string;
  canResolve: boolean;
  onResolved: () => void;
}) {
  const form = useForm<ResolveDisputeCaseFormValues>({
    resolver: zodResolver(resolveDisputeCaseFormSchema),
    defaultValues: defaultResolveDisputeCaseValues(activeCaseId),
  });
  useEffect(() => {
    form.reset(defaultResolveDisputeCaseValues(activeCaseId));
  }, [activeCaseId, form]);
  const mutation = useMutation({
    mutationFn: (values: ResolveDisputeCaseFormValues) =>
      sdk.billing.resolveDisputeCase(
        { id: values.case_id },
        buildResolveDisputeCaseRequest(values),
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: values.step_up_token || undefined,
          stepUpChallengeId: values.step_up_challenge_id || undefined,
        },
      ),
    onSuccess: () => onResolved(),
  });
  const result = unwrapDisputeResolution(mutation.data);
  const disabled = !canResolve || !activeCaseId;

  return (
    <ActionCard
      title="平台裁决"
      description="调用 POST /api/v1/cases/{id}/resolve；高风险动作要求 step-up、人工确认和强审计。"
      icon={<Gavel className="size-5" />}
      disabled={disabled}
      permissionText={
        activeCaseId
          ? `需要角色：${DISPUTE_RESOLVE_ALLOWED_ROLES.join(" / ")}`
          : "请先创建或选择 case_id"
      }
    >
      <form className="space-y-4" onSubmit={form.handleSubmit((values) => mutation.mutate(values))}>
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          <Field label="case_id">
            <Input {...form.register("case_id")} />
          </Field>
          <Field label="decision_type">
            <Input {...form.register("decision_type")} />
          </Field>
          <Field label="decision_code">
            <Input {...form.register("decision_code")} />
          </Field>
          <Field label="liability_type">
            <Input {...form.register("liability_type")} />
          </Field>
          <Field label="penalty_code">
            <Input {...form.register("penalty_code")} />
          </Field>
          <Field label="X-Step-Up-Token">
            <Input {...form.register("step_up_token")} />
          </Field>
          <Field label="X-Step-Up-Challenge-Id">
            <Input {...form.register("step_up_challenge_id")} />
          </Field>
        </div>
        <Field label="decision_text">
          <Textarea rows={3} {...form.register("decision_text")} />
        </Field>
        <Field label="metadata JSON">
          <Textarea rows={4} {...form.register("metadata_json")} />
        </Field>
        <CheckRow label="确认平台侧职责分离和人工裁决要求已满足" inputProps={form.register("confirm_sod")} />
        <CheckRow label="确认高风险动作已完成 step-up" inputProps={form.register("confirm_step_up")} />
        <CheckRow label="确认裁决会写入 dispute.case.resolve 审计" inputProps={form.register("confirm_audit")} />
        <IdempotencyField
          valueProps={form.register("idempotency_key")}
          action="resolve"
          onRegenerate={() => form.setValue("idempotency_key", createDisputeIdempotencyKey("resolve"))}
        />
        <FormErrors errors={form.formState.errors} />
        <div className="rounded-2xl bg-[var(--warning-soft)] p-3 text-xs leading-6 text-[var(--warning-ink)]">
          裁决会影响退款、赔付、保证金、信誉和风险标记；缺少 step-up 时后端应返回统一错误码。
        </div>
        <ActionFooter pending={mutation.isPending} disabled={disabled} label="提交裁决" />
      </form>
      {mutation.error ? <InlineError message={formatDisputeError(mutation.error)} /> : null}
      {result ? <ResolutionResult result={result} /> : null}
    </ActionCard>
  );
}

function ActionCard({
  title,
  description,
  icon,
  disabled,
  permissionText,
  children,
}: {
  title: string;
  description: string;
  icon: ReactNode;
  disabled: boolean;
  permissionText: string;
  children: ReactNode;
}) {
  return (
    <Card className={cn("space-y-4", disabled ? "opacity-80" : "")}>
      <PanelTitle icon={icon} title={title} description={description} />
      {disabled ? (
        <div className="rounded-2xl bg-[var(--warning-soft)] p-3 text-sm text-[var(--warning-ink)]">
          {permissionText}
        </div>
      ) : null}
      <fieldset disabled={disabled} className="space-y-4">
        {children}
      </fieldset>
    </Card>
  );
}

function CreatedCaseResult({ result }: { result: DisputeCaseResult }) {
  return (
    <ResultBox title="案件创建结果">
      <InfoRow label="case_id" value={result.case_id} />
      <InfoRow label="order_id" value={result.order_id} />
      <InfoRow label="reason_code" value={result.reason_code} />
      <InfoRow label="current_status" value={result.current_status} />
      <InfoRow label="decision_code" value={result.decision_code ?? "未返回"} />
      <InfoRow label="penalty_code" value={result.penalty_code ?? "未返回"} />
      <InfoRow label="evidence_count" value={String(result.evidence_count)} />
    </ResultBox>
  );
}

function EvidenceResult({ result }: { result: DisputeEvidenceResult }) {
  return (
    <ResultBox title="证据上传结果">
      <InfoRow label="evidence_id" value={result.evidence_id} />
      <InfoRow label="case_id" value={result.case_id} />
      <InfoRow label="object_type" value={result.object_type} />
      <InfoRow label="evidence_hash" value={result.object_hash ?? "未返回"} />
      <InfoRow label="object_path" value="已隐藏" />
      <InfoRow label="idempotent_replay" value={String(result.idempotent_replay)} />
      <InfoRow label="created_at" value={result.created_at} />
      <div className="rounded-2xl bg-[var(--panel-muted)] p-3 text-sm text-[var(--ink-soft)]">
        {hiddenObjectPathNotice(result)}
      </div>
    </ResultBox>
  );
}

function ResolutionResult({ result }: { result: DisputeResolutionResult }) {
  return (
    <ResultBox title="裁决结果">
      <InfoRow label="decision_id" value={result.decision_id} />
      <InfoRow label="case_id" value={result.case_id} />
      <InfoRow label="order_id" value={result.order_id} />
      <InfoRow label="current_status" value={result.current_status} />
      <InfoRow label="decision_code" value={result.decision_code} />
      <InfoRow label="penalty_code" value={result.penalty_code ?? "未返回"} />
      <InfoRow label="liability_type" value={result.liability_type ?? "未返回"} />
      <InfoRow label="step_up_bound" value={String(result.step_up_bound)} />
      <InfoRow label="idempotent_replay" value={String(result.idempotent_replay)} />
      <InfoRow label="decided_at" value={result.decided_at} />
    </ResultBox>
  );
}

function DisputeTrustPanel({
  order,
  activeCase,
}: {
  order: OrderDetail;
  activeCase: OrderDisputeSummary | DisputeCaseResult | null;
}) {
  return (
    <Card className="space-y-4">
      <PanelTitle
        icon={<ShieldAlert className="size-5" />}
        title="审计与链路承接"
        description="争议页展示链路字段承接，不把审计包导出或链上证明查询提前并入本任务。"
      />
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <TrustItem label="request_id" value="由 /api/platform 与 platform-core 生成；错误态回显 request_id" />
        <TrustItem label="tx_hash" value="未返回" />
        <TrustItem label="链状态" value="未返回；证据锚定由 audit anchor 后续联查" />
        <TrustItem label="投影状态" value={activeCase?.current_status ?? order.dispute_status} />
      </div>
    </Card>
  );
}

function DisputePermissionPanel({
  subject,
  sessionMode,
}: {
  subject: SessionSubject | null;
  sessionMode: string;
}) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <PanelTitle
        icon={<Ban className="size-5" />}
        title="争议页面权限态"
        description="没有 dispute.case.read 时不读取订单争议关系，也不展示写按钮为可执行。"
      />
      <div className="mt-4 grid gap-3 text-sm text-[var(--warning-ink)]">
        <InfoRow label="当前角色" value={formatList(subject?.roles ?? []) || sessionMode} />
        <InfoRow label="读取角色" value={DISPUTE_READ_ALLOWED_ROLES.join(" / ")} />
        <InfoRow label="创建/证据角色" value={DISPUTE_CREATE_ALLOWED_ROLES.join(" / ")} />
        <InfoRow label="裁决角色" value={DISPUTE_RESOLVE_ALLOWED_ROLES.join(" / ")} />
      </div>
    </Card>
  );
}

function DisputePreviewState({
  preview,
  subject,
  orderId,
}: {
  preview: string;
  subject: SessionSubject | null;
  orderId: string;
}) {
  if (preview === "loading") {
    return <DisputeLoadingPanel />;
  }
  if (preview === "error") {
    return (
      <DisputeErrorPanel
        title="争议页错误态"
        message="DISPUTE_STATUS_INVALID / request_id=req-preview-dispute"
      />
    );
  }
  if (preview === "forbidden") {
    return <DisputePermissionPanel subject={subject} sessionMode="preview" />;
  }
  return (
    <DisputeEmptyPanel
      title={orderId ? "当前订单没有争议案件" : "请输入 order_id 创建或跟踪争议"}
    />
  );
}

function DisputeLoadingPanel() {
  return (
    <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <LoaderCircle className="size-8 animate-spin" />
        <CardTitle>争议页面加载态</CardTitle>
        <CardDescription>正在读取当前主体与订单争议关系。</CardDescription>
      </div>
    </Card>
  );
}

function DisputeEmptyPanel({ title }: { title: string }) {
  return (
    <Card className="flex min-h-64 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <Scale className="size-8" />
        <CardTitle>{title}</CardTitle>
        <CardDescription>页面不会用 mock 争议案件充当真实完成证据。</CardDescription>
      </div>
    </Card>
  );
}

function DisputeErrorPanel({ title, message }: { title: string; message: string }) {
  return (
    <Card className="flex min-h-64 items-center justify-center border-[var(--danger-ring)] bg-[var(--danger-soft)]">
      <div className="flex flex-col items-center gap-3 text-center text-[var(--danger-ink)]">
        <AlertTriangle className="size-8" />
        <CardTitle>{title}</CardTitle>
        <CardDescription className="text-[var(--danger-ink)]">{message}</CardDescription>
      </div>
    </Card>
  );
}

function PanelTitle({
  icon,
  title,
  description,
}: {
  icon: ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="flex items-start gap-3">
      <span className="flex size-10 shrink-0 items-center justify-center rounded-2xl bg-[var(--accent-soft)] text-[var(--accent-strong)]">
        {icon}
      </span>
      <div>
        <CardTitle>{title}</CardTitle>
        <CardDescription className="mt-1">{description}</CardDescription>
      </div>
    </div>
  );
}

function IdempotencyField({
  valueProps,
  action,
  onRegenerate,
}: {
  valueProps: UseFormRegisterReturn;
  action: "case" | "evidence" | "resolve";
  onRegenerate: () => void;
}) {
  return (
    <div className="grid gap-3 md:grid-cols-[1fr_auto]">
      <Field label="X-Idempotency-Key">
        <Input {...valueProps} />
      </Field>
      <Button type="button" variant="secondary" onClick={onRegenerate}>
        重新生成 {action}
      </Button>
    </div>
  );
}

function CheckRow({
  label,
  inputProps,
}: {
  label: string;
  inputProps: UseFormRegisterReturn;
}) {
  return (
    <label className="flex items-start gap-2 rounded-2xl bg-[var(--panel-muted)] p-3 text-sm text-[var(--ink-soft)]">
      <input type="checkbox" className="mt-1" {...inputProps} />
      <span>{label}</span>
    </label>
  );
}

function ActionFooter({
  pending,
  disabled,
  label,
}: {
  pending: boolean;
  disabled: boolean;
  label: string;
}) {
  return (
    <Button type="submit" disabled={pending || disabled}>
      {pending ? <LoaderCircle className="mr-2 size-4 animate-spin" /> : null}
      {label}
    </Button>
  );
}

function ResultBox({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="rounded-2xl border border-black/10 bg-white/80 p-4">
      <div className="mb-3 text-sm font-semibold text-[var(--ink-strong)]">{title}</div>
      <div className="grid gap-2 text-sm md:grid-cols-2">{children}</div>
    </div>
  );
}

function InlineError({ message }: { message: string }) {
  return (
    <div className="rounded-2xl border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-3 text-sm text-[var(--danger-ink)]">
      {message}
    </div>
  );
}

function InlineEmpty({ text }: { text: string }) {
  return (
    <div className="rounded-[24px] bg-[var(--panel-muted)] p-4 text-sm text-[var(--ink-soft)]">
      {text}
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="grid gap-2 text-sm font-semibold text-[var(--ink-strong)]">
      {label}
      {children}
    </label>
  );
}

function FormErrors({ errors }: { errors: FieldErrors }) {
  const messages = Object.values(errors)
    .map((error) => error?.message)
    .filter(Boolean);
  if (!messages.length) {
    return null;
  }
  return (
    <div className="rounded-2xl border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-3 text-sm text-[var(--danger-ink)]">
      {messages.join(" / ")}
    </div>
  );
}

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[24px] bg-white/75 p-4">
      <div className="text-xs font-medium uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div className="mt-2 break-all text-sm font-semibold text-[var(--ink-strong)]">
        {value || "未返回"}
      </div>
    </div>
  );
}

function TrustItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[22px] border border-black/10 bg-[var(--panel-muted)] p-4 text-sm">
      <div className="font-semibold text-[var(--ink-strong)]">{label}</div>
      <div className="mt-2 leading-6 text-[var(--ink-soft)]">{value}</div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-start justify-between gap-3 rounded-2xl bg-white/60 px-3 py-2">
      <span className="shrink-0 text-xs font-medium uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </span>
      <span className="break-all text-right text-sm font-semibold text-[var(--ink-strong)]">
        {value || "未返回"}
      </span>
    </div>
  );
}

const selectClassName =
  "h-11 rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]";
