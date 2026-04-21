# V1-Core 实施进度日志 P4

本文件是实施进度日志的当前续写分卷正文。

- 正式入口页：`docs/开发任务/V1-Core-实施进度日志.md`
- 当前活动分卷以入口页为准；当前入口页指向本卷
- 若后续切换到新的 `P{N}` 分卷，必须先更新入口页，再开始续写新分卷

### BATCH-200（计划中）
- 任务：`NOTIF-001` Notification / Messaging / Template worker 骨架与运行基线
- 状态：计划中
- 说明：按冻结执行源启动 `NOTIF` 阶段，先修正已确认的本地运行基线缺口：把 `072/073/074` 纳入 `db/migrations/v1` 正式清单并补齐 downgrade / 验证脚本，同时把 `scripts/check-topic-topology.sh` 从“静态检查”升级为“静态 topology + `ops.event_route_policy` 运行态”双校验；基线修复完成后继续实现 `apps/notification-worker/` 骨架、健康检查、正式 topic / consumer group 与 `mock-log` V1 渠道闭环。
- 追溯：`docs/开发任务/V1-Core-TODO与预留清单.md` 中既有 `TODO-NOTIF-CONTRACT-001` 仅作为 `NOTIF-013/014` 后续承接，不提前作为本批完成证明。
### BATCH-200（待审批）
- 任务：`NOTIF-001` Notification / Messaging / Template worker 骨架与运行基线
- 状态：待审批
- 当前任务编号：`NOTIF-001`
- 前置依赖核对结果：`BOOT-002`、`ENV-010`、`CORE-009` 已作为 `NOTIF` 基线输入完成；本批先按已确认方案补齐本地 route-policy / migration / topology 校验基线，再继续 Worker 实现与验证。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：定位 `NOTIF-001` 的顺序、依赖、DoD、验收与 `technical_reference`。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认本批不是 README/占位，而是正式 `notification-worker -> dtp.notification.dispatch -> cg-notification-worker` 运行闭环。
  - `docs/开发准备/服务清单与服务边界正式版.md`：确认通知以 PostgreSQL 为正式记录、Kafka 为事件总线、Redis 为辅助状态，`notification-worker` 为外围进程。
  - `docs/开发准备/事件模型与Topic清单正式版.md`：核对 `notification.requested` 与 `dtp.notification.dispatch` 主题边界，不允许直接把 `dtp.outbox.domain-events` 当正式入口。
  - `docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`：核对本地依赖、环境变量与 Rust/Kafka/Redis/PostgreSQL 接入方式。
  - `docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核通知在主业务外侧、通过事件驱动推进，不反向定义业务状态。
  - `docs/04-runbooks/kafka-topics.md`、`docs/04-runbooks/notification-worker.md`、`docs/00-context/async-chain-write.md`：确认通知 topic、consumer group、异步写链与本地运行口径。
  - `infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：核对通知 route authority、active route seed 与 topic 拓扑。
  - `apps/notification-worker/**`、`apps/platform-core/src/modules/integration/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/05-test-cases/**`、`scripts/**`、`infra/**`：复用已有参考实现与本地脚本，但不把既有代码视为完成证明。
  - `../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md`、`../原始PRD/审计、证据链与回放设计.md`、`../原始PRD/日志、可观测性与告警设计.md`、`docs/开发任务/问题修复任务/A01-Kafka-Topic-口径统一.md`、`docs/开发任务/问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md`：落实通知事件、审计留痕、日志字段与命名边界冻结口径。
