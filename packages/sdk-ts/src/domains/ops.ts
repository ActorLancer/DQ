import { PlatformClient } from "../core/http";
import type { SuccessBody } from "../core/openapi";
import type { paths as OpsPaths } from "../generated/ops";

type HealthLiveOperation = OpsPaths["/health/live"]["get"];
type HealthReadyOperation = OpsPaths["/health/ready"]["get"];
type HealthDepsOperation = OpsPaths["/health/deps"]["get"];

export type HealthLiveResponse = SuccessBody<HealthLiveOperation>;
export type HealthReadyResponse = SuccessBody<HealthReadyOperation>;
export type HealthDepsResponse = SuccessBody<HealthDepsOperation>;

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
  };
}
