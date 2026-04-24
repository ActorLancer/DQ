#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const demoSeedVersion = "demo-v1-core-standard-scenarios";
const demoSeedName = "fixtures/demo bundle";
const providerKey = "mock_payment";

function usage() {
  console.log(`Usage: ./scripts/seed-demo.sh [--manifest <path>] [--skip-base-seeds] [--dry-run] [--no-verify]

Options:
  --manifest <path>     Override the base seed manifest. Default: db/seeds/manifest.csv
  --skip-base-seeds     Skip ./db/scripts/seed-up.sh and only import TEST demo orders/payment/delivery rows
  --dry-run             Print the import plan without writing demo data
  --no-verify           Skip ./scripts/check-demo-seed.sh after import
  --help                Show this message

Environment:
  DB_HOST DB_PORT DB_NAME DB_USER DB_PASSWORD
  Defaults match db/scripts/seed-runner.sh.`);
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
    assert(value, `missing ${key}`);
    assert(!mapped.has(value), `duplicate ${key}: ${value}`);
    mapped.set(value, item);
  }
  return mapped;
}

function resolveRepoPath(relativePath) {
  return path.join(rootDir, relativePath);
}

async function readJson(relativePath) {
  const raw = await readFile(resolveRepoPath(relativePath), "utf8");
  return JSON.parse(raw);
}

function parseArgs(argv) {
  const args = {
    manifest: "db/seeds/manifest.csv",
    skipBaseSeeds: false,
    dryRun: false,
    verify: true,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    switch (current) {
      case "--manifest":
        assert(argv[index + 1], "missing value for --manifest");
        args.manifest = argv[index + 1];
        index += 1;
        break;
      case "--skip-base-seeds":
        args.skipBaseSeeds = true;
        break;
      case "--dry-run":
        args.dryRun = true;
        break;
      case "--no-verify":
        args.verify = false;
        break;
      case "--help":
      case "-h":
        usage();
        process.exit(0);
        break;
      default:
        throw new Error(`unknown argument: ${current}`);
    }
  }

  return args;
}

function loadDbConfig() {
  return {
    host: process.env.DB_HOST ?? "127.0.0.1",
    port: process.env.DB_PORT ?? "5432",
    name: process.env.DB_NAME ?? "datab",
    user: process.env.DB_USER ?? "datab",
    password: process.env.DB_PASSWORD ?? "datab_local_pass",
  };
}

