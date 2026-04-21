# Search Reindex（BOOT-009）

- 重建索引需审计留痕。
- 别名切换前先验证新索引数据完整性。

## 本地命令

初始化 OpenSearch：

```bash
./infra/opensearch/init-opensearch.sh
```

启动 `search-indexer`：

```bash
KAFKA_BROKERS=127.0.0.1:9094 cargo run -p search-indexer
```

单实体重建：

```bash
curl -sS -X POST http://127.0.0.1:8080/api/v1/ops/search/reindex \
  -H 'content-type: application/json' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-reindex-local' \
  -H 'X-Idempotency-Key: idem-search-reindex-local' \
  -H 'X-Step-Up-Token: search-reindex-stepup' \
  -d '{
    "entity_scope":"product",
    "entity_id":"<product_uuid>",
    "mode":"single",
    "force":true
  }'
```

查看同步任务：

```bash
curl -sS http://127.0.0.1:8080/api/v1/ops/search/sync \
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-sync-local'
```

## 当前 V1 口径

- 正式 topic：`dtp.search.sync`
- 正式 worker：`workers/search-indexer`
- 正式鉴权：统一使用 `Authorization`，不再使用 `x-role`
- 正式写接口头：`X-Idempotency-Key`
- 高风险运维写接口：`POST /reindex`、`POST /aliases/switch`、`PATCH /ranking-profiles/{id}` 需要 `X-Step-Up-Token`
- 正式权限点：`ops.search_sync.read`、`ops.search_reindex.execute`、`ops.search_alias.manage`、`ops.search_cache.invalidate`、`ops.search_ranking.read`、`ops.search_ranking.manage`
- 写操作必须保留审计；搜索域错误码按接口协议收敛到 `SEARCH_*`
- 宿主机直连 Kafka 时使用 `127.0.0.1:9094`；容器内监听地址 `kafka:9092` 只供 compose 网络内部使用
- 正式缓存 key 前缀：`datab:v1:search:catalog:*`
- 正式 alias：`product_search_read/write`、`seller_search_read/write`
- 当前 runbook 只冻结正式口径，不代表统一鉴权 / 审计 / 错误码已经在运行时代码全部实现；进入 `SEARCHREC` / `AUD` 实现批次后，Agent 必须按 `A13` 同步修改代码、OpenAPI 与测试。
- `search-indexer` 的正式 consumer 口径还必须收敛为：统一 envelope `event_id` 幂等、`ops.consumer_idempotency_record`、失败进入 `ops.dead_letter_event + dtp.dead-letter` 双层隔离，并且只有在成功处理或失败已安全隔离后才允许提交 offset。
- 当前若仅能证明“topic 被消费过”或“OpenSearch 有结果”，仍不能视为 SEARCHREC consumer 可靠性已完成；进入实现批次后必须按 `A15` 补齐 worker 失败路径与 reprocess 验证。
- alias 权威源与切换边界当前已冻结为 `search.index_alias_binding + product/seller_search_read/write`；进入 `SEARCHREC-021` 实现批次后，初始化脚本、运行时默认值、ops 接口与 runbook 必须继续按同一套 alias 答案收口。
