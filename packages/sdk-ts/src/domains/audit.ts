import { PlatformClient } from "../core/http";
import type {
  HeaderParams,
  PathParams,
  QueryParams,
  RequestBody,
  SuccessBody,
} from "../core/openapi";
import type { paths as AuditPaths } from "../generated/audit";

type OrderAuditOperation = AuditPaths["/api/v1/audit/orders/{id}"]["get"];
type AuditTraceSearchOperation = AuditPaths["/api/v1/audit/traces"]["get"];
type AuditPackageExportOperation =
  AuditPaths["/api/v1/audit/packages/export"]["post"];
type AuditAnchorBatchListOperation =
  AuditPaths["/api/v1/audit/anchor-batches"]["get"];

export type OrderAuditPath = PathParams<OrderAuditOperation>;
export type OrderAuditQuery = QueryParams<OrderAuditOperation>;
export type OrderAuditResponse = SuccessBody<OrderAuditOperation>;
export type AuditTraceSearchQuery = QueryParams<AuditTraceSearchOperation>;
export type AuditTraceSearchResponse = SuccessBody<AuditTraceSearchOperation>;
export type AuditPackageExportRequest =
  RequestBody<AuditPackageExportOperation>;
export type AuditPackageExportHeaders =
  HeaderParams<AuditPackageExportOperation>;
export type AuditPackageExportResponse =
  SuccessBody<AuditPackageExportOperation>;
export type AuditAnchorBatchListQuery =
  QueryParams<AuditAnchorBatchListOperation>;
export type AuditAnchorBatchListResponse =
  SuccessBody<AuditAnchorBatchListOperation>;

export interface AuditPackageExportOptions {
  idempotencyKey: string;
  stepUpToken?: string;
  stepUpChallengeId?: string;
}

export function createAuditClient(client: PlatformClient) {
  return {
    getOrderAudit(
      path: OrderAuditPath,
      query: OrderAuditQuery = {},
    ) {
      return client.getJson<OrderAuditResponse, OrderAuditQuery>(
        "/api/v1/audit/orders/{id}",
        {
          pathParams: path,
          query,
        },
      );
    },
    searchTraces(query: AuditTraceSearchQuery) {
      return client.getJson<AuditTraceSearchResponse, AuditTraceSearchQuery>(
        "/api/v1/audit/traces",
        {
          query,
        },
      );
    },
    exportPackage(
      body: AuditPackageExportRequest,
      options: AuditPackageExportOptions,
    ) {
      const headers: AuditPackageExportHeaders = {
        "x-idempotency-key": options.idempotencyKey,
      };
      if (options.stepUpToken) {
        headers["x-step-up-token"] = options.stepUpToken;
      }
      if (options.stepUpChallengeId) {
        headers["x-step-up-challenge-id"] = options.stepUpChallengeId;
      }

      return client.postJson<
        AuditPackageExportResponse,
        AuditPackageExportRequest
      >("/api/v1/audit/packages/export", {
        body,
        headers,
      });
    },
    listAnchorBatches(query: AuditAnchorBatchListQuery = {}) {
      return client.getJson<
        AuditAnchorBatchListResponse,
        AuditAnchorBatchListQuery
      >("/api/v1/audit/anchor-batches", {
        query,
      });
    },
  };
}
