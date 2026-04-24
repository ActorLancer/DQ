"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { formatPlatformErrorForDisplay } from "@datab/sdk-ts";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  AlertTriangle,
  CheckCircle2,
  FileCode2,
  Link2,
  LoaderCircle,
  LockKeyhole,
  UploadCloud,
} from "lucide-react";
import { motion } from "motion/react";
import {
  useEffect,
  type InputHTMLAttributes,
  type TextareaHTMLAttributes,
} from "react";
import { useForm, useWatch } from "react-hook-form";
import { z } from "zod";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  canReadOrder,
  formatTradeError,
  orderStatusLabel,
  unwrapLifecycle,
  unwrapOrderDetail,
  type SessionSubject as OrderSubject,
} from "@/lib/order-workflow";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { isSellerProductRole } from "@/lib/seller-products-view";
import type { PortalSessionPreview } from "@/lib/session";
import { formatList } from "@/lib/utils";

import { DeliveryWorkflowShell } from "./delivery-workflow-shell";
import { PortalRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();

type SessionMode = "guest" | "bearer" | "local";

const contractConfirmSchema = z.object({
  contract_template_id: z.string().uuid("contract_template_id 必须是 UUID"),
  contract_digest: z.string().trim().min(8, "contract_digest 至少 8 个字符"),
  data_contract_id: z.string().trim().optional(),
  data_contract_digest: z.string().trim().optional(),
  signer_role: z.enum(["buyer_signatory", "buyer_operator", "legal_reviewer"]),
  variables_json: z
    .string()
    .trim()
    .min(2, "variables_json 不能为空")
    .refine(isJsonObject, "variables_json 必须是 JSON object"),
});

const paymentIntentSchema = z
  .object({
    order_id: z.string().uuid("order_id 必须是 UUID"),
    intent_type: z.string().trim().min(2),
    provider_key: z.string().trim().min(2),
    payer_subject_type: z.string().trim().min(2),
    payer_subject_id: z.string().uuid("payer_subject_id 必须是 UUID"),
    payment_amount: z
      .string()
      .trim()
      .regex(/^\d+(\.\d{1,8})?$/, "金额格式必须是最多 8 位小数"),
    payment_method: z.string().trim().min(2),
    currency_code: z.string().trim().min(3).max(8),
    idempotency_key: z.string().trim().min(12, "Idempotency-Key 不能为空"),
    step_up_token: z.string().trim().optional(),
    step_up_challenge_id: z.string().trim().optional(),
  })
  .superRefine((value, context) => {
    if (!value.step_up_token && !value.step_up_challenge_id) {
      context.addIssue({
        code: "custom",
        path: ["step_up_token"],
        message: "创建支付意图需要 step-up：填写 step_up_token 或 step_up_challenge_id",
      });
    }
  });

const paymentLockSchema = z.object({
  payment_intent_id: z.string().uuid("payment_intent_id 必须是 UUID"),
  lock_reason: z.string().trim().optional(),
});

const querySurfaceSchema = z.object({
  query_surface_id: z.string().trim().optional(),
  asset_object_id: z.string().trim().optional(),
  environment_id: z.string().trim().optional(),
  surface_type: z.enum(["template_query_lite", "sandbox_query", "report_result"]),
  binding_mode: z.enum(["managed_surface", "seller_managed"]),
  execution_scope: z.enum(["curated_zone", "product_zone", "result_zone"]),
  status: z.enum(["draft", "active", "disabled"]),
  input_contract_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  output_boundary_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  query_policy_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  quota_policy_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  metadata_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
});

const queryTemplateSchema = z.object({
  query_surface_id: z.string().uuid("query_surface_id 必须是 UUID"),
  query_template_id: z.string().trim().optional(),
  template_name: z.string().trim().min(2),
  template_type: z.string().trim().min(2),
  template_body_ref: z.string().trim().optional(),
  version_no: z.number().int().min(1),
  status: z.enum(["draft", "active", "disabled"]),
  whitelist_fields: z.string().trim().optional(),
  parameter_schema_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  analysis_rule_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  result_schema_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  export_policy_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  risk_guard_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
});

const assetObjectSchema = z.object({
  version_id: z.string().uuid("asset version 必须是 UUID"),
  object_kind: z.enum([
    "raw_object",
    "preview_object",
    "delivery_object",
    "report_object",
    "result_object",
  ]),
  object_name: z.string().trim().min(2),
  object_uri: z.string().trim().min(6),
  share_protocol: z.string().trim().optional(),
  schema_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  output_schema_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  freshness_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  access_constraints_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  metadata_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  idempotency_key: z.string().trim().min(12, "Idempotency-Key 不能为空"),
});

const releasePolicySchema = z.object({
  release_mode: z.enum(["snapshot", "revision"]),
  is_revision_subscribable: z.boolean(),
  update_frequency: z.string().trim().optional(),
  release_notes_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
});

const rawBatchSchema = z.object({
  owner_org_id: z.string().uuid("owner_org_id 必须是 UUID"),
  ingest_source_type: z.string().trim().min(2),
  declared_object_family: z.string().trim().optional(),
  source_declared_rights_json: z
    .string()
    .trim()
    .min(2)
    .refine(isJsonObject, "必须是 JSON object"),
  ingest_policy_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
});

const rawManifestSchema = z.object({
  raw_ingest_batch_id: z.string().uuid("raw_ingest_batch_id 必须是 UUID"),
  object_name: z.string().trim().min(2),
  object_uri: z.string().trim().optional(),
  mime_type: z.string().trim().optional(),
  container_type: z.string().trim().optional(),
  byte_size: z.number().int().min(0).optional(),
  object_hash: z.string().trim().optional(),
  source_time_range_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
  manifest_json: z.string().trim().min(2).refine(isJsonObject, "必须是 JSON object"),
});

export function OrderContractConfirmShell({
  orderId,
  sessionMode,
  initialSubject,
}: {
  orderId: string;
  sessionMode: SessionMode;
  initialSubject: PortalSessionPreview | null;
}) {
  const authQuery = useQuery({
    queryKey: ["portal", "contract-confirm", orderId, "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest",
  });
  const subject = (authQuery.data?.data ?? initialSubject ?? null) as OrderSubject | null;
  const canRead = sessionMode !== "guest" && canReadOrder(subject);

  const orderQuery = useQuery({
    queryKey: ["portal", "contract-confirm", orderId, "order-detail"],
    queryFn: () => sdk.trade.getOrderDetail({ id: orderId }),
    enabled: canRead,
  });
  const lifecycleQuery = useQuery({
    queryKey: ["portal", "contract-confirm", orderId, "lifecycle"],
    queryFn: () => sdk.trade.getOrderLifecycleSnapshots({ id: orderId }),
    enabled: canRead,
  });

  const order = unwrapOrderDetail(orderQuery.data);
  const lifecycle = unwrapLifecycle(lifecycleQuery.data);
  const contractRelation = order?.relations.contract;

  const form = useForm<z.infer<typeof contractConfirmSchema>>({
    resolver: zodResolver(contractConfirmSchema),
    defaultValues: {
      contract_template_id: "",
      contract_digest: "",
      data_contract_id: "",
      data_contract_digest: "",
      signer_role: "buyer_operator",
      variables_json: "{\n  \"source\": \"WEB-009\"\n}",
    },
  });
  const signerRole = useWatch({ control: form.control, name: "signer_role" });

  useEffect(() => {
    if (!contractRelation) {
      return;
    }
    form.reset({
      contract_template_id: contractRelation.contract_template_id ?? "",
      contract_digest: contractRelation.contract_digest ?? "",
      data_contract_id: contractRelation.data_contract_id ?? "",
      data_contract_digest: contractRelation.data_contract_digest ?? "",
      signer_role: "buyer_operator",
      variables_json: JSON.stringify(
        contractRelation.variables_json ?? { source: "WEB-009" },
        null,
        2,
      ),
    });
  }, [contractRelation, form]);

  const mutation = useMutation({
    mutationFn: (values: z.infer<typeof contractConfirmSchema>) =>
      sdk.trade.confirmOrderContract(
        { id: orderId },
        {
          contract_template_id: values.contract_template_id,
          contract_digest: values.contract_digest,
          data_contract_id: emptyToUndefined(values.data_contract_id),
          data_contract_digest: emptyToUndefined(values.data_contract_digest),
          signer_role: values.signer_role,
          variables_json: parseJsonObject(values.variables_json) ?? {},
        },
      ),
  });

  return (
    <PortalRouteScaffold routeKey="order_contract_confirm" params={{ orderId }}>
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.24 }}
        className="space-y-4"
      >
        <SessionSummary subject={subject} />
        {!canRead ? (
          <PermissionPanel required="trade.order.read / trade.contract.confirm" />
        ) : orderQuery.isPending || lifecycleQuery.isPending ? (
          <LoadingPanel label="合同确认页加载中" />
        ) : orderQuery.isError || lifecycleQuery.isError ? (
          <ErrorPanel
            title="合同确认读取失败"
            message={formatTradeError(orderQuery.error ?? lifecycleQuery.error)}
          />
        ) : !order ? (
          <EmptyPanel
            title="未找到订单"
            description="请确认 order_id 是否存在且当前主体具备读取权限。"
          />
        ) : (
          <div className="grid gap-4 xl:grid-cols-[1.1fr_1fr]">
            <Card>
              <CardTitle>合同快照与签署上下文</CardTitle>
              <CardDescription>
                读取 `GET /api/v1/orders/{`{id}`}` 与生命周期快照，展示当前合同/支付状态。
              </CardDescription>
              <InfoGrid
                items={[
                  ["order_id", order.order_id],
                  ["order_status", orderStatusLabel(order.current_state)],
                  ["payment_status", order.payment_status],
                  ["contract_status", contractRelation?.contract_status ?? "未返回"],
                  ["contract_id", contractRelation?.contract_id ?? "未返回"],
                  ["template_id", contractRelation?.contract_template_id ?? "未返回"],
                  ["signed_at", contractRelation?.signed_at ?? "未签署"],
                  [
                    "lifecycle.contract",
                    lifecycle?.contract?.contract_status ?? "未返回",
                  ],
                ]}
              />
              <JsonCard
                title="variables_json"
                value={contractRelation?.variables_json ?? {}}
              />
            </Card>

            <Card>
              <CardTitle>合同确认动作</CardTitle>
              <CardDescription>
                调用 `POST /api/v1/orders/{`{id}`}/contract-confirm`，将签署动作写入交易链路。
              </CardDescription>
              <form
                className="mt-4 grid gap-3"
                onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
              >
                <InputField
                  label="contract_template_id"
                  error={form.formState.errors.contract_template_id?.message}
                  {...form.register("contract_template_id")}
                />
                <InputField
                  label="contract_digest"
                  error={form.formState.errors.contract_digest?.message}
                  {...form.register("contract_digest")}
                />
                <InputField
                  label="data_contract_id (optional)"
                  error={form.formState.errors.data_contract_id?.message}
                  {...form.register("data_contract_id")}
                />
                <InputField
                  label="data_contract_digest (optional)"
                  error={form.formState.errors.data_contract_digest?.message}
                  {...form.register("data_contract_digest")}
                />
                <SelectField
                  label="signer_role"
                  value={signerRole ?? "buyer_operator"}
                  onChange={(value) =>
                    form.setValue(
                      "signer_role",
                      value as z.infer<typeof contractConfirmSchema>["signer_role"],
                      { shouldDirty: true },
                    )
                  }
                  options={[
                    ["buyer_operator", "buyer_operator"],
                    ["buyer_signatory", "buyer_signatory"],
                    ["legal_reviewer", "legal_reviewer"],
                  ]}
                />
                <TextareaField
                  label="variables_json"
                  rows={5}
                  error={form.formState.errors.variables_json?.message}
                  {...form.register("variables_json")}
                />
                <div className="rounded-2xl border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-3 text-xs text-[var(--warning-ink)]">
                  合同确认是关键交易动作，需在页面显式告知审计留痕；异常时请结合
                  `request_id` 回查后端。
                </div>
                <Button disabled={mutation.isPending} type="submit">
                  {mutation.isPending ? (
                    <LoaderCircle className="size-4 animate-spin" />
                  ) : (
                    <CheckCircle2 className="size-4" />
                  )}
                  提交合同确认
                </Button>
              </form>
              {mutation.isError ? (
                <InlineError error={mutation.error} fallback="合同确认失败" />
              ) : null}
              {mutation.data ? (
                <JsonCard title="确认结果" value={mutation.data.data ?? mutation.data} />
              ) : null}
            </Card>
          </div>
        )}
      </motion.div>
    </PortalRouteScaffold>
  );
}

