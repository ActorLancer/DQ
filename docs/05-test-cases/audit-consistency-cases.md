# Audit / Consistency 验收清单

当前文件承接 `AUD-003`、`AUD-004`、`AUD-005`、`AUD-006`、`AUD-007`、`AUD-008`、`AUD-009`、`AUD-010`、`AUD-011`、`AUD-012`、`AUD-013`、`AUD-014`、`AUD-015`、`AUD-016`、`AUD-017`、`AUD-018`、`AUD-019`、`AUD-020`、`AUD-021`、`AUD-023` 已落地的首版审计控制面验收矩阵，覆盖：

- 订单审计联查：`GET /api/v1/audit/orders/{id}`
- 全局审计 trace 查询：`GET /api/v1/audit/traces`
- 证据包导出：`POST /api/v1/audit/packages/export`
- 回放任务 dry-run：`POST /api/v1/audit/replay-jobs`
- 回放任务联查：`GET /api/v1/audit/replay-jobs/{id}`
- legal hold 创建 / 释放：`POST /api/v1/audit/legal-holds`、`POST /api/v1/audit/legal-holds/{id}/release`
- anchor batch 查看 / 重试：`GET /api/v1/audit/anchor-batches`、`POST /api/v1/audit/anchor-batches/{id}/retry`
- canonical outbox 查询：`GET /api/v1/ops/outbox`
- dead letter 查询：`GET /api/v1/ops/dead-letters`
- dead letter dry-run 重处理：`POST /api/v1/ops/dead-letters/{id}/reprocess`
- 一致性联查：`GET /api/v1/ops/consistency/{refType}/{refId}`
- 一致性修复 dry-run：`POST /api/v1/ops/consistency/reconcile`
- 外部事实查询 / 确认：`GET /api/v1/ops/external-facts`、`POST /api/v1/ops/external-facts/{id}/confirm`
- outbox publisher：`ops.outbox_event -> workers/outbox-publisher -> Kafka / ops.outbox_publish_attempt / ops.dead_letter_event`
- fabric adapter 四类摘要 handler：`dtp.audit.anchor / dtp.fabric.requests -> services/fabric-adapter -> evidence_batch_root / order_summary / authorization_summary / acceptance_summary -> ops.external_fact_receipt / audit.audit_event / ops.system_log / chain.chain_anchor`
- fabric callback listener：`services/fabric-event-listener -> dtp.fabric.callbacks -> ops.external_fact_receipt / audit.audit_event / ops.system_log / chain.chain_anchor / audit.anchor_batch`
- fabric CA admin：`platform-core IAM API -> services/fabric-ca-admin -> iam.fabric_identity_binding / iam.certificate_record / iam.certificate_revocation_record / ops.external_fact_receipt / audit.audit_event / ops.system_log`
- 交易链监控总览 / checkpoints：`GET /api/v1/ops/trade-monitor/orders/{orderId}`、`GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints`
- 公平性事件查询 / 处理：`GET /api/v1/ops/fairness-incidents`、`POST /api/v1/ops/fairness-incidents/{id}/handle`
- 投影缺口查询 / 关闭：`GET /api/v1/ops/projection-gaps`、`POST /api/v1/ops/projection-gaps/{id}/resolve`
- 观测总览 / 日志镜像查询导出 / trace 联查 / 告警与 incident / SLO：`GET /api/v1/ops/observability/overview`、`GET /api/v1/ops/logs/query`、`POST /api/v1/ops/logs/export`、`GET /api/v1/ops/traces/{traceId}`、`GET /api/v1/ops/alerts`、`GET /api/v1/ops/incidents`、`GET /api/v1/ops/slos`

后续 `search sync ops / export package aggregate` 等高风险控制面进入对应 `AUD` task 后，再继续追加到本文件，不得另起旁路清单。

## 前置条件

- 已执行：`set -a; source infra/docker/.env.local; set +a`
- 本地基础设施可用：`PostgreSQL`、`MinIO`、`Kafka`、`Redis`、`Keycloak / IAM`、观测栈
- 平台服务可用：`platform-core`
- 如需一次性跑首版 live smoke：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_consistency_lookup_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_consistency_reconcile_db_smoke -- --nocapture

