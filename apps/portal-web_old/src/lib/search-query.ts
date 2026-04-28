import { z } from "zod";

import type { SearchCatalogQuery } from "@datab/sdk-ts";

const entityScopeValues = ["all", "product", "service", "seller"] as const;
const sortValues = [
  "composite",
  "latest",
  "price_asc",
  "price_desc",
  "quality",
  "reputation",
  "hotness",
] as const;

export const entityScopeOptions = [
  { value: "all", label: "全部对象" },
  { value: "product", label: "数据商品" },
  { value: "service", label: "服务商品" },
  { value: "seller", label: "卖方主体" },
] as const satisfies readonly {
  value: NonNullable<SearchCatalogQuery["entity_scope"]>;
  label: string;
}[];

export const sortOptions = [
  { value: "composite", label: "综合排序" },
  { value: "latest", label: "上架时间" },
  { value: "price_asc", label: "价格从低到高" },
  { value: "price_desc", label: "价格从高到低" },
  { value: "quality", label: "质量评分" },
  { value: "reputation", label: "信誉评分" },
  { value: "hotness", label: "热度排序" },
] as const satisfies readonly {
  value: NonNullable<SearchCatalogQuery["sort"]>;
  label: string;
}[];

export const pageSizeOptions = [12, 24, 48] as const;

const decimalInput = z
  .string()
  .trim()
  .refine((value) => value === "" || /^\d+(\.\d{1,2})?$/.test(value), {
    message: "请输入最多两位小数的非负金额",
  });

export const searchFormSchema = z
  .object({
    q: z.string().trim().max(80, "关键词最多 80 个字符"),
    entity_scope: z.enum(entityScopeValues),
    industry: z.string().trim().max(60, "行业分类最多 60 个字符"),
    seller_org_id: z.string().trim().max(64, "seller_org_id 最多 64 个字符"),
    seller_type: z.string().trim().max(60, "seller_type 最多 60 个字符"),
    data_classification: z.string().trim().max(60, "敏感等级最多 60 个字符"),
    price_mode: z.string().trim().max(60, "价格模式最多 60 个字符"),
    tags: z.string().trim().max(160, "标签最多 160 个字符"),
    delivery_mode: z.string().trim().max(60, "交付方式最多 60 个字符"),
    price_min: decimalInput,
    price_max: decimalInput,
    sort: z.enum(sortValues),
    page_size: z.number().int().refine(
      (value) => pageSizeOptions.includes(value as 12 | 24 | 48),
      {
        message: "分页大小必须为 12 / 24 / 48",
      },
    ),
  })
  .refine(
    (value) =>
      value.price_min === "" ||
      value.price_max === "" ||
      Number(value.price_min) <= Number(value.price_max),
    {
      message: "最低价格不能大于最高价格",
      path: ["price_max"],
    },
  );

export type SearchFormValues = z.infer<typeof searchFormSchema>;

export const defaultSearchFormValues: SearchFormValues = {
  q: "",
  entity_scope: "all",
  industry: "",
  seller_org_id: "",
  seller_type: "",
  data_classification: "",
  price_mode: "",
  tags: "",
  delivery_mode: "",
  price_min: "",
  price_max: "",
  sort: "composite",
  page_size: 12,
};

type SearchParamReader = {
  get(name: string): string | null;
  getAll(name: string): string[];
};

export function parseSearchFormValues(
  params: SearchParamReader,
): SearchFormValues {
  const tags = params.getAll("tags");
  const rawPageSize = Number(params.get("page_size") ?? defaultSearchFormValues.page_size);
  const pageSize = pageSizeOptions.includes(rawPageSize as 12 | 24 | 48)
    ? rawPageSize
    : defaultSearchFormValues.page_size;

  const candidate = {
    q: params.get("q") ?? defaultSearchFormValues.q,
    entity_scope: pickOption(
      params.get("entity_scope"),
      entityScopeOptions,
      defaultSearchFormValues.entity_scope,
    ),
    industry: params.get("industry") ?? defaultSearchFormValues.industry,
    seller_org_id: params.get("seller_org_id") ?? defaultSearchFormValues.seller_org_id,
    seller_type: params.get("seller_type") ?? defaultSearchFormValues.seller_type,
    data_classification:
      params.get("data_classification") ?? defaultSearchFormValues.data_classification,
    price_mode: params.get("price_mode") ?? defaultSearchFormValues.price_mode,
    tags: tags.length ? tags.join(", ") : (params.get("tags") ?? defaultSearchFormValues.tags),
    delivery_mode: params.get("delivery_mode") ?? defaultSearchFormValues.delivery_mode,
    price_min: params.get("price_min") ?? defaultSearchFormValues.price_min,
    price_max: params.get("price_max") ?? defaultSearchFormValues.price_max,
    sort: pickOption(params.get("sort"), sortOptions, defaultSearchFormValues.sort),
    page_size: pageSize,
  };
  const parsed = searchFormSchema.safeParse(candidate);
  return parsed.success ? parsed.data : defaultSearchFormValues;
}

export function pageFromSearchParams(params: SearchParamReader): number {
  const page = Number(params.get("page") ?? "1");
  return Number.isInteger(page) && page > 0 ? page : 1;
}

export function formValuesToSearchQuery(
  values: SearchFormValues,
  page: number,
): SearchCatalogQuery {
  const parsed = searchFormSchema.parse(values);
  return stripEmptyQuery({
    q: parsed.q,
    entity_scope: parsed.entity_scope,
    industry: parsed.industry,
    seller_org_id: parsed.seller_org_id,
    seller_type: parsed.seller_type,
    data_classification: parsed.data_classification,
    price_mode: parsed.price_mode,
    tags: splitTags(parsed.tags),
    delivery_mode: parsed.delivery_mode,
    price_min: parsed.price_min ? Number(parsed.price_min) : undefined,
    price_max: parsed.price_max ? Number(parsed.price_max) : undefined,
    sort: parsed.sort,
    page,
    page_size: parsed.page_size,
    include_facets: true,
  });
}

export function formValuesToUrlSearchParams(
  values: SearchFormValues,
  page = 1,
): URLSearchParams {
  const query = formValuesToSearchQuery(values, page);
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null || value === "") {
      continue;
    }
    if (Array.isArray(value)) {
      for (const item of value) {
        params.append(key, String(item));
      }
      continue;
    }
    params.set(key, String(value));
  }
  return params;
}

export function splitTags(value: string): string[] {
  return value
    .split(/[,\s，、]+/)
    .map((tag) => tag.trim())
    .filter(Boolean);
}

function pickOption<const T extends readonly { value: string }[]>(
  value: string | null,
  options: T,
  fallback: T[number]["value"],
): T[number]["value"] {
  const matched = options.find((option) => option.value === value);
  return matched?.value ?? fallback;
}

function stripEmptyQuery(query: SearchCatalogQuery): SearchCatalogQuery {
  return Object.fromEntries(
    Object.entries(query).filter(([, value]) => {
      if (Array.isArray(value)) {
        return value.length > 0;
      }
      return value !== undefined && value !== null && value !== "";
    }),
  ) as SearchCatalogQuery;
}
