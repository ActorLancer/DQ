import { describe, expect, it } from "vitest";

import {
  consoleNavigationGroups,
  consoleRouteList,
  consoleRouteMap,
} from "./console-routes";

describe("console routes", () => {
  it("keeps route keys unique and route map complete", () => {
    const keys = consoleRouteList.map((route) => route.key);
    expect(new Set(keys).size).toBe(keys.length);
    expect(Object.keys(consoleRouteMap)).toHaveLength(consoleRouteList.length);
  });

  it("keeps navigation groups aligned with defined routes", () => {
    const knownKeys = new Set(consoleRouteList.map((route) => route.key));
    const groupedKeys = consoleNavigationGroups.flatMap((group) => group.keys);

    for (const key of groupedKeys) {
      expect(knownKeys.has(key)).toBe(true);
    }

    expect(new Set(groupedKeys).size).toBe(groupedKeys.length);
  });

  it("binds review routes to formal platform-core APIs", () => {
    expect(consoleRouteMap.review_subjects.apiBindings).toContain(
      "GET /api/v1/iam/orgs?status=pending_review",
    );
    expect(consoleRouteMap.review_products.apiBindings).toContain(
      "POST /api/v1/review/products/{id}",
    );
    expect(consoleRouteMap.review_compliance.apiBindings).toContain(
      "POST /api/v1/review/compliance/{id}",
    );
    expect(
      consoleRouteMap.review_subjects.apiBindings.some((binding) =>
        binding.includes("/api/v1/ops/review"),
      ),
    ).toBe(false);
  });
});
