# Rollback Recovery Cases

`TEST-020` 的正式目标是证明本地正式环境在“业务库重置 -> 重新迁移 -> 重放 seed -> 恢复 demo 数据”之后，仍能回到可验收的基线状态。

正式入口：

- 本地 / CI：`ENV_FILE=infra/docker/.env.local ./scripts/check-rollback-recovery.sh`

## 冻结边界

- 回滚演练只重置业务库 `datab`，不破坏 Keycloak 独立服务数据库。
- 迁移必须继续复用正式 `db/scripts/migrate-reset.sh -> migrate-up.sh` 顺序，不允许临时改用另一套 migration 入口。
- 基础 seed 必须继续复用 `smoke-local.sh` / `seed-up.sh` 正式路径；demo 数据恢复必须继续复用 `seed-demo.sh`，不允许手工插库。
- 恢复完成后，目标状态不是“库里能连上”，而是五条标准链路 demo 订单/支付/交付对象与正式运行态探针全部恢复。

## 子场景矩阵

| Case | 阶段 | 正式动作 | 正式断言 | 回查 |
| --- | --- | --- | --- | --- |
| `RBR-001` 基线确认 | 回滚前 | `smoke-local.sh` + `seed-demo.sh --skip-base-seeds` | 当前正式 demo 数据已存在 | `check-demo-seed.sh` 通过，且 demo fixture 专项计数为 `orders=10 / payment_intents=10 / delivery_records=11` |
| `RBR-002` 业务库重置 | 回滚中 | `down-local.sh` + `up-local.sh` + `db/scripts/migrate-reset.sh` | 业务库被重建且 demo/seed 数据清空 | `trade.order_main=0`、`public.seed_history=0` |
| `RBR-003` 基础恢复 | 重建后 | `smoke-local.sh` | 正式基础 seed、runtime、MinIO、Keycloak、Grafana、canonical topics 恢复 | `TEST-005` 全量通过 |
| `RBR-004` Demo 恢复 | 重建后 | `seed-local-iam-test-identities.sh` + `seed-demo.sh --skip-base-seeds` + `check-demo-seed.sh` | 五条标准链路 demo 数据恢复 | `check-demo-seed.sh` 通过、demo fixture 专项计数恢复到 `10/10/11`，并且 `check-keycloak-realm.sh` buyer grant 通过 |

## Checker 行为

`check-rollback-recovery.sh` 会按顺序执行：

1. 先通过 `smoke-local.sh`、`seed-local-iam-test-identities.sh`、`seed-demo.sh --skip-base-seeds` 与 `check-demo-seed.sh` 确认回滚前正式基线存在；回查口径固定为 demo fixture UUID 对象，不使用全库总数。
2. 停掉 host `platform-core` 和本地 compose stack，避免旧连接干扰业务库 drop/recreate。
3. 重新拉起 `core + observability + mocks` 依赖后，执行正式 `db/scripts/migrate-reset.sh`，验证业务库被重建且 `seed_history / trade.order_main` 清空。
4. 再执行 `smoke-local.sh` 重放正式基础 seed 和运行态恢复。
5. 最后恢复 IAM test principals 与 demo 数据，并通过 `check-demo-seed.sh` 与 `check-keycloak-realm.sh` 证明系统回到正式演示基线。

## Artifact

- `target/test-artifacts/rollback-recovery/summary.json`
- `baseline-demo-seed.txt`
- `baseline-order-count.txt`
- `post-reset-order-count.txt`
- `restored-demo-seed.txt`
- `restored-order-count.txt`
- `restored-payment-intent-count.txt`
- `restored-delivery-count.txt`

## 清理边界

- 本任务的目标是把环境恢复到正式 demo 基线，因此脚本结束后保留恢复后的本地环境和 demo 数据。
- CI workflow 会在 artifact 上传后执行 `down-local.sh`，避免 runner 泄漏本地资源。
