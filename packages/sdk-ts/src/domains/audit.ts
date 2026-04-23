import { PlatformClient } from "../core/http";
import type { QueryParams, SuccessBody } from "../core/openapi";
import type { paths as AuditPaths } from "../generated/audit";

type AuditTraceSearchOperation = AuditPaths["/api/v1/audit/traces"]["get"];

export type AuditTraceSearchQuery = QueryParams<AuditTraceSearchOperation>;
export type AuditTraceSearchResponse = SuccessBody<AuditTraceSearchOperation>;

export function createAuditClient(client: PlatformClient) {
  return {
    searchTraces(query: AuditTraceSearchQuery) {
      return client.getJson<AuditTraceSearchResponse, AuditTraceSearchQuery>(
        "/api/v1/audit/traces",
        {
          query,
        },
      );
    },
  };
}
