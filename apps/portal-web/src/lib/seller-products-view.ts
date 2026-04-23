import type {
  BindTemplateRequest,
  CreateAssetQualityReportRequest,
  CreateDataProductRequest,
  CreateProductSkuRequest,
  PatchDataProductRequest,
  ProductDetailResponse,
  ProductListResponse,
  PutProductMetadataProfileRequest,
  SubmitProductRequest,
} from "@datab/sdk-ts";
import { z } from "zod";

export type SellerProductListItem =
  ProductListResponse["data"]["items"][number];
export type SellerProductDetail = ProductDetailResponse["data"];

export const standardSkuOptions = [
  {
    sku_type: "FILE_STD",
    label: "文件快照",
    trade_mode: "snapshot_sale",
    billing_mode: "one_time",
    unit_name: "份",
    delivery_object_kind: "file_package",
    template_family: "CONTRACT_FILE_V1 / ACCEPT_FILE_V1",
  },
  {
    sku_type: "FILE_SUB",
    label: "版本订阅",
    trade_mode: "revision_subscription",
    billing_mode: "subscription",
    unit_name: "周期",
    delivery_object_kind: "revision_package",
    template_family: "CONTRACT_FILE_SUB_V1 / ACCEPT_FILE_SUB_V1",
  },
  {
    sku_type: "SHARE_RO",
    label: "只读共享",
    trade_mode: "share_grant",
    billing_mode: "subscription",
    unit_name: "授权",
    delivery_object_kind: "share_grant",
    template_family: "CONTRACT_SHARE_RO_V1 / ACCEPT_SHARE_RO_V1",
  },
  {
    sku_type: "API_SUB",
    label: "API 订阅",
    trade_mode: "api_subscription",
    billing_mode: "subscription",
    unit_name: "月",
    delivery_object_kind: "api_access",
    template_family: "CONTRACT_API_SUB_V1 / ACCEPT_API_SUB_V1",
  },
  {
    sku_type: "API_PPU",
    label: "API 按量",
    trade_mode: "api_pay_per_use",
    billing_mode: "usage",
    unit_name: "次",
    delivery_object_kind: "api_call",
    template_family: "CONTRACT_API_PPU_V1 / ACCEPT_API_PPU_V1",
  },
  {
    sku_type: "QRY_LITE",
    label: "模板查询 lite",
    trade_mode: "template_query",
    billing_mode: "usage",
    unit_name: "次",
    delivery_object_kind: "template_query",
    template_family: "CONTRACT_QUERY_LITE_V1 / ACCEPT_QUERY_LITE_V1",
  },
  {
    sku_type: "SBX_STD",
    label: "查询沙箱",
    trade_mode: "sandbox_workspace",
    billing_mode: "subscription",
    unit_name: "席位月",
    delivery_object_kind: "sandbox_workspace",
    template_family: "CONTRACT_SANDBOX_V1 / ACCEPT_SANDBOX_V1",
  },
  {
    sku_type: "RPT_STD",
    label: "固定报告/结果产品",
    trade_mode: "report_delivery",
    billing_mode: "one_time",
    unit_name: "份",
    delivery_object_kind: "report_package",
    template_family: "CONTRACT_REPORT_V1 / ACCEPT_REPORT_V1",
  },
] as const;

export type StandardSkuType = (typeof standardSkuOptions)[number]["sku_type"];

export const productStatusOptions = [
  "draft",
  "pending_review",
  "listed",
  "delisted",
  "frozen",
] as const;

