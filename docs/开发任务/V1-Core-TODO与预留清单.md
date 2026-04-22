# V1-Core TODO 与预留清单

本文件用于汇总当前阶段未实现但已明确识别的缺口、技术债和 `V2/V3` 预留点。

## 记录规则

- 代码里出现的 `TODO(...)` 必须同步登记到本文件
- 每条记录都要标出是否阻塞继续开发
- 已完成补齐的 TODO 不删除，改状态为 `closed`

## 字段说明

- 编号
- 对应任务编号
- 类型
- 模块
- 文件路径
- 当前状态
- 原因
- 后续补齐条件
- 是否阻塞继续开发
- 计划补齐阶段
- 责任建议

---

## TODO 模板

| 编号 | 对应任务编号 | 类型 | 模块 | 文件路径 | 当前状态 | 原因 | 后续补齐条件 | 是否阻塞继续开发 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TODO-001 | TASK-ID | V1-gap / V2-reserved / V3-reserved / tech-debt | module-name | path/to/file | open | 简述原因 | 简述补齐条件 | yes / no |

## 当前阻塞记录

| 编号 | 对应任务编号 | 类型 | 模块 | 文件路径 | 当前状态 | 原因 | 后续补齐条件 | 是否阻塞继续开发 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TODO-ENV-043-001 | ENV-043 | V1-gap | env-compose | `infra/docker/docker-compose.apps.local.example.yml` | closed | 已在 `BATCH-057` 补齐应用层 compose 占位文件，并完成 `docker compose config` 与本地自检/烟雾验证。 | 无；后续如进入应用容器化联调阶段，按该示例替换占位镜像为真实服务镜像。 | no |
| TODO-CTX-019-001 | CTX-019 | V1-gap | context | `docs/00-context/service-to-module-map.md` | closed | 任务清单要求的交付文件在仓库中缺失，导致 `CORE-032` 依赖文档基线不完整。 | 已在 `BATCH-050` 补齐 `docs/00-context/service-to-module-map.md` 并纳入审批。 | no |
| TODO-CTX-020-001 | CTX-020 | V1-gap | context | `docs/00-context/local-deployment-boundary.md` | closed | 任务清单要求的交付文件在仓库中缺失，导致本地部署边界冻结依据不完整。 | 已在 `BATCH-050` 补齐 `docs/00-context/local-deployment-boundary.md` 并纳入审批。 | no |
| TODO-DB-034-001 | DB-034 | V1-gap | db-seed | `db/seeds/031_sku_trigger_matrix.sql` | closed | 已在 `BATCH-078` 补齐 `BIL-023` 交付文档并完成 `031_sku_trigger_matrix.sql` + `verify-seed-031.sh` 落地，阻塞链解除。 | 无；后续若扩展 SKU，按同一矩阵表与文档双写规则追加并回归验证。 | no |
| TODO-DB-007-001 | DB-007 | V2-reserved | db-billing-profitshare | `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql` | accepted | 已在 `BATCH-184` 补齐 `payment.sub_merchant_binding` 与 `payment.split_instruction`；但 `payment.split_instruction.reward_id` 仅按 `V1` 方案 A 以可空 `uuid` 占位，不建立到 `billing.reward_record` 的外键。 | 进入 `V2` 分润阶段后落地 `billing.reward_record`，将 `reward_id` 升级为正式外键，并同步更新 migration、表字典、全集成基线与相关校验脚本。 | no |
| TODO-DB-007-002 | DB-007 | V1-gap | db-risk-freeze | `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql` | closed | `DB-007` 任务描述要求落地“风险处置、冻结记录”，但初版 `040` 迁移缺失 `risk.freeze_ticket` 与 `risk.governance_action_log`。 | 已在 `BATCH-184` 补齐两张表、索引、触发器与 roundtrip 校验，完成后关闭。 | no |
| TODO-PROC-BIL-001 | BIL-* | tech-debt | process-governance | `docs/开发任务/V1-Core-实施进度日志.md` | accepted | 历史执行顺序发生跨阶段偏移：在 IAM 阶段未完成前已进入并实现 `BIL-001~BIL-005` 与 `TRADE-030`。当前已获人工批准继续推进 `CAT~TRADE~DLV`，但必须保留该偏移追溯并在进入 BIL 阶段时执行一致性复核。 | 在 BIL 阶段完成“历史已实现任务 vs 冻结文档”逐条复核并补齐不一致项，形成专项审批记录后关闭。 | no |

## 当前非阻塞记录

