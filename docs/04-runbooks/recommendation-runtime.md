# Recommendation Runtime

## 运行边界

- 推荐编排当前实现在 `apps/platform-core/src/modules/recommendation/**`。
- `platform-core` 负责推荐请求、结果落库、曝光/点击入库与 canonical outbox。
- `recommendation-aggregator` 负责消费 `dtp.recommend.behavior`，更新信号聚合、关系边、搜索投影重刷和推荐缓存失效。
- `search-indexer` 继续负责消费 `dtp.search.sync`，将重刷后的搜索投影写入 OpenSearch。

`TEST-010` 对推荐读链的正式回归入口为 `ENV_FILE=infra/docker/.env.local ./scripts/check-searchrec-pg-authority.sh`。该 checker 会复用本 runbook 的推荐读取路径，并额外证明：即使候选召回仍能命中旧商品文档，只要 `PostgreSQL` 权威状态已变为 `frozen`，最终返回和新的 `recommendation_result_item` 都不能继续包含该商品。

## 本地启动

```bash
cargo run -p platform-core
KAFKA_BROKERS=127.0.0.1:9094 cargo run -p search-indexer
KAFKA_BROKERS=127.0.0.1:9094 \
REDIS_URL=redis://default:datab_redis_pass@127.0.0.1:6379/1 \
cargo run -p recommendation-aggregator
```

注意：

- 宿主机运行 worker 时，Kafka 地址应使用 `127.0.0.1:9094`。
- `recommendation-aggregator` 本地运行时，`REDIS_URL` 必须带上 Redis 鉴权信息；否则缓存失效步骤会因为 `NOAUTH` 失败，并把真实 `dtp.recommend.behavior` 事件送入 `ops.dead_letter_event + dtp.dead-letter`。
- `REDIS_URL` 默认推荐使用 DB `1`，避免与主会话缓存混用。

## 推荐主链路

1. `GET /api/v1/recommendations`
   - Bearer 鉴权 + `portal.recommendation.read`
   - `audit.access_audit(target_type='recommendation_result')`
   - `ops.system_log(message_text='recommendation lookup executed: GET /api/v1/recommendations')`
2. 读取 `recommend.placement_definition` / `recommend.ranking_profile`
3. `APP_MODE=staging` 走 OpenSearch 多路召回 + PG cohort/bundle/similarity 补充；`APP_MODE=local` 走 PostgreSQL 搜索投影驱动的最小候选策略（最新上架、同类目、同卖方、热销、零结果兜底）
4. PostgreSQL 最终业务校验
5. 写 `recommendation_request / recommendation_result / recommendation_result_item`
6. `POST /track/exposure` / `/track/click`
   - Bearer 鉴权 + `portal.recommendation.read`
   - 非空 `X-Idempotency-Key`
   - `audit.audit_event(action_name='recommendation.exposure.track' / 'recommendation.click.track')`
   - `audit.access_audit(target_type='recommendation_behavior')`
   - `ops.system_log(message_text='recommendation behavior tracked: POST ...')`
7. 写 `recommend.behavior_event`
8. canonical outbox 生成 `recommend.behavior_recorded`
9. topic 固定为 `dtp.recommend.behavior`
10. `recommendation-aggregator` 更新派生状态并失效缓存

## 基础模型基线

`SEARCHREC-008` 对推荐基础模型的冻结基线如下：

- 推荐位：
  - `home_featured`
  - `industry_featured`
  - `product_detail_similar`
  - `product_detail_bundle`
  - `seller_profile_featured`
  - `buyer_workbench_discovery`
  - `search_zero_result_fallback`
- 默认排序配置：
  - `recommend_v1_default`
  - `recommend_v1_detail`
  - `recommend_v1_bundle`
  - `recommend_v1_seller`
- 首批正式行为事件：
  - `recommendation_panel_viewed`
  - `recommendation_item_exposed`
  - `recommendation_item_clicked`
- `SEARCHREC-014` 首页固定样例：
  - `home_featured` 的 `recommend.placement_definition.metadata.fixed_samples` 固化五条标准链路官方商品样例
  - 排序顺序固定为 `S1 -> S2 -> S3 -> S4 -> S5`
  - 响应 `explanation_codes` 中应能回查 `placement:fixed_sample` 与 `scenario:S1..S5`

基础模型回查：

```sql
SELECT placement_code,
       placement_scope,
       page_context,
       default_ranking_profile_key,
       candidate_policy_json -> 'recall' AS recall_sources
FROM recommend.placement_definition
ORDER BY placement_code;
```

