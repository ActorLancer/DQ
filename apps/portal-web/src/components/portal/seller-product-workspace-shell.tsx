"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { PlatformApiError } from "@datab/sdk-ts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  BadgeCheck,
  Ban,
  Boxes,
  ClipboardCheck,
  DatabaseZap,
  FilePenLine,
  FileText,
  Fingerprint,
  Layers3,
  LoaderCircle,
  LockKeyhole,
  PackagePlus,
  Send,
  ShieldCheck,
  Tags,
} from "lucide-react";
import { motion } from "motion/react";
import type { Route } from "next";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useEffect, useState, type ReactNode } from "react";
import { useForm, useWatch } from "react-hook-form";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { createBrowserSdk } from "@/lib/platform-sdk";
import { portalRouteMap } from "@/lib/portal-routes";
import {
  buildCreateProductRequest,
  buildMetadataProfileRequest,
  buildPatchProductRequest,
  buildQualityReportRequest,
  buildSkuRequest,
  buildSubmitProductRequest,
  buildTemplateBindRequest,
  canSubmitProduct,
  createCatalogIdempotencyKey,
  defaultCreateProductValues,
  defaultMetadataValues,
  defaultQualityReportValues,
  defaultSkuValues,
  defaultSubmitProductValues,
  defaultTemplateBindValues,
  getSkuOption,
  isSellerProductRole,
  metadataProfileSchema,
  patchValuesFromProduct,
  productCreateSchema,
  productPatchSchema,
  productStatusLabel,
  productStatusOptions,
  qualityReportSchema,
  skuCreateSchema,
  standardSkuOptions,
  submitProductSchema,
  templateBindSchema,
  type MetadataProfileFormValues,
  type ProductCreateFormValues,
  type ProductPatchFormValues,
  type QualityReportFormValues,
  type SellerProductDetail,
  type SellerProductListItem,
  type SkuCreateFormValues,
  type SubmitProductFormValues,
  type TemplateBindFormValues,
} from "@/lib/seller-products-view";
import type { PortalSessionPreview } from "@/lib/session";
import { cn, formatList } from "@/lib/utils";

import {
  PreviewStateControls,
  ScaffoldPill,
  getPreviewState,
  type PreviewState,
} from "./state-preview";

const sdk = createBrowserSdk();
const centerMeta = portalRouteMap.seller_product_center;

type WorkspaceSection = "center" | "edit" | "skus" | "templates" | "metadata";

type ActionState = {
  tone: "idle" | "success" | "error";
  message: string;
  idempotencyKey?: string;
};

type SellerProductWorkspaceShellProps = {
  initialSection: WorkspaceSection;
  productId?: string;
  sessionMode: "guest" | "bearer" | "local";
  initialSubject: PortalSessionPreview | null;
};

