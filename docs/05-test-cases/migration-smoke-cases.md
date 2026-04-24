# Migration Smoke Cases

`TEST-004` 的正式入口是：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-migration-smoke.sh
```

## 覆盖目标

1. 启动当前正式本地 core stack：`infra/docker/docker-compose.local.yml` + `infra/docker/.env.local`
2. 初始化 MinIO 正式 buckets，避免 `platform-core` 启动前置条件缺失
3. 执行 `db/scripts/verify-db-compatibility.sh`
   - 空库升级
   - 全量 seed 导入
   - 回滚/重新升级
   - 最终 `migrate-status` 无 pending
4. 回查 `public.seed_history`，必须包含 `001/010/020/030/031/032/033`
5. 最终升级后真实启动 `platform-core-bin`
6. 回查正式健康端点与运行态：
   - `/health/live`
   - `/health/ready`
   - `/health/deps`
   - `/internal/runtime`

## 正式断言

- `platform-core` 启动模式固定为 `APP_MODE=local`
- provider 固定为 `PROVIDER_MODE=mock`
- `migration_version` 必须等于 `db/migrations/v1/manifest.csv` 当前最新版本
- `/health/deps` 至少验证 `db / redis / kafka / minio / keycloak` 为 `reachable=true`
- 宿主机 Kafka 边界继续使用 `127.0.0.1:9094`

## 失败定位

- `platform-core` 启动日志：`target/test-artifacts/test-004-platform-core.log`
- DB 回归链路失败时优先查看：
  - `db/scripts/verify-db-compatibility.sh`
  - `db/scripts/verify-migration-roundtrip.sh`
  - `db/scripts/migrate-status.sh`
