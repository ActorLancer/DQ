# Audit / Consistency 验收清单

当前文件承接 `AUD-003`、`AUD-004`、`AUD-005`、`AUD-006` 已落地的首版审计控制面验收矩阵，覆盖：

- 订单审计联查：`GET /api/v1/audit/orders/{id}`
- 全局审计 trace 查询：`GET /api/v1/audit/traces`
- 证据包导出：`POST /api/v1/audit/packages/export`
- 回放任务 dry-run：`POST /api/v1/audit/replay-jobs`
- 回放任务联查：`GET /api/v1/audit/replay-jobs/{id}`
- legal hold 创建 / 释放：`POST /api/v1/audit/legal-holds`、`POST /api/v1/audit/legal-holds/{id}/release`

后续 `legal hold`、anchor / Fabric、dead letter reprocess、reconcile 等高风险控制面进入对应 `AUD` task 后，再继续追加到本文件，不得另起旁路清单。

## 前置条件

- 已执行：`set -a; source infra/docker/.env.local; set +a`
- 本地基础设施可用：`PostgreSQL`、`MinIO`、`Kafka`、`Redis`、`Keycloak / IAM`、观测栈
- 平台服务可用：`platform-core`
- 如需一次性跑首版 live smoke：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture
```

## 验收矩阵

| 用例ID | 场景 | 输入 / 操作 | 预期结果 | 主要回查点 |
| --- | --- | --- | --- | --- |
| `AUD-CASE-001` | 订单审计联查 | `GET /api/v1/audit/orders/{id}` | 返回订单最小审计视图，租户读场景只允许 buyer/seller org | API 响应、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-002` | 全局 trace 查询 | `GET /api/v1/audit/traces?order_id=...&trace_id=...` | 返回同一请求链的审计事件分页结果 | API 响应、`audit.audit_event`、`audit.access_audit` |
| `AUD-CASE-003` | 证据包导出 | `POST /api/v1/audit/packages/export` + `x-step-up-challenge-id` | 写入 `audit.evidence_package`、MinIO 导出对象、`audit.audit_event(action_name='audit.package.export')` | API 响应、`audit.evidence_package`、`audit.evidence_manifest_item`、MinIO 对象、`ops.system_log` |
| `AUD-CASE-004` | replay 创建 | `POST /api/v1/audit/replay-jobs` with `dry_run=true` | 写入 `audit.replay_job + audit.replay_result`，MinIO replay report 落盘，并生成 `audit.replay.requested / completed` | API 响应、`audit.replay_job`、`audit.replay_result`、`audit.audit_event`、MinIO 对象 |
| `AUD-CASE-005` | replay 只允许 dry-run | `POST /api/v1/audit/replay-jobs` with `dry_run=false` | 返回 `409` 且错误码 `AUDIT_REPLAY_DRY_RUN_ONLY` | HTTP 响应、错误码 |
| `AUD-CASE-006` | replay 读取 | `GET /api/v1/audit/replay-jobs/{id}` | 返回 replay job + results；读取动作也必须落 `audit.access_audit` 与 `ops.system_log` | API 响应、`audit.access_audit(access_mode='replay')`、`ops.system_log` |
| `AUD-CASE-007` | 高风险动作鉴权 | 缺少权限或缺少 step-up 分别调用 export / replay | 返回 `403 / 400`，不得写业务副作用 | HTTP 响应、`audit.evidence_package / replay_job` 无新增 |
| `AUD-CASE-008` | legal hold 创建 | `POST /api/v1/audit/legal-holds` + `x-step-up-challenge-id` | 写入 `audit.legal_hold`，并产生 `audit.audit_event(action_name='audit.legal_hold.create')`；当前 hold 状态以 `audit.legal_hold` 为权威源，历史 evidence/package 行保持 append-only | API 响应、`audit.legal_hold`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-009` | legal hold 重复创建冲突 | 对同一 active scope 再次创建 | 返回 `409` 且错误码 `AUDIT_LEGAL_HOLD_ACTIVE` | HTTP 响应、错误码、无新增 hold |
| `AUD-CASE-010` | legal hold 释放 | `POST /api/v1/audit/legal-holds/{id}/release` | `status=released`、`approved_by / released_at` 落库，并产生 `audit.audit_event(action_name='audit.legal_hold.release')`；当前 hold 状态回到 `audit.legal_hold` 权威视图中的 `none` | API 响应、`audit.legal_hold`、`audit.audit_event`、`ops.system_log` |

## `AUD-005` 手工回放验证

1. 启动服务：

```bash
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
APP_PORT=18080 \
cargo run -p platform-core-bin
```

2. 准备最小业务对象与 step-up challenge。下面 SQL 使用 `gen_random_uuid()` 生成一组独立数据，返回：
   - `order_id`
   - `audit_user_id`
   - `replay_challenge_id`

```sql
WITH buyer AS (
  INSERT INTO core.organization (org_name, org_type, status, metadata)
  VALUES ('aud005-buyer-manual', 'enterprise', 'active', '{}'::jsonb)
  RETURNING org_id
), seller AS (
  INSERT INTO core.organization (org_name, org_type, status, metadata)
  VALUES ('aud005-seller-manual', 'enterprise', 'active', '{}'::jsonb)
  RETURNING org_id
), asset AS (
  INSERT INTO catalog.asset (owner_org_id, asset_name, asset_type, lifecycle_status, metadata)
  SELECT seller.org_id, 'aud005-asset-manual', 'dataset', 'published', '{}'::jsonb
  FROM seller
  RETURNING asset_id
), asset_version AS (
  INSERT INTO catalog.asset_version (asset_id, version_no, version_label, schema_json, metadata)
  SELECT asset.asset_id, 1, 'v1', '{}'::jsonb, '{}'::jsonb
  FROM asset
  RETURNING asset_version_id
), product AS (
  INSERT INTO catalog.product (owner_org_id, product_name, product_type, status, metadata)
  SELECT seller.org_id, 'aud005-product-manual', 'dataset', 'published', '{}'::jsonb
  FROM seller
  RETURNING product_id
), sku AS (
  INSERT INTO catalog.sku (
    product_id,
    asset_version_id,
    sku_code,
    sku_type,
    billing_mode,
    price_json,
    entitlement_json,
    status,
    metadata
  )
  SELECT product.product_id, asset_version.asset_version_id, 'AUD005-MANUAL', 'DATA', 'ONE_TIME',
         '{"amount":"88.00","currency":"CNY"}'::jsonb, '{}'::jsonb, 'active', '{}'::jsonb
  FROM product, asset_version
  RETURNING sku_id
), order_main AS (
  INSERT INTO trade.order_main (
    buyer_org_id,
    seller_org_id,
    product_id,
    sku_id,
    order_no,
    status,
    payment_status,
    delivery_status,
    acceptance_status,
    settlement_status,
    dispute_status,
    total_amount,
    currency,
    price_snapshot_json,
    metadata
  )
  SELECT buyer.org_id, seller.org_id, product.product_id, sku.sku_id, 'AUD005-MANUAL',
         'created', 'pending', 'pending', 'pending', 'pending', 'none',
         88.00, 'CNY', '{}'::jsonb, '{}'::jsonb
  FROM buyer, seller, product, sku
  RETURNING order_id
), audit_user AS (
  INSERT INTO core.user_account (
    org_id, login_id, display_name, user_type, status, mfa_status, email, attrs
  )
  SELECT buyer.org_id, 'aud005-manual-user', 'AUD005 Manual User',
         'human', 'active', 'verified', 'aud005-manual@example.com', '{}'::jsonb
  FROM buyer
  RETURNING user_id
), audit_seed AS (
  INSERT INTO audit.audit_event (
    event_schema_version, event_class, domain_name, ref_type, ref_id,
    actor_type, actor_id, actor_org_id, tenant_id, action_name, result_code,
    request_id, trace_id, event_time, sensitivity_level, metadata
  )
  SELECT 'v1', 'business', 'trade', 'order', order_main.order_id,
         'user', audit_user.user_id, buyer.org_id, buyer.org_id::text,
         'trade.order.create', 'accepted',
         'req-aud005-manual-seed', 'trace-aud005-manual-seed', now(), 'normal', '{}'::jsonb
  FROM order_main, audit_user, buyer
  RETURNING ref_id
), replay_challenge AS (
  INSERT INTO iam.step_up_challenge (
    user_id,
    challenge_type,
    target_action,
    target_ref_type,
    target_ref_id,
    challenge_status,
    expires_at,
    completed_at,
    metadata
  )
  SELECT audit_user.user_id,
         'mock_otp',
         'audit.replay.execute',
         'order',
         order_main.order_id,
         'verified',
         now() + interval '10 minutes',
         now(),
         jsonb_build_object('seed', 'aud005-manual')
  FROM audit_user, order_main
  RETURNING step_up_challenge_id
)
SELECT
  (SELECT order_id::text FROM order_main) AS order_id,
  (SELECT user_id::text FROM audit_user) AS audit_user_id,
  (SELECT step_up_challenge_id::text FROM replay_challenge) AS replay_challenge_id;