export function OrderPaymentLockShell({
  orderId,
  sessionMode,
  initialSubject,
}: {
  orderId: string;
  sessionMode: SessionMode;
  initialSubject: PortalSessionPreview | null;
}) {
  const authQuery = useQuery({
    queryKey: ["portal", "payment-lock", orderId, "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest",
  });
  const subject = (authQuery.data?.data ?? initialSubject ?? null) as OrderSubject | null;
  const canRead = sessionMode !== "guest" && canReadOrder(subject);
  const roles = subject?.roles ?? [];
  const canLock = roles.some((role) =>
    ["buyer_operator", "tenant_admin", "platform_admin", "platform_risk_settlement"].includes(role),
  );

  const orderQuery = useQuery({
    queryKey: ["portal", "payment-lock", orderId, "order-detail"],
    queryFn: () => sdk.trade.getOrderDetail({ id: orderId }),
    enabled: canRead,
  });
  const billingQuery = useQuery({
    queryKey: ["portal", "payment-lock", orderId, "billing-detail"],
    queryFn: () => sdk.billing.getBillingOrder({ order_id: orderId }),
    enabled: canRead,
  });

  const order = unwrapOrderDetail(orderQuery.data);
  const billing = billingQuery.data?.data ?? null;
  const inferredPayerId = subject?.tenant_id ?? subject?.org_id ?? order?.buyer_org_id ?? "";

  const intentForm = useForm<z.infer<typeof paymentIntentSchema>>({
    resolver: zodResolver(paymentIntentSchema),
    defaultValues: {
      order_id: orderId,
      intent_type: "order_lock",
      provider_key: "mock_payment_provider",
      payer_subject_type: "organization",
      payer_subject_id: inferredPayerId,
      payment_amount: order?.order_amount ?? "0",
      payment_method: "bank_transfer",
      currency_code: order?.currency_code ?? "CNY",
      idempotency_key: createWebIdempotencyKey("payment-intent"),
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const lockForm = useForm<z.infer<typeof paymentLockSchema>>({
    resolver: zodResolver(paymentLockSchema),
    defaultValues: {
      payment_intent_id: "",
      lock_reason: "WEB-009 payment lock action",
    },
  });

  useEffect(() => {
    intentForm.setValue("payer_subject_id", inferredPayerId, { shouldDirty: false });
    if (order) {
      intentForm.setValue("payment_amount", order.order_amount, { shouldDirty: false });
      intentForm.setValue("currency_code", order.currency_code, { shouldDirty: false });
    }
  }, [inferredPayerId, intentForm, order]);

  const createIntentMutation = useMutation({
    mutationFn: (values: z.infer<typeof paymentIntentSchema>) =>
      sdk.billing.createPaymentIntent(
        {
          order_id: values.order_id,
          intent_type: values.intent_type,
          provider_key: values.provider_key,
          payer_subject_type: values.payer_subject_type,
          payer_subject_id: values.payer_subject_id,
          payment_amount: values.payment_amount,
          payment_method: values.payment_method,
          currency_code: values.currency_code,
        },
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: emptyToUndefined(values.step_up_token),
          stepUpChallengeId: emptyToUndefined(values.step_up_challenge_id),
        },
      ),
    onSuccess: (response) => {
      const paymentIntentId = response.data?.payment_intent_id;
      if (paymentIntentId) {
        lockForm.setValue("payment_intent_id", paymentIntentId, { shouldDirty: true });
      }
    },
  });

  const watchedIntentId = useWatch({ control: lockForm.control, name: "payment_intent_id" });
  const paymentIntentQuery = useQuery({
    queryKey: ["portal", "payment-lock", orderId, "payment-intent", watchedIntentId],
    queryFn: () => sdk.billing.getPaymentIntent({ id: watchedIntentId }),
    enabled: canRead && Boolean(watchedIntentId),
  });

  const lockMutation = useMutation({
    mutationFn: (values: z.infer<typeof paymentLockSchema>) =>
      sdk.billing.lockOrderPayment(
        { id: orderId },
        {
          payment_intent_id: values.payment_intent_id,
          lock_reason: emptyToUndefined(values.lock_reason),
        },
      ),
  });

  return (
    <PortalRouteScaffold routeKey="order_payment_lock" params={{ orderId }}>
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.24 }}
        className="space-y-4"
      >
        <SessionSummary subject={subject} />
        {!canRead ? (
          <PermissionPanel required="trade.order.read / billing.deposit.lock" />
        ) : orderQuery.isPending || billingQuery.isPending ? (
          <LoadingPanel label="支付锁定页加载中" />
        ) : orderQuery.isError || billingQuery.isError ? (
          <ErrorPanel
            title="支付锁定读取失败"
            message={formatTradeError(orderQuery.error ?? billingQuery.error)}
          />
        ) : !order ? (
          <EmptyPanel title="未找到订单" description="请检查 order_id 与当前主体权限。" />
        ) : (
          <div className="grid gap-4 xl:grid-cols-[1.15fr_1fr]">
            <Card>
              <CardTitle>支付锁定状态</CardTitle>
              <CardDescription>
                读取订单与账单聚合，展示支付状态、金额与下一动作入口。
              </CardDescription>
              <InfoGrid
                items={[
                  ["order_id", order.order_id],
                  ["order_status", orderStatusLabel(order.current_state)],
                  ["payment_status", order.payment_status],
                  ["amount", `${order.order_amount} ${order.currency_code}`],
                  ["settlement_status", order.settlement_status],
                  [
                    "billing_event_count",
                    String(billing?.billing_events?.length ?? 0),
                  ],
                  ["refund_count", String(billing?.refunds?.length ?? 0)],
                  ["compensation_count", String(billing?.compensations?.length ?? 0)],
                ]}
              />
              <JsonCard
                title="payment_intent"
                value={paymentIntentQuery.data?.data ?? { notice: "输入 payment_intent_id 后可查询" }}
              />
            </Card>

            <div className="space-y-4">
              <Card>
                <CardTitle>创建支付意图</CardTitle>
                <CardDescription>
                  调用 `POST /api/v1/payments/intents`，要求 `Idempotency-Key + step-up`。
                </CardDescription>
                <form
                  className="mt-4 grid gap-3"
                  onSubmit={intentForm.handleSubmit((values) => createIntentMutation.mutate(values))}
                >
                  <InputField label="provider_key" error={intentForm.formState.errors.provider_key?.message} {...intentForm.register("provider_key")} />
                  <InputField label="payer_subject_id" error={intentForm.formState.errors.payer_subject_id?.message} {...intentForm.register("payer_subject_id")} />
                  <InputField label="payment_amount" error={intentForm.formState.errors.payment_amount?.message} {...intentForm.register("payment_amount")} />
                  <InputField label="payment_method" error={intentForm.formState.errors.payment_method?.message} {...intentForm.register("payment_method")} />
                  <InputField label="currency_code" error={intentForm.formState.errors.currency_code?.message} {...intentForm.register("currency_code")} />
                  <InputField label="Idempotency-Key" error={intentForm.formState.errors.idempotency_key?.message} {...intentForm.register("idempotency_key")} />
                  <InputField label="X-Step-Up-Token" error={intentForm.formState.errors.step_up_token?.message} {...intentForm.register("step_up_token")} />
                  <InputField label="X-Step-Up-Challenge-Id" error={intentForm.formState.errors.step_up_challenge_id?.message} {...intentForm.register("step_up_challenge_id")} />
                  <Button disabled={!canLock || createIntentMutation.isPending} type="submit">
                    {createIntentMutation.isPending ? (
                      <LoaderCircle className="size-4 animate-spin" />
                    ) : (
                      <LockKeyhole className="size-4" />
                    )}
                    创建 payment intent
                  </Button>
                </form>
                {!canLock ? <PermissionHint text="当前角色不可执行支付锁定动作。" /> : null}
                {createIntentMutation.isError ? (
                  <InlineError error={createIntentMutation.error} fallback="创建 payment intent 失败" />
                ) : null}
                {createIntentMutation.data ? (
                  <JsonCard title="createPaymentIntent 响应" value={createIntentMutation.data.data ?? createIntentMutation.data} />
                ) : null}
              </Card>

              <Card>
                <CardTitle>执行订单锁资</CardTitle>
                <CardDescription>
                  调用 `POST /api/v1/orders/{`{id}`}/lock`，绑定 payment_intent 与订单。
                </CardDescription>
                <form
                  className="mt-4 grid gap-3"
                  onSubmit={lockForm.handleSubmit((values) => lockMutation.mutate(values))}
                >
                  <InputField
                    label="payment_intent_id"
                    error={lockForm.formState.errors.payment_intent_id?.message}
                    {...lockForm.register("payment_intent_id")}
                  />
                  <InputField
                    label="lock_reason"
                    error={lockForm.formState.errors.lock_reason?.message}
                    {...lockForm.register("lock_reason")}
                  />
                  <Button disabled={!canLock || lockMutation.isPending} type="submit">
                    {lockMutation.isPending ? (
                      <LoaderCircle className="size-4 animate-spin" />
                    ) : (
                      <Link2 className="size-4" />
                    )}
                    提交 lock
                  </Button>
                </form>
                {lockMutation.isError ? (
                  <InlineError error={lockMutation.error} fallback="订单锁资失败" />
                ) : null}
                {lockMutation.data ? (
                  <JsonCard title="lock 响应" value={lockMutation.data.data ?? lockMutation.data} />
                ) : null}
              </Card>
            </div>
          </div>
        )}
      </motion.div>
    </PortalRouteScaffold>
  );
}

export function DeliveryQueryRunsShell({
  orderId,
  sessionMode,
  initialSubject,
}: {
  orderId: string;
  sessionMode: SessionMode;
  initialSubject: PortalSessionPreview | null;
}) {
  return (
    <DeliveryWorkflowShell
      kind="template-query"
      orderId={orderId}
      sessionMode={sessionMode}
      initialSubject={initialSubject}
    />
  );
}

export function SellerQuerySurfaceShell({
  productId,
  sessionMode,
  initialSubject,
}: {
  productId: string;
  sessionMode: SessionMode;
  initialSubject: PortalSessionPreview | null;
}) {
  const authQuery = useQuery({
    queryKey: ["portal", "seller-query-surface", productId, "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest",
  });
  const subject = (authQuery.data?.data ?? initialSubject ?? null) as OrderSubject | null;
  const canManage = sessionMode !== "guest" && isSellerProductRole(subject?.roles ?? []);
  const productQuery = useQuery({
    queryKey: ["portal", "seller-query-surface", productId, "product"],
    queryFn: () => sdk.catalog.getProductDetail({ id: productId }),
    enabled: canManage,
  });
  const product = productQuery.data?.data ?? null;

  const surfaceForm = useForm<z.infer<typeof querySurfaceSchema>>({
    resolver: zodResolver(querySurfaceSchema),
    defaultValues: {
      query_surface_id: "",
      asset_object_id: "",
      environment_id: "",
      surface_type: "template_query_lite",
      binding_mode: "managed_surface",
      execution_scope: "curated_zone",
      status: "draft",
      input_contract_json: "{\n  \"input\": \"required\"\n}",
      output_boundary_json: "{\n  \"max_rows\": 1000\n}",
      query_policy_json: "{\n  \"allow_sql\": false\n}",
      quota_policy_json: "{\n  \"daily\": 100\n}",
      metadata_json: "{\n  \"source\": \"WEB-007\"\n}",
    },
  });
  const templateForm = useForm<z.infer<typeof queryTemplateSchema>>({
    resolver: zodResolver(queryTemplateSchema),
    defaultValues: {
      query_surface_id: "",
      query_template_id: "",
      template_name: "template_query_default",
      template_type: "sql_template",
      template_body_ref: "",
      version_no: 1,
      status: "draft",
      whitelist_fields: "city, industry",
      parameter_schema_json: "{\n  \"type\": \"object\"\n}",
      analysis_rule_json: "{\n  \"review\": \"manual\"\n}",
      result_schema_json: "{\n  \"fields\": []\n}",
      export_policy_json: "{\n  \"allow_export\": false\n}",
      risk_guard_json: "{\n  \"threshold\": \"medium\"\n}",
    },
  });
  const surfaceType = useWatch({ control: surfaceForm.control, name: "surface_type" });
  const bindingMode = useWatch({ control: surfaceForm.control, name: "binding_mode" });
  const executionScope = useWatch({ control: surfaceForm.control, name: "execution_scope" });
  const surfaceStatus = useWatch({ control: surfaceForm.control, name: "status" });
  const templateStatus = useWatch({ control: templateForm.control, name: "status" });

  const createSurfaceMutation = useMutation({
    mutationFn: (values: z.infer<typeof querySurfaceSchema>) =>
      sdk.delivery.manageQuerySurface(
        { id: productId },
        {
          query_surface_id: emptyToUndefined(values.query_surface_id),
          asset_object_id: emptyToUndefined(values.asset_object_id),
          environment_id: emptyToUndefined(values.environment_id),
          surface_type: values.surface_type,
          binding_mode: values.binding_mode,
          execution_scope: values.execution_scope,
          status: values.status,
          input_contract_json: parseJsonObject(values.input_contract_json),
          output_boundary_json: parseJsonObject(values.output_boundary_json),
          query_policy_json: parseJsonObject(values.query_policy_json),
          quota_policy_json: parseJsonObject(values.quota_policy_json),
          metadata: parseJsonObject(values.metadata_json),
        },
      ),
    onSuccess: (response) => {
      const querySurfaceId = response.data?.query_surface_id;
      if (querySurfaceId) {
        templateForm.setValue("query_surface_id", querySurfaceId, { shouldDirty: true });
      }
    },
  });

  const createTemplateMutation = useMutation({
    mutationFn: (values: z.infer<typeof queryTemplateSchema>) =>
      sdk.delivery.manageQueryTemplate(
        { id: values.query_surface_id },
        {
          query_template_id: emptyToUndefined(values.query_template_id),
          template_name: values.template_name,
          template_type: values.template_type,
          template_body_ref: emptyToUndefined(values.template_body_ref),
          version_no: values.version_no,
          status: values.status,
          whitelist_fields: splitCsv(values.whitelist_fields),
          parameter_schema_json: parseJsonObject(values.parameter_schema_json),
          analysis_rule_json: parseJsonObject(values.analysis_rule_json),
          result_schema_json: parseJsonObject(values.result_schema_json),
          export_policy_json: parseJsonObject(values.export_policy_json),
          risk_guard_json: parseJsonObject(values.risk_guard_json),
        },
      ),
  });

  return (
    <PortalRouteScaffold routeKey="seller_query_surface" params={{ productId }}>
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.24 }}
        className="space-y-4"
      >
        <SessionSummary subject={subject} />
        {!canManage ? (
          <PermissionPanel required="delivery.query_surface.manage / delivery.query_template.manage" />
        ) : productQuery.isPending ? (
          <LoadingPanel label="查询面页面加载中" />
        ) : productQuery.isError ? (
          <ErrorPanel title="商品读取失败" message={formatCatalogError(productQuery.error)} />
        ) : !product ? (
          <EmptyPanel title="商品不存在" description="请确认 product_id 与租户权限。" />
        ) : (
          <div className="grid gap-4 xl:grid-cols-[1.1fr_1fr]">
            <Card>
              <CardTitle>查询面绑定上下文</CardTitle>
              <CardDescription>
                当前商品将通过 `POST /api/v1/products/{`{id}`}/query-surfaces` 管理查询面配置。
              </CardDescription>
              <InfoGrid
                items={[
                  ["product_id", product.product_id],
                  ["title", product.title],
                  ["status", product.status],
                  ["seller_org_id", product.seller_org_id],
                  ["delivery_type", product.delivery_type],
                  ["sku_count", String(product.skus.length)],
                ]}
              />
              <JsonCard title="product.skus" value={product.skus} />
            </Card>
            <Card>
              <CardTitle>创建/更新 QuerySurface</CardTitle>
              <form className="mt-4 grid gap-3" onSubmit={surfaceForm.handleSubmit((values) => createSurfaceMutation.mutate(values))}>
                <InputField label="query_surface_id (optional)" error={surfaceForm.formState.errors.query_surface_id?.message} {...surfaceForm.register("query_surface_id")} />
                <InputField label="asset_object_id (optional)" error={surfaceForm.formState.errors.asset_object_id?.message} {...surfaceForm.register("asset_object_id")} />
                <InputField label="environment_id (optional)" error={surfaceForm.formState.errors.environment_id?.message} {...surfaceForm.register("environment_id")} />
                <SelectField label="surface_type" value={surfaceType ?? "template_query_lite"} onChange={(value) => surfaceForm.setValue("surface_type", value as z.infer<typeof querySurfaceSchema>["surface_type"], { shouldDirty: true })} options={[["template_query_lite", "template_query_lite"], ["sandbox_query", "sandbox_query"], ["report_result", "report_result"]]} />
                <SelectField label="binding_mode" value={bindingMode ?? "managed_surface"} onChange={(value) => surfaceForm.setValue("binding_mode", value as z.infer<typeof querySurfaceSchema>["binding_mode"], { shouldDirty: true })} options={[["managed_surface", "managed_surface"], ["seller_managed", "seller_managed"]]} />
                <SelectField label="execution_scope" value={executionScope ?? "curated_zone"} onChange={(value) => surfaceForm.setValue("execution_scope", value as z.infer<typeof querySurfaceSchema>["execution_scope"], { shouldDirty: true })} options={[["curated_zone", "curated_zone"], ["product_zone", "product_zone"], ["result_zone", "result_zone"]]} />
                <SelectField label="status" value={surfaceStatus ?? "draft"} onChange={(value) => surfaceForm.setValue("status", value as z.infer<typeof querySurfaceSchema>["status"], { shouldDirty: true })} options={[["draft", "draft"], ["active", "active"], ["disabled", "disabled"]]} />
                <TextareaField label="input_contract_json" rows={3} error={surfaceForm.formState.errors.input_contract_json?.message} {...surfaceForm.register("input_contract_json")} />
                <TextareaField label="output_boundary_json" rows={3} error={surfaceForm.formState.errors.output_boundary_json?.message} {...surfaceForm.register("output_boundary_json")} />
                <TextareaField label="query_policy_json" rows={3} error={surfaceForm.formState.errors.query_policy_json?.message} {...surfaceForm.register("query_policy_json")} />
                <TextareaField label="quota_policy_json" rows={3} error={surfaceForm.formState.errors.quota_policy_json?.message} {...surfaceForm.register("quota_policy_json")} />
                <TextareaField label="metadata_json" rows={3} error={surfaceForm.formState.errors.metadata_json?.message} {...surfaceForm.register("metadata_json")} />
                <Button disabled={createSurfaceMutation.isPending} type="submit">
                  {createSurfaceMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <FileCode2 className="size-4" />}
                  提交 QuerySurface
                </Button>
              </form>
              {createSurfaceMutation.isError ? <InlineError error={createSurfaceMutation.error} fallback="提交 QuerySurface 失败" /> : null}
              {createSurfaceMutation.data ? <JsonCard title="QuerySurface 响应" value={createSurfaceMutation.data.data ?? createSurfaceMutation.data} /> : null}
            </Card>
            <Card className="xl:col-span-2">
              <CardTitle>创建/更新 QueryTemplate</CardTitle>
              <CardDescription>
                调用 `POST /api/v1/query-surfaces/{`{id}`}/templates`，绑定模板版本、白名单字段和风控规则。
              </CardDescription>
              <form className="mt-4 grid gap-3 lg:grid-cols-2" onSubmit={templateForm.handleSubmit((values) => createTemplateMutation.mutate(values))}>
                <InputField label="query_surface_id" error={templateForm.formState.errors.query_surface_id?.message} {...templateForm.register("query_surface_id")} />
                <InputField label="query_template_id (optional)" error={templateForm.formState.errors.query_template_id?.message} {...templateForm.register("query_template_id")} />
                <InputField label="template_name" error={templateForm.formState.errors.template_name?.message} {...templateForm.register("template_name")} />
                <InputField label="template_type" error={templateForm.formState.errors.template_type?.message} {...templateForm.register("template_type")} />
                <InputField label="template_body_ref (optional)" error={templateForm.formState.errors.template_body_ref?.message} {...templateForm.register("template_body_ref")} />
                <InputField label="version_no" type="number" error={templateForm.formState.errors.version_no?.message} {...templateForm.register("version_no", { valueAsNumber: true })} />
                <SelectField label="status" value={templateStatus ?? "draft"} onChange={(value) => templateForm.setValue("status", value as z.infer<typeof queryTemplateSchema>["status"], { shouldDirty: true })} options={[["draft", "draft"], ["active", "active"], ["disabled", "disabled"]]} />
                <InputField label="whitelist_fields(csv)" error={templateForm.formState.errors.whitelist_fields?.message} {...templateForm.register("whitelist_fields")} />
                <TextareaField className="lg:col-span-2" label="parameter_schema_json" rows={3} error={templateForm.formState.errors.parameter_schema_json?.message} {...templateForm.register("parameter_schema_json")} />
                <TextareaField className="lg:col-span-2" label="analysis_rule_json" rows={3} error={templateForm.formState.errors.analysis_rule_json?.message} {...templateForm.register("analysis_rule_json")} />
                <TextareaField className="lg:col-span-2" label="result_schema_json" rows={3} error={templateForm.formState.errors.result_schema_json?.message} {...templateForm.register("result_schema_json")} />
                <TextareaField className="lg:col-span-2" label="export_policy_json" rows={3} error={templateForm.formState.errors.export_policy_json?.message} {...templateForm.register("export_policy_json")} />
                <TextareaField className="lg:col-span-2" label="risk_guard_json" rows={3} error={templateForm.formState.errors.risk_guard_json?.message} {...templateForm.register("risk_guard_json")} />
                <div className="lg:col-span-2">
                  <Button disabled={createTemplateMutation.isPending} type="submit">
                    {createTemplateMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <CheckCircle2 className="size-4" />}
                    提交 QueryTemplate
                  </Button>
                </div>
              </form>
              {createTemplateMutation.isError ? <InlineError error={createTemplateMutation.error} fallback="提交 QueryTemplate 失败" /> : null}
              {createTemplateMutation.data ? <JsonCard title="QueryTemplate 响应" value={createTemplateMutation.data.data ?? createTemplateMutation.data} /> : null}
            </Card>
          </div>
        )}
      </motion.div>
    </PortalRouteScaffold>
  );
}

