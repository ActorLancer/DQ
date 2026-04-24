#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function resolveRepoPath(relativePath) {
  return path.join(rootDir, relativePath);
}

async function readJson(relativePath) {
  const raw = await readFile(resolveRepoPath(relativePath), "utf8");
  return JSON.parse(raw);
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function mapBy(items, key) {
  const mapped = new Map();
  for (const item of items) {
    const value = item[key];
    assert(value, `missing map key ${key}`);
    assert(!mapped.has(value), `duplicate ${key}: ${value}`);
    mapped.set(value, item);
  }
  return mapped;
}

function sameArray(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function normalizeDecimal(value) {
  assert(typeof value === "string" && value.length > 0, "decimal value must be non-empty string");
  if (!value.includes(".")) {
    return value;
  }
  return value.replace(/(\.\d*?[1-9])0+$/, "$1").replace(/\.0+$/, "").replace(/\.$/, "");
}

async function fetchJson(url, headers) {
  const response = await fetch(url, { headers });
  const body = await response.text();
  if (!response.ok) {
    throw new Error(`request failed ${response.status} ${url}: ${body}`);
  }
  return JSON.parse(body);
}

async function main() {
  const baseUrl = process.env.PLATFORM_CORE_BASE_URL || "http://127.0.0.1:8094";
  const artifactDir =
    process.env.STANDARD_SKU_COVERAGE_ARTIFACT_DIR ||
    resolveRepoPath("target/test-artifacts/standard-sku-coverage");
  await mkdir(artifactDir, { recursive: true });

  const [matrixDoc, scenariosDoc, ordersDoc, billingDoc, catalogDoc] = await Promise.all([
    readJson("fixtures/demo/sku-coverage-matrix.json"),
    readJson("fixtures/demo/scenarios.json"),
    readJson("fixtures/demo/orders.json"),
    readJson("fixtures/demo/billing.json"),
    readJson("fixtures/demo/catalog.json")
  ]);

  const scenarioMap = mapBy(scenariosDoc.scenarios, "scenario_code");
  const orderMap = mapBy(ordersDoc.order_blueprints, "order_blueprint_id");
  const triggerMap = mapBy(billingDoc.sku_billing_trigger_matrix, "sku_code");
  const templateMap = mapBy(catalogDoc.scenario_templates, "scenario_code");
  const billingSamplesByOrderId = new Map();
  for (const sample of billingDoc.billing_samples) {
    const existing = billingSamplesByOrderId.get(sample.order_blueprint_id) || [];
    existing.push(sample);
    billingSamplesByOrderId.set(sample.order_blueprint_id, existing);
  }

  const standardScenarioRequestId = `req-test023-standard-scenarios-${Date.now()}`;
  const standardScenarioResponse = await fetchJson(`${baseUrl}/api/v1/catalog/standard-scenarios`, {
    "x-role": "tenant_admin",
    "x-request-id": standardScenarioRequestId
  });
  assert(standardScenarioResponse.code === "OK", "standard scenarios response code must be OK");
  assert(standardScenarioResponse.message === "success", "standard scenarios response must be success");
  assert(standardScenarioResponse.request_id, "standard scenarios response missing request_id");
  const standardScenarioData = standardScenarioResponse.data;
  assert(Array.isArray(standardScenarioData), "standard scenarios data must be array");
  assert(standardScenarioData.length === 5, `standard scenarios count mismatch: ${standardScenarioData.length}`);
  const standardScenarioMap = mapBy(standardScenarioData, "scenario_code");
  assert(
    sameArray([...standardScenarioMap.keys()].sort(), ["S1", "S2", "S3", "S4", "S5"]),
    "standard scenarios api returned unexpected scenario code set"
  );
  await writeFile(
    path.join(artifactDir, "catalog-standard-scenarios-response.json"),
    JSON.stringify(standardScenarioResponse, null, 2)
  );

  const summary = {
    task_id: "TEST-023",
    run_id: String(Date.now()),
    base_url: baseUrl,
    catalog_scenario_count: standardScenarioData.length,
    standard_sku_count: matrixDoc.standard_sku_matrix.length,
    skus: []
  };

  for (const entry of matrixDoc.standard_sku_matrix) {
    const trigger = triggerMap.get(entry.sku_type);
    assert(trigger, `missing billing trigger rule for ${entry.sku_type}`);

    const basisOrder = orderMap.get(entry.billing_basis_order_blueprint_id);
    assert(basisOrder, `missing billing basis order for ${entry.sku_type}`);
    assert(
      basisOrder.buyer_org_id,
      `billing basis order missing buyer org for ${entry.sku_type}`
    );

    for (const link of entry.scenario_links) {
      const scenario = scenarioMap.get(link.scenario_code);
      const apiScenario = standardScenarioMap.get(link.scenario_code);
      const template = templateMap.get(link.scenario_code);
      assert(scenario, `missing scenario fixture ${link.scenario_code} for ${entry.sku_type}`);
      assert(apiScenario, `missing api scenario ${link.scenario_code} for ${entry.sku_type}`);
      assert(template, `missing template ${link.scenario_code} for ${entry.sku_type}`);
      assert(apiScenario.scenario_name === scenario.scenario_name, `scenario name mismatch for ${link.scenario_code}`);
      assert(apiScenario.primary_sku === template.primary_sku, `primary sku mismatch for ${link.scenario_code}`);
      assert(
        sameArray(apiScenario.supplementary_skus, template.supplementary_skus),
        `supplementary sku mismatch for ${link.scenario_code}`
      );
      assert(
        apiScenario.contract_template === template.contract_template,
        `contract template mismatch for ${link.scenario_code}`
      );
      assert(
        apiScenario.acceptance_template === template.acceptance_template,
        `acceptance template mismatch for ${link.scenario_code}`
      );
      assert(
        apiScenario.refund_template === template.refund_template,
        `refund template mismatch for ${link.scenario_code}`
      );
    }

    const billingRequestId = `req-test023-billing-${entry.sku_type.toLowerCase()}-${Date.now()}`;
    const billingResponse = await fetchJson(`${baseUrl}/api/v1/billing/${basisOrder.order_blueprint_id}`, {
      "x-role": "tenant_admin",
      "x-tenant-id": basisOrder.buyer_org_id,
      "x-request-id": billingRequestId
    });
    assert(billingResponse.code === "OK", `billing response code must be OK for ${entry.sku_type}`);
    assert(billingResponse.message === "success", `billing response must be success for ${entry.sku_type}`);
    assert(billingResponse.request_id, `billing response missing request_id for ${entry.sku_type}`);

    const detail = billingResponse.data;
    assert(detail.current_state === basisOrder.current_state, `current_state mismatch for ${entry.sku_type}`);
    assert(
      normalizeDecimal(detail.order_amount) === normalizeDecimal(basisOrder.order_amount),
      `order_amount mismatch for ${entry.sku_type}`
    );
    assert(detail.currency_code === basisOrder.currency_code, `currency mismatch for ${entry.sku_type}`);
    assert(detail.sku_billing_basis, `sku_billing_basis missing for ${entry.sku_type}`);
    assert(detail.sku_billing_basis.sku_type === entry.sku_type, `sku_billing_basis sku mismatch for ${entry.sku_type}`);
    assert(
      detail.sku_billing_basis.refund_entry === trigger.refund_entry,
      `refund_entry mismatch for ${entry.sku_type}`
    );
    assert(
      detail.sku_billing_basis.dispute_freeze_trigger === trigger.dispute_freeze_trigger,
      `dispute_freeze_trigger mismatch for ${entry.sku_type}`
    );
    assert(
      detail.sku_billing_basis.resume_settlement_trigger === trigger.resume_settlement_trigger,
      `resume_settlement_trigger mismatch for ${entry.sku_type}`
    );

    const sampleCount = (billingSamplesByOrderId.get(basisOrder.order_blueprint_id) || []).length;
    assert(Array.isArray(detail.billing_events), `billing_events missing for ${entry.sku_type}`);
    assert(
      detail.billing_events.length >= sampleCount,
      `billing_events count mismatch for ${entry.sku_type}: expected at least ${sampleCount}, got ${detail.billing_events.length}`
    );

    if (entry.sku_type === "API_SUB" || entry.sku_type === "API_PPU") {
      assert(detail.api_billing_basis, `api_billing_basis missing for ${entry.sku_type}`);
      assert(
        detail.api_billing_basis.sku_type === entry.sku_type,
        `api_billing_basis sku mismatch for ${entry.sku_type}`
      );
    }

    await writeFile(
      path.join(artifactDir, `billing-detail-${entry.sku_type.toLowerCase()}.json`),
      JSON.stringify(billingResponse, null, 2)
    );

    summary.skus.push({
      sku_type: entry.sku_type,
      scenario_links: entry.scenario_links,
      billing_basis_order_blueprint_id: entry.billing_basis_order_blueprint_id,
      current_state: detail.current_state,
      order_amount: detail.order_amount,
      billing_event_count: detail.billing_events.length,
      refund_entry: detail.sku_billing_basis.refund_entry,
      dispute_freeze_trigger: detail.sku_billing_basis.dispute_freeze_trigger,
      resume_settlement_trigger: detail.sku_billing_basis.resume_settlement_trigger,
      api_billing_basis_present: Boolean(detail.api_billing_basis),
      main_path_checks: entry.main_path_evidence.checks.map((check) => check.target),
      exception_path_checks: entry.exception_path_evidence.checks.map((check) => check.target),
      refund_or_dispute_mode: entry.refund_or_dispute_evidence.mode,
      refund_or_dispute_checks: entry.refund_or_dispute_evidence.checks.map((check) => check.target)
    });
  }

  await writeFile(
    path.join(artifactDir, "summary.json"),
    JSON.stringify(summary, null, 2)
  );

  console.log(
    `[ok] live sku coverage verified via ${baseUrl}: ` +
      `${summary.standard_sku_count} skus, ${summary.catalog_scenario_count} standard scenarios`
  );
}

main().catch((error) => {
  console.error(`[fail] ${error.message}`);
  process.exit(1);
});
