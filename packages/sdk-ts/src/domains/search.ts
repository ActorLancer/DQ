import { PlatformClient } from "../core/http";
import type {
  HeaderParams,
  PathParams,
  QueryParams,
  RequestBody,
  SuccessBody,
} from "../core/openapi";
import type { paths as SearchPaths } from "../generated/search";

type SearchCatalogOperation = SearchPaths["/api/v1/catalog/search"]["get"];
type SearchSyncOperation = SearchPaths["/api/v1/ops/search/sync"]["get"];
type SearchReindexOperation = SearchPaths["/api/v1/ops/search/reindex"]["post"];
type SearchAliasSwitchOperation =
  SearchPaths["/api/v1/ops/search/aliases/switch"]["post"];
type SearchCacheInvalidateOperation =
  SearchPaths["/api/v1/ops/search/cache/invalidate"]["post"];
type SearchRankingProfilesOperation =
  SearchPaths["/api/v1/ops/search/ranking-profiles"]["get"];
type SearchRankingProfilePatchOperation =
  SearchPaths["/api/v1/ops/search/ranking-profiles/{id}"]["patch"];

export type SearchCatalogQuery = QueryParams<SearchCatalogOperation>;
export type SearchCatalogResponse = SuccessBody<SearchCatalogOperation>;
export type SearchSyncQuery = QueryParams<SearchSyncOperation>;
export type SearchSyncResponse = SuccessBody<SearchSyncOperation>;
export type SearchReindexRequest = RequestBody<SearchReindexOperation>;
export type SearchReindexResponse = SuccessBody<SearchReindexOperation>;
export type SearchAliasSwitchRequest =
  RequestBody<SearchAliasSwitchOperation>;
export type SearchAliasSwitchResponse =
  SuccessBody<SearchAliasSwitchOperation>;
export type SearchCacheInvalidateRequest =
  RequestBody<SearchCacheInvalidateOperation>;
export type SearchCacheInvalidateResponse =
  SuccessBody<SearchCacheInvalidateOperation>;
export type SearchRankingProfilesResponse =
  SuccessBody<SearchRankingProfilesOperation>;
export type SearchRankingProfilePatchPath =
  PathParams<SearchRankingProfilePatchOperation>;
export type SearchRankingProfilePatchRequest =
  RequestBody<SearchRankingProfilePatchOperation>;
export type SearchRankingProfilePatchResponse =
  SuccessBody<SearchRankingProfilePatchOperation>;

export interface SearchOpsWriteOptions {
  idempotencyKey: string;
  stepUpToken?: string;
  stepUpChallengeId?: string;
}

function searchOpsHeaders(options: SearchOpsWriteOptions): HeadersInit {
  const headers: Record<string, string> = {
    "X-Idempotency-Key": options.idempotencyKey,
  };
  if (options.stepUpToken) {
    headers["X-Step-Up-Token"] = options.stepUpToken;
  }
  if (options.stepUpChallengeId) {
    headers["X-Step-Up-Challenge-Id"] = options.stepUpChallengeId;
  }
  return headers;
}

function searchCacheHeaders(
  options: Pick<SearchOpsWriteOptions, "idempotencyKey">,
): HeaderParams<SearchCacheInvalidateOperation> {
  return {
    "X-Idempotency-Key": options.idempotencyKey,
  };
}

export function createSearchClient(client: PlatformClient) {
  return {
    searchCatalog(query: SearchCatalogQuery) {
      return client.getJson<SearchCatalogResponse, SearchCatalogQuery>(
        "/api/v1/catalog/search",
        { query },
      );
    },
    listSearchSync(query: SearchSyncQuery = {}) {
      return client.getJson<SearchSyncResponse, SearchSyncQuery>(
        "/api/v1/ops/search/sync",
        { query },
      );
    },
    reindex(
      body: SearchReindexRequest,
      options: SearchOpsWriteOptions,
    ) {
      return client.postJson<SearchReindexResponse, SearchReindexRequest>(
        "/api/v1/ops/search/reindex",
        {
          body,
          headers: searchOpsHeaders(options),
        },
      );
    },
    switchAlias(
      body: SearchAliasSwitchRequest,
      options: SearchOpsWriteOptions,
    ) {
      return client.postJson<
        SearchAliasSwitchResponse,
        SearchAliasSwitchRequest
      >("/api/v1/ops/search/aliases/switch", {
        body,
        headers: searchOpsHeaders(options),
      });
    },
    invalidateCache(
      body: SearchCacheInvalidateRequest,
      options: Pick<SearchOpsWriteOptions, "idempotencyKey">,
    ) {
      return client.postJson<
        SearchCacheInvalidateResponse,
        SearchCacheInvalidateRequest
      >("/api/v1/ops/search/cache/invalidate", {
        body,
        headers: searchCacheHeaders(options),
      });
    },
    listRankingProfiles() {
      return client.getJson<SearchRankingProfilesResponse>(
        "/api/v1/ops/search/ranking-profiles",
      );
    },
    patchRankingProfile(
      path: SearchRankingProfilePatchPath,
      body: SearchRankingProfilePatchRequest,
      options: SearchOpsWriteOptions,
    ) {
      return client.patchJson<
        SearchRankingProfilePatchResponse,
        SearchRankingProfilePatchRequest
      >("/api/v1/ops/search/ranking-profiles/{id}", {
        pathParams: path,
        body,
        headers: searchOpsHeaders(options),
      });
    },
  };
}
