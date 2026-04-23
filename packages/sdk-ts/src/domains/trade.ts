import { PlatformClient } from "../core/http";
import type { PathParams, RequestBody, SuccessBody } from "../core/openapi";
import type { operations as TradeOperations } from "../generated/trade";

export type StandardOrderTemplatesResponse =
  SuccessBody<TradeOperations["listStandardOrderTemplates"]>;
export type CreateOrderRequest =
  RequestBody<TradeOperations["createOrder"]>;
export type CreateOrderResponse = SuccessBody<TradeOperations["createOrder"]>;
export type OrderDetailResponse =
  SuccessBody<TradeOperations["getOrderDetail"]>;
export type OrderLifecycleSnapshotsResponse =
  SuccessBody<TradeOperations["getOrderLifecycleSnapshots"]>;
export type CancelOrderResponse =
  SuccessBody<TradeOperations["cancelOrder"]>;

export type TradeMutationOptions = {
  idempotencyKey: string;
};

function mutationHeaders(options: TradeMutationOptions): HeadersInit {
  return {
    "X-Idempotency-Key": options.idempotencyKey,
  };
}

export function createTradeClient(client: PlatformClient) {
  return {
    listStandardOrderTemplates() {
      return client.getJson<StandardOrderTemplatesResponse>(
        "/api/v1/orders/standard-templates",
      );
    },
    createOrder(body: CreateOrderRequest, options: TradeMutationOptions) {
      return client.postJson<CreateOrderResponse, CreateOrderRequest>(
        "/api/v1/orders",
        {
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    getOrderDetail(pathParams: PathParams<TradeOperations["getOrderDetail"]>) {
      return client.getJson<OrderDetailResponse>("/api/v1/orders/{id}", {
        pathParams,
      });
    },
    getOrderLifecycleSnapshots(
      pathParams: PathParams<TradeOperations["getOrderLifecycleSnapshots"]>,
    ) {
      return client.getJson<OrderLifecycleSnapshotsResponse>(
        "/api/v1/orders/{id}/lifecycle-snapshots",
        { pathParams },
      );
    },
    cancelOrder(
      pathParams: PathParams<TradeOperations["cancelOrder"]>,
      options: TradeMutationOptions,
    ) {
      return client.postJson<CancelOrderResponse>(
        "/api/v1/orders/{id}/cancel",
        {
          pathParams,
          headers: mutationHeaders(options),
        },
      );
    },
  };
}