export function SellerShareModesShell({
  productId,
  sessionMode,
  initialSubject,
}: {
  productId: string;
  sessionMode: SessionMode;
  initialSubject: PortalSessionPreview | null;
}) {
  const authQuery = useQuery({
    queryKey: ["portal", "seller-share-modes", productId, "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest",
  });
  const subject = (authQuery.data?.data ?? initialSubject ?? null) as OrderSubject | null;
  const canManage = sessionMode !== "guest" && isSellerProductRole(subject?.roles ?? []);
  const productQuery = useQuery({
    queryKey: ["portal", "seller-share-modes", productId, "product"],
    queryFn: () => sdk.catalog.getProductDetail({ id: productId }),
    enabled: canManage,
  });
  const product = productQuery.data?.data ?? null;

  const objectForm = useForm<z.infer<typeof assetObjectSchema>>({
    resolver: zodResolver(assetObjectSchema),
    defaultValues: {
      version_id: "",
      object_kind: "delivery_object",
      object_name: "share-object-v1",
      object_uri: "storage://masked/share-object-v1",
      share_protocol: "presigned_read",
      schema_json: "{\n  \"fields\": []\n}",
      output_schema_json: "{\n  \"fields\": []\n}",
      freshness_json: "{\n  \"refresh\": \"daily\"\n}",
      access_constraints_json: "{\n  \"scope\": \"buyer_only\"\n}",
      metadata_json: "{\n  \"source\": \"WEB-007\"\n}",
      idempotency_key: createWebIdempotencyKey("asset-object"),
    },
  });
  const releaseForm = useForm<z.infer<typeof releasePolicySchema>>({
    resolver: zodResolver(releasePolicySchema),
    defaultValues: {
      release_mode: "revision",
      is_revision_subscribable: true,
      update_frequency: "P1D",
      release_notes_json: "{\n  \"channel\": \"seller_share_modes\"\n}",
    },
  });
  const objectKind = useWatch({ control: objectForm.control, name: "object_kind" });
  const releaseMode = useWatch({ control: releaseForm.control, name: "release_mode" });

  useEffect(() => {
    const versionId = product?.asset_version_id ?? "";
    if (versionId) {
      objectForm.setValue("version_id", versionId, { shouldDirty: false });
    }
  }, [objectForm, product?.asset_version_id]);

  const createObjectMutation = useMutation({
    mutationFn: (values: z.infer<typeof assetObjectSchema>) =>
      sdk.catalog.createAssetObject(
        { versionId: values.version_id },
        {
          object_kind: values.object_kind,
          object_name: values.object_name,
          object_uri: values.object_uri,
          share_protocol: emptyToUndefined(values.share_protocol),
          schema_json: parseJsonObject(values.schema_json),
          output_schema_json: parseJsonObject(values.output_schema_json),
          freshness_json: parseJsonObject(values.freshness_json),
          access_constraints: parseJsonObject(values.access_constraints_json),
          metadata: parseJsonObject(values.metadata_json),
        },
        { idempotencyKey: values.idempotency_key },
      ),
  });
  const patchReleaseMutation = useMutation({
    mutationFn: (values: z.infer<typeof releasePolicySchema>) =>
      sdk.catalog.patchAssetReleasePolicy(
        { assetId: product?.asset_id ?? "" },
        {
          release_mode: values.release_mode,
          is_revision_subscribable: values.is_revision_subscribable,
          update_frequency: emptyToUndefined(values.update_frequency),
          release_notes_json: parseJsonObject(values.release_notes_json),
        },
      ),
  });

  return (
    <PortalRouteScaffold routeKey="seller_share_modes" params={{ productId }}>
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.24 }}
        className="space-y-4"
      >
        <SessionSummary subject={subject} />
        {!canManage ? (
          <PermissionPanel required="catalog.asset_object.manage / catalog.asset_release.manage" />
        ) : productQuery.isPending ? (
          <LoadingPanel label="共享模式页面加载中" />
        ) : productQuery.isError ? (
          <ErrorPanel title="商品读取失败" message={formatCatalogError(productQuery.error)} />
        ) : !product ? (
          <EmptyPanel title="商品不存在" description="请确认 product_id 与租户权限。" />
        ) : (
          <div className="grid gap-4 xl:grid-cols-[1.1fr_1fr]">
            <Card>
              <CardTitle>共享模式上下文</CardTitle>
              <CardDescription>
                页面使用真实资产字段与 release-policy 契约，不暴露对象真实路径。
              </CardDescription>
              <InfoGrid
                items={[
                  ["product_id", product.product_id],
                  ["asset_id", product.asset_id],
                  ["asset_version_id", product.asset_version_id],
                  ["status", product.status],
                  ["sku_types", formatList(product.skus.map((sku) => sku.sku_type))],
                  ["seller_org_id", product.seller_org_id],
                ]}
              />
            </Card>
            <Card>
              <CardTitle>创建 Asset Object</CardTitle>
              <CardDescription>
                调用 `POST /api/v1/assets/{`{versionId}`}/objects`，写操作强制 `Idempotency-Key`。
              </CardDescription>
              <form className="mt-4 grid gap-3" onSubmit={objectForm.handleSubmit((values) => createObjectMutation.mutate(values))}>
                <InputField label="version_id" error={objectForm.formState.errors.version_id?.message} {...objectForm.register("version_id")} />
                <SelectField label="object_kind" value={objectKind ?? "delivery_object"} onChange={(value) => objectForm.setValue("object_kind", value as z.infer<typeof assetObjectSchema>["object_kind"], { shouldDirty: true })} options={[["delivery_object", "delivery_object"], ["result_object", "result_object"], ["report_object", "report_object"], ["preview_object", "preview_object"], ["raw_object", "raw_object"]]} />
                <InputField label="object_name" error={objectForm.formState.errors.object_name?.message} {...objectForm.register("object_name")} />
                <InputField label="object_uri(masked)" error={objectForm.formState.errors.object_uri?.message} {...objectForm.register("object_uri")} />
                <InputField label="share_protocol" error={objectForm.formState.errors.share_protocol?.message} {...objectForm.register("share_protocol")} />
                <InputField label="Idempotency-Key" error={objectForm.formState.errors.idempotency_key?.message} {...objectForm.register("idempotency_key")} />
                <TextareaField label="schema_json" rows={3} error={objectForm.formState.errors.schema_json?.message} {...objectForm.register("schema_json")} />
                <TextareaField label="output_schema_json" rows={3} error={objectForm.formState.errors.output_schema_json?.message} {...objectForm.register("output_schema_json")} />
                <TextareaField label="freshness_json" rows={3} error={objectForm.formState.errors.freshness_json?.message} {...objectForm.register("freshness_json")} />
                <TextareaField label="access_constraints_json" rows={3} error={objectForm.formState.errors.access_constraints_json?.message} {...objectForm.register("access_constraints_json")} />
                <TextareaField label="metadata_json" rows={3} error={objectForm.formState.errors.metadata_json?.message} {...objectForm.register("metadata_json")} />
                <Button disabled={createObjectMutation.isPending} type="submit">
                  {createObjectMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <Link2 className="size-4" />}
                  提交 Asset Object
                </Button>
              </form>
              {createObjectMutation.isError ? <InlineError error={createObjectMutation.error} fallback="创建 Asset Object 失败" /> : null}
              {createObjectMutation.data ? <JsonCard title="Asset Object 响应" value={createObjectMutation.data} /> : null}
            </Card>
            <Card className="xl:col-span-2">
              <CardTitle>更新 Release Policy</CardTitle>
              <CardDescription>
                调用 `PATCH /api/v1/assets/{`{assetId}`}/release-policy`，控制 snapshot/revision 策略。
              </CardDescription>
              <form className="mt-4 grid gap-3 md:grid-cols-2" onSubmit={releaseForm.handleSubmit((values) => patchReleaseMutation.mutate(values))}>
                <SelectField label="release_mode" value={releaseMode ?? "revision"} onChange={(value) => releaseForm.setValue("release_mode", value as z.infer<typeof releasePolicySchema>["release_mode"], { shouldDirty: true })} options={[["revision", "revision"], ["snapshot", "snapshot"]]} />
                <label className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm text-[var(--ink-soft)]">
                  <span className="font-medium text-[var(--ink-strong)]">is_revision_subscribable</span>
                  <input className="ml-3 align-middle" type="checkbox" {...releaseForm.register("is_revision_subscribable")} />
                </label>
                <InputField label="update_frequency(optional)" error={releaseForm.formState.errors.update_frequency?.message} {...releaseForm.register("update_frequency")} />
                <div className="md:col-span-2">
                  <TextareaField label="release_notes_json" rows={3} error={releaseForm.formState.errors.release_notes_json?.message} {...releaseForm.register("release_notes_json")} />
                </div>
                <div className="md:col-span-2">
                  <Button disabled={patchReleaseMutation.isPending || !product.asset_id} type="submit">
                    {patchReleaseMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <CheckCircle2 className="size-4" />}
                    提交 Release Policy
                  </Button>
                </div>
              </form>
              {patchReleaseMutation.isError ? <InlineError error={patchReleaseMutation.error} fallback="更新 release policy 失败" /> : null}
              {patchReleaseMutation.data ? <JsonCard title="Release Policy 响应" value={patchReleaseMutation.data} /> : null}
            </Card>
          </div>
        )}
      </motion.div>
    </PortalRouteScaffold>
  );
}

