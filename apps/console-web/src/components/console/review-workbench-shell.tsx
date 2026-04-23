"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  AlertTriangle,
  CheckCircle2,
  Fingerprint,
  Gavel,
  LoaderCircle,
  LockKeyhole,
  RefreshCcw,
  ShieldAlert,
  ShieldCheck,
  XCircle,
} from "lucide-react";
import { motion } from "motion/react";
import {
  startTransition,
  useDeferredValue,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useForm, useWatch } from "react-hook-form";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  buildReviewDecisionPayload,
  canReadReviewWorkbench,
  canWriteReviewDecision,
  countStandardSkuCoverage,
  createReviewIdempotencyKey,
  deriveComplianceSignals,
  formatDateTime,
  formatReviewError,
  labelReviewStatus,
  reviewAuditActions,
  reviewDecisionFormSchema,
  reviewWorkbenchTitles,
  type OrganizationReviewRow,
  type ProductReviewDetail,
  type ProductReviewRow,
  type ReviewDecisionFormValues,
  type ReviewWorkbenchKind,
  type SessionSubject,
} from "@/lib/review-workbench";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { cn } from "@/lib/utils";

import { ConsoleRouteScaffold } from "./route-scaffold";

const sdk = createBrowserSdk();

const routeKeys = {
  subjects: "review_subjects",
  products: "review_products",
  compliance: "review_compliance",
} as const;

const decisionLabels = {
  approve: "审核通过",
  reject: "审核驳回",
  block: "合规阻断",
};

const emptySubjectRows: OrganizationReviewRow[] = [];
const emptyProductRows: ProductReviewRow[] = [];

type ProductStatusFilter = "pending_review" | "draft" | "listed" | "frozen";

export function ReviewWorkbenchShell({ kind }: { kind: ReviewWorkbenchKind }) {
  const authQuery = useQuery({
    queryKey: ["console", "auth-me"],
    queryFn: () => sdk.iam.getAuthMe(),
  });
  const subject = authQuery.data?.data;
  const canRead = canReadReviewWorkbench(subject);

  return (
    <ConsoleRouteScaffold routeKey={routeKeys[kind]}>
      <div className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
        <Card className="overflow-hidden bg-[linear-gradient(135deg,rgba(42,34,26,0.96),rgba(117,68,33,0.92),rgba(212,145,58,0.78))] text-white">
          <div className="space-y-5">
            <div className="flex flex-wrap gap-2">
              <Badge className="bg-white/15 text-white">WEB-008</Badge>
              <Badge className="bg-white/15 text-white">{reviewWorkbenchTitles[kind]}</Badge>
              <Badge className="bg-white/15 text-white">
                audit:{reviewAuditActions[kind]}
              </Badge>
            </div>
            <div>
              <h1 className="text-3xl font-semibold tracking-tight">
                审核队列、详情、权限和决策写入在同一工作台闭环。
              </h1>
              <p className="mt-3 max-w-3xl text-sm leading-7 text-white/80">
                当前页面只通过 `/api/platform/**` 代理访问 `platform-core`，不直连
                PostgreSQL、Kafka、OpenSearch、Redis 或 Fabric。主按钮会展示幂等键、
                审计动作和高风险合规阻断确认。
              </p>
            </div>
            <div className="grid gap-3 md:grid-cols-4">
              <HeroMetric label="当前主体" value={subjectLabel(subject)} />
              <HeroMetric label="当前角色" value={subject?.roles.join(" / ") || "visitor"} />
              <HeroMetric label="租户/组织" value={subject?.tenant_id ?? subject?.org_id ?? "public"} />
              <HeroMetric label="认证上下文" value={subject?.auth_context_level ?? "public"} />
            </div>
          </div>
        </Card>

        <Card>
          <CardTitle>审核边界</CardTitle>
          <CardDescription className="mt-2">
            `platform_reviewer / platform_admin` 可查看和执行审核；高风险合规阻断以前端人工确认和可选
            `X-Step-Up-Token` 透传承接，不发明新的后端状态名。
          </CardDescription>
          <div className="mt-4 grid gap-3">
            <BoundaryRow icon={<Fingerprint className="size-4" />} label="主体上下文" value="页面顶部和工作台内均展示主体、角色、租户、作用域" />
            <BoundaryRow icon={<ShieldCheck className="size-4" />} label="权限点" value="review.subject/product/compliance.read + review.*.review" />
            <BoundaryRow icon={<Gavel className="size-4" />} label="审计动作" value={reviewAuditActions[kind]} />
            <BoundaryRow icon={<LockKeyhole className="size-4" />} label="写入头" value="X-Idempotency-Key，合规阻断可透传 X-Step-Up-Token" />
          </div>
        </Card>
      </div>

      {authQuery.isPending ? (
        <LoadingPanel title="正在解析控制台会话" />
      ) : authQuery.isError ? (
        <ErrorPanel error={authQuery.error} onRetry={() => void authQuery.refetch()} />
      ) : !canRead ? (
        <ForbiddenPanel subject={subject} />
      ) : kind === "subjects" ? (
        <SubjectReviewContent subject={subject} />
      ) : (
        <ProductReviewContent kind={kind} subject={subject} />
      )}
    </ConsoleRouteScaffold>
  );
}