| 编号 | 对应任务编号 | 类型 | 模块 | 文件路径 | 当前状态 | 原因 | 后续补齐条件 | 是否阻塞继续开发 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TODO-CORE-028-001 | CORE-028 | V1-gap | db | `apps/platform-core/crates/db/src/lib.rs` | closed | 已补齐 `OrderRepository` 的 PostgreSQL 实现与 `ORDER_REPOSITORY_BACKEND` 运行时切换装配。 | 无；后续仅需在真实业务表结构联调时补充更细粒度 SQL 回归样例。 | no |
| TODO-IAM-002-REPO-001 | IAM-002 | V1-gap | iam | `apps/platform-core/src/modules/iam/repository.rs` | closed | 初版实现为 API 直连 SQL，仓储层边界不清晰。 | 已在 `BATCH-084` 补齐 `PostgresIamRepository` 并由 API 复用，完成后关闭。 | no |
| TODO-IAM-003-JWT-001 | IAM-003 | V1-gap | iam/auth | `apps/platform-core/crates/auth/src/lib.rs` | closed | 仅有 mock token 解析，未满足 Keycloak claims 解析接入要求。 | 已在 `BATCH-084` 增加 `KeycloakClaimsJwtParser`（本地 claims 解析模式）并接入 `/api/v1/auth/me`。 | no |
| TODO-IAM-011-PATH-001 | IAM-011 | V1-gap | iam | `packages/openapi/iam.yaml` | closed | step-up 创建接口路径与冻结协议存在漂移。 | 已在 `BATCH-084` 补齐 `/api/v1/iam/step-up/challenges` 并保留 `/check` 兼容路径（deprecated）。 | no |
| TODO-IAM-016-TX-001 | IAM-016 | tech-debt | iam | `apps/platform-core/src/modules/iam/api.rs` | closed | 多个写接口存在“业务写入成功但审计写入失败导致接口失败”的事务一致性风险。 | 已在 `BATCH-084` 将 IAM 写接口统一改为“业务+审计同事务提交”。 | no |
| TODO-PROC-IAM-APPROVAL-001 | IAM-001~IAM-020 | tech-debt | process-governance | `docs/开发任务/V1-Core-人工审批记录.md` | closed | IAM 批次审批存在“口头通过”但审批文件缺少结构化条目，审计追溯不足。 | 已在 `BATCH-084` 补录 `BATCH-079~083` 审批记录条目。 | no |
| TODO-PROC-LOG-001 | process-governance | tech-debt | process-governance | `docs/开发任务/V1-Core-实施进度日志.md; docs/开发任务/V1-Core-实施进度日志-P1.md; docs/开发任务/V1-Core-实施进度日志-P2.md; docs/开发任务/V1-Core-实施进度日志-P3.md` | closed | 历史流程文档引用了不存在的总日志路径，实施日志权威入口与分卷规则未冻结，换手与审批追溯存在歧义。 | 已冻结“入口页 + 分卷正文”规则：`V1-Core-实施进度日志.md` 作为唯一入口页，`P1/P2/P3` 作为分卷正文，并同步更新流程文档、README、审批模板和基线说明。后续新增分卷前必须先更新入口页。 | no |
| TODO-BIL-011-001 | BIL-011 | V2-reserved | billing-split-management | `apps/platform-core/src/modules/billing/**; packages/openapi/billing.yaml` | accepted | `BIL-011` 的 `V1` 口径已明确为“人工打款执行落地 + 人工分账对象完整占位”；不提前实现 `V2` 的渠道分账/子商户扩展与分账管理接口。 | 进入 `V2` 分润阶段后补齐 `payment.split.manage` 控制面、分账配置/执行管理接口，并把 `payment.split_instruction`、`payment.sub_merchant_binding` 与正式 `billing.reward_record` / 分润对象衔接起来。 | no |
| TODO-AUD-OPENAPI-001 | AUD-003~AUD-028 | V1-gap | audit-ops-contract | `packages/openapi/audit.yaml; packages/openapi/ops.yaml; packages/openapi/search.yaml; docs/02-openapi/README.md` | accepted | `AUD-003` 已补齐 `GET /api/v1/audit/orders/{id}`、`GET /api/v1/audit/traces`，`AUD-004` 已补齐 `POST /api/v1/audit/packages/export`，`AUD-005` 已补齐 `POST /api/v1/audit/replay-jobs`、`GET /api/v1/audit/replay-jobs/{id}`，`AUD-006` 已补齐 `POST /api/v1/audit/legal-holds`、`POST /api/v1/audit/legal-holds/{id}/release`，`AUD-007` 已补齐 `GET /api/v1/audit/anchor-batches`、`POST /api/v1/audit/anchor-batches/{id}/retry`，`AUD-008` 已补齐 `GET /api/v1/ops/outbox`、`GET /api/v1/ops/dead-letters`，`AUD-009` 已补齐 outbox publisher 的 runbook、compose/port/观测文档与 Billing bridge 边界说明，`AUD-010` 已补齐 `POST /api/v1/ops/dead-letters/{id}/reprocess`，`AUD-011` 已补齐 `GET /api/v1/ops/consistency/{refType}/{refId}`，`AUD-012` 已补齐 `POST /api/v1/ops/consistency/reconcile`，`AUD-013` 已补齐 `fabric-adapter` 的 Go 运行入口与 request consume / receipt write-back runbook，`AUD-014` 已补齐四类 Fabric request handler 的契约与回查字段，`AUD-015` 已补齐 `fabric-event-listener` 的 callback topic / receipt metadata / runbook 口径，`AUD-016` 已补齐 `packages/openapi/iam.yaml` / `docs/02-openapi/iam.yaml` 中 Fabric 身份签发 / 吊销与证书吊销的正式错误码与执行边界说明，`AUD-017` 已补齐真实 `fabric-test-network` provider、pinned test-network / Go 链码 / live smoke runbook 与本地启动说明，`AUD-018` 已补齐 `GET /api/v1/ops/trade-monitor/orders/{orderId}`、`GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints`，`AUD-019` 已补齐 `GET /api/v1/ops/external-facts`、`POST /api/v1/ops/external-facts/{id}/confirm`，`AUD-020` 已补齐 `GET /api/v1/ops/fairness-incidents`、`POST /api/v1/ops/fairness-incidents/{id}/handle`，`AUD-021` 已补齐 `GET /api/v1/ops/projection-gaps`、`POST /api/v1/ops/projection-gaps/{id}/resolve`，`AUD-022` 已补齐 `search.yaml` 中 `GET /api/v1/catalog/search`、`GET /api/v1/ops/search/sync`、`POST /api/v1/ops/search/reindex`、`POST /api/v1/ops/search/aliases/switch`、`POST /api/v1/ops/search/cache/invalidate`、`GET/PATCH /api/v1/ops/search/ranking-profiles` 的正式 Bearer / idempotency / step-up / `SEARCH_*` 契约，并同步归档到 `docs/02-openapi/search.yaml`，`AUD-023` 已补齐 `GET /api/v1/ops/observability/overview`、`GET /api/v1/ops/logs/query`、`GET /api/v1/ops/logs`、`POST /api/v1/ops/logs/export`、`GET /api/v1/ops/traces/{traceId}`、`GET /api/v1/ops/alerts`、`GET /api/v1/ops/incidents`、`GET /api/v1/ops/slos` 的正式契约与示例，`AUD-024` 已补齐 `GET /api/v1/developer/trace` 的 `order_id / event_id / tx_hash` 单 selector 契约、`developer.trace.read`、tenant scope、`recent_logs / recent_outbox_events / recent_dead_letters / recent_audit_traces` 响应与归档副本；后续仅剩 `AUD-025+` 的其他高风险公共接口。 | 继续在 `AUD-025+` 批次补齐剩余公共控制面，并同步校验 `event_type / target_topic / aggregate_type / callback_event_id / provider_request_id / provider_reference / alias_name / tx_hash` 示例。 | no |
| TODO-AUD-TEST-001 | AUD-013~028 | V1-gap | audit-notif-fabric-cases | `docs/05-test-cases/README.md; docs/05-test-cases/audit-consistency-cases.md; docs/04-runbooks/notification-worker.md; docs/04-runbooks/fabric-local.md; docs/04-runbooks/fabric-adapter.md; docs/04-runbooks/fabric-event-listener.md; docs/04-runbooks/fabric-ca-admin.md; docs/04-runbooks/audit-replay.md; docs/04-runbooks/audit-legal-hold.md; docs/04-runbooks/audit-anchor-batches.md; docs/04-runbooks/audit-ops-outbox-dead-letters.md; docs/04-runbooks/audit-dead-letter-reprocess.md; docs/04-runbooks/outbox-publisher.md; docs/04-runbooks/audit-consistency-lookup.md; docs/04-runbooks/audit-consistency-reconcile.md; docs/04-runbooks/audit-trade-monitor.md; docs/04-runbooks/audit-external-facts.md; docs/04-runbooks/audit-fairness-incidents.md; docs/04-runbooks/audit-projection-gaps.md; docs/04-runbooks/audit-observability.md; docs/04-runbooks/developer-trace.md; docs/04-runbooks/search-reindex.md` | accepted | `AUD-013` 已把 `fabric-adapter` 的宿主机真实 `kcat + psql` smoke、`ops.external_fact_receipt + audit.audit_event + ops.system_log + chain.chain_anchor` 回查和 runbook 落盘，`AUD-014` 已把四类摘要 handler 的 `submission_kind / contract_name / transaction_name` smoke 和验收矩阵补齐，`AUD-015` 已把 callback listener 的 `dtp.fabric.callbacks`、`provider_request_id / callback_event_id / provider_status / payload_hash` 回查、`audit.anchor_batch / chain.chain_anchor` 更新与 runbook 落盘，`AUD-016` 已把 `fabric-ca-admin` 的 `step-up -> platform-core IAM API -> Go service -> PostgreSQL / audit / system_log` smoke、runbook 与验收矩阵补齐，`AUD-017` 已把 pinned `test-network`、Go 链码、`check-fabric-local.sh` 和 `fabric-adapter-live-smoke.sh` 的账本 + PG + 审计 + 系统日志回查补齐，`AUD-018` 已把 trade monitor 总览 / checkpoints 的 `audit_trade_monitor_db_smoke`、runbook 与验收矩阵补齐，`AUD-019` 已把 `external-facts` 查询 / confirm 的 `audit_external_fact_confirm_db_smoke`、宿主机 `curl + psql` 联调步骤、`audit-external-facts.md` 与验收矩阵补齐，`AUD-020` 已把 `fairness-incidents` 查询 / handle 的 `audit_fairness_incident_handle_db_smoke`、宿主机 `curl + psql` 联调步骤、`audit-fairness-incidents.md` 与验收矩阵补齐，`AUD-021` 已把 `projection-gaps` 查询 / resolve 的 `audit_projection_gap_resolve_db_smoke`、宿主机 `curl + psql` 联调步骤、`audit-projection-gaps.md` 与验收矩阵补齐，`AUD-022` 已把搜索运维控制面的 route tests、`search_api_and_ops_db_smoke`、宿主机 runbook 与验收矩阵补齐，`AUD-023` 已把 `observability_api_db_smoke`、宿主机 `curl + psql + MinIO + 观测栈` 联调步骤、`audit-observability.md` 与验收矩阵补齐，`AUD-024` 已把 `developer_trace_api_db_smoke`、`developer.trace.read`/tenant scope 负例、宿主机 `curl + psql` 联调步骤、`developer-trace.md` 与验收矩阵补齐；后续剩余缺口转入 `AUD-025+` 的后续高风险动作。 | 进入 `AUD-025+` 代码实现批次后，继续在 `docs/05-test-cases/audit-consistency-cases.md` 追加剩余控制面清单，并把相关 runbook 的实操步骤、联查入口和补救流程补完整。 | no |
| TODO-AUD-FABRIC-001 | AUD-013; AUD-026 | V1-gap | fabric-adapter-reliability | `services/fabric-adapter/internal/service/processor.go; docs/04-runbooks/fabric-adapter.md; docs/05-test-cases/audit-consistency-cases.md` | accepted | `AUD-013` 已打通 Go 版 `fabric-adapter` 的正式 topic 消费与回执回写，`AUD-026` 也已重新复验 `fabric-adapter` Go 单测、`check-fabric-local.sh` 与 `fabric-adapter-live-smoke.sh`；但当前仍是 at-least-once 基础框架，尚未把 `ops.consumer_idempotency_record + Redis` 短锁接入 Fabric consumer 幂等路径，人工 `kafka-console-producer` 注入时仍可能出现重复消息噪音。 | 在后续 Fabric consumer 可靠性批次中，为 `fabric-adapter` 补齐正式 idempotency record、Redis 短锁与重复投递隔离验证，再关闭本条 gap。 | no |
| TODO-AUD-WRITER-001 | AUD-001; AUD-002; AUD-029 | V1-gap | audit-writer-authority | `docs/开发任务/问题修复任务/A06-Audit-Kit-统一模型漂移.md; docs/开发任务/问题修复任务/A14-AUD-历史模块统一Writer与证据桥接缺口.md` | accepted | 当前批次只把统一 `audit/evidence` authority model 与历史模块收口义务写入任务清单和问题修复文档，不提前修改 `catalog / search / billing dispute` 运行时代码。 | 进入 `AUD` 代码实现批次后，必须先收口 `audit-kit / EvidenceItem / EvidenceManifest` 正式模型，再实现统一 `audit writer / evidence writer`，并桥接 `support.evidence_object` 与 `audit.evidence_item / evidence_manifest`，清理历史 ad-hoc 审计 SQL。 | no |
| TODO-NOTIF-OBS-001 | NOTIF-004; ENV-026; ENV-029 | V1-gap | notif-observability-scrape | `infra/docker/monitoring/prometheus.yml; scripts/check-observability-stack.sh; apps/platform-core/**; infra/docker/docker-compose.local.yml` | closed | 已在 `BATCH-236`（`AUD-023`）补齐 `platform-core` 正式 `/metrics` 端点（`platform_core_http_requests_total / platform_core_http_request_duration_seconds`），把 `mock-payment-provider` 的 Prometheus 抓取改为真实 `/metrics` stub，重建 `Prometheus / MinIO / mock-payment-provider` 后 `./scripts/check-observability-stack.sh` 全绿，通用观测栈误报关闭。 | 无；后续仅在新增宿主机进程或 exporter 时按同一 runbook / Prometheus 口径扩展。 | no |
| TODO-SEARCHREC-CONSUMER-001 | AUD-008; AUD-010; AUD-026; SEARCHREC-015; SEARCHREC-020 | V1-gap | searchrec-consumer-reliability | `docs/开发任务/问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md; docs/04-runbooks/search-reindex.md; docs/04-runbooks/recommendation-runtime.md; docs/05-test-cases/search-rec-cases.md` | accepted | 当前批次只把 SEARCHREC consumer 的正式口径收敛到任务清单与文档，不提前修改 `search-indexer / recommendation-aggregator` 运行时代码。 | 进入 `SEARCHREC / AUD` 代码实现批次后，必须补齐 `event_id` 幂等、`ops.consumer_idempotency_record`、`ops.dead_letter_event + dtp.dead-letter` 双层隔离、失败后 offset 提交策略与 `reprocess` 路径，并补 worker 侧副作用测试。 | no |
| TODO-SEARCHREC-AUTH-001 | AUD-022; SEARCHREC-018; SEARCHREC-019; SEARCHREC-015; SEARCHREC-016; SEARCHREC-017 | V1-gap | searchrec-auth-contract | `docs/开发任务/问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md; docs/04-runbooks/search-reindex.md; docs/04-runbooks/recommendation-runtime.md` | accepted | `AUD-022` 已完成搜索运维控制面的统一 Bearer 鉴权、正式权限点、必要 `X-Idempotency-Key / X-Step-Up-Token`、`audit.audit_event + audit.access_audit + ops.system_log` 与 `SEARCH_*` 错误码收口；剩余缺口主要位于 `SEARCHREC` 运行态与推荐运维控制面。 | 进入 `SEARCHREC-018/019` 与后续推荐 / search worker 批次后，继续把统一鉴权门面、正式权限点、高风险写接口的 `step-up / 审计 / 搜索域错误码` 推进到 recommendation runtime 与 consumer 侧。 | no |
| TODO-SEARCHREC-CONTRACT-TEST-001 | SEARCHREC-015; SEARCHREC-016; SEARCHREC-017 | V1-gap | searchrec-openapi-test | `packages/openapi/search.yaml; packages/openapi/recommendation.yaml; docs/02-openapi/search.yaml; docs/02-openapi/recommendation.yaml; docs/05-test-cases/search-rec-cases.md` | accepted | 当前批次只登记未来补齐义务，不提前伪造 `SEARCHREC` 的 OpenAPI 头、权限、错误码与测试矩阵完成态。 | 进入 `SEARCHREC` 代码实现批次后，必须补齐 `Authorization / X-Idempotency-Key / 必要 X-Step-Up-Token / 审计 / SEARCH_* 错误码` 的 OpenAPI 与测试验收项，并校验其与实现一致。 | no |
| TODO-SEARCHREC-FALLBACK-001 | SEARCHREC-001; SEARCHREC-004 | V1-gap | searchrec-runtime-mode | `docs/开发任务/问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md; docs/04-runbooks/local-startup.md` | accepted | 当前批次只强调“生产基线用 OpenSearch，`local/demo` 允许 PG fallback，最终仍回 PG 校验”的冻结目标，不提前修改搜索运行时代码。 | 进入 `SEARCHREC` 代码实现批次后，必须补齐搜索候选源运行模式、自检分流、`local/demo` 的 PG 搜索投影 fallback，并验证 `staging / production` 仍强制 OpenSearch。 | no |
| TODO-AUD-EVENT-001 | AUD-030; AUD-031 | V1-gap | audit-event-authority | `docs/开发任务/问题修复任务/A02-统一事件-Envelope-与路由权威源.md; docs/开发任务/问题修复任务/A05-Outbox-Publisher-DLQ-统一闭环缺口.md; docs/04-runbooks/kafka-topics.md` | accepted | 当前批次只把统一事件 `Envelope / event_route_policy / outbox publisher / 双层 DLQ` 的实现义务写入任务清单，不提前修改 trigger、publisher 或历史旁路消费代码。 | 进入 `AUD` 代码实现批次后，必须统一 `event_id / event_type / aggregate_type / target_topic`、启用 `ops.event_route_policy` 作为唯一路由权威、补齐 `outbox_publish_attempt`，并清理模块直接消费 `ops.outbox_event` 的旁路。 | no |
| TODO-NOTIF-CONTRACT-001 | NOTIF-014 | V1-gap | notif-contract-test | `docs/开发任务/问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md; docs/开发任务/问题修复任务/A11-测试与Smoke口径误报风险.md; docs/04-runbooks/notification-worker.md; docs/05-test-cases/README.md; docs/05-test-cases/notification-cases.md` | closed | 已在 `BATCH-213` 补齐 `docs/05-test-cases/notification-cases.md`，并同步 README / runbook 引用与通知验收项；`mock-log`、幂等、重试、DLQ、人工补发与审计联查义务已落盘。 | 无；后续若通知场景扩展到 `email / webhook` 实接，再按新阶段要求追加矩阵与运行态回归。 | no |
| TODO-SEARCHREC-ALIAS-001 | SEARCHREC-021 | V1-gap | search-alias-authority | `docs/开发任务/问题修复任务/A08-搜索Alias权威源与阶段边界冲突.md; docs/04-runbooks/search-reindex.md` | accepted | 当前批次只把搜索 alias 权威源与阶段边界收口义务写入任务清单，不提前修改运行时默认值、初始化脚本或运维接口。 | 进入 `SEARCHREC` 代码实现批次后，必须统一 `search.index_alias_binding`、初始化脚本、运行时默认值和 alias switch 运维能力，并验证其仍属于 V1 最小运维能力。 | no |
| TODO-ENV-NAMING-001 | ENV-058 | V1-gap | env-naming | `docs/开发任务/问题修复任务/A12-配置项与资源命名漂移.md; docs/开发准备/配置项与密钥管理清单.md; docs/开发准备/本地开发环境与中间件部署清单.md; docs/04-runbooks/local-startup.md; docs/04-runbooks/secrets-policy.md` | closed | 已冻结数据库 / MinIO / Keycloak 的 bootstrap 与运行时入口映射，并同步 `infra/docker/.env.local`、`docker-compose.local.yml`、runbook 与本地脚本；本条命名漂移 gap 已完成收口。 | 若后续重新引入 `PG_HOST / PG_PORT / PG_DATABASE / PG_USER / PG_PASSWORD` 或 `KEYCLOAK_ADMIN_USERNAME` 作为正式主配置名，需重新开启 TODO 并同步修正执行源。 | no |
| TODO-TEST-CANONICAL-001 | TEST-028 | V1-gap | test-smoke-authority | `docs/开发任务/问题修复任务/A01-Kafka-Topic-口径统一.md; docs/开发任务/问题修复任务/A11-测试与Smoke口径误报风险.md; docs/05-test-cases/README.md` | accepted | 当前批次只把 canonical smoke / contract checker 的实现义务写入任务清单，不提前生成新的 smoke 脚本或 CI 作业。 | 进入 `TEST / NOTIF / AUD / SEARCHREC` 代码实现批次后，必须补齐能显式拦截旧 topic、旧命名、占位鉴权和骨架接口的 smoke / contract checker。 | no |

