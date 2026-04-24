# GitHub Workflows 校准（BOOT-036）

`.github/workflows/` 已预留 CI 落位。

当前已落地：

- `canonical-contracts.yml`：执行 `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`，用于拦截 canonical topic / OpenAPI / consumer group / 文档边界漂移。
- `contract-tests.yml`：执行 `./scripts/check-api-contract-baseline.sh`，承接 `TEST-003` 正式 API contract baseline。
- `migration-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-migration-smoke.sh`，承接 `TEST-004` migration smoke。
- `local-environment-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-compose-smoke.sh`；CI 内承接 `TEST-016` compose smoke，并复用 `TEST-005` 的 `smoke-local.sh` 运行态检查与 canonical 静态漂移拦截。
- `schema-drift.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-schema-drift.sh`，承接 `TEST-017` 的 migration / `.sqlx` / `db::entity` / OpenAPI drift gate，并上传实体 / 表清单 artifact。
- `performance-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-performance-smoke.sh`，承接 `TEST-018` 搜索 / 下单 / 交付 / 审计联查四条正式 API 的基础性能守门，并上传耗时汇总、Prometheus / metrics 快照与 `platform-core` 请求日志。
- `failure-drills.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-failure-drills.sh`，承接 `TEST-019` 的 Kafka / OpenSearch / Fabric Adapter / Mock Payment 四类故障演练，并上传 group lag、响应快照、compose 日志与 live smoke 输出。
- `rollback-recovery.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-rollback-recovery.sh`，承接 `TEST-020` 的业务库 reset、基础 seed replay、环境重启与 demo 数据恢复演练，并上传 rollback/recovery 汇总与 compose 日志。
- `standard-sku-coverage.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-standard-sku-coverage.sh`，承接 `TEST-023` 的 8 个标准 SKU 覆盖矩阵、五条标准链路挂点、billing basis live readback 与 matrix artifact 上传。
- `order-orchestration.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-order-orchestration.sh`，承接 `TEST-024` 的编排链路 gate，汇总 `TEST-006` 前端 scenario baseline、`trade030 / dlv029 / dlv017 / dlv018 / dlv025 / bil024 / bil025` 动态 order ids 与 `20+ order` sign-off artifact。
- `share-ro-e2e.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-share-ro-e2e.sh`，承接 `TEST-025` 的 `SHARE_RO` 端到端 gate，汇总 `trade012 / dlv006 / bil026` 后端证据、门户 seller/buyer live E2E、临时 live fixture 与 `audit / outbox / DB` summary artifact。
- `qry-lite-e2e.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-qry-lite-e2e.sh`，承接 `TEST-026` 的 `QRY_LITE` 端到端 gate，汇总 `trade013 / dlv011 / dlv012 / dlv013 / bil024` 后端证据、门户 seller/buyer/risk live E2E、临时 live fixture 与 `audit / outbox / MinIO / billing` summary artifact。
- `order-e2e.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-order-e2e.sh`，承接 `TEST-006` 五条标准链路 order E2E。
- `provider-switch.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh`，承接 `TEST-007` 支付 / 签章 / 链写 provider 切换验收。
- `outbox-consistency.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-outbox-consistency.sh`，承接 `TEST-008` 事务写入 / outbox 发布 / consumer 幂等验收。
- `audit-completeness.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-audit-completeness.sh`，承接 `TEST-009` 审计留痕 / step-up / 证据导出验收。
- `search-rec-pg-authority.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-searchrec-pg-authority.sh`，承接 `TEST-010` 搜索 / 推荐回 PostgreSQL 最终校验验收。
- `payment-webhook-idempotency.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-payment-webhook-idempotency.sh`，承接 `TEST-011` 支付 webhook duplicate / out-of-order / late success 保护验收。
- `delivery-revocation.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-delivery-revocation.sh`，承接 `TEST-012` 文件票据 / share / API / sandbox 断权与正式入口拒绝验收。
- `dispute-settlement-linkage.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-dispute-settlement-linkage.sh`，承接 `TEST-013` 争议冻结结算与裁决后退款 / 赔付重算验收。
- `audit-replay-dry-run.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-audit-replay-dry-run.sh`，承接 `TEST-014` 审计 replay dry-run 差异报告、MinIO report 与权限 / step-up 验收。
- `ci-minimal-matrix.yml`：执行 `./scripts/check-ci-minimal-matrix.sh` 的 5 个 lane，承接 `TEST-015` 最小 CI 矩阵：Rust lint/test、TS lint/test、Go build/test、migration check、OpenAPI check。

说明：

- 早期 `BOOT-011` 留下的 `build.yml / lint.yml / test.yml` placeholder 已由 `TEST-015` 移除，避免继续给出无实际校验的假绿灯。
- 后续 `TEST` task 仍可继续追加专项 smoke / failure drill / compose 级 workflow，但都不应回退 `ci-minimal-matrix.yml` 的最小基线。