function SubjectReviewContent({ subject }: { subject?: SessionSubject }) {
  const [status, setStatus] = useState("pending_review");
  const [orgType, setOrgType] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const listQuery = useQuery({
    queryKey: ["review", "subjects", status, orgType],
    queryFn: () =>
      sdk.iam.listOrganizations({
        status,
        org_type: orgType || undefined,
      }),
  });
  const rows = listQuery.data?.data ?? emptySubjectRows;
  const effectiveSelectedId = rows.some((row) => row.org_id === selectedId)
    ? selectedId
    : (rows[0]?.org_id ?? null);

  const detailQuery = useQuery({
    queryKey: ["review", "subjects", effectiveSelectedId, "detail"],
    queryFn: () => sdk.iam.getOrganization({ id: effectiveSelectedId ?? "" }),
    enabled: Boolean(effectiveSelectedId),
  });

  const columns: Array<ColumnDef<OrganizationReviewRow>> = [
    {
      id: "subject",
      header: "主体",
      cell: ({ row }) => (
        <div>
          <div className="font-semibold text-[var(--ink-strong)]">
            {row.original.org_name}
          </div>
          <div className="mt-1 font-mono text-xs text-[var(--ink-subtle)]">
            {row.original.org_id}
          </div>
        </div>
      ),
    },
    {
      id: "status",
      header: "准入状态",
      cell: ({ row }) => (
        <StatusPill value={row.original.org_status} label={labelReviewStatus(row.original.org_status)} />
      ),
    },
    {
      id: "risk",
      header: "风险/审核",
      cell: ({ row }) => (
        <div className="space-y-1 text-xs text-[var(--ink-soft)]">
          <div>{labelReviewStatus(row.original.review_status)}</div>
          <div>{labelReviewStatus(row.original.risk_status)}</div>
        </div>
      ),
    },
    {
      id: "updated",
      header: "更新时间",
      cell: ({ row }) => formatDateTime(row.original.updated_at),
    },
  ];

  return (
    <WorkbenchGrid
      list={
        <Card className="min-w-0">
          <WorkbenchToolbar
            title="主体准入队列"
            description="读取 GET /api/v1/iam/orgs，按主体状态与类型筛选。"
            onRefresh={() => void listQuery.refetch()}
          >
            <SelectField
              label="状态"
              value={status}
              onChange={setStatus}
              options={[
                ["pending_review", "待审核"],
                ["active", "已启用"],
                ["rejected", "已拒绝"],
                ["frozen", "冻结"],
              ]}
            />
            <SelectField
              label="类型"
              value={orgType}
              onChange={setOrgType}
              options={[
                ["", "全部类型"],
                ["seller", "seller"],
                ["buyer", "buyer"],
                ["enterprise", "enterprise"],
              ]}
            />
          </WorkbenchToolbar>
          <QueryListState
            query={listQuery}
            rows={rows}
            emptyTitle="当前没有主体审核项"
            onRetry={() => void listQuery.refetch()}
          >
            <ReviewVirtualTable
              rows={rows}
              columns={columns}
              getRowId={(row) => row.org_id}
              selectedId={effectiveSelectedId}
              onSelectRow={setSelectedId}
            />
          </QueryListState>
        </Card>
      }
      detail={
        <div className="grid gap-4">
          <SubjectDetailCard
            detail={
              detailQuery.data?.data ??
              rows.find((row) => row.org_id === effectiveSelectedId)
            }
            isLoading={detailQuery.isPending && Boolean(effectiveSelectedId)}
            error={detailQuery.error}
            onRetry={() => void detailQuery.refetch()}
          />
          <ReviewDecisionPanel
            kind="subjects"
            targetId={effectiveSelectedId}
            subject={subject}
            targetTitle={
              detailQuery.data?.data?.org_name ??
              rows.find((row) => row.org_id === effectiveSelectedId)?.org_name
            }
          />
        </div>
      }
    />
  );
}

