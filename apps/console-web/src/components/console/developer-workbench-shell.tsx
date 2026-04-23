"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import type {
  ApplicationListResponse,
  DeveloperTraceResponse,
  HealthDepsResponse,
  MockPaymentSimulationResponse,
} from "@datab/sdk-ts";
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  CreditCard,
  KeyRound,
  LoaderCircle,
  LockKeyhole,
  Network,
  RotateCcw,
  Search,
  ShieldCheck,
  TerminalSquare,
} from "lucide-react";
import { motion } from "motion/react";
import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type ComponentProps,
  type ReactNode,
} from "react";
import { useForm, useWatch } from "react-hook-form";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  applicationCreateSchema,
  applicationPatchSchema,
  applicationSecretSchema,
  applicationStatuses,
  applicationTypes,
  buildCreateApplicationPayload,
  buildDeveloperTraceQuery,
  buildMockPaymentPayload,
  buildPatchApplicationPayload,
  buildRotateSecretPayload,
  canManageApplications,
  canReadDeveloper,
  canReadDeveloperTrace,
  canSimulateMockPayment,
  createDeveloperIdempotencyKey,
  developerTraceSchema,
  formatDeveloperError,
  mockPaymentSchema,
  mockPaymentScenarios,
  statusTone,
  subjectDisplayName,
  subjectRoles,
  traceLookupModes,
  type ApplicationCreateFormValues,
  type ApplicationPatchFormValues,
  type ApplicationSecretFormValues,
  type DeveloperTraceFormValues,
  type MockPaymentFormValues,
  type SessionSubject,
} from "@/lib/developer-workbench";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { cn } from "@/lib/utils";

import { ConsoleRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();
type ApplicationRow = ApplicationListResponse["data"][number];
type DeveloperTraceData = DeveloperTraceResponse["data"];
type MockPaymentData = MockPaymentSimulationResponse["data"];
type HealthDepsData = HealthDepsResponse["data"];

export function DeveloperHomeShell() {
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canRead = canReadDeveloper(subject);
  const scopeOrgId = subject?.tenant_id || subject?.org_id || undefined;
  const healthQuery = useQuery({
    queryKey: ["console", "developer", "health-deps"],
    queryFn: () => sdk.ops.healthDeps(),
    enabled: canRead,
  });
  const appsQuery = useQuery({
    queryKey: ["console", "developer", "apps", scopeOrgId],
    queryFn: () => sdk.iam.listApplications(scopeOrgId ? { org_id: scopeOrgId } : {}),
    enabled: canRead,
  });

  return (
    <ConsoleRouteScaffold routeKey="developer_home">
      <DeveloperHero
        title="开发者控制面"
        description="面向测试应用、API Key、调用日志、trace 联查和 Mock 支付的正式入口；所有请求经 console-web 受控代理访问 platform-core。"
        icon={<TerminalSquare className="size-5" />}
        badges={["developer.home.read", "Keycloak / IAM", "platform-core only"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <BoundaryCard />
      </div>
      {!canRead ? <PermissionNotice required="developer.home.read" /> : null}
      <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
        <NetworkCard health={healthQuery.data?.data} loading={healthQuery.isFetching} />
        <DeveloperQuickNav appCount={appsQuery.data?.data.length ?? 0} />
      </div>
      {appsQuery.isError ? <ErrorState error={appsQuery.error} /> : null}
    </ConsoleRouteScaffold>
  );
}

export function DeveloperAppsShell() {
  const queryClient = useQueryClient();
  const idSeed = useId().replace(/:/g, "");
  const [selectedApp, setSelectedApp] = useState<ApplicationRow | null>(null);
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canRead = canReadDeveloper(subject);
  const canManage = canManageApplications(subject);
  const scopeOrgId = subject?.tenant_id || subject?.org_id || undefined;
  const appsQuery = useQuery({
    queryKey: ["console", "developer", "apps", scopeOrgId],
    queryFn: () => sdk.iam.listApplications(scopeOrgId ? { org_id: scopeOrgId } : {}),
    enabled: canRead,
  });

  const createForm = useForm<ApplicationCreateFormValues>({
    resolver: zodResolver(applicationCreateSchema),
    defaultValues: {
      org_id: "",
      app_name: "WEB-016 Developer App",
      app_type: "api_client",
      client_id: `web016-${idSeed}`,
      client_secret_hash: "",
      idempotency_key: `web-016:app-create:${idSeed}`,
    },
  });
  const patchForm = useForm<ApplicationPatchFormValues>({
    resolver: zodResolver(applicationPatchSchema),
    defaultValues: {
      app_id: "",
      app_name: "",
      status: "active",
      idempotency_key: `web-016:app-patch:${idSeed}`,
    },
  });
  const secretForm = useForm<ApplicationSecretFormValues>({
    resolver: zodResolver(applicationSecretSchema),
    defaultValues: {
      app_id: "",
      client_secret_hash: "",
      idempotency_key: `web-016:app-secret:${idSeed}`,
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });

  const createMutation = useMutation({
    mutationFn: (values: ApplicationCreateFormValues) =>
      sdk.iam.createApplication(buildCreateApplicationPayload(values), {
        idempotencyKey: values.idempotency_key,
      }),
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: ["console", "developer", "apps"] });
      setSelectedApp(response.data);
      patchForm.setValue("app_id", response.data.app_id);
      secretForm.setValue("app_id", response.data.app_id);
      createForm.setValue("idempotency_key", createDeveloperIdempotencyKey("app-create"));
    },
  });
  const patchMutation = useMutation({
    mutationFn: (values: ApplicationPatchFormValues) =>
      sdk.iam.patchApplication(
        { id: values.app_id },
        buildPatchApplicationPayload(values),
        { idempotencyKey: values.idempotency_key },
      ),
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: ["console", "developer", "apps"] });
      setSelectedApp(response.data);
      patchForm.setValue("idempotency_key", createDeveloperIdempotencyKey("app-patch"));
    },
  });
  const rotateMutation = useMutation({
    mutationFn: (values: ApplicationSecretFormValues) =>
      sdk.iam.rotateApplicationSecret(
        { id: values.app_id },
        buildRotateSecretPayload(values),
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: values.step_up_token,
          stepUpChallengeId: values.step_up_challenge_id,
        },
      ),
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: ["console", "developer", "apps"] });
      setSelectedApp(response.data);
      secretForm.setValue("idempotency_key", createDeveloperIdempotencyKey("app-secret"));
    },
  });
  const revokeMutation = useMutation({
    mutationFn: (values: ApplicationSecretFormValues) =>
      sdk.iam.revokeApplicationSecret(
        { id: values.app_id },
        {
          idempotencyKey: values.idempotency_key,
          stepUpToken: values.step_up_token,
          stepUpChallengeId: values.step_up_challenge_id,
        },
      ),
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: ["console", "developer", "apps"] });
      setSelectedApp(response.data);
      secretForm.setValue("idempotency_key", createDeveloperIdempotencyKey("app-secret"));
    },
  });

  useEffect(() => {
    const orgId = subject?.tenant_id ?? subject?.org_id;
    if (orgId && !createForm.getValues("org_id")) {
      createForm.setValue("org_id", orgId);
    }
  }, [createForm, subject?.org_id, subject?.tenant_id]);

  function selectApplication(app: ApplicationRow) {
    setSelectedApp(app);
    patchForm.reset({
      app_id: app.app_id,
      app_name: app.app_name,
      status: applicationStatuses.includes(app.status as ApplicationPatchFormValues["status"])
        ? (app.status as ApplicationPatchFormValues["status"])
        : "active",
      idempotency_key: createDeveloperIdempotencyKey("app-patch"),
    });
    secretForm.reset({
      app_id: app.app_id,
      client_secret_hash: "",
      idempotency_key: createDeveloperIdempotencyKey("app-secret"),
      step_up_token: "",
      step_up_challenge_id: "",
    });
  }

  return (
    <ConsoleRouteScaffold routeKey="developer_apps">
      <DeveloperHero
        title="测试应用与 API Key"
        description="绑定 `/api/v1/apps` 与 credentials 接口；创建、更新、轮换、撤销都携带 Idempotency-Key，并只展示 secret 状态。"
        icon={<KeyRound className="size-5" />}
        badges={["developer.app.read", "developer.app.create", "developer.app.update"]}
      />
      <div className="grid gap-4 xl:grid-cols-[0.9fr_1.1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <AuditNotice />
      </div>
      {!canRead ? <PermissionNotice required="developer.app.read" /> : null}
      <ApplicationTable
        applications={appsQuery.data?.data ?? []}
        loading={appsQuery.isPending && canRead}
        error={appsQuery.error}
        selectedAppId={selectedApp?.app_id}
        onSelect={selectApplication}
      />
      <div className="grid gap-4 xl:grid-cols-3">
        <Card>
          <CardTitle>创建测试应用</CardTitle>
          <CardDescription className="mt-2">
            `client_secret_hash` 只接受预哈希材料；页面不会生成或显示明文 API Key。
          </CardDescription>
          <form
            className="mt-5 space-y-4"
            onSubmit={createForm.handleSubmit((values) => createMutation.mutate(values))}
          >
            <InputField label="org_id" error={createForm.formState.errors.org_id?.message} {...createForm.register("org_id")} />
            <InputField label="app_name" error={createForm.formState.errors.app_name?.message} {...createForm.register("app_name")} />
            <SelectField
              label="app_type"
              value={useWatch({ control: createForm.control, name: "app_type" })}
              options={applicationTypes.map((value) => [value, value])}
              onChange={(value) => createForm.setValue("app_type", value as ApplicationCreateFormValues["app_type"], { shouldDirty: true })}
            />
            <InputField label="client_id" error={createForm.formState.errors.client_id?.message} {...createForm.register("client_id")} />
            <InputField label="client_secret_hash" error={createForm.formState.errors.client_secret_hash?.message} {...createForm.register("client_secret_hash")} />
            <InputField label="Idempotency-Key" error={createForm.formState.errors.idempotency_key?.message} {...createForm.register("idempotency_key")} />
            <Button disabled={!canManage || createMutation.isPending} type="submit">
              {createMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <ShieldCheck className="size-4" />}
              创建应用
            </Button>
          </form>
          {createMutation.isError ? <InlineError error={createMutation.error} /> : null}
        </Card>
        <Card>
          <CardTitle>更新应用状态</CardTitle>
          <CardDescription className="mt-2">
            状态值沿用 IAM 返回语义；页面不发明新生命周期。
          </CardDescription>
          <form
            className="mt-5 space-y-4"
            onSubmit={patchForm.handleSubmit((values) => patchMutation.mutate(values))}
          >
            <InputField label="app_id" error={patchForm.formState.errors.app_id?.message} {...patchForm.register("app_id")} />
            <InputField label="app_name" error={patchForm.formState.errors.app_name?.message} {...patchForm.register("app_name")} />
            <SelectField
              label="status"
              value={useWatch({ control: patchForm.control, name: "status" })}
              options={applicationStatuses.map((value) => [value, value])}
              onChange={(value) => patchForm.setValue("status", value as ApplicationPatchFormValues["status"], { shouldDirty: true })}
            />
            <InputField label="Idempotency-Key" error={patchForm.formState.errors.idempotency_key?.message} {...patchForm.register("idempotency_key")} />
            <Button disabled={!canManage || patchMutation.isPending} type="submit">
              {patchMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <RotateCcw className="size-4" />}
              更新
            </Button>
          </form>
          {patchMutation.isError ? <InlineError error={patchMutation.error} /> : null}
        </Card>
        <Card>
          <CardTitle>API Key 轮换 / 撤销</CardTitle>
          <CardDescription className="mt-2">
            高敏动作展示人工确认和 step-up 透传入口；响应仅显示 `client_secret_status`。
          </CardDescription>
          <form className="mt-5 space-y-4">
            <InputField label="app_id" error={secretForm.formState.errors.app_id?.message} {...secretForm.register("app_id")} />
            <InputField label="client_secret_hash" error={secretForm.formState.errors.client_secret_hash?.message} {...secretForm.register("client_secret_hash")} />
            <InputField label="Idempotency-Key" error={secretForm.formState.errors.idempotency_key?.message} {...secretForm.register("idempotency_key")} />
            <InputField label="X-Step-Up-Token" error={secretForm.formState.errors.step_up_token?.message} {...secretForm.register("step_up_token")} />
            <InputField label="X-Step-Up-Challenge-Id" error={secretForm.formState.errors.step_up_challenge_id?.message} {...secretForm.register("step_up_challenge_id")} />
            <div className="flex flex-wrap gap-2">
              <Button
                disabled={!canManage || rotateMutation.isPending}
                type="button"
                onClick={secretForm.handleSubmit((values) => rotateMutation.mutate(values))}
              >
                {rotateMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <KeyRound className="size-4" />}
                轮换
              </Button>
              <Button
                disabled={!canManage || revokeMutation.isPending}
                type="button"
                variant="warning"
                onClick={secretForm.handleSubmit((values) => revokeMutation.mutate(values))}
              >
                {revokeMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <LockKeyhole className="size-4" />}
                撤销
              </Button>
            </div>
          </form>
          {rotateMutation.isError ? <InlineError error={rotateMutation.error} /> : null}
          {revokeMutation.isError ? <InlineError error={revokeMutation.error} /> : null}
        </Card>
      </div>
      {selectedApp ? <ApplicationDetail app={selectedApp} /> : null}
    </ConsoleRouteScaffold>
  );
}

