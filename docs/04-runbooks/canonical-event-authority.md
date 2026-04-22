# Canonical Event Authority（AUD-030）

`AUD-030` 的正式结论是：

- 应用层唯一正式事件生产入口：`apps/platform-core/src/shared/outbox.rs::write_canonical_outbox_event(...)`
- 运行时唯一正式 route authority：`ops.event_route_policy`
- `common.tg_write_outbox()` 已退役为异常函数
- 正式业务表上不再允许挂依赖 `tg_write_outbox` 的 trigger

## 最小前置

```bash
set -a
source infra/docker/.env.local
set +a
```

## 静态回查

1. 确认旧 trigger 已退役：

```sql
SELECT event_object_schema, event_object_table, trigger_name
FROM information_schema.triggers
WHERE action_statement ILIKE '%tg_write_outbox%'
ORDER BY event_object_schema, event_object_table, trigger_name;
```

预期：

- 结果为 `0 rows`

2. 确认退役函数不是旁路生产入口：

```sql
SELECT pg_get_functiondef(p.oid)
FROM pg_proc p
JOIN pg_namespace n ON n.oid = p.pronamespace
WHERE n.nspname = 'common'
  AND p.proname = 'tg_write_outbox';
```

预期：

- 函数体直接 `RAISE EXCEPTION`
- 错误文案包含 `ops.event_route_policy + application canonical outbox writer`

3. 确认关键 route policy 激活：

```sql
SELECT aggregate_type,
       event_type,
       target_topic,
       authority_scope,
       proof_commit_policy,
       status
FROM ops.event_route_policy
WHERE (aggregate_type, event_type) IN (
  ('trade.order', 'trade.order.created'),
  ('product', 'search.product.changed'),
  ('delivery.delivery_record', 'delivery.committed'),
  ('delivery.delivery_record', 'delivery.task.auto_created'),
  ('trade.order_main', 'billing.trigger.bridge'),
  ('audit.anchor_batch', 'audit.anchor_requested')
)
ORDER BY aggregate_type, event_type;
```

预期：

- 所有记录 `status='active'`
- `target_topic` 与当前 smoke 断言一致

## 运行态 smoke

执行：

```bash
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture

DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
cargo test -p platform-core cat022_search_visibility_fields_and_events_db_smoke -- --nocapture

DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
cargo test -p platform-core dlv002_file_delivery_commit_db_smoke -- --nocapture

DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
cargo test -p platform-core dlv029_delivery_task_autocreation_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
cargo test -p platform-core audit_trace_api_db_smoke -- --nocapture
```

预期统一成立：

- 同一业务动作、同一 `(aggregate_type, event_type, request_id)` 只写出一条正式 `ops.outbox_event`
- `payload` 顶层包含 `event_id / event_type / event_version / occurred_at / producer_service / aggregate_type / aggregate_id / request_id / event_schema_version / authority_scope / source_of_truth / proof_commit_policy / payload`
- `payload` 顶层不再包含 `event_name`
- `target_topic` 与 `ops.event_route_policy` 命中结果一致

## 覆盖场景

- `trade003_create_order_db_smoke`
  - 验证 `trade.order.created -> dtp.outbox.domain-events`
- `cat022_search_visibility_fields_and_events_db_smoke`
  - 验证 `search.product.changed -> dtp.search.sync`
- `dlv002_file_delivery_commit_db_smoke`
  - 验证 `delivery.committed` 与 `billing.trigger.bridge` 都走 canonical route，且每种事件各只写一条
- `dlv029_delivery_task_autocreation_db_smoke`
  - 验证 `delivery.task.auto_created -> dtp.outbox.domain-events`
- `audit_trace_api_db_smoke`
  - 验证 `audit.anchor_requested -> dtp.audit.anchor`

## 排障边界

- 若 `ops.outbox_event` 顶层仍出现 `event_name`，优先检查是否有模块绕过 `shared/outbox.rs`
- 若 `target_topic` 与 route policy 不一致，先回查 `ops.event_route_policy` 是否缺 seed 或被错误改写
- 若看到双写，先查当前业务对象是否仍挂了遗留 trigger，再查对应模块是否手工插入了第二条 outbox
