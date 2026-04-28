"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import type {
  ApplicationListResponse,
  DeveloperTraceResponse,
  MockPaymentSimulationResponse,
} from "@datab/sdk-ts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
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
import { useEffect, useId, useState, type ComponentProps, type ReactNode } from "react";
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

import { PortalRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();
type ApplicationRow = ApplicationListResponse["data"][number];
type DeveloperTraceData = DeveloperTraceResponse["data"];
type MockPaymentData = MockPaymentSimulationResponse["data"];

export function DeveloperHomeShell() {
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canRead = canReadDeveloper(subject);
  const scopeOrgId = subject?.tenant_id || subject?.org_id || undefined;
  const appsQuery = useQuery({
    queryKey: ["portal", "developer", "apps", scopeOrgId],
    queryFn: () => sdk.iam.listApplications(scopeOrgId ? { org_id: scopeOrgId } : {}),
    enabled: canRead,
  });

  return (
    <PortalRouteScaffold routeKey="developer_home">
      <DeveloperHero
        title="开发者工作台"
        description="面向测试应用、API Key、调用日志、trace 联查和 Mock 支付的门户入口；浏览器端只经过 `/api/platform/**` 访问 platform-core。"
        icon={<TerminalSquare className="size-5" />}
        badges={["developer.home.read", "SDK + OpenAPI", "platform-core only"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <BoundaryCard />
      </div>
      {!canRead ? <PermissionNotice required="developer.home.read" /> : null}
      <div className="grid gap-4 lg:grid-cols-3">
        <FeatureCard title="测试应用" value={`${appsQuery.data?.data.length ?? 0} apps`} description="读取 `/api/v1/apps`，创建和更新均附带 Idempotency-Key。" />
        <FeatureCard title="调用日志" value="recent_logs" description="通过 `/api/v1/developer/trace` 展示 request_id、trace_id 与 system log。" />
        <FeatureCard title="Mock 支付" value="success / fail / timeout" description="非生产入口，绑定正式 billing mock payment API。" />
      </div>
    </PortalRouteScaffold>
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
    queryKey: ["portal", "developer", "apps", scopeOrgId],
    queryFn: () => sdk.iam.listApplications(scopeOrgId ? { org_id: scopeOrgId } : {}),
    enabled: canRead,
  });
  const createForm = useForm<ApplicationCreateFormValues>({
    resolver: zodResolver(applicationCreateSchema),
    defaultValues: {
      org_id: "",
      app_name: "WEB-016 Portal App",
      app_type: "api_client",
      client_id: `portal-web016-${idSeed}`,
      client_secret_hash: "",
      idempotency_key: `web-016:portal-app-create:${idSeed}`,
    },
  });
  const patchForm = useForm<ApplicationPatchFormValues>({
    resolver: zodResolver(applicationPatchSchema),
    defaultValues: {
      app_id: "",
      app_name: "",
      status: "active",
      idempotency_key: `web-016:portal-app-patch:${idSeed}`,
    },
  });
  const secretForm = useForm<ApplicationSecretFormValues>({
    resolver: zodResolver(applicationSecretSchema),
    defaultValues: {
      app_id: "",
      client_secret_hash: "",
      idempotency_key: `web-016:portal-app-secret:${idSeed}`,
      step_up_token: "",
      step_up_challenge_id: "",
    },
  });

  useEffect(() => {
    const orgId = subject?.tenant_id ?? subject?.org_id;
    if (orgId && !createForm.getValues("org_id")) {
      createForm.setValue("org_id", orgId);
    }
  }, [createForm, subject?.org_id, subject?.tenant_id]);

  const createMutation = useMutation({
    mutationFn: (values: ApplicationCreateFormValues) =>
      sdk.iam.createApplication(buildCreateApplicationPayload(values), {
        idempotencyKey: values.idempotency_key,
      }),
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: ["portal", "developer", "apps"] });
      selectApplication(response.data);
      createForm.setValue("idempotency_key", createDeveloperIdempotencyKey("portal-app-create"));
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
      queryClient.invalidateQueries({ queryKey: ["portal", "developer", "apps"] });
      selectApplication(response.data);
      patchForm.setValue("idempotency_key", createDeveloperIdempotencyKey("portal-app-patch"));
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
      queryClient.invalidateQueries({ queryKey: ["portal", "developer", "apps"] });
      selectApplication(response.data);
      secretForm.setValue("idempotency_key", createDeveloperIdempotencyKey("portal-app-secret"));
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
      queryClient.invalidateQueries({ queryKey: ["portal", "developer", "apps"] });
      selectApplication(response.data);
      secretForm.setValue("idempotency_key", createDeveloperIdempotencyKey("portal-app-secret"));
    },
  });

  function selectApplication(app: ApplicationRow) {
    setSelectedApp(app);
    patchForm.reset({
      app_id: app.app_id,
      app_name: app.app_name,
      status: applicationStatuses.includes(app.status as ApplicationPatchFormValues["status"])
        ? (app.status as ApplicationPatchFormValues["status"])
        : "active",
      idempotency_key: createDeveloperIdempotencyKey("portal-app-patch"),
    });
    secretForm.reset({
      app_id: app.app_id,
      client_secret_hash: "",
      idempotency_key: createDeveloperIdempotencyKey("portal-app-secret"),
      step_up_token: "",
      step_up_challenge_id: "",
    });
  }

  return (
    <PortalRouteScaffold routeKey="developer_apps">
      <DeveloperHero
        title="应用管理与 API Key"
        description="创建测试应用、更新生命周期、轮换/撤销 API Key；响应只展示 secret 状态，所有写操作携带 Idempotency-Key。"
        icon={<KeyRound className="size-5" />}
        badges={["developer.app.read", "developer.app.create", "developer.app.update"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <AuditNotice />
      </div>
      {!canRead ? <PermissionNotice required="developer.app.read" /> : null}
      <ApplicationCards
        applications={appsQuery.data?.data ?? []}
        loading={appsQuery.isPending && canRead}
        error={appsQuery.error}
        selectedAppId={selectedApp?.app_id}
        onSelect={selectApplication}
      />
      <div className="grid gap-4 xl:grid-cols-3">
        <Card>
          <CardTitle>创建测试应用</CardTitle>
          <form className="mt-5 space-y-4" onSubmit={createForm.handleSubmit((values) => createMutation.mutate(values))}>
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
              创建
            </Button>
          </form>
          {createMutation.isError ? <InlineError error={createMutation.error} /> : null}
        </Card>
        <Card>
          <CardTitle>更新应用</CardTitle>
          <form className="mt-5 space-y-4" onSubmit={patchForm.handleSubmit((values) => patchMutation.mutate(values))}>
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
          <CardTitle>API Key 操作</CardTitle>
          <CardDescription className="mt-2">轮换和撤销只返回 `client_secret_status`，不暴露明文 secret。</CardDescription>
          <form className="mt-5 space-y-4">
            <InputField label="app_id" error={secretForm.formState.errors.app_id?.message} {...secretForm.register("app_id")} />
            <InputField label="client_secret_hash" error={secretForm.formState.errors.client_secret_hash?.message} {...secretForm.register("client_secret_hash")} />
            <InputField label="Idempotency-Key" error={secretForm.formState.errors.idempotency_key?.message} {...secretForm.register("idempotency_key")} />
            <InputField label="X-Step-Up-Token" error={secretForm.formState.errors.step_up_token?.message} {...secretForm.register("step_up_token")} />
            <div className="flex flex-wrap gap-2">
              <Button disabled={!canManage || rotateMutation.isPending} type="button" onClick={secretForm.handleSubmit((values) => rotateMutation.mutate(values))}>
                {rotateMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <KeyRound className="size-4" />}
                轮换
              </Button>
              <Button disabled={!canManage || revokeMutation.isPending} type="button" variant="warning" onClick={secretForm.handleSubmit((values) => revokeMutation.mutate(values))}>
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
    </PortalRouteScaffold>
  );
}

export function DeveloperTraceShell() {
  const [lookup, setLookup] = useState<DeveloperTraceFormValues | null>(null);
  const authQuery = useAuthMe();
  const subject = authQuery.data?.data;
  const canRead = canReadDeveloperTrace(subject);
  const traceForm = useForm<DeveloperTraceFormValues>({
    resolver: zodResolver(developerTraceSchema),
    defaultValues: { lookup_mode: "order_id", lookup_value: "" },
  });
  const traceQuery = useQuery({
    queryKey: ["portal", "developer", "trace", lookup],
    queryFn: () => sdk.ops.getDeveloperTrace(buildDeveloperTraceQuery(lookup!)),
    enabled: Boolean(canRead && lookup),
  });

  return (
    <PortalRouteScaffold routeKey="developer_trace">
      <DeveloperHero
        title="Trace 与调用日志联查"
        description="按 order_id / event_id / tx_hash 汇总业务状态、链状态、投影状态、审计与 system log。"
        icon={<Search className="size-5" />}
        badges={["developer.trace.read", "request_id", "tx_hash", "projection"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <TraceModeNotice />
      </div>
      {!canRead ? <PermissionNotice required="developer.trace.read" /> : null}
      <Card>
        <form className="grid gap-4 lg:grid-cols-[180px_1fr_auto]" onSubmit={traceForm.handleSubmit((values) => setLookup(values))}>
          <SelectField
            label="lookup_mode"
            value={useWatch({ control: traceForm.control, name: "lookup_mode" })}
            options={traceLookupModes.map((value) => [value, value])}
            onChange={(value) => traceForm.setValue("lookup_mode", value as DeveloperTraceFormValues["lookup_mode"], { shouldDirty: true })}
          />
          <InputField label="lookup_value" error={traceForm.formState.errors.lookup_value?.message} {...traceForm.register("lookup_value")} />
          <Button disabled={!canRead || traceQuery.isFetching} type="submit">
            {traceQuery.isFetching ? <LoaderCircle className="size-4 animate-spin" /> : <Search className="size-4" />}
            联查
          </Button>
        </form>
      </Card>
      {traceQuery.isPending && lookup ? <LoadingState label="正在联查 trace / audit / log" /> : null}
      {traceQuery.isError ? <ErrorState error={traceQuery.error} /> : null}
      {traceQuery.isSuccess ? (
        <DeveloperTraceResult data={traceQuery.data.data} />
      ) : lookup ? null : (
        <EmptyState title="等待联查键" description="输入正式联查键后展示 request_id、tx_hash、链状态和调用日志。" />
      )}
    </PortalRouteScaffold>
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
      idempotency_key: `web-016:portal-mock-payment:${idSeed}`,
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
      mockForm.setValue("idempotency_key", createDeveloperIdempotencyKey("portal-mock-payment"));
    },
  });

  return (
    <PortalRouteScaffold routeKey="developer_assets">
      <DeveloperHero
        title="Mock 支付操作入口"
        description="绑定 billing mock payment simulate-success / fail / timeout；页面展示非生产限制、幂等提交和审计提示。"
        icon={<CreditCard className="size-5" />}
        badges={["developer.mock_payment.simulate", "non-production", "audit"]}
      />
      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <SubjectCard subject={subject} loading={authQuery.isPending} />
        <FeatureCard title="测试资产" value="mock provider" description="测试钱包、faucet、RPC 和浏览器信息只作为后端受控配置展示。" />
      </div>
      {!canSimulate ? <PermissionNotice required="developer.mock_payment.simulate" /> : null}
      <Card>
        <CardTitle>触发 Mock 支付</CardTitle>
        <form className="mt-5 grid gap-4 xl:grid-cols-2" onSubmit={mockForm.handleSubmit((values) => mockMutation.mutate(values))}>
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
          <Button disabled={!canSimulate || mockMutation.isPending} type="submit">
            {mockMutation.isPending ? <LoaderCircle className="size-4 animate-spin" /> : <CreditCard className="size-4" />}
            触发 {scenario}
          </Button>
        </form>
        {mockMutation.isError ? <InlineError error={mockMutation.error} /> : null}
      </Card>
      {result ? <MockPaymentResult data={result} /> : <EmptyState title="等待 Mock 支付操作" description="提交后展示 provider_event_id、webhook_processed_status 和支付投影。" />}
    </PortalRouteScaffold>
  );
}

function useAuthMe() {
  return useQuery({
    queryKey: ["portal", "developer", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
  });
}

function DeveloperHero({ title, description, icon, badges }: { title: string; description: string; icon: ReactNode; badges: string[] }) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.25 }}
      className="rounded-[32px] border border-white/70 bg-[linear-gradient(135deg,rgba(247,251,246,0.98),rgba(229,242,247,0.92),rgba(255,255,255,0.9))] p-6 shadow-[0_24px_80px_rgba(18,41,52,0.10)]"
    >
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="max-w-3xl">
          <div className="mb-4 flex size-11 items-center justify-center rounded-2xl bg-[var(--accent-strong)] text-white">{icon}</div>
          <h2 className="text-2xl font-semibold tracking-tight text-[var(--ink-strong)]">{title}</h2>
          <p className="mt-3 max-w-2xl text-sm leading-6 text-[var(--ink-soft)]">{description}</p>
        </div>
        <div className="flex max-w-xl flex-wrap gap-2">
          {badges.map((badge) => <Badge key={badge}>{badge}</Badge>)}
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
          <CardDescription className="mt-2">来自 `GET /api/v1/auth/me`，用于开发者权限态与审计提示。</CardDescription>
        </div>
        <Badge className={badgeClass(subject ? "ok" : "warn")}>{loading ? "解析中" : subject?.mode ?? "未登录"}</Badge>
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
      <div className="flex items-center gap-3">
        <Network className="size-5 text-[var(--accent-strong)]" />
        <CardTitle>受控访问边界</CardTitle>
      </div>
      <CardDescription className="mt-2">
        portal-web 不直连 Kafka、PostgreSQL、OpenSearch、Redis、Fabric；所有状态通过 platform-core 和 sdk-ts 契约读取。
      </CardDescription>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        <InfoItem label="认证" value="Keycloak / IAM" />
        <InfoItem label="API" value="/api/platform/**" />
        <InfoItem label="审计" value="iam.app.* / mock payment" />
        <InfoItem label="链状态" value="tx_hash + projection" />
      </div>
    </Card>
  );
}