function ProductReviewContent({
  kind,
  subject,
}: {
  kind: "products" | "compliance";
  subject?: SessionSubject;
}) {
  const [status, setStatus] = useState<ProductStatusFilter>("pending_review");
  const [search, setSearch] = useState("");
  const deferredSearch = useDeferredValue(search.trim());
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const listQuery = useQuery({
    queryKey: ["review", kind, "products", status, deferredSearch],
    queryFn: () =>
      sdk.catalog.listProducts({
        status,
        q: deferredSearch || undefined,
        page: 1,
        page_size: 80,
      }),
  });
  const rows = listQuery.data?.data.items ?? emptyProductRows;
  const effectiveSelectedId = rows.some((row) => row.product_id === selectedId)
    ? selectedId
    : (rows[0]?.product_id ?? null);

  const detailQuery = useQuery({
    queryKey: ["review", kind, "product-detail", effectiveSelectedId],
    queryFn: () => sdk.catalog.getProductDetail({ id: effectiveSelectedId ?? "" }),
    enabled: Boolean(effectiveSelectedId),
  });

  const columns: Array<ColumnDef<ProductReviewRow>> = [
    {
      id: "product",
      header: "商品",
      cell: ({ row }) => (
        <div>
          <div className="font-semibold text-[var(--ink-strong)]">
            {row.original.title}
          </div>
          <div className="mt-1 font-mono text-xs text-[var(--ink-subtle)]">
            {row.original.product_id}
          </div>
        </div>
      ),
    },
    {
      id: "status",
      header: "状态",
      cell: ({ row }) => (
        <StatusPill value={row.original.status} label={labelReviewStatus(row.original.status)} />
      ),
    },
    {
      id: "category",
      header: "分类/SKU",
      cell: ({ row }) => (
        <div className="space-y-1 text-xs text-[var(--ink-soft)]">
          <div>{row.original.category}</div>
          <div>{row.original.delivery_type}</div>
        </div>
      ),
    },
    {
      id: "updated",
      header: "更新时间",
      cell: ({ row }) => formatDateTime(row.original.updated_at),
    },
  ];

  return (
    <WorkbenchGrid
      list={
        <Card className="min-w-0">
          <WorkbenchToolbar
            title={kind === "products" ? "商品审核队列" : "合规审核队列"}
            description="读取 GET /api/v1/products，默认聚焦 pending_review 商品。"
            onRefresh={() => void listQuery.refetch()}
          >
            <SelectField
              label="商品状态"
              value={status}
              onChange={(next) => setStatus(next as ProductStatusFilter)}
              options={[
                ["pending_review", "待审核"],
                ["draft", "草稿"],
                ["listed", "已上架"],
                ["frozen", "冻结"],
              ]}
            />
            <label className="grid gap-1 text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
              搜索
              <Input
                value={search}
                onChange={(event) => {
                  const next = event.target.value;
                  startTransition(() => setSearch(next));
                }}
                placeholder="标题、行业或关键词"
              />
            </label>
          </WorkbenchToolbar>
          <QueryListState
            query={listQuery}
            rows={rows}
            emptyTitle="当前没有商品审核项"
            onRetry={() => void listQuery.refetch()}
          >
            <ReviewVirtualTable
              rows={rows}
              columns={columns}
              getRowId={(row) => row.product_id}
              selectedId={effectiveSelectedId}
              onSelectRow={setSelectedId}
            />
          </QueryListState>
        </Card>
      }
      detail={
        <div className="grid gap-4">
          <ProductDetailCard
            kind={kind}
            detail={detailQuery.data?.data}
            fallback={rows.find((row) => row.product_id === effectiveSelectedId)}
            isLoading={detailQuery.isPending && Boolean(effectiveSelectedId)}
            error={detailQuery.error}
            onRetry={() => void detailQuery.refetch()}
          />
          <ReviewDecisionPanel
            kind={kind}
            targetId={effectiveSelectedId}
            subject={subject}
            targetTitle={
              detailQuery.data?.data?.title ??
              rows.find((row) => row.product_id === effectiveSelectedId)?.title
            }
          />
        </div>
      }
    />
  );
}

