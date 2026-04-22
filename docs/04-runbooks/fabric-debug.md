# Fabric Debug（BOOT-009）

- 关注 `fabric-adapter` 与 `fabric-event-listener` 日志。
- 链提交失败时先检查 outbox 事件与回执关联 ID。
- `AUD-013` 起，可先执行 `./scripts/fabric-adapter-test.sh` 与 `./scripts/fabric-adapter-run.sh` 排除 Go 进程本身问题，再回查 `ops.external_fact_receipt / audit.audit_event / ops.system_log / chain.chain_anchor`。
