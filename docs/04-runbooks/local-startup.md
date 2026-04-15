# Local Startup（BOOT-009）

1. 使用默认本地环境文件：`infra/docker/.env.local`
2. 环境检查：`./scripts/check-local-env.sh infra/docker/docker-compose.local.yml infra/docker/.env.local infra/docker/.env.local`
3. 启动本地栈（默认 `core` profile）：`make up-local`（或显式使用 `make up-core`）
4. 数据库检查：`./db/scripts/check-db-ready.sh`
5. 初始化 Kafka topics：`./infra/kafka/init-topics.sh`
6. 初始化 MinIO：`./infra/minio/init-minio.sh`
7. 初始化 OpenSearch：`./infra/opensearch/init-opensearch.sh`
8. 验证 Keycloak realm：`./scripts/check-keycloak-realm.sh`
9. 验证 Mock Payment 场景：`./scripts/check-mock-payment.sh`
10. 启动 Fabric 本地链（按需）：`make fabric-up`
11. 生成本地通道与链码占位工件：`make fabric-channel && ./infra/fabric/deploy-chaincode-placeholder.sh`
12. Fabric 自检：`./scripts/check-fabric-local.sh`
13. OTel Collector 自检：`./scripts/check-otel-collector.sh`
14. 按需叠加观测栈：`make up-observability`
15. 按需叠加 Fabric：`make up-fabric`
16. 一键演示模式（全量）：`make up-demo`
17. 观测栈自检：`./scripts/check-observability-stack.sh`
18. 健康检查：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`
    - 该检查包含端口与 HTTP 存活探测，以及命令级探测：`psql`、`redis-cli`、`kcat`（无 `kcat` 时回退 `kafka-topics.sh`）、`mc`、`curl`。
