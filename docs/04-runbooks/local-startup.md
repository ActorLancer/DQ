# Local Startup（BOOT-009 / ENV-033）

## 阶段 1：基础设施

1. 使用默认本地环境文件：`infra/docker/.env.local`
   - 正式映射规则：
     - `POSTGRES_*`、`MINIO_ROOT_*`、`KEYCLOAK_ADMIN*` 只负责本地基础设施 bootstrap
     - `DATABASE_URL`、`MINIO_ENDPOINT / MINIO_ACCESS_KEY / MINIO_SECRET_KEY`、`KEYCLOAK_BASE_URL / KEYCLOAK_REALM` 才是应用与脚本正式运行时入口
   - Kafka 默认对宿主机暴露 `127.0.0.1:9094`；如需同一局域网其他计算机访问，将 `KAFKA_EXTERNAL_ADVERTISED_HOST` 改成本机局域网 IP/DNS，并确认 `KAFKA_EXTERNAL_BIND_HOST=0.0.0.0`
2. 环境检查：`./scripts/check-local-env.sh infra/docker/docker-compose.local.yml infra/docker/.env.local infra/docker/.env.local`
3. 启动基础设施（默认 core）：`make up-local`（或 `make up-core`）
   - 仅支付/回执联调时，追加 `make up-mocks`（`core + mock-payment-provider`）
   - 启动脚本会先确保 `KEYCLOAK_DB_NAME` 对应的独立服务数据库存在，再拉起依赖服务，避免 `migrate-reset` 重建业务库后破坏 Keycloak
   - compose 默认会把容器内 `host.docker.internal` 固定到 Docker `host-gateway`，保证 Prometheus、Alertmanager、mock-payment-provider 等容器可稳定回连宿主机运行的 `platform-core` / `notification-worker` / callback 端口
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
    - `staging` 正式搜索链路必需；仅验证 `local / demo` 的 PostgreSQL 搜索投影 fallback 时可跳过
15. 执行本地 seed：`make seed-local`
16. 准备五条标准链路演示数据：
    - `fixtures/demo/manifest.json` 是 `TEST-001` 之后的正式 demo 数据包入口，供后续 `seed-demo.sh`、E2E 与验收矩阵复用
    - `fixtures/local/standard-scenarios-manifest.json` 与 `fixtures/local/standard-scenarios-sample.json` 继续保留为 `ENV-041` 本地 bootstrap 样例
    - `db/seeds/033_searchrec_recommendation_samples.sql` 会同步把五条官方场景商品写入 `catalog.*` 并固化到 `recommend.placement_definition(metadata.fixed_samples)`，使首页 `home_featured` 在演示环境中可直接作为五场景闭环入口
17. 导入正式 demo 订单、支付与交付对象：`./scripts/seed-demo.sh`
    - 默认会先执行 `db/scripts/seed-up.sh`，再按 `fixtures/demo/orders.json / billing.json / delivery.json` 写入 10 笔 demo 订单和对应支付 / 交付记录
    - 若基础 seed 已完成且只想重放 demo 订单链路，可使用：`./scripts/seed-demo.sh --skip-base-seeds`
    - 导入后执行：`./scripts/check-demo-seed.sh`

补充说明：

- 当前代码已按 `SEARCHREC-001` 收口搜索运行边界：`staging` 强制 `OpenSearch` 作为正式候选源，`local / demo` 允许走 `PostgreSQL` 搜索投影 fallback，且最终仍由 `PostgreSQL` 做可见性校验。
- 默认本地联调仍建议初始化 `OpenSearch`，因为搜索运维控制面、`search-indexer`、alias / rebuild / sync state 都依赖正式 `OpenSearch` 链路。
- 如果当前只验证 `local / demo` fallback，可跳过 `OpenSearch` 初始化和 `search-indexer` 启动；但 `Redis` 仍需可用以承载搜索短缓存。

## 阶段 4：应用

18. 启动主应用（platform-core）：
   `set -a; source infra/docker/.env.local; set +a; APP_MODE=staging APP_PORT=8094 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p platform-core`
   - 若验证 `local / demo` fallback，可改为 `APP_MODE=local` 或 `APP_MODE=demo`；此时 `platform-core` 不再要求 OpenSearch alias / index 在启动阶段已就绪
   - `platform-core` 和大部分本地脚本应统一从 `DATABASE_URL` 读取数据库入口，而不是直接读取 `POSTGRES_*`