```sql
SELECT profile_key,
       placement_scope,
       status
FROM recommend.ranking_profile
ORDER BY profile_key;
```

```sql
SELECT target_topic,
       consumer_group_hint
FROM ops.event_route_policy
WHERE aggregate_type = 'recommend.behavior_event'
  AND event_type = 'recommend.behavior_recorded';
```

## 鉴权与运维冻结口径

- 推荐侧正式鉴权统一使用 `Authorization`，不再使用 `x-role` 占位语义。
- `GET /api/v1/recommendations`、`POST /track/exposure`、`POST /track/click` 对应正式权限点为 `portal.recommendation.read`。
- `GET /api/v1/ops/recommendation/placements`、`GET /api/v1/ops/recommendation/ranking-profiles` 对应正式权限点为 `ops.recommendation.read`，并要求统一 Bearer 鉴权；成功读取后必须写入 `audit.access_audit(target_type='recommendation_placement' / 'recommendation_ranking_profile') + ops.system_log`。
- `PATCH /api/v1/ops/recommendation/placements/{placement_code}`、`PATCH /api/v1/ops/recommendation/ranking-profiles/{id}` 对应正式权限点为 `ops.recommendation.manage`，并要求审计。
- `PATCH /api/v1/ops/recommendation/placements/{placement_code}` 已收口为 `Authorization + X-Idempotency-Key + X-Step-Up-Token`，真实校验 `iam.step_up_challenge(target_action='recommendation.placement.patch', target_ref_type='recommendation_placement')`，并写入 `audit.audit_event(action_name='recommendation.placement.patch') + audit.access_audit + ops.system_log`。
- `PATCH /api/v1/ops/recommendation/ranking-profiles/{id}` 已收口为 `Authorization + X-Idempotency-Key + X-Step-Up-Token`，真实校验 `iam.step_up_challenge(target_action='recommendation.ranking_profile.patch', target_ref_type='recommendation_ranking_profile', target_ref_id=<ranking_profile_id>)`，并写入 `audit.audit_event(action_name='recommendation.ranking_profile.patch') + audit.access_audit(target_type='recommendation_ranking_profile') + ops.system_log`。
- `POST /api/v1/ops/recommendation/rebuild` 对应正式权限点为 `ops.recommend_rebuild.execute`，属于高风险动作，必须要求 `Authorization + X-Idempotency-Key + X-Step-Up-Token`；`step-up` 必须真实绑定 `iam.step_up_challenge(target_action='recommendation.rebuild.execute', target_ref_type='recommendation_rebuild')`，并写入 `audit.audit_event(action_name='recommendation.rebuild.execute') + audit.access_audit(target_type='recommendation_rebuild') + ops.system_log`。
- 推荐行为写接口必须使用 `X-Idempotency-Key`，不得继续以“header 存在即可”的占位方式定义正式幂等语义。
- 当前 runbook 只冻结正式口径，不代表统一鉴权 / 审计 / OpenAPI / 测试已经补齐；进入 `SEARCHREC` 实现批次后，Agent 必须按 `A13` 同步修改代码与契约。
- `recommendation-aggregator` 已按 `AUD-026 + SEARCHREC-020` 收口为正式 consumer：统一使用 envelope `event_id` 做幂等，写入 `ops.consumer_idempotency_record`，失败时先进入 `ops.dead_letter_event + dtp.dead-letter` 双层隔离，再决定 offset 提交。
- 当前若只验证推荐行为写库、outbox 行存在或缓存发生变化，仍不能视为 consumer 可靠性闭环完成；验收必须同时覆盖 worker 副作用、失败路径、双层 DLQ 与 `POST /api/v1/ops/dead-letters/{id}/reprocess` 的 `dry_run` 预演。

## 本地核对点

