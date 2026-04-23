import type { StandardScenariosResponse } from "@datab/sdk-ts";

import { frozenStandardScenarios } from "./standard-demo";

export type StandardScenarioSkuMapping = Pick<
  StandardScenariosResponse["data"][number],
  "scenario_code" | "scenario_name" | "primary_sku" | "supplementary_skus"
>;

type ProductSkuLike = {
  sku_type: string;
};

export type StandardScenarioCoverage = StandardScenarioSkuMapping & {
  primary_matched: boolean;
  supplementary_matched: string[];
  supplementary_missing: string[];
  matched_skus: string[];
  missing_skus: string[];
  match_level:
    | "primary_with_supplementary"
    | "primary_only"
    | "supplementary_only";
};

export function readStandardScenarioMappings(
  liveMappings: StandardScenarioSkuMapping[] | undefined,
): StandardScenarioSkuMapping[] {
  return liveMappings?.length ? liveMappings : frozenStandardScenarios;
}

export function matchStandardScenariosForProduct(
  skus: readonly ProductSkuLike[],
  scenarios: readonly StandardScenarioSkuMapping[],
): StandardScenarioCoverage[] {
  if (!skus.length) {
    return [];
  }

  const skuSet = new Set(skus.map((sku) => sku.sku_type));

  return scenarios
    .map((scenario) => {
      const primaryMatched = skuSet.has(scenario.primary_sku);
      const supplementaryMatched = scenario.supplementary_skus.filter((sku) =>
        skuSet.has(sku),
      );
      const supplementaryMissing = scenario.supplementary_skus.filter(
        (sku) => !skuSet.has(sku),
      );
      const matchedSkus = primaryMatched
        ? [scenario.primary_sku, ...supplementaryMatched]
        : [...supplementaryMatched];
      const missingSkus = primaryMatched
        ? supplementaryMissing
        : [scenario.primary_sku, ...supplementaryMissing];

      if (!matchedSkus.length) {
        return null;
      }

      return {
        ...scenario,
        primary_matched: primaryMatched,
        supplementary_matched: supplementaryMatched,
        supplementary_missing: supplementaryMissing,
        matched_skus: matchedSkus,
        missing_skus: missingSkus,
        match_level: primaryMatched
          ? supplementaryMatched.length
            ? "primary_with_supplementary"
            : "primary_only"
          : "supplementary_only",
      } satisfies StandardScenarioCoverage;
    })
    .filter((scenario): scenario is StandardScenarioCoverage => Boolean(scenario));
}
