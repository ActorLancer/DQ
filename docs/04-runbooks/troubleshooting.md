# Troubleshooting（BOOT-009）

- Docker daemon 不可达：确认 `docker ps` 正常。
- 端口冲突：修改 `.env` 中端口后重启。
- 依赖未就绪：执行 `./scripts/wait-for-services.sh core`。