function dbEnv(config) {
  return {
    ...process.env,
    DB_HOST: config.host,
    DB_PORT: String(config.port),
    DB_NAME: config.name,
    DB_USER: config.user,
    DB_PASSWORD: config.password,
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

function runCommand(command, args, options = {}) {
  return execFileSync(command, args, {
    cwd: rootDir,
    env: options.env ?? process.env,
    stdio: options.stdio ?? ["pipe", "pipe", "pipe"],
    input: options.input,
  });
}

function queryText(config, sql) {
  return runCommand("psql", [...psqlArgs(config), "-tAc", sql], {
    env: dbEnv(config),
  })
    .toString()
    .trim();
}

function queryJson(config, sql) {
  const output = queryText(config, sql);
  return output ? JSON.parse(output) : null;
}

function runPsqlScript(config, sql) {
  runCommand("psql", psqlArgs(config), {
    env: dbEnv(config),
    input: sql,
  });
}

function runBaseSeeds(config, manifest, dryRun) {
  const seedArgs = [];
  if (manifest) {
    seedArgs.push("--manifest", manifest);
  }
  if (dryRun) {
    seedArgs.push("--dry-run");
  }
  runCommand("./db/scripts/seed-up.sh", seedArgs, {
    env: dbEnv(config),
    stdio: "inherit",
  });
}

function stableHash(prefix, value) {
  return `${prefix}:${createHash("sha256").update(value).digest("hex")}`;
}

function jsonLiteral(value) {
  return `'${JSON.stringify(value).replace(/'/g, "''")}'::jsonb`;
}

function deriveUuid(sourceUuid, prefix) {
  assert(/^[0-9a-f]{8}-/i.test(sourceUuid), `invalid uuid source: ${sourceUuid}`);
  assert(prefix.length === 2, `uuid prefix must be 2 chars: ${prefix}`);
  return `${prefix}${sourceUuid.slice(2)}`;
}

function shiftIso(isoString, { days = 0, hours = 0, minutes = 0 } = {}) {
  const next = new Date(isoString);
  next.setUTCDate(next.getUTCDate() + days);
  next.setUTCHours(next.getUTCHours() + hours);
  next.setUTCMinutes(next.getUTCMinutes() + minutes);
  return next.toISOString();
}

function buildTimeline(orderBlueprint) {
  const scenarioIndex = Number(orderBlueprint.scenario_code.slice(1)) - 1;
  const startHour = orderBlueprint.scenario_role === "primary" ? 9 : 14;
  const createdAt = new Date(Date.UTC(2026, 0, 10 + scenarioIndex, startHour, 0, 0)).toISOString();
  const buyerLockedAt = orderBlueprint.payment_status === "paid" ? shiftIso(createdAt, { minutes: 10 }) : null;
  const deliveredAt = orderBlueprint.delivery_status === "completed" ? shiftIso(createdAt, { hours: 2 }) : null;
  const acceptedAt = orderBlueprint.acceptance_status === "accepted" ? shiftIso(createdAt, { hours: 3 }) : null;
  return {
    createdAt,
    buyerLockedAt,
    deliveredAt,
    acceptedAt,
  };
}

function inferQuantity(orderBlueprint, sku) {
  if (sku.sku_type === "API_PPU") {
    return "1000";
  }
  return "1";
}

function normalizeBillingMode(billingMode) {
  return billingMode === "metered" ? "pay_per_use" : billingMode;
}

function deriveSettlementBasis(billingMode, pricingMode) {
  switch (`${billingMode}:${pricingMode}`) {
    case "one_time:one_time":
    case "one_time:subscription":
    case "one_time:pay_per_use":
      return "one_time_final";
    case "subscription:one_time":
    case "subscription:subscription":
    case "subscription:pay_per_use":
      return "periodic_cycle";
    case "pay_per_use:one_time":
    case "pay_per_use:subscription":
    case "pay_per_use:pay_per_use":
      return "usage_metered";
    case "unknown:subscription":
      return "periodic_cycle";
    case "unknown:pay_per_use":
      return "usage_metered";
    default:
      return "manual_v1_default";
  }
}

function buildScenarioSkuSnapshot(orderBlueprint, sku, scenarioTemplate) {
  return {
    scenario_code: orderBlueprint.scenario_code,
    scenario_name: scenarioTemplate.scenario_name,
    selected_sku_id: orderBlueprint.sku_id,
    selected_sku_code: sku.sku_type,
    selected_sku_type: sku.sku_type,
    selected_sku_role: orderBlueprint.scenario_snapshot.selected_sku_role,
    primary_sku: scenarioTemplate.primary_sku,
    supplementary_skus: scenarioTemplate.supplementary_skus,
    contract_template: orderBlueprint.template_codes.contract_template,
    acceptance_template: orderBlueprint.template_codes.acceptance_template,
    refund_template: orderBlueprint.template_codes.refund_template,
    per_sku_snapshot_required: true,
    multi_sku_requires_independent_contract_authorization_settlement: true,
  };
}

function buildOrderPriceSnapshot(orderBlueprint, sku, scenarioTemplate, timeline) {
  const quantity = Number(inferQuantity(orderBlueprint, sku));
  const billingMode = normalizeBillingMode(sku.billing_mode);
  const pricingMode = billingMode;
  const unitPrice =
    quantity > 1
      ? (Number(orderBlueprint.order_amount) / quantity).toFixed(2)
      : orderBlueprint.order_amount;

  return {
    seed_source: "fixtures/demo",
    task_id: "TEST-002",
    product_id: orderBlueprint.product_id,
    sku_id: orderBlueprint.sku_id,
    sku_code: sku.sku_type,
    sku_type: sku.sku_type,
    pricing_mode: pricingMode,
    unit_price: unitPrice,
    currency_code: orderBlueprint.currency_code,
    billing_mode: billingMode,
    refund_mode: sku.refund_mode,
    settlement_terms: {
      settlement_basis: deriveSettlementBasis(billingMode, pricingMode),
      settlement_mode: "manual_v1",
    },
    tax_terms: {
      tax_policy: "platform_default",
      tax_code: "UNSPECIFIED",
      tax_inclusive: false,
    },
    scenario_snapshot: buildScenarioSkuSnapshot(orderBlueprint, sku, scenarioTemplate),
    captured_at: timeline.createdAt,
    source: "fixtures/demo/TEST-002",
  };
}

function inferGrantType(deliveryObjectKind) {
  switch (deliveryObjectKind) {
    case "api_access":
      return "api_access";
    case "api_metered_access":
      return "api_metered";
    case "encrypted_file_package":
      return "file_download";
    case "revision_subscription":
      return "file_subscription";
    case "sandbox_workspace":
      return "sandbox_access";
    case "share_grant":
      return "share_readonly";
    case "report_artifact":
      return "report_download";
    case "template_query_grant":
    case "query_result_artifact":
      return "query_template";
    default:
      return deliveryObjectKind;
  }
}

function buildSeedPayloads(fixtures, subjectData, catalogData, baseProductRows) {
  const productMap = mapBy(baseProductRows, "product_id");
  const skuMap = mapBy(catalogData.official_display_skus, "sku_id");
  const scenarioTemplateMap = mapBy(catalogData.scenario_templates, "scenario_code");
  const orderMap = mapBy(fixtures.orders.order_blueprints, "order_blueprint_id");
  const billingByOrder = new Map();
  for (const billingSample of fixtures.billing.billing_samples) {
    billingByOrder.set(billingSample.order_blueprint_id, billingSample);
  }
  const deliveryByOrder = new Map();
  for (const deliveryBlueprint of fixtures.delivery.delivery_blueprints) {
    const list = deliveryByOrder.get(deliveryBlueprint.order_blueprint_id) ?? [];
    list.push(deliveryBlueprint);
    deliveryByOrder.set(deliveryBlueprint.order_blueprint_id, list);
  }

  const appId = subjectData.applications[0]?.app_id;
  assert(appId, "fixtures/demo/subjects.json must provide an application for demo api delivery");

  const shareOrder = orderMap.get("34000000-0000-0000-0000-000000000103");
  const queryOrder = orderMap.get("34000000-0000-0000-0000-000000000005");
  assert(shareOrder, "missing SHARE_RO order blueprint");
  assert(queryOrder, "missing QRY_LITE order blueprint");

  const shareObjectId = deriveUuid(shareOrder.product_id, "61");
  const queryObjectId = deriveUuid(queryOrder.product_id, "61");
  const querySurfaceId = deriveUuid(queryOrder.product_id, "62");
  const queryTemplateId = deriveUuid(queryOrder.product_id, "63");

  const orderRows = [];
  const orderLineRows = [];
  const authorizationGrantRows = [];
  const paymentIntentRows = [];
  const paymentTransactionRows = [];
  const paymentWebhookRows = [];
  const billingEventRows = [];
  const assetObjectRows = [];
  const querySurfaceRows = [];
  const queryTemplateRows = [];
  const storageObjectRows = [];
  const deliveryRecordRows = [];
  const deliveryTicketRows = [];
  const apiCredentialRows = [];
  const apiUsageRows = [];
  const sandboxWorkspaceRows = [];
  const sandboxSessionRows = [];
  const shareGrantRows = [];
  const revisionSubscriptionRows = [];
  const templateQueryGrantRows = [];
  const queryRunRows = [];
  const reportArtifactRows = [];

  assetObjectRows.push(
    {
      asset_object_id: shareObjectId,
      asset_version_id: productMap.get(shareOrder.product_id).asset_version_id,
      object_kind: "structured_dataset",
      object_name: "demo-s3-share-object",
      object_locator: "warehouse://seller/test001/s3/orders-inventory",
      share_protocol: "share_grant",
      schema_json: {
        fields: [
          { name: "order_date", type: "date" },
          { name: "inventory_turnover", type: "number" },
        ],
      },
      output_schema_json: {
        fields: [
          { name: "order_date", type: "date" },
          { name: "inventory_turnover", type: "number" },
        ],
      },
      freshness_json: { cadence: "daily" },
      access_constraints: { read_only: true, exportable: false },
      metadata: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        scenario_code: "S3",
      },
    },
    {
      asset_object_id: queryObjectId,
      asset_version_id: productMap.get(queryOrder.product_id).asset_version_id,
      object_kind: "structured_dataset",
      object_name: "demo-s5-query-object",
      object_locator: "warehouse://seller/test001/s5/location-score",
      share_protocol: null,
      schema_json: {
        fields: [
          { name: "district", type: "string" },
          { name: "poi_density", type: "number" },
          { name: "footfall_score", type: "number" },
        ],
      },
      output_schema_json: {
        fields: [
          { name: "candidate_area", type: "string" },
          { name: "score", type: "number" },
          { name: "rationale", type: "string" },
        ],
      },
      freshness_json: { cadence: "monthly" },
      access_constraints: { read_only: true, exportable: false },
      metadata: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        scenario_code: "S5",
      },
    }
  );

  querySurfaceRows.push({
    query_surface_id: querySurfaceId,
    asset_version_id: productMap.get(queryOrder.product_id).asset_version_id,
    asset_object_id: queryObjectId,
    surface_type: "template_query_lite",
    binding_mode: "managed_surface",
    execution_scope: "curated_zone",
    input_contract_json: {
      required: ["city", "radius_km", "limit"],
      source_zones: ["curated_zone"],
    },
    output_boundary_json: {
      allow_raw_export: false,
      allowed_formats: ["json"],
      max_rows: 5,
      max_cells: 15,
    },
    query_policy_json: {
      analysis_rule: "whitelist_only",
      review_status: "approved",
    },
    quota_policy_json: {
      daily_limit: 2,
      monthly_limit: 8,
    },
    status: "active",
    metadata: {
      seed_source: "fixtures/demo",
      task_id: "TEST-002",
      scenario_code: "S5",
    },
  });

  queryTemplateRows.push({
    query_template_id: queryTemplateId,
    query_surface_id: querySurfaceId,
    template_name: "tpl_demo_s5_location_score_v1",
    template_type: "sql_template",
    template_body_ref: "minio://delivery-objects/templates/test002/s5/location_score_v1.sql",
    parameter_schema_json: {
      type: "object",
      required: ["city", "radius_km", "limit"],
      properties: {
        city: { type: "string" },
        radius_km: { type: "integer" },
        limit: { type: "integer" },
      },
    },
    analysis_rule_json: {
      analysis_rule: "whitelist_only",
      template_review_status: "approved",
    },
    result_schema_json: {
      fields: [
        { name: "candidate_area", type: "string" },
        { name: "score", type: "number" },
        { name: "rationale", type: "string" },
      ],
    },
    export_policy_json: {
      allow_raw_export: false,
      allowed_formats: ["json"],
      max_export_rows: 5,
      max_export_cells: 15,
    },
    risk_guard_json: {
      risk_mode: "strict",
      result_redaction: "masked_only",
    },
    status: "active",
    version_no: 1,
  });

  for (const orderBlueprint of fixtures.orders.order_blueprints) {
    const sku = skuMap.get(orderBlueprint.sku_id);
    const baseProduct = productMap.get(orderBlueprint.product_id);
    const billingSample = billingByOrder.get(orderBlueprint.order_blueprint_id);
    const deliveryBlueprints = deliveryByOrder.get(orderBlueprint.order_blueprint_id) ?? [];
    const primaryDeliveryBlueprint = deliveryBlueprints[0];
    const timeline = buildTimeline(orderBlueprint);
    const scenarioTemplate = scenarioTemplateMap.get(orderBlueprint.scenario_code);
    assert(sku, `missing sku for order ${orderBlueprint.order_blueprint_id}`);
    assert(baseProduct, `missing base product for order ${orderBlueprint.order_blueprint_id}`);
    assert(billingSample, `missing billing sample for order ${orderBlueprint.order_blueprint_id}`);
    assert(primaryDeliveryBlueprint, `missing delivery blueprint for order ${orderBlueprint.order_blueprint_id}`);
    assert(
      scenarioTemplate,
      `missing scenario template for order ${orderBlueprint.order_blueprint_id}`
    );

    orderRows.push({
      order_id: orderBlueprint.order_blueprint_id,
      product_id: orderBlueprint.product_id,
      asset_version_id: baseProduct.asset_version_id,
      buyer_org_id: orderBlueprint.buyer_org_id,
      seller_org_id: orderBlueprint.seller_org_id,
      sku_id: orderBlueprint.sku_id,
      status: orderBlueprint.current_state,
      payment_status: orderBlueprint.payment_status,
      payment_mode: "online",
      amount: orderBlueprint.order_amount,
      currency_code: orderBlueprint.currency_code,
      fee_preview_snapshot: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        billing_mode: billingSample.billing_mode,
      },
      payment_channel_snapshot: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        provider_key: providerKey,
        callback_topic: fixtures.billing.payment_provider.callback_topic,
      },
      price_snapshot_json: buildOrderPriceSnapshot(
        orderBlueprint,
        sku,
        scenarioTemplate,
        timeline
      ),
      trust_boundary_snapshot: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        delivery_objects: deliveryBlueprints.map((item) => item.delivery_object_kind),
      },
      storage_mode_snapshot: "platform_custody",
      delivery_route_snapshot: primaryDeliveryBlueprint.delivery_object_kind,
      platform_plaintext_access_snapshot: false,
      idempotency_key: orderBlueprint.idempotency_key,
      created_at: timeline.createdAt,
      buyer_locked_at: timeline.buyerLockedAt,
      delivered_at: timeline.deliveredAt,
      accepted_at: timeline.acceptedAt,
      delivery_status: orderBlueprint.delivery_status,
      acceptance_status: orderBlueprint.acceptance_status,
      settlement_status: orderBlueprint.settlement_status,
      dispute_status: orderBlueprint.dispute_status,
    });

    const quantity = inferQuantity(orderBlueprint, sku);
    orderLineRows.push({
      order_line_id: deriveUuid(orderBlueprint.order_blueprint_id, "35"),
      order_id: orderBlueprint.order_blueprint_id,
      sku_id: orderBlueprint.sku_id,
      quantity,
      unit_price:
        sku.sku_type === "API_PPU"
          ? (Number(orderBlueprint.order_amount) / Number(quantity)).toFixed(2)
          : orderBlueprint.order_amount,
      amount: orderBlueprint.order_amount,
    });

    authorizationGrantRows.push({
      authorization_grant_id: deriveUuid(orderBlueprint.order_blueprint_id, "36"),
      order_id: orderBlueprint.order_blueprint_id,
      grant_type: inferGrantType(primaryDeliveryBlueprint.delivery_object_kind),
      granted_to_type:
        primaryDeliveryBlueprint.delivery_object_kind === "api_access" ||
        primaryDeliveryBlueprint.delivery_object_kind === "api_metered_access"
          ? "app"
          : "org",
      granted_to_id:
        primaryDeliveryBlueprint.delivery_object_kind === "api_access" ||
        primaryDeliveryBlueprint.delivery_object_kind === "api_metered_access"
          ? appId
          : orderBlueprint.buyer_org_id,
      policy_snapshot: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        scenario_code: orderBlueprint.scenario_code,
        sku_type: sku.sku_type,
      },
      valid_from: timeline.buyerLockedAt ?? timeline.createdAt,
      status: "active",
    });

    paymentIntentRows.push({
      payment_intent_id: billingSample.payment_intent_id,
      order_id: orderBlueprint.order_blueprint_id,
      intent_type: "order_payment",
      provider_key: providerKey,
      payer_subject_type: "organization",
      payer_subject_id: orderBlueprint.buyer_org_id,
      payee_subject_type: "organization",
      payee_subject_id: orderBlueprint.seller_org_id,
      amount: billingSample.amount,
      price_currency_code: billingSample.currency_code,
      currency_code: billingSample.currency_code,
      payment_method: "mock_wallet",
      status: "succeeded",
      provider_intent_no: `demo-${billingSample.payment_intent_id}`,
      channel_reference_no: `demo-channel-${billingSample.payment_intent_id}`,
      request_id: orderBlueprint.request_id,
      idempotency_key: `payment-${orderBlueprint.idempotency_key}`,
      expire_at: shiftIso(timeline.createdAt, { hours: 1 }),
      capability_snapshot: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        supports_webhook: true,
      },
      metadata: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        scenario_code: orderBlueprint.scenario_code,
        billing_sample_id: billingSample.billing_sample_id,
      },
    });

    paymentTransactionRows.push({
      payment_transaction_id: deriveUuid(billingSample.payment_intent_id, "44"),
      payment_intent_id: billingSample.payment_intent_id,
      transaction_type: "charge",
      direction: "in",
      provider_transaction_no: `demo-txn-${billingSample.payment_intent_id}`,
      provider_status: "succeeded",
      amount: billingSample.amount,
      currency_code: billingSample.currency_code,
      channel_fee_amount: "0",
      settled_amount: billingSample.amount,
      occurred_at: timeline.buyerLockedAt ?? timeline.createdAt,
      raw_payload: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        provider_key: providerKey,
        event_type: billingSample.payment_callback_event,
      },
    });

    paymentWebhookRows.push({
      webhook_event_id: deriveUuid(billingSample.payment_intent_id, "45"),
      provider_key: providerKey,
      provider_event_id: `demo-webhook-${billingSample.payment_intent_id}`,
      event_type: billingSample.payment_callback_event,
      signature_verified: true,
      payment_intent_id: billingSample.payment_intent_id,
      payment_transaction_id: deriveUuid(billingSample.payment_intent_id, "44"),
      payload: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        provider_key: providerKey,
        payment_intent_id: billingSample.payment_intent_id,
      },
      processed_status: "processed",
      duplicate_flag: false,
      received_at: timeline.buyerLockedAt ?? timeline.createdAt,
      processed_at: shiftIso(timeline.buyerLockedAt ?? timeline.createdAt, { minutes: 1 }),
    });

    billingEventRows.push({
      billing_event_id: billingSample.billing_sample_id,
      order_id: orderBlueprint.order_blueprint_id,
      event_type: billingSample.billing_event_type,
      event_source: billingSample.billing_event_source,
      amount: billingSample.amount,
      currency_code: billingSample.currency_code,
      units: sku.sku_type === "API_PPU" ? "1000" : "1",
      occurred_at: timeline.acceptedAt ?? timeline.deliveredAt ?? timeline.buyerLockedAt ?? timeline.createdAt,
      metadata: {
        seed_source: "fixtures/demo",
        task_id: "TEST-002",
        provider_key: providerKey,
        scenario_code: orderBlueprint.scenario_code,
        sku_type: sku.sku_type,
        expected_notification_scene: billingSample.expected_notification_scene,
        expected_settlement_effect: billingSample.expected_settlement_effect,
      },
    });

    for (const deliveryBlueprint of deliveryBlueprints) {
      const committedAt = timeline.deliveredAt ?? timeline.buyerLockedAt ?? timeline.createdAt;
      const expiresAt = shiftIso(committedAt, { days: 30 });
      const payload = deliveryBlueprint.payload_snapshot;
      const objectId = ["encrypted_file_package", "report_artifact", "query_result_artifact"].includes(
        deliveryBlueprint.delivery_object_kind
      )
        ? deriveUuid(deliveryBlueprint.delivery_blueprint_id, "51")
        : null;

      if (objectId) {
        storageObjectRows.push({
          object_id: objectId,
          org_id: orderBlueprint.seller_org_id,
          object_type:
            deliveryBlueprint.delivery_object_kind === "encrypted_file_package"
              ? "file_package"
              : deliveryBlueprint.delivery_object_kind === "query_result_artifact"
                ? "query_result"
                : "report_artifact",
          object_uri: payload.object_uri ?? `minio://delivery-objects/demo/${deliveryBlueprint.delivery_blueprint_id}`,
          location_type: "platform_object_storage",
          managed_by_org_id: orderBlueprint.seller_org_id,
          content_type: payload.content_type ?? "application/json",
          size_bytes: String(payload.size_bytes ?? 1024),
          content_hash:
            payload.content_hash ?? stableHash("sha256", `${deliveryBlueprint.delivery_blueprint_id}:object`),
          encryption_algo:
            deliveryBlueprint.delivery_object_kind === "encrypted_file_package" ? "AES256" : null,
          plaintext_visible_to_platform: false,
          created_at: committedAt,
          storage_zone: "delivery",
        });
      }

      deliveryRecordRows.push({
        delivery_id: deliveryBlueprint.delivery_blueprint_id,
        order_id: orderBlueprint.order_blueprint_id,
        object_id: objectId,
        delivery_type: deliveryBlueprint.delivery_object_kind,
        delivery_route: deliveryBlueprint.delivery_object_kind,
        executor_type: "platform",
        status: "committed",
        delivery_commit_hash: stableHash("sha256", `${deliveryBlueprint.delivery_blueprint_id}:delivery`),
        trust_boundary_snapshot: {
          seed_source: "fixtures/demo",
          task_id: "TEST-002",
          scenario_code: deliveryBlueprint.scenario_code,
          source_fixture: deliveryBlueprint.source_fixture,
          expected_outbox_event: deliveryBlueprint.expected_outbox_event,
        },
        receipt_hash: stableHash("sha256", `${deliveryBlueprint.delivery_blueprint_id}:receipt`),
        committed_at: committedAt,
        expires_at: expiresAt,
        created_by: orderBlueprint.created_by_user_id,
        authority_model: "dual_layer",
        business_state_version: 1,
        proof_commit_state: deliveryBlueprint.expected_outbox_event ? "pending" : "n/a",
        proof_commit_policy: "async_evidence",
        external_fact_status: "n/a",
        reconcile_status: "pending_check",
        sensitive_delivery_mode: "standard",
        disclosure_review_status: "not_required",
      });

      switch (deliveryBlueprint.delivery_object_kind) {
        case "api_access":
        case "api_metered_access":
          apiCredentialRows.push({
            api_credential_id: deliveryBlueprint.delivery_blueprint_id,
            order_id: orderBlueprint.order_blueprint_id,
            app_id: appId,
            api_key_hash: stableHash("sha256", `${deliveryBlueprint.delivery_blueprint_id}:api-key`),
            upstream_mode: payload.upstream_mode,
            quota_json: payload.quota_json,
            status: "active",
            valid_from: committedAt,
            valid_to: expiresAt,
            sensitive_scope_snapshot: {
              seed_source: "fixtures/demo",
              task_id: "TEST-002",
              scenario_code: orderBlueprint.scenario_code,
              sku_type: sku.sku_type,
            },
          });

          apiUsageRows.push({
            api_usage_log_id: deriveUuid(deliveryBlueprint.delivery_blueprint_id, "65"),
            api_credential_id: deliveryBlueprint.delivery_blueprint_id,
            order_id: orderBlueprint.order_blueprint_id,
            app_id: appId,
            request_id: `${orderBlueprint.request_id}-usage`,
            response_code: 200,
            usage_units:
              deliveryBlueprint.delivery_object_kind === "api_metered_access"
                ? String(payload.quota_json?.prepaid_calls ?? 1000)
                : "1",
            occurred_at: shiftIso(committedAt, { minutes: 30 }),
          });
          break;
        case "encrypted_file_package":
          deliveryTicketRows.push({
            ticket_id: deriveUuid(deliveryBlueprint.delivery_blueprint_id, "71"),
            order_id: orderBlueprint.order_blueprint_id,
            buyer_org_id: orderBlueprint.buyer_org_id,
            token_hash: stableHash("sha256", `${deliveryBlueprint.delivery_blueprint_id}:ticket`),
            expire_at: expiresAt,
            download_limit: String(payload.download_limit ?? 3),
            status: "active",
            created_at: committedAt,
          });
          break;
        case "revision_subscription":
          revisionSubscriptionRows.push({
            revision_subscription_id: deliveryBlueprint.delivery_blueprint_id,
            order_id: orderBlueprint.order_blueprint_id,
            asset_id: baseProduct.asset_id,
            sku_id: orderBlueprint.sku_id,
            cadence: payload.cadence,
            delivery_channel: payload.delivery_channel,
            start_version_no: String(payload.revision_window?.from_version ?? 1),
            last_delivered_version_no: String(payload.revision_window?.from_version ?? 1),
            next_delivery_at: shiftIso(committedAt, { days: 30 }),
            subscription_status: "active",
            metadata: {
              seed_source: "fixtures/demo",
              task_id: "TEST-002",
              revision_window: payload.revision_window,
            },
          });
          break;
        case "sandbox_workspace":
          sandboxWorkspaceRows.push({
            sandbox_workspace_id: deliveryBlueprint.delivery_blueprint_id,
            order_id: orderBlueprint.order_blueprint_id,
            workspace_name: payload.workspace_name,
            status: "running",
            data_residency_mode: payload.data_residency_mode,
            export_policy: payload.export_policy_json,
            output_boundary_json: payload.export_policy_json,
            created_at: committedAt,
            clean_room_mode: payload.clean_room_mode,
            sensitive_boundary_level: "standard",
          });

          sandboxSessionRows.push({
            sandbox_session_id: deriveUuid(deliveryBlueprint.delivery_blueprint_id, "64"),
            sandbox_workspace_id: deliveryBlueprint.delivery_blueprint_id,
            user_id: orderBlueprint.created_by_user_id,
            started_at: shiftIso(committedAt, { minutes: 15 }),
            ended_at: null,
            session_status: "active",
            query_count: 3,
            export_attempt_count: 0,
          });
          break;
        case "share_grant":
          shareGrantRows.push({
            data_share_grant_id: deliveryBlueprint.delivery_blueprint_id,
            order_id: orderBlueprint.order_blueprint_id,
            asset_object_id: shareObjectId,
            recipient_ref: payload.recipient_ref,
            share_protocol: payload.share_protocol,
            access_locator: payload.access_locator,
            grant_status: "active",
            read_only: true,
            receipt_hash: stableHash("sha256", `${deliveryBlueprint.delivery_blueprint_id}:share`),
            granted_at: committedAt,
            expires_at: expiresAt,
            metadata: {
              seed_source: "fixtures/demo",
              task_id: "TEST-002",
              scope_json: payload.scope_json,
            },
          });
          break;
        case "report_artifact":
          reportArtifactRows.push({
            report_artifact_id: deliveryBlueprint.delivery_blueprint_id,
            order_id: orderBlueprint.order_blueprint_id,
            object_id: objectId,
            report_type: payload.report_type,
            version_no: 1,
            status: "ready",
            created_at: committedAt,
          });
          deliveryTicketRows.push({
            ticket_id: deriveUuid(deliveryBlueprint.delivery_blueprint_id, "71"),
            order_id: orderBlueprint.order_blueprint_id,
            buyer_org_id: orderBlueprint.buyer_org_id,
            token_hash: stableHash("sha256", `${deliveryBlueprint.delivery_blueprint_id}:ticket`),
            expire_at: expiresAt,
            download_limit: "3",
            status: "active",
            created_at: committedAt,
          });
          break;
        case "template_query_grant":
          templateQueryGrantRows.push({
            template_query_grant_id: deliveryBlueprint.delivery_blueprint_id,
            order_id: orderBlueprint.order_blueprint_id,
            asset_object_id: queryObjectId,
            sandbox_workspace_id: null,
            environment_id: null,
            template_type: "sql_template",
            template_digest: stableHash("sha256", "tpl_demo_s5_location_score_v1"),
            output_boundary_json: payload.output_boundary_json,
            run_quota_json: payload.run_quota_json,
            grant_status: "active",
            created_at: committedAt,
            query_surface_id: querySurfaceId,
            allowed_template_ids: payload.allowed_template_ids,
            execution_rule_snapshot: {
              seed_source: "fixtures/demo",
              task_id: "TEST-002",
              allow_raw_export: false,
              review_status: "approved",
            },
          });
          break;
        case "query_result_artifact":
          queryRunRows.push({
            query_run_id: deliveryBlueprint.delivery_blueprint_id,
            order_id: orderBlueprint.order_blueprint_id,
            template_query_grant_id: "41000000-0000-0000-0000-000000000009",
            sandbox_session_id: null,
            query_template_id: queryTemplateId,
            query_surface_id: querySurfaceId,
            requester_user_id: orderBlueprint.created_by_user_id,
            execution_mode: "template_query",
            request_payload_json: payload.request_payload_json,
            result_summary_json: {
              seed_source: "fixtures/demo",
              task_id: "TEST-002",
              preview: [
                { candidate_area: "Shanghai Jing'an", score: 92.4 },
                { candidate_area: "Shanghai Xuhui", score: 88.1 },
              ],
            },
            result_object_id: objectId,
            result_row_count: 2,
            billed_units: "1",
            export_attempt_count: 0,
            status: "succeeded",
            started_at: shiftIso(committedAt, { minutes: -5 }),
            completed_at: committedAt,
            masked_level: "masked",
            export_scope: "json_summary",
            sensitive_policy_snapshot: {
              seed_source: "fixtures/demo",
              task_id: "TEST-002",
              allow_raw_export: false,
            },
          });
          deliveryTicketRows.push({
            ticket_id: deriveUuid(deliveryBlueprint.delivery_blueprint_id, "71"),
            order_id: orderBlueprint.order_blueprint_id,
            buyer_org_id: orderBlueprint.buyer_org_id,
            token_hash: stableHash("sha256", `${deliveryBlueprint.delivery_blueprint_id}:ticket`),
            expire_at: expiresAt,
            download_limit: "1",
            status: "active",
            created_at: committedAt,
          });
          break;
        default:
          break;
      }
    }
  }

  const checksumPayload = {
    subjects: subjectData,
    catalog: catalogData,
    orders: fixtures.orders,
    billing: fixtures.billing,
    delivery: fixtures.delivery,
    provider_key: providerKey,
  };
  const checksum = createHash("sha256").update(JSON.stringify(checksumPayload)).digest("hex");

  return {
    checksum,
    rows: {
      orderRows,
      orderLineRows,
      authorizationGrantRows,
      paymentIntentRows,
      paymentTransactionRows,
      paymentWebhookRows,
      billingEventRows,
      assetObjectRows,
      querySurfaceRows,
      queryTemplateRows,
      storageObjectRows,
      deliveryRecordRows,
      deliveryTicketRows,
      apiCredentialRows,
      apiUsageRows,
      sandboxWorkspaceRows,
      sandboxSessionRows,
      shareGrantRows,
      revisionSubscriptionRows,
      templateQueryGrantRows,
      queryRunRows,
      reportArtifactRows,
    },
  };
}