export const productCreateSchema = z.object({
  asset_id: z.string().uuid("asset_id 必须是 UUID"),
  asset_version_id: z.string().uuid("asset_version_id 必须是 UUID"),
  seller_org_id: z.string().uuid("seller_org_id 必须是 UUID"),
  title: z.string().trim().min(2, "商品标题至少 2 个字符"),
  category: z.string().trim().min(1, "分类不能为空"),
  product_type: z.string().trim().min(1, "product_type 不能为空"),
  description: z.string().trim().optional(),
  price_mode: z.enum(["one_time", "subscription", "pay_per_use", "project_fee"]),
  price: z.string().trim().min(1, "价格不能为空"),
  currency_code: z.string().trim().length(3, "币种必须是 3 位代码"),
  delivery_type: z.string().trim().min(1, "交付方式不能为空"),
  allowed_usage: z.string().trim().optional(),
  searchable_text: z.string().trim().optional(),
  subtitle: z.string().trim().optional(),
  industry: z.string().trim().optional(),
  use_cases: z.string().trim().optional(),
  data_classification: z.string().trim().optional(),
  quality_score: z.string().trim().optional(),
  schema_version: z.string().trim().optional(),
  full_hash: z.string().trim().optional(),
  sample_hash: z.string().trim().optional(),
  origin_region: z.string().trim().optional(),
  allowed_region: z.string().trim().optional(),
});

export const productPatchSchema = z.object({
  title: z.string().trim().min(2, "商品标题至少 2 个字符"),
  category: z.string().trim().min(1, "分类不能为空"),
  description: z.string().trim().optional(),
  price_mode: z.enum(["one_time", "subscription", "pay_per_use", "project_fee"]),
  price: z.string().trim().min(1, "价格不能为空"),
  currency_code: z.string().trim().length(3, "币种必须是 3 位代码"),
  delivery_type: z.string().trim().min(1, "交付方式不能为空"),
  searchable_text: z.string().trim().optional(),
  subtitle: z.string().trim().optional(),
  industry: z.string().trim().optional(),
  use_cases: z.string().trim().optional(),
  data_classification: z.string().trim().optional(),
  quality_score: z.string().trim().optional(),
});

export const skuCreateSchema = z.object({
  sku_code: z.string().trim().min(2, "sku_code 不能为空"),
  sku_type: z.enum([
    "FILE_STD",
    "FILE_SUB",
    "SHARE_RO",
    "API_SUB",
    "API_PPU",
    "QRY_LITE",
    "SBX_STD",
    "RPT_STD",
  ]),
  billing_mode: z.enum(["one_time", "subscription", "usage", "project_fee"]),
  trade_mode: z.string().trim().min(1, "trade_mode 不能为空"),
  unit_name: z.string().trim().optional(),
  delivery_object_kind: z.string().trim().optional(),
  subscription_cadence: z.string().trim().optional(),
  share_protocol: z.string().trim().optional(),
  result_form: z.string().trim().optional(),
  template_id: z.string().trim().optional(),
  acceptance_mode: z.string().trim().min(1, "验收方式不能为空"),
  refund_mode: z.string().trim().min(1, "退款规则不能为空"),
});

export const metadataProfileSchema = z.object({
  business_description: z.string().trim().min(1, "业务描述不能为空"),
  data_content: z.string().trim().min(1, "数据内容不能为空"),
  structure_description: z.string().trim().min(1, "结构描述不能为空"),
  quality_description: z.string().trim().min(1, "质量描述不能为空"),
  compliance_description: z.string().trim().min(1, "合规描述不能为空"),
  delivery_description: z.string().trim().min(1, "交付描述不能为空"),
  version_description: z.string().trim().min(1, "版本描述不能为空"),
  authorization_description: z.string().trim().min(1, "授权描述不能为空"),
  responsibility_description: z.string().trim().min(1, "责任描述不能为空"),
  processing_overview: z.string().trim().min(1, "加工概览不能为空"),
});

export const qualityReportSchema = z.object({
  report_type: z.string().trim().min(1, "报告类型不能为空"),
  missing_rate: z.number().min(0).max(1),
  duplicate_rate: z.number().min(0).max(1),
  anomaly_rate: z.number().min(0).max(1),
  sampling_method: z.string().trim().min(1, "抽样方式不能为空"),
  report_hash: z.string().trim().min(8, "报告哈希至少 8 个字符"),
  coverage_note: z.string().trim().min(1, "覆盖范围不能为空"),
  freshness_note: z.string().trim().min(1, "新鲜度说明不能为空"),
});

