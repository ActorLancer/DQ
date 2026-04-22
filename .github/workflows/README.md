# GitHub Workflows 校准（BOOT-036）

`.github/workflows/` 已预留 CI 落位。

当前已落地：

- `canonical-contracts.yml`：执行 `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`，用于拦截 canonical topic / OpenAPI / consumer group / 文档边界漂移。

后续任务将继续补齐：

- 契约校验
- 迁移校验
- smoke test
- lint/test 工作流