function insertSqlFromJson(jsonRecords, columnSpec) {
  return `SELECT * FROM jsonb_to_recordset(${jsonLiteral(jsonRecords)}) AS x(${columnSpec})`;
}

function buildImportSql(seedRows) {
  const {
    orderRows,
    orderLineRows,
    authorizationGrantRows,
    paymentIntentRows,
    paymentTransactionRows,
    paymentWebhookRows,
    billingEventRows,
    assetObjectRows,
    querySurfaceRows,
    queryTemplateRows,
    storageObjectRows,
    deliveryRecordRows,
    deliveryTicketRows,
    apiCredentialRows,
    apiUsageRows,
    sandboxWorkspaceRows,
    sandboxSessionRows,
    shareGrantRows,
    revisionSubscriptionRows,
    templateQueryGrantRows,
    queryRunRows,
    reportArtifactRows,
  } = seedRows.rows;

  return `
BEGIN;

CREATE TABLE IF NOT EXISTS public.seed_history (
  id BIGSERIAL PRIMARY KEY,
  version TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  checksum_sha256 TEXT NOT NULL,
  executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

WITH source AS (
  ${insertSqlFromJson(
    orderRows,
    [
      "order_id text",
      "product_id text",
      "asset_version_id text",
      "buyer_org_id text",
      "seller_org_id text",
      "sku_id text",
      "status text",
      "payment_status text",
      "payment_mode text",
      "amount text",
      "currency_code text",
      "fee_preview_snapshot jsonb",
      "payment_channel_snapshot jsonb",
      "price_snapshot_json jsonb",
      "trust_boundary_snapshot jsonb",
      "storage_mode_snapshot text",
      "delivery_route_snapshot text",
      "platform_plaintext_access_snapshot boolean",
      "idempotency_key text",
      "created_at text",
      "buyer_locked_at text",
      "delivered_at text",
      "accepted_at text",
      "delivery_status text",
      "acceptance_status text",
      "settlement_status text",
      "dispute_status text",
    ].join(", ")
  )}
)
INSERT INTO trade.order_main (
  order_id,
  product_id,
  asset_version_id,
  buyer_org_id,
  seller_org_id,
  sku_id,
  status,
  payment_status,
  payment_mode,
  amount,
  currency_code,
  fee_preview_snapshot,
  payment_channel_snapshot,
  price_snapshot_json,
  trust_boundary_snapshot,
  storage_mode_snapshot,
  delivery_route_snapshot,
  platform_plaintext_access_snapshot,
  idempotency_key,
  created_at,
  buyer_locked_at,
  delivered_at,
  accepted_at,
  delivery_status,
  acceptance_status,
  settlement_status,
  dispute_status
)
SELECT
  order_id::uuid,
  product_id::uuid,
  asset_version_id::uuid,
  buyer_org_id::uuid,
  seller_org_id::uuid,
  sku_id::uuid,
  status,
  payment_status,
  payment_mode,
  amount::numeric,
  currency_code,
  fee_preview_snapshot,
  payment_channel_snapshot,
  price_snapshot_json,
  trust_boundary_snapshot,
  storage_mode_snapshot,
  delivery_route_snapshot,
  platform_plaintext_access_snapshot,
  idempotency_key,
  created_at::timestamptz,
  NULLIF(buyer_locked_at, '')::timestamptz,
  NULLIF(delivered_at, '')::timestamptz,
  NULLIF(accepted_at, '')::timestamptz,
  delivery_status,
  acceptance_status,
  settlement_status,
  dispute_status
FROM source
ON CONFLICT (order_id) DO UPDATE SET
  product_id = EXCLUDED.product_id,
  asset_version_id = EXCLUDED.asset_version_id,
  buyer_org_id = EXCLUDED.buyer_org_id,
  seller_org_id = EXCLUDED.seller_org_id,
  sku_id = EXCLUDED.sku_id,
  status = EXCLUDED.status,
  payment_status = EXCLUDED.payment_status,
  payment_mode = EXCLUDED.payment_mode,
  amount = EXCLUDED.amount,
  currency_code = EXCLUDED.currency_code,
  fee_preview_snapshot = EXCLUDED.fee_preview_snapshot,
  payment_channel_snapshot = EXCLUDED.payment_channel_snapshot,
  price_snapshot_json = EXCLUDED.price_snapshot_json,
  trust_boundary_snapshot = EXCLUDED.trust_boundary_snapshot,
  storage_mode_snapshot = EXCLUDED.storage_mode_snapshot,
  delivery_route_snapshot = EXCLUDED.delivery_route_snapshot,
  platform_plaintext_access_snapshot = EXCLUDED.platform_plaintext_access_snapshot,
  idempotency_key = EXCLUDED.idempotency_key,
  created_at = EXCLUDED.created_at,
  buyer_locked_at = EXCLUDED.buyer_locked_at,
  delivered_at = EXCLUDED.delivered_at,
  accepted_at = EXCLUDED.accepted_at,
  delivery_status = EXCLUDED.delivery_status,
  acceptance_status = EXCLUDED.acceptance_status,
  settlement_status = EXCLUDED.settlement_status,
  dispute_status = EXCLUDED.dispute_status,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    orderLineRows,
    [
      "order_line_id text",
      "order_id text",
      "sku_id text",
      "quantity text",
      "unit_price text",
      "amount text",
    ].join(", ")
  )}
)
INSERT INTO trade.order_line (order_line_id, order_id, sku_id, quantity, unit_price, amount)
SELECT
  order_line_id::uuid,
  order_id::uuid,
  sku_id::uuid,
  quantity::numeric,
  unit_price::numeric,
  amount::numeric
FROM source
ON CONFLICT (order_line_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  sku_id = EXCLUDED.sku_id,
  quantity = EXCLUDED.quantity,
  unit_price = EXCLUDED.unit_price,
  amount = EXCLUDED.amount;

WITH source AS (
  ${insertSqlFromJson(
    authorizationGrantRows,
    [
      "authorization_grant_id text",
      "order_id text",
      "grant_type text",
      "granted_to_type text",
      "granted_to_id text",
      "policy_snapshot jsonb",
      "valid_from text",
      "status text",
    ].join(", ")
  )}
)
INSERT INTO trade.authorization_grant (
  authorization_grant_id,
  order_id,
  grant_type,
  granted_to_type,
  granted_to_id,
  policy_snapshot,
  valid_from,
  status
)
SELECT
  authorization_grant_id::uuid,
  order_id::uuid,
  grant_type,
  granted_to_type,
  granted_to_id::uuid,
  policy_snapshot,
  valid_from::timestamptz,
  status
FROM source
ON CONFLICT (authorization_grant_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  grant_type = EXCLUDED.grant_type,
  granted_to_type = EXCLUDED.granted_to_type,
  granted_to_id = EXCLUDED.granted_to_id,
  policy_snapshot = EXCLUDED.policy_snapshot,
  valid_from = EXCLUDED.valid_from,
  status = EXCLUDED.status,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    paymentIntentRows,
    [
      "payment_intent_id text",
      "order_id text",
      "intent_type text",
      "provider_key text",
      "payer_subject_type text",
      "payer_subject_id text",
      "payee_subject_type text",
      "payee_subject_id text",
      "amount text",
      "price_currency_code text",
      "currency_code text",
      "payment_method text",
      "status text",
      "provider_intent_no text",
      "channel_reference_no text",
      "request_id text",
      "idempotency_key text",
      "expire_at text",
      "capability_snapshot jsonb",
      "metadata jsonb",
    ].join(", ")
  )}
)
INSERT INTO payment.payment_intent (
  payment_intent_id,
  order_id,
  intent_type,
  provider_key,
  payer_subject_type,
  payer_subject_id,
  payee_subject_type,
  payee_subject_id,
  amount,
  price_currency_code,
  currency_code,
  payment_method,
  status,
  provider_intent_no,
  channel_reference_no,
  request_id,
  idempotency_key,
  expire_at,
  capability_snapshot,
  metadata
)
SELECT
  payment_intent_id::uuid,
  order_id::uuid,
  intent_type,
  provider_key,
  payer_subject_type,
  payer_subject_id::uuid,
  payee_subject_type,
  payee_subject_id::uuid,
  amount::numeric,
  price_currency_code,
  currency_code,
  payment_method,
  status,
  provider_intent_no,
  channel_reference_no,
  request_id,
  idempotency_key,
  expire_at::timestamptz,
  capability_snapshot,
  metadata
FROM source
ON CONFLICT (payment_intent_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  intent_type = EXCLUDED.intent_type,
  provider_key = EXCLUDED.provider_key,
  payer_subject_type = EXCLUDED.payer_subject_type,
  payer_subject_id = EXCLUDED.payer_subject_id,
  payee_subject_type = EXCLUDED.payee_subject_type,
  payee_subject_id = EXCLUDED.payee_subject_id,
  amount = EXCLUDED.amount,
  price_currency_code = EXCLUDED.price_currency_code,
  currency_code = EXCLUDED.currency_code,
  payment_method = EXCLUDED.payment_method,
  status = EXCLUDED.status,
  provider_intent_no = EXCLUDED.provider_intent_no,
  channel_reference_no = EXCLUDED.channel_reference_no,
  request_id = EXCLUDED.request_id,
  idempotency_key = EXCLUDED.idempotency_key,
  expire_at = EXCLUDED.expire_at,
  capability_snapshot = EXCLUDED.capability_snapshot,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    paymentTransactionRows,
    [
      "payment_transaction_id text",
      "payment_intent_id text",
      "transaction_type text",
      "direction text",
      "provider_transaction_no text",
      "provider_status text",
      "amount text",
      "currency_code text",
      "channel_fee_amount text",
      "settled_amount text",
      "occurred_at text",
      "raw_payload jsonb",
    ].join(", ")
  )}
)
INSERT INTO payment.payment_transaction (
  payment_transaction_id,
  payment_intent_id,
  transaction_type,
  direction,
  provider_transaction_no,
  provider_status,
  amount,
  currency_code,
  channel_fee_amount,
  settled_amount,
  occurred_at,
  raw_payload
)
SELECT
  payment_transaction_id::uuid,
  payment_intent_id::uuid,
  transaction_type,
  direction,
  provider_transaction_no,
  provider_status,
  amount::numeric,
  currency_code,
  channel_fee_amount::numeric,
  settled_amount::numeric,
  occurred_at::timestamptz,
  raw_payload
FROM source
ON CONFLICT (payment_transaction_id) DO UPDATE SET
  payment_intent_id = EXCLUDED.payment_intent_id,
  transaction_type = EXCLUDED.transaction_type,
  direction = EXCLUDED.direction,
  provider_transaction_no = EXCLUDED.provider_transaction_no,
  provider_status = EXCLUDED.provider_status,
  amount = EXCLUDED.amount,
  currency_code = EXCLUDED.currency_code,
  channel_fee_amount = EXCLUDED.channel_fee_amount,
  settled_amount = EXCLUDED.settled_amount,
  occurred_at = EXCLUDED.occurred_at,
  raw_payload = EXCLUDED.raw_payload;

WITH source AS (
  ${insertSqlFromJson(
    paymentWebhookRows,
    [
      "webhook_event_id text",
      "provider_key text",
      "provider_event_id text",
      "event_type text",
      "signature_verified boolean",
      "payment_intent_id text",
      "payment_transaction_id text",
      "payload jsonb",
      "processed_status text",
      "duplicate_flag boolean",
      "received_at text",
      "processed_at text",
    ].join(", ")
  )}
)
INSERT INTO payment.payment_webhook_event (
  webhook_event_id,
  provider_key,
  provider_event_id,
  event_type,
  signature_verified,
  payment_intent_id,
  payment_transaction_id,
  payload,
  processed_status,
  duplicate_flag,
  received_at,
  processed_at
)
SELECT
  webhook_event_id::uuid,
  provider_key,
  provider_event_id,
  event_type,
  signature_verified,
  payment_intent_id::uuid,
  payment_transaction_id::uuid,
  payload,
  processed_status,
  duplicate_flag,
  received_at::timestamptz,
  processed_at::timestamptz
FROM source
ON CONFLICT (webhook_event_id) DO UPDATE SET
  provider_key = EXCLUDED.provider_key,
  provider_event_id = EXCLUDED.provider_event_id,
  event_type = EXCLUDED.event_type,
  signature_verified = EXCLUDED.signature_verified,
  payment_intent_id = EXCLUDED.payment_intent_id,
  payment_transaction_id = EXCLUDED.payment_transaction_id,
  payload = EXCLUDED.payload,
  processed_status = EXCLUDED.processed_status,
  duplicate_flag = EXCLUDED.duplicate_flag,
  received_at = EXCLUDED.received_at,
  processed_at = EXCLUDED.processed_at;

WITH source AS (
  ${insertSqlFromJson(
    billingEventRows,
    [
      "billing_event_id text",
      "order_id text",
      "event_type text",
      "event_source text",
      "amount text",
      "currency_code text",
      "units text",
      "occurred_at text",
      "metadata jsonb",
    ].join(", ")
  )}
)
INSERT INTO billing.billing_event (
  billing_event_id,
  order_id,
  event_type,
  event_source,
  amount,
  currency_code,
  units,
  occurred_at,
  metadata
)
SELECT
  billing_event_id::uuid,
  order_id::uuid,
  event_type,
  event_source,
  amount::numeric,
  currency_code,
  units::numeric,
  occurred_at::timestamptz,
  metadata
FROM source
ON CONFLICT (billing_event_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  event_type = EXCLUDED.event_type,
  event_source = EXCLUDED.event_source,
  amount = EXCLUDED.amount,
  currency_code = EXCLUDED.currency_code,
  units = EXCLUDED.units,
  occurred_at = EXCLUDED.occurred_at,
  metadata = EXCLUDED.metadata;

WITH source AS (
  ${insertSqlFromJson(
    assetObjectRows,
    [
      "asset_object_id text",
      "asset_version_id text",
      "object_kind text",
      "object_name text",
      "object_locator text",
      "share_protocol text",
      "schema_json jsonb",
      "output_schema_json jsonb",
      "freshness_json jsonb",
      "access_constraints jsonb",
      "metadata jsonb",
    ].join(", ")
  )}
)
INSERT INTO catalog.asset_object_binding (
  asset_object_id,
  asset_version_id,
  object_kind,
  object_name,
  object_locator,
  share_protocol,
  schema_json,
  output_schema_json,
  freshness_json,
  access_constraints,
  metadata
)
SELECT
  asset_object_id::uuid,
  asset_version_id::uuid,
  object_kind,
  object_name,
  object_locator,
  NULLIF(share_protocol, ''),
  schema_json,
  output_schema_json,
  freshness_json,
  access_constraints,
  metadata
FROM source
ON CONFLICT (asset_object_id) DO UPDATE SET
  asset_version_id = EXCLUDED.asset_version_id,
  object_kind = EXCLUDED.object_kind,
  object_name = EXCLUDED.object_name,
  object_locator = EXCLUDED.object_locator,
  share_protocol = EXCLUDED.share_protocol,
  schema_json = EXCLUDED.schema_json,
  output_schema_json = EXCLUDED.output_schema_json,
  freshness_json = EXCLUDED.freshness_json,
  access_constraints = EXCLUDED.access_constraints,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    querySurfaceRows,
    [
      "query_surface_id text",
      "asset_version_id text",
      "asset_object_id text",
      "surface_type text",
      "binding_mode text",
      "execution_scope text",
      "input_contract_json jsonb",
      "output_boundary_json jsonb",
      "query_policy_json jsonb",
      "quota_policy_json jsonb",
      "status text",
      "metadata jsonb",
    ].join(", ")
  )}
)
INSERT INTO catalog.query_surface_definition (
  query_surface_id,
  asset_version_id,
  asset_object_id,
  surface_type,
  binding_mode,
  execution_scope,
  input_contract_json,
  output_boundary_json,
  query_policy_json,
  quota_policy_json,
  status,
  metadata
)
SELECT
  query_surface_id::uuid,
  asset_version_id::uuid,
  asset_object_id::uuid,
  surface_type,
  binding_mode,
  execution_scope,
  input_contract_json,
  output_boundary_json,
  query_policy_json,
  quota_policy_json,
  status,
  metadata
FROM source
ON CONFLICT (query_surface_id) DO UPDATE SET
  asset_version_id = EXCLUDED.asset_version_id,
  asset_object_id = EXCLUDED.asset_object_id,
  surface_type = EXCLUDED.surface_type,
  binding_mode = EXCLUDED.binding_mode,
  execution_scope = EXCLUDED.execution_scope,
  input_contract_json = EXCLUDED.input_contract_json,
  output_boundary_json = EXCLUDED.output_boundary_json,
  query_policy_json = EXCLUDED.query_policy_json,
  quota_policy_json = EXCLUDED.quota_policy_json,
  status = EXCLUDED.status,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    queryTemplateRows,
    [
      "query_template_id text",
      "query_surface_id text",
      "template_name text",
      "template_type text",
      "template_body_ref text",
      "parameter_schema_json jsonb",
      "analysis_rule_json jsonb",
      "result_schema_json jsonb",
      "export_policy_json jsonb",
      "risk_guard_json jsonb",
      "status text",
      "version_no integer",
    ].join(", ")
  )}
)
INSERT INTO delivery.query_template_definition (
  query_template_id,
  query_surface_id,
  template_name,
  template_type,
  template_body_ref,
  parameter_schema_json,
  analysis_rule_json,
  result_schema_json,
  export_policy_json,
  risk_guard_json,
  status,
  version_no
)
SELECT
  query_template_id::uuid,
  query_surface_id::uuid,
  template_name,
  template_type,
  template_body_ref,
  parameter_schema_json,
  analysis_rule_json,
  result_schema_json,
  export_policy_json,
  risk_guard_json,
  status,
  version_no
FROM source
ON CONFLICT (query_template_id) DO UPDATE SET
  query_surface_id = EXCLUDED.query_surface_id,
  template_name = EXCLUDED.template_name,
  template_type = EXCLUDED.template_type,
  template_body_ref = EXCLUDED.template_body_ref,
  parameter_schema_json = EXCLUDED.parameter_schema_json,
  analysis_rule_json = EXCLUDED.analysis_rule_json,
  result_schema_json = EXCLUDED.result_schema_json,
  export_policy_json = EXCLUDED.export_policy_json,
  risk_guard_json = EXCLUDED.risk_guard_json,
  status = EXCLUDED.status,
  version_no = EXCLUDED.version_no,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    storageObjectRows,
    [
      "object_id text",
      "org_id text",
      "object_type text",
      "object_uri text",
      "location_type text",
      "managed_by_org_id text",
      "content_type text",
      "size_bytes text",
      "content_hash text",
      "encryption_algo text",
      "plaintext_visible_to_platform boolean",
      "created_at text",
      "storage_zone text",
    ].join(", ")
  )}
)
INSERT INTO delivery.storage_object (
  object_id,
  org_id,
  object_type,
  object_uri,
  location_type,
  managed_by_org_id,
  content_type,
  size_bytes,
  content_hash,
  encryption_algo,
  plaintext_visible_to_platform,
  created_at,
  storage_zone
)
SELECT
  object_id::uuid,
  org_id::uuid,
  object_type,
  object_uri,
  location_type,
  managed_by_org_id::uuid,
  content_type,
  size_bytes::bigint,
  content_hash,
  NULLIF(encryption_algo, ''),
  plaintext_visible_to_platform,
  created_at::timestamptz,
  storage_zone
FROM source
ON CONFLICT (object_id) DO UPDATE SET
  org_id = EXCLUDED.org_id,
  object_type = EXCLUDED.object_type,
  object_uri = EXCLUDED.object_uri,
  location_type = EXCLUDED.location_type,
  managed_by_org_id = EXCLUDED.managed_by_org_id,
  content_type = EXCLUDED.content_type,
  size_bytes = EXCLUDED.size_bytes,
  content_hash = EXCLUDED.content_hash,
  encryption_algo = EXCLUDED.encryption_algo,
  plaintext_visible_to_platform = EXCLUDED.plaintext_visible_to_platform,
  created_at = EXCLUDED.created_at,
  storage_zone = EXCLUDED.storage_zone;

WITH source AS (
  ${insertSqlFromJson(
    apiCredentialRows,
    [
      "api_credential_id text",
      "order_id text",
      "app_id text",
      "api_key_hash text",
      "upstream_mode text",
      "quota_json jsonb",
      "status text",
      "valid_from text",
      "valid_to text",
      "sensitive_scope_snapshot jsonb",
    ].join(", ")
  )}
)
INSERT INTO delivery.api_credential (
  api_credential_id,
  order_id,
  app_id,
  api_key_hash,
  upstream_mode,
  quota_json,
  status,
  valid_from,
  valid_to,
  sensitive_scope_snapshot
)
SELECT
  api_credential_id::uuid,
  order_id::uuid,
  app_id::uuid,
  api_key_hash,
  upstream_mode,
  quota_json,
  status,
  valid_from::timestamptz,
  valid_to::timestamptz,
  sensitive_scope_snapshot
FROM source
ON CONFLICT (api_credential_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  app_id = EXCLUDED.app_id,
  api_key_hash = EXCLUDED.api_key_hash,
  upstream_mode = EXCLUDED.upstream_mode,
  quota_json = EXCLUDED.quota_json,
  status = EXCLUDED.status,
  valid_from = EXCLUDED.valid_from,
  valid_to = EXCLUDED.valid_to,
  sensitive_scope_snapshot = EXCLUDED.sensitive_scope_snapshot,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    apiUsageRows,
    [
      "api_usage_log_id text",
      "api_credential_id text",
      "order_id text",
      "app_id text",
      "request_id text",
      "response_code integer",
      "usage_units text",
      "occurred_at text",
    ].join(", ")
  )}
)
INSERT INTO delivery.api_usage_log (
  api_usage_log_id,
  api_credential_id,
  order_id,
  app_id,
  request_id,
  response_code,
  usage_units,
  occurred_at
)
SELECT
  api_usage_log_id::uuid,
  api_credential_id::uuid,
  order_id::uuid,
  app_id::uuid,
  request_id,
  response_code,
  usage_units::numeric,
  occurred_at::timestamptz
FROM source
ON CONFLICT (api_usage_log_id) DO UPDATE SET
  api_credential_id = EXCLUDED.api_credential_id,
  order_id = EXCLUDED.order_id,
  app_id = EXCLUDED.app_id,
  request_id = EXCLUDED.request_id,
  response_code = EXCLUDED.response_code,
  usage_units = EXCLUDED.usage_units,
  occurred_at = EXCLUDED.occurred_at;

WITH source AS (
  ${insertSqlFromJson(
    sandboxWorkspaceRows,
    [
      "sandbox_workspace_id text",
      "order_id text",
      "workspace_name text",
      "status text",
      "data_residency_mode text",
      "export_policy jsonb",
      "output_boundary_json jsonb",
      "created_at text",
      "clean_room_mode text",
      "sensitive_boundary_level text",
    ].join(", ")
  )}
)
INSERT INTO delivery.sandbox_workspace (
  sandbox_workspace_id,
  order_id,
  workspace_name,
  status,
  data_residency_mode,
  export_policy,
  output_boundary_json,
  created_at,
  clean_room_mode,
  sensitive_boundary_level
)
SELECT
  sandbox_workspace_id::uuid,
  order_id::uuid,
  workspace_name,
  status,
  data_residency_mode,
  export_policy,
  output_boundary_json,
  created_at::timestamptz,
  clean_room_mode,
  sensitive_boundary_level
FROM source
ON CONFLICT (sandbox_workspace_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  workspace_name = EXCLUDED.workspace_name,
  status = EXCLUDED.status,
  data_residency_mode = EXCLUDED.data_residency_mode,
  export_policy = EXCLUDED.export_policy,
  output_boundary_json = EXCLUDED.output_boundary_json,
  created_at = EXCLUDED.created_at,
  clean_room_mode = EXCLUDED.clean_room_mode,
  sensitive_boundary_level = EXCLUDED.sensitive_boundary_level,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    sandboxSessionRows,
    [
      "sandbox_session_id text",
      "sandbox_workspace_id text",
      "user_id text",
      "started_at text",
      "ended_at text",
      "session_status text",
      "query_count integer",
      "export_attempt_count integer",
    ].join(", ")
  )}
)
INSERT INTO delivery.sandbox_session (
  sandbox_session_id,
  sandbox_workspace_id,
  user_id,
  started_at,
  ended_at,
  session_status,
  query_count,
  export_attempt_count
)
SELECT
  sandbox_session_id::uuid,
  sandbox_workspace_id::uuid,
  user_id::uuid,
  started_at::timestamptz,
  NULLIF(ended_at, '')::timestamptz,
  session_status,
  query_count,
  export_attempt_count
FROM source
ON CONFLICT (sandbox_session_id) DO UPDATE SET
  sandbox_workspace_id = EXCLUDED.sandbox_workspace_id,
  user_id = EXCLUDED.user_id,
  started_at = EXCLUDED.started_at,
  ended_at = EXCLUDED.ended_at,
  session_status = EXCLUDED.session_status,
  query_count = EXCLUDED.query_count,
  export_attempt_count = EXCLUDED.export_attempt_count;

WITH source AS (
  ${insertSqlFromJson(
    shareGrantRows,
    [
      "data_share_grant_id text",
      "order_id text",
      "asset_object_id text",
      "recipient_ref text",
      "share_protocol text",
      "access_locator text",
      "grant_status text",
      "read_only boolean",
      "receipt_hash text",
      "granted_at text",
      "expires_at text",
      "metadata jsonb",
    ].join(", ")
  )}
)
INSERT INTO delivery.data_share_grant (
  data_share_grant_id,
  order_id,
  asset_object_id,
  recipient_ref,
  share_protocol,
  access_locator,
  grant_status,
  read_only,
  receipt_hash,
  granted_at,
  expires_at,
  metadata
)
SELECT
  data_share_grant_id::uuid,
  order_id::uuid,
  asset_object_id::uuid,
  recipient_ref,
  share_protocol,
  access_locator,
  grant_status,
  read_only,
  receipt_hash,
  granted_at::timestamptz,
  expires_at::timestamptz,
  metadata
FROM source
ON CONFLICT (data_share_grant_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  asset_object_id = EXCLUDED.asset_object_id,
  recipient_ref = EXCLUDED.recipient_ref,
  share_protocol = EXCLUDED.share_protocol,
  access_locator = EXCLUDED.access_locator,
  grant_status = EXCLUDED.grant_status,
  read_only = EXCLUDED.read_only,
  receipt_hash = EXCLUDED.receipt_hash,
  granted_at = EXCLUDED.granted_at,
  expires_at = EXCLUDED.expires_at,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    revisionSubscriptionRows,
    [
      "revision_subscription_id text",
      "order_id text",
      "asset_id text",
      "sku_id text",
      "cadence text",
      "delivery_channel text",
      "start_version_no text",
      "last_delivered_version_no text",
      "next_delivery_at text",
      "subscription_status text",
      "metadata jsonb",
    ].join(", ")
  )}
)
INSERT INTO delivery.revision_subscription (
  revision_subscription_id,
  order_id,
  asset_id,
  sku_id,
  cadence,
  delivery_channel,
  start_version_no,
  last_delivered_version_no,
  next_delivery_at,
  subscription_status,
  metadata
)
SELECT
  revision_subscription_id::uuid,
  order_id::uuid,
  asset_id::uuid,
  sku_id::uuid,
  cadence,
  delivery_channel,
  start_version_no::integer,
  last_delivered_version_no::integer,
  next_delivery_at::timestamptz,
  subscription_status,
  metadata
FROM source
ON CONFLICT (revision_subscription_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  asset_id = EXCLUDED.asset_id,
  sku_id = EXCLUDED.sku_id,
  cadence = EXCLUDED.cadence,
  delivery_channel = EXCLUDED.delivery_channel,
  start_version_no = EXCLUDED.start_version_no,
  last_delivered_version_no = EXCLUDED.last_delivered_version_no,
  next_delivery_at = EXCLUDED.next_delivery_at,
  subscription_status = EXCLUDED.subscription_status,
  metadata = EXCLUDED.metadata,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    templateQueryGrantRows,
    [
      "template_query_grant_id text",
      "order_id text",
      "asset_object_id text",
      "sandbox_workspace_id text",
      "environment_id text",
      "template_type text",
      "template_digest text",
      "output_boundary_json jsonb",
      "run_quota_json jsonb",
      "grant_status text",
      "created_at text",
      "query_surface_id text",
      "allowed_template_ids jsonb",
      "execution_rule_snapshot jsonb",
    ].join(", ")
  )}
)
INSERT INTO delivery.template_query_grant (
  template_query_grant_id,
  order_id,
  asset_object_id,
  sandbox_workspace_id,
  environment_id,
  template_type,
  template_digest,
  output_boundary_json,
  run_quota_json,
  grant_status,
  created_at,
  query_surface_id,
  allowed_template_ids,
  execution_rule_snapshot
)
SELECT
  template_query_grant_id::uuid,
  order_id::uuid,
  asset_object_id::uuid,
  NULLIF(sandbox_workspace_id, '')::uuid,
  NULLIF(environment_id, '')::uuid,
  template_type,
  template_digest,
  output_boundary_json,
  run_quota_json,
  grant_status,
  created_at::timestamptz,
  query_surface_id::uuid,
  allowed_template_ids,
  execution_rule_snapshot
FROM source
ON CONFLICT (template_query_grant_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  asset_object_id = EXCLUDED.asset_object_id,
  sandbox_workspace_id = EXCLUDED.sandbox_workspace_id,
  environment_id = EXCLUDED.environment_id,
  template_type = EXCLUDED.template_type,
  template_digest = EXCLUDED.template_digest,
  output_boundary_json = EXCLUDED.output_boundary_json,
  run_quota_json = EXCLUDED.run_quota_json,
  grant_status = EXCLUDED.grant_status,
  created_at = EXCLUDED.created_at,
  query_surface_id = EXCLUDED.query_surface_id,
  allowed_template_ids = EXCLUDED.allowed_template_ids,
  execution_rule_snapshot = EXCLUDED.execution_rule_snapshot,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    queryRunRows,
    [
      "query_run_id text",
      "order_id text",
      "template_query_grant_id text",
      "sandbox_session_id text",
      "query_template_id text",
      "query_surface_id text",
      "requester_user_id text",
      "execution_mode text",
      "request_payload_json jsonb",
      "result_summary_json jsonb",
      "result_object_id text",
      "result_row_count integer",
      "billed_units text",
      "export_attempt_count integer",
      "status text",
      "started_at text",
      "completed_at text",
      "masked_level text",
      "export_scope text",
      "sensitive_policy_snapshot jsonb",
    ].join(", ")
  )}
)
INSERT INTO delivery.query_execution_run (
  query_run_id,
  order_id,
  template_query_grant_id,
  sandbox_session_id,
  query_template_id,
  query_surface_id,
  requester_user_id,
  execution_mode,
  request_payload_json,
  result_summary_json,
  result_object_id,
  result_row_count,
  billed_units,
  export_attempt_count,
  status,
  started_at,
  completed_at,
  masked_level,
  export_scope,
  sensitive_policy_snapshot
)
SELECT
  query_run_id::uuid,
  order_id::uuid,
  NULLIF(template_query_grant_id, '')::uuid,
  NULLIF(sandbox_session_id, '')::uuid,
  NULLIF(query_template_id, '')::uuid,
  NULLIF(query_surface_id, '')::uuid,
  requester_user_id::uuid,
  execution_mode,
  request_payload_json,
  result_summary_json,
  NULLIF(result_object_id, '')::uuid,
  result_row_count,
  billed_units::numeric,
  export_attempt_count,
  status,
  started_at::timestamptz,
  completed_at::timestamptz,
  masked_level,
  export_scope,
  sensitive_policy_snapshot
FROM source
ON CONFLICT (query_run_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  template_query_grant_id = EXCLUDED.template_query_grant_id,
  sandbox_session_id = EXCLUDED.sandbox_session_id,
  query_template_id = EXCLUDED.query_template_id,
  query_surface_id = EXCLUDED.query_surface_id,
  requester_user_id = EXCLUDED.requester_user_id,
  execution_mode = EXCLUDED.execution_mode,
  request_payload_json = EXCLUDED.request_payload_json,
  result_summary_json = EXCLUDED.result_summary_json,
  result_object_id = EXCLUDED.result_object_id,
  result_row_count = EXCLUDED.result_row_count,
  billed_units = EXCLUDED.billed_units,
  export_attempt_count = EXCLUDED.export_attempt_count,
  status = EXCLUDED.status,
  started_at = EXCLUDED.started_at,
  completed_at = EXCLUDED.completed_at,
  masked_level = EXCLUDED.masked_level,
  export_scope = EXCLUDED.export_scope,
  sensitive_policy_snapshot = EXCLUDED.sensitive_policy_snapshot,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    reportArtifactRows,
    [
      "report_artifact_id text",
      "order_id text",
      "object_id text",
      "report_type text",
      "version_no integer",
      "status text",
      "created_at text",
    ].join(", ")
  )}
)
INSERT INTO delivery.report_artifact (
  report_artifact_id,
  order_id,
  object_id,
  report_type,
  version_no,
  status,
  created_at
)
SELECT
  report_artifact_id::uuid,
  order_id::uuid,
  object_id::uuid,
  report_type,
  version_no,
  status,
  created_at::timestamptz
FROM source
ON CONFLICT (report_artifact_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  object_id = EXCLUDED.object_id,
  report_type = EXCLUDED.report_type,
  version_no = EXCLUDED.version_no,
  status = EXCLUDED.status,
  created_at = EXCLUDED.created_at,
  updated_at = NOW();

WITH source AS (
  ${insertSqlFromJson(
    deliveryTicketRows,
    [
      "ticket_id text",
      "order_id text",
      "buyer_org_id text",
      "token_hash text",
      "expire_at text",
      "download_limit text",
      "status text",
      "created_at text",
    ].join(", ")
  )}
)
INSERT INTO delivery.delivery_ticket (
  ticket_id,
  order_id,
  buyer_org_id,
  token_hash,
  expire_at,
  download_limit,
  status,
  created_at
)
SELECT
  ticket_id::uuid,
  order_id::uuid,
  buyer_org_id::uuid,
  token_hash,
  expire_at::timestamptz,
  download_limit::integer,
  status,
  created_at::timestamptz
FROM source
ON CONFLICT (ticket_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  buyer_org_id = EXCLUDED.buyer_org_id,
  token_hash = EXCLUDED.token_hash,
  expire_at = EXCLUDED.expire_at,
  download_limit = EXCLUDED.download_limit,
  status = EXCLUDED.status,
  created_at = EXCLUDED.created_at;

WITH source AS (
  ${insertSqlFromJson(
    deliveryRecordRows,
    [
      "delivery_id text",
      "order_id text",
      "object_id text",
      "delivery_type text",
      "delivery_route text",
      "executor_type text",
      "status text",
      "delivery_commit_hash text",
      "trust_boundary_snapshot jsonb",
      "receipt_hash text",
      "committed_at text",
      "expires_at text",
      "created_by text",
      "authority_model text",
      "business_state_version integer",
      "proof_commit_state text",
      "proof_commit_policy text",
      "external_fact_status text",
      "reconcile_status text",
      "sensitive_delivery_mode text",
      "disclosure_review_status text",
    ].join(", ")
  )}
)
INSERT INTO delivery.delivery_record (
  delivery_id,
  order_id,
  object_id,
  delivery_type,
  delivery_route,
  executor_type,
  status,
  delivery_commit_hash,
  trust_boundary_snapshot,
  receipt_hash,
  committed_at,
  expires_at,
  created_by,
  authority_model,
  business_state_version,
  proof_commit_state,
  proof_commit_policy,
  external_fact_status,
  reconcile_status,
  sensitive_delivery_mode,
  disclosure_review_status
)
SELECT
  delivery_id::uuid,
  order_id::uuid,
  NULLIF(object_id, '')::uuid,
  delivery_type,
  delivery_route,
  executor_type,
  status,
  delivery_commit_hash,
  trust_boundary_snapshot,
  receipt_hash,
  committed_at::timestamptz,
  expires_at::timestamptz,
  created_by::uuid,
  authority_model,
  business_state_version,
  proof_commit_state,
  proof_commit_policy,
  external_fact_status,
  reconcile_status,
  sensitive_delivery_mode,
  disclosure_review_status
FROM source
ON CONFLICT (delivery_id) DO UPDATE SET
  order_id = EXCLUDED.order_id,
  object_id = EXCLUDED.object_id,
  delivery_type = EXCLUDED.delivery_type,
  delivery_route = EXCLUDED.delivery_route,
  executor_type = EXCLUDED.executor_type,
  status = EXCLUDED.status,
  delivery_commit_hash = EXCLUDED.delivery_commit_hash,
  trust_boundary_snapshot = EXCLUDED.trust_boundary_snapshot,
  receipt_hash = EXCLUDED.receipt_hash,
  committed_at = EXCLUDED.committed_at,
  expires_at = EXCLUDED.expires_at,
  created_by = EXCLUDED.created_by,
  authority_model = EXCLUDED.authority_model,
  business_state_version = EXCLUDED.business_state_version,
  proof_commit_state = EXCLUDED.proof_commit_state,
  proof_commit_policy = EXCLUDED.proof_commit_policy,
  external_fact_status = EXCLUDED.external_fact_status,
  reconcile_status = EXCLUDED.reconcile_status,
  sensitive_delivery_mode = EXCLUDED.sensitive_delivery_mode,
  disclosure_review_status = EXCLUDED.disclosure_review_status,
  updated_at = NOW();

INSERT INTO public.seed_history (version, name, checksum_sha256)
VALUES ('${demoSeedVersion}', '${demoSeedName}', '${seedRows.checksum}')
ON CONFLICT (version) DO UPDATE SET
  name = EXCLUDED.name,
  checksum_sha256 = EXCLUDED.checksum_sha256,
  executed_at = NOW();

COMMIT;
`;
}

