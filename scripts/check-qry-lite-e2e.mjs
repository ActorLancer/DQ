#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const defaultRawDir = path.join(rootDir, "target/test-artifacts/qry-lite-e2e/raw");
const defaultSummaryDir = path.join(rootDir, "target/test-artifacts/qry-lite-e2e");
const rawDir = process.env.TEST026_ARTIFACT_DIR || defaultRawDir;
const summaryDir = process.env.TEST026_SUMMARY_DIR || defaultSummaryDir;
const portalArtifactPath =
  process.env.TEST026_PORTAL_ARTIFACT_FILE || path.join(rawDir, "portal-qry-lite-live.json");
const fixtureFile =
  process.env.TEST026_FIXTURE_FILE || path.join(summaryDir, "live-fixture.json");
const databaseUrl =
  process.env.DATABASE_URL || "postgres://datab:datab_local_pass@127.0.0.1:5432/datab";
const psqlBin = process.env.PSQL_BIN || "psql";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function readJson(filePath) {
  return JSON.parse(await readFile(filePath, "utf8"));
}

function queryJson(sql) {
  const output = execFileSync(
    psqlBin,
    [databaseUrl, "-tA", "-v", "ON_ERROR_STOP=1", "-c", sql],
    {
      cwd: rootDir,
      encoding: "utf8",
    },
  ).trim();
  assert(output, "psql query returned empty output");
  return JSON.parse(output);
}

function sqlString(value) {
  return `'${String(value).replace(/'/g, "''")}'`;
}

function parseDecimal(value, fieldName) {
  const parsed = Number(value);
  assert(Number.isFinite(parsed), `${fieldName} is not a finite decimal`);
  return parsed;
}

