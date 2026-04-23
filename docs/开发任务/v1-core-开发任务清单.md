# 数据交易平台 V1-Core 开发任务清单 v2.4（增量执行版 / 单 Agent 顺序执行）

## 0. 本版说明

- 本版以 `V1Core开发清单参考` 为主任务源，合并当前仓库中已存在的 `开发任务/`、应用骨架、部署脚本和开发前文档现状，形成 **增量执行版**。
- 本版当前共 **452** 个任务；在不删内容的前提下，继续把前置过重任务拆成更细子任务，便于单 Agent 顺序执行。
- 本版继续只覆盖 **V1-Core**，明确不抢跑 `V2/V3`。
- **CSV 是唯一执行源**；Markdown 仅作为阅读版。若两者存在表述差异，以 CSV 为准。
- 当前仓库已存在部分骨架与环境文件；任务执行应优先复用现有资产，禁止无意义重建。

## 0A. 技术参考标注规则

- 本版为每个任务保留 `技术参考`，统一使用相对路径。
- `V1-Core` 以 `../全集成文档/数据交易平台-全集成基线-V1.md` 为业务真值入口。
- 若多个技术参考之间存在冲突，按 `../全集成文档/数据交易平台-全集成映射索引.md` 的冲突处理规则执行。
- `技术选型正式版.md` 的正确引用路径为 `../开发准备/技术选型正式版.md`。
- 若 `技术参考` 引用了 `问题修复任务/A*.md`，必须先读该文档的 `## 0. 当前状态`，再读归档的历史问题起点；当前执行状态仍以 `CSV / TODO / README / runbook / scripts` 为准。

## 0B. 与旧清单合并后的执行口径

- 旧版 `开发任务/` 中已完成的基础骨架与环境资产，作为当前仓库事实输入，不再视为“仓库不存在”。
- `CTX-014 / BOOT-001 / BOOT-002 / ENV-001 / CORE-010` 等重任务已拆为更细子任务；父任务保留为收口与验收任务。
- 保留文档前置与冻结任务，不删内容；但通过拆解降低单任务过载。

## 1. 使用方式（单一顺序执行 AI Agent）

- 先看 `depends_on`，再看 `wave`；`depends_on` 是执行顺序的硬约束，`wave` 只是阶段提示：`W0 -> W1 -> W2 -> W3`。
- 本版状态列只保留 `yes / no` 两种值，不再使用 `partial / limited / serial-first` 作为状态值。
- 当前采用 **`NOTIF` 阶段切线规则**：
  - `NOTIF` 之前的任务统一视为已完成，状态记为 `yes`
  - `NOTIF` 及之后的任务统一视为未完成，状态记为 `no`
- 这是一条当前批次的执行治理规则，用于明确后续开发入口；它不代表 `NOTIF` 之后任务已经逐条重新验收，也不改变这些任务在正文中的最终目标定义。
- `NOTIF` 及之后的任务即便已经存在占位代码、runbook、README、OpenAPI 草稿或测试 README，也仍统一视为未完成，后续需要按正式实现批次重新开发、验证和验收。
- 单 Agent 场景下，仍以“主闭环优先”顺序执行；执行顺序由 `depends_on` 决定，`wave` 仅提供阶段提示。
- 所有业务副作用统一通过 **审计 + outbox + provider / worker** 链路完成，禁止直接在 handler 中硬编码外部调用。
- 任何实现若试图把 `SHARE_RO / QRY_LITE / RPT_STD` 再并回“文件/API/沙箱大类”，一律视为偏离本版清单。

## 2. 分组统计

| 分组 | 标题 | 任务数 |
|---|---|---:|
| CTX | 上下文、约束与执行规则（给 AI Agent 的统一说明） | 24 |
| BOOT | 仓库初始化与骨架搭建（这一组建议由我先做） | 32 |
| ENV | 环境部署与 Docker Compose（这一组优先级最高） | 58 |
| CORE | platform-core 基础骨架（这一组建议由我先做） | 51 |
| DB | 数据库与 Migration 落地 | 35 |
| IAM | IAM / Party / Access 领域 | 20 |
| CAT | Catalog / Contract Meta / Listing / Review 领域 | 26 |
| TRADE | Order / Contract / Authorization 主交易链路 | 33 |
| DLV | Delivery / Storage Gateway / Query Execution | 31 |
| BIL | Billing / Payment / Settlement / Dispute | 26 |
| NOTIF | Notification / Messaging / Template | 14 |
| AUD | Audit / Evidence / Consistency / Fabric / Ops | 31 |
| SEARCHREC | Search / Recommendation / Projection | 21 |
| WEB | 前端最小页面闭环（portal-web / console-web） | 22 |
| TEST | 测试、演示数据、验收与 CI | 28 |

## 3. 本版新增/重点修订任务索引

- **重任务拆解补丁**：`CTX-022` ~ `CTX-024`、`BOOT-021` ~ `BOOT-036`、`ENV-044` ~ `ENV-057`、`CORE-033` ~ `CORE-051`
- **增量执行补丁**：`CTX-014`、`BOOT-001`、`BOOT-002`、`ENV-001`、`CORE-010` 已改成基于当前仓库现状的收敛任务。
- **审计收口补丁**：`AUD-029` 已追加为“历史模块统一 `audit writer / evidence writer` 与旧证据表桥接”专门任务。
- **SEARCHREC consumer 可靠性补丁**：`SEARCHREC-020` 已追加为“search-indexer / recommendation-aggregator 的幂等、双层 DLQ 与可重处理闭环”专门任务。
- **任务清单映射补丁**：已追加 `ENV-058`、`AUD-030`、`AUD-031`、`NOTIF-013`、`NOTIF-014`、`SEARCHREC-021`、`TEST-028`，并把 `A01~A15` 的冻结要求补回执行源、TODO 与承接文档。
- **引用修正补丁**：全部 `../技术选型正式版.md` 已统一改为 `../开发准备/技术选型正式版.md`。
- **执行源规则**：CSV 为唯一执行源，Markdown 为阅读版。

## 4. 上下文、约束与执行规则（给 AI Agent 的统一说明） [CTX]

这一组用于冻结口径和边界。保留文档前置任务，但把过重任务拆成更细的子任务，方便单 Agent 顺序推进。

- **CTX-001** [ARCH][P0][W0][yes] 通读并建立 `docs/00-context/reading-index.md`，把 29 份已读文档按“总纲/全集成/专题PRD/技术选型”重新分组，写清主阅读顺序：`开发准备/技术选型正式版.md` → `全集成文档/数据交易平台-全集成基线-V1.md` → `全集成文档/数据交易平台-全集成基线-全阶段.md` → 对应专题 PRD。
  依赖：无
  交付：docs/00-context/reading-index.md; 技术选型正式版.md; 全集成文档/数据交易平台-全集成基线-V1.md; 全集成文档/数据交易平台-全集成基线-全阶段.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成映射索引.md:L102（5. 章节映射表） | ../开发准备/技术选型正式版.md:L22（2. 技术选型总原则） | ../全集成文档/数据交易平台-全集成基线-V1.md:L161（5. 范围定义）
- **CTX-002** [ARCH][P0][W0][yes] 在 `docs/00-context/v1-core-guardrails.md` 冻结 V1-Core 基本原则：`PostgreSQL` 是业务主状态权威、`Fabric` 是摘要与可信确认层、`Kafka` 不是业务真值、搜索/推荐/链回执最终都要回 PostgreSQL 校验。
  依赖：无
  交付：docs/00-context/v1-core-guardrails.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L22（2. 技术选型总原则） | ../开发准备/技术选型正式版.md:L44（3.2 职责边界） | ../全集成文档/数据交易平台-全集成基线-V1.md:L290（6. 关键产品原则）
- **CTX-003** [ARCH][P0][W0][yes] 在 `docs/00-context/v1-core-scope.md` 明确 V1-Core 只做标准交易闭环，不引入 `MPC/TEE/FL/ZKP/C2D` 正式能力，不把 `V2/V3` 内容混入本轮开发。
  依赖：无
  交付：docs/00-context/v1-core-scope.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L161（5. 范围定义） | ../开发准备/技术选型正式版.md:L178（7.1 V1 建议） | ../全集成文档/数据交易平台-全集成基线-全阶段.md:L256（5.4 V2 范围（扩展能力层））
- **CTX-004** [ARCH][P0][W0][yes] 在 `docs/00-context/architecture-style.md` 冻结架构风格为“模块化单体 `platform-core` + 外围独立进程”，明确当前阶段不做全面微服务拆分。
  依赖：无
  交付：docs/00-context/architecture-style.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计）
- **CTX-005** [ARCH][P0][W0][yes] 在 `docs/00-context/lifecycle-objects.md` 冻结生命周期对象：`Order`、`DigitalContract`、`Authorization`、`Delivery`、`Settlement`、`Dispute`、`BillingEvent`，后续模块命名与状态机必须围绕这些对象展开。
  依赖：无
  交付：docs/00-context/lifecycle-objects.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分） | ../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **CTX-006** [ARCH][P0][W0][yes] 在 `docs/00-context/standard-sku-truth.md` 冻结 V1 八个标准 SKU 真值：`FILE_STD`、`FILE_SUB`、`SHARE_RO`、`API_SUB`、`API_PPU`、`QRY_LITE`、`SBX_STD`、`RPT_STD`，并写清套餐不得反向替代 SKU 编码。
  依赖：无
  交付：docs/00-context/standard-sku-truth.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../原始PRD/数据产品分类与交易模式详细稿.md:L155（6. V1 标准数据产品目录） | ../原始PRD/数据对象产品族与交付模式增强设计.md:L292（4. 七类标准交易方式）
- **CTX-007** [ARCH][P0][W0][yes] 在 `docs/00-context/first-5-scenarios.md` 冻结首批五条标准链路，标注对应主 SKU、可选补充 SKU、验收路径与演示数据需求。
  依赖：无
  交付：docs/00-context/first-5-scenarios.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../业务流程/业务流程图-V1-完整版.md:L66（4. 主业务流程） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L21（6.2 业务流程设计）
- **CTX-008** [ARCH][P0][W0][yes] 在 `docs/00-context/run-modes.md` 冻结三套运行模式：`local`、`staging`、`demo`；要求一切环境切换通过配置完成，禁止手工改代码切换，并明确 `staging-local` 只允许作为 `local` 下的 staging-like 联调姿态，不得扩张为第四套正式 mode。
  依赖：无
  交付：docs/00-context/run-modes.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；三套正式运行模式与 `local` 子场景边界清晰；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用，且不会把本地 profile / 联调姿态误导成新的正式 mode。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../开发准备/技术选型正式版.md:L44（3.2 职责边界） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../00-context/local-deployment-boundary.md:L1（本地部署边界） | ../开发准备/本地开发环境与中间件部署清单.md:L267（当前开发模式冻结）
- **CTX-009** [ARCH][P0][W0][yes] 在 `docs/00-context/provider-boundary.md` 定义统一 Provider 适配原则：KYC/KYB、签章、支付、通知、链写入、风控外部能力必须有 `mock` / `real` 双实现。
  依赖：无
  交付：docs/00-context/provider-boundary.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座）
- **CTX-010** [ARCH][P0][W0][yes] 在 `docs/00-context/async-chain-write.md` 冻结“所有上链动作走异步事件链路”，明确 `outbox_event -> publisher worker -> dtp.audit.anchor / dtp.fabric.requests -> fabric-adapter` 主路径，业务请求不得同步阻塞等链确认。
  依赖：无
  交付：docs/00-context/async-chain-write.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计）
- **CTX-011** [ARCH][P0][W0][yes] 在 `docs/00-context/search-and-recommend-boundary.md` 冻结搜索与推荐边界：`OpenSearch`/推荐缓存只做读模型与召回，最终结果必须回 PostgreSQL 做可见性与状态校验。
  依赖：无
  交付：docs/00-context/search-and-recommend-boundary.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L88（4.1 PostgreSQL 层） | ../原始PRD/商品搜索、排序与索引同步设计.md:L319（10.1 核心原则） | ../原始PRD/商品推荐与个性化发现设计.md:L37（2.3 推荐结果不能绕过业务放行）
- **CTX-012** [ARCH][P0][W0][yes] 在 `docs/00-context/security-and-audit-floor.md` 冻结最低安全基线：统一 `request_id`、关键对象 ID、审计留痕、导出审计、再认证、SoD、最小披露。
  依赖：无
  交付：docs/00-context/security-and-audit-floor.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/IAM 技术接入方案.md:L246（8.1 必须具备的标准能力） | ../权限设计/接口权限校验清单.md:L53（3. V1 接口权限校验清单） | ../权限设计/后端鉴权中间件规则说明.md:L244（7. 审计要求）
- **CTX-013** [ARCH][P0][W0][yes] 在 `docs/00-context/ownership-strategy.md` 明确任务 ownership 规则：基础架构/仓库骨架/platform-core 主骨架/领域边界冻结由前置统筹阶段先执行；业务模块实现按 `depends_on` 顺序推进。
  依赖：无
  交付：docs/00-context/ownership-strategy.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L34（12.3 消息总线与异步任务） | ../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event）
- **CTX-014** [ARCH][P0][W0][yes] 在 `docs/00-context/current-gap-analysis.md` 记录当前仓库现状与目标差距：当前已存在基础代码仓、`apps/*` 目录骨架、`apps/platform-core` 最小服务、`部署脚本/docker-compose.local.yml`、开发任务与设计文档目录；但仍缺少按本清单统一收敛后的增量实现、运行时收口、OpenAPI 归档、CI 流程与正式 runbook。
  依赖：CTX-022; CTX-023; CTX-024
  交付：docs/00-context/current-gap-analysis.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：现有资产、缺口清单和后续迁移/复用策略都已写清，并被后续 BOOT/ENV/CORE 任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构）
- **CTX-015** [ARCH][P0][W0][yes] 在 `docs/00-context/v1-exit-criteria.md` 冻结 V1 退出标准：五条标准链路端到端验证通过；订单成功率/验收成功率/账单准确率达到门槛；证据包可导出并完成一次模拟监管检查。
  依赖：无
  交付：docs/00-context/v1-exit-criteria.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **CTX-016** [AGENT][P1][W2][yes] 生成 `docs/00-context/term-glossary.md`，整理术语：Tenant、Party、Application、Connector、ExecutionEnvironment、DataResource、DataProduct、SKU、Authorization、QuerySurface、QueryTemplate 等，避免后续命名漂移。
  依赖：无
  交付：docs/00-context/term-glossary.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成映射索引.md:L102（5. 章节映射表） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../原始PRD/数据产品分类与交易模式详细稿.md:L155（6. V1 标准数据产品目录）
- **CTX-017** [AGENT][P1][W2][yes] 生成 `docs/00-context/doc-to-module-map.md`，把每份专题 PRD 映射到具体模块、数据库 migration、OpenAPI 分组与测试域。
  依赖：无
  交付：docs/00-context/doc-to-module-map.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L17545（10. 页面与模块映射图） | ../页面说明书/页面说明书-V1-完整版.md:L936（12. 页面间路由关系） | ../权限设计/菜单树与路由表正式版.md:L30（3. V1 菜单树与路由）
- **CTX-018** [AGENT][P1][W2][yes] 生成 `docs/00-context/non-goals.md`，列出本轮明确不做事项：生产级公链锚定、凭证/NFT 正式发放、跨境自动交付、自然人大众供给、V2/V3 隐私计算。
  依赖：无
  交付：docs/00-context/non-goals.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/数据库表字典正式版.md:L22（2. 存储策略总览） | ../数据库设计/表关系总图-ER文本图.md:L15（2. 顶层域关系图）
- **CTX-019** [ARCH][P0][W0][yes] 在 `docs/00-context/service-to-module-map.md` 明确“技术选型文档中的服务名”到“当前阶段 platform-core 模块/外围进程”的映射，例如 `iam-service -> platform-core::iam + party + access`、`trade-service -> platform-core::order + contract + authorization + delivery`、`notification-worker -> apps/notification-worker`，禁止不同 Agent 各自按不同拆分方式新建服务。
  依赖：CTX-004; BOOT-002
  交付：docs/00-context/service-to-module-map.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L161（5. 范围定义） | ../未来问题清单/工程约束与上线前补齐事项.md:L9（1. 当前已冻结的最小结论） | ../未来问题清单/发布级基线文档硬化未决事项.md:L91（3. 进入最终发布版前应完成的动作）
- **CTX-020** [ARCH][P0][W0][yes] 在 `docs/00-context/local-deployment-boundary.md` 冻结本地部署边界：`docker-compose.local.yml` 先以“中间件 + mock provider + Fabric 测试网络”为主；业务应用默认通过 `make run-*` 本机启动；若后续需要容器化应用联调，另建 `docker-compose.apps.local.yml`，禁止在同一个 compose 文件里无限扩张职责。
  依赖：CTX-008; ENV-001
  交付：docs/00-context/local-deployment-boundary.md; docker-compose.local.yml; docker-compose.apps.local.yml
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **CTX-021** [ARCH][P0][W0][yes] 在 `docs/00-context/v1-closed-loop-matrix.md` 输出 V1-Core 闭环矩阵：8 个 SKU × 5 条标准链路 × 合同/授权/交付/验收/计费/退款/争议/审计，对每个交叉点写清“主触发点、状态推进点、证据对象、测试入口”。
  依赖：CTX-006; CTX-007; CTX-015
  交付：docs/00-context/v1-closed-loop-matrix.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../业务流程/业务流程图-V1-完整版.md:L66（4. 主业务流程）
