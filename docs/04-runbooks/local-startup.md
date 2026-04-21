# Local Startup（BOOT-009 / ENV-033）

## 阶段 1：基础设施

1. 使用默认本地环境文件：`infra/docker/.env.local`
   - 正式映射规则：
     - `POSTGRES_*`、`MINIO_ROOT_*`、`KEYCLOAK_ADMIN*` 只负责本地基础设施 bootstrap
     - `DATABASE_URL`、`MINIO_ENDPOINT / MINIO_ACCESS_KEY / MINIO_SECRET_KEY`、`KEYCLOAK_BASE_URL / KEYCLOAK_REALM` 才是应用与脚本正式运行时入口
2. 环境检查：`./scripts/check-local-env.sh infra/docker/docker-compose.local.yml infra/docker/.env.local infra/docker/.env.local`
3. 启动基础设施（默认 core）：`make up-local`（或 `make up-core`）
   - 仅支付/回执联调时，追加 `make up-mocks`（`core + mock-payment-provider`）
4. 等待依赖就绪：`./scripts/wait-for-services.sh core`
5. 基础设施健康检查：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
6. （可选）校验应用层占位编排文件：`docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml -f infra/docker/docker-compose.apps.local.example.yml config >/tmp/datab-compose-apps-config.yaml`
7. （可选）叠加应用层占位服务（仅联调参考，不替代本机进程启动）：
   `COMPOSE_PROFILES=core,apps docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml -f infra/docker/docker-compose.apps.local.example.yml up -d`

## 阶段 2：Schema / Migration

8. 数据库就绪检查：`./db/scripts/check-db-ready.sh`
9. 执行 migration 校验：`make migrate-up`
10. `platform-core` 仍以 `db/scripts/*` 和既有 SQL migration 作为正式 schema 入口；不要改用 `sqlx-cli migrate` 或 `sea-orm-cli migrate`
11. 如需刷新 SQLx 编译期校验元数据：`set -a; source infra/docker/.env.local; set +a; cargo sqlx prepare --workspace`

## 阶段 3：Seed

12. Kafka topics 会由 `make up-local` / `make up-core` / `make up-mocks` / `make up-demo` 通过 compose 内一次性 `kafka-topics-init` 自动初始化；如需手动重跑：`./infra/kafka/init-topics.sh`
13. 初始化 MinIO buckets：`./infra/minio/init-minio.sh`
14. 初始化 OpenSearch 索引：`./infra/opensearch/init-opensearch.sh`
15. 执行本地 seed：`make seed-local`
16. 准备五条标准链路演示数据：`fixtures/local/standard-scenarios-manifest.json` 与 `fixtures/local/standard-scenarios-sample.json`

补充说明：

- 当前本地启动默认仍初始化 `OpenSearch`，这是现阶段的默认联调路径，不代表 `SEARCHREC` 的 fallback 目标已经实现。
- 进入 `SEARCHREC-001 / SEARCHREC-004` 的后续实现批次后，正式目标应收敛为：`staging / production` 强制 `OpenSearch`，`local / demo` 允许只用 `PostgreSQL` 搜索投影运行，且最终仍由 `PostgreSQL` 做可见性校验。

## 阶段 4：应用

17. 启动主应用（platform-core）：
   `set -a; source infra/docker/.env.local; set +a; APP_PORT=8094 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p platform-core`
   - `platform-core` 和大部分本地脚本应统一从 `DATABASE_URL` 读取数据库入口，而不是直接读取 `POSTGRES_*`
18. 应用健康检查：
   `curl -fsS http://127.0.0.1:8094/health/live`
   `curl -fsS http://127.0.0.1:8094/health/ready`
19. 运行时数据访问职责：
   `SQLx` 负责连接池、事务、核心写路径和复杂 SQL；
   `SeaORM` 负责标准 CRUD、稳定读模型和固定关系加载
20. 按需叠加观测栈：`make up-observability`
21. 按需叠加 Fabric：`make up-fabric`
22. 一键演示模式（全量）：`make up-demo`
23. 支付/回执联调模式（`local` 子场景，非正式新 mode）：`make up-mocks`
   - 启动后可执行：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh mocks`

## 阶段 5：回执模拟

24. 验证 Keycloak realm：`./scripts/check-keycloak-realm.sh`
25. 启动 Fabric 本地链（按需）：`make fabric-up`
26. 生成本地通道与链码占位工件：`make fabric-channel && ./infra/fabric/deploy-chaincode-placeholder.sh`
27. Fabric 自检：`./scripts/check-fabric-local.sh`
28. OTel Collector 自检：`./scripts/check-otel-collector.sh`
29. 观测栈自检（仅 `make up-observability` 或 `make up-demo` 后执行）：`./scripts/check-observability-stack.sh`
30. 执行回执模拟（Mock Payment）：
   - 前置条件：已执行 `make up-mocks` 或 `make up-demo`
   - 检查命令：`./scripts/check-mock-payment.sh`
31. 全量健康检查（仅 `make up-demo` 后执行）：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`
    - 该检查包含端口与 HTTP 存活探测，以及命令级探测：`psql`、`redis-cli`、`kcat`（容器无 `kcat` 时优先临时 `kcat` 容器探测，再回退 `kafka-topics.sh`）、`mc`、`curl`。

## 阶段 6：配置快照与 Smoke

32. 导出当前本地配置快照：`./scripts/export-local-config.sh`
33. 运行本地 smoke 套件（建议在 `make up-demo` 后执行；若不用 `demo`，至少需要 `core + observability + mocks` 组合）：`ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`
    - 该 smoke 会按 `infra/kafka/topics.v1.json` 检查 canonical topics 是否真实存在，防止 auto-create 掩盖 topic 漂移。

## 迁移兼容说明（ENV-001 / ENV-057）

- 当前主编排入口：`infra/docker/docker-compose.local.yml`（配合根 `Makefile` 与 `scripts/`）。
- 旧目录 `部署脚本/docker-compose.local.yml` 视为历史兼容资产，不再作为主执行入口。
- 推荐命令：
  - 启动：`make up-local` / `make up-mocks` / `make up-demo`
  - 停止：`make down-local`
  - 健康检查：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core|full`
