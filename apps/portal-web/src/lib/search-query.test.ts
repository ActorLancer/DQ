import { describe, expect, it } from "vitest";

import {
  defaultSearchFormValues,
  formValuesToSearchQuery,
  formValuesToUrlSearchParams,
  pageFromSearchParams,
  parseSearchFormValues,
} from "./search-query";

describe("search query helpers", () => {
  it("parses URL params into safe form defaults", () => {
    const params = new URLSearchParams(
      "q=工业&entity_scope=product&tags=质量&tags=能耗&sort=quality&page_size=24",
    );

    expect(parseSearchFormValues(params)).toMatchObject({
      q: "工业",
      entity_scope: "product",
      tags: "质量, 能耗",
      sort: "quality",
      page_size: 24,
    });
  });

  it("falls back for unsupported enum params instead of inventing new contract values", () => {
    const params = new URLSearchParams("entity_scope=dataset&sort=custom&page=0");

    expect(parseSearchFormValues(params).entity_scope).toBe(
      defaultSearchFormValues.entity_scope,
    );
    expect(parseSearchFormValues(params).sort).toBe(defaultSearchFormValues.sort);
    expect(pageFromSearchParams(params)).toBe(1);
  });

  it("serializes only supported SDK query fields", () => {
    const query = formValuesToSearchQuery(
      {
        ...defaultSearchFormValues,
        q: "设备",
        entity_scope: "all",
        tags: "iot, 产线",
        price_min: "10.5",
        price_max: "99",
        sort: "price_asc",
      },
      2,
    );

    expect(query).toEqual({
      q: "设备",
      entity_scope: "all",
      tags: ["iot", "产线"],
      price_min: 10.5,
      price_max: 99,
      sort: "price_asc",
      page: 2,
      page_size: 12,
    });
    expect(formValuesToUrlSearchParams(defaultSearchFormValues).get("page")).toBe(
      "1",
    );
  });
});
