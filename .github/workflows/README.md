# GitHub Workflows 校准（BOOT-036）

`.github/workflows/` 已预留 CI 落位。

当前已落地：

- `canonical-contracts.yml`：执行 `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`，用于拦截 canonical topic / OpenAPI / consumer group / 文档边界漂移。
- `contract-tests.yml`：执行 `./scripts/check-api-contract-baseline.sh`，承接 `TEST-003` 正式 API contract baseline。
- `migration-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-migration-smoke.sh`，承接 `TEST-004` migration smoke。
- `local-environment-smoke.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`，承接 `TEST-005` 本地环境 smoke。
- `order-e2e.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-order-e2e.sh`，承接 `TEST-006` 五条标准链路 order E2E。
- `provider-switch.yml`：执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh`，承接 `TEST-007` 支付 / 签章 / 链写 provider 切换验收。

后续任务将继续补齐：

- smoke test
- lint/test 工作流
