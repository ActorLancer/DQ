type JsonContent<T> = T extends {
  content: { "application/json": infer TBody };
}
  ? TBody
  : never;

type Responses<TOperation> = TOperation extends { responses: infer TResponses }
  ? TResponses
  : never;

type BodyByStatus<TResponses, TStatus extends string | number> = TStatus extends keyof TResponses
  ? JsonContent<TResponses[TStatus]>
  : never;

export type SuccessBody<TOperation> = TOperation extends { responses: infer TResponses }
  ? BodyByStatus<TResponses, 200> | BodyByStatus<TResponses, "200">
  : never;

export type RequestBody<TOperation> = TOperation extends {
  requestBody: {
    content: { "application/json": infer TBody };
  };
}
  ? TBody
  : never;

export type QueryParams<TOperation> = TOperation extends {
  parameters: { query?: infer TQuery };
}
  ? NonNullable<TQuery>
  : never;

export type PathParams<TOperation> = TOperation extends {
  parameters: { path?: infer TPath };
}
  ? NonNullable<TPath>
  : never;