function ApplicationCards({ applications, loading, error, selectedAppId, onSelect }: { applications: ApplicationRow[]; loading: boolean; error: unknown; selectedAppId?: string; onSelect: (app: ApplicationRow) => void }) {
  return (
    <Card>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <CardTitle>应用列表</CardTitle>
          <CardDescription className="mt-2">真实读取 `GET /api/v1/apps`，卡片展示 API Key 状态和生命周期。</CardDescription>
        </div>
        <Badge>{applications.length} apps</Badge>
      </div>
      {loading ? <LoadingState label="正在读取应用列表" /> : null}
      {error ? <InlineError error={error} /> : null}
      {!loading && !error && !applications.length ? <EmptyState title="暂无测试应用" description="创建应用后会在此展示 client_id、secret 状态和生命周期。" /> : null}
      <div className="mt-5 grid gap-3 lg:grid-cols-2">
        {applications.map((app) => (
          <button
            key={app.app_id}
            className={cn("rounded-3xl border p-4 text-left transition hover:-translate-y-0.5 hover:bg-white", selectedAppId === app.app_id ? "border-[var(--accent-strong)] bg-[var(--accent-soft)]" : "border-black/10 bg-white/70")}
            onClick={() => onSelect(app)}
            type="button"
          >
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <div className="truncate font-semibold text-[var(--ink-strong)]">{app.app_name}</div>
                <div className="mt-1 truncate text-xs text-[var(--ink-soft)]">{app.client_id}</div>
              </div>
              <Badge className={badgeClass(statusTone(app.client_secret_status))}>{app.client_secret_status}</Badge>
            </div>
            <div className="mt-4 grid gap-2 text-xs text-[var(--ink-soft)] md:grid-cols-2">
              <span>{app.app_type}</span>
              <span>{app.status}</span>
            </div>
          </button>
        ))}
      </div>
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
            <CardDescription className="mt-2">展示 request_id、trace_id、tx_hash、链锚定和投影状态。</CardDescription>
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
        </div>
      </Card>
      <div className="grid gap-4 xl:grid-cols-2">
        <TraceList title="调用日志 recent_logs" items={data.recent_logs.map((item) => ({ key: item.system_log_id ?? item.message_text, title: item.message_text, detail: `request_id=${item.request_id ?? "无"} / trace_id=${item.trace_id ?? "无"}` }))} />
        <TraceList title="审计 recent_audit_traces" items={data.recent_audit_traces.map((item) => ({ key: item.audit_id ?? item.event_hash ?? item.action_name, title: `${item.domain_name}.${item.action_name}`, detail: `result=${item.result_code} / tx_hash=${item.tx_hash ?? "无"}` }))} />
      </div>
    </div>
  );
}

