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
