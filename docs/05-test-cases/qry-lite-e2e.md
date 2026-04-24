# TEST-026 QRY_LITE 端到端验收清单

`TEST-026` 的正式目标，不是再补一条 `QRY_LITE` smoke，而是把 `模板查询` 收口为一条可重复的正式 gate：

- 门户 `template-query / query-runs / billing/refunds` 页必须由正式 `seller_operator / buyer_operator / platform_risk_settlement` 角色承接
- 模板授权、参数校验、query run、结果读取、退款必须真实走 `/api/platform -> platform-core`
- `trade013 / dlv011 / dlv012 / dlv013 / bil024` 必须给出结构化后端证据
- `DB / audit / outbox / MinIO / billing` 必须能回查到同一条 `QRY_LITE` 闭环

## Authority

- 任务与顺序：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 流程 authority：`docs/业务流程/业务流程图-V1-完整版.md` `4.4.3`
- 页面 authority：`docs/页面说明书/页面说明书-V1-完整版.md` `7.7`
- 测试策略 authority：`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md` `15.1`
- 冻结用例：
  - `docs/05-test-cases/delivery-cases.md`
  - `docs/05-test-cases/payment-billing-cases.md`
  - `docs/05-test-cases/five-standard-scenarios-e2e.md`
  - `docs/05-test-cases/v1-core-acceptance-checklist.md`
- Demo authority：`fixtures/demo/orders.json`、`fixtures/demo/delivery.json`、`fixtures/demo/subjects.json`

## Official Entry

本地 / CI 正式入口：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-qry-lite-e2e.sh
```

该 checker 会串联：

1. `smoke-local.sh`
2. `seed-local-iam-test-identities.sh`
3. `seed-demo.sh --skip-base-seeds`
4. `trade013_qry_lite_state_machine_db_smoke`
5. `dlv011_template_grant_db_smoke`
6. `dlv012_template_run_db_smoke`
7. `dlv013_query_runs_db_smoke`
8. `bil024_billing_trigger_bridge_db_smoke`
9. `apps/portal-web/e2e/test026-qry-lite-live.spec.ts`
10. `scripts/check-qry-lite-e2e.mjs`

## Required Evidence

| 段落 | 正式入口 | 必须证明的事实 |
| --- | --- | --- |
| 状态机 | `trade013_qry_lite_state_machine_db_smoke` | `authorize_template -> validate_params -> execute_query -> make_result_available -> close_acceptance` 成立；`closed -> execute_query` 返回 `QRY_LITE_TRANSITION_FORBIDDEN` |
| 模板授权 | `dlv011_template_grant_db_smoke` | `POST /template-grants` 的 grant/update 成立；`trade.order_main`、`delivery.template_query_grant`、`delivery.delivery_record`、`audit.audit_event`、`ops.outbox_event` 联查成立 |
| 运行执行 | `dlv012_template_run_db_smoke` | `POST /template-runs` 的参数校验、审批票、MinIO 结果对象、`delivery.template_query.use` 审计与 `billing.trigger.bridge` 成立 |
| 结果读取 | `dlv013_query_runs_db_smoke` | `GET /template-runs` 返回 query history、policy hits、audit refs、result summary，且最新 run 与 DB/MinIO 一致 |
| 账单桥接 | `bil024_billing_trigger_bridge_db_smoke` | `QRY_LITE` 在 `execution_completed` 阶段桥接到 Billing，`one_time_charge` 与 published outbox 成立 |
| 门户 live | `test026-qry-lite-live.spec.ts` | `seller_operator` 真正提交 grant，`buyer_operator` 真正提交 query run 并读取结果，`platform_risk_settlement` 真正执行退款，浏览器只经过 `/api/platform/**` |
| 汇总 | `check-qry-lite-e2e.mjs` | portal `request_id` 与 `DB / audit / outbox / MinIO / billing` 对齐，形成单一 `summary.json` |

## Portal Live Boundary

门户 live E2E 必须满足以下约束：

- 使用 Keycloak password grant 的正式本地身份：
  - `local-seller-operator`
  - `local-buyer-operator`
  - `local-risk-settlement`
- 只访问：
  - `http://127.0.0.1:3101`
  - `/api/platform/**`
- 不允许浏览器直接命中：
  - `PostgreSQL`
  - `Kafka`
  - `Redis`
  - `OpenSearch`
  - `Fabric`
  - `platform-core :8094`

## DB / Audit / Outbox Readback

`TEST-026` 至少要回查以下正式对象：

- `trade.order_main`
- `delivery.template_query_grant`
- `delivery.query_execution_run`
- `delivery.storage_object`
- `billing.refund_record`
- `billing.settlement_record`
- `audit.audit_event`
- `ops.outbox_event`

正式回查点：

- grant 后 `current_state=template_authorized`
- run 后 `current_state=query_executed`、`status=completed`、`result_row_count=2`
- `delivery.template_query.enable` 审计为 `1`
- `delivery.template_query.use` 审计为 `1`
- `billing.refund.execute` 审计为 `1`
- grant 对应 `delivery.committed` outbox 真实存在
- run 对应 `billing.trigger.bridge` outbox 真实存在，`billing_trigger=bill_once_after_task_acceptance`
- refund 对应 `billing.refund_record` canonical outbox 与 `dtp.notification.dispatch` 真实存在
- MinIO `query-runs/{order_id}/{query_run_id}/result.json` 与 `delivery.query_execution_run.result_object_id` 一致

## Artifacts

默认产物目录：

```text
target/test-artifacts/qry-lite-e2e/
```

必须至少包含：

- `executed-cargo-tests.txt`
- `cargo-tests.log`
- `portal-live.log`
- `live-fixture.json`
- `summary.json`
- `raw/trade013-qry-lite-state-machine.json`
- `raw/dlv011-template-grant.json`
- `raw/dlv012-template-run.json`
- `raw/dlv013-query-runs.json`
- `raw/bil024-qry-lite-billing-bridge.json`
- `raw/portal-qry-lite-live.json`

## False Positive Guards

以下情况不算通过：

- 只证明 `/template-grants` 成功，没有 buyer run/read 和 risk refund
- 只证明 `query_execution_run` 有一行，没有回查 `trade.order_main / audit / outbox / MinIO`
- 只证明 run 返回 `HTTP 200`，没有验证审批票、参数校验和结果对象
- 只证明退款接口返回成功，没有走正式 `platform_risk_settlement`、step-up、`billing.refund.execute` 审计和通知/outbox
- 用 demo 固定订单直接反复改状态，导致 fixture 漂移且无法重复执行
