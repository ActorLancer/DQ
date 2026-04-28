import { describe, expect, it } from "vitest";

import { ORDER_SCENARIO_BLUEPRINTS } from "./order-workflow";
import {
  collectStandardDemoSkuCoverage,
  findStandardDemoGuide,
  frozenStandardScenarios,
  officialSkuOrder,
  standardDemoGuides,
} from "./standard-demo";

describe("standard demo paths", () => {
  it("keeps five official homepage demo paths in frozen order", () => {
    expect(standardDemoGuides.map((guide) => guide.scenarioCode)).toEqual([
      "S1",
      "S2",
      "S3",
      "S4",
      "S5",
    ]);
    expect(standardDemoGuides.map((guide) => guide.path)).toEqual([
      "/demos/S1",
      "/demos/S2",
      "/demos/S3",
      "/demos/S4",
      "/demos/S5",
    ]);
  });

  it("covers all eight V1 SKU types without collapsing independent SKUs", () => {
    expect(collectStandardDemoSkuCoverage()).toEqual([...officialSkuOrder]);
    expect(findStandardDemoGuide("S3")?.supplementarySkus).toContain("SHARE_RO");
    expect(findStandardDemoGuide("S5")?.primarySku).toBe("QRY_LITE");
    expect(findStandardDemoGuide("S4")?.supplementarySkus).toContain("RPT_STD");
  });

  it("matches order template contract, acceptance and refund mapping", () => {
    for (const guide of standardDemoGuides) {
      const orderTemplate = ORDER_SCENARIO_BLUEPRINTS.find(
        (template) => template.scenario_code === guide.scenarioCode,
      );
      expect(orderTemplate?.scenario_name).toBe(guide.scenarioName);
      expect(orderTemplate?.primary_sku).toBe(guide.primarySku);
      expect(orderTemplate?.supplementary_skus).toEqual([
        ...guide.supplementarySkus,
      ]);
      expect(orderTemplate?.contract_template).toBe(guide.contractTemplate);
      expect(orderTemplate?.acceptance_template).toBe(guide.acceptanceTemplate);
      expect(orderTemplate?.refund_template).toBe(guide.refundTemplate);
    }
  });

  it("builds frozen standard-scenario fallback from the same demo source", () => {
    expect(frozenStandardScenarios).toHaveLength(5);
    expect(frozenStandardScenarios.map((item) => item.scenario_code)).toEqual(
      standardDemoGuides.map((guide) => guide.scenarioCode),
    );
    expect(
      frozenStandardScenarios.flatMap((item) => [
        item.primary_sku,
        ...item.supplementary_skus,
      ]),
    ).toEqual([
      "API_SUB",
      "API_PPU",
      "FILE_STD",
      "FILE_SUB",
      "SBX_STD",
      "SHARE_RO",
      "API_SUB",
      "RPT_STD",
      "QRY_LITE",
      "RPT_STD",
    ]);
  });
});
