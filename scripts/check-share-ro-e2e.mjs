#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const defaultRawDir = path.join(rootDir, "target/test-artifacts/share-ro-e2e/raw");
const defaultSummaryDir = path.join(rootDir, "target/test-artifacts/share-ro-e2e");
const rawDir = process.env.TEST025_ARTIFACT_DIR || defaultRawDir;
const summaryDir = process.env.TEST025_SUMMARY_DIR || defaultSummaryDir;
const portalArtifactPath =
  process.env.TEST025_PORTAL_ARTIFACT_FILE || path.join(rawDir, "portal-share-live.json");
const fixtureFile =
  process.env.TEST025_FIXTURE_FILE || path.join(summaryDir, "live-fixture.json");
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

async function main() {
  await mkdir(summaryDir, { recursive: true });

  const requiredFiles = [
    "trade012-share-ro-state-machine.json",
    "dlv006-share-grant.json",
    "bil026-share-ro-billing.json",
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

  const trade012 = artifacts.get("trade012-share-ro-state-machine.json");
  assert(trade012.order.current_state === "dispute_interrupted", "trade012 final state mismatch");
  assert(trade012.order.settlement_status === "frozen", "trade012 settlement freeze mismatch");
  assert(
    String(trade012.conflict.message || "").includes("SHARE_RO_TRANSITION_FORBIDDEN"),
    "trade012 conflict guard mismatch",
  );

  const dlv006 = artifacts.get("dlv006-share-grant.json");
  assert(dlv006.grant_response.current_state === "share_granted", "dlv006 grant state mismatch");
  assert(dlv006.revoke_response.current_state === "revoked", "dlv006 revoke state mismatch");
  assert(dlv006.audit_counts.delivery_share_enable === 2, "dlv006 manage audit count mismatch");
  assert(dlv006.audit_counts.delivery_share_read === 1, "dlv006 read audit count mismatch");
  assert(
    dlv006.outbox.delivery_committed_topic === "dtp.outbox.domain-events",
    "dlv006 outbox topic mismatch",
  );
  assert(
    dlv006.outbox.billing_trigger === "bill_once_on_grant_effective",
    "dlv006 billing trigger mismatch",
  );

  const bil026 = artifacts.get("bil026-share-ro-billing.json");
  assert(
    bil026.cycle_order.db_counts.refund_adjustment === 1,
    "bil026 refund placeholder count mismatch",
  );
  assert(
    bil026.cycle_order.db_counts.settlement_status === "refunded",
    "bil026 cycle settlement mismatch",
  );
  assert(
    bil026.cycle_order.cycle_replay_replayed === true,
    "bil026 replay idempotency mismatch",
  );
  assert(
    bil026.dispute_order.settlement_status === "frozen",
    "bil026 dispute settlement freeze mismatch",
  );
  assert(
    bil026.dispute_order.dispute_status === "opened",
    "bil026 dispute status mismatch",
  );

  assert(portal.order_id === fixture.order_id, "portal artifact order_id drifted from live fixture");
  assert(portal.grant_response.current_state === "share_granted", "portal grant state mismatch");
  assert(
    portal.buyer_read_response.target_grant_status === "active",
    "portal buyer read state mismatch",
  );
  assert(portal.revoke_response.current_state === "revoked", "portal revoke state mismatch");
  assert(
    portal.order_after_revoke.current_state === "revoked",
    "portal order after revoke mismatch",
  );
  assert(
    portal.share_after_revoke.target_grant_status === "revoked",
    "portal share after revoke mismatch",
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
  'latest_grant_status', (
    SELECT grant_status
    FROM delivery.data_share_grant
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY created_at DESC, data_share_grant_id DESC
    LIMIT 1
  ),
  'latest_grant_recipient_ref', (
    SELECT recipient_ref
    FROM delivery.data_share_grant
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY created_at DESC, data_share_grant_id DESC
    LIMIT 1
  ),
  'latest_grant_receipt_hash', (
    SELECT receipt_hash
    FROM delivery.data_share_grant
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY created_at DESC, data_share_grant_id DESC
    LIMIT 1
  ),
  'latest_delivery_record_status', (
    SELECT status
    FROM delivery.delivery_record
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY updated_at DESC, delivery_id DESC
    LIMIT 1
  ),
  'latest_delivery_record_receipt_hash', (
    SELECT receipt_hash
    FROM delivery.delivery_record
    WHERE order_id = ${sqlString(portal.order_id)}::uuid
    ORDER BY updated_at DESC, delivery_id DESC
    LIMIT 1
  ),
  'manage_audit_count', (
    SELECT COUNT(*)::int
    FROM audit.audit_event
    WHERE action_name = 'delivery.share.enable'
      AND request_id IN (
        ${sqlString(portal.portal_proxy_requests.grant_request_id)},
        ${sqlString(portal.portal_proxy_requests.revoke_request_id)}
      )
  ),
  'trade_transition_audit_count', (
    SELECT COUNT(*)::int
    FROM audit.audit_event
    WHERE action_name = 'trade.order.share_ro.transition'
      AND request_id IN (
        ${sqlString(portal.portal_proxy_requests.grant_request_id)},
        ${sqlString(portal.portal_proxy_requests.revoke_request_id)}
      )
  ),
  'read_audit_count', (
    SELECT COUNT(*)::int
    FROM audit.audit_event
    WHERE action_name = 'delivery.share.read'
      AND request_id = ${sqlString(portal.portal_proxy_requests.buyer_read_request_id)}
  ),
  'delivery_outbox_topic', (
    SELECT target_topic
    FROM ops.outbox_event
    WHERE request_id = ${sqlString(portal.portal_proxy_requests.grant_request_id)}
      AND event_type = 'delivery.committed'
    ORDER BY created_at DESC, outbox_event_id DESC
    LIMIT 1
  ),
  'delivery_outbox_branch', (
    SELECT payload ->> 'delivery_branch'
    FROM ops.outbox_event
    WHERE request_id = ${sqlString(portal.portal_proxy_requests.grant_request_id)}
      AND event_type = 'delivery.committed'
    ORDER BY created_at DESC, outbox_event_id DESC
    LIMIT 1
  ),
  'billing_bridge_trigger', (
    SELECT payload -> 'billing_trigger_matrix' ->> 'billing_trigger'
    FROM ops.outbox_event
    WHERE request_id = ${sqlString(portal.portal_proxy_requests.grant_request_id)}
      AND event_type = 'billing.trigger.bridge'
    ORDER BY created_at DESC, outbox_event_id DESC
    LIMIT 1
  )
)::text
`);

  assert(dbEvidence.order_state === "revoked", "portal db order state mismatch");
  assert(dbEvidence.payment_status === "paid", "portal db payment status mismatch");
  assert(dbEvidence.delivery_status === "closed", "portal db delivery status mismatch");
  assert(dbEvidence.acceptance_status === "closed", "portal db acceptance status mismatch");
  assert(dbEvidence.settlement_status === "closed", "portal db settlement status mismatch");
  assert(dbEvidence.latest_grant_status === "revoked", "portal db grant status mismatch");
  assert(
    dbEvidence.latest_grant_recipient_ref === portal.recipient_ref,
    "portal db recipient_ref mismatch",
  );
  assert(
    dbEvidence.latest_grant_receipt_hash === portal.revoke_receipt_hash,
    "portal db grant receipt mismatch",
  );
  assert(
    dbEvidence.latest_delivery_record_status === "revoked",
    "portal db delivery record status mismatch",
  );
  assert(
    dbEvidence.latest_delivery_record_receipt_hash === portal.revoke_receipt_hash,
    "portal db delivery record receipt mismatch",
  );
  assert(dbEvidence.manage_audit_count === 2, "portal manage audit count mismatch");
  assert(
    dbEvidence.trade_transition_audit_count === 2,
    "portal trade transition audit count mismatch",
  );
  assert(dbEvidence.read_audit_count === 1, "portal read audit count mismatch");
  assert(
    dbEvidence.delivery_outbox_topic === "dtp.outbox.domain-events",
    "portal outbox topic mismatch",
  );
  assert(dbEvidence.delivery_outbox_branch === "share", "portal outbox branch mismatch");
  assert(
    dbEvidence.billing_bridge_trigger === "bill_once_on_grant_effective",
    "portal billing bridge trigger mismatch",
  );

  const signoffOrderIds = [
    trade012.order.order_id,
    dlv006.order.order_id,
    bil026.cycle_order.order_id,
    bil026.dispute_order.order_id,
    portal.order_id,
  ];
  const uniqueOrderIds = [...new Set(signoffOrderIds)];
  const summary = {
    task_id: "TEST-025",
    required_artifacts: [...requiredFiles, path.basename(portalArtifactPath)],
    fixture_order_id: fixture.order_id,
    signoff_order_ids: uniqueOrderIds,
    signoff_order_count: uniqueOrderIds.length,
    checkpoints: {
      share_state_machine: true,
      share_grant_revoke_backend: true,
      share_billing_refund_placeholder: true,
      share_dispute_freeze: true,
      portal_seller_grant: true,
      portal_buyer_read: true,
      portal_seller_revoke: true,
      audit_and_outbox_readback: true,
      browser_boundary_preserved: true,
    },
    portal: {
      order_id: portal.order_id,
      portal_proxy_requests: portal.portal_proxy_requests,
      db_evidence: dbEvidence,
    },
  };

  await writeFile(
    path.join(summaryDir, "summary.json"),
    JSON.stringify(summary, null, 2),
  );

  console.log(
    `[ok] TEST-025 SHARE_RO verified: ${summary.signoff_order_count} sign-off orders ` +
      `(state machine + grant/revoke + billing/dispute + portal live)`
  );
}

main().catch((error) => {
  console.error(`[fail] ${error.message}`);
  process.exit(1);
});
