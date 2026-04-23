import { PlatformClient } from "../core/http";
import type {
  PathParams,
  QueryParams,
  RequestBody,
  SuccessBody,
} from "../core/openapi";
import type { operations as IamOperations } from "../generated/iam";

export type OrganizationListResponse =
  SuccessBody<IamOperations["listOrganizations"]>;
export type OrganizationResponse =
  SuccessBody<IamOperations["getOrganization"]>;
export type LoginRequest = RequestBody<IamOperations["authLogin"]>;
export type LoginResponse = SuccessBody<IamOperations["authLogin"]>;
export type LogoutRequest = RequestBody<IamOperations["authLogout"]>;
export type LogoutResponse = SuccessBody<IamOperations["authLogout"]>;
export type AuthMeResponse = SuccessBody<IamOperations["getAuthMe"]>;
export type OrganizationListQuery =
  QueryParams<IamOperations["listOrganizations"]>;
export type ApplicationListQuery =
  QueryParams<IamOperations["listApplications"]>;
export type ApplicationListResponse =
  SuccessBody<IamOperations["listApplications"]>;
export type ApplicationResponse =
  SuccessBody<IamOperations["getApplication"]>;
export type CreateApplicationRequest =
  RequestBody<IamOperations["createApplication"]>;
export type PatchApplicationRequest =
  RequestBody<IamOperations["patchApplication"]>;
export type RotateApplicationSecretRequest =
  RequestBody<IamOperations["rotateApplicationSecret"]>;

export interface IamMutationOptions {
  idempotencyKey: string;
  stepUpToken?: string;
  stepUpChallengeId?: string;
}

function mutationHeaders(options: IamMutationOptions): HeadersInit {
  return {
    "X-Idempotency-Key": options.idempotencyKey,
    ...(options.stepUpToken ? { "X-Step-Up-Token": options.stepUpToken } : {}),
    ...(options.stepUpChallengeId
      ? { "X-Step-Up-Challenge-Id": options.stepUpChallengeId }
      : {}),
  };
}

export function createIamClient(client: PlatformClient) {
  return {
    listOrganizations(query?: OrganizationListQuery) {
      return client.getJson<OrganizationListResponse, OrganizationListQuery>(
        "/api/v1/iam/orgs",
        { query },
      );
    },
    getOrganization(pathParams: PathParams<IamOperations["getOrganization"]>) {
      return client.getJson<OrganizationResponse>("/api/v1/iam/orgs/{id}", {
        pathParams,
      });
    },
    listApplications(query?: ApplicationListQuery) {
      return client.getJson<ApplicationListResponse, ApplicationListQuery>(
        "/api/v1/apps",
        { query },
      );
    },
    createApplication(
      body: CreateApplicationRequest,
      options: IamMutationOptions,
    ) {
      return client.postJson<ApplicationResponse, CreateApplicationRequest>(
        "/api/v1/apps",
        {
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    getApplication(pathParams: PathParams<IamOperations["getApplication"]>) {
      return client.getJson<ApplicationResponse>("/api/v1/apps/{id}", {
        pathParams,
      });
    },
    patchApplication(
      pathParams: PathParams<IamOperations["patchApplication"]>,
      body: PatchApplicationRequest,
      options: IamMutationOptions,
    ) {
      return client.patchJson<ApplicationResponse, PatchApplicationRequest>(
        "/api/v1/apps/{id}",
        {
          pathParams,
          body,
          headers: mutationHeaders(options),
        },
      );
    },
    rotateApplicationSecret(
      pathParams: PathParams<IamOperations["rotateApplicationSecret"]>,
      body: RotateApplicationSecretRequest,
      options: IamMutationOptions,
    ) {
      return client.postJson<
        ApplicationResponse,
        RotateApplicationSecretRequest
      >("/api/v1/apps/{id}/credentials/rotate", {
        pathParams,
        body,
        headers: mutationHeaders(options),
      });
    },
    revokeApplicationSecret(
      pathParams: PathParams<IamOperations["revokeApplicationSecret"]>,
      options: IamMutationOptions,
    ) {
      return client.postJson<ApplicationResponse, undefined>(
        "/api/v1/apps/{id}/credentials/revoke",
        {
          pathParams,
          headers: mutationHeaders(options),
        },
      );
    },
    getAuthMe() {
      return client.getJson<AuthMeResponse>("/api/v1/auth/me");
    },
    login(body: LoginRequest) {
      return client.postJson<LoginResponse, LoginRequest>("/api/v1/auth/login", {
        body,
      });
    },
    logout(body: LogoutRequest) {
      return client.postJson<LogoutResponse, LogoutRequest>(
        "/api/v1/auth/logout",
        { body },
      );
    },
  };
}
