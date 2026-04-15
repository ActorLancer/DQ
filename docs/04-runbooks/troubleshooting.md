# Troubleshooting（BOOT-009）

- Docker daemon 不可达：确认 `docker ps` 正常。
- 端口冲突：修改 `.env` 中端口后重启。
- 依赖未就绪：执行 `./scripts/wait-for-services.sh core`。
- 需要清理本仓库本地资源时：
  - 先预览：`./scripts/prune-local.sh --dry-run`
  - 再执行：`./scripts/prune-local.sh --force`
  - 该脚本仅清理当前 compose project 与 `infra/fabric/state`，避免误删其他开发项目容器。