export const templateBindSchema = z
  .object({
    target_scope: z.enum(["product", "sku"]),
    sku_id: z.string().trim().optional(),
    template_id: z.string().uuid("template_id 必须是 UUID"),
    binding_type: z.enum(["contract", "acceptance", "refund", "license"]),
  })
  .refine((value) => value.target_scope === "product" || Boolean(value.sku_id), {
    path: ["sku_id"],
    message: "按 SKU 绑定模板时必须选择 sku_id",
  });

export const submitProductSchema = z.object({
  submission_note: z.string().trim().min(6, "提交说明至少 6 个字符"),
});

export type ProductCreateFormValues = z.infer<typeof productCreateSchema>;
export type ProductPatchFormValues = z.infer<typeof productPatchSchema>;
export type SkuCreateFormValues = z.infer<typeof skuCreateSchema>;
export type MetadataProfileFormValues = z.infer<typeof metadataProfileSchema>;
export type QualityReportFormValues = z.infer<typeof qualityReportSchema>;
export type TemplateBindFormValues = z.infer<typeof templateBindSchema>;
export type SubmitProductFormValues = z.infer<typeof submitProductSchema>;

export function getSkuOption(skuType: string) {
  return standardSkuOptions.find((option) => option.sku_type === skuType);
}

export function isSellerProductRole(roles: string[]): boolean {
  return roles.some((role) =>
    ["platform_admin", "tenant_admin", "seller_operator"].includes(role),
  );
}

export function canSubmitProduct(product: SellerProductDetail | null | undefined): {
  allowed: boolean;
  reason: string;
} {
  if (!product) {
    return { allowed: false, reason: "请先选择商品草稿。" };
  }
  if (product.status !== "draft") {
    return {
      allowed: false,
      reason: `当前商品状态为 ${product.status}，只有 draft 可提交审核。`,
    };
  }
  if (!product.skus.length) {
    return { allowed: false, reason: "提交审核前必须至少配置一个 SKU。" };
  }
  if (product.skus.some((sku) => !standardSkuOptions.some((option) => option.sku_type === sku.sku_type))) {
    return { allowed: false, reason: "SKU 必须全部命中 V1 八个标准 SKU 真值。" };
  }
  return { allowed: true, reason: "商品仍为草稿且已有标准 SKU，可提交审核。" };
}

export function productStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    draft: "草稿",
    pending_review: "待审核",
    listed: "已上架",
    delisted: "已下架",
    frozen: "已冻结",
  };
  return labels[status] ?? status;
}

export function splitCsv(value: string | undefined): string[] {
  return (value ?? "")
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

export function emptyToUndefined(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

export function createCatalogIdempotencyKey(action: string): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `web-007-${action}-${crypto.randomUUID()}`;
  }
  return `web-007-${action}-${Date.now()}`;
}

export function buildCreateProductRequest(
  values: ProductCreateFormValues,
): CreateDataProductRequest {
  return {
    asset_id: values.asset_id,
    asset_version_id: values.asset_version_id,
    seller_org_id: values.seller_org_id,
    title: values.title.trim(),
    category: values.category.trim(),
    product_type: values.product_type.trim(),
    description: emptyToUndefined(values.description),
    price_mode: values.price_mode,
    price: values.price.trim(),
    currency_code: values.currency_code.trim().toUpperCase(),
    delivery_type: values.delivery_type.trim(),
    allowed_usage: splitCsv(values.allowed_usage),
    searchable_text: emptyToUndefined(values.searchable_text),
    subtitle: emptyToUndefined(values.subtitle),
    industry: emptyToUndefined(values.industry),
    use_cases: splitCsv(values.use_cases),
    data_classification: emptyToUndefined(values.data_classification),
    quality_score: emptyToUndefined(values.quality_score),
    metadata: buildProductMetadata(values),
  };
}

