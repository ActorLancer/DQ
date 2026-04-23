import { PlatformClient } from "../core/http";
import type { QueryParams, SuccessBody } from "../core/openapi";
import type { paths as OpsPaths } from "../generated/ops";

type HealthLiveOperation = OpsPaths["/health/live"]["get"];
type HealthReadyOperation = OpsPaths["/health/ready"]["get"];
type HealthDepsOperation = OpsPaths["/health/deps"]["get"];
type DeveloperTraceOperation = OpsPaths["/api/v1/developer/trace"]["get"];
type OutboxOperation = OpsPaths["/api/v1/ops/outbox"]["get"];
type ObservabilityOverviewOperation =
  OpsPaths["/api/v1/ops/observability/overview"]["get"];

export type HealthLiveResponse = SuccessBody<HealthLiveOperation>;
export type HealthReadyResponse = SuccessBody<HealthReadyOperation>;
export type HealthDepsResponse = SuccessBody<HealthDepsOperation>;
export type DeveloperTraceQuery = QueryParams<DeveloperTraceOperation>;
export type DeveloperTraceResponse = SuccessBody<DeveloperTraceOperation>;
export type OutboxQuery = QueryParams<OutboxOperation>;
export type OutboxResponse = SuccessBody<OutboxOperation>;
export type ObservabilityOverviewResponse =
  SuccessBody<ObservabilityOverviewOperation>;

export function createOpsClient(client: PlatformClient) {
  return {
    healthLive() {
      return client.getJson<HealthLiveResponse>("/health/live");
    },
    healthReady() {
      return client.getJson<HealthReadyResponse>("/health/ready");
    },
    healthDeps() {
      return client.getJson<HealthDepsResponse>("/health/deps");
    },
    getDeveloperTrace(query: DeveloperTraceQuery) {
      return client.getJson<DeveloperTraceResponse, DeveloperTraceQuery>(
        "/api/v1/developer/trace",
        {
          query,
        },
      );
    },
    listOutbox(query: OutboxQuery = {}) {
      return client.getJson<OutboxResponse, OutboxQuery>(
        "/api/v1/ops/outbox",
        {
          query,
        },
      );
    },
    getObservabilityOverview() {
      return client.getJson<ObservabilityOverviewResponse>(
        "/api/v1/ops/observability/overview",
      );
    },
  };
}