function ReviewDecisionPanel({
  kind,
  targetId,
  subject,
  targetTitle,
}: {
  kind: ReviewWorkbenchKind;
  targetId: string | null;
  subject?: SessionSubject;
  targetTitle?: string;
}) {
  const queryClient = useQueryClient();
  const canWrite = canWriteReviewDecision(subject);
  const form = useForm<ReviewDecisionFormValues>({
    resolver: zodResolver(reviewDecisionFormSchema),
    defaultValues: {
      action: kind === "compliance" ? "block" : "approve",
      action_reason: "",
      idempotency_key: createReviewIdempotencyKey(kind),
      step_up_token: "",
      step_up_challenge_id: "",
      block_confirmation: "",
    },
  });
  const selectedAction = useWatch({
    control: form.control,
    name: "action",
  });

  useEffect(() => {
    form.reset({
      action: kind === "compliance" ? "block" : "approve",
      action_reason: "",
      idempotency_key: createReviewIdempotencyKey(kind),
      step_up_token: "",
      step_up_challenge_id: "",
      block_confirmation: "",
    });
  }, [form, kind, targetId]);

  const mutation = useMutation({
    mutationFn: async (values: ReviewDecisionFormValues) => {
      if (!targetId) {
        throw new Error("未选择审核目标");
      }
      const body = buildReviewDecisionPayload(values);
      const options = {
        idempotencyKey: values.idempotency_key,
        stepUpToken: values.step_up_token || undefined,
        stepUpChallengeId: values.step_up_challenge_id || undefined,
      };
      if (kind === "subjects") {
        return sdk.catalog.reviewSubject({ id: targetId }, body, options);
      }
      if (kind === "products") {
        return sdk.catalog.reviewProduct({ id: targetId }, body, options);
      }
      return sdk.catalog.reviewCompliance({ id: targetId }, body, options);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["review"] });
      form.reset({
        action: kind === "compliance" ? "block" : "approve",
        action_reason: "",
        idempotency_key: createReviewIdempotencyKey(kind),
        step_up_token: "",
        step_up_challenge_id: "",
        block_confirmation: "",
      });
    },
  });

  const error = mutation.error ? formatReviewError(mutation.error) : null;
  const successData = mutation.data?.data;

  return (
    <Card className="border-[var(--warning-ring)] bg-[linear-gradient(160deg,rgba(255,255,255,0.92),rgba(252,237,203,0.68))]">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <CardTitle>审核决策</CardTitle>
          <CardDescription className="mt-2">
            目标：{targetTitle ?? targetId ?? "未选择"}。提交前会锁定按钮并透传
            `X-Idempotency-Key`。
          </CardDescription>
        </div>
        <Badge className="bg-[var(--warning-soft)] text-[var(--warning-ink)]">
          {reviewAuditActions[kind]}
        </Badge>
      </div>

      {!canWrite ? (
        <div className="mt-5 rounded-[24px] border border-[var(--warning-ring)] bg-[var(--warning-soft)] p-4 text-sm text-[var(--warning-ink)]">
          当前角色只能查看或尚未具备审核权限，主按钮已拦截。需要
          `platform_reviewer` 或 `platform_admin`。
        </div>
      ) : null}

      <form
        className="mt-5 grid gap-4"
        onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
      >
        <div className="grid gap-4 md:grid-cols-2">
          <label className="grid gap-2 text-sm font-medium text-[var(--ink-strong)]">
            决策动作
            <select
              {...form.register("action")}
              className="h-11 rounded-2xl border border-black/10 bg-white/90 px-4 text-sm outline-none focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]"
            >
              <option value="approve">{decisionLabels.approve}</option>
              <option value="reject">{decisionLabels.reject}</option>
              {kind === "compliance" ? (
                <option value="block">{decisionLabels.block}</option>
              ) : null}
            </select>
          </label>
          <label className="grid gap-2 text-sm font-medium text-[var(--ink-strong)]">
            X-Idempotency-Key
            <div className="flex gap-2">
              <Input {...form.register("idempotency_key")} />
              <Button
                type="button"
                variant="secondary"
                onClick={() =>
                  form.setValue("idempotency_key", createReviewIdempotencyKey(kind))
                }
              >
                <RefreshCcw className="size-4" />
              </Button>
            </div>
            <FieldError message={form.formState.errors.idempotency_key?.message} />
          </label>
        </div>

        <label className="grid gap-2 text-sm font-medium text-[var(--ink-strong)]">
          审核原因
          <Textarea
            {...form.register("action_reason")}
            placeholder="写明准入依据、驳回原因、合规风险或需补充材料。"
          />
          <FieldError message={form.formState.errors.action_reason?.message} />
        </label>

        {kind === "compliance" && selectedAction === "block" ? (
          <div className="grid gap-4 rounded-[24px] border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-4 md:grid-cols-3">
            <label className="grid gap-2 text-sm font-medium text-[var(--danger-ink)]">
              人工确认
              <Input {...form.register("block_confirmation")} placeholder="输入 BLOCK" />
              <FieldError
                message={form.formState.errors.block_confirmation?.message}
              />
            </label>
            <label className="grid gap-2 text-sm font-medium text-[var(--danger-ink)]">
              X-Step-Up-Token
              <Input {...form.register("step_up_token")} placeholder="可选，若 IAM 已下发则透传" />
            </label>
            <label className="grid gap-2 text-sm font-medium text-[var(--danger-ink)]">
              step-up challenge
              <Input
                {...form.register("step_up_challenge_id")}
                placeholder="可选 challenge_id"
              />
            </label>
          </div>
        ) : null}

        {error ? (
          <div className="rounded-[24px] border border-[var(--danger-ring)] bg-[var(--danger-soft)] p-4 text-sm text-[var(--danger-ink)]">
            <div className="font-semibold">{error.title}</div>
            <div className="mt-1">{error.message}</div>
            <div className="mt-2 font-mono text-xs">request_id: {error.requestId}</div>
          </div>
        ) : null}

        {successData ? (
          <div className="rounded-[24px] border border-emerald-200 bg-emerald-50 p-4 text-sm text-emerald-800">
            <div className="flex items-center gap-2 font-semibold">
              <CheckCircle2 className="size-4" />
              决策已写入
            </div>
            <div className="mt-2 font-mono text-xs">
              review_task_id: {"review_task_id" in successData ? successData.review_task_id : "n/a"} / status: {successData.status}
            </div>
          </div>
        ) : null}

        <div className="flex flex-wrap items-center gap-3">
          <Button
            type="submit"
            disabled={!targetId || !canWrite || mutation.isPending}
            variant={selectedAction === "block" ? "warning" : "default"}
          >
            {mutation.isPending ? (
              <LoaderCircle className="size-4 animate-spin" />
            ) : selectedAction === "approve" ? (
              <CheckCircle2 className="size-4" />
            ) : (
              <XCircle className="size-4" />
            )}
            {mutation.isPending ? "提交中" : decisionLabels[selectedAction]}
          </Button>
          <div className="text-xs leading-5 text-[var(--ink-subtle)]">
            写入后端：POST{" "}
            {kind === "subjects"
              ? "/api/v1/review/subjects/{id}"
              : kind === "products"
                ? "/api/v1/review/products/{id}"
                : "/api/v1/review/compliance/{id}"}
          </div>
        </div>
      </form>
    </Card>
  );
}

