#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const demoSeedVersion = "demo-v1-core-standard-scenarios";
const providerKey = "mock_payment";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function resolveRepoPath(relativePath) {
  return path.join(rootDir, relativePath);
}

async function readJson(relativePath) {
  const raw = await readFile(resolveRepoPath(relativePath), "utf8");
  return JSON.parse(raw);
}

function loadDbConfig() {
  return {
    host: process.env.DB_HOST ?? "127.0.0.1",
    port: process.env.DB_PORT ?? "55432",
    name: process.env.DB_NAME ?? "luna_data_trading",
    user: process.env.DB_USER ?? "luna",
    password: process.env.DB_PASSWORD ?? "5686",
  };
}

function dbEnv(config) {
  return {
    ...process.env,
    PGPASSWORD: config.password,
  };
}

function psqlArgs(config, extraArgs = []) {
  return [
    "-h",
    config.host,
    "-p",
    String(config.port),
    "-U",
    config.user,
    "-d",
    config.name,
    "-v",
    "ON_ERROR_STOP=1",
    "-X",
    "-q",
    ...extraArgs,
  ];
}

function queryText(config, sql) {
  return execFileSync("psql", [...psqlArgs(config), "-tAc", sql], {
    cwd: rootDir,
    env: dbEnv(config),
    stdio: ["ignore", "pipe", "pipe"],
  })
    .toString()
    .trim();
}

function queryJson(config, sql) {
  const output = queryText(config, sql);
  return output ? JSON.parse(output) : null;
}