```

3. 创建 replay job：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/replay-jobs \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud005-manual-create' \
  -H 'x-trace-id: trace-aud005-manual-create' \
  -H 'x-step-up-challenge-id: <replay_challenge_id>' \
  -d '{
    "replay_type": "state_replay",
    "ref_type": "order",
    "ref_id": "<order_id>",
    "reason": "manual replay verification",
    "dry_run": true,
    "options": {
      "trigger": "manual_http_check"
    }
  }'
```

4. 读取 replay job：

```bash
curl -sS http://127.0.0.1:18080/api/v1/audit/replay-jobs/<replay_job_id> \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud005-manual-get' \
  -H 'x-trace-id: trace-aud005-manual-get'
```

## 回查清单

1. 回查 replay 任务与结果：

```sql
SELECT replay_type, ref_type, ref_id::text, dry_run, status, request_reason
FROM audit.replay_job
WHERE replay_job_id = '<replay_job_id>'::uuid;

SELECT step_name, result_code
FROM audit.replay_result
WHERE replay_job_id = '<replay_job_id>'::uuid
ORDER BY created_at, replay_result_id;
```

预期：

- `dry_run=true`
- `status=completed`
- 存在 4 条结果：`target_snapshot`、`audit_timeline`、`evidence_projection`、`execution_policy`
- `execution_policy.result_code='AUDIT_REPLAY_DRY_RUN_ONLY'`