function queryBaseProducts(config, productIds) {
  return queryJson(
    config,
    `SELECT COALESCE(json_agg(row_to_json(t) ORDER BY t.product_id), '[]'::json)::text
     FROM (
       SELECT
         product_id::text AS product_id,
         asset_id::text AS asset_id,
         asset_version_id::text AS asset_version_id,
         seller_org_id::text AS seller_org_id,
         title
       FROM catalog.product
       WHERE product_id = ANY (ARRAY[${productIds.map((item) => `'${item}'::uuid`).join(", ")}])
     ) AS t`
  );
}

function assertSubjectAndProductCoverage(config, subjects, catalog) {
  const orgCount = Number(
    queryText(
      config,
      `SELECT COUNT(*) FROM core.organization WHERE org_id = ANY (ARRAY[${subjects.organizations
        .map((item) => `'${item.org_id}'::uuid`)
        .join(", ")}])`
    )
  );
  const userCount = Number(
    queryText(
      config,
      `SELECT COUNT(*) FROM core.user_account WHERE user_id = ANY (ARRAY[${subjects.users
        .map((item) => `'${item.user_id}'::uuid`)
        .join(", ")}])`
    )
  );
  const appCount = Number(
    queryText(
      config,
      `SELECT COUNT(*) FROM core.application WHERE app_id = ANY (ARRAY[${subjects.applications
        .map((item) => `'${item.app_id}'::uuid`)
        .join(", ")}])`
    )
  );
  const providerCount = Number(
    queryText(config, `SELECT COUNT(*) FROM payment.provider WHERE provider_key = '${providerKey}'`)
  );
  assert(orgCount === subjects.organizations.length, `demo organizations missing after seed-up: expected ${subjects.organizations.length}, got ${orgCount}`);
  assert(userCount === subjects.users.length, `demo users missing after seed-up: expected ${subjects.users.length}, got ${userCount}`);
  assert(appCount === subjects.applications.length, `demo applications missing after seed-up: expected ${subjects.applications.length}, got ${appCount}`);
  assert(providerCount === 1, `payment.provider(provider_key='${providerKey}') missing; run migrations before seed-demo`);

  const productRows = queryBaseProducts(
    config,
    catalog.official_display_products.map((item) => item.product_id)
  );
  assert(
    productRows.length === catalog.official_display_products.length,
    `official demo products missing after seed-up: expected ${catalog.official_display_products.length}, got ${productRows.length}`
  );
  return productRows;
}