## 示例记录

以下两行仅用于展示填写格式，不代表仓库当前真实任务状态，也不应引用真实任务编号。

| 编号 | 对应任务编号 | 类型 | 模块 | 文件路径 | 当前状态 | 原因 | 后续补齐条件 | 是否阻塞继续开发 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TODO-EXAMPLE-001 | TASK-EXAMPLE-001 | V2-reserved | example-module | `path/to/example.rs` | accepted | 示例：当前阶段只保留最小边界，不展开下一阶段增强能力。 | 进入后续阶段时再补齐扩展能力，并同步更新接口、错误码与审计事件。 | no |
| TODO-EXAMPLE-002 | TASK-EXAMPLE-002 | V1-gap | example-module | `path/to/example.rs` | open | 示例：当前只完成占位或格式约定，尚未进入正式实现闭环。 | 进入对应阶段后补齐实现、测试和文档，再关闭该记录。 | yes |

## 状态说明

- `open`：尚未处理
- `accepted`：已知缺口，当前允许继续
- `blocked`：阻塞继续开发
- `closed`：已补齐并验证

## 批次更新记录

- `BATCH-220`（`AUD-007`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 anchor batch 查看 / retry、对应 runbook 与验收矩阵的最新状态。
- `BATCH-221`（`AUD-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `GET /api/v1/ops/outbox`、`GET /api/v1/ops/dead-letters`、`audit-ops-outbox-dead-letters.md` 与 `AUD-008` 验收矩阵的最新状态。
- `BATCH-222`（`AUD-009`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `outbox-publisher` worker、`outbox-publisher.md`、宿主机/compose 入口、Prometheus scrape / alert 与 `AUD-009` 验收矩阵的最新状态。
- `BATCH-223`（`AUD-010`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `POST /api/v1/ops/dead-letters/{id}/reprocess`、`audit-dead-letter-reprocess.md`、真实 `curl + psql` 联调与 `AUD-010` 验收矩阵的最新状态。
- `BATCH-224`（`AUD-011`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `GET /api/v1/ops/consistency/{refType}/{refId}`、`audit-consistency-lookup.md`、真实 `curl + psql` 联调与 `AUD-011` 验收矩阵的最新状态。
- `BATCH-225`（`AUD-012`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `POST /api/v1/ops/consistency/reconcile`、`audit-consistency-reconcile.md`、真实 `curl + psql` dry-run 联调与 `AUD-012` 验收矩阵的最新状态。
- `BATCH-226`（`AUD-013`）：新增 `V1-gap` 项 `TODO-AUD-FABRIC-001`（`fabric-adapter` consumer 幂等 / Redis 短锁留待 `AUD-026` 闭环）；并把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `fabric-adapter` Go 运行入口、`fabric-adapter.md` 与 `AUD-013` 验收矩阵的最新状态。
- `BATCH-227`（`AUD-014`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `fabric-adapter` 四类摘要 handler、`submission_kind / contract_name / transaction_name` 实际回查、`fabric-adapter.md` 与 `AUD-014` 验收矩阵的最新状态。
- `BATCH-228`（`AUD-015`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `fabric-event-listener` 的 `dtp.fabric.callbacks`、callback metadata 字段、`fabric-event-listener.md` 与 `AUD-015` 验收矩阵的最新状态。
- `BATCH-229`（`AUD-016`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `fabric-ca-admin` 的 Go 执行面、`fabric-ca-admin.md`、`iam_fabric_ca_admin_db_smoke` 与 `AUD-016` 验收矩阵的最新状态。
- `BATCH-231`（`AUD-018`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 trade monitor 总览 / checkpoints、`audit-trade-monitor.md` 与 `AUD-018` 验收矩阵的最新状态。
- `BATCH-232`（`AUD-019`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `GET /api/v1/ops/external-facts`、`POST /api/v1/ops/external-facts/{id}/confirm`、`audit-external-facts.md` 与 `AUD-019` 验收矩阵的最新状态。
- `BATCH-233`（`AUD-020`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `GET /api/v1/ops/fairness-incidents`、`POST /api/v1/ops/fairness-incidents/{id}/handle`、`audit-fairness-incidents.md` 与 `AUD-020` 验收矩阵的最新状态。
- `BATCH-234`（`AUD-021`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `GET /api/v1/ops/projection-gaps`、`POST /api/v1/ops/projection-gaps/{id}/resolve`、`audit-projection-gaps.md` 与 `AUD-021` 验收矩阵的最新状态。
- `BATCH-236`（`AUD-023`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `GET /api/v1/ops/observability/overview`、`GET /api/v1/ops/logs/query`、`GET /api/v1/ops/logs`、`POST /api/v1/ops/logs/export`、`GET /api/v1/ops/traces/{traceId}`、`GET /api/v1/ops/alerts`、`GET /api/v1/ops/incidents`、`GET /api/v1/ops/slos`、`audit-observability.md` 与 `AUD-023` 验收矩阵的最新状态，并关闭历史观测 gap `TODO-NOTIF-OBS-001`。
- `BATCH-237`（`AUD-024`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 `GET /api/v1/developer/trace`、`developer.trace.read`、`developer-trace.md` 与 `AUD-024` 验收矩阵的最新状态，并明确该接口只读取 PostgreSQL 正式对象，Go/Fabric 服务继续作为外围回写层。
- `BATCH-219`（`AUD-006`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 更新到包含 legal hold create/release、对应 runbook 与验收矩阵的最新状态。
- `BATCH-002`（`CTX-001`, `CTX-002`, `CTX-003`, `CTX-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-003`（`CTX-005`, `CTX-006`, `CTX-007`, `CTX-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-004`（`CTX-009`, `CTX-010`, `CTX-011`, `CTX-012`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-005`（`CTX-022`, `CTX-023`, `CTX-024`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-006`（`CTX-013`, `CTX-014`, `CTX-015`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-007`（`CTX-016`, `CTX-017`, `CTX-018`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-008`（`CTX-021`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-009`（`BOOT-021`, `BOOT-022`, `BOOT-023`, `BOOT-024`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-010`（`BOOT-029`, `BOOT-030`, `BOOT-031`, `BOOT-032`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-011`（`BOOT-033`, `BOOT-034`, `BOOT-035`, `BOOT-036`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-012`（`BOOT-001`, `BOOT-002`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-013`（`BOOT-003`, `BOOT-004`, `BOOT-005`, `BOOT-006`, `BOOT-007`, `BOOT-008`, `BOOT-009`, `BOOT-010`, `BOOT-011`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-014`（`BOOT-012`, `BOOT-013`, `BOOT-014`, `BOOT-015`, `BOOT-016`, `BOOT-017`, `BOOT-018`, `BOOT-019`, `BOOT-020`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-015`（`ENV-002`, `ENV-003`, `ENV-004`, `ENV-005`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-016`（`ENV-006`, `ENV-007`, `ENV-008`, `ENV-009`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-017`（`ENV-010`, `ENV-011`, `ENV-012`, `ENV-013`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-018`（`ENV-014`, `ENV-015`, `ENV-016`, `ENV-017`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-019`（`ENV-018`, `ENV-019`, `ENV-020`, `ENV-021`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-020`（`ENV-022`, `ENV-023`, `ENV-024`, `ENV-025`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-021`（`ENV-026`, `ENV-027`, `ENV-028`, `ENV-029`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-022`（`ENV-030`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-023`（`ENV-031`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-024`（`ENV-032`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-025`（`ENV-033`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-026`（`ENV-034`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-027`（`ENV-035`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-028`（`ENV-036`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-029`（`ENV-037`, `ENV-038`, `ENV-039`, `ENV-040`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-030`（`ENV-041`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-031`（`ENV-044`, `ENV-045`, `ENV-046`, `ENV-047`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-032`（`ENV-048`, `ENV-049`, `ENV-050`, `ENV-051`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-033`（`ENV-052`, `ENV-053`, `ENV-054`, `ENV-055`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-034`（`ENV-056`, `ENV-057`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-035`（`ENV-001`, `ENV-042`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-036`（`ENV-043`）：新增 `V1-gap` 阻塞项 `TODO-ENV-043-001`（`CORE-032` 前置未完成）。
- `BATCH-037`（`CORE-001`, `CORE-002`, `CORE-003`, `CORE-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-038`（`CORE-005`, `CORE-006`, `CORE-007`, `CORE-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-039`（`CORE-009`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-040`（`CORE-011`, `CORE-012`, `CORE-013`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-041`（`CORE-014`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-042`（`CORE-015`, `CORE-016`, `CORE-017`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-043`（`CORE-018`, `CORE-019`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-044`（`CORE-020`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-045`（`CORE-021`, `CORE-022`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-046`（`CORE-023`, `CORE-024`, `CORE-025`, `CORE-026`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-047`（`CORE-027`, `CORE-028`, `CORE-029`）：补记 `V1-gap` 项 `TODO-CORE-028-001`（非阻塞，追踪运行时持久化仓储接入）。
- `BATCH-048`（`CORE-030`, `CORE-031`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-049`（`CORE-033`, `CORE-034`, `CORE-035`, `CORE-036`）：新增阻塞项 `TODO-CTX-019-001`、`TODO-CTX-020-001`（历史前置任务交付文件缺失，影响后续依赖链）。
- `BATCH-050`（`CTX-019`, `CTX-020`）：补齐阻塞缺口并关闭 `TODO-CTX-019-001`、`TODO-CTX-020-001`。
- `BATCH-051`（`CORE-037`, `CORE-038`, `CORE-039`, `CORE-040`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-052`（`CORE-041`, `CORE-042`, `CORE-043`, `CORE-044`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-053`（`CORE-045`, `CORE-046`, `CORE-047`, `CORE-048`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-054`（`CORE-049`, `CORE-050`, `CORE-051`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-055`（`CORE-010`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-056`（`CORE-032`）：更新阻塞项 `TODO-ENV-043-001` 的阻塞原因与补齐条件，保持状态 `blocked`（待 `CORE-032` 审批通过后解除）。
- `BATCH-057`（`ENV-043`）：关闭阻塞项 `TODO-ENV-043-001`（已补齐 compose 占位文件并通过 compose config + 本地自检 + smoke 验证）。
- `BATCH-058`（`CORE-022`, `CORE-028`）：关闭 `TODO-CORE-028-001`（已补齐 PostgreSQL 仓储实现与运行时 DI 切换），并完成启动自检对 topic/bucket/alias 的存在性探测增强。
- `BATCH-059`（`DB-001`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-060`（`DB-002`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-061`（`DB-003`, `DB-004`, `DB-005`, `DB-006`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-062`（`DB-007`, `DB-008`, `DB-009`, `DB-010`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-063`（`DB-011`, `DB-012`, `DB-013`, `DB-014`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-064`（`DB-015`, `DB-016`, `DB-017`, `DB-018`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-065`（`DB-019`, `DB-020`, `DB-021`, `DB-022`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-066`（`DB-023`, `DB-024`, `DB-025`, `DB-026`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-067`（`DB-027`, `DB-028`, `DB-029`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-068`（`DB-030`, `DB-031`, `DB-032`, `DB-033`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-069`（`DB-035`）：新增阻塞项 `TODO-DB-034-001`（`DB-034` 依赖 `BIL-023` 未完成，触发强制暂停）。
- `BATCH-070`（`BIL-001`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-071`（`BIL-002`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-072`（`BIL-003`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-073`（`BIL-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-074`（`BIL-002`, `BIL-003` 返工）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-075`（`BIL-005`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-076`（`TRADE-030`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-077`（流程纠偏：BIL 阶段冻结登记）：新增阻塞项 `TODO-PROC-BIL-001`，冻结“继续新增 BIL 任务”和“已完成 BIL 任务改动”，待 `IAM-001~IAM-020` 审批通过后再人工解冻。
- `BATCH-078`（`BIL-023`, `DB-034`）：关闭阻塞项 `TODO-DB-034-001`（已补齐 SKU 计费触发矩阵文档、`031` 种子与校验脚本并通过验证）；`TODO-PROC-BIL-001` 保持冻结，仅执行本次依赖解锁所需最小范围补齐。
- `BATCH-079`（`IAM-001`, `IAM-002`, `IAM-003`, `IAM-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-080`（`IAM-005`, `IAM-006`, `IAM-007`, `IAM-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-081`（`IAM-009`, `IAM-010`, `IAM-011`, `IAM-012`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-082`（`IAM-013`, `IAM-014`, `IAM-015`, `IAM-016`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-083`（`IAM-017`, `IAM-018`, `IAM-019`, `IAM-020`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-084`（`IAM-002`, `IAM-003`, `IAM-011`, `IAM-020` 缺口修复）：关闭 `TODO-IAM-002-REPO-001`、`TODO-IAM-003-JWT-001`、`TODO-IAM-011-PATH-001`、`TODO-IAM-016-TX-001`、`TODO-PROC-IAM-APPROVAL-001`。
- `BATCH-085`（`CAT-001`）：无新增 `V1-gap / V2-reserved / V3-reserved`；将 `TODO-PROC-BIL-001` 从 `blocked` 调整为 `accepted`，记录“已获人工批准继续后续阶段、进入 BIL 时必须执行一致性复核”的审计口径。
- `BATCH-086`（`CAT-002`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；保持 `TODO-PROC-BIL-001` 追溯约束不变。
- `BATCH-087`（`CAT-003`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-088`（`CAT-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-089`（`CAT-005`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-090`（`CAT-006`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-091`（`CAT-007`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-092`（`CAT-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-093`（`CAT-009`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-094`（`CAT-010`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-095`（`CAT-011`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-096`（`CAT-012`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-097`（`CAT-013`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-098`（`CAT-014`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-099`（`CAT-015`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-100`（`CAT-016`, `CAT-017`, `CAT-018`, `CAT-019`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-101`（`CAT-016~CAT-019` 审计修复）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-102`（`CAT-020`, `CAT-021`, `CAT-022`, `CAT-023`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-103`（`CAT-020` 返工重做，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-104`（`CAT-021`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-105`（`CAT-022`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-106`（`CAT-023`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-107`（`CAT-024`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-108`（`CAT-025`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-109`（`CAT-026`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-110`（`TRADE-001`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-111`（`TRADE-002`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-112`（`TRADE-003`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-113`（`TRADE-004`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-114`（`TRADE-005`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-115`（`TRADE-006`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-116`（`TRADE-007`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-117`（`TRADE-008`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-118`（`TRADE-009`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-119`（`TRADE-010`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-120`（`TRADE-011`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-121`（`TRADE-012`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-122`（`TRADE-013`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-123`（`TRADE-014`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-124`（`TRADE-015`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-125`（`TRADE-016`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-126`（`TRADE-017`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-127`（`TRADE-018`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-128`（`TRADE-019`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-129`（`TRADE-020`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-130`（`TRADE-021`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-131`（`TRADE-022`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-132`（`TRADE-023`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-133`（`TRADE-024`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-134`（`TRADE-025`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；修复授权聚合查询中的联表歧义列后，`scope / subject / resource / action` 最小结构已在迁移结果、订单详情聚合与生命周期快照中稳定输出；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-135`（`TRADE-026`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；合同确认链路已接入签章 provider 占位，`local/mock` 模式下通过 `provider-kit` 生成签章引用并持久化到 `contract.contract_signer.signature_digest`，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-136`（`TRADE-027`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；主交易链路集成 smoke 已覆盖下单、合同确认、锁资前校验、非法状态跳转、自动断权，并修正测试清理顺序以避免临时业务数据残留；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-137`（`TRADE-028`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`docs/02-openapi/trade.yaml` 已从 `packages/openapi/trade.yaml` 同步落盘，README 已建立引用，且路径/方法与 `order/api/mod.rs` 校验一致；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-138`（`TRADE-029`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`docs/05-test-cases/order-state-machine.md` 已按 8 个标准 SKU 落盘状态转换测试矩阵，README 已建立索引，且 `trade008~trade015` 与真实 `SHARE_RO` 迁移联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-139`（`TRADE-030`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；支付结果编排已收紧到 `created / contract_effective` 两类可支付主状态，成功链路补写 `buyer_locked_at`，并新增 success / failed / timeout / early-state-ignore 专项 smoke 与真实 webhook 联调证据；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-140`（`TRADE-031`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；统一可交付判定器已接入 8 个标准 SKU 的首个交付/开通动作，门禁通过后落库最小 `delivery.delivery_record(prepared)`，并经 `trade008~trade015`、`trade031` DB smoke 与真实 `SHARE_RO` API 联调验证；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-141`（`TRADE-032`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；五条标准链路的场景到 SKU 快照规则已落地到下单、补冻价快照、合同、授权四条主路径，歧义 SKU 未指定 `scenario_code` 时显式阻断，指定场景后会把 `scenario_snapshot/scenario_sku_snapshot` 同步写入订单、合同变量、授权策略快照；`packages/openapi/trade.yaml` 与 `docs/02-openapi/trade.yaml` 已同步更新，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-142`（`TRADE-033`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`docs/01-architecture/order-orchestration.md` 已冻结主交易链路的主状态、支付/交付/验收/结算/争议子状态、互斥关系与乱序回调保护，并已由 `docs/README.md` 建立索引；真实 webhook 联调已验证 `payment.webhook.out_of_order_ignored` 与 `order.payment.result.ignored` 双层防护，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-143`（`DLV-001`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已落地 `storage-gateway` 聚合与 MinIO 实体联调，`delivery.storage_gateway.read` 审计、订单详情/生命周期快照聚合、OpenAPI schema 与 SQLx 离线校验均通过，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-144`（`DLV-002`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`POST /api/v1/orders/{id}/deliver` 文件分支已打通 `prepared -> committed`、`storage_object / key_envelope / delivery_ticket`、订单推进到 `delivered`、`delivery.file.commit` 审计与真实 MinIO/API/DB 联调，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-145`（`DLV-003`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`GET /api/v1/orders/{id}/download-ticket` 已打通买方身份、次数/时效校验、Redis DB 3 短时票据缓存、`delivery.file.download` 审计与真实 API/Redis/MinIO/DB 联调，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-146`（`DLV-004`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；受控下载入口 `GET /api/v1/orders/{id}/download` 已接入 ticket 验证中间件、Redis/DB 双重校验、`delivery.delivery_receipt` 下载日志、`delivery.file.download` 审计与真实 MinIO/API/Redis/DB 联调，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-147`（`DLV-005`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`FILE_SUB` 订阅 API 已打通 `delivery.revision_subscription` 的创建/查询/暂停后状态同步/续订恢复，并补齐 `delivery.subscription.manage/read` 审计与真实 API 联调；运行库中 `catalog.product.subscription_cadence` 当前未落地，已按 `trade.order_main.price_snapshot_json` 与 `catalog.product.metadata.subscription_cadence` 回退兼容，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-148`（`DLV-006`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`SHARE_RO` 的 `share-grants` API 已打通 `delivery.data_share_grant + delivery.delivery_record + trade.order_main` 的共享开通/查询/撤权闭环，撤权时保留 `subscriber_ref/scope_json` 元数据并同步关闭交付记录与订单主状态；真实 API/DB 联调通过，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-149`（`DLV-007`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`POST /api/v1/orders/{id}/deliver` 已打通 `API_SUB/API_PPU` 的应用绑定、API 凭证签发、配额与限流配置，并联动 `core.application + delivery.api_credential + delivery.delivery_record + trade.order_main` 与 `delivery.api.enable` / `trade.order.api_sub.transition` / `trade.order.api_ppu.transition` 审计；真实 API/DB 联调、`cargo check/test`、`cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-150`（`DLV-008`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`GET /api/v1/orders/{id}/usage-log` 已打通 `delivery.api_usage_log + delivery.api_credential + core.application + trade.order_main` 的最小披露读取链路，补齐 `delivery.api.log.read` 权限、应用归属校验、读审计与真实 API/DB 联调；`cargo check/test`、`cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-151`（`DLV-009`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`POST /api/v1/products/{id}/query-surfaces` 已打通 `catalog.query_surface_definition + catalog.asset_version + catalog.asset_object_binding + core.execution_environment` 的查询面创建/维护链路，完成读取区域、执行环境、输出边界、卖方作用域与审计校验，并通过真实 API/DB 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-152`（`DLV-010`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`POST /api/v1/query-surfaces/{id}/templates` 已打通 QueryTemplate 版本管理、参数/输出 schema 冻结、白名单字段双写与导出策略校验，并通过真实 API/DB 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-153`（`DLV-011`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`POST /api/v1/orders/{id}/template-grants` 已打通 QuerySurface 白名单模板授权、输出边界/执行配额快照、交付记录复用与 `QRY_LITE` 状态联动，并修复更新授权时残留 `prepared delivery_record` 的一致性缺口；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-154`（`DLV-012`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`POST /api/v1/orders/{id}/template-runs` 已打通模板授权有效性、参数 schema、输出边界、风控、`delivery.query_execution_run`、MinIO 结果对象、`QRY_LITE` 订单状态推进与 `delivery.template_query.use` 审计闭环，并通过真实 API/DB/MinIO 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-155`（`DLV-013`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`GET /api/v1/orders/{id}/template-runs` 已补齐查询运行记录读取链路，稳定返回参数摘要、策略命中、审计引用与结果对象摘要，并通过真实 API/DB/MinIO 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-156`（`DLV-014`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`POST /api/v1/orders/{id}/sandbox-workspaces` 已打通 `delivery.sandbox_workspace + delivery.sandbox_session + delivery.delivery_record + trade.order_main` 的沙箱开通/更新闭环，完成执行环境、seat 用户、会话到期、导出策略与状态联动，并通过真实 API/DB 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-157`（`DLV-015`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；沙箱工作区模型已冻结并稳定持久化 `workspace / session / seat / export control / attestation` 五段结构，`delivery.sensitive_execution_policy + delivery.attestation_record` 与 `delivery.sandbox.enable` 审计已通过真实 API/DB 联调验证；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-158`（`DLV-016`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`SBX_STD` 执行环境模型已冻结 `isolation_level / export_policy / audit_policy / trusted_attestation_flag / supported_product_types / current_capacity / runtime_isolation`，并将 `gVisor` 占位字段稳定写入 API 响应、`policy_snapshot`、`trust_boundary_snapshot` 与 `delivery.sandbox.enable` 审计；真实 API/DB 联调通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-159`（`DLV-017`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`POST /api/v1/orders/{id}/deliver` 已补齐 `report` 分支，联动 `delivery.storage_object + delivery.delivery_record + delivery.report_artifact + trade.order_main` 完成报告交付、版本号、报告 hash、交付回执与 `delivery.report.commit` 审计；真实 MinIO/API/DB 联调通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-160`（`DLV-018`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；统一验收接口已在 `delivery` 子域落地，覆盖 `FILE_STD / FILE_SUB / RPT_STD` 的人工验收通过/拒收、buyer-scope 校验、交付记录回写与 `delivery.accept / delivery.reject` 审计；真实 API/DB 联调通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-161`（`DLV-019`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；文件与报告交付记录现在会稳定冻结 `watermark_mode / watermark_rule / fingerprint_fields / watermark_hash / watermark_policy.pipeline` 占位结构，订单详情/生命周期读取与写入保持一致，并已通过真实 API/DB 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-162`（`DLV-020`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已为文件交付、API 开通、共享开通、模板授权、沙箱开通、报告交付六类路径补齐标准化 `delivery.committed` outbox 事件，并通过真实 API/DB 联调确认 `dtp.outbox.domain-events` 落库；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-163`（`DLV-021`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已统一收敛下载票据、API credential、共享授权、沙箱工作区/会话与对应 `delivery_record` 的自动断权，并通过真实 API/DB/Redis 联调验证断权后缓存失效与资源不可继续访问；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-164`（`DLV-022`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；敏感执行策略接口已冻结 `policy_scope / execution_mode / output_boundary / export_control / step_up / attestation / policy_snapshot` 最小结构，并完成真实 API/DB 联调与 OpenAPI 对齐；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-165`（`DLV-023`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；结果披露审查接口已冻结 `review_status / masking_level / export_scope / approval_ticket_id / decision_snapshot` 最小结构，联动 `query_run`/`delivery_record` 审查状态并通过真实 API/DB/MinIO 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-166`（`DLV-024`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；执行证明读取与销毁/保留证明接口已冻结 `attestation / proof_snapshot / object_link` 最小结构，联动 `delivery.attestation_record`、`delivery.destruction_attestation`、`delivery.storage_object` 与订单回收状态，并通过真实 API/DB/MinIO 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-167`（`DLV-025`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已新增 Delivery/Storage/Query 聚合集成 smoke，覆盖文件下载票据、API 开通、模板授权执行、沙箱开通、报告交付与验收通过/拒收，并通过真实 API/DB/Redis/Outbox 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-168`（`DLV-026`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已为 S1~S5 五条标准链路补齐最小交付对象 manifest、示例对象与 `curl` demo 脚本，并通过资产完整性测试、真实 API/DB/Redis/Outbox/MinIO 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-169`（`DLV-027`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已生成 `docs/02-openapi/delivery.yaml` 并补充 README 索引，完成与 `packages/openapi/delivery.yaml` 的同步校验、delivery router 路径/方法一致性校验和真实 API/审计联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-170`（`DLV-028`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已生成 `docs/05-test-cases/delivery-cases.md` 并补充 README 索引，冻结交付超时、重复开通、票据过期、撤权后访问、验收失败五类用例矩阵，并通过 `dlv004/017/018/021` smoke、真实 `download-ticket -> ticket expired` 与 `reject -> delivery.reject` 联调验证；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-171`（`DLV-029`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已实现交付任务自动创建器，在 `FILE_STD / FILE_SUB / API_SUB` 与支付结果编排进入 `pending_delivery` 时自动落库 `delivery.delivery_record(status=prepared)`、写入 `creation_source / responsible_scope / retry_count / manual_takeover` 元数据并产出 `delivery.task.auto_created` outbox；修复支付 webhook 成功链路被交付门禁反向打断的回归后，`trade030/trade031/dlv002/dlv007/dlv017/dlv029` smoke 与真实 `FILE_STD/API_SUB` API 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-172`（`DLV-030`，单任务批次）：已为文件交付、共享开通、API 开通、模板执行成功、沙箱开通、报告交付、验收通过统一写入 `billing.trigger.bridge` outbox，目标主题 `billing.events`；当 `billing.sku_billing_trigger_matrix` 缺失时回退到与 `db/seeds/031_sku_trigger_matrix.sql` 一致的冻结矩阵快照，并在现库缺少 `ops.outbox_event.idempotency_key` 唯一索引时改用 `WHERE NOT EXISTS` 保持幂等；`cargo test`、SQLx 元数据、离线查询校验、`dlv002/dlv006/dlv007/dlv012/dlv014/dlv017/dlv018` smoke 与真实 `deliver -> accept` API 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-173`（`DLV-031`，单任务批次）：已新增统一 SKU 验收触发矩阵并收敛 Delivery/Trade 运行时与测试口径，修正 `FILE_SUB / SHARE_RO / API_PPU / RPT_STD` 验收模板码到冻结基线；`SHARE_RO` 成功开通进入 `share_granted / delivered / accepted`，`SBX_STD` 成功开通进入 `seat_issued / delivered / accepted`，文件/报告仍维持人工签收；`cargo test`、11 个 DB smoke、SQLx 元数据、离线查询校验与真实 `FILE_STD accept + SHARE_RO grant` API 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-174`（`BIL-001`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；Billing 支付基础控制面已收敛到冻结支付域协议，真实读写 `payment.jurisdiction_profile / payment.corridor_policy / payment.payout_preference`，补齐权限、step-up 占位、审计、DB smoke、SQLx 元数据、离线查询校验与真实 API/DB 联调；`mock-payment-provider` 可达校验通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-175`（`BIL-002`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；支付意图 create/read/cancel 已收敛到冻结支付协议，创建强制 step-up + idempotency，补齐 provider/jurisdiction/corridor/provider_account/fee_preview 预校验、详情聚合、DB smoke、SQLx 元数据、离线查询校验与真实 API/DB 联调；`mock-payment-provider` 可达校验通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-176`（`BIL-003`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。订单锁资一致性已按冻结文档补齐；补充真实 DB smoke、租户范围/价格一致性/跨订单冲突校验、`order.payment.lock(.idempotent_replay)` 审计与 OpenAPI 收敛；`TODO-PROC-BIL-001` 追溯保持。
- `BATCH-177`（`BIL-004`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；Mock Payment Provider 适配器已接入 Billing 路由，真实调用 WireMock success/fail/timeout 三类场景并串联既有 webhook 主链路；已补齐 `developer.mock_payment_case` 落库、OpenAPI、DB smoke、WireMock 校验、SQLx 元数据、离线查询校验与真实 API/DB 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-178`（`BIL-005`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；支付 webhook 主链路已补齐冻结协议字段、`payment.payment_transaction` 入库与 `payment_webhook_event.payment_transaction_id` 关联，覆盖 `processed / duplicate / rejected_signature / rejected_replay / out_of_order_ignored` 五条分支；`packages/openapi/billing.yaml` 与 `docs/02-openapi/billing.yaml` 已同步更新，`bil005` DB smoke、`cargo test`、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-179`（`BIL-006`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `BillingEvent` 领域模型与幂等仓储，真实支付 success webhook 会自动生成 `one_time_charge / recurring_charge` 账单事件并写入 `billing.events` Kafka outbox，补齐退款/赔付/人工结算/调用量收费语义校验；`bil006` DB smoke、`cargo test`、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-180`（`BIL-007`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `GET /api/v1/billing/{order_id}` 与账单聚合读仓储，返回 `BillingEvent / Settlement / Refund / Compensation / Invoice` 明细及 `tax/invoice` 占位字段；OpenAPI 补齐账单聚合 schema 与缺失的 `BillingEvent` 组件，`bil007` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-181`（`BIL-008`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已补齐 `Settlement / SettlementSummary` 领域模型与账单聚合摘要输出，`GET /api/v1/billing/{order_id}` 现在稳定返回应结金额、平台抽佣、渠道手续费、退款/赔付调整、供方应收与摘要状态；`bil008` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-182`（`BIL-009`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `POST /api/v1/refunds`、退款 DTO 与退款仓储，要求裁决结果 + step-up + 幂等键 + 审计齐备；退款执行会联动 live `mock-payment-provider`、生成 `billing_event` 与 `billing.events` outbox、更新 `settlement.refund_amount` 与争议状态；`bil009` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-183`（`BIL-010`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `POST /api/v1/compensations`、赔付 DTO 与赔付仓储，要求裁决结果 + step-up + 幂等键 + 审计齐备；赔付执行会联动 live `mock-payment-provider` 的手工打款模拟、生成 `billing_event` 与 `billing.events` outbox、更新 `settlement.compensation_amount` 与争议状态；`bil010` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；退款/赔付执行权限已收紧到平台侧高风险角色；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-184`（`DB-007` 纠偏修复）：新增 `V2-reserved` 项 `TODO-DB-007-001`，并关闭 `V1-gap` 项 `TODO-DB-007-002`；已在 `040_billing_support_risk.sql` 中补齐 `payment.sub_merchant_binding`、`payment.split_instruction`、`risk.freeze_ticket`、`risk.governance_action_log`，将 `reward_id` 明确记录为 `V1` 可空 `uuid` 占位、`V2` 再升级为 `billing.reward_record` 外键；`verify-migration-040-056.sh` 已同步纳入新表/索引/触发器校验，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `2026-04-20`（`BIL-011` 口径澄清补记）：新增 `V2-reserved` 项 `TODO-BIL-011-001`，明确 `V1` 只落“人工打款执行 + 人工分账对象完整占位”，不提前实现 `V2` 的渠道分账/子商户扩展与分账管理接口，后续进入 `V2` 分润阶段再补齐控制面与正式接口。
- `BATCH-184`（`BIL-011`，单任务批次）：已新增 `POST /api/v1/payouts/manual` 与人工打款仓储，真实调用 live `mock-payment-provider` 完成 `payment.payout_instruction + payment.split_instruction + payment.sub_merchant_binding + billing.billing_event + ops.outbox_event(billing.events)` 写入，补齐 `GET /api/v1/billing/{order_id}` 的 `payouts/split_placeholders` 聚合，并通过 `bil011` smoke、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调；`TODO-BIL-011-001` 明确保留 `V2` 分账控制面到后续阶段，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-185`（`BIL-012`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `POST /api/v1/payments/reconciliation/import`，支持 multipart 导入、文件 hash 幂等、防止同维度不同文件覆盖、`payment.reconciliation_statement / payment.reconciliation_diff` 落库与 `payment_intent / settlement_record` reconcile 状态回写；`bil012` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-186`（`BIL-013`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `POST /api/v1/cases`、`POST /api/v1/cases/{id}/evidence`、`POST /api/v1/cases/{id}/resolve`，按方案 A 将争议创建/证据上传收敛为买方侧入口，平台侧裁决强制 step-up；证据对象真实写入 MinIO，案件创建与裁决真实写入 `dtp.outbox.domain-events` 边界事件，`bil013` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-187`（`BIL-014`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已在争议开启事务内补齐 `billing.settlement_record` 冻结、`delivery.delivery_record / delivery.delivery_ticket` 中止、`trade.order_main` 分层状态更新、`risk.freeze_ticket / governance_action_log / audit.legal_hold` 留痕，并在提交后真实失效 Redis 下载票据缓存；`bil014` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `curl + psql + redis-cli` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-188`（`BIL-015`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增统一 `Settlement` 聚合器并接入 `billing_event / refund / compensation / manual payout` 写路径，保证多事件重算只保留单条 `billing.settlement_record` 且同步回写 `trade.order_main.settlement_status / settled_at`；`bil015` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `payments/intents -> order lock -> webhook -> billing read` API 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-189`（`BIL-016`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已在统一 `Settlement` 聚合器中补齐 `settlement.created / settlement.completed` Kafka outbox 边界事件，载荷固定输出 `summary_state / proof_commit_state` 并通过 `payments/intents -> order lock -> webhook -> manual payout -> billing read` 真实 API 联调验证；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-190`（`BIL-017`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已为 `API_SUB / API_PPU` 落地最小计费规则，`bill_cycle` 与 `settle_success_call` 会在订单事务内生成 `recurring_charge / usage_charge`、重算结算、写入 `billing.events` outbox，并通过真实 `curl + psql` 联调验证 `api_billing_basis` 聚合与幂等重放；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-191`（`BIL-018`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已为 `FILE_STD / FILE_SUB / SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 冻结默认计费/退款占位规则，`GET /api/v1/billing/{order_id}` 新增 `sku_billing_basis` 聚合，且 `SHARE_RO enable_share` 会在订单事务内生成占位 `one_time_charge`、重算结算并写入 `billing.events` outbox；已通过真实 `curl + psql` 联调验证 `billing.event.record.share_ro_enable` 与账单聚合；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-192`（`BIL-019`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增统一支付/账单集成 smoke，覆盖 `payment intent -> order lock -> webhook(success/failed/timeout) -> billing read` 与 `dispute -> resolve -> refund/compensation -> settlement recompute` 两条主链路，并通过真实 `curl + psql` 联调验证支付、争议、退款、赔付和账单聚合；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-193`（`BIL-020`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已确认 `docs/02-openapi/billing.yaml` 与 `packages/openapi/billing.yaml` 完全同步，`docs/02-openapi/README.md` 已纳入 Billing 归档索引，并通过路由/方法一致性校验与真实 `GET /api/v1/billing/{order_id}` 联调验证；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-194`（`BIL-021`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `docs/05-test-cases/payment-billing-cases.md`，冻结回调乱序、重复回调、重复扣费防护、争议冻结、退款/赔付重算和结算摘要 outbox 的最小回归矩阵，并通过真实 `curl + psql` 联调验证 success/duplicate/out_of_order 与 settlement freeze；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-195`（`BIL-022`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增统一 `payment_result_processor` 与 `POST /api/v1/payments/intents/{id}/poll-result`，收敛 webhook/polling 的幂等处理、乱序保护、`payment_transaction` 写入、`payment_intent.metadata` 结果来源快照、审计与 `billing.events` outbox；`bil022` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `curl + psql` 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-196`（`BIL-023`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已重写 `docs/03-db/sku-billing-trigger-matrix.md`，把 8 个标准 SKU 的计费触发点、结算周期、退款入口、赔付入口、争议冻结点和恢复结算点冻结为单一业务口径，并显式绑定 `db/seeds/031_sku_trigger_matrix.sql` 与运行时代码快照；`bil017/bil018` smoke、`verify-seed-031.sh`、SQLx 元数据、离线查询校验与真实 `GET /billing -> enable_share -> GET /billing` API 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-197`（`BIL-024`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `billing_bridge_repository` 与 `POST /api/v1/billing/{order_id}/bridge-events/process`，把 `billing.trigger.bridge` outbox 统一物化为标准 `billing.billing_event` 并更新 `billing.settlement_record / ops.outbox_event.status / audit.audit_event`；`bil024` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `accept -> bridge-events/process -> GET /billing` API 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-198`（`BIL-025`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已新增 `billing_adjustment_repository`，将拒收/争议升级冻结与裁决/人工修正释放统一建模为 `refund_adjustment(settlement_dispute_hold/release)`，并接入退款、赔付、人工打款三条执行链路；`settlement_aggregate_repository` 现显式聚合 adjustment 事件，避免净额回零后错误回退旧快照；`bil014/bil011/bil019/bil025` smoke、SQLx 元数据、离线查询校验与真实 `reject -> create case -> resolve -> manual payout` API 联调均通过；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-199`（`BIL-026`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved` 项；已为 `SHARE_RO` 补齐开通费、周期共享费、撤权退款占位与争议冻结规则，新增 `POST /api/v1/billing/{order_id}/share-ro/cycle-charge`、扩展 `sku_billing_basis` 最小口径，并通过 `bil026/dlv006/bil018` smoke、全量测试、SQLx 元数据、离线查询校验与真实 `GET /billing -> enable_share -> grant -> cycle-charge -> revoke / create case` API 联调；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-214`（`AUD-001`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已将 `audit-kit` 收口为统一 `AuditEvent` 基座，补齐强化 schema 对齐字段与 `EvidenceManifest / ReplayJob / AnchorBatch / LegalHold / AuditAccessRecord` 等正式对象，并在 `platform-core::modules::audit` 中新增共享 trace DTO、writer DB 映射与跨 crate 集成测试；`cargo check/test`、`cargo test -p audit-kit`、`cargo test -p db`、`cargo sqlx prepare --workspace` 与离线查询校验均通过。
- `BATCH-215`（`AUD-002`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已为 `audit.evidence_item / audit.evidence_manifest / audit.evidence_manifest_item` 落地统一 evidence writer foundation，并将真实“争议证据上传 -> MinIO -> support.evidence_object”链路桥接到正式审计 authority model；`cargo check/test`、`cargo test -p audit-kit`、`cargo sqlx prepare --workspace`、离线查询校验和真实 `bil013` DB + MinIO smoke 均通过。
- `BATCH-216`（`AUD-003`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已落地 `GET /api/v1/audit/orders/{id}`、`GET /api/v1/audit/traces` 正式读取控制面，补齐 `audit.trace.read` 权限与 tenant/order scope 校验、`audit.access_audit + ops.system_log` 双留痕，以及 `packages/openapi/audit.yaml` 与 `docs/02-openapi/audit.yaml` 的同步归档；`cargo check/test`、`AUD_DB_SMOKE=1 cargo test -p platform-core`、`cargo sqlx prepare --workspace`、离线查询校验与 `check-openapi-schema.sh` 均通过。`TODO-AUD-OPENAPI-001` 已收敛为仅跟踪后续导出 / replay / ops 控制面缺口。
- `BATCH-217`（`AUD-004`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已落地 `POST /api/v1/audit/packages/export` 正式导出控制面，补齐 `audit.package.export` 权限、`reason` 必填、`x-step-up-token / x-step-up-challenge-id` 校验、`audit.evidence_package + MinIO` 双写，以及 `audit.audit_event + audit.access_audit + ops.system_log` 三层留痕；`cargo check/test`、带 `AUD_DB_SMOKE=1` 的真实导出链路测试、`cargo sqlx prepare --workspace`、离线查询校验、`check-openapi-schema.sh` 与一次真实 `curl + psql` 联调均通过。`TODO-AUD-OPENAPI-001` 已进一步收敛为仅跟踪 `legal hold / replay / ops` 控制面缺口。
- `BATCH-218`（`AUD-005`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已落地 `POST /api/v1/audit/replay-jobs`、`GET /api/v1/audit/replay-jobs/{id}` 正式 replay dry-run 控制面，补齐 `audit.replay.execute / audit.replay.read` 权限、`V1 dry_run=true` 强约束、`audit.replay_job + audit.replay_result + MinIO replay report` 落盘，以及 `audit.audit_event + audit.access_audit + ops.system_log` 三层留痕；`packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/04-runbooks/audit-replay.md` 已同步落盘；`cargo check/test`、`AUD_DB_SMOKE=1 cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture`、`cargo sqlx prepare --workspace`、离线查询校验、`check-openapi-schema.sh` 与一次真实 `curl + psql + MinIO object` 联调均通过。`TODO-AUD-OPENAPI-001` 已进一步收敛为仅跟踪 `legal hold / anchor / ops` 控制面，`TODO-AUD-TEST-001` 则收敛为继续追加后续 `AUD`/Fabric/一致性矩阵。
- `BATCH-230`（`AUD-017`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；已把 `fabric-adapter` 收敛为真实 `mock / fabric-test-network` 双 provider，并新增 pinned `2.5.15 / 1.5.17` test-network 下载/补丁/链码部署脚本、Go 链码 `datab-audit-anchor`、`fabric-adapter-live-smoke.sh` 与 `check-fabric-local.sh` 账本校验；`go test ./...`、真实 live smoke、`cargo check/test`、`cargo sqlx prepare --workspace`、离线查询校验与 `check-topic-topology.sh` 均通过。`TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 已收敛为仅继续跟踪 `AUD-021 projection-gaps` 公共控制面。
- `BATCH-203`（`NOTIF-004`，单任务批次）：新增非阻塞 `V1-gap` 项 `TODO-NOTIF-OBS-001`；本批已完成支付成功 -> 待交付通知模板与 buyer/seller/ops 差异化发送逻辑、真实 webhook -> canonical outbox -> Kafka topic replay -> `notification-worker` 消费验证，以及 Keycloak / Tempo / Prometheus / Grafana / Alertmanager 本地观测基线修复；`notification-worker` 指标链路已打通，但通用 observability 自检中 `platform-core / mock-payment-provider` 的历史 scrape 口径仍需后续收口；`TODO-AUD-EVENT-001` 对应的 outbox publisher 自动发布链仍按 `AUD-009/030/031` 后续完成。
