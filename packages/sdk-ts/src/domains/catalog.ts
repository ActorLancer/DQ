import { PlatformClient } from "../core/http";
import type {
  PathParams,
  SuccessBody,
} from "../core/openapi";
import type { operations as CatalogOperations } from "../generated/catalog";

export type StandardScenariosResponse =
  SuccessBody<CatalogOperations["getStandardScenarioTemplates"]>;
export type ProductDetailResponse =
  SuccessBody<CatalogOperations["getProductDetail"]>;
export type SellerProfileResponse =
  SuccessBody<CatalogOperations["getSellerProfile"]>;

export function createCatalogClient(client: PlatformClient) {
  return {
    getStandardScenarioTemplates() {
      return client.getJson<StandardScenariosResponse>(
        "/api/v1/catalog/standard-scenarios",
      );
    },
    getProductDetail(pathParams: PathParams<CatalogOperations["getProductDetail"]>) {
      return client.getJson<ProductDetailResponse>("/api/v1/products/{id}", {
        pathParams,
      });
    },
    getSellerProfile(
      pathParams: PathParams<CatalogOperations["getSellerProfile"]>,
    ) {
      return client.getJson<SellerProfileResponse>(
        "/api/v1/sellers/{orgId}/profile",
        { pathParams },
      );
    },
  };
}