IAM_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core iam_fabric_ca_admin_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_fairness_incident_handle_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_projection_gap_resolve_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core observability_api_db_smoke -- --nocapture
```

## 验收矩阵

| 用例ID | 场景 | 输入 / 操作 | 预期结果 | 主要回查点 |
| --- | --- | --- | --- | --- |
| `AUD-CASE-001` | 订单审计联查 | `GET /api/v1/audit/orders/{id}` | 返回订单最小审计视图，租户读场景只允许 buyer/seller org | API 响应、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-002` | 全局 trace 查询 | `GET /api/v1/audit/traces?order_id=...&trace_id=...` | 返回同一请求链的审计事件分页结果 | API 响应、`audit.audit_event`、`audit.access_audit` |
| `AUD-CASE-003` | 证据包导出 | `POST /api/v1/audit/packages/export` + `x-step-up-challenge-id` | 写入 `audit.evidence_package`、MinIO 导出对象、`audit.audit_event(action_name='audit.package.export')` | API 响应、`audit.evidence_package`、`audit.evidence_manifest_item`、MinIO 对象、`ops.system_log` |
| `AUD-CASE-004` | replay 创建 | `POST /api/v1/audit/replay-jobs` with `dry_run=true` | 写入 `audit.replay_job + audit.replay_result`，MinIO replay report 落盘，并生成 `audit.replay.requested / completed` | API 响应、`audit.replay_job`、`audit.replay_result`、`audit.audit_event`、MinIO 对象 |
| `AUD-CASE-005` | replay 只允许 dry-run | `POST /api/v1/audit/replay-jobs` with `dry_run=false` | 返回 `409` 且错误码 `AUDIT_REPLAY_DRY_RUN_ONLY` | HTTP 响应、错误码 |
| `AUD-CASE-006` | replay 读取 | `GET /api/v1/audit/replay-jobs/{id}` | 返回 replay job + results；读取动作也必须落 `audit.access_audit` 与 `ops.system_log` | API 响应、`audit.access_audit(access_mode='replay')`、`ops.system_log` |
| `AUD-CASE-007` | 高风险动作鉴权 | 缺少权限或缺少 step-up 分别调用 export / replay | 返回 `403 / 400`，不得写业务副作用 | HTTP 响应、`audit.evidence_package / replay_job` 无新增 |
| `AUD-CASE-008` | legal hold 创建 | `POST /api/v1/audit/legal-holds` + `x-step-up-challenge-id` | 写入 `audit.legal_hold`，并产生 `audit.audit_event(action_name='audit.legal_hold.create')`；当前 hold 状态以 `audit.legal_hold` 为权威源，历史 evidence/package 行保持 append-only | API 响应、`audit.legal_hold`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-009` | legal hold 重复创建冲突 | 对同一 active scope 再次创建 | 返回 `409` 且错误码 `AUDIT_LEGAL_HOLD_ACTIVE` | HTTP 响应、错误码、无新增 hold |
| `AUD-CASE-010` | legal hold 释放 | `POST /api/v1/audit/legal-holds/{id}/release` | `status=released`、`approved_by / released_at` 落库，并产生 `audit.audit_event(action_name='audit.legal_hold.release')`；当前 hold 状态回到 `audit.legal_hold` 权威视图中的 `none` | API 响应、`audit.legal_hold`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-011` | anchor batch 查看 | `GET /api/v1/audit/anchor-batches?anchor_status=failed&batch_scope=audit_event&chain_id=fabric-local` | 返回 `audit.anchor_batch + chain.chain_anchor` 联查视图，可看到 `anchor_batch_id / batch_scope / record_count / batch_root / chain_id / tx_hash / anchor_status / anchored_at` | API 响应、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-012` | failed batch retry | `POST /api/v1/audit/anchor-batches/{id}/retry` + verified `x-step-up-challenge-id` | 仅允许 `status=failed`；成功后 `audit.anchor_batch.status=retry_requested`，并写出 canonical outbox `audit.anchor_requested -> dtp.audit.anchor` | API 响应、`audit.anchor_batch`、`ops.outbox_event(target_topic='dtp.audit.anchor')`、`audit.audit_event(action_name='audit.anchor.retry')`、`audit.access_audit(access_mode='retry')`、`ops.system_log` |
| `AUD-CASE-013` | canonical outbox 查询 | `GET /api/v1/ops/outbox?target_topic=dtp.search.sync&event_type=search.product.changed` | 返回 `ops.outbox_event` 分页结果，并包含最新 `ops.outbox_publish_attempt`；查询动作写入 `audit.access_audit + ops.system_log` | API 响应、`ops.outbox_event`、`ops.outbox_publish_attempt`、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-014` | dead letter + SEARCHREC 幂等联查 | `GET /api/v1/ops/dead-letters?failure_stage=consumer_handler` | 返回 `ops.dead_letter_event` 分页结果，并挂出 `ops.consumer_idempotency_record`，可定位 `search-indexer` 或 `recommendation-aggregator` 的失败隔离记录 | API 响应、`ops.dead_letter_event`、`ops.consumer_idempotency_record`、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-015` | outbox publisher 发布成功 | 写入一条 `ops.outbox_event(status=pending,target_topic=dtp.outbox.domain-events)`，运行 `cargo test -p outbox-publisher outbox_publisher_db_smoke -- --nocapture` | `workers/outbox-publisher` 把事件发到 Kafka，`ops.outbox_event.status=published`，并写入 `ops.outbox_publish_attempt(result_code='published')`、`audit.audit_event(action_name='outbox.publisher.publish')`、`ops.system_log(service_name='outbox-publisher')` | Kafka 消息、`ops.outbox_event`、`ops.outbox_publish_attempt`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-016` | outbox publisher 失败隔离 | 写入一条 `ops.outbox_event(status=pending,target_topic=dtp.missing.topic,max_retries=1)`，运行同一 smoke | 事件进入 `ops.dead_letter_event(failure_stage='outbox.publish')`，并向 Kafka `dtp.dead-letter` 发布隔离消息；原 outbox 行变为 `dead_lettered` | `ops.dead_letter_event`、Kafka `dtp.dead-letter`、`ops.outbox_publish_attempt(result_code='dead_lettered')`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-017` | SEARCHREC dead letter dry-run 重处理 | `POST /api/v1/ops/dead-letters/{id}/reprocess` + verified `x-step-up-challenge-id` + `{"reason":"...","dry_run":true}` | 仅允许 `reprocess_status=not_reprocessed` 的 SEARCHREC consumer dead letter；返回 `dry_run_ready` 预演计划，不改变 `ops.dead_letter_event.reprocess_status`，并写入 `audit.audit_event(action_name='ops.dead_letter.reprocess.dry_run')`、`audit.access_audit(access_mode='reprocess')`、`ops.system_log` | API 响应、`ops.dead_letter_event`、`audit.audit_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-018` | 一致性联查 | `GET /api/v1/ops/consistency/order/{order_id}` | 返回业务状态、proof/anchor 状态、外部事实状态，以及最近 `ops.outbox_event / ops.dead_letter_event / audit.audit_event`；查询动作写入 `audit.access_audit(target_type='consistency_query')` 与 `ops.system_log` | API 响应、`trade.order_main`、`chain.chain_anchor`、`ops.chain_projection_gap`、`ops.external_fact_receipt`、`ops.outbox_event`、`ops.dead_letter_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-019` | 一致性修复 dry-run | `POST /api/v1/ops/consistency/reconcile` + verified `x-step-up-challenge-id` + `{"ref_type":"order","ref_id":"...","mode":"full","dry_run":true,"reason":"..."}` | 不新增 `reconcile_job` 表、不改写 `ops.chain_projection_gap`，只返回修复建议并写入 `audit.audit_event(action_name='ops.consistency.reconcile.dry_run')`、`audit.access_audit(access_mode='reconcile', target_type='consistency_reconcile')`、`ops.system_log`；同时不得写出 `dtp.consistency.reconcile` 新 outbox 事件 | API 响应、`ops.chain_projection_gap` 仍为原状态、`audit.audit_event`、`audit.access_audit`、`ops.system_log`、`ops.outbox_event(request_id=...)` |
| `AUD-CASE-020` | Fabric adapter 四类摘要 request consume + receipt write-back | 启动 `./scripts/fabric-adapter-run.sh`，使用 `kcat` 向 `dtp.audit.anchor` 写入 `audit.anchor_requested`，向 `dtp.fabric.requests` 分别写入 `summary_type=order_summary / authorization_summary / acceptance_summary` 的 `fabric.proof_submit_requested` | `services/fabric-adapter` 真实消费两条正式 topic，但在 Go 侧显式分派到 `evidence_batch_root / order_summary / authorization_summary / acceptance_summary` 四类 handler，使用 Go mock provider 生成回执，并把 `submission_kind / contract_name / transaction_name` 写入 `ops.external_fact_receipt.metadata`、`receipt_payload`、`audit.audit_event(action_name='fabric.adapter.submit')`、`ops.system_log(message_text='fabric adapter accepted submit event')`；若 payload 提供 `chain_anchor_id`，则 `chain.chain_anchor.status=submitted` 且 `reconcile_status=pending_check` | Kafka topic 内容、`ops.external_fact_receipt`、`audit.audit_event`、`ops.system_log`、`chain.chain_anchor`、`cg-fabric-adapter` consumer group |
| `AUD-CASE-021` | Fabric callback listener consume + callback write-back | 启动 `./scripts/fabric-adapter-run.sh` 与 `./scripts/fabric-event-listener-run.sh`；先通过 `dtp.audit.anchor / dtp.fabric.requests` 生成 source receipt，再把其中一条 source receipt 标记 `mock_callback_status=failed` | `services/fabric-event-listener` 轮询已提交 source receipt，生成 `fabric.commit_confirmed / fabric.commit_failed`，发布到 `dtp.fabric.callbacks`，并把 `provider_code / provider_request_id / callback_event_id / event_version / provider_status / provider_occurred_at / payload_hash` 写回 `ops.external_fact_receipt.metadata`；同时写入 `audit.audit_event(action_name='fabric.event_listener.callback')`、`ops.system_log(message_text='fabric event listener published callback')`，成功链更新 `chain.chain_anchor.status='anchored'` 与 `audit.anchor_batch.status='anchored'`，失败链更新 `chain.chain_anchor.status='failed'` | Kafka `dtp.fabric.callbacks`、`ops.external_fact_receipt`、`audit.audit_event`、`ops.system_log`、`chain.chain_anchor`、`audit.anchor_batch` |
| `AUD-CASE-022` | Fabric CA admin 证书签发 / 吊销执行面 | 启动 `./scripts/fabric-ca-admin-run.sh`，通过 `platform-core` 先完成 step-up challenge，再调用 `POST /api/v1/iam/fabric-identities/{id}/issue` 与 `POST /api/v1/iam/certificates/{id}/revoke` | Rust `platform-core` 负责权限、step-up、公网错误码与 `audit.audit_event(actor_id=真实操作者)`；Go `services/fabric-ca-admin` 负责执行证书签发 / 吊销，真实更新 `iam.fabric_identity_binding / iam.certificate_record / iam.certificate_revocation_record`，并写入 `ops.external_fact_receipt(fact_type in ('certificate_issue_receipt','certificate_revocation_receipt'))`、`ops.system_log(message_text in ('fabric ca admin issued identity','fabric ca admin revoked certificate'))` | HTTP 响应、`iam.step_up_challenge`、`iam.fabric_identity_binding`、`iam.certificate_record`、`iam.certificate_revocation_record`、`ops.external_fact_receipt`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-023` | 交易链监控总览 | `GET /api/v1/ops/trade-monitor/orders/{order_id}` | 返回订单维度 trade monitor 视图，包含最近 checkpoint / external fact / fairness incident / projection gap 摘要；平台与租户角色都必须通过正式权限与 order scope 校验；查询动作写入 `audit.access_audit + ops.system_log` | API 响应、`trade.order_main`、`ops.trade_lifecycle_checkpoint`、`ops.external_fact_receipt`、`risk.fairness_incident`、`ops.chain_projection_gap`、`chain.chain_anchor`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-024` | 生命周期检查点过滤查询 | `GET /api/v1/ops/trade-monitor/orders/{order_id}/checkpoints?checkpoint_status=pending&lifecycle_stage=delivery&page=1&page_size=20` | 返回过滤后的 `ops.trade_lifecycle_checkpoint` 分页结果；tenant 侧必须命中 buyer/seller order scope；查询动作写入 `audit.access_audit + ops.system_log` | API 响应、`ops.trade_lifecycle_checkpoint`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-025` | 外部事实分页查询 | `GET /api/v1/ops/external-facts?order_id=...&receipt_status=pending&provider_type=mock_payment_provider&from=...&to=...` | 返回 `ops.external_fact_receipt` 分页结果；查询动作写入 `audit.access_audit(target_type='external_fact_query', access_mode='masked')` 与 `ops.system_log` | API 响应、`ops.external_fact_receipt`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-026` | 外部事实确认 | `POST /api/v1/ops/external-facts/{id}/confirm` + verified `x-step-up-challenge-id` + `{"confirm_result":"confirmed","reason":"...","operator_note":"..."}` | 仅允许 `receipt_status='pending'`；成功后只更新 `ops.external_fact_receipt.receipt_status / confirmed_at / metadata.manual_confirmation / metadata.rule_evaluation`，写入 `audit.audit_event(action_name='ops.external_fact.confirm')`、`audit.access_audit(access_mode='confirm', target_type='external_fact_receipt')`、`ops.system_log`，且不得直接改写 `trade.order_main.external_fact_status` | API 响应、`ops.external_fact_receipt`、`trade.order_main`、`audit.audit_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-027` | 公平性事件分页查询 | `GET /api/v1/ops/fairness-incidents?order_id=...&incident_type=seller_delivery_delay&severity=high&fairness_incident_status=open&assigned_role_key=platform_risk_settlement&assigned_user_id=...` | 返回 `risk.fairness_incident` 分页结果；查询动作写入 `audit.access_audit(target_type='fairness_incident_query', access_mode='masked')` 与 `ops.system_log` | API 响应、`risk.fairness_incident`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-028` | 公平性事件处理 | `POST /api/v1/ops/fairness-incidents/{id}/handle` + verified `x-step-up-challenge-id` + `{"action":"close","resolution_summary":"...","auto_action_override":"notify_ops","freeze_settlement":true,"create_dispute_suggestion":true}` | 仅允许 `fairness_incident_status='open'`；成功后只更新 `risk.fairness_incident.status / auto_action_code / resolution_summary / metadata / closed_at`，写入 `audit.audit_event(action_name='risk.fairness_incident.handle')`、`audit.access_audit(access_mode='handle', target_type='fairness_incident')`、`ops.system_log`，且 `freeze_settlement / freeze_delivery / create_dispute_suggestion` 只作为 `linked_action_plan` 建议留痕，不得直接改写 `trade.order_main` | API 响应、`risk.fairness_incident`、`trade.order_main`、`audit.audit_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-029` | 投影缺口分页查询 | `GET /api/v1/ops/projection-gaps?aggregate_type=order&aggregate_id=...&order_id=...&chain_id=fabric-local&gap_type=missing_callback&gap_status=open` | 返回 `ops.chain_projection_gap` 分页结果；查询动作写入 `audit.access_audit(target_type='projection_gap_query', access_mode='masked')` 与 `ops.system_log` | API 响应、`ops.chain_projection_gap`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-030` | 投影缺口关闭 | `POST /api/v1/ops/projection-gaps/{id}/resolve` + verified `x-step-up-challenge-id` + `{"dry_run":false,"resolution_mode":"callback_confirmed","reason":"...","expected_state_digest":"..."}` | 默认支持 `dry_run=true` 预演；真实关闭仅允许 `gap_status!='resolved'`，并在 `expected_state_digest` 命中时更新 `ops.chain_projection_gap.gap_status / resolved_at / resolution_summary / metadata / request_id / trace_id`，写入 `audit.audit_event(action_name='ops.projection_gap.resolve')`、`audit.access_audit(access_mode='resolve', target_type='projection_gap')`、`ops.system_log`，且不得写出 `dtp.consistency.reconcile` 新 outbox 事件 | API 响应、`ops.chain_projection_gap`、`audit.audit_event`、`audit.access_audit`、`ops.system_log`、`ops.outbox_event` |
| `AUD-CASE-031` | 观测总览 | `GET /api/v1/ops/observability/overview` | 返回 `backend_statuses / alert_summary / key_services / slo_summary / recent_incidents`；其中 backend 状态必须来自对 `Prometheus / Alertmanager / Grafana / Loki / Tempo / OTel Collector` 的真实探测，查询动作写入 `audit.access_audit(target_type='observability_overview')` 与 `ops.system_log` | API 响应、`ops.observability_backend`、`ops.alert_event`、`ops.incident_ticket`、`ops.slo_definition`、`ops.slo_snapshot`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-032` | 日志镜像查询 | `GET /api/v1/ops/logs/query?trace_id=...&page=1&page_size=20` | 返回 `ops.system_log` 镜像分页结果；`/api/v1/ops/logs` 兼容别名与 `/logs/query` 返回口径一致；查询动作写入 `audit.access_audit(target_type='system_log_query', access_mode='masked')` 与 `ops.system_log` | API 响应、`ops.system_log`、`audit.access_audit` |
| `AUD-CASE-033` | 原始日志导出 | `POST /api/v1/ops/logs/export` + verified `x-step-up-challenge-id` + `{"reason":"...","trace_id":"..."}` | 至少要求一个 selector；成功后把 JSON 导出对象写入 `MinIO(report-results)`，并写入 `audit.audit_event(action_name='ops.log.export')`、`audit.access_audit(target_type='system_log_export', access_mode='export')`、`ops.system_log`；`step_up_bound=true` | API 响应、MinIO `report-results` 对象、`audit.audit_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-034` | Trace 联查 | `GET /api/v1/ops/traces/{trace_id}` | 以 `ops.trace_index` 为正式索引返回 trace 信息，同时返回 `related_log_count / related_alert_count` 与 `tempo_link / grafana_link`；查询动作写入 `audit.access_audit(target_type='trace_lookup')` 与 `ops.system_log` | API 响应、`ops.trace_index`、`ops.system_log`、`ops.alert_event`、`audit.access_audit` |
| `AUD-CASE-035` | 告警中心查询 | `GET /api/v1/ops/alerts?severity=high&source_backend_key=prometheus_main` | 返回 `ops.alert_event` 分页结果；查询动作写入 `audit.access_audit(target_type='alert_query', access_mode='masked')` 与 `ops.system_log` | API 响应、`ops.alert_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-036` | 事故工单查询 | `GET /api/v1/ops/incidents?owner_role_key=platform_audit_security` | 返回 `ops.incident_ticket` 分页结果与最新 incident event 摘要；查询动作写入 `audit.access_audit(target_type='incident_query', access_mode='masked')` 与 `ops.system_log` | API 响应、`ops.incident_ticket`、`ops.incident_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-037` | SLO 查询 | `GET /api/v1/ops/slos?service_name=platform-core` | 返回 `ops.slo_definition` 联查最新 `ops.slo_snapshot` 的分页结果；查询动作写入 `audit.access_audit(target_type='slo_query', access_mode='masked')` 与 `ops.system_log` | API 响应、`ops.slo_definition`、`ops.slo_snapshot`、`audit.access_audit`、`ops.system_log` |

