import { PlatformClient } from "../core/http";
import type { PathParams, RequestBody, SuccessBody } from "../core/openapi";
import type { paths as DeliveryPaths } from "../generated/delivery";

type CommitOrderDeliveryOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/deliver"]["post"]
>;
type AcceptOrderOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/accept"]["post"]
>;
type RejectOrderOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/reject"]["post"]
>;
type IssueDownloadTicketOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/download-ticket"]["get"]
>;
type ManageRevisionSubscriptionOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/subscriptions"]["post"]
>;
type GetRevisionSubscriptionOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/subscriptions"]["get"]
>;
type ManageShareGrantOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/share-grants"]["post"]
>;
type GetShareGrantsOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/share-grants"]["get"]
>;
type ManageTemplateGrantOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/template-grants"]["post"]
>;
type ExecuteTemplateRunOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/template-runs"]["post"]
>;
type ManageSandboxWorkspaceOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/sandbox-workspaces"]["post"]
>;
type GetQueryRunsOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/template-runs"]["get"]
>;
type GetApiUsageLogOperation = NonNullable<
  DeliveryPaths["/api/v1/orders/{id}/usage-log"]["get"]
>;
type ManageQuerySurfaceOperation = NonNullable<
  DeliveryPaths["/api/v1/products/{id}/query-surfaces"]["post"]
>;
type ManageQueryTemplateOperation = NonNullable<
  DeliveryPaths["/api/v1/query-surfaces/{id}/templates"]["post"]
>;

export type CommitOrderDeliveryRequest =
  RequestBody<CommitOrderDeliveryOperation>;
export type CommitOrderDeliveryResponse =
  SuccessBody<CommitOrderDeliveryOperation>;
export type AcceptOrderRequest = RequestBody<AcceptOrderOperation>;
export type AcceptOrderResponse = SuccessBody<AcceptOrderOperation>;
export type RejectOrderRequest = RequestBody<RejectOrderOperation>;
export type RejectOrderResponse = SuccessBody<RejectOrderOperation>;
export type DownloadTicketResponse =
  SuccessBody<IssueDownloadTicketOperation>;
export type ManageRevisionSubscriptionRequest =
  RequestBody<ManageRevisionSubscriptionOperation>;
export type ManageRevisionSubscriptionResponse =
  SuccessBody<ManageRevisionSubscriptionOperation>;
export type RevisionSubscriptionResponse =
  SuccessBody<GetRevisionSubscriptionOperation>;
export type ManageShareGrantRequest = RequestBody<ManageShareGrantOperation>;
export type ManageShareGrantResponse =
  SuccessBody<ManageShareGrantOperation>;
export type ShareGrantListResponse = SuccessBody<GetShareGrantsOperation>;
export type ManageTemplateGrantRequest =
  RequestBody<ManageTemplateGrantOperation>;
export type ManageTemplateGrantResponse =
  SuccessBody<ManageTemplateGrantOperation>;
export type ExecuteTemplateRunRequest =
  RequestBody<ExecuteTemplateRunOperation>;
export type ExecuteTemplateRunResponse =
  SuccessBody<ExecuteTemplateRunOperation>;
export type ManageSandboxWorkspaceRequest =
  RequestBody<ManageSandboxWorkspaceOperation>;
export type ManageSandboxWorkspaceResponse =
  SuccessBody<ManageSandboxWorkspaceOperation>;
export type QueryRunsResponse = SuccessBody<GetQueryRunsOperation>;
export type ApiUsageLogResponse = SuccessBody<GetApiUsageLogOperation>;
export type ManageQuerySurfaceRequest =
  RequestBody<ManageQuerySurfaceOperation>;
export type ManageQuerySurfaceResponse =
  SuccessBody<ManageQuerySurfaceOperation>;
export type ManageQueryTemplateRequest =
  RequestBody<ManageQueryTemplateOperation>;
export type ManageQueryTemplateResponse =
  SuccessBody<ManageQueryTemplateOperation>;

export type DeliveryMutationOptions = {
  idempotencyKey: string;
};

function mutationHeaders(options: DeliveryMutationOptions): HeadersInit {
  return {
    "X-Idempotency-Key": options.idempotencyKey,
  };
}

