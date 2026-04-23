import { PlatformClient, PlatformApiError } from "./core/http";
import { createAuditClient } from "./domains/audit";
import { createCatalogClient } from "./domains/catalog";
import { createIamClient } from "./domains/iam";
import { createOpsClient } from "./domains/ops";
import { createRecommendationClient } from "./domains/recommendation";
import { createSearchClient } from "./domains/search";
import { createTradeClient } from "./domains/trade";

export type { PlatformClientConfig, RequestOptions } from "./core/http";
export { PlatformApiError, PlatformClient } from "./core/http";
export type {
  PathParams,
  QueryParams,
  RequestBody,
  SuccessBody,
} from "./core/openapi";
export * from "./domains/audit";
export * from "./domains/catalog";
export * from "./domains/iam";
export * from "./domains/ops";
export * from "./domains/recommendation";
export * from "./domains/search";
export * from "./domains/trade";

export function createDatabSdk(config: ConstructorParameters<typeof PlatformClient>[0]) {
  const client = new PlatformClient(config);

  return {
    client,
    audit: createAuditClient(client),
    iam: createIamClient(client),
    ops: createOpsClient(client),
    catalog: createCatalogClient(client),
    search: createSearchClient(client),
    recommendation: createRecommendationClient(client),
    trade: createTradeClient(client),
  };
}
