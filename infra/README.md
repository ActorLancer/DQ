# infra 目录校准（BOOT-034）

`infra/` 统一基础设施落位，当前冻结的子目录如下：

- `docker/`
- `fabric/`
- `keycloak/`
- `kafka/`
- `postgres/`
- `minio/`
- `opensearch/`
- `redis/`
- `prometheus/`
- `grafana/`
- `loki/`
- `tempo/`
- `otel/`

说明：

- 现有 `infra/monitoring`、`infra/k8s` 继续保留以兼容既有资产。
- 后续任务按依赖将配置逐步收敛到上述目录，不在本批次迁移文件内容。