补充说明：

- `AUD-008` 同步补齐 `ops.external_fact_receipt` 与 `ops.chain_projection_gap` 的仓储查询能力，但其公共 HTTP 控制面接口分别由后续交易链监控 / 一致性任务承接。
- `reconcile` 在 `V1` 中不是独立正式表；不要把 `ops.chain_projection_gap` 宣传成 `reconcile_job` 的同义词。
- `AUD-013` 完成 `fabric-adapter` 基础框架与 mock provider 回执回写；`AUD-014` 已补齐四类摘要 handler 占位；`AUD-015` 已补齐 `fabric-event-listener` 的 callback 轮询源、Kafka callback 发布与 DB 回写；`AUD-016` 已补齐 `fabric-ca-admin` 的证书治理执行面；`fabric-test-network / Gateway / chaincode / real Fabric CA` 留待 `AUD-017`。
- `AUD-019` 已把 `ops.external_fact_receipt` 的公共 HTTP 控制面补齐到查询 / confirm；`AUD-020` 已把 `risk.fairness_incident` 的公共 HTTP 控制面补齐到查询 / handle；`AUD-021` 已把 `projection-gaps` 的公共 HTTP 控制面补齐到查询 / resolve；`AUD-023` 已补齐观测总览、日志镜像查询 / 导出、trace 联查、告警 / incident / SLO 查询。