function SubjectDetailCard({
  detail,
  isLoading,
  error,
  onRetry,
}: {
  detail?: OrganizationReviewRow;
  isLoading: boolean;
  error: unknown;
  onRetry: () => void;
}) {
  if (isLoading) {
    return <LoadingPanel title="正在读取主体详情" compact />;
  }
  if (error) {
    return <ErrorPanel error={error} onRetry={onRetry} compact />;
  }
  if (!detail) {
    return <EmptyPanel title="请选择主体审核项" compact />;
  }

  return (
    <Card>
      <DetailHeader
        eyebrow="Subject Review"
        title={detail.org_name}
        subtitle={detail.org_id}
        status={detail.org_status}
      />
      <InfoGrid
        items={[
          ["主体类型", detail.org_type],
          ["准入状态", labelReviewStatus(detail.org_status)],
          ["审核状态", labelReviewStatus(detail.review_status)],
          ["风险状态", labelReviewStatus(detail.risk_status)],
          ["可售状态", labelReviewStatus(detail.sellable_status)],
          ["司法辖区", detail.jurisdiction_code ?? "未声明"],
          ["合规等级", detail.compliance_level ?? "未声明"],
          ["认证等级", detail.certification_level ?? "未声明"],
          ["身份绑定状态", detail.certification_level ? "已提交认证材料" : "待补充认证"],
          ["黑名单命中", detail.blacklist_active ? "active" : "none"],
          ["冻结原因", detail.freeze_reason ?? "无"],
          ["更新时间", formatDateTime(detail.updated_at)],
        ]}
      />
      <ReferenceList title="主体名单引用" values={[
        `whitelist: ${detail.whitelist_refs.join(" / ") || "无"}`,
        `graylist: ${detail.graylist_refs.join(" / ") || "无"}`,
        `blacklist: ${detail.blacklist_refs.join(" / ") || "无"}`,
      ]} />
    </Card>
  );
}

