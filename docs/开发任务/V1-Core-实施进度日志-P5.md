# V1-Core 实施进度日志 P5

本文件是实施进度日志的当前续写分卷正文。

- 正式入口页：`docs/开发任务/V1-Core-实施进度日志.md`
- 当前活动分卷以入口页为准；当前入口页指向本卷
- 若后续切换到新的 `P{N}` 分卷，必须先更新入口页，再开始续写新分卷

### BATCH-214（计划中）
- 任务：AUD-001 统一 `AuditEvent` 模型与共享审计写入基座
- 状态：计划中
- 说明：从 `AUD` 阶段起点开始收口统一审计模型，先以冻结任务源、审计 PRD、审计接口协议、领域模型和 `055_audit_hardening.sql` 为权威依据，扩展 `audit-kit` 统一事件模型，补齐与 `DB schema / API DTO` 对齐的正式字段语义，并在 `modules/audit` 中建立可供后续 `audit writer / 联查 / 导出` 复用的共享映射基础。当前批次不把历史业务模块一次性全部改完，而是先建立唯一权威模型与可复用映射入口，为 `AUD-002 / AUD-029` 的证据对象与历史模块 writer 收口提供稳定基座。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-001`，完成后再进入 `AUD-002`。
### BATCH-214（待审批）
- 任务：`AUD-001` 统一 `AuditEvent` 模型与共享审计写入基座
- 状态：待审批
- 当前任务编号：`AUD-001`
- 前置依赖核对结果：`CORE-007`、`CORE-008`、`DB-008`、`ENV-022` 已作为前序基线完成；`NOTIF-001 ~ NOTIF-014` 已全部完成并本地提交，当前正式进入 `AUD` 阶段。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-001` 的依赖、DoD、验收与 `technical_reference`。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `AUD` 阶段 PostgreSQL/MinIO/Kafka/Redis/Keycloak/OpenSearch/Fabric/观测栈职责边界与双层权威模型。
  - `docs/原始PRD/审计、证据链与回放设计.md`：提取五层审计体系、统一审计事件最小字段集与正式审计对象清单。
  - `docs/原始PRD/双层权威模型与链上链下一致性设计.md`、`docs/原始PRD/链上链下技术架构与能力边界稿.md`、`docs/原始PRD/日志、可观测性与告警设计.md`：确认审计域与观测域分层、Fabric 是证明层而非主状态机。
  - `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`、`docs/数据库设计/接口协议/一致性与事件接口协议正式版.md`：确认 `AuditTrace`、导出/回放/legal hold 与后续事件链路协议。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：确认现有领域模型中的 `AuditEvent` 聚合仍过于简化，需要在本批收口。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`、`infra/docker/docker-compose.local.yml`：复核后续 `AUD` 链路基线。
  - `docs/数据库设计/V1/upgrade/050_audit_search_dev_ops.sql`、`docs/数据库设计/V1/upgrade/055_audit_hardening.sql`：对齐 `audit.audit_event` 强化字段与 `evidence_manifest / replay_job / anchor_batch / legal_hold / access_audit` 相关表结构。
  - `docs/开发任务/问题修复任务/A06-Audit-Kit-统一模型漂移.md`、`A02-统一事件-Envelope-与路由权威源.md`、`A03-统一事务模板-落地真实审计与Outbox-Writer.md`、`A14-AUD-历史模块统一Writer与证据桥接缺口.md`：确认本批必须先建立唯一统一模型与共享 writer foundation。
  - `apps/platform-core/src/modules/audit/**`、`apps/platform-core/src/modules/integration/**`、`apps/platform-core/src/modules/order/**`、`apps/platform-core/src/modules/delivery/**`、`apps/platform-core/src/modules/billing/**`、`apps/platform-core/src/modules/catalog/**`、`services/fabric-adapter/**`、`services/fabric-event-listener/**`、`services/fabric-ca-admin/**`、`workers/outbox-publisher/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/04-runbooks/**`、`docs/05-test-cases/**`、`infra/**`、`scripts/**`：确认当前 `audit` 模块仍是空骨架，历史模块普遍存在 ad-hoc `audit.audit_event` 写入。
- 实现要点：
  - `apps/platform-core/crates/audit-kit/src/lib.rs`：将统一模型扩展为正式 `AuditEvent`，补齐 `event_schema_version / event_class / domain_name / ref_type / ref_id / actor / tenant / request_id / trace_id / action_name / result_code / error_code / auth / hash chain / evidence_manifest / retention / legal_hold / sensitivity / occurred_at / ingested_at / metadata`；同时新增 `EvidenceManifest`、`EvidencePackage`、`ReplayJob`、`ReplayResult`、`AnchorBatch`、`LegalHold`、`RetentionPolicy`、`AuditAccessRecord` 等正式对象，并保留 `AuditWriter` 接口。
  - `apps/platform-core/src/modules/audit/domain/mod.rs`：作为统一审计领域导出层，避免 `platform-core` 内再次扩散第二套审计 DTO。
  - `apps/platform-core/src/modules/audit/dto/mod.rs`：新增 `AuditTraceView`，把统一模型直接投影到后续联查/读取路径需要的字段集合。
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：新增 `AuditEventInsert` 与 `INSERT_AUDIT_EVENT_SQL`，收口后续审计 writer 的 DB 强化 schema 映射；由于 `audit.audit_event` 没有独立 `tenant_id` 列，本批约定将其保存在 `metadata.tenant_id`。
  - `apps/platform-core/src/modules/audit/tests/mod.rs`、`apps/platform-core/tests/audit_model_contract_integration.rs`：新增模块测试与跨 crate 集成测试，证明统一模型能被 writer foundation / lookup/export DTO 直接复用。
  - `apps/platform-core/crates/db/src/tests.rs`：`TransactionBundle` 样例切换到 `AuditEvent::business(...)`，验证统一事务模板仍能消费新的统一模型。
  - `apps/platform-core/src/lib.rs`：导出 `pub mod modules;` 以承载 crate 级集成测试与后续审计模块复用。
  - `docs/开发任务/V1-Core-实施进度日志.md`：入口页已切换到 `P5`。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `cargo test -p audit-kit`
  5. `cargo test -p db`
  6. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
  8. `cargo test -p platform-core modules::audit -- --nocapture`
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅有仓库既存 `unused_*` warning，无新增编译失败。
  - `cargo test -p platform-core` 通过：`262` 个单元/模块测试全部通过，新增 `tests/audit_model_contract_integration.rs` 也通过；既存 `iam_party_access_flow_live` 继续保持 ignored。
  - `cargo test -p audit-kit` 通过，新增统一模型默认值和正式对象序列化测试通过。
  - `cargo test -p db` 通过，证明 `TransactionBundle` 与 `TxTemplate` 已能消费新的统一 `AuditEvent` 样例。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存。
  - `./scripts/check-query-compile.sh` 通过；首次与 `sqlx prepare` 并发时出现离线缓存缺失，待 `sqlx prepare` 完成后单独重跑即通过，判定为并发读取旧缓存，而非口径冲突。
  - `cargo test -p platform-core modules::audit -- --nocapture` 通过，验证统一模型到 writer foundation / trace view 的映射。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-001`
  - `审计、证据链与回放设计.md`：第 `3`、`4`、`5` 节的统一事件与正式审计对象语义
  - `审计、证据链与回放接口协议正式版.md`：`AuditTrace` 核心字段与高风险审计控制面基础对象
  - `050_audit_search_dev_ops.sql`、`055_audit_hardening.sql`：`audit.audit_event` 强化字段和审计域正式对象表结构
  - `A06 / A03 / A14`：统一模型漂移、writer 收口和历史模块桥接的前置要求
- 覆盖的任务清单条目：`AUD-001`
- 未覆盖项：
  - 历史业务模块统一接入共享 `audit writer`、替换 ad-hoc `INSERT audit.audit_event` 写法，留待 `AUD-029`。
  - `EvidenceItem / EvidenceManifest` 真实落 PostgreSQL + MinIO、导出包、回放、legal hold、anchor batch、Fabric 写链与回执，留待 `AUD-002 ~ AUD-007` 及后续 `AUD` 批次。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已在 `docs/开发任务/V1-Core-TODO与预留清单.md` 增补 `BATCH-214` 的“无新增 gap/reserved”记录。
- 备注：
  - 本批没有发现 `CSV / Markdown / technical_reference / schema` 之间的冲突，不触发暂停条件。
  - `tenant_id` 在统一模型层是正式字段；鉴于当前 `audit.audit_event` 没有独立列，本批在共享 writer foundation 中将其显式保存在 `metadata.tenant_id`，避免后续 writer / export / lookup 语义丢失。
### BATCH-215（计划中）
- 任务：AUD-002 统一 `EvidenceItem` / `EvidenceManifest` 模型与证据桥接基座
- 状态：计划中
- 说明：在 `AUD-001` 已完成统一审计事件模型的基础上，继续收口统一证据对象。当前批次将以 `audit.evidence_item / audit.evidence_manifest / audit.evidence_manifest_item` 为正式权威落点，补齐共享 evidence writer foundation、shared DTO / DB 映射与历史 `support.evidence_object` 桥接语义，并优先把现有“争议证据上传 -> MinIO”这条真实链路桥接到正式审计证据模型，证明 `PG + 对象存储` 双写不是假接入。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-002`，完成后再进入 `AUD-003`。
### BATCH-215（待审批）
- 任务：`AUD-002` 统一 `EvidenceItem` / `EvidenceManifest` 模型与证据桥接基座
- 状态：待审批
- 当前任务编号：`AUD-002`
- 前置依赖核对结果：`CORE-007`、`CORE-008`、`DB-008`、`ENV-022` 已作为前序基线完成；`AUD-001` 已于本地提交 `d1fadf5` 完成，当前批次可继续收口统一证据 authority model。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-002` 的 DoD、验收与 `technical_reference`，要求 `EvidenceItem / EvidenceManifest` 同时支持 `PG + 对象存储` 双写语义，并作为导出、回放、legal hold、历史桥接的统一对象。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `PostgreSQL / MinIO / Kafka / Redis / IAM / OpenSearch / Fabric / 观测栈` 的职责边界，确认 `MinIO` 是证据对象承载层、`PostgreSQL` 是审计索引主权威。
  - `docs/原始PRD/审计、证据链与回放设计.md`：提取证据清单、证据包、回放、legal hold 的统一对象语义，确认 `EvidenceManifest` 必须能作为导出/回放/法证联查入口。
  - `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`：确认 `evidence_manifest_id`、`replay_job`、导出接口与 legal hold 控制面的字段口径。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：确认 `审计与证据聚合` 的正式对象边界。
  - `docs/开发任务/问题修复任务/A06-Audit-Kit-统一模型漂移.md`、`A14-AUD-历史模块统一Writer与证据桥接缺口.md`：确认本批必须先落地统一 `EvidenceItem / EvidenceManifest` authority model，再把历史 `support.evidence_object` 桥接到 `audit` 域。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`、`infra/docker/docker-compose.local.yml`：复核后续 `AUD` 链路共用基线，无新增 topic/route authority 偏移。
  - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`、`050_audit_search_dev_ops.sql`、`055_audit_hardening.sql`：确认历史 `support.evidence_object` 仍存在，而正式 `audit.evidence_item / audit.evidence_manifest / audit.evidence_manifest_item` 已具备承载 authority model 的 schema。
  - `apps/platform-core/src/modules/audit/**`、`apps/platform-core/src/modules/billing/repo/dispute_repository.rs`、`apps/platform-core/src/modules/billing/tests/bil013_dispute_case_db.rs`、`apps/platform-core/src/modules/storage/application/object_store.rs`：确认现有真实“争议证据上传 -> MinIO -> support.evidence_object”链路可作为桥接切入点，但不能继续停留在私有证据表。
- 实现要点：
  - `apps/platform-core/crates/audit-kit/src/lib.rs`：补齐 `EvidenceManifestItem` 正式模型，使 `EvidenceItem / EvidenceManifest / EvidenceManifestItem` 三层结构完整可序列化。
  - `apps/platform-core/src/modules/audit/domain/mod.rs`、`dto/mod.rs`：新增 `EvidenceItemView / EvidenceManifestView / EvidenceManifestItemView`，统一承接导出、回放、legal hold 与历史桥接的读取投影，避免后续再分裂第二套 DTO。
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：新增 `INSERT_EVIDENCE_ITEM_SQL`、`INSERT_EVIDENCE_MANIFEST_SQL`、`INSERT_EVIDENCE_MANIFEST_ITEM_SQL` 与对应 insert 映射，形成共享 evidence writer 的 DB 权威写入口。
  - `apps/platform-core/src/modules/audit/application/mod.rs`：新增共享 evidence writer foundation，提供 `record_evidence_snapshot`、`bridge_support_evidence_object`、`bridge_metadata`。当前实现会在证据上传后真实写入 `audit.evidence_item`，按同一 `manifest_ref_type/ref_id` 装载当前作用域下的所有证据项，生成 append-only `audit.evidence_manifest` 快照和 `audit.evidence_manifest_item` 关联，并把桥接 ID 回写到历史 `support.evidence_object.metadata`。
  - `apps/platform-core/src/modules/billing/repo/dispute_repository.rs`：将现有真实 `upload_dispute_evidence` 流程从“仅写 `support.evidence_object`”扩展为“MinIO 上传 + `support.evidence_object` + `audit.evidence_item / manifest / manifest_item` + 历史桥接元数据回写”同事务链路，保持 `support.evidence_object` 兼容读取，同时明确其不再是唯一 authority。
  - `apps/platform-core/src/modules/billing/tests/bil013_dispute_case_db.rs`：扩展真实 smoke，对 `support.evidence_object.metadata.audit_evidence_*`、`audit.evidence_item`、`audit.evidence_manifest`、`audit.evidence_manifest_item` 做数据库回查，并继续通过 `fetch_object_bytes` 从 MinIO 取回对象，证明 `PG + 对象存储` 双写与历史桥接同时成立。
  - `apps/platform-core/src/modules/audit/tests/mod.rs`：新增正式字段映射测试，验证 `EvidenceItem / EvidenceManifest / EvidenceManifestItem` 的读取视图能稳定承载导出、回放、legal hold 和历史桥接所需字段。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `cargo test -p audit-kit`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil013_dispute_case_db_smoke -- --nocapture`
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；输出仍只有仓库既存 `unused_*` warning，无新增编译失败。
  - `cargo test -p platform-core` 通过：`263` 个测试全部通过，新增 `evidence_views_preserve_export_replay_legal_hold_and_history_bridge_fields` 也通过。
  - `cargo test -p audit-kit` 通过，证明新增 `EvidenceManifestItem` 模型与统一证据对象序列化保持稳定。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存；新增 evidence writer SQL 可被离线模式识别。
  - `./scripts/check-query-compile.sh` 通过。
  - `TRADE_DB_SMOKE=1 ... bil013_dispute_case_db_smoke` 通过：真实完成争议开案、证据上传、MinIO 对象写入与回读、`support.evidence_object -> audit.evidence_item / audit.evidence_manifest / audit.evidence_manifest_item` 桥接回查，并保留审计域证据对象不做清理。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-002`
  - `审计、证据链与回放设计.md`：第 `3`、`5` 节的统一证据对象、证据包、回放、legal hold 语义
  - `审计、证据链与回放接口协议正式版.md`：`evidence_manifest_id`、导出、回放与高风险控制面对象
  - `040_billing_support_risk.sql`、`055_audit_hardening.sql`：历史 `support.evidence_object` 与正式 `audit.evidence_*` 表结构
  - `A06 / A14`：统一 evidence authority model 与历史证据桥接缺口修复
- 覆盖的任务清单条目：`AUD-002`
- 未覆盖项：
  - 导出包对象实际写入 MinIO、`legal hold` 持久化与回放任务编排，留待 `AUD-003 ~ AUD-006`。
  - 更大范围的历史模块统一接入 evidence writer，留待 `AUD-029` 继续收口。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`。
- 备注：
  - 本批没有发现 `CSV / Markdown / technical_reference / schema / 现有代码` 之间的冲突，不触发暂停条件。
  - 当前桥接策略明确把 `support.evidence_object` 定位为兼容历史读取的落点，正式 authority 已收口到 `audit.evidence_item / audit.evidence_manifest / audit.evidence_manifest_item`。
### BATCH-216（计划中）
- 任务：AUD-003 订单审计联查与全局 `audit trace` API
- 状态：计划中
- 说明：在 `AUD-001 / AUD-002` 已完成统一审计/证据 authority model 的基础上，当前批次补齐 `GET /api/v1/audit/orders/{id}` 与 `GET /api/v1/audit/traces` 两条正式读取控制面。实现将覆盖最小披露 DTO、`audit.trace.read` 角色与 tenant/order scope 校验、读取行为的 `audit.access_audit + ops.system_log` 留痕、主 router 挂载，以及 `packages/openapi/audit.yaml` 与 `docs/02-openapi/audit.yaml` 的同步落盘，并通过真实 API + DB smoke 验证“读接口不是空壳”。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-003`，完成后再进入 `AUD-004`。
### BATCH-216（待审批）
- 任务：AUD-003 订单审计联查与全局 `audit trace` API
- 状态：待审批
- 完成情况：
  - 已新增 `apps/platform-core/src/modules/audit/api` 正式路由与 handler，挂载 `GET /api/v1/audit/orders/{id}`、`GET /api/v1/audit/traces` 到主应用 router，不再停留在 README / OpenAPI 草稿。
  - 已补齐 `OrderAuditView / AuditTraceQuery / AuditTracePageView` 等读取 DTO，并在仓储层实现 `trade.order_main + audit.audit_event` 的 order scope 校验、按 `order_id / ref_type / ref_id / request_id / trace_id / action_name / result_code` 过滤查询，以及 `audit.access_audit + ops.system_log` 双留痕。
  - 已落实正式权限口径：平台与监管只需 `audit.trace.read` 级读取权限；租户侧必须同时满足 `x-tenant-id` 与订单 buyer/seller scope 命中，避免通过全局 trace 查询越权读他租户订单。
  - 已同步补齐 `packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`、OpenAPI README 与 `scripts/check-openapi-schema.sh`，把 `AUD-003` 的两条正式读取接口作为已实现契约归档，而不是继续留在占位状态。
  - 已新增 `apps/platform-core/src/modules/audit/tests/api_db.rs`，覆盖缺失权限、缺失 `x-request-id`、平台读取、租户按订单范围读取、跨租户拒绝，以及真实 `audit.access_audit / ops.system_log` DB 回查。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - `cargo test -p platform-core modules::audit -- --nocapture` 通过，新增 audit API 路由测试和 DB smoke 通过。
  - `AUD_DB_SMOKE=1 cargo test -p platform-core` 通过：全量 `platform-core` 测试 `266 passed; 0 failed`，其中 `modules::audit::tests::api_db::audit_trace_api_db_smoke` 真实完成 `GET /api/v1/audit/orders/{id}` 与 `GET /api/v1/audit/traces` API 调用、订单级与全局 trace 查询、跨租户 `403` 拒绝，以及 `audit.access_audit / ops.system_log` 回查。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过，确认 `packages/openapi/audit.yaml` 与 `docs/02-openapi/audit.yaml` 同步，且包含 `AUD-003` 两条正式路径。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-003`
  - `审计、证据链与回放设计.md`：订单审计联查、全局 trace 检索与访问行为留痕
  - `审计、证据链与回放接口协议正式版.md`：`/api/v1/audit/orders/{id}` 与 `/api/v1/audit/traces` 的 DTO / 权限 / 错误口径
  - `全量领域模型与对象关系说明.md`：订单与审计对象聚合边界
  - `A04-AUD-Ops-接口与契约落地缺口.md`：审计查询控制面与 OpenAPI 归档缺口修复
- 覆盖的任务清单条目：`AUD-003`
- 未覆盖项：
  - 证据包导出、理由强制、step-up 校验与 MinIO 导出对象写入，留待 `AUD-004`。
  - legal hold、replay、dead letter reprocess 与一致性修复控制面，留待后续 `AUD` 批次。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 收敛为“`AUD-003` 已交付读接口，后续仅保留导出/回放/ops 控制面缺口”。
- 备注：
  - 本批没有发现 `CSV / Markdown / technical_reference / schema / 现有代码` 之间的冲突，不触发暂停条件。
  - 读取行为的正式审计域为 `audit.access_audit`，普通运行日志仅作为 `ops.system_log` 观测辅助，不替代审计权威源。
### BATCH-217（计划中）
- 任务：AUD-004 证据包导出接口
- 状态：计划中
- 说明：在 `AUD-002` 的统一 `EvidenceItem / EvidenceManifest` authority model 与 `AUD-003` 的审计联查读取控制面基础上，当前批次补齐 `POST /api/v1/audit/packages/export`。实现将覆盖平台级 `audit.package.export` 权限、导出理由必填、`x-step-up-token / x-step-up-challenge-id` 校验、`audit.evidence_package` + MinIO 导出对象双写、正式 `audit.audit_event + audit.access_audit + ops.system_log` 三层留痕，以及 `packages/openapi/audit.yaml` / `docs/02-openapi/audit.yaml` 的同步契约更新，并通过真实 API + DB + MinIO smoke 证明导出包不是空壳记录。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-004`，完成后再进入 `AUD-005`。
### BATCH-217（待审批）
- 任务：`AUD-004` 证据包导出接口
- 状态：待审批
- 当前任务编号：`AUD-004`
- 前置依赖核对结果：`AUD-001` 统一 `AuditEvent` authority model、`AUD-002` 统一 `EvidenceItem / EvidenceManifest` authority model、`AUD-003` 订单审计联查与全局 trace API 已本地提交完成；导出接口所依赖的 `audit.evidence_*` authority model、审计联查读取控制面与 OpenAPI 归档基线已满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-004` DoD 为 `POST /api/v1/audit/packages/export` 的接口、DTO、权限、审计、错误码与最小测试完成，且实现不得偏离 OpenAPI。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `PostgreSQL + MinIO + Redis + Keycloak/IAM + 观测栈` 在 `AUD` 高风险控制面中的职责边界，确认导出必须是真对象写入，不允许只留日志或占位记录。
  - `docs/原始PRD/审计、证据链与回放设计.md`：确认导出包必须服务订单/案件证据联查，高风险动作必须要求理由、权限、step-up 与正式审计。
  - `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`：确认 `POST /api/v1/audit/packages/export` 的请求字段、`audit.package.export` 权限、`reason` 必填、step-up 绑定与最小错误口径。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：确认导出包覆盖 order / case 及其关联审计、证据、legal hold 摘要对象。
  - `docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`：确认导出控制面必须落到正式 router + OpenAPI，而不是只写 README / 草稿。
  - `docs/开发任务/问题修复任务/A06-Audit-Kit-统一模型漂移.md`：确认 `EvidencePackage` 必须复用统一 authority model，不能额外发明第二套 DTO / writer。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`、`infra/docker/docker-compose.local.yml`：确认本批不新增 topic/route authority，导出仍是 `platform-core` 内部高风险控制面，不通过旁路 worker。
  - `docs/数据库设计/V1/upgrade/050_audit_search_dev_ops.sql`、`docs/数据库设计/V1/upgrade/055_audit_hardening.sql`：确认 `audit.evidence_package` 的正式字段为结构化列，不存在 `metadata` 列；运行态验证中也已回查本地库结构与文档一致，导出附加信息应保存在 `audit.audit_event / audit.access_audit / evidence_manifest.metadata` 等正式审计对象中。
  - `apps/platform-core/src/modules/audit/**`、`packages/openapi/**`、`docs/02-openapi/**`、`scripts/check-openapi-schema.sh`：确认现有实现仅覆盖读取控制面，导出接口仍未正式落地。
- 实现要点：
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `POST /api/v1/audit/packages/export`，要求 `x-request-id`、平台级 `audit.package.export` 权限、`reason` 必填、`x-step-up-token / x-step-up-challenge-id` 至少其一，且 `x-step-up-challenge-id` 必须绑定当前 actor、状态 `verified`、未过期，并兼容现网已有 `audit.evidence.export` challenge action。
  - 导出目标支持 `order / case(dispute_case)`；读取 `trade.order_main` 或 `support.dispute_case` 的正式快照，并串联 `audit.audit_event`、`audit.evidence_manifest / item`、历史 `support.evidence_object`、`audit.legal_hold` 生成统一证据包 JSON，按 `masked_level=summary|masked|unmasked` 做最小披露。
  - `apps/platform-core/src/modules/storage/application/object_store.rs` 既有 MinIO writer 被真实复用：导出包对象写入 `s3://evidence-packages/exports/{ref_type}/{ref_id}/package-{id}.json`，失败时回滚 DB 事务并删除临时对象。
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：新增 `insert_evidence_package` 正式 writer；实现按 `audit.evidence_package` 真实结构列写入，不再假定存在 `metadata` 列，同时保留统一 `EvidencePackage` 响应对象中的 `metadata` 供 API 返回使用。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：扩展路由级权限/step-up 校验，并新增带 `AUD_DB_SMOKE=1` 的真实导出链路测试，覆盖 `MinIO` 对象上传与回读、`audit.evidence_package`、`audit.audit_event`、`audit.access_audit`、`ops.system_log`、`audit.evidence_manifest_item` 回查。
  - `packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`、README、`scripts/check-openapi-schema.sh`：补齐导出路径、请求/响应 schema 与文档落盘。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core modules::audit -- --nocapture`
  4. `cargo test -p platform-core`
  5. `AUD_DB_SMOKE=1 cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture`
  6. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
  8. `./scripts/check-openapi-schema.sh`
  9. 真实 HTTP 联调：`DATABASE_URL=... APP_PORT=18080 cargo run -p platform-core-bin` + `curl -X POST http://127.0.0.1:18080/api/v1/audit/packages/export ...`
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - `cargo test -p platform-core modules::audit -- --nocapture` 通过；新增导出路由校验和 DB smoke 全部通过。
  - `cargo test -p platform-core` 通过：`268 passed; 0 failed`，确认本批改动未回归既有业务主链。
  - `AUD_DB_SMOKE=1 cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture` 通过；真实完成导出包对象上传、`audit.evidence_package` / `audit.audit_event` / `audit.access_audit` / `ops.system_log` 回查，以及 MinIO 对象读取验证。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过。
  - 真实 HTTP 联调通过：显式以 `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab APP_PORT=18080 cargo run -p platform-core-bin` 启动服务后，执行 `curl -X POST http://127.0.0.1:18080/api/v1/audit/packages/export` 返回 `200`，并回查到：
    - `audit.evidence_package`：`package_type=order_evidence_package`、`masked_level=masked`、`access_mode=export`
    - `audit.audit_event`：`action_name=audit.package.export`，`metadata.reason='curl export verification'`
    - `audit.access_audit`：`access_mode=export`
    - `ops.system_log`：`audit package export executed: POST /api/v1/audit/packages/export`
  - 运行态修正证明：真实 `AUD_DB_SMOKE` 暴露了两个实现偏差，已在本批修正并复验通过：
    - 导出 smoke 原先直接引用不存在的 `core.user_account`，现改为先种最小用户再创建 `iam.step_up_challenge`。
    - `audit.evidence_package` 运行态 schema 不存在 `metadata` 列，现已按正式结构化列落库，并把附加导出信息放入 `audit.audit_event / audit.access_audit / evidence_manifest.metadata`。
### BATCH-219（计划中）
- 任务：AUD-006 legal hold 控制面接口
- 状态：计划中
- 说明：在 `AUD-003 ~ AUD-005` 已落地审计联查、证据包导出和 replay dry-run 控制面的基础上，本批补齐 `POST /api/v1/audit/legal-holds` 与 `POST /api/v1/audit/legal-holds/{id}/release`。实现将复用统一 `LegalHold` authority model 与既有 step-up / 权限 / 审计 helper，要求高风险动作真实经过 `Keycloak/IAM` 权限与 step-up、把 legal hold 状态落 `audit.legal_hold`、并将创建/释放行为同步写入 `audit.audit_event + audit.access_audit + ops.system_log`，同时补齐 OpenAPI、runbook、验收矩阵和至少一条真实 API + DB 验证。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-006`，完成后再进入 `AUD-007`。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-006`
  - `审计、证据链与回放设计.md`：legal hold、高风险动作、正式审计与最小披露要求
  - `审计、证据链与回放接口协议正式版.md`：`POST /api/v1/audit/legal-holds`、`POST /api/v1/audit/legal-holds/{id}/release` 的请求、权限、理由与 step-up 口径
  - `全量领域模型与对象关系说明.md`：order / dispute_case 与审计 / hold 聚合边界
  - `050_audit_search_dev_ops.sql`、`055_audit_hardening.sql`：`audit.legal_hold`、`audit.access_audit`、`audit.audit_event` 正式 schema 与 append-only 约束
  - `A04`、`A06`：控制面契约落地与统一 authority model 收口
- 覆盖的任务清单条目：`AUD-006`
- 未覆盖项：
  - anchor / Fabric callback、reconcile、dead letter reprocess 与一致性修复高风险接口，留待 `AUD-007+`。
  - Fabric 锚定、回执、外部事实与双层权威联查，留待后续 `AUD` 批次。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛为“`AUD-003 ~ AUD-006` 已交付查询 / 导出 / replay / legal hold 控制面，后续仅跟踪 anchor / Fabric / ops / 一致性矩阵缺口”。
- 备注：
  - 本批没有发现需要人工确认的 `CSV / Markdown / technical_reference` 冲突，不触发暂停条件。
  - 手工清理验证时，尝试删除已被 append-only `audit.audit_event` 引用的 `iam.step_up_challenge` 会触发 FK 的 `SET NULL -> UPDATE audit.audit_event`，被 append-only trigger 正常拒绝；因此高风险动作验证产生的 challenge 记录按运行态现状保留，不把审计域强行改造成可回写对象。
### BATCH-219（待审批）
- 任务：`AUD-006` legal hold 控制面接口
- 状态：待审批
- 当前任务编号：`AUD-006`
- 前置依赖核对结果：`AUD-001` 统一 `AuditEvent` authority model、`AUD-002` 统一 `EvidenceItem / EvidenceManifest` authority model、`AUD-003` 审计读取控制面、`AUD-004` 证据包导出控制面、`AUD-005` replay dry-run 控制面均已本地提交完成；`audit.legal_hold` 正式 schema、平台级审计角色门面、`iam.step_up_challenge` 运行态链路与本地 `PostgreSQL / MinIO / Kafka / OpenSearch / Redis / Keycloak` 基线满足当前批次依赖。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-006` DoD 为 `POST /api/v1/audit/legal-holds`、`POST /api/v1/audit/legal-holds/{id}/release` 的接口、DTO、权限校验、错误码、审计与最小测试齐备，并要求至少一条真实 API/集成验证。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `AUD` 阶段高风险控制面必须真实经过 `PostgreSQL / Redis / Kafka / Keycloak/IAM / MinIO / OpenSearch / 观测栈` 的职责边界；本批 legal hold 不允许只留骨架接口。
  - `docs/原始PRD/审计、证据链与回放设计.md`：确认 `legal hold` 属于高风险动作，必须经过 step-up、正式审计与后续导出/回放联查。
  - `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`：确认 `audit.legal_hold.manage` 是正式权限点，`hold_scope_type / hold_scope_id / reason_code / metadata` 是创建入参，释放动作需要 `reason`，并要求有正式错误码与运行态审计留痕。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：确认 `order / dispute_case / evidence_package` 与 `legal hold` 的聚合边界；当前 hold authority 在 `audit.legal_hold`，不是历史 evidence 快照。
  - `docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`、`docs/开发任务/问题修复任务/A06-Audit-Kit-统一模型漂移.md`：确认 legal hold 控制面必须落正式 router/OpenAPI/runbook/test-case，并复用统一 `LegalHold` model，不允许另造临时 DTO。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`072_canonical_outbox_route_policy.sql`、`074_event_topology_route_extensions.sql`、`infra/docker/docker-compose.local.yml`：确认本批不新增旁路 topic / worker；legal hold 是 `platform-core` 内部高风险控制面，不通过 Fabric / outbox 改写主状态。
- 实现要点：
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `POST /api/v1/audit/legal-holds` 与 `POST /api/v1/audit/legal-holds/{id}/release`，要求 `x-request-id`；两个动作都要求平台级 `audit.legal_hold.manage` 权限、`x-user-id` 与 step-up。
  - legal hold create：规范化 `hold_scope_type`，支持 `order` 与 `case/dispute_case`；校验 `hold_scope_id`、`reason_code`、可选 `retention_policy_id / hold_until`；按 scope 唯一 active hold 做 `409 AUDIT_LEGAL_HOLD_ACTIVE` 冲突保护；写入 `audit.legal_hold(status=active)`，并同步追加 `audit.audit_event(action_name='audit.legal_hold.create')` 与 `ops.system_log`。
  - legal hold release：路径参数必须是合法 `legal_hold_id`；step-up challenge 必须绑定 `target_action='audit.legal_hold.manage'` 与 `target_ref_type='legal_hold'`；仅允许释放当前 active hold；更新 `audit.legal_hold(status=released, approved_by, released_at, metadata.release_reason)`，并同步追加 `audit.audit_event(action_name='audit.legal_hold.release')` 与 `ops.system_log`。
  - 运行态口径修正：`audit.evidence_item` 与 `audit.evidence_package` 都由 append-only trigger 保护，不能被 legal hold create/release 原地回写；当前 hold 状态的正式权威源是 `audit.legal_hold`，历史 evidence/package 只保留快照。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：补齐 legal hold 路由级权限 / step-up 测试，并把 live smoke 扩展为 create -> duplicate conflict -> release -> `audit.legal_hold / audit.audit_event / ops.system_log` 全链回查。
  - `packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`、`scripts/check-openapi-schema.sh`、`docs/04-runbooks/audit-legal-hold.md`、`docs/05-test-cases/audit-consistency-cases.md`、相关 README：同步补齐契约、手工 `curl`、SQL 回查、清理约束与 append-only 说明。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core modules::audit -- --nocapture`
  4. `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture`
  5. `cargo test -p platform-core`
  6. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
  8. `./scripts/check-openapi-schema.sh`
  9. 真实 HTTP 联调：
     - `set -a; source infra/docker/.env.local; set +a; APP_HOST=127.0.0.1 APP_PORT=18080 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p platform-core`
     - 通过 `psql` 插入最小 `order` scope 图、`core.user_account` 与两条 `iam.step_up_challenge`
     - `curl -X POST /api/v1/audit/legal-holds` 返回 `200`
     - `curl -X POST /api/v1/audit/legal-holds/{id}/release` 返回 `200`
     - 回查 `audit.legal_hold.status=released`、`metadata.release_reason='manual curl release'`、create/release 各 1 条 `audit.audit_event` 与 `ops.system_log`
- 验证结果：
  - legal hold 路由级权限 / step-up 测试通过；`modules::audit` 模块测试通过。
  - `AUD_DB_SMOKE=1` 的真实 smoke 通过，create / duplicate conflict / release 与 DB 回查一致。
  - 全量 `cargo test -p platform-core` 通过；`cargo check -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh`、`./scripts/check-openapi-schema.sh` 全部通过。
  - 真实 HTTP 联调通过：`/healthz=200`，create/release 两个请求均 `200`，DB 回查到 `audit.legal_hold.status=released`、`release_reason=manual curl release`、`create_audit_count=1`、`release_audit_count=1`、`create_log_count=1`、`release_log_count=1`、`active_hold_count=0`。
  - 运行态修正结论已验证：删除已被 `audit.audit_event` 引用的 `iam.step_up_challenge` 会触发 FK 尝试 `SET NULL`，被 append-only trigger 拒绝；因此临时 `order / catalog.*` 业务图已清理，而与审计链绑定的 `core.user_account / iam.step_up_challenge / audit.legal_hold / audit.audit_event / ops.system_log` 按审计口径保留。
- 备注：
  - 本批没有发现需要人工确认的 `CSV / Markdown / technical_reference` 冲突，不触发暂停条件。
  - `AUD-006` 本地完成后，下一批按顺序进入 `AUD-007`。
### BATCH-220（计划中）
- 任务：AUD-007 AnchorBatch 模型与查看 / 重试接口
- 状态：计划中
- 说明：在 `AUD-003 ~ AUD-006` 已补齐审计联查、证据包导出、replay dry-run 与 legal hold 控制面的基础上，本批继续补齐 `GET /api/v1/audit/anchor-batches` 与 `POST /api/v1/audit/anchor-batches/{id}/retry`。实现将复用统一 `AnchorBatch` authority model，并以 `audit.anchor_batch + audit.anchor_item + chain.chain_anchor` 为正式读取源；retry 动作将作为高风险控制面，要求真实经过 `audit.anchor.manage` 权限与 step-up，落正式 `audit.audit_event / audit.access_audit / ops.system_log`，并按 canonical route 写出 `audit.anchor_requested -> dtp.audit.anchor` 的 outbox 事件，不能自创旁路 topic 或直接把 `dtp.outbox.domain-events` 当正式入口。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-007`，完成后再进入 `AUD-008`。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-007`
  - `技术选型正式版.md`：`Fabric / PostgreSQL / Kafka / MinIO / Redis / Keycloak / OpenSearch / 观测栈` 的职责边界
  - `链上链下技术架构与能力边界稿.md`：链下主状态机 + Fabric 证明层边界，链失败不能反向定义主业务状态
  - `审计、证据链与回放设计.md`：`AnchorBatch / AnchorItem`、审计批次根与 `audit.anchor_requested`
  - `审计、证据链与回放接口协议正式版.md`：`AnchorBatch` 核心字段、`GET /api/v1/audit/anchor-batches`、`POST /api/v1/audit/anchor-batches/{id}/retry`
  - `全量领域模型与对象关系说明.md`：`AnchorBatch 1 -> N AnchorItem` 与链上摘要聚合边界
  - `055_audit_hardening.sql`、`074_event_topology_route_extensions.sql`、`kafka-topics.md`、`fabric-local.md`：`audit.anchor_batch / audit.anchor_requested -> dtp.audit.anchor -> fabric-adapter` 的正式 schema / route / topic 口径
- 预定实现要点：
  - `apps/platform-core/src/modules/audit/domain/**`：补齐 `AnchorBatch` 查询参数、分页视图与 retry 请求/响应 DTO。
  - `apps/platform-core/src/modules/audit/repo/**`：新增 `audit.anchor_batch` 列表查询、`chain.chain_anchor` 联查、failed batch 读取与 retry 元数据更新。
  - `apps/platform-core/src/modules/audit/api/**`：新增 anchor batch read/manage 权限、router、handler、step-up 绑定、错误码与审计留痕。
  - retry 动作仅允许对 `status=failed` 的 batch 发起；成功受理后写 canonical outbox 事件，目标 topic 必须命中 `dtp.audit.anchor`。
  - `packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`、`docs/04-runbooks/**`、`docs/05-test-cases/**`：同步补齐契约、手工 `curl`、DB / outbox 回查与清理约束。
- 备注：
  - 当前没有发现必须暂停的文档冲突；`AnchorBatch` 的字段口径可通过 `audit.anchor_batch` 与 `chain.chain_anchor` 联合映射出 `anchor_status / tx_hash / anchored_at`。
### BATCH-220（待审批）
- 任务：`AUD-007` AnchorBatch 模型与查看 / 重试接口
- 状态：待审批
- 当前任务编号：`AUD-007`
- 前置依赖核对结果：`CORE-007`、`CORE-008`、`DB-008`、`ENV-022` 已满足；`AUD-003 ~ AUD-006` 已补齐 audit query / export / replay / legal hold 控制面，`055_audit_hardening.sql` 已提供 `audit.anchor_batch / audit.anchor_item`，`074_event_topology_route_extensions.sql` 与 `scripts/check-topic-topology.sh` 已冻结并校验 `audit.anchor_batch / audit.anchor_requested -> dtp.audit.anchor` 运行态 route seed。
- 已阅读证据（文件+要点）：
  - `v1-core-开发任务清单.csv / .md`：确认 `AUD-007` DoD 为 `GET /api/v1/audit/anchor-batches`、`POST /api/v1/audit/anchor-batches/{id}/retry` 的接口、DTO、权限、审计、错误码、最小测试与 OpenAPI 对齐。
  - `技术选型正式版.md`、`链上链下技术架构与能力边界稿.md`、`审计、证据链与回放设计.md`、`审计、证据链与回放接口协议正式版.md`、`全量领域模型与对象关系说明.md`：确认 `AnchorBatch / AnchorItem` 是正式 authority model，`Fabric` 只是证明层，retry 只能回写 proof / request 状态，不能反向定义主业务状态。
  - `事件模型与Topic清单正式版.md`、`kafka-topics.md`、`fabric-local.md`、`074_event_topology_route_extensions.sql`：确认正式事件为 `audit.anchor_requested`，目标 topic 为 `dtp.audit.anchor`，不能走 `dtp.outbox.domain-events` 旁路。
  - `apps/platform-core/src/modules/audit/**`、`packages/openapi/**`、`docs/04-runbooks/**`、`docs/05-test-cases/**`：确认现有实现只覆盖 `AUD-003 ~ AUD-006`；anchor batch 查询 / retry、runbook 与验收矩阵仍待当前批次落地。
- 实现摘要：
  - `apps/platform-core/src/modules/audit/domain/mod.rs`：新增 `AnchorBatchQuery`、`AnchorBatchPageView`、`AuditAnchorBatchRetryRequest`、`AuditAnchorBatchRetryView`。
  - `apps/platform-core/src/modules/audit/dto/mod.rs`：新增 `AnchorBatchView`，统一把 `audit.anchor_batch + chain.chain_anchor` 映射为 API 视图，补齐 `tx_hash / anchor_status / anchored_at / window_* / metadata`。
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：新增 anchor batch 列表读取、单批次读取、`failed -> retry_requested` 原子更新，并把 `chain.chain_anchor` 的 `tx_hash / authority_model / reconcile_status / last_reconciled_at` 注入统一 metadata。
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `GET /api/v1/audit/anchor-batches` 与 `POST /api/v1/audit/anchor-batches/{id}/retry`，补齐 `audit.anchor.read / audit.anchor.manage` 权限、step-up challenge 绑定、`AUDIT_ANCHOR_BATCH_NOT_RETRYABLE` 冲突控制、`audit.audit_event / audit.access_audit / ops.system_log` 留痕，以及 canonical outbox `audit.anchor_requested -> dtp.audit.anchor` 写入。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：新增 route-level 权限 / step-up 测试，并扩展 live smoke 到 anchor batch list + retry + DB / outbox / audit / access / system_log 真实回查。
  - `packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`、`packages/openapi/README.md`、`docs/02-openapi/README.md`：补齐 anchor batch 查询 / retry 路径与 schema，并同步归档成熟度说明。
  - `docs/04-runbooks/audit-anchor-batches.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`：补齐手工 `curl`、step-up challenge、SQL / outbox 回查、清理约束与验收矩阵。
- 真实验证：
  1. `cargo check -p platform-core`
  2. `cargo test -p platform-core modules::audit::tests::api_db::route_tests -- --nocapture`
  3. `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture`
  4. live smoke 已真实验证：
     - `GET /api/v1/audit/anchor-batches` 返回 failed batch，回查 `audit.access_audit + ops.system_log`
     - `POST /api/v1/audit/anchor-batches/{id}/retry` 仅在 `status=failed` 时受理
     - `audit.anchor_batch.status` 更新为 `retry_requested`
     - `ops.outbox_event.target_topic='dtp.audit.anchor'`
     - `event_type='audit.anchor_requested'`
     - `audit.audit_event.action_name='audit.anchor.retry'`
     - `audit.access_audit(access_mode='retry')`
     - `ops.system_log` 留痕齐全
- TODO / 预留清单更新：
  - 无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
  - 已更新 `TODO-AUD-OPENAPI-001`、`TODO-AUD-TEST-001`，把 `AUD-007` 的 anchor batch 契约、runbook 与验收矩阵纳入追踪现状。
- 清理与保留：
  - live smoke 产生的 `audit.anchor_batch / audit.audit_event / audit.access_audit / ops.system_log` 按审计 append-only 保留。
  - 与高风险动作绑定的 `core.user_account / iam.step_up_challenge` 未做强删，保持与既有 `AUD-004~006` 相同的审计保留策略。
- 结论：
  - `AUD-007` 已按冻结口径完成并通过真实验证。
  - 下一批应严格顺序进入 `AUD-008`，继续补齐 Fabric request / callback / reconcile 主链，不得把当前 control-plane 完成态误报为 Fabric 全链路已完成。
### BATCH-218（计划中）
- 任务：AUD-005 审计回放任务接口
- 状态：计划中
- 说明：在 `AUD-003` 的审计联查读取控制面与 `AUD-004` 的证据导出控制面基础上，当前批次补齐 `POST /api/v1/audit/replay-jobs`、`GET /api/v1/audit/replay-jobs/{id}`。实现将覆盖平台级 `audit.replay.execute / audit.replay.read` 权限、`V1` 默认 `dry-run` 约束、`x-step-up-token / x-step-up-challenge-id` 校验、`audit.replay_job + audit.replay_result` 正式落库、`audit.audit_event + audit.access_audit + ops.system_log` 三层留痕，以及 `packages/openapi/audit.yaml` / `docs/02-openapi/audit.yaml` / `docs/05-test-cases/audit-consistency-cases.md` 的同步更新，并通过真实 API + DB 手工/集成验证证明 replay 不是占位接口。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-005`，完成后再进入 `AUD-006`。
### BATCH-218（待审批）
- 任务：`AUD-005` 审计回放任务接口
- 状态：待审批
- 当前任务编号：`AUD-005`
- 前置依赖核对结果：`AUD-001` 统一 `AuditEvent` authority model、`AUD-002` 统一 `EvidenceItem / EvidenceManifest` authority model、`AUD-003` 审计读取控制面、`AUD-004` 证据包导出控制面均已本地提交完成；`audit.replay_job / audit.replay_result` schema、MinIO 证据存储基线与平台级审计权限门面已满足当前批次依赖。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-005` DoD 为 `POST /api/v1/audit/replay-jobs`、`GET /api/v1/audit/replay-jobs/{id}` 的接口、DTO、权限、审计、错误码和最小测试齐备，且 `V1` 默认 `dry-run`。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `PostgreSQL + MinIO + Kafka + Redis + Keycloak/IAM + 观测栈` 在 `AUD` 高风险控制面中的职责边界，确认 replay 需要真实 step-up、正式审计留痕与对象存储回查，不允许只返回占位 DTO。
  - `docs/原始PRD/审计、证据链与回放设计.md`：确认 replay 类型包含 `forensic_replay / state_replay / reconciliation_replay / compensation_replay`，有副作用的 replay 必须二次认证，`V1` 仅允许 dry-run。
  - `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`：确认 `POST /api/v1/audit/replay-jobs`、`GET /api/v1/audit/replay-jobs/{id}` 是正式 `V1` 接口，错误码必须包含 `AUDIT_REPLAY_DRY_RUN_ONLY`，访问模式属于 `replay`。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：确认 replay 目标围绕 order / dispute_case / evidence_package 等审计与证据聚合对象展开。
  - `docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`：确认 replay 控制面必须落到正式 router、OpenAPI、runbook 与 test-case，而不是只写 README/草稿。
  - `docs/开发任务/问题修复任务/A06-Audit-Kit-统一模型漂移.md`：确认 `ReplayJob / ReplayResult` 必须复用统一 authority model，不能再发明第二套 ad-hoc 结构。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`、`infra/docker/docker-compose.local.yml`：确认本批不新增 topic / route authority；replay 为 `platform-core` 内部高风险控制面，不通过旁路 worker 定义业务主状态。
  - `apps/platform-core/src/modules/audit/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/04-runbooks/**`、`docs/05-test-cases/**`、`scripts/check-openapi-schema.sh`：确认现有实现只覆盖 `AUD-003/004` 查询+导出控制面，replay 契约、runbook 与验收矩阵仍未正式落盘。
- 实现要点：
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `POST /api/v1/audit/replay-jobs` 与 `GET /api/v1/audit/replay-jobs/{id}`，要求 `x-request-id`；创建 replay 时要求 `x-user-id`、平台级 `audit.replay.execute` 权限与 `x-step-up-token / x-step-up-challenge-id` 至少其一，读取 replay 时要求 `audit.replay.read` 权限。
  - `POST /api/v1/audit/replay-jobs`：统一规范化 `replay_type / ref_type / reason`；`dry_run` 缺省为 `true`，显式传 `false` 会返回 `409 + AUDIT_REPLAY_DRY_RUN_ONLY`；`x-step-up-challenge-id` 必须绑定当前 actor、状态 `verified`、未过期，并与 `audit.replay.execute + ref_type/ref_id` 目标一致。
  - replay 目标当前真实支持 `order / case(dispute_case) / evidence_package`，以及存在正式 `audit.audit_event` 记录的其他 UUID 型审计对象；实现会读取目标快照、审计时间线、evidence manifests/items 与 legal hold 摘要，生成 dry-run replay report。
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：新增 `insert_replay_job`、`insert_replay_result`、`load_replay_job_detail` 等正式 writer / reader，把 `audit.replay_job + audit.replay_result` 结构化落库。
  - replay report 会通过既有 MinIO writer 落到 `s3://evidence-packages/replays/{ref_type}/{ref_id}/replay-{job_id}.json`，并通过统一 evidence writer 以 `audit_replay_report` 形式桥接到 `audit.evidence_item / evidence_manifest`；同事务追加 `audit.audit_event(audit.replay.requested / audit.replay.completed)`、`audit.access_audit(access_mode='replay')`、`ops.system_log`。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：扩展 replay 路由级权限 / step-up / dry-run 约束测试，并将 `AUD_DB_SMOKE=1` 的 live smoke 扩展为真实 replay create + lookup + DB + MinIO 回查。
  - `packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`、`packages/openapi/README.md`、`docs/02-openapi/README.md`、`scripts/check-openapi-schema.sh`：补齐 replay 路径、schema、错误码 token 与归档同步校验。
  - `docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`、`docs/04-runbooks/audit-replay.md`、`docs/04-runbooks/README.md`：补齐 replay dry-run 的正式验收矩阵、手工 SQL 种数与 `curl` 示例、DB/MinIO 回查、故障处理与清理约束。
- 验证步骤：
  1. `cargo fmt --all`
  2. `./scripts/check-openapi-schema.sh`
  3. `cargo check -p platform-core`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture`
  8. 真实 HTTP 联调：
     - `set -a; source infra/docker/.env.local; set +a; APP_PORT=18080 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core-bin`
     - `curl -X POST http://127.0.0.1:18080/api/v1/audit/replay-jobs ...`
     - `curl http://127.0.0.1:18080/api/v1/audit/replay-jobs/{id} ...`
     - `psql` 回查 `audit.replay_job / audit.replay_result / audit.audit_event / audit.access_audit / ops.system_log`
     - 使用 `minio/mc` 容器镜像读取 replay report 对象
- 验证结果：
  - `cargo fmt --all` 通过。
  - `./scripts/check-openapi-schema.sh` 通过，确认 replay 路径、`AUDIT_REPLAY_DRY_RUN_ONLY` token 与 `docs/02-openapi/audit.yaml` 归档同步一致。
  - `cargo check -p platform-core` 通过。
  - `cargo test -p platform-core` 通过：`272 passed; 0 failed`，包含 replay 路由权限 / step-up / dry-run 约束单测。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `AUD_DB_SMOKE=1 ... audit_trace_api_db_smoke` 通过：真实完成 replay create + lookup、`audit.replay_job / audit.replay_result / audit.audit_event / audit.access_audit / ops.system_log` 回查，以及 MinIO replay report 读取验证。
  - 真实 HTTP 联调通过：
    - `GET /health/live`、`GET /health/ready` 返回 `200`。
    - 手工 `POST /api/v1/audit/replay-jobs` 返回 `200`，得到 `replay_job_id=09f87d7b-604b-492f-8fb9-640852696505`、`replay_status=completed`、`dry_run=true`、4 条 replay results。
    - 手工 `GET /api/v1/audit/replay-jobs/{id}` 返回 `200`，读取结果与创建响应一致。
    - `psql` 回查：
      - `audit.replay_job`：`replay_type=state_replay`、`ref_type=order`、`dry_run=true`、`status=completed`、`request_reason='manual replay verification'`
      - `audit.replay_result`：存在 `target_snapshot / audit_timeline / evidence_projection / execution_policy` 四步；其中 `execution_policy.result_code='AUDIT_REPLAY_DRY_RUN_ONLY'`
      - `audit.audit_event`：存在 `audit.replay.requested`、`audit.replay.completed`
      - `audit.access_audit`：创建与读取两次记录均为 `access_mode='replay'`
      - `ops.system_log`：存在 `audit replay job executed: POST /api/v1/audit/replay-jobs` 与 `audit replay lookup executed: GET /api/v1/audit/replay-jobs/{id}`
    - MinIO replay report 已真实存在并可读取，内容包含：
      - `replay_job_id=09f87d7b-604b-492f-8fb9-640852696505`
      - `dry_run=true`
      - `recommendation=collect_evidence_before_replay`
      - `results[*].step_name`
      - `target.order_id=181ab37d-fb63-45a1-a4bc-da9cc7b180e0`
  - 运行态校正证明：第一次手工启动直接 `source infra/docker/.env.local` 时，应用拿到容器内 Kafka 地址 `kafka:9092` 导致启动失败；已按 `docs/04-runbooks/local-startup.md` 的宿主机正式入口改为显式覆盖 `KAFKA_BROKERS / KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094` 后复验通过，不构成冻结文档冲突。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-005`
  - `审计、证据链与回放设计.md`：replay 类型、dry-run 边界、高风险动作 step-up
  - `审计、证据链与回放接口协议正式版.md`：`/api/v1/audit/replay-jobs`、`/api/v1/audit/replay-jobs/{id}`、`AUDIT_REPLAY_DRY_RUN_ONLY`
  - `全量领域模型与对象关系说明.md`：order / dispute_case / evidence_package 等审计聚合边界
  - `A04`、`A06`：replay 控制面契约落地与统一 authority model 收口
  - `docs/04-runbooks/local-startup.md`：宿主机启动 `platform-core` 必须显式使用 `127.0.0.1:9094`
- 覆盖的任务清单条目：`AUD-005`
- 未覆盖项：
  - `AUD-006` legal hold 控制面
  - anchor / Fabric request / callback / reconcile
  - dead letter reprocess 与一致性修复高风险接口
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 收敛为“`AUD-003~005` 已交付查询 + 导出 + replay，后续仅跟踪 legal hold / anchor / ops 控制面缺口”，并把 `TODO-AUD-TEST-001` 收敛为继续追加后续 `AUD`/Fabric/一致性验收矩阵。
- 备注：
  - 本批没有发现需要人工确认的 `CSV / Markdown / technical_reference` 冲突，不触发暂停条件。
  - 手工联调结束后，已清理可删除的业务测试对象：`trade.order_main`、`catalog.product_sku`、`catalog.product`、`catalog.asset_version`、`catalog.data_asset` 与卖方组织。
  - 与高风险动作强绑定的 `iam.step_up_challenge`、其关联用户和买方组织在本地尝试删除时，会触发 `audit.audit_event` 的 FK `SET NULL -> UPDATE`，被 append-only trigger 正常拒绝；因此这类由正式审计对象引用的支持记录按运行态现状保留，不绕过审计域约束做强删。
### BATCH-221（计划中）
- 任务：AUD-008 Outbox / Dead Letter / Idempotency / External Fact / Projection Gap 查询基座
- 状态：计划中
- 说明：基于 `AUD-003 ~ AUD-007` 已落地的审计控制面，当前批次开始进入 `ops / consistency` 正式基础设施查询层。已重新按 `CSV > Markdown > technical_reference > 其他文档` 复核 `AUD-008`，并根据冻结 schema/接口确认：任务文案中的 `external_receipt` 应收敛为正式 `ops.external_fact_receipt`；`reconcile_job` 不引入独立正式表，而由 `ops.chain_projection_gap` 承接持久化查询对象，`POST /api/v1/ops/consistency/reconcile` 控制面动作保留给 `AUD-012`。本批将先修正文案漂移，再实现 `ops.outbox_event / ops.dead_letter_event / ops.consumer_idempotency_record / ops.external_fact_receipt / ops.chain_projection_gap` 的仓储与查询接口，并补齐 `GET /api/v1/ops/outbox`、`GET /api/v1/ops/dead-letters` 的最小正式控制面、权限、审计与最小测试，确保 SEARCHREC 后续能真实联查 dead letter 与 consumer 幂等记录。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-008`，完成后再进入 `AUD-009`。
### BATCH-221（待审批）
- 任务：`AUD-008` Outbox / Dead Letter / Idempotency / External Fact / Projection Gap 查询基座
- 状态：待审批
- 当前任务编号：`AUD-008`
- 前置依赖核对结果：`CORE-007`、`CORE-008`、`DB-008`、`ENV-022` 已满足；`AUD-003 ~ AUD-007` 已本地提交完成，审计 query / export / replay / legal hold / anchor batch 基线可供当前批次复用。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-008` DoD 为正式仓储、查询接口、权限、审计、错误码、最小测试齐备，并明确 SEARCHREC consumer 的 dead letter / idempotency 联查是当前批次验收重点。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `PostgreSQL / Kafka / Redis / Keycloak / MinIO / OpenSearch / 观测栈 / Fabric` 的职责边界，确认 `ops` 查询控制面不能代替主状态权威。
  - `docs/原始PRD/审计、证据链与回放设计.md`、`docs/原始PRD/双层权威模型与链上链下一致性设计.md`、`docs/原始PRD/链上链下技术架构与能力边界稿.md`、`docs/原始PRD/日志、可观测性与告警设计.md`：确认 outbox / dead letter / external fact / projection gap 属于双层权威与一致性治理正式对象，观测域不能替代审计域。
  - `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`、`docs/数据库设计/接口协议/一致性与事件接口协议正式版.md`、`docs/数据库设计/接口协议/交易链监控与公平性接口协议正式版.md`：确认 `GET /api/v1/ops/outbox`、`GET /api/v1/ops/dead-letters` 当前应先落地，`ops.external_fact_receipt` 与 `ops.chain_projection_gap` 先作为仓储查询对象，`reconcile` 动作留给 `AUD-012`，`projection-gaps` 公共控制面留给后续任务。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`、`infra/docker/docker-compose.local.yml`：确认 canonical outbox、formal topic、route policy 与本地基础设施边界。
  - `docs/开发任务/问题修复任务/A02-统一事件-Envelope-与路由权威源.md`、`A03-统一事务模板-落地真实审计与Outbox-Writer.md`、`A04-AUD-Ops-接口与契约落地缺口.md`、`A05-Outbox-Publisher-DLQ-统一闭环缺口.md`、`A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md`：确认 `AUD-008` 当前必须先落仓储与查询基座，支撑后续 SEARCHREC consumer 可靠性与 `AUD-010` reprocess。
  - `apps/platform-core/src/modules/audit/**`、`apps/platform-core/src/modules/integration/**`、`apps/platform-core/src/modules/order/**`、`apps/platform-core/src/modules/delivery/**`、`apps/platform-core/src/modules/billing/**`、`apps/platform-core/src/modules/catalog/**`、`services/fabric-adapter/**`、`services/fabric-event-listener/**`、`services/fabric-ca-admin/**`、`workers/outbox-publisher/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/04-runbooks/**`、`docs/05-test-cases/**`、`infra/**`、`scripts/**`：确认现有 `ops / consistency` 控制面仍未正式落地，已有 worker/README/OpenAPI 片段只能参考。
- 实现摘要：
  - 已按人工确认修正文案漂移：`AUD-008` 任务清单、问题修复文档与一致性协议中，`external_receipt` 收敛为正式 `ops.external_fact_receipt`，`reconcile_job` 收敛为“`ops.chain_projection_gap` 持久化查询对象 + `AUD-012` reconcile 控制面动作”，不再引入独立正式表。
  - `apps/platform-core/src/modules/audit/domain/mod.rs`、`dto/mod.rs`：新增 `OpsOutboxQuery / OpsDeadLetterQuery / ConsumerIdempotencyQuery / ExternalFactReceiptQuery / ChainProjectionGapQuery` 与对应分页/视图 DTO，统一承接 `AUD-008` 查询对象。
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：新增 `ops.outbox_event`、`ops.dead_letter_event`、`ops.consumer_idempotency_record`、`ops.external_fact_receipt`、`ops.chain_projection_gap` 的正式记录模型、分页查询与单对象读取；`dead_letter` 查询会把 `event_id -> consumer_idempotency_record` 关系直接挂到返回结果，供 SEARCHREC 失败隔离联查。
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `GET /api/v1/ops/outbox`、`GET /api/v1/ops/dead-letters`，补齐 `OpsOutboxRead / OpsDeadLetterRead` 权限、`x-request-id` 强约束、过滤参数标准化，以及 `audit.access_audit + ops.system_log` 的正式留痕。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：新增 route-level 权限 / `x-request-id` 测试，并扩展 live smoke，真实插入 `ops.outbox_event + ops.outbox_publish_attempt + ops.dead_letter_event + ops.consumer_idempotency_record + ops.external_fact_receipt + ops.chain_projection_gap`，验证 API 返回、仓储联查、审计与系统日志回查以及业务测试数据清理。
  - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、`scripts/check-openapi-schema.sh`：补齐 `AUD-008` 两条正式 `ops` 路径、分页 schema、`consumer_idempotency_records` 契约与 archive 同步校验。
  - `docs/04-runbooks/audit-ops-outbox-dead-letters.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`：补齐宿主机启动口径、手工 `curl`、SQL 回查、SEARCHREC 失败隔离说明与 `AUD-008` 验收矩阵。
- 真实验证：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `cargo test -p platform-core rejects_ops_outbox_without_permission -- --nocapture`
  5. `cargo test -p platform-core ops_dead_letters_requires_request_id -- --nocapture`
  6. `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_trace_api_db_smoke -- --nocapture`
  7. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  8. `./scripts/check-query-compile.sh`
  9. `./scripts/check-openapi-schema.sh`
  10. 宿主机真实 `curl` 联调：
     - 使用 `KAFKA_BROKERS=127.0.0.1:9094 APP_PORT=18080 cargo run -p platform-core-bin` 启动服务，避免宿主机误用 compose 内部地址 `kafka:9092`
     - 通过 `psql` 插入最小 `ops.outbox_event / ops.outbox_publish_attempt / ops.dead_letter_event / ops.consumer_idempotency_record` 样本
     - `curl GET /api/v1/ops/outbox?target_topic=dtp.search.sync&request_id=...` 返回 `total=1`、`target_topic=dtp.search.sync`、`latest_publish_attempt.result_code=failed`
     - `curl GET /api/v1/ops/dead-letters?trace_id=...&failure_stage=consumer_handler` 返回 `total=1`、`consumer_idempotency_records[0].consumer_name=search-indexer`、`result_code=dead_lettered`
     - 数据库回查 `audit.access_audit=2`、`ops.system_log=2`
     - 已清理临时业务测试数据：`ops.consumer_idempotency_record`、`ops.dead_letter_event`、`ops.outbox_publish_attempt`、`ops.outbox_event`；`audit.access_audit` 与 `ops.system_log` 保留
- 覆盖的冻结文档条目：`AUD-008`
- 未覆盖项：
  - outbox publisher 真发布链路、Kafka publish / callback / dual-DLQ 闭环，留待 `AUD-009` 与 `AUD-010`
  - 一致性联查 / 修复控制面、`projection-gaps` 公共接口与 Fabric request/callback，留待 `AUD-011+`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，将 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛到包含 `AUD-008` 的最新状态。
- 备注：
  - 本批未发现新的 `CSV / Markdown / technical_reference / schema / runbook / 代码` 冲突，不触发暂停条件。
  - 宿主机手工联调时确认：`platform-core-bin` 若不显式设置 `KAFKA_BROKERS=127.0.0.1:9094` 会回落到 compose 内部 broker 地址，当前已在 `AUD-008` 新增 runbook 中固定宿主机启动口径。
### BATCH-222（计划中）
- 任务：`AUD-009` 实现 outbox publisher worker，从数据库读取待发布事件并推送到 Kafka
- 状态：计划中
- 说明：在 `AUD-008` 已完成 canonical outbox / dead letter / idempotency / external fact / projection gap 查询基座后，当前批次开始补齐真正的 `outbox -> publisher -> Kafka` 动态闭环。实现将创建正式 `workers/outbox-publisher` Rust worker，按统一 event envelope 与 `ops.event_route_policy` / canonical topic 口径轮询 `ops.outbox_event`、记录 `ops.outbox_publish_attempt`、处理重试与双层 DLQ，并补齐最小健康检查、Prometheus 指标、runbook 与宿主机/compose 运行入口。同时会清理 Billing 把 `ops.outbox_event` 当私有工作队列的默认路径：`/api/v1/billing/{order_id}/bridge-events/process` 不再扫描 `status='pending'` 作为主工作队列，而只保留显式手工桥接 / 回放语义，避免继续把 canonical outbox 状态机污染成业务消费队列。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-009`，完成后再进入 `AUD-010`。
### BATCH-222（待审批）
- 任务：`AUD-009` outbox publisher worker
- 状态：待审批
- 当前任务编号：`AUD-009`
- 前置依赖核对结果：`CORE-007`、`CORE-008`、`DB-008`、`ENV-022` 已满足；`AUD-008` 已本地提交完成，`ops.outbox_event / ops.dead_letter_event / ops.consumer_idempotency_record / ops.external_fact_receipt / ops.chain_projection_gap` 查询基座可供当前批次复用。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-009` DoD 是真实 outbox publisher 闭环，不允许继续把通知/Fabric 保留为 `dtp.outbox.domain-events` 的并行入口。
  - `docs/开发任务/问题修复任务/A01-Kafka-Topic-口径统一.md`、`A02-统一事件-Envelope-与路由权威源.md`、`A04-AUD-Ops-接口与契约落地缺口.md`、`A05-Outbox-Publisher-DLQ-统一闭环缺口.md`：确认 worker 必须只按 canonical topic / event envelope 发布、写 `ops.outbox_publish_attempt`、支持双层死信，并清理模块绕过 publisher 直接消费 `ops.outbox_event` 的旧路径。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `PostgreSQL` 是主权威、`Kafka` 是总线、`Redis` 是辅助状态、worker 是外围进程不能反向定义主业务状态。
  - `docs/原始PRD/双层权威模型与链上链下一致性设计.md`、`docs/数据库设计/接口协议/一致性与事件接口协议正式版.md`、`docs/04-runbooks/kafka-topics.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：确认正式 topic、route-policy seed 与 `dtp.dead-letter` / `dtp.outbox.domain-events` 的冻结语义。
  - `apps/platform-core/src/shared/outbox.rs`、`apps/platform-core/src/modules/billing/repo/billing_bridge_repository.rs`、`apps/platform-core/src/modules/billing/tests/bil024_billing_trigger_bridge_db.rs`、`workers/outbox-publisher/**`、`workers/README.md`、`infra/docker/docker-compose.apps.local.example.yml`、`infra/docker/monitoring/prometheus.yml`、`infra/docker/monitoring/alert-rules.yml`：确认已有代码里 `workers/outbox-publisher` 仍为空，Billing bridge 仍错误把 canonical outbox 当私有工作队列，需在本批一起纠正。
- 实现摘要：
  - `Cargo.toml`、`Cargo.lock`、`workers/outbox-publisher/Cargo.toml`、`workers/outbox-publisher/src/main.rs`：新增正式 `outbox-publisher` Rust worker，轮询 `ops.outbox_event(status='pending')`、以 `FOR UPDATE SKIP LOCKED` claim 行、向 Kafka 发布统一 envelope，并为每次尝试写 `ops.outbox_publish_attempt`。
  - worker 成功路径会把 outbox 行更新为 `published`，失败路径按指数退避回写 `retry_count / available_at / last_error_*`；重试耗尽后把事件标记为 `dead_lettered`，同步写入 `ops.dead_letter_event(failure_stage='outbox.publish')` 并发布 Kafka `dtp.dead-letter` 隔离消息。
  - worker 追加 `audit.audit_event(outbox.publisher.publish / outbox.publisher.dead_lettered)` 与 `ops.system_log(service_name='outbox-publisher')`，并暴露 `/health/live`、`/health/ready`、`/metrics`；Prometheus 指标包含 `outbox_publisher_publish_attempts_total`、`outbox_publisher_pending_events`、`outbox_publisher_cycle_claimed_events`。
  - `apps/platform-core/src/modules/billing/repo/billing_bridge_repository.rs`、`apps/platform-core/src/modules/billing/tests/bil024_billing_trigger_bridge_db.rs`：清理 Billing 直接消费 `status='pending'` outbox 的旧路径，改为只处理已由 publisher 发布完成的 `billing.trigger.bridge`，保留显式人工桥接 / 回放入口，但不再把 canonical outbox 当默认工作队列。
  - `docs/04-runbooks/outbox-publisher.md`、`docs/04-runbooks/README.md`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/observability-local.md`、`docs/04-runbooks/port-matrix.md`、`workers/README.md`、`infra/docker/.env.local`、`infra/docker/docker-compose.apps.local.example.yml`、`infra/docker/monitoring/prometheus.yml`、`infra/docker/monitoring/alert-rules.yml`：补齐宿主机/compose 启动口径、端口、Prometheus scrape、告警规则与回查步骤。
  - `docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`、`packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`：补齐 `AUD-009` 验收矩阵，并把 Billing bridge 描述收敛为“只处理已发布的 bridge events”。
- 真实验证：
  1. `cargo fmt --all`
  2. `cargo check -p outbox-publisher`
  3. `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo test -p outbox-publisher`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil024_billing_trigger_bridge_db_smoke -- --nocapture`
  5. `cargo check -p platform-core`
  6. `cargo test -p platform-core`
  7. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  8. `./scripts/check-query-compile.sh`
  9. `./scripts/check-openapi-schema.sh`
  10. `./scripts/check-topic-topology.sh`
  11. 宿主机真实 worker 联调：
      - `APP_PORT=8098 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p outbox-publisher`
      - `curl http://127.0.0.1:8098/health/ready`
      - 手工向 `ops.outbox_event` 插入一条 `dtp.outbox.domain-events` 事件与一条 `dtp.missing.topic` 事件
      - `psql` 回查 `ops.outbox_event / ops.outbox_publish_attempt / ops.dead_letter_event / audit.audit_event / ops.system_log`
      - `kafka-console-consumer` 回查 `dtp.outbox.domain-events` 与 `dtp.dead-letter`
      - `curl http://127.0.0.1:8098/metrics` 与 `curl http://127.0.0.1:9090/api/v1/query?...` 回查 worker 指标
- 验证结果：
  - `cargo fmt --all`、`cargo check -p outbox-publisher`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh`、`./scripts/check-openapi-schema.sh`、`./scripts/check-topic-topology.sh` 全部通过。
  - `AUD_DB_SMOKE=1 cargo test -p outbox-publisher` 通过：真实覆盖 `published` 与 `dead_lettered` 两条正式路径，验证 `ops.outbox_publish_attempt`、`ops.dead_letter_event`、Kafka `dtp.outbox.domain-events / dtp.dead-letter`、`audit.audit_event` 与 `ops.system_log`。
  - `TRADE_DB_SMOKE=1 cargo test -p platform-core bil024_billing_trigger_bridge_db_smoke -- --nocapture` 通过：证明 Billing bridge 只消费 `status='published'` 的正式 bridge events，不再把 `pending` outbox 当主工作队列。
  - 宿主机真实 worker 联调通过：
    - `/health/ready` 返回 `{\"status\":\"ready\",\"worker_id\":\"outbox-publisher\"}`。
    - `curl http://127.0.0.1:8098/metrics` 可见 `outbox_publisher_publish_attempts_total{result=\"published\"}`、`{result=\"dead_lettered\"}` 与 `outbox_publisher_pending_events=0`。
    - Prometheus 在重载后可查询到 `up{job=\"outbox-publisher\"}=1` 与 `outbox_publisher_publish_attempts_total` 指标。
    - 手工插入 `request_id=req-aud009-manual-ok-1776827697141` 后，`ops.outbox_event.status=published`、`ops.outbox_publish_attempt.result_code=published`，Kafka `dtp.outbox.domain-events` 收到统一 envelope。
    - 手工插入 `request_id=req-aud009-manual-fail-1776827697141` 后，`ops.outbox_event.status=dead_lettered`、`ops.dead_letter_event.failure_stage=outbox.publish`、`ops.outbox_publish_attempt.result_code=dead_lettered`，Kafka `dtp.dead-letter` 收到隔离消息。
    - `audit.audit_event` 回查到 `outbox.publisher.publish` 与 `outbox.publisher.dead_lettered`；`ops.system_log` 回查到 `service_name='outbox-publisher'` 的 `published / dead-lettered` 记录，并与同一组 `request_id / trace_id` 对齐。
  - 手工联调结束后，已清理临时业务测试数据：`ops.outbox_publish_attempt`、`ops.dead_letter_event`、`ops.outbox_event`；`audit.audit_event` 与 `ops.system_log` 作为 append-only 留痕保留。
- 覆盖的冻结文档条目：`AUD-009`
- 未覆盖项：
  - dead letter reprocess 正式控制面，留待 `AUD-010`
  - consistency/reconcile 控制面与 projection-gap 公共接口，留待 `AUD-011~AUD-012`
  - Go/Fabric request / callback / CA / listener 正式链路，留待 `AUD-013+`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛到包含 `AUD-009` worker、runbook 与验收矩阵的最新状态。
- 备注：
  - 本批未发现新的 `CSV / Markdown / technical_reference / route-policy / topics / runbook / 代码` 冲突，不触发暂停条件。
  - `datab-prometheus` 在配置变更后需要重启/重载才能抓到新增 `outbox-publisher` job；当前已完成重载并验证 `up{job="outbox-publisher"}=1`。
### BATCH-223（计划中）
- 任务：`AUD-010` dead letter 重处理接口
- 状态：计划中
- 说明：在 `AUD-009` 已补齐 outbox publisher 与双层 DLQ 基座后，当前批次开始落地 `POST /api/v1/ops/dead-letters/{id}/reprocess` 正式控制面。按 `CSV > Markdown > technical_reference > 其他文档` 重新核对后，当前冻结目标聚焦 SEARCHREC consumer 失败事件：接口必须真实要求权限与 `step-up`，默认 `dry_run`，并能对 `search-indexer / recommendation-aggregator` 的 `ops.dead_letter_event + ops.consumer_idempotency_record + Kafka DLQ` 给出正式重处理预演、审计留痕和 runbook / 测试矩阵。Go/Fabric 仍按任务顺序留在 `AUD-013+`，本批不越级实现。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-010`，完成后再进入 `AUD-011`。
### BATCH-223（待审批）
- 任务：`AUD-010` dead letter 重处理接口
- 状态：待审批
- 当前任务编号：`AUD-010`
- 前置依赖核对结果：`AUD-008` 已交付 `ops.outbox_event / ops.dead_letter_event / ops.consumer_idempotency_record` 仓储与查询接口，`AUD-009` 已交付 `outbox-publisher` 正式 worker 与双层 DLQ 基座；本地 `PostgreSQL / Kafka / Redis / Keycloak / Prometheus / Alertmanager / Loki / Tempo / Grafana / OpenSearch` 运行基线满足当前批次依赖。按任务顺序，Go/Fabric 相关能力仍留待 `AUD-013+`，本批不越级实现。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-010` DoD 为 `POST /api/v1/ops/dead-letters/{id}/reprocess`、DTO、权限、`step-up`、审计与测试齐备，并要求覆盖 `SEARCHREC` dead letter 重处理路径。
  - `docs/开发任务/问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md`：确认当前批次只先落控制面 dry-run 预演，不提前伪造 worker 侧正式 replay 成功闭环；正式 worker 可靠性仍由后续 `SEARCHREC` 任务补齐。
  - `docs/数据库设计/接口协议/一致性与事件接口协议正式版.md`、`docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`、`docs/开发任务/问题修复任务/A05-Outbox-Publisher-DLQ-统一闭环缺口.md`：确认 dead letter 重处理属于高风险 `ops` 控制面，必须真实要求权限、`step-up`、正式审计与 runbook / OpenAPI 落盘。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `AUD` 阶段边界，确认 `Kafka` 是事件总线、`PostgreSQL` 是 dead letter / 审计主权威、`Redis` 是辅助状态、`Keycloak/IAM` 必须真实参与高风险动作身份与 `step-up` 校验。
  - `docs/原始PRD/审计、证据链与回放设计.md`、`docs/原始PRD/日志、可观测性与告警设计.md`：确认 dead letter 重处理属于高风险审计动作，观测域不能替代审计域。
  - `docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：复核当前批次不新增 topic / route authority，`dtp.search.sync` / `dtp.recommend.behavior` 仍是正式 SEARCHREC 消费入口。
  - `docs/04-runbooks/audit-ops-outbox-dead-letters.md`、`apps/platform-core/src/modules/audit/**`、`workers/search-indexer/src/main.rs`、`workers/recommendation-aggregator/src/main.rs`：确认当前 worker 侧仍存在 offset / replay 可靠性后续义务，因此 `AUD-010` 当前正确口径是“高风险重处理预演 + 正式留痕”，而不是提前伪造完整 worker 重放成功态。
- 实现要点：
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `POST /api/v1/ops/dead-letters/{id}/reprocess` 正式接口，要求 `x-request-id`、平台级 `ops.dead_letter.reprocess` 权限、`x-user-id`、`x-step-up-token / x-step-up-challenge-id` 至少其一；对 `dry_run=false` 明确返回 `409 AUDIT_DEAD_LETTER_REPROCESS_DRY_RUN_ONLY`。
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：新增单条 `ops.dead_letter_event` 装载接口，连带回读 `ops.consumer_idempotency_record`，用于控制面判定 SEARCHREC consumer lineage。
  - `apps/platform-core/src/modules/audit/domain/mod.rs`：新增 `OpsDeadLetterReprocessRequest / OpsDeadLetterReprocessView`，明确返回 dead letter 正文、consumer 名称、consumer group、target topic 与 replay preview plan。
  - SEARCHREC 口径收口：
    - 仅允许 `failure_stage='consumer_handler'`
    - 仅允许 `target_topic='dtp.search.sync'` 或 `target_topic='dtp.recommend.behavior'`
    - 仅允许 `reprocess_status='not_reprocessed'`
    - 返回 `search-indexer / cg-search-indexer` 或 `recommendation-aggregator / cg-recommendation-aggregator` 的正式预演计划
  - 审计与访问留痕：
    - `audit.audit_event(action_name='ops.dead_letter.reprocess.dry_run', result_code='dry_run_completed')`
    - `audit.access_audit(access_mode='reprocess', target_type='dead_letter_event')`
    - `ops.system_log(message_text='ops dead letter reprocess prepared: POST /api/v1/ops/dead-letters/{id}/reprocess')`
  - 契约与文档：
    - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml` 新增路径与 schema
    - `docs/04-runbooks/audit-dead-letter-reprocess.md` 新增运行手册
    - `docs/05-test-cases/audit-consistency-cases.md` 新增 `AUD-CASE-017`
    - `docs/04-runbooks/README.md`、`docs/05-test-cases/README.md` 同步索引更新
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core dead_letter_reprocess -- --nocapture`
  4. `cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture`
  5. `cargo test -p platform-core`
  6. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
  8. `./scripts/check-openapi-schema.sh`
  9. `./scripts/check-topic-topology.sh`
  10. 真实运行态联调：`cargo run -p platform-core-bin` + `curl POST /api/v1/ops/dead-letters/{id}/reprocess` + `psql` 回查
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - `cargo test -p platform-core dead_letter_reprocess -- --nocapture` 通过，覆盖权限拒绝、缺失 `step-up`、`dry_run=false` 拒绝与 live smoke。
  - `cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture` 通过，真实覆盖 `dtp.search.sync` 与 `dtp.recommend.behavior` 两条 SEARCHREC dead letter dry-run 预演。
  - `cargo test -p platform-core` 通过：`283 passed; 0 failed`，确认本批改动未回归既有业务主链。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存；随后 `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过。
  - `./scripts/check-topic-topology.sh` 通过，确认通知 / Fabric / canonical 路由权威未被本批改坏。
  - 真实运行态联调通过：
    - `curl http://127.0.0.1:18080/healthz` 返回 `{"success":true,"data":"ok"}`
    - 手工插入一条 `dtp.search.sync -> search-indexer` dead letter 与 verified `iam.step_up_challenge`
    - `curl POST /api/v1/ops/dead-letters/{id}/reprocess` 返回 `200`，`data.status='dry_run_ready'`、`data.step_up_bound=true`、`consumer_names=['search-indexer']`、`consumer_groups=['cg-search-indexer']`、`replay_target_topic='dtp.search.sync'`
    - `psql` 回查确认 `ops.dead_letter_event.reprocess_status` 仍为 `not_reprocessed`、`reprocessed_at IS NULL`
    - `audit.audit_event / audit.access_audit / ops.system_log` 三层留痕全部按正式口径落盘
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-010`
  - `一致性与事件接口协议正式版.md`：dead letter 重处理控制面、错误码与高风险动作口径
  - `A04 / A05 / A15`：`ops` 控制面契约、双层 DLQ 与 SEARCHREC consumer 可靠性分层收口
  - `kafka-topics.md`、`topics.v1.json`、`072/074`：SEARCHREC topic 与 route authority 不漂移
- 覆盖的任务清单条目：`AUD-010`
- 未覆盖项：
  - `consistency/reconcile` 控制面与 projection-gap 公共接口，留待 `AUD-011 ~ AUD-012`
  - Go/Fabric request / callback / CA / listener 正式链路，留待 `AUD-013+`
  - SEARCHREC worker 侧正式 replay、offset 策略与副作用闭环，留待后续 `SEARCHREC` 可靠性任务
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛到包含 `AUD-010` 的最新状态。
- 备注：
  - 本批未发现新的 `CSV / Markdown / technical_reference / route-policy / topics / runbook / 代码` 冲突，不触发暂停条件。
  - 手工联调插入的 `ops.dead_letter_event / ops.consumer_idempotency_record` 已清理；与 append-only 审计链绑定的 `iam.step_up_challenge / core.user_account / core.organization` 继续按既有 `AUD-004 ~ AUD-009` 的运行态策略保留，不通过强删破坏审计引用关系。
### BATCH-224（计划中）
- 任务：`AUD-011` 一致性联查接口
- 状态：计划中
- 说明：在 `AUD-008 ~ AUD-010` 已补齐 `ops` 查询基座、outbox publisher 与 dead letter reprocess 控制面后，当前批次开始落地 `GET /api/v1/ops/consistency/{refType}/{refId}`。按 `CSV > Markdown > technical_reference > 其他文档` 重新核对后，本批冻结目标是提供统一一致性联查视图，返回业务状态、证明状态、外部事实状态、最近 outbox / dead letter / 审计链路，并真实要求正式权限与查询留痕；`consistency/reconcile` 动作仍留给 `AUD-012`，Go/Fabric request / callback 仍留给 `AUD-013+`，本批不越级实现。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-011`，完成后再进入 `AUD-012`。
### BATCH-224（待审批）
- 任务：`AUD-011` 一致性联查接口
- 状态：待审批
- 当前任务编号：`AUD-011`
- 前置依赖核对结果：`AUD-008` 已交付 `ops.outbox_event / ops.dead_letter_event / ops.consumer_idempotency_record / ops.external_fact_receipt / ops.chain_projection_gap` 查询基座，`AUD-009` 已交付 `outbox-publisher` 正式 worker 与双层 DLQ 基座，`AUD-010` 已交付 SEARCHREC dead letter dry-run 重处理控制面；本地 `PostgreSQL / Kafka / Redis / Keycloak / Prometheus / Alertmanager / Loki / Tempo / Grafana / OpenSearch` 运行基线满足当前批次依赖。按任务顺序，Go/Fabric request / callback / CA / listener 仍留待 `AUD-013+`，本批不越级实现。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-011` DoD 为 `GET /api/v1/ops/consistency/{refType}/{refId}`、DTO、权限、审计、错误码与最小测试齐备，并要求实现与 OpenAPI 不漂移。
  - `docs/原始PRD/双层权威模型与链上链下一致性设计.md`、`docs/数据库设计/接口协议/一致性与事件接口协议正式版.md`、`docs/领域模型/全量领域模型与对象关系说明.md`：确认正式返回面必须覆盖业务状态、Fabric/证明状态、外部事实状态，以及最近 outbox / dead letter / audit trace。
  - `docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`：确认 `AUD-011` 只是只读联查控制面，`POST /api/v1/ops/consistency/reconcile` 留给 `AUD-012`，`projection-gaps` 公共接口留给 `AUD-021`。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `PostgreSQL` 是一致性镜像和查询主权威、`Kafka` 是事件总线、`Redis` 是辅助状态、观测域不能替代审计域。
  - `docs/原始PRD/审计、证据链与回放设计.md`、`docs/原始PRD/日志、可观测性与告警设计.md`、`docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：确认当前批次不新增 topic / route authority，只消费已冻结的 dual-authority / Fabric / outbox / DLQ 只读状态。
  - `apps/platform-core/src/modules/audit/**`、`apps/platform-core/src/modules/order/**`、`apps/platform-core/src/modules/delivery/**`、`apps/platform-core/src/modules/billing/**`：确认 `056_dual_authority_consistency.sql` 已为 `order / digital_contract / delivery_record / settlement_record / payment_intent / refund_intent / payout_instruction` 七类正式对象补齐一致性镜像字段，本批按此作为支持范围。
- 实现要点：
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：补齐一致性对象仓储装载与聚合查询，支持七类正式 dual-authority 对象的主状态读取，以及最近 `ops.outbox_event`、`ops.dead_letter_event`、`ops.external_fact_receipt`、`ops.chain_projection_gap`、`chain.chain_anchor`、`audit.audit_event` 联查。
  - `apps/platform-core/src/modules/audit/domain/mod.rs`：新增 `OpsConsistencyBusinessStateView / OpsConsistencyProofStateView / OpsConsistencyExternalFactStateView / OpsConsistencyView` 正式返回 DTO。
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `GET /api/v1/ops/consistency/{refType}/{refId}`；要求 `x-request-id`、正式 `OpsConsistencyRead` 权限；对 path `refType` 做正式规范化，拒绝未冻结对象；读取后写入 `audit.access_audit(target_type='consistency_query')` 与 `ops.system_log`。
  - 支持的 `refType` 规范化：
    - `order`
    - `contract / digital_contract`
    - `delivery / delivery_record`
    - `settlement / settlement_record`
    - `payment / payment_intent`
    - `refund / refund_intent`
    - `payout / payout_instruction`
  - 契约与文档：
    - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml` 新增 `GET /api/v1/ops/consistency/{refType}/{refId}` 路径和 schema
    - `docs/04-runbooks/audit-consistency-lookup.md` 新增宿主机调用、正式 `refType`、SQL 回查和排障手册
    - `docs/05-test-cases/audit-consistency-cases.md` 新增 `AUD-CASE-018`
    - `docs/04-runbooks/README.md`、`docs/05-test-cases/README.md` 同步索引更新
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core rejects_ops_consistency_without_permission -- --nocapture`
  4. `cargo test -p platform-core rejects_ops_consistency_with_unsupported_ref_type -- --nocapture`
  5. `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_consistency_lookup_db_smoke -- --nocapture`
  6. `cargo test -p platform-core`
  7. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  8. `./scripts/check-query-compile.sh`
  9. `./scripts/check-openapi-schema.sh`
  10. `./scripts/check-topic-topology.sh`
  11. 真实运行态联调：`cargo run -p platform-core-bin` + `curl GET /api/v1/ops/consistency/order/{order_id}` + `psql` 回查
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - `cargo test -p platform-core rejects_ops_consistency_without_permission -- --nocapture` 通过。
  - `cargo test -p platform-core rejects_ops_consistency_with_unsupported_ref_type -- --nocapture` 通过。
  - `AUD_DB_SMOKE=1 ... cargo test -p platform-core audit_consistency_lookup_db_smoke -- --nocapture` 通过，真实插入 `trade.order_main + ops.outbox_event + ops.dead_letter_event + ops.consumer_idempotency_record + ops.external_fact_receipt + chain.chain_anchor + ops.chain_projection_gap`，并回查 `audit.access_audit + ops.system_log`。
  - `cargo test -p platform-core` 通过：`286 passed; 0 failed`。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存；随后 `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过。
  - `./scripts/check-topic-topology.sh` 通过，确认 `AUD-011` 未破坏 canonical topic / route authority。
  - 真实运行态联调通过：
    - `curl http://127.0.0.1:18080/healthz` 返回 `{"success":true,"data":"ok"}`
    - 手工插入一条 `order` 一致性样本，补齐 `trade.order_main` dual-authority 字段、`ops.external_fact_receipt`、`ops.chain_projection_gap`、`chain.chain_anchor`、`ops.outbox_event`、`ops.dead_letter_event`
    - `curl GET /api/v1/ops/consistency/order/{order_id}` 返回 `200`，响应中可见 `business_state / proof_state / external_fact_state / recent_outbox_events / recent_dead_letters / recent_audit_traces`
    - `psql` 回查确认 `audit.access_audit(access_mode='masked', target_type='consistency_query')` 与 `ops.system_log(message_text='ops lookup executed: GET /api/v1/ops/consistency/{refType}/{refId}')` 已落盘
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-011`
  - `双层权威模型与链上链下一致性设计.md`：一致性联查返回面
  - `一致性与事件接口协议正式版.md`：`GET /api/v1/ops/consistency/{refType}/{refId}`
  - `A04-AUD-Ops-接口与契约落地缺口.md`：只读 consistency 接口与后续 reconcile / projection-gap 边界
- 覆盖的任务清单条目：`AUD-011`
- 未覆盖项：
  - `POST /api/v1/ops/consistency/reconcile` 干预动作，留待 `AUD-012`
  - `GET/POST /api/v1/ops/projection-gaps` 公共接口，留待 `AUD-021`
  - Go/Fabric request / callback / CA / listener 正式链路，留待 `AUD-013+`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛到包含 `AUD-011` 的最新状态。
- 备注：
  - 本批未发现新的 `CSV / Markdown / technical_reference / route-policy / topics / runbook / 代码` 冲突，不触发暂停条件。
  - 手工联调产生的 `ops.outbox_event / ops.dead_letter_event / ops.consumer_idempotency_record / ops.external_fact_receipt / ops.chain_projection_gap / chain.chain_anchor / trade.order_main / catalog.*` 业务测试数据已清理。
  - 手工联调中尝试删除 `core.user_account / core.organization` 时，因 `audit.access_audit` 与 `audit.audit_event` 的 append-only trigger 阻止 `ON DELETE SET NULL` 更新而失败；这与既有 `AUD-004 ~ AUD-010` 的留痕对象策略一致，因此保留该最小主体样本，不通过强删破坏审计引用关系。
### BATCH-225（计划中）
- 任务：`AUD-012` 一致性修复 dry-run 控制面
- 状态：计划中
- 说明：在 `AUD-011` 已补齐只读一致性联查后，当前批次继续落地 `POST /api/v1/ops/consistency/reconcile`。按你刚确认的冻结口径，本批不会引入 `reconcile_job` 表，也不会把 `ops.chain_projection_gap` 宣传成其同义词；实现边界收束为 `ops.chain_projection_gap` 持久化对象上的高风险控制面动作，要求 `dry_run + step-up + 审计 + 修复建议输出`，并明确真正执行型 worker / Go Fabric 交互仍留给 `AUD-013+` 与后续 consistency / projection-gap 任务。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-012`，完成后再进入 `AUD-013`。
### BATCH-225（待审批）
- 任务：`AUD-012` 一致性修复 dry-run 控制面
- 状态：待审批
- 当前任务编号：`AUD-012`
- 前置依赖核对结果：`AUD-008` 已交付 `ops.outbox_event / ops.dead_letter_event / ops.consumer_idempotency_record / ops.external_fact_receipt / ops.chain_projection_gap` 查询基座并修正文案漂移；`AUD-009` 已交付 `outbox-publisher` 正式 worker 与双层 DLQ 基线；`AUD-010` 已交付 dead letter dry-run 重处理控制面；`AUD-011` 已交付只读一致性联查接口。按任务顺序，真正执行型 reconcile worker、Fabric request/callback、Go 适配器与 listener 仍留待 `AUD-013+`，本批不越级实现。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-012` DoD 为 `POST /api/v1/ops/consistency/reconcile`，`V1` 先支持 `dry_run + 记录修复建议`，实现不得与 OpenAPI 漂移。
  - `docs/原始PRD/双层权威模型与链上链下一致性设计.md`、`docs/原始PRD/链上链下技术架构与能力边界稿.md`、`docs/数据库设计/接口协议/一致性与事件接口协议正式版.md`、`docs/领域模型/全量领域模型与对象关系说明.md`：确认 `reconcile` 在 `V1` 是高风险控制面动作，不单列正式 `reconcile_job` 表，持久化查询对象由 `ops.chain_projection_gap` 承接，返回面要能反映双层权威状态、projection gap 与修复建议。
  - `docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`：确认 `AUD-008` 的残留命名已收敛为正式 `ops.external_fact_receipt / ops.chain_projection_gap`，`consistency/reconcile` 仍由 `AUD-012` 承接，`projection-gaps` 公共接口留给 `AUD-021`。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `PostgreSQL` 是运行态和一致性镜像主权威、`Kafka` 是总线非真相源、`Redis` 是辅助状态、`Fabric` 是证明层而非主业务状态机、观测域不能替代审计域。
  - `docs/原始PRD/审计、证据链与回放设计.md`、`docs/原始PRD/日志、可观测性与告警设计.md`、`docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：确认本批只返回 `dtp.consistency.reconcile` 目标建议，不提前生成新的 outbox 事件，也不抢跑 Go/Fabric 真实执行链路。
  - `apps/platform-core/src/modules/audit/**`、`apps/platform-core/src/modules/order/**`、`apps/platform-core/src/modules/delivery/**`、`apps/platform-core/src/modules/billing/**`、`services/fabric-adapter/**`、`services/fabric-event-listener/**`、`services/fabric-ca-admin/**`、`workers/outbox-publisher/**`：确认现有 Rust 侧已具备 `AUD-011` 一致性读取基座，Fabric 相关 Go 服务目录仍是后续任务的正式实现范围，而不是本批的完成证据。
- 实现要点：
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `POST /api/v1/ops/consistency/reconcile`，要求 `x-request-id`、正式 `OpsConsistencyReconcile` 权限、`x-user-id`、verified `x-step-up-challenge-id` 绑定当前 actor / `ref_type` / `ref_id` / `target_action=ops.consistency.reconcile`；`V1` 强制 `dry_run=true`，`dry_run=false` 返回 `409 AUDIT_CONSISTENCY_RECONCILE_DRY_RUN_ONLY`。
  - `apps/platform-core/src/modules/audit/domain/mod.rs`：新增 `OpsConsistencyReconcileRequest / OpsConsistencyRepairRecommendationView / OpsConsistencyReconcileView` 正式 DTO，返回 `subject_snapshot / projection_gap_status_breakdown / related_projection_gaps / recommendations / reconcile_target_topic`。
  - 复用 `AUD-011` 仓储查询基座，联查 `trade.order_main` dual-authority 字段、最近 `ops.outbox_event / ops.dead_letter_event / ops.external_fact_receipt / ops.chain_projection_gap / chain.chain_anchor`，生成修复建议；建议对象仍锚定正式 `ops.chain_projection_gap`，不引入 `reconcile_job`。
  - 本批显式不做的事情：
    - 不新增 `reconcile_job` 表
    - 不把 `ops.chain_projection_gap` 宣传成 `reconcile_job` 同义词
    - 不写新的 `dtp.consistency.reconcile` outbox 事件
    - 不改写 `ops.chain_projection_gap.gap_status / resolution_summary`
    - 不提前实现 Go/Fabric request / callback / CA / listener 执行链路
  - 审计与运行留痕：
    - `audit.audit_event(action_name='ops.consistency.reconcile.dry_run')`
    - `audit.access_audit(access_mode='reconcile', target_type='consistency_reconcile')`
    - `ops.system_log(message_text='ops consistency reconcile prepared: POST /api/v1/ops/consistency/reconcile')`
  - 契约与文档：
    - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml` 新增 `POST /api/v1/ops/consistency/reconcile`
    - `docs/04-runbooks/audit-consistency-reconcile.md` 新增宿主机调用、SQL 回查、排障与“无执行副作用”说明
    - `docs/05-test-cases/audit-consistency-cases.md` 新增 `AUD-CASE-019`
    - `docs/04-runbooks/README.md`、`docs/05-test-cases/README.md` 同步索引更新
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core rejects_ops_consistency_reconcile_without_permission -- --nocapture`
  4. `cargo test -p platform-core ops_consistency_reconcile_requires_step_up -- --nocapture`
  5. `cargo test -p platform-core ops_consistency_reconcile_enforces_dry_run_only -- --nocapture`
  6. `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_consistency_reconcile_db_smoke -- --nocapture`
  7. `cargo test -p platform-core`
  8. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  9. `./scripts/check-query-compile.sh`
  10. `./scripts/check-openapi-schema.sh`
  11. `./scripts/check-topic-topology.sh`
  12. 真实运行态联调：`cargo run -p platform-core-bin` + `curl POST /api/v1/ops/consistency/reconcile` + `psql` 回查
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - 三条路由专项测试 `rejects_ops_consistency_reconcile_without_permission`、`ops_consistency_reconcile_requires_step_up`、`ops_consistency_reconcile_enforces_dry_run_only` 均通过。
  - `AUD_DB_SMOKE=1 ... cargo test -p platform-core audit_consistency_reconcile_db_smoke -- --nocapture` 通过，真实插入 `trade.order_main + ops.outbox_event + ops.dead_letter_event + ops.external_fact_receipt + chain.chain_anchor + ops.chain_projection_gap + iam.step_up_challenge`，并回查 `audit.audit_event + audit.access_audit + ops.system_log + 无 reconcile outbox + projection gap 未变更`。
  - `cargo test -p platform-core` 通过：`289 passed; 0 failed`。
  - `DATABASE_URL=... cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存；随后 `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过。
  - `./scripts/check-topic-topology.sh` 通过，确认本批未破坏 canonical topic / route authority。
  - 真实运行态联调通过：
    - `cargo run -p platform-core-bin` 启动后，`curl POST /api/v1/ops/consistency/reconcile` 返回 `200`
    - 响应中 `status=dry_run_ready`、`step_up_bound=true`、`reconcile_target_topic=dtp.consistency.reconcile`、`recommendation_count=4`
    - `psql` 回查确认 `audit.audit_event|1|manual consistency reconcile preview|full`
    - `psql` 回查确认 `audit.access_audit(access_mode='reconcile', target_type='consistency_reconcile', step_up_challenge_id=<manual challenge>)`
    - `psql` 回查确认 `ops.system_log` 存在对应 `prepared` 记录
    - `psql` 回查确认 `ops.outbox_event(request_id=<manual request_id>, target_topic='dtp.consistency.reconcile') = 0`
    - `psql` 回查确认 `ops.chain_projection_gap.gap_status='open'` 且原 `resolution_summary.seed` 保持不变
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-012`
  - `双层权威模型与链上链下一致性设计.md`：`reconcile` 控制面与双层权威边界
  - `一致性与事件接口协议正式版.md`：`POST /api/v1/ops/consistency/reconcile`
  - `A04-AUD-Ops-接口与契约落地缺口.md`：`AUD-008 / AUD-012` 的正式对象命名与控制面边界
- 覆盖的任务清单条目：`AUD-012`
- 未覆盖项：
  - `GET /api/v1/ops/projection-gaps`、`POST /api/v1/ops/projection-gaps/{id}/resolve` 公共接口，留待 `AUD-021`
  - 真正执行型 `dtp.consistency.reconcile` publish / consumer / callback / reconcile worker，留待后续 `AUD` 批次
  - Go/Fabric request / callback / CA / listener / chaincode 正式链路，留待 `AUD-013+`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛到包含 `AUD-012` 的最新状态。
- 备注：
  - 本批未发现新的 `CSV / Markdown / technical_reference / route-policy / topics / runbook / 代码` 冲突，不触发暂停条件。
  - 手工联调产生的 `ops.outbox_event / ops.dead_letter_event / ops.external_fact_receipt / ops.chain_projection_gap / chain.chain_anchor / trade.order_main / catalog.*` 业务测试数据已清理。
  - 尝试继续删除手工联调使用的 `core.organization / core.user_account / iam.step_up_challenge` 时，数据库因 `audit.access_audit` append-only trigger 拒绝 `ON DELETE SET NULL` 更新；该最小主体样本因此保留，以免通过强删破坏正式审计引用关系。
### BATCH-226（计划中）
- 任务：`AUD-013` 初始化 `services/fabric-adapter/`（Go）
- 状态：计划中
- 说明：在 `AUD-012` 已完成只读/干预型 consistency 控制面后，当前批次正式进入 Go/Fabric 链路实现。按 `CSV > Markdown > technical_reference > 其他文档` 重新核对后，本批边界收束为 `fabric-adapter` 的正式基础框架：建立 Go module、Kafka consumer、canonical envelope 解析、`dtp.audit.anchor / dtp.fabric.requests` 单入口、链提交 Provider seam、以及“提交回执基础层”对 `ops.external_fact_receipt + audit.audit_event + ops.system_log` 的正式回写。`AUD-014` 再细化四类摘要消息处理，`AUD-015/016` 再补 listener / CA admin，`AUD-017` 再切 `mock / fabric-test-network` provider。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-013`，完成后再进入 `AUD-014`。
### BATCH-226（待审批）
- 任务：`AUD-013` 初始化 `services/fabric-adapter/`（Go）
- 状态：待审批
- 实现摘要：
  - 新增 `services/fabric-adapter/` Go module、`cmd/fabric-adapter` 入口、配置加载、Kafka consumer、canonical envelope 解析、mock provider、PostgreSQL 回执写回层。
  - `fabric-adapter` 正式消费入口固定为 `dtp.audit.anchor / dtp.fabric.requests`，consumer group 固定为 `cg-fabric-adapter`；不消费 `dtp.outbox.domain-events`。
  - 当前 provider 以 Go `MockProvider` 返回 deterministic `tx_hash` 与 receipt payload，正式 `fabric-test-network / Gateway / chaincode` provider 留待 `AUD-014~AUD-017`。
  - 每条消费成功后写入：
    - `ops.external_fact_receipt`
    - `audit.audit_event(action_name='fabric.adapter.submit')`
    - `ops.system_log(message_text='fabric adapter accepted submit event')`
    - 若消息携带 `chain_anchor_id`，则更新 `chain.chain_anchor.tx_hash / status / reconcile_status`
  - 新增 Go 工具链脚本：
    - `scripts/go-env.sh`
    - `scripts/fabric-adapter-bootstrap.sh`
    - `scripts/fabric-adapter-test.sh`
    - `scripts/fabric-adapter-run.sh`
    - `Makefile` 对应 target
  - 工具链缓存统一收敛到 `third_party/external-deps/go`，并修复了 `go-env.sh` 的仓库根目录解析，避免误把缓存写到 `services/**`。
  - 文档同步：
    - 新增 `docs/04-runbooks/fabric-adapter.md`
    - 更新 `docs/04-runbooks/README.md`
    - 更新 `docs/04-runbooks/fabric-local.md`
    - 更新 `docs/04-runbooks/fabric-debug.md`
    - 更新 `docs/05-test-cases/audit-consistency-cases.md`
    - 更新 `docs/05-test-cases/README.md`
- 验证步骤：
  1. `find services/fabric-adapter -name '*.go' -print0 | xargs -0 gofmt -w`
  2. `bash ./scripts/fabric-adapter-test.sh`
  3. `bash -lc 'source /home/luna/Documents/DataB/scripts/go-env.sh && cd /home/luna/Documents/DataB/services/fabric-adapter && go build ./...'`
  4. `docker exec datab-kafka /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --describe --topic dtp.audit.anchor`
  5. `docker exec datab-kafka /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --describe --topic dtp.fabric.requests`
  6. 真实运行态联调：`./scripts/fabric-adapter-run.sh` + `kcat` 向 `dtp.audit.anchor / dtp.fabric.requests` 注入 canonical JSON + `psql` 回查
  7. `docker exec datab-kafka /opt/kafka/bin/kafka-consumer-groups.sh --bootstrap-server localhost:9092 --group cg-fabric-adapter --describe`
- 验证结果：
  - Go 侧 `gofmt`、`go test ./...`、`go build ./...` 全部通过。
  - `dtp.audit.anchor`、`dtp.fabric.requests` 两个正式 topic 运行态存在，`cg-fabric-adapter` 在真实进程启动后成功创建并消费。
  - 真实运行态联调通过：
    - `./scripts/fabric-adapter-run.sh` 启动后，Go 进程打印 `fabric-adapter starting`
    - 使用 `kcat` 容器向 `dtp.audit.anchor` 注入 `audit.anchor_requested` 后，`ops.external_fact_receipt(request_id='req-aud013-anchor-kcat') = 1`
    - 使用 `kcat` 容器向 `dtp.fabric.requests` 注入 `fabric.proof_submit_requested` 后，`ops.external_fact_receipt(request_id='req-aud013-proof-kcat') = 1`
    - `audit.audit_event(request_id in (...), action_name='fabric.adapter.submit') = 2`
    - `ops.system_log(request_id in (...), message_text='fabric adapter accepted submit event') = 2`
    - `chain.chain_anchor` 对应测试行被更新为 `status='submitted'`、`reconcile_status='pending_check'`，`tx_hash` 来自 Go mock provider
    - `ops.external_fact_receipt.receipt_payload.mode = 'mock'`，`metadata.topic` 分别为 `dtp.audit.anchor / dtp.fabric.requests`
  - 清理结果：
    - `ops.external_fact_receipt`、`audit.anchor_batch`、`chain.chain_anchor` 测试业务数据已清理
    - `audit.audit_event` 与 `ops.system_log` append-only 留痕保留
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-013`
  - `技术选型正式版.md`：Go 负责 Fabric 相关服务
  - `链上链下技术架构与能力边界稿.md`：Fabric adapter / listener / CA admin 分层边界
  - `async-chain-write.md`：`dtp.audit.anchor / dtp.fabric.requests -> fabric-adapter -> fabric-event-listener`
  - `kafka-topics.md`、`074_event_topology_route_extensions.sql`：正式 topic / consumer group / route authority
- 覆盖的任务清单条目：`AUD-013`
- 未覆盖项：
  - 四类摘要消息的正式 handler 细化，留待 `AUD-014`
  - `fabric-event-listener`、`dtp.fabric.callbacks`、回执回调消费链路，留待 `AUD-015`
  - `fabric-ca-admin` 与 CA 管理边界，留待 `AUD-016`
  - `mock / fabric-test-network` provider 切换、真实 Gateway / chaincode / test-network 联调，留待 `AUD-017`
  - `ops.consumer_idempotency_record + Redis` 的 Fabric consumer 幂等闭环，留待 `AUD-026`
- 新增 TODO / 预留项：
  - 新增 `TODO(V1-gap, AUD-013)`：`services/fabric-adapter/internal/service/processor.go`
  - 已同步登记 `TODO-AUD-FABRIC-001` 到 `docs/开发任务/V1-Core-TODO与预留清单.md`
  - 同步更新 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 为包含 `AUD-013` 的最新状态
- 备注：
  - 手工排障时观察到 `kafka-console-producer.sh` 注入同一 JSON 可能造成重复消息噪音；正式 smoke 已改用 `kcat` 容器，并据此完成单条请求 ID 的唯一性回查。
  - 本批未发现新的 `CSV / Markdown / technical_reference / route-policy / topics / runbook / 代码` 冲突，不触发暂停条件。
### BATCH-227（计划中）
- 任务：`AUD-014` 在 `fabric-adapter` 中实现四类摘要消息处理占位
- 状态：计划中
- 说明：在 `AUD-013` 已完成 Go `fabric-adapter` 基础框架、Kafka consumer、canonical envelope 解析、mock provider 与 PostgreSQL 回执回写的基础上，当前批次按 `CSV > Markdown > technical_reference > 其他文档` 继续细化正式消息处理面。冻结边界收束为：保持 `dtp.audit.anchor / dtp.fabric.requests` 双 topic 单入口不变，在 Go 适配器内部显式区分 `evidence_batch_root / order_summary / authorization_summary / acceptance_summary` 四类摘要消息，补齐分类校验、handler dispatch、provider request metadata、回执/审计/日志留痕字段和真实 smoke 样例，为 `AUD-015~017` 的 callback listener、CA admin 与真实 `fabric-test-network / Gateway / chaincode` provider 铺底，不新增旁路 topic、不把 Fabric 反向做成业务主状态机。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-014`，完成后再进入 `AUD-015`。
### BATCH-227（待审批）
- 任务：`AUD-014` 在 `fabric-adapter` 中实现四类摘要消息处理占位
- 状态：待审批
- 前置依赖核对结果：`AUD-013` 已交付 Go 版 `fabric-adapter` 基础框架、`dtp.audit.anchor / dtp.fabric.requests` 正式消费入口、mock provider 与 PostgreSQL 回执回写；本地 `PostgreSQL / Kafka / Redis / Keycloak / MinIO / Prometheus / Alertmanager / Loki / Tempo / Grafana` 运行基线可用，`cg-fabric-adapter` consumer group 运行态存在，满足本批继续细化 Go/Fabric handler 的依赖。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-014` 的顺序、DoD 与“仅保留 `dtp.audit.anchor / dtp.fabric.requests` 单入口”的冻结要求。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/技术选型正式版.md`：确认 Go 负责 Fabric 外围适配，`Fabric` 是摘要证明层而不是主状态机。
  - `docs/原始PRD/链上链下技术架构与能力边界稿.md`、`docs/领域模型/全量领域模型与对象关系说明.md`：确认四类摘要对象是 `订单摘要 / 授权摘要 / 验收摘要 / 证据批次根`。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`：复核 `audit.anchor_requested -> dtp.audit.anchor -> fabric-adapter`、`fabric.proof_submit_requested -> dtp.fabric.requests -> fabric-adapter` 的正式拓扑和 consumer group。
  - `services/fabric-adapter/**`、`infra/fabric/deploy-chaincode-placeholder.sh`、`docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`：确认当前应在 Go 侧补齐 `evidence_batch_root / order_digest / authorization_digest / acceptance_digest` 的 handler 占位，而不是新增旁路 topic 或新的持久化对象名。
- 实现摘要：
  - `services/fabric-adapter/internal/service/dispatch.go`、`dispatch_test.go`：新增 Go 侧 `Dispatcher + SubmissionHandler` 分派层，把 `audit.anchor_requested` 固定解析为 `evidence_batch_root`，把 `fabric.proof_submit_requested` 按 `summary_type` 解析为 `order_summary / authorization_summary / acceptance_summary`，并在 `summary_type` 缺失或非法时直接拒绝消费。
  - `services/fabric-adapter/internal/provider/provider.go`：扩展 `SubmissionRequest`，把 `submission_kind / contract_name / transaction_name / summary_digest / anchor_batch_id / chain_anchor_id` 贯穿到 Go mock provider；`receipt_payload` 现已带正式 handler 分类元数据，供后续 `AUD-015~017` 的 Gateway / chaincode / callback 直接复用。
  - `services/fabric-adapter/internal/service/processor.go`、`processor_test.go`：`ProcessMessage` 现先做 handler dispatch，再调用 provider / persister，并把 `submission_kind / contract_name / transaction_name` 打到进程日志里。
  - `services/fabric-adapter/internal/store/postgres.go`：`ops.external_fact_receipt.metadata`、`receipt_payload`、`audit.audit_event.metadata`、`ops.system_log.structured_payload` 现都带 `submission_kind / contract_name / transaction_name / summary_type / summary_digest`；同时补强 `resolveOrderID`，允许后续通过 `chain.chain_anchor` 反查 order 关联。
  - 文档同步：
    - 重写 `docs/04-runbooks/fabric-adapter.md`，固化四类摘要 handler 映射、`psql -v ON_ERROR_STOP=1` smoke 规范、正式回查字段与排障说明。
    - 更新 `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/fabric-debug.md`、`docs/04-runbooks/README.md`。
    - 更新 `docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`，把 `AUD-CASE-020` 收口到四类摘要 handler 验收矩阵。
    - 更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛到包含 `AUD-014` 的最新状态。
- 验证步骤：
  1. `find services/fabric-adapter -name '*.go' -print0 | xargs -0 gofmt -w`
  2. `bash ./scripts/fabric-adapter-test.sh`
  3. `bash -lc 'source /home/luna/Documents/DataB/scripts/go-env.sh && cd /home/luna/Documents/DataB/services/fabric-adapter && go build ./...'`
  4. 真实运行态 smoke：保持 `./scripts/fabric-adapter-run.sh` 运行，使用 `psql -v ON_ERROR_STOP=1` 准备 `chain_anchor + anchor_batch` 最小对象，分别向 `dtp.audit.anchor` 注入 `audit.anchor_requested`、向 `dtp.fabric.requests` 注入 `order_summary / authorization_summary / acceptance_summary` 三类 `fabric.proof_submit_requested`，再用 `psql` 回查 `ops.external_fact_receipt / audit.audit_event / ops.system_log / chain.chain_anchor` 与 `docker exec datab-kafka ... kafka-consumer-groups.sh --group cg-fabric-adapter --describe`。
  5. `cargo fmt --all`
  6. `cargo check -p platform-core`
  7. `cargo test -p platform-core`
  8. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  9. `./scripts/check-query-compile.sh`
  10. `./scripts/check-topic-topology.sh`
- 验证结果：
  - Go 侧 `gofmt`、`go test ./...`、`go build ./...` 全部通过。
  - 真实运行态 smoke 通过：
    - 使用新的 `req-aud014b-*` 请求 ID 注入四类摘要消息后，`ops.external_fact_receipt / audit.audit_event / ops.system_log` 对每个 request id 均只有 `1` 条记录。
    - `ops.external_fact_receipt.metadata` 与 `receipt_payload` 中的 `submission_kind / contract_name / transaction_name` 分别命中：
      - `evidence_batch_root / evidence_batch_root / SubmitEvidenceBatchRoot`
      - `order_summary / order_digest / SubmitOrderDigest`
      - `authorization_summary / authorization_digest / SubmitAuthorizationDigest`
      - `acceptance_summary / acceptance_digest / SubmitAcceptanceDigest`
    - 四条测试 `chain.chain_anchor` 都被更新为 `status='submitted'`、`reconcile_status='pending_check'`，`tx_hash` 来自 Go mock provider。
    - `cg-fabric-adapter` consumer group 运行态存在且 lag 为 `0`。
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅有仓库既存 `unused_*` warning，无新增编译失败。
  - `cargo test -p platform-core` 通过：`290` 个测试通过，既存 `iam_party_access_flow_live` 继续保持 ignored。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-topic-topology.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-014`
  - `技术选型正式版.md`：Go 负责 Fabric 外围适配与链码交互边界
  - `链上链下技术架构与能力边界稿.md`：Fabric adapter / listener / CA admin 分层
  - `全量领域模型与对象关系说明.md`：`订单摘要 / 授权摘要 / 验收摘要 / 证据批次根`
  - `async-chain-write.md`、`kafka-topics.md`、`074_event_topology_route_extensions.sql`：`dtp.audit.anchor / dtp.fabric.requests` 单入口冻结
- 覆盖的任务清单条目：`AUD-014`
- 未覆盖项：
  - `fabric-event-listener`、`dtp.fabric.callbacks` 与回执消费链路，留待 `AUD-015`
  - `fabric-ca-admin` 与 CA 管理边界，留待 `AUD-016`
  - `mock / fabric-test-network` provider 切换、真实 Gateway / chaincode / test-network 联调，留待 `AUD-017`
  - `ops.consumer_idempotency_record + Redis` 的 Fabric consumer 幂等闭环，留待 `AUD-026`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md` 的 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001`。
- 备注：
  - 首轮手工 smoke 因 `psql` 默认未启用 `ON_ERROR_STOP`，在 SQL 种子失败后仍继续向 Kafka 注入消息，导致 append-only `audit.audit_event / ops.system_log` 出现重复 request id 留痕；该问题已定位为脚本执行方式，而非 Go handler 重复写入，随后用新的 `req-aud014b-*` 和 `psql -v ON_ERROR_STOP=1` 重新验证并取得干净结果。
  - 本批未发现新的 `CSV / Markdown / technical_reference / route-policy / topics / runbook / 代码` 冲突，不触发暂停条件。
### BATCH-228（计划中）
- 任务：`AUD-015` 初始化 `services/fabric-event-listener/`（Go）
- 状态：计划中
- 说明：在 `AUD-014` 已把 `fabric-adapter` 收口到四类摘要 handler 之后，当前批次按 `CSV > Markdown > technical_reference > 其他文档` 继续推进正式 Fabric callback 主链。冻结边界收束为：在 Go 侧初始化 `fabric-event-listener`，消费 commit status / chaincode event 回执，形成 `dtp.fabric.callbacks` 正式回执消息，并把外部事实回执回写到 PostgreSQL 权威对象中，为后续 `AUD-016 / AUD-017` 的 CA admin 与真实 `fabric-test-network / Gateway / chaincode` 交互铺底；本批不反向定义主业务状态，不新增旁路 topic，也不把 README 占位当作完成证据。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-015`，完成后再进入 `AUD-016`。
### BATCH-228（待审批）
- 任务：`AUD-015` 初始化 `services/fabric-event-listener/`（Go）
- 状态：待审批
- 说明：
  - 已按冻结口径落地 `services/fabric-event-listener/` 正式 Go 进程，当前 local 模式以“已提交 source receipt -> mock commit callback”形式跑通 callback 主链，不新增旁路 topic，也不把 `fabric-event-listener` 反向做成业务主状态机。
  - 正式输出固定为 `dtp.fabric.callbacks`，并真实回写：
    - `ops.external_fact_receipt`
    - `audit.audit_event`
    - `ops.system_log`
    - `chain.chain_anchor`
    - `audit.anchor_batch`（仅证据批次根）
  - callback 统一补齐并持久化了冻结要求的字段：
    - `provider_code`
    - `provider_request_id`
    - `callback_event_id`
    - `event_version`
    - `provider_status`
    - `provider_occurred_at`
    - `payload_hash`
- 实现摘要：
  - `services/fabric-event-listener/`：
    - 新增 Go module、`cmd/fabric-event-listener` 运行入口、配置装载、Kafka callback publisher、mock callback provider、轮询处理器与 PostgreSQL store。
    - listener 轮询 `ops.external_fact_receipt(fact_type in ('fabric_submit_receipt','fabric_anchor_submit_receipt'), receipt_status='submitted')` 且未标记 `listener_callback_event_id` 的 source receipt，生成确定性 `fabric.commit_confirmed / fabric.commit_failed` callback envelope。
    - callback envelope 正式发布到 `dtp.fabric.callbacks`，并把 `callback_event_id / provider_request_id / provider_status / payload_hash / source_receipt_id / submission_kind / chain_anchor_id` 等字段贯穿到 Kafka payload 与 PostgreSQL metadata。
    - 成功 callback 会把 `chain.chain_anchor.status='anchored'`、`reconcile_status='matched'`，并在 anchor batch 场景同步把 `audit.anchor_batch.status='anchored'`；失败 callback 会把 `chain.chain_anchor.status='failed'`、`reconcile_status='pending_check'`。
    - source receipt 会被标记 `listener_callback_event_id / listener_callback_status / listener_callback_payload_hash / listener_service_name`，避免重复轮询。
  - 运行入口与留痕：
    - 新增 `scripts/fabric-event-listener-bootstrap.sh`、`scripts/fabric-event-listener-test.sh`、`scripts/fabric-event-listener-run.sh`，统一复用 `scripts/go-env.sh` 和 `third_party/external-deps/go`。
    - 更新 `Makefile`，新增 `fabric-event-listener-bootstrap / test / run` 目标。
  - 文档与验收：
    - 新增 `docs/04-runbooks/fabric-event-listener.md`，落盘 bootstrap / test / run、mock callback smoke、Kafka / DB 回查与清理步骤。
    - 更新 `docs/04-runbooks/fabric-local.md`、`fabric-debug.md`、`README.md`，把 `AUD-015` 的 callback 主链纳入正式运行说明。
    - 更新 `docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`，新增 `AUD-CASE-021`，把 `dtp.fabric.callbacks`、callback metadata、`chain_anchor / anchor_batch` 状态变化纳入正式验收矩阵。
    - 更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛到包含 `AUD-015` 的最新状态。
- 验证步骤：
  1. `find services/fabric-event-listener -name '*.go' -print0 | xargs -0 gofmt -w`
  2. `bash ./scripts/fabric-event-listener-bootstrap.sh`
  3. `bash ./scripts/fabric-event-listener-test.sh`
  4. `bash -lc 'source /home/luna/Documents/DataB/scripts/go-env.sh && cd /home/luna/Documents/DataB/services/fabric-event-listener && go build ./...'`
  5. 真实运行态 smoke：
     - 保持 `./scripts/fabric-adapter-run.sh` 运行，由 `dtp.audit.anchor / dtp.fabric.requests` 先生成 source receipt
     - 用 `psql -v ON_ERROR_STOP=1` 准备 `chain.chain_anchor + audit.anchor_batch` 最小对象
     - 向 `dtp.audit.anchor` 注入 `audit.anchor_requested`
     - 向 `dtp.fabric.requests` 注入 `fabric.proof_submit_requested(summary_type=order_summary)`
     - 先把 order summary 的 source receipt 标记 `mock_callback_status=failed`，再启动 `./scripts/fabric-event-listener-run.sh`
     - 用 `kcat` 回查 `dtp.fabric.callbacks`
     - 用 `psql` 回查 `ops.external_fact_receipt / audit.audit_event / ops.system_log / chain.chain_anchor / audit.anchor_batch`
  6. `cargo fmt --all`
  7. `cargo check -p platform-core`
  8. `cargo test -p platform-core`
  9. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  10. `./scripts/check-query-compile.sh`
  11. `./scripts/check-topic-topology.sh`
- 验证结果：
  - Go 侧 `gofmt`、`go test ./...`、`go build ./...` 全部通过。
  - 真实运行态 smoke 通过：
    - `fabric-adapter -> source receipt -> fabric-event-listener -> dtp.fabric.callbacks -> PostgreSQL` 主链真实跑通。
    - 新样本 `req-aud015b-anchor-kcat` 产生：
      - `ops.external_fact_receipt.fact_type='fabric_anchor_commit_receipt'`
      - `receipt_status='confirmed'`
      - `metadata.callback_event_id='fabric-callback-77a69199cffaa71d'`
      - `provider_status='confirmed'`
    - 新样本 `req-aud015b-order-kcat` 产生：
      - `ops.external_fact_receipt.fact_type='fabric_commit_receipt'`
      - `receipt_status='failed'`
      - `metadata.callback_event_id='fabric-callback-35853156513da26b'`
      - `provider_status='failed'`
    - `audit.audit_event(action_name='fabric.event_listener.callback')` 对两个 request id 各落 `1` 条，结果分别为 `confirmed / failed`。
    - `ops.system_log(message_text='fabric event listener published callback')` 对两个 request id 各落 `1` 条，`structured_payload.event_type` 分别为 `fabric.commit_confirmed / fabric.commit_failed`。
    - `dtp.fabric.callbacks` 中可回查到对应 callback envelope，且 `callback_event_id / provider_request_id / payload_hash / source_receipt_id` 齐备。
    - 成功链 `chain.chain_anchor.status='anchored'`、`reconcile_status='matched'`，`audit.anchor_batch.status='anchored'`；失败链 `chain.chain_anchor.status='failed'`、`reconcile_status='pending_check'`。
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅有仓库既存 `unused_*` warning，无新增编译失败。
  - `cargo test -p platform-core` 通过：`290` 个测试通过，既存 `iam_party_access_flow_live` 继续保持 ignored。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-topic-topology.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-015`
  - `技术选型正式版.md`：Go 负责 Fabric 外围适配与事件监听边界
  - `链上链下技术架构与能力边界稿.md`：listener 负责 callback / chaincode event，不负责主业务状态
  - `全量领域模型与对象关系说明.md`：链上摘要 / 证据批次根 / 回执证明聚合
  - `事件模型与Topic清单正式版.md`、`kafka-topics.md`、`async-chain-write.md`：`dtp.fabric.callbacks` 正式回执拓扑
  - `A04-AUD-Ops-接口与契约落地缺口.md`：Fabric callback / listener 闭环义务
- 覆盖的任务清单条目：`AUD-015`
- 未覆盖项：
  - `fabric-ca-admin` 与 CA 管理边界，留待 `AUD-016`
  - `mock / fabric-test-network` provider 切换、真实 Gateway / chaincode / test-network 联调，留待 `AUD-017`
  - `platform-core.consistency` 的正式 callback 消费执行面与更完整 reconcile / projection gap 闭环，留待后续 `AUD` 批次
  - `ops.consumer_idempotency_record + Redis` 的 Fabric consumer 幂等闭环，留待 `AUD-026`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md` 的 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001`。
- 备注：
  - 首轮 listener 联调先撞到两个运行态问题，均已在当前批修正并重新验证：
    - `ops.external_fact_receipt` 允许空 `order_id / provider_key / request_id / trace_id`，初版扫描逻辑未对空值做 `COALESCE`，已修复。
    - `jsonb_build_object` 更新 source receipt metadata 时参数未显式 cast，PostgreSQL 无法推断类型，已补全 `::text`。
  - 真实 smoke 过程中还发现：若测试 `chain.chain_anchor.ref_type='order'` 但 `ref_id` 指向不存在的订单，`fabric-adapter` 会按既有正式逻辑把该 `ref_id` 解析成 `order_id`，从而触发 `ops.external_fact_receipt.order_id` 外键；该现象已按正式口径修正测试数据，不构成冻结文档冲突。
  - 为隔离本批新样本，已把历史遗留的三条 `submitted` 测试 source receipt 标记为跳过，避免污染 `AUD-015` 的 callback smoke 判定；本批新产生的业务测试数据已清理，`audit.audit_event / ops.system_log` 按 append-only 保留。
  - 本批未发现新的 `CSV / Markdown / technical_reference / route-policy / topics / runbook / 代码` 冲突，不触发暂停条件。
### BATCH-229（计划中）
- 任务：`AUD-016` 初始化 `services/fabric-ca-admin/`（Go）
- 状态：计划中
- 说明：在 `AUD-015` 已完成 `fabric-event-listener` callback 主链后，当前批次按 `CSV > Markdown > technical_reference > 其他文档` 继续推进 Fabric 证书治理执行面。冻结边界收束为：新增 Go 版 `fabric-ca-admin` 服务，承接 `Fabric 身份签发 / 吊销 / 证书吊销` 的执行逻辑；Rust `platform-core` 保留正式公网 IAM API、权限点、step-up 和审计主体，但不再直接把 Fabric 身份/证书治理写成本地 DB 占位。当前批次先以 `mock` CA provider 跑通最小正式闭环：`step-up -> platform-core IAM API -> fabric-ca-admin -> PostgreSQL(身份/证书/外部事实) -> audit.audit_event / ops.system_log`，并为 `AUD-017` 的 `mock / fabric-test-network` provider 切换与真实 Fabric CA 治理铺底；本批不发明新 topic、不反向把 Go 服务做成主状态机、不把审批流程扩展成第二套未冻结接口。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-016`，完成后再进入 `AUD-017`。
### BATCH-229（待审批）
- 任务：`AUD-016` 初始化 `services/fabric-ca-admin/`（Go）
- 状态：待审批
- 交付清单：
  - `services/fabric-ca-admin/`
    - 新增 Go module、`cmd/fabric-ca-admin` 入口、配置装载、内部 HTTP handler、mock CA provider、PostgreSQL store 与执行服务层。
    - 正式暴露内部执行接口：
      - `GET /healthz`
      - `POST /internal/fabric-identities/{id}/issue`
      - `POST /internal/fabric-identities/{id}/revoke`
      - `POST /internal/certificates/{id}/revoke`
    - 执行面真实更新：
      - `iam.fabric_identity_binding`
      - `iam.certificate_record`
      - `iam.certificate_revocation_record`
      - `ops.external_fact_receipt`
      - `ops.system_log`
  - `apps/platform-core/src/modules/iam/**`
    - `GET /api/v1/iam/fabric-identities`、`GET /api/v1/iam/certificates` 继续由 Rust 公网 IAM 控制面承接。
    - `POST /api/v1/iam/fabric-identities/{id}/issue`、`POST /api/v1/iam/fabric-identities/{id}/revoke`、`POST /api/v1/iam/certificates/{id}/revoke` 不再直接写本地 DB，占位逻辑已替换为真实调用 `fabric-ca-admin`。
    - 补齐正式权限点、`step-up`、真实操作者审计主体与错误码映射：
      - `iam.fabric_identity.read`
      - `iam.fabric_identity.issue`
      - `iam.fabric_identity.revoke`
      - `iam.certificate.read`
      - `iam.certificate.revoke`
    - `step-up` 高风险动作映射补齐：
      - `iam.fabric.identity.issue`
      - `iam.fabric.identity.revoke`
      - `iam.certificate.revoke`
  - OpenAPI / 文档 / 脚本：
    - 更新 `packages/openapi/iam.yaml`、`docs/02-openapi/iam.yaml`
    - 新增 `docs/04-runbooks/fabric-ca-admin.md`
    - 更新 `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/README.md`
    - 更新 `docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`
    - 新增 `services/fabric-ca-admin/README.md`
    - 新增 `scripts/fabric-ca-admin-bootstrap.sh`、`scripts/fabric-ca-admin-test.sh`、`scripts/fabric-ca-admin-run.sh`
    - 更新 `Makefile`、`scripts/README.md`、`services/README.md`
    - 更新 `infra/docker/.env.local` 与 `docs/开发准备/配置项与密钥管理清单.md`
- 实现摘要：
  - Go 侧执行面与 Rust 公网控制面已按冻结边界解耦：
    - Rust 负责权限、`step-up`、公网错误码、审计主体
    - Go 负责签发 / 吊销执行、DB 回执落盘和系统日志
  - 签发链路要求 `iam.fabric_identity_binding.status='approved'`，以现有正式 schema 内的审批态作为 `AUD-016` 最小可用门槛，不引入第二套未冻结审批接口。
  - Go 服务会把：
    - `certificate_issue_receipt / ca.certificate_issued`
    - `certificate_revocation_receipt / ca.certificate_revoked`
    写入 `ops.external_fact_receipt`，并同步写入：
    - `fabric ca admin issued identity`
    - `fabric ca admin revoked identity`
    - `fabric ca admin revoked certificate`
  - Rust 侧公网接口会把真实操作者、角色和 `step_up_challenge_id` 写入 `audit.audit_event`，不再把 Go 服务伪装成审计主体。
  - 为让 `step-up/check` live smoke 稳定通过，本批补入固定测试夹具：
    - `core.organization.org_id='10000000-0000-0000-0000-000000000416'`
    - `core.user_account.user_id='10000000-0000-0000-0000-000000000417'`
    仅作为 `AUD-016` 本地平台操作员 fixture，避免重复重跑时再因 `iam.step_up_challenge.user_id` 外键失败。
- 验证步骤：
  1. `find services/fabric-ca-admin -name '*.go' -print0 | xargs -0 gofmt -w`
  2. `bash ./scripts/fabric-ca-admin-bootstrap.sh`
  3. `bash ./scripts/fabric-ca-admin-test.sh`
  4. `bash -lc 'source /home/luna/Documents/DataB/scripts/go-env.sh && cd /home/luna/Documents/DataB/services/fabric-ca-admin && go build ./...'`
  5. `curl -sS http://127.0.0.1:18112/healthz`
  6. `IAM_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core iam_fabric_ca_admin_db_smoke -- --nocapture`
  7. `psql ... SELECT action_name, actor_id::text, request_id, event_time FROM audit.audit_event ...`
  8. `psql ... SELECT message_text, request_id, created_at FROM ops.system_log ...`
  9. `cargo fmt --all`
  10. `cargo check -p platform-core`
  11. `cargo test -p platform-core`
  12. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  13. `./scripts/check-query-compile.sh`
  14. `./scripts/check-openapi-schema.sh`
- 验证结果：
  - Go 侧 `gofmt`、`go test ./...`、`go build ./...` 全部通过。
  - `fabric-ca-admin` 真实运行态健康检查通过：`{"service":"fabric-ca-admin","status":"ok"}`。
  - `IAM_DB_SMOKE=1 cargo test -p platform-core iam_fabric_ca_admin_db_smoke -- --nocapture` 通过，验证了：
    - `step-up -> platform-core -> fabric-ca-admin -> PostgreSQL` 主链真实跑通。
    - 签发后 `iam.fabric_identity_binding.status='issued'`、`iam.certificate_record.status='active'`。
    - 吊销后 `iam.certificate_record.status='revoked'`、`iam.fabric_identity_binding.status='revoked'`，并存在 `iam.certificate_revocation_record`。
    - `ops.external_fact_receipt` 业务测试数据在 smoke 内已按规则清理。
  - append-only 留痕回查通过：
    - `audit.audit_event` 最新两条为：
      - `iam.fabric.identity.issue / actor_id=10000000-0000-0000-0000-000000000417 / request_id=req-aud016-issue-db-smoke-1776837252908876954`
      - `iam.certificate.revoke / actor_id=10000000-0000-0000-0000-000000000417 / request_id=req-aud016-revoke-db-smoke-1776837252908876954`
    - `ops.system_log` 最新两条为：
      - `fabric ca admin issued identity / request_id=req-aud016-issue-db-smoke-1776837252908876954`
      - `fabric ca admin revoked certificate / request_id=req-aud016-revoke-db-smoke-1776837252908876954`
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅有仓库既存 `unused_*` warning。
  - `cargo test -p platform-core` 通过：`293` 个测试通过，既存 `iam_party_access_flow_live` 继续保持 ignored。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-016`
  - `技术选型正式版.md`：Go 只承担 Fabric 域职责，包括 `fabric-ca-admin`
  - `链上链下技术架构与能力边界稿.md`：Rust 控制面 + Go Fabric 执行面分层
  - `身份与会话接口协议正式版.md`：`fabric-identities / certificates` 正式公网路径
  - `权限设计/接口权限校验清单.md`：Fabric 身份 / 证书治理的正式权限点与高风险动作
  - `A04-AUD-Ops-接口与契约落地缺口.md`：AUD/Ops/Fabric 控制面契约收口义务
- 覆盖的任务清单条目：`AUD-016`
- 未覆盖项：
  - 真实 `Fabric CA / test-network / Gateway / chaincode` 仍留待 `AUD-017`
  - `projection-gaps` 公共控制面仍留待 `AUD-021`
  - `ops.consumer_idempotency_record + Redis` 的 Fabric consumer 幂等闭环仍留待 `AUD-026`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md` 的 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001`。
- 备注：
  - 首轮 live smoke 暴露两个运行态问题，当前批已修复并重新验证：
    1. `iam.step_up_challenge.user_id` 外键要求真实操作员主体，初版 smoke 未先准备 `core.user_account`；现已补固定 operator fixture。
    2. 首次失败残留固定 `registry_name` 种子，重跑命中 `uq_fabric_ca_registry_name`；现已改为带纳秒后缀的唯一 seed / request id，避免重复重跑互相污染。
  - 本批业务测试数据已清理；`audit.audit_event / ops.system_log` 按 append-only 保留。
  - 本批未发现新的 `CSV / Markdown / technical_reference / IAM 权限模型 / route-policy / topics / runbook / 代码` 冲突，不触发暂停条件。
### BATCH-230（计划中）
- 任务：AUD-017 链写入 Provider 切换到 `mock / fabric-test-network`
- 状态：计划中
- 说明：按 `AUD-017` 冻结口径，把 `fabric-adapter` 的 Go 执行面从单一 `mock` provider 扩展为可在 local 模式下切换 `mock` 与 `fabric-test-network` 两种实现，并把仓库内目前仍是 placeholder 的 `infra/fabric/**` 本地链基线一并收口为可重复执行的 test-network bootstrap / channel / chaincode 部署脚本，统一把外部依赖下载到 `third_party/external-deps/`。本批坚持既定语言边界：Rust 继续只负责平台控制面与审计主体，所有 Fabric Gateway、链码提交、链码部署、网络初始化都由 Go 服务和仓库脚本承接。
- 追溯：你已明确确认 `Go` 负责所有 Fabric 交互、链码和部署；本批据此推进 `AUD-017`，不再延续 placeholder Fabric 基线。
### BATCH-230（待审批）
- 任务：`AUD-017` 链写入 Provider 切换到 `mock / fabric-test-network`
- 状态：待审批
- 当前任务编号：`AUD-017`
- 前置依赖核对结果：`CORE-007`、`CORE-008`、`DB-008`、`ENV-022` 已作为前序基线完成；`AUD-016` 已于本地提交 `1cd8353` 完成，当前批次可继续把 `fabric-adapter` 切到真实 `fabric-test-network` provider。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-017` DoD 为 local 模式可切换 `mock / fabric-test-network`，且至少一条集成测试或手工验证通过，并在审计/日志中留下痕迹。
  - `docs/开发准备/技术选型正式版.md`、`docs/原始PRD/链上链下技术架构与能力边界稿.md`、`docs/领域模型/全量领域模型与对象关系说明.md`、`docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`：复核 `Go` 负责 Fabric Gateway / 链码 / 部署，Rust 只保留控制面与审计主体。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/fabric-adapter.md`、`docs/04-runbooks/provider-switch.md`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/compose-boundary.md`：核对本地启动、provider 切换和 compose/Fabric 边界需要同步改成真实 `test-network` 口径。
  - `infra/docker/docker-compose.local.yml`、`Makefile`、`scripts/check-fabric-local.sh`、`scripts/fabric-adapter-run.sh`、`infra/fabric/**`：确认仓库内仍存在 placeholder Fabric compose 与链码部署脚本，需要在本批收口为真实可执行基线。
  - `services/fabric-adapter/**`：确认现有 provider 仅有 `mock`，尚未真实等待 `commit status`，不满足 `AUD` 阶段 Fabric 回执要求。
- 实现要点：
  - `infra/fabric/**`：
    - 新增 `install-deps.sh`、`patch-samples.sh`、`deploy-chaincode.sh`、`query-anchor.sh`，统一把 Fabric 依赖下载到 `third_party/external-deps/fabric/`，并把官方 `fabric-samples` 默认 `latest/main` 漂移收口到 pinned `2.5.15 / 1.5.17`。
    - 新增 Go 链码 `infra/fabric/chaincode/datab-audit-anchor/`，实现 `SubmitEvidenceBatchRoot / SubmitOrderDigest / SubmitAuthorizationDigest / SubmitAcceptanceDigest / GetAnchorByReference`。
    - `fabric-up/down/reset/channel` 全部切到真实 `test-network` 与 Go 链码部署；删除 placeholder `infra/fabric/docker-compose.fabric.local.yml`。
    - `Makefile` 与 `infra/docker/docker-compose.local.yml` 收口：Fabric 不再由本地 compose 承担 placeholder 容器，正式由 `infra/fabric/*.sh` 启停。
  - `services/fabric-adapter/**`：
    - 新增 `fabric_gateway` provider、provider factory 与配置项，支持 `FABRIC_ADAPTER_PROVIDER_MODE=mock|fabric-test-network`。
    - 真实 `fabric-test-network` provider 使用 Go SDK 直连 Gateway，提交链码后显式等待 `commit status`，并把 `commit_status / commit_block / gateway_status=committed` 写入 receipt payload。
    - 新增 `live_smoke_test.go`，真实完成 `Gateway -> chaincode -> PostgreSQL receipt/audit/system_log -> ledger query` 联查。
  - 脚本与文档：
    - 新增 `scripts/fabric-env.sh`、`scripts/fabric-adapter-live-smoke.sh`。
    - 修复 `scripts/fabric-adapter-run.sh` 的环境变量覆盖顺序，允许外部显式切换 `FABRIC_ADAPTER_PROVIDER_MODE` 与 `TOPIC_*`。
    - 更新 `docs/04-runbooks/fabric-local.md`、`fabric-adapter.md`、`provider-switch.md`、`local-startup.md`、`compose-boundary.md`、`scripts/README.md`、`services/README.md` 到真实 `test-network` 口径。
    - 新增 `infra/fabric/state/.gitignore`，忽略运行时生成的 `runtime.env`。
- 验证步骤：
  1. `find services/fabric-adapter -name '*.go' -print0 | xargs -0 gofmt -w`
  2. `find infra/fabric/chaincode/datab-audit-anchor -name '*.go' -print0 | xargs -0 gofmt -w`
  3. `bash -n scripts/fabric-env.sh infra/fabric/install-deps.sh infra/fabric/patch-samples.sh infra/fabric/deploy-chaincode.sh infra/fabric/query-anchor.sh infra/fabric/fabric-up.sh infra/fabric/fabric-down.sh infra/fabric/fabric-reset.sh infra/fabric/fabric-channel.sh scripts/check-fabric-local.sh scripts/fabric-adapter-run.sh scripts/fabric-adapter-live-smoke.sh`
  4. `source ./scripts/go-env.sh && cd services/fabric-adapter && go mod tidy && go test ./...`
  5. `cd infra/fabric/chaincode/datab-audit-anchor && go mod tidy && go test ./...`
  6. `./infra/fabric/install-deps.sh`
  7. `./infra/fabric/fabric-up.sh`
  8. `./scripts/check-fabric-local.sh`
  9. `./scripts/fabric-adapter-live-smoke.sh`
  10. `timeout 5 env FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network FABRIC_ADAPTER_CONSUMER_GROUP=cg-fabric-adapter-aud017-startup-check TOPIC_AUDIT_ANCHOR=dtp.audit.anchor.smoke.aud017 TOPIC_FABRIC_REQUESTS=dtp.fabric.requests.smoke.aud017 ./scripts/fabric-adapter-run.sh`
  11. `cargo fmt --all`
  12. `cargo check -p platform-core`
  13. `cargo test -p platform-core`
  14. `set -a && source infra/docker/.env.local && set +a && cargo sqlx prepare --workspace`
  15. `./scripts/check-query-compile.sh`
  16. `./scripts/check-topic-topology.sh`
- 验证结果：
  - Go 侧 `gofmt`、`go mod tidy`、`go test ./...` 全部通过；新增 live smoke 测试会在 `FABRIC_ADAPTER_LIVE_SMOKE=1` 时真实连 Fabric。
  - `./infra/fabric/install-deps.sh` 通过，并把官方依赖下载到 `third_party/external-deps/fabric/`；补丁脚本已把样例网络的 `latest` 镜像标签收敛为 pinned `2.5.15 / 1.5.17`。
  - `./infra/fabric/fabric-up.sh` 通过，真实启动 `ca_org1 / ca_org2 / ca_orderer / orderer.example.com / peer0.org1.example.com / peer0.org2.example.com`，创建 channel `datab-channel`，并成功部署 Go 链码 `datab-audit-anchor`。
  - `./scripts/check-fabric-local.sh` 通过，确认 channel、链码版本/sequence 与 `Ping` 均正确。
  - `./scripts/fabric-adapter-live-smoke.sh` 通过：真实提交 `audit.anchor_requested`，等待 `commit status=VALID`，并回查：
    - `ops.external_fact_receipt.receipt_status=committed`
    - `audit.audit_event.action_name=fabric.adapter.submit`
    - `ops.system_log.message_text='fabric adapter accepted submit event'`
    - `chain.chain_anchor.status=submitted` 且 `tx_hash=provider_reference`
    - `./infra/fabric/query-anchor.sh anchor_batch <id> evidence_batch_root` 返回账本记录，`transaction_id` 与 `provider_reference` 一致。
  - `timeout ... ./scripts/fabric-adapter-run.sh` 通过启动检查：日志已显示 `provider_mode=fabric-test-network` 且 `audit_anchor_topic / fabric_requests_topic` 可由外部环境覆盖。
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh`、`./scripts/check-topic-topology.sh` 全部通过；Rust 侧仅剩仓库既存 `unused_*` warning。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-017`
  - `技术选型正式版.md`：Go 负责 Fabric 执行面
  - `链上链下技术架构与能力边界稿.md`：Rust 控制面 + Go Fabric 执行面分层
  - `A04-AUD-Ops-接口与契约落地缺口.md`：Fabric 适配器/本地运行契约必须落成正式脚本与 runbook
- 覆盖的任务清单条目：`AUD-017`
- 未覆盖项：
  - `projection-gaps` 公共控制面仍留待 `AUD-021`
  - `fabric-event-listener` 的真实 chaincode event callback 全闭环与 consumer 幂等/DLQ/reprocess 统一矩阵仍留待后续 `AUD` 批次
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 收敛为仅继续跟踪 `AUD-021 projection-gaps`。
- 备注：
  - 本批中途发现官方 `fabric-samples` 默认拉取 `main/latest`，会把 peer 拉到 `3.1.4` 并导致 Go 链码安装口径漂移；当前已通过仓库脚本固定到 pinned `2.5.15 / 1.5.17`，不再依赖机器偶然状态。
  - 本批未发现新的 `CSV / Markdown / technical_reference / topics / route-policy / Fabric 组件边界 / Go-Rust 分层` 冲突，不触发暂停条件。
### BATCH-231（计划中）
- 任务：AUD-018 交易链监控总览接口与 checkpoints 接口
- 状态：计划中
- 说明：按 `AUD-018` 冻结口径，实现 `GET /api/v1/ops/trade-monitor/orders/{orderId}` 与 `GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints`。本批以 `ops.trade_lifecycle_checkpoint`、`ops.external_fact_receipt`、`risk.fairness_incident`、`ops.chain_projection_gap` 和 `trade.order_main / chain.chain_anchor / audit.audit_event` 为正式读模型来源，返回订单维度总览、最近检查点/外部事实/公平性事件/链投影缺口摘要，并补齐 `ops.trade_monitor.read` 权限、tenant+order scope 校验、`audit.access_audit + ops.system_log` 查询留痕、OpenAPI、runbook 与测试矩阵。当前不扩展到 `external-facts / fairness-incidents / projection-gaps` 公共接口，这些分别留待 `AUD-019 ~ AUD-021`。
- 追溯：按 `CSV > Markdown > 其他辅助文档` 执行；本批严格对应 `AUD-018`，完成后再进入 `AUD-019`。
### BATCH-231（待审批）
- 任务：`AUD-018` 交易链监控总览接口与 checkpoints 接口
- 状态：待审批
- 当前任务编号：`AUD-018`
- 前置依赖核对结果：`CORE-007`、`CORE-008`、`DB-008`、`ENV-022` 已作为前序基线完成；`AUD-017` 已于本地提交 `b022570` 完成，当前批次可继续在既有 Go/Fabric 写链基线上补齐交易链监控只读 control-plane。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `AUD-018` 的 DoD、验收与 `technical_reference`，要求 `GET /api/v1/ops/trade-monitor/orders/{orderId}`、`GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints` 的接口、DTO、权限、审计与最小测试齐备。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `PostgreSQL / Kafka / Redis / Keycloak / OpenSearch / Fabric / 观测栈` 的职责边界，确认本批是 `platform-core` 只读聚合面，Fabric 真实交互继续由 Go 服务承担。
  - `docs/原始PRD/交易链监控、公平性与信任安全设计.md`：提取六层交易链监控模型，确认总览必须同时暴露检查点、外部事实、公平性事件和链投影缺口摘要。
  - `docs/数据库设计/接口协议/交易链监控与公平性接口协议正式版.md`：确认 `ops.trade_monitor.read`、两个接口路径及过滤字段口径。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认本接口读取的状态来自正式链上链下双层权威对象，而不是旁路 topic。
  - `docs/开发任务/问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md`：确认 trade-monitor 必须同步落正式 router、OpenAPI、runbook 与测试矩阵。
  - `docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/kafka-topics.md`、`docs/00-context/async-chain-write.md`、`infra/kafka/topics.v1.json`、`docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql`、`docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`、`infra/docker/docker-compose.local.yml`：复核本批不新增 topic / route authority，不把 `dtp.outbox.domain-events` 当 trade-monitor 读源。
  - `apps/platform-core/src/modules/audit/**`、`services/fabric-adapter/**`、`services/fabric-event-listener/**`、`services/fabric-ca-admin/**`、`workers/outbox-publisher/**`、`packages/openapi/**`、`docs/02-openapi/**`、`docs/04-runbooks/**`、`docs/05-test-cases/**`、`infra/**`、`scripts/**`：确认现有 `AUD` 代码已具备 `ops.external_fact_receipt / ops.chain_projection_gap / consistency` 查询基础，但 trade-monitor 接口、OpenAPI、runbook 与矩阵尚未完整落地。
- 实现要点：
  - `apps/platform-core/src/modules/audit/domain/mod.rs`、`dto/mod.rs`：新增 `TradeMonitorCheckpointQuery`、`TradeMonitorOverviewView`、`TradeMonitorCheckpointPageView` 以及 `TradeLifecycleCheckpointView / FairnessIncidentView`，统一 trade-monitor 控制面的正式读 DTO。
  - `apps/platform-core/src/modules/audit/repo/mod.rs`：新增 `TradeLifecycleCheckpointRecord / Page`、`FairnessIncidentRecord / Page`、`search_trade_lifecycle_checkpoints_by_order`、`search_recent_fairness_incidents_for_order`、`count_open_fairness_incidents_for_order`，把 `ops.trade_lifecycle_checkpoint` 与 `risk.fairness_incident` 纳入正式仓储查询能力。
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`：新增 `GET /api/v1/ops/trade-monitor/orders/{orderId}` 与 `/checkpoints`，补齐 `ops.trade_monitor.read` 权限门面、`tenant_admin / tenant_audit_readonly` 的 `tenant + order scope` 校验，以及 `audit.access_audit + ops.system_log` 查询留痕。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：新增路由级权限 / `x-request-id` 测试与 `audit_trade_monitor_db_smoke`，真实写入 `trade.order_main / ops.trade_lifecycle_checkpoint / ops.external_fact_receipt / risk.fairness_incident / ops.chain_projection_gap / chain.chain_anchor` 并调用两条正式 API 做回查。
  - `docs/数据库设计/V1/upgrade/068_trade_chain_monitoring_authz.sql`、`db/scripts/verify-migration-065-068.sh`：补齐 `tenant_admin / tenant_audit_readonly -> ops.trade_monitor.read` 的正式 role-permission seed 校验，避免 tenant scope 只能靠 handler 临时白名单。
  - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、`packages/openapi/README.md`、`docs/02-openapi/README.md`：同步补齐 trade-monitor 两条路径、schema、示例与成熟度说明。
  - `docs/04-runbooks/audit-trade-monitor.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`：补齐宿主机联调手册、验收矩阵与索引。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `./scripts/check-openapi-schema.sh`
  7. `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_trade_monitor_db_smoke -- --nocapture`
  8. 真实宿主机联调：`APP_PORT=18080 cargo run -p platform-core-bin` + `curl GET /api/v1/ops/trade-monitor/orders/{orderId}` + `curl GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints` + `psql` 回查
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既存 `unused_*` warning，无新增编译失败。
  - `cargo test -p platform-core` 通过：`296 passed; 0 failed`，新增 `audit_trade_monitor_db_smoke`、权限路由测试与既有 `AUD` smoke 全部通过。
  - `cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过，确认 `packages/openapi/ops.yaml` 与 `docs/02-openapi/ops.yaml` 同步且 schema 骨架完整。
  - `AUD_DB_SMOKE=1 ... audit_trade_monitor_db_smoke` 通过：真实完成 `trade monitor overview / checkpoints` API 调用、tenant scope 校验、`audit.access_audit / ops.system_log` 回查与临时业务对象清理。
  - 真实宿主机联调通过：手工写入一笔最小订单图与 `checkpoint / external fact / fairness incident / projection gap / chain anchor` 后，执行：
    - `GET /api/v1/ops/trade-monitor/orders/7957152d-6e66-4a57-9826-ae78a72ca65d`
    - `GET /api/v1/ops/trade-monitor/orders/7957152d-6e66-4a57-9826-ae78a72ca65d/checkpoints?checkpoint_status=pending&lifecycle_stage=delivery&page=1&page_size=20`
    - `tenant_admin + x-tenant-id` 的同订单总览查询
    并回查到：
    - `audit.access_audit` 共 `3` 条，`target_type in ('trade_monitor_query','trade_checkpoint_query')`
    - `ops.system_log` 共 `3` 条，对应 overview/checkpoints 两条正式路径
    - `trade.order_main.status='buyer_locked'`
    - `ops.trade_lifecycle_checkpoint` 命中 `delivery_prepared / pending`
    - `ops.external_fact_receipt.receipt_status='confirmed'`
    - `risk.fairness_incident.status='open'`
    - `ops.chain_projection_gap.gap_status='open'`
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-018`
  - `交易链监控、公平性与信任安全设计.md`：六层交易链监控模型
  - `交易链监控与公平性接口协议正式版.md`：trade-monitor 两条正式接口、`ops.trade_monitor.read` 与过滤口径
  - `业务流程图-V1-完整版.md`：链上链下一致性与事件流读取边界
  - `A04-AUD-Ops-接口与契约落地缺口.md`：trade-monitor 契约 / runbook / 测试收口缺口
  - `068_trade_chain_monitoring_authz.sql`：trade-monitor 权限与角色绑定
- 覆盖的任务清单条目：`AUD-018`
- 未覆盖项：
  - `GET /api/v1/ops/external-facts`、`POST /confirm` 留待 `AUD-019`
  - `GET /api/v1/ops/fairness-incidents`、`POST /handle` 留待 `AUD-020`
  - `GET /api/v1/ops/projection-gaps`、`POST /resolve` 留待 `AUD-021`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 推进到包含 `AUD-018 trade-monitor` 的最新状态。
- 备注：
  - 本批没有新增 Go / Fabric 执行面代码；这是因为 `AUD-018` 按冻结口径属于 `platform-core` 的只读聚合面。Fabric 的真实写链、Gateway、chaincode event listener 与 CA admin 仍保持 `AUD-013 ~ AUD-017` 已落地的 `Go` 分层，不回退到 Rust 直接交互。
  - 手工清理时，尝试删除本次联调用于访问的 `core.user_account` 会触发 `audit.access_audit.accessor_user_id -> SET NULL -> UPDATE append-only`，被 append-only guard 正常拒绝；因此本批按运行态边界只清理订单、商品和监控对象等临时业务数据，保留该用户及组织作为审计留痕依赖，不视为业务测试脏数据外泄。
  - 本批未发现新的 `CSV / Markdown / technical_reference / authz seed / OpenAPI / Go-Rust 分层` 冲突，不触发暂停条件。
### BATCH-232（计划中）
- 任务：`AUD-019` 外部事实查询 / 确认接口
- 状态：计划中
- 说明：按 `AUD-019` 冻结口径，实现 `GET /api/v1/ops/external-facts` 与 `POST /api/v1/ops/external-facts/{id}/confirm`。本批严格以 `ops.external_fact_receipt` 为正式持久化对象，不新增 `external_receipt` 或其他旁路表；查询接口补齐 `order_id / ref_type / ref_id / fact_type / provider_type / receipt_status / request_id / trace_id / from / to` 过滤、platform-only 鉴权与 `audit.access_audit + ops.system_log` 留痕；确认接口补齐 `ops.external_fact.manage`、`step-up`、`pending` 状态约束、确认结果回写、正式审计事件和系统日志，并明确“不直接改写业务主状态，只记录回执确认结果并为后续规则评估保留正式留痕”。同时同步 `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、runbook、测试矩阵和 `AUD_DB_SMOKE + 宿主机 curl + psql` 验证。
- 追溯：已按 `CSV > Markdown > technical_reference > 其他辅助文档` 重新核对 `AUD-019`、`交易链监控与公平性接口协议正式版`、`067/068/072/074`、`A04`、`trade-monitor` / `consistency` / `fabric-local` runbook 与当前 `audit` 模块实现；当前未发现需要暂停的人为冲突。
### BATCH-232（待审批）
- 任务：`AUD-019` 外部事实查询 / 确认接口
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`、`domain/mod.rs`、`repo/mod.rs`：新增 `GET /api/v1/ops/external-facts` 与 `POST /api/v1/ops/external-facts/{id}/confirm`，补齐 `order_id / ref_type / ref_id / fact_type / provider_type / receipt_status / request_id / trace_id / from / to` 过滤、`ops.external_fact.read / ops.external_fact.manage` 权限矩阵、`step-up` 绑定、`pending` 状态保护，以及 `ops.external_fact_receipt` 的正式查询 / 单对象装载 / confirmation 回写。
  - `apps/platform-core/src/modules/audit/api/handlers.rs`：确认动作只更新 `ops.external_fact_receipt.receipt_status / confirmed_at / metadata`，把 `manual_confirmation` 与 `rule_evaluation.status='pending_follow_up'` 写回 receipt metadata；同步写入 `audit.audit_event(action_name='ops.external_fact.confirm')`、`audit.access_audit(target_type='external_fact_receipt')` 与 `ops.system_log`，并明确不直接改写 `trade.order_main.external_fact_status`。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：新增路由级 `permission / request-id / step-up` 测试与 `audit_external_fact_confirm_db_smoke`；真实插入最小订单图、`ops.external_fact_receipt(receipt_status='pending')`、`iam.step_up_challenge`，调用 list + confirm 两条正式 API，并回查 `ops.external_fact_receipt / trade.order_main / audit.audit_event / audit.access_audit / ops.system_log`。
  - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、`packages/openapi/README.md`、`docs/02-openapi/README.md`：归档 `external-facts` 两条正式接口、请求/响应 schema 与示例，并把剩余公共控制面缺口收敛到 `AUD-020~021`。
  - `docs/04-runbooks/audit-external-facts.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`：补齐 `AUD-019` 的宿主机 `curl + psql` 操作手册、验收矩阵、排障说明与回查 SQL。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：无新增 `V1-gap / V2-reserved / V3-reserved`；`TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 已推进到仅剩 `AUD-020~021` 的 `fairness-incidents / projection-gaps` 公共控制面缺口。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既存 `unused_*` warning，无新增编译失败。
  - `cargo test -p platform-core` 通过：`299 passed; 0 failed`，新增 `audit_external_fact_confirm_db_smoke` 与路由级权限 / step-up 测试均通过。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过，确认 `packages/openapi/ops.yaml` 与 `docs/02-openapi/ops.yaml` 同步且 schema 骨架完整。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_external_fact_confirm_db_smoke -- --nocapture` 通过：真实完成 `ops.external_fact_receipt` 回写、`rule_evaluation.status='pending_follow_up'` 留痕、`trade.order_main.external_fact_status` 不变、`audit.audit_event + audit.access_audit + ops.system_log` 回查与临时业务对象清理。
  - 真实宿主机联调通过：启动 `APP_PORT=18080 cargo run -p platform-core-bin`，用 `psql` 手工写入一笔最小订单图、`ops.external_fact_receipt(receipt_status='pending')` 与 verified `iam.step_up_challenge`，然后执行：
    - `GET /api/v1/ops/external-facts?order_id=a0e9e886-9d16-47d5-be1f-5ade77a4bade&receipt_status=pending&provider_type=mock_payment_provider&page=1&page_size=20`
    - `POST /api/v1/ops/external-facts/3a343777-d9da-44fa-8f93-78a572181cd5/confirm`
    并回查到：
    - `ops.external_fact_receipt.receipt_status='confirmed'`
    - `ops.external_fact_receipt.metadata.manual_confirmation.confirm_result='confirmed'`
    - `ops.external_fact_receipt.metadata.rule_evaluation.status='pending_follow_up'`
    - `trade.order_main.external_fact_status='pending_receipt'`，未被 confirm 直接改写
    - `audit.audit_event(action_name='ops.external_fact.confirm', result_code='confirmed') = 1`
    - `audit.access_audit` 共 `2` 条，`target_type in ('external_fact_query','external_fact_receipt')`
    - `ops.system_log` 共 `2` 条，对应 list / confirm 两条正式路径
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-019`
  - `交易链监控与公平性接口协议正式版.md`：`GET /api/v1/ops/external-facts`、`POST /api/v1/ops/external-facts/{id}/confirm`
  - `067_trade_chain_monitoring.sql`、`068_trade_chain_monitoring_authz.sql`：`ops.external_fact_receipt` 正式对象与 `ops.external_fact.read / manage` 权限绑定
  - `A04-AUD-Ops-接口与契约落地缺口.md`：external-facts 公共控制面的契约 / runbook / 测试收口缺口
  - `async-chain-write.md`、`kafka-topics.md`：确认本批仍是正式 receipt 控制面，不旁路出新的 topic / outbox 链
- 覆盖的任务清单条目：`AUD-019`
- 未覆盖项：
  - `GET /api/v1/ops/fairness-incidents`、`POST /handle` 留待 `AUD-020`
  - `GET /api/v1/ops/projection-gaps`、`POST /resolve` 留待 `AUD-021`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 推进到包含 `AUD-019 external-facts` 的最新状态。
- 备注：
  - 本批没有新增 Go / Fabric 执行面代码；这是因为 `AUD-019` 按冻结口径属于 `platform-core` 的外部事实控制面。Fabric 的真实写链、Gateway、chaincode event listener 与 CA admin 仍保持 `AUD-013 ~ AUD-017` 已落地的 `Go` 分层，不回退到 Rust 直接交互。
  - 手工清理时，尝试删除本次联调用到的 `iam.step_up_challenge` 会触发 `audit.audit_event.step_up_challenge_id -> SET NULL -> UPDATE append-only`，尝试删除 `core.user_account / buyer organization` 会触发 `audit.access_audit.accessor_user_id -> SET NULL -> UPDATE append-only`；这两类对象因此按审计依赖保留，不视为业务测试脏数据外泄。其余订单 / 商品 / receipt 业务测试数据已清理。
  - 本批未发现新的 `CSV / Markdown / technical_reference / authz seed / OpenAPI / Go-Rust 分层` 冲突，不触发暂停条件。
### BATCH-233（计划中）
- 任务：`AUD-020` 公平性事件查询 / 处理接口
- 状态：计划中
- 说明：按 `AUD-020` 冻结口径，实现 `GET /api/v1/ops/fairness-incidents` 与 `POST /api/v1/ops/fairness-incidents/{id}/handle`。本批以 `risk.fairness_incident` 为唯一正式持久化对象，查询接口补齐 `order_id / incident_type / severity / fairness_incident_status / assigned_role_key / assigned_user_id / request_id / trace_id` 过滤、platform-only 鉴权与 `audit.access_audit + ops.system_log` 留痕；处理接口补齐 `risk.fairness_incident.handle`、`step-up`、事件状态约束、处理结果回写、正式审计事件和系统日志，并明确“只能处理事件与联动建议，不直接篡改业务事实”。同时同步 `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、runbook、测试矩阵和 `AUD_DB_SMOKE + 宿主机 curl + psql` 验证。
- 追溯：已按 `CSV > Markdown > technical_reference > 其他辅助文档` 重新核对 `AUD-020`、`交易链监控与公平性接口协议正式版`、`交易链监控、公平性与信任安全设计`、`067/068/072/074`、`A04`、`trade-monitor` / `external-facts` / `fabric-local` runbook 与当前 `audit` 模块实现；当前未发现需要暂停的人为冲突。
### BATCH-233（待审批）
- 任务：`AUD-020` 公平性事件查询 / 处理接口
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`、`domain/mod.rs`、`repo/mod.rs`：新增 `GET /api/v1/ops/fairness-incidents` 与 `POST /api/v1/ops/fairness-incidents/{id}/handle`，补齐 `order_id / incident_type / severity / fairness_incident_status / assigned_role_key / assigned_user_id / request_id / trace_id` 过滤、`risk.fairness_incident.read / risk.fairness_incident.handle` 权限矩阵、`step-up` 绑定、`open -> close` 状态保护，以及 `risk.fairness_incident` 的正式查询 / 单对象装载 / manual handle 回写。
  - `apps/platform-core/src/modules/audit/api/handlers.rs`：处理动作只更新 `risk.fairness_incident.status / auto_action_code / resolution_summary / metadata / closed_at`，把 `handling` 与 `linked_action_plan` 写回 incident metadata；`freeze_settlement / freeze_delivery / create_dispute_suggestion` 仅记录为 `suggestion_only` 联动建议，不直接改写 `trade.order_main`，并同步写入 `audit.audit_event(action_name='risk.fairness_incident.handle')`、`audit.access_audit(target_type='fairness_incident')` 与 `ops.system_log`。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：新增路由级 `permission / step-up` 测试与 `audit_fairness_incident_handle_db_smoke`；真实插入最小订单图、`ops.trade_lifecycle_checkpoint`、`ops.external_fact_receipt(receipt_status='pending')`、`risk.fairness_incident(status='open')` 与 verified `iam.step_up_challenge`，调用 list + handle 两条正式 API，并回查 `risk.fairness_incident / trade.order_main / audit.audit_event / audit.access_audit / ops.system_log`。
  - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、`packages/openapi/README.md`、`docs/02-openapi/README.md`：归档 `fairness-incidents` 两条正式接口、请求/响应 schema 与示例，并把剩余公共控制面缺口收敛到 `AUD-021 projection-gaps`。
  - `docs/04-runbooks/audit-fairness-incidents.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`：补齐 `AUD-020` 的宿主机 `curl + psql` 操作手册、验收矩阵、排障说明与回查 SQL。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：无新增 `V1-gap / V2-reserved / V3-reserved`；`TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 已推进到仅剩 `AUD-021 projection-gaps` 公共控制面缺口。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既存 `unused_*` warning，无新增编译失败。
  - `cargo test -p platform-core` 通过：`302 passed; 0 failed`，新增 `audit_fairness_incident_handle_db_smoke` 与路由级权限 / step-up 测试均通过。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过，确认 `packages/openapi/ops.yaml` 与 `docs/02-openapi/ops.yaml` 同步且 schema 骨架完整。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_fairness_incident_handle_db_smoke -- --nocapture` 通过：真实完成 `risk.fairness_incident` 查询 / handle、`linked_action_plan` 建议留痕、`trade.order_main` 主状态不变、`audit.audit_event + audit.access_audit + ops.system_log` 回查与临时业务对象清理。
  - 真实宿主机联调通过：启动 `APP_PORT=18080 cargo run -p platform-core-bin`，用 `psql` 手工写入一笔最小订单图、`ops.trade_lifecycle_checkpoint`、`ops.external_fact_receipt`、`risk.fairness_incident(status='open')` 与 verified `iam.step_up_challenge`，然后执行：
    - `GET /api/v1/ops/fairness-incidents?order_id=...&incident_type=seller_delivery_delay&severity=high&fairness_incident_status=open&assigned_role_key=platform_risk_settlement&assigned_user_id=...&page=1&page_size=20`
    - `POST /api/v1/ops/fairness-incidents/<fairness_incident_id>/handle`
    并回查到：
    - `risk.fairness_incident.status='closed'`
    - `risk.fairness_incident.auto_action_code='notify_ops'`
    - `risk.fairness_incident.metadata.handling.action='close'`
    - `risk.fairness_incident.metadata.linked_action_plan.status='suggestion_recorded'`
    - `trade.order_main.settlement_status='pending_settlement'`、`delivery_status='pending_delivery'`、`dispute_status='none'`，未被 handle 直接改写
    - `audit.audit_event(action_name='risk.fairness_incident.handle', result_code='close') = 1`
    - `audit.access_audit` 共 `2` 条，`target_type in ('fairness_incident_query','fairness_incident')`
    - `ops.system_log` 共 `2` 条，对应 list / handle 两条正式路径
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-020`
  - `交易链监控与公平性接口协议正式版.md`：`GET /api/v1/ops/fairness-incidents`、`POST /api/v1/ops/fairness-incidents/{id}/handle`
  - `交易链监控、公平性与信任安全设计.md`：公平性事件来源、等级与人工处理边界
  - `067_trade_chain_monitoring.sql`、`068_trade_chain_monitoring_authz.sql`：`risk.fairness_incident` 正式对象与 `risk.fairness_incident.read / handle` 权限绑定
  - `A04-AUD-Ops-接口与契约落地缺口.md`：fairness-incidents 公共控制面的契约 / runbook / 测试收口缺口
  - `async-chain-write.md`、`kafka-topics.md`：确认本批仍是正式 incident 控制面，不旁路出新的 topic / outbox 链
- 覆盖的任务清单条目：`AUD-020`
- 未覆盖项：
  - `GET /api/v1/ops/projection-gaps`、`POST /api/v1/ops/projection-gaps/{id}/resolve` 留待 `AUD-021`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 推进到仅剩 `AUD-021 projection-gaps` 的最新状态。
- 备注：
  - 本批没有新增 Go / Fabric 执行面代码；这是因为 `AUD-020` 按冻结口径属于 `platform-core` 的公平性事件控制面。Fabric 的真实写链、Gateway、chaincode、event listener 与 CA admin 仍保持 `AUD-013 ~ AUD-017` 已落地的 `Go` 分层，不回退到 Rust 直接交互。
  - 手工清理时，尝试删除本次联调用到的 `iam.step_up_challenge` 会触发 `audit.audit_event.step_up_challenge_id -> SET NULL -> UPDATE append-only`，尝试删除 `core.user_account / buyer organization` 会触发 `audit.access_audit.accessor_user_id -> SET NULL -> UPDATE append-only`；这两类对象因此按审计依赖保留，不视为业务测试脏数据外泄。其余订单 / 商品 / checkpoint / fairness incident / external fact 业务测试数据已清理。
  - 本批未发现新的 `CSV / Markdown / technical_reference / authz seed / OpenAPI / Go-Rust 分层` 冲突，不触发暂停条件。
### BATCH-234（计划中）
- 任务：`AUD-021` 投影缺口查询 / 关闭接口
- 状态：计划中
- 说明：按 `AUD-021` 冻结口径，实现 `GET /api/v1/ops/projection-gaps` 与 `POST /api/v1/ops/projection-gaps/{id}/resolve`。本批以 `ops.chain_projection_gap` 为唯一正式持久化对象，查询接口补齐 `aggregate_type / aggregate_id / order_id / chain_id / gap_type / gap_status / request_id / trace_id` 过滤、platform-only 鉴权与 `audit.access_audit + ops.system_log` 留痕；关闭接口补齐 `ops.projection_gap.manage`、`step-up`、缺口状态约束、`dry_run` 预演、`expected_state_digest` 乐观校验、正式审计事件和系统日志，并明确“`reconcile` 仍由 `AUD-012` 的控制面承接，`resolve` 只关闭正式 `ops.chain_projection_gap` 对象，不引入 `reconcile_job` 表”。同时同步 `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、runbook、测试矩阵和 `AUD_DB_SMOKE + 宿主机 curl + psql` 验证。
- 追溯：已按 `CSV > Markdown > technical_reference > 其他辅助文档` 重新核对 `AUD-021`、`交易链监控与公平性接口协议正式版`、`一致性与事件接口协议正式版`、`交易链监控、公平性与信任安全设计`、`067/068/072/074`、`A04`、`audit-consistency-reconcile` / `audit-trade-monitor` / `kafka-topics` / `async-chain-write` runbook 与当前 `audit` 模块实现；当前未发现需要暂停的人为冲突。
### BATCH-234（待审批）
- 任务：`AUD-021` 投影缺口查询 / 关闭接口
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/audit/api/router.rs`、`handlers.rs`、`domain/mod.rs`、`repo/mod.rs`：新增 `GET /api/v1/ops/projection-gaps` 与 `POST /api/v1/ops/projection-gaps/{id}/resolve`，补齐 `aggregate_type / aggregate_id / order_id / chain_id / gap_type / gap_status / request_id / trace_id` 过滤、`ops.projection_gap.read / manage` 正式权限点、`step-up` 绑定、`dry_run=true` 默认预演、`expected_state_digest` 乐观校验，以及 `ops.chain_projection_gap` 的正式查询 / 单对象装载 / manual resolve 回写。
  - `apps/platform-core/src/modules/audit/api/handlers.rs`：关闭动作只更新 `ops.chain_projection_gap.gap_status / resolved_at / request_id / trace_id / resolution_summary / metadata`，显式把 `formal_persistent_object='ops.chain_projection_gap'` 与 `control_plane_action='projection_gap.resolve'` 写入审计 metadata；同时写入 `audit.audit_event(action_name='ops.projection_gap.resolve')`、`audit.access_audit(target_type='projection_gap')` 与 `ops.system_log`，并明确不引入 `reconcile_job` 表、不直接发布 `dtp.consistency.reconcile`。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：新增路由级 `permission / step-up` 测试与 `audit_projection_gap_resolve_db_smoke`；真实插入最小订单图、`ops.chain_projection_gap(status='open')` 与 verified `iam.step_up_challenge`，调用 list + dry-run + execute 三条正式路径，并回查 `ops.chain_projection_gap / audit.audit_event / audit.access_audit / ops.system_log / ops.outbox_event`。
  - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、`packages/openapi/README.md`、`docs/02-openapi/README.md`：归档 `projection-gaps` 两条正式接口、请求/响应 schema 与示例，并把剩余公共控制面缺口推进到 `AUD-022+` 的搜索运维接口。
  - `docs/04-runbooks/audit-projection-gaps.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`：补齐 `AUD-021` 的宿主机 `curl + psql` 操作手册、验收矩阵、排障说明与“只关闭正式 projection gap 对象、不派生 reconcile job”边界。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：无新增 `V1-gap / V2-reserved / V3-reserved`；`TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 已推进到 `AUD-022+` 搜索运维控制面。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既存 `unused_*` warning，无新增编译失败。
  - `cargo test -p platform-core projection_gap -- --nocapture` 通过：新增 `projection-gaps` 路由级权限 / step-up 测试与 `audit_projection_gap_resolve_db_smoke` 全部通过。
  - `cargo test -p platform-core` 通过：`305 passed; 0 failed`，确认本批改动未回归既有业务主链。
  - `cargo build -p platform-core-bin` 通过；宿主机联调前显式刷新 `platform-core-bin` 可执行文件，避免把 `cargo check` 的静态编译结果误当成最新 host binary。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线查询缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过，确认 `packages/openapi/ops.yaml` 与 `docs/02-openapi/ops.yaml` 同步且 schema 骨架完整。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_projection_gap_resolve_db_smoke -- --nocapture` 通过：真实完成 `projection-gaps` 查询 / dry-run / resolve、`audit.audit_event + audit.access_audit + ops.system_log` 回查，以及 `ops.outbox_event(target_topic='dtp.consistency.reconcile') = 0` 断言。
  - 真实宿主机联调通过：启动 `APP_PORT=18080 target/debug/platform-core-bin`，用 `psql` 手工写入一笔最小订单图、`ops.chain_projection_gap(status='open')` 与 verified `iam.step_up_challenge`，然后执行：
    - `GET /api/v1/ops/projection-gaps?aggregate_type=order&aggregate_id=...&order_id=...&chain_id=fabric-local&gap_type=missing_callback&gap_status=open&page=1&page_size=10`
    - `POST /api/v1/ops/projection-gaps/<chain_projection_gap_id>/resolve` 的 `dry_run=true`
    - `POST /api/v1/ops/projection-gaps/<chain_projection_gap_id>/resolve` 的真实执行
    并回查到：
    - `ops.chain_projection_gap.gap_status='resolved'`
    - `ops.chain_projection_gap.resolution_summary.manual_resolution.reason='confirmed callback backfilled into projection gap'`
    - `ops.chain_projection_gap.metadata.manual_resolution.current_state_digest` 与 dry-run 返回值一致
    - `audit.audit_event(action_name='ops.projection_gap.resolve') = 2`
    - `audit.access_audit` 共 `3` 条，`target_type in ('projection_gap_query','projection_gap')`
    - `ops.system_log` 共 `3` 条，对应 list / dry-run / execute 三条正式路径
    - `ops.outbox_event(target_topic='dtp.consistency.reconcile') = 0`，确认 `resolve` 不直接派发 reconcile 执行事件
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-021`
  - `交易链监控与公平性接口协议正式版.md`：`GET /api/v1/ops/projection-gaps`、`POST /api/v1/ops/projection-gaps/{id}/resolve`
  - `一致性与事件接口协议正式版.md`：`V1 reconcile` 是控制面动作，不单列正式 `reconcile_job` 表；`AUD-021` 只关闭 `ops.chain_projection_gap`
  - `交易链监控、公平性与信任安全设计.md`：投影缺口属于双层权威模型下的一致性缺口对象，不直接反写业务主状态
  - `067_trade_chain_monitoring.sql`、`068_trade_chain_monitoring_authz.sql`：`ops.chain_projection_gap` 正式对象与 `ops.projection_gap.read / manage` 权限绑定
  - `A04-AUD-Ops-接口与契约落地缺口.md`：projection-gaps 公共控制面的契约 / runbook / 测试收口缺口
  - `async-chain-write.md`、`kafka-topics.md`：确认本批仍是正式 gap 控制面，不旁路出新的 topic / outbox 链
- 覆盖的任务清单条目：`AUD-021`
- 未覆盖项：
  - `GET /api/v1/ops/search/sync`、重建 / alias 切换 / 缓存失效 / 排序配置更新接口，留待 `AUD-022`
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已同步更新 `docs/开发任务/V1-Core-TODO与预留清单.md`，把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 推进到 `AUD-022+` 的最新状态。
- 备注：
  - 本批没有新增 Go / Fabric 执行面代码；这是因为 `AUD-021` 按冻结口径属于 `platform-core` 的 projection gap 控制面。Fabric 的真实写链、Gateway、chaincode、event listener 与 CA admin 仍保持 `AUD-013 ~ AUD-017` 已落地的 `Go` 分层，不回退到 Rust 直接交互。
  - 宿主机联调最初打到旧版 `platform-core-bin` 导致 `404`，已通过显式 `cargo build -p platform-core-bin` 刷新 host binary 后复验通过；判定为本地验证流程问题，而非接口实现缺陷。
  - 手工清理时，尝试删除本次联调用到的 `iam.step_up_challenge` 会触发 `audit.audit_event.step_up_challenge_id -> SET NULL -> UPDATE append-only`，尝试删除 `core.user_account / buyer organization` 会触发 `audit.access_audit.accessor_user_id -> SET NULL -> UPDATE append-only`；这两类对象因此按审计依赖保留，不视为业务测试脏数据外泄。其余订单 / 商品 / projection gap 业务测试数据已清理。
  - 本批未发现新的 `CSV / Markdown / technical_reference / authz seed / OpenAPI / Go-Rust 分层` 冲突，不触发暂停条件。
### BATCH-235（计划中）
- 任务：`AUD-022` 搜索同步状态 / 重建 / 别名切换 / 缓存失效 / 排序配置更新接口
- 状态：计划中
- 说明：按 `AUD-022` 冻结口径，对现有 `apps/platform-core/src/modules/search/**` 进行正式化改造。当前仓库已有 `GET /api/v1/ops/search/sync`、`POST /api/v1/ops/search/reindex`、`POST /api/v1/ops/search/aliases/switch`、`POST /api/v1/ops/search/cache/invalidate`、`GET/PATCH /api/v1/ops/search/ranking-profiles` 的基础实现，但仍停留在 `x-role`、placeholder `step-up`、缺失正式审计 / `audit.access_audit + ops.system_log`、缺失 `SEARCH_*` 错误码、OpenAPI 过薄的状态。本批将切换到 `Authorization: Bearer` + 正式权限点、必要 `X-Idempotency-Key / X-Step-Up-Token`、搜索域错误码、正式审计与 host smoke，并收口 alias / Redis key / OpenSearch authority 到冻结口径。
- 追溯：已按 `CSV > Markdown > technical_reference > 其他辅助文档` 重新核对 `AUD-022`、`商品搜索/排序与索引同步设计`、`商品搜索/排序与索引同步接口协议正式版`、`接口权限校验清单`、`A07`、`A08`、`A12`、`A13`、`search-reindex` / `opensearch-local` runbook、`057/060/070` 与现有 `search-indexer / search module / search openapi / search tests` 实现。已确认正式完成标准是：搜索运维控制面必须真实接入 `Authorization + 权限点 + step-up + 审计 + SEARCH_*`，并且不再使用 `x-role` 占位；别名权威源以 `search.index_alias_binding + product/seller_search_read/write` 为准；Redis 真实承担搜索缓存失效，OpenSearch 真实承担 alias 切换与重建目标。
### BATCH-235（待审批）
- 任务：`AUD-022` 搜索同步状态 / 重建 / 别名切换 / 缓存失效 / 排序配置更新接口
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/search/api/handlers.rs`、`service.rs`、`repo/mod.rs`：把搜索运维控制面从 `x-role` 切换到 `Authorization: Bearer <access_token>`，补齐正式权限点 `portal.search.read / ops.search_*`、`X-Idempotency-Key`、高风险 `X-Step-Up-Token` 绑定、`iam.step_up_challenge` 真实校验、搜索域 `SEARCH_*` 错误码，以及 `audit.audit_event + audit.access_audit + ops.system_log` 的正式留痕；同时新增 `get_alias_binding_id` 并把 alias 权威源收口到 `search.index_alias_binding + product/seller_search_read/write`。
  - `apps/platform-core/src/modules/search/tests/mod.rs`、`search_api_db.rs`：路由测试改用真实 bearer token 负例；新增 `search_api_and_ops_db_smoke`，真实串起 PostgreSQL、Redis 与 OpenSearch，覆盖目录搜索 `cache_hit=false -> true`、缓存失效、`search.index_sync_task` 排队、`GET /sync`、`GET/PATCH /ranking-profiles`、`POST /aliases/switch` 及其审计回查。
  - `packages/openapi/search.yaml`、`docs/02-openapi/search.yaml`、`packages/openapi/README.md`、`docs/02-openapi/README.md`：把 `Search / Ops Search` 从占位升级成正式契约，补齐 Bearer 鉴权、`X-Idempotency-Key`、必要 `X-Step-Up-Token`、请求/响应 schema 与 `SEARCH_*` 错误码说明。
  - `docs/04-runbooks/search-reindex.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/search-rec-cases.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`：补齐 `AUD-022` 的宿主机联调步骤、step-up 使用口径、Redis / OpenSearch / PostgreSQL / 审计回查与验收矩阵。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：无新增 `V1-gap / V2-reserved / V3-reserved`；把 `TODO-AUD-OPENAPI-001`、`TODO-AUD-TEST-001`、`TODO-SEARCHREC-AUTH-001` 推进到 `AUD-022` 已完成、仅剩 `AUD-023+ / SEARCHREC` 后续缺口的最新状态。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既存 `unused_*` warning，无新增编译失败。
  - `IAM_JWT_PARSER=keycloak_claims cargo test -p platform-core route_tests -- --nocapture` 通过：新增的 Search 路由 Bearer 鉴权 / 权限负例通过，同时确认未回归既有 audit / recommendation 路由测试。
  - `SEARCH_DB_SMOKE=1 IAM_JWT_PARSER=keycloak_claims DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core search_api_and_ops_db_smoke -- --nocapture` 通过：真实完成搜索缓存命中与失效、reindex 排队、sync 查询、ranking profile 更新、alias 切换，以及 `audit.audit_event / audit.access_audit / ops.system_log` 回查。
  - `IAM_JWT_PARSER=keycloak_claims cargo test -p platform-core` 通过：`305 passed; 0 failed`，确认 Search 改造未回归其他模块。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过。
  - 宿主机真实 API smoke 通过：
    - 先以 `set -a; source infra/docker/.env.local; set +a; KAFKA_BROKERS=127.0.0.1:9094 IAM_JWT_PARSER=keycloak_claims APP_PORT=18080 target/debug/platform-core-bin` 启动最新二进制，修正早期因环境变量未导出导致的旧基线误报。
    - 使用宿主机 `curl` + bearer token 真实验证 `buyer_operator` 访问 `GET /api/v1/ops/search/sync` 被 `IAM_UNAUTHORIZED` 拒绝。
    - 使用宿主机 `curl` 真实验证 `GET /api/v1/catalog/search` 两次调用分别返回 `cache_hit=false -> true`，并通过 `redis-cli GET datab:v1:search:catalog:product:<sha256>` 回查缓存键存在。
    - 使用宿主机 `curl POST /api/v1/ops/search/cache/invalidate` 真实失效缓存，并通过 `redis-cli` 回查缓存键已删除。
    - 使用宿主机 `curl POST /api/v1/ops/search/reindex` 先验证缺少 `X-Idempotency-Key` 返回 `SEARCH_QUERY_INVALID`，再在 verified `iam.step_up_challenge` 下验证成功入队；随后通过 `psql` 回查 `search.index_sync_task(sync_status='queued')`。
    - 使用宿主机 `curl GET /api/v1/ops/search/sync`、`GET /api/v1/ops/search/ranking-profiles`、`PATCH /api/v1/ops/search/ranking-profiles/{id}`、`POST /api/v1/ops/search/aliases/switch` 真实完成 OpenSearch alias 切换与 ranking profile 更新，并通过 `psql + curl http://127.0.0.1:9200/_alias/...` 回查 `search.index_alias_binding.active_index_name` 与 alias 目标索引已变化。
    - 使用 `psql` 回查 `audit.audit_event(action_name=search.reindex.queue / search.cache.invalidate / search.ranking_profile.patch / search.alias.switch)`、`audit.access_audit(target_type=search_reindex / search_cache / search_ranking_profile / search_alias_binding)`、`ops.system_log(message_text='search ops ...')`，并确认高风险写操作上的 `step_up_challenge_id` 与请求头一致。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-022`
  - `商品搜索、排序与索引同步设计.md`
  - `商品搜索、排序与索引同步接口协议正式版.md`
  - `接口权限校验清单.md`
  - `A07-搜索同步链路与搜索接口闭环缺口.md`
  - `A08-搜索Alias权威源与阶段边界冲突.md`
  - `A12-配置项与资源命名漂移.md`
  - `A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md`
  - `057_search_sync_architecture.sql`、`060_seed_authz_v1.sql`、`070_seed_role_permissions_v1.sql`
- 覆盖的任务清单条目：`AUD-022`
- 未覆盖项：
  - `AUD-023+` 其余 AUD 高风险控制面
  - `SEARCHREC` 后续 worker 幂等 / DLQ / reprocess 可靠性闭环
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；仅把既有 TODO 的已完成范围推进到 `AUD-022`。
- 备注：
  - 本批没有新增 Go / Fabric 执行面代码；这是因为 `AUD-022` 按冻结口径属于 `platform-core` 的搜索运维控制面。Go 与 Fabric 的真实交互仍保持 `AUD-013 ~ AUD-017` 已落地的 `fabric-adapter / fabric-event-listener / fabric-ca-admin / chaincode` 分层，不回退到 Rust 直接与 Fabric 通信。
  - 本批最初 smoke 失败并非实现口径冲突，而是测试断言把统一错误响应误当成 `error.code` 包装，以及把 `audit.access_audit.step_up_challenge_id` 的可空列按非空字符串读取；修正测试后，正式链路与回查全部通过。
  - 宿主机 smoke 的业务对象、OpenSearch 文档与临时索引已清理；`core.user_account / core.organization` 在尝试删除时会触发 `audit.access_audit` 的 append-only 保护，无法执行 `ON DELETE SET NULL` 更新，因此继续按既有 `AUD` 阶段策略保留最小主体样本，不通过强删破坏审计引用关系。
  - 本批未发现新的 `CSV / Markdown / technical_reference / authz seed / OpenAPI / Go-Rust 分层` 冲突，不触发暂停条件。
### BATCH-236（计划中）
- 任务：`AUD-023` 观测总览 / 日志镜像查询导出 / trace 联查 / 告警与事故工单接口
- 状态：计划中
- 说明：按 `AUD-023` 冻结口径，当前批次在 `apps/platform-core/src/modules/audit/**` 补齐 `GET /api/v1/ops/observability/overview`、日志镜像查询/导出、`GET /api/v1/ops/traces/{traceId}`、`GET /api/v1/ops/alerts`、`GET /api/v1/ops/incidents`，并把 `ops.observability.read / ops.log.query / ops.log.export / ops.trace.read / ops.alert.read / ops.incident.read`、必要 step-up、审计与观测域错误码收口到正式实现。`PostgreSQL` 继续作为 `ops.system_log / ops.trace_index / ops.alert_event / ops.incident_ticket / ops.slo_*` 的主权威，`Prometheus / Alertmanager / Loki / Tempo / Grafana` 必须通过真实健康探测、联查 URL 或结果摘要进入接口返回与 runbook，不允许只读本地占位配置。
- 追溯：已重新核对 `CSV / Markdown`、`日志、可观测性与告警设计.md`、`日志与可观测性接口协议正式版.md`、`页面说明书-V1-完整版.md` 第 21 章、`A04-AUD-Ops-接口与契约落地缺口.md`、`059_logging_observability.sql`、`060/070 authz seed`、`observability-local.md`、`check-observability-stack.sh`、`docker-compose.local.yml` 以及现有 `audit` 模块与 `ops.yaml`。当前确认的正式完成标准是：观测总览要能显示 backend/告警/SLO/incident 摘要；日志查询/导出要走 `ops.system_log + Loki` 边界、导出动作用 step-up + 审计 + MinIO 对象；trace 联查要以 `ops.trace_index` 为正式索引并联到 `Tempo`；告警/工单查询要真实覆盖 `ops.alert_event / ops.incident_ticket` 并保留 `request_id / trace_id / object_id` 联查能力。
### BATCH-236（待审批）
- 任务：`AUD-023` 观测总览 / 日志镜像查询导出 / trace 联查 / 告警与事故工单接口
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/audit/api/{router,handlers}.rs`、`domain/mod.rs`、`dto/mod.rs`、`repo/mod.rs`：补齐 `GET /api/v1/ops/observability/overview`、`GET /api/v1/ops/logs/query`、兼容别名 `GET /api/v1/ops/logs`、`POST /api/v1/ops/logs/export`、`GET /api/v1/ops/traces/{traceId}`、`GET /api/v1/ops/alerts`、`GET /api/v1/ops/incidents`、`GET /api/v1/ops/slos`；以 `ops.system_log / ops.trace_index / ops.alert_event / ops.incident_ticket / ops.slo_* / ops.observability_backend` 为正式读模型，导出动作真实写 `MinIO(report-results)`、`audit.audit_event`、`audit.access_audit` 与 `ops.system_log`，并把 `Prometheus / Alertmanager / Grafana / Loki / Tempo / OTel Collector` 的真实 probe 状态回填到总览接口。
  - `apps/platform-core/crates/http/src/lib.rs`、`Cargo.toml`：为宿主机 `platform-core` 补正式 `/metrics` 端点，暴露 `platform_core_http_requests_total` 与 `platform_core_http_request_duration_seconds`，Prometheus 现在可以真实抓取宿主机 `platform-core:8094/metrics`，不再依赖占位 exporter。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：新增 `AUD-023` 路由权限负例与 `observability_api_db_smoke`，真实串起 PostgreSQL、MinIO、审计留痕、`trace_id / request_id / object_id` 联查，并校验 overview/logs/export/trace/alerts/incidents/slos 全链路。
  - `infra/docker/monitoring/prometheus.yml`、`infra/docker/docker-compose.local.yml`、`infra/mock-payment/mappings/admin-metrics.json`：将 `mock-payment-provider` 的 Prometheus 抓取收口到真实 `/metrics`，补齐 `MINIO_PROMETHEUS_AUTH_TYPE=public`，重建 `Prometheus / MinIO / mock-payment-provider` 后 `platform-core / mock-payment-provider / minio-exporter` 全部转为 `up`。
  - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、`docs/04-runbooks/audit-observability.md`、`docs/04-runbooks/observability-local.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`、`packages/openapi/README.md`、`docs/02-openapi/README.md`：同步补齐正式契约、宿主机联调步骤、Prometheus / MinIO / trace / alert 回查与 `AUD-023` 验收矩阵。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：无新增 `V1-gap / V2-reserved / V3-reserved`；关闭历史观测缺口 `TODO-NOTIF-OBS-001`，并把 `TODO-AUD-OPENAPI-001`、`TODO-AUD-TEST-001` 推进到 `AUD-023` 已完成的最新状态。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo test -p http@0.1.0` 通过；新增 `metrics` path 归一化测试通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既存 `unused_*` warning，无新增编译失败。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core` 通过：`313 passed; 0 failed`。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core observability_api_db_smoke -- --nocapture` 通过：真实回查 `MinIO(report-results)`、`audit.audit_event / audit.access_audit / ops.system_log`。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过。
  - `./scripts/check-observability-stack.sh` 通过：`platform-core / mock-payment-provider / kafka-exporter / postgres-exporter / redis-exporter / minio-exporter / opensearch-exporter` target、规则、datasource、dashboard 全绿。
  - 宿主机真实接口联调通过：
    - `curl http://127.0.0.1:8094/metrics` 可见 `platform_core_http_requests_total`、`platform_core_http_request_duration_seconds`。
    - `curl http://127.0.0.1:8089/metrics` 可见 `mock_payment_provider_up 1`。
    - 使用 `curl` 真实调用 `GET /api/v1/ops/observability/overview`、`GET /api/v1/ops/logs/query`、`GET /api/v1/ops/traces/{traceId}`、`GET /api/v1/ops/alerts`、`GET /api/v1/ops/incidents`、`GET /api/v1/ops/slos`，并通过 `jq` 回查 `backend_keys / related_log_count / related_alert_count / incident / slo` 等正式字段。
    - 手工插入的临时 `trace_index / alert_rule / alert_event / incident_ticket / slo_*` 样本已清理；尝试删除对应 `ops.system_log` 时被 append-only guard 拦截，说明该对象在本地运行基线下也是真正式约束，不做破坏性清理。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-023`
  - `日志、可观测性与告警设计.md`
  - `日志与可观测性接口协议正式版.md`
  - `页面说明书-V1-完整版.md` 第 21 章
  - `A04-AUD-Ops-接口与契约落地缺口.md`
  - `059_logging_observability.sql`
  - `060_seed_authz_v1.sql`、`070_seed_role_permissions_v1.sql`
  - `observability-local.md`、`audit-observability.md`、`check-observability-stack.sh`
- 覆盖的任务清单条目：`AUD-023`
- 未覆盖项：
  - `AUD-024+` 其余 AUD 高风险控制面
  - `SEARCHREC` 后续 worker 幂等 / DLQ / reprocess 可靠性闭环
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；本批同时关闭了历史 `TODO-NOTIF-OBS-001`。
- 备注：
  - 本批没有新增 Go / Fabric 执行面代码；这是因为 `AUD-023` 按冻结口径属于 `platform-core` 的观测控制面，不要求新增 Fabric handler。Go 与 Fabric 的真实交互继续保持 `AUD-013 ~ AUD-017` 已落地的 `fabric-adapter / fabric-event-listener / fabric-ca-admin / chaincode` 分层。
  - `./scripts/check-observability-stack.sh` 最初失败不是文档口径冲突，而是宿主机 `platform-core` 缺少正式 `/metrics`、`mock-payment-provider` 使用了 WireMock 保留的 `__admin` 路径、MinIO 指标抓取仍在旧配置。补齐正式 `/metrics` 与重建容器后已收口。
  - 手工 live 验证阶段，`ops.system_log` 被 append-only guard 拦截删除；该约束与 `audit.*` 一样属于正式留痕保护，本批按真实约束保留相关日志镜像记录，不通过绕过 trigger 的方式伪造“清理成功”。
  - 本批未发现新的 `CSV / Markdown / technical_reference / Prometheus scrape / OpenAPI / Go-Rust 分层` 冲突，不触发暂停条件。
### BATCH-237（计划中）
- 任务：`AUD-024` Developer 状态联查接口
- 状态：计划中
- 说明：按 `AUD-024` 冻结口径，当前批次在 `apps/platform-core/src/modules/audit/**` 补齐 `GET /api/v1/developer/trace`，以 `order_id / event_id / tx_hash` 单 selector 快速定位订单、审计、outbox、dead letter、外部事实、投影缺口、链锚定与日志/trace 索引。实现必须保持 `PostgreSQL` 为正式查询权威源，`Loki / Tempo` 只通过 `ops.system_log / ops.trace_index` 镜像索引参与联查，不新建第二套 developer truth source；权限收口到 `developer.trace.read`，tenant 侧强制 `x-tenant-id + order scope`，读取动作真实写入 `audit.access_audit + ops.system_log`。
- 追溯：已重新核对 `CSV / Markdown`、`全量领域模型与对象关系说明.md` 4.12、`页面说明书-V1-完整版.md` 11.3、`日志、可观测性与告警设计.md` 6 节、`全集成基线-V1.md` 中开发者通道/权限映射/状态联查页口径、`A04-AUD-Ops-接口与契约落地缺口.md`，以及现有 `audit/api|repo|dto|domain`、`ops.trace_index / ops.system_log / ops.outbox_event / ops.dead_letter_event / ops.external_fact_receipt / ops.chain_projection_gap / chain.chain_anchor` 实现。当前确认的正式完成标准是：接口、DTO、权限、调试留痕、错误码、最小测试和 OpenAPI 同步落盘；至少一条真实 API + DB 回查验证通过，能从 `order_id / event_id / tx_hash` 中任一输入定位到正式对象链路。
### BATCH-237（待审批）
- 任务：`AUD-024` Developer 状态联查接口
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/audit/api/{router,handlers}.rs`、`domain/mod.rs`、`dto/mod.rs`、`repo/mod.rs`：补齐 `GET /api/v1/developer/trace`，支持 `order_id / event_id / tx_hash` 单 selector 联查；读取 `trade.order_main / audit.audit_event / ops.outbox_event / ops.dead_letter_event / ops.external_fact_receipt / ops.chain_projection_gap / ops.trade_lifecycle_checkpoint / ops.trace_index / ops.system_log / chain.chain_anchor` 正式对象，返回 `subject / matched_* / recent_*` 视图，并在每次读取后真实写入 `audit.access_audit(target_type='developer_trace_query')` 与 `ops.system_log`。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`：新增 `developer_trace_requires_single_selector`、`rejects_developer_trace_without_permission` 路由负例和 `developer_trace_api_db_smoke`，真实覆盖 `order_id / event_id / tx_hash` 三类 selector、tenant scope、审计留痕与系统日志回查。
  - `packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`、`packages/openapi/README.md`、`docs/02-openapi/README.md`、`scripts/check-openapi-schema.sh`：补齐 `GET /api/v1/developer/trace` 契约、`developer.trace.read`、`DeveloperTraceLookupResponse` schema 与归档校验。
  - `docs/04-runbooks/developer-trace.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`：补齐宿主机 `curl + psql` 联调、`AUD-024` 验收矩阵、TODO 台账推进与 Go/Fabric 外围回写边界说明。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既存 `unused_*` warning，无新增编译失败。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core route_tests -- --nocapture` 通过，`developer trace` 权限与单 selector 负例均为绿色。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core developer_trace_api_db_smoke -- --nocapture` 通过：真实回查 `ops.outbox_event / ops.dead_letter_event / ops.external_fact_receipt / chain.chain_anchor / ops.chain_projection_gap / ops.trade_lifecycle_checkpoint / ops.trace_index / ops.system_log / audit.audit_event / audit.access_audit`。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core` 通过：`316 passed; 0 failed`。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新 `.sqlx` 离线缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-openapi-schema.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-024`
  - `全量领域模型与对象关系说明.md` 4.12
  - `页面说明书-V1-完整版.md` 11.3
  - `日志、可观测性与告警设计.md` 6 节
  - `全集成基线-V1.md` 中开发者通道、权限映射、状态联查页
  - `A04-AUD-Ops-接口与契约落地缺口.md`
- 覆盖的任务清单条目：`AUD-024`
- 未覆盖项：
  - `AUD-025+` 其余 AUD 高风险控制面
  - Go/Fabric 后续新执行动作与更深层回执编排，仍留给后续对应任务
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；仅把 `TODO-AUD-OPENAPI-001` 与 `TODO-AUD-TEST-001` 推进到 `AUD-024` 已完成的最新状态。
- 备注：
  - 本批没有新增 Go / Fabric 执行面代码；`AUD-024` 按冻结口径属于 `platform-core` 的开发者联查控制面。Go 服务 `fabric-adapter / fabric-event-listener / fabric-ca-admin` 继续作为链提交、callback、CA 操作的外围写入层，本接口只读取它们回写到 PostgreSQL 的正式对象。
  - smoke 首次失败不是文档冲突，而是测试断言误把种子订单的 `payment_status='paid'` 写成 `unpaid`；修正断言后，正式链路与 DB 回查全部通过。
  - 当前实现兼容 `tenant_developer / developer_admin / platform_admin / platform_audit_security` 读取角色，以覆盖既有 seed 与全集成文档中的已存在角色漂移；tenant 侧仍强制 `x-tenant-id + buyer/seller order scope`，没有放宽对象边界。
### BATCH-238（计划中）
- 任务：`AUD-025` 审计与 ops 最小权限矩阵收口
- 状态：计划中
- 说明：按最新人工裁决，当前批次不采用“把文档收窄到现有 seed”的收口方式，而采用 `B+ authority 分层收口`。正式 authority 固定为：`060_seed_authz_v1.sql` 作为唯一正式角色集合、`docs/权限设计/角色权限矩阵正式版.md + 菜单权限映射表.md + 接口权限校验清单.md` 作为唯一正式权限分配与页面/接口职责、`070_seed_role_permissions_v1.sql` 作为可执行镜像、代码/runbook/test 作为派生层不得反向定义角色。当前实现目标是：补齐 `070` 与本地 seed 对文档矩阵的缺口，清理 `audit/api/handlers.rs` 中旧别名角色硬编码，把鉴权逻辑收口成“正式权限点 + 正式核心角色 + 受控兼容别名”，并同步把 runbook / test-case / 任务文档中的 `developer_admin / audit_admin / node_ops_admin / consistency_operator / platform_auditor` 等旧示例切回 `060` 的 12 个正式核心角色。
- 追溯：已重新核对 `CSV / Markdown`、`A03-统一事务模板-落地真实审计与Outbox-Writer.md`、`A04-AUD-Ops-接口与契约落地缺口.md`、`060_seed_authz_v1.sql`、`070_seed_role_permissions_v1.sql`、`角色权限矩阵正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`audit/api/handlers.rs`、`search/service.rs`、`developer-trace.md`、`audit-consistency-reconcile.md`、`audit-trade-monitor.md`、`audit-consistency-cases.md`。当前确认的正式完成标准是：最小权限矩阵在 seed / 本地 DB / 代码 / runbook / test-case / 任务文档六层一致；正式角色仍只保留 `060` 的 12 个核心角色；`developer.trace.read`、`ops.consistency.*`、`ops.outbox.read`、`ops.dead_letter.*`、`audit.anchor.*`、`ops.search_sync.read`、`ops.search_cache.invalidate` 等权限点按冻结文档分层收口；至少一条真实 API/DB 联调证明权限与 step-up 口径已按正式矩阵生效。
### BATCH-238（待审批）
- 任务：`AUD-025` 审计与 ops 最小权限矩阵收口
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/audit/api/{handlers,mod}.rs`、`tests/{mod,api_db}.rs`：把审计与 ops 控制面的授权逻辑收口到 `AuditPermission` 权限点，新增 `permission_code / canonical_role_key / is_allowed`，正式矩阵只认 `060_seed_authz_v1.sql` 的核心角色；`developer_admin / audit_admin / node_ops_admin / consistency_operator / platform_auditor` 仅保留为受控兼容别名，并在读取 `x-role` 时立即归一化，避免继续把旧岗位别名当成正式 authority。
  - `apps/platform-core/src/modules/search/{service.rs,tests/mod.rs}`：补齐 `ops.search_sync.read`、`ops.search_cache.invalidate` 对 `platform_audit_security` 的正式授权，并新增矩阵单测，确保搜索运维控制面与冻结权限文档一致。
  - `docs/数据库设计/V1/upgrade/070_seed_role_permissions_v1.sql`：把 `developer.trace.read`、`ops.consistency.read`、`ops.consistency.reconcile`、`ops.outbox.read`、`ops.dead_letter.read`、`ops.dead_letter.reprocess`、`audit.anchor.read`、`audit.anchor.manage`、`ops.search_sync.read`、`ops.search_cache.invalidate` 补齐到与权限设计文档一致的可执行 seed；本地 PostgreSQL 已重新执行该 seed 并做只读 SQL 回查。
  - `docs/权限设计/菜单权限映射表.md`、`docs/04-runbooks/{developer-trace,audit-consistency-reconcile,audit-observability,audit-trade-monitor,search-reindex}.md`、`docs/05-test-cases/audit-consistency-cases.md`：清理 `developer_admin / audit_admin / node_ops_admin / consistency_operator / platform_auditor` 等旧示例，统一成正式核心角色；同时把搜索运维页与 `developer.trace.read` 相关菜单描述修回正式矩阵口径，不再让 authority 文档内部继续分叉。
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：为 `AUD-025` 补充 authority 分层约束，明确 `060` 是正式角色集合、权限设计文档是正式分配 authority、`070` 是可执行镜像，代码 / runbook / test 不得再自创第三套角色口径。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既有 `unused_*` warning，无新增编译错误。
  - `cargo test -p platform-core modules::audit::tests -- --nocapture` 通过，新增 `audit_permission_matrix_matches_core_roles_and_distinct_points` 与 `legacy_audit_role_aliases_only_survive_as_compatibility_mapping`，并覆盖 `audit_consistency_reconcile_db_smoke`、`audit_dead_letter_reprocess_db_smoke`、`developer_trace_api_db_smoke` 等真实 AUD 控制面 DB smoke。
  - `cargo test -p platform-core modules::search::tests -- --nocapture` 通过，`search_api_and_ops_db_smoke` 与 `search_ops_matrix_matches_formal_roles` 证明搜索运维控制面已按正式矩阵放行 `platform_audit_security` 的同步状态/缓存失效读写边界。
  - `cargo test -p platform-core` 通过：`319 passed; 0 failed; 1 ignored`。
  - `DATABASE_URL=postgresql://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新工作区 `.sqlx` 离线缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `psql 'postgresql://datab:datab_local_pass@127.0.0.1:5432/datab' -f docs/数据库设计/V1/upgrade/070_seed_role_permissions_v1.sql` 已执行，通过把本地 `authz.role_permission` 种子补齐到最新 authority。
  - `psql` 只读回查通过：`developer.trace.read -> platform_audit_security,tenant_developer`，`ops.consistency.read / ops.consistency.reconcile / ops.outbox.read / ops.dead_letter.read / ops.dead_letter.reprocess / audit.anchor.read / audit.anchor.manage / ops.search_sync.read / ops.search_cache.invalidate -> platform_admin,platform_audit_security`，与冻结权限矩阵一致。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-025`
  - `060_seed_authz_v1.sql`
  - `070_seed_role_permissions_v1.sql`
  - `角色权限矩阵正式版.md`
  - `菜单权限映射表.md`
  - `接口权限校验清单.md`
  - `A03-统一事务模板-落地真实审计与Outbox-Writer.md`
  - `A04-AUD-Ops-接口与契约落地缺口.md`
- 覆盖的任务清单条目：`AUD-025`
- 未覆盖项：
  - `AUD-026+` 的剩余高风险控制面与后续动作
  - 非 AUD 模块里仍存在的历史别名角色痕迹，留给对应领域任务按其冻结口径继续收口
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
- 备注：
  - 本批没有新增 Go / Fabric 执行面代码；这是因为 `AUD-025` 的冻结目标是审计与 ops 控制面的正式权限矩阵收口，不是新增链交互服务。Go 与 Fabric 的真实交互继续保持 `AUD-013 ~ AUD-017` 已落地的 `fabric-adapter / fabric-event-listener / fabric-ca-admin / chaincode` 分层。
  - 旧别名角色没有被恢复为正式角色，只降级为 `audit/api/handlers.rs` 中的过渡兼容映射，用于消化历史 header/文档漂移；正式 bearer/seed/runbook/test 已全部切回 `060` 的 12 个核心角色。
  - `cargo sqlx prepare --workspace` 与本地 `psql` 回查需要宿主机数据库访问，沙箱内被 `Operation not permitted` 拦截；切到宿主机执行后通过，属于环境权限差异，不是冻结文档冲突。
### BATCH-239（计划中）
- 任务：`AUD-026` 审计 / 一致性 / Fabric / Ops 集成测试闭环
- 状态：计划中
- 说明：按用户要求先对 `AUD-026` 做完整复核，不把已有代码、README、runbook 或旧 smoke 直接视为已完成。复核结果已确认当前仓库还不能签收 `AUD-026`：`outbox-publisher` 的 live smoke 已在 `AUD-009` 落地，`audit package export / replay dry-run / Fabric receipt write-back` 也已有各自 smoke，但 `SEARCHREC` consumer 闭环仍未完成，其中 `workers/search-indexer` 还在失败后无条件 `commit_message`，也没有 `ops.consumer_idempotency_record + ops.dead_letter_event + dtp.dead-letter` 双层隔离；`workers/recommendation-aggregator` 只有部分幂等 smoke，没有正式双层 DLQ、失败隔离和 offset 提交策略验证；相应 runbook / test-case 仍把这些项标记为“后续任务”。本批将据此补齐正式 worker 可靠性实现与集成测试，并把文档更新到“已落地可验证”状态。
- 追溯：已重新核对 `CSV / Markdown`、`审计、证据链与回放设计.md` 第 3 节、`审计、证据链与回放接口协议正式版.md` 第 5 节、`全量领域模型与对象关系说明.md` 4.9、`事件模型与Topic清单正式版.md`、`kafka-topics.md`、`async-chain-write.md`、`A04-AUD-Ops-接口与契约落地缺口.md`、`A05-Outbox-Publisher-DLQ-统一闭环缺口.md`、`A11-测试与Smoke口径误报风险.md`、`A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md`，以及现有 `workers/search-indexer`、`workers/recommendation-aggregator`、`workers/outbox-publisher`、`services/fabric-adapter`、`docs/04-runbooks/search-reindex.md`、`docs/04-runbooks/recommendation-runtime.md`、`docs/05-test-cases/search-rec-cases.md` 与 `audit-consistency-cases.md`。当前确认的正式完成标准是：至少覆盖 SEARCHREC consumer 的正式 `event_id` 幂等、失败隔离、DB/Kafka 双层 DLQ、worker 侧副作用和 dry-run reprocess 联查；并能用真实 smoke / DB / Kafka / Redis / OpenSearch 回查证明 outbox publish、consumer 幂等、receipt write-back、审计包导出和 dry-run replay 口径一致，不再依赖“只看 outbox 行存在”或“只看手工 seed OpenSearch”式误报验证。
### BATCH-239（待审批）
- 任务：`AUD-026` 审计 / 一致性 / Fabric / Ops 集成测试闭环
- 状态：待审批
- 实现摘要：
  - `workers/search-indexer/src/main.rs`：补齐正式 consumer 可靠性闭环。`search-indexer` 现在基于统一 envelope `event_id` 写入 `ops.consumer_idempotency_record`，只有在成功处理或失败已安全落入 `ops.dead_letter_event + dtp.dead-letter` 双层隔离后才提交 offset；失败时同步回写 `search.index_sync_task(sync_status='failed')`，成功时真实写 OpenSearch 并失效 Redis 搜索缓存。补充 `search_indexer_db_smoke`，真实验证副作用、重复投递去重、双层 DLQ、Kafka 隔离消息与缓存删除。
  - `workers/recommendation-aggregator/src/main.rs`：把现有“部分幂等”收口成正式 consumer 闭环。先通过 `ops.consumer_idempotency_record` 做 `event_id` 幂等门禁，再执行热度聚合、关系边更新、`search.index_sync_task` 回流与推荐缓存失效；失败时统一写 `ops.dead_letter_event + dtp.dead-letter`，并把幂等状态更新为 `dead_lettered`。补充 `recommendation_aggregator_db_smoke`，真实验证推荐行为副作用、重复投递、双层 DLQ 与失败隔离。
  - `docs/04-runbooks/{search-reindex,recommendation-runtime}.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/{search-rec-cases,README.md}`：把仍写成“后续任务”的 SEARCHREC consumer 可靠性口径改成已落地可验证，补齐宿主机回归命令、双层 DLQ / `dry_run reprocess` 回查项，并把 runbook 索引同步到 `recommendation-runtime.md`。
  - `services/fabric-adapter/internal/service/processor.go`、`docs/开发任务/V1-Core-TODO与预留清单.md`：修正 Fabric reliability 遗留注释与 TODO 台账，不再错误宣称 `AUD-026` 会清空 Fabric consumer 自身的 idempotency / Redis 短锁缺口；本批仅重新复验 `fabric-adapter` Go 单测、`check-fabric-local.sh` 与 `fabric-adapter-live-smoke.sh`，保持该 gap 继续留给后续专门批次。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既有 `unused_*` warning，无新增编译错误。
  - `cargo test -p platform-core` 通过：`319 passed; 0 failed; 1 ignored`。
  - `cargo check -p search-indexer`、`cargo test -p search-indexer` 通过；`SEARCHREC_WORKER_DB_SMOKE=1 ... cargo test -p search-indexer search_indexer_db_smoke -- --nocapture` 通过，真实回查 OpenSearch 文档、Redis 缓存失效、`ops.consumer_idempotency_record(result_code='processed'/'dead_lettered')`、`ops.dead_letter_event(target_topic='dtp.search.sync')` 与 Kafka `dtp.dead-letter`。
  - `cargo check -p recommendation-aggregator`、`cargo test -p recommendation-aggregator` 通过；`SEARCHREC_WORKER_DB_SMOKE=1 ... cargo test -p recommendation-aggregator recommendation_aggregator_db_smoke -- --nocapture` 通过，真实回查 `search.search_signal_aggregate`、`recommend.entity_similarity`、`recommend.bundle_relation`、`search.index_sync_task`、Redis 推荐缓存、`ops.consumer_idempotency_record`、`ops.dead_letter_event(target_topic='dtp.recommend.behavior')` 与 Kafka `dtp.dead-letter`。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p outbox-publisher outbox_publisher_db_smoke -- --nocapture` 通过，证明 canonical outbox publish / retry / DB+Kafka 双层 DLQ 仍保持正式闭环。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture` 通过，真实覆盖 `search-indexer` 与 `recommendation-aggregator` 的 SEARCHREC dead letter `dry_run` 重处理预演、`step-up` 绑定、`audit.audit_event`、`audit.access_audit` 与 `ops.system_log`。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_trace_api_db_smoke -- --nocapture` 通过，重新验证审计包导出、MinIO 对象回查与 replay dry-run 报告路径。
  - `go test ./...`、`go build ./...` 在 `services/fabric-adapter` 与 `services/fabric-event-listener` 下通过；`./scripts/check-fabric-local.sh` 与 `./scripts/fabric-adapter-live-smoke.sh` 通过，真实验证 Fabric test-network、Gateway 提交、账本回查以及 `ops.external_fact_receipt / chain.chain_anchor` 回写。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新工作区 `.sqlx` 离线缓存。
  - `./scripts/check-query-compile.sh` 通过；首次并行误报是 `check-query-compile.sh` 在 `.sqlx` 刷新前抢先读取旧 cache，串行重跑后已恢复绿色。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-026`
  - `审计、证据链与回放设计.md` 第 3 节
  - `审计、证据链与回放接口协议正式版.md` 第 5 节
  - `全量领域模型与对象关系说明.md` 4.9
  - `事件模型与Topic清单正式版.md`
  - `kafka-topics.md`
  - `async-chain-write.md`
  - `A04-AUD-Ops-接口与契约落地缺口.md`
  - `A05-Outbox-Publisher-DLQ-统一闭环缺口.md`
  - `A11-测试与Smoke口径误报风险.md`
  - `A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md`
- 覆盖的任务清单条目：`AUD-026`
- 未覆盖项：
  - Fabric consumer 自身的 `ops.consumer_idempotency_record + Redis` 短锁闭环仍保留为 `TODO-AUD-FABRIC-001`，留待后续专门批次收口；本批只复验当前 `fabric-adapter` 的 live smoke 与回执写回链路。
  - `SEARCHREC-015 / SEARCHREC-017 / SEARCHREC-020` 仍会在对应阶段继续补齐更宽的统一鉴权 / step-up / OpenAPI / test-case 矩阵；本批不越级改写其主任务边界。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；仅更新 `TODO-AUD-FABRIC-001` 的原因与补齐条件，避免继续把 Fabric consumer reliability 错绑到 `AUD-026`。
- 备注：
  - 本批开始时 `./scripts/check-fabric-local.sh` 和 `fabric-adapter-live-smoke.sh` 失败，根因是本地 Fabric test-network 尚未启动，`fabric-adapter` 的 Gateway 目标 `127.0.0.1:7051` 不可达。执行 `make up-fabric` 后，`check-fabric-local.sh` 与 live smoke 恢复通过，属于环境基线缺失，不是冻结文档冲突。
  - 早期失败的 `search-indexer` smoke 使用了随机 `event_id`，没有对应正式 `ops.outbox_event`，因此触发 `search.index_sync_task.source_event_id` 外键失败；已将 smoke 修正为真实 outbox event 语义，并清理该次失败残留的业务测试数据。`audit.audit_event` / `audit.access_audit` 等 append-only 审计记录按规则保留未清理。
### BATCH-240（计划中）
- 任务：`AUD-027` 生成 `docs/02-openapi/audit.yaml`、`ops.yaml` 第一版并与实现校验
- 状态：计划中
- 说明：重新核对 `AUD-027` 的冻结要求后确认，本仓库当前已经存在 `packages/openapi/{audit,ops}.yaml` 与 `docs/02-openapi/{audit,ops}.yaml`，但这本身不能视为任务完成；必须进一步证明两份归档副本与当前 `platform-core.audit` 路由、当前已实现的 AUD/Ops 控制面路径、正式术语命名和 README 索引已对齐，且 `check-openapi-schema.sh` 不再只校验局部路径。本批将据此补强 OpenAPI 契约校验脚本、同步 README/索引文字，并用真实脚本校验和至少一条已落地 AUD smoke 证明“OpenAPI 文件已落盘且与实现同口径”。
- 追溯：已重新核对 `CSV / Markdown`、`审计、证据链与回放设计.md` 第 3 节、`审计、证据链与回放接口协议正式版.md` 第 5 节、`全量领域模型与对象关系说明.md` 4.9、`A04-AUD-Ops-接口与契约落地缺口.md`，以及通用冻结文档 `服务清单与服务边界正式版.md`、`事件模型与Topic清单正式版.md`、`本地开发环境与中间件部署清单.md`、`配置项与密钥管理清单.md`、`技术选型正式版.md`、`平台总体架构设计草案.md`、`数据交易平台-全集成基线-V1.md`、`fabric-local.md`、`kafka-topics.md`、`async-chain-write.md`、`072_canonical_outbox_route_policy.sql`、`074_event_topology_route_extensions.sql` 与 `infra/docker/docker-compose.local.yml`。当前确认的正式完成标准是：`docs/02-openapi/audit.yaml`、`docs/02-openapi/ops.yaml` 与 `packages/openapi` 同步、被 README 索引引用、覆盖当前 `apps/platform-core/src/modules/audit/api/router.rs` 已落地路径，并通过 `./scripts/check-openapi-schema.sh` 与至少一条 AUD API smoke 共同证明契约没有漂移。
### BATCH-240（待审批）
- 任务：`AUD-027` 生成 `docs/02-openapi/audit.yaml`、`ops.yaml` 第一版并与实现校验
- 状态：待审批
- 实现摘要：
  - `scripts/check-openapi-schema.sh`：补强 `audit.yaml / ops.yaml` 的 drift guard，不再只校验局部骨架路径。当前脚本已覆盖 `apps/platform-core/src/modules/audit/api/router.rs` 中全部已落地 audit / ops / developer 路径，包括 anchor batch、dead letter reprocess、external facts、fairness incidents、projection gaps、consistency lookup / reconcile、trade monitor、observability、logs alias / export、trace、alerts、incidents、slos，并额外校验关键 response schema、权限点和术语 token，继续强制 `packages/openapi/{audit,ops}.yaml` 与 `docs/02-openapi/{audit,ops}.yaml` 逐字同步。
  - `docs/02-openapi/README.md`、`packages/openapi/README.md`：把 `audit.yaml` / `ops.yaml` 的索引说明更新到 `AUD-027` 正式口径，明确这两份文件已经进入“归档副本 + 实现对齐校验”状态，不再只是“未来待补”；同时补充 `AUD-027` 之后的三条硬约束：包内文件与归档副本必须逐字同步、必须被 README 索引引用、必须通过 `./scripts/check-openapi-schema.sh` 校验当前已落地路径与关键术语。
  - `docs/02-openapi/{audit,ops}.yaml` 与 `packages/openapi/{audit,ops}.yaml`：本批复核确认两组文件当前已逐字同步，且路径覆盖与当前 `platform-core.audit` 路由一致，因此未额外改写 YAML 内容，只通过更严格的脚本校验和 README 索引把“文件存在”提升为“正式校验绑定”。
- 验证：
  - `./scripts/check-openapi-schema.sh` 通过，确认 `packages/openapi/{audit,ops}.yaml` 的 OpenAPI 头、路径、关键术语和 `docs/02-openapi/{audit,ops}.yaml` 归档副本同步状态均符合当前实现期要求。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture` 通过，重新验证 `audit.package.export`、MinIO 导出对象回查、`audit.replay-jobs` dry-run、`audit.replay_result`、`audit.access_audit` 与 `ops.system_log`，证明 `audit.yaml` 所述主控制面路径和当前实现一致。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture` 通过，重新验证 `POST /api/v1/ops/dead-letters/{id}/reprocess` 的 dry-run 预演、`step-up` 绑定、`audit.audit_event`、`audit.access_audit` 与 `ops.system_log`，证明 `ops.yaml` 对高风险 ops 控制面的归档和当前实现一致。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-027`
  - `审计、证据链与回放设计.md` 第 3 节
  - `审计、证据链与回放接口协议正式版.md` 第 5 节
  - `全量领域模型与对象关系说明.md` 4.9
  - `A04-AUD-Ops-接口与契约落地缺口.md`
- 覆盖的任务清单条目：`AUD-027`
- 未覆盖项：
  - 无。当前任务目标是 `docs/02-openapi/audit.yaml`、`ops.yaml` 第一版归档与实现对齐校验；搜索运维控制面仍由 `search.yaml` 正式承接，不属于本任务范围。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
- 备注：
  - 本批没有重写 `audit.yaml / ops.yaml` 的大段内容，因为复核结果显示两份 YAML 与当前 `platform-core.audit` 路由已一致；真实缺口在于校验脚本和 README 索引仍停留在局部路径 / “未来待补”口径。已通过脚本与索引同步把该 gap 收口。
### BATCH-241（计划中）
- 任务：`AUD-028` 生成 `docs/05-test-cases/audit-consistency-cases.md`，覆盖链下成功链上失败、链上成功链下未更新、回调乱序、重复事件、修复演练
- 状态：计划中
- 说明：重新核对 `AUD-028` 的冻结要求后确认，`docs/05-test-cases/audit-consistency-cases.md` 当前已经收录大批 `AUD-003~AUD-026` 控制面验收项，但还没有把 `AUD-028` 要求的五类一致性场景显式收口成正式矩阵。仅凭“文件很长、已有用例”不能视为完成；必须进一步把五类场景绑定到正式对象、正式接口、现有 runbook 和可重复验证入口，并同步更新 `docs/05-test-cases/README.md` 的索引说明，避免后续 Agent 继续把 callback 晚到、链失败、重复事件和修复演练当成隐含知识处理。
- 追溯：已重新核对 `CSV / Markdown`、`审计、证据链与回放设计.md` 第 3 节、`审计、证据链与回放接口协议正式版.md` 第 5 节、`全量领域模型与对象关系说明.md` 4.9、`A04-AUD-Ops-接口与契约落地缺口.md`，以及通用冻结文档 `服务清单与服务边界正式版.md`、`事件模型与Topic清单正式版.md`、`本地开发环境与中间件部署清单.md`、`配置项与密钥管理清单.md`、`技术选型正式版.md`、`平台总体架构设计草案.md`、`数据交易平台-全集成基线-V1.md`、`审计、证据链与回放设计.md`、`双层权威模型与链上链下一致性设计.md`、`链上链下技术架构与能力边界稿.md`、`日志、可观测性与告警设计.md`、`审计、证据链与回放接口协议正式版.md`、`一致性与事件接口协议正式版.md`、`fabric-local.md`、`kafka-topics.md`、`async-chain-write.md`、`072_canonical_outbox_route_policy.sql`、`074_event_topology_route_extensions.sql` 与 `infra/docker/docker-compose.local.yml`。当前确认的正式完成标准是：`audit-consistency-cases.md` 与 README 明确覆盖五类一致性场景，使用正式术语和正式对象，不再把普通日志、README 叙述或旁路 topic 当作验收证明；并至少用一条真实 smoke 证明这些场景中的审计/日志留痕链路可回查。
### BATCH-241（待审批）
- 任务：`AUD-028` 生成 `docs/05-test-cases/audit-consistency-cases.md`，覆盖链下成功链上失败、链上成功链下未更新、回调乱序、重复事件、修复演练
- 状态：待审批
- 实现摘要：
  - `docs/05-test-cases/audit-consistency-cases.md`：在既有 `AUD-003~AUD-026` 验收矩阵之前新增 `AUD-028` 五类一致性场景映射表，把“链下成功链上失败、链上成功链下未更新、回调乱序 / 晚到、重复事件、修复演练”分别绑定到正式对象、正式接口、现有 case ID、runbook 与最小验证入口，避免后续再把这些场景当成散落在长文里的隐含口径。
  - 场景映射表明确冻结：`ops.external_fact_receipt / chain.chain_anchor / audit.anchor_batch / ops.chain_projection_gap / ops.dead_letter_event / ops.consumer_idempotency_record / audit.audit_event / audit.access_audit / ops.system_log` 才是 `AUD-028` 的正式回查对象；`Fabric` 只负责提交 / 回执 / 摘要证明，不得反向覆盖链下主事实。
  - `docs/05-test-cases/README.md`：把 `audit-consistency-cases.md` 的索引说明更新到 `AUD-028` 口径，明确该文件已经正式承接五类一致性场景，后续剩余高风险控制面从 `AUD-029+` 起继续追加，不再用 `AUD-027+` 的旧说法。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既有 warning，无新增编译错误。
  - `cargo test -p platform-core` 通过：`319 passed; 0 failed`，其中 `audit_consistency_reconcile_db_smoke`、`audit_projection_gap_resolve_db_smoke`、`audit_dead_letter_reprocess_db_smoke`、`audit_trade_monitor_db_smoke` 与 `audit_trace_api_db_smoke` 保持绿色。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过，并刷新工作区 `.sqlx` 缓存。
  - `./scripts/check-query-compile.sh` 通过。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_consistency_reconcile_db_smoke -- --nocapture` 通过，真实验证 `ops.consistency.reconcile.dry_run`、`audit.audit_event`、`audit.access_audit`、`ops.system_log` 与“无 `dtp.consistency.reconcile` outbox 副作用”。
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_projection_gap_resolve_db_smoke -- --nocapture` 通过，真实验证 `ops.chain_projection_gap` 的 query + dry-run + execute resolve、`audit.audit_event(action_name='ops.projection_gap.resolve')`、`audit.access_audit`、`ops.system_log` 与“链上成功链下未更新 / 回调晚到后修复”的正式回查对象。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-028`
  - `审计、证据链与回放设计.md` 第 3 节
  - `审计、证据链与回放接口协议正式版.md` 第 5 节
  - `全量领域模型与对象关系说明.md` 4.9
  - `A04-AUD-Ops-接口与契约落地缺口.md`
- 覆盖的任务清单条目：`AUD-028`
- 未覆盖项：
  - 无新增未覆盖项；本任务目标是把五类一致性场景显式收口到正式 test-case 与 README 索引，不额外扩展新的 API 或 runbook 旁路。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
- 备注：
  - 本批没有改动业务代码；复核结果显示现有 smoke 与 runbook 已能支撑 `AUD-028`，真实缺口在于 test-case 文档尚未把五类一致性场景显式绑定到正式对象与验证入口。已通过场景映射表和 README 索引把该 gap 收口。
### BATCH-242（计划中）
- 任务：`AUD-029` 收敛已完成阶段的历史模块到统一 `audit writer / evidence writer`
- 状态：计划中
- 说明：重新核对 `AUD-029` 的冻结要求后确认，当前主要分叉点集中在三类位置：`catalog` 仍通过 `modules/catalog/api/support.rs` 手写 `INSERT audit.audit_event`；`billing` 仍通过 `modules/billing/db.rs` 手写第二套审计 helper；`billing dispute` 虽已把证据对象桥接到 `audit.evidence_item / audit.evidence_manifest`，但仍需确认其 bridge 继续作为正式权威入口，而不是把 `support.evidence_object` 误当并行真相源。同时运行时 `platform-core` 仍默认注入 `NoopAuditWriter / NoopOutboxWriter`，与 `A03` 的冻结口径冲突。本批将优先抽出统一审计写入入口，改造 `catalog / billing` helper 接入该入口，保留 `support.evidence_object -> audit.evidence_item / evidence_manifest` 的正式桥接，并清理正式运行时里无意义的 no-op 默认注入。
- 追溯：已重新核对 `CSV / Markdown`、`服务清单与服务边界正式版.md` 5.3.15、`审计、证据链与回放设计.md` 第 5 节、`055_audit_hardening.sql` 的 `audit.evidence_*` 表、`A03-统一事务模板-落地真实审计与Outbox-Writer.md`、`A06-Audit-Kit-统一模型漂移.md`、`A14-AUD-历史模块统一Writer与证据桥接缺口.md`，以及通用冻结文档 `事件模型与Topic清单正式版.md`、`本地开发环境与中间件部署清单.md`、`配置项与密钥管理清单.md`、`技术选型正式版.md`、`平台总体架构设计草案.md`、`数据交易平台-全集成基线-V1.md`、`审计、证据链与回放设计.md`、`双层权威模型与链上链下一致性设计.md`、`链上链下技术架构与能力边界稿.md`、`日志、可观测性与告警设计.md`、`审计、证据链与回放接口协议正式版.md`、`一致性与事件接口协议正式版.md`、`fabric-local.md`、`kafka-topics.md`、`async-chain-write.md`、`072_canonical_outbox_route_policy.sql`、`074_event_topology_route_extensions.sql` 与 `infra/docker/docker-compose.local.yml`。当前确认的正式完成标准是：`catalog / search / billing dispute` 产生的审计与证据对象能够通过统一 `AuditEvent / EvidenceItem / EvidenceManifest` 联查；不再新增 ad-hoc `INSERT audit.audit_event` 或第二套证据权威表；至少一条真实集成测试能证明旧模块对象不会再游离在审计 authority model 之外。
### BATCH-242（待审批）
- 任务：`AUD-029` 收敛已完成阶段的历史模块到统一 `audit writer / evidence writer`
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/audit/application/mod.rs`：新增统一 `AuditWriteCommand`、`build_audit_event(...)` 与 `write_audit_event(...)`，把 `AuditEvent` 的默认 schema/version、anchor policy、retention class、legal hold、sensitivity 与 metadata 包装逻辑收口到正式 `audit.application` 层，再由 `modules::audit::repo::insert_audit_event(...)` 落正式 `audit.audit_event`。
  - `apps/platform-core/src/modules/catalog/api/support.rs`、`apps/platform-core/src/modules/billing/db.rs`、`apps/platform-core/src/modules/iam/api.rs`：移除各自手写的 `INSERT audit.audit_event` helper，统一改为调用 `modules::audit::application::write_audit_event(...)`。其中 `catalog` / `billing` / `iam` 仍保留自身 `actor_role`、`event_id`、`step_up_challenge_id`、`certificate_id`、`external_fact_receipt_id` 等业务 metadata，但不再派生第二套审计写入语义。
  - `apps/platform-core/src/lib.rs`：删除正式运行时默认注入的 `NoopAuditWriter / NoopOutboxWriter`，避免生产路径继续静默落到 no-op writer；`db` crate 中的 no-op 仅保留在 rollback fixture 与测试边界。
  - `apps/platform-core/src/modules/audit/tests/mod.rs`：新增 `application_writer_builds_unified_role_and_user_events` 单测，确认统一 writer 在 role / user 两种 actor 下都能保留 `actor_type`、`actor_id`、`step_up_challenge_id`、`sensitivity_level` 与 metadata 字段。
  - `docs/04-runbooks/audit-authority-writer.md`、`docs/04-runbooks/README.md`：补充 `AUD-029` 的正式 authority runbook，明确 `catalog / search / billing` 的审计落点，以及 `support.evidence_object -> audit.evidence_item / audit.evidence_manifest / audit.evidence_manifest_item` 的兼容桥接边界与最小验证入口。
- 验证：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅剩仓库既有 warning，无新增编译错误。
  - `cargo test -p platform-core` 通过：`320 passed; 0 failed`，新增的 `application_writer_builds_unified_role_and_user_events` 绿色；`iam_fabric_ca_admin_db_smoke`、`search_api_and_ops_db_smoke`、`bil013_dispute_case_db_smoke`、`cat024_catalog_listing_review_end_to_end_db_smoke` 等既有链路未回归。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace` 通过。
  - `./scripts/check-query-compile.sh` 通过。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core cat024_catalog_listing_review_end_to_end_db_smoke -- --nocapture` 通过，证明 `catalog` 主链仍可完成端到端评审流程。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core search_api_and_ops_db_smoke -- --nocapture` 通过；随后用 `psql` 回查 `audit.audit_event`，确认 `search.alias.switch`、`search.ranking_profile.patch`、`search.reindex.queue`、`search.cache.invalidate` 等事件持续落在正式 `audit.audit_event`，未退化成旁路日志。
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil013_dispute_case_db_smoke -- --nocapture` 通过；该 smoke 内部已真实回查 `support.evidence_object.metadata.audit_evidence_item_id / audit_evidence_manifest_id`、`audit.evidence_item.metadata.legacy_bridge.legacy_table='support.evidence_object'` 与 `audit.evidence_manifest`，然后按规则清理临时业务测试数据，因此运行后库中不保留临时证据对象。
  - `rg -n "INSERT INTO audit\\.audit_event" apps/platform-core/src/modules/{catalog,billing,iam,audit,search}` 复核通过：除了 `modules/audit/repo/mod.rs` 正式仓储和其测试断言外，`catalog / billing / iam / search` 已不再手写审计 SQL。
  - `rg -n "NoopAuditWriter|NoopOutboxWriter" apps/platform-core/src/lib.rs apps/platform-core/src/modules apps/platform-core/crates/db/src` 复核通过：正式运行时入口已无 no-op 默认注入，仅测试夹具保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-029`
  - `服务清单与服务边界正式版.md` 5.3.15
  - `审计、证据链与回放设计.md` 第 5 节
  - `055_audit_hardening.sql`
  - `A03-统一事务模板-落地真实审计与Outbox-Writer.md`
  - `A06-Audit-Kit-统一模型漂移.md`
  - `A14-AUD-历史模块统一Writer与证据桥接缺口.md`
- 覆盖的任务清单条目：`AUD-029`
- 未覆盖项：
  - `order / delivery` 模块仍存在历史 `INSERT audit.audit_event` 路径，但它们不在 `AUD-029` 的交付范围内；本批未越级改写后续主线模块。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
- 备注：
  - `search` 目前仍直接构造 `AuditEvent::business + insert_audit_event(...)`，这是正式 authority model 的既有正确实现，因此本批未强行改造成另一层 wrapper。
  - `support.evidence_object` 仍保留为历史兼容表，但本批已通过 runbook 与 smoke 复核明确：它只承载 legacy bridge，不再定义正式导出 / replay / legal hold 的主查询口径。
### BATCH-243（计划中）
- 任务：`AUD-030` 收敛统一事件 envelope 与路由权威源
- 状态：计划中
- 说明：重新核对 `AUD-030` 的冻结要求后确认，当前仓库主链实现已基本切到 `apps/platform-core/src/shared/outbox.rs` 的 canonical outbox writer，并由 `ops.event_route_policy` 解析 route，数据库也已不存在挂到 `common.tg_write_outbox()` 的正式 trigger；但现有验收证据仍偏分散，尚未把“同一业务动作不会重复写出不同协议事件”“正式 envelope 顶层不再携带 `event_name`”“route authority 确实来自 `ops.event_route_policy`”这三件事通过固定 smoke、runbook 与 test-case 显式冻结下来。本批将优先补强 `trade003 / dlv002 / dlv029 / cat022 / audit anchor retry` 现有 smoke 的断言，再补一份 `AUD-030` 专用 runbook / test-case，把 route authority、触发器退役、canonical 顶层字段和无双写验证入口收口成正式留痕。
- 追溯：已重新核对 `CSV / Markdown`、`事件模型与Topic清单正式版.md` 2.3~2.5、`一致性与事件接口协议正式版.md` 2~4、`双层权威模型与链上链下一致性设计.md` 6.1~7.1、`A02-统一事件-Envelope-与路由权威源.md`，以及通用冻结文档 `服务清单与服务边界正式版.md`、`本地开发环境与中间件部署清单.md`、`配置项与密钥管理清单.md`、`技术选型正式版.md`、`平台总体架构设计草案.md`、`数据交易平台-全集成基线-V1.md`、`审计、证据链与回放设计.md`、`双层权威模型与链上链下一致性设计.md`、`链上链下技术架构与能力边界稿.md`、`日志、可观测性与告警设计.md`、`审计、证据链与回放接口协议正式版.md`、`一致性与事件接口协议正式版.md`、`fabric-local.md`、`kafka-topics.md`、`async-chain-write.md`、`072_canonical_outbox_route_policy.sql`、`074_event_topology_route_extensions.sql` 与 `infra/docker/docker-compose.local.yml`。当前确认的正式完成标准是：应用层 canonical outbox writer 与 `ops.event_route_policy` 成为唯一正式事件生产入口；旧 trigger 仅以退役异常函数残留、无任何正式表继续挂载；至少一条真实 smoke 能显式证明 canonical 顶层字段完整、`event_name` 不再作为正式顶层字段、target topic 命中唯一 route policy，且同一业务动作不会重复写出同协议事件。
### BATCH-243（待审批）
- 任务：`AUD-030` 收敛统一事件 envelope 与路由权威源
- 状态：待审批
- 实现摘要：
  - `apps/platform-core/src/modules/order/tests/trade003_create_order_db.rs`、`apps/platform-core/src/modules/catalog/tests/cat022_search_visibility_db.rs`、`apps/platform-core/src/modules/delivery/tests/dlv002_file_delivery_commit_db.rs`、`apps/platform-core/src/modules/delivery/tests/dlv029_delivery_task_autocreation_db.rs`、`apps/platform-core/src/modules/audit/tests/api_db.rs`：补强现有 canonical smoke，把验收口径从“能查到 outbox 行”提升为“route policy 命中正确 + 同一业务动作同协议事件只写一条 + payload 顶层 canonical 字段完整 + `event_name` 不再存在”。其中：
    - `trade003` 固定验证 `trade.order.created -> dtp.outbox.domain-events`
    - `cat022` 固定验证 `search.product.changed -> dtp.search.sync`
    - `dlv002` 固定验证 `delivery.committed` 与 `billing.trigger.bridge` 各只写一条正式事件
    - `dlv029` 固定验证 `delivery.task.auto_created -> dtp.outbox.domain-events`
    - `audit_trace_api_db_smoke` 固定验证 `audit.anchor_requested -> dtp.audit.anchor`
  - `docs/04-runbooks/canonical-event-authority.md`：新增 `AUD-030` 专用 runbook，正式写明唯一事件生产入口、唯一 route authority、`tg_write_outbox` 退役状态、关键 route seed、五条 smoke 与静态 SQL 回查。
  - `docs/05-test-cases/canonical-event-authority-cases.md`：新增 `AUD-030` 验收矩阵，冻结五类运行态 case 和一条 trigger 退役静态 case，避免后续再从分散 smoke 中推断 canonical route authority。
  - `docs/04-runbooks/README.md`、`docs/05-test-cases/README.md`：补充 `AUD-030` 索引，明确 canonical envelope / route authority 已有正式验收文件，而不是隐含在 `AUD-008 / AUD-009` 的 runbook 里。
- 验证：
  - 静态 SQL 回查通过：
    - `information_schema.triggers` 中 `action_statement ILIKE '%tg_write_outbox%'` 结果为 `0 rows`
    - `pg_get_functiondef(common.tg_write_outbox)` 显示该函数仅 `RAISE EXCEPTION 'common.tg_write_outbox is retired; use ops.event_route_policy + application canonical outbox writer instead'`
    - `ops.event_route_policy` 中 `trade.order.created`、`search.product.changed`、`delivery.committed`、`delivery.task.auto_created`、`billing.trigger.bridge`、`audit.anchor_requested` 六条关键路由均为 `status='active'`
  - 任务专项 smoke 通过：
    - `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture`
    - `CATALOG_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core cat022_search_visibility_fields_and_events_db_smoke -- --nocapture`
    - `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv002_file_delivery_commit_db_smoke -- --nocapture`
    - `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv029_delivery_task_autocreation_db_smoke -- --nocapture`
    - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_trace_api_db_smoke -- --nocapture`
  - 通用校验通过：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-030`
  - `事件模型与Topic清单正式版.md` 2.3~2.5
  - `一致性与事件接口协议正式版.md` 2~4
  - `双层权威模型与链上链下一致性设计.md` 6.1~7.1
  - `A02-统一事件-Envelope-与路由权威源.md`
- 覆盖的任务清单条目：`AUD-030`
- 未覆盖项：
  - 无。当前任务要求的 canonical envelope、route authority、trigger 退役与无双写验收已全部固定到代码断言、runbook 与 test-case。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
- 备注：
  - `common.tg_write_outbox()` 仍保留函数名仅用于显式失败与历史迁移识别，这不构成正式主链路回退；当前数据库没有任何正式 trigger 继续引用它。
  - 本批没有改动 canonical writer 主逻辑，真实缺口在于验收证据未被显式冻结。已通过更严的 smoke 断言和专用文档把 `AUD-030` 收口成可重复复核的正式基线。
### BATCH-244（计划中）
- 任务：`AUD-031` 清理把 `ops.outbox_event` 当私有工作队列的旁路实现
- 状态：计划中
- 说明：重新核对 `AUD-031` 的冻结要求后确认，当前 canonical `outbox -> publisher -> Kafka -> dead-letter` 主链已经在 `AUD-009 / AUD-010 / AUD-026 / AUD-030` 落地，但 Billing 侧仍保留 `POST /api/v1/billing/{order_id}/bridge-events/process` 这条会直接读取 `ops.outbox_event(event_type=billing.trigger.bridge)` 的人工桥接路径。现状相较旧版已收紧到“只处理 `status='published'`、`published_at IS NOT NULL`、`target_topic='dtp.outbox.domain-events'` 的已发布事件”，不再把 `pending` outbox 当默认私有工作队列；但当前缺少针对这一边界的显式负例与 publish attempt 联查验收，后续 Agent 仍可能误把这条 API 当成对 pending outbox 的默认消费入口。本批将优先补强 Billing bridge 代码约束、`bil024` / AUD smoke 断言、runbook 与 test-case，显式证明历史旁路已经被桥接到正式 publisher 链路，而不是继续绕过 `outbox_publish_attempt / dead_letter / reprocess` 主闭环。
- 追溯：已重新核对 `CSV / Markdown`、`双层权威模型与链上链下一致性设计.md` 6.3~7.2、`056_dual_authority_consistency.sql`、`一致性与事件接口协议正式版.md` 2~5、`A05-Outbox-Publisher-DLQ-统一闭环缺口.md`，以及通用冻结文档 `服务清单与服务边界正式版.md`、`事件模型与Topic清单正式版.md`、`本地开发环境与中间件部署清单.md`、`配置项与密钥管理清单.md`、`技术选型正式版.md`、`平台总体架构设计草案.md`、`数据交易平台-全集成基线-V1.md`、`审计、证据链与回放设计.md`、`审计、证据链与回放接口协议正式版.md`、`fabric-local.md`、`kafka-topics.md`、`async-chain-write.md`、`072_canonical_outbox_route_policy.sql`、`074_event_topology_route_extensions.sql` 与 `infra/docker/docker-compose.local.yml`。当前确认的正式完成标准是：`outbox-publisher` 与 `ops.outbox_publish_attempt` 仍是唯一正式 publish authority；Billing bridge 不再把 `ops.outbox_event` 当 `pending` 私有工作队列，且至少一条真实 smoke 或手工回查能够证明 `billing.trigger.bridge` 只能在 publish attempt 成功后被显式人工桥接、失败隔离与 `ops.outbox_publish_attempt` 可联查，并与 `dtp.dead-letter / reprocess` 总体口径不冲突。
### BATCH-244（待审批）
- 任务：`AUD-031` 清理把 `ops.outbox_event` 当私有工作队列的旁路实现
- 状态：待审批
- 完成情况：
  - `apps/platform-core/src/modules/billing/repo/billing_bridge_repository.rs`：Billing bridge 读取 `billing.trigger.bridge` 时，新增对最新 `ops.outbox_publish_attempt` 的 lateral 联查，只允许 `result_code='published'` 的已发布 outbox 进入物化；同时把 `bridge_publish_attempt_id / bridge_publish_attempt_no` 写入 `billing.billing_event.metadata`，让桥接结果能回溯到正式 publish authority。
  - `apps/platform-core/src/modules/billing/tests/bil024_billing_trigger_bridge_db.rs`：现有八类 SKU smoke 全部补充 `bridge_publish_attempt_id` 断言；新增“outbox 已标 published 但不存在 publish attempt”负例，显式证明该事件不会被 bridge API 处理，也不会生成 `billing.billing_event`。
  - `docs/04-runbooks/outbox-publisher.md`、`docs/04-runbooks/README.md`：冻结 Billing bridge 只处理 `status='published'`、`published_at IS NOT NULL`、最新 `ops.outbox_publish_attempt.result_code='published'` 的正式边界，并要求物化结果保留 `bridge_outbox_event_id / bridge_publish_attempt_id`。
  - `docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/README.md`：新增 `AUD-CASE-016A`，把 “Billing bridge published-only + publish-attempt gating” 纳入正式验收矩阵，避免后续再把该 API 解读为对 `pending` outbox 的默认消费入口。
- 验证：
  - `cargo fmt --all`
  - `cargo check -p platform-core`
  - `cargo test -p platform-core`
  - `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil024_billing_trigger_bridge_db_smoke -- --nocapture`
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p outbox-publisher outbox_publisher_db_smoke -- --nocapture`
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  - `./scripts/check-query-compile.sh`
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`AUD-031`
  - `双层权威模型与链上链下一致性设计.md` 6.3~7.2
  - `一致性与事件接口协议正式版.md` 2~5
  - `A05-Outbox-Publisher-DLQ-统一闭环缺口.md`
- 覆盖的任务清单条目：`AUD-031`
- 未覆盖项：
  - 无。当前任务要求的“移除或桥接旁路消费、收口到正式 publisher 语义、补 publish attempt 联查”已经由代码约束、负例 smoke、runbook 与 test-case 一并固定。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
- 备注：
  - `POST /api/v1/billing/{order_id}/bridge-events/process` 仍保留为显式人工桥接入口，但它不再具备“扫描 pending outbox”的隐式工作队列语义；当前已被收口为“只消费正式 publish authority 已成功发布的 bridge 事件”的受控补桥路径。