export function SellerProductWorkspaceShell({
  initialSection,
  productId,
  sessionMode,
  initialSubject,
}: SellerProductWorkspaceShellProps) {
  const queryClient = useQueryClient();
  const searchParams = useSearchParams();
  const preview = getPreviewState(searchParams);
  const roles = initialSubject?.roles ?? [];
  const canOperate =
    sessionMode === "bearer" &&
    Boolean(initialSubject) &&
    isSellerProductRole(roles);
  const isPlatformOperator = roles.includes("platform_admin");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [keyword, setKeyword] = useState("");
  const [selectedProductId, setSelectedProductId] = useState<string | undefined>(
    productId,
  );
  const [actionState, setActionState] = useState<ActionState>({
    tone: "idle",
    message: "等待卖方上架操作。",
  });
  const listStatusFilter = productStatusOptions.includes(
    statusFilter as (typeof productStatusOptions)[number],
  )
    ? (statusFilter as (typeof productStatusOptions)[number])
    : undefined;

  const productListQuery = useQuery({
    queryKey: [
      "portal",
      "seller-products",
      initialSubject?.tenant_id,
      statusFilter,
      keyword,
    ],
    enabled: canOperate && preview === "ready",
    queryFn: () =>
      sdk.catalog.listProducts({
        seller_org_id: isPlatformOperator
          ? undefined
          : initialSubject?.tenant_id ?? initialSubject?.org_id,
        status: listStatusFilter,
        q: keyword || undefined,
        page: 1,
        page_size: 20,
      }),
  });

  const productDetailQuery = useQuery({
    queryKey: ["portal", "seller-product-detail", selectedProductId],
    enabled: canOperate && preview === "ready" && Boolean(selectedProductId),
    queryFn: () => sdk.catalog.getProductDetail({ id: selectedProductId ?? "" }),
  });
  const product = productDetailQuery.data?.data;

  const createForm = useForm<ProductCreateFormValues>({
    resolver: zodResolver(productCreateSchema),
    defaultValues: defaultCreateProductValues(
      initialSubject?.tenant_id ?? initialSubject?.org_id,
    ),
  });
  const patchForm = useForm<ProductPatchFormValues>({
    resolver: zodResolver(productPatchSchema),
    defaultValues: patchValuesFromProduct(product),
  });
  const skuForm = useForm<SkuCreateFormValues>({
    resolver: zodResolver(skuCreateSchema),
    defaultValues: defaultSkuValues(),
  });
  const metadataForm = useForm<MetadataProfileFormValues>({
    resolver: zodResolver(metadataProfileSchema),
    defaultValues: defaultMetadataValues(),
  });
  const qualityForm = useForm<QualityReportFormValues>({
    resolver: zodResolver(qualityReportSchema),
    defaultValues: defaultQualityReportValues(),
  });
  const templateForm = useForm<TemplateBindFormValues>({
    resolver: zodResolver(templateBindSchema),
    defaultValues: defaultTemplateBindValues(),
  });
  const submitForm = useForm<SubmitProductFormValues>({
    resolver: zodResolver(submitProductSchema),
    defaultValues: defaultSubmitProductValues(),
  });

  const selectedSkuType = useWatch({
    control: skuForm.control,
    name: "sku_type",
  });
  const selectedTemplateScope = useWatch({
    control: templateForm.control,
    name: "target_scope",
  });

  useEffect(() => {
    patchForm.reset(patchValuesFromProduct(product));
  }, [patchForm, product]);

  useEffect(() => {
    const option = getSkuOption(selectedSkuType);
    if (!option) {
      return;
    }
    skuForm.setValue("billing_mode", option.billing_mode);
    skuForm.setValue("trade_mode", option.trade_mode);
    skuForm.setValue("unit_name", option.unit_name);
    skuForm.setValue("delivery_object_kind", option.delivery_object_kind);
  }, [selectedSkuType, skuForm]);

  const refreshSellerProductQueries = async (nextProductId?: string) => {
    await queryClient.invalidateQueries({ queryKey: ["portal", "seller-products"] });
    await queryClient.invalidateQueries({
      queryKey: ["portal", "seller-product-detail", nextProductId ?? selectedProductId],
    });
  };

  const createProductMutation = useMutation({
    mutationFn: async (values: ProductCreateFormValues) => {
      const idempotencyKey = createCatalogIdempotencyKey("product-create");
      const response = await sdk.catalog.createProductDraft(
        buildCreateProductRequest(values),
        { idempotencyKey },
      );
      return { response, idempotencyKey };
    },
    onSuccess: async ({ response, idempotencyKey }) => {
      setSelectedProductId(response.data.product_id);
      setActionState({
        tone: "success",
        message: `商品草稿已创建：${response.data.product_id}`,
        idempotencyKey,
      });
      await refreshSellerProductQueries(response.data.product_id);
    },
    onError: (error) => setActionState(toErrorState(error)),
  });

  const patchProductMutation = useMutation({
    mutationFn: async (values: ProductPatchFormValues) => {
      if (!selectedProductId) {
        throw new Error("缺少 product_id，无法保存草稿。");
      }
      const idempotencyKey = createCatalogIdempotencyKey("product-patch");
      const response = await sdk.catalog.patchProductDraft(
        { id: selectedProductId },
        buildPatchProductRequest(values),
        { idempotencyKey },
      );
      return { response, idempotencyKey };
    },
    onSuccess: async ({ response, idempotencyKey }) => {
      setActionState({
        tone: "success",
        message: `商品草稿已保存：${response.data.product_id}`,
        idempotencyKey,
      });
      await refreshSellerProductQueries(response.data.product_id);
    },
    onError: (error) => setActionState(toErrorState(error)),
  });

  const createSkuMutation = useMutation({
    mutationFn: async (values: SkuCreateFormValues) => {
      if (!selectedProductId) {
        throw new Error("缺少 product_id，无法创建 SKU。");
      }
      const idempotencyKey = createCatalogIdempotencyKey("sku-create");
      const response = await sdk.catalog.createProductSku(
        { id: selectedProductId },
        buildSkuRequest(values),
        { idempotencyKey },
      );
      return { response, idempotencyKey };
    },
    onSuccess: async ({ response, idempotencyKey }) => {
      setActionState({
        tone: "success",
        message: `SKU 已创建：${response.data.sku_id}`,
        idempotencyKey,
      });
      await refreshSellerProductQueries(response.data.product_id);
    },
    onError: (error) => setActionState(toErrorState(error)),
  });

  const metadataMutation = useMutation({
    mutationFn: async (values: MetadataProfileFormValues) => {
      if (!selectedProductId) {
        throw new Error("缺少 product_id，无法保存元信息。");
      }
      const idempotencyKey = createCatalogIdempotencyKey("metadata-profile");
      const response = await sdk.catalog.putProductMetadataProfile(
        { id: selectedProductId },
        buildMetadataProfileRequest(values),
        { idempotencyKey },
      );
      return { response, idempotencyKey };
    },
    onSuccess: async ({ response, idempotencyKey }) => {
      setActionState({
        tone: "success",
        message: `元信息版本已保存：${response.data.product_metadata_profile_id}`,
        idempotencyKey,
      });
      await refreshSellerProductQueries(response.data.product_id);
    },
    onError: (error) => setActionState(toErrorState(error)),
  });

  const qualityMutation = useMutation({
    mutationFn: async (values: QualityReportFormValues) => {
      if (!product) {
        throw new Error("缺少商品详情，无法定位 asset_version_id。");
      }
      const idempotencyKey = createCatalogIdempotencyKey("quality-report");
      const response = await sdk.catalog.createAssetQualityReport(
        { versionId: product.asset_version_id },
        buildQualityReportRequest(values),
        { idempotencyKey },
      );
      return { response, idempotencyKey };
    },
    onSuccess: async ({ response, idempotencyKey }) => {
      setActionState({
        tone: "success",
        message: `质量报告已登记：${response.data.quality_report_id}`,
        idempotencyKey,
      });
      await refreshSellerProductQueries(selectedProductId);
    },
    onError: (error) => setActionState(toErrorState(error)),
  });

  const templateMutation = useMutation({
    mutationFn: async (values: TemplateBindFormValues) => {
      if (!selectedProductId) {
        throw new Error("缺少 product_id，无法绑定模板。");
      }
      const idempotencyKey = createCatalogIdempotencyKey("template-bind");
      const body = buildTemplateBindRequest(values);
      const response =
        values.target_scope === "product"
          ? await sdk.catalog.bindProductTemplate(
              { id: selectedProductId },
              body,
              { idempotencyKey },
            )
          : await sdk.catalog.bindSkuTemplate(
              { id: values.sku_id ?? "" },
              body,
              { idempotencyKey },
            );
      return { response, idempotencyKey };
    },
    onSuccess: async ({ response, idempotencyKey }) => {
      setActionState({
        tone: "success",
        message: `模板已绑定到 ${response.data.binding_scope}:${response.data.target_id}`,
        idempotencyKey,
      });
      await refreshSellerProductQueries(selectedProductId);
    },
    onError: (error) => setActionState(toErrorState(error)),
  });

  const submitMutation = useMutation({
    mutationFn: async (values: SubmitProductFormValues) => {
      if (!selectedProductId) {
        throw new Error("缺少 product_id，无法提交审核。");
      }
      const idempotencyKey = createCatalogIdempotencyKey("product-submit");
      const response = await sdk.catalog.submitProduct(
        { id: selectedProductId },
        buildSubmitProductRequest(values),
        { idempotencyKey },
      );
      return { response, idempotencyKey };
    },
    onSuccess: async ({ response, idempotencyKey }) => {
      setActionState({
        tone: "success",
        message: `商品已提交审核：${response.data.review_task_id}`,
        idempotencyKey,
      });
      await refreshSellerProductQueries(response.data.product_id);
    },
    onError: (error) => setActionState(toErrorState(error)),
  });

  const listData = productListQuery.data?.data;
  const products = listData?.items ?? [];
  const selectedProduct =
    product ?? products.find((item) => item.product_id === selectedProductId);

  if (preview === "loading") {
    return (
      <SellerProductPageFrame
        preview={preview}
        sessionMode={sessionMode}
        subject={initialSubject}
        canOperate={canOperate}
        selectedProductId={selectedProductId}
      >
        <StateCard
          icon={<LoaderCircle className="size-8 animate-spin" />}
          title="上架中心加载态"
          message="正在读取商品草稿、SKU、元信息与模板绑定入口。"
        />
      </SellerProductPageFrame>
    );
  }

  if (preview === "error") {
    return (
      <SellerProductPageFrame
        preview={preview}
        sessionMode={sessionMode}
        subject={initialSubject}
        canOperate={canOperate}
        selectedProductId={selectedProductId}
      >
        <StateCard
          tone="danger"
          icon={<AlertTriangle className="size-8" />}
          title="上架中心错误态"
          message="CAT_VALIDATION_FAILED: 页面必须承接后端统一错误码、request_id 与重试入口。"
        />
      </SellerProductPageFrame>
    );
  }

  if (preview === "empty") {
    return (
      <SellerProductPageFrame
        preview={preview}
        sessionMode={sessionMode}
        subject={initialSubject}
        canOperate={canOperate}
        selectedProductId={selectedProductId}
      >
        <StateCard
          icon={<Boxes className="size-8" />}
          title="没有商品草稿"
          message="空态不使用 mock 商品；请通过新建商品表单调用 POST /api/v1/products 创建真实草稿。"
        />
      </SellerProductPageFrame>
    );
  }

  if (preview === "forbidden" || !canOperate) {
    return (
      <SellerProductPageFrame
        preview={preview}
        sessionMode={sessionMode}
        subject={initialSubject}
        canOperate={canOperate}
        selectedProductId={selectedProductId}
      >
        <StateCard
          tone="warning"
          icon={<Ban className="size-8" />}
          title="上架中心权限态"
          message={`需要权限：catalog.product.list / catalog.product.create / catalog.product.update / catalog.sku.create；当前会话 ${sessionMode}，角色 ${formatList(roles)}。`}
        />
      </SellerProductPageFrame>
    );
  }

  return (
    <SellerProductPageFrame
      preview={preview}
      sessionMode={sessionMode}
      subject={initialSubject}
      canOperate={canOperate}
      selectedProductId={selectedProductId}
    >
      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_360px]">
        <div className="space-y-5">
          <SectionTabs section={initialSection} selectedProductId={selectedProductId} />
          {productListQuery.isPending && initialSection === "center" ? (
            <StateCard
              icon={<LoaderCircle className="size-8 animate-spin" />}
              title="正在读取商品列表"
              message="GET /api/v1/products 正在返回卖方商品草稿与状态统计。"
            />
          ) : productListQuery.isError && initialSection === "center" ? (
            <StateCard
              tone="danger"
              icon={<AlertTriangle className="size-8" />}
              title="商品列表读取失败"
              message={describeError(productListQuery.error)}
            />
          ) : (
            <WorkspaceSectionContent
              section={initialSection}
              products={products}
              product={product}
              selectedProduct={selectedProduct}
              selectedProductId={selectedProductId}
              statusFilter={statusFilter}
              keyword={keyword}
              total={listData?.total ?? 0}
              statusCounts={listData?.status_counts ?? []}
              productPending={productDetailQuery.isPending}
              productError={productDetailQuery.error}
              actionState={actionState}
              createForm={createForm}
              patchForm={patchForm}
              skuForm={skuForm}
              metadataForm={metadataForm}
              qualityForm={qualityForm}
              templateForm={templateForm}
              submitForm={submitForm}
              selectedTemplateScope={selectedTemplateScope}
              onStatusFilterChange={setStatusFilter}
              onKeywordChange={setKeyword}
              onSelectProduct={setSelectedProductId}
              onCreateProduct={createForm.handleSubmit((values) =>
                createProductMutation.mutate(values),
              )}
              onPatchProduct={patchForm.handleSubmit((values) =>
                patchProductMutation.mutate(values),
              )}
              onCreateSku={skuForm.handleSubmit((values) =>
                createSkuMutation.mutate(values),
              )}
              onSaveMetadata={metadataForm.handleSubmit((values) =>
                metadataMutation.mutate(values),
              )}
              onCreateQuality={qualityForm.handleSubmit((values) =>
                qualityMutation.mutate(values),
              )}
              onBindTemplate={templateForm.handleSubmit((values) =>
                templateMutation.mutate(values),
              )}
              onSubmitProduct={submitForm.handleSubmit((values) =>
                submitMutation.mutate(values),
              )}
              isMutating={
                createProductMutation.isPending ||
                patchProductMutation.isPending ||
                createSkuMutation.isPending ||
                metadataMutation.isPending ||
                qualityMutation.isPending ||
                templateMutation.isPending ||
                submitMutation.isPending
              }
            />
          )}
        </div>

        <ActionRail
          product={product}
          selectedProductId={selectedProductId}
          actionState={actionState}
        />
      </div>
    </SellerProductPageFrame>
  );
}

