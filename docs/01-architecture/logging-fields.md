# 统一日志字段规范（BOOT-013）

## 核心字段（最小集合）

- `request_id`
- `trace_id`
- `tenant_id`
- `actor_id`
- `order_id`
- `event_id`
- `provider`
- `mode`
- `result_code`

## 补充字段（按场景）

- 身份与权限：`session_id`, `permission_code`, `scope_snapshot`
- 交付与计费：`authorization_id`, `delivery_id`, `billing_event_id`
- 一致性链路：`outbox_event_id`, `external_fact_id`, `reconcile_id`

## 规则

- 所有高风险操作日志必须含 `request_id` + 关键对象 ID。
- 不得记录明文密钥、密码、令牌。
- 日志级别建议：业务成功 `info`，可恢复异常 `warn`，不可恢复异常 `error`。
