# 04-runbooks

用于收敛本地联调、中间件运行、故障排查与应急操作手册。

- `notification-worker.md`：通知正式链路、审计联查、DLQ、人工补发 / replay。
- `search-reindex.md`：`AUD-022 / AUD-026` 搜索运维控制面与 `search-indexer` worker 的 Bearer 鉴权、step-up、Redis 缓存失效、OpenSearch alias 切换、排序配置更新、consumer 幂等、双层 DLQ、dry-run reprocess 与审计回查。
- `recommendation-runtime.md`：`SEARCHREC / AUD-026` 推荐主链路、`recommendation-aggregator` worker、副作用、consumer 幂等、双层 DLQ、dry-run reprocess 与缓存失效回归。
- `fabric-local.md`：Fabric 本地网络、Gateway、anchor / callback 联调。
- `fabric-adapter.md`：`AUD-013 / AUD-014 / AUD-026` Go 版 `fabric-adapter` 的 bootstrap / test / run、四类摘要 handler、Redis 短锁 + `ops.consumer_idempotency_record` 重复投递隔离、Kafka canonical smoke、回执回写与排障。
- `fabric-event-listener.md`：`AUD-015` Go 版 `fabric-event-listener` 的 bootstrap / test / run、mock callback 链路、`dtp.fabric.callbacks`、外部事实回执回写与排障。
- `fabric-ca-admin.md`：`AUD-016` Go 版 `fabric-ca-admin` 的 bootstrap / test / run、`fabric-identities / certificates` 执行面、step-up 串联、`ops.external_fact_receipt / audit.audit_event / ops.system_log` 回查与排障。
- `audit-replay.md`：`AUD-005` replay dry-run 控制面的权限、step-up、回查与故障处理。
- `audit-legal-hold.md`：`AUD-006` legal hold 创建 / 释放控制面的权限、step-up、回查与故障处理。
- `audit-anchor-batches.md`：`AUD-007` anchor batch 查看 / retry 控制面的权限、step-up、DB / outbox / 审计回查与运行态边界。
- `audit-ops-outbox-dead-letters.md`：`AUD-008` canonical outbox / dead letter 查询、SEARCHREC consumer 幂等联查、审计留痕与运行态边界。
- `outbox-publisher.md`：`AUD-009 / AUD-031` canonical outbox publisher 的宿主机/compose 启动、Kafka 发布、双层 DLQ、`ops.outbox_publish_attempt` 联查、Billing bridge published-only 手工回放边界与 Prometheus 指标。
- `audit-dead-letter-reprocess.md`：`AUD-010` SEARCHREC dead letter dry-run 重处理控制面的权限、step-up、DB / 审计回查与运行态边界。
- `audit-consistency-lookup.md`：`AUD-011` 一致性联查接口的宿主机调用、正式 `refType`、DB / 审计回查与运行态边界。
- `audit-consistency-reconcile.md`：`AUD-012` 一致性修复 dry-run 控制面的权限、step-up、`ops.chain_projection_gap` 预演建议、DB / 审计回查与“无执行副作用”边界。
- `audit-authority-writer.md`：`AUD-029` 历史模块统一 `audit writer / evidence writer`、`support.evidence_object -> audit.evidence_*` 桥接与最小 smoke / SQL 回查。
- `canonical-event-authority.md`：`AUD-030` canonical outbox writer、`ops.event_route_policy`、退役 trigger、统一 envelope 顶层字段与无双写验收入口。
- `audit-external-facts.md`：`AUD-019` 外部事实查询 / 确认控制面的权限、step-up、`ops.external_fact_receipt` 回查与“不直接改业务主状态”边界。
- `audit-fairness-incidents.md`：`AUD-020` 公平性事件查询 / 处理控制面的权限、step-up、`risk.fairness_incident` 回查与“只记录联动建议、不直接改业务主状态”边界。
- `audit-projection-gaps.md`：`AUD-021` 投影缺口查询 / 关闭控制面的权限、step-up、`ops.chain_projection_gap` 回查、`dry_run` 预演与“只关闭正式 gap 对象、不派生 reconcile job”边界。
- `audit-observability.md`：`AUD-023` 观测总览、日志镜像查询 / 导出、trace 联查、告警 / 事故 / SLO 查询的权限、step-up、MinIO 对象回查与观测栈联查边界。
- `developer-trace.md`：`AUD-024` 开发者状态联查的 `order_id / event_id / tx_hash` 单 selector 查询、tenant+order scope、Go/Fabric 回写状态可见性、`audit.access_audit / ops.system_log` 回查与排障边界。
- `audit-trade-monitor.md`：`AUD-018` 交易链监控总览 / checkpoints 的宿主机调用、tenant+order scope、`trade_lifecycle_checkpoint / external_fact / fairness / projection_gap / chain_anchor` 回查与运行态边界。