- **CTX-022** [ARCH][P0][W0][yes] 盘点当前仓库已存在的目录、应用骨架、脚本、环境文件和设计文档，输出 `exists / partial / missing` 资产清单，为后续增量执行建立事实基础。
  依赖：CTX-001
  交付：docs/00-context/current-repo-assets.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计） | ../开发准备/仓库拆分与目录结构建议.md:L1（仓库拆分与目录结构建议） | ../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`）
- **CTX-023** [ARCH][P0][W0][yes] 盘点当前本地部署与运维资产：`infra/docker/docker-compose.local.yml`、PostgreSQL 迁移测试编排、数据库校验脚本、现有容器配置、Mock Provider、Fabric 测试网络脚本和本地观测配置。
  依赖：CTX-001
  交付：docs/00-context/current-local-stack-assets.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../../scripts/validate_database_migrations.sh:L1（迁移校验脚本入口） | ../../部署脚本/docker-compose.postgres-test.yml:L1（PostgreSQL 迁移测试编排） | ../开发准备/配置项与密钥管理清单.md:L1（本地配置与密钥边界） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **CTX-024** [ARCH][P0][W0][yes] 对照当前仓库里的 `开发任务/`、`开发准备/`、`开发前设计文档/` 与本清单，输出“已存在可复用 / 需迁移 / 需重写 / 可删除重复”的差异清单。
  依赖：CTX-001
  交付：docs/00-context/current-task-baseline-gap.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发任务/README.md:L1（开发任务目录 README） | ../开发前设计文档/README.md:L1（若存在则引用阅读入口） | ../开发准备/服务清单与服务边界正式版.md:L1（服务清单与服务边界正式版）

## 5. 仓库初始化与骨架搭建（这一组建议由我先做） [BOOT]

这一组负责把当前已有仓库收敛成可开发工程仓。已有资产优先复用，缺失项补齐。

- **BOOT-001** [ARCH][P0][W0][yes] 补齐并校准仓库根目录基础文件：`README.md`、`.gitignore`、`.editorconfig`、`.gitattributes`、`.env.example`、`.env.local.example`、`.env.staging.example`、`.env.demo.example`；若已有文件则以增量修改为主，不重复覆盖已有有效内容。
  依赖：BOOT-021; BOOT-022; BOOT-023; BOOT-024
  交付：README.md; .env.example; .env.local.example; .env.staging.example
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：根目录基础文件齐备；现有文件已校准到当前基线；未破坏已有可用配置。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **BOOT-002** [ARCH][P0][W0][yes] 校准并补齐仓库目录骨架，确保当前仓库结构与目标目录树一致，并将最终结构写入 `docs/01-architecture/repo-layout.md`；已有目录优先复用，缺失目录补齐。
  依赖：BOOT-029; BOOT-030; BOOT-031; BOOT-032; BOOT-033; BOOT-034; BOOT-035; BOOT-036
  交付：docs/01-architecture/repo-layout.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：目标目录树与当前仓库实际结构对齐，`repo-layout.md` 能作为后续任务的唯一路径参考。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **BOOT-003** [ARCH][P0][W0][yes] 在根目录创建 `Makefile`，统一封装 `make up-local`、`make up-core`、`make up-mocks`、`make up-observability`、`make up-fabric`、`make up-demo`、`make down-local`、`make logs`、`make migrate-up`、`make migrate-down`、`make seed-local`、`make test`、`make lint`。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：Makefile
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **BOOT-004** [ARCH][P0][W0][yes] 在根目录创建 `scripts/` 下的统一入口脚本：`bootstrap.sh`、`up-local.sh`、`down-local.sh`、`wait-for-services.sh`、`seed-demo.sh`、`reset-local.sh`。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：scripts/; bootstrap.sh; up-local.sh; down-local.sh
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **BOOT-005** [ARCH][P0][W0][yes] 建立多语言工作区规范：Rust 用 workspace，Go 各服务独立 module，Python 用独立 package 管理，前端用 pnpm workspace；把约束写进 `docs/01-architecture/multi-language-workspace.md`。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/multi-language-workspace.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **BOOT-006** [ARCH][P0][W0][yes] 创建 `packages/shared-config/`，统一维护环境变量 schema、默认值、模式切换规则与服务间共享配置键名。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：packages/shared-config/
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **BOOT-007** [ARCH][P0][W0][yes] 创建 `packages/openapi/`，约定 OpenAPI 文件按领域拆分：`iam.yaml`、`catalog.yaml`、`trade.yaml`、`billing.yaml`、`audit.yaml`、`ops.yaml`，并提供合并脚本。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：packages/openapi/; iam.yaml; catalog.yaml; trade.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计）
- **BOOT-008** [ARCH][P0][W0][yes] 创建 `db/migrations/v1/`、`db/seeds/`、`db/scripts/` 目录，并约定升级脚本、回滚脚本、幂等 seed、测试数据 seed 的命名规则。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：db/migrations/v1/; db/seeds/; db/scripts/
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计）
- **BOOT-009** [ARCH][P0][W0][yes] 创建 `docs/04-runbooks/`，预置 `local-startup.md`、`troubleshooting.md`、`provider-switch.md`、`fabric-debug.md`、`search-reindex.md`。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/04-runbooks/; local-startup.md; troubleshooting.md; provider-switch.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计）
- **BOOT-010** [ARCH][P0][W0][yes] 创建 `fixtures/`，用于保存本地演示数据、标准测试请求、样例产品元数据、模拟证据包与 API 调用样例。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：fixtures/
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计）
- **BOOT-011** [ARCH][P0][W0][yes] 创建 `.github/workflows/` 基础 CI 骨架：`lint.yml`、`test.yml`、`build.yml`，先不追求完整矩阵，但要有占位。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：.github/workflows/; lint.yml; test.yml; build.yml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L5（12.1 API 设计规范） | ../全集成文档/数据交易平台-全集成映射索引.md:L102（5. 章节映射表） | ../全集成文档/数据交易平台-全集成基线-V1.md:L34851（9. 接口、事件与集成协议）
- **BOOT-012** [ARCH][P0][W0][yes] 约定统一错误码字典文件 `docs/01-architecture/error-codes.md`，按领域前缀规划：`IAM_`、`CAT_`、`TRD_`、`DLV_`、`BIL_`、`AUD_`、`OPS_`。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/error-codes.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L5（12.1 API 设计规范） | ../全集成文档/数据交易平台-全集成映射索引.md:L102（5. 章节映射表） | ../全集成文档/数据交易平台-全集成基线-V1.md:L34851（9. 接口、事件与集成协议）
- **BOOT-013** [ARCH][P0][W0][yes] 约定统一日志字段规范文件 `docs/01-architecture/logging-fields.md`，固定 `request_id`、`trace_id`、`tenant_id`、`order_id`、`event_id`、`provider`、`mode`、`actor_id` 等字段名。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/logging-fields.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准） | ../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路）
- **BOOT-014** [ARCH][P0][W0][yes] 建立 `docs/01-architecture/module-dependency-rules.md`，明确 `platform-core` 模块依赖单向约束，禁止循环依赖与跨模块直接绕库。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/module-dependency-rules.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准） | ../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路）
- **BOOT-015** [ARCH][P0][W0][yes] 创建 `docs/01-architecture/issue-template.md` 与 `docs/01-architecture/pr-template.md`，为后续 AI Agent 承接任务提供固定输入格式、完成定义与验收模板。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/issue-template.md; docs/01-architecture/pr-template.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准） | ../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路）
- **BOOT-016** [ARCH][P0][W0][yes] 创建 `docs/01-architecture/ownership-matrix.md`，把平台主骨架、环境编排、状态机、事件模型、OpenAPI 冻结列为主控任务，把模块实现与前端页面列为可分发任务。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/ownership-matrix.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **BOOT-017** [AGENT][P1][W2][yes] 为每个顶层目录补充 `README.md`，说明职责、边界、依赖与禁止事项。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：README.md
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **BOOT-018** [AGENT][P1][W2][yes] 建立 `docs/01-architecture/naming-conventions.md`，统一 crate/package/service/module/file 名称风格和数据库对象命名规范。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/naming-conventions.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **BOOT-019** [AGENT][P1][W2][yes] 建立 `docs/01-architecture/versioning-policy.md`，定义迁移版本、OpenAPI 版本、Provider 版本、演示数据版本、模板版本之间的关联规则。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/versioning-policy.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **BOOT-020** [AGENT][P1][W2][yes] 建立 `docs/01-architecture/release-policy.md`，先定义 local/staging/demo 的发布粒度、回滚原则、数据重置策略与演示环境保护规则。
  依赖：CTX-001; CTX-004; CTX-008; CTX-013; CTX-014
  交付：docs/01-architecture/release-policy.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **BOOT-021** [ARCH][P0][W0][yes] 补齐并校准根目录 `README.md`，明确仓库定位、主应用、外围进程、开发入口和文档阅读顺序。
  依赖：CTX-014
  交付：README.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L1（仓库拆分与目录结构建议） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案） | ../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计）
- **BOOT-022** [ARCH][P0][W0][yes] 补齐并校准 `.gitignore`、`.editorconfig`、`.gitattributes`，统一多语言仓库的格式、文本属性和忽略规则。
  依赖：CTX-014
  交付：.gitignore; .editorconfig; .gitattributes
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L1（仓库拆分与目录结构建议） | ../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../开发准备/配置项与密钥管理清单.md:L1（配置项与密钥管理清单）
- **BOOT-023** [ARCH][P0][W0][yes] 补齐根目录环境变量样例文件，并与 `packages/shared-config` 及本地部署变量命名保持一致。
  依赖：CTX-014
  交付：.env.example; .env.local.example; .env.staging.example; .env.demo.example
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/配置项与密钥管理清单.md:L1（配置项与密钥管理清单） | ../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）
- **BOOT-024** [ARCH][P0][W0][yes] 记录“基于当前仓库增量初始化”的约束，明确已有文件的复用策略、禁止覆盖项和需迁移项。
  依赖：CTX-014
  交付：docs/01-architecture/repo-init-notes.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发任务/README.md:L1（开发任务目录 README） | ../开发准备/仓库拆分与目录结构建议.md:L1（仓库拆分与目录结构建议） | ../全集成文档/数据交易平台-全集成基线-V1.md:L161（5. 范围定义）
- **BOOT-029** [ARCH][P0][W0][yes] 校准 `apps/` 目录，确认 `platform-core`、`portal-web`、`console-web`、`notification-worker` 的落位和命名不再漂移。
  依赖：CTX-014
  交付：apps/
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L132（5.1 `apps/platform-core`） | ../开发准备/平台总体架构设计草案.md:L62（主应用） | ../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`）
- **BOOT-030** [ARCH][P0][W0][yes] 校准 `services/` 目录，明确 `fabric-adapter`、`fabric-event-listener`、`fabric-ca-admin`、`mock-payment-provider` 的落位。
  依赖：CTX-014
  交付：services/
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../开发准备/平台总体架构设计草案.md:L123（主应用与外围边界） | ../开发准备/服务清单与服务边界正式版.md:L568（外围进程）
- **BOOT-031** [ARCH][P0][W0][yes] 校准 `workers/` 目录，明确 `search-indexer`、`outbox-publisher`、`data-processing-worker`、`quality-profiler`、`report-job` 的落位。
  依赖：CTX-014
  交付：workers/
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../开发准备/平台总体架构设计草案.md:L123（主应用与外围边界） | ../开发准备/事件模型与Topic清单正式版.md:L170（V1 Topic 清单）
- **BOOT-032** [ARCH][P0][W0][yes] 校准 `packages/` 目录，明确 `openapi`、`sdk-ts`、`ui`、`shared-config`、`observability-contracts` 的边界。
  依赖：CTX-014
  交付：packages/
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L1（仓库拆分与目录结构建议） | ../开发准备/接口清单与OpenAPI-Schema冻结表.md:L1（接口清单与OpenAPI-Schema冻结表） | ../开发准备/配置项与密钥管理清单.md:L1（配置项与密钥管理清单）
- **BOOT-033** [ARCH][P0][W0][yes] 校准 `db/` 目录，统一迁移、种子、脚本与数据库文档归档结构，并与现有 `数据库设计/` 目录建立映射。
  依赖：CTX-014
  交付：db/
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L1（仓库拆分与目录结构建议） | ../数据库设计/README.md:L1（数据库设计 README） | ../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单）
- **BOOT-034** [ARCH][P0][W0][yes] 校准 `infra/` 目录，统一 `docker/fabric/keycloak/kafka/postgres/minio/opensearch/redis/prometheus/grafana/loki/tempo/otel` 的落位。
  依赖：CTX-014
  交付：infra/
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **BOOT-035** [ARCH][P0][W0][yes] 校准 `docs/` 目录，确保 `00-context/01-architecture/02-openapi/03-db/04-runbooks/05-test-cases` 与任务清单一致。
  依赖：CTX-014
  交付：docs/
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L1（仓库拆分与目录结构建议） | ../开发任务/README.md:L1（开发任务目录 README） | ../开发准备/测试用例矩阵正式版.md:L1（测试用例矩阵正式版）
- **BOOT-036** [ARCH][P0][W0][yes] 校准 `fixtures/` 与 `.github/workflows/` 目录，为演示数据、CI、契约校验和 smoke test 保留统一落位。
  依赖：CTX-014
  交付：fixtures/; .github/workflows/
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：完成物已创建并可被后续任务引用。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/测试用例矩阵正式版.md:L1（测试用例矩阵正式版） | ../开发准备/仓库拆分与目录结构建议.md:L1（仓库拆分与目录结构建议） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）

## 6. 环境部署与 Docker Compose（这一组优先级最高） [ENV]

这一组是本轮第一优先级。没有稳定本地环境，后续所有业务任务都只能空转。