function ProductDetailCard({
  kind,
  detail,
  fallback,
  isLoading,
  error,
  onRetry,
}: {
  kind: "products" | "compliance";
  detail?: ProductReviewDetail;
  fallback?: ProductReviewRow;
  isLoading: boolean;
  error: unknown;
  onRetry: () => void;
}) {
  if (isLoading) {
    return <LoadingPanel title="正在读取商品详情" compact />;
  }
  if (error) {
    return <ErrorPanel error={error} onRetry={onRetry} compact />;
  }
  if (!detail && !fallback) {
    return <EmptyPanel title="请选择商品审核项" compact />;
  }

  const base = detail ?? fallback;
  const signals = deriveComplianceSignals(detail);
  const skuCoverage = countStandardSkuCoverage(detail);

  return (
    <Card>
      <DetailHeader
        eyebrow={kind === "products" ? "Product Review" : "Compliance Review"}
        title={base?.title ?? "未选择商品"}
        subtitle={base?.product_id ?? "n/a"}
        status={base?.status}
      />
      <InfoGrid
        items={[
          ["卖方主体", base?.seller_org_id ?? "未返回"],
          ["目录分类", base?.category ?? "未返回"],
          ["商品类型", base?.product_type ?? "未返回"],
          ["交付方式", base?.delivery_type ?? "未返回"],
          ["价格", `${base?.price ?? "0"} ${base?.currency_code ?? ""}`],
          ["分类分级", detail?.data_classification ?? "未声明"],
          ["质量评分", detail?.quality_score ?? "未声明"],
          ["索引状态", detail?.index_sync_status ?? "未读取"],
          ["更新时间", formatDateTime(base?.updated_at)],
        ]}
      />

      {detail ? (
        <div className="mt-5 grid gap-4">
          <ReferenceList
            title="SKU 真值覆盖"
            values={skuCoverage.map((item) =>
              `${item.skuType}: ${item.present ? "present" : "missing"}`,
            )}
          />
          <ReferenceList
            title="商品审核重点"
            values={[
              `Hash/版本：asset=${detail.asset_id} / version=${detail.asset_version_id}`,
              `用途：${detail.allowed_usage.join(" / ") || "未声明"}`,
              `场景：${detail.use_cases.join(" / ") || "未声明"}`,
              `元数据字段数：${Object.keys(detail.metadata).length}`,
            ]}
          />
          {kind === "compliance" ? (
            <ReferenceList
              title="合规信号"
              values={[
                `分类分级：${signals.dataClassification}`,
                `使用目的：${signals.declaredPurpose}`,
                `地域：${signals.region}`,
                `导出限制：${signals.exportPolicy}`,
                `自动阻断结果：${signals.automaticBlockResult}`,
                ...(signals.highRiskSignals.length
                  ? signals.highRiskSignals
                  : ["未命中高风险标签"]),
              ]}
              danger={signals.highRiskSignals.length > 0}
            />
          ) : null}
        </div>
      ) : (
        <div className="mt-5 rounded-[24px] bg-black/[0.04] p-4 text-sm text-[var(--ink-soft)]">
          列表已返回基础商品数据，详情 API 尚未完成读取或当前未选中商品。
        </div>
      )}
    </Card>
  );
}

function ReviewVirtualTable<TData>({
  rows,
  columns,
  getRowId,
  selectedId,
  onSelectRow,
}: {
  rows: TData[];
  columns: Array<ColumnDef<TData>>;
  getRowId: (row: TData) => string;
  selectedId: string | null;
  onSelectRow: (id: string) => void;
}) {
  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable({
    data: rows,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getRowId,
  });
  const tableRows = table.getRowModel().rows;
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: tableRows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 72,
    overscan: 8,
  });

  return (
    <div className="mt-4 overflow-hidden rounded-[24px] border border-black/10 bg-white/70">
      <div className="hidden grid-cols-[1.4fr_0.75fr_0.8fr_0.8fr] gap-3 border-b border-black/10 bg-black/[0.03] px-4 py-3 text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)] md:grid">
        {table.getHeaderGroups()[0]?.headers.map((header) => (
          <div key={header.id}>
            {flexRender(header.column.columnDef.header, header.getContext())}
          </div>
        ))}
      </div>
      <div ref={parentRef} className="max-h-[520px] overflow-auto">
        <div
          className="relative"
          style={{ height: `${virtualizer.getTotalSize()}px` }}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const row = tableRows[virtualRow.index];
            if (!row) {
              return null;
            }
            const id = getRowId(row.original);
            const selected = id === selectedId;

            return (
              <button
                key={row.id}
                type="button"
                onClick={() => onSelectRow(id)}
                className={cn(
                  "absolute left-0 top-0 w-full border-b border-black/5 px-4 py-3 text-left transition",
                  "grid gap-2 md:grid-cols-[1.4fr_0.75fr_0.8fr_0.8fr] md:items-center",
                  selected
                    ? "bg-[var(--accent-soft)]"
                    : "bg-white/55 hover:bg-black/[0.035]",
                )}
                style={{ transform: `translateY(${virtualRow.start}px)` }}
              >
                {row.getVisibleCells().map((cell) => (
                  <div key={cell.id} className="min-w-0 text-sm text-[var(--ink-soft)]">
                    <div className="md:hidden text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
                      {String(cell.column.columnDef.header ?? "")}
                    </div>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </div>
                ))}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function WorkbenchGrid({ list, detail }: { list: ReactNode; detail: ReactNode }) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28 }}
      className="grid gap-4 xl:grid-cols-[1.05fr_0.95fr]"
    >
      {list}
      {detail}
    </motion.section>
  );
}

