#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function resolveRepoPath(relativePath) {
  return path.join(rootDir, relativePath);
}

async function readJsonPath(filePath) {
  const raw = await readFile(filePath, "utf8");
  return JSON.parse(raw);
}

async function readRepoJson(relativePath) {
  return readJsonPath(resolveRepoPath(relativePath));
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function findCase(artifact, caseName) {
  return artifact.orders.find((entry) => entry.case === caseName);
}

function collectArtifactOrderIds(artifact) {
  const orderIds = [];
  for (const entry of artifact.orders || []) {
    if (entry.order_id) {
      orderIds.push(entry.order_id);
    }
  }
  if (artifact.unpublished_order?.order_id) {
    orderIds.push(artifact.unpublished_order.order_id);
  }
  return orderIds;
}

async function main() {
  const rawDir =
    process.env.TEST024_ARTIFACT_DIR ||
    resolveRepoPath("target/test-artifacts/order-orchestration/raw");
  const summaryDir =
    process.env.TEST024_SUMMARY_DIR ||
    resolveRepoPath("target/test-artifacts/order-orchestration");

  await mkdir(summaryDir, { recursive: true });

  const requiredFiles = [
    "trade030-payment-result-orchestrator.json",
    "dlv029-delivery-task-autocreation.json",
    "dlv017-report-delivery.json",
    "dlv018-acceptance.json",
    "dlv025-delivery-integration.json",
    "bil024-billing-trigger-bridge.json",
    "bil025-reject-freeze.json",
    "bil025-manual-adjustment.json"
  ];

  const artifactEntries = await Promise.all(
    requiredFiles.map(async (fileName) => {
      const filePath = path.join(rawDir, fileName);
      const payload = await readJsonPath(filePath);
      return [fileName, payload];
    })
  );
  const artifacts = new Map(artifactEntries);

  const ordersDoc = await readRepoJson("fixtures/demo/orders.json");
  const primaryDemoOrders = ordersDoc.order_blueprints.filter(
    (order) => order.scenario_role === "primary"
  );
  assert(primaryDemoOrders.length === 5, `expected 5 primary demo orders, got ${primaryDemoOrders.length}`);

  const trade030 = artifacts.get("trade030-payment-result-orchestrator.json");
  const trade030Success = findCase(trade030, "success");
  const trade030Ignored = findCase(trade030, "ignored_out_of_order");
  assert(trade030.orders.length === 4, "trade030 artifact order count mismatch");
  assert(trade030Success?.current_state === "buyer_locked", "trade030 success state mismatch");
  assert(trade030Success?.payment_status === "paid", "trade030 success payment mismatch");
  assert(
    trade030Ignored?.current_state === "contract_pending",
    "trade030 ignored webhook current_state mismatch"
  );
  assert(
    trade030Ignored?.payment_status === "unpaid",
    "trade030 ignored webhook payment_status mismatch"
  );

  const dlv029 = artifacts.get("dlv029-delivery-task-autocreation.json");
  assert(dlv029.orders.length === 3, "dlv029 artifact order count mismatch");
  for (const entry of dlv029.orders) {
    assert(entry.delivery_status === "prepared", `dlv029 delivery status mismatch for ${entry.order_id}`);
    assert(entry.creation_source, `dlv029 missing creation_source for ${entry.order_id}`);
  }

  const dlv017 = artifacts.get("dlv017-report-delivery.json");
  const reportCommit = findCase(dlv017, "report_delivery_commit");
  assert(dlv017.orders.length === 1, "dlv017 artifact order count mismatch");
  assert(reportCommit?.current_state === "report_delivered", "dlv017 current_state mismatch");
  assert(reportCommit?.duplicate_operation === "already_committed", "dlv017 duplicate replay mismatch");

  const dlv018 = artifacts.get("dlv018-acceptance.json");
  const acceptancePassed = findCase(dlv018, "file_acceptance_passed");
  const acceptanceRejected = findCase(dlv018, "report_acceptance_rejected");
  assert(acceptancePassed?.current_state === "accepted", "dlv018 accepted state mismatch");
  assert(
    acceptancePassed?.duplicate_operation === "already_accepted",
    "dlv018 duplicate acceptance mismatch"
  );
  assert(acceptanceRejected?.current_state === "rejected", "dlv018 rejected state mismatch");
  assert(
    acceptanceRejected?.settlement_status === "blocked",
    "dlv018 rejected settlement status mismatch"
  );
  assert(acceptanceRejected?.dispute_status === "open", "dlv018 rejected dispute status mismatch");

  const dlv025 = artifacts.get("dlv025-delivery-integration.json");
  assert(dlv025.orders.length === 5, "dlv025 artifact order count mismatch");
  assert(findCase(dlv025, "file_delivery_acceptance")?.current_state === "accepted", "dlv025 file state mismatch");
  assert(findCase(dlv025, "api_delivery_enable")?.current_state === "api_key_issued", "dlv025 api state mismatch");
  assert(findCase(dlv025, "query_result_available")?.current_state === "query_executed", "dlv025 query state mismatch");
  assert(findCase(dlv025, "sandbox_delivery_enable")?.current_state === "seat_issued", "dlv025 sandbox state mismatch");
  assert(findCase(dlv025, "report_delivery_rejected")?.current_state === "rejected", "dlv025 report state mismatch");

  const bil024 = artifacts.get("bil024-billing-trigger-bridge.json");
  const bil024ProcessedCount = bil024.orders.filter((entry) => entry.processing_mode === "processed").length;
  assert(bil024ProcessedCount >= 6, `bil024 processed bridge count too small: ${bil024ProcessedCount}`);
  assert(
    bil024.unpublished_order?.processing_mode === "unpublished_ignored",
    "bil024 unpublished bridge mismatch"
  );

  const bil025Reject = artifacts.get("bil025-reject-freeze.json");
  const rejectFreeze = findCase(bil025Reject, "delivery_reject_freeze");
  assert(rejectFreeze?.current_state === "rejected", "bil025 reject state mismatch");
  assert(rejectFreeze?.settlement_status === "blocked", "bil025 reject settlement mismatch");
  assert(rejectFreeze?.dispute_status === "open", "bil025 reject dispute mismatch");

  const bil025Manual = artifacts.get("bil025-manual-adjustment.json");
  const manualAdjustment = findCase(bil025Manual, "manual_adjustment_resolution");
  assert(
    manualAdjustment?.summary_state === "order_settlement:settled:manual",
    "bil025 manual summary_state mismatch"
  );

  const primaryDemoOrderIds = primaryDemoOrders.map((order) => order.order_blueprint_id);
  const dynamicOrderIds = [
    ...new Set(
      [...artifacts.values()].flatMap((artifact) => collectArtifactOrderIds(artifact))
    )
  ];
  const signoffOrderIds = [...new Set([...primaryDemoOrderIds, ...dynamicOrderIds])];
  assert(signoffOrderIds.length >= 20, `sign-off order count too small: ${signoffOrderIds.length}`);

  const summary = {
    task_id: "TEST-024",
    required_artifacts: requiredFiles,
    primary_demo_order_ids: primaryDemoOrderIds,
    frontend_baseline_order_count: primaryDemoOrderIds.length,
    dynamic_order_ids: dynamicOrderIds,
    dynamic_orchestration_order_count: dynamicOrderIds.length,
    signoff_order_ids: signoffOrderIds,
    signoff_order_count: signoffOrderIds.length,
    checkpoints: {
      frontend_scenarios: primaryDemoOrderIds.length,
      webhook_out_of_order: true,
      delivery_task_autocreation: true,
      delivery_duplicate_protected: true,
      acceptance_duplicate_protected: true,
      acceptance_rejected_freeze: true,
      billing_trigger_bridge: true,
      settlement_recomputed: true
    }
  };

  await writeFile(
    path.join(summaryDir, "summary.json"),
    JSON.stringify(summary, null, 2)
  );

  console.log(
    `[ok] TEST-024 orchestration verified: ${summary.signoff_order_count} sign-off orders ` +
      `(${primaryDemoOrderIds.length} frontend baseline + ${dynamicOrderIds.length} dynamic orchestration)`
  );
}

main().catch((error) => {
  console.error(`[fail] ${error.message}`);
  process.exit(1);
});