export function AssetRawIngestShell({
  assetId,
  sessionMode,
  initialSubject,
}: {
  assetId: string;
  sessionMode: SessionMode;
  initialSubject: PortalSessionPreview | null;
}) {
  const authQuery = useQuery({
    queryKey: ["portal", "asset-raw-ingest", assetId, "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
    enabled: sessionMode !== "guest",
  });
  const subject = (authQuery.data?.data ?? initialSubject ?? null) as OrderSubject | null;
  const canManage = sessionMode !== "guest" && isSellerProductRole(subject?.roles ?? []);

  const batchForm = useForm<z.infer<typeof rawBatchSchema>>({
    resolver: zodResolver(rawBatchSchema),
    defaultValues: {
      owner_org_id: subject?.tenant_id ?? subject?.org_id ?? "",
      ingest_source_type: "seller_upload",
      declared_object_family: "dataset",
      source_declared_rights_json: "{\n  \"license\": \"seller_owned\"\n}",
      ingest_policy_json: "{\n  \"scan\": \"required\"\n}",
    },
  });
  const manifestForm = useForm<z.infer<typeof rawManifestSchema>>({
    resolver: zodResolver(rawManifestSchema),
    defaultValues: {
      raw_ingest_batch_id: "",
      object_name: "raw-object-sample.csv",
      object_uri: "storage://masked/raw-object-sample.csv",
      mime_type: "text/csv",
      container_type: "file",
      byte_size: 1024,
      object_hash: "sha256:web007raw",
      source_time_range_json: "{\n  \"from\": \"2026-01-01\",\n  \"to\": \"2026-01-31\"\n}",
      manifest_json: "{\n  \"columns\": [\"city\", \"energy\"]\n}",
    },
  });

  useEffect(() => {
    if (!subject) {
      return;
    }
    const ownerOrgId = subject.tenant_id ?? subject.org_id ?? "";
    if (ownerOrgId) {
      batchForm.setValue("owner_org_id", ownerOrgId, { shouldDirty: false });
    }
  }, [batchForm, subject]);

  const batchMutation = useMutation({
    mutationFn: (values: z.infer<typeof rawBatchSchema>) =>
      sdk.catalog.createRawIngestBatch(
        { assetId },
        {
          owner_org_id: values.owner_org_id,
          ingest_source_type: values.ingest_source_type,
          declared_object_family: emptyToUndefined(values.declared_object_family),
          source_declared_rights_json: parseJsonObject(values.source_declared_rights_json),
          ingest_policy_json: parseJsonObject(values.ingest_policy_json),
        },
      ),
    onSuccess: (response) => {
      const batchId = response.data.raw_ingest_batch_id;
      if (batchId) {
        manifestForm.setValue("raw_ingest_batch_id", batchId, { shouldDirty: true });
      }
    },
  });

  const manifestMutation = useMutation({
    mutationFn: (values: z.infer<typeof rawManifestSchema>) =>
      sdk.catalog.createRawObjectManifest(
        { id: values.raw_ingest_batch_id },
        {
          raw_ingest_batch_id: values.raw_ingest_batch_id,
          object_name: values.object_name,
          object_uri: emptyToUndefined(values.object_uri),
          mime_type: emptyToUndefined(values.mime_type),
          container_type: emptyToUndefined(values.container_type),
          byte_size: values.byte_size,
          object_hash: emptyToUndefined(values.object_hash),
          source_time_range_json: parseJsonObject(values.source_time_range_json),
          manifest_json: parseJsonObject(values.manifest_json),
        },
      ),
  });

  return (
    <PortalRouteScaffold routeKey="asset_raw_ingest_center" params={{ assetId }}>
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.24 }}
        className="space-y-4"
      >
        <SessionSummary subject={subject} />
        {!canManage ? (
          <PermissionPanel required="catalog.raw_ingest.manage / catalog.extraction.manage / catalog.preview.manage" />
        ) : (
          <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
            <Card>
              <CardTitle>创建 Raw Ingest Batch</CardTitle>
              <CardDescription>
                调用 `POST /api/v1/assets/{`{assetId}`}/raw-ingest-batches`，记录接入批次与权利声明。
              </CardDescription>
              <form className="mt-4 grid gap-3" onSubmit={batchForm.handleSubmit((values) => batchMutation.mutate(values))}>
                <InputField label="owner_org_id" error={batchForm.formState.errors.owner_org_id?.message} {...batchForm.register("owner_org_id")} />
                <InputField label="ingest_source_type" error={batchForm.formState.errors.ingest_source_type?.message} {...batchForm.register("ingest_source_type")} />
                <InputField label="declared_object_family" error={batchForm.formState.errors.declared_object_family?.message} {...batchForm.register("declared_object_family")} />
                <TextareaField label="source_declared_rights_json" rows={3} error={batchForm.formState.errors.source_declared_rights_json?.message} {...batchForm.register("source_declared_rights_json")} />
                <TextareaField label="ingest_policy_json" rows={3} error={batchForm.formState.errors.ingest_policy_json?.message} {...batchForm.register("ingest_policy_json")} />
                <Button disabled={batchMutation.isPending} type="submit">
                  {batchMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <UploadCloud className="size-4" />}
                  提交 ingest batch
                </Button>
              </form>
              {batchMutation.isError ? <InlineError error={batchMutation.error} fallback="创建 ingest batch 失败" /> : null}
              {batchMutation.data ? <JsonCard title="RawIngestBatch 响应" value={batchMutation.data} /> : null}
            </Card>
            <Card>
              <CardTitle>创建 Raw Object Manifest</CardTitle>
              <CardDescription>
                调用 `POST /api/v1/raw-ingest-batches/{`{id}`}/manifests`，绑定对象摘要与元信息。
              </CardDescription>
              <form className="mt-4 grid gap-3" onSubmit={manifestForm.handleSubmit((values) => manifestMutation.mutate(values))}>
                <InputField label="raw_ingest_batch_id" error={manifestForm.formState.errors.raw_ingest_batch_id?.message} {...manifestForm.register("raw_ingest_batch_id")} />
                <InputField label="object_name" error={manifestForm.formState.errors.object_name?.message} {...manifestForm.register("object_name")} />
                <InputField label="object_uri(masked)" error={manifestForm.formState.errors.object_uri?.message} {...manifestForm.register("object_uri")} />
                <InputField label="mime_type" error={manifestForm.formState.errors.mime_type?.message} {...manifestForm.register("mime_type")} />
                <InputField label="container_type" error={manifestForm.formState.errors.container_type?.message} {...manifestForm.register("container_type")} />
                <InputField label="byte_size" type="number" error={manifestForm.formState.errors.byte_size?.message} {...manifestForm.register("byte_size", { valueAsNumber: true })} />
                <InputField label="object_hash" error={manifestForm.formState.errors.object_hash?.message} {...manifestForm.register("object_hash")} />
                <TextareaField label="source_time_range_json" rows={3} error={manifestForm.formState.errors.source_time_range_json?.message} {...manifestForm.register("source_time_range_json")} />
                <TextareaField label="manifest_json" rows={3} error={manifestForm.formState.errors.manifest_json?.message} {...manifestForm.register("manifest_json")} />
                <Button disabled={manifestMutation.isPending} type="submit">
                  {manifestMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <FileCode2 className="size-4" />}
                  提交 manifest
                </Button>
              </form>
              {manifestMutation.isError ? <InlineError error={manifestMutation.error} fallback="创建 manifest 失败" /> : null}
              {manifestMutation.data ? <JsonCard title="RawObjectManifest 响应" value={manifestMutation.data} /> : null}
            </Card>
          </div>
        )}
      </motion.div>
    </PortalRouteScaffold>
  );
}

