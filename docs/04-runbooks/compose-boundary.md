# Compose Boundary（ENV-042 / ENV-001）

## 目标

- 明确 `infra/docker/docker-compose.local.yml` 的责任边界。
- 固化可选 profile、端口矩阵与中间件服务清单。
- 避免把业务应用进程（`platform-core` 等）直接塞进基础设施 compose。

## 责任范围

`docker-compose.local.yml` 只负责本地基础设施与外围支撑组件：

- `postgres`
- `redis`
- `kafka`
- `kafka-topics-init`（一次性 topic 初始化）
- `minio`
- `opensearch`
- `keycloak`
- `otel-collector`
- `prometheus`
- `alertmanager`
- `grafana`
- `loki`
- `tempo`
- `mock-payment-provider`
- `fabric-ca`
- `fabric-orderer`
- `fabric-peer`

## Profile 约束

- `core`：`postgres/redis/kafka/minio/opensearch/keycloak/otel-collector`
- `kafka-topics-init` 作为 compose 内定义的一次性 init service，由 `scripts/up-local.sh` 在 `core/demo` 启动后显式调用，不属于业务常驻进程。
- `observability`：`prometheus/alertmanager/*-exporter/grafana/loki/tempo`
- `fabric`：`fabric-ca/fabric-orderer/fabric-peer`
- `demo`：全量联调集合（覆盖 core + observability + fabric + mock-payment）
- `mocks`：`mock-payment-provider` 单独可选

## 端口矩阵（主入口）

详细矩阵见 `docs/04-runbooks/port-matrix.md`，关键端口：

- Core：`5432/6379/9094/9000/9200/8081/4317/4318`
- Observability：`9090/9093/3000/3100/3200`
- Fabric：`7054/7050/7051/7053`
- Mock Payment：`8089`

## 不纳入 compose 的业务进程

以下进程不属于基础设施编排，默认以源码进程或后续专用编排启动：

- `platform-core`
- `fabric-adapter`
- `notification-worker`
- `outbox-publisher`
- `search-indexer`

如需容器化联调，使用后续独立占位文件（见 `ENV-043` 任务约束），不得污染基础设施 compose 主文件。

## 启动顺序与前置动作

1. `make up-local` / `make up-demo`
2. `./scripts/wait-for-services.sh core|full`
3. Seed 初始化：
   - Kafka topics 默认由 compose 内一次性 `kafka-topics-init` 自动初始化；如需手动重跑：`./infra/kafka/init-topics.sh`
   - `./infra/minio/init-minio.sh`
   - `./infra/opensearch/init-opensearch.sh`
4. 健康检查：
   - `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core|full`

## 旧资产兼容边界（ENV-001）

- 历史文件：`部署脚本/docker-compose.local.yml`
- 当前主入口：`infra/docker/docker-compose.local.yml`
- 兼容策略：保留历史文件用于追溯，不再作为默认执行入口；新任务统一走根 `Makefile` + `scripts/` + `infra/docker/` 结构。
