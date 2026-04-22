# Audit Authority Writer（AUD-029）

`AUD-029` 之后，历史模块的审计与证据写入按以下口径收口：

- `catalog`：统一通过 `apps/platform-core/src/modules/catalog/api/support.rs` 调 `modules::audit::application::write_audit_event(...)`
- `search`：统一通过 `AuditEvent::business + modules::audit::repo::insert_audit_event(...)` 写正式 `audit.audit_event`
- `billing`：统一通过 `apps/platform-core/src/modules/billing/db.rs` 调 `modules::audit::application::write_audit_event(...)`
- `billing dispute` 证据对象：继续保留 `support.evidence_object` 兼容表，但正式权威联查对象是
  - `audit.evidence_item`
  - `audit.evidence_manifest`
  - `audit.evidence_manifest_item`

兼容桥接规则：

- `support.evidence_object.metadata.audit_evidence_item_id`
- `support.evidence_object.metadata.audit_evidence_manifest_id`

必须可反查到正式 `audit.evidence_*` 对象；`support.evidence_object` 不能再作为独立证据权威源。

## 最小验证

先加载本地环境：

```bash
set -a
source infra/docker/.env.local
set +a
```

执行三条最小 smoke：

```bash
cargo test -p platform-core cat024_catalog_listing_review_db::tests::cat024_catalog_listing_review_end_to_end_db_smoke -- --nocapture

cargo test -p platform-core search_api_and_ops_db_smoke -- --nocapture

cargo test -p platform-core bil013_dispute_case_db::tests::bil013_dispute_case_db_smoke -- --nocapture
```

## 回查重点

### Catalog / Search / Billing 审计事件

确认请求对应 `audit.audit_event` 已落盘，且 metadata 中保留统一 writer 痕迹：

```sql
SELECT domain_name,
       ref_type,
       action_name,
       result_code,
       metadata ->> 'writer' AS writer
FROM audit.audit_event
WHERE request_id = '<request_id>'
ORDER BY created_at DESC, audit_id DESC;
```

预期：

- `writer = 'audit.application.write_audit_event'` 的 `catalog` / `billing` 审计事件可见
- `search` 事件可通过正式 `audit.audit_event` 联查，不再依赖旁路表或普通日志

### Billing Dispute 证据桥接

```sql
SELECT evidence_id::text,
       metadata ->> 'audit_evidence_item_id' AS audit_evidence_item_id,
       metadata ->> 'audit_evidence_manifest_id' AS audit_evidence_manifest_id
FROM support.evidence_object
WHERE case_id = '<case_id>'::uuid
ORDER BY created_at DESC, evidence_id DESC;

SELECT evidence_item_id::text,
       ref_type,
       ref_id::text,
       object_uri,
       metadata -> 'legacy_bridge' ->> 'legacy_table' AS legacy_table
FROM audit.evidence_item
WHERE evidence_item_id = '<audit_evidence_item_id>'::uuid;

SELECT evidence_manifest_id::text,
       manifest_scope,
       ref_type,
       ref_id::text,
       manifest_hash
FROM audit.evidence_manifest
WHERE evidence_manifest_id = '<audit_evidence_manifest_id>'::uuid;
```

预期：

- `support.evidence_object` 能回查到 `audit_evidence_item_id / audit_evidence_manifest_id`
- `audit.evidence_item.metadata.legacy_bridge.legacy_table = 'support.evidence_object'`
- `audit.evidence_manifest` 与 `audit.evidence_manifest_item` 完整存在

## 排障边界

- 若 `catalog` 或 `billing` 仍写出 ad-hoc `INSERT audit.audit_event`，优先检查对应 helper 是否绕过了 `modules::audit::application::write_audit_event(...)`
- 若 dispute evidence 只在 `support.evidence_object` 找得到，先检查 `record_evidence_snapshot(...)` 与 `bridge_support_evidence_object(...)` 是否在同一事务内执行
- `support.evidence_object` 仍允许作为兼容表存在，但它只保存 legacy bridge，不再定义正式导出 / 回放 / legal hold 的主查询口径
