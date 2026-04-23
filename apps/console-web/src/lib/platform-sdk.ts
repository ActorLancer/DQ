import { createDatabSdk } from "@datab/sdk-ts";

import { getPlatformCoreBaseUrl } from "./env";

export function createBrowserSdk() {
  return createDatabSdk({
    baseUrl: "/api/platform",
  });
}

export function createServerSdk(headers?: HeadersInit) {
  return createDatabSdk({
    baseUrl: getPlatformCoreBaseUrl(),
    defaultHeaders: headers,
  });
}
