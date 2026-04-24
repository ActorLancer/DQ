# GitHub Workflows 校准（BOOT-036）

`.github/workflows/` 已预留 CI 落位。

当前已落地：

- `canonical-contracts.yml`：执行 `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`，用于拦截 canonical topic / OpenAPI / consumer group / 文档边界漂移。
- `contract-tests.yml`：执行 `./scripts/check-api-contract-baseline.sh`，承接 `TEST-003` 正式 API contract baseline。
- `migration-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-migration-smoke.sh`，承接 `TEST-004` migration smoke。
- `local-environment-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-compose-smoke.sh`；CI 内承接 `TEST-016` compose smoke，并复用 `TEST-005` 的 `smoke-local.sh` 运行态检查与 canonical 静态漂移拦截。
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