function SellerProductPageFrame({
  preview,
  sessionMode,
  subject,
  canOperate,
  selectedProductId,
  children,
}: {
  preview: PreviewState;
  sessionMode: "guest" | "bearer" | "local";
  subject: PortalSessionPreview | null;
  canOperate: boolean;
  selectedProductId?: string;
  children: ReactNode;
}) {
  return (
    <div className="space-y-6">
      <motion.section
        initial={{ opacity: 0, y: 18 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.28 }}
        className="grid gap-4 xl:grid-cols-[1.45fr_0.85fr]"
      >
        <Card className="overflow-hidden bg-[radial-gradient(circle_at_8%_12%,rgba(233,151,70,0.20),transparent_26%),linear-gradient(135deg,rgba(255,255,255,0.98),rgba(236,243,231,0.9),rgba(244,238,224,0.72))]">
          <div className="space-y-5">
            <div className="flex flex-wrap gap-2">
              <ScaffoldPill>{centerMeta.group}</ScaffoldPill>
              <ScaffoldPill>{centerMeta.key}</ScaffoldPill>
              <ScaffoldPill tone="warning">preview:{preview}</ScaffoldPill>
              <ScaffoldPill>{canOperate ? "Bearer API ready" : "requires seller role"}</ScaffoldPill>
            </div>
            <div className="max-w-4xl">
              <Badge>Seller Listing Workspace</Badge>
              <h1 className="mt-3 text-3xl font-semibold tracking-[-0.045em] text-[var(--ink-strong)] md:text-5xl">
                卖方上架中心
              </h1>
              <CardDescription className="mt-4 text-base">
                商品草稿、SKU 真值、元信息、质量报告、模板绑定与提交审核全部通过
                `platform-core` 正式 API 和 `packages/sdk-ts` 调用；浏览器只经过 `/api/platform/**` 受控代理。
              </CardDescription>
            </div>
            <div className="grid gap-3 md:grid-cols-4">
              <HeaderMetric label="product_id" value={selectedProductId ?? "未选择"} />
              <HeaderMetric label="查看权限" value={centerMeta.viewPermission} />
              <HeaderMetric label="主动作权限" value={formatList(centerMeta.primaryPermissions)} />
              <HeaderMetric label="会话模式" value={sessionMode} />
            </div>
            <PreviewStateControls />
          </div>
        </Card>

        <Card>
          <div className="space-y-4">
            <div className="flex items-center gap-3">
              <div className="rounded-2xl bg-[var(--accent-soft)] p-3 text-[var(--accent-strong)]">
                <LockKeyhole className="size-5" />
              </div>
              <div>
                <CardTitle>当前主体访问上下文</CardTitle>
                <CardDescription>敏感页面显式展示主体、角色、租户与作用域。</CardDescription>
              </div>
            </div>
            <div className="grid gap-3">
              <ContextRow label="主体" value={subject?.display_name ?? subject?.login_id ?? subject?.user_id ?? "未建立 Bearer 会话"} />
              <ContextRow label="角色" value={subject?.roles.join(", ") || "无"} />
              <ContextRow label="租户/组织" value={subject?.tenant_id ?? subject?.org_id ?? "无"} />
              <ContextRow label="作用域" value={subject?.auth_context_level ?? "无"} />
            </div>
          </div>
        </Card>
      </motion.section>
      {children}
    </div>
  );
}