19. 按需启动 canonical outbox publisher：
   `set -a; source infra/docker/.env.local; set +a; APP_PORT=8098 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p outbox-publisher`
   - `outbox-publisher` 轮询 `ops.outbox_event`，写入 `ops.outbox_publish_attempt`，并在重试耗尽时落 `ops.dead_letter_event + dtp.dead-letter`
20. 应用健康检查：
   `curl -fsS http://127.0.0.1:8094/health/live`
   `curl -fsS http://127.0.0.1:8094/health/ready`
   `curl -fsS http://127.0.0.1:8098/health/ready`
21. 运行时数据访问职责：
   `SQLx` 负责连接池、事务、核心写路径和复杂 SQL；
   `SeaORM` 负责标准 CRUD、稳定读模型和固定关系加载
22. 按需叠加观测栈：`make up-observability`
23. 按需叠加 Fabric：`make up-fabric`
   - 该命令会先启动 `core` 基础设施，再运行 `infra/fabric/fabric-up.sh`
   - Fabric 不再依赖本地 compose 内 placeholder 容器
24. 一键演示模式（全量）：`make up-demo`
25. 支付/回执联调模式（`local` 子场景，非正式新 mode）：`make up-mocks`
   - 启动后可执行：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh mocks`

## 阶段 5：回执模拟

26. 验证 Keycloak realm 与 password grant：`./scripts/check-keycloak-realm.sh`
   - 若 realm 被旧独立数据库污染，可执行：`make keycloak-reset-local`
27. 启动 Fabric 本地链（按需）：`make fabric-up`
28. 生成本地通道并部署真实 Go 链码：`make fabric-channel`
29. Fabric 自检：`./scripts/check-fabric-local.sh`
30. Fabric adapter 实链 smoke：`./scripts/fabric-adapter-live-smoke.sh`
31. OTel Collector 自检：`./scripts/check-otel-collector.sh`
32. 观测栈自检（仅 `make up-observability` 或 `make up-demo` 后执行）：`./scripts/check-observability-stack.sh`
33. 执行回执模拟（Mock Payment）：
   - 前置条件：已执行 `make up-mocks` 或 `make up-demo`
   - 检查命令：`./scripts/check-mock-payment.sh`
34. 全量健康检查（仅 `make up-demo` 后执行）：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`
    - 该检查包含端口与 HTTP 存活探测，以及命令级探测：`psql`、`redis-cli`、`kcat`（容器无 `kcat` 时优先临时 `kcat` 容器探测，再回退 `kafka-topics.sh`）、`mc`、`curl`。

## 阶段 6：配置快照与 Smoke

35. 导出当前本地配置快照：`./scripts/export-local-config.sh`
36. 运行本地 smoke 套件（建议在 `make up-demo` 后执行；若不用 `demo`，至少需要 `core + observability + mocks` 组合）：`ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`
    - 该 smoke 会按 `infra/kafka/topics.v1.json` 检查 canonical topics 是否真实存在，防止 auto-create 掩盖 topic 漂移。

## Kafka 局域网访问验证

如果已把 `KAFKA_EXTERNAL_ADVERTISED_HOST` 设置为本机局域网 IP/DNS，例如 `192.168.1.20`，重启后在本机验证端口监听：

```bash
docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml port kafka 9094
docker exec datab-kafka /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list
```

在另一台同局域网机器上验证：

```bash
kcat -b 192.168.1.20:9094 -L
```

如果远端 `kcat -L` 返回的 broker 地址仍是 `localhost:9094`，说明 `KAFKA_EXTERNAL_ADVERTISED_HOST` 没有改成局域网可达地址，需修改后重启 Kafka。

## 迁移兼容说明（ENV-001 / ENV-057）

- 当前主编排入口：`infra/docker/docker-compose.local.yml`（配合根 `Makefile` 与 `scripts/`）。
- 旧目录 `部署脚本/docker-compose.local.yml` 视为历史兼容资产，不再作为主执行入口。
- 推荐命令：
  - 启动：`make up-local` / `make up-mocks` / `make up-demo`
  - 停止：`make down-local`
  - 健康检查：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core|full`
