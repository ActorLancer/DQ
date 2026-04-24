#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rawDir =
  process.env.TEST027_ARTIFACT_DIR ||
  path.join(rootDir, "target/test-artifacts/notification-smoke/raw");
const summaryDir =
  process.env.TEST027_SUMMARY_DIR ||
  path.join(rootDir, "target/test-artifacts/notification-smoke");
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

function multiset(values) {
  return [...values].sort();
}

function sameCodes(actual, expected, message) {
  assert(
    JSON.stringify(multiset(actual)) === JSON.stringify(multiset(expected)),
    `${message}: expected ${JSON.stringify(expected)} got ${JSON.stringify(actual)}`,
  );
}

function queryJson(sql) {
  const output = execFileSync(
    psqlBin,
    [databaseUrl, "-tA", "-v", "ON_ERROR_STOP=1", "-c", sql],
    { cwd: rootDir, encoding: "utf8" },
  ).trim();
  assert(output, "psql query returned empty output");
  return JSON.parse(output);
}

function sqlString(value) {
  return `'${String(value).replace(/'/g, "''")}'`;
}

function isOkEnvelope(payload, expectedData) {
  return (
    payload &&
    payload.code === "OK" &&
    payload.message === "success" &&
    typeof payload.request_id === "string" &&
    payload.request_id.length > 0 &&
    payload.data === expectedData
  );
}

