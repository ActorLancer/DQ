# Failure Drill Cases

`TEST-019` 的正式目标不是“停几个服务看看报错”，而是把故障注入、主链路退化、恢复与回查证据收口成一个可重复执行的 checker。

正式入口：

- 本地 / CI：`ENV_FILE=infra/docker/.env.local ./scripts/check-failure-drills.sh`

## 冻结边界

- `PostgreSQL` 是业务真值源；`Kafka` 只是异步事件总线。
- `OpenSearch` 不是搜索真值；`local / demo` 必须退化到 `PostgreSQL` 搜索投影，并继续保留 `Redis` 搜索短缓存。
- `fabric-adapter` 停机时不得伪造 `ops.external_fact_receipt`，只能形成真实 Kafka lag，恢复后再回补。
- `mock-payment-provider` 的 timeout 必须是真延迟，不允许把“立即返回一段 timeout JSON”冒充上游超时。

## 子场景矩阵

| Drill | 故障注入 | 主链路 | 正式断言 | 正式回查 |
| --- | --- | --- | --- | --- |
| `FDR-001` Kafka 停机 | `docker compose stop kafka` | `POST /api/v1/orders` | 订单仍创建成功，`current_state=created` | `/health/deps` 中 `kafka.reachable=false`；`trade.order_main.status=created`；`ops.outbox_event` 有正式事件 |
| `FDR-002` OpenSearch 不可用 | `docker compose stop opensearch` | `GET /api/v1/catalog/search` | 两次搜索都返回 `backend=postgresql`，且 `cache_hit=false -> true` | `docker compose ps opensearch`；两次正式 API 响应；`Redis` 搜索短缓存命中 |
| `FDR-003` Fabric Adapter 停机 | 停掉宿主机 `fabric-adapter` 进程 | `audit.anchor_requested -> dtp.audit.anchor` | 停机期间不写 `ops.external_fact_receipt`；恢复后补消费 | `kafka-consumer-groups.sh --describe` lag > 0；`ops.external_fact_receipt` 停机时 `0`、恢复后 `1` |
| `FDR-004` Mock Payment 延迟 | `POST /mock/payment/charge/timeout` | live mock payment smoke | timeout 接口真实延迟约 `15s` 且返回 `504`；`platform-core` 超时路径落到 `expired` | timeout 响应耗时；`bil004_mock_payment_adapter_db_smoke` live log |

## Checker 行为

`check-failure-drills.sh` 会按顺序执行：

1. 复用 `smoke-local.sh`、`seed-local-iam-test-identities.sh`、`seed-demo.sh`、`check-demo-seed.sh` 和 `check-mock-payment.sh` 建立正式基线。
2. 停掉 `kafka`，用真实 Keycloak Bearer 调 `POST /api/v1/orders`，再回查 `trade.order_main + ops.outbox_event`。
3. 清空搜索短缓存后停掉 `opensearch`，连续执行两次正式搜索 API，验证 `backend=postgresql` 与 `cache_hit=false -> true`。
4. 前两个 HTTP drill 完成后停止 host `platform-core`，先把唯一 consumer group 的 `dtp.audit.anchor / dtp.fabric.requests` offset reset 到当前 tail，再启动一次 `fabric-adapter` 做 warm-up；随后停机注入新的 `audit.anchor_requested` 事件，验证 lag 和 receipt 行为，并在恢复后确认 lag 归零。
5. 直接打 `mock-payment-provider` timeout 端点验证真实延迟，再执行 `bil004_mock_payment_adapter_db_smoke` 证明 `platform-core` 超时状态机落盘。
6. 将响应、group describe、compose 状态、live smoke 输出汇总到 `target/test-artifacts/failure-drills/summary.json`。

## 清理边界

- 业务测试对象会清理：
  - `trade.order_main / trade.order_line`
  - `ops.outbox_event`
  - `audit.anchor_batch / chain.chain_anchor`
  - `ops.external_fact_receipt / ops.consumer_idempotency_record`
- append-only 留痕不清理：
  - `audit.audit_event`
  - `ops.system_log`

## 失败定位

- `target/test-artifacts/failure-drills/test-019-platform-core.log`
- `target/test-artifacts/failure-drills/test-019-fabric-adapter.log`
- `target/test-artifacts/failure-drills/health-deps-kafka-down.json`
- `target/test-artifacts/failure-drills/fabric-group-down.txt`
- `target/test-artifacts/failure-drills/bil004-mock-payment-live.log`
- `target/test-artifacts/failure-drills/summary.json`
