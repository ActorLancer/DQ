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