function SessionSummary({
  subject,
}: {
  subject: OrderSubject | PortalSessionPreview | null;
}) {
  return (
    <Card>
      <CardTitle>当前会话主体</CardTitle>
      <CardDescription>敏感页面固定显示主体、角色、租户和作用域。</CardDescription>
      <InfoGrid
        items={[
          ["主体", subject?.display_name ?? subject?.login_id ?? subject?.user_id ?? "guest"],
          ["角色", subject?.roles?.length ? subject.roles.join(" / ") : "guest"],
          ["租户", subject?.tenant_id ?? subject?.org_id ?? "未绑定"],
          ["作用域", subject?.auth_context_level ?? "未返回"],
        ]}
      />
    </Card>
  );
}

function PermissionPanel({ required }: { required: string }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <CardTitle>权限不足</CardTitle>
      <CardDescription className="text-[var(--warning-ink)]">
        当前会话不具备该页面读取或执行权限：{required}
      </CardDescription>
    </Card>
  );
}

function LoadingPanel({ label }: { label: string }) {
  return (
    <Card className="flex min-h-48 items-center justify-center bg-[var(--panel-muted)]">
      <div className="flex flex-col items-center gap-3 text-[var(--ink-soft)]">
        <LoaderCircle className="size-7 animate-spin" />
        <span>{label}</span>
      </div>
    </Card>
  );
}

