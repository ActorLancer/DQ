import { PlatformClient } from "../core/http";
import type { PathParams, RequestBody, SuccessBody } from "../core/openapi";
import type { paths as BillingPaths } from "../generated/billing";

type GetBillingOrderOperation = NonNullable<
  BillingPaths["/api/v1/billing/{order_id}"]["get"]
>;
type ExecuteRefundOperation = NonNullable<
  BillingPaths["/api/v1/refunds"]["post"]
>;
type ExecuteCompensationOperation = NonNullable<
  BillingPaths["/api/v1/compensations"]["post"]
>;

export type BillingOrderDetailResponse =
  SuccessBody<GetBillingOrderOperation>;
export type ExecuteRefundRequest = RequestBody<ExecuteRefundOperation>;
export type ExecuteRefundResponse = SuccessBody<ExecuteRefundOperation>;
export type ExecuteCompensationRequest =
  RequestBody<ExecuteCompensationOperation>;
export type ExecuteCompensationResponse =
  SuccessBody<ExecuteCompensationOperation>;

export type BillingMutationOptions = {
  idempotencyKey: string;
  stepUpToken?: string;
  stepUpChallengeId?: string;
};

function mutationHeaders(options: BillingMutationOptions): HeadersInit {
  return {
    "X-Idempotency-Key": options.idempotencyKey,
    ...(options.stepUpToken ? { "X-Step-Up-Token": options.stepUpToken } : {}),
    ...(options.stepUpChallengeId
      ? { "X-Step-Up-Challenge-Id": options.stepUpChallengeId }
      : {}),
  };
}

export function createBillingClient(client: PlatformClient) {
  return {
    getBillingOrder(pathParams: PathParams<GetBillingOrderOperation>) {
      return client.getJson<BillingOrderDetailResponse>(
        "/api/v1/billing/{order_id}",
        { pathParams },
      );
    },
    executeRefund(
      body: ExecuteRefundRequest,
      options: BillingMutationOptions,
    ) {
      return client.postJson<ExecuteRefundResponse, ExecuteRefundRequest>(
        "/api/v1/refunds",
        {
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    executeCompensation(
      body: ExecuteCompensationRequest,
      options: BillingMutationOptions,
    ) {
      return client.postJson<
        ExecuteCompensationResponse,
        ExecuteCompensationRequest
      >("/api/v1/compensations", {
        body,
        headers: mutationHeaders(options),
      });
    },
  };
}