2. 回查正式审计与访问留痕：

```sql
SELECT action_name, result_code
FROM audit.audit_event
WHERE ref_type = 'replay_job'
  AND ref_id = '<replay_job_id>'::uuid
ORDER BY event_time, audit_id;

SELECT access_mode, request_id
FROM audit.access_audit
WHERE target_type = 'replay_job'
  AND target_id = '<replay_job_id>'::uuid
ORDER BY created_at, access_audit_id;

SELECT message_text
FROM ops.system_log
WHERE request_id IN ('req-aud005-manual-create', 'req-aud005-manual-get')
ORDER BY created_at, system_log_id;
```

预期：

- `audit.audit_event` 至少存在 `audit.replay.requested`、`audit.replay.completed`
- `audit.access_audit.access_mode='replay'`
- `ops.system_log` 包含：
  - `audit replay job executed: POST /api/v1/audit/replay-jobs`
  - `audit replay lookup executed: GET /api/v1/audit/replay-jobs/{id}`

3. 回查 MinIO replay report：

```sql
SELECT options_json ->> 'report_storage_uri'
FROM audit.replay_job
WHERE replay_job_id = '<replay_job_id>'::uuid;
```

预期：

- `report_storage_uri` 指向 `s3://evidence-packages/replays/<ref_type>/<ref_id>/replay-<job_id>.json`
- 对象可正常读取
- 内容包含：
  - `recommendation=dry_run_completed`
  - `results[*].step_name`
  - `step_up.challenge_id`
  - `target.order_id=<order_id>`

## `AUD-006` 手工 legal hold 验证

1. 准备 step-up challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.legal_hold.manage',
  'order',
  '<order_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud006-manual-create')
)
RETURNING step_up_challenge_id::text;
```

2. 创建 legal hold：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/legal-holds \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud006-manual-create' \
  -H 'x-trace-id: trace-aud006-manual-create' \
  -H 'x-step-up-challenge-id: <create_challenge_id>' \
  -d '{
    "hold_scope_type": "order",
    "hold_scope_id": "<order_id>",
    "reason_code": "regulator_investigation",
    "metadata": {
      "ticket": "AUD-OPS-006"
    }
  }'
```

3. 再次创建同一 scope，确认冲突：

预期：

- 返回 `409`
- 错误码为 `AUDIT_LEGAL_HOLD_ACTIVE`

4. 为释放动作准备新的 challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.legal_hold.manage',
  'legal_hold',
  '<legal_hold_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud006-manual-release')
)
RETURNING step_up_challenge_id::text;
```

5. 释放 legal hold：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/legal-holds/<legal_hold_id>/release \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud006-manual-release' \
  -H 'x-trace-id: trace-aud006-manual-release' \
  -H 'x-step-up-challenge-id: <release_challenge_id>' \
  -d '{
    "reason": "manual review cleared hold"
  }'
```

6. 回查主记录与 scope 状态：

```sql
SELECT status, requested_by::text, approved_by::text, released_at, metadata ->> 'release_reason'
FROM audit.legal_hold
WHERE legal_hold_id = '<legal_hold_id>'::uuid;

SELECT COUNT(*)::bigint
FROM audit.legal_hold
WHERE hold_scope_type = 'order'
  AND hold_scope_id = '<order_id>'::uuid
  AND status = 'active';
```

预期：

- 创建后 `status=active`
- 释放后 `status=released`
- `approved_by=<audit_user_id>`
- `metadata.release_reason='manual review cleared hold'`
- 若上述活跃 hold 计数在释放后重新查询，应返回 `0`
- 历史 `audit.evidence_item / audit.evidence_package` 行保持 append-only，不作为当前 hold 状态权威源

## 清理约束

- 业务测试数据可清理：`trade.order_main` 及本手工步骤创建的临时 `core.organization / catalog.*` scope 图数据
- 与高风险动作审计链绑定的 `core.user_account / iam.step_up_challenge` 在当前运行态不做强删；删除它们会触发 FK 尝试回写 append-only `audit.audit_event`
- 审计数据不清理：`audit.audit_event`、`audit.access_audit`、`ops.system_log`、`audit.replay_job`、`audit.replay_result`、`audit.legal_hold`、MinIO replay report 及相关 evidence snapshot 按 append-only 或审计保留规则保留

## 当前未覆盖项

- `AUD-007+` anchor / Fabric request / callback / reconcile
- `AUD-010+` dead letter reprocess / consistency repair / OpenSearch ops

进入对应批次后，必须在本文件继续追加，不得把本文件视为 `AUD` 全阶段完成证明。
