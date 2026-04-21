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
### BATCH-202（计划中）
- 任务：`NOTIF-003` Notification 模板模型与渲染预览
- 状态：计划中
- 说明：在 `NOTIF-002` 已冻结通知协议的前提下，当前批次把模板从文件占位升级为正式模型，至少覆盖模板编码、语言、变量 schema、渠道、启用状态、版本号、渲染结果预览与 fallback 文案，并让 `notification-worker` 真正从 PostgreSQL 模板权威源加载与渲染，而不是继续把 `templates/*.json` 当唯一正式来源。
- 追溯：本批先完成模板模型、模板存储、预览与运行时加载，不提前混入 `NOTIF-004 ~ NOTIF-007` 的具体业务触发逻辑；后续场景模板只允许在本批建立的模型上增量补齐。
### BATCH-202（待审批）
- 任务：`NOTIF-003` Notification 模板模型与渲染预览
- 状态：待审批
- 当前任务编号：`NOTIF-003`
- 前置依赖核对结果：`NOTIF-001` 已提供 `notification-worker` 运行基线与真实发送/重试/DLQ 闭环；`NOTIF-002` 已冻结 scene catalog、正式 payload、幂等键与 canonical outbox 写入口。当前批次在此基础上补模板权威源与渲染模型，不再回退到旧的文件占位口径。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认本批必须实现模板编码、语言、变量 schema、渠道、启用状态、版本号、渲染预览与 fallback 文案。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核通知模板的权威状态应落 PostgreSQL，Kafka 仅负责事件传播，Worker 读取模板后执行外围发送。
  - `docs/04-runbooks/kafka-topics.md`、`docs/04-runbooks/notification-worker.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：确认模板变更不能改变正式 topic/consumer 边界，仍必须走 `notification.requested -> dtp.notification.dispatch -> notification-worker`。
  - `apps/notification-worker/**`、`apps/platform-core/src/modules/integration/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/05-test-cases/**`、`scripts/**`、`infra/**`：核对现有实现只有文件模板与 README 占位，没有正式模板模型；本批可复用 `notification-contract` 与既有 Worker 审计/指标链路。
  - `docs/data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md`、`docs/原始PRD/审计、证据链与回放设计.md`、`docs/原始PRD/日志、可观测性与告警设计.md`、`docs/开发任务/问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md`：复核模板预览、审计字段、日志字段与通知链路边界。
- 实现要点：
  - 新增迁移 `075_notification_template_model.sql` / downgrade，把模板正式落到 PostgreSQL `ops.notification_template`，字段覆盖：
    - `template_code`
    - `language_code`
    - `channel`
    - `version_no`
    - `enabled`
    - `status`
    - `variables_schema_json`
    - `title_template`
    - `body_template`
    - `fallback_body_template`
    - `metadata`
  - `075` 同时 seed `DEFAULT_NOTIFICATION_V1`、`NOTIFY_GENERIC_V1` 与 13 个 scene 对应的 `NOTIFY_*_V1` 模板，统一使用 `mock-log + zh-CN + version_no=1 + active/enabled` 基线。
  - `apps/notification-worker/src/template.rs` 重写为 DB-first 模型：
    - 运行时优先从 `ops.notification_template` 读取 `enabled=true AND status='active'` 的最新版本
    - 指定语言找不到时回退到默认语言 `zh-CN`
    - 指定模板缺失时回退到 `DEFAULT_NOTIFICATION_V1`
    - 最后才回退到 `apps/notification-worker/templates/*.json` file fallback
  - 模板渲染补齐最小 schema 校验与严格渲染：
    - `variables_schema_json` 在运行时真实校验
    - 缺失 required 变量时视为模板渲染失败
    - body 渲染失败时使用 `fallback_body_template`
    - 渲染结果返回 `version_no`、解析语言、schema、fallback 使用情况与模板 metadata
  - 新增 `POST /internal/notifications/templates/preview`，允许在不发 Kafka 事件的情况下预览模板渲染结果，返回：
    - `template_code`
    - `channel`
    - `language_code`
    - `requested_language_code`
    - `version_no`
    - `template_enabled`
    - `template_status`
    - `template_fallback_used`
    - `body_fallback_used`
    - `variable_schema`
    - `template_metadata`
    - `title`
    - `body`
  - 正式发送路径也切到同一模板模型：`notification-worker` 消费 Kafka 后先从 PostgreSQL 模板表解析模板，再执行 `mock-log` 发送；模板渲染/校验失败不再直接丢出，而是进入既有 retry / DLQ 处理路径。
  - 更新 `apps/notification-worker/README.md`、`docs/04-runbooks/notification-worker.md` 与 file fallback JSON 样例，明确 PostgreSQL 才是模板权威源，文件模板仅用于 local fallback。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p notification-worker`
  3. `cargo test -p notification-worker`
  4. `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/migrate-reset.sh`
  5. `NOTIF_TEMPLATE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p notification-worker notif003_template_model_db_smoke -- --nocapture`
  6. `cargo check -p platform-core`
  7. `cargo test -p platform-core`
  8. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  9. `./scripts/check-query-compile.sh`
  10. 启动 `notification-worker`：
      `APP_PORT=8097 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 KAFKA_BROKERS=127.0.0.1:9094 cargo run -p notification-worker`
  11. `curl http://127.0.0.1:8097/health/live`、`/health/ready`、`/health/deps`
  12. `curl -X POST /internal/notifications/templates/preview`：
      使用 `template_code=NOTIFY_PAYMENT_SUCCEEDED_V1`、`language_code=en-US`、只提供 `variables.subject`，验证语言回退到 `zh-CN` 且 body 触发 fallback 文案。
  13. `curl -X POST /internal/notifications/send`：
      注入 `payment.succeeded` 事件并显式传入 `subject/message/source_event/subject_refs/links`，验证真实发送路径使用 PostgreSQL 模板，不再回退到 `DEFAULT_NOTIFICATION_V1`。
  14. 使用 `psql` 回查 `ops.notification_template`、`ops.consumer_idempotency_record`、`ops.system_log`、`ops.trace_index`、`audit.audit_event`，使用 `redis-cli` 回查短状态，使用 `curl /metrics` 回查指标，使用 `docker exec datab-kafka /opt/kafka/bin/kafka-console-consumer.sh` 回查 Kafka 留存消息。
  15. 清理 Redis 短状态键；DB 的审计/日志/trace 运行痕迹按 append-only / 运行证据保留。
- 验证结果：
  - `075` 迁移成功进入本地基线，`migrate-reset.sh` 已真实执行到 `075_notification_template_model.sql`；`ops.notification_template` 中可查到 `NOTIFY_PAYMENT_SUCCEEDED_V1 / zh-CN / mock-log / version_no=1 / enabled / active`。
  - `cargo fmt --all`、`cargo check -p notification-worker`、`cargo test -p notification-worker`、`NOTIF_TEMPLATE_DB_SMOKE=1 ... notif003_template_model_db_smoke`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh` 全部通过。
  - `notif003_template_model_db_smoke` 真实连接 PostgreSQL，确认 active 模板数不少于 15 条，并验证：
    - `NOTIFY_PAYMENT_SUCCEEDED_V1` 从 DB 读出
    - 请求语言 `en-US` 回退到 `zh-CN`
    - body 因缺少 `message` 触发 fallback 文案
  - 手工 preview 验证通过：
    - `POST /internal/notifications/templates/preview` 返回 `template_code=NOTIFY_PAYMENT_SUCCEEDED_V1`
    - `language_code=zh-CN`
    - `requested_language_code=en-US`
    - `version_no=1`
    - `template_fallback_used=false`
    - `body_fallback_used=true`
    - `variable_schema.required=["subject"]`
    - `body=notification=payment.succeeded recipient=buyer.preview@example.test`
  - 手工真实发送验证通过：
    - `POST /internal/notifications/send` 返回 `topic=dtp.notification.dispatch`
    - Worker 日志显示 `template_code=NOTIFY_PAYMENT_SUCCEEDED_V1`
    - 不再使用 `DEFAULT_NOTIFICATION_V1` 作为成功场景的正式模板
  - PostgreSQL / Redis / Kafka / metrics 回查通过：
    - `ops.consumer_idempotency_record.result_code=processed`
    - `ops.system_log` 中 `template_code=NOTIFY_PAYMENT_SUCCEEDED_V1`
    - `ops.trace_index` 中 `root_span_name=notification.dispatch` 且 metadata 记录 `template_code=NOTIFY_PAYMENT_SUCCEEDED_V1`
    - `audit.audit_event` 中 `action_name=notification.dispatch.sent / result_code=success`
    - `/metrics` 返回 `notification_worker_events_total{result="processed"} 1`、`notification_worker_send_total{channel="mock-log",result="success"} 1`、`notification_worker_retry_queue_depth 0`
    - Kafka `dtp.notification.dispatch` 留存消息中可见 `payload.template_code=NOTIFY_PAYMENT_SUCCEEDED_V1`
    - Redis 短状态在回查后已清理
  - 本批没有改变正式事件链与 consumer 边界：通知仍通过 `notification.requested -> dtp.notification.dispatch -> notification-worker` 发送，`dtp.outbox.domain-events` 仍只作为 `source_event.target_topic` 溯源字段。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`NOTIF-003`
  - `12-API 设计、事件模型与消息总线.md`：模板/通知事件的最小消息约束
  - `审计、证据链与回放设计.md`：模板预览与发送链路的审计字段要求
  - `日志、可观测性与告警设计.md`：模板渲染与发送镜像日志字段
  - `A10-NOTIF-通知链路与命名边界缺口.md`：模板、运行时与联查不得继续停留在 provider/file placeholder
- 覆盖的任务清单条目：`NOTIF-003`
- 未覆盖项：无。后续 `NOTIF-004 ~ NOTIF-007` 继续在本批模板模型上填充买方/卖方/运营差异化正文与业务触发逻辑。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`docs/开发任务/V1-Core-TODO与预留清单.md` 本批无需变更。
- 备注：`V1-Core-人工审批记录.md` 仍由你手工维护；本批保留 PostgreSQL 中的发送/审计/trace 运行痕迹作为实施证据，不做删除。
### BATCH-203（计划中）
- 任务：`NOTIF-004` 支付成功 -> 待交付通知模板与发送逻辑
- 状态：计划中
- 说明：当前批次围绕 `TRADE-030` 的支付成功编排点落真实通知触发：支付结果把订单推进到 `buyer_locked / pending_delivery` 后，需要按冻结协议生成买方、卖方、运营三类通知，并确保业务用户通知正文不暴露内部风控/审计字段。
- 追溯：本批先在既有 `notification-contract + ops.notification_template` 模型上补 `payment.succeeded / order.pending_delivery` 的模板版本与触发逻辑；若真实链路验证受 `outbox-publisher` 缺失阻塞，将先把它作为当前批次的必要运行前置补齐，不另起并行 task。
### BATCH-203（待审批）
- 任务：`NOTIF-004` 支付成功 -> 待交付通知模板与发送逻辑
- 状态：待审批
- 当前任务编号：`NOTIF-004`
- 前置依赖核对结果：`NOTIF-002` 已冻结 `notification.requested -> dtp.notification.dispatch -> notification-worker` 协议、payload 与幂等键；`TRADE-030` 已提供支付成功后订单进入 `pending_delivery` 的编排点。本批不提前实现 `AUD-009` outbox publisher，而是在既有 canonical outbox writer 基础上完成支付成功通知模板、触发逻辑与 worker 消费验证。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认本批完成定义是“支付成功 -> 待交付”通知模板与发送逻辑，需区分 buyer / seller / ops 可见内容，禁止把内部风控/审计字段暴露给业务用户。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核通知仍以 PostgreSQL 为主记录、Kafka 为总线、Redis 为辅助状态、`notification-worker` 为外围消费进程，并要求本地 Keycloak / Redis / Kafka / Prometheus / Alertmanager / Loki / Tempo / Grafana 为真实依赖。
  - `docs/04-runbooks/kafka-topics.md`、`docs/04-runbooks/notification-worker.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：确认通知正式 topic、route authority、consumer group 与异步写链边界。
  - `../业务流程/业务流程图-V1-完整版.md`、`../页面说明书/页面说明书-V1-完整版.md`、`docs/开发任务/问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md`：确认支付成功后买方、卖方、运营三方通知的业务语义、订单详情入口与最小披露边界。
  - `apps/notification-worker/**`、`apps/platform-core/src/modules/integration/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/05-test-cases/**`、`scripts/**`、`infra/**`：复用既有通知协议、模板模型、worker 审计/重试/指标能力与本地脚本，但不把已有 README / 草稿实现视为任务已完成证明。
- 实现要点：
  - `apps/platform-core/src/modules/integration/application/mod.rs` 新增 `queue_payment_success_notifications(...)`，在支付成功时统一组装 buyer / seller / ops 三类 `notification.requested` payload，并真实写入 canonical outbox：
    - buyer：`payment.succeeded / NOTIFY_PAYMENT_SUCCEEDED_V1`
    - seller：`order.pending_delivery / NOTIFY_PENDING_DELIVERY_V1`
    - ops：`order.pending_delivery / NOTIFY_PENDING_DELIVERY_V1`
  - buyer / seller payload 仅保留订单、商品、金额、状态与操作入口；ops payload 才保留 `billing_event_id / payment_intent_id / provider_reference_id / provider_result_source` 等联查字段。
  - `apps/platform-core/src/modules/billing/payment_result_processor.rs` 在支付成功分支中接入通知编排，确保支付 webhook / 轮询结果统一走同一通知触发逻辑。
  - 新增迁移 `076_notification_payment_success_pending_delivery_templates.sql` / downgrade，正式把 `NOTIFY_PAYMENT_SUCCEEDED_V1`、`NOTIFY_PENDING_DELIVERY_V1` version `2` 落为当前启用模板，并把 version `1` 标记为归档回退版本。
  - 新增 `apps/platform-core/src/modules/integration/tests/notif004_payment_success_db.rs`，真实验证支付成功后 canonical outbox 中 buyer / seller / ops 三类消息的模板码、事件码、source_event 与字段披露边界。
  - `apps/notification-worker/src/event.rs` / `src/main.rs` 补齐顶层 `trace_id=null` 兼容：worker 运行时统一以 `effective_trace_id = trace_id || request_id` 写入幂等、trace、审计与日志，避免 canonical outbox 顶层空值导致消费失败。
  - 本批联调中同时修正本地基础设施基线：
    - Keycloak 改为使用独立 `KEYCLOAK_DB_NAME=keycloak`，`up-local` 启动前自动确保服务数据库存在，避免 `migrate-reset` 重建业务库后破坏 realm 表
    - Tempo healthcheck 改为镜像内可执行的 `/busybox/wget`
    - compose 统一把 `host.docker.internal` 固定到 `host-gateway`，让 Prometheus / Alertmanager / mock-payment-provider 可以稳定访问宿主机进程
    - Prometheus / Grafana / runbook 补齐 `notification-worker:8097` 与 `platform-core:8094` 本地口径、通知事件图表与 `NotificationRetryQueueBacklog` 告警规则
- 验证步骤：
  1. `bash -n scripts/up-local.sh scripts/ensure-local-service-dbs.sh scripts/check-observability-stack.sh infra/postgres/initdb/003_service_databases.sh`
  2. `docker compose --profile core --profile observability --profile mocks --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config >/tmp/datab-compose-local-check.yaml`
  3. `./scripts/ensure-local-service-dbs.sh infra/docker/.env.local`
  4. `docker compose --profile core --profile observability --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml up -d --force-recreate keycloak tempo`
  5. `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`
  6. `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/migrate-reset.sh`
  7. `NOTIF_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core notif004_payment_success_notifications_db_smoke -- --nocapture`
  8. `cargo fmt --all`
  9. `cargo check -p platform-core`
  10. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core`
  11. `cargo check -p notification-worker`
  12. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p notification-worker`
  13. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  14. `./scripts/check-query-compile.sh`
  15. 启动 `platform-core`：
      `set -a; source infra/docker/.env.local; set +a; APP_MODE=local PROVIDER_MODE=mock APP_PORT=8094 KAFKA_BROKERS=127.0.0.1:9094 cargo run -p platform-core-bin`
  16. 启动 `notification-worker`：
      `set -a; source infra/docker/.env.local; set +a; APP_MODE=local PROVIDER_MODE=mock APP_PORT=8097 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 KAFKA_BROKERS=127.0.0.1:9094 cargo run -p notification-worker`
  17. `curl http://127.0.0.1:8094/health/live`、`/health/ready`，`curl http://127.0.0.1:8097/health/live`、`/health/ready`、`/metrics`
  18. 构造真实支付成功 webhook，`POST /api/v1/payments/webhooks/mock_payment` 后用 `psql` 回查 `ops.outbox_event` 中 `request_id=req-notif004-live-webhook-1776790875359510889` 的 canonical outbox 记录。
  19. 由于 `AUD-009` outbox publisher 尚未进入依赖范围，本批从 canonical outbox 导出的原始 envelope 回放到正式 topic `dtp.notification.dispatch`，随后观察 `notification-worker` Kafka 消费、数据库落库、Redis 短状态与 `mock-log` 输出。
  20. `psql` 回查 `ops.consumer_idempotency_record`、`ops.system_log`、`audit.audit_event`、`ops.trace_index`；`redis-cli -n 2` 回查通知短状态；`curl -G http://127.0.0.1:9090/api/v1/query --data-urlencode 'query=notification_worker_events_total'`、Grafana `/api/search`、Alertmanager `/api/v2/status` 回查通知指标、看板与告警规则。
  21. 清理临时业务测试数据：删除订单/支付/商品/幂等/Redis 短状态等业务/辅助状态；`audit.audit_event` 与 `ops.system_log` 作为 append-only 留痕保留。
- 验证结果：
  - Keycloak / Tempo 本地运行基线修复完成：`http://127.0.0.1:8081/realms/platform-local/.well-known/openid-configuration` 可用，`docker ps` 中 `datab-keycloak`、`datab-tempo` 均为 `healthy`；`ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh` 全部通过。
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo check -p notification-worker`、`cargo test -p notification-worker`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh` 全部通过。
  - `notif004_payment_success_notifications_db_smoke` 在真实数据库中验证了支付成功后会写出 3 条通知 canonical outbox，且 buyer / seller / ops 的 `notification_code / template_code / source_event / metadata` 与冻结口径一致。
  - 真实 webhook 验证通过：`processed_status=processed`；`ops.outbox_event` 中回查到 3 条目标 `dtp.notification.dispatch` 的通知记录，分别对应 buyer / seller / ops。
  - 正式 topic 消费验证通过：`notification-worker` 只消费 `dtp.notification.dispatch`，Kafka 回查可见通知 envelope；worker 日志显示 3 条通知均通过 `mock-log` 成功送达。
  - PostgreSQL / Redis / 审计 / trace 回查通过：
    - `ops.consumer_idempotency_record` 中 3 条通知均为 `processed / attempt=1 / source=kafka`
    - `ops.system_log` 记录 3 条 `notification sent via mock-log`
    - `audit.audit_event` 写入 3 条 `notification.dispatch.sent / success`
    - `ops.trace_index` 写入 3 条 `root_service_name=notification-worker / root_span_name=notification.dispatch`
    - Redis DB `2` 中 3 个 `datab:v1:notification:state:*` 短状态均为 `processed`
  - 模板与字段最小披露验证通过：
    - buyer 正文使用 `NOTIFY_PAYMENT_SUCCEEDED_V1`，未暴露 `payment_intent_id / provider_reference_id`
    - seller 正文使用 `NOTIFY_PENDING_DELIVERY_V1`，`show_ops_context=false`，未暴露内部联查字段
    - ops 正文使用 `NOTIFY_PENDING_DELIVERY_V1`，`show_ops_context=true`，可见 `billing_event / payment_intent / provider_ref / source`
  - 观测链路回查通过：
    - `notification-worker` `/metrics` 暴露 `notification_worker_events_total{result="processed"} 3`、`notification_worker_send_total{channel="mock-log",result="success"} 3`、`notification_worker_retry_queue_depth 0`
    - Prometheus 抓取 `notification-worker` 成功，`up{job="notification-worker"}=1`，`notification_worker_events_total` 可查询
    - Grafana `Platform Overview` 看板可查，通知指标图表已注册
    - Alertmanager 状态可用，Prometheus 规则中存在 `NotificationRetryQueueBacklog`
  - 清理结果：`trade.order_main=0`、`payment.payment_intent=0`、`ops.outbox_event(request_id)=0`、`ops.consumer_idempotency_record=0`、`ops.trace_index(request_id)=0`，Redis 通知短状态键已删除；`audit.audit_event(request_id)=8`、`ops.system_log(request_id)=3` 作为 append-only 留痕保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`NOTIF-004`
  - `业务流程图-V1-完整版.md`：支付成功后待交付与通知时机
  - `页面说明书-V1-完整版.md`：订单详情页通知入口与业务用户可见信息边界
  - `A10-NOTIF-通知链路与命名边界缺口.md`：正式事件链、topic、进程名与渠道边界
  - `notification-worker.md`、`kafka-topics.md`、`topics.v1.json`、`072/074`：通知路由、consumer group 与 canonical outbox 口径
- 覆盖的任务清单条目：`NOTIF-004`
- 未覆盖项：
  - 不提前实现 `AUD-009` outbox publisher；本批只验证 canonical outbox 写入与正式 Kafka topic 消费两端，自动发布进程仍按既有 `TODO-AUD-EVENT-001` / `AUD-009; AUD-030; AUD-031` 后续收口。
- 新增 TODO / 预留项：
  - 新增非阻塞 `TODO-NOTIF-OBS-001`：通用 observability 自检仍包含历史 `platform-core / mock-payment-provider` Prometheus target 口径，但两者当前未提供可直接抓取的正式 metrics endpoint，本批只把 `notification-worker` 观测链路打通并登记后续补齐条件。
- 备注：
  - `ops.system_log` 与 `audit.audit_event` 均受 append-only 保护，本批按规则保留验证痕迹，不做删除。
  - `V1-Core-人工审批记录.md` 仍由你手工维护；本批未写入。
### BATCH-204（计划中）
- 任务：`NOTIF-005` 交付完成 -> 待验收通知模板与发送逻辑
- 状态：计划中
- 说明：当前批次围绕 `DLV-030` 已冻结的六类交付桥接点落真实通知触发：文件包、共享开通、API 开通、查询结果可取、沙箱开通、报告交付完成后，需要按交付分支与实际 `acceptance_status` 生成买方、卖方、运营通知。手工验收分支（文件/报告）优先给买方发 `order.pending_acceptance`，自动验收或启用完成分支给买方发 `delivery.completed`；卖方与运营统一收到 `delivery.completed`，且业务用户正文不得暴露内部联查字段。
- 追溯：本批在既有 `notification-contract + ops.notification_template + delivery bridge` 口径上增量实现，不提前改写 `DLV-030/031` 的交付状态机或 `AUD-009` outbox publisher；如发现查询结果可取场景的 source-event 口径与冻结文档存在不可安全推断冲突，将按规则暂停并提问。
### BATCH-204（待审批）
- 任务：`NOTIF-005` 交付完成 -> 待验收通知模板与发送逻辑
- 状态：待审批
- 当前任务编号：`NOTIF-005`
- 前置依赖核对结果：`NOTIF-002` 已冻结正式通知 scene catalog、payload 与幂等键；`DLV-030` 已把文件/共享/API/查询/沙箱/报告六类交付桥接点接入 canonical outbox。当前批次只在这些已冻结桥接点上补正式模板与通知触发，不回退为同步硬编码发送，也不提前实现 `AUD-009` 自动 outbox publisher。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认本批 DoD 是“Worker 可消费事件并发送 mock 通知；模板/幂等/重试可验证；审计和 runbook 已覆盖”，且必须覆盖六类交付结果。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核通知记录主权威源是 PostgreSQL，Kafka 负责异步分发，Redis 负责短状态/重试，`notification-worker` 是外围进程。
  - `docs/04-runbooks/kafka-topics.md`、`docs/04-runbooks/notification-worker.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：确认正式链路仍是 `notification.requested -> dtp.notification.dispatch -> notification-worker`，且不得把 `dtp.outbox.domain-events` 当正式消费入口。
  - `../业务流程/业务流程图-V1-完整版.md`、`../页面说明书/页面说明书-V1-完整版.md`、`docs/开发任务/问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md`：确认“交付完成 -> 待验收”业务时机、订单详情/验收页入口，以及业务用户与运营的字段披露边界。
  - `apps/notification-worker/**`、`apps/platform-core/src/modules/integration/**`、`apps/platform-core/src/modules/delivery/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/05-test-cases/**`、`scripts/**`、`infra/**`：复核现有 `notification-worker`、模板模型、交付仓储与本地联调脚本，确认已有实现只能作为参考，不视为任务完成证明。
- 实现要点：
  - `apps/platform-core/src/modules/integration/application/mod.rs` 新增 `queue_delivery_completion_notifications(...)`，统一装配六类交付完成通知：
    - 文件包 / 报告：买方 `order.pending_acceptance / NOTIFY_PENDING_ACCEPTANCE_V1`
    - 共享开通 / API 开通 / 查询结果可取 / 沙箱开通：买方 `delivery.completed / NOTIFY_DELIVERY_COMPLETED_V1`
    - 卖方、运营：统一 `delivery.completed / NOTIFY_DELIVERY_COMPLETED_V1`
  - 新增交付通知上下文装配，统一输出 `product_title / buyer_org_name / seller_org_name / order_amount / payment_status / delivery_status / acceptance_status / delivery_branch_label / action_label / action_href`；仅 `ops` audience 附带 `delivery_ref_type / delivery_ref_id / receipt_hash / delivery_commit_hash`。
  - 文件、共享、API、查询、沙箱、报告六类交付仓储在原有桥接 / outbox 事务内接入通知写入，保持“业务状态 -> canonical outbox -> 正式 topic 消费”顺序一致，不额外引入旁路发送。
  - 新增迁移 `077_notification_delivery_completed_pending_acceptance_templates.sql` / downgrade，正式把 `NOTIFY_DELIVERY_COMPLETED_V1`、`NOTIFY_PENDING_ACCEPTANCE_V1` 升级为 version `2`，并把 version `1` 归档。
  - 新增 `apps/platform-core/src/modules/integration/tests/notif005_delivery_completion_db.rs`，真实验证六类交付场景写出的 `notification_code / template_code / source_event / metadata`、幂等键稳定性以及 buyer / seller / ops 字段可见性。
  - 增强 `dlv002_file_delivery_commit_db_smoke` 与 `dlv007_api_delivery_db_smoke`，在原有交付 DB smoke 上直接回查 `dtp.notification.dispatch` canonical outbox，确保文件/API 两类真实桥接点已接通知而非仅靠单元逻辑。
  - 更新 `apps/notification-worker/README.md`、`docs/04-runbooks/notification-worker.md`，把 `NOTIF-005` 的 audience 映射、模板版本与重试联查步骤落到文档。
- 验证步骤：
  1. `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/migrate-up.sh`
  2. `cargo fmt --all`
  3. `cargo check -p platform-core`
  4. `cargo test -p platform-core`
  5. `cargo check -p notification-worker`
  6. `cargo test -p notification-worker`
  7. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  8. `./scripts/check-query-compile.sh`
  9. `NOTIF_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core notif005_delivery_completion_notifications_db_smoke -- --nocapture`
  10. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv002_file_delivery_commit_db_smoke -- --nocapture`
  11. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv007_api_delivery_db_smoke -- --nocapture`
  12. `POST /internal/notifications/templates/preview` 分别预览：
      - `order.pending_acceptance / buyer / NOTIFY_PENDING_ACCEPTANCE_V1`
      - `delivery.completed / ops / NOTIFY_DELIVERY_COMPLETED_V1`
  13. `POST /internal/notifications/send` 手工注入三类 Kafka 事件并回查：
      - 待验收正常送达
      - 交付完成正常送达 + 同一 `event_id` 二次重放去重
      - 强制失败一次后重试成功
  14. 用 `psql`、`redis-cli`、`curl /metrics`、Prometheus API 回查 `ops.consumer_idempotency_record`、`ops.system_log`、`ops.trace_index`、`audit.audit_event`、Redis 短状态/重试队列、`notification_worker_events_total`、`notification_worker_send_total`、`notification_worker_retry_queue_depth`
  15. 清理本批手工验证产生的非 append-only 辅助状态：`ops.consumer_idempotency_record`、`ops.trace_index`、Redis 短状态与重试载荷；`audit.audit_event` / `ops.system_log` 按留痕保留。
- 验证结果：
  - `077` 已正式应用到本地数据库，`ops.notification_template` 回查显示：
    - `NOTIFY_DELIVERY_COMPLETED_V1` active version=`2`
    - `NOTIFY_PENDING_ACCEPTANCE_V1` active version=`2`
    - version `1` 均已归档
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo check -p notification-worker`、`cargo test -p notification-worker`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh` 全部通过。
  - `notif005_delivery_completion_notifications_db_smoke` 真实验证六类交付场景均能写出 3 条通知 canonical outbox，且买方在文件/报告分支命中 `order.pending_acceptance / NOTIFY_PENDING_ACCEPTANCE_V1`，在共享/API/查询/沙箱分支命中 `delivery.completed / NOTIFY_DELIVERY_COMPLETED_V1`；卖方/运营统一命中 `delivery.completed / NOTIFY_DELIVERY_COMPLETED_V1`。
  - `dlv002_file_delivery_commit_db_smoke` 与 `dlv007_api_delivery_db_smoke` 均通过，证明真实文件/API 交付仓储路径已经在原事务内写出通知 canonical outbox，而不是只在测试注入逻辑中成立。
  - 模板预览验证通过：
    - 待验收样例命中 `NOTIFY_PENDING_ACCEPTANCE_V1` version `2`，正文包含订单、商品、金额、交付分支与“查看并验收”入口
    - 运营交付完成样例命中 `NOTIFY_DELIVERY_COMPLETED_V1` version `2`，正文包含 `delivery_ref=query_execution_run/... receipt=... commit=...` 联查字段
  - Worker 运行态验证通过：
    - 待验收样例：`ops.consumer_idempotency_record.result_code=processed / attempt=1`，`ops.system_log` 写入 `notification sent via mock-log`，`ops.trace_index.root_span_name=notification.dispatch`，`audit.audit_event.action_name=notification.dispatch.sent`
    - 同一 `event_id` 的交付完成样例二次投递后，`notification_worker_events_total{result="duplicate"}` 增长，数据库未新增第二条处理记录
    - 强制失败一次的交付完成样例先写 `notification.dispatch.retry_scheduled / failed`，Redis `notification:retry-queue` 深度由 `1` 回到 `0`，最终在 attempt=`2` 成功送达，`ops.trace_index` 同时可见 `notification.retrying` 与 `notification.dispatch`
  - Prometheus 回查通过：
    - `up{job="notification-worker"}=1`
    - `notification_worker_events_total{result="processed"}=6`
    - `notification_worker_events_total{result="duplicate"}=1`
    - `notification_worker_events_total{result="retrying"}=1`
    - `notification_worker_send_total{channel="mock-log",result="success"}=6`
    - `notification_worker_send_total{channel="mock-log",result="failed"}=1`
    - `notification_worker_retry_queue_depth=0`
  - 清理结果：本批手工验证的 Redis 短状态/重试载荷与 `ops.consumer_idempotency_record`、`ops.trace_index` 已按规则清理；`audit.audit_event` 与 `ops.system_log` 作为运行留痕保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`NOTIF-005`
  - `业务流程图-V1-完整版.md`：交付、验真与验收主流程中的“交付完成 -> 待验收”时机
  - `页面说明书-V1-完整版.md`：验收页、订单详情页入口与不同 audience 的最小可见字段
  - `A10-NOTIF-通知链路与命名边界缺口.md`：正式 topic、scene catalog、渠道边界与 `notification-worker` 命名冻结
  - `notification-worker.md`、`kafka-topics.md`、`topics.v1.json`、`072/074`：正式消费链、route authority、consumer group 与 canonical outbox 口径
- 覆盖的任务清单条目：`NOTIF-005`
- 未覆盖项：
  - 不提前实现 `AUD-009` outbox publisher；本批只验证交付桥接点写入 canonical outbox 与 worker 消费两端。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`docs/开发任务/V1-Core-TODO与预留清单.md` 本批无需变更。
- 备注：
  - `V1-Core-人工审批记录.md` 仍由你手工维护；本批未写入。
  - 查询结果可取场景保持冻结 source-event 口径：`delivery.query_execution_run / delivery.template_query.use`，未发明新的旁路事件名。