- 实现要点：
  - 把 `072/073/074` 正式纳入 `db/migrations/v1/manifest.csv` 与 `db/migrations/v1/checksums.sha256`，补齐 `docs/数据库设计/V1/downgrade/072_canonical_outbox_route_policy.sql`、`073_recommendation_runtime_alignment.sql`，并更新 `db/scripts/verify-migration-070.sh`、`db/scripts/verify-migration-roundtrip.sh`、`db/migrations/v1/README.md`。
  - `scripts/check-topic-topology.sh` 改为同时校验静态 topology 与当前数据库 `ops.event_route_policy` 运行态，避免“文档存在但本地 route seed 缺失”误报。
  - 新建独立 crate `apps/notification-worker/`，正式接入 Kafka、PostgreSQL、Redis 与 Prometheus 指标；运行名/消费主题/consumer group 冻结为 `notification-worker -> dtp.notification.dispatch -> cg-notification-worker`。
  - `notification-worker` 提供 `GET /health/live`、`GET /health/ready`、`GET /health/deps`、`GET /metrics` 与 `POST /internal/notifications/send`，其中 `/health/deps` 真实探测 DB/Redis/Kafka/Keycloak。
  - Worker 仅处理 `notification.requested`，模板从 `apps/notification-worker/templates/` 加载，V1 正式发送渠道为 `mock-log`，并在 PostgreSQL 中真实写入 `ops.consumer_idempotency_record`、`ops.system_log`、`ops.trace_index`、`ops.dead_letter_event`、`ops.alert_event` 与 `audit.audit_event`。
  - Redis 真实承担通知短期辅助状态、重试队列与重试载荷存储；失败按策略进入 retry / DLQ。重复事件不再覆盖已存在的 `processed/dead_lettered/retrying` 短状态，真正开始处理时写入 `processing` 短状态，保证 Redis 辅助状态与 PostgreSQL 主状态一致。
  - 更新 `apps/notification-worker/README.md`、`docs/04-runbooks/notification-worker.md`、`docs/04-runbooks/kafka-topics.md`、`docs/04-runbooks/fabric-local.md`、`infra/docker/docker-compose.apps.local.example.yml`，把 Worker 启动、健康检查、topic 校验与本地 compose 口径落到文档/脚本。
  - `docs/开发任务/V1-Core-实施进度日志.md` 已切换活动分卷到 `P4`，本批日志写入当前分卷。
- 验证步骤：
  1. `bash -n scripts/check-topic-topology.sh db/scripts/verify-migration-070.sh db/scripts/verify-migration-roundtrip.sh db/scripts/migrate-up.sh db/scripts/migration-runner.sh`
  2. `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/migration-runner.sh status`
  3. `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/migrate-reset.sh`
  4. `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/verify-migration-070.sh`
  5. `./scripts/check-topic-topology.sh`
  6. `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/verify-migration-roundtrip.sh`
  7. `cargo fmt --all`
  8. `cargo check -p notification-worker`
  9. `cargo test -p notification-worker`
  10. `cargo check -p platform-core`
  11. `cargo test -p platform-core`
  12. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  13. `./scripts/check-query-compile.sh`
  14. 启动 `notification-worker`：
      `APP_MODE=local PROVIDER_MODE=mock APP_PORT=8097 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 KAFKA_BROKERS=127.0.0.1:9094 cargo run -p notification-worker`
  15. `curl http://127.0.0.1:8097/health/live`、`/health/ready`、`/health/deps`、`/metrics`
  16. 通过 `POST /internal/notifications/send` 真实注入 4 组通知事件：成功、一次重试后成功、重试耗尽进入 DLQ、同一 `event_id` 重放去重；之后用 `psql`、`redis-cli`、`curl /metrics` 回查正式记录、重试状态、DLQ、审计、trace、alert 与 Prometheus 指标。
  17. 清理临时业务测试数据：删除本批事件对应的 `ops.alert_event`、`ops.dead_letter_event`、`ops.trace_index`、`ops.consumer_idempotency_record` 与 Redis 短状态/重试载荷；`audit.audit_event` 按 append-only 保留，`ops.system_log` 因数据库 append-only 保护无法删除。