## `AUD-011` 手工一致性联查验证

1. 启动服务：

```bash
set -a
source infra/docker/.env.local
set +a

APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core-bin
```

2. 触发 live smoke 或自行准备一笔带双层权威字段的对象：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_consistency_lookup_db_smoke -- --nocapture
```

3. 查询一致性视图：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/consistency/order/<order_id>" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud011-manual' \
  -H 'x-trace-id: trace-aud011-manual'
```

4. 回查查询留痕：

```sql
SELECT access_mode, target_type, target_id::text
FROM audit.access_audit
WHERE request_id = 'req-aud011-manual';

SELECT message_text, structured_payload
FROM ops.system_log
WHERE request_id = 'req-aud011-manual'
  AND message_text = 'ops lookup executed: GET /api/v1/ops/consistency/{refType}/{refId}';
```

## `AUD-012` 手工一致性修复 dry-run 验证

1. 启动服务：

```bash
set -a
source infra/docker/.env.local
set +a

APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core-bin
```

2. 触发 live smoke 或自行准备一笔带 `ops.chain_projection_gap` 的对象：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_consistency_reconcile_db_smoke -- --nocapture
```

3. 发起 dry-run 修复预演：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/consistency/reconcile" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud012-manual' \
  -H 'x-trace-id: trace-aud012-manual' \
  -H 'x-step-up-challenge-id: <reconcile_step_up_id>' \
  -d '{
    "ref_type": "order",
    "ref_id": "<order_id>",
    "mode": "full",
    "dry_run": true,
    "reason": "manual consistency reconcile preview"
  }'
```

4. 回查 dry-run 留痕与“无执行副作用”：

```sql
SELECT action_name, result_code, metadata ->> 'mode'
FROM audit.audit_event
WHERE request_id = 'req-aud012-manual'
  AND action_name = 'ops.consistency.reconcile.dry_run';

SELECT access_mode, target_type, target_id::text
FROM audit.access_audit
WHERE request_id = 'req-aud012-manual';

SELECT message_text, structured_payload
FROM ops.system_log
WHERE request_id = 'req-aud012-manual'
  AND message_text = 'ops consistency reconcile prepared: POST /api/v1/ops/consistency/reconcile';

SELECT gap_status, resolution_summary
FROM ops.chain_projection_gap
WHERE aggregate_type = 'order'
  AND aggregate_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_projection_gap_id DESC
LIMIT 5;

SELECT count(*) AS reconcile_preview_outbox_count
FROM ops.outbox_event
WHERE request_id = 'req-aud012-manual'
  AND target_topic = 'dtp.consistency.reconcile';
```

5. 预期：

- 返回 `dry_run_ready`
- `recommendations` 非空，且推荐目标 topic 为 `dtp.consistency.reconcile`
- `ops.chain_projection_gap` 仍保持原 `gap_status / resolution_summary`
- 当前请求不会写出新的 `dtp.consistency.reconcile` outbox 事件
- `audit.audit_event + audit.access_audit + ops.system_log` 三层留痕齐备

## `AUD-018` 手工交易链监控验证

1. 启动服务：

```bash
set -a
source infra/docker/.env.local
set +a

APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core-bin
```

2. 先跑 live smoke，确认最小订单图、checkpoint、external fact、fairness incident、projection gap、chain anchor 与 API/审计联查都可用：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_trade_monitor_db_smoke -- --nocapture
```

3. 对现存订单执行总览查询：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/trade-monitor/orders/<order_id>" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud018-manual-overview' \
  -H 'x-trace-id: trace-aud018-manual'
```

4. 对同一订单执行 checkpoints 过滤查询：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/trade-monitor/orders/<order_id>/checkpoints?checkpoint_status=pending&lifecycle_stage=delivery&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud018-manual-checkpoints' \
  -H 'x-trace-id: trace-aud018-manual'
```

5. 回查读审计与系统日志：

```sql
SELECT access_mode, target_type, target_id::text
FROM audit.access_audit
WHERE request_id IN ('req-aud018-manual-overview', 'req-aud018-manual-checkpoints')
ORDER BY created_at;

SELECT message_text, structured_payload
FROM ops.system_log
WHERE request_id IN ('req-aud018-manual-overview', 'req-aud018-manual-checkpoints')
ORDER BY created_at;
```

6. 回查正式对象：

```sql
SELECT status, proof_commit_state, external_fact_status, reconcile_status
FROM trade.order_main
WHERE order_id = '<order_id>'::uuid;

SELECT checkpoint_code, lifecycle_stage, checkpoint_status
FROM ops.trade_lifecycle_checkpoint
WHERE order_id = '<order_id>'::uuid
ORDER BY COALESCE(occurred_at, expected_by, created_at) DESC,
         created_at DESC,
         trade_lifecycle_checkpoint_id DESC
LIMIT 10;

SELECT fact_type, provider_type, receipt_status
FROM ops.external_fact_receipt
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
ORDER BY COALESCE(confirmed_at, received_at, occurred_at) DESC,
         external_fact_receipt_id DESC
LIMIT 10;

SELECT incident_type, severity, lifecycle_stage, status
FROM risk.fairness_incident
WHERE order_id = '<order_id>'::uuid
ORDER BY created_at DESC, fairness_incident_id DESC
LIMIT 10;

SELECT gap_type, gap_status, chain_id
FROM ops.chain_projection_gap
WHERE order_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_projection_gap_id DESC
LIMIT 10;
```

7. 预期：

- overview 能看到最近 `checkpoint / external fact / fairness incident / projection gap`
- checkpoints 过滤结果只返回命中的 `delivery + pending`
- `audit.access_audit + ops.system_log` 至少各 2 条
- trade monitor 读取的是正式 PostgreSQL authority，不依赖旁路 Kafka 消费结果

## `AUD-019` 手工外部事实查询 / confirm 验证

1. 启动服务：

```bash
set -a
source infra/docker/.env.local
set +a

APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core-bin
```

2. 先跑 live smoke，确认 `ops.external_fact_receipt` 查询 / confirm、step-up、审计与“不直接改业务主状态”边界都已生效：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_external_fact_confirm_db_smoke -- --nocapture
```

3. 查询 pending receipt：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/external-facts?order_id=<order_id>&receipt_status=pending&provider_type=mock_payment_provider&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud019-list' \
  -H 'x-trace-id: trace-aud019'
```

4. 对其中一条 pending receipt 发起 confirm：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/external-facts/<external_fact_receipt_id>/confirm" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud019-confirm' \
  -H 'x-trace-id: trace-aud019' \
  -H 'x-step-up-challenge-id: <confirm_step_up_id>' \
  -d '{
    "confirm_result": "confirmed",
    "reason": "operator verified payment callback",
    "operator_note": "provider callback digest matches expected invoice"
  }'
```

5. 回查 receipt 与主状态边界：

```sql
SELECT receipt_status,
       confirmed_at,
       metadata -> 'manual_confirmation' ->> 'confirm_result' AS confirm_result,
       metadata -> 'manual_confirmation' ->> 'reason' AS confirm_reason,
       metadata -> 'rule_evaluation' ->> 'status' AS rule_evaluation_status
FROM ops.external_fact_receipt
WHERE external_fact_receipt_id = '<external_fact_receipt_id>'::uuid;

SELECT external_fact_status, reconcile_status, proof_commit_state
FROM trade.order_main
WHERE order_id = '<order_id>'::uuid;
```

6. 回查审计与系统日志：

```sql
SELECT action_name, result_code, ref_type, ref_id::text
FROM audit.audit_event
WHERE request_id = 'req-aud019-confirm'
  AND action_name = 'ops.external_fact.confirm';

SELECT access_mode, target_type, target_id::text
FROM audit.access_audit
WHERE request_id IN ('req-aud019-list', 'req-aud019-confirm')
ORDER BY created_at;

SELECT message_text, structured_payload
FROM ops.system_log
WHERE request_id IN ('req-aud019-list', 'req-aud019-confirm')
ORDER BY created_at;
```

7. 预期：