function WorkbenchToolbar({
  title,
  description,
  children,
  onRefresh,
}: {
  title: string;
  description: string;
  children: ReactNode;
  onRefresh: () => void;
}) {
  return (
    <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
      <div>
        <CardTitle>{title}</CardTitle>
        <CardDescription className="mt-2">{description}</CardDescription>
      </div>
      <div className="flex flex-col gap-3 md:flex-row md:items-end">
        {children}
        <Button type="button" variant="secondary" onClick={onRefresh}>
          <RefreshCcw className="size-4" />
          刷新
        </Button>
      </div>
    </div>
  );
}

function QueryListState<TData>({
  query,
  rows,
  emptyTitle,
  onRetry,
  children,
}: {
  query: { isPending: boolean; isError: boolean; error: unknown };
  rows: TData[];
  emptyTitle: string;
  onRetry: () => void;
  children: ReactNode;
}) {
  if (query.isPending) {
    return <LoadingPanel title="正在读取审核队列" compact />;
  }
  if (query.isError) {
    return <ErrorPanel error={query.error} onRetry={onRetry} compact />;
  }
  if (!rows.length) {
    return <EmptyPanel title={emptyTitle} compact />;
  }
  return children;
}

function LoadingPanel({ title, compact = false }: { title: string; compact?: boolean }) {
  return (
    <Card className={cn("flex items-center justify-center bg-[var(--panel-muted)]", compact ? "min-h-40" : "min-h-72")}>
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <LoaderCircle className="size-8 animate-spin" />
        <CardTitle>{title}</CardTitle>
        <CardDescription>正在通过受控 API 代理向 platform-core 读取正式数据。</CardDescription>
      </div>
    </Card>
  );
}

function EmptyPanel({ title, compact = false }: { title: string; compact?: boolean }) {
  return (
    <Card className={cn("flex items-center justify-center bg-[var(--panel-muted)]", compact ? "min-h-40" : "min-h-72")}>
      <div className="flex flex-col items-center gap-3 text-center text-[var(--ink-soft)]">
        <ShieldCheck className="size-8" />
        <CardTitle>{title}</CardTitle>
        <CardDescription>空态来自真实 API 返回，不以前端 mock 伪造待审数据。</CardDescription>
      </div>
    </Card>
  );
}

function ErrorPanel({
  error,
  onRetry,
  compact = false,
}: {
  error: unknown;
  onRetry: () => void;
  compact?: boolean;
}) {
  const detail = formatReviewError(error);
  return (
    <Card className={cn("border-[var(--danger-ring)] bg-[var(--danger-soft)]", compact ? "min-h-40" : "min-h-72")}>
      <div className="flex h-full flex-col justify-center gap-4 text-[var(--danger-ink)]">
        <div className="flex items-center gap-2">
          <AlertTriangle className="size-5" />
          <CardTitle className="text-[var(--danger-ink)]">{detail.title}</CardTitle>
        </div>
        <CardDescription className="text-[var(--danger-ink)]">
          {detail.message}
        </CardDescription>
        <div className="font-mono text-xs">request_id: {detail.requestId}</div>
        <div>
          <Button type="button" variant="warning" onClick={onRetry}>
            <RefreshCcw className="size-4" />
            重试
          </Button>
        </div>
      </div>
    </Card>
  );
}