export function buildPatchProductRequest(
  values: ProductPatchFormValues,
): PatchDataProductRequest {
  return {
    title: values.title.trim(),
    category: values.category.trim(),
    description: emptyToUndefined(values.description),
    price_mode: values.price_mode,
    price: values.price.trim(),
    currency_code: values.currency_code.trim().toUpperCase(),
    delivery_type: values.delivery_type.trim(),
    searchable_text: emptyToUndefined(values.searchable_text),
    subtitle: emptyToUndefined(values.subtitle),
    industry: emptyToUndefined(values.industry),
    use_cases: splitCsv(values.use_cases),
    data_classification: emptyToUndefined(values.data_classification),
    quality_score: emptyToUndefined(values.quality_score),
  };
}

export function buildSkuRequest(values: SkuCreateFormValues): CreateProductSkuRequest {
  return {
    sku_code: values.sku_code.trim(),
    sku_type: values.sku_type,
    unit_name: emptyToUndefined(values.unit_name),
    billing_mode: values.billing_mode,
    trade_mode: values.trade_mode.trim(),
    delivery_object_kind: emptyToUndefined(values.delivery_object_kind),
    subscription_cadence: emptyToUndefined(values.subscription_cadence),
    share_protocol: emptyToUndefined(values.share_protocol),
    result_form: emptyToUndefined(values.result_form),
    template_id: emptyToUndefined(values.template_id),
    acceptance_mode: values.acceptance_mode.trim(),
    refund_mode: values.refund_mode.trim(),
  };
}

export function buildMetadataProfileRequest(
  values: MetadataProfileFormValues,
): PutProductMetadataProfileRequest {
  return {
    metadata_version_no: 1,
    business_description_json: textBlock(values.business_description),
    data_content_json: textBlock(values.data_content),
    structure_description_json: textBlock(values.structure_description),
    quality_description_json: textBlock(values.quality_description),
    compliance_description_json: textBlock(values.compliance_description),
    delivery_description_json: textBlock(values.delivery_description),
    version_description_json: textBlock(values.version_description),
    authorization_description_json: textBlock(values.authorization_description),
    responsibility_description_json: textBlock(values.responsibility_description),
    processing_overview_json: textBlock(values.processing_overview),
    status: "draft",
    metadata: {
      updated_from: "portal-web",
      task_id: "WEB-007",
    },
  };
}

export function buildQualityReportRequest(
  values: QualityReportFormValues,
): CreateAssetQualityReportRequest {
  return {
    report_type: values.report_type.trim(),
    coverage_range_json: textBlock(values.coverage_note),
    freshness_json: textBlock(values.freshness_note),
    missing_rate: values.missing_rate,
    duplicate_rate: values.duplicate_rate,
    anomaly_rate: values.anomaly_rate,
    sampling_method: values.sampling_method.trim(),
    report_hash: values.report_hash.trim(),
    metrics_json: {
      missing_rate: values.missing_rate,
      duplicate_rate: values.duplicate_rate,
      anomaly_rate: values.anomaly_rate,
    },
    status: "draft",
    metadata: {
      updated_from: "portal-web",
      task_id: "WEB-007",
    },
  };
}

export function buildTemplateBindRequest(
  values: TemplateBindFormValues,
): BindTemplateRequest {
  return {
    template_id: values.template_id,
    binding_type: values.binding_type,
  };
}

export function buildSubmitProductRequest(
  values: SubmitProductFormValues,
): SubmitProductRequest {
  return {
    submission_note: values.submission_note.trim(),
  };
}

export function defaultCreateProductValues(tenantId?: string): ProductCreateFormValues {
  return {
    asset_id: "",
    asset_version_id: "",
    seller_org_id: tenantId ?? "",
    title: "",
    category: "industrial_data",
    product_type: "data_product",
    description: "",
    price_mode: "one_time",
    price: "1000",
    currency_code: "CNY",
    delivery_type: "file_download",
    allowed_usage: "internal_analysis",
    searchable_text: "",
    subtitle: "",
    industry: "industrial_manufacturing",
    use_cases: "risk_control, efficiency_analysis",
    data_classification: "L2",
    quality_score: "0.90",
    schema_version: "v1",
    full_hash: "",
    sample_hash: "",
    origin_region: "CN",
    allowed_region: "CN",
  };
}