export function createDeliveryClient(client: PlatformClient) {
  return {
    commitOrderDelivery(
      pathParams: PathParams<CommitOrderDeliveryOperation>,
      body: CommitOrderDeliveryRequest,
      options: DeliveryMutationOptions,
    ) {
      return client.postJson<
        CommitOrderDeliveryResponse,
        CommitOrderDeliveryRequest
      >("/api/v1/orders/{id}/deliver", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    issueDownloadTicket(pathParams: PathParams<IssueDownloadTicketOperation>) {
      return client.getJson<DownloadTicketResponse>(
        "/api/v1/orders/{id}/download-ticket",
        { pathParams },
      );
    },
    acceptOrder(
      pathParams: PathParams<AcceptOrderOperation>,
      body: AcceptOrderRequest,
      options: DeliveryMutationOptions,
    ) {
      return client.postJson<AcceptOrderResponse, AcceptOrderRequest>(
        "/api/v1/orders/{id}/accept",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    rejectOrder(
      pathParams: PathParams<RejectOrderOperation>,
      body: RejectOrderRequest,
      options: DeliveryMutationOptions,
    ) {
      return client.postJson<RejectOrderResponse, RejectOrderRequest>(
        "/api/v1/orders/{id}/reject",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    manageRevisionSubscription(
      pathParams: PathParams<ManageRevisionSubscriptionOperation>,
      body: ManageRevisionSubscriptionRequest,
      options: DeliveryMutationOptions,
    ) {
      return client.postJson<
        ManageRevisionSubscriptionResponse,
        ManageRevisionSubscriptionRequest
      >("/api/v1/orders/{id}/subscriptions", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    getRevisionSubscription(
      pathParams: PathParams<GetRevisionSubscriptionOperation>,
    ) {
      return client.getJson<RevisionSubscriptionResponse>(
        "/api/v1/orders/{id}/subscriptions",
        { pathParams },
      );
    },
    manageShareGrant(
      pathParams: PathParams<ManageShareGrantOperation>,
      body: ManageShareGrantRequest,
      options: DeliveryMutationOptions,
    ) {
      return client.postJson<ManageShareGrantResponse, ManageShareGrantRequest>(
        "/api/v1/orders/{id}/share-grants",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    getShareGrants(pathParams: PathParams<GetShareGrantsOperation>) {
      return client.getJson<ShareGrantListResponse>(
        "/api/v1/orders/{id}/share-grants",
        { pathParams },
      );
    },
    manageTemplateGrant(
      pathParams: PathParams<ManageTemplateGrantOperation>,
      body: ManageTemplateGrantRequest,
      options: DeliveryMutationOptions,
    ) {
      return client.postJson<
        ManageTemplateGrantResponse,
        ManageTemplateGrantRequest
      >("/api/v1/orders/{id}/template-grants", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    executeTemplateRun(
      pathParams: PathParams<ExecuteTemplateRunOperation>,
      body: ExecuteTemplateRunRequest,
      options: DeliveryMutationOptions,
    ) {
      return client.postJson<
        ExecuteTemplateRunResponse,
        ExecuteTemplateRunRequest
      >("/api/v1/orders/{id}/template-runs", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    manageSandboxWorkspace(
      pathParams: PathParams<ManageSandboxWorkspaceOperation>,
      body: ManageSandboxWorkspaceRequest,
      options: DeliveryMutationOptions,
    ) {
      return client.postJson<
        ManageSandboxWorkspaceResponse,
        ManageSandboxWorkspaceRequest
      >("/api/v1/orders/{id}/sandbox-workspaces", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    getQueryRuns(pathParams: PathParams<GetQueryRunsOperation>) {
      return client.getJson<QueryRunsResponse>(
        "/api/v1/orders/{id}/template-runs",
        { pathParams },
      );
    },
    getApiUsageLog(pathParams: PathParams<GetApiUsageLogOperation>) {
      return client.getJson<ApiUsageLogResponse>(
        "/api/v1/orders/{id}/usage-log",
        { pathParams },
      );
    },
    manageQuerySurface(
      pathParams: PathParams<ManageQuerySurfaceOperation>,
      body: ManageQuerySurfaceRequest,
    ) {
      return client.postJson<
        ManageQuerySurfaceResponse,
        ManageQuerySurfaceRequest
      >("/api/v1/products/{id}/query-surfaces", {
        pathParams,
        body,
      });
    },
    manageQueryTemplate(
      pathParams: PathParams<ManageQueryTemplateOperation>,
      body: ManageQueryTemplateRequest,
    ) {
      return client.postJson<
        ManageQueryTemplateResponse,
        ManageQueryTemplateRequest
      >("/api/v1/query-surfaces/{id}/templates", {
        pathParams,
        body,
      });
    },
  };
}
