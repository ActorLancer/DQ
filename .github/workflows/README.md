# GitHub Workflows 校准（BOOT-036）

`.github/workflows/` 已预留 CI 落位。

当前已落地：

- `canonical-contracts.yml`：执行 `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`，用于拦截 canonical topic / OpenAPI / consumer group / 文档边界漂移。
- `contract-tests.yml`：执行 `./scripts/check-api-contract-baseline.sh`，承接 `TEST-003` 正式 API contract baseline。
- `migration-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-migration-smoke.sh`，承接 `TEST-004` migration smoke。
- `local-environment-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`，承接 `TEST-005` 本地环境 smoke。
- `order-e2e.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-order-e2e.sh`，承接 `TEST-006` 五条标准链路 order E2E。
- `provider-switch.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh`，承接 `TEST-007` 支付 / 签章 / 链写 provider 切换验收。
- `outbox-consistency.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-outbox-consistency.sh`，承接 `TEST-008` 事务写入 / outbox 发布 / consumer 幂等验收。
- `audit-completeness.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-audit-completeness.sh`，承接 `TEST-009` 审计留痕 / step-up / 证据导出验收。
- `search-rec-pg-authority.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-searchrec-pg-authority.sh`，承接 `TEST-010` 搜索 / 推荐回 PostgreSQL 最终校验验收。
- `payment-webhook-idempotency.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-payment-webhook-idempotency.sh`，承接 `TEST-011` 支付 webhook duplicate / out-of-order / late success 保护验收。

后续任务将继续补齐：

- smoke test
- lint/test 工作流