function WorkspaceSectionContent({
  section,
  products,
  product,
  selectedProduct,
  selectedProductId,
  statusFilter,
  keyword,
  total,
  statusCounts,
  productPending,
  productError,
  actionState,
  createForm,
  patchForm,
  skuForm,
  metadataForm,
  qualityForm,
  templateForm,
  submitForm,
  selectedTemplateScope,
  onStatusFilterChange,
  onKeywordChange,
  onSelectProduct,
  onCreateProduct,
  onPatchProduct,
  onCreateSku,
  onSaveMetadata,
  onCreateQuality,
  onBindTemplate,
  onSubmitProduct,
  isMutating,
}: {
  section: WorkspaceSection;
  products: SellerProductListItem[];
  product?: SellerProductDetail;
  selectedProduct?: SellerProductDetail | SellerProductListItem;
  selectedProductId?: string;
  statusFilter: string;
  keyword: string;
  total: number;
  statusCounts: { status: string; count: number }[];
  productPending: boolean;
  productError: unknown;
  actionState: ActionState;
  createForm: ReturnType<typeof useForm<ProductCreateFormValues>>;
  patchForm: ReturnType<typeof useForm<ProductPatchFormValues>>;
  skuForm: ReturnType<typeof useForm<SkuCreateFormValues>>;
  metadataForm: ReturnType<typeof useForm<MetadataProfileFormValues>>;
  qualityForm: ReturnType<typeof useForm<QualityReportFormValues>>;
  templateForm: ReturnType<typeof useForm<TemplateBindFormValues>>;
  submitForm: ReturnType<typeof useForm<SubmitProductFormValues>>;
  selectedTemplateScope: "product" | "sku";
  onStatusFilterChange: (value: string) => void;
  onKeywordChange: (value: string) => void;
  onSelectProduct: (value: string) => void;
  onCreateProduct: () => void;
  onPatchProduct: () => void;
  onCreateSku: () => void;
  onSaveMetadata: () => void;
  onCreateQuality: () => void;
  onBindTemplate: () => void;
  onSubmitProduct: () => void;
  isMutating: boolean;
}) {
  if (section === "center") {
    return (
      <div className="grid gap-5 2xl:grid-cols-[minmax(0,1fr)_420px]">
        <div className="space-y-5">
          <ProductListPanel
            products={products}
            total={total}
            statusCounts={statusCounts}
            statusFilter={statusFilter}
            keyword={keyword}
            selectedProductId={selectedProductId}
            onStatusFilterChange={onStatusFilterChange}
            onKeywordChange={onKeywordChange}
            onSelectProduct={onSelectProduct}
          />
          <StandardSkuMatrix />
        </div>
        <CreateProductPanel
          form={createForm}
          onSubmit={onCreateProduct}
          disabled={isMutating}
        />
      </div>
    );
  }

  return (
    <div className="space-y-5">
      <SelectedProductBanner
        product={selectedProduct}
        productPending={productPending}
        productError={productError}
      />
      {section === "edit" ? (
        <EditProductPanel
          form={patchForm}
          product={product}
          onSubmit={onPatchProduct}
          disabled={isMutating || !product}
        />
      ) : null}
      {section === "skus" ? (
        <SkuConfigPanel
          form={skuForm}
          product={product}
          onSubmit={onCreateSku}
          disabled={isMutating || !product}
        />
      ) : null}
      {section === "metadata" ? (
        <MetadataAndQualityPanel
          metadataForm={metadataForm}
          qualityForm={qualityForm}
          product={product}
          onSaveMetadata={onSaveMetadata}
          onCreateQuality={onCreateQuality}
          disabled={isMutating || !product}
        />
      ) : null}
      {section === "templates" ? (
        <TemplateBindPanel
          form={templateForm}
          product={product}
          selectedScope={selectedTemplateScope}
          onSubmit={onBindTemplate}
          disabled={isMutating || !product}
        />
      ) : null}
      <SubmitReviewPanel
        form={submitForm}
        product={product}
        actionState={actionState}
        onSubmit={onSubmitProduct}
        disabled={isMutating || !product}
      />
    </div>
  );
}