- **ENV-001** [ARCH][P0][W0][yes] 在现有本地部署资产基础上收敛出正式 `infra/docker/docker-compose.local.yml`，统一编排 `postgres`、`redis`、`kafka`、`minio`、`opensearch`、`keycloak`、`otel-collector`、`prometheus`、`alertmanager`、`grafana`、`loki`、`tempo`、`mock-payment-provider`、`fabric-test-network`。
  依赖：ENV-044; ENV-045; ENV-046; ENV-047; ENV-048; ENV-049; ENV-050; ENV-051; ENV-052; ENV-053; ENV-054; ENV-055; ENV-056; ENV-057
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功；已有部署脚本已合并进统一结构。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过，且现有 `部署脚本/docker-compose.local.yml` 已完成迁移或兼容说明。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-002** [ARCH][P0][W0][yes] 为 `docker-compose.local.yml` 建立统一 network、volume、restart policy、healthcheck、resource limit、log driver 策略，避免服务随机 ready 导致联调不稳定。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：docker-compose.local.yml
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-003** [ARCH][P0][W0][yes] 在 `infra/docker/docker-compose.local.override.example.yml` 预留开发者个性化覆盖能力，例如端口冲突、挂载本地代码、替换镜像源。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.override.example.yml
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-004** [ARCH][P0][W0][yes] 在 `infra/docker/.env.local` 示例文件中定义所有本地 compose 变量：端口、初始账号、bucket 名称、topic 名称、索引别名、realm 名称、链网络名称。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/docker/.env.local
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-005** [ARCH][P0][W0][yes] 为 PostgreSQL 准备 `infra/postgres/initdb/` 脚本，创建业务 schema、扩展、默认角色、最小权限、时区、编码与连接参数。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/postgres/initdb/
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../数据库设计/README.md:L67（4. 迁移策略） | ../数据库设计/数据库设计总说明.md:L20（2. 数据库分层） | ../数据库设计/V1/upgrade/001_extensions_and_schemas.sql:L1（V1 migration）
- **ENV-006** [ARCH][P0][W0][yes] 在 PostgreSQL 容器中启用本项目所需扩展（如 `pgcrypto`、`uuid-ossp` 或最终选定扩展），并验证迁移脚本可直接运行。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L44（3.2 职责边界） | ../原始PRD/数据商品存储与分层存储设计.md:L130（4. 分层存储区设计） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L35（9.3 数据与存储策略）
- **ENV-007** [ARCH][P0][W0][yes] 编写 `infra/postgres/postgresql.conf` 与 `infra/postgres/pg_hba.conf` 本地调优版本，保证本地联调性能和可连接性。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/postgres/postgresql.conf; infra/postgres/pg_hba.conf
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L34（12.3 消息总线与异步任务） | ../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../原始PRD/双层权威模型与链上链下一致性设计.md:L400（9. 分域规则）
- **ENV-008** [AGENT][P0][W0][yes] 为 PostgreSQL 建立 `db/scripts/check-db-ready.sh`，等待数据库可连接并完成 schema 检查后再启动依赖服务。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：db/scripts/check-db-ready.sh
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L34（12.3 消息总线与异步任务） | ../原始PRD/链上链下技术架构与能力边界稿.md:L305（10. 事件流设计） | ../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座）
- **ENV-009** [ARCH][P0][W0][yes] 选择 Kafka 本地模式（建议 KRaft 单节点），编写 `infra/kafka/docker-compose` 片段与 topic 初始化脚本。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/kafka/docker-compose
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../开发准备/技术选型正式版.md:L44（3.2 职责边界）
- **ENV-010** [ARCH][P0][W0][yes] 建立 Kafka topic 初始化脚本，至少包含：`dtp.outbox.domain-events`、`dtp.search.sync`、`dtp.recommend.behavior`、`dtp.notification.dispatch`、`dtp.fabric.requests`、`dtp.fabric.callbacks`、`dtp.payment.callbacks`、`dtp.audit.anchor`、`dtp.consistency.reconcile`、`dtp.dead-letter`，并明确 topic 只能来自 `infra/kafka/topics.v1.json`；通知与 Fabric 的正式消费入口分别固定为 `dtp.notification.dispatch` 与 `dtp.audit.anchor / dtp.fabric.requests`，不直接消费 `dtp.outbox.domain-events`。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式） | ../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | 问题修复任务/A01-Kafka-Topic-口径统一.md:L1（Kafka topic 口径统一）
- **ENV-011** [AGENT][P0][W0][yes] 为 Kafka 配置 consumer group、DLQ、retention、cleanup policy 的本地默认值，并写入 `docs/04-runbooks/kafka-topics.md`；本地 topic 必须经 `topics.v1.json + init-topics.sh` 显式初始化，不再依赖 auto-create；同时明确区分关键拓扑静态检查（`check-topic-topology.sh`）与全量 canonical smoke（`smoke-local.sh`）的职责边界，并冻结通知/Fabric 的专用 topic 单入口。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：docs/04-runbooks/kafka-topics.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警） | 问题修复任务/A01-Kafka-Topic-口径统一.md:L1（Kafka topic 口径统一） | 问题修复任务/A12-配置项与资源命名漂移.md:L1（配置项与资源命名漂移）
- **ENV-012** [ARCH][P0][W0][yes] 为 Redis 提供基础配置：缓存 DB 划分、过期策略、命名空间前缀、密码与持久化策略。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警）
- **ENV-013** [AGENT][P0][W0][yes] 在 `docs/04-runbooks/redis-keys.md` 约定 key 模式：幂等键、会话缓存、权限缓存、推荐缓存、搜索缓存、限流计数、下载票据缓存。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：docs/04-runbooks/redis-keys.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警） | 问题修复任务/A12-配置项与资源命名漂移.md:L1（配置项与资源命名漂移）
- **ENV-014** [ARCH][P0][W0][yes] 配置 MinIO：创建 `raw-data`、`preview-artifacts`、`delivery-objects`、`report-results`、`evidence-packages`、`model-artifacts` bucket。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警）
- **ENV-015** [AGENT][P0][W0][yes] 编写 MinIO 初始化脚本，自动创建 bucket、bucket policy、测试对象、生命周期规则与本地访问别名。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警）
- **ENV-016** [ARCH][P0][W0][yes] 配置 OpenSearch 本地单节点，提供 index template、analysis 设置、别名策略与 demo 数据索引初始化。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警） | 问题修复任务/A12-配置项与资源命名漂移.md:L1（配置项与资源命名漂移）
- **ENV-017** [AGENT][P0][W0][yes] 补充 OpenSearch 索引初始化脚本，至少生成 `product_search_read`、`product_search_write`、`seller_search_read`、`seller_search_write` 以及 `search_sync_jobs_v1` 等别名/索引，并与 `search.index_alias_binding` 的结构化口径保持一致。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L126（6.1 V1 支持的支付方式） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算） | 问题修复任务/A12-配置项与资源命名漂移.md:L1（配置项与资源命名漂移）
- **ENV-018** [ARCH][P0][W0][yes] 配置 Keycloak 容器与 realm import 机制，导入 `platform-local` realm、基础角色、测试用户、客户端、MFA 占位流程。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合）
- **ENV-019** [AGENT][P0][W0][yes] 创建 `infra/keycloak/realm-export/platform-local-realm.json` 占位文件，并约定后续由 IAM 模块增量维护。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/keycloak/realm-export/platform-local-realm.json
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合）
- **ENV-020** [ARCH][P0][W0][yes] 为 Mock Payment Provider 提供独立容器与启动参数，保证可以在本地模拟支付成功、失败、超时、退款成功、人工打款成功。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-mocks` 或 `make up-demo` 后，Mock Payment Provider 可触发 success/fail/timeout/refund/manual-transfer 场景，且 `scripts/check-local-stack.sh mocks|full` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合）
- **ENV-021** [AGENT][P0][W0][yes] 实现 Mock Payment Provider 的 compose 健康检查与 readiness 路径，并在 `docs/04-runbooks/mock-payment.md` 写明如何手工触发模拟事件。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：docs/04-runbooks/mock-payment.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：文档已明确写出 `mock-payment-provider` 仅属于 `mocks/demo`，且执行 `make up-mocks` 或 `make up-demo` 后 `./scripts/check-mock-payment.sh` 与 `scripts/check-local-stack.sh mocks|full` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合） | ../04-runbooks/compose-profiles.md:L1（Compose profile 边界）
- **ENV-022** [ARCH][P0][W0][yes] 在 `infra/fabric/` 下接入 Hyperledger Fabric 测试网络启动脚本，封装 peer/orderer/ca 初始化，保证 local 模式也能选择启用测试链。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/fabric/
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-023** [ARCH][P0][W0][yes] 提供 `make fabric-up`、`make fabric-down`、`make fabric-reset`、`make fabric-channel` 脚本包装，降低本地链环境操作复杂度。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-024** [AGENT][P0][W0][yes] 为 Fabric 本地网络补充最小链码部署脚本，占位实现订单摘要、授权摘要、验收摘要、证据批次根写入接口。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-025** [ARCH][P0][W0][yes] 配置 OpenTelemetry Collector，统一接收 `platform-core`、workers、Go 服务的 traces/logs/metrics 并转发到 Prometheus/Loki/Tempo。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-026** [AGENT][P0][W0][yes] 配置 Prometheus 抓取目标，至少覆盖 platform-core、mock-payment-provider、Kafka exporter、Postgres exporter、Redis exporter、MinIO exporter、OpenSearch exporter。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-demo`（或至少 `core + observability + mocks` 组合）后，Prometheus 可抓到 `mock-payment-provider` 与其他 exporter 指标。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-027** [AGENT][P0][W0][yes] 配置 Alertmanager 最小规则集：服务不可用、队列积压、DB 连接失败、链适配失败、outbox 重试异常、DLQ 增长。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-028** [AGENT][P0][W0][yes] 配置 Grafana 数据源（Prometheus/Loki/Tempo）并导入 4 组初始 dashboard：平台总览、数据库、Kafka、应用链路追踪。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-029** [AGENT][P0][W0][yes] 配置 Loki 与 Tempo 的本地存储挂载、保留周期和清理策略，避免本地长期调试把磁盘写满。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-030** [ARCH][P0][W0][yes] 在 compose 中实现 profile 机制：`core`、`observability`、`mocks`、`fabric`、`demo`，允许先启动最小核心栈，再按需附加观测、Mock Payment 与链环境。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-031** [AGENT][P0][W0][yes] 为每个基础服务补充 `curl`/`nc`/`mc`/`kcat`/`psql` 级别的启动后自检脚本，组成 `scripts/check-local-stack.sh`。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：scripts/check-local-stack.sh
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-032** [ARCH][P0][W0][yes] 建立 `make up-core`、`make up-observability`、`make up-mocks`、`make up-fabric`、`make up-demo` 组合命令，满足文档要求的 local/staging/demo 三套模式切换基础，并显式承接 `local` 下的 `mocks` 子 profile。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../04-runbooks/compose-profiles.md:L1（Compose profile 边界）
- **ENV-033** [ARCH][P0][W0][yes] 在 `docs/04-runbooks/local-startup.md` 写清本地启动顺序：先基础设施，再 schema/migration，再 seed，再应用，再回执模拟。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：docs/04-runbooks/local-startup.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：文档已明确区分 `make up-local`、`make up-mocks`、`make up-demo` 的用途，并写清 `./scripts/check-mock-payment.sh` 只在 `mocks/demo` 前置条件满足时执行。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../04-runbooks/compose-profiles.md:L1（Compose profile 边界）
- **ENV-034** [AGENT][P1][W2][yes] 补充 `docker-compose.staging.example.yml` 占位文件，不部署真实生产资源，但明确后续 Helm/K8s 迁移时组件映射关系。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：docker-compose.staging.example.yml
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L126（6.1 V1 支持的支付方式） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **ENV-035** [AGENT][P1][W2][yes] 在 `docs/04-runbooks/secrets-policy.md` 写明本地 secrets 管理规则，明确哪些变量可入 `.env.local`，哪些必须走 secret 文件或 CI Secret。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：docs/04-runbooks/secrets-policy.md; .env.local
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-036** [AGENT][P1][W2][yes] 增加 `scripts/prune-local.sh`，用于安全清理本地卷、网络、链状态与演示数据，避免误删开发者其他容器。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：scripts/prune-local.sh
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警）
- **ENV-037** [AGENT][P1][W2][yes] 增加 `scripts/export-local-config.sh`，把当前 compose 配置解析为只读快照，方便不同 AI Agent 对齐本地环境。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：scripts/export-local-config.sh
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警）
- **ENV-038** [AGENT][P1][W2][yes] 增加 `docs/04-runbooks/port-matrix.md`，维护所有端口、URL、默认用户名、默认密码、初始 bucket、初始 topic 的矩阵表。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：docs/04-runbooks/port-matrix.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警）
- **ENV-039** [AGENT][P1][W2][yes] 补充基础服务故障排查手册，分别覆盖 PostgreSQL/Kafka/Keycloak/MinIO/OpenSearch/Fabric 启动失败的诊断步骤。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警）
- **ENV-040** [AGENT][P1][W2][yes] 为本地环境建立 smoke test 套件，至少校验：数据库可迁移、bucket 已创建、realm 已导入、topic 已存在、Grafana 可登录、Mock Payment 可回调。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：infra/**; docs/04-runbooks/**; scripts/**; fixtures/local/**
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-demo`（或至少 `core + observability + mocks` 组合）后，`ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7430（30. 日志、可观测性与告警） | ../04-runbooks/local-startup.md:L1（本地启动顺序）
- **ENV-041** [AGENT][P1][W2][yes] 在 `fixtures/local/` 下准备五条标准链路所需最小演示数据：企业主体、卖方、买方、产品、SKU、模板、订单、支付与交付样例。
  依赖：BOOT-001; BOOT-002; BOOT-003; BOOT-004
  交付：fixtures/local/
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/09-通用基础能力：身份、密钥、存储、日志、监控与运维.md:L61（9.5 运维控制要求） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **ENV-042** [ARCH][P0][W0][yes] 编写 `docs/04-runbooks/compose-boundary.md`，明确 `docker-compose.local.yml` 负责的中间件服务清单、可选 profile、端口矩阵与不纳入 compose 的业务进程，避免不同 Agent 自行把应用容器塞进基础设施编排。
  依赖：ENV-001; CTX-020
  交付：docs/04-runbooks/compose-boundary.md; docker-compose.local.yml
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-043** [ARCH][P1][W2][yes] 预留 `infra/docker/docker-compose.apps.local.example.yml` 占位文件，仅用于后续需要容器化联调 `platform-core`、`fabric-adapter`、`notification-worker`、`outbox-publisher`、`search-indexer` 时参考；当前 V1 第一阶段默认不用它阻塞开发。
  依赖：ENV-042; CORE-032
  交付：infra/docker/docker-compose.apps.local.example.yml
  完成定义：compose/脚本可执行；healthcheck 与自检通过；runbook 已更新；至少一条 smoke test 成功。
  验收：执行 `make up-local` 或等价脚本后，`scripts/check-local-stack.sh` 通过。
  阻塞风险：本地基础设施不稳定会阻塞所有联调和测试。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L5（14.1 环境规划） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构）
- **ENV-044** [ARCH][P0][W0][yes] 先拆分并收敛 PostgreSQL 服务块，包含卷、健康检查、初始化脚本挂载与端口。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：对应服务块已写入 compose，并能被 `docker compose config` 解析。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../数据库设计/README.md:L67（4. 迁移策略） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）
- **ENV-045** [ARCH][P0][W0][yes] 拆分并收敛 Redis 服务块，明确内存、持久化和健康检查配置。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：Redis 服务块已写入 compose，并能单独启动与健康检查。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **ENV-046** [ARCH][P0][W0][yes] 拆分并收敛 Kafka 服务块，明确 broker、kraft、topic 初始化前置条件。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：Kafka 服务块已写入 compose，并能单独启动与健康检查。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/事件模型与Topic清单正式版.md:L170（V1 Topic 清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）
- **ENV-047** [ARCH][P0][W0][yes] 拆分并收敛 MinIO 服务块，明确 bucket 初始化和凭证变量。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：MinIO 服务块已写入 compose，并能单独启动与健康检查。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/数据商品存储与分层存储设计.md:L130（4. 分层存储区设计）
- **ENV-048** [ARCH][P0][W0][yes] 拆分并收敛 OpenSearch 服务块，明确索引别名、内存限制与单节点模式。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：OpenSearch 服务块已写入 compose，并能单独启动与健康检查。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）
- **ENV-049** [ARCH][P0][W0][yes] 拆分并收敛 Keycloak 服务块，明确 realm 导入、client 初始化和管理端口。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：Keycloak 服务块已写入 compose，并能单独启动与健康检查。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L1（身份认证、注册登录与会话管理设计） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）
- **ENV-050** [ARCH][P0][W0][yes] 拆分并收敛 OTel Collector、Prometheus、Alertmanager 服务块。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：OTel/Prometheus/Alertmanager 服务块已写入 compose，并能单独启动。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **ENV-051** [ARCH][P0][W0][yes] 拆分并收敛 Grafana、Loki、Tempo 服务块。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：Grafana/Loki/Tempo 服务块已写入 compose，并能单独启动。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **ENV-052** [ARCH][P0][W0][yes] 拆分并收敛 mock-payment-provider 服务块，明确回调端口和本地演练变量。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：Mock Payment Provider 服务块已写入 compose，并明确只属于 `mocks/demo`；执行 `make up-mocks` 或 `make up-demo` 后可单独启动，不污染默认 `core`。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../开发准备/服务清单与服务边界正式版.md:L568（外围进程） | ../04-runbooks/compose-profiles.md:L1（Compose profile 边界）
- **ENV-053** [ARCH][P0][W0][yes] 拆分并收敛 Fabric 测试网络服务块或其外部脚本调用边界。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 分组已拆出；服务定义清晰；可被总 compose 聚合。
  验收：Fabric 测试网络块已写入 compose 或兼容脚本说明已补齐。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../开发准备/服务清单与服务边界正式版.md:L568（外围进程）
- **ENV-054** [ARCH][P0][W0][yes] 抽取并统一 compose 的 network、volume、restart、healthcheck、resource limit 策略。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/docker-compose.local.yml
  完成定义：compose 公共配置已收口。
  验收：network/volume/restart/healthcheck 策略已统一配置。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）
- **ENV-055** [ARCH][P0][W0][yes] 拆出 compose 环境变量样例和 override 示例，统一端口、初始凭证、bucket、topic、realm、索引别名。
  依赖：BOOT-003; BOOT-004
  交付：infra/docker/.env.local; infra/docker/docker-compose.local.override.example.yml
  完成定义：compose 变量和 override 示例已收口。
  验收：环境变量示例和 override 示例都可被开发者直接复用。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/配置项与密钥管理清单.md:L1（配置项与密钥管理清单） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）
- **ENV-056** [ARCH][P0][W0][yes] 补齐等待依赖与本地栈 smoke check 脚本，明确业务应用启动前的依赖检查。
  依赖：BOOT-003; BOOT-004
  交付：scripts/wait-for-services.sh; scripts/check-local-stack.sh
  完成定义：本地启动顺序脚本已收口。
  验收：等待脚本与本地栈检查脚本可执行，并能为后续任务复用。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/测试用例矩阵正式版.md:L1（测试用例矩阵正式版） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）
- **ENV-057** [ARCH][P0][W0][yes] 把 compose 服务分块、启动顺序、兼容现有 `部署脚本/` 的迁移说明写入本地启动 runbook。
  依赖：BOOT-003; BOOT-004
  交付：docs/04-runbooks/local-startup.md
  完成定义：本地启动 runbook 已收口。
  验收：runbook 与最终 compose 一致，能够指导从零启动本地环境。
  阻塞风险：本地环境未分块会导致 compose 维护困难。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境）

- **ENV-058** [AGENT][P0][W1][yes] 收敛配置项与资源命名：冻结数据库 / MinIO / Keycloak 的 bootstrap 与运行时入口映射，统一 `DATABASE_URL / MINIO_* / KEYCLOAK_* / BUCKET_* / INDEX_ALIAS_* / Redis key / compose 主入口` 口径，并同步文档、样例、脚本与 runbook。
  依赖：ENV-011; ENV-013; ENV-016; ENV-017; CORE-028
  交付：docs/开发准备/**; infra/**; docs/04-runbooks/**; scripts/**; apps/platform-core/src/**
  完成定义：配置清单、本地样例、初始化脚本、运行时入口说明与 runbook 已统一使用同一套数据库 / MinIO / Keycloak 映射、bucket、index alias、Redis key 与 compose 主入口命名；旧的 `PG_*` 主配置名与 `KEYCLOAK_ADMIN_USERNAME` 已退回历史兼容说明，不再作为正式默认口径。
  验收：既有本地 smoke 或手工校验通过，并能证明 `DATABASE_URL`、`MINIO_ACCESS_KEY / MINIO_SECRET_KEY`、`KEYCLOAK_REALM` 与 `.env.local`、compose bootstrap、runbook、脚本说明一致。
  阻塞风险：命名漂移会导致环境配置、资源初始化与运维动作对错对象。
  技术参考：../开发准备/配置项与密钥管理清单.md:L1（配置项与密钥管理清单） | ../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../04-runbooks/local-startup.md:L1（Local Startup） | ../04-runbooks/secrets-policy.md:L1（Local Secrets Policy） | ../开发准备/事件模型与Topic清单正式版.md:L1（事件模型与 Topic 清单） | 问题修复任务/A12-配置项与资源命名漂移.md:L1（配置项与资源命名漂移）
## 7. platform-core 基础骨架（这一组建议由我先做） [CORE]

这一组负责在现有 `platform-core` 最小骨架基础上补齐统一运行时、共享 crate 与模块模板。

- **CORE-001** [ARCH][P0][W0][yes] 初始化 `apps/platform-core/` Rust workspace，至少包含 `bin/platform-core` 与若干内部 crate：`kernel`、`config`、`http`、`db`、`auth`、`audit-kit`、`outbox-kit`、`provider-kit`。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-002** [ARCH][P0][W0][yes] 创建 `crates/kernel`，定义应用启动器、模块注册器、依赖注入容器、生命周期钩子与 shutdown 流程。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-003** [ARCH][P0][W0][yes] 创建 `crates/config`，统一读取环境变量、配置文件、运行模式与 provider 选择，保证 `local/staging/demo` 切换只靠配置。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-004** [ARCH][P0][W0][yes] 创建 `crates/http`，封装 HTTP server、router、middleware、分页、错误映射、统一响应结构与健康检查接口。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-005** [ARCH][P0][W0][yes] 创建 `crates/db`，封装 PostgreSQL 连接池、事务 helper、迁移执行器、只读查询与写事务边界。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-006** [ARCH][P0][W0][yes] 创建 `crates/auth`，封装 Keycloak/JWT 解析、会话主体提取、权限检查中间件、step-up 占位接口。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-007** [ARCH][P0][W0][yes] 创建 `crates/audit-kit`，统一审计事件写入接口、证据清单挂接接口、审计上下文传播与导出申请记录。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-008** [ARCH][P0][W0][yes] 创建 `crates/outbox-kit`，统一写 `outbox_event` 的 API、事件 envelope、幂等键、发布状态、重试策略。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-009** [ARCH][P0][W0][yes] 创建 `crates/provider-kit`，定义 KYC、签章、支付、通知、Fabric 写入等 Provider trait，要求每个 trait 至少有 `mock` 与 `real` 实现入口。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-010** [ARCH][P0][W0][yes] 在 `platform-core` 现有最小骨架基础上，补齐并校准 `src/modules/` 目录及模块模板：`iam`、`party`、`access`、`catalog`、`contract_meta`、`listing`、`review`、`order`、`contract`、`authorization`、`delivery`、`billing`、`dispute`、`audit`、`consistency`、`search`、`recommendation`、`developer`、`ops`。
  依赖：CORE-033; CORE-034; CORE-035; CORE-036; CORE-037; CORE-038; CORE-039; CORE-040; CORE-041; CORE-042; CORE-043; CORE-044; CORE-045; CORE-046; CORE-047; CORE-048; CORE-049; CORE-050; CORE-051
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：模块目录与模板齐备；主应用可编译；现有骨架已并入统一模块结构。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-011** [ARCH][P0][W0][yes] 为每个模块建立统一子目录：`api/`、`application/`、`domain/`、`repo/`、`dto/`、`events/`、`tests/`，避免后续模块风格失控。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L37（6.3 模块设计） | ../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分）
- **CORE-012** [ARCH][P0][W0][yes] 实现统一 `AppError` / `ErrorCode` / `ErrorResponse` 体系，错误码从 `docs/01-architecture/error-codes.md` 生成或校验。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：docs/01-architecture/error-codes.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L34（12.3 消息总线与异步任务） | ../领域模型/全量领域模型与对象关系说明.md:L1298（4.12 开发与调试支持聚合）
- **CORE-013** [ARCH][P0][W0][yes] 实现请求级中间件：`request_id` 生成/透传、日志上下文、trace 注入、租户上下文解析、访问日志。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L34（12.3 消息总线与异步任务） | ../领域模型/全量领域模型与对象关系说明.md:L1298（4.12 开发与调试支持聚合）
- **CORE-014** [ARCH][P0][W0][yes] 实现统一 DB 事务模板：业务对象修改、审计事件写入、outbox 事件写入必须支持在一个事务里完成。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L34（12.3 消息总线与异步任务） | ../领域模型/全量领域模型与对象关系说明.md:L1298（4.12 开发与调试支持聚合）
- **CORE-015** [ARCH][P0][W0][yes] 实现统一分页与筛选组件，供目录搜索、订单列表、审计列表、ops 列表复用。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合）
- **CORE-016** [ARCH][P0][W0][yes] 实现统一健康检查：`/health/live`、`/health/ready`、`/health/deps`，覆盖 DB/Redis/Kafka/MinIO/Keycloak/Fabric Adapter 可达性。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合）
- **CORE-017** [ARCH][P0][W0][yes] 实现统一模式页 `/internal/runtime`，返回当前 `mode`、provider 选择、版本号、Git SHA、迁移版本。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合）
- **CORE-018** [ARCH][P0][W0][yes] 实现统一幂等键中间件，支持订单创建、支付锁定、回调处理、模板执行、重试场景。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../权限设计/后端鉴权中间件规则说明.md:L191（5. 异步任务鉴权） | ../全集成文档/数据交易平台-全集成基线-全阶段.md:L5476（回调与外部回执幂等要求） | ../领域模型/全量领域模型与对象关系说明.md:L1507（8. 权限与作用域模型）
- **CORE-019** [ARCH][P0][W0][yes] 实现统一审计注解机制，确保 handler 层可声明审计动作、风险等级、对象类型、对象 ID、结果状态。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../权限设计/后端鉴权中间件规则说明.md:L244（7. 审计要求） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../领域模型/全量领域模型与对象关系说明.md:L1507（8. 权限与作用域模型）
- **CORE-020** [ARCH][P0][W0][yes] 实现统一权限门面，避免业务 handler 直接调用 Keycloak；真正的业务放行仍在应用层/访问控制层执行。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L1700（14.10.1 V1-Core（首笔标准交易闭环必需）） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议）
- **CORE-021** [ARCH][P0][W0][yes] 实现统一时间与 ID 策略：UTC 存储、展示层本地化；关键对象主键统一 UUID，外部可读编号独立生成。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分） | ../领域模型/全量领域模型与对象关系说明.md:L1332（5. 核心对象关系图） | ../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计）
- **CORE-022** [ARCH][P0][W0][yes] 实现 `platform-core` 启动时的模块自检，校验必要配置、Provider 绑定、关键 topic、bucket 与索引别名存在。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分） | ../领域模型/全量领域模型与对象关系说明.md:L1332（5. 核心对象关系图） | ../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计）
- **CORE-023** [ARCH][P0][W0][yes] 实现本地开发调试页面或 JSON 端点 `/internal/dev/trace-links`，给出 Grafana/Loki/Tempo/Keycloak/MinIO/OpenSearch 链接。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分） | ../领域模型/全量领域模型与对象关系说明.md:L1332（5. 核心对象关系图） | ../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计）
- **CORE-024** [AGENT][P0][W0][yes] 为 `platform-core` 添加基础单元测试框架、测试数据库支持、事务回滚测试夹具。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L76（3. 顶层领域划分） | ../领域模型/全量领域模型与对象关系说明.md:L1332（5. 核心对象关系图） | ../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计）
- **CORE-025** [AGENT][P0][W0][yes] 为 `platform-core` 添加 SQLx/SeaORM/最终选定库的查询编译检查流程，防止 schema 变化后运行期才报错。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L5（12.1 API 设计规范） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型）
- **CORE-026** [AGENT][P0][W0][yes] 为 `platform-core` 添加 OpenAPI 自动导出或校验骨架，确保领域接口实现与 `packages/openapi/*.yaml` 不漂移。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：packages/openapi/*.yaml
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L5（12.1 API 设计规范） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型）
- **CORE-027** [AGENT][P1][W2][yes] 加入 feature flags 机制，用于控制演示功能、公链锚定、真实 Provider、敏感场景实验特性。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **CORE-028** [AGENT][P1][W2][yes] 加入基于 trait 的仓储接口与内存假实现，便于业务规则测试先于基础设施联调。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **CORE-029** [AGENT][P1][W2][yes] 加入统一领域事件总线（进程内），用于模块间解耦；但要明确真正的跨进程副作用仍依赖 DB outbox + Kafka。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **CORE-030** [AGENT][P1][W2][yes] 为 `platform-core` 添加开发者首页 `/internal/dev/overview`，展示运行模式、最近 outbox、最近 dead letter、最近链回执。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **CORE-031** [AGENT][P1][W2][yes] 增加 `cargo xtask` 或等价工具，用于一键执行格式化、lint、OpenAPI 校验、迁移检查、seed 导入。
  依赖：BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001
  交付：apps/platform-core/**; docs/01-architecture/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **CORE-032** [ARCH][P0][W0][yes] 在 `docs/01-architecture/service-runtime-map.md` 固化运行时拓扑：哪些能力在 `platform-core` 模块内，哪些是外围独立进程，哪些只保留接口/trait 占位；并给每个模块标注同步边界、异步边界和所有权。
  依赖：CTX-019; CTX-020; CORE-010
  交付：docs/01-architecture/service-runtime-map.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理）
- **CORE-033** [ARCH][P0][W0][yes] 补齐身份基础模块骨架：`iam`、`party`、`access`，并统一模块目录模板。
  依赖：BOOT-002; ENV-001
  交付：apps/platform-core/src/modules/iam/**; apps/platform-core/src/modules/party/**; apps/platform-core/src/modules/access/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **CORE-034** [ARCH][P0][W0][yes] 补齐供给侧模块骨架：`catalog`、`contract_meta`、`listing`、`review`。
  依赖：BOOT-002; ENV-001
  交付：apps/platform-core/src/modules/catalog/**; apps/platform-core/src/modules/contract_meta/**; apps/platform-core/src/modules/listing/**; apps/platform-core/src/modules/review/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **CORE-035** [ARCH][P0][W0][yes] 补齐交易前半段模块骨架：`order`、`contract`。
  依赖：BOOT-002; ENV-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **CORE-036** [ARCH][P0][W0][yes] 补齐授权与交付模块骨架：`authorization`、`delivery`。
  依赖：BOOT-002; ENV-001
  交付：apps/platform-core/src/modules/authorization/**; apps/platform-core/src/modules/delivery/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **CORE-037** [ARCH][P0][W0][yes] 补齐支付与争议模块骨架：`billing`、`dispute`。
  依赖：BOOT-002; ENV-001
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **CORE-038** [ARCH][P0][W0][yes] 补齐审计与一致性模块骨架：`audit`、`consistency`。
  依赖：BOOT-002; ENV-001
  交付：apps/platform-core/src/modules/audit/**; apps/platform-core/src/modules/consistency/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **CORE-039** [ARCH][P0][W0][yes] 补齐搜索与推荐模块骨架：`search`、`recommendation`。
  依赖：BOOT-002; ENV-001
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../原始PRD/商品搜索、排序与索引同步设计.md:L319（10.1 核心原则） | ../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论）
- **CORE-040** [ARCH][P0][W0][yes] 补齐开发与运维模块骨架：`developer`、`ops`。
  依赖：BOOT-002; ENV-001
  交付：apps/platform-core/src/modules/developer/**; apps/platform-core/src/modules/ops/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案）
- **CORE-041** [ARCH][P0][W0][yes] 创建并校准 `crates/kernel`，收口共享类型、ID、时间、错误、分页等基础能力。
  依赖：BOOT-002; ENV-001
  交付：crates/kernel
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L132（5.1 `apps/platform-core`） | ../开发准备/技术选型正式版.md:L53（4. 语言分工） | ../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计）
- **CORE-042** [ARCH][P0][W0][yes] 创建并校准 `crates/config` 与 `crates/http`，收口模式加载、配置 schema、HTTP 入口与响应包装。
  依赖：BOOT-002; ENV-001
  交付：crates/config; crates/http
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L132（5.1 `apps/platform-core`） | ../开发准备/配置项与密钥管理清单.md:L1（配置项与密钥管理清单） | ../开发准备/接口清单与OpenAPI-Schema冻结表.md:L1（接口清单与OpenAPI-Schema冻结表）
- **CORE-043** [ARCH][P0][W0][yes] 创建并校准 `crates/db` 与 `crates/auth`，收口数据库接入、事务模板、OIDC/会话基础能力。
  依赖：BOOT-002; ENV-001
  交付：crates/db; crates/auth
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L132（5.1 `apps/platform-core`） | ../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L1（身份认证、注册登录与会话管理设计）
- **CORE-044** [ARCH][P0][W0][yes] 创建并校准 `crates/audit-kit`、`crates/outbox-kit`、`crates/provider-kit`。
  依赖：BOOT-002; ENV-001
  交付：crates/audit-kit; crates/outbox-kit; crates/provider-kit
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/仓库拆分与目录结构建议.md:L132（5.1 `apps/platform-core`） | ../开发准备/事件模型与Topic清单正式版.md:L1（事件模型与Topic清单正式版） | ../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`）
- **CORE-045** [ARCH][P0][W0][yes] 为各业务模块统一建立 `api/application/domain/repo/dto/events/tests` 子目录模板，避免模块风格漂移。
  依赖：CORE-033; CORE-034; CORE-035; CORE-036; CORE-037; CORE-038; CORE-039; CORE-040
  交付：apps/platform-core/src/modules/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案） | ../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../全集成文档/数据交易平台-全集成基线-V1.md:L19294（7. 技术架构与服务设计）
- **CORE-046** [ARCH][P0][W0][yes] 把共享 crate 与业务模块、外围进程的运行时所有权和依赖写入 `service-runtime-map`。
  依赖：CORE-041; CORE-042; CORE-043; CORE-044
  交付：docs/01-architecture/service-runtime-map.md
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`） | ../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议）
- **CORE-047** [ARCH][P0][W0][yes] 拆分并收敛请求级中间件链：`request_id`、日志上下文、trace、租户上下文、错误响应。
  依赖：CORE-042; CORE-043
  交付：apps/platform-core/src/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/接口清单与OpenAPI-Schema冻结表.md:L1（接口清单与OpenAPI-Schema冻结表） | ../开发准备/统一错误码字典正式版.md:L1（统一错误码字典正式版） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范）
- **CORE-048** [ARCH][P0][W0][yes] 拆分并收敛健康检查与运行时信息端点：`/health/live`、`/health/ready`、`/health/deps`、`/internal/runtime`。
  依赖：CORE-043; CORE-047
  交付：apps/platform-core/src/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案） | ../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/服务清单与服务边界正式版.md:L165（5. 主应用 `platform-core`）
- **CORE-049** [ARCH][P0][W0][yes] 拆分并收敛 Provider trait：KYC、签章、支付、通知、Fabric 提交等，每类至少有 `mock` 与 `real` 入口。
  依赖：CORE-044
  交付：crates/provider-kit
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L568（外围进程） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议） | ../原始PRD/支付、资金流与轻结算设计.md:L1（支付、资金流与轻结算设计）
- **CORE-050** [ARCH][P0][W0][yes] 拆分并收敛内部调试端点：`/internal/dev/trace-links`、`/internal/dev/overview`。
  依赖：CORE-038; CORE-047; CORE-048
  交付：apps/platform-core/src/**
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/平台总体架构设计草案.md:L1（平台总体架构设计草案） | ../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **CORE-051** [ARCH][P0][W0][yes] 收敛编译、测试、OpenAPI 校验、迁移检查的一键入口，保证 `platform-core` 骨架可重复验证。
  依赖：CORE-041; CORE-042; CORE-043; CORE-044
  交付：apps/platform-core/**; Makefile
  完成定义：代码可编译并接入主应用启动；依赖边界符合约束；基础单测通过；必要文档已同步。
  验收：`cargo build && cargo test` 通过，`/health/ready` 返回成功。
  阻塞风险：边界未冻结会导致后续目录、模块和命名漂移。
  技术参考：../开发准备/本地开发环境与中间件部署清单.md:L1（本地开发环境与中间件部署清单） | ../开发准备/测试用例矩阵正式版.md:L1（测试用例矩阵正式版） | ../开发准备/技术选型正式版.md:L121（5. 服务拆分建议）

## 8. 数据库与 Migration 落地 [DB]

这一组要把状态机、SKU、场景和计费触发点变成可查询、可回放、可 seed 的事实源。

- **DB-001** [ARCH][P0][W0][yes] 在 `db/migrations/v1/` 建立升级/回滚命名规则，并让 migration runner 按数字顺序执行、记录 checksum、支持 dry-run。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/migrations/v1/
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/README.md:L67（4. 迁移策略） | ../数据库设计/数据库设计总说明.md:L176（7. 迁移执行顺序） | ../数据库设计/数据库设计总说明.md:L45（3. 设计原则）
- **DB-002** [ARCH][P0][W0][yes] 创建 `001_extensions_and_schemas.sql`，初始化业务 schema、公共函数、更新时间 trigger、基础扩展。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：001_extensions_and_schemas.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/README.md:L67（4. 迁移策略） | ../数据库设计/数据库设计总说明.md:L45（3. 设计原则） | ../数据库设计/V1/upgrade/001_extensions_and_schemas.sql:L1（V1 migration）
- **DB-003** [ARCH][P0][W0][yes] 创建 `010_identity_and_access.sql`，落地租户、组织主体、部门、用户、应用、连接器、执行环境、角色、权限、邀请、会话、设备等表。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：010_identity_and_access.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/010_identity_and_access.sql:L1（V1 migration） | ../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合）
- **DB-004** [ARCH][P0][W0][yes] 创建 `020_catalog_contract.sql`，落地数据资源、数据产品、SKU、模板绑定、数字合约元数据与基础目录对象。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：020_catalog_contract.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/020_catalog_contract.sql:L1（V1 migration） | ../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合）
- **DB-005** [ARCH][P0][W0][yes] 创建 `025_review_workflow.sql`，落地主体审核、产品审核、合规审核、审批节点、意见、阻断与恢复记录。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：025_review_workflow.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/025_review_workflow.sql:L1（V1 migration） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **DB-006** [ARCH][P0][W0][yes] 创建 `030_trade_delivery.sql`，落地订单、交付、下载票据、共享授权、订阅、查询授权、沙箱工作区、报告工件等表。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：030_trade_delivery.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/030_trade_delivery.sql:L1（V1 migration） | ../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合）
- **DB-007** [ARCH][P0][W0][yes] 创建 `040_billing_support_risk.sql`，落地支付意图、账单事件、结算、退款、赔付、争议、风险处置、冻结记录。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：040_billing_support_risk.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合）
- **DB-008** [ARCH][P0][W0][yes] 创建 `050_audit_search_dev_ops.sql`，落地审计事件、证据对象、证据包、搜索同步、开发者应用日志、ops 作业表。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：050_audit_search_dev_ops.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/050_audit_search_dev_ops.sql:L1（V1 migration） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合）
- **DB-009** [ARCH][P0][W0][yes] 创建 `055_audit_hardening.sql`，补强审计保全、导出申请、legal hold、回放任务、锚定批次、保留策略。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：055_audit_hardening.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/055_audit_hardening.sql:L1（V1 migration） | ../原始PRD/审计、证据链与回放设计.md:L140（5. 证据模型）
- **DB-010** [ARCH][P0][W0][yes] 创建 `056_dual_authority_consistency.sql`，落地双层权威与一致性表：外部事实回执、状态镜像、对账记录、修复任务、异常状态。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：056_dual_authority_consistency.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/056_dual_authority_consistency.sql:L1（V1 migration） | ../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event）
- **DB-011** [ARCH][P0][W0][yes] 创建 `057_search_sync_architecture.sql`，落地搜索投影、索引作业、别名切换记录、重建任务、缓存失效记录。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：057_search_sync_architecture.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/057_search_sync_architecture.sql:L1（V1 migration） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计）
- **DB-012** [ARCH][P0][W0][yes] 创建 `058_recommendation_module.sql`，落地推荐位、推荐配置、曝光/点击行为流落表、候选缓存与重建任务表。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：058_recommendation_module.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/058_recommendation_module.sql:L1（V1 migration） | ../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论）
- **DB-013** [ARCH][P0][W0][yes] 创建 `059_logging_observability.sql`，落地日志镜像索引、观测查询作业、告警事件、事故工单、SLO 定义。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：059_logging_observability.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/059_logging_observability.sql:L1（V1 migration） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **DB-014** [ARCH][P0][W0][yes] 创建 `060_seed_authz_v1.sql`，预置基础权限点、系统角色、菜单映射、默认租户角色模板。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：060_seed_authz_v1.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/060_seed_authz_v1.sql:L1（V1 migration） | ../权限设计/权限点清单.md:L94（4. V1 基础权限点）
- **DB-015** [ARCH][P0][W0][yes] 创建 `061_data_object_trade_modes.sql`，补齐对象家族、交易方式、交付模式、权利集、SKU 真值映射。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：061_data_object_trade_modes.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/061_data_object_trade_modes.sql:L1（V1 migration） | ../原始PRD/数据对象产品族与交付模式增强设计.md:L292（4. 七类标准交易方式）
- **DB-016** [ARCH][P0][W0][yes] 创建 `062_data_product_metadata_contract.sql`，落地元数据档案、字段定义、质量报告、数据契约、法律依据证明。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：062_data_product_metadata_contract.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/062_data_product_metadata_contract.sql:L1（V1 migration） | ../原始PRD/数据商品元信息与数据契约设计.md:L112（4. 十大元信息域）
- **DB-017** [ARCH][P0][W0][yes] 创建 `063_raw_processing_pipeline.sql`，落地原始接入批次、原始对象清单、格式识别、抽取标准化任务、加工责任链任务。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：063_raw_processing_pipeline.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1（V1 migration） | ../原始PRD/数据原样处理与产品化加工流程设计.md:L189（4. 交易前正式流程）
- **DB-018** [ARCH][P0][W0][yes] 创建 `064_storage_layering_architecture.sql`，落地存储分层对象、对象 URI、托管方式、custody profile、交付对象清单。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：064_storage_layering_architecture.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/064_storage_layering_architecture.sql:L1（V1 migration） | ../原始PRD/数据商品存储与分层存储设计.md:L130（4. 分层存储区设计）
- **DB-019** [ARCH][P0][W0][yes] 创建 `065_query_execution_plane.sql`，落地 QuerySurface、QueryTemplate、TemplateVersion、QueryGrant、QueryRun、执行结果对象。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：065_query_execution_plane.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/065_query_execution_plane.sql:L1（V1 migration） | ../原始PRD/数据商品查询与执行面设计.md:L35（3. 四个核心对象）
- **DB-020** [ARCH][P0][W0][yes] 创建 `066_sensitive_data_controlled_delivery.sql`，落地敏感处理策略、安全预览、执行策略快照、结果披露审查、销毁/保留证明。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：066_sensitive_data_controlled_delivery.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/066_sensitive_data_controlled_delivery.sql:L1（V1 migration） | ../原始PRD/敏感数据处理与受控交付设计.md:L124（5. 敏感数据完整交易链闭环）
- **DB-021** [ARCH][P0][W0][yes] 创建 `067_trade_chain_monitoring.sql`，落地交易链检查点、公平性事件、外部事实监控、投影缺口与监控策略。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：067_trade_chain_monitoring.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/067_trade_chain_monitoring.sql:L1（V1 migration） | ../原始PRD/交易链监控、公平性与信任安全设计.md:L174（5. 六层交易链监控模型）
- **DB-022** [ARCH][P0][W0][yes] 创建 `068_trade_chain_monitoring_authz.sql`，补齐交易监控相关权限、可见范围、对象授权映射。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：068_trade_chain_monitoring_authz.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/068_trade_chain_monitoring_authz.sql:L1（V1 migration） | ../权限设计/权限点清单.md:L94（4. V1 基础权限点）
- **DB-023** [ARCH][P0][W0][yes] 创建 `070_seed_role_permissions_v1.sql`，把最终 V1 角色权限种子固化为可重放脚本，并提供校验 SQL。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：070_seed_role_permissions_v1.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L3（1. 设计范围） | ../数据库设计/V1/upgrade/070_seed_role_permissions_v1.sql:L1（V1 migration） | ../权限设计/角色权限矩阵正式版.md:L88（7. V1 关键权限到角色映射）
- **DB-024** [ARCH][P0][W0][yes] 为每个 migration 同步编写 downgrade 脚本，至少支持本地重建、回滚演练与失败定位，不要求线上任意点回滚，但要能本地自洽。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/migrations/v1/**; db/seeds/**; db/scripts/**; docs/03-db/**
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/README.md:L67（4. 迁移策略） | ../数据库设计/数据库设计总说明.md:L176（7. 迁移执行顺序） | ../数据库设计/V1/downgrade/001_extensions_and_schemas.sql:L1（downgrade pattern）
- **DB-025** [AGENT][P0][W0][yes] 为 migrations 增加 `db/scripts/migrate-up.sh`、`migrate-down.sh`、`migrate-status.sh`、`migrate-reset.sh`。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/scripts/migrate-up.sh; migrate-down.sh; migrate-status.sh; migrate-reset.sh
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/README.md:L67（4. 迁移策略） | ../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **DB-026** [AGENT][P0][W0][yes] 创建 `db/seeds/001_base_lookup.sql`，预置枚举值、状态字典、产品类目、行业标签、风险等级、交付模式。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/seeds/001_base_lookup.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库表字典正式版.md:L544（5.3 `catalog` 数据资产、商品与元数据） | ../原始PRD/数据产品分类与交易模式详细稿.md:L155（6. V1 标准数据产品目录） | ../数据库设计/V1/upgrade/061_data_object_trade_modes.sql:L1（V1 migration）
- **DB-027** [AGENT][P0][W0][yes] 创建 `db/seeds/010_test_tenants.sql`，生成本地演示租户、卖方、买方、运营用户、审计员、开发者用户。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/seeds/010_test_tenants.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库表字典正式版.md:L245（5.1 `core` 身份与主体） | ../数据库设计/V1/upgrade/010_identity_and_access.sql:L1（V1 migration） | ../用户角色说明/全量用户角色说明.md:L88（4. 核心角色总览）
- **DB-028** [AGENT][P0][W0][yes] 创建 `db/seeds/020_test_products.sql`，生成八个标准 SKU 的基础商品与模板绑定示例。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/seeds/020_test_products.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库表字典正式版.md:L1043（`catalog.product_sku`） | ../数据库设计/V1/upgrade/061_data_object_trade_modes.sql:L1（V1 migration） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **DB-029** [AGENT][P0][W0][yes] 创建 `db/seeds/030_test_orders.sql`，生成 FILE_STD/FILE_SUB/SHARE_RO/API_SUB/API_PPU/QRY_LITE/SBX_STD/RPT_STD 全量订单样例，并额外覆盖五条标准链路的场景订单，用于页面联调和回放测试。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/seeds/030_test_orders.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库表字典正式版.md:L1321（5.6 `trade` 交易主链路） | ../数据库设计/V1/upgrade/030_trade_delivery.sql:L1（V1 migration） | ../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路）
- **DB-030** [AGENT][P1][W2][yes] 生成 `docs/03-db/table-catalog.md`，按 schema 输出表字典、主键、唯一键、外键、索引与对象职责。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：docs/03-db/table-catalog.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库表字典正式版.md:L243（5. 正式表字典） | ../数据库设计/表关系总图-ER文本图.md:L123（3. V1 业务主链路 ER 图） | ../数据库设计/数据库设计总说明.md:L3（1. 设计范围）
- **DB-031** [AGENT][P1][W2][yes] 生成 `docs/03-db/state-machine-to-table-map.md`，把订单/授权/交付/结算/争议状态机映射到具体字段。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：docs/03-db/state-machine-to-table-map.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../数据库设计/数据库表字典正式版.md:L1321（5.6 `trade` 交易主链路）
- **DB-032** [AGENT][P1][W2][yes] 建立数据库兼容性测试，保证新 migration 不破坏已有 seed、本地回滚后能重新升级。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/migrations/v1/**; db/seeds/**; db/scripts/**; docs/03-db/**
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库设计总说明.md:L176（7. 迁移执行顺序） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../数据库设计/README.md:L67（4. 迁移策略）
- **DB-033** [AGENT][P1][W2][yes] 建立索引审查任务，对高频查询路径（目录搜索回查、订单详情、审计联查、ops 列表）补充索引基线。
  依赖：BOOT-008; ENV-005; ENV-006; CORE-005
  交付：db/migrations/v1/**; db/seeds/**; db/scripts/**; docs/03-db/**
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../数据库设计/数据库表字典正式版.md:L2419（5.13 `search` 倒排与向量检索） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程） | ../全集成文档/数据交易平台-全集成基线-V1.md:L4824（44. 商品搜索、排序与索引同步设计）
- **DB-034** [AGENT][P0][W1][yes] 创建 `db/seeds/031_sku_trigger_matrix.sql`，把 8 个 SKU 的支付触发、交付触发、验收触发、计费触发、退款触发、争议冻结触发固化为可查询样例数据或配置表，供前后端、测试和审计共用。
  依赖：DB-029; CTX-021; BIL-023
  交付：db/seeds/031_sku_trigger_matrix.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（计费触发相关对象） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **DB-035** [AGENT][P0][W0][yes] 创建 `db/seeds/032_five_scenarios.sql`，把五条标准链路与主/补充 SKU、合同模板、验收模板、退款模板的映射固化为演示数据，禁止后续页面或测试使用自创场景名称替代官方场景名。
  依赖：DB-028; CTX-007; CTX-021
  交付：db/seeds/032_five_scenarios.sql
  完成定义：migration/seed 可执行；空库升级成功；回滚或重建可复现；关键约束与索引已验证。
  验收：空库 `migrate-up -> seed -> reset -> migrate-up` 全流程通过。
  阻塞风险：schema/seed 漂移会让接口和状态机无法闭环。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../业务流程/业务流程图-V1-完整版.md:L66（4. 主业务流程）

## 9. IAM / Party / Access 领域 [IAM]

这一组落地主体、身份、登录、会话与访问控制基础能力。

- **IAM-001** [AGENT][P0][W1][yes] 实现 `party` 模块的 Tenant/Organization 聚合：组织注册、组织状态、主体类型、司法辖区、认证等级、黑白灰名单引用。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../数据库设计/V1/upgrade/010_identity_and_access.sql:L1（V1 migration） | ../业务流程/业务流程图-V1-完整版.md:L68（4.1 主体准入与基础身份建立）
- **IAM-002** [AGENT][P0][W1][yes] 实现 Department/User/Application/Connector/ExecutionEnvironment 模型、仓储与最小 CRUD。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../数据库设计/V1/upgrade/010_identity_and_access.sql:L1（V1 migration） | ../业务流程/业务流程图-V1-完整版.md:L68（4.1 主体准入与基础身份建立）
- **IAM-003** [AGENT][P0][W1][yes] 实现 `iam` 模块的登录上下文镜像，接入 Keycloak token 解析、本地测试用户、基础 session 载荷。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L3（1. 目标） | ../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式）
- **IAM-004** [AGENT][P0][W1][yes] 实现 `POST /api/v1/orgs/register`，支持匿名或半匿名注册入口，记录风险画像与审计事件。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L3（1. 目标） | ../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式）
- **IAM-005** [AGENT][P0][W1][yes] 实现 `POST /api/v1/users/invite` 与 `POST /api/v1/iam/invitations` 系列接口，支持租户管理员邀请成员。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L3（1. 目标） | ../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式）
- **IAM-006** [AGENT][P0][W1][yes] 实现 `GET/POST /api/v1/iam/sessions*`、`/devices*`，落地最小会话管理与设备撤销。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L3（1. 目标） | ../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式）
- **IAM-007** [AGENT][P0][W1][yes] 实现 `POST /api/v1/apps`、`PATCH /api/v1/apps/{id}`、`GET /api/v1/apps/{id}`，支撑 API 产品与开发者应用绑定。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式） | ../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口）
- **IAM-008** [AGENT][P0][W1][yes] 实现应用密钥基础模型：client id、client secret、轮换、吊销、状态、归属租户。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式） | ../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口）
- **IAM-009** [AGENT][P0][W1][yes] 在 `access` 模块中落地角色、权限点、作用域、按钮/接口放行规则，不把业务放行直接下沉给 Keycloak。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../权限设计/接口权限校验清单.md:L53（3. V1 接口权限校验清单） | ../权限设计/权限点清单.md:L94（4. V1 基础权限点） | ../原始PRD/角色与权限矩阵详细稿.md:L103（5. 角色分层）
- **IAM-010** [AGENT][P0][W1][yes] 实现基础 RBAC 种子加载与权限校验中间件，覆盖租户、平台、审计、开发者四类角色。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../权限设计/接口权限校验清单.md:L53（3. V1 接口权限校验清单） | ../权限设计/权限点清单.md:L94（4. V1 基础权限点） | ../原始PRD/角色与权限矩阵详细稿.md:L103（5. 角色分层）
- **IAM-011** [AGENT][P0][W1][yes] 实现高风险操作 step-up 占位流程，对冻结、赔付、证据导出、重放、权限变更提供统一检查接口。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../权限设计/接口权限校验清单.md:L53（3. V1 接口权限校验清单） | ../权限设计/权限点清单.md:L94（4. V1 基础权限点） | ../原始PRD/角色与权限矩阵详细稿.md:L103（5. 角色分层）
- **IAM-012** [AGENT][P0][W1][yes] 实现基础 MFA 占位配置，local 模式允许 mock，再认证链路先走简化流程但 API 形态必须冻结。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L3（1. 目标） | ../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式）
- **IAM-013** [AGENT][P0][W1][yes] 实现企业 OIDC 连接占位接口 `POST /api/v1/iam/sso/connections`，即便 local 不启用真实联邦，也要先冻结配置模型。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L3（1. 目标） | ../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式）
- **IAM-014** [AGENT][P0][W1][yes] 实现 Fabric 身份镜像接口：`GET /api/v1/iam/fabric-identities`、签发/吊销接口占位、证书吊销占位。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座）
- **IAM-015** [AGENT][P0][W1][yes] 实现 `party` 与 `review` 的联动字段：主体审核状态、风控状态、可售状态、冻结原因。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../数据库设计/V1/upgrade/010_identity_and_access.sql:L1（V1 migration） | ../业务流程/业务流程图-V1-完整版.md:L68（4.1 主体准入与基础身份建立）
- **IAM-016** [AGENT][P0][W1][yes] 实现会话与设备审计：登录、登出、邀请、撤销、角色变更都落审计事件。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L3（1. 目标） | ../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式）
- **IAM-017** [AGENT][P1][W3][yes] 完成组织-部门-用户-应用-连接器-执行环境的列表与详情接口，用于后台管理页面联调。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../数据库设计/V1/upgrade/010_identity_and_access.sql:L1（V1 migration） | ../业务流程/业务流程图-V1-完整版.md:L68（4.1 主体准入与基础身份建立）
- **IAM-018** [AGENT][P1][W3][yes] 增加本地模式下的测试身份脚本，一键生成卖方管理员、买方管理员、运营管理员、审计员、开发者账号。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../数据库设计/V1/upgrade/010_identity_and_access.sql:L1（V1 migration） | ../业务流程/业务流程图-V1-完整版.md:L68（4.1 主体准入与基础身份建立）
- **IAM-019** [AGENT][P1][W3][yes] 为 IAM/Party/Access 编写集成测试：组织注册、邀请成员、创建应用、权限拒绝、会话撤销、设备撤销。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：apps/platform-core/src/modules/iam/**; packages/openapi/iam.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L96（4.1 身份与主体聚合） | ../数据库设计/V1/upgrade/010_identity_and_access.sql:L1（V1 migration） | ../业务流程/业务流程图-V1-完整版.md:L68（4.1 主体准入与基础身份建立）
- **IAM-020** [AGENT][P1][W3][yes] 生成 `docs/02-openapi/iam.yaml` 第一版并与实现校验。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018
  交付：docs/02-openapi/iam.yaml
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口） | ../原始PRD/身份认证、注册登录与会话管理设计.md:L3（1. 目标） | ../原始PRD/IAM 技术接入方案.md:L130（5. V1 推荐落地方式）

## 10. Catalog / Contract Meta / Listing / Review 领域 [CAT]

这一组落地商品目录、元信息、模板绑定、审核与可售状态。

- **CAT-001** [AGENT][P0][W1][yes] 实现 DataResource、AssetVersion、DataProduct、ProductSKU 基础模型与仓储。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **CAT-002** [AGENT][P0][W1][yes] 实现 `POST /api/v1/products`、`PATCH /api/v1/products/{id}`，支持创建与编辑商品草稿。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **CAT-003** [AGENT][P0][W1][yes] 实现 `POST /api/v1/products/{id}/skus`、`PATCH /api/v1/skus/{id}`，并校验标准 SKU 真值、trade_mode 合法性、模板兼容性。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **CAT-004** [AGENT][P0][W1][yes] 实现原始接入批次 `POST /api/v1/assets/{assetId}/raw-ingest-batches` 与清单维护接口。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据原样处理与产品化加工流程设计.md:L189（4. 交易前正式流程） | ../业务流程/业务流程图-V1-完整版.md:L157（4.2A 数据原样处理与产品化加工主流程） | ../数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1（V1 migration）
- **CAT-005** [AGENT][P0][W1][yes] 实现原始对象清单 `POST /api/v1/raw-ingest-batches/{id}/manifests`，支持对象 URI、hash、格式、大小、归属记录。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据原样处理与产品化加工流程设计.md:L189（4. 交易前正式流程） | ../业务流程/业务流程图-V1-完整版.md:L157（4.2A 数据原样处理与产品化加工主流程） | ../数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1（V1 migration）
- **CAT-006** [AGENT][P0][W1][yes] 实现格式识别与确认接口 `POST /api/v1/raw-object-manifests/{id}/detect-format`。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据原样处理与产品化加工流程设计.md:L189（4. 交易前正式流程） | ../业务流程/业务流程图-V1-完整版.md:L157（4.2A 数据原样处理与产品化加工主流程） | ../数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1（V1 migration）
- **CAT-007** [AGENT][P0][W1][yes] 实现抽取/标准化任务接口 `POST /api/v1/raw-object-manifests/{id}/extraction-jobs`，先以任务记录与状态机为主，不强耦合真实计算引擎。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据原样处理与产品化加工流程设计.md:L189（4. 交易前正式流程） | ../业务流程/业务流程图-V1-完整版.md:L157（4.2A 数据原样处理与产品化加工主流程） | ../数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1（V1 migration）
- **CAT-008** [AGENT][P0][W1][yes] 实现预览工件接口 `POST /api/v1/assets/{versionId}/preview-artifacts`，支持样例文件、schema 预览、预览掩码策略。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据原样处理与产品化加工流程设计.md:L189（4. 交易前正式流程） | ../业务流程/业务流程图-V1-完整版.md:L157（4.2A 数据原样处理与产品化加工主流程） | ../数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1（V1 migration）
- **CAT-009** [AGENT][P0][W1][yes] 实现元信息档案接口 `PUT /api/v1/products/{id}/metadata-profile`，覆盖十大元信息域的最小结构。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据商品元信息与数据契约设计.md:L112（4. 十大元信息域） | ../原始PRD/数据商品元信息与数据契约设计.md:L86（3.2 数据契约必须单独建模） | ../数据库设计/V1/upgrade/062_data_product_metadata_contract.sql:L1（V1 migration）
- **CAT-010** [AGENT][P0][W1][yes] 实现字段结构说明接口 `POST /api/v1/assets/{versionId}/field-definitions`。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据商品元信息与数据契约设计.md:L112（4. 十大元信息域） | ../原始PRD/数据商品元信息与数据契约设计.md:L86（3.2 数据契约必须单独建模） | ../数据库设计/V1/upgrade/062_data_product_metadata_contract.sql:L1（V1 migration）
- **CAT-011** [AGENT][P0][W1][yes] 实现质量报告接口 `POST /api/v1/assets/{versionId}/quality-reports`，保存指标、采样方式、报告 URI/hash。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据商品元信息与数据契约设计.md:L112（4. 十大元信息域） | ../原始PRD/数据商品元信息与数据契约设计.md:L86（3.2 数据契约必须单独建模） | ../数据库设计/V1/upgrade/062_data_product_metadata_contract.sql:L1（V1 migration）
- **CAT-012** [AGENT][P0][W1][yes] 实现加工责任链任务接口 `POST /api/v1/assets/{versionId}/processing-jobs`，记录输入来源、责任主体、处理摘要。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据原样处理与产品化加工流程设计.md:L189（4. 交易前正式流程） | ../业务流程/业务流程图-V1-完整版.md:L157（4.2A 数据原样处理与产品化加工主流程） | ../数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1（V1 migration）
- **CAT-013** [AGENT][P0][W1][yes] 实现数据契约接口 `POST /api/v1/skus/{id}/data-contracts` 与 `GET /api/v1/skus/{id}/data-contracts/{contractId}`。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据商品元信息与数据契约设计.md:L112（4. 十大元信息域） | ../原始PRD/数据商品元信息与数据契约设计.md:L86（3.2 数据契约必须单独建模） | ../数据库设计/V1/upgrade/062_data_product_metadata_contract.sql:L1（V1 migration）
- **CAT-014** [AGENT][P0][W1][yes] 实现可交付对象接口 `POST /api/v1/assets/{versionId}/objects`，区分原始对象、预览对象、交付对象、报告对象、结果对象。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据商品存储与分层存储设计.md:L155（5. 闭环逻辑） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../数据库设计/V1/upgrade/064_storage_layering_architecture.sql:L1（V1 migration）
- **CAT-015** [AGENT][P0][W1][yes] 实现版本发布/订阅策略接口 `PATCH /api/v1/assets/{assetId}/release-policy`。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/数据商品存储与分层存储设计.md:L155（5. 闭环逻辑） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../数据库设计/V1/upgrade/064_storage_layering_architecture.sql:L1（V1 migration）
- **CAT-016** [AGENT][P0][W1][yes] 实现商品提交审核接口 `POST /api/v1/products/{id}/submit`，校验必填字段、模板齐全、风控阻断条件。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **CAT-017** [AGENT][P0][W1][yes] 实现基础 Listing 状态机：草稿、待审核、已上架、已下架、已冻结。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **CAT-018** [AGENT][P0][W1][yes] 实现主体审核 `POST /api/v1/review/subjects/{id}`、产品审核 `POST /api/v1/review/products/{id}`、合规审核 `POST /api/v1/review/compliance/{id}`。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程） | ../数据库设计/V1/upgrade/025_review_workflow.sql:L1（V1 migration） | ../权限设计/接口权限校验清单.md:L53（3. V1 接口权限校验清单）
- **CAT-019** [AGENT][P0][W1][yes] 实现商品冻结/下架接口 `POST /api/v1/products/{id}/suspend`，对接 risk/product freeze 权限与审计。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程） | ../数据库设计/V1/upgrade/025_review_workflow.sql:L1（V1 migration） | ../权限设计/接口权限校验清单.md:L53（3. V1 接口权限校验清单）
- **CAT-020** [AGENT][P0][W1][yes] 实现卖方主页接口 `GET /api/v1/sellers/{orgId}/profile` 与商品详情接口 `GET /api/v1/products/{id}`。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **CAT-021** [AGENT][P0][W1][yes] 实现模板绑定接口 `POST /api/v1/products/{id}/bind-template`、`POST /api/v1/skus/{id}/bind-template`、`PATCH /api/v1/policies/{id}`，按 SKU 强校验模板族。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../原始PRD/数据对象产品族与交付模式增强设计.md:L292（4. 七类标准交易方式） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **CAT-022** [AGENT][P0][W1][yes] 实现商品与卖方搜索可见性字段，为后续搜索读模型同步打底。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../全集成文档/数据交易平台-全集成基线-V1.md:L4824（44. 商品搜索、排序与索引同步设计）
- **CAT-023** [AGENT][P1][W3][yes] 为五条标准链路分别准备标准商品模板、元信息模板、契约模板、审核样例。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../原始PRD/数据产品分类与交易模式详细稿.md:L155（6. V1 标准数据产品目录）
- **CAT-024** [AGENT][P1][W3][yes] 为 Catalog/Listing/Review 编写集成测试：商品创建、SKU 创建、质量报告、契约发布、提交审核、审核通过/驳回、冻结。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：apps/platform-core/src/modules/catalog/**; packages/openapi/catalog.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **CAT-025** [AGENT][P1][W3][yes] 生成 `docs/02-openapi/catalog.yaml` 第一版并与实现校验。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：docs/02-openapi/catalog.yaml
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程）
- **CAT-026** [AGENT][P1][W3][yes] 生成 `docs/05-test-cases/catalog-review-cases.md`，覆盖上架规则、字段缺失、模板不匹配、风险阻断。
  依赖：CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005
  交付：docs/05-test-cases/catalog-review-cases.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用；宿主机示例不再把 Kafka 误写成容器内 `9092`。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹；宿主机示例与端口矩阵保持一致。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L200（4.2 目录与商品聚合） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L86（4.2 商品创建、模板绑定与上架流程） | ../05-test-cases/README.md:L7（测试样例公共前置条件） | ../04-runbooks/local-startup.md:L39（宿主机 Kafka 地址） | ../04-runbooks/port-matrix.md:L15（Kafka 外部端口矩阵）

## 11. Order / Contract / Authorization 主交易链路 [TRADE]

这一组落地下单、合同、生命周期快照与主状态机推进。

- **TRADE-001** [AGENT][P0][W1][yes] 实现询报价/样例申请/POC 申请的最小数据模型，即便前台暂时简化，也要为订单前置动作保留结构。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-002** [AGENT][P0][W1][yes] 实现价格快照模型：订单创建时固化 SKU 价格、计费模式、结算口径、退款口径与税务信息快照。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-003** [AGENT][P0][W1][yes] 实现 `POST /api/v1/orders`，校验商品状态、价格快照、权限、风险阻断、审计。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-004** [AGENT][P0][W1][yes] 实现订单详情 `GET /api/v1/orders/{id}`，返回主状态与分层子状态。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-005** [AGENT][P0][W1][yes] 实现订单取消 `POST /api/v1/orders/{id}/cancel`，按状态机限制可取消阶段与退款分支。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-006** [AGENT][P0][W1][yes] 实现合同确认 `POST /api/v1/orders/{id}/contract-confirm`，关联数字合约快照与签署角色。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L530（4.3 合同与策略聚合） | ../页面说明书/页面说明书-V1-完整版.md:L522（6.2 合同确认页） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **TRADE-007** [AGENT][P0][W1][yes] 实现订单主状态机字段：`current_state`、`payment_status`、`delivery_status`、`acceptance_status`、`settlement_status`、`dispute_status`。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-008** [AGENT][P0][W1][yes] 实现文件交易状态机 `FILE_STD`：创建、待锁资、待交付、待验收、已完成、已退款/争议等路径。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-009** [AGENT][P0][W1][yes] 实现文件订阅状态机 `FILE_SUB`：订阅建立、周期交付、周期验收、暂停、到期、续订。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-010** [AGENT][P0][W1][yes] 实现 API 订阅状态机 `API_SUB`：锁资、应用绑定、密钥开通、试调用、正式可用、周期计费、终止。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-011** [AGENT][P0][W1][yes] 实现 API 按次付费状态机 `API_PPU`：授权、额度/计费口径、调用结算、到期或停用。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-012** [AGENT][P0][W1][yes] 实现只读共享状态机 `SHARE_RO`：共享开通、访问授权、撤销、到期、争议中断。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-013** [AGENT][P0][W1][yes] 实现模板查询状态机 `QRY_LITE`：模板授权、参数校验、执行、结果可取、验收关闭。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-014** [AGENT][P0][W1][yes] 实现查询沙箱状态机 `SBX_STD`：空间开通、账号/席位下发、执行、受限导出、到期或撤权。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-015** [AGENT][P0][W1][yes] 实现报告产品状态机 `RPT_STD`：任务建立、报告生成、报告交付、验收、结算。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-016** [AGENT][P0][W1][yes] 实现数字合约聚合：合同模板、合同快照、签署状态、签约主体、签署时间、摘要上链引用。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L530（4.3 合同与策略聚合） | ../原始PRD/数据商品元信息与数据契约设计.md:L86（3.2 数据契约必须单独建模） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **TRADE-017** [AGENT][P0][W1][yes] 实现授权聚合：Authorization、UsagePolicy、grant、revoke、expire、suspend、恢复。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L530（4.3 合同与策略聚合） | ../原始PRD/数据商品查询与执行面设计.md:L185（8. 与授权、计费、审计的关系） | ../原始PRD/敏感数据处理与受控交付设计.md:L124（5. 敏感数据完整交易链闭环）
- **TRADE-018** [AGENT][P0][W1][yes] 实现基础断权机制：订单取消、到期、风控冻结、争议升级后自动触发交付入口断权。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L530（4.3 合同与策略聚合） | ../原始PRD/数据商品查询与执行面设计.md:L185（8. 与授权、计费、审计的关系） | ../原始PRD/敏感数据处理与受控交付设计.md:L124（5. 敏感数据完整交易链闭环）
- **TRADE-019** [AGENT][P0][W1][yes] 实现生命周期摘要接口 `GET /api/v1/orders/{id}/lifecycle-snapshots`，返回对象化字段名而不是拼装字符串。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **TRADE-020** [AGENT][P0][W1][yes] 实现订单创建事务：业务对象 + 审计事件 + outbox 事件同事务落库。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-021** [AGENT][P0][W1][yes] 实现支付锁定前的前置校验：主体状态、商品状态、审核状态、模板齐备、价格快照完整。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-022** [AGENT][P0][W1][yes] 实现订单与合同/授权/交付/账单/争议的一对多或一对一关系装配器，便于详情页与审计联查。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-023** [AGENT][P0][W1][yes] 实现五条标准链路的订单模板：工业设备 API 订阅、质量日报文件包、供应链查询沙箱、零售 API/报告订阅、选址查询服务。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../业务流程/业务流程图-V1-完整版.md:L66（4. 主业务流程）
- **TRADE-024** [AGENT][P1][W1][yes] 为订单状态机补充拒绝非法回退保护，避免回调乱序导致状态倒退。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1445（7. 生命周期总表） | ../data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65（6.5 订单状态机） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **TRADE-025** [AGENT][P1][W3][yes] 为授权模块补充 scope/subject/resource/action 最小结构，便于未来接 OPA 但 V1 不强依赖 OPA。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L530（4.3 合同与策略聚合） | ../原始PRD/数据商品元信息与数据契约设计.md:L86（3.2 数据契约必须单独建模） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **TRADE-026** [AGENT][P1][W3][yes] 为合同模块补充电子签章 Provider 占位与 mock 实现；local 模式先不依赖真实签章服务。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L530（4.3 合同与策略聚合） | ../原始PRD/数据商品元信息与数据契约设计.md:L86（3.2 数据契约必须单独建模） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **TRADE-027** [AGENT][P1][W3][yes] 为主交易链路编写集成测试：下单、合同确认、锁资前校验、非法状态跳转、自动断权。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-028** [AGENT][P1][W2][yes] 生成 `docs/02-openapi/trade.yaml` 第一版并与实现校验。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：docs/02-openapi/trade.yaml
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-029** [AGENT][P1][W1][yes] 生成 `docs/05-test-cases/order-state-machine.md`，按 8 个标准 SKU 编写状态转换测试矩阵。
  依赖：CORE-014; DB-006; IAM-001; CAT-001
  交付：docs/05-test-cases/order-state-machine.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-030** [AGENT][P0][W1][yes] 实现支付结果到订单推进编排器：支付成功时把订单从“待锁资/待支付”推进到“已锁资/待交付”，支付失败推进到“支付失败待处理”，支付超时进入“支付超时待补偿/待取消”，并保证状态不可倒退。
  依赖：BIL-005; TRADE-007; CORE-014
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-031** [AGENT][P0][W1][yes] 实现“可交付判定器”，综合支付状态、合同状态、主体状态、商品审核状态、风控状态，只有全部满足时才创建交付任务并把订单推进到“待交付”；禁止支付成功后直接绕过前置校验进入已交付。
  依赖：TRADE-021; TRADE-030; CAT-010
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-032** [AGENT][P0][W1][yes] 实现五条标准链路的场景到 SKU 快照规则：一个场景可包含主 SKU 与补充 SKU，但订单、合同、授权、验收、结算仍必须按 SKU 单独快照，不允许只记录场景名。
  依赖：CTX-021; TRADE-023
  交付：apps/platform-core/src/modules/order/**; apps/platform-core/src/modules/contract/**; apps/platform-core/src/modules/authorization/**; packages/openapi/trade.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）
- **TRADE-033** [AGENT][P0][W1][yes] 输出 `docs/01-architecture/order-orchestration.md`，画清订单主状态、支付子状态、交付子状态、验收子状态、结算子状态、争议子状态之间的推进规则、互斥关系与回调乱序保护。
  依赖：TRADE-007; TRADE-024; BIL-022; DLV-029
  交付：docs/01-architecture/order-orchestration.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L620（4.4 交易与订单聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环）） | ../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程）

## 12. Delivery / Storage Gateway / Query Execution [DLV]

这一组落地授权、交付、下载、共享、查询与结果交付。

- **DLV-001** [AGENT][P0][W1][yes] 实现 `storage-gateway` 领域模型：对象定位、bucket/key、hash、watermark 策略、下载限制、访问审计。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合） | ../业务流程/业务流程图-V1-完整版.md:L270（4.4.1 文件类交付） | ../页面说明书/页面说明书-V1-完整版.md:L590（7.1 文件交付页）
- **DLV-002** [AGENT][P0][W1][yes] 实现文件交付接口 `POST /api/v1/orders/{id}/deliver` 的文件分支，关联可交付对象、到期时间、下载次数、回执摘要。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合） | ../业务流程/业务流程图-V1-完整版.md:L270（4.4.1 文件类交付） | ../页面说明书/页面说明书-V1-完整版.md:L590（7.1 文件交付页）
- **DLV-003** [AGENT][P0][W1][yes] 实现下载票据接口 `GET /api/v1/orders/{id}/download-ticket`，支持签发、过期、次数控制、审计日志。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合） | ../业务流程/业务流程图-V1-完整版.md:L270（4.4.1 文件类交付） | ../页面说明书/页面说明书-V1-完整版.md:L590（7.1 文件交付页）
- **DLV-004** [AGENT][P0][W1][yes] 实现下载票据验证中间件，确保文件下载请求必须带有效 ticket 并回写下载日志。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合） | ../业务流程/业务流程图-V1-完整版.md:L270（4.4.1 文件类交付） | ../页面说明书/页面说明书-V1-完整版.md:L590（7.1 文件交付页）
- **DLV-005** [AGENT][P0][W1][yes] 实现周期订阅接口 `POST /api/v1/orders/{id}/subscriptions` 与查询接口，支持 `FILE_SUB` 周期交付。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L290（4.4.1A 版本订阅交付） | ../原始PRD/数据对象产品族与交付模式增强设计.md:L292（4. 七类标准交易方式） | ../页面说明书/页面说明书-V1-完整版.md:L590（7.1 文件交付页）
- **DLV-006** [AGENT][P0][W1][yes] 实现共享授权接口 `POST /api/v1/orders/{id}/share-grants`、`GET /api/v1/orders/{id}/share-grants`。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L314（4.4.1B 只读共享交付） | ../页面说明书/页面说明书-V1-完整版.md:L625（7.3 只读共享开通页） | ../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合）
- **DLV-007** [AGENT][P0][W1][yes] 实现 API 开通分支：`POST /api/v1/orders/{id}/deliver` 的 API 模式，生成应用绑定、访问凭证、配额、限流配置。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L334（4.4.2 API 类交付） | ../页面说明书/页面说明书-V1-完整版.md:L611（7.2 API 开通页） | ../原始PRD/数据对象产品族与交付模式增强设计.md:L292（4. 七类标准交易方式）
- **DLV-008** [AGENT][P0][W1][yes] 实现 API 使用日志接口 `GET /api/v1/orders/{id}/usage-log`，返回按最小披露裁剪后的调用信息。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L334（4.4.2 API 类交付） | ../页面说明书/页面说明书-V1-完整版.md:L611（7.2 API 开通页） | ../原始PRD/数据对象产品族与交付模式增强设计.md:L292（4. 七类标准交易方式）
- **DLV-009** [AGENT][P0][W1][yes] 实现 QuerySurface 接口 `POST /api/v1/products/{id}/query-surfaces`，保存可查询区域、执行环境、输出限制。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/数据商品查询与执行面设计.md:L35（3. 四个核心对象） | ../页面说明书/页面说明书-V1-完整版.md:L668（7.6 查询面与模板配置页） | ../数据库设计/V1/upgrade/065_query_execution_plane.sql:L1（V1 migration）
- **DLV-010** [AGENT][P0][W1][yes] 实现 QueryTemplate 接口 `POST /api/v1/query-surfaces/{id}/templates`，支持模板版本、参数 schema、输出 schema、白名单字段。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/数据商品查询与执行面设计.md:L35（3. 四个核心对象） | ../页面说明书/页面说明书-V1-完整版.md:L668（7.6 查询面与模板配置页） | ../数据库设计/V1/upgrade/065_query_execution_plane.sql:L1（V1 migration）
- **DLV-011** [AGENT][P0][W1][yes] 实现模板授权接口 `POST /api/v1/orders/{id}/template-grants`，只允许命中白名单模板。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/数据商品查询与执行面设计.md:L127（5. V1 业务闭环） | ../页面说明书/页面说明书-V1-完整版.md:L639（7.4 模板查询开通页） | ../页面说明书/页面说明书-V1-完整版.md:L685（7.7 查询运行与结果记录页）
- **DLV-012** [AGENT][P0][W1][yes] 实现模板执行接口 `POST /api/v1/orders/{id}/template-runs`，做参数校验、风控校验、输出边界校验、审计。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/数据商品查询与执行面设计.md:L127（5. V1 业务闭环） | ../页面说明书/页面说明书-V1-完整版.md:L639（7.4 模板查询开通页） | ../页面说明书/页面说明书-V1-完整版.md:L685（7.7 查询运行与结果记录页）
- **DLV-013** [AGENT][P0][W1][yes] 实现查询运行记录接口 `GET /api/v1/orders/{id}/query-runs`。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/数据商品查询与执行面设计.md:L127（5. V1 业务闭环） | ../页面说明书/页面说明书-V1-完整版.md:L639（7.4 模板查询开通页） | ../页面说明书/页面说明书-V1-完整版.md:L685（7.7 查询运行与结果记录页）
- **DLV-014** [AGENT][P0][W1][yes] 实现沙箱开通接口 `POST /api/v1/orders/{id}/sandbox-workspaces`，创建空间、座位、到期时间、导出限制、状态。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L349（4.4.3 沙箱 / 模板查询类交付） | ../页面说明书/页面说明书-V1-完整版.md:L654（7.5 查询沙箱开通页） | ../原始PRD/数据商品查询与执行面设计.md:L127（5. V1 业务闭环）
- **DLV-015** [AGENT][P0][W1][yes] 实现沙箱工作区模型：workspace、session、seat、export control、attestation 引用。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L349（4.4.3 沙箱 / 模板查询类交付） | ../页面说明书/页面说明书-V1-完整版.md:L654（7.5 查询沙箱开通页） | ../原始PRD/数据商品查询与执行面设计.md:L127（5. V1 业务闭环）
- **DLV-016** [AGENT][P0][W1][yes] 为 `SBX_STD` 预留 gVisor 执行隔离参数位，即便 local 模式先只做配置占位，也要把环境模型冻结。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L349（4.4.3 沙箱 / 模板查询类交付） | ../页面说明书/页面说明书-V1-完整版.md:L654（7.5 查询沙箱开通页） | ../原始PRD/数据商品查询与执行面设计.md:L127（5. V1 业务闭环）
- **DLV-017** [AGENT][P0][W1][yes] 实现报告交付接口 `POST /api/v1/orders/{id}/deliver` 的报告分支，生成 report artifact、报告 hash、交付回执。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L388（4.4.4 报告类交付） | ../页面说明书/页面说明书-V1-完整版.md:L701（7.8 报告 / 结果产品交付页） | ../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合）
- **DLV-018** [AGENT][P0][W1][yes] 实现验收通过接口 `POST /api/v1/orders/{id}/accept` 与拒收接口 `POST /api/v1/orders/{id}/reject`。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../页面说明书/页面说明书-V1-完整版.md:L714（7.9 验收页） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **DLV-019** [AGENT][P0][W1][yes] 实现交付对象水印/指纹策略占位模型，保证文件和报告后续可挂接真实水印流水线。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合） | ../业务流程/业务流程图-V1-完整版.md:L270（4.4.1 文件类交付） | ../页面说明书/页面说明书-V1-完整版.md:L590（7.1 文件交付页）
- **DLV-020** [AGENT][P0][W1][yes] 实现 Delivery 回执 outbox 事件：文件交付、API 开通、共享开通、模板授权、沙箱开通、报告交付都要发事件。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程）
- **DLV-021** [AGENT][P0][W1][yes] 实现 `delivery` 模块的自动断权联动：到期、取消、争议升级、风控冻结时撤销下载票据、API key、共享授权、沙箱席位。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L430（5. 异常与争议流程） | ../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合） | ../原始PRD/敏感数据处理与受控交付设计.md:L124（5. 敏感数据完整交易链闭环）
- **DLV-022** [AGENT][P0][W1][yes] 实现敏感执行策略接口 `POST /api/v1/orders/{id}/sensitive-execution-policies`，即便首轮只做普通场景，也要冻结结构。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/敏感数据处理与受控交付设计.md:L124（5. 敏感数据完整交易链闭环） | ../原始PRD/数据商品查询与执行面设计.md:L185（8. 与授权、计费、审计的关系） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7458（31. 敏感数据受控交付补充基线）
- **DLV-023** [AGENT][P0][W1][yes] 实现结果披露审查接口 `POST /api/v1/query-runs/{id}/disclosure-review` 占位，便于高敏场景后续扩展。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/敏感数据处理与受控交付设计.md:L124（5. 敏感数据完整交易链闭环） | ../原始PRD/数据商品查询与执行面设计.md:L185（8. 与授权、计费、审计的关系） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7458（31. 敏感数据受控交付补充基线）
- **DLV-024** [AGENT][P0][W1][yes] 实现执行证明查看接口 `GET /api/v1/orders/{id}/attestations` 与销毁/保留证明接口。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/敏感数据处理与受控交付设计.md:L124（5. 敏感数据完整交易链闭环） | ../原始PRD/数据商品查询与执行面设计.md:L185（8. 与授权、计费、审计的关系） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7458（31. 敏感数据受控交付补充基线）
- **DLV-025** [AGENT][P1][W3][yes] 为 Delivery/Storage/Query 编写集成测试：文件下载票据、API 开通、模板授权与执行、沙箱开通、报告交付、验收通过/拒收。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../页面说明书/页面说明书-V1-完整版.md:L996（14. 页面覆盖校验）
- **DLV-026** [AGENT][P1][W3][yes] 为五条标准链路分别准备最小交付对象与演示脚本。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../页面说明书/页面说明书-V1-完整版.md:L996（14. 页面覆盖校验）
- **DLV-027** [AGENT][P1][W3][yes] 生成 `docs/02-openapi/delivery.yaml` 或并入 `trade.yaml` 的交付子域文档。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：docs/02-openapi/delivery.yaml; trade.yaml
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../页面说明书/页面说明书-V1-完整版.md:L996（14. 页面覆盖校验）
- **DLV-028** [AGENT][P1][W3][yes] 生成 `docs/05-test-cases/delivery-cases.md`，覆盖交付超时、重复开通、票据过期、撤权后访问、验收失败。
  依赖：TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008
  交付：docs/05-test-cases/delivery-cases.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../页面说明书/页面说明书-V1-完整版.md:L996（14. 页面覆盖校验）
- **DLV-029** [AGENT][P0][W1][yes] 实现“交付任务自动创建器”：订单进入“待交付”后按 SKU 自动生成文件交付/API 开通/共享开通/模板授权/沙箱开通/报告交付任务，并记录创建来源、责任主体、重试次数与人工接管标识。
  依赖：TRADE-031; DLV-002; DLV-007; DLV-017
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合） | ../全集成文档/数据交易平台-全集成基线-V1.md:L1723（15. 核心交易链路设计（完整闭环））
- **DLV-030** [AGENT][P0][W1][yes] 实现交付完成到计费触发的桥接事件：文件交付、共享开通、API 正式开通、模板执行成功、沙箱开通、报告交付、验收通过等动作必须产出标准化 outbox 事件，供 Billing 聚合而不是由 Billing 侧猜测状态。
  依赖：DLV-020; BIL-006; CORE-008
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L400（4.5 结算、退款、赔付与关闭流程） | ../原始PRD/数据商品查询与执行面设计.md:L185（8. 与授权、计费、审计的关系） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **DLV-031** [AGENT][P0][W1][yes] 实现按 SKU 的验收触发矩阵：`FILE_STD/FILE_SUB/RPT_STD` 以交付包/报告签收为主，`API_SUB/API_PPU` 以开通成功和首个有效调用为主，`QRY_LITE` 以模板执行成功并结果可取为主，`SHARE_RO/SBX_STD` 以共享或工作区开通成功为主；并写入统一规则文件供测试复用。
  依赖：DLV-018; TRADE-029; BIL-023
  交付：apps/platform-core/src/modules/delivery/**; apps/platform-core/src/modules/storage/**; packages/openapi/delivery.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）

## 13. Billing / Payment / Settlement / Dispute [BIL]

这一组落地支付、账单、托管、结算、退款与争议闭环。

- **BIL-001** [AGENT][P0][W1][yes] 实现 Payment Jurisdiction / Corridor / Payout Preference 基础模型与接口占位。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-002** [AGENT][P0][W1][yes] 实现支付意图 `POST /api/v1/payments/intents` / `GET /api/v1/payments/intents/{id}` / `cancel`。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-003** [AGENT][P0][W1][yes] 实现订单锁资接口 `POST /api/v1/orders/{id}/lock`，把订单与支付意图关联起来。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-004** [AGENT][P0][W1][yes] 实现 Mock Payment Provider 适配器，支持 success/fail/timeout webhook 三种模拟结果。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-005** [AGENT][P0][W1][yes] 实现支付 webhook 接口 `POST /api/v1/payments/webhooks/{provider}`，支持签名占位、幂等、防重放、乱序保护。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-006** [AGENT][P0][W1][yes] 实现账单事件模型 `BillingEvent`，覆盖一次性收费、周期收费、调用量收费、退款、赔付、人工结算。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-007** [AGENT][P0][W1][yes] 实现账单查看接口 `GET /api/v1/billing/{order_id}`，返回账单明细、结算状态、税务/发票占位字段。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-008** [AGENT][P0][W1][yes] 实现 Settlement 模型：应结金额、平台抽佣、供方应收、退款/赔付调整、结算摘要。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-009** [AGENT][P0][W1][yes] 实现退款接口 `POST /api/v1/refunds`，要求裁决结果、step-up、审计齐全。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1002（4.7 争议与售后聚合） | ../业务流程/业务流程图-V1-完整版.md:L473（5.3 争议处理流程） | ../页面说明书/页面说明书-V1-完整版.md:L773（8.3 争议提交页）
- **BIL-010** [AGENT][P0][W1][yes] 实现赔付接口 `POST /api/v1/compensations`，要求裁决结果、step-up、审计齐全。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1002（4.7 争议与售后聚合） | ../业务流程/业务流程图-V1-完整版.md:L473（5.3 争议处理流程） | ../页面说明书/页面说明书-V1-完整版.md:L773（8.3 争议提交页）
- **BIL-011** [AGENT][P0][W1][yes] 实现人工打款/人工分账占位模型，V1 先支持人工执行但对象与状态必须完整。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-012** [AGENT][P0][W1][yes] 实现支付对账导入接口 `POST /api/v1/payments/reconciliation/import` 占位，保存对账差异结果。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-013** [AGENT][P0][W1][yes] 实现争议案件接口 `POST /api/v1/cases`、证据上传 `POST /api/v1/cases/{id}/evidence`、裁决 `POST /api/v1/cases/{id}/resolve`。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1002（4.7 争议与售后聚合） | ../业务流程/业务流程图-V1-完整版.md:L473（5.3 争议处理流程） | ../页面说明书/页面说明书-V1-完整版.md:L773（8.3 争议提交页）
- **BIL-014** [AGENT][P0][W1][yes] 实现 Dispute 对 Order/Delivery/Billing 的联动：争议发起时可冻结结算、可中止交付、可触发审计保全。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1002（4.7 争议与售后聚合） | ../业务流程/业务流程图-V1-完整版.md:L473（5.3 争议处理流程） | ../页面说明书/页面说明书-V1-完整版.md:L773（8.3 争议提交页）
- **BIL-015** [AGENT][P0][W1][yes] 实现 Billing Event 到 Settlement 的聚合计算器，并保证幂等重算能力。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-016** [AGENT][P0][W1][yes] 实现账单/结算摘要 outbox 事件，为后续 Fabric 存证和审计归档做准备。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-017** [AGENT][P0][W1][yes] 为 API_SUB/API_PPU 设计最小计费口径：订阅周期账单 + 按调用量追加事件。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-018** [AGENT][P0][W1][yes] 为 FILE_STD/FILE_SUB/SHARE_RO/QRY_LITE/SBX_STD/RPT_STD 设计默认计费口径与退款逻辑占位，并补充共享开通类 SKU 的计费触发点。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-019** [AGENT][P1][W3][yes] 为支付与账单编写集成测试：支付成功、支付失败、超时重试、退款、赔付、争议升级、账单重算。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-020** [AGENT][P1][W2][yes] 生成 `docs/02-openapi/billing.yaml` 第一版并与实现校验。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：docs/02-openapi/billing.yaml
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-021** [AGENT][P1][W3][yes] 生成 `docs/05-test-cases/payment-billing-cases.md`，覆盖回调乱序、重复回调、重复扣费防护、结算冻结。
  依赖：TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009
  交付：docs/05-test-cases/payment-billing-cases.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-022** [AGENT][P0][W1][yes] 实现支付结果处理器：消费支付 webhook 或支付轮询结果，幂等更新 PaymentIntent、Order.payment_status、订单主状态推进记录与审计事件，确保“success 后 fail”“timeout 后 success”等乱序场景不破坏最终状态。
  依赖：BIL-005; TRADE-007; TRADE-030
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **BIL-023** [AGENT][P0][W1][yes] 输出 `docs/03-db/sku-billing-trigger-matrix.md`，逐个写清 8 个 SKU 的计费触发点、结算周期、退款入口、赔付入口、争议冻结点、恢复结算点，作为 BillingEvent 生成的唯一业务口径。
  依赖：CTX-021; BIL-017; BIL-018
  交付：docs/03-db/sku-billing-trigger-matrix.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-024** [AGENT][P0][W1][yes] 实现交付/执行/验收到 BillingEvent 的桥接器：对 `FILE_STD/FILE_SUB/SHARE_RO/API_SUB/API_PPU/QRY_LITE/SBX_STD/RPT_STD` 分别把“支付锁定、周期出账、调用量上报、执行成功、验收通过、退款、赔付、人工结算”映射为标准 BillingEvent。
  依赖：BIL-006; BIL-023; DLV-030
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-025** [AGENT][P0][W1][yes] 实现 BillingEvent 冲销与 Settlement 冻结规则：拒收、争议升级、裁决退款、赔付、人工修正时必须生成冲销/补差事件，并能重新聚合出正确 Settlement，不允许直接手改最终金额。
  依赖：BIL-015; BIL-024; BIL-013; BIL-014
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）
- **BIL-026** [AGENT][P0][W1][yes] 为 `SHARE_RO` 补齐最小计费口径：共享开通费、周期共享费、撤权退款占位、争议冻结规则，并补充相应账单样例、API 响应样例与测试用例。
  依赖：TRADE-012; BIL-018
  交付：apps/platform-core/src/modules/billing/**; apps/platform-core/src/modules/dispute/**; packages/openapi/billing.yaml
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：主编排缺失会造成支付、交付、验收、结算链路断裂。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L895（4.6 账单、托管与分润聚合） | ../数据库设计/V1/upgrade/040_billing_support_risk.sql:L1（V1 migration） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计）

## 14. Notification / Messaging / Template [NOTIF]

这一组落地通知模板、消息分发与外部回调通知。

- **NOTIF-001** [AGENT][P0][W1][no] 初始化 `apps/notification-worker/` 骨架，约定运行模式、消费 topic、模板目录、发送适配器、重试队列与健康检查接口。
  依赖：BOOT-002; ENV-010; CORE-009
  交付：apps/notification-worker/
  完成定义：正式进程名、topic、consumer group、V1 渠道边界与健康检查口径已冻结为 `notification-worker -> dtp.notification.dispatch -> cg-notification-worker`；`notification-worker` 不直接消费 `dtp.outbox.domain-events`；Worker 可消费事件并发送 `mock-log` 通知；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | 问题修复任务/A01-Kafka-Topic-口径统一.md:L1（Kafka topic 口径统一） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-002** [AGENT][P0][W1][no] 定义通知事件协议：统一收口到 `notification.requested -> dtp.notification.dispatch -> notification-worker`，覆盖订单创建、支付成功、支付失败、待交付、交付完成、待验收、验收通过、拒收、争议升级、退款完成、赔付完成、监管冻结、恢复结算；统一事件字段和幂等键，不再并行消费 `dtp.outbox.domain-events`。
  依赖：NOTIF-001; TRADE-033; BIL-023
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：通知 Worker 可消费事件并发送 mock 通知；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | 问题修复任务/A01-Kafka-Topic-口径统一.md:L1（Kafka topic 口径统一） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-003** [AGENT][P0][W1][no] 实现通知模板模型：模板编码、语言、变量 schema、渠道、启用状态、版本号、渲染结果预览与 fallback 文案。
  依赖：NOTIF-001; NOTIF-002
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：通知 Worker 可消费事件并发送 mock 通知；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-004** [AGENT][P0][W1][no] 实现“支付成功 -> 待交付”通知模板与发送逻辑，区分买方、卖方、运营可见内容，禁止把内部风控/审计字段直接暴露给业务用户。
  依赖：NOTIF-002; TRADE-030
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：通知 Worker 可消费事件并发送 mock 通知；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L204（4.3 买方搜索、选购与下单流程） | ../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../页面说明书/页面说明书-V1-完整版.md:L556（6.4 订单详情页） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-005** [AGENT][P0][W1][no] 实现“交付完成 -> 待验收”通知模板与发送逻辑，覆盖文件包、共享开通、API 开通、查询结果可取、沙箱开通、报告交付六类交付结果。
  依赖：NOTIF-002; DLV-030
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：通知 Worker 可消费事件并发送 mock 通知；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程） | ../页面说明书/页面说明书-V1-完整版.md:L714（7.9 验收页） | ../页面说明书/页面说明书-V1-完整版.md:L556（6.4 订单详情页） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-006** [AGENT][P0][W1][no] 实现“验收通过/拒收/退款完成/赔付完成”通知模板与发送逻辑，并把动作摘要与后续待办链接到订单详情、账单页或争议页。
  依赖：NOTIF-002; DLV-018; BIL-025
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：通知 Worker 可消费事件并发送 mock 通知；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L400（4.5 结算、退款、赔付与关闭流程） | ../页面说明书/页面说明书-V1-完整版.md:L738（8.1 账单中心） | ../页面说明书/页面说明书-V1-完整版.md:L773（8.3 争议提交页） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-007** [AGENT][P0][W1][no] 实现“争议升级/监管冻结/恢复结算”通知模板与发送逻辑，要求最小披露、角色隔离与审计留痕。
  依赖：NOTIF-002; BIL-013; BIL-014
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：通知 Worker 可消费事件并发送 mock 通知；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L473（5.3 争议处理流程） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/交易链监控、公平性与信任安全设计.md:L174（5. 六层交易链监控模型） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-008** [AGENT][P0][W1][no] 实现通知发送适配器抽象，至少支持 `mock-log` 渠道；预留 email/webhook 渠道接口，但 local 模式先把结果写日志和审计表。
  依赖：NOTIF-001; NOTIF-002; CORE-018
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：通知发送适配器抽象已建立；`V1` 只实接 `mock-log` 渠道，`email/webhook` 仅保留 provider 边界；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-009** [AGENT][P0][W1][no] 实现通知幂等、重试与 DLQ：重复事件不重复发送，失败事件可重试并转入 dead letter，同时保留人工重放入口。
  依赖：NOTIF-008; ENV-010
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：通知 Worker 可消费事件并发送 mock 通知；通知幂等、重试、DLQ、人工重放入口与审计联查口径已冻结；模板/幂等/重试可验证；审计和 runbook 已覆盖。
  验收：触发对应事件后能看到通知发送记录、幂等去重和失败重试结果。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | 问题修复任务/A01-Kafka-Topic-口径统一.md:L1（Kafka topic 口径统一） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-010** [AGENT][P1][W3][no] 实现通知审计联查：按订单号/案件号/通知模板查看发送记录、渲染变量、渠道结果、重试轨迹与关联事件。
  依赖：NOTIF-009; AUD-004
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：当前执行源已冻结通知联查的目标范围、正式事件链与后续 OpenAPI / 验收承接，不再将其视为已完成运行时能力。
  验收：当前阶段以 runbook、`docs/05-test-cases/README.md` 与 TODO 已明确后续需补齐的发送记录、渲染变量、渠道结果、重试轨迹与关联事件范围为准；进入 `NOTIF` 代码实现批次后再按真实运行结果验收。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../04-runbooks/notification-worker.md:L12（当前批次边界） | ../05-test-cases/README.md:L24（通知验收清单尚未落盘） | V1-Core-TODO与预留清单.md:L65（NOTIF OpenAPI 与测试义务） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-011** [AGENT][P1][W3][no] 在 `docs/04-runbooks/notification-worker.md` 写明通知事件来源、模板清单、发送策略、失败排查与人工补发流程。
  依赖：NOTIF-001; NOTIF-002; NOTIF-009
  交付：docs/04-runbooks/notification-worker.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：runbook 已明确正式进程名、正式 topic、consumer group、V1 渠道边界、失败排查、人工补发入口，以及当前批次未完成的 OpenAPI / `notification-cases.md` 承接关系，并被 README / 任务清单引用。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../04-runbooks/notification-worker.md:L12（当前批次边界） | ../05-test-cases/README.md:L24（通知验收清单尚未落盘） | V1-Core-TODO与预留清单.md:L65（NOTIF OpenAPI 与测试义务） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界）
- **NOTIF-012** [AGENT][P1][W3][no] 为通知链路编写集成测试：支付成功通知、交付完成通知、拒收通知、争议升级通知、重复事件去重、失败重试与 DLQ；校验正式链路 `notification.requested -> dtp.notification.dispatch -> notification-worker`、`mock-log` 渠道与审计痕迹。
  依赖：NOTIF-004; NOTIF-005; NOTIF-006; NOTIF-009
  交付：apps/notification-worker/**; docs/04-runbooks/notification-worker.md
  完成定义：当前执行源已冻结通知集成测试目标场景、正式事件链、`mock-log` / `DLQ` / 审计痕迹与后续 `notification-cases.md` 承接，不再将其视为已完成测试基线。
  验收：当前阶段以 runbook、`docs/05-test-cases/README.md` 与 TODO 已明确后续需补齐的通知事件链路验收矩阵为准；进入 `NOTIF` 代码实现批次后再按真实集成测试结果验收。
  阻塞风险：事件和模板不统一会导致重复通知、漏通知或越权披露。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | ../04-runbooks/notification-worker.md:L12（当前批次边界） | ../05-test-cases/README.md:L24（通知验收清单尚未落盘） | V1-Core-TODO与预留清单.md:L65（NOTIF OpenAPI 与测试义务） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险）
- **NOTIF-013** [AGENT][P1][W3][no] 补齐通知联查与控制面 OpenAPI 归档/示例，覆盖发送记录、模板、渠道结果、重试/DLQ、人工补发入口与 `event_type / target_topic / aggregate_type` 过滤口径。
  依赖：NOTIF-010; AUD-003; AUD-025
  交付：packages/openapi/ops.yaml; docs/02-openapi/ops.yaml
  完成定义：通知联查与控制面 OpenAPI 归档/示例已建立；发送记录、模板、渠道结果、重试/DLQ/人工补发与事件过滤口径已与正式事件链一致，且不再使用旧命名。
  验收：至少一条契约校验或手工 API 验证通过，并能证明通知联查示例与 `notification.requested -> dtp.notification.dispatch -> notification-worker` 正式口径一致。
  阻塞风险：通知控制面契约缺失会导致后续联查、控制台与测试继续在错误接口上叠加返工。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L93（通知相关审计事件） | ../原始PRD/日志、可观测性与告警设计.md:L115（通知与观测字段规范） | ../开发准备/事件模型与Topic清单正式版.md:L128（通知事件与 topic） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L20（事件路由与一致性控制面） | ../02-openapi/README.md:L10（实现阶段 OpenAPI 约束） | ../04-runbooks/notification-worker.md:L17（通知运行与 OpenAPI 承接） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险）
- **NOTIF-014** [AGENT][P1][W3][no] 生成 `docs/05-test-cases/notification-cases.md`，覆盖支付成功、交付完成、验收通过、拒收、争议升级、监管冻结/恢复结算、重复去重、失败重试、DLQ 与人工补发。
  依赖：NOTIF-009; NOTIF-010; NOTIF-013
  交付：docs/05-test-cases/notification-cases.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用，并明确登记 mock-log、幂等、重试、DLQ、人工补发与审计联查验收项。
  验收：至少一条 smoke 或手工验证通过，并能证明通知验收清单覆盖正式 topic、正式进程名、正式渠道边界与联查路径。
  阻塞风险：通知测试基线若继续缺失，会让 NOTIF 阶段只能证明“有日志输出”，不能证明正式链路闭环。
  技术参考：../开发准备/测试用例矩阵正式版.md:L1（通知链路验收基线） | ../开发准备/事件模型与Topic清单正式版.md:L128（通知事件与 topic） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L80（dead letter 与重处理） | ../05-test-cases/README.md:L7（测试样例批次边界） | ../04-runbooks/notification-worker.md:L14（NOTIF 当前批次边界） | 问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md:L1（NOTIF 通知链路与命名边界） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险）
## 15. Audit / Evidence / Consistency / Fabric / Ops [AUD]

这一组落地审计、证据链、回放、一致性和 Fabric 最小摘要链路。

- **AUD-001** [AGENT][P0][W1][no] 实现 `AuditEvent` 统一模型，所有关键动作至少记录 actor、object、action、result、request_id、trace_id、tenant_id，并作为统一 `audit writer` 的唯一模型基座。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；`audit-kit / DB schema / API DTO` 字段语义已对齐，不再允许业务模块再定义第二套审计事件模型。
  验收：至少一条集成测试或手工 API 验证通过，并能证明统一模型已可被后续 `audit writer`、联查与导出路径复用。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A06-Audit-Kit-统一模型漂移.md | 问题修复任务/A02-统一事件-Envelope-与路由权威源.md:L1（统一事件 Envelope 与路由权威源） | 问题修复任务/A03-统一事务模板-落地真实审计与Outbox-Writer.md:L1（统一事务模板与 Writer 收口） | 问题修复任务/A14-AUD-历史模块统一Writer与证据桥接缺口.md:L1（历史模块统一 Writer 与证据桥接）
- **AUD-002** [AGENT][P0][W1][no] 实现 `EvidenceItem` 与 `EvidenceManifest` 统一模型，支持关联对象存储 URI、hash、来源、保留策略，并作为统一 `evidence writer` 与历史证据桥接的权威对象。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；证据权威对象已能承接 `PG + 对象存储` 双写语义，并为历史 `support.evidence_object` 等桥接提供统一落点。
  验收：至少一条集成测试或手工 API 验证通过，并能证明 `EvidenceItem / EvidenceManifest` 已可作为导出、回放、legal hold 与历史证据桥接的统一对象。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A06-Audit-Kit-统一模型漂移.md | 问题修复任务/A14-AUD-历史模块统一Writer与证据桥接缺口.md | 问题修复任务/A14-AUD-历史模块统一Writer与证据桥接缺口.md:L1（历史模块统一 Writer 与证据桥接）
- **AUD-003** [AGENT][P0][W1][no] 实现 `GET /api/v1/audit/orders/{id}`、`GET /api/v1/audit/traces`，提供订单与全局审计联查。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-004** [AGENT][P0][W1][no] 实现证据包导出接口 `POST /api/v1/audit/packages/export`，要求导出理由、step-up、审计留痕。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A06-Audit-Kit-统一模型漂移.md
- **AUD-005** [AGENT][P0][W1][no] 实现回放任务接口 `POST /api/v1/audit/replay-jobs`、`GET /api/v1/audit/replay-jobs/{id}`，V1 默认 dry-run。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A06-Audit-Kit-统一模型漂移.md
- **AUD-006** [AGENT][P0][W1][no] 实现 legal hold 接口 `POST /api/v1/audit/legal-holds` / `release`。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A06-Audit-Kit-统一模型漂移.md
- **AUD-007** [AGENT][P0][W1][no] 实现 AnchorBatch 模型和查看/重试接口 `GET /api/v1/audit/anchor-batches` / `POST /retry`。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A06-Audit-Kit-统一模型漂移.md
- **AUD-008** [AGENT][P0][W1][no] 实现 `ops.outbox_event`、`ops.dead_letter_event`、`ops.consumer_idempotency_record`、`ops.external_fact_receipt`、`ops.chain_projection_gap` 的仓储与查询接口，并明确 `consistency/reconcile` 在 `V1` 中是控制面动作，由 `AUD-012` 承接，不单列正式 `reconcile_job` 表。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；`ops.dead_letter_event` 与 `ops.consumer_idempotency_record` 已可支撑 SEARCHREC consumer 的失败隔离、幂等与联查。
  验收：至少一条集成测试或手工 API 验证通过，并能查询 SEARCHREC consumer 的 dead letter 与幂等记录。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L42（4. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1539（9. 链上链下对象映射） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md | 问题修复任务/A02-统一事件-Envelope-与路由权威源.md:L1（统一事件 Envelope 与路由权威源） | 问题修复任务/A03-统一事务模板-落地真实审计与Outbox-Writer.md:L1（统一事务模板与 Writer 收口） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A05-Outbox-Publisher-DLQ-统一闭环缺口.md:L1（Outbox Publisher 与 DLQ 闭环） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md:L1（SEARCHREC Consumer 幂等与 DLQ 闭环）
- **AUD-009** [AGENT][P0][W1][no] 实现 outbox publisher worker，从数据库读取待发布事件并推送到 Kafka。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；publisher 只按统一 event envelope 与 `ops.event_route_policy` / canonical topic 口径发布，记录 `outbox_publish_attempt`，并与上下游模块联调通过；不再为 `notification-worker` / `fabric-adapter` 保留 `dtp.outbox.domain-events` 并行入口。
  验收：至少一条集成测试或手工 API 验证通过，并能证明发布尝试、目标 topic、失败隔离与审计痕迹可联查，且不存在并行 topic 口径。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L57（4.2 outbox 查询） | ../领域模型/全量领域模型与对象关系说明.md:L1539（9. 链上链下对象映射） | 问题修复任务/A01-Kafka-Topic-口径统一.md:L1（Kafka topic 口径统一） | 问题修复任务/A02-统一事件-Envelope-与路由权威源.md:L1（统一事件 Envelope 与路由权威源） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A05-Outbox-Publisher-DLQ-统一闭环缺口.md:L1（Outbox Publisher 与 DLQ 闭环）
- **AUD-010** [AGENT][P0][W1][no] 实现 dead letter 重处理接口 `POST /api/v1/ops/dead-letters/{id}/reprocess`，支持 dry-run 与 step-up，并能覆盖 SEARCHREC consumer 失败事件。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；`search-indexer` 与 `recommendation-aggregator` 失败事件可通过该接口做 dry-run 重处理，并与 Kafka DLQ / DB dead letter 对齐。
  验收：至少一条集成测试或手工 API 验证通过，并能验证 SEARCHREC dead letter 的 dry-run / step-up 重处理路径。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L80（4.4 dead letter 重处理） | ../领域模型/全量领域模型与对象关系说明.md:L1539（9. 链上链下对象映射） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A05-Outbox-Publisher-DLQ-统一闭环缺口.md:L1（Outbox Publisher 与 DLQ 闭环） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md:L1（SEARCHREC Consumer 幂等与 DLQ 闭环）
- **AUD-011** [AGENT][P0][W1][no] 实现一致性联查接口 `GET /api/v1/ops/consistency/{refType}/{refId}`，返回业务状态、证明状态、外部事实状态。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L44（4.1 一致性联查） | ../领域模型/全量领域模型与对象关系说明.md:L1539（9. 链上链下对象映射） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-012** [AGENT][P0][W1][no] 实现一致性修复接口 `POST /api/v1/ops/consistency/reconcile`，V1 先支持 dry-run + 记录修复建议。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L90（4.5 一致性修复） | ../领域模型/全量领域模型与对象关系说明.md:L1539（9. 链上链下对象映射） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-013** [AGENT][P0][W1][no] 初始化 `services/fabric-adapter/`（Go），定义从 `dtp.audit.anchor` / `dtp.fabric.requests` 接收业务摘要事件、调用链码、回写链回执的基础框架。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：services/fabric-adapter/
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-014** [AGENT][P0][W1][no] 在 `fabric-adapter` 中先实现订单摘要、授权摘要、验收摘要、证据批次根四类消息处理占位，并保持 `dtp.audit.anchor` / `dtp.fabric.requests` 单入口。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-015** [AGENT][P0][W1][no] 初始化 `services/fabric-event-listener/`（Go），消费链码事件并回写数据库外部事实回执。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：services/fabric-event-listener/
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-016** [AGENT][P0][W1][no] 初始化 `services/fabric-ca-admin/`（Go），封装成员证书签发/吊销接口占位。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：services/fabric-ca-admin/
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-017** [AGENT][P0][W1][no] 实现链写入 Provider 接口，让 local 模式可切换 `mock` 与 `fabric-test-network` 两种实现。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../开发准备/技术选型正式版.md:L30（3. 平台核心技术底座） | ../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../领域模型/全量领域模型与对象关系说明.md:L1171（4.10 链上摘要与公链增强聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-018** [AGENT][P0][W1][no] 实现交易链监控总览接口 `GET /api/v1/ops/trade-monitor/orders/{orderId}` 与 checkpoints 接口。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/交易链监控、公平性与信任安全设计.md:L174（5. 六层交易链监控模型） | ../数据库设计/接口协议/交易链监控与公平性接口协议正式版.md:L171（6. V1 接口） | ../业务流程/业务流程图-V1-完整版.md:L522（6. 链上链下一致性与事件流） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-019** [AGENT][P0][W1][no] 实现外部事实查询/确认接口 `GET /api/v1/ops/external-facts`、`POST /confirm`。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/交易链监控、公平性与信任安全设计.md:L174（5. 六层交易链监控模型） | ../数据库设计/接口协议/交易链监控与公平性接口协议正式版.md:L201（6.3 外部事实回执查询） | ../业务流程/业务流程图-V1-完整版.md:L522（6. 链上链下一致性与事件流） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-020** [AGENT][P0][W1][no] 实现公平性事件查询/处理接口 `GET /api/v1/ops/fairness-incidents`、`POST /handle`。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/交易链监控、公平性与信任安全设计.md:L174（5. 六层交易链监控模型） | ../数据库设计/接口协议/交易链监控与公平性接口协议正式版.md:L242（6.5 公平性事件查询） | ../业务流程/业务流程图-V1-完整版.md:L522（6. 链上链下一致性与事件流） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-021** [AGENT][P0][W1][no] 实现投影缺口查询/关闭接口 `GET /api/v1/ops/projection-gaps`、`POST /resolve`。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/交易链监控、公平性与信任安全设计.md:L174（5. 六层交易链监控模型） | ../数据库设计/接口协议/交易链监控与公平性接口协议正式版.md:L323（6.9 链投影缺口查询） | ../业务流程/业务流程图-V1-完整版.md:L522（6. 链上链下一致性与事件流） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-022** [AGENT][P0][W1][no] 实现搜索同步状态接口 `GET /api/v1/ops/search/sync`、重建/别名切换/缓存失效/排序配置更新接口，并切换到统一鉴权 / 正式权限点 / 审计 / 搜索域错误码口径；搜索 alias 与缓存命名必须遵循统一权威源。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、统一鉴权、正式权限点、必要的 `X-Step-Up-Token`、审计、搜索域错误码和最小测试已齐备；实现与 OpenAPI 不漂移，且不再使用 `x-role` 占位。
  验收：至少一条集成测试或手工 API 验证通过，并能验证 `Authorization + 权限点 + step-up + 审计 + SEARCH_*` 的正式口径。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../业务流程/业务流程图-V1-完整版.md:L563（6.3 搜索与索引同步流程） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | ../权限设计/接口权限校验清单.md:L155（搜索运维接口权限与 step-up） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md | 问题修复任务/A08-搜索Alias权威源与阶段边界冲突.md:L1（搜索 Alias 权威源与阶段边界） | 问题修复任务/A12-配置项与资源命名漂移.md:L1（配置项与资源命名漂移） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md:L1（SEARCHREC 契约收口）
- **AUD-023** [AGENT][P0][W1][no] 实现观测总览接口 `GET /api/v1/ops/observability/overview`、日志镜像查询/导出、trace 联查、告警与事故工单接口。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../数据库设计/接口协议/日志与可观测性接口协议正式版.md:L23（3. 查询接口） | ../页面说明书/页面说明书-V1-完整版.md:L921（11.5 搜索运维页） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-024** [AGENT][P0][W1][no] 实现 Developer 联查接口 `GET /api/v1/developer/trace`，按订单号、事件号、交易哈希快速定位。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../领域模型/全量领域模型与对象关系说明.md:L1298（4.12 开发与调试支持聚合） | ../页面说明书/页面说明书-V1-完整版.md:L909（11.3 状态联查页） | ../原始PRD/日志、可观测性与告警设计.md:L115（6. 日志字段规范） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-025** [AGENT][P0][W1][no] 建立审计与 ops 的最小权限矩阵：只读、导出、回放、修复、重处理、重建、锚定重试都要区分权限点。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  约束补充：正式角色集合 authority 以 `060_seed_authz_v1.sql` 为准；正式权限分配 authority 以 `docs/权限设计/角色权限矩阵正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md` 为准；`070_seed_role_permissions_v1.sql` 必须成为该矩阵的可执行镜像；代码 / runbook / test-case / OpenAPI 示例不得继续以旧别名角色充当正式 authority。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A03-统一事务模板-落地真实审计与Outbox-Writer.md:L1（统一事务模板与 Writer 收口） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-026** [AGENT][P1][W3][no] 为审计/一致性/Fabric/Ops 编写集成测试：outbox 发布、consumer 幂等、回执回写、重试、DB/Kafka 双层 DLQ、审计包导出、dry-run 回放。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：apps/platform-core/src/modules/audit/**; services/fabric-adapter/**; workers/**; docs/04-runbooks/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；至少覆盖 SEARCHREC consumer 的幂等、失败隔离、重处理与 worker 侧副作用验证。
  验收：至少一条集成测试或手工 API 验证通过，并能覆盖 SEARCHREC consumer 的 DB/Kafka 双层 DLQ 与 reprocess 路径。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口） | 问题修复任务/A05-Outbox-Publisher-DLQ-统一闭环缺口.md:L1（Outbox Publisher 与 DLQ 闭环） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md:L1（SEARCHREC Consumer 幂等与 DLQ 闭环）
- **AUD-027** [AGENT][P1][W3][no] 生成 `docs/02-openapi/audit.yaml`、`ops.yaml` 第一版并与实现校验。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：docs/02-openapi/audit.yaml; ops.yaml
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-028** [AGENT][P1][W3][no] 生成 `docs/05-test-cases/audit-consistency-cases.md`，覆盖链下成功链上失败、链上成功链下未更新、回调乱序、重复事件、修复演练。
  依赖：CORE-007; CORE-008; DB-008; ENV-022
  交付：docs/05-test-cases/audit-consistency-cases.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L33（3. 五层审计体系） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../领域模型/全量领域模型与对象关系说明.md:L1125（4.9 审计与证据聚合） | 问题修复任务/A04-AUD-Ops-接口与契约落地缺口.md:L1（AUD/Ops 接口与契约落地缺口）
- **AUD-029** [AGENT][P0][W1][no] 收敛已完成阶段的历史模块到统一 `audit writer / evidence writer`：将 `catalog / search / billing dispute` 等路径中的 ad-hoc 审计写入与 `support.evidence_object` 私有证据写法迁移或桥接到 `AuditEvent / EvidenceItem / EvidenceManifest` 正式权威模型，禁止继续绕过统一写入器。
  依赖：AUD-001; AUD-002; CORE-007; CORE-008; DB-008
  交付：apps/platform-core/src/modules/audit/**; apps/platform-core/src/modules/catalog/**; apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/billing/**; docs/04-runbooks/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；历史模块已接入统一 `audit writer / evidence writer`，不再保留新的 ad-hoc 审计 SQL 或第二套权威证据表，且关键事务遵循“主对象 + 审计 + outbox”同口径。
  验收：至少一条集成测试或手工 API 验证通过，并能证明 `catalog / search / billing dispute` 的审计与证据对象可在统一审计权威模型中联查，不再出现“对象在 PG + MinIO，但不在 audit authority model” 的分叉。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../开发准备/服务清单与服务边界正式版.md:L493（证据对象 / 证据包） | ../原始PRD/审计、证据链与回放设计.md:L140（5. 证据模型） | ../数据库设计/V1/upgrade/055_audit_hardening.sql:L50（audit evidence tables） | 问题修复任务/A03-统一事务模板-落地真实审计与Outbox-Writer.md | 问题修复任务/A06-Audit-Kit-统一模型漂移.md | 问题修复任务/A14-AUD-历史模块统一Writer与证据桥接缺口.md | 问题修复任务/A03-统一事务模板-落地真实审计与Outbox-Writer.md:L1（统一事务模板与 Writer 收口） | 问题修复任务/A14-AUD-历史模块统一Writer与证据桥接缺口.md:L1（历史模块统一 Writer 与证据桥接）
- **AUD-030** [AGENT][P0][W1][no] 收敛统一事件 envelope 与路由权威源：应用层 canonical outbox writer、`ops.event_route_policy`、统一顶层字段与 target topic 选择必须成为唯一正式路径，停止触发器自动派生与业务旁路写入并存。
  依赖：AUD-001; AUD-008; AUD-009; DB-008
  交付：apps/platform-core/src/shared/**; apps/platform-core/src/modules/**; docs/04-runbooks/**; docs/05-test-cases/**
  完成定义：统一事件 envelope 顶层字段、canonical outbox writer 与 `ops.event_route_policy` 已成为唯一正式来源；不再允许 trigger 自动派生、业务旁路写入与手工路由并存。
  验收：至少一条集成测试或手工 API 验证通过，并能证明同一业务动作不会重复写出不同协议事件，且 target topic 来源唯一。
  阻塞风险：事件 envelope 与路由权威不唯一会导致 publisher、consumer、DLQ、回放和审计无法共享同一协议。
  技术参考：../开发准备/事件模型与Topic清单正式版.md:L80（正式事件 envelope 与 route policy） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L20（事件模型与路由） | ../原始PRD/双层权威模型与链上链下一致性设计.md:L266（outbox 与 publisher） | 问题修复任务/A02-统一事件-Envelope-与路由权威源.md:L1（统一事件 Envelope 与路由权威源）
- **AUD-031** [AGENT][P0][W1][no] 清理把 `ops.outbox_event` 当私有工作队列的旁路实现，统一收口到 `outbox publisher -> Kafka -> consumer -> dead letter / reprocess` 正式闭环，并补 publish attempt 联查。
  依赖：AUD-008; AUD-009; AUD-010; BIL-024
  交付：workers/outbox-publisher/**; apps/platform-core/src/modules/billing/**; docs/04-runbooks/**; docs/05-test-cases/**
  完成定义：正式 outbox publisher、`outbox_publish_attempt`、DB/Kafka 双层 DLQ 与 reprocess 闭环已建立；业务模块不再直接把 `ops.outbox_event` 当私有工作队列消费。
  验收：至少一条集成测试或手工 API 验证通过，并能证明 `outbox_publish_attempt` 可联查、失败隔离成立，且历史旁路消费已被移除或桥接到正式链路。
  阻塞风险：若继续保留 `ops.outbox_event` 私有工作队列旁路，AUD / NOTIF / SEARCHREC 后续实现会在错误分发语义上持续扩散。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L307（consumer / DLQ / replay） | ../数据库设计/V1/upgrade/056_dual_authority_consistency.sql:L1（dual authority consistency schema） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L80（dead letter 重处理） | 问题修复任务/A05-Outbox-Publisher-DLQ-统一闭环缺口.md:L1（Outbox Publisher 与 DLQ 闭环）
## 16. Search / Recommendation / Projection [SEARCHREC]

这一组落地搜索读模型、索引同步与最小推荐闭环。

- **SEARCHREC-001** [AGENT][P0][W2][no] 初始化 `workers/search-indexer/`，消费 `dtp.search.sync` 主题，把目录与卖方资料投影到 OpenSearch；`search.index_sync_task` 仅作为同步状态与重建作业记录表，不再作为默认主消费路径；同时明确 `staging / production` 使用 OpenSearch、`local / demo` 允许 PG 搜索投影 fallback。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：workers/search-indexer/
  完成定义：业务规则、状态机、审计、事件与测试已齐备；`staging / production` 以 OpenSearch 为正式候选源，`local / demo` 的 PG fallback 边界与启动条件已明确，且最终仍回 PostgreSQL 做可见性校验。
  验收：至少一条集成测试或手工 API 验证通过，并能区分 OpenSearch 正式模式与 `local / demo` PG fallback 模式。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L140（local/demo 允许 PG 投影运行） | ../原始PRD/商品搜索、排序与索引同步设计.md:L153（PostgreSQL fallback 搜索） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | 问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md | 问题修复任务/A01-Kafka-Topic-口径统一.md:L1（Kafka topic 口径统一） | 问题修复任务/A02-统一事件-Envelope-与路由权威源.md:L1（统一事件 Envelope 与路由权威源） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md:L1（SEARCHREC Consumer 幂等与 DLQ 闭环）
- **SEARCHREC-002** [AGENT][P0][W2][no] 落地商品搜索投影结构，至少包含：标题、摘要、行业、标签、类目、SKU、卖方、价格区间、上架状态、审核状态、可见性。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | 问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md
- **SEARCHREC-003** [AGENT][P0][W2][no] 落地卖方搜索投影结构，至少包含：主体名称、行业、地区、认证标签、主打商品、评分摘要。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | 问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md
- **SEARCHREC-004** [AGENT][P0][W2][no] 实现 `GET /api/v1/catalog/search`；`staging / production` 先查 OpenSearch，`local / demo` 允许走 PG 搜索投影 fallback，最终都必须回 PostgreSQL 做状态与可见性校验。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；运行模式边界清晰，生产基线使用 OpenSearch，`local / demo` 允许 PG fallback，且两种模式最终都回 PostgreSQL 做可见性校验；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能验证生产基线与 `local / demo` fallback 的差异化运行口径。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L140（local/demo 允许 PG 投影运行） | ../原始PRD/商品搜索、排序与索引同步设计.md:L153（PostgreSQL fallback 搜索） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | 问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md
- **SEARCHREC-005** [AGENT][P0][W2][no] 实现搜索排序基线：相关性、更新时间、热度、信誉权重的简单组合；不得把推荐逻辑硬塞进搜索主排序。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | 问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md
- **SEARCHREC-006** [AGENT][P0][W2][no] 实现搜索缓存与失效策略，结合 Redis 减少弱一致短时抖动，并统一到 `datab:v1:search:catalog:*` 等正式 key 命名。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | 问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md | 问题修复任务/A12-配置项与资源命名漂移.md:L1（配置项与资源命名漂移）
- **SEARCHREC-007** [AGENT][P0][W2][no] 实现搜索同步作业表与异常记录表，支持重试、对账与 ops 查看，并为后续 alias authority / alias switch 运维能力提供统一状态落点。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | 问题修复任务/A07-搜索同步链路与搜索接口闭环缺口.md | 问题修复任务/A08-搜索Alias权威源与阶段边界冲突.md:L1（搜索 Alias 权威源与阶段边界）
- **SEARCHREC-008** [AGENT][P0][W2][no] 初始化推荐模块基础模型：placement、candidate source、ranking profile、exposure/click event。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论） | ../原始PRD/商品推荐与个性化发现设计.md:L112（6. 推荐位设计） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L23（3. 推荐请求接口） | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约）
- **SEARCHREC-009** [AGENT][P0][W2][no] 实现推荐结果接口 `GET /api/v1/recommendations`，返回前必须回 PostgreSQL 校验。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论） | ../原始PRD/商品推荐与个性化发现设计.md:L112（6. 推荐位设计） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L23（3. 推荐请求接口） | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约）
- **SEARCHREC-010** [AGENT][P0][W2][no] 实现推荐曝光/点击记录接口 `POST /api/v1/recommendations/track/exposure`、`/click`，要求幂等、审计，并与 `recommend.behavior_recorded -> dtp.recommend.behavior` 正式行为流契约对齐。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论） | ../原始PRD/商品推荐与个性化发现设计.md:L112（6. 推荐位设计） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L23（3. 推荐请求接口） | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md:L1（SEARCHREC Consumer 幂等与 DLQ 闭环）
- **SEARCHREC-011** [AGENT][P0][W2][no] 实现推荐位配置接口 `GET/PATCH /api/v1/ops/recommendation/placements*`。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论） | ../原始PRD/商品推荐与个性化发现设计.md:L112（6. 推荐位设计） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L23（3. 推荐请求接口） | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约）
- **SEARCHREC-012** [AGENT][P0][W2][no] 实现推荐重建接口 `POST /api/v1/ops/recommendation/rebuild`，允许重算缓存或候选结果。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：接口、DTO、权限校验、审计、错误码和最小测试已齐备；实现与 OpenAPI 不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论） | ../原始PRD/商品推荐与个性化发现设计.md:L112（6. 推荐位设计） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L23（3. 推荐请求接口） | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约）
- **SEARCHREC-013** [AGENT][P0][W2][no] 为 local 模式准备最小候选召回策略：最新上架、同类目、同卖方、热销、零结果兜底。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论） | ../原始PRD/商品推荐与个性化发现设计.md:L112（6. 推荐位设计） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L23（3. 推荐请求接口） | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约）
- **SEARCHREC-014** [AGENT][P0][W2][no] 为五条标准链路商品准备固定推荐位样例，确保演示环境首页可直接进入闭环。
  依赖：CAT-001; DB-011; DB-012; CORE-008
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；与上下游模块联调通过。
  验收：至少一条集成测试或手工 API 验证通过，并能在审计/日志中看到对应痕迹。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论） | ../原始PRD/商品推荐与个性化发现设计.md:L112（6. 推荐位设计） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L23（3. 推荐请求接口） | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约）
- **SEARCHREC-018** [AGENT][P0][W1][no] 将搜索/推荐 handler 与 service 切到统一鉴权门面和正式权限点，移除 `x-role` 占位语义。
  依赖：AUD-022; SEARCHREC-004; SEARCHREC-009; SEARCHREC-010; SEARCHREC-011; SEARCHREC-012
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; docs/04-runbooks/search-reindex.md; docs/04-runbooks/recommendation-runtime.md
  完成定义：搜索/推荐接口统一接入正式 `Authorization` 鉴权门面与权限点；写接口头约束与幂等要求齐备；运行时不再读取 `x-role` 占位。
  验收：至少一条集成测试或手工 API 验证通过，并能证明 `Authorization + 正式权限点` 已替代 `x-role`。
  阻塞风险：若继续保留占位权限语义，后续 `step-up`、OpenAPI 与测试都会在错误契约上叠加返工。
  技术参考：../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L22（搜索请求头与权威边界） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L129（推荐鉴权与审计） | ../权限设计/接口权限校验清单.md:L155（搜索运维权限与 step-up） | ../权限设计/接口权限校验清单.md:L420（推荐读取与配置权限） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md:L1（SEARCHREC 契约收口）
- **SEARCHREC-019** [AGENT][P0][W1][no] 为 SEARCHREC 运维写接口接入 `step-up`、审计留痕与搜索域 `SEARCH_*` 错误码，并统一推荐运维写接口的高风险控制。
  依赖：AUD-022; SEARCHREC-018
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; apps/platform-core/src/modules/audit/**; docs/04-runbooks/search-reindex.md; docs/04-runbooks/recommendation-runtime.md
  完成定义：搜索运维写接口已接正式 `X-Step-Up-Token`、审计与 `SEARCH_*` 错误码；推荐运维写接口的高风险控制与审计要求已按冻结口径落地；实现与冻结文档不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能验证 `step-up`、审计痕迹和搜索域错误码。
  阻塞风险：若高风险写接口继续沿用 header presence / 通用错误码，占位口径会被进一步固化到 OpenAPI 和测试。
  技术参考：../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L26（搜索写接口幂等） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L41（搜索运维接口族） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L86（搜索错误码） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L137（推荐运维修改需审计并建议 step-up） | ../权限设计/接口权限校验清单.md:L156（搜索重建高风险控制） | ../权限设计/接口权限校验清单.md:L424（推荐配置修改高风险控制） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md:L82（审计与 Step-Up） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md:L1（SEARCHREC 契约收口）
- **SEARCHREC-020** [AGENT][P0][W1][no] 为 `search-indexer` 与 `recommendation-aggregator` 补齐 Kafka consumer 可靠性闭环：基于 `event_id` 的 consumer 幂等、`ops.consumer_idempotency_record`、数据库 `ops.dead_letter_event` 与 Kafka `dtp.dead-letter` 双层隔离、失败后再决定 offset 提交，以及与 `AUD-010` 的 dry-run / step-up 重处理对接。
  依赖：AUD-008; AUD-010; SEARCHREC-001; SEARCHREC-010
  交付：workers/search-indexer/**; workers/recommendation-aggregator/**; apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; docs/04-runbooks/search-reindex.md; docs/04-runbooks/recommendation-runtime.md
  完成定义：两个 SEARCHREC consumer 都已按统一事件 envelope 基于 `event_id` 幂等消费；失败路径能同时进入 `ops.dead_letter_event` 与 `dtp.dead-letter`；只有在成功处理或失败已安全隔离后才提交 offset；实现与统一重处理口径不漂移。
  验收：至少一条集成测试或手工 API 验证通过，并能证明 `search-indexer` 与 `recommendation-aggregator` 的 worker 侧副作用、幂等、DB/Kafka 双层 DLQ 与 reprocess 路径成立。
  阻塞风险：若 consumer 失败仍直接提交 offset，搜索投影与推荐行为流会出现静默丢数，后续审计、联查与回放都无法补救。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L307（consumer / DLQ / replay） | ../数据库设计/V1/upgrade/056_dual_authority_consistency.sql:L90（consumer_idempotency_record） | ../数据库设计/接口协议/一致性与事件接口协议正式版.md:L80（dead letter 重处理） | ../开发准备/事件模型与Topic清单正式版.md:L199（统一 DLQ topic） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md:L1（SEARCHREC Consumer 幂等与 DLQ 闭环）
- **SEARCHREC-015** [AGENT][P1][W3][no] 为搜索与推荐编写一致性与 worker 可靠性测试：商品下架后搜索结果消失、冻结后推荐不可见、重建后 alias 生效，并覆盖统一鉴权 / step-up / 审计 / 错误码、consumer 幂等、双层 DLQ 与可重处理口径。
  依赖：CAT-001; DB-011; DB-012; CORE-008; SEARCHREC-019; SEARCHREC-020
  交付：apps/platform-core/src/modules/search/**; apps/platform-core/src/modules/recommendation/**; workers/search-indexer/**; workers/recommendation-aggregator/**
  完成定义：业务规则、状态机、审计、事件与测试已齐备；统一鉴权、正式权限点、必要 `step-up`、审计、错误码、worker 侧副作用、失败隔离与重处理校验齐备；测试不再使用 `x-role` 占位。
  验收：至少一条集成测试或手工 API 验证通过，并能覆盖 `Authorization`、`X-Idempotency-Key`、必要 `X-Step-Up-Token`、审计痕迹、`SEARCH_*` 错误码、consumer 幂等、DB/Kafka 双层 DLQ 与 reprocess 路径。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L129（推荐运维与权限口径） | ../权限设计/接口权限校验清单.md:L155（SEARCHREC 权限与 step-up） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md | 问题修复任务/A08-搜索Alias权威源与阶段边界冲突.md:L1（搜索 Alias 权威源与阶段边界） | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md:L1（SEARCHREC 契约收口） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md:L1（SEARCHREC Consumer 幂等与 DLQ 闭环）
- **SEARCHREC-016** [AGENT][P1][W3][no] 生成 `docs/02-openapi/search.yaml` / `recommendation.yaml` 第一版并与实现校验；冻结 `Authorization / 权限点 / Idempotency / Step-Up / 审计 / 错误码` 口径。
  依赖：CAT-001; DB-011; DB-012; CORE-008; SEARCHREC-015
  交付：docs/02-openapi/search.yaml; recommendation.yaml
  完成定义：Search / Recommendation 归档占位已建立；当前实现期唯一设计参考固定为 `packages/openapi/search.yaml`、`packages/openapi/recommendation.yaml`；进入实现批次后必须把统一鉴权 / 权限点 / `X-Idempotency-Key` / 必要 `X-Step-Up-Token` / 审计 / 错误码补齐后再生成归档第一版。
  验收：至少一条集成测试或手工 API 验证通过，并能证明 OpenAPI 不再使用 `x-role` 占位，且与正式权限和错误码口径一致。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L129（推荐运维与权限口径） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md:L1（SEARCHREC 契约收口）
- **SEARCHREC-017** [AGENT][P1][W3][no] 生成 `docs/05-test-cases/search-rec-cases.md`，覆盖投影延迟、回 PG 校验、推荐曝光幂等、零结果兜底，并登记统一鉴权 / step-up / 审计 / 错误码、consumer 幂等、双层 DLQ 与可重处理验收项。
  依赖：CAT-001; DB-011; DB-012; CORE-008; SEARCHREC-016; SEARCHREC-020
  交付：docs/05-test-cases/search-rec-cases.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用，并明确登记统一鉴权 / `step-up` / 审计 / 错误码、consumer 幂等、双层 DLQ 与可重处理验收项，禁止继续使用 `x-role` 占位语义。
  验收：至少一条集成测试或手工 API 验证通过，并能证明测试矩阵覆盖 `Authorization`、正式权限点、必要 `X-Step-Up-Token`、审计留痕、搜索域错误码、worker 侧副作用与 DLQ/reprocess 路径。
  阻塞风险：依赖模块未就绪时容易出现返工或实现口径不一致。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品搜索、排序与索引同步设计.md:L164（6. 搜索投影设计） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | ../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md:L129（推荐运维与权限口径） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md | 问题修复任务/A09-推荐主链路与行为流契约缺口.md:L1（推荐主链路与行为流契约） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险） | 问题修复任务/A13-SEARCHREC-统一鉴权-Step-Up-审计与契约口径缺口.md:L1（SEARCHREC 契约收口） | 问题修复任务/A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md:L1（SEARCHREC Consumer 幂等与 DLQ 闭环）
- **SEARCHREC-021** [AGENT][P0][W1][no] 收敛搜索 alias 权威源与阶段边界：统一 `product_search_read/write`、`seller_search_read/write` 与 `search.index_alias_binding`，同步初始化脚本、运行默认值、ops 接口与 runbook，明确 alias switch 属于 V1 最小运维能力。
  依赖：SEARCHREC-007; AUD-022; ENV-017
  交付：apps/platform-core/src/modules/search/**; workers/search-indexer/**; infra/opensearch/**; docs/04-runbooks/search-reindex.md
  完成定义：搜索 alias 权威源、结构化命名和阶段边界已统一；`search.index_alias_binding`、初始化脚本、运行默认值、ops 接口与 runbook 共享同一套 alias 答案，且 alias switch 被明确纳入 V1 最小运维能力。
  验收：至少一条集成测试或手工 API 验证通过，并能证明 schema / 脚本 / runbook / ops 接口使用同一套 alias 名称与切换边界。
  阻塞风险：若 alias 权威源与阶段边界继续漂移，重建、切换、初始化脚本与运维接口会分别对着不同索引/别名工作。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（V1 正式搜索方案） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L41（搜索运维接口族） | ../开发准备/技术选型正式版.md:L24（OpenSearch 读模型边界） | ../开发准备/事件模型与Topic清单正式版.md:L80（搜索同步事件与 topic） | ../04-runbooks/search-reindex.md:L45（当前 V1 口径） | 问题修复任务/A08-搜索Alias权威源与阶段边界冲突.md:L1（搜索 Alias 权威源与阶段边界）
## 17. 前端最小页面闭环（portal-web / console-web） [WEB]

这一组落地门户和控制台的最小页面闭环。

- `PortalRouteScaffold / ConsoleRouteScaffold` 仅是布局与元信息容器，不构成任务完成证据。
- `preview` 仅用于显式调试态测试；正式页面与主回归验收必须基于真实 API 状态，不得依赖 `preview` URL 参数。

- **WEB-001** [AGENT][P0][W2][no] 初始化 `apps/portal-web/` Next.js 项目，接入 pnpm workspace、基础布局、登录态接入、API SDK。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L17（2. 页面域总览） | ../页面说明书/页面说明书-V1-完整版.md:L936（12. 页面间路由关系） | ../开发准备/技术选型正式版.md:L53（4. 语言分工）
- **WEB-002** [AGENT][P0][W2][no] 初始化 `apps/console-web/` Next.js 项目，承接运营、审计、开发者、ops 页面。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/console-web/
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L17（2. 页面域总览） | ../页面说明书/页面说明书-V1-完整版.md:L835（10.1 审计联查页） | ../开发准备/技术选型正式版.md:L53（4. 语言分工）
- **WEB-003** [AGENT][P0][W2][no] 实现门户首页：场景导航、推荐位、搜索入口、标准链路快捷入口。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L137（4.1 首页） | ../原始PRD/商品推荐与个性化发现设计.md:L112（6. 推荐位设计） | ../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路）
- **WEB-004** [AGENT][P0][W2][no] 实现商品搜索页：关键词、筛选、排序、结果卡片、空状态与错误状态。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L194（4.3 搜索页） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口） | ../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案）
- **WEB-005** [AGENT][P0][W2][no] 实现商品详情页：元信息、卖方信息、SKU、价格、样例预览、下单入口、审核状态徽标。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L262（4.5 产品详情页） | ../数据库设计/接口协议/目录与商品接口协议正式版.md:L82（5. V1 接口） | ../权限设计/按钮级权限说明.md:L43（3. V1 关键页面按钮说明）
- **WEB-006** [AGENT][P0][W2][no] 实现卖方主页：主体信息、认证标识、商品列表、联系方式/咨询入口。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L238（4.4 卖方主页） | ../业务流程/业务流程图-V1-完整版.md:L253（4.3.1 卖方主页查看流程） | ../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md:L34（4. V1 接口）
- **WEB-007** [AGENT][P0][W2][no] 实现卖方上架中心：商品草稿、SKU 编辑、元信息、质量报告、模板绑定、提交审核。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L307（5.1 上架中心） | ../页面说明书/页面说明书-V1-完整版.md:L333（5.2 产品编辑页） | ../页面说明书/页面说明书-V1-完整版.md:L432（5.3 SKU 配置页）
- **WEB-008** [AGENT][P0][W2][no] 实现审核工作台：主体审核、商品审核、合规审核列表与详情。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L796（9.1 主体审核台） | ../页面说明书/页面说明书-V1-完整版.md:L802（9.2 产品审核台） | ../页面说明书/页面说明书-V1-完整版.md:L808（9.3 合规审核台）
- **WEB-009** [AGENT][P0][W2][no] 实现订单创建与订单详情页，必须显式支持 8 个标准 SKU（FILE_STD/FILE_SUB/SHARE_RO/API_SUB/API_PPU/QRY_LITE/SBX_STD/RPT_STD），并按五条标准链路提供官方命名的下单入口与详情视图。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L501（6.1 询单/下单页） | ../页面说明书/页面说明书-V1-完整版.md:L556（6.4 订单详情页） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
- **WEB-010** [AGENT][P0][W2][no] 实现交付中心页面：文件交付、API 开通、共享开通、模板授权、沙箱开通、报告交付入口。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L590（7.1 文件交付页） | ../页面说明书/页面说明书-V1-完整版.md:L611（7.2 API 开通页） | ../页面说明书/页面说明书-V1-完整版.md:L625（7.3 只读共享开通页）
- **WEB-011** [AGENT][P0][W2][no] 实现验收页面：通过、拒收、拒收原因、生命周期摘要。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L714（7.9 验收页） | ../权限设计/按钮级权限说明.md:L43（3. V1 关键页面按钮说明） | ../业务流程/业务流程图-V1-完整版.md:L268（4.4 交付、验真与验收主流程）
- **WEB-012** [AGENT][P0][W2][no] 实现账单页面：账单明细、支付状态、退款/赔付状态、争议入口。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L738（8.1 账单中心） | ../页面说明书/页面说明书-V1-完整版.md:L759（8.2 退款/赔付处理页） | ../全集成文档/数据交易平台-全集成基线-V1.md:L7194（27. 支付、资金流与轻结算）
- **WEB-013** [AGENT][P0][W2][no] 实现争议页面：创建案件、上传证据、查看裁决。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L773（8.3 争议提交页） | ../业务流程/业务流程图-V1-完整版.md:L473（5.3 争议处理流程） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口）
- **WEB-014** [AGENT][P0][W2][no] 实现审计联查页：按订单号查看审计事件、证据对象、链回执、外部事实。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L835（10.1 审计联查页） | ../页面说明书/页面说明书-V1-完整版.md:L858（10.2 证据包导出页） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口）
- **WEB-015** [AGENT][P0][W2][no] 实现 ops 页面：outbox、dead letter、一致性联查、搜索同步、推荐重建、观测总览入口。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L921（11.5 搜索运维页） | ../页面说明书/页面说明书-V1-完整版.md:L909（11.3 状态联查页） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈）
- **WEB-016** [AGENT][P0][W2][no] 实现开发者页面：应用管理、API Key、调用日志、trace 联查、Mock 支付操作入口。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L889（11.1 开发者首页） | ../页面说明书/页面说明书-V1-完整版.md:L903（11.2 测试应用页） | ../页面说明书/页面说明书-V1-完整版.md:L909（11.3 状态联查页）
- **WEB-017** [AGENT][P1][W3][no] 为五条标准链路各做一个从首页直达的演示路径和说明卡片。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../页面说明书/页面说明书-V1-完整版.md:L137（4.1 首页） | ../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../页面说明书/页面说明书-V1-完整版.md:L936（12. 页面间路由关系）
- **WEB-018** [AGENT][P1][W3][no] 为 portal/console 编写最小 E2E 测试：登录、搜索、商品查看、下单、交付、验收、联查；拆分 live 回归与 preview 状态预演两套用例并独立执行。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过；live 套件不得依赖 `preview` URL 参数，预演态需独立测试套件承接。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../页面说明书/页面说明书-V1-完整版.md:L996（14. 页面覆盖校验） | ../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路）
- **WEB-019** [AGENT][P1][W3][no] 建立前端错误码到文案映射，确保和后端统一错误码字典对齐。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L5（12.1 API 设计规范） | ../权限设计/按钮级权限说明.md:L43（3. V1 关键页面按钮说明） | ../数据库设计/接口协议/身份与会话接口协议正式版.md:L36（4. V1 接口）
- **WEB-020** [AGENT][P1][W3][no] 生成 `docs/05-test-cases/web-smoke-cases.md`。
  依赖：BOOT-007; CORE-026; TRADE-028; BIL-020
  交付：docs/05-test-cases/web-smoke-cases.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../页面说明书/页面说明书-V1-完整版.md:L996（14. 页面覆盖校验） | ../页面说明书/页面说明书-V1-完整版.md:L936（12. 页面间路由关系）
- **WEB-021** [AGENT][P0][W2][no] 在订单创建页、商品详情页、演示入口页显式展示“场景名 -> 主 SKU / 补充 SKU”映射，避免前端把 SHARE_RO、QRY_LITE、RPT_STD 再次并回大类。
  依赖：WEB-005; WEB-009; CTX-021; DB-035
  交付：apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../页面说明书/页面说明书-V1-完整版.md:L501（6.1 询单/下单页） | ../页面说明书/页面说明书-V1-完整版.md:L262（4.5 产品详情页）
- **WEB-022** [AGENT][P1][W3][no] 在 console 中补充通知联查页或嵌入式面板，页面通过 `console-web` 同源代理调用 `platform-core` 的 `/api/v1/ops/notifications/*` 正式 facade，由 `platform-core` 内部转发 `notification-worker`，支持按订单号查看已发送通知、失败通知、重试状态、关联模板与 dead letter replay，不得使用 mock 或 UI 占位。
  依赖：NOTIF-010; WEB-015
  交付：apps/platform-core/**; apps/portal-web/**; apps/console-web/**; packages/sdk-ts/**; docs/05-test-cases/notification-cases.md; docs/04-runbooks/notification-worker.md
  完成定义：页面可访问；空态/错态/权限态可用；与接口契约对齐；`console-web` 仅经 `/api/platform/**` 调用 `platform-core` 正式 API；`platform-core` 提供通知联查 / replay facade 并内部转发 `notification-worker`；最小 E2E 或手工 smoke 通过。
  验收：页面手工 smoke 或 Playwright/Cypress 最小链路通过，并能证明浏览器只访问同源页面与 `/api/platform/**`；通知联查 / replay 由 `platform-core` facade 承接。
  阻塞风险：前端命名或展示漂移会让场景与 SKU 真值失配。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../页面说明书/页面说明书-V1-完整版.md:L835（10.1 审计联查页） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../05-test-cases/notification-cases.md:L1（通知验收路径） | ../04-runbooks/notification-worker.md:L1（通知 worker 与 facade 边界） | ../开发准备/服务清单与服务边界正式版.md:L1（前端与 worker 边界） | ../packages/openapi/ops.yaml:L1（通知 facade 契约）

## 18. 测试、演示数据、验收与 CI [TEST]

这一组落地单元、契约、E2E、CI 与最终验收包装。

- **TEST-001** [ARCH][P0][W3][no] 在 `fixtures/demo/` 生成五条标准链路的完整演示数据包：主体、商品、SKU、模板、订单、交付对象、账单样例、审计样例。
  依赖：ENV-040; DB-032; CORE-024
  交付：fixtures/demo/
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-002** [ARCH][P0][W3][no] 设计 `seed-demo.sh`，支持一键导入演示租户、演示用户、演示商品、演示订单、演示支付与交付记录。
  依赖：ENV-040; DB-032; CORE-024
  交付：seed-demo.sh
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-003** [AGENT][P0][W3][no] 建立 contract test 目录，覆盖 OpenAPI schema、错误码、状态机枚举、关键响应字段。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L5（12.1 API 设计规范） | ../全集成文档/数据交易平台-全集成基线-V1.md:L34851（9. 接口、事件与集成协议） | ../页面说明书/页面说明书-V1-完整版.md:L996（14. 页面覆盖校验）
- **TEST-004** [AGENT][P0][W3][no] 建立 migration smoke test，验证空库升级、种子导入、应用启动、重置回滚与重新升级。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../数据库设计/数据库设计总说明.md:L176（7. 迁移执行顺序） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../数据库设计/README.md:L67（4. 迁移策略）
- **TEST-005** [AGENT][P0][W3][no] 建立本地环境 smoke test，验证 compose 启动、核心服务 ready、Grafana 数据源可连、Keycloak realm 导入成功，并校验 canonical topics 与关键控制面入口不再回退到旧口径。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单；宿主机与容器内 Kafka 地址边界已写入公共前置条件与活跃 test-case。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果；宿主机示例统一使用 `127.0.0.1:9094`，容器内探测继续使用 `kafka:9092` / `localhost:9092`。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../05-test-cases/README.md:L7（测试样例公共前置条件） | ../04-runbooks/local-startup.md:L39（宿主机 Kafka 地址） | ../04-runbooks/port-matrix.md:L15（Kafka 外部端口矩阵） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险）
- **TEST-006** [AGENT][P0][W3][no] 建立订单端到端测试，严格按五条标准链路命名与验收：工业设备运行指标 API 订阅、工业质量与产线日报文件包交付、供应链协同查询沙箱、零售门店经营分析 API / 报告订阅、商圈/门店选址查询服务。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-007** [AGENT][P0][W3][no] 建立 Provider 切换测试：mock 支付 / mock 链写 / mock 签章 与 real 占位实现的切换不改业务代码。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-008** [AGENT][P0][W3][no] 建立 outbox 一致性测试：DB 事务成功时有 outbox，事务失败时无 outbox，重复消费不重复产生副作用。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../原始PRD/双层权威模型与链上链下一致性设计.md:L329（7.1 outbox_event） | ../原始PRD/双层权威模型与链上链下一致性设计.md:L441（10. 不允许的错误模型） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-009** [AGENT][P0][W3][no] 建立审计完备性测试：关键操作必须产生审计事件，证据导出必须 step-up，非法导出被拒绝。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../页面说明书/页面说明书-V1-完整版.md:L858（10.2 证据包导出页） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-010** [AGENT][P0][W3][no] 建立搜索与推荐回 PG 校验测试：下架/冻结商品不可在结果中漏校验出现。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案） | ../原始PRD/商品推荐与个性化发现设计.md:L54（3. 架构结论） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-011** [AGENT][P0][W3][no] 建立支付 webhook 幂等测试：重复 success 回调、success 后 fail 回调、timeout 后 success 回调。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../数据库设计/接口协议/支付域接口协议正式版.md:L158（6. 幂等与一致性） | ../原始PRD/支付、资金流与轻结算设计.md:L36（4. 分层架构） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-012** [AGENT][P0][W3][no] 建立交付与断权测试：撤权后下载票据失效、API key 失效、共享授权不可用、沙箱会话终止。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L430（5. 异常与争议流程） | ../领域模型/全量领域模型与对象关系说明.md:L709（4.5 交付与执行聚合） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-013** [AGENT][P0][W3][no] 建立争议与结算联动测试：争议中冻结结算、裁决后退款或赔付正确入账。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L473（5.3 争议处理流程） | ../原始PRD/支付、资金流与轻结算设计.md:L165（7. 平台收费规则设计） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-014** [AGENT][P0][W3][no] 建立审计回放 dry-run 测试：能按订单回放关键状态并输出差异报告。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../原始PRD/审计、证据链与回放设计.md:L208（8. 回放设计） | ../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md:L102（5. V1 接口） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-015** [AGENT][P0][W3][no] 建立 CI 流水线最小矩阵：Rust lint/test、TS lint/test、Go build/test、migration check、OpenAPI check。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-016** [AGENT][P0][W3][no] 在 CI 中加入 compose 级别 smoke 作业，至少启动核心服务并执行健康检查，同时校验 canonical topics / consumer group / 关键 OpenAPI 归档不再漂移到旧命名或骨架接口。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险）
- **TEST-017** [AGENT][P0][W3][no] 在 CI 中加入 schema drift 检查，避免代码实体与 migration/OpenAPI 漂移。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md:L43（14.4 持续交付与版本治理） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-018** [AGENT][P1][W3][no] 补充性能冒烟：单次搜索、下单、交付、审计联查的基础响应时间门槛。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准） | ../原始PRD/日志、可观测性与告警设计.md:L58（4. V1 正式采用的观测技术栈） | ../原始PRD/商品搜索、排序与索引同步设计.md:L128（5. V1 正式方案）
- **TEST-019** [AGENT][P1][W3][no] 补充故障演练脚本：Kafka 停机、Fabric Adapter 停机、OpenSearch 不可用、Mock Payment 延迟，验证主链路退化行为。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../原始PRD/链上链下技术架构与能力边界稿.md:L54（4. 分层架构） | ../原始PRD/交易链监控、公平性与信任安全设计.md:L174（5. 六层交易链监控模型） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-020** [AGENT][P1][W3][no] 补充回滚演练脚本：重置本地库、重放 seed、重新启动环境、恢复演示数据。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../数据库设计/数据库设计总说明.md:L176（7. 迁移执行顺序） | ../开发准备/技术选型正式版.md:L147（6. 本地与联调环境） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-021** [AGENT][P1][W3][no] 输出 `docs/05-test-cases/v1-core-acceptance-checklist.md`，把 V1 退出标准转化为可执行验收用例。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/v1-core-acceptance-checklist.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-022** [AGENT][P1][W3][no] 输出 `docs/05-test-cases/five-standard-scenarios-e2e.md`，明确每条标准链路的输入数据、期望状态、验证点，供后续顺序执行与回归验证使用。
  依赖：ENV-040; DB-032; CORE-024
  交付：docs/05-test-cases/five-standard-scenarios-e2e.md
  完成定义：文档/规则文件已落盘；主结构完整；与现有术语和命名一致；被 README、索引或上游任务引用。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-023** [AGENT][P0][W3][no] 建立 8 个标准 SKU 覆盖矩阵测试，验证每个 SKU 至少有一条主路径、一条异常路径、一条退款或争议路径，并与五条标准链路建立映射。
  依赖：TRADE-029; DB-034; DB-035
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-024** [AGENT][P0][W3][no] 建立“支付成功 -> 待交付 -> 交付完成 -> 待验收 -> 验收通过/拒收 -> 结算/退款”编排链路测试，重点覆盖 webhook 乱序、交付重复、验收重复、结算重算。
  依赖：TRADE-030; DLV-029; DLV-030; BIL-024; BIL-025
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../全集成文档/数据交易平台-全集成基线-V1.md:L216（5.3.2 首批 5 条标准链路） | ../全集成文档/数据交易平台-全集成基线-V1.md:L229（5.3.2A 首批标准场景到 V1 SKU 与模板映射） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（15.2 分阶段验收标准）
- **TEST-025** [AGENT][P0][W3][no] 建立 `SHARE_RO` 端到端测试：共享开通、访问授权、撤销、争议冻结、恢复或退款，确保该 SKU 不是只停留在状态机定义。
  依赖：TRADE-012; DLV-006; BIL-026
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L314（4.4.1B 只读共享交付） | ../页面说明书/页面说明书-V1-完整版.md:L625（7.3 只读共享开通页） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-026** [AGENT][P0][W3][no] 建立 `QRY_LITE` 端到端测试：模板授权、参数校验、执行成功、结果可取、验收关闭、退款/拒绝非法重复执行。
  依赖：TRADE-013; DLV-011; DLV-012; BIL-024
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：测试脚本可在本地或 CI 运行；断言明确；失败可定位；结果纳入验收清单。
  验收：测试在本地和 CI 至少一处可重复通过，并产生可读结果。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../业务流程/业务流程图-V1-完整版.md:L349（4.4.3 沙箱 / 模板查询类交付） | ../页面说明书/页面说明书-V1-完整版.md:L685（7.7 查询运行与结果记录页） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略）
- **TEST-027** [AGENT][P1][W3][no] 建立通知链路 smoke test，验证支付成功、交付完成、验收通过、争议升级至少四类事件会通过 `notification.requested -> dtp.notification.dispatch -> notification-worker` 触发 `mock-log` 通知并留下审计记录。
  依赖：NOTIF-004; NOTIF-005; NOTIF-006; NOTIF-009
  交付：docs/05-test-cases/**; fixtures/**; .github/workflows/**
  完成定义：当前执行源已登记通知 smoke 的目标场景、正式事件链与后续 `notification-cases.md` 承接，不再将通知 smoke 视为已跑通。
  验收：当前阶段以 `docs/05-test-cases/README.md`、runbook 与 TODO 已明确 smoke 义务和后续落盘项为准；进入 `NOTIF` 代码实现批次后再以本地 / CI 可重复通过的通知 smoke 结果验收。
  阻塞风险：没有自动化验证会让并行开发后难以回归收敛。
  技术参考：../data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md:L15（12.2 事件模型） | ../原始PRD/审计、证据链与回放设计.md:L93（4. 审计事件模型） | ../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L5（15.1 测试策略） | ../04-runbooks/notification-worker.md:L12（当前批次边界） | ../05-test-cases/README.md:L24（通知验收清单尚未落盘） | V1-Core-TODO与预留清单.md:L65（NOTIF OpenAPI 与测试义务） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险）
- **TEST-028** [AGENT][P0][W3][no] 建立 canonical smoke / contract checker，强制校验正式 topic、正式接口、正式 consumer group、OpenAPI 示例与验收矩阵，不再允许只测旧 topic、骨架接口或局部 outbox 行；全量 topic existence smoke 不得再被局部静态 topology checker 替代。
  依赖：TEST-005; TEST-016; AUD-028; SEARCHREC-017; NOTIF-014
  交付：scripts/**; docs/05-test-cases/**; .github/workflows/**
  完成定义：本地与 CI 的 smoke / contract checker 已对准正式接口、canonical topic、consumer group、OpenAPI 示例与验收矩阵；不再允许只检查旧 topic、骨架路由或局部 outbox 行；宿主机与容器内 Kafka 地址边界能被显式校验；`check-topic-topology.sh` 与 `smoke-local.sh` 的职责边界已在 runbook 与测试说明中冻结。
  验收：测试在本地和 CI 至少一处可重复通过，并能证明错误 topic、旧命名、占位鉴权、骨架接口、宿主机误用 `127.0.0.1:9092` 或把局部 topology checker 误当全量 canonical checker 的用法会被显式拦截。
  阻塞风险：错误正反馈会让 topic、OpenAPI、通知和 SEARCHREC 漂移在“测试通过”的掩护下继续扩散。
  技术参考：../data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md:L18（分阶段验收标准） | ../05-test-cases/README.md:L7（测试样例公共前置条件） | ../04-runbooks/port-matrix.md:L15（Kafka 外部端口矩阵） | ../04-runbooks/local-startup.md:L39（宿主机 Kafka 地址） | 问题修复任务/A01-Kafka-Topic-口径统一.md:L1（Kafka topic 口径统一） | 问题修复任务/A11-测试与Smoke口径误报风险.md:L1（测试与 Smoke 口径误报风险）
