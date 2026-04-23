import { PlatformClient } from "../core/http";
import type {
  QueryParams,
  RequestBody,
  SuccessBody,
} from "../core/openapi";
import type { paths as RecommendationPaths } from "../generated/recommendation";

type RecommendationsOperation =
  RecommendationPaths["/api/v1/recommendations"]["get"];
type TrackExposureOperation =
  RecommendationPaths["/api/v1/recommendations/track/exposure"]["post"];
type TrackClickOperation =
  RecommendationPaths["/api/v1/recommendations/track/click"]["post"];

export type RecommendationsQuery = QueryParams<RecommendationsOperation>;
export type RecommendationsResponse = SuccessBody<RecommendationsOperation>;
export type TrackExposureRequest = RequestBody<TrackExposureOperation>;
export type TrackExposureResponse = SuccessBody<TrackExposureOperation>;
export type TrackClickRequest = RequestBody<TrackClickOperation>;
export type TrackClickResponse = SuccessBody<TrackClickOperation>;

export function createRecommendationClient(client: PlatformClient) {
  return {
    getRecommendations(query: RecommendationsQuery) {
      return client.getJson<RecommendationsResponse, RecommendationsQuery>(
        "/api/v1/recommendations",
        { query },
      );
    },
    trackExposure(body: TrackExposureRequest, idempotencyKey: string) {
      return client.postJson<TrackExposureResponse, TrackExposureRequest>(
        "/api/v1/recommendations/track/exposure",
        {
          body,
          headers: {
            "x-idempotency-key": idempotencyKey,
          },
        },
      );
    },
    trackClick(body: TrackClickRequest, idempotencyKey: string) {
      return client.postJson<TrackClickResponse, TrackClickRequest>(
        "/api/v1/recommendations/track/click",
        {
          body,
          headers: {
            "x-idempotency-key": idempotencyKey,
          },
        },
      );
    },
  };
}