function ProductListPanel({
  products,
  total,
  statusCounts,
  statusFilter,
  keyword,
  selectedProductId,
  onStatusFilterChange,
  onKeywordChange,
  onSelectProduct,
}: {
  products: SellerProductListItem[];
  total: number;
  statusCounts: { status: string; count: number }[];
  statusFilter: string;
  keyword: string;
  selectedProductId?: string;
  onStatusFilterChange: (value: string) => void;
  onKeywordChange: (value: string) => void;
  onSelectProduct: (value: string) => void;
}) {
  return (
    <Card className="space-y-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <CardTitle>商品草稿与审核状态</CardTitle>
          <CardDescription className="mt-1">
            列表绑定 `GET /api/v1/products`，支持状态筛选、关键词查询和空态。
          </CardDescription>
        </div>
        <Badge className="bg-black/[0.04] text-[var(--ink-soft)]">total {total}</Badge>
      </div>

      <div className="grid gap-3 md:grid-cols-4">
        <StatusCountCard label="全部" count={total} active={statusFilter === "all"} onClick={() => onStatusFilterChange("all")} />
        {productStatusOptions.slice(0, 3).map((status) => (
          <StatusCountCard
            key={status}
            label={productStatusLabel(status)}
            count={statusCounts.find((item) => item.status === status)?.count ?? 0}
            active={statusFilter === status}
            onClick={() => onStatusFilterChange(status)}
          />
        ))}
      </div>

      <div className="grid gap-3 md:grid-cols-[1fr_220px]">
        <Input
          value={keyword}
          onChange={(event) => onKeywordChange(event.target.value)}
          placeholder="按标题、分类或 product_id 搜索"
        />
        <select
          className={selectClassName}
          value={statusFilter}
          onChange={(event) => onStatusFilterChange(event.target.value)}
        >
          <option value="all">全部状态</option>
          {productStatusOptions.map((status) => (
            <option key={status} value={status}>
              {productStatusLabel(status)}
            </option>
          ))}
        </select>
      </div>

      {products.length ? (
        <div className="overflow-hidden rounded-[24px] border border-black/10 bg-white/75">
          <div className="hidden grid-cols-[1.2fr_0.8fr_0.7fr_0.7fr_0.7fr] gap-3 border-b border-black/10 bg-black/[0.03] px-4 py-3 text-xs font-semibold uppercase tracking-[0.16em] text-[var(--ink-subtle)] md:grid">
            <span>商品</span>
            <span>状态</span>
            <span>价格</span>
            <span>交付</span>
            <span>动作</span>
          </div>
          <div className="max-h-[520px] overflow-auto">
            {products.map((item) => (
              <div
                key={item.product_id}
                className={cn(
                  "grid gap-3 border-b border-black/[0.06] px-4 py-3 text-sm last:border-b-0 md:grid-cols-[1.2fr_0.8fr_0.7fr_0.7fr_0.7fr]",
                  selectedProductId === item.product_id && "bg-[var(--accent-soft)]/60",
                )}
              >
                <div className="min-w-0">
                  <button
                    type="button"
                    onClick={() => onSelectProduct(item.product_id)}
                    className="line-clamp-1 text-left font-semibold text-[var(--ink-strong)] underline-offset-4 hover:underline"
                  >
                    {item.title}
                  </button>
                  <div className="mt-1 truncate text-xs text-[var(--ink-subtle)]">
                    {item.product_id}
                  </div>
                </div>
                <span className="min-w-0">
                  <StatusBadge status={item.status} />
                </span>
                <span className="min-w-0 break-words text-[var(--ink-soft)]">
                  {item.currency_code} {item.price}
                </span>
                <span className="min-w-0 break-words text-[var(--ink-soft)]">{item.delivery_type}</span>
                <div className="flex min-w-0 flex-wrap gap-2">
                  <Button type="button" size="sm" variant="secondary" onClick={() => onSelectProduct(item.product_id)}>
                    选中
                  </Button>
                  <Button asChild size="sm" variant="ghost">
                    <Link href={`/seller/products/${item.product_id}/edit` as Route}>编辑</Link>
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <StateCard
          icon={<Boxes className="size-7" />}
          title="没有匹配的商品草稿"
          message="当前筛选条件没有返回真实商品；请调整筛选或创建新草稿。"
        />
      )}
    </Card>
  );
}

function CreateProductPanel({
  form,
  onSubmit,
  disabled,
}: {
  form: ReturnType<typeof useForm<ProductCreateFormValues>>;
  onSubmit: () => void;
  disabled: boolean;
}) {
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<PackagePlus className="size-5" />}
        title="新建商品草稿"
        description="写操作强制携带 Idempotency-Key，并由 catalog.product.create 审计。"
      />
      <form className="space-y-4" onSubmit={onSubmit}>
        <Field label="asset_id" error={form.formState.errors.asset_id?.message}>
          <Input {...form.register("asset_id")} placeholder="资产 UUID" />
        </Field>
        <Field label="asset_version_id" error={form.formState.errors.asset_version_id?.message}>
          <Input {...form.register("asset_version_id")} placeholder="资产版本 UUID" />
        </Field>
        <Field label="seller_org_id" error={form.formState.errors.seller_org_id?.message}>
          <Input {...form.register("seller_org_id")} placeholder="卖方组织 UUID" />
        </Field>
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="标题" error={form.formState.errors.title?.message}>
            <Input {...form.register("title")} placeholder="工业设备运行指标包" />
          </Field>
          <Field label="分类" error={form.formState.errors.category?.message}>
            <Input {...form.register("category")} />
          </Field>
        </div>
        <Field label="描述">
          <Textarea {...form.register("description")} placeholder="说明数据范围、样例与上架价值" />
        </Field>
        <div className="grid gap-3 md:grid-cols-3">
          <Field label="price_mode" error={form.formState.errors.price_mode?.message}>
            <select className={selectClassName} {...form.register("price_mode")}>
              <option value="one_time">one_time</option>
              <option value="subscription">subscription</option>
              <option value="pay_per_use">pay_per_use</option>
              <option value="project_fee">project_fee</option>
            </select>
          </Field>
          <Field label="价格" error={form.formState.errors.price?.message}>
            <Input {...form.register("price")} />
          </Field>
          <Field label="币种" error={form.formState.errors.currency_code?.message}>
            <Input {...form.register("currency_code")} />
          </Field>
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="delivery_type" error={form.formState.errors.delivery_type?.message}>
            <Input {...form.register("delivery_type")} placeholder="file_download" />
          </Field>
          <Field label="allowed_usage">
            <Input {...form.register("allowed_usage")} placeholder="internal_analysis, risk_control" />
          </Field>
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="schema_version">
            <Input {...form.register("schema_version")} />
          </Field>
          <Field label="sample_hash">
            <Input {...form.register("sample_hash")} />
          </Field>
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="origin_region">
            <Input {...form.register("origin_region")} />
          </Field>
          <Field label="allowed_region">
            <Input {...form.register("allowed_region")} placeholder="CN, SG" />
          </Field>
        </div>
        <Button type="submit" disabled={disabled}>
          <PackagePlus className="size-4" />
          创建草稿
        </Button>
      </form>
    </Card>
  );
}

function EditProductPanel({
  form,
  product,
  onSubmit,
  disabled,
}: {
  form: ReturnType<typeof useForm<ProductPatchFormValues>>;
  product?: SellerProductDetail;
  onSubmit: () => void;
  disabled: boolean;
}) {
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<FilePenLine className="size-5" />}
        title="产品编辑页"
        description="PATCH /api/v1/products/{id} 仅允许草稿态编辑；错误码按统一字典回显。"
      />
      <form className="space-y-4" onSubmit={onSubmit}>
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="标题" error={form.formState.errors.title?.message}>
            <Input {...form.register("title")} disabled={!product} />
          </Field>
          <Field label="分类" error={form.formState.errors.category?.message}>
            <Input {...form.register("category")} disabled={!product} />
          </Field>
        </div>
        <Field label="描述">
          <Textarea {...form.register("description")} disabled={!product} />
        </Field>
        <div className="grid gap-3 md:grid-cols-4">
          <Field label="price_mode">
            <select className={selectClassName} {...form.register("price_mode")} disabled={!product}>
              <option value="one_time">one_time</option>
              <option value="subscription">subscription</option>
              <option value="pay_per_use">pay_per_use</option>
              <option value="project_fee">project_fee</option>
            </select>
          </Field>
          <Field label="价格">
            <Input {...form.register("price")} disabled={!product} />
          </Field>
          <Field label="币种">
            <Input {...form.register("currency_code")} disabled={!product} />
          </Field>
          <Field label="delivery_type">
            <Input {...form.register("delivery_type")} disabled={!product} />
          </Field>
        </div>
        <div className="grid gap-3 md:grid-cols-3">
          <Field label="行业">
            <Input {...form.register("industry")} disabled={!product} />
          </Field>
          <Field label="数据分级">
            <Input {...form.register("data_classification")} disabled={!product} />
          </Field>
          <Field label="质量分">
            <Input {...form.register("quality_score")} disabled={!product} />
          </Field>
        </div>
        <Field label="use_cases">
          <Input {...form.register("use_cases")} disabled={!product} />
        </Field>
        <Button type="submit" disabled={disabled}>
          <FilePenLine className="size-4" />
          保存草稿
        </Button>
      </form>
    </Card>
  );
}

