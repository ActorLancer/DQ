export interface PlatformClientConfig {
  baseUrl: string;
  fetch?: typeof fetch;
  defaultHeaders?: HeadersInit;
}

export interface RequestOptions<TQuery = Record<string, unknown>, TBody = unknown> {
  pathParams?: Record<string, string | number>;
  query?: TQuery;
  body?: TBody;
  headers?: HeadersInit;
  signal?: AbortSignal;
  cache?: RequestCache;
}

export interface PlatformErrorPayload {
  code?: string;
  message?: string;
  request_id?: string | null;
}

export class PlatformApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly requestId?: string | null;
  readonly payload?: unknown;

  constructor(
    status: number,
    payload: PlatformErrorPayload | string | undefined,
    fallbackMessage: string,
  ) {
    const message =
      typeof payload === "string"
        ? payload
        : payload?.message || fallbackMessage;
    super(message);
    this.name = "PlatformApiError";
    this.status = status;
    this.code =
      typeof payload === "string"
        ? "UNKNOWN"
        : payload?.code || "UNKNOWN";
    this.requestId =
      typeof payload === "string" ? undefined : payload?.request_id;
    this.payload = payload;
  }
}

export class PlatformClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: typeof fetch;
  private readonly defaultHeaders: HeadersInit;

  constructor(config: PlatformClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.fetchImpl =
      config.fetch ??
      ((input, init) => fetch(input, init));
    this.defaultHeaders = config.defaultHeaders ?? {};
  }

  async getJson<TResponse, TQuery = Record<string, unknown>>(
    pathTemplate: string,
    options: RequestOptions<TQuery> = {},
  ): Promise<TResponse> {
    return this.requestJson<TResponse, TQuery>(pathTemplate, "GET", options);
  }

  async postJson<TResponse, TBody = unknown, TQuery = Record<string, unknown>>(
    pathTemplate: string,
    options: RequestOptions<TQuery, TBody> = {},
  ): Promise<TResponse> {
    return this.requestJson<TResponse, TQuery, TBody>(
      pathTemplate,
      "POST",
      options,
    );
  }

  async patchJson<TResponse, TBody = unknown, TQuery = Record<string, unknown>>(
    pathTemplate: string,
    options: RequestOptions<TQuery, TBody> = {},
  ): Promise<TResponse> {
    return this.requestJson<TResponse, TQuery, TBody>(
      pathTemplate,
      "PATCH",
      options,
    );
  }

  async putJson<TResponse, TBody = unknown, TQuery = Record<string, unknown>>(
    pathTemplate: string,
    options: RequestOptions<TQuery, TBody> = {},
  ): Promise<TResponse> {
    return this.requestJson<TResponse, TQuery, TBody>(
      pathTemplate,
      "PUT",
      options,
    );
  }

  async deleteJson<TResponse, TBody = unknown, TQuery = Record<string, unknown>>(
    pathTemplate: string,
    options: RequestOptions<TQuery, TBody> = {},
  ): Promise<TResponse> {
    return this.requestJson<TResponse, TQuery, TBody>(
      pathTemplate,
      "DELETE",
      options,
    );
  }

  private async requestJson<
    TResponse,
    TQuery = Record<string, unknown>,
    TBody = unknown,
  >(
    pathTemplate: string,
    method: string,
    options: RequestOptions<TQuery, TBody>,
  ): Promise<TResponse> {
    const headers = new Headers(this.defaultHeaders);
    const requestHeaders = new Headers(options.headers);
    requestHeaders.forEach((value, key) => headers.set(key, value));
    if (!headers.has("x-request-id")) {
      headers.set("x-request-id", createRequestId());
    }

    const url = appendQuery(
      `${this.baseUrl}${compilePath(pathTemplate, options.pathParams)}`,
      options.query,
    );
    const init: RequestInit = {
      method,
      headers,
      signal: options.signal,
      cache: options.cache,
    };

    if (options.body !== undefined) {
      if (!headers.has("content-type")) {
        headers.set("content-type", "application/json");
      }
      init.body = JSON.stringify(options.body);
    }

    const response = await this.fetchImpl(url, init);
    const contentType = response.headers.get("content-type") ?? "";
    const parsedBody = contentType.includes("application/json")
      ? await response.json()
      : await response.text();

    if (!response.ok) {
      throw new PlatformApiError(
        response.status,
        parsedBody as PlatformErrorPayload | string | undefined,
        `Request failed for ${method} ${pathTemplate}`,
      );
    }

    return parsedBody as TResponse;
  }
}

export function compilePath(
  pathTemplate: string,
  pathParams?: Record<string, string | number>,
): string {
  if (!pathParams) {
    return pathTemplate;
  }

  return pathTemplate.replace(/\{([^}]+)\}/g, (_match, key: string) => {
    const value = pathParams[key];
    if (value === undefined || value === null) {
      throw new Error(`Missing path parameter: ${key}`);
    }
    return encodeURIComponent(String(value));
  });
}

export function appendQuery<TQuery>(
  input: string,
  query?: TQuery,
): string {
  if (!query) {
    return input;
  }

  const url = new URL(input, "http://local.invalid");
  const entries = Object.entries(query as Record<string, unknown>);
  for (const [key, value] of entries) {
    if (value === undefined || value === null || value === "") {
      continue;
    }

    if (Array.isArray(value)) {
      for (const item of value) {
        if (item !== undefined && item !== null && item !== "") {
          url.searchParams.append(key, String(item));
        }
      }
      continue;
    }

    url.searchParams.set(key, String(value));
  }

  const rendered = url.pathname + url.search;
  return rendered.startsWith("/") ? rendered : input;
}

export function createRequestId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `req-${crypto.randomUUID()}`;
  }
  return `req-${Date.now()}`;
}