async function main() {
  const dbConfig = loadDbConfig();
  const subjects = await readJson("fixtures/demo/subjects.json");
  const catalog = await readJson("fixtures/demo/catalog.json");
  const orders = await readJson("fixtures/demo/orders.json");
  const billing = await readJson("fixtures/demo/billing.json");
  const delivery = await readJson("fixtures/demo/delivery.json");

  const counts = queryJson(
    dbConfig,
    `SELECT json_build_object(
      'organizations', (SELECT COUNT(*) FROM core.organization WHERE org_id = ANY (ARRAY[${subjects.organizations
        .map((item) => `'${item.org_id}'::uuid`)
        .join(", ")}])),
      'users', (SELECT COUNT(*) FROM core.user_account WHERE user_id = ANY (ARRAY[${subjects.users
        .map((item) => `'${item.user_id}'::uuid`)
        .join(", ")}])),
      'applications', (SELECT COUNT(*) FROM core.application WHERE app_id = ANY (ARRAY[${subjects.applications
        .map((item) => `'${item.app_id}'::uuid`)
        .join(", ")}])),
      'products', (SELECT COUNT(*) FROM catalog.product WHERE product_id = ANY (ARRAY[${catalog.official_display_products
        .map((item) => `'${item.product_id}'::uuid`)
        .join(", ")}])),
      'orders', (SELECT COUNT(*) FROM trade.order_main WHERE order_id = ANY (ARRAY[${orders.order_blueprints
        .map((item) => `'${item.order_blueprint_id}'::uuid`)
        .join(", ")}])),
      'payment_intents', (SELECT COUNT(*) FROM payment.payment_intent WHERE payment_intent_id = ANY (ARRAY[${billing.billing_samples
        .map((item) => `'${item.payment_intent_id}'::uuid`)
        .join(", ")}])),
      'payment_transactions', (SELECT COUNT(*) FROM payment.payment_transaction WHERE payment_transaction_id = ANY (ARRAY[${billing.billing_samples
        .map((item) => `'44${item.payment_intent_id.slice(2)}'::uuid`)
        .join(", ")}])),
      'payment_webhooks', (SELECT COUNT(*) FROM payment.payment_webhook_event WHERE webhook_event_id = ANY (ARRAY[${billing.billing_samples
        .map((item) => `'45${item.payment_intent_id.slice(2)}'::uuid`)
        .join(", ")}])),
      'billing_events', (SELECT COUNT(*) FROM billing.billing_event WHERE billing_event_id = ANY (ARRAY[${billing.billing_samples
        .map((item) => `'${item.billing_sample_id}'::uuid`)
        .join(", ")}])),
      'delivery_records', (SELECT COUNT(*) FROM delivery.delivery_record WHERE delivery_id = ANY (ARRAY[${delivery.delivery_blueprints
        .map((item) => `'${item.delivery_blueprint_id}'::uuid`)
        .join(", ")}])),
      'api_credentials', (SELECT COUNT(*) FROM delivery.api_credential WHERE api_credential_id = ANY (ARRAY[${delivery.delivery_blueprints
        .filter((item) => item.delivery_object_kind === 'api_access' || item.delivery_object_kind === 'api_metered_access')
        .map((item) => `'${item.delivery_blueprint_id}'::uuid`)
        .join(", ")}])),
      'api_usage_logs', (SELECT COUNT(*) FROM delivery.api_usage_log WHERE api_usage_log_id = ANY (ARRAY[${delivery.delivery_blueprints
        .filter((item) => item.delivery_object_kind === 'api_access' || item.delivery_object_kind === 'api_metered_access')
        .map((item) => `'65${item.delivery_blueprint_id.slice(2)}'::uuid`)
        .join(", ")}])),
      'sandbox_workspaces', (SELECT COUNT(*) FROM delivery.sandbox_workspace WHERE sandbox_workspace_id = '41000000-0000-0000-0000-000000000005'::uuid),
      'sandbox_sessions', (SELECT COUNT(*) FROM delivery.sandbox_session WHERE sandbox_session_id = '64000000-0000-0000-0000-000000000005'::uuid),
      'share_grants', (SELECT COUNT(*) FROM delivery.data_share_grant WHERE data_share_grant_id = '41000000-0000-0000-0000-000000000006'::uuid),
      'revision_subscriptions', (SELECT COUNT(*) FROM delivery.revision_subscription WHERE revision_subscription_id = '41000000-0000-0000-0000-000000000004'::uuid),
      'template_query_grants', (SELECT COUNT(*) FROM delivery.template_query_grant WHERE template_query_grant_id = '41000000-0000-0000-0000-000000000009'::uuid),
      'query_runs', (SELECT COUNT(*) FROM delivery.query_execution_run WHERE query_run_id = '41000000-0000-0000-0000-000000000010'::uuid),
      'report_artifacts', (SELECT COUNT(*) FROM delivery.report_artifact WHERE report_artifact_id = ANY (ARRAY['41000000-0000-0000-0000-000000000008'::uuid, '41000000-0000-0000-0000-000000000011'::uuid])),
      'delivery_tickets', (SELECT COUNT(*) FROM delivery.delivery_ticket WHERE ticket_id = ANY (ARRAY['71000000-0000-0000-0000-000000000003'::uuid, '71000000-0000-0000-0000-000000000008'::uuid, '71000000-0000-0000-0000-000000000010'::uuid, '71000000-0000-0000-0000-000000000011'::uuid])),
      'storage_objects', (SELECT COUNT(*) FROM delivery.storage_object WHERE object_id = ANY (ARRAY['51000000-0000-0000-0000-000000000003'::uuid, '51000000-0000-0000-0000-000000000008'::uuid, '51000000-0000-0000-0000-000000000010'::uuid, '51000000-0000-0000-0000-000000000011'::uuid])),
      'asset_objects', (SELECT COUNT(*) FROM catalog.asset_object_binding WHERE asset_object_id = ANY (ARRAY['61000000-0000-0000-0000-000000000311'::uuid, '61000000-0000-0000-0000-000000000313'::uuid])),
      'query_surfaces', (SELECT COUNT(*) FROM catalog.query_surface_definition WHERE query_surface_id = '62000000-0000-0000-0000-000000000313'::uuid),
      'query_templates', (SELECT COUNT(*) FROM delivery.query_template_definition WHERE query_template_id = '63000000-0000-0000-0000-000000000313'::uuid),
      'seed_history', (SELECT COUNT(*) FROM public.seed_history WHERE version = '${demoSeedVersion}')
    )::text`
  );

  assert(counts.organizations === subjects.organizations.length, `organization count mismatch: ${counts.organizations}`);
  assert(counts.users === subjects.users.length, `user count mismatch: ${counts.users}`);
  assert(counts.applications === subjects.applications.length, `application count mismatch: ${counts.applications}`);
  assert(counts.products === catalog.official_display_products.length, `product count mismatch: ${counts.products}`);
  assert(counts.orders === orders.order_blueprints.length, `order count mismatch: ${counts.orders}`);
  assert(counts.payment_intents === billing.billing_samples.length, `payment_intent count mismatch: ${counts.payment_intents}`);
  assert(counts.payment_transactions === billing.billing_samples.length, `payment_transaction count mismatch: ${counts.payment_transactions}`);
  assert(counts.payment_webhooks === billing.billing_samples.length, `payment_webhook count mismatch: ${counts.payment_webhooks}`);
  assert(counts.billing_events === billing.billing_samples.length, `billing_event count mismatch: ${counts.billing_events}`);
  assert(counts.delivery_records === delivery.delivery_blueprints.length, `delivery_record count mismatch: ${counts.delivery_records}`);
  assert(counts.api_credentials === 3, `api_credential count mismatch: ${counts.api_credentials}`);
  assert(counts.api_usage_logs === 3, `api_usage_log count mismatch: ${counts.api_usage_logs}`);
  assert(counts.sandbox_workspaces === 1, `sandbox_workspace count mismatch: ${counts.sandbox_workspaces}`);
  assert(counts.sandbox_sessions === 1, `sandbox_session count mismatch: ${counts.sandbox_sessions}`);
  assert(counts.share_grants === 1, `data_share_grant count mismatch: ${counts.share_grants}`);
  assert(counts.revision_subscriptions === 1, `revision_subscription count mismatch: ${counts.revision_subscriptions}`);
  assert(counts.template_query_grants === 1, `template_query_grant count mismatch: ${counts.template_query_grants}`);
  assert(counts.query_runs === 1, `query_execution_run count mismatch: ${counts.query_runs}`);
  assert(counts.report_artifacts === 2, `report_artifact count mismatch: ${counts.report_artifacts}`);
  assert(counts.delivery_tickets === 4, `delivery_ticket count mismatch: ${counts.delivery_tickets}`);
  assert(counts.storage_objects === 4, `storage_object count mismatch: ${counts.storage_objects}`);
  assert(counts.asset_objects === 2, `asset_object count mismatch: ${counts.asset_objects}`);
  assert(counts.query_surfaces === 1, `query_surface count mismatch: ${counts.query_surfaces}`);
  assert(counts.query_templates === 1, `query_template count mismatch: ${counts.query_templates}`);
  assert(counts.seed_history === 1, `seed_history trace missing: ${counts.seed_history}`);

  const providerKeys = queryJson(
    dbConfig,
    `SELECT COALESCE(json_agg(DISTINCT provider_key), '[]'::json)::text
     FROM payment.payment_intent
     WHERE payment_intent_id = ANY (ARRAY[${billing.billing_samples
       .map((item) => `'${item.payment_intent_id}'::uuid`)
       .join(", ")}])`
  );
  assert(
    Array.isArray(providerKeys) && providerKeys.length === 1 && providerKeys[0] === providerKey,
    `payment provider key drift: ${JSON.stringify(providerKeys)}`
  );

  const seededOrderCount = Number(
    queryText(
      dbConfig,
      `SELECT COUNT(*) FROM trade.order_main
       WHERE order_id = ANY (ARRAY[${orders.order_blueprints
         .map((item) => `'${item.order_blueprint_id}'::uuid`)
         .join(", ")}])
         AND price_snapshot_json ->> 'seed_source' = 'fixtures/demo'`
    )
  );
  assert(seededOrderCount === orders.order_blueprints.length, `seed source marker missing on demo orders: ${seededOrderCount}`);

  const seededDeliveryCount = Number(
    queryText(
      dbConfig,
      `SELECT COUNT(*) FROM delivery.delivery_record
       WHERE delivery_id = ANY (ARRAY[${delivery.delivery_blueprints
         .map((item) => `'${item.delivery_blueprint_id}'::uuid`)
         .join(", ")}])
         AND trust_boundary_snapshot ->> 'seed_source' = 'fixtures/demo'`
    )
  );
  assert(seededDeliveryCount === delivery.delivery_blueprints.length, `seed source marker missing on delivery records: ${seededDeliveryCount}`);

  console.log("[ok] demo seed verified");
  console.log(`  - organizations: ${counts.organizations}`);
  console.log(`  - users: ${counts.users}`);
  console.log(`  - applications: ${counts.applications}`);
  console.log(`  - official products: ${counts.products}`);
  console.log(`  - demo orders: ${counts.orders}`);
  console.log(`  - payment intents/transactions/webhooks/billing: ${counts.payment_intents}/${counts.payment_transactions}/${counts.payment_webhooks}/${counts.billing_events}`);
  console.log(`  - delivery records/api creds/api usage: ${counts.delivery_records}/${counts.api_credentials}/${counts.api_usage_logs}`);
  console.log(`  - storage tickets/reports/query grants: ${counts.storage_objects}/${counts.delivery_tickets}/${counts.report_artifacts}/${counts.template_query_grants}`);
  console.log(`  - sandbox/share/revision/query-run: ${counts.sandbox_workspaces}/${counts.share_grants}/${counts.revision_subscriptions}/${counts.query_runs}`);
}

main().catch((error) => {
  console.error(`[fail] ${error.message}`);
  process.exit(1);
});
