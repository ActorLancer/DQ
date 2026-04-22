# Canonical Event Authority Cases（AUD-030）

`AUD-030` 的验收目标不是“能写出事件”，而是证明：

- canonical envelope 是唯一正式协议
- `ops.event_route_policy` 是唯一 route authority
- 同一业务动作不会重复写出同协议事件
- `event_name` 已退出正式 outbox 顶层字段

## 验收矩阵

| Case ID | 场景 | 入口 | 预期 |
| --- | --- | --- | --- |
| `AUD030-CASE-001` | 订单创建 canonical envelope | `trade003_create_order_db_smoke` | `trade.order.created` 只写 1 条；`target_topic` 命中 `ops.event_route_policy`；payload 顶层包含 canonical 字段，且无 `event_name` |
| `AUD030-CASE-002` | 搜索同步 canonical route | `cat022_search_visibility_fields_and_events_db_smoke` | create / patch 两次请求各只写 1 条 `search.product.changed`；`target_topic=dtp.search.sync` 来自 route policy；无 `event_name` |
| `AUD030-CASE-003` | 交付提交与 bridge 无双写 | `dlv002_file_delivery_commit_db_smoke` | `delivery.committed` 与 `billing.trigger.bridge` 各只写 1 条；两者都命中 route policy；无 `event_name` |
| `AUD030-CASE-004` | 自动交付任务 canonical envelope | `dlv029_delivery_task_autocreation_db_smoke` | 每个新 delivery task 只写 1 条 `delivery.task.auto_created`；payload 顶层 canonical 字段完整；无 `event_name` |
| `AUD030-CASE-005` | anchor retry canonical route | `audit_trace_api_db_smoke` | `audit.anchor_requested` 只写 1 条；`target_topic=dtp.audit.anchor` 来自 route policy；payload 顶层 canonical 字段完整；无 `event_name` |
| `AUD030-CASE-006` | 旧 trigger 已退役 | `psql` 静态回查 | `information_schema.triggers` 中无 `tg_write_outbox` 使用者；`common.tg_write_outbox()` 只会抛异常 |

## 执行命令

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

## 静态 SQL

```sql
SELECT event_object_schema, event_object_table, trigger_name
FROM information_schema.triggers
WHERE action_statement ILIKE '%tg_write_outbox%';

SELECT aggregate_type,
       event_type,
       target_topic,
       authority_scope,
       proof_commit_policy
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

## 回查要点

- `ops.outbox_event` 同一 `(aggregate_type, event_type, request_id)` 计数为 `1`
- `payload.event_schema_version='v1'`
- `payload.source_of_truth='database'`
- `payload.event_name` 缺失
- `target_topic` 与 route policy 对应记录一致
