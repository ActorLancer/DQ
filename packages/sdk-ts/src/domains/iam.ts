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