- 查询能返回 `ops.external_fact_receipt` 正式分页结果
- confirm 只允许 `receipt_status='pending'`
- confirm 成功后 `receipt_status / confirmed_at / metadata.manual_confirmation / metadata.rule_evaluation` 更新
- `audit.audit_event(action_name='ops.external_fact.confirm')`、`audit.access_audit(access_mode='confirm')`、`ops.system_log` 同时可回查
- `trade.order_main.external_fact_status` 保持原值，不被 confirm 直接改写

## `AUD-013 / AUD-014` 手工 fabric-adapter 验证

1. 启动适配器：

```bash
set -a
source infra/docker/.env.local
set +a

./scripts/fabric-adapter-run.sh
```

2. 准备最小 `chain_anchor + anchor_batch` 测试对象。

建议使用 `psql -v ON_ERROR_STOP=1`，避免 SQL 失败后 shell 仍继续向 Kafka 注入消息，污染 append-only 审计留痕：

```sql
INSERT INTO chain.chain_anchor (chain_anchor_id, chain_id, anchor_type, ref_type, ref_id, digest, status)
VALUES
  ('77777777-7777-4777-8777-777777777777'::uuid, 'fabric-local', 'audit_anchor_batch', 'anchor_batch', '66666666-6666-4666-8666-666666666666'::uuid, 'aud014b-root-evidence', 'pending'),
  ('88888888-8888-4888-8888-888888888888'::uuid, 'fabric-local', 'order_summary', 'chain_anchor', NULL, 'aud014b-root-order', 'pending'),
  ('99999999-9999-4999-8999-999999999999'::uuid, 'fabric-local', 'authorization_summary', 'chain_anchor', NULL, 'aud014b-root-auth', 'pending'),
  ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'::uuid, 'fabric-local', 'acceptance_summary', 'chain_anchor', NULL, 'aud014b-root-accept', 'pending');

INSERT INTO audit.anchor_batch (
  anchor_batch_id, batch_scope, chain_id, record_count, batch_root, status, chain_anchor_id, metadata
) VALUES (
  '66666666-6666-4666-8666-666666666666'::uuid,
  'audit_event',
  'fabric-local',
  1,
  'aud014b-root-evidence',
  'retry_requested',
  '77777777-7777-4777-8777-777777777777'::uuid,
  '{}'::jsonb
);
```

3. 用 `kcat` 注入四类 canonical 事件：

```bash
cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.audit.anchor
{"event_id":"aud014b-anchor-evt-kcat","event_type":"audit.anchor_requested","event_version":1,"occurred_at":"2026-04-22T05:02:00Z","producer_service":"platform-core.audit","aggregate_type":"audit.anchor_batch","aggregate_id":"66666666-6666-4666-8666-666666666666","request_id":"req-aud014b-anchor-kcat","trace_id":"trace-aud014b-anchor-kcat","idempotency_key":"idemp-aud014b-anchor-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"anchor_batch_id":"66666666-6666-4666-8666-666666666666","batch_scope":"audit_event","chain_id":"fabric-local","record_count":1,"batch_root":"aud014b-root-evidence","anchor_status":"retry_requested"},"anchor_batch_id":"66666666-6666-4666-8666-666666666666","batch_scope":"audit_event","chain_id":"fabric-local","record_count":1,"batch_root":"aud014b-root-evidence","anchor_status":"retry_requested","chain_anchor_id":"77777777-7777-4777-8777-777777777777"}
JSON

cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.fabric.requests
{"event_id":"aud014b-order-evt-kcat","event_type":"fabric.proof_submit_requested","event_version":1,"occurred_at":"2026-04-22T05:02:01Z","producer_service":"platform-core.integration","aggregate_type":"chain.chain_anchor","aggregate_id":"88888888-8888-4888-8888-888888888888","request_id":"req-aud014b-order-kcat","trace_id":"trace-aud014b-order-kcat","idempotency_key":"idemp-aud014b-order-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"chain_anchor_id":"88888888-8888-4888-8888-888888888888","chain_id":"fabric-local","summary_type":"order_summary","summary_digest":"aud014b-root-order"},"chain_anchor_id":"88888888-8888-4888-8888-888888888888","chain_id":"fabric-local","summary_type":"order_summary","summary_digest":"aud014b-root-order"}
JSON

cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.fabric.requests
{"event_id":"aud014b-auth-evt-kcat","event_type":"fabric.proof_submit_requested","event_version":1,"occurred_at":"2026-04-22T05:02:02Z","producer_service":"platform-core.integration","aggregate_type":"chain.chain_anchor","aggregate_id":"99999999-9999-4999-8999-999999999999","request_id":"req-aud014b-auth-kcat","trace_id":"trace-aud014b-auth-kcat","idempotency_key":"idemp-aud014b-auth-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"chain_anchor_id":"99999999-9999-4999-8999-999999999999","chain_id":"fabric-local","summary_type":"authorization_summary","summary_digest":"aud014b-root-auth"},"chain_anchor_id":"99999999-9999-4999-8999-999999999999","chain_id":"fabric-local","summary_type":"authorization_summary","summary_digest":"aud014b-root-auth"}
JSON

cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.fabric.requests
{"event_id":"aud014b-accept-evt-kcat","event_type":"fabric.proof_submit_requested","event_version":1,"occurred_at":"2026-04-22T05:02:03Z","producer_service":"platform-core.integration","aggregate_type":"chain.chain_anchor","aggregate_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","request_id":"req-aud014b-accept-kcat","trace_id":"trace-aud014b-accept-kcat","idempotency_key":"idemp-aud014b-accept-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"chain_anchor_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","chain_id":"fabric-local","summary_type":"acceptance_summary","summary_digest":"aud014b-root-accept"},"chain_anchor_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","chain_id":"fabric-local","summary_type":"acceptance_summary","summary_digest":"aud014b-root-accept"}
JSON
```

4. 回查结果：

```sql
SELECT request_id,
       metadata ->> 'submission_kind' AS submission_kind,
       metadata ->> 'contract_name' AS contract_name,
       receipt_payload ->> 'transaction_name' AS transaction_name
FROM ops.external_fact_receipt
WHERE request_id IN (
  'req-aud014b-anchor-kcat',
  'req-aud014b-order-kcat',
  'req-aud014b-auth-kcat',
  'req-aud014b-accept-kcat'
)
ORDER BY request_id;

SELECT request_id, count(*)
FROM audit.audit_event
WHERE request_id IN (
  'req-aud014b-anchor-kcat',
  'req-aud014b-order-kcat',
  'req-aud014b-auth-kcat',
  'req-aud014b-accept-kcat'
)
  AND action_name = 'fabric.adapter.submit'
GROUP BY request_id
ORDER BY request_id;

SELECT request_id, count(*)
FROM ops.system_log
WHERE request_id IN (
  'req-aud014b-anchor-kcat',
  'req-aud014b-order-kcat',
  'req-aud014b-auth-kcat',
  'req-aud014b-accept-kcat'
)
  AND message_text = 'fabric adapter accepted submit event'
GROUP BY request_id
ORDER BY request_id;

SELECT chain_anchor_id::text, anchor_type, status, tx_hash, reconcile_status
FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '77777777-7777-4777-8777-777777777777'::uuid,
  '88888888-8888-4888-8888-888888888888'::uuid,
  '99999999-9999-4999-8999-999999999999'::uuid,
  'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'::uuid
)
ORDER BY chain_anchor_id;

```

5. 清理：

```sql
DELETE FROM ops.external_fact_receipt
WHERE request_id IN (
  'req-aud014b-anchor-kcat',
  'req-aud014b-order-kcat',
  'req-aud014b-auth-kcat',
  'req-aud014b-accept-kcat'
);

DELETE FROM audit.anchor_batch
WHERE anchor_batch_id = '66666666-6666-4666-8666-666666666666'::uuid;

DELETE FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '77777777-7777-4777-8777-777777777777'::uuid,
  '88888888-8888-4888-8888-888888888888'::uuid,
  '99999999-9999-4999-8999-999999999999'::uuid,
  'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'::uuid
);
```

`audit.audit_event` 与 `ops.system_log` 保留作为 append-only 留痕。

## `AUD-005` 手工回放验证

1. 启动服务：

```bash
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
APP_PORT=18080 \
cargo run -p platform-core-bin
```

