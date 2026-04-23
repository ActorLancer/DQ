#!/usr/bin/env node

import { readFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function resolveRepoPath(relativePath) {
  return path.join(rootDir, relativePath);
}

async function fileExists(relativePath) {
  try {
    await access(resolveRepoPath(relativePath));
    return true;
  } catch {
    return false;
  }
}

async function readJson(relativePath) {
  const fullPath = resolveRepoPath(relativePath);
  const raw = await readFile(fullPath, "utf8");
  return JSON.parse(raw);
}

async function readText(relativePath) {
  return readFile(resolveRepoPath(relativePath), "utf8");
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

async function main() {
  const manifest = await readJson("fixtures/demo/manifest.json");
  const sections = {};
  for (const [sectionName, relativePath] of Object.entries(manifest.files)) {
    assert(await fileExists(relativePath), `missing section file: ${relativePath}`);
    sections[sectionName] = await readJson(relativePath);
  }

  for (const sourcePath of manifest.sources) {
    assert(await fileExists(sourcePath), `missing upstream source file: ${sourcePath}`);
  }

  const scenarios = sections.scenarios.scenarios;
  const scenarioCodes = scenarios.map((item) => item.scenario_code);
  assert(
    sameArray(scenarioCodes, manifest.official_scenario_order),
    `scenario order mismatch: expected ${manifest.official_scenario_order.join(",")} got ${scenarioCodes.join(",")}`
  );

  const templateMap = mapBy(sections.catalog.scenario_templates, "scenario_code");
  const scenarioMap = mapBy(scenarios, "scenario_code");
  const productMap = mapBy(sections.catalog.official_display_products, "product_id");
  const skuMap = mapBy(sections.catalog.official_display_skus, "sku_id");
  const orderMap = mapBy(sections.orders.order_blueprints, "order_blueprint_id");
  const deliveryMap = mapBy(sections.delivery.delivery_blueprints, "delivery_blueprint_id");
  const billingMap = mapBy(sections.billing.billing_samples, "billing_sample_id");
  const auditMap = mapBy(sections.audit.audit_suites, "audit_suite_id");
  const orgMap = mapBy(sections.subjects.organizations, "org_id");
  const userMap = mapBy(sections.subjects.users, "user_id");
  const applicationMap = mapBy(sections.subjects.applications, "app_id");
  const billingTriggerMap = mapBy(sections.billing.sku_billing_trigger_matrix, "sku_code");

  const coveredSkus = new Set();
  for (const template of sections.catalog.scenario_templates) {
    coveredSkus.add(template.primary_sku);
    for (const sku of template.supplementary_skus) {
      coveredSkus.add(sku);
    }
  }
  assert(
    sameArray([...coveredSkus].sort(), [...manifest.coverage.standard_skus].sort()),
    `standard sku coverage mismatch: expected ${manifest.coverage.standard_skus.join(",")}`
  );

  const fixedSamples = [...sections.catalog.home_featured_fixed_samples].sort(
    (left, right) => left.sample_order - right.sample_order
  );
  assert(
    fixedSamples.length === manifest.official_scenario_order.length,
    `home_featured sample count mismatch: ${fixedSamples.length}`
  );
  fixedSamples.forEach((sample, index) => {
    const expectedScenario = manifest.official_scenario_order[index];
    const scenario = scenarioMap.get(expectedScenario);
    assert(sample.sample_order === index, `home_featured sample order mismatch for ${sample.scenario_code}`);
    assert(sample.scenario_code === expectedScenario, `home_featured scenario mismatch at position ${index}`);
    assert(
      sample.product_id === scenario.official_display_product_id,
      `home_featured product mismatch for ${sample.scenario_code}`
    );
  });

  for (const scenario of scenarios) {
    const template = templateMap.get(scenario.scenario_code);
    assert(template, `missing template for ${scenario.scenario_code}`);

    assert(orgMap.has(scenario.participants.seller_org_id), `missing seller org for ${scenario.scenario_code}`);
    assert(orgMap.has(scenario.participants.buyer_org_id), `missing buyer org for ${scenario.scenario_code}`);
    assert(orgMap.has(scenario.participants.ops_org_id), `missing ops org for ${scenario.scenario_code}`);
    assert(userMap.has(scenario.participants.buyer_primary_user_id), `missing buyer user for ${scenario.scenario_code}`);

    const product = productMap.get(scenario.official_display_product_id);
    assert(product, `missing official display product for ${scenario.scenario_code}`);
    assert(product.scenario_code === scenario.scenario_code, `product scenario mismatch for ${scenario.scenario_code}`);

    const expectedSkuIds = [product.primary_sku_id, ...product.supplementary_sku_ids];
    assert(
      sameArray(expectedSkuIds, scenario.official_display_sku_ids),
      `official sku ids mismatch for ${scenario.scenario_code}`
    );

    const primaryOrder = orderMap.get(scenario.primary_order_blueprint_id);
    assert(primaryOrder, `missing primary order blueprint for ${scenario.scenario_code}`);
    assert(primaryOrder.scenario_code === scenario.scenario_code, `primary order scenario mismatch for ${scenario.scenario_code}`);
    assert(primaryOrder.product_id === scenario.official_display_product_id, `primary order product mismatch for ${scenario.scenario_code}`);
    assert(primaryOrder.scenario_role === "primary", `primary order role mismatch for ${scenario.scenario_code}`);
    assert(primaryOrder.scenario_snapshot.primary_sku === template.primary_sku, `primary sku mismatch for ${scenario.scenario_code}`);
    assert(
      sameArray(primaryOrder.scenario_snapshot.supplementary_skus, template.supplementary_skus),
      `supplementary sku mismatch for ${scenario.scenario_code}`
    );
    assert(primaryOrder.template_codes.contract_template === template.contract_template, `contract template mismatch for ${scenario.scenario_code}`);
    assert(primaryOrder.template_codes.acceptance_template === template.acceptance_template, `acceptance template mismatch for ${scenario.scenario_code}`);
    assert(primaryOrder.template_codes.refund_template === template.refund_template, `refund template mismatch for ${scenario.scenario_code}`);
    assert(
      primaryOrder.created_by_user_id === scenario.participants.buyer_primary_user_id,
      `primary order actor mismatch for ${scenario.scenario_code}`
    );

    const scenarioDeliveryActions = new Set();
    for (const deliveryId of primaryOrder.delivery_blueprint_ids) {
      const delivery = deliveryMap.get(deliveryId);
      assert(delivery, `missing primary delivery blueprint ${deliveryId}`);
      assert(delivery.order_blueprint_id === primaryOrder.order_blueprint_id, `primary delivery order mismatch for ${scenario.scenario_code}`);
      assert(await fileExists(delivery.source_fixture), `missing delivery source fixture ${delivery.source_fixture}`);
      scenarioDeliveryActions.add(delivery.expected_audit_action);
    }
    for (const billingId of primaryOrder.billing_sample_ids) {
      const billing = billingMap.get(billingId);
      assert(billing, `missing primary billing sample ${billingId}`);
      assert(billing.order_blueprint_id === primaryOrder.order_blueprint_id, `primary billing order mismatch for ${scenario.scenario_code}`);
      assert(billingTriggerMap.has(billing.sku_type), `missing sku trigger for ${billing.sku_type}`);
    }

    for (const supplementaryOrderId of scenario.supplementary_order_blueprint_ids) {
      const order = orderMap.get(supplementaryOrderId);
      assert(order, `missing supplementary order ${supplementaryOrderId}`);
      assert(order.scenario_code === scenario.scenario_code, `supplementary order scenario mismatch for ${scenario.scenario_code}`);
      assert(order.product_id === scenario.official_display_product_id, `supplementary order product mismatch for ${scenario.scenario_code}`);
      assert(order.scenario_role === "supplementary", `supplementary order role mismatch for ${scenario.scenario_code}`);
      const supplementarySku = skuMap.get(order.sku_id);
      assert(supplementarySku, `missing supplementary sku ${order.sku_id}`);
      assert(
        template.supplementary_skus.includes(supplementarySku.sku_type),
        `supplementary sku type mismatch for ${scenario.scenario_code}: ${supplementarySku.sku_type}`
      );
      for (const deliveryId of order.delivery_blueprint_ids) {
        const delivery = deliveryMap.get(deliveryId);
        assert(delivery, `missing supplementary delivery blueprint ${deliveryId}`);
        assert(delivery.order_blueprint_id === order.order_blueprint_id, `supplementary delivery order mismatch for ${scenario.scenario_code}`);
        assert(await fileExists(delivery.source_fixture), `missing delivery source fixture ${delivery.source_fixture}`);
        scenarioDeliveryActions.add(delivery.expected_audit_action);
      }
      for (const billingId of order.billing_sample_ids) {
        const billing = billingMap.get(billingId);
        assert(billing, `missing supplementary billing sample ${billingId}`);
        assert(billing.order_blueprint_id === order.order_blueprint_id, `supplementary billing order mismatch for ${scenario.scenario_code}`);
        assert(billingTriggerMap.has(billing.sku_type), `missing sku trigger for ${billing.sku_type}`);
      }
    }

    for (const skuId of scenario.official_display_sku_ids) {
      assert(skuMap.has(skuId), `missing sku ${skuId} for ${scenario.scenario_code}`);
    }

    const auditSuite = auditMap.get(scenario.audit_suite_id);
    assert(auditSuite, `missing audit suite for ${scenario.scenario_code}`);
    assert(auditSuite.scenario_code === scenario.scenario_code, `audit scenario mismatch for ${scenario.scenario_code}`);
    assert(
      sameArray(
        [...auditSuite.order_blueprint_ids].sort(),
        [scenario.primary_order_blueprint_id, ...scenario.supplementary_order_blueprint_ids].sort()
      ),
      `audit order set mismatch for ${scenario.scenario_code}`
    );
    for (const requiredAction of [
      "catalog.standard.scenarios.read",
      "trade.order.create",
      "billing.event.recorded",
      "audit.package.export"
    ]) {
      assert(
        auditSuite.must_record_actions.includes(requiredAction),
        `missing audit action ${requiredAction} for ${scenario.scenario_code}`
      );
    }
    for (const deliveryAction of scenarioDeliveryActions) {
      assert(
        auditSuite.must_record_actions.includes(deliveryAction),
        `audit suite missing delivery action ${deliveryAction} for ${scenario.scenario_code}`
      );
    }
    assert(
      auditSuite.step_up_required_actions.includes("audit.package.export"),
      `step-up export missing for ${scenario.scenario_code}`
    );
    assert(
      auditSuite.denied_without_step_up_actions.includes("audit.package.export"),
      `denied-without-step-up export missing for ${scenario.scenario_code}`
    );
  }

  for (const order of sections.orders.order_blueprints) {
    const scenario = scenarioMap.get(order.scenario_code);
    const template = templateMap.get(order.scenario_code);
    const product = productMap.get(order.product_id);
    const sku = skuMap.get(order.sku_id);
    assert(scenario, `missing scenario for order ${order.order_blueprint_id}`);
    assert(template, `missing template for order ${order.order_blueprint_id}`);
    assert(product, `missing product for order ${order.order_blueprint_id}`);
    assert(sku, `missing sku for order ${order.order_blueprint_id}`);
    assert(product.product_id === scenario.official_display_product_id, `order product not bound to official display product ${order.order_blueprint_id}`);
    assert(sku.product_id === order.product_id, `order sku/product mismatch ${order.order_blueprint_id}`);
    assert(order.scenario_snapshot.scenario_code === order.scenario_code, `scenario snapshot mismatch ${order.order_blueprint_id}`);
    if (order.scenario_role === "primary") {
      assert(sku.sku_type === template.primary_sku, `primary order sku type mismatch ${order.order_blueprint_id}`);
      assert(order.scenario_snapshot.selected_sku_role === "primary", `primary order selected role mismatch ${order.order_blueprint_id}`);
    } else {
      assert(template.supplementary_skus.includes(sku.sku_type), `supplementary order sku type mismatch ${order.order_blueprint_id}`);
      assert(order.scenario_snapshot.selected_sku_role === "supplementary", `supplementary order selected role mismatch ${order.order_blueprint_id}`);
    }
    assert(order.template_codes.contract_template === template.contract_template, `order contract template mismatch ${order.order_blueprint_id}`);
    assert(order.template_codes.acceptance_template === template.acceptance_template, `order acceptance template mismatch ${order.order_blueprint_id}`);
    assert(order.template_codes.refund_template === template.refund_template, `order refund template mismatch ${order.order_blueprint_id}`);
    assert(orgMap.has(order.buyer_org_id), `missing buyer org for order ${order.order_blueprint_id}`);
    assert(orgMap.has(order.seller_org_id), `missing seller org for order ${order.order_blueprint_id}`);
    assert(userMap.has(order.created_by_user_id), `missing actor for order ${order.order_blueprint_id}`);
  }

  for (const application of sections.subjects.applications) {
    assert(orgMap.has(application.org_id), `application org missing: ${application.app_id}`);
  }
  assert(applicationMap.has("10000000-0000-0000-0000-000000000401"), "missing buyer api application");

  const standardScenarioSource = await readText("apps/platform-core/src/modules/catalog/standard_scenarios.rs");
  const searchrecSeedSource = await readText("db/seeds/033_searchrec_recommendation_samples.sql");
  const localManifestSource = await readText("fixtures/local/standard-scenarios-manifest.json");
  for (const scenario of scenarios) {
    const template = templateMap.get(scenario.scenario_code);
    const product = productMap.get(scenario.official_display_product_id);
    assert(
      standardScenarioSource.includes(scenario.scenario_name),
      `catalog standard scenario source missing scenario name ${scenario.scenario_name}`
    );
    assert(
      standardScenarioSource.includes(template.contract_template),
      `catalog standard scenario source missing template code ${template.contract_template}`
    );
    assert(
      searchrecSeedSource.includes(product.product_id),
      `searchrec seed missing official display product ${product.product_id}`
    );
    assert(
      searchrecSeedSource.includes(scenario.scenario_name),
      `searchrec seed missing scenario name ${scenario.scenario_name}`
    );
    assert(
      localManifestSource.includes(`"scenario_id": "${scenario.scenario_code}"`) ||
        localManifestSource.includes(`"scenario_code": "${scenario.scenario_code}"`),
      `local scenario manifest missing ${scenario.scenario_code}`
    );
  }

  console.log(
    `[ok] demo fixture package validated: ${scenarios.length} scenarios, ` +
      `${sections.catalog.official_display_skus.length} official display skus, ` +
      `${sections.orders.order_blueprints.length} order blueprints, ` +
      `${sections.delivery.delivery_blueprints.length} delivery blueprints, ` +
      `${sections.billing.billing_samples.length} billing samples, ` +
      `${sections.audit.audit_suites.length} audit suites`
  );
}

main().catch((error) => {
  console.error(`[fail] ${error.message}`);
  process.exit(1);
});
