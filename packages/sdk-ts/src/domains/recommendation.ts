import { PlatformClient } from "../core/http";
import type {
  PathParams,
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
type RecommendationPlacementsOperation =
  RecommendationPaths["/api/v1/ops/recommendation/placements"]["get"];
type RecommendationPlacementPatchOperation =
  RecommendationPaths["/api/v1/ops/recommendation/placements/{placement_code}"]["patch"];
type RecommendationRankingProfilesOperation =
  RecommendationPaths["/api/v1/ops/recommendation/ranking-profiles"]["get"];
type RecommendationRankingProfilePatchOperation =
  RecommendationPaths["/api/v1/ops/recommendation/ranking-profiles/{id}"]["patch"];
type RecommendationRebuildOperation =
  RecommendationPaths["/api/v1/ops/recommendation/rebuild"]["post"];

export type RecommendationsQuery = QueryParams<RecommendationsOperation>;
export type RecommendationsResponse = SuccessBody<RecommendationsOperation>;
export type TrackExposureRequest = RequestBody<TrackExposureOperation>;
export type TrackExposureResponse = SuccessBody<TrackExposureOperation>;
export type TrackClickRequest = RequestBody<TrackClickOperation>;
export type TrackClickResponse = SuccessBody<TrackClickOperation>;
export type RecommendationPlacementsResponse =
  SuccessBody<RecommendationPlacementsOperation>;
export type RecommendationPlacementPatchPath =
  PathParams<RecommendationPlacementPatchOperation>;
export type RecommendationPlacementPatchRequest =
  RequestBody<RecommendationPlacementPatchOperation>;
export type RecommendationPlacementPatchResponse =
  SuccessBody<RecommendationPlacementPatchOperation>;
export type RecommendationRankingProfilesResponse =
  SuccessBody<RecommendationRankingProfilesOperation>;
export type RecommendationRankingProfilePatchPath =
  PathParams<RecommendationRankingProfilePatchOperation>;
export type RecommendationRankingProfilePatchRequest =
  RequestBody<RecommendationRankingProfilePatchOperation>;
export type RecommendationRankingProfilePatchResponse =
  SuccessBody<RecommendationRankingProfilePatchOperation>;
export type RecommendationRebuildRequest =
  RequestBody<RecommendationRebuildOperation>;
export type RecommendationRebuildResponse =
  SuccessBody<RecommendationRebuildOperation>;

export interface RecommendationOpsWriteOptions {
  idempotencyKey: string;
  stepUpToken?: string;
  stepUpChallengeId?: string;
}

function recommendationOpsHeaders(
  options: RecommendationOpsWriteOptions,
): HeadersInit {
  const headers: Record<string, string> = {
    "X-Idempotency-Key": options.idempotencyKey,
  };
  if (options.stepUpToken) {
    headers["X-Step-Up-Token"] = options.stepUpToken;
  }
  if (options.stepUpChallengeId) {
    headers["X-Step-Up-Challenge-Id"] = options.stepUpChallengeId;
  }
  return headers;
}

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
    listPlacements() {
      return client.getJson<RecommendationPlacementsResponse>(
        "/api/v1/ops/recommendation/placements",
      );
    },
    patchPlacement(
      path: RecommendationPlacementPatchPath,
      body: RecommendationPlacementPatchRequest,
      options: RecommendationOpsWriteOptions,
    ) {
      return client.patchJson<
        RecommendationPlacementPatchResponse,
        RecommendationPlacementPatchRequest
      >("/api/v1/ops/recommendation/placements/{placement_code}", {
        pathParams: path,
        body,
        headers: recommendationOpsHeaders(options),
      });
    },
    listRankingProfiles() {
      return client.getJson<RecommendationRankingProfilesResponse>(
        "/api/v1/ops/recommendation/ranking-profiles",
      );
    },
    patchRankingProfile(
      path: RecommendationRankingProfilePatchPath,
      body: RecommendationRankingProfilePatchRequest,
      options: RecommendationOpsWriteOptions,
    ) {
      return client.patchJson<
        RecommendationRankingProfilePatchResponse,
        RecommendationRankingProfilePatchRequest
      >("/api/v1/ops/recommendation/ranking-profiles/{id}", {
        pathParams: path,
        body,
        headers: recommendationOpsHeaders(options),
      });
    },
    rebuild(
      body: RecommendationRebuildRequest,
      options: RecommendationOpsWriteOptions,
    ) {
      return client.postJson<
        RecommendationRebuildResponse,
        RecommendationRebuildRequest
      >("/api/v1/ops/recommendation/rebuild", {
        body,
        headers: recommendationOpsHeaders(options),
      });
    },
  };
}
