# Fabric Debug（BOOT-009）

- 关注 `fabric-adapter` 与 `fabric-event-listener` 日志。
- 链提交失败时先检查 outbox 事件与回执关联 ID。
- `AUD-014` 起，除进程能启动外，还要回查 `submission_kind / contract_name / transaction_name` 是否分别命中 `evidence_batch_root / order_digest / authorization_digest / acceptance_digest` 四类摘要处理占位；若 `summary_type` 缺失，Go handler 应直接报错而不是默默回退到默认摘要类型。