2. 准备最小业务对象与 step-up challenge。下面 SQL 使用 `gen_random_uuid()` 生成一组独立数据，返回：
   - `order_id`
   - `audit_user_id`
   - `replay_challenge_id`

```sql
WITH buyer AS (
  INSERT INTO core.organization (org_name, org_type, status, metadata)
  VALUES ('aud005-buyer-manual', 'enterprise', 'active', '{}'::jsonb)
  RETURNING org_id
), seller AS (
  INSERT INTO core.organization (org_name, org_type, status, metadata)
  VALUES ('aud005-seller-manual', 'enterprise', 'active', '{}'::jsonb)
  RETURNING org_id
), asset AS (
  INSERT INTO catalog.asset (owner_org_id, asset_name, asset_type, lifecycle_status, metadata)
  SELECT seller.org_id, 'aud005-asset-manual', 'dataset', 'published', '{}'::jsonb
  FROM seller
  RETURNING asset_id
), asset_version AS (
  INSERT INTO catalog.asset_version (asset_id, version_no, version_label, schema_json, metadata)
  SELECT asset.asset_id, 1, 'v1', '{}'::jsonb, '{}'::jsonb
  FROM asset
  RETURNING asset_version_id
), product AS (
  INSERT INTO catalog.product (owner_org_id, product_name, product_type, status, metadata)
  SELECT seller.org_id, 'aud005-product-manual', 'dataset', 'published', '{}'::jsonb
  FROM seller
  RETURNING product_id
), sku AS (
  INSERT INTO catalog.sku (
    product_id,
    asset_version_id,
    sku_code,
    sku_type,
    billing_mode,
    price_json,
    entitlement_json,
    status,
    metadata
  )
  SELECT product.product_id, asset_version.asset_version_id, 'AUD005-MANUAL', 'DATA', 'ONE_TIME',
         '{"amount":"88.00","currency":"CNY"}'::jsonb, '{}'::jsonb, 'active', '{}'::jsonb
  FROM product, asset_version
  RETURNING sku_id
), order_main AS (
  INSERT INTO trade.order_main (
    buyer_org_id,
    seller_org_id,
    product_id,
    sku_id,
    order_no,
    status,
    payment_status,
    delivery_status,
    acceptance_status,
    settlement_status,
    dispute_status,
    total_amount,
    currency,
    price_snapshot_json,
    metadata
  )
  SELECT buyer.org_id, seller.org_id, product.product_id, sku.sku_id, 'AUD005-MANUAL',
         'created', 'pending', 'pending', 'pending', 'pending', 'none',
         88.00, 'CNY', '{}'::jsonb, '{}'::jsonb
  FROM buyer, seller, product, sku
  RETURNING order_id
), audit_user AS (
  INSERT INTO core.user_account (
    org_id, login_id, display_name, user_type, status, mfa_status, email, attrs
  )
  SELECT buyer.org_id, 'aud005-manual-user', 'AUD005 Manual User',
         'human', 'active', 'verified', 'aud005-manual@example.com', '{}'::jsonb
  FROM buyer
  RETURNING user_id
), audit_seed AS (
  INSERT INTO audit.audit_event (
    event_schema_version, event_class, domain_name, ref_type, ref_id,
    actor_type, actor_id, actor_org_id, tenant_id, action_name, result_code,
    request_id, trace_id, event_time, sensitivity_level, metadata
  )
  SELECT 'v1', 'business', 'trade', 'order', order_main.order_id,
         'user', audit_user.user_id, buyer.org_id, buyer.org_id::text,
         'trade.order.create', 'accepted',
         'req-aud005-manual-seed', 'trace-aud005-manual-seed', now(), 'normal', '{}'::jsonb
  FROM order_main, audit_user, buyer
  RETURNING ref_id
), replay_challenge AS (
  INSERT INTO iam.step_up_challenge (
    user_id,
    challenge_type,
    target_action,
    target_ref_type,
    target_ref_id,
    challenge_status,
    expires_at,
    completed_at,
    metadata
  )
  SELECT audit_user.user_id,
         'mock_otp',
         'audit.replay.execute',
         'order',
         order_main.order_id,
         'verified',
         now() + interval '10 minutes',
         now(),
         jsonb_build_object('seed', 'aud005-manual')
  FROM audit_user, order_main
  RETURNING step_up_challenge_id
)
SELECT
  (SELECT order_id::text FROM order_main) AS order_id,
  (SELECT user_id::text FROM audit_user) AS audit_user_id,
  (SELECT step_up_challenge_id::text FROM replay_challenge) AS replay_challenge_id;
```

3. 创建 replay job：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/replay-jobs \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud005-manual-create' \
  -H 'x-trace-id: trace-aud005-manual-create' \
  -H 'x-step-up-challenge-id: <replay_challenge_id>' \
  -d '{
    "replay_type": "state_replay",
    "ref_type": "order",
    "ref_id": "<order_id>",
    "reason": "manual replay verification",
    "dry_run": true,
    "options": {
      "trigger": "manual_http_check"
    }
  }'
```

4. 读取 replay job：

```bash
curl -sS http://127.0.0.1:18080/api/v1/audit/replay-jobs/<replay_job_id> \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud005-manual-get' \
  -H 'x-trace-id: trace-aud005-manual-get'
```

## 回查清单

1. 回查 replay 任务与结果：

```sql
SELECT replay_type, ref_type, ref_id::text, dry_run, status, request_reason
FROM audit.replay_job
WHERE replay_job_id = '<replay_job_id>'::uuid;

SELECT step_name, result_code
FROM audit.replay_result
WHERE replay_job_id = '<replay_job_id>'::uuid
ORDER BY created_at, replay_result_id;
```

预期：

- `dry_run=true`
- `status=completed`
- 存在 4 条结果：`target_snapshot`、`audit_timeline`、`evidence_projection`、`execution_policy`
- `execution_policy.result_code='AUDIT_REPLAY_DRY_RUN_ONLY'`

2. 回查正式审计与访问留痕：

```sql
SELECT action_name, result_code
FROM audit.audit_event
WHERE ref_type = 'replay_job'
  AND ref_id = '<replay_job_id>'::uuid
ORDER BY event_time, audit_id;

SELECT access_mode, request_id
FROM audit.access_audit
WHERE target_type = 'replay_job'
  AND target_id = '<replay_job_id>'::uuid
ORDER BY created_at, access_audit_id;

SELECT message_text
FROM ops.system_log
WHERE request_id IN ('req-aud005-manual-create', 'req-aud005-manual-get')
ORDER BY created_at, system_log_id;
```

预期：

- `audit.audit_event` 至少存在 `audit.replay.requested`、`audit.replay.completed`
- `audit.access_audit.access_mode='replay'`
- `ops.system_log` 包含：
  - `audit replay job executed: POST /api/v1/audit/replay-jobs`
  - `audit replay lookup executed: GET /api/v1/audit/replay-jobs/{id}`

3. 回查 MinIO replay report：

```sql
SELECT options_json ->> 'report_storage_uri'
FROM audit.replay_job
WHERE replay_job_id = '<replay_job_id>'::uuid;
```

预期：

- `report_storage_uri` 指向 `s3://evidence-packages/replays/<ref_type>/<ref_id>/replay-<job_id>.json`
- 对象可正常读取
- 内容包含：
  - `recommendation=dry_run_completed`
  - `results[*].step_name`
  - `step_up.challenge_id`
  - `target.order_id=<order_id>`

## `AUD-006` 手工 legal hold 验证

1. 准备 step-up challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.legal_hold.manage',
  'order',
  '<order_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud006-manual-create')
)
RETURNING step_up_challenge_id::text;
```

2. 创建 legal hold：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/legal-holds \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud006-manual-create' \
  -H 'x-trace-id: trace-aud006-manual-create' \
  -H 'x-step-up-challenge-id: <create_challenge_id>' \
  -d '{
    "hold_scope_type": "order",
    "hold_scope_id": "<order_id>",
    "reason_code": "regulator_investigation",
    "metadata": {
      "ticket": "AUD-OPS-006"
    }
  }'
```

3. 再次创建同一 scope，确认冲突：

预期：

- 返回 `409`
- 错误码为 `AUDIT_LEGAL_HOLD_ACTIVE`

