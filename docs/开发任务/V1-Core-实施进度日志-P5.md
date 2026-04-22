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