async function main() {
  await mkdir(summaryDir, { recursive: true });

  const [
    payment,
    delivery,
    acceptance,
    dispute,
    worker,
    paymentLookup,
    disputeLookup,
    lookupContext,
    workerLiveHealth,
    workerReadyHealth,
    workerMetrics,
    outboxPublisherLive,
    outboxPublisherReady,
  ] = await Promise.all([
    readJson(path.join(rawDir, "notif004-payment-success.json")),
    readJson(path.join(rawDir, "notif005-delivery-completion.json")),
    readJson(path.join(rawDir, "notif006-acceptance-outcome.json")),
    readJson(path.join(rawDir, "notif007-dispute-settlement.json")),
    readJson(path.join(rawDir, "notif012-worker-live-smoke.json")),
    readJson(path.join(rawDir, "platform-payment-audit-search.json")),
    readJson(path.join(rawDir, "platform-dispute-audit-search.json")),
    readJson(path.join(rawDir, "platform-audit-lookups.json")),
    readJson(path.join(rawDir, "notification-worker.health.live.json")),
    readJson(path.join(rawDir, "notification-worker.health.ready.json")),
    readFile(path.join(rawDir, "notification-worker.metrics.prom"), "utf8"),
    readJson(path.join(rawDir, "outbox-publisher.health.live.json")),
    readJson(path.join(rawDir, "outbox-publisher.health.ready.json")),
  ]);

  assert(
    workerLiveHealth.status === "live" || isOkEnvelope(workerLiveHealth, "ok"),
    "notification-worker live health mismatch",
  );
  assert(
    workerReadyHealth.status === "ready" || isOkEnvelope(workerReadyHealth, "ready"),
    "notification-worker ready health mismatch",
  );
  assert(outboxPublisherLive.status === "live", "outbox-publisher live health mismatch");
  assert(outboxPublisherReady.status === "ready", "outbox-publisher ready health mismatch");
  assert(
    workerMetrics.includes("notification_worker_events_total"),
    "notification-worker metrics missing notification_worker_events_total",
  );
  assert(
    workerMetrics.includes("notification_worker_send_total"),
    "notification-worker metrics missing notification_worker_send_total",
  );

  assert(payment.live_chain, "notif004 live_chain artifact missing");
  assert(payment.live_chain.outbox.published_count === 3, "notif004 published_count mismatch");
  assert(payment.live_chain.mock_log.count === 3, "notif004 mock-log count mismatch");
  assert(payment.live_chain.audit.count === 3, "notif004 audit count mismatch");
  sameCodes(
    payment.live_chain.mock_log.notification_codes,
    ["payment.succeeded", "order.pending_delivery", "order.pending_delivery"],
    "notif004 notification codes mismatch",
  );

  assert(delivery.branches.length === 6, "notif005 branch count mismatch");
  for (const branch of delivery.branches) {
    assert(branch.live_chain, `notif005 branch ${branch.delivery_branch} live_chain missing`);
    assert(
      branch.live_chain.outbox.published_count === 3,
      `notif005 branch ${branch.delivery_branch} published_count mismatch`,
    );
    assert(
      branch.live_chain.mock_log.count === 3,
      `notif005 branch ${branch.delivery_branch} mock-log count mismatch`,
    );
    assert(
      branch.live_chain.audit.count === 3,
      `notif005 branch ${branch.delivery_branch} audit count mismatch`,
    );
  }

  assert(acceptance.passed.live_chain, "notif006 passed live_chain missing");
  assert(
    acceptance.passed.live_chain.outbox.published_count === 3,
    "notif006 passed published_count mismatch",
  );
  sameCodes(
    acceptance.passed.live_chain.mock_log.notification_codes,
    ["acceptance.passed", "acceptance.passed", "acceptance.passed"],
    "notif006 passed notification codes mismatch",
  );

  assert(dispute.dispute.live_chain, "notif007 dispute live_chain missing");
  assert(
    dispute.dispute.live_chain.outbox.published_count === 6,
    "notif007 dispute published_count mismatch",
  );
  sameCodes(
    dispute.dispute.live_chain.mock_log.notification_codes,
    [
      "dispute.escalated",
      "dispute.escalated",
      "dispute.escalated",
      "settlement.frozen",
      "settlement.frozen",
      "settlement.frozen",
    ],
    "notif007 dispute notification codes mismatch",
  );

  const workerSuccessCodes = worker.success_cases.map((entry) => entry.notification_code);
  for (const requiredCode of [
    "payment.succeeded",
    "delivery.completed",
    "acceptance.passed",
    "dispute.escalated",
  ]) {
    assert(
      workerSuccessCodes.includes(requiredCode),
      `notif012 worker live smoke missing success case ${requiredCode}`,
    );
  }
  assert(worker.retry.current_status === "processed", "notif012 retry status mismatch");
  assert(
    worker.dead_letter.current_status === "dead_lettered",
    "notif012 dead letter status mismatch",
  );
  assert(worker.metrics.duplicate >= 1, "notif012 duplicate metric mismatch");
  assert(worker.metrics.retrying >= 1, "notif012 retry metric mismatch");
  assert(worker.metrics.dead_lettered >= 1, "notif012 dead-letter metric mismatch");

  assert(paymentLookup.code === "OK", "payment lookup API code mismatch");
  assert(paymentLookup.message === "success", "payment lookup API message mismatch");
  assert(paymentLookup.data.total >= 1, "payment lookup API total mismatch");
  assert(
    paymentLookup.data.records.some((record) => record.notification_code === "payment.succeeded"),
    "payment lookup API missing payment.succeeded record",
  );
  assert(disputeLookup.code === "OK", "dispute lookup API code mismatch");
  assert(disputeLookup.message === "success", "dispute lookup API message mismatch");
  assert(disputeLookup.data.total >= 3, "dispute lookup API total mismatch");
  assert(
    disputeLookup.data.records.some((record) => record.notification_code === "dispute.escalated"),
    "dispute lookup API missing dispute.escalated record",
  );

  const lookupAudit = queryJson(`
SELECT json_build_object(
  'payment_lookup_audit_count', (
    SELECT COUNT(*)::int
    FROM audit.audit_event
    WHERE request_id = ${sqlString(lookupContext.payment_lookup_request_id)}
      AND action_name = 'notification.dispatch.lookup'
  ),
  'dispute_lookup_audit_count', (
    SELECT COUNT(*)::int
    FROM audit.audit_event
    WHERE request_id = ${sqlString(lookupContext.dispute_lookup_request_id)}
      AND action_name = 'notification.dispatch.lookup'
  )
);
  `);
  assert(lookupAudit.payment_lookup_audit_count === 1, "payment lookup audit count mismatch");
  assert(lookupAudit.dispute_lookup_audit_count === 1, "dispute lookup audit count mismatch");

  const summary = {
    task_id: "TEST-027",
    official_checker: "ENV_FILE=infra/docker/.env.local ./scripts/check-notification-smoke.sh",
    artifacts: {
      payment,
      delivery,
      acceptance,
      dispute,
      worker,
    },
    control_plane_lookup: {
      payment: paymentLookup,
      dispute: disputeLookup,
      audit_counts: lookupAudit,
    },
    runtime: {
      notification_worker: {
        live: workerLiveHealth,
        ready: workerReadyHealth,
      },
      outbox_publisher: {
        live: outboxPublisherLive,
        ready: outboxPublisherReady,
      },
    },
  };

  await writeFile(
    path.join(summaryDir, "summary.json"),
    JSON.stringify(summary, null, 2),
  );
}

main().catch((error) => {
  console.error(`[error] ${error.message}`);
  process.exit(1);
});