async function main() {
  await mkdir(summaryDir, { recursive: true });

  const requiredFiles = [
    "trade013-qry-lite-state-machine.json",
    "dlv011-template-grant.json",
    "dlv012-template-run.json",
    "dlv013-query-runs.json",
    "bil024-qry-lite-billing-bridge.json",
  ];

  const artifactEntries = await Promise.all(
    requiredFiles.map(async (fileName) => {
      const payload = await readJson(path.join(rawDir, fileName));
      return [fileName, payload];
    }),
  );
  const artifacts = new Map(artifactEntries);
  const portal = await readJson(portalArtifactPath);
  const fixture = await readJson(fixtureFile);

  const trade013 = artifacts.get("trade013-qry-lite-state-machine.json");
  assert(trade013.order.current_state === "closed", "trade013 final state mismatch");
  assert(trade013.order.acceptance_status === "closed", "trade013 acceptance closure mismatch");
  assert(
    String(trade013.conflict.message || "").includes("QRY_LITE_TRANSITION_FORBIDDEN"),
    "trade013 illegal replay guard mismatch",
  );

  const dlv011 = artifacts.get("dlv011-template-grant.json");
  assert(
    dlv011.create_response.current_state === "template_authorized",
    "dlv011 create state mismatch",
  );
  assert(dlv011.create_response.grant_status === "active", "dlv011 grant status mismatch");
  assert(dlv011.audit_counts.delivery_template_query_enable === 2, "dlv011 audit count mismatch");
  assert(
    dlv011.outbox.delivery_committed_count === 2,
    "dlv011 delivery committed outbox count mismatch",
  );

  const dlv012 = artifacts.get("dlv012-template-run.json");
  assert(
    dlv012.success_response.current_state === "query_executed",
    "dlv012 run current_state mismatch",
  );
  assert(dlv012.success_response.status === "completed", "dlv012 run status mismatch");
  assert(dlv012.success_response.result_row_count === 2, "dlv012 result rows mismatch");
  assert(
    String(dlv012.missing_approval.message || "").includes("approval_ticket_id is required"),
    "dlv012 missing approval guard mismatch",
  );
  assert(
    dlv012.outbox.billing_trigger === "bill_once_after_task_acceptance",
    "dlv012 billing trigger mismatch",
  );

  const dlv013 = artifacts.get("dlv013-query-runs.json");
  assert(dlv013.query_runs_response.query_run_count === 2, "dlv013 query run count mismatch");
  assert(
    dlv013.query_runs_response.latest_query_template_name === "sales_overview",
    "dlv013 template name mismatch",
  );
  assert(
    dlv013.audit_counts.delivery_template_query_run_read === 1,
    "dlv013 read audit count mismatch",
  );

  const bil024 = artifacts.get("bil024-qry-lite-billing-bridge.json");
  assert(
    bil024.qry_lite.billing_event.event_type === "one_time_charge",
    "bil024 qry lite billing event mismatch",
  );
  assert(
    bil024.qry_lite.outbox.published === true,
    "bil024 qry lite outbox published mismatch",
  );

  assert(portal.order_id === fixture.order_id, "portal artifact order_id drifted from live fixture");
  assert(
    portal.grant_response.current_state === "template_authorized",
    "portal grant current_state mismatch",
  );
  assert(portal.run_response.current_state === "query_executed", "portal run current_state mismatch");
  assert(portal.run_response.status === "completed", "portal run status mismatch");
  assert(
    portal.query_runs_read_response.target_status === "completed",
    "portal query runs read mismatch",
  );
  assert(
    portal.refund_response.current_status === "succeeded",
    "portal refund current_status mismatch",
  );
  assert(
    portal.order_after_refund.payment_status === "refunded",
    "portal order after refund payment_status mismatch",
  );
  assert(
    portal.order_after_refund.dispute_status === "resolved",
    "portal order after refund dispute_status mismatch",
  );
  assert(
    Array.isArray(portal.restricted_requests) && portal.restricted_requests.length === 0,
    "portal browser crossed restricted backend boundary",
  );

  const dbEvidence = queryJson(`
SELECT json_build_object(
  'order_state', (SELECT status FROM trade.order_main WHERE order_id = ${sqlString(portal.order_id)}::uuid),
  'payment_status', (SELECT payment_status FROM trade.order_main WHERE order_id = ${sqlString(portal.order_id)}::uuid),
  'delivery_status', (SELECT delivery_status FROM trade.order_main WHERE order_id = ${sqlString(portal.order_id)}::uuid),
  'acceptance_status', (SELECT acceptance_status FROM trade.order_main WHERE order_id = ${sqlString(portal.order_id)}::uuid),
  'settlement_status', (SELECT settlement_status FROM trade.order_main WHERE order_id = ${sqlString(portal.order_id)}::uuid),
  'dispute_status', (SELECT dispute_status FROM trade.order_main WHERE order_id = ${sqlString(portal.order_id)}::uuid),
  'template_grant_count', (
    SELECT COUNT(*)::int
    FROM delivery.template_query_grant
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
  ),
  'latest_grant_status', (
    SELECT grant_status
    FROM delivery.template_query_grant
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY updated_at DESC, template_query_grant_id DESC
    LIMIT 1
  ),
  'latest_query_run_status', (
    SELECT status
    FROM delivery.query_execution_run
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY created_at DESC, query_run_id DESC
    LIMIT 1
  ),
  'latest_query_run_row_count', (
    SELECT result_row_count
    FROM delivery.query_execution_run
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY created_at DESC, query_run_id DESC
    LIMIT 1
  ),
  'latest_result_object_id', (
    SELECT result_object_id::text
    FROM delivery.query_execution_run
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY created_at DESC, query_run_id DESC
    LIMIT 1
  ),
  'grant_audit_count', (
    SELECT COUNT(*)::int
    FROM audit.audit_event
    WHERE action_name = 'delivery.template_query.enable'
      AND request_id = ${sqlString(portal.portal_proxy_requests.grant_request_id)}
  ),
  'run_audit_count', (
    SELECT COUNT(*)::int
    FROM audit.audit_event
    WHERE action_name = 'delivery.template_query.use'
      AND request_id = ${sqlString(portal.portal_proxy_requests.run_request_id)}
  ),
  'refund_audit_count', (
    SELECT COUNT(*)::int
    FROM audit.audit_event
    WHERE action_name = 'billing.refund.execute'
      AND request_id = ${sqlString(portal.portal_proxy_requests.refund_request_id)}
  ),
  'delivery_committed_topic', (
    SELECT target_topic
    FROM ops.outbox_event
    WHERE request_id = ${sqlString(portal.portal_proxy_requests.grant_request_id)}
      AND event_type = 'delivery.committed'
    ORDER BY created_at DESC, outbox_event_id DESC
    LIMIT 1
  ),
  'billing_bridge_topic', (
    SELECT target_topic
    FROM ops.outbox_event
    WHERE request_id = ${sqlString(portal.portal_proxy_requests.run_request_id)}
      AND event_type = 'billing.trigger.bridge'
    ORDER BY created_at DESC, outbox_event_id DESC
    LIMIT 1
  ),
  'billing_bridge_trigger', (
    SELECT payload -> 'billing_trigger_matrix' ->> 'billing_trigger'
    FROM ops.outbox_event
    WHERE request_id = ${sqlString(portal.portal_proxy_requests.run_request_id)}
      AND event_type = 'billing.trigger.bridge'
    ORDER BY created_at DESC, outbox_event_id DESC
    LIMIT 1
  ),
  'refund_outbox_count', (
    SELECT COUNT(*)::int
    FROM ops.outbox_event
    WHERE aggregate_type = 'billing.refund_record'
      AND aggregate_id = ${sqlString(portal.refund_response.refund_id)}::uuid
  ),
  'refund_notification_count', (
    SELECT COUNT(*)::int
    FROM ops.outbox_event
    WHERE request_id = ${sqlString(portal.portal_proxy_requests.refund_request_id)}
      AND target_topic = 'dtp.notification.dispatch'
  ),
  'refund_notification_scene', (
    SELECT payload -> 'payload' ->> 'notification_code'
    FROM ops.outbox_event
    WHERE request_id = ${sqlString(portal.portal_proxy_requests.refund_request_id)}
      AND target_topic = 'dtp.notification.dispatch'
    ORDER BY created_at ASC, outbox_event_id ASC
    LIMIT 1
  ),
  'billing_refund_count', (
    SELECT COUNT(*)::int
    FROM billing.refund_record
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
  ),
  'settlement_refund_amount', (
    SELECT refund_amount::text
    FROM billing.settlement_record
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY updated_at DESC, settlement_id DESC
    LIMIT 1
  )
)::text
`);

  assert(dbEvidence.order_state === "query_executed", "portal db order_state mismatch");
  assert(dbEvidence.payment_status === "refunded", "portal db payment_status mismatch");
  assert(dbEvidence.delivery_status === "delivered", "portal db delivery_status mismatch");
  assert(dbEvidence.acceptance_status === "accepted", "portal db acceptance_status mismatch");
  assert(dbEvidence.dispute_status === "resolved", "portal db dispute_status mismatch");
  assert(dbEvidence.template_grant_count === 1, "portal db template grant count mismatch");
  assert(dbEvidence.latest_grant_status === "active", "portal db template grant status mismatch");
  assert(dbEvidence.latest_query_run_status === "completed", "portal db query run status mismatch");
  assert(dbEvidence.latest_query_run_row_count === 2, "portal db query run row count mismatch");
  assert(
    dbEvidence.latest_result_object_id === portal.run_response.result_object_id,
    "portal db result object mismatch",
  );
  assert(dbEvidence.grant_audit_count === 1, "portal db grant audit count mismatch");
  assert(dbEvidence.run_audit_count === 1, "portal db run audit count mismatch");
  assert(dbEvidence.refund_audit_count === 1, "portal db refund audit count mismatch");
  assert(
    dbEvidence.delivery_committed_topic === "dtp.outbox.domain-events",
    "portal db delivery outbox topic mismatch",
  );
  assert(
    dbEvidence.billing_bridge_topic === "dtp.outbox.domain-events",
    "portal db billing bridge topic mismatch",
  );
  assert(
    dbEvidence.billing_bridge_trigger === "bill_once_after_task_acceptance",
    "portal db billing trigger mismatch",
  );
  assert(dbEvidence.refund_outbox_count === 1, "portal db refund outbox count mismatch");
  assert(dbEvidence.refund_notification_count >= 1, "portal db refund notification count mismatch");
  assert(
    dbEvidence.refund_notification_scene === "refund.completed",
    "portal db refund notification scene mismatch",
  );
  assert(dbEvidence.billing_refund_count === 1, "portal db refund_record count mismatch");
  assert(
    parseDecimal(dbEvidence.settlement_refund_amount, "db settlement_refund_amount") ===
      parseDecimal(fixture.order_amount, "fixture.order_amount"),
    "portal db settlement refund amount mismatch",
  );

  const summary = {
    test_id: "TEST-026",
    sku_type: "QRY_LITE",
    order_id: portal.order_id,
    case_id: portal.case_id,
    sign_off_order: [
      "trade013-qry-lite-state-machine",
      "dlv011-template-grant",
      "dlv012-template-run",
      "dlv013-query-runs-read",
      "bil024-qry-lite-billing-bridge",
      "portal-live-grant-run-refund",
    ],
    artifacts: {
      raw_dir: rawDir,
      portal_artifact: portalArtifactPath,
      fixture_file: fixtureFile,
    },
    trade013: {
      final_state: trade013.order.current_state,
      conflict_message: trade013.conflict.message,
    },
    dlv011: {
      grant_status: dlv011.create_response.grant_status,
      current_state: dlv011.create_response.current_state,
      outbox_count: dlv011.outbox.delivery_committed_count,
    },
    dlv012: {
      query_run_state: dlv012.success_response.current_state,
      query_run_status: dlv012.success_response.status,
      result_row_count: dlv012.success_response.result_row_count,
      billing_trigger: dlv012.outbox.billing_trigger,
    },
    dlv013: {
      query_run_count: dlv013.query_runs_response.query_run_count,
      latest_query_template_name: dlv013.query_runs_response.latest_query_template_name,
    },
    bil024: {
      billing_event_type: bil024.qry_lite.billing_event.event_type,
      published: bil024.qry_lite.outbox.published,
    },
    portal: {
      grant_request_id: portal.portal_proxy_requests.grant_request_id,
      run_request_id: portal.portal_proxy_requests.run_request_id,
      refund_request_id: portal.portal_proxy_requests.refund_request_id,
      grant_state: portal.grant_response.current_state,
      run_state: portal.run_response.current_state,
      refund_status: portal.refund_response.current_status,
      restricted_requests: portal.restricted_requests,
    },
    db_evidence: dbEvidence,
  };

  await writeFile(
    path.join(summaryDir, "summary.json"),
    JSON.stringify(summary, null, 2),
  );

  console.log(`[ok] TEST-026 summary written: ${path.join(summaryDir, "summary.json")}`);
}

await main();