- `recommend.behavior_event.attrs ->> 'idempotency_key'` 可用于检查曝光/点击幂等。
- `ops.outbox_event.target_topic` 必须为 `dtp.recommend.behavior`。
- `ops.event_route_policy` 中 `recommend.behavior_event / recommend.behavior_recorded` 必须存在。
- 推荐位补丁生效后，`recommend.placement_definition.default_ranking_profile_key / metadata` 必须已更新，且 `datab:v1:recommend:*` 与 `datab:v1:recommend:seen:*:{placement_code}` 相关 Redis key 应被失效。
- `recommend.placement_definition.default_ranking_profile_key` 必须能在 `recommend.ranking_profile.profile_key` 中解析到有效配置。
- `GET /api/v1/recommendations` 后必须能在 `audit.access_audit` 中回查 `target_type='recommendation_result'`，并在 `ops.system_log` 中回查 `recommendation lookup executed: GET /api/v1/recommendations`。
- `GET /api/v1/ops/recommendation/ranking-profiles` 后必须能在 `audit.access_audit` 中回查 `target_type='recommendation_ranking_profile'`，并在 `ops.system_log` 中回查 `recommendation ops lookup executed: GET /api/v1/ops/recommendation/ranking-profiles`。
- `PATCH /api/v1/ops/recommendation/ranking-profiles/{id}` 后必须能在 `audit.audit_event` 中回查 `recommendation.ranking_profile.patch`，在 `audit.access_audit` 中回查 `target_type='recommendation_ranking_profile'` 和 `step_up_challenge_id`，并在 `ops.system_log` 中回查 `recommendation ops action executed: PATCH /api/v1/ops/recommendation/ranking-profiles/{id}`。
- `APP_MODE=local` 下，`recommend.recommendation_request.request_attrs ->> 'candidate_backend'` 与 `recommend.recommendation_result.metadata ->> 'candidate_backend'` 必须为 `postgresql_local_minimal`，且同一查询两次命中应观察到 `cache_hit=false -> true`。
- `home_featured` 在演示种子完成后必须能稳定返回五条标准链路官方商品：`工业设备运行指标 API 订阅 / 工业质量与产线日报文件包交付 / 供应链协同查询沙箱 / 零售门店经营分析 API / 报告订阅 / 商圈/门店选址查询服务`。
- `recommend.placement_definition.metadata ->> 'fixed_sample_set'` 必须为 `five_standard_scenarios_v1`，且 `jsonb_array_length(metadata -> 'fixed_samples') = 5`。
- `recommend.recommendation_request.candidate_source_summary ->> 'placement_sample'` 在首页五场景样例请求中必须为 `5`。
- `placement_code=search_zero_result_fallback` 在 `APP_MODE=local` 下必须返回非空候选，且至少一个 `recommendation_result_item.feature_snapshot -> 'recall_sources'` 或响应 `explanation_codes` 能证明 `fallback:zero_result` 已参与兜底。
- `POST /api/v1/recommendations/track/exposure`、`POST /api/v1/recommendations/track/click` 后必须能在 `audit.audit_event` 中回查 `recommendation.exposure.track / recommendation.click.track`，在 `audit.access_audit` 中回查 `target_type='recommendation_behavior'`，并在 `ops.system_log` 中回查 `recommendation behavior tracked: POST ...`。
- `POST /api/v1/ops/recommendation/rebuild` 后必须能在 `audit.audit_event` 中回查 `recommendation.rebuild.execute`，在 `audit.access_audit` 中回查 `target_type='recommendation_rebuild'` 和 `step_up_challenge_id`，并在 `ops.system_log` 中回查 `recommendation ops action executed: POST /api/v1/ops/recommendation/rebuild`。
- 推荐重建触发 `scope=all/features/subject_profile/cohort/signals/similarity/bundle` 时，必须真实重刷 `recommend.subject_profile_snapshot`、`recommend.cohort_popularity`、`search.search_signal_aggregate`、`recommend.entity_similarity`、`recommend.bundle_relation` 中对应派生表；`scope=cache` 或 `purge_cache=true` 时，必须真实删除 `datab:v1:recommend:*` 与命中的 `datab:v1:recommend:seen:*`。
- `recommend.behavior_event` 写入 `recommendation_item_exposed / recommendation_item_clicked` 后，`recommend.subject_profile_snapshot` 与 `recommend.cohort_popularity` 必须同步更新。
- `search.index_sync_task` 会因推荐行为热度更新而出现新的 `queued` 任务。
- Redis 推荐缓存前缀：`datab:v1:recommend:*`
- Redis 推荐已看集合前缀：`datab:v1:recommend:seen:*`
- SEARCHREC consumer 失败时，运维还应能在 `ops.dead_letter_event` 与 Kafka `dtp.dead-letter` 中看到对应隔离记录。

## 回归命令

```bash
RECOMMEND_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  APP_MODE=staging \
  cargo test -p platform-core recommendation_api_full_runtime_db_smoke -- --nocapture

RECOMMEND_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  APP_MODE=local OPENSEARCH_ENDPOINT=http://127.0.0.1:1 \
  cargo test -p platform-core recommendation_local_minimal_candidate_db_smoke -- --nocapture

RECOMMEND_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  APP_MODE=staging \
  cargo test -p platform-core recommendation_home_featured_standard_scenarios_db_smoke -- --nocapture

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
