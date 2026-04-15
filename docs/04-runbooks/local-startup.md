# Local Startup（BOOT-009 / ENV-033）

## 阶段 1：基础设施

1. 使用默认本地环境文件：`infra/docker/.env.local`
2. 环境检查：`./scripts/check-local-env.sh infra/docker/docker-compose.local.yml infra/docker/.env.local infra/docker/.env.local`
3. 启动基础设施（默认 core）：`make up-local`（或 `make up-core`）
4. 基础设施健康检查：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`

## 阶段 2：Schema / Migration

5. 数据库就绪检查：`./db/scripts/check-db-ready.sh`
6. 执行 migration 校验：`make migrate-up`

## 阶段 3：Seed

7. 初始化 Kafka topics：`./infra/kafka/init-topics.sh`
8. 初始化 MinIO buckets：`./infra/minio/init-minio.sh`
9. 初始化 OpenSearch 索引：`./infra/opensearch/init-opensearch.sh`
10. 执行本地 seed：`make seed-local`

## 阶段 4：应用

11. 启动主应用（platform-core）：`cargo run -p platform-core`
12. 应用健康检查：`curl -fsS http://127.0.0.1:8080/healthz`
13. 按需叠加观测栈：`make up-observability`
14. 按需叠加 Fabric：`make up-fabric`
15. 一键演示模式（全量）：`make up-demo`

## 阶段 5：回执模拟

16. 验证 Keycloak realm：`./scripts/check-keycloak-realm.sh`
17. 启动 Fabric 本地链（按需）：`make fabric-up`
18. 生成本地通道与链码占位工件：`make fabric-channel && ./infra/fabric/deploy-chaincode-placeholder.sh`
19. Fabric 自检：`./scripts/check-fabric-local.sh`
20. OTel Collector 自检：`./scripts/check-otel-collector.sh`
21. 观测栈自检：`./scripts/check-observability-stack.sh`
22. 执行回执模拟（Mock Payment）：`./scripts/check-mock-payment.sh`
23. 全量健康检查：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`
    - 该检查包含端口与 HTTP 存活探测，以及命令级探测：`psql`、`redis-cli`、`kcat`（容器无 `kcat` 时优先临时 `kcat` 容器探测，再回退 `kafka-topics.sh`）、`mc`、`curl`。
