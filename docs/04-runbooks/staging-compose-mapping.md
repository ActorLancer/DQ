# Staging Compose Mapping (ENV-034)

## 目标

- 提供 `docker-compose.staging.example.yml` 的占位说明。
- 明确从 Compose 组件到 Helm/Kubernetes 资源的映射关系。
- 强调该文件仅用于演示与迁移对齐，不用于真实生产部署。

## 使用边界

- 文件路径：`/docker-compose.staging.example.yml`
- 仅允许占位值（`REPLACE_ME`），禁止写入真实密钥、真实云端地址、真实支付/链路凭据。
- 仅用于：
  - 组件清单对齐
  - 依赖关系梳理
  - K8s 迁移映射评审

## Compose -> Helm/K8s 映射

| Compose Service | K8s 目标资源 | 说明 |
| --- | --- | --- |
| `platform-core` | `Deployment` + `Service` + `ConfigMap` + `Secret` | 核心 API 服务；配置与凭据分离。 |
| `postgres` | `StatefulSet` + `PVC` + `Service` | 持久化数据库。 |
| `redis` | `StatefulSet` + `PVC` + `Service` | 缓存与会话存储。 |
| `kafka` | `StatefulSet` + `PVC` + `Service` | 事件总线。 |
| `minio` | `Deployment/StatefulSet` + `PVC` + `Service` + `Ingress` | 对象存储与控制台。 |
| `opensearch` | `StatefulSet` + `PVC` + `Service` | 检索索引。 |
| `keycloak` | `Deployment` + `Service` + `ConfigMap` + `Secret` | 身份与鉴权。 |
| `fabric-adapter` | `Deployment` + `Service` | 上链接入适配。 |
| `otel-collector` | `Deployment` + `ConfigMap` + `Service` | 遥测采集入口。 |
| `prometheus` | `Deployment` + `PVC` + `ConfigMap` + `Service` | 指标采集。 |
| `grafana` | `Deployment` + `PVC` + `ConfigMap` + `Service` + `Ingress` | 指标与日志看板。 |
| `loki` | `StatefulSet` + `PVC` + `Service` | 日志存储。 |
| `tempo` | `StatefulSet` + `PVC` + `Service` | Trace 存储。 |
| `mock-payment-provider` | `Deployment` + `Service` | 非生产 mock provider。 |

## 验证步骤

1. 静态解析：`docker compose -f docker-compose.staging.example.yml config`
2. 本地基础栈健康检查（等价验收）：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`

## 迁移注意事项

- 迁移到 Helm 时，所有 `REPLACE_ME` 必须改为 `Secret` 注入。
- `mock-payment-provider` 仅用于本地/测试环境；Staging/Prod 按 provider 边界切换真实实现。
- 网络策略、资源限制、探针阈值在 Helm values 中单独定义，不以此 Compose 占位文件作为生产配置来源。
