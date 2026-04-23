import { describe, expect, it } from "vitest";

import {
  matchStandardScenariosForProduct,
  readStandardScenarioMappings,
} from "./standard-scenario-view";

describe("standard scenario view helpers", () => {
  it("falls back to the frozen five standard scenarios when live mappings are absent", () => {
    const scenarios = readStandardScenarioMappings(undefined);

    expect(scenarios).toHaveLength(5);
    expect(scenarios.map((item) => item.primary_sku)).toContain("QRY_LITE");
    expect(
      scenarios.find((item) => item.scenario_code === "S3")?.supplementary_skus,
    ).toEqual(["SHARE_RO"]);
  });

  it("keeps primary and supplementary sku roles distinct during product matching", () => {
    const scenarios = readStandardScenarioMappings(undefined);
    const coverage = matchStandardScenariosForProduct(
      [{ sku_type: "API_SUB" }, { sku_type: "API_PPU" }],
      scenarios,
    );

    expect(coverage).toHaveLength(2);
    expect(coverage.find((item) => item.scenario_code === "S1")).toMatchObject({
      primary_matched: true,
      supplementary_matched: ["API_PPU"],
      match_level: "primary_with_supplementary",
    });
    expect(coverage.find((item) => item.scenario_code === "S4")).toMatchObject({
      primary_matched: true,
      supplementary_matched: [],
      missing_skus: ["RPT_STD"],
      match_level: "primary_only",
    });
  });

  it("preserves supplementary-only hits instead of folding them back into main categories", () => {
    const scenarios = readStandardScenarioMappings(undefined);
    const coverage = matchStandardScenariosForProduct(
      [{ sku_type: "SHARE_RO" }],
      scenarios,
    );

    expect(coverage).toEqual([
      expect.objectContaining({
        scenario_code: "S3",
        primary_matched: false,
        matched_skus: ["SHARE_RO"],
        missing_skus: ["SBX_STD"],
        match_level: "supplementary_only",
      }),
    ]);
  });
});