4. 为释放动作准备新的 challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.legal_hold.manage',
  'legal_hold',
  '<legal_hold_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud006-manual-release')
)
RETURNING step_up_challenge_id::text;
```

5. 释放 legal hold：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/legal-holds/<legal_hold_id>/release \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud006-manual-release' \
  -H 'x-trace-id: trace-aud006-manual-release' \
  -H 'x-step-up-challenge-id: <release_challenge_id>' \
  -d '{
    "reason": "manual review cleared hold"
  }'
```

6. 回查主记录与 scope 状态：

```sql
SELECT status, requested_by::text, approved_by::text, released_at, metadata ->> 'release_reason'
FROM audit.legal_hold
WHERE legal_hold_id = '<legal_hold_id>'::uuid;

SELECT COUNT(*)::bigint
FROM audit.legal_hold
WHERE hold_scope_type = 'order'
  AND hold_scope_id = '<order_id>'::uuid
  AND status = 'active';
```

预期：

- 创建后 `status=active`
- 释放后 `status=released`
- `approved_by=<audit_user_id>`
- `metadata.release_reason='manual review cleared hold'`
- 若上述活跃 hold 计数在释放后重新查询，应返回 `0`
- 历史 `audit.evidence_item / audit.evidence_package` 行保持 append-only，不作为当前 hold 状态权威源

## `AUD-007` 手工锚定批次验证

1. 查询 failed anchor batches：

```bash
curl -sS 'http://127.0.0.1:18080/api/v1/audit/anchor-batches?anchor_status=failed&batch_scope=audit_event&chain_id=fabric-local' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud007-manual-list' \
  -H 'x-trace-id: trace-aud007-manual-list'
```

预期：

- 至少返回 1 条 failed batch
- `items[0]` 中可见 `tx_hash / anchor_status / chain_id`

2. 为 retry 准备 step-up challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.anchor.manage',
  'anchor_batch',
  '<anchor_batch_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud007-manual-retry')
)
RETURNING step_up_challenge_id::text;
```

3. 触发 retry：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/anchor-batches/<anchor_batch_id>/retry \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud007-manual-retry' \
  -H 'x-trace-id: trace-aud007-manual-retry' \
  -H 'x-step-up-challenge-id: <retry_challenge_id>' \
  -d '{
    "reason": "retry failed batch after gateway timeout",
    "metadata": {
      "ticket_id": "AUD-OPS-007"
    }
  }'
```

4. 回查 batch 与 outbox：

```sql
SELECT
  status,
  metadata ->> 'previous_status',
  metadata -> 'retry_request' ->> 'reason'
FROM audit.anchor_batch
WHERE anchor_batch_id = '<anchor_batch_id>'::uuid;

SELECT
  target_topic,
  event_type,
  aggregate_type,
  payload ->> 'anchor_status',
  payload ->> 'previous_anchor_status'
FROM ops.outbox_event
WHERE request_id = 'req-aud007-manual-retry'
ORDER BY created_at DESC, outbox_event_id DESC
LIMIT 1;
```

预期：

- `audit.anchor_batch.status='retry_requested'`
- `metadata.previous_status='failed'`
- `ops.outbox_event.target_topic='dtp.audit.anchor'`
- `event_type='audit.anchor_requested'`
- `payload.anchor_status='retry_requested'`

5. 回查审计与系统日志：

```sql
SELECT COUNT(*)::bigint
FROM audit.audit_event
WHERE request_id = 'req-aud007-manual-retry'
  AND action_name = 'audit.anchor.retry';

SELECT COUNT(*)::bigint
FROM audit.access_audit
WHERE request_id IN ('req-aud007-manual-list', 'req-aud007-manual-retry');

SELECT message_text
FROM ops.system_log
WHERE request_id IN ('req-aud007-manual-list', 'req-aud007-manual-retry')
ORDER BY created_at;
```

预期：

- `audit.audit_event` 至少 1 条
- `audit.access_audit` 覆盖 list + retry
- `ops.system_log` 含 list 与 retry 两条记录

## `AUD-020` 手工公平性事件查询 / handle 验证

1. 准备一条最小公平性事件：

```sql
INSERT INTO ops.trade_lifecycle_checkpoint (
  trade_lifecycle_checkpoint_id,
  order_id,
  ref_type,
  ref_id,
  checkpoint_code,
  lifecycle_stage,
  checkpoint_status,
  occurred_at,
  source_type,
  request_id,
  trace_id,
  metadata
) VALUES (
  '<checkpoint_id>'::uuid,
  '<order_id>'::uuid,
  'order',
  '<order_id>'::uuid,
  'delivery_follow_up',
  'delivery',
  'pending',
  now() - interval '5 minutes',
  'system',
  'req-aud020-manual-list',
  'trace-aud020-manual',
  jsonb_build_object('source', 'aud020-manual')
);

INSERT INTO ops.external_fact_receipt (
  external_fact_receipt_id,
  order_id,
  ref_type,
  ref_id,
  fact_type,
  provider_type,
  provider_reference,
  receipt_status,
  receipt_payload,
  receipt_hash,
  occurred_at,
  request_id,
  trace_id,
  metadata
) VALUES (
  '<external_fact_receipt_id>'::uuid,
  '<order_id>'::uuid,
  'order',
  '<order_id>'::uuid,
  'payment_callback',
  'mock_payment_provider',
  'provider-ref-aud020-manual',
  'pending',
  jsonb_build_object('manual', true),
  'aud020-manual-receipt-hash',
  now() - interval '7 minutes',
  'req-aud020-manual-list',
  'trace-aud020-manual',
  jsonb_build_object('source', 'aud020-manual')
);

INSERT INTO risk.fairness_incident (
  fairness_incident_id,
  order_id,
  ref_type,
  ref_id,
  incident_type,
  severity,
  lifecycle_stage,
  detected_by_type,
  source_checkpoint_id,
  source_receipt_id,
  status,
  auto_action_code,
  assigned_role_key,
  assigned_user_id,
  resolution_summary,
  request_id,
  trace_id,
  metadata
) VALUES (
  '<fairness_incident_id>'::uuid,
  '<order_id>'::uuid,
  'order',
  '<order_id>'::uuid,
  'seller_delivery_delay',
  'high',
  'delivery',
  'rule_engine',
  '<checkpoint_id>'::uuid,
  '<external_fact_receipt_id>'::uuid,
  'open',
  'notify_ops',
  'platform_risk_settlement',
  '<operator_user_id>'::uuid,
  'awaiting manual review',
  'req-aud020-manual-list',
  'trace-aud020-manual',
  jsonb_build_object('source', 'aud020-manual')
);

INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<operator_user_id>'::uuid,
  'mock_otp',
  'risk.fairness_incident.handle',
  'fairness_incident',
  '<fairness_incident_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud020-manual')
)
RETURNING step_up_challenge_id::text;
```

2. 查询公平性事件：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/fairness-incidents?order_id=<order_id>&incident_type=seller_delivery_delay&severity=high&fairness_incident_status=open&assigned_role_key=platform_risk_settlement&assigned_user_id=<operator_user_id>&page=1&page_size=20" \
  -H 'x-role: platform_risk_settlement' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud020-manual-list' \
  -H 'x-trace-id: trace-aud020-manual'
```

3. 处理公平性事件：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/fairness-incidents/<fairness_incident_id>/handle" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_risk_settlement' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud020-manual-handle' \
  -H 'x-trace-id: trace-aud020-manual' \
  -H 'x-step-up-challenge-id: <handle_step_up_id>' \
  -d '{
    "action": "close",
    "resolution_summary": "manual review confirmed delivery delay risk",
    "auto_action_override": "notify_ops",
    "freeze_settlement": true,
    "freeze_delivery": false,
    "create_dispute_suggestion": true
  }'
```

4. 回查处理结果与“无业务主状态副作用”：