function printPlan(fixtures) {
  const summary = {
    organizations: fixtures.subjects.organizations.length,
    users: fixtures.subjects.users.length,
    applications: fixtures.subjects.applications.length,
    products: fixtures.catalog.official_display_products.length,
    skus: fixtures.catalog.official_display_skus.length,
    orders: fixtures.orders.order_blueprints.length,
    payment_samples: fixtures.billing.billing_samples.length,
    delivery_blueprints: fixtures.delivery.delivery_blueprints.length,
  };
  console.log("[dry-run] seed-demo plan");
  Object.entries(summary).forEach(([key, value]) => {
    console.log(`  - ${key}: ${value}`);
  });
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const dbConfig = loadDbConfig();
  const fixtures = {
    subjects: await readJson("fixtures/demo/subjects.json"),
    catalog: await readJson("fixtures/demo/catalog.json"),
    orders: await readJson("fixtures/demo/orders.json"),
    billing: await readJson("fixtures/demo/billing.json"),
    delivery: await readJson("fixtures/demo/delivery.json"),
  };

  if (!args.skipBaseSeeds) {
    console.log(`[info] running base seed manifest: ${args.manifest}`);
    runBaseSeeds(dbConfig, args.manifest, args.dryRun);
  }

  if (args.dryRun) {
    printPlan(fixtures);
    return;
  }

  const tablesReady = queryText(
    dbConfig,
    "SELECT (to_regclass('core.organization') IS NOT NULL AND to_regclass('catalog.product') IS NOT NULL AND to_regclass('trade.order_main') IS NOT NULL AND to_regclass('payment.payment_intent') IS NOT NULL AND to_regclass('delivery.delivery_record') IS NOT NULL)::text"
  );
  assert(
    tablesReady === "t" || tablesReady === "true",
    "required schema tables are missing; run migrations before seed-demo"
  );

  const productRows = assertSubjectAndProductCoverage(dbConfig, fixtures.subjects, fixtures.catalog);
  const seedRows = buildSeedPayloads(fixtures, fixtures.subjects, fixtures.catalog, productRows);
  const sql = buildImportSql(seedRows);

  console.log("[info] importing demo order/payment/delivery bundle");
  runPsqlScript(dbConfig, sql);

  if (args.verify) {
    console.log("[info] verifying demo seed");
    runCommand("node", ["./scripts/check-demo-seed.mjs"], {
      env: dbEnv(dbConfig),
      stdio: "inherit",
    });
  }

  console.log("[ok] demo seed imported");
}

main().catch((error) => {
  console.error(`[fail] ${error.message}`);
  process.exit(1);
});
