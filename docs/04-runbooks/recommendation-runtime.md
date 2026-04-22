# Recommendation Runtime

## 运行边界

- 推荐编排当前实现在 `apps/platform-core/src/modules/recommendation/**`。
- `platform-core` 负责推荐请求、结果落库、曝光/点击入库与 canonical outbox。
- `recommendation-aggregator` 负责消费 `dtp.recommend.behavior`，更新信号聚合、关系边、搜索投影重刷和推荐缓存失效。
- `search-indexer` 继续负责消费 `dtp.search.sync`，将重刷后的搜索投影写入 OpenSearch。

## 本地启动

```bash
cargo run -p platform-core
KAFKA_BROKERS=127.0.0.1:9094 cargo run -p search-indexer
KAFKA_BROKERS=127.0.0.1:9094 cargo run -p recommendation-aggregator
```

注意：

- 宿主机运行 worker 时，Kafka 地址应使用 `127.0.0.1:9094`。
- `REDIS_URL` 默认推荐使用 DB `1`，避免与主会话缓存混用。

## 推荐主链路

1. `GET /api/v1/recommendations`
2. 读取 `recommend.placement_definition` / `recommend.ranking_profile`
3. OpenSearch 多路召回 + PG cohort/bundle/similarity 补充
4. PostgreSQL 最终业务校验
5. 写 `recommendation_request / recommendation_result / recommendation_result_item`
6. `POST /track/exposure` / `/track/click`
7. 写 `recommend.behavior_event`
8. canonical outbox 生成 `recommend.behavior_recorded`
9. topic 固定为 `dtp.recommend.behavior`
10. `recommendation-aggregator` 更新派生状态并失效缓存

## 鉴权与运维冻结口径

- 推荐侧正式鉴权统一使用 `Authorization`，不再使用 `x-role` 占位语义。
- `GET /api/v1/recommendations`、`POST /track/exposure`、`POST /track/click` 对应正式权限点为 `portal.recommendation.read`。
- `GET /api/v1/ops/recommendation/placements`、`GET /api/v1/ops/recommendation/ranking-profiles` 对应正式权限点为 `ops.recommendation.read`。
- `PATCH /api/v1/ops/recommendation/placements/{placement_code}`、`PATCH /api/v1/ops/recommendation/ranking-profiles/{id}` 对应正式权限点为 `ops.recommendation.manage`，并要求审计；高风险修改进入实现批次后应接入 `X-Step-Up-Token`。
- `POST /api/v1/ops/recommendation/rebuild` 对应正式权限点为 `ops.recommend_rebuild.execute`，并要求审计；是否强制 `step-up` 以后续实现批次按风险级别落地。
- 推荐行为写接口必须使用 `X-Idempotency-Key`，不得继续以“header 存在即可”的占位方式定义正式幂等语义。
- 当前 runbook 只冻结正式口径，不代表统一鉴权 / 审计 / OpenAPI / 测试已经补齐；进入 `SEARCHREC` 实现批次后，Agent 必须按 `A13` 同步修改代码与契约。
- `recommendation-aggregator` 已按 `AUD-026` 收口为正式 consumer：统一使用 envelope `event_id` 做幂等，写入 `ops.consumer_idempotency_record`，失败时先进入 `ops.dead_letter_event + dtp.dead-letter` 双层隔离，再决定 offset 提交。
- 当前若只验证推荐行为写库、outbox 行存在或缓存发生变化，仍不能视为 consumer 可靠性闭环完成；验收必须同时覆盖 worker 副作用、失败路径、双层 DLQ 与 `POST /api/v1/ops/dead-letters/{id}/reprocess` 的 `dry_run` 预演。

## 本地核对点

- `recommend.behavior_event.attrs ->> 'idempotency_key'` 可用于检查曝光/点击幂等。
- `ops.outbox_event.target_topic` 必须为 `dtp.recommend.behavior`。
- `ops.event_route_policy` 中 `recommend.behavior_event / recommend.behavior_recorded` 必须存在。
- `search.index_sync_task` 会因推荐行为热度更新而出现新的 `queued` 任务。
- Redis 推荐缓存前缀：`datab:v1:recommend:*`
- Redis 推荐已看集合前缀：`datab:v1:recommend:seen:*`
- SEARCHREC consumer 失败时，运维还应能在 `ops.dead_letter_event` 与 Kafka `dtp.dead-letter` 中看到对应隔离记录。

## 回归命令

```bash
RECOMMEND_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core recommendation_api_full_runtime_db_smoke -- --nocapture

SEARCHREC_WORKER_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
  cargo test -p recommendation-aggregator recommendation_aggregator_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture
```

说明：

- 第一条命令覆盖推荐请求、曝光、点击、ops 配置与 rebuild 主链路。
- 第二条命令覆盖 `dtp.recommend.behavior` 消费后的幂等记录、热度聚合、关系边增量、`search.index_sync_task` 回流、缓存失效，以及失败时写入 `ops.dead_letter_event + dtp.dead-letter`。
- 第三条命令覆盖 `recommendation-aggregator -> dtp.recommend.behavior` 的 SEARCHREC dead letter `dry_run` 重处理预演、`step-up` 绑定、`audit.audit_event`、`audit.access_audit` 与 `ops.system_log`。
- 当前 OpenSearch 推荐召回应对齐现有搜索索引映射：`status`、`seller_id`、`id` 为直接 keyword 字段，不应误写成 `status.keyword`、`seller_id.keyword`、`id.keyword`。
