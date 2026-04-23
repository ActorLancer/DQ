import { PlatformClient } from "../core/http";
import type {
  PathParams,
  QueryParams,
  RequestBody,
  SuccessBody,
} from "../core/openapi";
import type { paths as OpsPaths } from "../generated/ops";

type HealthLiveOperation = OpsPaths["/health/live"]["get"];
type HealthReadyOperation = OpsPaths["/health/ready"]["get"];
type HealthDepsOperation = OpsPaths["/health/deps"]["get"];
type DeveloperTraceOperation = OpsPaths["/api/v1/developer/trace"]["get"];
type OutboxOperation = OpsPaths["/api/v1/ops/outbox"]["get"];
type DeadLettersOperation = OpsPaths["/api/v1/ops/dead-letters"]["get"];
type DeadLetterReprocessOperation =
  OpsPaths["/api/v1/ops/dead-letters/{id}/reprocess"]["post"];
type ObservabilityOverviewOperation =
  OpsPaths["/api/v1/ops/observability/overview"]["get"];
type TradeMonitorOverviewOperation =
  OpsPaths["/api/v1/ops/trade-monitor/orders/{orderId}"]["get"];
type TradeMonitorCheckpointsOperation =
  OpsPaths["/api/v1/ops/trade-monitor/orders/{orderId}/checkpoints"]["get"];
type ExternalFactsOperation = OpsPaths["/api/v1/ops/external-facts"]["get"];
type ProjectionGapsOperation = OpsPaths["/api/v1/ops/projection-gaps"]["get"];
type ConsistencyOperation =
  OpsPaths["/api/v1/ops/consistency/{refType}/{refId}"]["get"];
type ConsistencyReconcileOperation =
  OpsPaths["/api/v1/ops/consistency/reconcile"]["post"];

export type HealthLiveResponse = SuccessBody<HealthLiveOperation>;
export type HealthReadyResponse = SuccessBody<HealthReadyOperation>;
export type HealthDepsResponse = SuccessBody<HealthDepsOperation>;
export type DeveloperTraceQuery = QueryParams<DeveloperTraceOperation>;
export type DeveloperTraceResponse = SuccessBody<DeveloperTraceOperation>;
export type OutboxQuery = QueryParams<OutboxOperation>;
export type OutboxResponse = SuccessBody<OutboxOperation>;
export type DeadLettersQuery = QueryParams<DeadLettersOperation>;
export type DeadLettersResponse = SuccessBody<DeadLettersOperation>;
export type DeadLetterReprocessPath =
  PathParams<DeadLetterReprocessOperation>;
export type DeadLetterReprocessRequest =
  RequestBody<DeadLetterReprocessOperation>;
export type DeadLetterReprocessResponse =
  SuccessBody<DeadLetterReprocessOperation>;
export type ObservabilityOverviewResponse =
  SuccessBody<ObservabilityOverviewOperation>;
export type TradeMonitorOverviewPath =
  PathParams<TradeMonitorOverviewOperation>;
export type TradeMonitorOverviewResponse =
  SuccessBody<TradeMonitorOverviewOperation>;
export type TradeMonitorCheckpointsPath =
  PathParams<TradeMonitorCheckpointsOperation>;
export type TradeMonitorCheckpointsQuery =
  QueryParams<TradeMonitorCheckpointsOperation>;
export type TradeMonitorCheckpointsResponse =
  SuccessBody<TradeMonitorCheckpointsOperation>;
export type ExternalFactsQuery = QueryParams<ExternalFactsOperation>;
export type ExternalFactsResponse = SuccessBody<ExternalFactsOperation>;
export type ProjectionGapsQuery = QueryParams<ProjectionGapsOperation>;
export type ProjectionGapsResponse = SuccessBody<ProjectionGapsOperation>;
export type ConsistencyPath = PathParams<ConsistencyOperation>;
export type ConsistencyResponse = SuccessBody<ConsistencyOperation>;
export type ConsistencyReconcileRequest =
  RequestBody<ConsistencyReconcileOperation>;
export type ConsistencyReconcileResponse =
  SuccessBody<ConsistencyReconcileOperation>;

export interface OpsControlPlaneWriteOptions {
  idempotencyKey: string;
  stepUpToken?: string;
  stepUpChallengeId?: string;
}

function controlPlaneHeaders(options: OpsControlPlaneWriteOptions): HeadersInit {
  const headers: Record<string, string> = {
    "x-idempotency-key": options.idempotencyKey,
  };
  if (options.stepUpToken) {
    headers["x-step-up-token"] = options.stepUpToken;
  }
  if (options.stepUpChallengeId) {
    headers["x-step-up-challenge-id"] = options.stepUpChallengeId;
  }
  return headers;
}

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
    listDeadLetters(query: DeadLettersQuery = {}) {
      return client.getJson<DeadLettersResponse, DeadLettersQuery>(
        "/api/v1/ops/dead-letters",
        {
          query,
        },
      );
    },
    reprocessDeadLetter(
      path: DeadLetterReprocessPath,
      body: DeadLetterReprocessRequest,
      options: OpsControlPlaneWriteOptions,
    ) {
      return client.postJson<
        DeadLetterReprocessResponse,
        DeadLetterReprocessRequest
      >("/api/v1/ops/dead-letters/{id}/reprocess", {
        pathParams: path,
        body,
        headers: controlPlaneHeaders(options),
      });
    },
    getObservabilityOverview() {
      return client.getJson<ObservabilityOverviewResponse>(
        "/api/v1/ops/observability/overview",
      );
    },
    getTradeMonitorOverview(path: TradeMonitorOverviewPath) {
      return client.getJson<TradeMonitorOverviewResponse>(
        "/api/v1/ops/trade-monitor/orders/{orderId}",
        {
          pathParams: path,
        },
      );
    },
    listTradeMonitorCheckpoints(
      path: TradeMonitorCheckpointsPath,
      query: TradeMonitorCheckpointsQuery = {},
    ) {
      return client.getJson<
        TradeMonitorCheckpointsResponse,
        TradeMonitorCheckpointsQuery
      >("/api/v1/ops/trade-monitor/orders/{orderId}/checkpoints", {
        pathParams: path,
        query,
      });
    },
    listExternalFacts(query: ExternalFactsQuery = {}) {
      return client.getJson<ExternalFactsResponse, ExternalFactsQuery>(
        "/api/v1/ops/external-facts",
        {
          query,
        },
      );
    },
    listProjectionGaps(query: ProjectionGapsQuery = {}) {
      return client.getJson<ProjectionGapsResponse, ProjectionGapsQuery>(
        "/api/v1/ops/projection-gaps",
        {
          query,
        },
      );
    },
    getConsistency(path: ConsistencyPath) {
      return client.getJson<ConsistencyResponse>(
        "/api/v1/ops/consistency/{refType}/{refId}",
        {
          pathParams: path,
        },
      );
    },
    reconcileConsistency(
      body: ConsistencyReconcileRequest,
      options: OpsControlPlaneWriteOptions,
    ) {
      return client.postJson<
        ConsistencyReconcileResponse,
        ConsistencyReconcileRequest
      >("/api/v1/ops/consistency/reconcile", {
        body,
        headers: controlPlaneHeaders(options),
      });
    },
  };
}