function SkuConfigPanel({
  form,
  product,
  onSubmit,
  disabled,
}: {
  form: ReturnType<typeof useForm<SkuCreateFormValues>>;
  product?: SellerProductDetail;
  onSubmit: () => void;
  disabled: boolean;
}) {
  return (
    <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_420px]">
      <Card className="space-y-5">
        <PanelTitle
          icon={<Tags className="size-5" />}
          title="SKU 配置页"
          description="先选择标准 SKU 真值，再填写商业套餐编码；trade_mode 只作为交付/计费路径投影。"
        />
        <StandardSkuMatrix compact />
        <div className="space-y-3">
          <CardTitle className="text-base">当前商品 SKU</CardTitle>
          {product?.skus.length ? (
            <div className="grid gap-3 md:grid-cols-2">
              {product.skus.map((sku) => (
                <div key={sku.sku_id} className="rounded-3xl border border-black/10 bg-white/75 p-4">
                  <div className="flex flex-wrap items-center gap-2">
                    <StatusBadge status={sku.status} />
                    <Badge className="bg-black/[0.04] text-[var(--ink-soft)]">{sku.sku_type}</Badge>
                  </div>
                  <div className="mt-3 font-semibold text-[var(--ink-strong)]">{sku.sku_code}</div>
                  <div className="mt-1 text-xs text-[var(--ink-subtle)]">{sku.sku_id}</div>
                  <div className="mt-3 grid gap-2 text-sm text-[var(--ink-soft)]">
                    <span>billing_mode: {sku.billing_mode}</span>
                    <span>trade_mode: {sku.trade_mode}</span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <StateCard
              icon={<Boxes className="size-7" />}
              title="尚未配置 SKU"
              message="提交审核前必须至少创建一个命中 V1 八个标准 SKU 的可售对象。"
            />
          )}
        </div>
      </Card>

      <Card className="space-y-5">
        <PanelTitle
          icon={<Layers3 className="size-5" />}
          title="新增 SKU"
          description="POST /api/v1/products/{id}/skus 会校验标准 SKU、trade_mode 与模板兼容性。"
        />
        <form className="space-y-4" onSubmit={onSubmit}>
          <Field label="sku_type" error={form.formState.errors.sku_type?.message}>
            <select className={selectClassName} {...form.register("sku_type")} disabled={!product}>
              {standardSkuOptions.map((option) => (
                <option key={option.sku_type} value={option.sku_type}>
                  {option.sku_type} - {option.label}
                </option>
              ))}
            </select>
          </Field>
          <Field label="sku_code" error={form.formState.errors.sku_code?.message}>
            <Input {...form.register("sku_code")} disabled={!product} />
          </Field>
          <div className="grid gap-3 md:grid-cols-2">
            <Field label="billing_mode">
              <select className={selectClassName} {...form.register("billing_mode")} disabled={!product}>
                <option value="one_time">one_time</option>
                <option value="subscription">subscription</option>
                <option value="usage">usage</option>
                <option value="project_fee">project_fee</option>
              </select>
            </Field>
            <Field label="trade_mode">
              <Input {...form.register("trade_mode")} disabled={!product} />
            </Field>
          </div>
          <div className="grid gap-3 md:grid-cols-2">
            <Field label="unit_name">
              <Input {...form.register("unit_name")} disabled={!product} />
            </Field>
            <Field label="delivery_object_kind">
              <Input {...form.register("delivery_object_kind")} disabled={!product} />
            </Field>
          </div>
          <Field label="template_id（可选）">
            <Input {...form.register("template_id")} disabled={!product} placeholder="合同模板 UUID" />
          </Field>
          <div className="grid gap-3 md:grid-cols-2">
            <Field label="acceptance_mode">
              <Input {...form.register("acceptance_mode")} disabled={!product} />
            </Field>
            <Field label="refund_mode">
              <Input {...form.register("refund_mode")} disabled={!product} />
            </Field>
          </div>
          <Button type="submit" disabled={disabled}>
            <Tags className="size-4" />
            创建 SKU
          </Button>
        </form>
      </Card>
    </div>
  );
}

function MetadataAndQualityPanel({
  metadataForm,
  qualityForm,
  product,
  onSaveMetadata,
  onCreateQuality,
  disabled,
}: {
  metadataForm: ReturnType<typeof useForm<MetadataProfileFormValues>>;
  qualityForm: ReturnType<typeof useForm<QualityReportFormValues>>;
  product?: SellerProductDetail;
  onSaveMetadata: () => void;
  onCreateQuality: () => void;
  disabled: boolean;
}) {
  return (
    <div className="grid gap-5 xl:grid-cols-2">
      <Card className="space-y-5">
        <PanelTitle
          icon={<ClipboardCheck className="size-5" />}
          title="十域元信息"
          description="PUT /api/v1/products/{id}/metadata-profile 固化业务、内容、结构、质量、合规、交付、版本、授权、责任、加工概览。"
        />
        <form className="space-y-4" onSubmit={onSaveMetadata}>
          {metadataFields.map((field) => (
            <Field
              key={field.name}
              label={field.label}
              error={metadataForm.formState.errors[field.name]?.message}
            >
              <Textarea
                {...metadataForm.register(field.name)}
                disabled={!product}
                className="min-h-20"
              />
            </Field>
          ))}
          <Button type="submit" disabled={disabled}>
            <ClipboardCheck className="size-4" />
            保存元信息
          </Button>
        </form>
      </Card>

      <Card className="space-y-5">
        <PanelTitle
          icon={<FileText className="size-5" />}
          title="质量报告"
          description="POST /api/v1/assets/{versionId}/quality-reports 只记录报告摘要、指标和哈希，不暴露真实对象路径。"
        />
        <div className="rounded-3xl bg-black/[0.03] p-4 text-sm text-[var(--ink-soft)]">
          asset_version_id: {product?.asset_version_id ?? "未选择商品"}
        </div>
        <form className="space-y-4" onSubmit={onCreateQuality}>
          <Field label="report_type" error={qualityForm.formState.errors.report_type?.message}>
            <Input {...qualityForm.register("report_type")} disabled={!product} />
          </Field>
          <div className="grid gap-3 md:grid-cols-3">
            <Field label="missing_rate">
              <Input type="number" step="0.001" {...qualityForm.register("missing_rate", { valueAsNumber: true })} disabled={!product} />
            </Field>
            <Field label="duplicate_rate">
              <Input type="number" step="0.001" {...qualityForm.register("duplicate_rate", { valueAsNumber: true })} disabled={!product} />
            </Field>
            <Field label="anomaly_rate">
              <Input type="number" step="0.001" {...qualityForm.register("anomaly_rate", { valueAsNumber: true })} disabled={!product} />
            </Field>
          </div>
          <Field label="sampling_method">
            <Input {...qualityForm.register("sampling_method")} disabled={!product} />
          </Field>
          <Field label="report_hash" error={qualityForm.formState.errors.report_hash?.message}>
            <Input {...qualityForm.register("report_hash")} disabled={!product} />
          </Field>
          <Field label="coverage_range_json.summary">
            <Textarea {...qualityForm.register("coverage_note")} disabled={!product} className="min-h-20" />
          </Field>
          <Field label="freshness_json.summary">
            <Textarea {...qualityForm.register("freshness_note")} disabled={!product} className="min-h-20" />
          </Field>
          <Button type="submit" disabled={disabled}>
            <FileText className="size-4" />
            登记质量报告
          </Button>
        </form>
      </Card>
    </div>
  );
}

function TemplateBindPanel({
  form,
  product,
  selectedScope,
  onSubmit,
  disabled,
}: {
  form: ReturnType<typeof useForm<TemplateBindFormValues>>;
  product?: SellerProductDetail;
  selectedScope: "product" | "sku";
  onSubmit: () => void;
  disabled: boolean;
}) {
  return (
    <Card className="space-y-5">
      <PanelTitle
        icon={<ShieldCheck className="size-5" />}
        title="模板绑定页"
        description="模板绑定必须经后端校验模板版本、SKU 兼容性与审计，不在前端伪造通过。"
      />
      <div className="grid gap-3 md:grid-cols-4">
        {standardSkuOptions.map((option) => (
          <div key={option.sku_type} className="rounded-3xl border border-black/10 bg-white/75 p-4">
            <Badge className="bg-black/[0.04] text-[var(--ink-soft)]">{option.sku_type}</Badge>
            <div className="mt-3 text-sm font-semibold text-[var(--ink-strong)]">{option.label}</div>
            <p className="mt-1 text-xs leading-5 text-[var(--ink-soft)]">{option.template_family}</p>
          </div>
        ))}
      </div>
      <form className="grid gap-4 md:grid-cols-2" onSubmit={onSubmit}>
        <Field label="绑定目标">
          <select className={selectClassName} {...form.register("target_scope")} disabled={!product}>
            <option value="product">product 默认模板</option>
            <option value="sku">单个 SKU 模板</option>
          </select>
        </Field>
        {selectedScope === "sku" ? (
          <Field label="sku_id" error={form.formState.errors.sku_id?.message}>
            <select className={selectClassName} {...form.register("sku_id")} disabled={!product}>
              <option value="">请选择 SKU</option>
              {(product?.skus ?? []).map((sku) => (
                <option key={sku.sku_id} value={sku.sku_id}>
                  {sku.sku_type} / {sku.sku_code}
                </option>
              ))}
            </select>
          </Field>
        ) : (
          <div className="rounded-3xl bg-black/[0.03] p-4 text-sm text-[var(--ink-soft)]">
            product_id: {product?.product_id ?? "未选择商品"}
          </div>
        )}
        <Field label="template_id" error={form.formState.errors.template_id?.message}>
          <Input {...form.register("template_id")} disabled={!product} placeholder="模板 UUID" />
        </Field>
        <Field label="binding_type" error={form.formState.errors.binding_type?.message}>
          <select className={selectClassName} {...form.register("binding_type")} disabled={!product}>
            <option value="contract">contract</option>
            <option value="acceptance">acceptance</option>
            <option value="refund">refund</option>
            <option value="license">license</option>
          </select>
        </Field>
        <div className="md:col-span-2">
          <Button type="submit" disabled={disabled}>
            <ShieldCheck className="size-4" />
            绑定模板
          </Button>
        </div>
      </form>
    </Card>
  );
}

function SubmitReviewPanel({
  form,
  product,
  actionState,
  onSubmit,
  disabled,
}: {
  form: ReturnType<typeof useForm<SubmitProductFormValues>>;
  product?: SellerProductDetail;
  actionState: ActionState;
  onSubmit: () => void;
  disabled: boolean;
}) {
  const gate = canSubmitProduct(product);

  return (
    <Card className="space-y-5 border-[var(--warning-ring)] bg-[linear-gradient(135deg,rgba(255,255,255,0.96),rgba(255,247,224,0.82))]">
      <PanelTitle
        icon={<Send className="size-5" />}
        title="提交审核"
        description="POST /api/v1/products/{id}/submit 会产生审核任务、outbox 事件和审计记录。"
      />
      <div className="grid gap-3 md:grid-cols-3">
        <InfoTile label="提交门禁" value={gate.allowed ? "允许" : "阻断"} />
        <InfoTile label="审计动作" value="catalog.product.submit" />
        <InfoTile label="幂等键" value={actionState.idempotencyKey ?? "提交时生成"} />
      </div>
      <CardDescription className={gate.allowed ? undefined : "text-[var(--warning-ink)]"}>
        {gate.reason}
      </CardDescription>
      <form className="space-y-4" onSubmit={onSubmit}>
        <Field label="submission_note" error={form.formState.errors.submission_note?.message}>
          <Textarea {...form.register("submission_note")} disabled={!product} />
        </Field>
        <Button type="submit" disabled={disabled || !gate.allowed}>
          <Send className="size-4" />
          提交审核
        </Button>
      </form>
    </Card>
  );
}

function ActionRail({
  product,
  selectedProductId,
  actionState,
}: {
  product?: SellerProductDetail;
  selectedProductId?: string;
  actionState: ActionState;
}) {
  return (
    <div className="space-y-5">
      <Card className="space-y-4">
        <PanelTitle
          icon={<Fingerprint className="size-5" />}
          title="联调与审计提示"
          description="前端不直连 Kafka / PostgreSQL / OpenSearch / Redis / Fabric。"
        />
        <div className="grid gap-3">
          <ContextRow label="API 边界" value="/api/platform -> platform-core" />
          <ContextRow label="SDK 契约" value="@datab/sdk-ts catalog domain" />
          <ContextRow label="写操作幂等" value="X-Idempotency-Key" />
          <ContextRow label="审计提示" value="创建、编辑、SKU、元信息、模板、提交均留痕" />
        </div>
      </Card>

      <Card className={cn(
        "space-y-4",
        actionState.tone === "success" && "border-emerald-200 bg-emerald-50/80",
        actionState.tone === "error" && "border-[var(--danger-ring)] bg-[var(--danger-soft)]",
      )}>
        <PanelTitle
          icon={<DatabaseZap className="size-5" />}
          title="最近动作结果"
          description="错误态展示后端 code / request_id，不吞掉统一错误码。"
        />
        <CardDescription
          className={actionState.tone === "error" ? "text-[var(--danger-ink)]" : undefined}
        >
          {actionState.message}
        </CardDescription>
        {actionState.idempotencyKey ? (
          <div className="break-all rounded-3xl bg-white/75 p-3 text-xs text-[var(--ink-soft)]">
            Idempotency-Key: {actionState.idempotencyKey}
          </div>
        ) : null}
      </Card>

      <Card className="space-y-4">
        <PanelTitle
          icon={<BadgeCheck className="size-5" />}
          title="当前草稿快照"
          description="链上状态与投影状态由后续审核/审计页展示，本页仅保留提交审计入口。"
        />
        <ContextRow label="product_id" value={selectedProductId ?? "未选择"} />
        <ContextRow label="状态" value={product?.status ? productStatusLabel(product.status) : "未返回"} />
        <ContextRow label="asset_id" value={product?.asset_id ?? "未返回"} />
        <ContextRow label="asset_version_id" value={product?.asset_version_id ?? "未返回"} />
        <ContextRow label="SKU 数" value={String(product?.skus.length ?? 0)} />
      </Card>
    </div>
  );
}

function SectionTabs({
  section,
  selectedProductId,
}: {
  section: WorkspaceSection;
  selectedProductId?: string;
}) {
  const productPath = selectedProductId ?? "draft";
  const items: { key: WorkspaceSection; label: string; href: string }[] = [
    { key: "center", label: "上架中心", href: "/seller/products" },
    { key: "edit", label: "产品编辑", href: `/seller/products/${productPath}/edit` },
    { key: "skus", label: "SKU 配置", href: `/seller/products/${productPath}/skus` },
    { key: "metadata", label: "元信息/质量", href: `/seller/products/${productPath}/metadata-contracts` },
    { key: "templates", label: "模板绑定", href: `/seller/products/${productPath}/templates` },
  ];

  return (
    <div className="flex flex-wrap gap-2">
      {items.map((item) => (
        <Button
          key={item.key}
          asChild
          variant={item.key === section ? "default" : "secondary"}
          size="sm"
        >
          <Link href={item.href as Route}>{item.label}</Link>
        </Button>
      ))}
    </div>
  );
}

function SelectedProductBanner({
  product,
  productPending,
  productError,
}: {
  product?: SellerProductDetail | SellerProductListItem;
  productPending: boolean;
  productError: unknown;
}) {
  if (productPending) {
    return (
      <StateCard
        icon={<LoaderCircle className="size-7 animate-spin" />}
        title="正在读取商品详情"
        message="GET /api/v1/products/{id} 正在返回元信息与 SKU 快照。"
      />
    );
  }
  if (productError) {
    return (
      <StateCard
        tone="danger"
        icon={<AlertTriangle className="size-7" />}
        title="商品详情读取失败"
        message={describeError(productError)}
      />
    );
  }
  if (!product) {
    return (
      <StateCard
        icon={<Boxes className="size-7" />}
        title="未选择商品"
        message="请从上架中心选择真实草稿，或先创建商品草稿后再配置 SKU、元信息与模板。"
      />
    );
  }
  return (
    <Card className="grid gap-4 md:grid-cols-[1fr_auto]">
      <div>
        <div className="flex flex-wrap items-center gap-2">
          <StatusBadge status={product.status} />
          <Badge className="bg-black/[0.04] text-[var(--ink-soft)]">{product.category}</Badge>
        </div>
        <CardTitle className="mt-3">{product.title}</CardTitle>
        <CardDescription className="mt-1 break-all">
          product_id: {product.product_id} / seller_org_id: {product.seller_org_id}
        </CardDescription>
      </div>
      <div className="grid gap-2 text-sm text-[var(--ink-soft)] md:text-right">
        <span>{product.currency_code} {product.price}</span>
        <span>{product.delivery_type}</span>
      </div>
    </Card>
  );
}

function StandardSkuMatrix({ compact = false }: { compact?: boolean }) {
  return (
    <Card className={cn("space-y-4", compact && "bg-black/[0.025] p-4 shadow-none")}>
      <div className="flex items-center justify-between gap-3">
        <div>
          <CardTitle className={compact ? "text-base" : undefined}>V1 八个标准 SKU</CardTitle>
          <CardDescription className="mt-1">
            SHARE_RO / QRY_LITE / SBX_STD / RPT_STD 按独立 SKU 展示，不并回大类。
          </CardDescription>
        </div>
      </div>
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        {standardSkuOptions.map((option) => (
          <div key={option.sku_type} className="rounded-3xl border border-black/10 bg-white/80 p-4">
            <Badge className="bg-black/[0.04] text-[var(--ink-soft)]">{option.sku_type}</Badge>
            <div className="mt-3 font-semibold text-[var(--ink-strong)]">{option.label}</div>
            <div className="mt-2 space-y-1 text-xs leading-5 text-[var(--ink-soft)]">
              <div>trade_mode: {option.trade_mode}</div>
              <div>billing_mode: {option.billing_mode}</div>
              <div>delivery: {option.delivery_object_kind}</div>
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
}

function StateCard({
  icon,
  title,
  message,
  tone = "muted",
}: {
  icon: ReactNode;
  title: string;
  message: string;
  tone?: "muted" | "warning" | "danger";
}) {
  return (
    <Card
      className={cn(
        "flex min-h-56 items-center justify-center",
        tone === "muted" && "bg-[var(--panel-muted)]",
        tone === "warning" && "border-[var(--warning-ring)] bg-[var(--warning-soft)]",
        tone === "danger" && "border-[var(--danger-ring)] bg-[var(--danger-soft)]",
      )}
    >
      <div
        className={cn(
          "flex max-w-2xl flex-col items-center gap-3 text-center text-[var(--ink-soft)]",
          tone === "warning" && "text-[var(--warning-ink)]",
          tone === "danger" && "text-[var(--danger-ink)]",
        )}
      >
        {icon}
        <CardTitle>{title}</CardTitle>
        <CardDescription
          className={cn(
            tone === "warning" && "text-[var(--warning-ink)]",
            tone === "danger" && "text-[var(--danger-ink)]",
          )}
        >
          {message}
        </CardDescription>
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
      <div className="rounded-2xl bg-[var(--accent-soft)] p-3 text-[var(--accent-strong)]">
        {icon}
      </div>
      <div>
        <CardTitle>{title}</CardTitle>
        <CardDescription className="mt-1">{description}</CardDescription>
      </div>
    </div>
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
    <label className="block space-y-2 text-sm font-medium text-[var(--ink-strong)]">
      <span>{label}</span>
      {children}
      {error ? <span className="text-xs text-[var(--danger-ink)]">{error}</span> : null}
    </label>
  );
}

function HeaderMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-3xl bg-white/65 p-4">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</div>
      <div className="mt-2 truncate text-sm font-semibold text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}

function ContextRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[92px_1fr] gap-3 rounded-2xl bg-black/[0.03] px-3 py-2 text-sm">
      <span className="text-[var(--ink-subtle)]">{label}</span>
      <span className="break-all font-medium text-[var(--ink-strong)]">{value}</span>
    </div>
  );
}

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-3xl bg-white/75 p-4">
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</div>
      <div className="mt-2 break-all text-sm font-semibold text-[var(--ink-strong)]">{value}</div>
    </div>
  );
}

function StatusCountCard({
  label,
  count,
  active,
  onClick,
}: {
  label: string;
  count: number;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "rounded-3xl border p-4 text-left transition",
        active
          ? "border-[var(--accent-strong)] bg-[var(--accent-soft)]"
          : "border-black/10 bg-white/75 hover:bg-white",
      )}
    >
      <div className="text-xs uppercase tracking-[0.16em] text-[var(--ink-subtle)]">{label}</div>
      <div className="mt-2 text-2xl font-semibold text-[var(--ink-strong)]">{count}</div>
    </button>
  );
}