function TraceList({ title, items }: { title: string; items: Array<{ key: string; title: string; detail: string }> }) {
  return (
    <Card>
      <CardTitle>{title}</CardTitle>
      <div className="mt-4 space-y-3">
        {items.length ? items.slice(0, 8).map((item) => (
          <div key={item.key} className="rounded-2xl bg-black/[0.04] p-4">
            <div className="font-medium text-[var(--ink-strong)]">{item.title}</div>
            <div className="mt-1 text-xs leading-5 text-[var(--ink-soft)]">{item.detail}</div>
          </div>
        )) : <CardDescription>当前联查键暂无该类记录。</CardDescription>}
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
        <InfoItem label="provider_event_id" value={data.provider_event_id} />
        <InfoItem label="webhook_status" value={data.webhook_processed_status} />
        <InfoItem label="transaction_id" value={data.payment_transaction_id ?? "无"} />
        <InfoItem label="payment_status" value={data.applied_payment_status ?? "无"} />
        <InfoItem label="duplicate" value={String(data.duplicate_webhook)} />
      </div>
    </Card>
  );
}

function AuditNotice() {
  return (
    <Card>
      <CardTitle>审计留痕提示</CardTitle>
      <CardDescription className="mt-2">应用创建、更新、secret 轮换和撤销都会记录 `iam.app.*` 审计动作；明文 secret 不返回。</CardDescription>
    </Card>
  );
}