```sql
SELECT status,
       auto_action_code,
       metadata -> 'handling' ->> 'action' AS handled_action,
       metadata -> 'linked_action_plan' ->> 'status' AS action_plan_status,
       metadata -> 'linked_action_plan' ->> 'execution_mode' AS execution_mode,
       closed_at IS NOT NULL AS closed
FROM risk.fairness_incident
WHERE fairness_incident_id = '<fairness_incident_id>'::uuid;

SELECT settlement_status, delivery_status, dispute_status
FROM trade.order_main
WHERE order_id = '<order_id>'::uuid;

SELECT COUNT(*)::bigint
FROM audit.audit_event
WHERE request_id = 'req-aud020-manual-handle'
  AND action_name = 'risk.fairness_incident.handle'
  AND result_code = 'close';

SELECT COUNT(*)::bigint
FROM audit.access_audit
WHERE request_id IN ('req-aud020-manual-list', 'req-aud020-manual-handle')
  AND target_type IN ('fairness_incident_query', 'fairness_incident');

SELECT COUNT(*)::bigint
FROM ops.system_log
WHERE request_id IN ('req-aud020-manual-list', 'req-aud020-manual-handle')
  AND message_text IN (
    'ops lookup executed: GET /api/v1/ops/fairness-incidents',
    'risk fairness incident handle executed: POST /api/v1/ops/fairness-incidents/{id}/handle'
  );
```

预期：

- `risk.fairness_incident.status='closed'`
- `metadata.handling.action='close'`
- `metadata.linked_action_plan.status='suggestion_recorded'`
- `metadata.linked_action_plan.execution_mode='suggestion_only'`
- `trade.order_main.settlement_status / delivery_status / dispute_status` 保持原值
- `audit.audit_event = 1`
- `audit.access_audit = 2`
- `ops.system_log = 2`

## `AUD-021` 手工投影缺口查询 / 关闭验证

1. 先写入或复用一条 `ops.chain_projection_gap(gap_status='open')`，并准备 verified step-up challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<operator_user_id>'::uuid,
  'mock_otp',
  'ops.projection_gap.manage',
  'projection_gap',
  '<chain_projection_gap_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud021-manual')
)
RETURNING step_up_challenge_id::text;
```

2. 查询投影缺口：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/projection-gaps?aggregate_type=order&aggregate_id=<order_id>&order_id=<order_id>&chain_id=fabric-local&gap_type=missing_callback&gap_status=open&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud021-manual-list' \
  -H 'x-trace-id: trace-aud021-manual'
```

3. 先做 dry-run：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/projection-gaps/<chain_projection_gap_id>/resolve" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud021-manual-dry-run' \
  -H 'x-trace-id: trace-aud021-manual' \
  -H 'x-step-up-challenge-id: <resolve_step_up_id>' \
  -d '{
    "dry_run": true,
    "resolution_mode": "callback_confirmed",
    "reason": "preview close projection gap after callback verification"
  }'
```

记录返回里的 `state_digest`。

4. 再执行真实关闭：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/projection-gaps/<chain_projection_gap_id>/resolve" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud021-manual-execute' \
  -H 'x-trace-id: trace-aud021-manual' \
  -H 'x-step-up-challenge-id: <resolve_step_up_id>' \
  -d '{
    "dry_run": false,
    "resolution_mode": "callback_confirmed",
    "reason": "confirmed callback backfilled into projection gap",
    "expected_state_digest": "<state_digest_from_dry_run>"
  }'
```

5. 回查缺口关闭与“无 reconcile outbox 副作用”：

```sql
SELECT gap_status,
       resolved_at IS NOT NULL AS resolved,
       resolution_summary -> 'manual_resolution' ->> 'reason' AS resolve_reason,
       resolution_summary -> 'manual_resolution' ->> 'resolution_mode' AS resolution_mode
FROM ops.chain_projection_gap
WHERE chain_projection_gap_id = '<chain_projection_gap_id>'::uuid;

SELECT COUNT(*)::bigint
FROM audit.audit_event
WHERE request_id IN ('req-aud021-manual-dry-run', 'req-aud021-manual-execute')
  AND action_name = 'ops.projection_gap.resolve';

SELECT COUNT(*)::bigint
FROM audit.access_audit
WHERE request_id IN ('req-aud021-manual-list', 'req-aud021-manual-dry-run', 'req-aud021-manual-execute')
  AND target_type IN ('projection_gap_query', 'projection_gap');

SELECT COUNT(*)::bigint
FROM ops.system_log
WHERE request_id IN ('req-aud021-manual-list', 'req-aud021-manual-dry-run', 'req-aud021-manual-execute')
  AND message_text IN (
    'ops lookup executed: GET /api/v1/ops/projection-gaps',
    'ops projection gap resolve prepared: POST /api/v1/ops/projection-gaps/{id}/resolve',
    'ops projection gap resolve executed: POST /api/v1/ops/projection-gaps/{id}/resolve'
  );

SELECT COUNT(*)::bigint
FROM ops.outbox_event
WHERE request_id IN ('req-aud021-manual-dry-run', 'req-aud021-manual-execute')
  AND target_topic = 'dtp.consistency.reconcile';
```

预期：

- `ops.chain_projection_gap.gap_status='resolved'`
- `resolved_at` 非空
- `audit.audit_event = 2`
- `audit.access_audit = 3`
- `ops.system_log = 3`
- `ops.outbox_event(target_topic='dtp.consistency.reconcile') = 0`

## 清理约束

- 业务测试数据可清理：`trade.order_main` 及本手工步骤创建的临时 `core.organization / catalog.*` scope 图数据
- 与高风险动作审计链绑定的 `core.user_account / iam.step_up_challenge` 在当前运行态不做强删；删除它们会触发 FK 尝试回写 append-only `audit.audit_event`
- 审计数据不清理：`audit.audit_event`、`audit.access_audit`、`ops.system_log`、`audit.replay_job`、`audit.replay_result`、`audit.legal_hold`、`audit.anchor_batch`、MinIO replay report 及相关 evidence snapshot 按 append-only 或审计保留规则保留
- `ops.outbox_event` 若为本次手工 retry 临时产生的待发布记录，只允许按唯一 `request_id` 精确删除；不得宽泛清库

## AUD-022 搜索运维控制面

验收范围：

- `GET /api/v1/ops/search/sync`
- `POST /api/v1/ops/search/reindex`
- `POST /api/v1/ops/search/cache/invalidate`
- `POST /api/v1/ops/search/aliases/switch`
- `GET /api/v1/ops/search/ranking-profiles`
- `PATCH /api/v1/ops/search/ranking-profiles/{id}`

正式验收点：

- 统一使用 `Authorization: Bearer <access_token>`，不再接受 `x-role`
- `reindex / aliases/switch / ranking-profiles/{id}` 必须要求 verified `X-Step-Up-Token`
- 所有写接口必须要求 `X-Idempotency-Key`
- `GET /sync`、`GET /ranking-profiles` 必须写入 `audit.access_audit + ops.system_log`
- 写接口必须写入 `audit.audit_event + audit.access_audit + ops.system_log`
- `POST /reindex` 必须真实写入 `search.index_sync_task(sync_status='queued')`
- `POST /cache/invalidate` 必须真实删除 `datab:v1:search:catalog:*`
- `POST /aliases/switch` 必须真实更新 `search.index_alias_binding.active_index_name` 与 OpenSearch alias target
- `PATCH /ranking-profiles/{id}` 必须真实更新 `search.ranking_profile`
- 搜索域错误必须收口到 `SEARCH_QUERY_INVALID / SEARCH_BACKEND_UNAVAILABLE / SEARCH_RESULT_STALE` 与写权限专属 forbid code

自动化验证：

- `IAM_JWT_PARSER=keycloak_claims cargo test -p platform-core route_tests -- --nocapture`
- `SEARCH_DB_SMOKE=1 IAM_JWT_PARSER=keycloak_claims DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core search_api_and_ops_db_smoke -- --nocapture`

自动化 smoke 覆盖：

- 目录搜索两次，确认 `cache_hit=false -> true`
- Redis 缓存 key 真实存在后再被 `POST /cache/invalidate` 删除
- reindex step-up 通过后，`search.index_sync_task` 真实排队
- `GET /sync` 能回查到刚排队的任务
- `GET /ranking-profiles` 能列出正式 profile
- `PATCH /ranking-profiles/{id}` 真实更新数据库
- `POST /aliases/switch` 真实切换 OpenSearch alias 与 `search.index_alias_binding`
- 六条路径都回查 `audit.audit_event / audit.access_audit / ops.system_log`

## 当前未覆盖项

- `AUD-023+` 后续剩余 AUD 高风险控制面

进入对应批次后，必须在本文件继续追加，不得把本文件视为 `AUD` 全阶段完成证明。
