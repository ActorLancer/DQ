# 当前本地部署与运维资产盘点（CTX-023）

## 1. 盘点对象

- `部署脚本/docker-compose.local.yml`
- 数据库与环境校验脚本
- Mock Provider
- Fabric 测试链相关资产
- 本地观测栈资产

## 2. 资产状态（exists / partial / missing）

| 资产 | 状态 | 说明 |
| --- | --- | --- |
| `部署脚本/docker-compose.local.yml` | exists | 已编排 `postgres/redis/kafka/minio/opensearch/keycloak`，并含 observability 与 mocks profile。 |
| `部署脚本/docker-compose.postgres-test.yml` | exists | 可用于数据库迁移测试。 |
| `scripts/check-local-env.sh` | exists | 可检查 compose、env、docker daemon 可用性。 |
| `scripts/verify-local-stack.sh` | exists | 可按 core/obs/mocks/full 执行运行态探活。 |
| `scripts/validate_database_migrations.sh` | exists | 数据库迁移校验入口。 |
| `mock-payment-provider`（compose profile） | partial | 服务容器已接入，但回调场景编排与 runbook 需继续补齐。 |
| `infra/docker/monitoring/prometheus.yml` | exists | 观测配置已存在。 |
| `infra/docker/monitoring/loki-config.yml` | exists | 观测配置已存在。 |
| `infra/docker/monitoring/tempo.yml` | exists | 观测配置已存在。 |
| `Alertmanager` 服务编排 | missing | 当前 `docker-compose.local.yml` 未见 `alertmanager` 服务。 |
| `otel-collector` 服务编排 | missing | 当前 `docker-compose.local.yml` 未见 `otel-collector`。 |
| `Fabric` 测试网络启动脚本 | partial | `infra/fabric/` 目录存在，但未见可执行启动脚本与 compose 集成。 |
| `Keycloak realm import` 固化资产 | partial | `keycloak` 服务存在，realm 导入资产待补齐。 |
| `Kafka topic 初始化脚本` | missing | 未发现本地 topic 初始化脚本。 |
| `MinIO bucket 初始化脚本` | missing | 未发现 bucket 初始化脚本。 |

## 3. 与冻结环境清单的一致性结论

1. 核心中间件本地编排已具备可运行基础，满足“先跑通 V1 核心链路”的最低条件。
2. 运维完整性仍有缺口：`alertmanager`、`otel-collector`、topic/bucket 初始化、Fabric 网络脚本化能力尚未收口。
3. 后续 ENV 任务应优先在现有 compose 与 scripts 基础上补齐缺口，而非替换整套本地栈。