function EmptyPanel({ title, description }: { title: string; description: string }) {
  return (
    <Card className="border-dashed border-[var(--line-soft)]">
      <CardTitle>{title}</CardTitle>
      <CardDescription>{description}</CardDescription>
    </Card>
  );
}

function ErrorPanel({ title, message }: { title: string; message: string }) {
  return (
    <Card className="border-[var(--danger-ring)] bg-[var(--danger-soft)]">
      <CardTitle className="text-[var(--danger-ink)]">{title}</CardTitle>
      <CardDescription className="text-[var(--danger-ink)]">{message}</CardDescription>
    </Card>
  );
}

function InlineError({ error, fallback }: { error: unknown; fallback: string }) {
  const description = formatPlatformErrorForDisplay(error, {
    fallbackCode: "WEB_ROUTE_ACTION_FAILED",
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

function JsonCard({ title, value }: { title: string; value: unknown }) {
  return (
    <div className="mt-4 space-y-2">
      <Badge className="bg-black/10 text-[var(--ink-soft)]">{title}</Badge>
      <pre className="max-h-56 overflow-auto rounded-2xl bg-black/[0.06] p-3 text-xs text-[var(--ink-soft)]">
        {safeStringify(value)}
      </pre>
    </div>
  );
}

function InfoGrid({ items }: { items: Array<[string, string]> }) {
  return (
    <div className="mt-4 grid gap-2 md:grid-cols-2">
      {items.map(([label, value]) => (
        <div
          className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm text-[var(--ink-soft)]"
          key={`${label}-${value}`}
        >
          <div className="text-xs uppercase tracking-[0.12em] text-[var(--ink-subtle)]">
            {label}
          </div>
          <div className="mt-1 font-medium text-[var(--ink-strong)]">{value}</div>
        </div>
      ))}
    </div>
  );
}

function PermissionHint({ text }: { text: string }) {
  return (
    <div className="mt-3 rounded-2xl border border-[var(--warning-ring)] bg-[var(--warning-soft)] px-3 py-2 text-xs text-[var(--warning-ink)]">
      {text}
    </div>
  );
}

type InputFieldProps = InputHTMLAttributes<HTMLInputElement> & {
  label: string;
  error?: string;
};

function InputField({ label, error, className, ...props }: InputFieldProps) {
  return (
    <label className={className}>
      <span className="mb-1 block text-xs text-[var(--ink-soft)]">{label}</span>
      <Input {...props} />
      {error ? <span className="mt-1 block text-xs text-[var(--danger-ink)]">{error}</span> : null}
    </label>
  );
}

type TextareaFieldProps = TextareaHTMLAttributes<HTMLTextAreaElement> & {
  label: string;
  error?: string;
};

function TextareaField({ label, error, className, ...props }: TextareaFieldProps) {
  return (
    <label className={className}>
      <span className="mb-1 block text-xs text-[var(--ink-soft)]">{label}</span>
      <Textarea {...props} />
      {error ? <span className="mt-1 block text-xs text-[var(--danger-ink)]">{error}</span> : null}
    </label>
  );
}

function SelectField({
  label,
  options,
  value,
  onChange,
}: {
  label: string;
  options: Array<[string, string]>;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label>
      <span className="mb-1 block text-xs text-[var(--ink-soft)]">{label}</span>
      <select
        className="flex h-10 w-full rounded-md border border-[var(--line-soft)] bg-transparent px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-ring)]"
        onChange={(event) => onChange(event.currentTarget.value)}
        value={value}
      >
        {options.map(([optionValue, optionLabel]) => (
          <option key={optionValue} value={optionValue}>
            {optionLabel}
          </option>
        ))}
      </select>
    </label>
  );
}

function parseJsonObject(value: string | undefined) {
  if (!value?.trim()) {
    return undefined;
  }
  const parsed = JSON.parse(value) as unknown;
  return typeof parsed === "object" && parsed && !Array.isArray(parsed)
    ? (parsed as Record<string, unknown>)
    : undefined;
}

function isJsonObject(value: string) {
  try {
    return Boolean(parseJsonObject(value));
  } catch {
    return false;
  }
}

function emptyToUndefined(value: string | undefined) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function splitCsv(value: string | undefined): string[] | undefined {
  const normalized = value
    ?.split(",")
    .map((item) => item.trim())
    .filter(Boolean);
  return normalized?.length ? normalized : undefined;
}

function createWebIdempotencyKey(scope: string) {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `web-route:${scope}:${crypto.randomUUID()}`;
  }
  return `web-route:${scope}:${Date.now()}`;
}

function safeStringify(value: unknown) {
  try {
    return JSON.stringify(value ?? null, null, 2);
  } catch {
    return String(value);
  }
}

function formatCatalogError(error: unknown) {
  return formatPlatformErrorForDisplay(error, {
    fallbackCode: "CATALOG_ROUTE_FAILED",
    fallbackDescription: "请结合 request_id 与后端审计记录排查。",
  });
}
