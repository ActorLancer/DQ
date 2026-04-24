# TEST-005 Local Environment Smoke Fixture

本目录冻结 `TEST-005` 的本地环境 smoke 基线。

- `runtime-baseline.env`：固定 `smoke-local.sh` 使用的 compose profile、宿主机 `platform-core` 运行参数与 Kafka 双地址口径。
- `required-control-plane-endpoints.json`：固定本批必须真实回查的 Keycloak / Grafana / Prometheus / Alertmanager / Mock Payment / `platform-core` 控制面入口。

约束：

- Kafka topic 真值仍以 `infra/kafka/topics.v1.json` 为唯一 authority，本目录不重复维护 topic catalog。
- Grafana datasource / dashboard 真值仍以 `fixtures/local/observability-stack-manifest.json` 与 `infra/grafana/provisioning/**` 为准。
- Keycloak realm / 用户 / claim 真值仍以 `infra/keycloak/realm-export/platform-local-realm.json` 与 `fixtures/local/keycloak-realm-manifest.json` 为准。
