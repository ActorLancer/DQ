# TEST-012 Delivery Revocation

`TEST-012` 的正式目标是证明交付资源在退款、到期、争议与风控冻结后会被真实断权，而不是只在某张表里写一条状态。通过标准不是“状态变了”，而是要同时证明：旧下载票据不能继续使用，share/API/sandbox 的正式入口不能继续推进，`PostgreSQL / Redis / 审计` 也都同步反映断权结果。

## 正式入口

- 本地 / CI checker：`ENV_FILE=infra/docker/.env.local ./scripts/check-delivery-revocation.sh`
- checker 会先执行：
  - `./scripts/smoke-local.sh`
- 然后串行运行：
  - `TRADE_DB_SMOKE=1 cargo test -p platform-core dlv021_auto_cutoff_resources_db_smoke -- --nocapture`

## 覆盖闭环

- `dlv021_auto_cutoff_resources_db_smoke`
  - 文件退款后：
    - `delivery.delivery_ticket.status = revoked`
    - Redis download ticket cache 被删除
    - 旧 `download_token` 再次访问 `GET /api/v1/orders/{id}/download` 返回 `409`
  - `SHARE_RO` 到期 / 争议中断后：
    - `delivery.data_share_grant.grant_status = expired / suspended`
    - `delivery.delivery_record.status = expired / suspended`
    - `GET /api/v1/orders/{id}/share-grants` 返回非 active grant
    - 再次执行 `grant_read_access` 返回 `409 SHARE_RO_TRANSITION_FORBIDDEN`
  - `API_PPU` 风控断权后：
    - `delivery.api_credential.status = suspended`
    - `delivery.delivery_record.status = suspended`
    - `GET /api/v1/orders/{id}/usage-log` 暴露 `credential_status = suspended`
    - 再次执行 `settle_success_call` 返回 `409 API_PPU_TRANSITION_FORBIDDEN`
  - `SBX_STD` 到期后：
    - `delivery.sandbox_workspace.status = expired`
    - `delivery.sandbox_session.session_status = expired`
    - `delivery.delivery_record.status = expired`
    - 再次执行 `execute_sandbox_query` 返回 `409 SBX_STD_TRANSITION_FORBIDDEN`
  - 四类 cutoff 都继续回查 `audit.audit_event`

## 关键不变量

- 文件断权后，旧 token 不能继续消费；仅删除 Redis key 不算通过，还要确认 DB ticket 状态同步为 `revoked`
- share grant 断权后，正式读接口只能看到 `expired / suspended`，不能再把旧 grant 当 active access 继续推进
- API credential 断权后，管理面必须能读到 `suspended`，执行面也不能再推进成功调用状态
- sandbox 到期后，workspace / session / delivery record 必须同步关闭，不能继续推进查询执行
- 审计必须保留：
  - `delivery.file.auto_cutoff.revoked`
  - `delivery.share.auto_cutoff.expired`
  - `delivery.share.auto_cutoff.suspended`
  - `delivery.api.auto_cutoff.suspended`
  - `delivery.sandbox.auto_cutoff.expired`

## 关键回查

- 文件票据 / Redis：

```sql
SELECT ticket_id::text,
       status,
       download_count,
       to_char(expire_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
FROM delivery.delivery_ticket
WHERE order_id = '<file_order_uuid>'::uuid;
```

- 共享 / API / 沙箱状态：

```sql
SELECT 'share' AS resource_kind,
       order_id::text,
       grant_status AS resource_status
FROM delivery.data_share_grant
WHERE order_id IN ('<share_expire_order_uuid>'::uuid, '<share_dispute_order_uuid>'::uuid)
UNION ALL
SELECT 'api',
       order_id::text,
       status
FROM delivery.api_credential
WHERE order_id = '<api_order_uuid>'::uuid
UNION ALL
SELECT 'sandbox_workspace',
       order_id::text,
       status
FROM delivery.sandbox_workspace
WHERE order_id = '<sandbox_order_uuid>'::uuid;
```

- sandbox session / delivery record / audit：

```sql
SELECT action_name,
       request_id,
       result_code
FROM audit.audit_event
WHERE request_id = ANY(ARRAY[
  '<file_cutoff_req_id>',
  '<share_expire_req_id>',
  '<share_dispute_req_id>',
  '<api_cutoff_req_id>',
  '<sandbox_cutoff_req_id>'
]);
```

## DLV / TRADE 映射

- `DLV-CASE-4.4`：撤权后访问
- `trade011_api_ppu_state_machine_db_smoke`：`disable_access` 后禁止继续推进
- `trade012_share_ro_state_machine_db_smoke`：共享断权后禁止继续 `grant_read_access`
- `trade014_sbx_std_state_machine_db_smoke`：沙箱到期后禁止继续 `execute_sandbox_query`

## 禁止误报

- 只看 `delivery_ticket` / `api_credential` / `data_share_grant` / `sandbox_session` 任意一张表不算通过；必须继续确认正式入口也不能再用
- 只看 `GET /share-grants` 或 `GET /usage-log` 返回 `200` 不算通过；必须进一步证明返回的是停用态或后续正式 transition 已被拒绝
- 只看旧下载 token 失败不算全部通过；还要确认 Redis cache 已删、DB ticket 已同步断权、审计事件已落库
