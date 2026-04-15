# Local Startup（BOOT-009）

1. 使用默认本地环境文件：`infra/docker/.env.local`
2. 环境检查：`./scripts/check-local-env.sh infra/docker/docker-compose.local.yml infra/docker/.env.local infra/docker/.env.local`
3. 启动本地栈：`make up-local`
4. 数据库检查：`./db/scripts/check-db-ready.sh`
5. 初始化 Kafka topics：`./infra/kafka/init-topics.sh`
6. 初始化 MinIO：`./infra/minio/init-minio.sh`
7. 初始化 OpenSearch：`./infra/opensearch/init-opensearch.sh`
8. 健康检查：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