export function DeveloperTraceShell() {
  const [lookup, setLookup] = useState<DeveloperTraceFormValues | null>(null);
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canRead = canReadDeveloperTrace(subject);
  const traceForm = useForm<DeveloperTraceFormValues>({
    resolver: zodResolver(developerTraceSchema),
    defaultValues: {
      lookup_mode: "order_id",
      lookup_value: "",
    },
  });
  const traceQuery = useQuery({
    queryKey: ["console", "developer", "trace", lookup],
    queryFn: () => sdk.ops.getDeveloperTrace(buildDeveloperTraceQuery(lookup!)),
    enabled: Boolean(canRead && lookup),
  });

  return (
    <ConsoleRouteScaffold routeKey="developer_trace">
      <DeveloperHero
        title="状态与调用日志联查"
        description="通过 `/api/v1/developer/trace` 按 order_id / event_id / tx_hash 汇总业务状态、链状态、投影状态、审计和 system log。"
        icon={<Search className="size-5" />}
        badges={["developer.trace.read", "request_id", "tx_hash", "projection"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <TraceModeNotice />
      </div>
      {!canRead ? <PermissionNotice required="developer.trace.read" /> : null}
      <Card>
        <form
          className="grid gap-4 lg:grid-cols-[180px_1fr_auto]"
          onSubmit={traceForm.handleSubmit((values) => setLookup(values))}
        >
          <SelectField
            label="lookup_mode"
            value={useWatch({ control: traceForm.control, name: "lookup_mode" })}
            options={traceLookupModes.map((value) => [value, value])}
            onChange={(value) => traceForm.setValue("lookup_mode", value as DeveloperTraceFormValues["lookup_mode"], { shouldDirty: true })}
          />
          <InputField
            label="lookup_value"
            placeholder="order_id / event_id / tx_hash"
            error={traceForm.formState.errors.lookup_value?.message}
            {...traceForm.register("lookup_value")}
          />
          <Button disabled={!canRead || traceQuery.isFetching} type="submit">
            {traceQuery.isFetching ? <LoaderCircle className="size-4 animate-spin" /> : <Search className="size-4" />}
            联查
          </Button>
        </form>
      </Card>
      {traceQuery.isPending && lookup ? <LoadingState label="正在联查 trace / system log / audit" /> : null}
      {traceQuery.isError ? <ErrorState error={traceQuery.error} /> : null}
      {traceQuery.isSuccess ? (
        <DeveloperTraceResult data={traceQuery.data.data} />
      ) : lookup ? null : (
        <EmptyState title="等待联查键" description="输入正式 order_id、event_id 或 tx_hash 后展示 request_id、链状态、投影状态与调用日志。" />
      )}
    </ConsoleRouteScaffold>
  );
}

export function DeveloperAssetsShell() {
  const idSeed = useId().replace(/:/g, "");
  const [result, setResult] = useState<MockPaymentData | null>(null);
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canSimulate = canSimulateMockPayment(subject);
  const mockForm = useForm<MockPaymentFormValues>({
    resolver: zodResolver(mockPaymentSchema),
    defaultValues: {
      payment_intent_id: "",
      scenario: "success",
      delay_seconds: 0,
      duplicate_webhook: false,
      partial_refund_amount: "",
      idempotency_key: `web-016:mock-payment:${idSeed}`,
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });
  const scenario = useWatch({ control: mockForm.control, name: "scenario" });
  const mockMutation = useMutation({
    mutationFn: async (values: MockPaymentFormValues) => {
      const path = { id: values.payment_intent_id };
      const body = buildMockPaymentPayload(values);
      const options = {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token,
        stepUpChallengeId: values.step_up_challenge_id,
      };
      if (values.scenario === "success") {
        return sdk.billing.simulateMockPaymentSuccess(path, body, options);
      }
      if (values.scenario === "fail") {
        return sdk.billing.simulateMockPaymentFail(path, body, options);
      }
      return sdk.billing.simulateMockPaymentTimeout(path, body, options);
    },
    onSuccess: (response) => {
      setResult(response.data);
      mockForm.setValue("idempotency_key", createDeveloperIdempotencyKey("mock-payment"));
    },
  });

  return (
    <ConsoleRouteScaffold routeKey="developer_assets">
      <DeveloperHero
        title="Mock 支付与测试资产"
        description="非生产 Mock 支付操作入口，绑定 simulate-success / fail / timeout 正式接口，并展示审计、幂等和权限提示。"
        icon={<CreditCard className="size-5" />}
        badges={["developer.mock_payment.simulate", "non-production", "audit"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <TestAssetsCard />
      </div>
      {!canSimulate ? <PermissionNotice required="developer.mock_payment.simulate" /> : null}
      <Card>
        <CardTitle>Mock 支付操作</CardTitle>
        <CardDescription className="mt-2">
          只接受已有 `payment_intent_id`，不会在前端直连支付 provider 或数据库；审计记录 append-only 保留。
        </CardDescription>
        <form
          className="mt-5 grid gap-4 xl:grid-cols-2"
          onSubmit={mockForm.handleSubmit((values) => mockMutation.mutate(values))}
        >
          <InputField label="payment_intent_id" error={mockForm.formState.errors.payment_intent_id?.message} {...mockForm.register("payment_intent_id")} />
          <SelectField
            label="scenario"
            value={scenario}
            options={mockPaymentScenarios.map((value) => [value, value])}
            onChange={(value) => mockForm.setValue("scenario", value as MockPaymentFormValues["scenario"], { shouldDirty: true })}
          />
          <InputField label="delay_seconds" type="number" error={mockForm.formState.errors.delay_seconds?.message} {...mockForm.register("delay_seconds", { valueAsNumber: true })} />
          <InputField label="partial_refund_amount" error={mockForm.formState.errors.partial_refund_amount?.message} {...mockForm.register("partial_refund_amount")} />
          <InputField label="Idempotency-Key" error={mockForm.formState.errors.idempotency_key?.message} {...mockForm.register("idempotency_key")} />
          <InputField label="X-Step-Up-Token" error={mockForm.formState.errors.step_up_token?.message} {...mockForm.register("step_up_token")} />
          <label className="flex items-center gap-3 rounded-2xl bg-black/[0.04] px-4 py-3 text-sm text-[var(--ink-soft)]">
            <input className="size-4" type="checkbox" {...mockForm.register("duplicate_webhook")} />
            duplicate_webhook 回放
          </label>
          <InputField label="X-Step-Up-Challenge-Id" error={mockForm.formState.errors.step_up_challenge_id?.message} {...mockForm.register("step_up_challenge_id")} />
          <div className="xl:col-span-2">
            <Button disabled={!canSimulate || mockMutation.isPending} type="submit">
              {mockMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <CreditCard className="size-4" />}
              触发 Mock 支付 {scenario}
            </Button>
          </div>
        </form>
        {mockMutation.isError ? <InlineError error={mockMutation.error} /> : null}
      </Card>
      {result ? <MockPaymentResult data={result} /> : <EmptyState title="等待 Mock 支付操作" description="提交后展示 provider_event_id、webhook_processed_status、payment_transaction_id 与支付状态投影。" />}
    </ConsoleRouteScaffold>
  );
}

function useAuthMe() {
  return useQuery({
    queryKey: ["console", "developer", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
  });
}

function DeveloperHero({
  title,
  description,
  icon,
  badges,
}: {
  title: string;
  description: string;
  icon: ReactNode;
  badges: string[];
}) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.25 }}
      className="rounded-[32px] border border-white/70 bg-[linear-gradient(135deg,rgba(242,250,246,0.98),rgba(226,241,246,0.92),rgba(255,255,255,0.9))] p-6 shadow-[0_24px_80px_rgba(18,41,52,0.10)]"
    >
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="max-w-3xl">
          <div className="mb-4 flex size-11 items-center justify-center rounded-2xl bg-[var(--accent-strong)] text-white">
            {icon}
          </div>
          <h2 className="text-2xl font-semibold tracking-tight text-[var(--ink-strong)]">{title}</h2>
          <p className="mt-3 max-w-2xl text-sm leading-6 text-[var(--ink-soft)]">{description}</p>
        </div>
        <div className="flex max-w-xl flex-wrap gap-2">
          {badges.map((badge) => (
            <Badge key={badge}>{badge}</Badge>
          ))}
        </div>
      </div>
    </motion.section>
  );
}

function SubjectCard({ subject, loading }: { subject?: SessionSubject; loading: boolean }) {
  return (
    <Card>
      <div className="flex items-start justify-between gap-4">
        <div>
          <CardTitle>当前主体 / 角色 / 租户 / 作用域</CardTitle>
          <CardDescription className="mt-2">
            来自 `GET /api/v1/auth/me`，用于开发者页面权限态和审计提示。
          </CardDescription>
        </div>
        <Badge className={badgeClass(subject ? "ok" : "warn")}>
          {loading ? "解析中" : subject?.mode ?? "未登录"}
        </Badge>
      </div>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        <InfoItem label="主体" value={loading ? "解析中" : subjectDisplayName(subject)} />
        <InfoItem label="角色" value={subjectRoles(subject)} />
        <InfoItem label="租户" value={subject?.tenant_id ?? subject?.org_id ?? "未绑定"} />
        <InfoItem label="作用域" value={subject?.auth_context_level ?? "未解析"} />
      </div>
    </Card>
  );
}

function BoundaryCard() {
  return (
    <Card>
      <CardTitle>受控访问边界</CardTitle>
      <CardDescription className="mt-2">
        浏览器端只请求 `/api/platform/**`，由 Next.js server route 代理到 `platform-core`；不得直连 Kafka、PostgreSQL、OpenSearch、Redis、Fabric。
      </CardDescription>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        {["platform-core API", "Keycloak / IAM", "SDK TS OpenAPI", "WebSocket reserved"].map((item) => (
          <div key={item} className="rounded-2xl bg-black/[0.04] px-4 py-3 text-sm font-medium text-[var(--ink-strong)]">
            {item}
          </div>
        ))}
      </div>
    </Card>
  );
}

function NetworkCard({ health, loading }: { health?: HealthDepsData; loading: boolean }) {
  return (
    <Card>
      <div className="flex items-center gap-3">
        <Network className="size-5 text-[var(--accent-strong)]" />
        <CardTitle>网络与运行模式</CardTitle>
      </div>
      <CardDescription className="mt-2">
        开发者页展示 RPC、浏览器、faucet 与模式说明，但真实数据仍由 `platform-core` 聚合。
      </CardDescription>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        <InfoItem label="运行模式" value="local / staging / demo" />
        <InfoItem label="健康状态" value={loading ? "检查中" : health ? "health/deps reachable" : "等待权限"} />
        <InfoItem label="RPC 边界" value="后端受控配置" />
        <InfoItem label="区块浏览器" value="只显示 tx_hash / status" />
      </div>
    </Card>
  );
}

function DeveloperQuickNav({ appCount }: { appCount: number }) {
  return (
    <Card>
      <CardTitle>调试导航</CardTitle>
      <CardDescription className="mt-2">
        应用、API Key、trace、Mock 支付四个入口均已绑定正式 API。
      </CardDescription>
      <div className="mt-5 grid gap-3">
        <InfoItem label="测试应用数" value={String(appCount)} />
        <InfoItem label="调用日志" value="/api/v1/developer/trace recent_logs" />
        <InfoItem label="Mock 支付" value="simulate-success / fail / timeout" />
      </div>
    </Card>
  );
}

function ApplicationTable({
  applications,
  loading,
  error,
  selectedAppId,
  onSelect,
}: {
  applications: ApplicationRow[];
  loading: boolean;
  error: unknown;
  selectedAppId?: string;
  onSelect: (app: ApplicationRow) => void;
}) {
  const columns = useMemo<ColumnDef<ApplicationRow>[]>(
    () => [
      {
        header: "应用",
        accessorKey: "app_name",
        cell: ({ row }) => (
          <button
            className="text-left font-semibold text-[var(--ink-strong)] underline-offset-4 hover:underline"
            onClick={() => onSelect(row.original)}
            type="button"
          >
            {row.original.app_name}
          </button>
        ),
      },
      { header: "client_id", accessorKey: "client_id" },
      { header: "app_type", accessorKey: "app_type" },
      {
        header: "status",
        cell: ({ row }) => <Badge className={badgeClass(statusTone(row.original.status))}>{row.original.status}</Badge>,
      },
      {
        header: "secret",
        cell: ({ row }) => (
          <Badge className={badgeClass(statusTone(row.original.client_secret_status))}>
            {row.original.client_secret_status}
          </Badge>
        ),
      },
    ],
    [onSelect],
  );
  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable({
    data: applications,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });
  const rows = table.getRowModel().rows;
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 64,
    overscan: 8,
  });

  return (
    <Card>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <CardTitle>应用列表</CardTitle>
          <CardDescription className="mt-2">
            真实读取 `GET /api/v1/apps`，列表使用 TanStack Table + Virtual 承载长列表。
          </CardDescription>
        </div>
        <Badge>{applications.length} apps</Badge>
      </div>
      {loading ? <LoadingState label="正在读取应用列表" /> : null}
      {error ? <InlineError error={error} /> : null}
      {!loading && !error && !applications.length ? (
        <EmptyState title="暂无测试应用" description="创建应用后会在此展示 client_id、secret 状态和生命周期。" />
      ) : null}
      {applications.length ? (
        <div className="mt-5 overflow-hidden rounded-3xl border border-black/10 bg-white/70">
          <div className="grid grid-cols-[1.4fr_1.2fr_0.8fr_0.7fr_0.7fr] gap-3 border-b border-black/10 px-4 py-3 text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
            {table.getHeaderGroups()[0]?.headers.map((header) => (
              <div key={header.id}>{flexRender(header.column.columnDef.header, header.getContext())}</div>
            ))}
          </div>
          <div ref={parentRef} className="relative h-[320px] overflow-auto">
            <div style={{ height: `${virtualizer.getTotalSize()}px` }}>
              {virtualizer.getVirtualItems().map((virtualRow) => {
                const row = rows[virtualRow.index];
                if (!row) {
                  return null;
                }
                return (
                  <div
                    key={row.id}
                    className={cn(
                      "absolute left-0 grid w-full grid-cols-[1.4fr_1.2fr_0.8fr_0.7fr_0.7fr] gap-3 px-4 py-3 text-sm",
                      selectedAppId === row.original.app_id ? "bg-[var(--accent-soft)]" : "bg-white/70",
                    )}
                    style={{ transform: `translateY(${virtualRow.start}px)` }}
                  >
                    {row.getVisibleCells().map((cell) => (
                      <div key={cell.id} className="min-w-0 truncate">
                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                      </div>
                    ))}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      ) : null}
    </Card>
  );
}

function ApplicationDetail({ app }: { app: ApplicationRow }) {
  return (
    <Card>
      <CardTitle>已选应用调用配置</CardTitle>
      <div className="mt-5 grid gap-3 md:grid-cols-3">
        <InfoItem label="app_id" value={app.app_id} />
        <InfoItem label="client_id" value={app.client_id} />
        <InfoItem label="secret_status" value={app.client_secret_status} />
        <InfoItem label="org_id" value={app.org_id} />
        <InfoItem label="status" value={app.status} />
        <InfoItem label="正式调用边界" value="Authorization: Bearer + SDK" />
      </div>
    </Card>
  );
}

function DeveloperTraceResult({ data }: { data: DeveloperTraceData }) {
  const subject = data.subject;
  return (
    <div className="space-y-4">
      <Card>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <CardTitle>业务状态 / 链状态 / 投影状态</CardTitle>
            <CardDescription className="mt-2">
              `request_id`、`tx_hash`、链状态和投影状态来自后端联查结果，不在前端拼装真相源。
            </CardDescription>
          </div>
          <Badge className={badgeClass(statusTone(subject.reconcile_status))}>{subject.reconcile_status}</Badge>
        </div>
        <div className="mt-5 grid gap-3 md:grid-cols-3">
          <InfoItem label="request_id" value={subject.request_id ?? "无"} />
          <InfoItem label="trace_id" value={subject.trace_id ?? "无"} />
          <InfoItem label="tx_hash" value={data.matched_chain_anchor?.tx_hash ?? data.recent_chain_anchors[0]?.tx_hash ?? "无"} />
          <InfoItem label="business_status" value={subject.business_status} />
          <InfoItem label="payment_status" value={subject.payment_status} />
          <InfoItem label="proof_commit_state" value={subject.proof_commit_state} />
          <InfoItem label="projection_gap" value={data.matched_projection_gap?.gap_status ?? subject.reconcile_status} />
          <InfoItem label="external_fact_status" value={subject.external_fact_status} />
          <InfoItem label="resolved_order_id" value={subject.resolved_order_id} />
        </div>
      </Card>
      <div className="grid gap-4 xl:grid-cols-2">
        <TraceList
          title="调用日志 recent_logs"
          items={data.recent_logs.map((item) => ({
            key: item.system_log_id ?? item.request_id ?? item.created_at ?? item.message_text,
            title: item.message_text,
            detail: `request_id=${item.request_id ?? "无"} / trace_id=${item.trace_id ?? "无"} / level=${item.log_level ?? "n/a"}`,
          }))}
        />
        <TraceList
          title="审计 recent_audit_traces"
          items={data.recent_audit_traces.map((item) => ({
            key: item.audit_id ?? item.request_id ?? item.event_hash ?? "audit",
            title: `${item.domain_name}.${item.action_name}`,
            detail: `result=${item.result_code} / request_id=${item.request_id ?? "无"} / tx_hash=${item.tx_hash ?? "无"}`,
          }))}
        />
        <TraceList
          title="outbox / dead letter"
          items={[
            ...data.recent_outbox_events.map((item) => ({
              key: item.outbox_event_id ?? `${item.aggregate_type}-${item.event_type}`,
              title: `${item.aggregate_type}.${item.event_type}`,
              detail: `status=${item.status} / request_id=${item.request_id ?? "无"}`,
            })),
            ...data.recent_dead_letters.map((item) => ({
              key: item.dead_letter_event_id ?? `${item.aggregate_type}-${item.event_type}`,
              title: item.failure_stage ?? item.event_type,
              detail: `status=${item.reprocess_status} / request_id=${item.request_id ?? "无"}`,
            })),
          ]}
        />
        <TraceList
          title="链锚定 / 投影"
          items={[
            ...data.recent_chain_anchors.map((item) => ({
              key: item.chain_anchor_id ?? `${item.chain_id}-${item.digest}`,
              title: item.status,
              detail: `tx_hash=${item.tx_hash ?? "无"} / reconcile=${item.reconcile_status}`,
            })),
            ...data.recent_projection_gaps.map((item) => ({
              key: item.chain_projection_gap_id ?? `${item.chain_id}-${item.gap_type}`,
              title: item.gap_status,
              detail: `aggregate=${item.aggregate_type}:${item.aggregate_id ?? "无"} / gap_type=${item.gap_type}`,
            })),
          ]}
        />
      </div>
    </div>
  );
}

function TraceList({
  title,
  items,
}: {
  title: string;
  items: Array<{ key: string; title: string; detail: string }>;
}) {
  return (
    <Card>
      <CardTitle>{title}</CardTitle>
      <div className="mt-4 space-y-3">
        {items.length ? (
          items.slice(0, 8).map((item) => (
            <div key={item.key} className="rounded-2xl bg-black/[0.04] p-4">
              <div className="font-medium text-[var(--ink-strong)]">{item.title}</div>
              <div className="mt-1 text-xs leading-5 text-[var(--ink-soft)]">{item.detail}</div>
            </div>
          ))
        ) : (
          <CardDescription>当前联查键暂无该类记录。</CardDescription>
        )}
      </div>
    </Card>
  );
}

function MockPaymentResult({ data }: { data: MockPaymentData }) {
  return (
    <Card>
      <CardTitle>Mock 支付结果</CardTitle>
      <div className="mt-5 grid gap-3 md:grid-cols-3">
        <InfoItem label="case_id" value={data.mock_payment_case_id} />
        <InfoItem label="payment_intent_id" value={data.payment_intent_id} />
        <InfoItem label="scenario" value={data.scenario_type} />
        <InfoItem label="provider_event_id" value={data.provider_event_id} />
        <InfoItem label="provider_status" value={data.provider_status} />
        <InfoItem label="webhook_processed_status" value={data.webhook_processed_status} />
        <InfoItem label="payment_transaction_id" value={data.payment_transaction_id ?? "无"} />
        <InfoItem label="applied_payment_status" value={data.applied_payment_status ?? "无"} />
        <InfoItem label="duplicate_webhook" value={String(data.duplicate_webhook)} />
      </div>
    </Card>
  );
}

function TestAssetsCard() {
  return (
    <Card>
      <CardTitle>测试资产与限制</CardTitle>
      <CardDescription className="mt-2">
        测试钱包、RPC、浏览器和 faucet 信息只作为调试导航展示；Mock 支付入口只允许非生产 provider。
      </CardDescription>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        <InfoItem label="provider" value="mock_payment_provider" />
        <InfoItem label="运行形态" value="non-production only" />
        <InfoItem label="状态回查" value="payment + audit + trace" />
        <InfoItem label="幂等保护" value="X-Idempotency-Key" />
      </div>
    </Card>
  );
}

function AuditNotice() {
  return (
    <Card>
      <CardTitle>审计留痕提示</CardTitle>
      <CardDescription className="mt-2">
        应用创建、更新、secret 轮换、secret 撤销会记录 `iam.app.*` 审计动作；高敏操作需要人工确认和 step-up 上下文透传。
      </CardDescription>
      <div className="mt-5 rounded-2xl border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-4 text-sm text-[var(--warning-ink)]">
        下载类对象路径不会在页面暴露；API Key 明文不会从后端返回，也不会在前端持久化。
      </div>
    </Card>
  );
}

function TraceModeNotice() {
  return (
    <Card>
      <CardTitle>联查模式</CardTitle>
      <CardDescription className="mt-2">
        当前正式接口支持 `order_id`、`event_id`、`tx_hash` 三种 selector；`request_id` 在结果中展示并可继续跳转审计 / ops 联查。
      </CardDescription>
      <div className="mt-5 flex flex-wrap gap-2">
        {traceLookupModes.map((mode) => (
          <Badge key={mode}>{mode}</Badge>
        ))}
      </div>
    </Card>
  );
}

function PermissionNotice({ required }: { required: string }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <CardTitle>权限态</CardTitle>
      <CardDescription className="mt-2 text-[var(--warning-ink)]">
        当前主体缺少 `{required}` 或对应角色，页面已拦截写操作并保留审计提示。
      </CardDescription>
    </Card>
  );
}

function EmptyState({ title, description }: { title: string; description: string }) {
  return (
    <Card className="border-dashed">
      <CardTitle>{title}</CardTitle>
      <CardDescription className="mt-2">{description}</CardDescription>
    </Card>
  );
}

function LoadingState({ label }: { label: string }) {
  return (
    <Card>
      <div className="flex items-center gap-3 text-sm text-[var(--ink-soft)]">
        <LoaderCircle className="size-4 animate-spin" />
        {label}
      </div>
    </Card>
  );
}

function ErrorState({ error }: { error: unknown }) {
  return (
    <Card className="border-red-200 bg-red-50/80">
      <CardTitle>错误态</CardTitle>
      <CardDescription className="mt-2 text-red-700">{formatDeveloperError(error)}</CardDescription>
    </Card>
  );
}

function InlineError({ error }: { error: unknown }) {
  return (
    <div className="mt-4 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
      {formatDeveloperError(error)}
    </div>
  );
}

function InfoItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-2xl bg-black/[0.04] p-4">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</div>
      <div className="mt-2 truncate text-sm font-medium text-[var(--ink-strong)]" title={value}>
        {value}
      </div>
    </div>
  );
}

function InputField({
  label,
  error,
  ...props
}: ComponentProps<typeof Input> & { label: string; error?: string }) {
  return (
    <label className="block space-y-2">
      <span className="text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</span>
      <Input {...props} />
      {error ? <span className="text-xs text-red-600">{error}</span> : null}
    </label>
  );
}

function SelectField({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: Array<[string, string]>;
  onChange: (value: string) => void;
}) {
  return (
    <label className="block space-y-2">
      <span className="text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</span>
      <select
        className="h-11 w-full rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        {options.map(([optionValue, labelText]) => (
          <option key={optionValue} value={optionValue}>
            {labelText}
          </option>
        ))}
      </select>
    </label>
  );
}

function badgeClass(tone: string) {
  if (tone === "ok") {
    return "border-emerald-200 bg-emerald-50 text-emerald-700";
  }
  if (tone === "danger") {
    return "border-red-200 bg-red-50 text-red-700";
  }
  if (tone === "warn") {
    return "border-amber-200 bg-amber-50 text-amber-700";
  }
  return "border-slate-200 bg-slate-50 text-slate-600";
}
