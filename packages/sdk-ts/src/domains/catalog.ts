import { PlatformClient } from "../core/http";
import type {
  PathParams,
  QueryParams,
  RequestBody,
  SuccessBody,
} from "../core/openapi";
import type { operations as CatalogOperations } from "../generated/catalog";

export type StandardScenariosResponse =
  SuccessBody<CatalogOperations["getStandardScenarioTemplates"]>;
export type ProductListResponse =
  SuccessBody<CatalogOperations["listProducts"]>;
export type ProductDetailResponse =
  SuccessBody<CatalogOperations["getProductDetail"]>;
export type SellerProfileResponse =
  SuccessBody<CatalogOperations["getSellerProfile"]>;
export type DataProductResponse =
  SuccessBody<CatalogOperations["createProductDraft"]>;
export type ProductSkuResponse =
  SuccessBody<CatalogOperations["createProductSku"]>;
export type ProductSubmitResponse =
  SuccessBody<CatalogOperations["submitProduct"]>;
export type TemplateBindingResponse =
  SuccessBody<CatalogOperations["bindSkuTemplate"]>;
export type ProductMetadataProfileResponse =
  SuccessBody<CatalogOperations["putProductMetadataProfile"]>;
export type AssetQualityReportResponse =
  SuccessBody<CatalogOperations["createAssetQualityReport"]>;
export type ReviewDecisionResponse =
  SuccessBody<CatalogOperations["reviewSubject"]>;
export type ProductReviewResponse =
  SuccessBody<CatalogOperations["reviewProduct"]>;

export type ListProductsQuery = QueryParams<CatalogOperations["listProducts"]>;
export type CreateDataProductRequest =
  RequestBody<CatalogOperations["createProductDraft"]>;
export type PatchDataProductRequest =
  RequestBody<CatalogOperations["patchProductDraft"]>;
export type SubmitProductRequest =
  RequestBody<CatalogOperations["submitProduct"]>;
export type PutProductMetadataProfileRequest =
  RequestBody<CatalogOperations["putProductMetadataProfile"]>;
export type CreateProductSkuRequest =
  RequestBody<CatalogOperations["createProductSku"]>;
export type PatchProductSkuRequest =
  RequestBody<CatalogOperations["patchProductSku"]>;
export type BindTemplateRequest =
  RequestBody<CatalogOperations["bindSkuTemplate"]>;
export type CreateAssetQualityReportRequest =
  RequestBody<CatalogOperations["createAssetQualityReport"]>;
export type ReviewDecisionRequest =
  RequestBody<CatalogOperations["reviewSubject"]>;

export type CatalogMutationOptions = {
  idempotencyKey: string;
  stepUpToken?: string;
  stepUpChallengeId?: string;
};

function mutationHeaders(options: CatalogMutationOptions): HeadersInit {
  const headers: Record<string, string> = {
    "X-Idempotency-Key": options.idempotencyKey,
  };
  if (options.stepUpToken) {
    headers["X-Step-Up-Token"] = options.stepUpToken;
  }
  if (options.stepUpChallengeId) {
    headers["x-step-up-challenge-id"] = options.stepUpChallengeId;
  }
  return headers;
}

export function createCatalogClient(client: PlatformClient) {
  return {
    getStandardScenarioTemplates() {
      return client.getJson<StandardScenariosResponse>(
        "/api/v1/catalog/standard-scenarios",
      );
    },
    listProducts(query?: ListProductsQuery) {
      return client.getJson<ProductListResponse, ListProductsQuery>(
        "/api/v1/products",
        { query },
      );
    },
    createProductDraft(
      body: CreateDataProductRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<DataProductResponse, CreateDataProductRequest>(
        "/api/v1/products",
        {
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    getProductDetail(pathParams: PathParams<CatalogOperations["getProductDetail"]>) {
      return client.getJson<ProductDetailResponse>("/api/v1/products/{id}", {
        pathParams,
      });
    },
    patchProductDraft(
      pathParams: PathParams<CatalogOperations["patchProductDraft"]>,
      body: PatchDataProductRequest,
      options: CatalogMutationOptions,
    ) {
      return client.patchJson<DataProductResponse, PatchDataProductRequest>(
        "/api/v1/products/{id}",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    getSellerProfile(
      pathParams: PathParams<CatalogOperations["getSellerProfile"]>,
    ) {
      return client.getJson<SellerProfileResponse>(
        "/api/v1/sellers/{orgId}/profile",
        { pathParams },
      );
    },
    putProductMetadataProfile(
      pathParams: PathParams<CatalogOperations["putProductMetadataProfile"]>,
      body: PutProductMetadataProfileRequest,
      options: CatalogMutationOptions,
    ) {
      return client.putJson<
        ProductMetadataProfileResponse,
        PutProductMetadataProfileRequest
      >("/api/v1/products/{id}/metadata-profile", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    createProductSku(
      pathParams: PathParams<CatalogOperations["createProductSku"]>,
      body: CreateProductSkuRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<ProductSkuResponse, CreateProductSkuRequest>(
        "/api/v1/products/{id}/skus",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    reviewSubject(
      pathParams: PathParams<CatalogOperations["reviewSubject"]>,
      body: ReviewDecisionRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<ReviewDecisionResponse, ReviewDecisionRequest>(
        "/api/v1/review/subjects/{id}",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    reviewProduct(
      pathParams: PathParams<CatalogOperations["reviewProduct"]>,
      body: ReviewDecisionRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<ProductReviewResponse, ReviewDecisionRequest>(
        "/api/v1/review/products/{id}",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    reviewCompliance(
      pathParams: PathParams<CatalogOperations["reviewCompliance"]>,
      body: ReviewDecisionRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<ReviewDecisionResponse, ReviewDecisionRequest>(
        "/api/v1/review/compliance/{id}",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    patchProductSku(
      pathParams: PathParams<CatalogOperations["patchProductSku"]>,
      body: PatchProductSkuRequest,
      options: CatalogMutationOptions,
    ) {
      return client.patchJson<ProductSkuResponse, PatchProductSkuRequest>(
        "/api/v1/skus/{id}",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    bindProductTemplate(
      pathParams: PathParams<CatalogOperations["bindProductTemplate"]>,
      body: BindTemplateRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<TemplateBindingResponse, BindTemplateRequest>(
        "/api/v1/products/{id}/bind-template",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    bindSkuTemplate(
      pathParams: PathParams<CatalogOperations["bindSkuTemplate"]>,
      body: BindTemplateRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<TemplateBindingResponse, BindTemplateRequest>(
        "/api/v1/skus/{id}/bind-template",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    createAssetQualityReport(
      pathParams: PathParams<CatalogOperations["createAssetQualityReport"]>,
      body: CreateAssetQualityReportRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<
        AssetQualityReportResponse,
        CreateAssetQualityReportRequest
      >("/api/v1/assets/{versionId}/quality-reports", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    submitProduct(
      pathParams: PathParams<CatalogOperations["submitProduct"]>,
      body: SubmitProductRequest,
      options: CatalogMutationOptions,
    ) {
      return client.postJson<ProductSubmitResponse, SubmitProductRequest>(
        "/api/v1/products/{id}/submit",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
  };
}
