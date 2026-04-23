import { PlatformClient } from "../core/http";
import type {
  MultipartBody,
  PathParams,
  RequestBody,
  SuccessBody,
} from "../core/openapi";
import type { paths as BillingPaths } from "../generated/billing";

type GetBillingOrderOperation = NonNullable<
  BillingPaths["/api/v1/billing/{order_id}"]["get"]
>;
type CreatePaymentIntentOperation = NonNullable<
  BillingPaths["/api/v1/payments/intents"]["post"]
>;
type GetPaymentIntentOperation = NonNullable<
  BillingPaths["/api/v1/payments/intents/{id}"]["get"]
>;
type LockOrderPaymentOperation = NonNullable<
  BillingPaths["/api/v1/orders/{id}/lock"]["post"]
>;
type ExecuteRefundOperation = NonNullable<
  BillingPaths["/api/v1/refunds"]["post"]
>;
type ExecuteCompensationOperation = NonNullable<
  BillingPaths["/api/v1/compensations"]["post"]
>;
type CreateDisputeCaseOperation = NonNullable<
  BillingPaths["/api/v1/cases"]["post"]
>;
type UploadDisputeEvidenceOperation = NonNullable<
  BillingPaths["/api/v1/cases/{id}/evidence"]["post"]
>;
type ResolveDisputeCaseOperation = NonNullable<
  BillingPaths["/api/v1/cases/{id}/resolve"]["post"]
>;
type MockPaymentSuccessOperation = NonNullable<
  BillingPaths["/api/v1/mock/payments/{id}/simulate-success"]["post"]
>;
type MockPaymentFailOperation = NonNullable<
  BillingPaths["/api/v1/mock/payments/{id}/simulate-fail"]["post"]
>;
type MockPaymentTimeoutOperation = NonNullable<
  BillingPaths["/api/v1/mock/payments/{id}/simulate-timeout"]["post"]
>;

export type BillingOrderDetailResponse =
  SuccessBody<GetBillingOrderOperation>;
export type CreatePaymentIntentRequest =
  RequestBody<CreatePaymentIntentOperation>;
export type CreatePaymentIntentResponse =
  SuccessBody<CreatePaymentIntentOperation>;
export type PaymentIntentDetailResponse =
  SuccessBody<GetPaymentIntentOperation>;
export type LockOrderPaymentRequest =
  RequestBody<LockOrderPaymentOperation>;
export type LockOrderPaymentResponse =
  SuccessBody<LockOrderPaymentOperation>;
export type ExecuteRefundRequest = RequestBody<ExecuteRefundOperation>;
export type ExecuteRefundResponse = SuccessBody<ExecuteRefundOperation>;
export type ExecuteCompensationRequest =
  RequestBody<ExecuteCompensationOperation>;
export type ExecuteCompensationResponse =
  SuccessBody<ExecuteCompensationOperation>;
export type CreateDisputeCaseRequest =
  RequestBody<CreateDisputeCaseOperation>;
export type CreateDisputeCaseResponse =
  SuccessBody<CreateDisputeCaseOperation>;
export type UploadDisputeEvidenceMultipartRequest =
  MultipartBody<UploadDisputeEvidenceOperation>;
export type UploadDisputeEvidenceResponse =
  SuccessBody<UploadDisputeEvidenceOperation>;
export type ResolveDisputeCaseRequest =
  RequestBody<ResolveDisputeCaseOperation>;
export type ResolveDisputeCaseResponse =
  SuccessBody<ResolveDisputeCaseOperation>;
export type MockPaymentSimulationPath =
  PathParams<MockPaymentSuccessOperation>;
export type MockPaymentSimulationRequest =
  RequestBody<MockPaymentSuccessOperation>;
export type MockPaymentSimulationResponse =
  SuccessBody<MockPaymentSuccessOperation>;

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
    createPaymentIntent(
      body: CreatePaymentIntentRequest,
      options: BillingMutationOptions,
    ) {
      return client.postJson<
        CreatePaymentIntentResponse,
        CreatePaymentIntentRequest
      >("/api/v1/payments/intents", {
        body,
        headers: mutationHeaders(options),
      });
    },
    getPaymentIntent(pathParams: PathParams<GetPaymentIntentOperation>) {
      return client.getJson<PaymentIntentDetailResponse>(
        "/api/v1/payments/intents/{id}",
        { pathParams },
      );
    },
    lockOrderPayment(
      pathParams: PathParams<LockOrderPaymentOperation>,
      body: LockOrderPaymentRequest,
    ) {
      return client.postJson<LockOrderPaymentResponse, LockOrderPaymentRequest>(
        "/api/v1/orders/{id}/lock",
        {
          pathParams,
          body,
        },
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
    createDisputeCase(
      body: CreateDisputeCaseRequest,
      options: BillingMutationOptions,
    ) {
      return client.postJson<CreateDisputeCaseResponse, CreateDisputeCaseRequest>(
        "/api/v1/cases",
        {
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    uploadDisputeEvidence(
      pathParams: PathParams<UploadDisputeEvidenceOperation>,
      body: FormData,
      options: BillingMutationOptions,
    ) {
      return client.postFormData<UploadDisputeEvidenceResponse>(
        "/api/v1/cases/{id}/evidence",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    resolveDisputeCase(
      pathParams: PathParams<ResolveDisputeCaseOperation>,
      body: ResolveDisputeCaseRequest,
      options: BillingMutationOptions,
    ) {
      return client.postJson<
        ResolveDisputeCaseResponse,
        ResolveDisputeCaseRequest
      >("/api/v1/cases/{id}/resolve", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    simulateMockPaymentSuccess(
      pathParams: PathParams<MockPaymentSuccessOperation>,
      body: RequestBody<MockPaymentSuccessOperation>,
      options: BillingMutationOptions,
    ) {
      return client.postJson<
        SuccessBody<MockPaymentSuccessOperation>,
        RequestBody<MockPaymentSuccessOperation>
      >("/api/v1/mock/payments/{id}/simulate-success", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    simulateMockPaymentFail(
      pathParams: PathParams<MockPaymentFailOperation>,
      body: RequestBody<MockPaymentFailOperation>,
      options: BillingMutationOptions,
    ) {
      return client.postJson<
        SuccessBody<MockPaymentFailOperation>,
        RequestBody<MockPaymentFailOperation>
      >("/api/v1/mock/payments/{id}/simulate-fail", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    simulateMockPaymentTimeout(
      pathParams: PathParams<MockPaymentTimeoutOperation>,
      body: RequestBody<MockPaymentTimeoutOperation>,
      options: BillingMutationOptions,
    ) {
      return client.postJson<
        SuccessBody<MockPaymentTimeoutOperation>,
        RequestBody<MockPaymentTimeoutOperation>
      >("/api/v1/mock/payments/{id}/simulate-timeout", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
  };
}
