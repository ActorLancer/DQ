import { PlatformClient } from "../core/http";
import type { SuccessBody } from "../core/openapi";
import type { operations as TradeOperations } from "../generated/trade";

export type StandardOrderTemplatesResponse =
  SuccessBody<TradeOperations["listStandardOrderTemplates"]>;

export function createTradeClient(client: PlatformClient) {
  return {
    listStandardOrderTemplates() {
      return client.getJson<StandardOrderTemplatesResponse>(
        "/api/v1/orders/standard-templates",
      );
    },
  };
}
