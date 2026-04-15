# Troubleshooting（BOOT-009 / ENV-039）

## 通用排查入口

1. 查看容器状态：`docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'`
2. 查看关键日志：
   - `docker logs datab-postgres --tail 200`
   - `docker logs datab-kafka --tail 200`
   - `docker logs datab-keycloak --tail 200`
   - `docker logs datab-minio --tail 200`
   - `docker logs datab-opensearch --tail 200`
3. 重新做基线检测：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`

## PostgreSQL 启动失败

- 常见症状：`datab-postgres` 重启循环、`pg_isready` 超时、5432 端口不可达。
- 诊断步骤：
  1. 检查容器日志中是否有权限/配置报错：`docker logs datab-postgres --tail 200`
  2. 检查配置文件挂载：`infra/postgres/postgresql.conf`、`infra/postgres/pg_hba.conf`
  3. 检查卷状态：`docker volume ls | rg datab-local_postgres_data`
  4. 容器内验证：`docker exec datab-postgres pg_isready -U "${POSTGRES_USER:-datab}" -d "${POSTGRES_DB:-datab}"`
- 修复建议：
  - 端口冲突时调整 `POSTGRES_PORT`
  - 数据目录损坏时执行 `./scripts/prune-local.sh --force` 后重启

## Kafka 启动失败

- 常见症状：9092/9094 不可达、topic 初始化报 `connection refused`。
- 诊断步骤：
  1. 查看 broker 日志：`docker logs datab-kafka --tail 300`
  2. 检查 listener 配置：`KAFKA_ADVERTISED_LISTENERS` 与 `KAFKA_EXTERNAL_PORT` 是否一致
  3. 容器内探测：`docker exec datab-kafka /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list`
  4. 重新初始化 topic：`./infra/kafka/init-topics.sh`
- 修复建议：
  - 避免本机已有 Kafka 占用 9094
  - 若 broker 元数据异常，先 `make down-local` 后 `make up-local`

## Keycloak 启动失败

- 常见症状：8081 无响应、realm 未导入、管理员登录失败。
- 诊断步骤：
  1. 查看日志：`docker logs datab-keycloak --tail 300`
  2. 检查依赖 DB 健康：`docker ps | rg datab-postgres`
  3. 检查 realm 导入挂载：`infra/keycloak/realm-export`
  4. realm 校验：`./scripts/check-keycloak-realm.sh`
- 修复建议：
  - DB 凭据不一致时统一 `POSTGRES_*` 与 `KEYCLOAK_*`
  - realm JSON 损坏时恢复 `infra/keycloak/realm-export` 基线文件

## MinIO 启动失败

- 常见症状：9000 可达但 bucket 初始化失败、9001 控制台空白。
- 诊断步骤：
  1. 日志排查：`docker logs datab-minio --tail 200`
  2. 健康探测：`curl -fsS http://127.0.0.1:9000/minio/health/live`
  3. bucket 初始化：`./infra/minio/init-minio.sh`
  4. 使用 `mc` 探测：`docker run --rm --network host minio/mc:RELEASE.2025-08-13T08-35-41Z --help >/dev/null`
- 修复建议：
  - 确认 `MINIO_ROOT_USER/PASSWORD` 与 env 文件一致
  - bucket 不存在时先执行初始化脚本再运行 smoke

## OpenSearch 启动失败

- 常见症状：9200 端口无响应、集群健康一直 yellow/red。
- 诊断步骤：
  1. 查看日志：`docker logs datab-opensearch --tail 300`
  2. 基线探测：`curl -fsS http://127.0.0.1:9200`
  3. 检查内存参数：`OPENSEARCH_JAVA_OPTS`
  4. 索引初始化：`./infra/opensearch/init-opensearch.sh`
- 修复建议：
  - 内存不足时降低 `-Xms/-Xmx`
  - 数据卷损坏时通过 `prune-local --force` 重建

## Fabric 启动失败

- 常见症状：`fabric-ca/orderer/peer` 容器启动但状态异常，链路自检失败。
- 诊断步骤：
  1. 查看 fabric profile 容器：`docker ps -a | rg 'datab-fabric'`
  2. 检查状态目录：`find infra/fabric/state -maxdepth 2 -type d`
  3. 执行链路自检：`./scripts/check-fabric-local.sh`
  4. 重置 Fabric 状态：`make fabric-reset`
- 修复建议：
  - profile 未启用时使用 `make up-fabric` 或 `make up-demo`
  - 状态污染时执行 `make fabric-reset && make fabric-up`

## 资源安全清理

- 先预览：`./scripts/prune-local.sh --dry-run`
- 再执行：`./scripts/prune-local.sh --force`
- 该脚本仅清理当前 compose project 与 `infra/fabric/state`，避免误删其他项目容器。