function StatusBadge({ status }: { status: string }) {
  return (
    <span
      className={cn(
        "inline-flex rounded-full px-3 py-1 text-xs font-semibold",
        status === "draft" && "bg-slate-100 text-slate-700",
        status === "pending_review" && "bg-amber-100 text-amber-800",
        status === "listed" && "bg-emerald-100 text-emerald-800",
        status === "delisted" && "bg-zinc-100 text-zinc-700",
        status === "frozen" && "bg-rose-100 text-rose-800",
      )}
    >
      {productStatusLabel(status)}
    </span>
  );
}

function toErrorState(error: unknown): ActionState {
  return {
    tone: "error",
    message: describeError(error),
  };
}

function describeError(error: unknown): string {
  if (error instanceof PlatformApiError) {
    const request = error.requestId ? ` request_id=${error.requestId}` : "";
    return `${error.code}: ${error.message}${request}`;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "UNKNOWN: 上架中心请求失败。";
}

const metadataFields: {
  name: keyof MetadataProfileFormValues;
  label: string;
}[] = [
  { name: "business_description", label: "业务描述" },
  { name: "data_content", label: "数据内容" },
  { name: "structure_description", label: "结构描述" },
  { name: "quality_description", label: "质量描述" },
  { name: "compliance_description", label: "合规描述" },
  { name: "delivery_description", label: "交付描述" },
  { name: "version_description", label: "版本描述" },
  { name: "authorization_description", label: "授权描述" },
  { name: "responsibility_description", label: "责任描述" },
  { name: "processing_overview", label: "加工概览" },
];

const selectClassName =
  "flex h-11 w-full rounded-2xl border border-black/10 bg-white/90 px-4 text-sm text-[var(--ink-strong)] outline-none transition focus:border-[var(--accent-strong)] focus:ring-2 focus:ring-[var(--accent-soft)] disabled:opacity-50";
