# 04-runbooks

用于收敛本地联调、中间件运行、故障排查与应急操作手册。

- `notification-worker.md`：通知正式链路、审计联查、DLQ、人工补发 / replay。
- `fabric-local.md`：Fabric 本地网络、Gateway、anchor / callback 联调。
- `fabric-adapter.md`：`AUD-013 / AUD-014` Go 版 `fabric-adapter` 的 bootstrap / test / run、四类摘要 handler、Kafka canonical smoke、回执回写与排障。
- `fabric-event-listener.md`：`AUD-015` Go 版 `fabric-event-listener` 的 bootstrap / test / run、mock callback 链路、`dtp.fabric.callbacks`、外部事实回执回写与排障。
- `fabric-ca-admin.md`：`AUD-016` Go 版 `fabric-ca-admin` 的 bootstrap / test / run、`fabric-identities / certificates` 执行面、step-up 串联、`ops.external_fact_receipt / audit.audit_event / ops.system_log` 回查与排障。
- `audit-replay.md`：`AUD-005` replay dry-run 控制面的权限、step-up、回查与故障处理。
- `audit-legal-hold.md`：`AUD-006` legal hold 创建 / 释放控制面的权限、step-up、回查与故障处理。
- `audit-anchor-batches.md`：`AUD-007` anchor batch 查看 / retry 控制面的权限、step-up、DB / outbox / 审计回查与运行态边界。
- `audit-ops-outbox-dead-letters.md`：`AUD-008` canonical outbox / dead letter 查询、SEARCHREC consumer 幂等联查、审计留痕与运行态边界。
- `outbox-publisher.md`：`AUD-009` canonical outbox publisher 的宿主机/compose 启动、Kafka 发布、双层 DLQ、Billing bridge 手工回放边界与 Prometheus 指标。
- `audit-dead-letter-reprocess.md`：`AUD-010` SEARCHREC dead letter dry-run 重处理控制面的权限、step-up、DB / 审计回查与运行态边界。
- `audit-consistency-lookup.md`：`AUD-011` 一致性联查接口的宿主机调用、正式 `refType`、DB / 审计回查与运行态边界。
- `audit-consistency-reconcile.md`：`AUD-012` 一致性修复 dry-run 控制面的权限、step-up、`ops.chain_projection_gap` 预演建议、DB / 审计回查与“无执行副作用”边界。