export function patchValuesFromProduct(
  product: SellerProductDetail | null | undefined,
): ProductPatchFormValues {
  return {
    title: product?.title ?? "",
    category: product?.category ?? "",
    description: product?.description ?? "",
    price_mode: priceModeValue(product?.price_mode),
    price: product?.price ?? "0",
    currency_code: product?.currency_code ?? "CNY",
    delivery_type: product?.delivery_type ?? "file_download",
    searchable_text: product?.searchable_text ?? "",
    subtitle: product?.subtitle ?? "",
    industry: product?.industry ?? "",
    use_cases: product?.use_cases.join(", ") ?? "",
    data_classification: product?.data_classification ?? "",
    quality_score: product?.quality_score ?? "",
  };
}

export function defaultSkuValues(skuType: StandardSkuType = "FILE_STD"): SkuCreateFormValues {
  const option = getSkuOption(skuType) ?? standardSkuOptions[0];
  return {
    sku_code: `${option.sku_type}-BASIC`,
    sku_type: option.sku_type,
    billing_mode: option.billing_mode,
    trade_mode: option.trade_mode,
    unit_name: option.unit_name,
    delivery_object_kind: option.delivery_object_kind,
    subscription_cadence: option.billing_mode === "subscription" ? "monthly" : "",
    share_protocol: option.sku_type === "SHARE_RO" ? "read_only_share" : "",
    result_form: option.sku_type === "QRY_LITE" ? "bounded_result" : "",
    template_id: "",
    acceptance_mode: option.sku_type.startsWith("API") ? "auto_accept" : "manual_accept",
    refund_mode: "manual_refund",
  };
}

export function defaultMetadataValues(): MetadataProfileFormValues {
  return {
    business_description: "面向企业内部分析与运营优化的标准数据产品。",
    data_content: "包含受控样例、字段说明和可验证的数据范围。",
    structure_description: "字段定义、主键、分区键和时间字段由原始接入链路沉淀。",
    quality_description: "缺失率、重复率、异常率均通过质量报告记录。",
    compliance_description: "用途、地域、导出与授权边界按 V1 合规要求声明。",
    delivery_description: "按所选标准 SKU 进入对应交付链路。",
    version_description: "当前上架版本基于 asset_version_id 固化。",
    authorization_description: "交易后按合同模板和 Usage Policy 生成授权。",
    responsibility_description: "卖方负责数据来源、质量说明和持续履约。",
    processing_overview: "原始接入、格式检测、加工任务和质量评估形成可审计链条。",
  };
}

export function defaultQualityReportValues(): QualityReportFormValues {
  return {
    report_type: "pre_listing_quality",
    missing_rate: 0.01,
    duplicate_rate: 0.005,
    anomaly_rate: 0.002,
    sampling_method: "stratified_sample",
    report_hash: "sha256-web-007-quality",
    coverage_note: "覆盖本次上架 asset_version_id 的主要字段与样例范围。",
    freshness_note: "数据新鲜度按源系统更新时间和抽样时间共同评估。",
  };
}

export function defaultTemplateBindValues(): TemplateBindFormValues {
  return {
    target_scope: "product",
    sku_id: "",
    template_id: "",
    binding_type: "contract",
  };
}

export function defaultSubmitProductValues(): SubmitProductFormValues {
  return {
    submission_note: "WEB-007 上架中心提交审核，SKU、元信息和质量报告已补齐。",
  };
}

function buildProductMetadata(values: ProductCreateFormValues) {
  return {
    schema_version: emptyToUndefined(values.schema_version),
    full_hash: emptyToUndefined(values.full_hash),
    sample_hash: emptyToUndefined(values.sample_hash),
    origin_region: emptyToUndefined(values.origin_region),
    allowed_region: splitCsv(values.allowed_region),
    task_id: "WEB-007",
  };
}

function textBlock(summary: string) {
  return {
    summary: summary.trim(),
  };
}

function priceModeValue(value: string | undefined): ProductPatchFormValues["price_mode"] {
  if (
    value === "one_time" ||
    value === "subscription" ||
    value === "pay_per_use" ||
    value === "project_fee"
  ) {
    return value;
  }
  return "one_time";
}