function ForbiddenPanel({ subject }: { subject?: SessionSubject }) {
  return (
    <Card className="border-[var(--warning-ring)] bg-[var(--warning-soft)]">
      <div className="flex flex-col gap-4 text-[var(--warning-ink)] md:flex-row md:items-center md:justify-between">
        <div className="flex items-start gap-3">
          <ShieldAlert className="mt-1 size-6" />
          <div>
            <CardTitle className="text-[var(--warning-ink)]">审核工作台权限不足</CardTitle>
            <CardDescription className="mt-2 text-[var(--warning-ink)]">
              当前角色：{subject?.roles.join(" / ") || "visitor"}。需要
              `platform_reviewer` 或 `platform_admin` 才能查看审核队列和执行主按钮。
            </CardDescription>
          </div>
        </div>
        <Badge className="bg-white/60 text-[var(--warning-ink)]">
          forbidden state
        </Badge>
      </div>
    </Card>
  );
}

function DetailHeader({
  eyebrow,
  title,
  subtitle,
  status,
}: {
  eyebrow: string;
  title: string;
  subtitle: string;
  status?: string | null;
}) {
  return (
    <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
      <div>
        <div className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--ink-subtle)]">
          {eyebrow}
        </div>
        <CardTitle className="mt-1">{title}</CardTitle>
        <div className="mt-2 font-mono text-xs text-[var(--ink-subtle)]">{subtitle}</div>
      </div>
      <StatusPill value={status ?? "unknown"} label={labelReviewStatus(status)} />
    </div>
  );
}

function InfoGrid({ items }: { items: Array<[string, string]> }) {
  return (
    <div className="mt-5 grid gap-3 md:grid-cols-2">
      {items.map(([label, value]) => (
        <div key={label} className="rounded-[20px] bg-black/[0.035] p-3">
          <div className="text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--ink-subtle)]">
            {label}
          </div>
          <div className="mt-1 break-words text-sm font-medium text-[var(--ink-strong)]">
            {value}
          </div>
        </div>
      ))}
    </div>
  );
}

function ReferenceList({
  title,
  values,
  danger = false,
}: {
  title: string;
  values: string[];
  danger?: boolean;
}) {
  return (
    <div
      className={cn(
        "rounded-[24px] p-4",
        danger
          ? "border border-[var(--danger-ring)] bg-[var(--danger-soft)] text-[var(--danger-ink)]"
          : "bg-black/[0.04] text-[var(--ink-soft)]",
      )}
    >
      <div className="text-xs font-semibold uppercase tracking-[0.18em]">{title}</div>
      <div className="mt-3 flex flex-wrap gap-2">
        {values.map((value) => (
          <span
            key={value}
            className="rounded-full bg-white/70 px-3 py-1 text-xs font-medium"
          >
            {value}
          </span>
        ))}
      </div>
    </div>
  );
}

function HeroMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[24px] bg-white/12 p-4">
      <div className="text-xs uppercase tracking-[0.18em] text-white/60">{label}</div>
      <div className="mt-2 break-words text-sm font-semibold text-white">{value}</div>
    </div>
  );
}

function BoundaryRow({
  icon,
  label,
  value,
}: {
  icon: ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-start gap-3 rounded-[22px] bg-black/[0.04] p-3">
      <div className="mt-0.5 text-[var(--accent-strong)]">{icon}</div>
      <div>
        <div className="text-sm font-semibold text-[var(--ink-strong)]">{label}</div>
        <div className="mt-1 text-sm leading-6 text-[var(--ink-soft)]">{value}</div>
      </div>
    </div>
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
    <label className="grid gap-1 text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
      {label}
      <select
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="h-11 rounded-2xl border border-black/10 bg-white/90 px-4 text-sm normal-case tracking-normal text-[var(--ink-strong)] outline-none focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)]"
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

function StatusPill({ value, label }: { value: string; label: string }) {
  const hot = ["pending_review", "manual_review", "watch"].includes(value);
  const danger = ["rejected", "frozen", "restricted"].includes(value);
  return (
    <span
      className={cn(
        "inline-flex w-fit rounded-full px-3 py-1 text-xs font-semibold",
        danger
          ? "bg-[var(--danger-soft)] text-[var(--danger-ink)]"
          : hot
            ? "bg-[var(--warning-soft)] text-[var(--warning-ink)]"
            : "bg-emerald-50 text-emerald-800",
      )}
    >
      {label}
    </span>
  );
}

function FieldError({ message }: { message?: string }) {
  if (!message) {
    return null;
  }
  return <div className="text-sm text-[var(--danger-ink)]">{message}</div>;
}

function subjectLabel(subject?: SessionSubject) {
  return subject?.display_name ?? subject?.login_id ?? subject?.user_id ?? "游客";
}
