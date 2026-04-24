# TEST-025 SHARE_RO 端到端验收清单

`TEST-025` 的正式目标，不是再补一条 `SHARE_RO` smoke，而是把 `只读共享` 收口为一条可重复的正式 gate：

- 门户 `share` 页必须由正式 `seller_operator / buyer_operator` 角色承接
- grant / read / revoke 必须真实走 `/api/platform -> platform-core`
- 争议冻结、撤权退款占位必须有 billing 侧正式证据
- DB / audit / outbox 必须能回查到同一条 `SHARE_RO` 闭环

## Authority

- 任务与顺序：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 流程 authority：`docs/业务流程/业务流程图-V1-完整版.md` `4.4.1B`
- 页面 authority：`docs/页面说明书/页面说明书-V1-完整版.md` `7.3`
- 测试策略 authority：`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md` `15.1`
- 冻结用例：
  - `docs/05-test-cases/order-state-machine.md`
  - `docs/05-test-cases/delivery-cases.md`
  - `docs/05-test-cases/payment-billing-cases.md`
  - `docs/05-test-cases/delivery-revocation-cases.md`
  - `docs/05-test-cases/v1-core-acceptance-checklist.md`
- Demo authority：`fixtures/demo/orders.json`、`fixtures/demo/delivery.json`、`fixtures/demo/subjects.json`

## Official Entry

本地 / CI 正式入口：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-share-ro-e2e.sh
```

该 checker 会串联：

1. `smoke-local.sh`
2. `seed-local-iam-test-identities.sh`
3. `seed-demo.sh --skip-base-seeds`
4. `trade012_share_ro_state_machine_db_smoke`
5. `dlv006_share_grant_db_smoke`
6. `bil026_share_ro_billing_db_smoke`
7. `apps/portal-web/e2e/test025-share-ro-live.spec.ts`
8. `scripts/check-share-ro-e2e.mjs`

## Required Evidence

| 段落 | 正式入口 | 必须证明的事实 |
| --- | --- | --- |
| 状态机 | `trade012_share_ro_state_machine_db_smoke` | `enable_share -> grant_read_access -> confirm_first_query -> interrupt_dispute` 成立；`revoked -> grant_read_access` 返回 `SHARE_RO_TRANSITION_FORBIDDEN` |
| 授权 / 撤权 | `dlv006_share_grant_db_smoke` | `POST/GET /share-grants` 的 grant/read/revoke 成立；`trade.order_main`、`delivery.data_share_grant`、`delivery.delivery_record`、`audit.audit_event`、`ops.outbox_event` 联查成立 |
| 账单 / 争议 | `bil026_share_ro_billing_db_smoke` | `enable_share -> cycle_charge -> revoke` 产生 `one_time_charge / recurring_charge / refund_adjustment`；争议打开后结算冻结 |
| 门户 live | `test025-share-ro-live.spec.ts` | `seller_operator` 真正提交 grant/revoke，`buyer_operator` 真正读取 share grant 列表，浏览器只经过 `/api/platform/**` |
| 汇总 | `check-share-ro-e2e.mjs` | portal `request_id` 与 DB/audit/outbox 对齐，形成单一 `summary.json` |

## Portal Live Boundary

门户 live E2E 必须满足以下约束：

- 使用 Keycloak password grant 的正式本地身份：
  - `local-seller-operator`
  - `local-buyer-operator`
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

`TEST-025` 至少要回查以下正式对象：

- `trade.order_main`
- `delivery.data_share_grant`
- `delivery.delivery_record`
- `billing.billing_event`
- `billing.settlement_record`
- `audit.audit_event`
- `ops.outbox_event`

正式回查点：

- grant 后 `current_state=share_granted`
- revoke 后 `current_state=revoked`
- 最新 `data_share_grant.grant_status=revoked`
- 最新 `delivery_record.status=revoked`
- `delivery.share.enable` 审计为 `2`
- `delivery.share.read` 审计为 `1`
- `trade.order.share_ro.transition` 审计为 `2`
- grant 对应 `delivery.committed` 与 `billing.trigger.bridge` outbox 真实存在

## Artifacts

默认产物目录：

```text
target/test-artifacts/share-ro-e2e/
```

必须至少包含：

- `executed-cargo-tests.txt`
- `cargo-tests.log`
- `portal-live.log`
- `live-fixture.json`
- `summary.json`
- `raw/trade012-share-ro-state-machine.json`
- `raw/dlv006-share-grant.json`
- `raw/bil026-share-ro-billing.json`
- `raw/portal-share-live.json`

## False Positive Guards

以下情况不算通过：

- 只证明页面打开，没有真实提交 grant/revoke
- 只证明 `delivery.data_share_grant` 有一行，没有回查 `order_main / delivery_record / audit / outbox`
- 只证明 `SHARE_RO` 状态机存在，没有账单/争议冻结证据
- 用 demo 固定订单直接反复改状态，导致 fixture 漂移且无法重复执行
- 用本地伪 session 或 `x-role` 旁路替代 Keycloak bearer 身份
