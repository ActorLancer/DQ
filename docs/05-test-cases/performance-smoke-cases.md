# TEST-018 Performance Smoke 正式清单

## 目标

- 将 `TEST-018` 收口为单一正式入口，持续拦截四条关键链路的明显性能回退：
  - `GET /api/v1/catalog/search`
  - `POST /api/v1/orders`
  - `POST /api/v1/orders/{id}/deliver`
  - `GET /api/v1/audit/orders/{id}`
- 本地与 CI 统一复用 `ENV_FILE=infra/docker/.env.local ./scripts/check-performance-smoke.sh`，不在 workflow 或 runbook 中再散落第二套性能 smoke 命令。
- 性能门槛采用冻结文档 `26.2 性能 SLO` 的基础口径：`标准下单 / 合同查看 / 账单查询 p95 <= 2 秒`；当前 smoke 以“单次正式 API <= 2.0 秒”作为明显回退守门线，而不是容量压测。

## 正式入口

- 本地 / CI 统一入口：
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-performance-smoke.sh`
- CI workflow：
  - `.github/workflows/performance-smoke.yml`

## 收口路径

| 阶段 | 正式命令 | 验收重点 |
| --- | --- | --- |
| Local stack / observability 前置 | `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh` | 必须真实拉起 `core + observability + mocks`，并继续校验 Keycloak realm、Prometheus / Grafana、canonical topic 与宿主机 / 容器 Kafka 双地址边界。 |
| Demo / IAM 基线 | `./scripts/seed-local-iam-test-identities.sh` + `./scripts/seed-demo.sh --skip-base-seeds` + `./scripts/check-demo-seed.sh` | 性能 smoke 必须运行在五条标准链路的正式 demo 商品与正式 IAM claim 上，不允许绕过 Keycloak 或使用临时硬编码用户。 |
| 真实业务链路 | `GET /api/v1/catalog/search` -> `POST /api/v1/orders` -> `POST /api/v1/orders/{id}/contract-confirm` -> `POST /api/v1/orders/{id}/api-sub/transition` -> `POST /api/v1/orders/{id}/deliver` -> `GET /api/v1/audit/orders/{id}` | `deliver` 不是伪请求，必须建立真实订单、合同、锁资和 `api_endpoint` 绑定后再执行；审计联查必须回到正式 audit API。 |
| Observability 证据 | `summary.json` + Prometheus query artifact + `/metrics` snapshot + `platform-core` request log | 不能只看 `HTTP 200`；必须留下实际耗时、请求 ID、Prometheus `up{job=\"platform-core\"}`、四条业务 API 的 HTTP duration 指标与应用 request log。 |
| 清理 | checker `EXIT` cleanup | 允许写入临时订单 / 合同 / 交付 / 应用 / `api_endpoint` 绑定；执行后必须清理业务测试数据，但审计 append-only 记录保留。 |

## 验收矩阵

| Case ID | 场景 | 必须证明的事实 | 最低回查 |
| --- | --- | --- | --- |
| `TEST018-CASE-001` | Search 单次性能守门 | 正式 Keycloak buyer token 下，`GET /api/v1/catalog/search` 返回正式 envelope、命中 demo 商品，并且 `time_total <= 2.0s`。 | `target/test-artifacts/performance-smoke/search-response.json`、`search.time.txt` |
| `TEST018-CASE-002` | Order create 单次性能守门 | 基于 `fixtures/demo/orders.json` 的 `S1 primary` 正式蓝图创建订单成功，`current_state == created`，且 `time_total <= 2.0s`。 | `order-create-response.json`、`order-create.time.txt` |
| `TEST018-CASE-003` | Delivery 单次性能守门 | 订单经历 `contract-confirm + api-sub lock_funds` 后，`deliver` 能真实签发 API credential，且 `time_total <= 2.0s`。 | `contract-confirm-response.json`、`api-sub-lock-response.json`、`delivery-response.json`、`delivery.time.txt` |
| `TEST018-CASE-004` | Audit 联查单次性能守门 | `GET /api/v1/audit/orders/{id}` 能回查刚创建订单的真实 trace，且 `time_total <= 2.0s`。 | `audit-order-response.json`、`audit-order.time.txt` |
| `TEST018-CASE-005` | Observability 证据完备 | Prometheus 能报告 `platform-core` target up，`/metrics` 中可见四条业务 API 的 `platform_core_http_request_duration_seconds_count`，应用日志保留相同 `request_id` 的 request finished 记录。 | `prometheus-platform-core-up.json`、`platform-core-metrics.prom`、`test-018-platform-core.log` |
| `TEST018-CASE-006` | 失败可定位 | checker 和 workflow 会保留原始请求/响应、计时、metrics、Prometheus query 与应用日志，便于判断是搜索、下单、交付、审计还是观测层回退。 | `.github/workflows/performance-smoke.yml` artifact |

## 边界

- `TEST-018` 是基础性能冒烟，不替代压力测试、容量评估或长期趋势监控。
- 当前门槛统一采用 `2.0s`，用于拦截“明显退化”；它不是最终 p95 报表，也不承诺代表高并发表现。
- `TEST-018` 不替代 `TEST-005` 本地环境 smoke、`TEST-006` 五条标准链路 E2E、`TEST-009` 审计完备性、`TEST-010` 搜索 / 推荐 PG 权威或 `TEST-028` canonical checker；它只负责四条正式 API 的基础性能回退守门与观测证据留痕。