- 验证结果：
  - migration / topology 基线修复后，本地 `ops.event_route_policy` 已存在通知与相关链路的 active route：`notification.requested -> dtp.notification.dispatch` 等运行态记录可查。
  - `cargo fmt --all`、`cargo check -p notification-worker`、`cargo test -p notification-worker`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh` 全部通过。
  - `notification-worker` 健康检查通过：`/health/live=ok`、`/health/ready=ready`、`/health/deps` 返回 DB/Redis/Kafka/Keycloak 均可达，`/metrics` 暴露通知指标。
  - 正式通知链路验证通过：注入 `notification.requested` 后，Worker 仅消费 `dtp.notification.dispatch`，未直接消费 `dtp.outbox.domain-events`；`mock-log` 为真实 V1 发送渠道。
  - 成功、重试、DLQ、去重验证通过：
    - 成功事件写入 `ops.consumer_idempotency_record.result_code=processed`
    - 重试事件先进入 `retrying`，随后 `processed`
    - 失败事件重试耗尽后写入 `ops.dead_letter_event` 与 `ops.alert_event`
    - 同一 `event_id` 二次投递时，数据库保持 `processed|1`，Redis 短状态保持 `processed`，不再被 `duplicate` 覆盖
  - PostgreSQL / Redis / 审计 / trace / 指标回查通过：
    - `ops.consumer_idempotency_record`、`ops.dead_letter_event`、`ops.alert_event`、`audit.audit_event`、`ops.trace_index` 均按预期写入
    - Redis `retry-queue` 深度可见并在重试完成后回到 `0`
    - `/metrics` 返回 `notification_worker_events_total`、`notification_worker_send_total`、`notification_worker_retry_queue_depth`
  - 临时业务测试数据清理结果：`ops.alert_event=0`、`ops.dead_letter_event=0`、`ops.trace_index=0`、`ops.consumer_idempotency_record=0`、Redis 短状态已清空；`audit.audit_event` 依要求保留；`ops.system_log` 受 `audit.tg_append_only_guard()` 保护，仍保留 7 条本批镜像日志，属于数据库 append-only 约束下的可接受残留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`NOTIF-001`
  - `事件模型与Topic清单正式版.md`、`topics.v1.json`、`072/074` SQL：通知事件与 topic / route authority
  - `审计、证据链与回放设计.md`：通知审计与 replay 轨迹
  - `日志、可观测性与告警设计.md`：日志字段、trace 与告警链路
  - `A01-Kafka-Topic-口径统一.md`、`A10-NOTIF-通知链路与命名边界缺口.md`：通知 topic / consumer / 命名边界统一
- 覆盖的任务清单条目：`NOTIF-001`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-NOTIF-CONTRACT-001` 仍留待后续 `NOTIF-013/014` 承接。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-201（计划中）
- 任务：`NOTIF-002` Notification 事件协议统一与正式生产入口
- 状态：计划中
- 说明：在 `NOTIF-001` 已落地 Worker 运行基线的前提下，当前批次把“通知协议”本身从口头约束落到代码：在 `platform-core.integration` 中冻结 `notification.requested` 的正式 scene catalog、payload 字段、source-event 元数据与幂等键生成规则，统一收口到 `notification.dispatch_request / notification.requested / dtp.notification.dispatch`；同时更新 `notification-worker` 对新协议字段的解析与 runbook，确保后续 `NOTIF-004 ~ NOTIF-007` 只是在同一协议上补具体模板和业务触发逻辑，而不是再次发明旁路事件。
- 追溯：本批只处理协议、统一生产入口、协议样例和验证，不提前把 `NOTIF-003 ~ NOTIF-009` 的模板/渠道/联查/OpenAPI/TestCase 整批混做；既有 `TODO-NOTIF-CONTRACT-001` 继续保留给后续控制面与验收文档承接。
### BATCH-201（待审批）
- 任务：`NOTIF-002` Notification 事件协议统一与正式生产入口
- 状态：待审批
- 当前任务编号：`NOTIF-002`
- 前置依赖核对结果：`NOTIF-001` 已完成并提供 `notification-worker` 运行基线；`TRADE-033`、`BIL-023` 作为上游事实源已在主线代码中落地，当前批次据此冻结通知 scene catalog 与来源事件口径，但不提前把业务触发逻辑混入本批。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `NOTIF-002` 的正式目标是冻结 `notification.requested -> dtp.notification.dispatch -> notification-worker` 协议，不再并行消费 `dtp.outbox.domain-events`。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核通知协议必须以 PostgreSQL canonical outbox 为正式生产入口、Kafka 为总线、Redis 为短期辅助状态，`notification-worker` 为外围消费进程。
  - `docs/04-runbooks/kafka-topics.md`、`docs/04-runbooks/notification-worker.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：核对通知 route authority、topic 绑定与 worker 正式消费入口。
  - `apps/notification-worker/**`、`apps/platform-core/src/modules/integration/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/05-test-cases/**`、`scripts/**`、`infra/**`：复用现有实现基线，但不把任何参考代码视为已完成证据。
  - `../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md`、`../原始PRD/审计、证据链与回放设计.md`、`../原始PRD/日志、可观测性与告警设计.md`、`docs/开发任务/问题修复任务/A01-Kafka-Topic-口径统一.md`、`docs/开发任务/问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md`、`docs/01-architecture/order-orchestration.md`、`docs/03-db/sku-billing-trigger-matrix.md`：冻结 13 类通知场景、来源事件命名、审计字段、日志字段与 Kafka/topic 边界。
- 实现要点：
  - 新增共享 crate `apps/platform-core/crates/notification-contract/`，把 `NotificationScene`、`NotificationAudience`、`NotificationRequestedPayload`、`NotificationSourceEvent`、`NotificationSubjectRef`、`NotificationActionLink`、`NotificationRetryPolicy`、`build_notification_request_payload`、`build_notification_idempotency_key` 统一收口成正式协议。
  - scene catalog 冻结为 13 个正式场景：订单创建、支付成功、支付失败、待交付、交付完成、待验收、验收通过、拒收、争议升级、退款完成、赔付完成、监管冻结、恢复结算；默认模板编码和 V1 渠道默认值统一在共享协议中定义。
  - `apps/platform-core/src/modules/integration/application/mod.rs` 新增 `prepare_notification_request` 与 `queue_notification_request`：统一生成 `notification.requested` payload、稳定幂等键，并把 canonical outbox 记录写入 `ops.outbox_event`，固定 `aggregate_type=notification.dispatch_request`、`event_type=notification.requested`、`producer_service=platform-core.integration`、`target_topic=dtp.notification.dispatch`。
  - `apps/platform-core/src/modules/integration/tests/notification_contract_db.rs` 新增 live DB smoke：13 个场景全部真实写入 canonical outbox，并验证重复请求被同一幂等键抑制，不再在本批通过旁路 topic/事件绕过正式入口。
  - `apps/notification-worker/src/event.rs` 改为消费共享协议 crate，`POST /internal/notifications/send` 也按同一协议构造 envelope，支持 `notification_code`、`audience_scope`、`source_event`、`subject_refs`、`links` 等字段，不再接受“只有模板号和自由 payload”的旧形态作为正式口径。
  - `apps/notification-worker/src/template.rs` 的渲染上下文补充 `notification_code`、`template_code`、`audience_scope`、`source_event`、`subject_refs`、`links`，保证后续模板任务在同一协议字段上扩展。
  - `apps/notification-worker/README.md` 与 `docs/04-runbooks/notification-worker.md` 增补冻结 scene catalog、最小 payload 字段、幂等键公式与手工注入示例，明确后续 `NOTIF-004 ~ NOTIF-007` 只允许在此协议上填充具体模板和业务触发逻辑。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p notification-contract`
  3. `cargo test -p notification-contract`
  4. `cargo check -p notification-worker`
  5. `cargo test -p notification-worker`
  6. `cargo check -p platform-core`
  7. `NOTIF_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core notif002_notification_contract_db_smoke -- --nocapture`
  8. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  9. `cargo test -p platform-core`
  10. `./scripts/check-query-compile.sh`
  11. 启动 `notification-worker`：
      `APP_PORT=8097 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 KAFKA_BROKERS=127.0.0.1:9094 cargo run -p notification-worker`
  12. `curl http://127.0.0.1:8097/health/live`、`/health/ready`、`/health/deps`
  13. 通过 `POST /internal/notifications/send` 手工注入一条带 `notification_code/audience_scope/source_event/subject_refs/links` 的 `payment.succeeded` 通知事件。
  14. 使用 `psql` 回查 `ops.consumer_idempotency_record`、`ops.system_log`、`ops.trace_index`、`audit.audit_event`，使用 `redis-cli` 回查短期状态，使用 `curl /metrics` 回查 Prometheus 指标。
  15. 使用 `docker exec datab-kafka /opt/kafka/bin/kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic dtp.notification.dispatch --from-beginning --timeout-ms 5000` 回查 Kafka 留存消息，确认 topic 上实际承载的是冻结后的正式协议字段。
  16. 清理 Redis 临时短状态键；DB 中 audit / system log / trace 等运行痕迹按 append-only / 运行留痕口径保留。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p notification-contract`、`cargo test -p notification-contract`、`cargo check -p notification-worker`、`cargo test -p notification-worker`、`cargo check -p platform-core`、`NOTIF_DB_SMOKE=1 ... notif002_notification_contract_db_smoke`、`cargo sqlx prepare --workspace`、`cargo test -p platform-core`、`./scripts/check-query-compile.sh` 全部通过。
  - `notif002_notification_contract_db_smoke` 在真实数据库中完成 13 个 scene 的 canonical outbox 落库，并确认重复写入被同一幂等键抑制；样例记录回查到 `event_type=notification.requested`、`target_topic=dtp.notification.dispatch`、`payload.source_event.event_type=billing.event.recorded`、`payload.subject_refs[0].ref_type=order`。
  - `notification-worker` 按 `dtp.notification.dispatch` 启动，`/health/live=ok`、`/health/ready=ready`、`/health/deps` 返回 DB/Redis/Kafka/Keycloak 全部 reachable。
  - 手工注入的 `payment.succeeded` 事件返回 `topic=dtp.notification.dispatch`，Worker 日志显示该事件被真实消费并通过 `mock-log` 送达；Kafka 回查到留存消息中包含 `notification_code=payment.succeeded`、`audience_scope=buyer`、`source_event.event_type=billing.event.recorded`、`subject_refs`、`links`、稳定幂等键与 `aggregate_type=notification.dispatch_request`。
  - PostgreSQL / Redis / 指标回查通过：
    - `ops.consumer_idempotency_record` 中该事件为 `processed` 且 `attempt=1`
    - `ops.system_log` 写入 `notification sent via mock-log`
    - `ops.trace_index` 记录 `root_span_name=notification.dispatch`
    - `audit.audit_event` 写入 `notification.dispatch.sent / success`
    - `/metrics` 返回 `notification_worker_events_total{result="processed"} 1`、`notification_worker_send_total{channel="mock-log",result="success"} 1`、`notification_worker_retry_queue_depth 0`
    - Redis 短状态为 `processed`，验证后已删除本次手工测试键
  - 本批再次确认 `notification-worker` 运行时只消费 `dtp.notification.dispatch`，没有把 `dtp.outbox.domain-events` 当正式消费入口；`dtp.outbox.domain-events` 仅作为 `source_event.target_topic` 溯源字段保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`NOTIF-002`
  - `12-API 设计、事件模型与消息总线.md`：通知场景/事件模型
  - `审计、证据链与回放设计.md`：通知审计字段与回放最小元数据
  - `日志、可观测性与告警设计.md`：通知日志 / trace / metrics 字段
  - `A01-Kafka-Topic-口径统一.md`、`A10-NOTIF-通知链路与命名边界缺口.md`：通知正式 topic / aggregate_type / event_type 统一口径
  - `order-orchestration.md`、`sku-billing-trigger-matrix.md`：支付/订单等来源事件语义边界
- 覆盖的任务清单条目：`NOTIF-002`
- 未覆盖项：无。后续 `NOTIF-003 ~ NOTIF-009` 继续基于本批冻结协议实现模板、渠道适配与业务触发，不再修改 scene catalog 或正式 topic 口径。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-NOTIF-CONTRACT-001` 仍按既有计划留给 `NOTIF-013/014` 承接。
- 备注：本批未改 `docs/开发任务/V1-Core-TODO与预留清单.md`，因为没有新增 gap / reserved 项；`V1-Core-人工审批记录.md` 仍由你手工维护。