function TraceModeNotice() {
  return (
    <Card>
      <CardTitle>联查模式</CardTitle>
      <CardDescription className="mt-2">正式 selector 为 order_id、event_id、tx_hash；request_id 在结果中展示。</CardDescription>
      <div className="mt-5 flex flex-wrap gap-2">{traceLookupModes.map((mode) => <Badge key={mode}>{mode}</Badge>)}</div>
    </Card>
  );
}

function FeatureCard({ title, value, description }: { title: string; value: string; description: string }) {
  return (
    <Card>
      <CardTitle>{title}</CardTitle>
      <div className="mt-3 text-xl font-semibold text-[var(--ink-strong)]">{value}</div>
      <CardDescription className="mt-2">{description}</CardDescription>
    </Card>
  );
}

function PermissionNotice({ required }: { required: string }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <CardTitle>权限态</CardTitle>
      <CardDescription className="mt-2 text-[var(--warning-ink)]">当前主体缺少 `{required}` 或对应角色，写操作已被拦截。</CardDescription>
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
  return <div className="mt-4 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">{formatDeveloperError(error)}</div>;
}

function InfoItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-2xl bg-black/[0.04] p-4">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</div>
      <div className="mt-2 truncate text-sm font-medium text-[var(--ink-strong)]" title={value}>{value}</div>
    </div>
  );
}

function InputField({ label, error, ...props }: ComponentProps<typeof Input> & { label: string; error?: string }) {
  return (
    <label className="block space-y-2">
      <span className="text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</span>
      <Input {...props} />
      {error ? <span className="text-xs text-red-600">{error}</span> : null}
    </label>
  );
}

function SelectField({ label, value, options, onChange }: { label: string; value: string; options: Array<[string, string]>; onChange: (value: string) => void }) {
  return (
    <label className="block space-y-2">
      <span className="text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</span>
      <select className="h-11 w-full rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]" value={value} onChange={(event) => onChange(event.target.value)}>
        {options.map(([optionValue, labelText]) => <option key={optionValue} value={optionValue}>{labelText}</option>)}
      </select>
    </label>
  );
}

function badgeClass(tone: string) {
  if (tone === "ok") return "border-emerald-200 bg-emerald-50 text-emerald-700";
  if (tone === "danger") return "border-red-200 bg-red-50 text-red-700";
  if (tone === "warn") return "border-amber-200 bg-amber-50 text-amber-700";
  return "border-slate-200 bg-slate-50 text-slate-600";
}
