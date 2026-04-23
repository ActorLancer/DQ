import { PlatformClient } from "../core/http";
import type { QueryParams, SuccessBody } from "../core/openapi";
import type { paths as SearchPaths } from "../generated/search";

type SearchCatalogOperation = SearchPaths["/api/v1/catalog/search"]["get"];

export type SearchCatalogQuery = QueryParams<SearchCatalogOperation>;
export type SearchCatalogResponse = SuccessBody<SearchCatalogOperation>;

export function createSearchClient(client: PlatformClient) {
  return {
    searchCatalog(query: SearchCatalogQuery) {
      return client.getJson<SearchCatalogResponse, SearchCatalogQuery>(
        "/api/v1/catalog/search",
        { query },
      );
    },
  };
}
