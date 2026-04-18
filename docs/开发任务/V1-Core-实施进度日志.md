# V1-Core 实施进度日志

本文件用于汇总记录每个实现批次的目标、实现结果、验证结果和待审批状态。

## 记录规则

- 每个批次追加一个新小节
- 不覆盖历史记录
- 批次编号建议格式：`BATCH-001`、`BATCH-002`
- 当前批次未审批通过前，不得开始下一个批次
- 每个批次至少出现两次记录：
  - `状态：计划中`
  - `状态：待审批`
- 如发生返工，必须追加：
  - `状态：返工中`
  - `状态：返工后待审批`

---

## 批次模板

### BATCH-XXX

- 状态：计划中 / 待审批 / 返工中 / 返工后待审批
- 当前任务编号：
- 当前批次目标：
- 前置依赖核对结果：
- 预计涉及文件：
- 已实现功能：
- 涉及文件：
- 验证步骤：
- 验证结果：
- 覆盖的冻结文档条目：
- 覆盖的任务清单条目：
- 未覆盖项：
- 新增 TODO / 预留项：
- 待人工审批结论：待审批
- 备注：

---

## 示例记录

### BATCH-001

- 状态：计划中
- 当前任务编号：BOOT-001, BOOT-002
- 当前批次目标：收敛当前仓库目录骨架，确认 `platform-core` 与基础目录符合单仓多应用结构要求。
- 前置依赖核对结果：无前置阻塞；`CTX` 上下文冻结任务已完成。
- 预计涉及文件：`apps/platform-core/`、`apps/portal-web/`、`apps/console-web/`、`infra/`、`开发任务/README.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：实现后执行目录结构核对；确认与任务清单、仓库结构建议一致。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`
- 覆盖的任务清单条目：`BOOT-001`, `BOOT-002`
- 未覆盖项：无
- 新增 TODO / 预留项：`TODO-CTX-019-001`、`TODO-CTX-020-001`（`V1-gap`，阻塞；用于跟踪历史前置任务交付文件缺失）。
- 待人工审批结论：待审批
- 备注：开始编码前记录。

### BATCH-001

- 状态：待审批
- 当前任务编号：BOOT-001, BOOT-002
- 当前批次目标：收敛当前仓库目录骨架，确认 `platform-core` 与基础目录符合单仓多应用结构要求。
- 前置依赖核对结果：无前置阻塞；`CTX` 上下文冻结任务已完成。
- 预计涉及文件：`apps/platform-core/`、`apps/portal-web/`、`apps/console-web/`、`infra/`、`开发任务/README.md`
- 已实现功能：完成仓库目录骨架校对与补齐；确认主应用与外围应用目录落位符合当前冻结架构。
- 涉及文件：`apps/platform-core/`、`开发任务/README.md`
- 验证步骤：1. 目录树核对；2. 对照任务清单检查交付路径；3. 对照冻结文档核对主应用和外围进程边界。
- 验证结果：通过；目录存在且命名符合冻结要求。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`BOOT-001`, `BOOT-002`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：等待人工确认后进入下一批。

### BATCH-002

- 状态：待审批
- 当前任务编号：CTX-001, CTX-002, CTX-003, CTX-004
- 当前批次目标：冻结 V1-Core 执行上下文，完成阅读索引、V1 核心守则、V1 范围边界和架构风格文档落盘，作为后续 BOOT/ENV/CORE 的统一前置约束。
- 前置依赖核对结果：四个任务 `depends_on` 均为空；当前仓库不存在真实历史审批记录（现有内容为模板与示例），不构成执行阻塞。
- 预计涉及文件：`docs/00-context/reading-index.md`、`docs/00-context/v1-core-guardrails.md`、`docs/00-context/v1-core-scope.md`、`docs/00-context/architecture-style.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成 `docs/00-context` 四份文档落盘；冻结主阅读顺序、V1 状态权威守则、V1 与 V2/V3 边界、模块化单体 + 外围进程架构风格；并在 TODO 汇总文件登记“本批次无新增 TODO”。
- 涉及文件：`docs/00-context/reading-index.md`、`docs/00-context/v1-core-guardrails.md`、`docs/00-context/v1-core-scope.md`、`docs/00-context/architecture-style.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 校验交付路径文件存在；2. 用关键字检查四个文档是否覆盖任务要求核心约束；3. 对照 CSV `CTX-001~CTX-004` 的交付与范围逐项核验。
- 验证结果：通过。文件存在性检查通过；关键约束语句齐备；任务交付路径与内容均覆盖。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`（主状态权威、职责边界、V1 与 V2/V3 分层）、`全集成文档/数据交易平台-全集成基线-V1.md`（范围定义、关键产品原则）、`全集成文档/数据交易平台-全集成基线-全阶段.md`（V2/V3 范围边界）、`开发准备/服务清单与服务边界正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`CTX-001`, `CTX-002`, `CTX-003`, `CTX-004`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已按流程暂停，等待人工在 `开发任务/V1-Core-人工审批记录.md` 给出审批结论后再进入下一批。

### BATCH-003

- 状态：待审批
- 当前任务编号：CTX-005, CTX-006, CTX-007, CTX-008
- 当前批次目标：冻结生命周期主对象、V1 标准 SKU 真值、首批五条标准链路映射和运行模式边界，为后续域模型与交易编排任务提供统一约束。
- 前置依赖核对结果：四个任务 `depends_on` 均为空；用户确认上一批已审批通过。
- 预计涉及文件：`docs/00-context/lifecycle-objects.md`、`docs/00-context/standard-sku-truth.md`、`docs/00-context/first-5-scenarios.md`、`docs/00-context/run-modes.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成生命周期对象冻结、8 个标准 SKU 真值冻结、首批 5 条标准链路与 SKU 映射冻结、三套运行模式冻结。
- 涉及文件：`docs/00-context/lifecycle-objects.md`、`docs/00-context/standard-sku-truth.md`、`docs/00-context/first-5-scenarios.md`、`docs/00-context/run-modes.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 交付路径文件存在性检查；2. 关键字检查对象/SKU/场景/运行模式四类约束；3. 对照 CSV `CTX-005~CTX-008` 逐条核验。
- 验证结果：通过。四个交付文件均已创建且内容覆盖任务描述，术语与冻结文档保持一致。
- 覆盖的冻结文档条目：`领域模型/全量领域模型与对象关系说明.md`（生命周期对象）、`全集成文档/数据交易平台-全集成基线-V1.md`（首批 5 条标准链路与 SKU 映射、范围边界）、`业务流程/业务流程图-V1-完整版.md`（主流程）、`开发准备/技术选型正式版.md`（运行模式与职责边界）
- 覆盖的任务清单条目：`CTX-005`, `CTX-006`, `CTX-007`, `CTX-008`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次实现并暂停，等待人工审批记录后再进入下一批。

### BATCH-004

- 状态：待审批
- 当前任务编号：CTX-009, CTX-010, CTX-011, CTX-012
- 当前批次目标：冻结外部 Provider 适配边界、上链异步链路、搜索推荐边界与安全审计最低基线，确保后续实现不越界且具备可审计性。
- 前置依赖核对结果：四个任务 `depends_on` 均为空；用户确认上一批审批已通过。
- 预计涉及文件：`docs/00-context/provider-boundary.md`、`docs/00-context/async-chain-write.md`、`docs/00-context/search-and-recommend-boundary.md`、`docs/00-context/security-and-audit-floor.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成 Provider 双实现边界冻结、上链异步主路径冻结、搜索推荐“回 PostgreSQL 最终校验”边界冻结、安全与审计最低基线冻结。
- 涉及文件：`docs/00-context/provider-boundary.md`、`docs/00-context/async-chain-write.md`、`docs/00-context/search-and-recommend-boundary.md`、`docs/00-context/security-and-audit-floor.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 交付文件存在性检查；2. 关键字检查四项边界规则（mock/real、outbox->Kafka->fabric-adapter、搜索推荐回主库校验、安全审计底线）；3. 对照 CSV `CTX-009~CTX-012` 逐条核验。
- 验证结果：通过。四个交付文件均已落盘且覆盖任务要求，术语与冻结文档一致。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/事件模型与Topic清单正式版.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`权限设计/接口权限校验清单.md`、`权限设计/后端鉴权中间件规则说明.md`
- 覆盖的任务清单条目：`CTX-009`, `CTX-010`, `CTX-011`, `CTX-012`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停，等待人工审批后进入下一批。

### BATCH-013

- 状态：待审批
- 当前任务编号：BOOT-003, BOOT-004, BOOT-005, BOOT-006, BOOT-007, BOOT-008, BOOT-009, BOOT-010, BOOT-011
- 当前批次目标：一次性完成 BOOT-003~011 的仓库基础能力收口，包括根 Makefile、统一脚本入口、多语言工作区规范、共享配置与 OpenAPI 骨架、DB 迁移目录规则、runbook 模板、fixtures 样例、CI workflow 占位。
- 前置依赖核对结果：上述 9 个任务依赖 `CTX-001/004/008/013/014`，均已完成且审批通过。
- 预计涉及文件：`Makefile`、`scripts/*.sh`、`docs/01-architecture/multi-language-workspace.md`、`packages/shared-config/**`、`packages/openapi/**`、`db/migrations/v1/**`、`docs/04-runbooks/**`、`fixtures/**`、`.github/workflows/*.yml`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成根 `Makefile` 统一入口；完成 `scripts/` 统一脚本入口（bootstrap/up/down/wait/seed/reset）；完成多语言工作区规范文档；完成 `packages/shared-config` 共享配置骨架；完成 `packages/openapi` 分域 OpenAPI 骨架与合并脚本；完成 `db/migrations/v1` 命名规则说明；完成 runbook 预置文件；完成 fixtures 本地样例；完成 `.github/workflows` 基础 CI 占位。
- 涉及文件：`Makefile`、`scripts/bootstrap.sh`、`scripts/up-local.sh`、`scripts/down-local.sh`、`scripts/wait-for-services.sh`、`scripts/seed-demo.sh`、`scripts/reset-local.sh`、`docs/01-architecture/multi-language-workspace.md`、`packages/shared-config/README.md`、`packages/shared-config/defaults.env`、`packages/shared-config/schema.md`、`packages/shared-config/mode-switch.md`、`packages/openapi/README.md`、`packages/openapi/iam.yaml`、`packages/openapi/catalog.yaml`、`packages/openapi/trade.yaml`、`packages/openapi/billing.yaml`、`packages/openapi/audit.yaml`、`packages/openapi/ops.yaml`、`packages/openapi/merge-openapi.sh`、`db/migrations/v1/README.md`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/troubleshooting.md`、`docs/04-runbooks/provider-switch.md`、`docs/04-runbooks/fabric-debug.md`、`docs/04-runbooks/search-reindex.md`、`fixtures/local/README.md`、`fixtures/local/product-sample.json`、`fixtures/local/api-request-sample.json`、`fixtures/local/evidence-package-sample.json`、`.github/workflows/lint.yml`、`.github/workflows/test.yml`、`.github/workflows/build.yml`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 文件存在与脚本可执行性检查；2. `make -n up-local down-local logs migrate-up migrate-down seed-local test lint` 解析校验；3. 对照 CSV `BOOT-003~011` 逐项核验交付路径和内容。
- 验证结果：通过。目标文件齐备；脚本可执行；Makefile 目标可解析；CI/Runbook/OpenAPI/fixtures 骨架已落盘。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/仓库拆分与目录结构建议.md`、`开发准备/配置项与密钥管理清单.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/测试用例矩阵正式版.md`、`全集成文档/数据交易平台-全集成基线-V1.md`
- 覆盖的任务清单条目：`BOOT-003`, `BOOT-004`, `BOOT-005`, `BOOT-006`, `BOOT-007`, `BOOT-008`, `BOOT-009`, `BOOT-010`, `BOOT-011`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：待审批
- 备注：本组 9 个任务已按用户要求一次性推进并收口。

### BATCH-014

- 状态：待审批
- 当前任务编号：BOOT-012, BOOT-013, BOOT-014, BOOT-015, BOOT-016, BOOT-017, BOOT-018, BOOT-019, BOOT-020
- 当前批次目标：一次性完成 BOOT-012~020 的工程治理文档收口：错误码映射、日志字段、模块依赖规则、Issue/PR 模板、ownership 矩阵、顶层目录 README、命名规范、版本策略、发布策略。
- 前置依赖核对结果：上述 9 个任务依赖 `CTX-001/004/008/013/014`，均已完成且审批通过。
- 预计涉及文件：`docs/01-architecture/error-codes.md`、`docs/01-architecture/logging-fields.md`、`docs/01-architecture/module-dependency-rules.md`、`docs/01-architecture/issue-template.md`、`docs/01-architecture/pr-template.md`、`docs/01-architecture/ownership-matrix.md`、`docs/01-architecture/naming-conventions.md`、`docs/01-architecture/versioning-policy.md`、`docs/01-architecture/release-policy.md`、顶层目录 `README.md` 补齐文件、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成错误码字典分组文档、统一日志字段规范、模块依赖单向约束、Issue/PR 固定模板；新增 ownership 矩阵、命名规范、版本关联策略、发布策略；补齐缺失的顶层目录 `README.md`（`.github`、`config`、`scripts`、`tests`、`tools`、`部署脚本`）。
- 涉及文件：`docs/01-architecture/error-codes.md`、`docs/01-architecture/logging-fields.md`、`docs/01-architecture/module-dependency-rules.md`、`docs/01-architecture/issue-template.md`、`docs/01-architecture/pr-template.md`、`docs/01-architecture/ownership-matrix.md`、`docs/01-architecture/naming-conventions.md`、`docs/01-architecture/versioning-policy.md`、`docs/01-architecture/release-policy.md`、`.github/README.md`、`config/README.md`、`scripts/README.md`、`tests/README.md`、`tools/README.md`、`部署脚本/README.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 校验 `BOOT-012~020` 对应交付路径文件存在；2. 校验顶层目录 README 覆盖（跳过 `.git`、`target`）；3. 对照 CSV 条目逐项核验交付与描述一致性。
- 验证结果：通过。`BOOT-012~020` 所有交付文件均存在，顶层目录 README 覆盖完整（`.git`/`target` 按规则跳过），内容与任务描述一致。
- 覆盖的冻结文档条目：`开发准备/统一错误码字典正式版.md`、`开发准备/测试用例矩阵正式版.md`、`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`、`全集成文档/数据交易平台-全集成基线-V1.md`
- 覆盖的任务清单条目：`BOOT-012`, `BOOT-013`, `BOOT-014`, `BOOT-015`, `BOOT-016`, `BOOT-017`, `BOOT-018`, `BOOT-019`, `BOOT-020`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：待审批
- 备注：按 BOOT-001~036 的“每 9 个一组”规则执行；本组完成后暂停，等待人工审批。

### BATCH-005

- 状态：待审批
- 当前任务编号：CTX-022, CTX-023, CTX-024
- 当前批次目标：盘点当前仓库资产、本地部署与运维资产、任务与冻结文档基线差异，形成 `exists / partial / missing` 与“可复用/迁移/重写/去重”清单，为 `CTX-014` 提供前置输入。
- 前置依赖核对结果：三项任务共同依赖 `CTX-001`，已完成且通过审批；用户确认上一批已审批通过。
- 预计涉及文件：`docs/00-context/current-repo-assets.md`、`docs/00-context/current-local-stack-assets.md`、`docs/00-context/current-task-baseline-gap.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成仓库资产盘点、本地部署与运维资产盘点、任务基线差异清单，输出可复用/迁移/重写/去重建议。
- 涉及文件：`docs/00-context/current-repo-assets.md`、`docs/00-context/current-local-stack-assets.md`、`docs/00-context/current-task-baseline-gap.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 目录与文件清点（apps/packages/infra/scripts/docs/部署脚本）；2. 对 compose 与脚本资产核对；3. 关键字检查三份文档是否覆盖任务要求分类维度；4. 对照 CSV `CTX-022~CTX-024` 逐条核验。
- 验证结果：通过。三份交付文档均落盘，且分别覆盖 `exists/partial/missing`、本地栈资产盘点、基线差异四分类输出。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`、`开发任务/README.md`、`开发前设计文档/README.md`
- 覆盖的任务清单条目：`CTX-022`, `CTX-023`, `CTX-024`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停，待人工审批后再进入 `CTX-014`。

### BATCH-006

- 状态：通过
- 当前任务编号：CTX-013, CTX-014, CTX-015
- 当前批次目标：冻结任务 ownership 策略、输出当前仓库与目标基线差距分析、冻结 V1 退出标准，形成后续 BOOT/ENV/CORE 的执行与验收门槛。
- 前置依赖核对结果：`CTX-013`/`CTX-015` 无依赖；`CTX-014` 依赖 `CTX-022; CTX-023; CTX-024`，已完成且用户确认审批通过。
- 预计涉及文件：`docs/00-context/ownership-strategy.md`、`docs/00-context/current-gap-analysis.md`、`docs/00-context/v1-exit-criteria.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成 ownership 策略冻结、仓库现状与目标差距分析、V1 退出标准冻结三份文档落盘。
- 涉及文件：`docs/00-context/ownership-strategy.md`、`docs/00-context/current-gap-analysis.md`、`docs/00-context/v1-exit-criteria.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 关键约束词检查（`depends_on`、批次粒度、审批门禁）；2. 差距分析检查（现状资产、缺口、迁移策略）；3. 退出标准检查（五条链路、八 SKU、证据包与监管检查、质量门槛）；4. 对照 CSV `CTX-013~CTX-015` 逐条核验。
- 验证结果：通过。三份交付文档均已创建且覆盖任务描述范围。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`全集成文档/数据交易平台-全集成基线-V1.md`、`领域模型/全量领域模型与对象关系说明.md`、`开发准备/测试用例矩阵正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`CTX-013`, `CTX-014`, `CTX-015`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停，等待人工审批通过后再进入下一批。

### BATCH-007

- 状态：通过
- 当前任务编号：CTX-016, CTX-017, CTX-018
- 当前批次目标：补齐术语统一、专题文档到模块映射、V1 非目标清单，降低后续实现阶段命名漂移和范围外开发风险。
- 前置依赖核对结果：三项任务在 CSV 中 `depends_on` 为空；用户确认上一批审批通过，可继续执行。
- 预计涉及文件：`docs/00-context/term-glossary.md`、`docs/00-context/doc-to-module-map.md`、`docs/00-context/non-goals.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成术语表、专题文档到模块映射、V1 非目标清单三份文档落盘。
- 涉及文件：`docs/00-context/term-glossary.md`、`docs/00-context/doc-to-module-map.md`、`docs/00-context/non-goals.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 文件存在性检查；2. 核验术语覆盖（Tenant/Party/Application/Connector/ExecutionEnvironment/DataResource/DataProduct/SKU/Authorization/QuerySurface/QueryTemplate）；3. 核验映射表覆盖模块域/迁移域/OpenAPI/测试域；4. 核验非目标清单与 V1 边界一致。
- 验证结果：通过。三份交付文档均存在且覆盖任务目标，术语与范围边界一致。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成映射索引.md`、`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`开发准备/测试用例矩阵正式版.md`、`数据库设计/数据库设计总说明.md`
- 覆盖的任务清单条目：`CTX-016`, `CTX-017`, `CTX-018`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停，等待人工审批后进入下一批。

### BATCH-008

- 状态：待审批
- 当前任务编号：CTX-021
- 当前批次目标：输出 V1-Core 闭环矩阵（8 SKU × 5 标准链路 × 合同/授权/交付/验收/计费/退款/争议/审计），明确每个交叉点的主触发点、状态推进点、证据对象、测试入口。
- 前置依赖核对结果：`CTX-021` 依赖 `CTX-006; CTX-007; CTX-015`，均已完成且获审批通过；`CTX-019/CTX-020` 依赖 `BOOT-002/ENV-001` 未完成，继续阻塞。
- 预计涉及文件：`docs/00-context/v1-closed-loop-matrix.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成 V1 闭环矩阵文档，覆盖 5 条标准链路下 8 SKU 全交叉点，并为每个交叉点给出主触发点、状态推进点、证据对象、测试入口。
- 涉及文件：`docs/00-context/v1-closed-loop-matrix.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 核验矩阵文档包含 8 SKU × 5 场景维度；2. 核验每个交叉点字段完整；3. 对照 CSV `CTX-021` 描述核验交付覆盖。
- 验证结果：通过。矩阵内容完整，字段齐备，可作为后续 BOOT/CORE/TEST 任务引用入口。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`（首批 5 条标准链路与 SKU 映射）、`业务流程/业务流程图-V1-完整版.md`、`开发准备/测试用例矩阵正式版.md`
- 覆盖的任务清单条目：`CTX-021`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停；下一可执行任务将优先等待 `CTX-019/CTX-020` 依赖解除或按 CSV 选择其他已满足依赖任务。

### BATCH-009

- 状态：待审批
- 当前任务编号：BOOT-021, BOOT-022, BOOT-023, BOOT-024
- 当前批次目标：完成仓库基础初始化收口：根 README、仓库元文件（ignore/editorconfig/gitattributes）、环境样例文件、增量初始化说明文档。
- 前置依赖核对结果：四项任务共同依赖 `CTX-014`，已完成并审批通过。
- 预计涉及文件：`README.md`、`.gitignore`、`.editorconfig`、`.gitattributes`、`.env.example`、`.env.local.example`、`.env.staging.example`、`.env.demo.example`、`docs/01-architecture/repo-init-notes.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成根 README 补齐；完成 `.gitignore/.editorconfig/.gitattributes` 校准；完成四套环境样例文件补齐；完成增量初始化说明文档落盘。
- 涉及文件：`README.md`、`.gitignore`、`.editorconfig`、`.gitattributes`、`.env.example`、`.env.local.example`、`.env.staging.example`、`.env.demo.example`、`docs/01-architecture/repo-init-notes.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 校验 CSV 交付路径文件存在；2. 校验 `.gitignore` 放行四个 `.env.*.example`；3. 校验样例文件不含真实密钥且变量命名与本地部署口径一致；4. 校验 `repo-init-notes.md` 包含复用策略与禁止覆盖项。
- 验证结果：通过。目标文件均已落盘且内容与任务要求一致。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/技术选型正式版.md`、`开发准备/配置项与密钥管理清单.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`BOOT-021`, `BOOT-022`, `BOOT-023`, `BOOT-024`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停，等待人工审批后再进入下一批。

### BATCH-010

- 状态：待审批
- 当前任务编号：BOOT-029, BOOT-030, BOOT-031, BOOT-032
- 当前批次目标：校准 `apps/`、`services/`、`workers/`、`packages/` 的目录边界与命名落位，形成可引用的目录约束基础。
- 前置依赖核对结果：四项任务共同依赖 `CTX-014`，已完成且审批通过。
- 预计涉及文件：`apps/`、`services/`、`workers/`、`packages/`、`docs/01-architecture/service-worker-package-layout.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成 `apps/` 校准说明；完成 `services/`、`workers/` 目录落位与占位文件；完成 `packages/` 边界校准与新增共享包目录落位；完成布局说明文档。
- 涉及文件：`apps/README.md`、`services/README.md`、`services/*/.gitkeep`、`workers/README.md`、`workers/*/.gitkeep`、`packages/README.md`、`packages/openapi/.gitkeep`、`packages/sdk-ts/.gitkeep`、`packages/ui/.gitkeep`、`packages/shared-config/.gitkeep`、`packages/observability-contracts/.gitkeep`、`docs/01-architecture/service-worker-package-layout.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 检查 `services/`、`workers/`、新增 `packages/*` 目录及占位文件存在；2. 检查 `apps/README.md` 与布局文档是否覆盖 BOOT-029~032 指定名单；3. 对照 CSV 描述逐条核验交付边界。
- 验证结果：通过。目录与说明文件均已落盘，命名与边界要求可被后续任务引用。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/技术选型正式版.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`开发准备/配置项与密钥管理清单.md`
- 覆盖的任务清单条目：`BOOT-029`, `BOOT-030`, `BOOT-031`, `BOOT-032`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停，等待人工审批后再进入下一批。

### BATCH-011

- 状态：待审批
- 当前任务编号：BOOT-033, BOOT-034, BOOT-035, BOOT-036
- 当前批次目标：收敛 `db/`、`infra/`、`docs/`、`fixtures/` 与 `.github/workflows` 目录边界，为后续迁移、runbook、CI 与测试资产提供统一落位。
- 前置依赖核对结果：四项任务共同依赖 `CTX-014`，已完成且审批通过。
- 预计涉及文件：`db/`、`infra/`、`docs/`、`fixtures/`、`.github/workflows/`、`docs/01-architecture/repo-layout.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成 `db/` 目录收敛与映射说明；完成 `infra/` 子目录边界校准；完成 `docs/00-context~05-test-cases` 分层落位与说明；完成 `fixtures/` 与 `.github/workflows/` 目录预留与说明。
- 涉及文件：`db/README.md`、`db/migrations/.gitkeep`、`db/seeds/.gitkeep`、`db/scripts/.gitkeep`、`infra/README.md`、`infra/*/.gitkeep`、`docs/README.md`、`docs/02-openapi/README.md`、`docs/03-db/README.md`、`docs/04-runbooks/README.md`、`docs/05-test-cases/README.md`、`fixtures/README.md`、`fixtures/local/.gitkeep`、`.github/workflows/README.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 检查 `db/infra/docs/fixtures/.github/workflows` 目标目录存在；2. 核验说明文档覆盖 BOOT-033~036 要求；3. 对照 CSV 逐条核验目录与命名边界。
- 验证结果：通过。目标目录和边界说明均已落盘，可被后续 BOOT/ENV/TEST 任务直接引用。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/测试用例矩阵正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`BOOT-033`, `BOOT-034`, `BOOT-035`, `BOOT-036`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停，等待人工审批后再进入下一批。

### BATCH-012

- 状态：待审批
- 当前任务编号：BOOT-001, BOOT-002
- 当前批次目标：完成 BOOT 父任务收口，校验根目录基础文件与目录骨架完整性，并产出 `docs/01-architecture/repo-layout.md` 作为后续唯一目录参考。
- 前置依赖核对结果：`BOOT-001` 依赖 `BOOT-021~024` 已完成；`BOOT-002` 依赖 `BOOT-029~036` 已完成；用户确认前批已审批通过。
- 预计涉及文件：`README.md`、`.gitignore`、`.editorconfig`、`.gitattributes`、`.env.example`、`.env.local.example`、`.env.staging.example`、`.env.demo.example`、`docs/01-architecture/repo-layout.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成 BOOT-001 收口核验（基础文件齐备且已校准）；完成 BOOT-002 目录树固化文档 `repo-layout.md`，明确当前可执行目录边界。
- 涉及文件：`README.md`、`.gitignore`、`.editorconfig`、`.gitattributes`、`.env.example`、`.env.local.example`、`.env.staging.example`、`.env.demo.example`、`docs/01-architecture/repo-layout.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 校验 BOOT-001 交付文件存在且内容有效；2. 校验 `repo-layout.md` 与当前目录结构一致；3. 对照 CSV `BOOT-001/002` 描述核验收口完成度。
- 验证结果：通过。基础文件齐备，目录结构已固化并可被后续任务引用。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`BOOT-001`, `BOOT-002`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已完成本批次并暂停，等待人工审批后进入下一批。

### BATCH-015

- 状态：待审批
- 当前任务编号：ENV-002, ENV-003, ENV-004, ENV-005
- 当前批次目标：完成本地 compose 正式落位与 PostgreSQL 初始化目录收口，建立统一 network/volume/healthcheck 策略、override 示例、环境变量样例以及 initdb 脚本目录。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/docker-compose.local.yml`、`infra/docker/docker-compose.local.override.example.yml`、`infra/docker/.env.local`、`infra/postgres/initdb/`、`scripts/check-local-env.sh`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `infra/docker/docker-compose.local.yml` 并统一 network/volume/restart/logging/healthcheck 策略；新增 `infra/docker/docker-compose.local.override.example.yml`；新增 `infra/docker/.env.local`（端口、账号、bucket、topic、索引别名、realm、链网络变量）；新增 `infra/postgres/initdb` 目录与扩展/schema/角色初始化 SQL；新增根 `docker-compose.local.yml` 兼容入口；更新 `Makefile` 与 `scripts/bootstrap.sh`、`scripts/up-local.sh`、`scripts/down-local.sh`、`scripts/check-local-env.sh` 默认切换到 `infra/docker` 路径。
- 涉及文件：`infra/docker/docker-compose.local.yml`、`docker-compose.local.yml`、`infra/docker/docker-compose.local.override.example.yml`、`infra/docker/.env.local`、`infra/postgres/initdb/README.md`、`infra/postgres/initdb/001_extensions_and_schemas.sql`、`infra/postgres/initdb/002_roles_and_grants.sql`、`Makefile`、`scripts/check-local-env.sh`、`scripts/bootstrap.sh`、`scripts/up-local.sh`、`scripts/down-local.sh`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 交付路径文件存在性检查；2. `docker compose -f infra/docker/docker-compose.local.yml --env-file infra/docker/.env.local config`；3. `docker compose -f docker-compose.local.yml --env-file infra/docker/.env.local config`；4. `bash scripts/check-local-env.sh infra/docker/docker-compose.local.yml infra/docker/.env.local infra/docker/.env.local static`。
- 验证结果：通过。上述校验均成功，compose 语法可解析，路径兼容入口可用。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/配置项与密钥管理清单.md`、`开发准备/仓库拆分与目录结构建议.md`、`数据库设计/数据库设计总说明.md`
- 覆盖的任务清单条目：`ENV-002`, `ENV-003`, `ENV-004`, `ENV-005`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已按“先写记录后实现”执行；本批完成后暂停等待审批。

### BATCH-016

- 状态：待审批
- 当前任务编号：ENV-006, ENV-007, ENV-008, ENV-009
- 当前批次目标：完成 PostgreSQL 扩展启用与配置收口、DB 就绪检查脚本、Kafka KRaft 片段与 topic 初始化脚本，确保本地中间件链路可自检。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/postgres/postgresql.conf`、`infra/postgres/pg_hba.conf`、`db/scripts/check-db-ready.sh`、`infra/kafka/docker-compose*.yml`、`infra/kafka/*.sh`、`scripts/**`、`docs/04-runbooks/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：补齐 PostgreSQL 本地调优配置与访问控制文件（`postgresql.conf`、`pg_hba.conf`），并接入 `infra/docker/docker-compose.local.yml`；新增 `db/scripts/check-db-ready.sh`（连接等待 + schema/extension 检查）；新增 `infra/kafka/docker-compose.kafka.local.yml`（KRaft 单节点片段）与 `infra/kafka/init-topics.sh`（初始化 6 个基础 topic）；新增 `scripts/check-local-stack.sh` 作为统一自检入口兼容脚本；更新 runbook（`local-startup.md`、`postgres-local.md`、`kafka-local.md`）。
- 涉及文件：`infra/postgres/postgresql.conf`、`infra/postgres/pg_hba.conf`、`infra/docker/docker-compose.local.yml`、`db/scripts/check-db-ready.sh`、`infra/kafka/docker-compose.kafka.local.yml`、`infra/kafka/init-topics.sh`、`scripts/check-local-stack.sh`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/postgres-local.md`、`docs/04-runbooks/kafka-local.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 交付路径文件存在性检查；2. `docker compose -f infra/docker/docker-compose.local.yml --env-file infra/docker/.env.local config`；3. `docker compose -f infra/kafka/docker-compose.kafka.local.yml --env-file infra/docker/.env.local config`；4. `bash -n db/scripts/check-db-ready.sh scripts/check-local-stack.sh infra/kafka/init-topics.sh scripts/check-local-env.sh scripts/up-local.sh scripts/down-local.sh scripts/bootstrap.sh`。
- 验证结果：通过。文件齐备，compose 片段可解析，脚本语法检查通过。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/事件模型与Topic清单正式版.md`、`数据库设计/数据库设计总说明.md`
- 覆盖的任务清单条目：`ENV-006`, `ENV-007`, `ENV-008`, `ENV-009`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批次已完成并暂停，等待人工审批后进入下一批。

### BATCH-017

- 状态：待审批
- 当前任务编号：ENV-010, ENV-011, ENV-012, ENV-013
- 当前批次目标：完善 Kafka topic 初始化能力与本地默认策略文档，补齐 Redis 基础配置并冻结 key 命名模式，确保本地缓存/消息中间件口径可复用。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/kafka/init-topics.sh`、`docs/04-runbooks/kafka-topics.md`、`infra/redis/redis.conf`、`docs/04-runbooks/redis-keys.md`、`infra/docker/.env.local`、`infra/docker/docker-compose.local.yml`、`docs/04-runbooks/local-startup.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完善 `infra/kafka/init-topics.sh`（创建 6 个标准 topic 并下发 `retention.ms`、`cleanup.policy`）；新增 `docs/04-runbooks/kafka-topics.md`（consumer group、DLQ、retention、cleanup policy 本地默认值）；新增 `infra/redis/redis.conf` 并接入 compose（DB 划分、持久化、密码、淘汰策略）；新增 `docs/04-runbooks/redis-keys.md`（幂等键/会话缓存/权限缓存/推荐缓存/限流计数/下载票据缓存 key 模式）；更新 `docs/04-runbooks/local-startup.md` 增加 Kafka topic 初始化步骤；补充 `fixtures/local/kafka-topics-manifest.json` 供本地初始化与测试引用。
- 涉及文件：`infra/kafka/init-topics.sh`、`docs/04-runbooks/kafka-topics.md`、`infra/redis/redis.conf`、`docs/04-runbooks/redis-keys.md`、`infra/docker/.env.local`、`infra/docker/docker-compose.local.yml`、`docs/04-runbooks/local-startup.md`、`fixtures/local/kafka-topics-manifest.json`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 交付路径文件存在性检查；2. `docker compose -f infra/docker/docker-compose.local.yml --env-file infra/docker/.env.local config`；3. `bash -n infra/kafka/init-topics.sh scripts/check-local-stack.sh scripts/up-local.sh scripts/down-local.sh`；4. 文档关键字段 grep 检查（consumer group、DLQ、retention、cleanup policy、redis key 模式）。
- 验证结果：通过。目标文件存在，compose 可解析，脚本语法检查通过，文档关键字段覆盖满足任务要求。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/事件模型与Topic清单正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`原始PRD/日志、可观测性与告警设计.md`
- 覆盖的任务清单条目：`ENV-010`, `ENV-011`, `ENV-012`, `ENV-013`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批次已完成并暂停，等待人工审批后进入下一批。

### BATCH-018

- 状态：待审批
- 当前任务编号：ENV-014, ENV-015, ENV-016, ENV-017
- 当前批次目标：完成 MinIO bucket 与初始化脚本收口，完成 OpenSearch 模板/索引/别名初始化脚本，确保对象存储与搜索索引可一键初始化并可验证。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/minio/*`、`infra/opensearch/*`、`scripts/*`、`docs/04-runbooks/*`、`fixtures/local/*`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `infra/minio/init-minio.sh`，可自动创建 6 个 bucket（`raw-data`、`preview-artifacts`、`delivery-objects`、`report-results`、`evidence-packages`、`model-artifacts`），设置预览 bucket 匿名下载策略、30 天生命周期规则，并写入测试对象；新增 `infra/opensearch/index-template-catalog.json` 与 `infra/opensearch/init-opensearch.sh`，完成索引模板、3 个别名索引（`catalog_products_v1`、`seller_profiles_v1`、`search_sync_jobs_v1`）及 demo 文档初始化；新增 runbook `minio-local.md`、`opensearch-local.md`，更新 `local-startup.md` 初始化顺序；补充 `fixtures/local/minio-buckets-manifest.json` 与 `fixtures/local/opensearch-indices-manifest.json`。
- 涉及文件：`infra/minio/init-minio.sh`、`infra/opensearch/index-template-catalog.json`、`infra/opensearch/init-opensearch.sh`、`docs/04-runbooks/minio-local.md`、`docs/04-runbooks/opensearch-local.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/minio-buckets-manifest.json`、`fixtures/local/opensearch-indices-manifest.json`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n infra/minio/init-minio.sh infra/opensearch/init-opensearch.sh`；2. `jq -e . infra/opensearch/index-template-catalog.json` 与 manifest JSON 语法检查；3. 实跑 `./infra/minio/init-minio.sh` 并验证 bucket/对象/策略/生命周期；4. 实跑 `./infra/opensearch/init-opensearch.sh` 并验证 alias 与文档计数；5. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。MinIO 初始化脚本执行成功并验证 bucket/策略/生命周期/测试对象；OpenSearch 初始化脚本执行成功并验证 3 个 alias 与各自 `_count=1`；全栈 core healthcheck 通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/事件模型与Topic清单正式版.md`、`原始PRD/日志、可观测性与告警设计.md`
- 覆盖的任务清单条目：`ENV-014`, `ENV-015`, `ENV-016`, `ENV-017`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已按用户新增要求完成“静态+实跑”执行验证；本批次完成后暂停等待审批。

### BATCH-019

- 状态：待审批
- 当前任务编号：ENV-018, ENV-019, ENV-020, ENV-021
- 当前批次目标：完成 Keycloak realm import 机制与平台本地 realm 占位文件，完善 Mock Payment Provider 容器与可触发的成功/失败/超时/退款/人工打款场景，并补齐 runbook。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/keycloak/**`、`infra/docker/docker-compose.local.yml`、`infra/mock-payment/**`、`docs/04-runbooks/mock-payment.md`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：完成 Keycloak realm import 配置（`start-dev --import-realm` + import 目录挂载），新增 `infra/keycloak/realm-export/platform-local-realm.json`（realm、角色、测试用户、客户端、MFA 占位）；完成 Mock Payment Provider 容器启动参数与 mappings 挂载，新增支付成功/失败/超时、退款成功、人工打款成功、就绪检查映射；新增 `docs/04-runbooks/mock-payment.md`、`docs/04-runbooks/keycloak-local.md`；新增验证脚本 `scripts/check-keycloak-realm.sh`、`scripts/check-mock-payment.sh`；新增 fixtures 清单 `keycloak-realm-manifest.json`、`mock-payment-scenarios.json`。
- 涉及文件：`infra/docker/docker-compose.local.yml`、`infra/keycloak/realm-export/platform-local-realm.json`、`infra/mock-payment/mappings/health-ready.json`、`infra/mock-payment/mappings/payment-success.json`、`infra/mock-payment/mappings/payment-fail.json`、`infra/mock-payment/mappings/payment-timeout.json`、`infra/mock-payment/mappings/refund-success.json`、`infra/mock-payment/mappings/manual-transfer-success.json`、`infra/mock-payment/__files/.gitkeep`、`docs/04-runbooks/mock-payment.md`、`docs/04-runbooks/keycloak-local.md`、`docs/04-runbooks/local-startup.md`、`scripts/check-keycloak-realm.sh`、`scripts/check-mock-payment.sh`、`fixtures/local/keycloak-realm-manifest.json`、`fixtures/local/mock-payment-scenarios.json`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n scripts/check-keycloak-realm.sh scripts/check-mock-payment.sh`；2. `jq -e .` 校验 realm/mappings/fixtures JSON；3. `docker compose ... config`；4. `docker compose ... up -d keycloak mock-payment-provider`；5. 实跑 `./scripts/check-keycloak-realm.sh`；6. 实跑 `./scripts/check-mock-payment.sh`（含 timeout 场景）；7. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。Keycloak realm 导入验证通过；Mock Payment 五类场景验证通过（成功/失败/超时/退款/人工打款）；core 健康检查通过。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`领域模型/全量领域模型与对象关系说明.md`、`原始PRD/链上链下技术架构与能力边界稿.md`
- 覆盖的任务清单条目：`ENV-018`, `ENV-019`, `ENV-020`, `ENV-021`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已按要求完成“静态+实跑”验证；等待人工审批后进入下一批。

### BATCH-020

- 状态：待审批
- 当前任务编号：ENV-022, ENV-023, ENV-024, ENV-025
- 当前批次目标：接入 Fabric 本地测试网络与脚本包装，补充最小链码部署占位流程，并完成 OpenTelemetry Collector 统一采集转发配置与验证脚本。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/fabric/**`、`Makefile`、`infra/docker/docker-compose.local.yml`、`infra/otel/otel-collector-config.yaml`、`scripts/check-fabric-local.sh`、`scripts/check-otel-collector.sh`、`scripts/verify-local-stack.sh`、`docs/04-runbooks/**`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `infra/fabric/docker-compose.fabric.local.yml` 及 `fabric-up/down/reset/channel` 脚本，提供 `make fabric-up/down/reset/channel` 包装入口；新增 `infra/fabric/deploy-chaincode-placeholder.sh`，生成订单摘要、授权摘要、验收摘要、证据批次根四类链码占位接口工件；新增 `scripts/check-fabric-local.sh` 做 Fabric 容器与工件自检；新增 `infra/otel/otel-collector-config.yaml` 与 compose `otel-collector` 服务，统一接收 OTLP 并转发到 Prometheus/Loki/Tempo；新增 `scripts/check-otel-collector.sh`（含重试）；更新 `scripts/verify-local-stack.sh` 增加 OTel health/metrics 核验；新增 runbook `fabric-local.md`、`otel-local.md`，更新 `local-startup.md` 串联 Fabric/OTel 流程；新增 `fixtures/local/fabric-local-manifest.json`、`fixtures/local/otel-collector-manifest.json`。
- 涉及文件：`infra/fabric/docker-compose.fabric.local.yml`、`infra/fabric/fabric-up.sh`、`infra/fabric/fabric-down.sh`、`infra/fabric/fabric-reset.sh`、`infra/fabric/fabric-channel.sh`、`infra/fabric/deploy-chaincode-placeholder.sh`、`infra/fabric/state/.gitkeep`、`infra/fabric/state/.gitignore`、`Makefile`、`infra/otel/otel-collector-config.yaml`、`infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`scripts/check-fabric-local.sh`、`scripts/check-otel-collector.sh`、`scripts/verify-local-stack.sh`、`docs/04-runbooks/fabric-local.md`、`docs/04-runbooks/otel-local.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/fabric-local-manifest.json`、`fixtures/local/otel-collector-manifest.json`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n` 校验所有新增/更新 shell 脚本；2. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config` 与 `docker compose -f infra/fabric/docker-compose.fabric.local.yml config`；3. `jq -e .` 校验新增 fixtures JSON；4. 实跑 `make fabric-up && make fabric-channel && ./infra/fabric/deploy-chaincode-placeholder.sh && ./scripts/check-fabric-local.sh && make fabric-down`；5. 实跑 `docker compose ... up -d otel-collector` 与 `./scripts/check-otel-collector.sh`；6. 实跑 `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。Fabric 包装命令、通道工件与链码占位工件均成功生成并通过自检；OTel Collector 启动正常，health/metrics 端点可达；`check-local-stack.sh core` 通过并包含 OTel 核验。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`、`原始PRD/链上链下技术架构与能力边界稿.md`
- 覆盖的任务清单条目：`ENV-022`, `ENV-023`, `ENV-024`, `ENV-025`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批次按“静态+实跑”完成验证；Docker 相关实跑在提权模式执行，避免沙箱网络/权限误报。

### BATCH-021

- 状态：待审批
- 当前任务编号：ENV-026, ENV-027, ENV-028, ENV-029
- 当前批次目标：补齐本地观测栈闭环，完成 Prometheus 抓取目标、Alertmanager 最小告警规则、Grafana 数据源与初始 dashboard，以及 Loki/Tempo 存储与保留策略配置。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/monitoring/**`、`infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`infra/grafana/**`、`scripts/**`、`docs/04-runbooks/**`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：扩展 `prometheus.yml` 抓取目标，覆盖 `platform-core`、`mock-payment-provider`、`kafka-exporter`、`postgres-exporter`、`redis-exporter`、`minio-exporter`、`opensearch-exporter`；新增 `alert-rules.yml` 与 `alertmanager.yml`，落地服务不可用、队列积压、DB 连接失败、链适配失败、outbox 重试异常、DLQ 增长最小规则集；在 compose 增加 `alertmanager` 与四类 exporter 服务（Kafka/Postgres/Redis/OpenSearch）；新增 Grafana provisioning（Prometheus/Loki/Tempo 数据源）与 4 组 dashboard（平台总览、数据库、Kafka、应用链路追踪）；为 Loki/Tempo 增加本地持久卷挂载，配置保留周期与清理策略；新增 `scripts/check-observability-stack.sh` 与 runbook `observability-local.md`，并更新 `local-startup.md`。
- 涉及文件：`infra/docker/monitoring/prometheus.yml`、`infra/docker/monitoring/alert-rules.yml`、`infra/docker/monitoring/alertmanager.yml`、`infra/docker/monitoring/loki-config.yml`、`infra/docker/monitoring/tempo.yml`、`infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`infra/grafana/provisioning/datasources/datasources.yml`、`infra/grafana/provisioning/dashboards/dashboards.yml`、`infra/grafana/dashboards/platform-overview.json`、`infra/grafana/dashboards/database-overview.json`、`infra/grafana/dashboards/kafka-overview.json`、`infra/grafana/dashboards/application-tracing.json`、`scripts/check-observability-stack.sh`、`scripts/verify-local-stack.sh`、`docs/04-runbooks/observability-local.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/observability-stack-manifest.json`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n scripts/check-observability-stack.sh scripts/verify-local-stack.sh`；2. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config`；3. `jq -e .` 校验新增 dashboard/fixture JSON；4. `docker compose ... --profile observability up -d`；5. `./scripts/check-observability-stack.sh`；6. `docker compose ... --profile mocks up -d mock-payment-provider`；7. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`。
- 验证结果：通过。Prometheus 抓取目标与告警规则存在并可查询；Grafana 数据源与 4 个 dashboard 已自动导入；Loki/Tempo 存储卷挂载生效并可用；`check-observability-stack.sh` 通过；`check-local-stack.sh full` 通过（含 core+observability+mocks）。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`、`原始PRD/日志、可观测性与告警设计.md`
- 覆盖的任务清单条目：`ENV-026`, `ENV-027`, `ENV-028`, `ENV-029`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：为避免容器冷启动瞬时 `503/302` 误判，`scripts/verify-local-stack.sh` 已补充 HTTP 重试与 mock admin 路径修正；本批次仍遵循“静态+实跑”双重验证。

### BATCH-022

- 状态：待审批
- 当前任务编号：ENV-030
- 当前批次目标：在本地 compose 中实现 `core`、`observability`、`fabric`、`demo` 四类 profile 机制，支持最小核心栈先启动并可按需叠加观测与链环境。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/docker-compose.local.yml`、`scripts/up-local.sh`、`docs/04-runbooks/**`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `docker-compose.local.yml` 为核心服务补齐 `core` profile，为观测服务统一补齐 `observability` profile 并纳入 `demo`，新增 `fabric-ca/fabric-orderer/fabric-peer` 三个 `fabric` profile 服务并纳入 `demo`，将 `mock-payment-provider` 纳入 `demo`；更新 `scripts/up-local.sh` 默认 `COMPOSE_PROFILES=core`，实现 `make up-local` 默认仅拉起最小核心栈；新增 runbook `compose-profiles.md` 与 fixture `compose-profiles-manifest.json`，并更新 `local-startup.md` 说明 profile 组合启动方式。
- 涉及文件：`infra/docker/docker-compose.local.yml`、`scripts/up-local.sh`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/compose-profiles.md`、`fixtures/local/compose-profiles-manifest.json`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n scripts/up-local.sh scripts/down-local.sh`；2. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config` 与 `COMPOSE_PROFILES=demo ... config`；3. `make down-local && make up-local` 验证默认 `core` 启动；4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`；5. `COMPOSE_PROFILES=fabric docker compose ... up -d fabric-ca fabric-orderer fabric-peer` + `docker ps` 校验三容器在运行。
- 验证结果：通过。默认 `make up-local` 仅拉起 core profile 且 `check-local-stack.sh core` 通过；`fabric` profile 三容器可独立拉起；`demo` profile 配置可解析。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`、`原始PRD/链上链下技术架构与能力边界稿.md`
- 覆盖的任务清单条目：`ENV-030`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批次按复杂任务单批闭环执行，已完成静态+实跑校验。

### BATCH-023

- 状态：待审批
- 当前任务编号：ENV-031
- 当前批次目标：为基础服务补充 `curl`/`nc`/`mc`/`kcat`/`psql` 级别启动后自检，收敛到 `scripts/check-local-stack.sh` 统一入口。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`scripts/verify-local-stack.sh`、`scripts/check-local-stack.sh`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/**`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：增强 `scripts/verify-local-stack.sh`，在原有 TCP/HTTP 检测基础上新增命令级探测：Postgres `psql`、Redis `redis-cli`、Kafka `kcat`（容器无 `kcat` 时先用临时 `kcat` 容器探测，再回退 `kafka-topics.sh` 元数据探测）、MinIO `mc`、OpenSearch API 命令探测，并补充 Keycloak 容器内 TCP 命令探测；`check_tcp` 优先使用 `nc`；`check_docker_exec` 增加重试；更新 `local-startup.md` 说明命令级探测内容；新增 `fixtures/local/local-healthcheck-probes-manifest.json` 作为探测项清单。
- 涉及文件：`scripts/verify-local-stack.sh`、`docs/04-runbooks/local-startup.md`、`fixtures/local/local-healthcheck-probes-manifest.json`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n scripts/verify-local-stack.sh scripts/check-local-stack.sh`；2. `jq -e . fixtures/local/local-healthcheck-probes-manifest.json`；3. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config`；4. `make down-local && make up-local` 后执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`；5. `COMPOSE_PROFILES=demo docker compose ... up -d` 后执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`。
- 验证结果：通过。`core` 与 `full` 模式均通过；日志显示 `nc/psql/redis-cli/mc` 探测成功，Kafka 在缺少容器内 `kcat` 时由临时 `kcat` 容器探测通过。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`原始PRD/链上链下技术架构与能力边界稿.md`
- 覆盖的任务清单条目：`ENV-031`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：保持 `scripts/check-local-stack.sh` 作为统一入口，命令级探测能力全部内聚于 `verify-local-stack.sh`。

### BATCH-024

- 状态：待审批
- 当前任务编号：ENV-032
- 当前批次目标：建立 `make up-core`、`make up-observability`、`make up-fabric`、`make up-demo` 组合命令，支持 local/staging/demo 三套模式切换基础。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`Makefile`、`docs/04-runbooks/compose-profiles.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `Makefile` 新增 `up-core`、`up-observability`、`up-fabric`、`up-demo` 四个组合命令，均复用 `scripts/up-local.sh` 并通过 `COMPOSE_PROFILES` 控制模式；更新 `compose-profiles.md` 改为以 `make` 命令为主入口；更新 `local-startup.md` 的 profile 启动步骤；新增 `fixtures/local/make-up-modes-manifest.json` 记录模式与验证矩阵。
- 涉及文件：`Makefile`、`docs/04-runbooks/compose-profiles.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/make-up-modes-manifest.json`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `make -n up-core up-observability up-fabric up-demo`；2. `jq -e . fixtures/local/make-up-modes-manifest.json`；3. `docker compose ... config`；4. `make down-local` 后依次执行 `make up-core`、`make up-observability`、`make up-fabric`、`make up-demo`；5. 执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core` 与 `... full`。
- 验证结果：通过。四个 `make up-*` 命令均可执行并按预期切换 profile；`core` 与 `full` 健康检查均通过。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`ENV-032`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`up-local` 仍保持兼容入口，默认等价于 `up-core`。

### BATCH-025

- 状态：待审批
- 当前任务编号：ENV-033
- 当前批次目标：在 `docs/04-runbooks/local-startup.md` 明确本地启动顺序：基础设施 -> schema/migration -> seed -> 应用 -> 回执模拟。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`docs/04-runbooks/local-startup.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：重写 `local-startup.md` 为五阶段顺序化 runbook：阶段 1 基础设施（环境检查、`make up-local`、core 健康检查）；阶段 2 schema/migration（DB 就绪检查、`make migrate-up`）；阶段 3 seed（Kafka/MinIO/OpenSearch 初始化 + `make seed-local`）；阶段 4 应用（`cargo run -p platform-core` 与 `/healthz`）；阶段 5 回执模拟（Keycloak/Fabric/OTel/Observability 检查 + Mock Payment 回执模拟 + full 健康检查）。
- 涉及文件：`docs/04-runbooks/local-startup.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. runbook 文本审阅（顺序完整性、命名一致性）；2. `make up-local`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。`make up-local` 与 `check-local-stack.sh core` 均通过，runbook 顺序与任务要求一致。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`ENV-033`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：启动顺序已显式固定为“基础设施 -> schema/migration -> seed -> 应用 -> 回执模拟”，且命令均指向仓库现有入口。

### BATCH-026

- 状态：待审批
- 当前任务编号：ENV-034
- 当前批次目标：补充 `docker-compose.staging.example.yml` 占位文件，不部署真实生产资源，并明确后续 Helm/K8s 迁移组件映射关系。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`docker-compose.staging.example.yml`、`docs/04-runbooks/staging-compose-mapping.md`、`开发任务/V1-Core-TODO与预留清单.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `docker-compose.staging.example.yml`，提供平台核心、数据库、缓存、消息、存储、检索、鉴权、链适配、可观测与 mock 支付等服务占位；文件内全部采用示例镜像和 `REPLACE_ME` 占位值，未引入真实生产资源；新增 `docs/04-runbooks/staging-compose-mapping.md`，明确 Compose 到 Helm/K8s 资源映射与迁移边界。
- 涉及文件：`docker-compose.staging.example.yml`、`docs/04-runbooks/staging-compose-mapping.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `docker compose -f docker-compose.staging.example.yml config`；2. `make up-local`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。Compose 配置静态解析成功；本地基础栈可启动；core 模式健康检查全部通过（Postgres/Redis/Kafka/MinIO/OpenSearch/Keycloak/OTel 均通过）。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`、`全集成文档/数据交易平台-全集成基线-V1.md`（部署与环境相关约束）
- 覆盖的任务清单条目：`ENV-034`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`make up-local` 与 `check-local-stack.sh` 在沙箱内无法访问 Docker daemon/本机端口，已在沙箱外执行并得到通过结果。

### BATCH-027

- 状态：待审批
- 当前任务编号：ENV-035
- 当前批次目标：补充 `docs/04-runbooks/secrets-policy.md`，并明确 `.env.local` 可存放与禁止存放的变量边界。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`docs/04-runbooks/secrets-policy.md`、`infra/docker/.env.local`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `docs/04-runbooks/secrets-policy.md`，按“非敏感配置/本地演示凭据/敏感凭据/私钥证书材料”给出落位规则，明确 `.env.local`、`.env.local.secret` 与 CI Secret 的边界；更新 `infra/docker/.env.local` 注释分层，标注可入本地文件项与禁止写入真实生产密钥的约束。
- 涉及文件：`docs/04-runbooks/secrets-policy.md`、`infra/docker/.env.local`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 文档结构与变量边界检查；2. `make up-local`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。secrets policy 已落盘且规则完整；`make up-local` 成功；`check-local-stack.sh core` 全部探测通过。
- 覆盖的冻结文档条目：`开发准备/配置项与密钥管理清单.md`、`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`ENV-035`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`make up-local` 与 `check-local-stack.sh` 在沙箱内无法访问 Docker daemon/本机端口，已在沙箱外执行并得到通过结果。

### BATCH-028

- 状态：待审批
- 当前任务编号：ENV-036
- 当前批次目标：新增 `scripts/prune-local.sh`，用于安全清理本地卷、网络、链状态与演示数据，避免误删其他开发项目容器。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`scripts/prune-local.sh`、`docs/04-runbooks/troubleshooting.md`、`scripts/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `scripts/prune-local.sh`，默认 `--dry-run` 列出将被清理的当前 compose project 资源（卷/网络）与 Fabric 状态目录；仅在 `--force` 时执行 `docker compose down -v --remove-orphans`、Fabric compose 清理与 `infra/fabric/state` 重建；脚本先校验 Docker daemon 可达并限定 project 作用域，避免误删其他项目容器。更新 `docs/04-runbooks/troubleshooting.md` 与 `scripts/README.md` 的使用说明。
- 涉及文件：`scripts/prune-local.sh`、`docs/04-runbooks/troubleshooting.md`、`scripts/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n scripts/prune-local.sh`；2. `./scripts/prune-local.sh --dry-run`；3. `make up-local`；4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。脚本语法检查通过；`--dry-run` 能列出 `datab-local` 作用域资源；`make up-local` 启动成功；`check-local-stack.sh core` 全部探测通过。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`、`全集成文档/数据交易平台-全集成基线-V1.md`
- 覆盖的任务清单条目：`ENV-036`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`prune-local`、`make up-local`、`check-local-stack` 在沙箱内均无法访问 Docker daemon，验证已在沙箱外执行并通过。

### BATCH-029

- 状态：待审批
- 当前任务编号：ENV-037, ENV-038, ENV-039, ENV-040
- 当前批次目标：一次性完成本地环境配置快照导出、端口矩阵文档、基础服务故障排查手册扩展与本地 smoke test 套件落地。
- 前置依赖核对结果：4 个任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`scripts/export-local-config.sh`、`scripts/smoke-local.sh`、`docs/04-runbooks/port-matrix.md`、`docs/04-runbooks/troubleshooting.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/**`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `scripts/export-local-config.sh`，将 compose 解析结果导出为只读快照（default/core/demo 三组）；新增 `docs/04-runbooks/port-matrix.md`，维护端口、URL、默认账号口径及初始 bucket/topic 矩阵；扩展 `docs/04-runbooks/troubleshooting.md`，分别补齐 PostgreSQL/Kafka/Keycloak/MinIO/OpenSearch/Fabric 启动失败诊断步骤；新增 `scripts/smoke-local.sh` 与 `fixtures/local/local-smoke-suite-manifest.json`，覆盖“数据库可迁移、bucket 已创建、realm 已导入、topic 已存在、Grafana 可登录、Mock Payment 可回调”六项 smoke 检查；同步更新 `local-startup.md` 与 `scripts/README.md`。
- 涉及文件：`scripts/export-local-config.sh`、`scripts/smoke-local.sh`、`docs/04-runbooks/port-matrix.md`、`docs/04-runbooks/troubleshooting.md`、`docs/04-runbooks/local-startup.md`、`scripts/README.md`、`fixtures/local/local-smoke-suite-manifest.json`、`fixtures/local/config-snapshots/.gitkeep`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n scripts/export-local-config.sh scripts/smoke-local.sh scripts/prune-local.sh`；2. `./scripts/export-local-config.sh`；3. `make up-local`；4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`；5. `make up-demo`；6. `./infra/kafka/init-topics.sh`；7. `./infra/minio/init-minio.sh`；8. `ENV_FILE=infra/docker/.en37v.local ./scripts/check-local-stack.sh full`；9. `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`。
- 验证结果：通过。新增脚本语法检查通过；配置快照导出成功；`core`/`full` 自检均通过；smoke 套件六项检查全部通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`、`全集成文档/数据交易平台-全集成基线-V1.md`（环境与可观测性约束）
- 覆盖的任务清单条目：`ENV-037`, `ENV-038`, `ENV-039`, `ENV-040`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：Docker 相关验证命令在沙箱内不可达，已在沙箱外执行并通过。

### BATCH-030

- 状态：待审批
- 当前任务编号：ENV-041
- 当前批次目标：在 `fixtures/local/` 下补齐五条标准链路所需最小演示数据（企业主体、卖方、买方、产品、SKU、模板、订单、支付、交付样例）。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`fixtures/local/**`、`docs/04-runbooks/local-startup.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `fixtures/local/standard-scenarios-manifest.json` 与 `fixtures/local/standard-scenarios-sample.json`，覆盖首批五条标准链路（S1~S5）最小演示数据：企业主体、卖方、买方、产品、8 个 V1 SKU、模板、订单、支付、交付样例；更新 `fixtures/local/README.md` 与 `docs/04-runbooks/local-startup.md`，明确 fixture 入口和启动阶段引用。
- 涉及文件：`fixtures/local/standard-scenarios-manifest.json`、`fixtures/local/standard-scenarios-sample.json`、`fixtures/local/README.md`、`docs/04-runbooks/local-startup.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `jq -e . fixtures/local/standard-scenarios-manifest.json`；2. `jq -e . fixtures/local/standard-scenarios-sample.json`；3. `make up-local`；4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`；5. `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`。
- 验证结果：通过。新增 fixtures JSON 语法校验通过；`up-local` 成功；`core` 健康检查通过；smoke 套件通过（至少一条 smoke test 成功，且六项检查均通过）。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/测试用例矩阵正式版.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`、`全集成文档/数据交易平台-全集成基线-V1.md`
- 覆盖的任务清单条目：`ENV-041`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：Docker 相关命令在沙箱内不可达，已在沙箱外执行并通过。

### BATCH-031

- 状态：待审批
- 当前任务编号：ENV-044, ENV-045, ENV-046, ENV-047
- 当前批次目标：一次性完成 `docker-compose.local.yml` 的 PostgreSQL/Redis/Kafka/MinIO 服务块收敛，确保服务定义清晰且可被总 compose 聚合解析。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-003; BOOT-004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/docker-compose.local.yml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `docker-compose.local.yml` 中显式标注四个服务块（PostgreSQL/Redis/Kafka/MinIO）；Redis 服务块补充内存策略参数（`REDIS_MAXMEMORY`、`REDIS_MAXMEMORY_POLICY`）并保留持久化卷与健康检查；Kafka 与 MinIO 服务块增加 topic/bucket 初始化前置约束注释，明确与 `infra/kafka/init-topics.sh`、`infra/minio/init-minio.sh` 的衔接关系。
- 涉及文件：`infra/docker/docker-compose.local.yml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config`；2. `make up-local`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。compose 聚合解析成功；`up-local` 可启动；core 健康检查通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/事件模型与Topic清单正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`ENV-044`, `ENV-045`, `ENV-046`, `ENV-047`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：Docker 相关命令在沙箱内不可达，已在沙箱外执行并通过。

### BATCH-032

- 状态：待审批
- 当前任务编号：ENV-048, ENV-049, ENV-050, ENV-051
- 当前批次目标：一次性完成 OpenSearch/Keycloak/OTel+Prometheus+Alertmanager/Grafana+Loki+Tempo 服务块收敛，补齐块级约束并验证可独立启动。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-003; BOOT-004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/docker-compose.local.yml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：OpenSearch 服务块补充索引别名环境变量、内存参数与单节点模式说明；Keycloak 服务块补充 realm/client 初始化来源约束与管理端口职责说明；OTel/Prometheus/Alertmanager 服务块补充块级注释及 Prometheus、Alertmanager 健康检查；Grafana/Loki/Tempo 服务块补充块级注释及健康检查，确保可独立启动和健康判定一致。
- 涉及文件：`infra/docker/docker-compose.local.yml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config`；2. `make up-demo`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`；4. `./scripts/check-observability-stack.sh`。
- 验证结果：通过。compose 解析成功；demo profile 启动成功；core 健康检查通过；observability 栈 targets/rules/datasources/dashboards 校验通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/事件模型与Topic清单正式版.md`、`原始PRD/日志、可观测性与告警设计.md`
- 覆盖的任务清单条目：`ENV-048`, `ENV-049`, `ENV-050`, `ENV-051`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：Docker 相关命令在沙箱内不可达，已在沙箱外执行并通过。

### BATCH-033

- 状态：待审批
- 当前任务编号：ENV-052, ENV-053, ENV-054, ENV-055
- 当前批次目标：一次性完成 Mock Payment/Fabric/compose 公共策略/.env+override 示例收敛，确保可复用且可验证。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-003; BOOT-004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`infra/docker/docker-compose.local.override.example.yml`、`.env.local.example`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 compose 中收敛 `mock-payment-provider` 服务块并补充回调演练变量（`MOCK_PAYMENT_CALLBACK_PORT`、`MOCK_PAYMENT_CALLBACK_BASE_URL`）；收敛 Fabric 测试网络块并补充外部脚本边界说明，Fabric 端口改为可配置变量；抽取并强化 compose 公共策略（`x-service-defaults` 下统一 restart/network/logging 与本地资源限制 guidance）；补齐 `.env.local`、`.env.local.example`、`docker-compose.local.override.example.yml` 的统一变量示例（端口、初始凭证、bucket/topic/realm、索引别名相关项）。
- 涉及文件：`infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`infra/docker/docker-compose.local.override.example.yml`、`.env.local.example`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config`；2. `make up-demo`；3. `./infra/minio/init-minio.sh`；4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`；5. `./scripts/check-observability-stack.sh`；6. `./scripts/check-mock-payment.sh`；7. `make fabric-channel && ./infra/fabric/deploy-chaincode-placeholder.sh && ./scripts/check-fabric-local.sh`。
- 验证结果：通过。compose 解析成功；demo profile 启动成功；full 健康检查通过；observability/moc-payment/fabric 校验均通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/配置项与密钥管理清单.md`、`开发准备/技术选型正式版.md`、`开发准备/服务清单与服务边界正式版.md`、`原始PRD/日志、可观测性与告警设计.md`
- 覆盖的任务清单条目：`ENV-052`, `ENV-053`, `ENV-054`, `ENV-055`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：Docker 相关命令在沙箱内不可达，已在沙箱外执行并通过；fabric 运行态产物已清理，未纳入提交。

### BATCH-034

- 状态：待审批
- 当前任务编号：ENV-056, ENV-057
- 当前批次目标：收敛本地依赖等待脚本与本地栈检查脚本，并确保 runbook 对最终 compose 与迁移兼容关系说明完整。
- 前置依赖核对结果：2 个任务均依赖 `BOOT-003; BOOT-004`，均已完成且你已确认审批通过。
- 预计涉及文件：`scripts/wait-for-services.sh`、`scripts/check-local-stack.sh`、`docs/04-runbooks/local-startup.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：`wait-for-services.sh` 从单次探测脚本升级为可重试等待器（支持 `TIMEOUT_SECONDS`、`INTERVAL_SECONDS`）；`check-local-stack.sh` 改为“先 wait 再 verify”的串联入口；`local-startup.md` 补齐等待步骤和旧 `部署脚本/` 到 `infra/docker/` 的迁移兼容说明。
- 涉及文件：`scripts/wait-for-services.sh`、`scripts/check-local-stack.sh`、`docs/04-runbooks/local-startup.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n scripts/wait-for-services.sh scripts/check-local-stack.sh`；2. `make up-local`；3. `./scripts/wait-for-services.sh core`；4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。脚本语法检查通过；等待脚本在 core 模式返回 ready；check-local-stack core 全量探测通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/测试用例矩阵正式版.md`、`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`ENV-056`, `ENV-057`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：Docker 相关命令在沙箱内不可达，已在沙箱外执行并通过。

### BATCH-035

- 状态：待审批
- 当前任务编号：ENV-001, ENV-042
- 当前批次目标：收敛 ENV 全量 compose 主入口边界，补齐 compose 责任边界文档与旧目录兼容说明。
- 前置依赖核对结果：`ENV-001` 依赖 `ENV-044~057`，均已完成；`ENV-042` 依赖 `ENV-001; CTX-020`，本批次已满足。
- 预计涉及文件：`docs/04-runbooks/compose-boundary.md`、`部署脚本/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `compose-boundary.md`，明确 `docker-compose.local.yml` 的服务清单、可选 profile、端口矩阵入口与“不纳入 compose 的业务进程”边界；补齐旧 `部署脚本/docker-compose.local.yml` 到 `infra/docker/docker-compose.local.yml` 的兼容迁移说明；确认主 compose 已收敛 `postgres/redis/kafka/minio/opensearch/keycloak/otel/prometheus/alertmanager/grafana/loki/tempo/mock-payment/fabric` 全量服务。
- 涉及文件：`docs/04-runbooks/compose-boundary.md`、`部署脚本/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config`；2. `./scripts/wait-for-services.sh core`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：通过。compose 聚合解析通过；等待脚本通过；core 健康检查通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`原始PRD/链上链下技术架构与能力边界稿.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`ENV-001`, `ENV-042`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`ENV-043` 仍受 `CORE-032` 依赖约束，不在本批闭环范围内。

### BATCH-036

- 状态：计划中
- 当前任务编号：ENV-043
- 当前批次目标：预留 `infra/docker/docker-compose.apps.local.example.yml` 占位文件（仅供后续容器化联调参考）。
- 前置依赖核对结果：`ENV-043` 依赖 `ENV-042; CORE-032`；`ENV-042` 已完成，`CORE-032` 未完成（`docs/01-architecture/service-runtime-map.md` 尚不存在）。
- 预计涉及文件：`infra/docker/docker-compose.apps.local.example.yml`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：未开始。
- 涉及文件：无
- 验证步骤：阻塞，待依赖解除后执行。
- 验证结果：阻塞。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`原始PRD/链上链下技术架构与能力边界稿.md`
- 覆盖的任务清单条目：`ENV-043`
- 未覆盖项：`CORE-032` 前置依赖未满足。
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：触发强制暂停条件“前置任务未完成”，按流程不得继续推进 `ENV-043` 实现。

### BATCH-037

- 状态：待审批
- 当前任务编号：CORE-001, CORE-002, CORE-003, CORE-004
- 当前批次目标：在现有 `apps/platform-core` 骨架上完成 V1-Core 首批基础能力收敛：初始化 Rust workspace 子结构（`bin/platform-core` + `crates/kernel|config|http`）、接入统一启动流程、提供 `/health/ready` 可用入口，并保持可编译可测试。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，上述任务已完成且你已确认审批通过。
- 预计涉及文件：`Cargo.toml`、`apps/platform-core/Cargo.toml`、`apps/platform-core/bin/platform-core/**`、`apps/platform-core/crates/kernel/**`、`apps/platform-core/crates/config/**`、`apps/platform-core/crates/http/**`、`apps/platform-core/src/**`、`docs/01-architecture/**`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `apps/platform-core/bin/platform-core` 启动入口；新增 `crates/kernel`（应用启动器、模块注册器、依赖容器、生命周期钩子、shutdown 流程）；新增 `crates/config`（`local/staging/demo` + `mock/real` provider 配置装载）；新增 `crates/http`（HTTP server 封装、统一 `ApiResponse/ErrorResponse`、分页结构、`/health/live` 与 `/health/ready`）；主应用 `platform-core` 改为统一运行入口并复用上述 crate；补充 `docs/01-architecture/platform-core-workspace.md` 说明目录与职责边界。
- 涉及文件：`Cargo.toml`、`Cargo.lock`、`apps/platform-core/Cargo.toml`、`apps/platform-core/src/main.rs`、`apps/platform-core/src/lib.rs`、`apps/platform-core/bin/platform-core/Cargo.toml`、`apps/platform-core/bin/platform-core/src/main.rs`、`apps/platform-core/crates/kernel/Cargo.toml`、`apps/platform-core/crates/kernel/src/lib.rs`、`apps/platform-core/crates/config/Cargo.toml`、`apps/platform-core/crates/config/src/lib.rs`、`apps/platform-core/crates/http/Cargo.toml`、`apps/platform-core/crates/http/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. 运行 `cargo run -p platform-core` 进行 `/health/ready` 运行态探测。
- 验证结果：`cargo build` 通过；`cargo test` 通过；运行态探测在当前沙箱受限环境下启动报错 `bind listener failed: Operation not permitted`，无法在该环境完成本地端口绑定验证，但路由与处理器已在编译期完成接线。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/仓库拆分与目录结构建议.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`开发准备/技术选型正式版.md`
- 覆盖的任务清单条目：`CORE-001`, `CORE-002`, `CORE-003`, `CORE-004`
- 未覆盖项：运行态端口绑定与真实 HTTP 探活在当前沙箱受限环境未完成（非代码逻辑缺口，属环境限制）。
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：代码实现已收口；如需补跑 `/health/ready` 真实请求，请在可绑定本地端口的环境执行 `cargo run -p platform-core` 后 `curl http://127.0.0.1:8080/health/ready`。

### BATCH-038

- 状态：待审批
- 当前任务编号：CORE-005, CORE-006, CORE-007, CORE-008
- 当前批次目标：继续补齐 `platform-core` 内部共享 crate：`db`、`auth`、`audit-kit`、`outbox-kit`，并落地最小可编译能力边界（事务模板、会话与权限门面、审计写入接口、outbox 事件写入接口）。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，上述任务已完成且你已确认审批通过。
- 预计涉及文件：`Cargo.toml`、`apps/platform-core/Cargo.toml`、`apps/platform-core/crates/db/**`、`apps/platform-core/crates/auth/**`、`apps/platform-core/crates/audit-kit/**`、`apps/platform-core/crates/outbox-kit/**`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `crates/db`（连接配置结构、池抽象、迁移执行接口、只读/写事务模板与边界）；新增 `crates/auth`（JWT 解析接口、会话主体提取、权限检查门面、step-up 占位网关、Bearer 提取）；新增 `crates/audit-kit`（审计上下文、证据清单、审计事件写入接口与导出记录接口）；新增 `crates/outbox-kit`（事件 envelope、幂等键、发布状态、重试策略、统一写入接口）；主应用启动时将 DB/事务、权限、step-up、审计、outbox 的基础实现注入容器；文档补充四个 crate 职责边界说明。
- 涉及文件：`Cargo.toml`、`Cargo.lock`、`apps/platform-core/Cargo.toml`、`apps/platform-core/src/lib.rs`、`apps/platform-core/crates/db/Cargo.toml`、`apps/platform-core/crates/db/src/lib.rs`、`apps/platform-core/crates/auth/Cargo.toml`、`apps/platform-core/crates/auth/src/lib.rs`、`apps/platform-core/crates/audit-kit/Cargo.toml`、`apps/platform-core/crates/audit-kit/src/lib.rs`、`apps/platform-core/crates/outbox-kit/Cargo.toml`、`apps/platform-core/crates/outbox-kit/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `cargo run -p platform-core` 并 `curl http://127.0.0.1:8080/health/ready`。
- 验证结果：通过。`cargo build` 通过；`cargo test` 通过（新增 `db/auth/audit-kit/outbox-kit` 单测均通过）；运行态探测返回 `{"success":true,"data":"ready"}`。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/仓库拆分与目录结构建议.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/事件模型与Topic清单正式版.md`、`开发准备/技术选型正式版.md`
- 覆盖的任务清单条目：`CORE-005`, `CORE-006`, `CORE-007`, `CORE-008`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批仅补齐 `db/auth/audit-kit/outbox-kit`；`provider-kit` 在下一批按任务顺序继续推进。

### BATCH-039

- 状态：待审批
- 当前任务编号：CORE-009
- 当前批次目标：创建 `crates/provider-kit`，收敛 KYC/签章/支付/通知/Fabric 写入 Provider trait，并为每类 Provider 提供 `mock` 与 `real` 实现入口。
- 前置依赖核对结果：`CORE-009` 依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，均已完成且你已确认审批通过；`CORE-010` 依赖 `CORE-033~051` 未满足，本批不越过阻塞项。
- 预计涉及文件：`Cargo.toml`、`apps/platform-core/Cargo.toml`、`apps/platform-core/crates/provider-kit/**`、`apps/platform-core/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `crates/provider-kit`，定义 `KycProvider`、`SigningProvider`、`PaymentProvider`、`NotificationProvider`、`FabricWriterProvider` 五类 trait；每类都提供 `mock` 与 `real` 两套实现入口与工厂函数；主应用启动流程按配置模式将五类 provider 统一注入容器；架构文档补充 `provider-kit` 职责边界与目录说明。
- 涉及文件：`Cargo.toml`、`Cargo.lock`、`apps/platform-core/Cargo.toml`、`apps/platform-core/src/lib.rs`、`apps/platform-core/crates/provider-kit/Cargo.toml`、`apps/platform-core/crates/provider-kit/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `cargo run -p platform-core` + `curl http://127.0.0.1:8080/health/ready`。
- 验证结果：通过。`cargo build` 通过；`cargo test` 通过（新增 `provider-kit` 单测通过）；运行态探测返回 `{"success":true,"data":"ready"}`。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/仓库拆分与目录结构建议.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/技术选型正式版.md`
- 覆盖的任务清单条目：`CORE-009`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：后续 `CORE-010` 仍受 `CORE-033~051` 前置约束，需按依赖顺序推进。

### BATCH-040

- 状态：待审批
- 当前任务编号：CORE-011, CORE-012, CORE-013
- 当前批次目标：在不越过 `CORE-010` 阻塞的前提下，连续完成模块统一子目录模板、统一错误体系（含错误码文档校验）和请求级中间件链路收敛。
- 前置依赖核对结果：`CORE-011~013` 依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，均已完成且你已确认审批通过；`CORE-010` 依赖 `CORE-033~051` 未满足，本批次跳过阻塞项按“首个可执行连续任务”推进。
- 预计涉及文件：`apps/platform-core/src/modules/**`、`apps/platform-core/crates/kernel/**`、`apps/platform-core/crates/http/**`、`docs/01-architecture/error-codes.md`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：为现有全部模块补齐 `api/application/domain/repo/dto/events/tests` 统一子目录模板；在 `crates/kernel` 收敛 `AppError/ErrorCode/ErrorResponse` 并新增 `error-codes.md` 前缀校验；在 `crates/http` 收敛请求级中间件（`request_id` 生成/透传、`trace` 注入、租户上下文解析、访问日志），并回写 `x-request-id/x-trace-id/x-tenant-id` 响应头。
- 涉及文件：`Cargo.toml`、`Cargo.lock`、`apps/platform-core/src/modules/**/.gitkeep`、`apps/platform-core/src/lib.rs`、`apps/platform-core/crates/kernel/Cargo.toml`、`apps/platform-core/crates/kernel/src/lib.rs`、`apps/platform-core/crates/http/Cargo.toml`、`apps/platform-core/crates/http/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. 模块模板完整性检查（逐模块逐子目录）；2. `cargo build`；3. `cargo test`；4. `cargo run -p platform-core` + `curl http://127.0.0.1:8080/health/ready`；5. 带 `x-request-id/x-tenant-id` 请求 `curl -D - http://127.0.0.1:8080/health/live` 校验中间件响应头。
- 验证结果：通过。模块模板检查返回 `module-template-ok`；`cargo build` 通过；`cargo test` 通过（新增 kernel 错误码文档校验测试通过）；`/health/ready` 返回 `{"success":true,"data":"ready"}`；带头请求返回 `x-request-id=req-b040`、`x-trace-id=req-b040`、`x-tenant-id=tenant-b040`，访问日志包含 request_id/trace_id/tenant_id/method/path/status/elapsed。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/统一错误码字典正式版.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`开发准备/技术选型正式版.md`
- 覆盖的任务清单条目：`CORE-011`, `CORE-012`, `CORE-013`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`CORE-010` 仍因依赖阻塞，后续需等 `CORE-033~051` 满足后再处理。

### BATCH-041

- 状态：待审批
- 当前任务编号：CORE-014
- 当前批次目标：实现统一 DB 事务模板，保证业务对象修改、审计事件写入、outbox 事件写入可在同一事务模板内完成（单次 begin/commit 或 begin/rollback）。
- 前置依赖核对结果：`CORE-014` 依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，均已完成且你已确认审批通过。
- 预计涉及文件：`apps/platform-core/crates/db/**`、`apps/platform-core/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `crates/db` 新增 `TransactionBundle` 统一事务编排模型，收敛 `business_mutations`、`audit_events`、`outbox_events` 三类写入；新增 `execute_business_audit_outbox` 与 `execute_with_lifecycle`，以单次 begin 后统一执行并在成功时 commit、失败时 rollback；新增 `BusinessMutationWriter` 与 `TxLifecycleHook` 契约及默认实现，主应用启动注入 `NoopBusinessMutationWriter`；文档补齐统一事务模板说明。
- 涉及文件：`Cargo.lock`、`apps/platform-core/crates/db/Cargo.toml`、`apps/platform-core/crates/db/src/lib.rs`、`apps/platform-core/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test -p db -p platform-core`；3. `cargo run -p platform-core` + `curl http://127.0.0.1:8080/health/ready`。
- 验证结果：通过。`cargo build` 通过；`cargo test -p db -p platform-core` 通过（新增 `tx_bundle_commits`、`tx_bundle_rolls_back_on_failure` 单测通过）；运行态探测返回 `{"success":true,"data":"ready"}`。
- 覆盖的冻结文档条目：`原始PRD/双层权威模型与链上链下一致性设计.md`、`data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md`、`领域模型/全量领域模型与对象关系说明.md`
- 覆盖的任务清单条目：`CORE-014`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：当前为事务模板骨架实现；后续模块在接入真实 DB driver 时复用该模板实现真实事务句柄绑定。

### BATCH-042

- 状态：待审批
- 当前任务编号：CORE-015, CORE-016, CORE-017
- 当前批次目标：一次性完成统一分页筛选组件、统一健康检查补齐（`/health/deps`）和统一运行时模式页增强（版本/Git SHA/迁移版本）。
- 前置依赖核对结果：3 个任务均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，已完成且你已确认审批通过。
- 预计涉及文件：`apps/platform-core/crates/http/**`、`apps/platform-core/crates/config/**`、`apps/platform-core/src/**`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `crates/http` 新增统一分页与筛选模型 `Pagination`、`FilterQuery`、`ListQuery` 并补充边界测试；新增 `/health/deps` 端点，覆盖 `DB/Redis/Kafka/MinIO/Keycloak/Fabric Adapter` 可达性探测；在 `crates/config` 扩展 `RuntimeConfig` 字段 `service_version/git_sha/migration_version` 并从环境变量加载，`/internal/runtime` 返回增强字段；补齐架构文档关于分页组件、健康检查与 runtime 字段说明。
- 涉及文件：`apps/platform-core/crates/http/src/lib.rs`、`apps/platform-core/crates/config/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `cargo run -p platform-core` + `curl /health/ready`；4. `curl /health/deps`；5. `curl /internal/runtime`。
- 验证结果：通过。`cargo build` 通过；`cargo test` 通过（新增 `http` 分页与筛选单测通过）；运行态探测通过：`/health/ready` 返回成功，`/health/deps` 返回结构化依赖可达性（当前本地 `fabric-adapter` 未启动时 `reachable=false`，其余依赖为 `true`），`/internal/runtime` 正确返回 `mode/provider/service_version/git_sha/migration_version`。
- 覆盖的冻结文档条目：`原始PRD/审计、证据链与回放设计.md`、`原始PRD/日志、可观测性与告警设计.md`、`领域模型/全量领域模型与对象关系说明.md`
- 覆盖的任务清单条目：`CORE-015`, `CORE-016`, `CORE-017`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批按“连续 3 个简单任务”执行并统一汇报。

### BATCH-043

- 状态：待审批
- 当前任务编号：CORE-018, CORE-019
- 当前批次目标：一次性完成统一幂等键中间件与统一审计注解机制，供创建订单、支付回调、模板执行与重试场景复用。
- 前置依赖核对结果：`CORE-018`, `CORE-019` 均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，均已完成且你已确认审批通过。
- 预计涉及文件：`apps/platform-core/crates/http/**`、`apps/platform-core/crates/audit-kit/**`、`apps/platform-core/src/**`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `crates/http` 中间件链新增统一幂等键收敛（优先 `idempotency-key`，兼容 `x-idempotency-key`，缺失时回落 `request_id`），并将 `idempotency_key` 注入 `RequestContext`、访问日志及响应头 `x-idempotency-key`；在 `crates/audit-kit` 新增 `AuditAnnotation`、`AuditRiskLevel`、`AuditResultStatus`，支持 handler 层声明审计动作、风险等级、对象类型、对象 ID、结果状态；在 `crates/http` 提供 `set_audit_annotation/get_audit_annotation` 以挂载与读取请求级审计注解。
- 涉及文件：`apps/platform-core/crates/http/Cargo.toml`、`apps/platform-core/crates/http/src/lib.rs`、`apps/platform-core/crates/audit-kit/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. 运行 `platform-core` 并校验幂等头透传与审计注解输出。
- 验证结果：通过。`cargo build` 通过；`cargo test` 通过（新增 `audit-kit` 注解构建单测与 `http` 幂等键解析单测通过）；运行态校验通过：携带 `idempotency-key: idem-b043` 请求时响应头返回 `x-idempotency-key: idem-b043`，仅携带 `x-request-id: req-b043` 请求时回退返回 `x-idempotency-key: req-b043`，访问日志包含 `idempotency_key` 字段。
- 覆盖的冻结文档条目：`权限设计/后端鉴权中间件规则说明.md`、`原始PRD/审计、证据链与回放设计.md`、`全集成文档/数据交易平台-全集成基线-全阶段.md`
- 覆盖的任务清单条目：`CORE-018`, `CORE-019`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批按“连续 2 个简单任务”执行并统一汇报，`CORE-020`（权限门面）作为复杂任务在下一批单独推进。

### BATCH-044

- 状态：待审批
- 当前任务编号：CORE-020
- 当前批次目标：实现统一权限门面，收敛会话解析与权限评估入口，避免业务 handler 直接调用 Keycloak；业务放行仍由应用层/访问控制层执行。
- 前置依赖核对结果：`CORE-020` 依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，均已完成且你已确认审批通过。
- 预计涉及文件：`apps/platform-core/crates/auth/**`、`apps/platform-core/src/**`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `crates/auth` 新增统一权限门面 `AuthorizationFacade` 与默认实现 `UnifiedAuthorizationFacade`，收敛 `Bearer -> SessionSubject` 解析与权限评估入口；新增 `AuthorizationRequest/AuthorizationDecision` 统一输入输出模型，门面仅返回评估结果，不直接放行业务动作；在 `platform-core` 启动阶段将 `Arc<dyn AuthorizationFacade>` 注入容器，形成业务层可复用的单一权限入口，避免 handler 直接面向外部 IAM SDK。
- 涉及文件：`apps/platform-core/crates/auth/src/lib.rs`、`apps/platform-core/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. 运行 `platform-core` 完成基础探活，校验权限门面接入不破坏启动链路。
- 验证结果：通过。`cargo build` 通过；`cargo test` 通过（新增 `auth` 门面单测：会话解析+权限放行、无权限拒绝）；运行态 `cargo run -p platform-core` 启动成功，`/health/ready` 返回 `{"success":true,"data":"ready"}`。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`、`原始PRD/链上链下技术架构与能力边界稿.md`、`开发准备/技术选型正式版.md`
- 覆盖的任务清单条目：`CORE-020`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批按“复杂单任务”执行。

### BATCH-045

- 状态：待审批
- 当前任务编号：CORE-021, CORE-022
- 当前批次目标：一次性完成统一时间与 ID 策略（UTC + UUID 主键 + 外部可读编号生成）以及启动自检（必要配置、Provider 绑定、关键 topic/bucket/索引别名）。
- 前置依赖核对结果：`CORE-021`, `CORE-022` 均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，均已完成且你已确认审批通过。
- 预计涉及文件：`apps/platform-core/crates/kernel/**`、`apps/platform-core/crates/http/**`、`apps/platform-core/src/**`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `crates/kernel` 收敛统一时间与 ID 策略，新增 `UtcTimestampMs`（UTC 存储基准）、`EntityId`（UUID 主键封装）、`new_external_readable_id(prefix)`（外部可读编号生成）；在 `crates/http` 改为复用 `kernel::new_uuid_string` 生成请求 ID，统一 ID 生成入口；在 `platform-core` 启动流程加入 `startup_self_check`，校验关键 `topic/bucket/index alias` 配置，并在 `CoreModule` 启动后校验 KYC/签章/支付/通知/Fabric Provider 绑定完整性。
- 涉及文件：`apps/platform-core/crates/kernel/Cargo.toml`、`apps/platform-core/crates/kernel/src/lib.rs`、`apps/platform-core/crates/http/Cargo.toml`、`apps/platform-core/crates/http/src/lib.rs`、`apps/platform-core/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `cargo run -p platform-core` + `curl /health/ready`。
- 验证结果：通过。`cargo build` 通过；`cargo test` 通过（新增 kernel 时间与 ID 策略单测通过）；运行态 `cargo run -p platform-core` 启动成功并输出 `startup self-check passed` 日志，`/health/ready` 返回 `{"success":true,"data":"ready"}`。
- 覆盖的冻结文档条目：`领域模型/全量领域模型与对象关系说明.md`、`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/技术选型正式版.md`
- 覆盖的任务清单条目：`CORE-021`, `CORE-022`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批按“连续 2 个简单任务”执行并统一汇报。

### BATCH-046

- 状态：待审批
- 当前任务编号：CORE-023, CORE-024, CORE-025, CORE-026
- 当前批次目标：一次性提升骨架层实现密度，补齐本地调试 trace-links 端点、基础测试夹具、查询编译检查流程以及 OpenAPI 校验骨架。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，均已完成且你已确认审批通过。
- 预计涉及文件：`apps/platform-core/crates/http/**`、`apps/platform-core/crates/db/**`、`apps/platform-core/src/**`、`scripts/**`、`packages/openapi/**`、`Makefile`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `/internal/dev/trace-links` JSON 端点，返回 Grafana/Loki/Tempo/Keycloak/MinIO/OpenSearch 本地链接（支持 `DEV_LINK_HOST` 与端口变量覆盖）；在 `crates/db` 增加 `TestDbFixture`、`run_transaction_rollback_fixture` 与回滚夹具单测，形成基础单测框架与测试数据库入口；新增 `query-compile-check` 特性与脚本 `scripts/check-query-compile.sh`，把查询编译检查前置到本地/CI；新增 `scripts/check-openapi-schema.sh` 并完善 `packages/openapi/ops.yaml` 路径骨架，避免已实现端点与 OpenAPI 骨架漂移；`Makefile` 增加 `query-compile-check`、`openapi-check` 目标。
- 涉及文件：`apps/platform-core/crates/http/src/lib.rs`、`apps/platform-core/crates/db/Cargo.toml`、`apps/platform-core/crates/db/src/lib.rs`、`apps/platform-core/src/lib.rs`、`scripts/check-query-compile.sh`、`scripts/check-openapi-schema.sh`、`packages/openapi/ops.yaml`、`Makefile`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `./scripts/check-query-compile.sh`；4. `./scripts/check-openapi-schema.sh`；5. `cargo run -p platform-core`；6. `curl /internal/dev/trace-links`；7. `curl /health/ready`。
- 验证结果：通过。`cargo build`/`cargo test` 通过（新增 `db/http` 单测通过）；`query-compile-check` 通过；`openapi-check` 通过；运行态 `/internal/dev/trace-links` 与 `/health/ready` 均返回成功。
- 覆盖的冻结文档条目：`领域模型/全量领域模型与对象关系说明.md`、`全集成文档/数据交易平台-全集成基线-V1.md`、`data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md`、`原始PRD/日志、可观测性与告警设计.md`
- 覆盖的任务清单条目：`CORE-023`, `CORE-024`, `CORE-025`, `CORE-026`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批按“连续 4 个简单任务”执行并统一汇报。

### BATCH-047

- 状态：通过
- 当前任务编号：CORE-027, CORE-028, CORE-029
- 当前批次目标：一次性补齐 feature flags 机制、基于 trait 的仓储接口与内存假实现、进程内统一领域事件总线。
- 前置依赖核对结果：3 个任务均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，均已完成且你已确认审批通过。
- 预计涉及文件：`apps/platform-core/crates/config/**`、`apps/platform-core/crates/db/**`、`apps/platform-core/crates/kernel/**`、`apps/platform-core/src/**`、`infra/docker/.env.local`、`.env.local.example`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `crates/config` 新增 `FeatureFlags`（`FF_DEMO_FEATURES`、`FF_CHAIN_ANCHORING`、`FF_REAL_PROVIDER`、`FF_SENSITIVE_EXPERIMENTS`）统一装载并纳入 `RuntimeConfig` 输出；`startup_self_check` 增加 `provider=real` 与 `FF_REAL_PROVIDER` 的一致性约束；在 `crates/db` 新增 `OrderRepository` trait 与 `InMemoryOrderRepository`，供业务规则测试脱离基础设施联调；在 `crates/kernel` 新增 `InProcessEventBus` 与 `DomainEventEnvelope`，并在 `CoreModule` 启动时注册事件总线并发布模块启动事件（进程内总线，不替代 outbox+Kafka）。
- 涉及文件：`apps/platform-core/crates/config/src/lib.rs`、`apps/platform-core/crates/db/Cargo.toml`、`apps/platform-core/crates/db/src/lib.rs`、`apps/platform-core/crates/kernel/src/lib.rs`、`apps/platform-core/src/lib.rs`、`infra/docker/.env.local`、`.env.local.example`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `./scripts/check-query-compile.sh`；4. `./scripts/check-openapi-schema.sh`；5. `APP_PORT=18080 cargo run -p platform-core`；6. `curl http://127.0.0.1:18080/internal/runtime`；7. `curl http://127.0.0.1:18080/health/ready`。
- 验证结果：通过。`cargo build` 与 `cargo test` 通过（新增 config/db/kernel 单测通过）；查询编译检查与 OpenAPI 校验脚本通过；运行态 `/internal/runtime` 返回 `feature_flags` 字段，`/health/ready` 返回成功。
- 覆盖的冻结文档条目：`原始PRD/日志、可观测性与告警设计.md`、`data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`CORE-027`, `CORE-028`, `CORE-029`
- 未覆盖项：无
- 新增 TODO / 预留项：`TODO-CORE-028-001`（`V1-gap`，非阻塞）已补记。内容为：`OrderRepository` 当前仅完成内存假实现用于规则测试前置，运行时持久化仓储接入待后续领域任务补齐。
- 待人工审批结论：通过
- 备注：本批按“连续 3 个简单任务”执行并统一汇报；运行态校验因本机 `8080` 已被占用，改用 `APP_PORT=18080` 验证。根据人工复核意见补充了 `CORE-028` 的显式 `V1-gap` 标注与 TODO 汇总登记，避免“假实现”被误读为已完成持久化接入。

### BATCH-048

- 状态：通过
- 当前任务编号：CORE-030, CORE-031
- 当前批次目标：一次性补齐开发者总览端点 `/internal/dev/overview` 与一键工程校验入口（`cargo xtask` 或等价工具），并保持现有骨架、脚本与目录结构可复用。
- 前置依赖核对结果：`CORE-030`, `CORE-031` 均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，上述前置均已完成并经人工审批通过。
- 预计涉及文件：`apps/platform-core/crates/http/**`、`apps/platform-core/src/**`、`packages/openapi/ops.yaml`、`scripts/check-openapi-schema.sh`、`scripts/validate_database_migrations.sh`、`xtask/**`、`.cargo/config.toml`、`Makefile`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：在 `crates/http` 新增 `/internal/dev/overview`，返回 `run_mode/provider_mode` 与最近 `outbox/dead-letter/chain-receipt` 快照；新增进程内 ring buffer 与 `record_outbox_event/record_dead_letter_event/record_chain_receipt` 记录函数，默认保留最近 10 条；在 `platform-core` 启动时登记 bootstrap outbox 快照（并在 `FF_CHAIN_ANCHORING=true` 时登记链回执占位快照）；补齐 OpenAPI `ops.yaml` 的 `/internal/dev/overview` 路径与校验脚本守护；新增 `xtask` crate、`.cargo` alias 与 `make xtask` 入口，实现 `cargo xtask all` 一键执行 `fmt/lint/openapi-check/migrate-check/seed`。
- 涉及文件：`apps/platform-core/crates/http/src/lib.rs`、`apps/platform-core/src/lib.rs`、`packages/openapi/ops.yaml`、`scripts/check-openapi-schema.sh`、`scripts/validate_database_migrations.sh`、`xtask/Cargo.toml`、`xtask/src/main.rs`、`.cargo/config.toml`、`Cargo.toml`、`Makefile`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo fmt --all`；2. `cargo build`；3. `cargo test`；4. `./scripts/check-openapi-schema.sh`；5. `cargo xtask all`；6. `APP_PORT=18082 cargo run -p platform-core`；7. `curl http://127.0.0.1:18082/internal/dev/overview`；8. `curl http://127.0.0.1:18082/health/ready`。
- 验证结果：通过。`cargo build`/`cargo test` 通过（新增 `http` 单测 `dev_overview_feed_is_capped` 通过）；OpenAPI 校验通过；`cargo xtask all` 全链路执行完成（包含 `validate_database_migrations.sh` 与 `seed-demo.sh`）；运行态 `/internal/dev/overview` 返回 `run_mode/provider_mode` 与最近 outbox 快照，`/health/ready` 返回成功。补充修复：`migrate-check` 已改为使用相对路径 `./docs/数据库设计/*` 并移除旧路径与 `ls` 报错，现可正确执行 `V1/V2/V3` upgrade+downgrade。
- 覆盖的冻结文档条目：`原始PRD/日志、可观测性与告警设计.md`、`data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`CORE-030`, `CORE-031`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批按“连续 2 个简单任务”执行并统一汇报；运行态端口绑定与 Docker migration 校验在本环境需提升权限执行，结果均已通过。

### BATCH-049

- 状态：通过
- 当前任务编号：CORE-033, CORE-034, CORE-035, CORE-036
- 当前批次目标：按连续简单任务一次性补齐并校准身份/供给/交易前半段/授权交付模块骨架，统一模块目录模板并接入 `modules/mod.rs`。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-002; ENV-001`，均已完成且审批通过；`CORE-032` 依赖 `CORE-010`（而 `CORE-010` 依赖 `CORE-033~051`）当前不满足，已按 CSV 依赖顺序后置处理。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/平台总体架构设计草案.md`
- 已实现功能：完成 `CORE-033~036` 模块骨架校准：确认 `iam/party/access`、`catalog/contract_meta/listing`、`order/contract`、`authorization/delivery` 目录模板均已落盘；新增缺失的 `review` 模块目录模板（`api/application/domain/dto/events/repo/tests`）并接入 `modules/mod.rs`，确保供给侧模块骨架满足任务约束。
- 涉及文件：`apps/platform-core/src/modules/mod.rs`、`apps/platform-core/src/modules/review/mod.rs`、`apps/platform-core/src/modules/review/api/.gitkeep`、`apps/platform-core/src/modules/review/application/.gitkeep`、`apps/platform-core/src/modules/review/domain/.gitkeep`、`apps/platform-core/src/modules/review/dto/.gitkeep`、`apps/platform-core/src/modules/review/events/.gitkeep`、`apps/platform-core/src/modules/review/repo/.gitkeep`、`apps/platform-core/src/modules/review/tests/.gitkeep`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `APP_PORT=18083 cargo run -p platform-core`；4. `curl http://127.0.0.1:18083/health/ready`。
- 验证结果：通过。`cargo build` 和 `cargo test` 通过；运行态 `/health/ready` 返回 `{"success":true,"data":"ready"}`。
- 覆盖的冻结文档条目：`开发准备/服务清单与服务边界正式版.md`、`领域模型/全量领域模型与对象关系说明.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`CORE-033`, `CORE-034`, `CORE-035`, `CORE-036`
- 未覆盖项：`CORE-032` 仍未执行（依赖 `CORE-010`，且 `CORE-010` 依赖 `CORE-033~051`），已按顺序后置。
- 新增 TODO / 预留项：`TODO-CTX-019-001`、`TODO-CTX-020-001`（后续在 `BATCH-050` 已关闭）。
- 待人工审批结论：通过
- 备注：本批按“连续 4 个简单任务”执行并统一汇报；未越过 `CORE-032` 依赖顺序。

### BATCH-050

- 状态：通过
- 当前任务编号：CTX-019, CTX-020
- 当前批次目标：补齐历史阻塞缺口文档 `service-to-module-map.md` 与 `local-deployment-boundary.md`，恢复 `CORE-032` 前置文档完整性。
- 前置依赖核对结果：`CTX-019` 依赖 `CTX-004; BOOT-002`，`CTX-020` 依赖 `CTX-008; ENV-001`，上述依赖均已完成并审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发准备/技术选型正式版.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：新增 `docs/00-context/service-to-module-map.md`，冻结设计服务名到 `platform-core` 模块/外围独立进程映射，并标注同步边界、异步边界、所有权；新增 `docs/00-context/local-deployment-boundary.md`，冻结本地部署边界（`docker-compose.local.yml` 只负责中间件与基础依赖，业务应用默认本机进程运行，应用容器联调使用独立 `docker-compose.apps.local.yml`）。
- 涉及文件：`docs/00-context/service-to-module-map.md`、`docs/00-context/local-deployment-boundary.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `test -f docs/00-context/service-to-module-map.md`；2. `test -f docs/00-context/local-deployment-boundary.md`；3. `rg -n \"iam-service|trade-service|notification-service|docker-compose.local.yml|docker-compose.apps.local.yml\" docs/00-context/service-to-module-map.md docs/00-context/local-deployment-boundary.md`；4. `cargo build`；5. `cargo test`。
- 验证结果：通过。两份缺失文档已落盘且关键约束条目可检索；`cargo build` 与 `cargo test` 通过。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/技术选型正式版.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`CTX-019`, `CTX-020`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增；已关闭 `TODO-CTX-019-001`、`TODO-CTX-020-001`。
- 待人工审批结论：通过
- 备注：本批用于修复历史前置缺口；完成后可按顺序继续后续 CORE 批次。

### BATCH-051（实施完成）

- 状态：通过
- 当前任务编号：CORE-037, CORE-038, CORE-039, CORE-040
- 当前批次目标：按连续简单任务一次性校准并冻结 `billing/dispute`、`audit/consistency`、`search/recommendation`、`developer/ops` 模块骨架，确保目录模板与模块命名与任务清单一致。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-002; ENV-001`，依赖已完成且你已确认审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/平台总体架构设计草案.md`
- 已实现功能：确认并固化 `billing/dispute`、`audit/consistency`、`developer/ops` 目录模板齐备（`api/application/domain/dto/events/repo/tests`）；将 `CORE-039` 模块命名从 `recommend` 收敛为 `recommendation`，完成模块导出与目录模板迁移；同步更新服务映射文档与 workspace 架构说明中的模块命名，消除 `recommend`/`recommendation` 术语漂移。
- 涉及文件：`apps/platform-core/src/modules/mod.rs`、`apps/platform-core/src/modules/recommendation/mod.rs`、`apps/platform-core/src/modules/recommendation/api/.gitkeep`、`apps/platform-core/src/modules/recommendation/application/.gitkeep`、`apps/platform-core/src/modules/recommendation/domain/.gitkeep`、`apps/platform-core/src/modules/recommendation/dto/.gitkeep`、`apps/platform-core/src/modules/recommendation/events/.gitkeep`、`apps/platform-core/src/modules/recommendation/repo/.gitkeep`、`apps/platform-core/src/modules/recommendation/tests/.gitkeep`、`docs/00-context/service-to-module-map.md`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `APP_PORT=18084 cargo run -p platform-core`（运行态探活前置）。
- 验证结果：通过（编译与测试）。`cargo build`、`cargo test` 均通过；运行态启动完成到 `startup self-check passed`，但当前沙箱限制导致监听端口失败（`bind listener failed: Operation not permitted`），故本批以编译/测试通过作为交付验证，端口探活待宿主机环境复验。
- 覆盖的冻结文档条目：`开发准备/服务清单与服务边界正式版.md`、`领域模型/全量领域模型与对象关系说明.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`CORE-037`, `CORE-038`, `CORE-039`, `CORE-040`
- 未覆盖项：运行态 `/health/ready` 在当前沙箱因端口绑定限制未完成复验，需在宿主机环境补验。
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批按“连续 4 个简单任务”执行并统一汇报；未变更既有骨架主轴，仅做模块命名与模板一致性校准。

### BATCH-052（实施完成）

- 状态：待审批
- 当前任务编号：CORE-041, CORE-042, CORE-043, CORE-044
- 当前批次目标：按连续简单任务一次性校准共享 crates 边界，确保 `kernel/config/http/db/auth/audit-kit/outbox-kit/provider-kit` 的能力分层与任务定义一致。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-002; ENV-001`，依赖已完成且你已确认审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发准备/仓库拆分与目录结构建议.md`、`docs/开发准备/服务清单与服务边界正式版.md`
- 已实现功能：将分页/筛选共享模型从 `crates/http` 收口到 `crates/kernel`（`PaginationQuery`、`Pagination`、`FilterQuery`、`ListQuery`、`PaginationMeta`），并新增 kernel 侧单测；`crates/http` 改为复用 `kernel` 模型，去除重复定义；同步更新 `platform-core-workspace` 对“分页与筛选组件”的归属描述，明确由 `kernel` 收口、`http` 与业务模块复用。其余 `config/db/auth/audit-kit/outbox-kit/provider-kit` 维持既有已落盘实现并通过编译测试复验。
- 涉及文件：`apps/platform-core/crates/kernel/src/lib.rs`、`apps/platform-core/crates/http/src/lib.rs`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `APP_HOST=127.0.0.1 APP_PORT=18080 cargo run -p platform-core`。
- 验证结果：通过（编译与测试）。`cargo build`、`cargo test` 均通过；运行态启动到 self-check 成功，但沙箱环境禁止端口监听，报 `bind listener failed: Operation not permitted (os error 1)`，需在宿主机复验 `/health/ready`。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`开发准备/事件模型与Topic清单正式版.md`
- 覆盖的任务清单条目：`CORE-041`, `CORE-042`, `CORE-043`, `CORE-044`
- 未覆盖项：运行态 `/health/ready` 在当前沙箱环境未完成端口监听复验（非实现缺陷，受执行环境限制）。
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批按“连续 4 个简单任务”一次性执行并汇总；未引入 V2/V3 正式实现。

### BATCH-053（实施完成）

- 状态：待审批
- 当前任务编号：CORE-045, CORE-046, CORE-047, CORE-048
- 当前批次目标：按连续简单任务一次性完成模块模板统一校准、运行时拓扑文档落盘、请求级中间件链收敛以及健康/运行态端点收敛。
- 前置依赖核对结果：`CORE-045` 依赖 `CORE-033~040`，`CORE-046` 依赖 `CORE-041~044`，`CORE-047` 依赖 `CORE-042; CORE-043`，`CORE-048` 依赖 `CORE-043; CORE-047`；上述前置均已完成且你已确认审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/平台总体架构设计草案.md`
- 已实现功能：
  - `CORE-045`：在 `apps/platform-core/src/modules/mod.rs` 新增模板一致性单测 `all_modules_follow_template_layout`，对所有模块统一校验 `api/application/domain/repo/dto/events/tests` 七类子目录，防止后续骨架漂移。
  - `CORE-046`：新增 `docs/01-architecture/service-runtime-map.md`，完整冻结共享 crate、业务模块与外围进程的运行时所有权、同步边界与异步边界。
  - `CORE-047`：请求级链路继续收敛在 `crates/http`，并通过统一构建入口承载 `request_id/trace/tenant/idempotency/access-log`（保持既有实现不回退）。
  - `CORE-048`：把 `/internal/runtime` 端点收敛进 `crates/http::build_router`，由 `platform-core` 传入运行时配置构建路由，健康与运行态端点归口在 HTTP 基础层。
- 涉及文件：`apps/platform-core/src/modules/mod.rs`、`apps/platform-core/crates/http/Cargo.toml`、`apps/platform-core/crates/http/src/lib.rs`、`apps/platform-core/src/lib.rs`、`docs/01-architecture/service-runtime-map.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `APP_HOST=127.0.0.1 APP_PORT=8080 cargo run -p platform-core`。
- 验证结果：`cargo build` 与 `cargo test` 通过（新增模块模板校验测试通过）；`cargo run` 在当前沙箱环境因端口监听受限失败（`bind listener failed: Operation not permitted`），非代码逻辑错误，需宿主机复验 `/health/ready`。
- 覆盖的冻结文档条目：`开发准备/平台总体架构设计草案.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`开发准备/本地开发环境与中间件部署清单.md`
- 覆盖的任务清单条目：`CORE-045`, `CORE-046`, `CORE-047`, `CORE-048`
- 未覆盖项：仅宿主机运行态端口监听复验（受沙箱限制）。
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：已修复仓库现状差异（`service-runtime-map.md` 缺失）并纳入本批交付。

### BATCH-054（实施完成）

- 状态：待审批
- 当前任务编号：CORE-049, CORE-050, CORE-051
- 当前批次目标：按连续 3 个简单任务完成 Provider trait 收敛校准、内部调试端点收敛拆分、一键验证入口补齐（编译/测试/OpenAPI/迁移检查）。
- 前置依赖核对结果：`CORE-049` 依赖 `CORE-044`，`CORE-050` 依赖 `CORE-038; CORE-047; CORE-048`，`CORE-051` 依赖 `CORE-041; CORE-042; CORE-043; CORE-044`；上述前置均已完成且你已确认审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`
- 已实现功能：
  - `CORE-049`：强化 `crates/provider-kit` 的收敛验证，扩展单测覆盖 KYC、签章、支付、通知、Fabric 五类 provider 的 `mock/real` 入口与调用返回，确保 trait 分层与入口完整。
  - `CORE-050`：在 `crates/http` 中新增 `build_internal_dev_router()`，将 `/internal/dev/trace-links` 与 `/internal/dev/overview` 显式收敛到内部调试子路由后再并入主路由。
  - `CORE-051`：增强 `xtask all`，纳入 `cargo test --workspace` 与 `query-compile-check`；`Makefile` 新增 `core-verify` 统一入口并指向 `cargo xtask all`。
  - 同步更新 `docs/01-architecture/platform-core-workspace.md`，对一键校验入口与校验项进行对齐说明。
- 涉及文件：`apps/platform-core/crates/provider-kit/src/lib.rs`、`apps/platform-core/crates/http/src/lib.rs`、`xtask/src/main.rs`、`Makefile`、`docs/01-architecture/platform-core-workspace.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `cargo xtask all`。
- 验证结果：通过。`cargo build`、`cargo test` 通过；`cargo xtask all` 在格式化修正后复跑通过，含 `fmt/check/test/query-compile/openapi-check/migrate-check/seed` 全链路。`migrate-check` 需 Docker 权限，已在沙箱外执行成功。
- 覆盖的冻结文档条目：`开发准备/服务清单与服务边界正式版.md`、`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`
- 覆盖的任务清单条目：`CORE-049`, `CORE-050`, `CORE-051`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`CORE-010` 与 `CORE-032` 前置依赖已在本批补齐，可在下一批按顺序处理并关闭历史阻塞链。

### BATCH-055

- 状态：计划中
- 当前任务编号：CORE-010
- 当前批次目标：在既有骨架基础上完成 `src/modules` 目录与模板校准，确保 `CORE-010` 指定模块集合模板齐备并清理历史遗留目录漂移。
- 前置依赖核对结果：`CORE-010` 依赖 `CORE-033~051`，相关任务均已完成且你已确认审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/01-architecture/platform-core-workspace.md`、`docs/01-architecture/service-runtime-map.md`
- 预计涉及文件：`apps/platform-core/src/modules/mod.rs`、`apps/platform-core/src/modules/**`、`docs/开发任务/V1-Core-实施进度日志.md`

### BATCH-055（实施完成）

- 状态：待审批
- 当前任务编号：CORE-010
- 当前批次目标：在既有骨架基础上完成 `src/modules` 目录与模板校准，确保 `CORE-010` 指定模块集合模板齐备并清理历史遗留目录漂移。
- 前置依赖核对结果：`CORE-010` 依赖 `CORE-033~051`，相关任务均已完成且你已确认审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/01-architecture/platform-core-workspace.md`、`docs/01-architecture/service-runtime-map.md`
- 已实现功能：将模块模板校验用例改为按 `CORE-010` 指定 19 个模块集合进行精确校验（`iam/party/access/catalog/contract_meta/listing/review/order/contract/authorization/delivery/billing/dispute/audit/consistency/search/recommendation/developer/ops`）；新增 `legacy_recommend_module_directory_is_absent` 回归用例；移除历史遗留的空目录 `src/modules/recommend`，避免 `recommend`/`recommendation` 双轨并存漂移。
- 涉及文件：`apps/platform-core/src/modules/mod.rs`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `cargo xtask all`。
- 验证结果：通过。`cargo build` 与 `cargo test` 通过（新增 `core010_required_modules_follow_template_layout` 与 `legacy_recommend_module_directory_is_absent` 通过）；`cargo xtask all` 通过（含 fmt/check/test/query-compile/openapi-check/migrate-check/seed，migrate-check 在沙箱外 Docker 权限下通过）。
- 覆盖的冻结文档条目：`开发准备/平台总体架构设计草案.md`、`开发准备/服务清单与服务边界正式版.md`、`全集成文档/数据交易平台-全集成基线-V1.md`
- 覆盖的任务清单条目：`CORE-010`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`CORE-010` 完成后，`CORE-032` 依赖已满足，可在下一批执行并联动解除 `TODO-ENV-043-001` 阻塞链。

### BATCH-056（实施完成）

- 状态：待审批
- 当前任务编号：CORE-032
- 当前批次目标：固化 `service-runtime-map` 运行时拓扑边界，明确 `platform-core` 模块内能力、外围独立进程能力与 trait 占位能力，并完成编译/测试/运行态探活验证。
- 前置依赖核对结果：`CORE-032` 依赖 `CTX-019; CTX-020; CORE-010`，对应任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`、`docs/开发准备/服务清单与服务边界正式版.md`
- 已实现功能：对 `docs/01-architecture/service-runtime-map.md` 按 `CORE-032` 做正式校准，新增“CORE-032 三分边界冻结”章节，显式冻结 `platform-core` 模块内能力、外围独立进程能力、trait/接口占位能力三类职责边界；标题同步为 `CORE-032 / CORE-046`，避免任务追溯歧义。
- 涉及文件：`docs/01-architecture/service-runtime-map.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `cargo build`；2. `cargo test`；3. `APP_HOST=127.0.0.1 APP_PORT=18080 cargo run -p platform-core`；4. `curl -fsS http://127.0.0.1:18080/health/ready`。
- 验证结果：通过。`cargo build`、`cargo test` 均通过；服务启动日志显示 `startup self-check passed`；`/health/ready` 返回 `{"success":true,"data":"ready"}`（HTTP 200）。
- 覆盖的冻结文档条目：`开发准备/服务清单与服务边界正式版.md`、`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`CORE-032`
- 未覆盖项：无
- 新增 TODO / 预留项：无（仅更新既有阻塞项 `TODO-ENV-043-001` 的阻塞原因与补齐条件，状态保持 `blocked`，等待本批审批通过后解除）
- 待人工审批结论：通过
- 备注：本批未变更业务代码与目录骨架，仅对运行时拓扑冻结文档与阻塞链追踪文档做一致性校准。

### BATCH-057（实施完成）

- 状态：通过
- 当前任务编号：ENV-043
- 当前批次目标：补齐 `infra/docker/docker-compose.apps.local.example.yml` 占位文件，固定业务应用容器化联调参考编排，并完成 compose 配置校验与本地自检闭环。
- 前置依赖核对结果：`ENV-043` 依赖 `ENV-042; CORE-032`，对应任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/服务清单与服务边界正式版.md`
- 已实现功能：
  - 新增 `infra/docker/docker-compose.apps.local.example.yml`，为 `platform-core`、`fabric-adapter`、`notification-service`、`outbox-publisher`、`search-indexer` 提供容器化联调占位编排（`apps` profile，依赖 core 中间件健康状态）。
  - 更新 runbook `docs/04-runbooks/local-startup.md`，新增“应用层占位编排”可选校验与叠加启动步骤，保持默认 V1 流程仍以本机进程启动为主。
  - 关闭阻塞项 `TODO-ENV-043-001`，解除 `ENV-043` 阻塞链。
- 涉及文件：`infra/docker/docker-compose.apps.local.example.yml`、`docs/04-runbooks/local-startup.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml -f infra/docker/docker-compose.apps.local.example.yml config >/tmp/datab-compose-apps-config.yaml`
  2. `make up-local`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `make up-demo`
  5. `./infra/kafka/init-topics.sh`
  6. `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`
- 验证结果：通过。compose config 校验通过；`check-local-stack.sh core` 全项通过；`smoke-local.sh` 首次因 topic 未初始化失败（`topic missing: outbox.events`），补跑 `init-topics.sh` 后再次执行 smoke 全通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/技术选型正式版.md`
- 覆盖的任务清单条目：`ENV-043`
- 未覆盖项：无
- 新增 TODO / 预留项：无（已关闭 `TODO-ENV-043-001`）
- 待人工审批结论：通过
- 备注：本批仅新增应用层 compose 占位编排与 runbook 步骤，不改变 V1 默认“基础设施容器 + 业务本机进程”主流程。

### BATCH-058（实施完成）

- 状态：通过
- 当前任务编号：CORE-022, CORE-028
- 当前批次目标：补齐两个已识别缺口：`CORE-022` 资源存在性自检从“仅配置值校验”提升到“实际存在性探测”；`CORE-028` 补齐 PostgreSQL 仓储实现并接入运行时 DI 切换。
- 前置依赖核对结果：`CORE-022` 与 `CORE-028` 均依赖 `BOOT-001; BOOT-002; BOOT-005; BOOT-006; ENV-001`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/服务清单与服务边界正式版.md`
- 已实现功能：
  - `CORE-022`：`startup_self_check` 升级为异步探测，新增三类实际存在性校验：Kafka metadata 校验 required topics；MinIO bucket HEAD 校验（200/403 视为存在）；OpenSearch alias 校验（`/_alias/{name}` 返回 200）。
  - `CORE-028`：在 `crates/db` 新增 `PostgresOrderRepository`（`upsert/find_by_id/list_by_tenant`），新增 `OrderRepositoryBackend` 与 `ORDER_REPOSITORY_BACKEND` 解析，新增 `build_order_repository` 工厂；在 `platform-core` 启动链路中完成 `Arc<dyn OrderRepository>` DI 绑定。
  - 新增 `CORE-028` 相关单测：后端选择默认值与 postgres 解析校验。
- 涉及文件：`apps/platform-core/src/lib.rs`、`apps/platform-core/crates/db/src/lib.rs`、`apps/platform-core/crates/db/Cargo.toml`、`apps/platform-core/Cargo.toml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo build`
  2. `cargo test`
  3. `APP_HOST=127.0.0.1 APP_PORT=18080 KAFKA_BROKERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core`
  4. `curl -fsS http://127.0.0.1:18080/health/ready`
- 验证结果：通过。`cargo build`、`cargo test` 全通过；运行态启动日志显示 `startup self-check passed`；`/health/ready` 返回 `{"success":true,"data":"ready"}`（HTTP 200）。
- 覆盖的冻结文档条目：`开发准备/事件模型与Topic清单正式版.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`CORE-022`, `CORE-028`
- 未覆盖项：无
- 新增 TODO / 预留项：无（已关闭 `TODO-CORE-028-001`）
- 待人工审批结论：通过
- 备注：运行态验证阶段若未显式设置 `KAFKA_BROKERS`，默认 `localhost:9092` 可能与本地映射端口不一致，已在验证命令中固定为 `127.0.0.1:9094`。

### BATCH-059（实施完成）

- 状态：通过
- 当前任务编号：DB-001
- 当前批次目标：在 `db/migrations/v1/` 落地可执行迁移基线（命名规则、执行清单、checksum 锁文件），并实现支持数字顺序执行、checksum 记录、dry-run 的 migration runner。
- 前置依赖核对结果：`DB-001` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，以上任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/README.md`、`docs/数据库设计/数据库设计总说明.md`
- 已实现功能：
  - 新增 `db/migrations/v1/manifest.csv`，按版本号维护 `upgrade/downgrade` 对应关系。
  - 新增 `db/migrations/v1/checksums.sha256`，锁定所有迁移 SQL 的 checksum。
  - 新增 `db/scripts/migration-runner.sh`，支持 `up/down/status`、`--dry-run`、按版本顺序执行与 `schema_migration_history` 记录。
  - 新增包装脚本 `migrate-up.sh`、`migrate-down.sh`、`migrate-status.sh`、`migrate-reset.sh`。
  - 修正 `db/scripts/check-db-ready.sh` 扩展检查项，与当前迁移基线保持一致（`pgcrypto/citext/pg_trgm/btree_gist/vector`）。
  - 更新 `db/migrations/v1/README.md`，补充执行规则、清单与校验说明。
- 涉及文件：`db/migrations/v1/README.md`、`db/migrations/v1/manifest.csv`、`db/migrations/v1/checksums.sha256`、`db/scripts/migration-runner.sh`、`db/scripts/migrate-up.sh`、`db/scripts/migrate-down.sh`、`db/scripts/migrate-status.sh`、`db/scripts/migrate-reset.sh`、`db/scripts/check-db-ready.sh`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `docker compose -p luna_db_test -f 部署脚本/docker-compose.postgres-test.yml down -v`
  2. `docker compose -p luna_db_test -f 部署脚本/docker-compose.postgres-test.yml up -d`
  3. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/migrate-up.sh --dry-run`
  4. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/migrate-up.sh`
  5. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/migrate-status.sh`
  6. `./scripts/seed-demo.sh`
  7. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/migrate-reset.sh`
  8. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/migrate-up.sh`
  9. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/migrate-down.sh --dry-run`
  10. `sha256sum -c db/migrations/v1/checksums.sha256`
  11. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/check-db-ready.sh`
- 验证结果：通过。空库升级、状态查询、重建再升级、降级 dry-run、checksum 校验均通过；`check-db-ready.sh` 修正后通过。`seed-demo.sh` 当前为可执行占位实现，已成功执行但未写入业务演示数据。
- 覆盖的冻结文档条目：`数据库设计/README.md`（迁移策略）、`数据库设计/数据库设计总说明.md`（迁移顺序、设计原则）
- 覆盖的任务清单条目：`DB-001`
- 未覆盖项：`seed-demo.sh` 的业务演示数据导入逻辑尚未在本任务范围内实现（后续由 DB seeds 任务补齐）。
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证使用的数据库容器为 `luna-postgres-test`（`部署脚本/docker-compose.postgres-test.yml`）。

### BATCH-060（实施完成）

- 状态：通过
- 当前任务编号：DB-002
- 当前批次目标：完成 `001_extensions_and_schemas.sql` 的落地验收，确保“基础扩展 + 业务 schema + 公共函数 + 更新时间 trigger 基座”满足 V1 迁移基线要求，并形成可重复执行的自动校验脚本。
- 前置依赖核对结果：`DB-002` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/README.md`、`docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/V1/upgrade/001_extensions_and_schemas.sql`
- 已实现功能：
  - 新增 `db/scripts/verify-migration-001.sh`，对 `001` 迁移结果做可执行校验：基础扩展、业务 schema、`common` schema 下 trigger 函数完整性。
  - 修复 `db/scripts/migrate-reset.sh`：重建前先终止目标库活动连接，避免 `DROP DATABASE ... is being accessed by other users` 导致验证链路不稳定。
  - 复核 `docs/数据库设计/V1/upgrade/001_extensions_and_schemas.sql` 与任务要求一致（扩展、schema、公共函数、更新时间 trigger 基座已覆盖）。
- 涉及文件：`db/scripts/verify-migration-001.sh`、`db/scripts/migrate-reset.sh`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `docker ps --filter name=luna-postgres-test`
  2. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/migrate-reset.sh`
  3. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/verify-migration-001.sh`
- 验证结果：通过。`migrate-reset.sh` 可稳定重建并完成全量迁移；`verify-migration-001.sh` 返回 `[ok] migration 001 baseline verified`。
- 覆盖的冻结文档条目：`数据库设计/README.md`（迁移策略）、`数据库设计/数据库设计总说明.md`（设计原则）、`数据库设计/V1/upgrade/001_extensions_and_schemas.sql`
- 覆盖的任务清单条目：`DB-002`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432`）。

### BATCH-061（实施完成）

- 状态：通过
- 当前任务编号：DB-003, DB-004, DB-005, DB-006
- 当前批次目标：按连续 4 个简单任务一次性完成 `010/020/025/030` 迁移落地验收，补齐“表结构 + 关键索引 + 关键触发器”自动校验脚本，并通过空库重建链路验证可复现性。
- 前置依赖核对结果：`DB-003~DB-006` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/V1/upgrade/010_identity_and_access.sql`、`docs/数据库设计/V1/upgrade/020_catalog_contract.sql`、`docs/数据库设计/V1/upgrade/025_review_workflow.sql`、`docs/数据库设计/V1/upgrade/030_trade_delivery.sql`
- 已实现功能：
  - 新增 `db/scripts/verify-migration-010-030.sh`，对 `010/020/025/030` 迁移结果执行自动校验，覆盖关键业务表、关键索引、关键触发器与关键外键约束。
  - 将 DB-003~DB-006 的“落地验收”标准固化为可复用脚本，避免仅依赖人工抽查。
  - 更新 `db/migrations/v1/README.md`，补充 `001` 与 `010~030` 基线验证脚本说明。
- 涉及文件：`db/scripts/verify-migration-010-030.sh`、`db/migrations/v1/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `docker ps --filter name=luna-postgres-test`
  2. `./db/scripts/migrate-reset.sh`
  3. `./db/scripts/verify-migration-001.sh`
  4. `./db/scripts/verify-migration-010-030.sh`
  5. `./db/scripts/migrate-status.sh`
  6. `sha256sum -c db/migrations/v1/checksums.sha256`
- 验证结果：通过。空库重建与全量迁移成功；`verify-migration-001.sh` 返回 `[ok] migration 001 baseline verified`；`verify-migration-010-030.sh` 返回 `[ok] migrations 010/020/025/030 baseline verified`；`migrate-status.sh` 显示 `001~070` 全量 `up` 记录且无 pending；checksum 全量校验通过。
- 覆盖的冻结文档条目：`数据库设计/数据库设计总说明.md`（设计范围、迁移顺序）、`数据库设计/V1/upgrade/010_identity_and_access.sql`、`数据库设计/V1/upgrade/020_catalog_contract.sql`、`数据库设计/V1/upgrade/025_review_workflow.sql`、`数据库设计/V1/upgrade/030_trade_delivery.sql`
- 覆盖的任务清单条目：`DB-003`, `DB-004`, `DB-005`, `DB-006`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`）。

### BATCH-062（实施完成）

- 状态：通过
- 当前任务编号：DB-007, DB-008, DB-009, DB-010
- 当前批次目标：按连续 4 个简单任务一次性完成 `040/050/055/056` 迁移落地验收，补齐“关键表 + 关键索引 + 关键触发器 + 关键约束”自动校验脚本，并通过空库重建链路验证可复现性。
- 前置依赖核对结果：`DB-007~DB-010` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`、`docs/数据库设计/V1/upgrade/050_audit_search_dev_ops.sql`、`docs/数据库设计/V1/upgrade/055_audit_hardening.sql`、`docs/数据库设计/V1/upgrade/056_dual_authority_consistency.sql`
- 已实现功能：
  - 新增 `db/scripts/verify-migration-040-056.sh`，对 `040/050/055/056` 迁移结果执行自动校验，覆盖关键业务表、关键索引、关键触发器与关键约束。
  - 更新 `db/migrations/v1/README.md`，补充 `040~056` 基线验证脚本说明，形成分段可审计验证入口。
  - 修复本批验证脚本中的触发器 schema 错配（`trg_product_search_refresh` 从 `search` 更正为 `catalog`），并完成复验通过。
- 涉及文件：`db/scripts/verify-migration-040-056.sh`、`db/migrations/v1/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `./db/scripts/migrate-reset.sh`
  2. `./db/scripts/verify-migration-001.sh`
  3. `./db/scripts/verify-migration-010-030.sh`
  4. `./db/scripts/verify-migration-040-056.sh`
  5. `./db/scripts/migrate-status.sh`
  6. `sha256sum -c db/migrations/v1/checksums.sha256`
- 验证结果：通过。空库重建与全量迁移成功；`verify-migration-001.sh`、`verify-migration-010-030.sh`、`verify-migration-040-056.sh` 均返回 `[ok]`；`migrate-status.sh` 显示 `001~070` 全量 `up` 且无 pending；checksum 全量校验通过。
- 覆盖的冻结文档条目：`数据库设计/数据库设计总说明.md`（设计范围、迁移顺序）、`数据库设计/V1/upgrade/040_billing_support_risk.sql`、`数据库设计/V1/upgrade/050_audit_search_dev_ops.sql`、`数据库设计/V1/upgrade/055_audit_hardening.sql`、`数据库设计/V1/upgrade/056_dual_authority_consistency.sql`
- 覆盖的任务清单条目：`DB-007`, `DB-008`, `DB-009`, `DB-010`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`），`ivfflat` 少量数据提示为 PostgreSQL notice，不影响本批验收。

### BATCH-063（实施完成）

- 状态：通过
- 当前任务编号：DB-011, DB-012, DB-013, DB-014
- 当前批次目标：按连续 4 个简单任务一次性完成 `057/058/059/060` 迁移落地验收，补齐“关键表 + 关键索引 + 关键触发器 + 种子数据”自动校验脚本，并通过空库重建链路验证可复现性。
- 前置依赖核对结果：`DB-011~DB-014` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/V1/upgrade/057_search_sync_architecture.sql`、`docs/数据库设计/V1/upgrade/058_recommendation_module.sql`、`docs/数据库设计/V1/upgrade/059_logging_observability.sql`、`docs/数据库设计/V1/upgrade/060_seed_authz_v1.sql`
- 已实现功能：
  - 新增 `db/scripts/verify-migration-057-060.sh`，对 `057/058/059/060` 迁移结果执行自动校验，覆盖搜索投影、推荐对象、观测对象与权限种子基线。
  - 在校验中加入 `authz.role_definition` 与 `authz.permission_definition` 的关键角色/权限点存在性与最小数量门槛检查，确保 `060_seed_authz_v1.sql` 可执行且结果稳定。
  - 更新 `db/migrations/v1/README.md`，补充 `057~060` 基线验证脚本说明。
- 涉及文件：`db/scripts/verify-migration-057-060.sh`、`db/migrations/v1/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `./db/scripts/migrate-reset.sh`
  2. `./db/scripts/verify-migration-001.sh`
  3. `./db/scripts/verify-migration-010-030.sh`
  4. `./db/scripts/verify-migration-040-056.sh`
  5. `./db/scripts/verify-migration-057-060.sh`
  6. `./db/scripts/migrate-status.sh`
  7. `sha256sum -c db/migrations/v1/checksums.sha256`
- 验证结果：通过。空库重建与全量迁移成功；四段校验脚本均返回 `[ok]`；`migrate-status.sh` 显示 `001~070` 全量 `up` 且无 pending；checksum 全量校验通过。
- 覆盖的冻结文档条目：`数据库设计/数据库设计总说明.md`（设计范围、迁移顺序）、`数据库设计/V1/upgrade/057_search_sync_architecture.sql`、`数据库设计/V1/upgrade/058_recommendation_module.sql`、`数据库设计/V1/upgrade/059_logging_observability.sql`、`数据库设计/V1/upgrade/060_seed_authz_v1.sql`
- 覆盖的任务清单条目：`DB-011`, `DB-012`, `DB-013`, `DB-014`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`），`ivfflat` 少量数据提示为 PostgreSQL notice，不影响本批验收。

### BATCH-064（实施完成）

- 状态：通过
- 当前任务编号：DB-015, DB-016, DB-017, DB-018
- 当前批次目标：按连续 4 个简单任务一次性完成 `061/062/063/064` 迁移落地验收，补齐“关键表 + 关键索引 + 关键触发器 + 关键新增字段”自动校验脚本，并通过空库重建链路验证可复现性。
- 前置依赖核对结果：`DB-015~DB-018` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/V1/upgrade/061_data_object_trade_modes.sql`、`docs/数据库设计/V1/upgrade/062_data_product_metadata_contract.sql`、`docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql`、`docs/数据库设计/V1/upgrade/064_storage_layering_architecture.sql`
- 已实现功能：
  - 新增 `db/scripts/verify-migration-061-064.sh`，对 `061/062/063/064` 迁移结果执行自动校验，覆盖对象家族/交易方式、元信息契约、原样加工流水线、分层存储对象的关键表、索引、触发器和关键新增字段。
  - 在字段级校验中加入 `catalog.asset_version`、`catalog.product_sku`、`contract.digital_contract`、`catalog.asset_storage_binding`、`delivery.storage_object` 等关键新增列，确保 `ALTER TABLE` 结果可审计。
  - 更新 `db/migrations/v1/README.md`，补充 `061~064` 基线验证脚本说明。
- 涉及文件：`db/scripts/verify-migration-061-064.sh`、`db/migrations/v1/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `./db/scripts/migrate-reset.sh`
  2. `./db/scripts/verify-migration-001.sh`
  3. `./db/scripts/verify-migration-010-030.sh`
  4. `./db/scripts/verify-migration-040-056.sh`
  5. `./db/scripts/verify-migration-057-060.sh`
  6. `./db/scripts/verify-migration-061-064.sh`
  7. `./db/scripts/migrate-status.sh`
  8. `sha256sum -c db/migrations/v1/checksums.sha256`
- 验证结果：通过。空库重建与全量迁移成功；所有分段校验脚本均返回 `[ok]`；`migrate-status.sh` 显示 `001~070` 全量 `up` 且无 pending；checksum 全量校验通过。
- 覆盖的冻结文档条目：`数据库设计/数据库设计总说明.md`（设计范围、迁移顺序）、`数据库设计/V1/upgrade/061_data_object_trade_modes.sql`、`数据库设计/V1/upgrade/062_data_product_metadata_contract.sql`、`数据库设计/V1/upgrade/063_raw_processing_pipeline.sql`、`数据库设计/V1/upgrade/064_storage_layering_architecture.sql`
- 覆盖的任务清单条目：`DB-015`, `DB-016`, `DB-017`, `DB-018`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`）；并行触发时序导致的一次瞬时误报已通过串行复跑消除，最终结果以复跑通过为准。

### BATCH-065（实施完成）

- 状态：通过
- 当前任务编号：DB-019, DB-020, DB-021, DB-022
- 当前批次目标：按连续 4 个简单任务一次性完成 `065/066/067/068` 迁移落地验收，补齐“关键表 + 关键索引 + 关键触发器 + 关键新增字段 + 权限种子映射”自动校验脚本，并通过空库重建链路验证可复现性。
- 前置依赖核对结果：`DB-019~DB-022` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/V1/upgrade/065_query_execution_plane.sql`、`docs/数据库设计/V1/upgrade/066_sensitive_data_controlled_delivery.sql`、`docs/数据库设计/V1/upgrade/067_trade_chain_monitoring.sql`、`docs/数据库设计/V1/upgrade/068_trade_chain_monitoring_authz.sql`
- 已实现功能：
  - 新增 `db/scripts/verify-migration-065-068.sh`，对 `065/066/067/068` 迁移结果执行自动校验，覆盖查询执行面、敏感受控交付、交易链监控对象与监控权限映射。
  - 在字段级校验中加入 `delivery.template_query_grant`、`delivery.query_execution_run`、`delivery.delivery_record`、`delivery.api_credential`、`catalog.data_asset` 等关键新增列，确保 `ALTER TABLE` 结果可审计。
  - 新增 `068` 权限种子校验：关键 `authz.permission_definition` 与 `authz.role_permission` 映射存在性校验。
  - 更新 `db/migrations/v1/README.md`，补充 `065~068` 基线验证脚本说明。
- 涉及文件：`db/scripts/verify-migration-065-068.sh`、`db/migrations/v1/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `./db/scripts/migrate-reset.sh`
  2. `./db/scripts/verify-migration-001.sh`
  3. `./db/scripts/verify-migration-010-030.sh`
  4. `./db/scripts/verify-migration-040-056.sh`
  5. `./db/scripts/verify-migration-057-060.sh`
  6. `./db/scripts/verify-migration-061-064.sh`
  7. `./db/scripts/verify-migration-065-068.sh`
  8. `./db/scripts/migrate-status.sh`
  9. `sha256sum -c db/migrations/v1/checksums.sha256`
- 验证结果：通过。空库重建与全量迁移成功；所有分段校验脚本均返回 `[ok]`；`migrate-status.sh` 显示 `001~070` 全量 `up` 且无 pending；checksum 全量校验通过。
- 覆盖的冻结文档条目：`数据库设计/数据库设计总说明.md`（设计范围、迁移顺序）、`数据库设计/V1/upgrade/065_query_execution_plane.sql`、`数据库设计/V1/upgrade/066_sensitive_data_controlled_delivery.sql`、`数据库设计/V1/upgrade/067_trade_chain_monitoring.sql`、`数据库设计/V1/upgrade/068_trade_chain_monitoring_authz.sql`
- 覆盖的任务清单条目：`DB-019`, `DB-020`, `DB-021`, `DB-022`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`）；曾出现一次并行时序导致的瞬时误报，已通过串行重跑确认最终结果。

### BATCH-066（实施完成）

- 状态：通过
- 当前任务编号：DB-023, DB-024, DB-025, DB-026
- 当前批次目标：按连续 4 个简单任务一次性完成 `070` 权限种子迁移校验、downgrade 自洽演练、迁移脚本能力复核与基础 lookup 种子落地，形成可重放的 migration/seed 基线。
- 前置依赖核对结果：`DB-023~DB-026` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/V1/upgrade/070_seed_role_permissions_v1.sql`、`docs/数据库设计/V1/downgrade/*.sql`、`docs/权限设计/角色权限矩阵正式版.md`
- 已实现功能：
  - `DB-023`：在 `070_seed_role_permissions_v1.sql` 末尾新增内置校验 SQL（角色数量、权限数量、角色权限映射数量门槛），把“最终 V1 角色权限种子固化 + 校验 SQL”落地到同一迁移脚本。
  - `DB-024`：新增 `db/scripts/verify-migration-roundtrip.sh`，执行“全量升级 -> 全量降级 -> 全量升级”回滚演练；并修复 `db/scripts/migration-runner.sh` 的方向判定逻辑（按每版本最新方向判定 up/down），确保本地回滚后可再次升级。
  - `DB-025`：基于现有 `migrate-up/down/status/reset` 脚本完成能力复核，补齐 runner 对 repeated up/down 的正确行为；脚本链路在回滚演练中完成可执行验证。
  - `DB-026`：新增 `db/seeds/001_base_lookup.sql` 与 `db/seeds/manifest.csv`，落地枚举值、状态字典、产品类目、行业标签、风险等级、交付模式等基础 lookup 种子；新增 `db/scripts/seed-runner.sh`、`db/scripts/seed-up.sh`、`db/scripts/verify-seed-001.sh`。
  - 新增 `db/scripts/verify-migration-070.sh`，用于独立校验 `070` 的关键权限与映射基线。
  - 更新 `db/migrations/v1/README.md`，补充 `070` 校验、回滚演练与 seed 执行/校验入口说明。
- 涉及文件：`docs/数据库设计/V1/upgrade/070_seed_role_permissions_v1.sql`、`db/scripts/migration-runner.sh`、`db/scripts/verify-migration-roundtrip.sh`、`db/scripts/verify-migration-070.sh`、`db/seeds/manifest.csv`、`db/seeds/001_base_lookup.sql`、`db/scripts/seed-runner.sh`、`db/scripts/seed-up.sh`、`db/scripts/verify-seed-001.sh`、`db/migrations/v1/README.md`、`db/migrations/v1/checksums.sha256`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `docker ps --filter name=luna-postgres-test`
  2. `./db/scripts/migrate-reset.sh`
  3. `./db/scripts/verify-migration-001.sh`
  4. `./db/scripts/verify-migration-010-030.sh`
  5. `./db/scripts/verify-migration-040-056.sh`
  6. `./db/scripts/verify-migration-057-060.sh`
  7. `./db/scripts/verify-migration-061-064.sh`
  8. `./db/scripts/verify-migration-065-068.sh`
  9. `./db/scripts/verify-migration-070.sh`
  10. `./db/scripts/verify-migration-roundtrip.sh`
  11. `./db/scripts/seed-up.sh`
  12. `./db/scripts/verify-seed-001.sh`
  13. `./db/scripts/migrate-status.sh`
  14. `sha256sum -c db/migrations/v1/checksums.sha256`
- 验证结果：通过。`070` 校验通过；回滚演练通过（全量 downgrade 后可全量 re-upgrade）；seed 执行与 `001` 校验通过；`migrate-status.sh` 显示 pending 为空；checksum 全量通过。
- 覆盖的冻结文档条目：`数据库设计/数据库设计总说明.md`（迁移顺序与本地可重建原则）、`数据库设计/V1/upgrade/070_seed_role_permissions_v1.sql`（V1 角色权限最终固化）、`权限设计/角色权限矩阵正式版.md`（关键角色权限映射）
- 覆盖的任务清单条目：`DB-023`, `DB-024`, `DB-025`, `DB-026`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`）；`ivfflat` notice 为低数据量提示，不影响本批验收。

### BATCH-067（实施完成）

- 状态：通过
- 当前任务编号：DB-027, DB-028, DB-029
- 当前批次目标：按连续 3 个简单任务一次性完成本地演示租户/用户种子、8 个标准 SKU 商品与模板绑定种子、全量订单样例与五条标准链路场景订单种子，并纳入可执行验证链路。
- 前置依赖核对结果：`DB-027~DB-029` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/V1/upgrade/010_identity_and_access.sql`、`docs/数据库设计/V1/upgrade/020_catalog_contract.sql`、`docs/数据库设计/V1/upgrade/030_trade_delivery.sql`、`docs/数据库设计/V1/upgrade/061_data_object_trade_modes.sql`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - `DB-027`：新增 `db/seeds/010_test_tenants.sql`，落地本地演示租户、卖方、买方、运营、审计、开发与管理员用户，并写入 `authz.subject_role_binding`。
  - `DB-028`：新增 `db/seeds/020_test_products.sql`，落地 8 个标准 SKU（`FILE_STD/FILE_SUB/SHARE_RO/API_SUB/API_PPU/QRY_LITE/SBX_STD/RPT_STD`）对应资产、版本、商品、SKU 与模板绑定示例。
  - `DB-029`：新增 `db/seeds/030_test_orders.sql`，落地 13 条订单样例（8 条 SKU 全量订单 + 5 条标准链路场景订单），并补充授权、API 凭证、沙箱与报告示例对象。
  - 更新 `db/seeds/manifest.csv`，接入 `010/020/030` 种子顺序。
  - 新增 `db/scripts/verify-seed-010-030.sh` 并更新 `db/migrations/v1/README.md`，将本批种子基线纳入可执行校验。
- 涉及文件：`db/seeds/manifest.csv`、`db/seeds/010_test_tenants.sql`、`db/seeds/020_test_products.sql`、`db/seeds/030_test_orders.sql`、`db/scripts/verify-seed-010-030.sh`、`db/migrations/v1/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `./db/scripts/migrate-reset.sh`
  2. `./db/scripts/seed-up.sh`
  3. `./db/scripts/verify-seed-001.sh`
  4. `./db/scripts/verify-seed-010-030.sh`
  5. `./db/scripts/migrate-status.sh`
  6. `psql -h 127.0.0.1 -p 55432 -U luna -d luna_data_trading -tAc "SELECT version, checksum_sha256 FROM public.seed_history ORDER BY version;"`
  7. `sha256sum -c db/migrations/v1/checksums.sha256`
- 验证结果：通过。`seed-up` 成功执行 `001/010/020/030`；`verify-seed-001.sh` 与 `verify-seed-010-030.sh` 均返回 `[ok]`；`seed_history` 已记录 `001/010/020/030`；`migrate-status.sh` 显示无 pending；migration checksum 全量通过。
- 覆盖的冻结文档条目：`数据库设计/V1/upgrade/010_identity_and_access.sql`（身份主体基础结构）、`数据库设计/V1/upgrade/020_catalog_contract.sql`（商品/模板/SKU 结构）、`数据库设计/V1/upgrade/030_trade_delivery.sql`（订单主链路结构）、`数据库设计/V1/upgrade/061_data_object_trade_modes.sql`（交易方式/交付模式扩展）、`全集成文档/数据交易平台-全集成基线-V1.md`（首批标准链路与 SKU 映射）
- 覆盖的任务清单条目：`DB-027`, `DB-028`, `DB-029`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`）。

### BATCH-068（实施完成）

- 状态：通过
- 当前任务编号：DB-030, DB-031, DB-032, DB-033
- 当前批次目标：按连续 4 个简单任务一次性完成 `docs/03-db` 表字典与状态机映射、迁移兼容性回归脚本、索引审查基线与高频路径索引补强，并完成全链路可执行验证。
- 前置依赖核对结果：`DB-030~DB-033` 依赖 `BOOT-008; ENV-005; ENV-006; CORE-005`，上述依赖任务均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/数据库表字典正式版.md`、`docs/数据库设计/表关系总图-ER文本图.md`、`docs/领域模型/全量领域模型与对象关系说明.md`、`docs/数据库设计/数据库设计总说明.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - `DB-030`：新增 `db/scripts/export-table-catalog.sh` 并导出 `docs/03-db/table-catalog.md`（按 schema 输出表、主键、唯一键、外键、索引、职责）。
  - `DB-031`：新增 `docs/03-db/state-machine-to-table-map.md`，完成订单/授权/交付/结算/争议状态机到字段与索引的映射。
  - `DB-032`：新增 `db/scripts/verify-db-compatibility.sh` 与 `docs/03-db/migration-compatibility.md`，建立 migration+seed 兼容性回归链路（重建、种子、回滚重放、再种子、状态检查）。
  - `DB-033`：新增 `db/scripts/review-index-baseline.sh` 与 `docs/03-db/index-review-baseline.md`；补充高频路径索引基线（搜索回查、订单详情、审计联查、ops 列表）并落入升级 SQL：
    - `020_catalog_contract.sql`：`idx_product_tag_tag_id`
    - `030_trade_delivery.sql`：`idx_order_line_order_id`、`idx_authorization_grant_order_id`、`idx_delivery_ticket_order_id`、`idx_sandbox_workspace_order_id`、`idx_report_artifact_order_id`
    - `050_audit_search_dev_ops.sql`：`idx_job_run_status_started`、`idx_chain_anchor_status_created`、`idx_contract_event_projection_ref`、`idx_mock_payment_case_status`
  - 更新 `docs/03-db/README.md` 与 `db/migrations/v1/README.md` 索引，补充新文档与脚本入口。
  - 更新 `db/migrations/v1/checksums.sha256` 与新增索引后的 migration checksum。
- 涉及文件：`db/scripts/export-table-catalog.sh`、`db/scripts/verify-db-compatibility.sh`、`db/scripts/review-index-baseline.sh`、`docs/03-db/table-catalog.md`、`docs/03-db/state-machine-to-table-map.md`、`docs/03-db/migration-compatibility.md`、`docs/03-db/index-review-baseline.md`、`docs/03-db/README.md`、`docs/数据库设计/V1/upgrade/020_catalog_contract.sql`、`docs/数据库设计/V1/upgrade/030_trade_delivery.sql`、`docs/数据库设计/V1/upgrade/050_audit_search_dev_ops.sql`、`db/migrations/v1/README.md`、`db/migrations/v1/checksums.sha256`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `./db/scripts/migrate-reset.sh`
  2. `./db/scripts/export-table-catalog.sh`
  3. `./db/scripts/verify-db-compatibility.sh`
  4. `./db/scripts/review-index-baseline.sh`
  5. `./db/scripts/migrate-status.sh`
  6. `sha256sum -c db/migrations/v1/checksums.sha256`
- 验证结果：通过。兼容性回归脚本返回 `[ok] db compatibility baseline verified`；索引审查脚本返回 `[ok] index baseline review passed`；`migrate-status.sh` 无 pending；checksum 全量通过。
- 覆盖的冻结文档条目：`数据库表字典正式版`（表字典结构映射）、`领域模型生命周期总表`（状态机映射）、`数据库设计总说明`（迁移顺序/重建原则）、`全集成基线-V1`（搜索与链路一致性要求）
- 覆盖的任务清单条目：`DB-030`, `DB-031`, `DB-032`, `DB-033`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`）；`ivfflat` notice 为低数据量提示，不影响验收。

### BATCH-069（实施完成）

- 状态：通过
- 当前任务编号：DB-035
- 当前批次目标：完成 `db/seeds/032_five_scenarios.sql`，把五条标准链路与主/补充 SKU、合同模板、验收模板、退款模板映射固化为可查询演示数据，并接入 seed 执行与校验链路。
- 前置依赖核对结果：`DB-035` 依赖 `DB-028; CTX-007; CTX-021`，上述依赖均已完成且审批通过；`DB-034` 依赖 `BIL-023` 未完成，按强制暂停规则登记阻塞项 `TODO-DB-034-001`。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/00-context/first-5-scenarios.md`、`docs/00-context/v1-closed-loop-matrix.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - 新增 `db/seeds/032_five_scenarios.sql`，将五条标准链路官方场景名（S1~S5）固化到 `developer.test_application`，并在 `metadata` 中写入主/补充 SKU、合同模板、验收模板、退款模板及场景样例订单映射。
  - 更新 `db/seeds/manifest.csv`，接入 `032` 种子执行顺序。
  - 新增 `db/scripts/verify-seed-032.sh`，校验五条场景映射、主 SKU 口径和模板映射完整性。
  - 更新 `db/migrations/v1/README.md`，补充 `032` 种子与校验脚本入口说明。
- 涉及文件：`db/seeds/032_five_scenarios.sql`、`db/seeds/manifest.csv`、`db/scripts/verify-seed-032.sh`、`db/migrations/v1/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `./db/scripts/migrate-reset.sh`
  2. `./db/scripts/seed-up.sh`
  3. `./db/scripts/verify-seed-001.sh`
  4. `./db/scripts/verify-seed-010-030.sh`
  5. `./db/scripts/verify-seed-032.sh`
  6. `./db/scripts/verify-db-compatibility.sh`
  7. `./db/scripts/migrate-status.sh`
  8. `sha256sum -c db/migrations/v1/checksums.sha256`
  9. `psql -h 127.0.0.1 -p 55432 -U luna -d luna_data_trading -tAc "SELECT version, name FROM public.seed_history ORDER BY version;"`
- 验证结果：通过。`seed-up` 成功执行 `001/010/020/030/032`；`verify-seed-001.sh`、`verify-seed-010-030.sh`、`verify-seed-032.sh` 均返回 `[ok]`；`verify-db-compatibility.sh` 返回 `[ok] db compatibility baseline verified`；`migrate-status.sh` 无 pending；`seed_history` 已记录 `032`；migration checksum 全量通过。
- 覆盖的冻结文档条目：`first-5-scenarios.md`（五条标准链路官方命名）、`v1-closed-loop-matrix.md`（主挂点/补充挂点 SKU 口径）、`全集成基线-V1`（首批标准场景到 SKU/模板映射）
- 覆盖的任务清单条目：`DB-035`
- 未覆盖项：`DB-034`（前置依赖 `BIL-023` 未完成，已登记阻塞）
- 新增 TODO / 预留项：`TODO-DB-034-001`（`blocked`）
- 待人工审批结论：通过
- 备注：验证数据库容器为 `luna-postgres-test`（`127.0.0.1:55432 -> 5432`）；`ivfflat` notice 为低数据量提示，不影响本批验收。

### BATCH-070（实施完成）

- 状态：待审批
- 当前任务编号：BIL-001
- 当前批次目标：实现 `Payment Jurisdiction / Corridor / Payout Preference` 的基础模型与接口占位，补齐最小权限校验、审计留痕、错误码映射、OpenAPI 草案与最小测试。
- 前置依赖核对结果：`BIL-001` 依赖 `TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009`；当前仓库显示上述依赖已完成并审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/支付域接口协议正式版.md`、`docs/原始PRD/支付、资金流与轻结算设计.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - 在 `billing` 模块落地基础领域模型：`JurisdictionProfile`、`CorridorPolicy`、`PayoutPreference`。
  - 在 `billing` 模块落地最小服务与权限判定：`BillingPermission::ReadPolicy`，允许角色 `platform_admin / platform_finance_operator / tenant_admin`。
  - 新增支付域接口占位：
    - `GET /api/v1/billing/policies`
    - `GET /api/v1/billing/payout-preferences/{beneficiary_subject_id}`
  - 接口返回结构化模型数据，拒绝未授权角色并返回 `IAM_UNAUTHORIZED` 错误码。
  - 接口处理新增日志审计留痕占位（`billing.policy.read`、`billing.payout_preference.read`）。
  - 更新 `packages/openapi/billing.yaml`，与当前实现路径、权限头、响应模型对齐。
  - 补齐最小测试：权限拒绝、授权成功、模型基线校验。
- 涉及文件：`apps/platform-core/Cargo.toml`、`apps/platform-core/src/lib.rs`、`apps/platform-core/src/modules/billing/mod.rs`、`apps/platform-core/src/modules/billing/domain.rs`、`apps/platform-core/src/modules/billing/service.rs`、`apps/platform-core/src/modules/billing/api.rs`、`packages/openapi/billing.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt`
  2. `cargo test -p platform-core`
- 验证结果：通过。新增 4 条 `billing` 相关测试全部通过（权限拒绝、权限放行、Jurisdiction/Corridor 基线、模块配置），`platform-core` 总体测试通过。
- 覆盖的冻结文档条目：`支付域接口协议正式版`（7.1/7.2/7.3 基础对象协议与 V1 读接口边界）、`支付、资金流与轻结算设计`（独立支付子域、SG 起步司法辖区与走廊策略）、`全集成基线-V1`（支付域与交易域解耦、V1 最小支付域占位）
- 覆盖的任务清单条目：`BIL-001`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过

### BATCH-071（实施完成）

- 状态：通过
- 当前任务编号：BIL-002
- 当前批次目标：实现支付意图接口 `POST /api/v1/payments/intents`、`GET /api/v1/payments/intents/{id}`、`POST /api/v1/payments/intents/{id}/cancel`，覆盖最小权限校验、错误码响应、幂等键复用与数据库落表。
- 前置依赖核对结果：`BIL-002` 依赖 `TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009`，当前均已完成并审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/支付域接口协议正式版.md`、`docs/原始PRD/支付、资金流与轻结算设计.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - 新增支付意图接口并接入 PostgreSQL `payment.payment_intent` 表真实读写：
    - `POST /api/v1/payments/intents`
    - `GET /api/v1/payments/intents/{id}`
    - `POST /api/v1/payments/intents/{id}/cancel`
  - 支持 `x-idempotency-key` 幂等复用：重复创建返回同一 `payment_intent`。
  - 新增支付意图权限模型：`PaymentIntentRead / PaymentIntentCreate / PaymentIntentCancel`；落地 `x-role` 校验。
  - 取消支付支持状态保护：`succeeded/failed/expired` 禁止取消，返回冲突错误。
  - 更新 `packages/openapi/billing.yaml`，补齐支付意图请求/响应/错误码契约。
  - 保持审计留痕：支付意图创建、幂等重放、读取、取消均输出结构化日志。
- 涉及文件：`apps/platform-core/Cargo.toml`、`apps/platform-core/src/modules/billing/api.rs`、`apps/platform-core/src/modules/billing/service.rs`、`packages/openapi/billing.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt`
  2. `cargo test -p platform-core`
  3. `./scripts/check-mock-payment.sh`
  4. 启动服务（联调环境变量）：`DATABASE_URL=postgres://luna:5686@127.0.0.1:55432/luna_data_trading PROVIDER_MODE=mock KAFKA_BROKERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core-bin`
  5. 接口联调（HTTP）：
     - `POST /api/v1/payments/intents`（`x-role: tenant_admin`）
     - 幂等重放同请求（同 `x-idempotency-key`）
     - `GET /api/v1/payments/intents/{id}`（`x-role: tenant_operator`）
     - `POST /api/v1/payments/intents/{id}/cancel`（`tenant_operator` 预期拒绝）
     - `POST /api/v1/payments/intents/{id}/cancel`（`tenant_admin` 预期成功）
  6. 数据库回查：
     - `SELECT payment_intent_id::text,status,idempotency_key,request_id,provider_key,amount::text FROM payment.payment_intent WHERE idempotency_key='idem-bil002-001';`
     - `SELECT payment_intent_id::text,status,updated_at::text FROM payment.payment_intent WHERE payment_intent_id='4f4b3a2e-508b-4902-ba35-97aa905b3772'::uuid;`
- 验证结果：通过。单测 `8/8` 通过；mock payment 脚本返回 `[ok]`；创建接口返回 `200` 且写入数据库；幂等重放返回 `200` 且同一 `payment_intent_id`；无权限取消返回 `403 + IAM_UNAUTHORIZED`；管理员取消返回 `200`，数据库状态更新为 `canceled`。
- 覆盖的冻结文档条目：`支付域接口协议正式版`（V1 支付意图创建/查询/取消、幂等规则）、`支付、资金流与轻结算设计`（支付编排层最小闭环）、`全集成基线-V1`（支付域与交易域解耦、V1 Mock+真实接口占位）
- 覆盖的任务清单条目：`BIL-002`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批联调数据库容器为 `luna-postgres-test`（`127.0.0.1:55432`，`luna/luna_data_trading`）；mock 支付容器为 `datab-mock-payment-provider`（`127.0.0.1:8089`）。

### BATCH-072（实施完成）

- 状态：通过
- 当前任务编号：BIL-003
- 当前批次目标：实现 `POST /api/v1/orders/{id}/lock`，把订单与支付意图关联，落地最小权限校验、错误码、审计日志与数据库写入。
- 前置依赖核对结果：`BIL-003` 依赖 `TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009`，当前均已完成并审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/支付域接口协议正式版.md`、`docs/原始PRD/支付、资金流与轻结算设计.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - 新增订单锁资接口：`POST /api/v1/orders/{id}/lock`。
  - 接口校验 `payment_intent_id` 存在且与订单一致；不一致返回 `409 + BIL_PROVIDER_FAILED`。
  - 锁资落库更新 `trade.order_main`：
    - `payment_status='locked'`
    - `buyer_locked_at`（首次锁定）
    - `payment_channel_snapshot` 追加 `payment_intent_id/provider_key/lock_reason/locked_at`
  - 写入 `trade.order_status_history` 锁资历史记录。
  - 新增权限 `BillingPermission::OrderLock`，拒绝 `tenant_operator` 执行锁资。
  - 更新 `packages/openapi/billing.yaml`，补充 `/api/v1/orders/{id}/lock` 请求/响应契约。
  - 输出结构化日志：`action=\"order.payment.lock\"`。
- 涉及文件：`apps/platform-core/src/modules/billing/api.rs`、`apps/platform-core/src/modules/billing/service.rs`、`packages/openapi/billing.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt`
  2. `cargo test -p platform-core`
  3. `./scripts/check-mock-payment.sh`
  4. 启动服务（联调环境变量）：`DATABASE_URL=postgres://luna:5686@127.0.0.1:55432/luna_data_trading PROVIDER_MODE=mock KAFKA_BROKERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core-bin`
  5. 接口联调：
     - `POST /api/v1/payments/intents`（创建用于锁资的支付意图）
     - `POST /api/v1/orders/{id}/lock`（`tenant_operator`，预期 `403`）
     - `POST /api/v1/orders/{id}/lock`（`tenant_admin`，预期 `200`）
     - 用“属于其他订单”的 `payment_intent_id` 调锁资（预期 `409`）
  6. DB 回查：
     - `trade.order_main` 中 `payment_status` 与 `payment_channel_snapshot.payment_intent_id`
     - `trade.order_status_history` 新增记录
- 验证结果：通过。单测 `10/10`；mock-payment 探针 `[ok]`；权限拒绝返回 `403 + IAM_UNAUTHORIZED`；授权锁资返回 `200` 并落库；跨订单 intent 锁资返回 `409 + BIL_PROVIDER_FAILED`；历史表成功写入。
- 覆盖的冻结文档条目：`支付域接口协议正式版`（支付状态一致性与接口约束）、`支付、资金流与轻结算设计`（支付编排到订单联动）、`全集成基线-V1`（支付域驱动主链路闭环）
- 覆盖的任务清单条目：`BIL-003`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批联调数据库容器为 `luna-postgres-test`（`127.0.0.1:55432`，用户 `luna`，库 `luna_data_trading`）；mock 支付容器为 `datab-mock-payment-provider`（`127.0.0.1:8089`）。

### BATCH-073（实施完成）

- 状态：通过
- 当前任务编号：BIL-004
- 当前批次目标：实现 Mock Payment Provider 适配器，支持 `success/fail/timeout` 三种模拟结果并可生成对应 webhook 事件载荷，补齐最小测试与容器联调验证。
- 前置依赖核对结果：`BIL-004` 依赖 `TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009`，当前均已完成并审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/支付域接口协议正式版.md`、`docs/原始PRD/支付、资金流与轻结算设计.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - 在 `provider-kit` 扩展 `PaymentProvider` 适配能力，新增 `simulate_webhook` 接口与 `MockPaymentScenario`（`success/fail/timeout`）统一枚举。
  - 新增 `MockPaymentWebhookEvent` 结构，统一 webhook 事件最小字段（`provider_event_id/payment_intent_id/event_type/provider_status/http_status_code`）。
  - `MockPaymentProvider` 支持两种模式：
    - `stub`（默认）：直接返回三类场景事件。
    - `live`：通过 `MOCK_PAYMENT_BASE_URL` 调用 mock-payment 容器接口（`/mock/payment/charge/success|fail|timeout`）并映射响应。
  - `timeout` 场景支持超时语义：3 秒客户端超时视为有效超时结果。
  - 补齐单测：
    - `stub` 三场景覆盖测试；
    - `live` 三场景联调测试（`#[ignore]`，联调时显式执行）。
- 涉及文件：`apps/platform-core/crates/provider-kit/Cargo.toml`、`apps/platform-core/crates/provider-kit/src/lib.rs`、`Cargo.lock`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p provider-kit`
  3. `cargo test -p platform-core`
  4. `MOCK_PAYMENT_ADAPTER_MODE=live cargo test -p provider-kit live_mock_payment_adapter_hits_three_mock_paths -- --ignored --nocapture`
  5. `./scripts/check-mock-payment.sh`
- 验证结果：通过。`provider-kit` 单测 `2 passed + 1 ignored`，`platform-core` 单测 `10 passed`；`live` 联调用例通过；`check-mock-payment.sh` 返回 `[ok] mock-payment scenarios verified`。
- 覆盖的冻结文档条目：`支付域接口协议正式版`（支付 provider 适配、幂等与一致性约束）、`支付、资金流与轻结算设计`（支付子域 provider 抽象与 mock 联调路径）、`全集成基线-V1`（V1 mock/real provider 双轨边界）
- 覆盖的任务清单条目：`BIL-004`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：联调容器为 `datab-mock-payment-provider`（`127.0.0.1:8089`）；沙箱环境不能直接访问本机端口，live 联调与脚本探针使用提权执行完成验证。

### BATCH-074（返工后待审批）

- 状态：返工后待审批
- 当前任务编号：BIL-002, BIL-003
- 当前批次目标：修复 `BIL-003` 的状态历史一致性缺陷并补齐 `BIL-002/BIL-003` 审计落地。
- 前置依赖核对结果：`BIL-002/BIL-003` 前置依赖均已完成且审批通过，无阻塞。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/支付域接口协议正式版.md`、`docs/原始PRD/支付、资金流与轻结算设计.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - `BIL-003` 一致性修复：删除 `order lock` 接口中手工插入 `trade.order_status_history` 的逻辑，避免生成 `old_status=new_status` 的伪状态迁移记录。
  - `BIL-002` 审计补齐：`create/get/cancel payment_intent` 成功路径新增 `audit.audit_event` 持久化，记录 `action_name/ref_type/ref_id/result_code/request_id/trace_id/actor_role`。
  - `BIL-003` 审计补齐：`order lock` 成功路径新增 `audit.audit_event` 持久化（`order.payment.lock`）。
  - 保持原有权限校验、错误码与 OpenAPI 路径不变，避免接口契约漂移。
- 涉及文件：`apps/platform-core/src/modules/billing/api.rs`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务：`DATABASE_URL=postgres://luna:5686@127.0.0.1:55432/luna_data_trading PROVIDER_MODE=mock KAFKA_BROKERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core-bin`
  4. 联调接口：
     - `POST /api/v1/payments/intents`
     - `GET /api/v1/payments/intents/{id}`
     - `POST /api/v1/payments/intents/{id}/cancel`
     - `POST /api/v1/orders/{id}/lock`
  5. 审计回查：`SELECT action_name, ref_type, ref_id::text, result_code FROM audit.audit_event WHERE ref_id IN (...) ORDER BY event_time DESC LIMIT 12;`
  6. 状态历史回查：`SELECT old_status,new_status,reason_code,changed_at FROM trade.order_status_history WHERE order_id=... ORDER BY changed_at DESC LIMIT 5;`
- 验证结果：通过。接口创建/读取/取消/锁资均返回 `success=true`；`audit.audit_event` 可回查到 `payment.intent.create/read/cancel` 与 `order.payment.lock` 记录；本次锁资操作未再写入新的 `old_status=new_status` 伪迁移历史。
- 覆盖的冻结文档条目：`支付域接口协议正式版`（支付意图与锁资接口、幂等与一致性）、`支付、资金流与轻结算设计`（支付审计留痕）、`全集成基线-V1`（审计与业务日志分离，审计主链落库）。
- 覆盖的任务清单条目：`BIL-002`, `BIL-003`（返工补齐）
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：联调数据库容器 `luna-postgres-test`（`127.0.0.1:55432`），mock 支付容器 `datab-mock-payment-provider`（`127.0.0.1:8089`）。

### BATCH-075（实施完成）

- 状态：通过
- 当前任务编号：BIL-005
- 当前批次目标：实现支付 webhook 接口 `POST /api/v1/payments/webhooks/{provider}`，支持签名占位、幂等、防重放、乱序保护，并补齐审计与最小测试。
- 前置依赖核对结果：`BIL-005` 依赖 `TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009`，当前均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/支付域接口协议正式版.md`、`docs/原始PRD/支付、资金流与轻结算设计.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 已实现功能：
  - 新增 webhook 接口：`POST /api/v1/payments/webhooks/{provider}`。
  - 签名占位：支持 `x-provider-signature`，`mock_payment` 默认要求值为 `mock-signature`（可由 `MOCK_PAYMENT_WEBHOOK_SIGNATURE` 覆盖）。
  - 幂等去重：基于 `payment.payment_webhook_event(provider_key, provider_event_id)` 唯一性，实现重复回调标记 `duplicate` 且不重复推进状态。
  - 防重放：基于 `x-webhook-timestamp` / `occurred_at_ms` 执行时间窗校验（向后 15 分钟、向前 2 分钟），超窗标记 `rejected_replay`。
  - 乱序保护：基于 `payment_intent.metadata.webhook_last_occurred_at_ms` 与状态等级进行回退保护，旧事件标记 `out_of_order_ignored`。
  - 状态推进：映射 `payment.succeeded|failed|timeout` 到 `payment_intent.status`（`succeeded|failed|expired`），并回写 webhook 元数据。
  - 审计落地：写入 `audit.audit_event`，覆盖 `payment.webhook.processed / duplicate / rejected_replay / rejected_signature / out_of_order_ignored`。
  - OpenAPI 更新：补齐 webhook 路径、请求/响应 DTO 与错误响应描述。
  - 最小单测补齐：新增 webhook 规则函数测试（状态映射、重放窗口、状态等级回退保护）。
- 涉及文件：`apps/platform-core/src/modules/billing/api.rs`、`apps/platform-core/Cargo.toml`、`packages/openapi/billing.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务：`DATABASE_URL=postgres://luna:5686@127.0.0.1:55432/luna_data_trading PROVIDER_MODE=mock KAFKA_BROKERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core-bin`
  4. 创建支付意图：`POST /api/v1/payments/intents`
  5. webhook 场景联调：
     - 正常成功回调（`processed`）
     - 同 `provider_event_id` 重复回调（`duplicate`）
     - 旧时间戳失败回调（`out_of_order_ignored`）
     - 超窗重放回调（`rejected_replay`）
     - 错误签名回调（`rejected_signature`）
  6. DB 回查：
     - `payment.payment_intent` 状态与 webhook 元数据
     - `payment.payment_webhook_event` 处理状态与重复标记
     - `audit.audit_event` 对应动作记录
- 验证结果：通过。`platform-core` 单测 `13/13` 通过；联调场景返回符合预期：`processed / duplicate / out_of_order_ignored / rejected_replay / rejected_signature`；`payment_intent` 最终状态保持 `succeeded`，未被旧失败事件回退；审计表可见对应动作记录。
- 覆盖的冻结文档条目：`支付域接口协议正式版`（webhook 幂等与一致性要求）、`支付、资金流与轻结算设计`（支付回调入口与防重放）、`全集成基线-V1`（支付域审计与状态防回退约束）
- 覆盖的任务清单条目：`BIL-005`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：联调数据库容器 `luna-postgres-test`（`127.0.0.1:55432`），mock 支付容器 `datab-mock-payment-provider`（`127.0.0.1:8089`）。

### BATCH-076（实施完成）

- 状态：通过
- 当前任务编号：TRADE-030
- 当前批次目标：实现支付结果到订单推进编排器：支付成功推进到“已锁资/待交付”，支付失败推进到“支付失败待处理”，支付超时推进到“支付超时待补偿/待取消”，并保证状态不可倒退。
- 前置依赖核对结果：`TRADE-030` 依赖 `BIL-005; TRADE-007; CORE-014`，当前已满足（`BIL-005` 已实现，`TRADE-007/CORE-014` 历史已完成并审批通过）。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/领域模型/全量领域模型与对象关系说明.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`、`docs/业务流程/业务流程图-V1-完整版.md`
- 已实现功能：
  - 新增订单支付结果编排器（`order` 模块）：
    - 支付成功：推进 `order_main.status -> buyer_locked`，`payment_status -> paid`，`last_reason_code -> payment_succeeded_to_buyer_locked`。
    - 支付失败：推进 `order_main.status -> payment_failed_pending_resolution`，`payment_status -> failed`。
    - 支付超时：推进 `order_main.status -> payment_timeout_pending_compensation_cancel`，`payment_status -> expired`。
  - 增加不可倒退保护：当订单已进入 `seller_delivering/delivered/accepted/settled/closed` 等后序状态时，后续失败/超时事件只记审计，不回退订单状态。
  - 在 `BIL-005` webhook 的 `processed` 分支挂接编排器，实现“支付结果 -> 订单推进”的实时联动闭环。
  - 增加订单审计记录：
    - `order.payment.result.applied`
    - `order.payment.result.ignored`
  - 更新 `packages/openapi/trade.yaml`，补充 `TRADE-030` 的事件驱动编排说明与状态迁移 schema。
- 涉及文件：`apps/platform-core/src/modules/order/mod.rs`、`apps/platform-core/src/modules/order/domain/mod.rs`、`apps/platform-core/src/modules/order/application/mod.rs`、`apps/platform-core/src/modules/billing/api.rs`、`packages/openapi/trade.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务：`DATABASE_URL=postgres://luna:5686@127.0.0.1:55432/luna_data_trading PROVIDER_MODE=mock KAFKA_BROKERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core-bin`
  4. 构造前序订单（`status=contract_effective`）并创建支付意图。
  5. 回调 `payment.succeeded`（当前时戳）后再回调旧时戳 `payment.failed`，验证乱序保护。
  6. DB 回查：
     - `trade.order_main` 的 `status/payment_status/last_reason_code`
     - `audit.audit_event` 的 `order.payment.result.applied/ignored`
     - `payment.payment_webhook_event` 的处理状态
- 验证结果：通过。构造订单 `30000000-0000-0000-0000-000000009901` 在 `payment.succeeded` 后推进为 `buyer_locked|paid|payment_succeeded_to_buyer_locked`；随后旧时戳 `payment.failed` 被标记 `out_of_order_ignored`，未回退订单状态；审计表存在 `order.payment.result.applied` 记录。
- 覆盖的冻结文档条目：`全量领域模型`（订单主状态 `created->buyer_locked->seller_delivering...`）、`全集成基线-V1`（支付结果驱动订单推进与防回退）、`业务流程图-V1`（支付后进入待交付链路）。
- 覆盖的任务清单条目：`TRADE-030`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：联调数据库容器 `luna-postgres-test`（`127.0.0.1:55432`），mock 支付容器 `datab-mock-payment-provider`（`127.0.0.1:8089`）。

### BATCH-077（待审批）

- 状态：通过
- 当前任务编号：N/A（流程纠偏批次）
- 当前批次目标：冻结“继续新增 BIL 任务”和“已完成 BIL 任务改动”，形成可审计回溯记录。
- 前置依赖核对结果：不涉及代码构建与环境依赖；仅文档治理调整。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（阶段顺序基准）、`docs/开发任务/Agent-开发与半人工审核流程.md`（流程执行约束）
- 已实现功能：
  - 在 TODO 清单新增 `TODO-PROC-BIL-001`（`blocked`），显式冻结 BIL 线后续新增任务与既有已完成 BIL 任务改动。
  - 明确冻结原因、解冻前置条件（`IAM-001~IAM-020` 完成并审批通过）及人工解冻口径。
  - 在 TODO 批次更新记录中补记 `BATCH-077` 冻结登记条目，保证审计回溯链完整。
- 涉及文件：`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `rg -n "TODO-PROC-BIL-001|BATCH-077" docs/开发任务/V1-Core-TODO与预留清单.md docs/开发任务/V1-Core-实施进度日志.md`
  2. `git diff -- docs/开发任务/V1-Core-TODO与预留清单.md docs/开发任务/V1-Core-实施进度日志.md`
- 验证结果：通过。冻结条目与批次记录均已落盘，内容包含冻结范围、阻塞状态、解冻条件与回溯信息。
- 覆盖的冻结文档条目：任务顺序基准、批次流程治理与 TODO 审计回溯要求。
- 覆盖的任务清单条目：N/A（流程纠偏批次，非新增业务任务实现）。
- 未覆盖项：无
- 新增 TODO / 预留项：`TODO-PROC-BIL-001`（`tech-debt`，`blocked`）
- 待人工审批结论：通过
- 备注：本批次不涉及业务代码修改，不触发构建或集成测试。

### BATCH-078（待审批）

- 状态：通过
- 当前任务编号：BIL-023, DB-034
- 当前批次目标：完成 `DB-034` 全量闭环（依赖文档 + 种子 SQL + 校验脚本 + 可执行验证）。
- 前置依赖核对结果：
  - `BIL-023`：依赖满足，已补齐交付 `docs/03-db/sku-billing-trigger-matrix.md`。
  - `DB-034`：`DB-029; CTX-021` 已完成，`BIL-023` 已在本批补齐后满足。
  - `TODO-PROC-BIL-001`：保持冻结；本批仅做 `DB-034` 所需最小依赖补齐，不推进其它 BIL 开发项。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`、`docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`、`docs/原始PRD/支付、资金流与轻结算设计.md`
- 已实现功能：
  - `BIL-023`：新增 `docs/03-db/sku-billing-trigger-matrix.md`，逐项冻结 8 个 SKU 的支付触发、交付触发、验收触发、计费触发、结算周期、退款入口、赔付入口、争议冻结点、恢复结算点。
  - `DB-034`：新增 `db/seeds/031_sku_trigger_matrix.sql`，将上述 8 SKU 口径固化到 `billing.sku_billing_trigger_matrix`（可查询配置表，支持幂等 upsert）。
  - 新增 `db/scripts/verify-seed-031.sh`，校验矩阵表记录总数与关键 SKU 字段映射。
  - 更新 `db/seeds/manifest.csv` 接入 `031`；更新 `docs/03-db/README.md` 索引。
- 涉及文件：`docs/03-db/sku-billing-trigger-matrix.md`、`db/seeds/031_sku_trigger_matrix.sql`、`db/scripts/verify-seed-031.sh`、`db/seeds/manifest.csv`、`docs/03-db/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `bash db/scripts/migrate-reset.sh`
  2. `bash db/scripts/migrate-up.sh`
  3. `bash db/scripts/seed-up.sh`
  4. `bash db/scripts/verify-seed-031.sh`
  5. `bash db/scripts/verify-seed-032.sh`
  6. `bash db/scripts/verify-db-compatibility.sh`
- 验证结果：通过。`migrate-reset/up`、`seed-up`、`verify-seed-031`、`verify-seed-032`、`verify-db-compatibility` 全部通过；运行中仅出现历史已知 `NOTICE`（如 ivfflat 小样本提示、不存在触发器/索引的 drop 提示），无失败或中断。
- 覆盖的冻结文档条目：V1 首批 8 SKU 触发口径冻结、计费触发与争议冻结恢复口径、数据库配置可查询基线。
- 覆盖的任务清单条目：`BIL-023`, `DB-034`
- 未覆盖项：无
- 新增 TODO / 预留项：无（关闭 `TODO-DB-034-001`，`TODO-PROC-BIL-001` 保持冻结）
- 待人工审批结论：通过
- 备注：本批仅执行 `DB-034` 阻塞链解除与闭环验证，不扩展其他 BIL 功能任务。

### BATCH-079（待审批）

- 状态：通过
- 当前任务编号：IAM-001, IAM-002, IAM-003, IAM-004
- 当前批次目标：完成 IAM 起始 4 个任务闭环（主体聚合、基础身份 CRUD、登录上下文镜像、组织注册接口）。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018` 均已完成并审批通过；本批无阻塞。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/身份与会话接口协议正式版.md`、`docs/数据库设计/V1/upgrade/010_identity_and_access.sql`、`docs/业务流程/业务流程图-V1-完整版.md`
- 已实现功能：
  - `IAM-001`：实现组织主体聚合能力：`POST /api/v1/orgs/register` 与 `GET /api/v1/iam/orgs/{id}`，返回组织状态、主体类型、司法辖区、合规等级、认证等级、黑白灰名单引用与黑名单激活状态。
  - `IAM-002`：实现 Department/User/Application/Connector/ExecutionEnvironment 最小 CRUD（create/get，Application 支持 patch），并落审计。
  - `IAM-003`：实现登录上下文镜像 `GET /api/v1/auth/me`，支持 Bearer token 解析（mock parser）与本地测试用户模式（`x-login-id`）两条链路。
  - `IAM-004`：组织注册接口支持匿名/半匿名入口字段，记录风险画像与审计事件（`iam.org.register`）。
  - 路由接入：`platform-core` 主路由已 merge `modules::iam::api::router()`。
  - OpenAPI 同步：`packages/openapi/iam.yaml` 从空文件更新为首版路径清单并对齐实现接口。
- 涉及文件：`apps/platform-core/src/modules/iam/mod.rs`、`apps/platform-core/src/modules/iam/domain.rs`、`apps/platform-core/src/modules/iam/service.rs`、`apps/platform-core/src/modules/iam/api.rs`、`apps/platform-core/src/lib.rs`、`packages/openapi/iam.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务（`APP_PORT=18080`）并手工调用：
     - `POST /api/v1/orgs/register`
     - `GET /api/v1/iam/orgs/{id}`
     - `POST /api/v1/iam/departments`
     - `POST /api/v1/iam/users`
     - `POST /api/v1/apps` + `PATCH /api/v1/apps/{id}`
     - `POST /api/v1/iam/connectors`
     - `POST /api/v1/iam/execution-environments`
     - `GET /api/v1/auth/me`（`x-login-id` 模式）
  4. 审计回查：
     - `SELECT action_name, ref_type, result_code FROM audit.audit_event WHERE action_name LIKE 'iam.%' ORDER BY event_time DESC LIMIT 12;`
- 验证结果：通过。`platform-core` 单测 `21/21` 通过；手工接口调用全部成功；审计表回查到 `iam.org.register/read`、`iam.department.create`、`iam.user.create`、`iam.app.create/patch`、`iam.connector.create`、`iam.execution_environment.create`、`iam.session.context.read`。
- 覆盖的冻结文档条目：身份与会话接口基线（注册/会话）、`010_identity_and_access.sql` 的核心对象模型（organization/department/user/application/connector/execution_environment）、主体准入流程。
- 覆盖的任务清单条目：`IAM-001`, `IAM-002`, `IAM-003`, `IAM-004`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`TODO-PROC-BIL-001` 仍保持冻结状态，本批未扩展其它 BIL 任务。

### BATCH-080（待审批）

- 状态：通过
- 当前任务编号：IAM-005, IAM-006, IAM-007, IAM-008
- 当前批次目标：在 `iam` 模块补齐成员邀请、会话/设备最小管理、应用接口闭环与应用密钥轮换/吊销能力，并同步审计、权限、OpenAPI 与最小测试。
- 前置依赖核对结果：`IAM-005~008` 统一依赖 `CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018`，均已完成并审批通过；本批无阻塞。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/身份与会话接口协议正式版.md`、`docs/数据库设计/V1/upgrade/010_identity_and_access.sql`、`docs/原始PRD/IAM 技术接入方案.md`
- 已实现功能：
  - `IAM-005`：新增邀请接口 `POST /api/v1/users/invite`、`POST /api/v1/iam/invitations`、`GET /api/v1/iam/invitations`、`POST /api/v1/iam/invitations/{id}/cancel`，落地 `iam.invitation` 表写入与取消。
  - `IAM-006`：新增会话/设备接口 `GET /api/v1/iam/sessions`、`POST /api/v1/iam/sessions/{id}/revoke`、`GET /api/v1/iam/devices`、`POST /api/v1/iam/devices/{id}/revoke`，落地最小会话管理与设备撤销。
  - `IAM-007`：保留并验证 `POST /api/v1/apps`、`PATCH /api/v1/apps/{id}`、`GET /api/v1/apps/{id}`，同步扩展返回字段 `client_secret_status`，用于应用与 API 产品绑定可见性。
  - `IAM-008`：新增应用密钥管理接口 `POST /api/v1/apps/{id}/credentials/rotate`、`POST /api/v1/apps/{id}/credentials/revoke`；落地 `client_secret_hash` 轮换/吊销与 metadata 状态记录（`active/revoked`）。
  - 审计补齐：新增 `iam.user.invite`、`iam.invitation.create/cancel`、`iam.session.revoke`、`iam.device.revoke`、`iam.app.secret.rotate/revoke` 事件写入。
  - OpenAPI 同步：`packages/openapi/iam.yaml` 已新增上述路径，确保契约与实现一致。
- 涉及文件：`apps/platform-core/src/modules/iam/domain.rs`、`apps/platform-core/src/modules/iam/api.rs`、`packages/openapi/iam.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务并手工 API 验证（`APP_PORT=18080`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`）：
     - `POST /api/v1/orgs/register`
     - `POST /api/v1/iam/invitations`
     - `POST /api/v1/apps`
     - `POST /api/v1/apps/{id}/credentials/rotate`
     - `POST /api/v1/apps/{id}/credentials/revoke`
     - `GET /api/v1/iam/sessions`
     - `GET /api/v1/iam/devices`
  4. 审计回查：`SELECT action_name, ref_type, result_code FROM audit.audit_event WHERE action_name IN ('iam.user.invite','iam.invitation.create','iam.app.secret.rotate','iam.app.secret.revoke','iam.session.revoke','iam.device.revoke') ORDER BY event_time DESC LIMIT 12;`
- 验证结果：通过。`platform-core` 单测 `23/23` 通过；手工 API 返回 `success=true`，密钥状态按 `active -> revoked` 变化；会话/设备查询成功返回数组；审计回查到 `iam.invitation.create`、`iam.app.secret.rotate`、`iam.app.secret.revoke` 记录。
- 覆盖的冻结文档条目：身份与会话接口基线中的 `invitations`、`sessions`、`devices` 路径；`010_identity_and_access.sql` 中 `iam.invitation/iam.user_session/iam.trusted_device/core.application` 对象；IAM 技术接入文档中的本地模式最小闭环要求。
- 覆盖的任务清单条目：`IAM-005`, `IAM-006`, `IAM-007`, `IAM-008`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：`TODO-PROC-BIL-001` 持续保持冻结状态；本批未扩展 BIL 线。

### BATCH-081（待审批）

- 状态：通过
- 当前任务编号：IAM-009, IAM-010, IAM-011, IAM-012
- 当前批次目标：在 IAM/Access 模块补齐角色-权限-作用域放行规则、RBAC 种子加载与统一权限校验中间件，并实现 step-up 占位与 MFA 占位接口。
- 前置依赖核对结果：`IAM-009~012` 统一依赖 `CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018`，均已完成并审批通过；本批无阻塞。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/权限设计/接口权限校验清单.md`、`docs/权限设计/权限点清单.md`、`docs/数据库设计/接口协议/身份与会话接口协议正式版.md`
- 已实现功能：
  - `IAM-009`：在 `access` 模块新增放行规则模型 `AccessRule` 与规则集 `ACCESS_RULES`，覆盖权限点、作用域、按钮与 API 放行模式；新增 IAM 接口 `GET /api/v1/iam/access/rules` 与 `POST /api/v1/iam/access/check`。
  - `IAM-010`：在 `iam/service` 新增 RBAC 种子加载机制（`OnceLock`）与角色域分层（tenant/platform/audit/developer）；`is_allowed` 统一走种子权限映射，不再硬编码散落判断；新增 `GET /api/v1/iam/rbac/seeds`。
  - `IAM-011`：实现高风险 step-up 占位链路：`POST /api/v1/iam/step-up/check` 创建挑战并统一判定冻结/赔付/证据导出/回放/权限变更；`POST /api/v1/iam/step-up/challenges/{id}/verify` 完成简化验码与状态推进。
  - `IAM-012`：实现 MFA 占位接口：`GET/POST /api/v1/iam/mfa/authenticators`、`DELETE /api/v1/iam/mfa/authenticators/{id}`；local 模式支持 `IAM_MFA_MODE=mock|disabled`，并冻结接口形态用于后续真实接入。
  - 审计补齐：新增 `iam.step_up.check`、`iam.step_up.verify`、`iam.mfa.authenticator.create`、`iam.mfa.authenticator.delete` 审计事件。
  - OpenAPI 同步：`packages/openapi/iam.yaml` 新增 access/rbac/step-up/mfa 路径并对齐实现。
- 涉及文件：`apps/platform-core/src/modules/access/mod.rs`、`apps/platform-core/src/modules/iam/service.rs`、`apps/platform-core/src/modules/iam/domain.rs`、`apps/platform-core/src/modules/iam/api.rs`、`packages/openapi/iam.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务并手工 API 验证（`APP_PORT=18080`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`IAM_MFA_MODE=mock`，`IAM_STEP_UP_MODE=mock`）：
     - `GET /api/v1/iam/access/rules`
     - `POST /api/v1/iam/access/check`
     - `POST /api/v1/iam/step-up/check`
     - `POST /api/v1/iam/step-up/challenges/{id}/verify`
     - `POST /api/v1/iam/mfa/authenticators`
     - `DELETE /api/v1/iam/mfa/authenticators/{id}`
  4. 审计回查：`SELECT action_name, ref_type, result_code FROM audit.audit_event WHERE action_name IN ('iam.step_up.check','iam.step_up.verify','iam.mfa.authenticator.create','iam.mfa.authenticator.delete') ORDER BY event_time DESC LIMIT 12;`
- 验证结果：通过。`platform-core` 单测 `27/27` 通过；手工 API 返回 `success=true`；step-up 挑战从 `challenge_required` 到 `verified`；MFA 从 `active` 到 `revoked`；审计表查询到上述四类动作记录。
- 覆盖的冻结文档条目：权限点与接口放行基线（角色/作用域/按钮/API 放行规则）、身份与会话协议中的 `step-up` 与 `mfa` 接口形态、IAM 技术接入本地 mock 占位策略。
- 覆盖的任务清单条目：`IAM-009`, `IAM-010`, `IAM-011`, `IAM-012`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批在不重建骨架前提下复用既有 IAM API 结构扩展；`TODO-PROC-BIL-001` 持续保持冻结状态。

### BATCH-082（待审批）

- 状态：通过
- 当前任务编号：IAM-013, IAM-014, IAM-015, IAM-016
- 当前批次目标：补齐企业 OIDC 连接占位接口、Fabric 身份镜像与证书吊销占位接口、party-review 联动字段、以及登录/登出/邀请/撤销/角色变更的审计闭环。
- 前置依赖核对结果：`IAM-013~016` 统一依赖 `CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018`，均已完成并审批通过；本批无阻塞。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/身份与会话接口协议正式版.md`、`docs/数据库设计/V1/upgrade/010_identity_and_access.sql`、`docs/原始PRD/IAM 技术接入方案.md`
- 已实现功能：
  - `IAM-013`：新增企业 OIDC 占位接口 `POST/GET /api/v1/iam/sso/connections`、`PATCH /api/v1/iam/sso/connections/{id}`，落地 `iam.sso_connection` 配置模型与本地占位状态流转。
  - `IAM-014`：新增 Fabric 身份镜像与证书占位接口：`GET /api/v1/iam/fabric-identities`、`POST /api/v1/iam/fabric-identities/{id}/issue`、`POST /api/v1/iam/fabric-identities/{id}/revoke`、`GET /api/v1/iam/certificates`、`POST /api/v1/iam/certificates/{id}/revoke`。
  - `IAM-015`：扩展组织聚合返回字段并实现联动写入接口 `PATCH /api/v1/iam/orgs/{id}/party-review-linkage`，覆盖 `review_status/risk_status/sellable_status/freeze_reason`。
  - `IAM-016`：补齐会话与设备审计链路中的关键动作留痕：新增 `POST /api/v1/auth/login`、`POST /api/v1/auth/logout`、`POST /api/v1/iam/users/{id}/roles`，并落审计 `iam.session.login/logout`、`iam.user.role.change`；与前批已有邀请/撤销审计形成闭环。
  - RBAC 扩展：新增 `SsoRead/SsoWrite/FabricRead/FabricWrite/SessionWrite/RoleChangeWrite` 权限并纳入种子加载映射。
  - OpenAPI 同步：`packages/openapi/iam.yaml` 补充上述全部路径，保持契约与实现一致。
- 涉及文件：`apps/platform-core/src/modules/iam/domain.rs`、`apps/platform-core/src/modules/iam/service.rs`、`apps/platform-core/src/modules/iam/api.rs`、`apps/platform-core/src/modules/access/mod.rs`、`packages/openapi/iam.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务并手工 API 验证（`APP_PORT=18080`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`）：
     - `PATCH /api/v1/iam/orgs/{id}/party-review-linkage`
     - `POST /api/v1/auth/login`
     - `POST /api/v1/auth/logout`
     - `POST /api/v1/iam/users/{id}/roles`
     - `POST /api/v1/iam/sso/connections`
     - `PATCH /api/v1/iam/sso/connections/{id}`
     - `GET /api/v1/iam/fabric-identities`
     - `GET /api/v1/iam/certificates`
  4. 审计回查：`SELECT action_name, ref_type, result_code FROM audit.audit_event WHERE action_name IN ('iam.org.party_review_linkage.patch','iam.session.login','iam.session.logout','iam.user.role.change','iam.sso.connection.create','iam.sso.connection.patch') ORDER BY event_time DESC LIMIT 20;`
- 验证结果：通过。`platform-core` 单测 `30/30` 通过；手工 API 全部返回 `success=true`；组织联动字段回写生效；审计回查命中 `iam.org.party_review_linkage.patch`、`iam.session.login/logout`、`iam.user.role.change`、`iam.sso.connection.create/patch`。
- 覆盖的冻结文档条目：身份与会话协议中的 `sso/connections`、`fabric-identities`、`certificates`、`step-up/mfa` 语义边界；`010_identity_and_access.sql` 中 `iam.sso_connection/iam.fabric_identity_binding/iam.certificate_record/iam.user_session` 对象；IAM 技术接入方案中的 local 占位实现要求。
- 覆盖的任务清单条目：`IAM-013`, `IAM-014`, `IAM-015`, `IAM-016`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批未引入 V2/V3 正式能力，仅冻结 V1 占位接口与状态口径；`TODO-PROC-BIL-001` 继续保持冻结状态。

### BATCH-083（待审批）

- 状态：通过
- 当前任务编号：IAM-017, IAM-018, IAM-019, IAM-020
- 当前批次目标：补齐 IAM 六类对象列表与详情联调接口、本地测试身份一键脚本、IAM/Party/Access 集成测试，以及 `docs/02-openapi/iam.yaml` 归档交付。
- 前置依赖核对结果：`IAM-017~020` 统一依赖 `CORE-001; CORE-004; CORE-005; CORE-006; DB-003; ENV-018`，均已完成并审批通过；本批无阻塞。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/身份与会话接口协议正式版.md`、`docs/数据库设计/V1/upgrade/010_identity_and_access.sql`、`docs/原始PRD/IAM 技术接入方案.md`
- 已实现功能：
  - `IAM-017`：补齐组织/部门/用户/应用/连接器/执行环境的列表与详情联调用接口：
    - `GET /api/v1/iam/orgs`、`GET /api/v1/iam/orgs/{id}`
    - `GET/POST /api/v1/iam/departments`、`GET /api/v1/iam/departments/{id}`
    - `GET/POST /api/v1/iam/users`、`GET /api/v1/iam/users/{id}`
    - `GET/POST /api/v1/apps`、`GET /api/v1/apps/{id}`
    - `GET/POST /api/v1/iam/connectors`、`GET /api/v1/iam/connectors/{id}`
    - `GET/POST /api/v1/iam/execution-environments`、`GET /api/v1/iam/execution-environments/{id}`
    并支持常用过滤参数（`org_id/status/department_id/connector_id`）。
  - `IAM-018`：新增本地测试身份脚本 `scripts/seed-local-iam-test-identities.sh`，一键生成卖方管理员、买方管理员、运营管理员、审计员、开发者账号及角色绑定；脚本具备组织/部门自补齐能力与幂等 upsert。
  - `IAM-019`：新增 live 集成测试 `apps/platform-core/tests/iam_party_access_integration.rs`（`#[ignore]`），覆盖组织注册、邀请成员、创建应用、权限拒绝、会话撤销、设备撤销，并扩展验证列表/详情接口可用性。
  - `IAM-020`：生成并落盘 `docs/02-openapi/iam.yaml` 第一版（由 `packages/openapi/iam.yaml` 同步），并在 `docs/02-openapi/README.md` 增加归档说明。
- 涉及文件：`apps/platform-core/src/modules/iam/domain.rs`、`apps/platform-core/src/modules/iam/api.rs`、`apps/platform-core/tests/iam_party_access_integration.rs`、`scripts/seed-local-iam-test-identities.sh`、`packages/openapi/iam.yaml`、`docs/02-openapi/iam.yaml`、`docs/02-openapi/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab ./scripts/seed-local-iam-test-identities.sh`
  4. 启动服务后执行 live 集成测试：
     `IAM_IT_BASE_URL=http://127.0.0.1:18080 IAM_IT_DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core --test iam_party_access_integration -- --ignored --nocapture`
- 验证结果：通过。单测 `30/30` 通过；本地身份脚本成功生成 5 个目标账号；live 集成测试 `1 passed`，覆盖 IAM-019 要求的关键链路；`docs/02-openapi/iam.yaml` 已存在并与 `packages/openapi/iam.yaml` 同步。
- 覆盖的冻结文档条目：身份与会话接口协议（列表/详情、邀请、会话、设备、应用绑定）、`010_identity_and_access.sql`（core/iam/authz 对象）、IAM 技术接入方案（local 模式测试身份与联调能力）。
- 覆盖的任务清单条目：`IAM-017`, `IAM-018`, `IAM-019`, `IAM-020`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：通过
- 备注：本批保持 V1 范围，不引入 V2/V3 正式实现；`TODO-PROC-BIL-001` 继续保持冻结状态。

### BATCH-084（待审批）

- 状态：通过
- 当前任务编号：IAM-002, IAM-003, IAM-011, IAM-020
- 当前批次目标：修复 IAM 阶段审查发现的 5 个缺口：事务一致性、Keycloak token 解析、step-up 契约路径、IAM 仓储层边界、人工审批记录补录。
- 前置依赖核对结果：`IAM-001~IAM-020` 均已实现且已口头审批通过；本批为 IAM 阶段内部修复批，无新增跨阶段依赖阻塞。
- 已实现功能：
  1. 事务一致性修复：IAM 写接口统一改为“业务写入 + 审计写入同事务提交”，避免业务已落库但审计失败导致接口失败的半成功状态。
  2. IAM-002 仓储层补齐：新增 `PostgresIamRepository`，并将 Department/User/Application/Connector/ExecutionEnvironment CRUD 迁移到仓储层复用。
  3. IAM-003 token 解析补齐：在 `auth` crate 新增 `KeycloakClaimsJwtParser`（JWT payload claims 解析），`/api/v1/auth/me` 默认采用 `keycloak_claims` 解析，保留 `mock` 回退。
  4. IAM-011/IAM-020 契约修复：补齐 `POST /api/v1/iam/step-up/challenges`，保留 `/api/v1/iam/step-up/check` 兼容并标记 deprecated；OpenAPI 双份文档同步。
  5. 流程修复：在 `V1-Core-人工审批记录.md` 补录 `BATCH-079~083` 的结构化审批条目。
- 涉及文件：
  - `apps/platform-core/src/modules/iam/api.rs`
  - `apps/platform-core/src/modules/iam/repository.rs`（新增）
  - `apps/platform-core/src/modules/iam/mod.rs`
  - `apps/platform-core/crates/auth/src/lib.rs`
  - `apps/platform-core/crates/auth/Cargo.toml`
  - `packages/openapi/iam.yaml`
  - `docs/02-openapi/iam.yaml`
  - `docs/开发任务/V1-Core-人工审批记录.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
  - `docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p auth`
  3. `cargo test -p platform-core`
  4. `cmp -s packages/openapi/iam.yaml docs/02-openapi/iam.yaml`
- 验证结果：通过。`auth` 测试 `4/4` 通过（新增 keycloak claims 解析单测）；`platform-core` 测试 `30/30` 通过；IAM live 集成测试保持 `ignored`（需运行服务）；OpenAPI 两份文件一致。
- 覆盖的冻结文档条目：
  - `docs/数据库设计/接口协议/身份与会话接口协议正式版.md`（step-up 路径与 token 会话语义）
  - `docs/开发任务/v1-core-开发任务清单.csv` 中 `IAM-002/003/011/020`
  - `docs/开发任务/Agent-开发与半人工审核流程.md`（审批记录与批次留痕要求）
- 覆盖的任务清单条目：`IAM-002`, `IAM-003`, `IAM-011`, `IAM-020`
- 未覆盖项：无（本批目标内）
- 新增 TODO / 预留项：无新增未完成项；已关闭本批对应 5 个缺口追踪项（见 TODO 清单 `TODO-IAM-002-REPO-001` 等）。
- 待人工审批结论：通过

### BATCH-085

- 状态：通过
- 当前任务编号：CAT-001
- 当前批次目标：实现 Catalog 领域基础模型与仓储（DataResource、AssetVersion、DataProduct、ProductSKU），为 `CAT-002+` 接口任务提供持久化与结构基线。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-084` 已通过，满足继续执行条件。
- 预计涉及文件：`apps/platform-core/src/modules/catalog/mod.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：
  1. 在 `catalog` 模块新增基础领域模型：`CreateDataResourceRequest/DataResourceView`、`CreateAssetVersionRequest/AssetVersionView`、`CreateDataProductRequest/DataProductView`、`CreateProductSkuRequest/ProductSkuView`。
  2. 冻结并实现标准 SKU 真值集合 `FILE_STD/FILE_SUB/SHARE_RO/API_SUB/API_PPU/QRY_LITE/SBX_STD/RPT_STD` 及校验函数 `is_standard_sku_type`。
  3. 新增 `PostgresCatalogRepository`，实现 DataResource/AssetVersion/DataProduct/ProductSKU 的创建与读取仓储方法（含 SKU 列表查询）。
  4. `packages/openapi/catalog.yaml` 新增基础 schema（`DataResource/AssetVersion/DataProduct/ProductSku`），用于后续 `CAT-002+` 接口复用。
- 涉及文件：`apps/platform-core/src/modules/catalog/mod.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 对照 `CAT-001 technical_reference` 逐项核对模型与仓储字段语义（DataAsset/DataAssetVersion/Product/ProductSKU）。
  4. 本地数据库写入回滚验证（`datab-postgres:5432`）：在单事务中依次插入 `core.organization -> catalog.data_asset -> catalog.asset_version -> catalog.product -> catalog.product_sku`，读取返回 ID 后 `ROLLBACK`，并复查测试商品行数为 0。
- 验证结果：通过。`cargo test -p platform-core` 结果 `31 passed, 0 failed, 1 ignored`；新增用例 `modules::catalog::domain::tests::standard_sku_truth_list_matches_v1_frozen_set` 通过；数据库事务插入链路成功并已回滚，复查 `catalog.product` 测试数据残留为 `0`。
- 覆盖的冻结文档条目：`docs/领域模型/全量领域模型与对象关系说明.md`（4.2 目录与商品聚合）、`docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5.1/5.2 商品与 SKU）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2 商品创建、模板绑定与上架流程）。
- 覆盖的任务清单条目：`CAT-001`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增代码 TODO；`TODO-PROC-BIL-001` 状态调整为 `accepted`（已获人工批准继续后续阶段，但保留历史偏移追溯并要求进入 BIL 阶段执行一致性复核）。
- 待人工审批结论：通过
- 备注：本批只做 `CAT-001`，未推进 `CAT-002+` 接口实现。

### BATCH-086

- 状态：通过
- 当前任务编号：CAT-002
- 当前批次目标：实现 `POST /api/v1/products` 与 `PATCH /api/v1/products/{id}`，支持商品草稿创建与编辑，补齐权限校验、审计、错误码与最小测试。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-085` 已审批通过，允许执行。
- 预计涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/service.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/mod.rs`、`apps/platform-core/src/lib.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：
  1. 新增 `catalog` API 路由并接入主应用：`POST /api/v1/products`、`PATCH /api/v1/products/{id}`。
  2. 新增 `CatalogPermission::ProductDraftWrite` 与角色矩阵校验，拦截未授权角色。
  3. 创建/编辑商品草稿均采用“业务写入 + 审计写入同事务提交”。
  4. 新增 `PatchDataProductRequest` 与仓储方法 `patch_data_product`，仅允许 `status='draft'` 商品编辑。
  5. 新增最小 API 拒绝测试（无权限创建/编辑返回 `403`）与权限矩阵测试。
  6. 更新 `packages/openapi/catalog.yaml`：补齐 `POST/PATCH` 路径与请求 schema，保持实现与契约一致。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/service.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/mod.rs`、`apps/platform-core/src/lib.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调验证（`APP_PORT=18081`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`）：
     - 准备最小前置数据：`core.organization`、`catalog.data_asset`、`catalog.asset_version`
     - 调用 `POST /api/v1/products` 创建草稿
     - 调用 `PATCH /api/v1/products/{id}` 编辑草稿
     - 查询 `audit.audit_event` 验证 `catalog.product.create` / `catalog.product.patch`
     - 清理测试数据（product/asset_version/data_asset/organization）
- 验证结果：通过。`cargo test -p platform-core` 结果 `34 passed, 0 failed, 1 ignored`；API 创建与编辑均返回 `success=true`；审计查询命中 `catalog.product.create` 与 `catalog.product.patch` 两条记录；测试数据已清理。
- 覆盖的冻结文档条目：`docs/领域模型/全量领域模型与对象关系说明.md`（4.2 目录与商品聚合）、`docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5.1 商品接口）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2 商品创建、模板绑定与上架流程）。
- 覆盖的任务清单条目：`CAT-002`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved`；继续保持 `TODO-PROC-BIL-001` 追溯约束。
- 待人工审批结论：通过
- 备注：首次联调使用了回滚批次的历史临时 ID 导致一次 `OPS_INTERNAL`，已更正为真实前置测试数据后通过全链路验证并完成清理。

### BATCH-087

- 状态：通过
- 当前任务编号：CAT-003
- 当前批次目标：实现 `POST /api/v1/products/{id}/skus` 与 `PATCH /api/v1/skus/{id}`，并完成标准 SKU 真值、trade_mode 合法性、模板兼容性校验。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-086` 已审批通过，允许执行。
- 预计涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/service.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：
  1. 实现 `POST /api/v1/products/{id}/skus` 与 `PATCH /api/v1/skus/{id}`，接入 `platform-core` 主路由。
  2. 落地标准 SKU 真值校验：非 `FILE_STD/FILE_SUB/SHARE_RO/API_SUB/API_PPU/QRY_LITE/SBX_STD/RPT_STD` 直接拒绝。
  3. 落地 trade_mode 合法性与 SKU 兼容性校验，冻结映射：`FILE_STD->snapshot_sale`、`FILE_SUB->revision_subscription`、`SHARE_RO->share_grant`、`API_SUB->api_subscription`、`API_PPU->api_pay_per_use`、`QRY_LITE->template_query`、`SBX_STD->sandbox_workspace`、`RPT_STD->report_delivery`。
  4. 落地模板兼容性校验：若请求携带 `template_id`，校验 `contract.template_definition.applicable_sku_types` 含当前 `sku_type`。
  5. SKU 创建/编辑采用“业务写入 + 审计写入同事务提交”，并补充审计动作：`catalog.sku.create`、`catalog.sku.patch`。
  6. 更新 OpenAPI：新增 `/api/v1/products/{id}/skus`、`/api/v1/skus/{id}` 及对应请求 schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/service.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18082`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`）：
     - 准备前置数据：`organization -> data_asset -> asset_version -> product(draft) -> template_definition(active, applicable_sku_types=['FILE_STD'])`
     - 调用 `POST /api/v1/products/{id}/skus` 创建 SKU
     - 调用 `PATCH /api/v1/skus/{id}` 编辑 SKU
     - 查询 `audit.audit_event` 校验 `catalog.sku.create/patch`
     - 清理测试数据（product/template/asset_version/data_asset/organization）
- 验证结果：通过。`cargo test -p platform-core` 结果 `38 passed, 0 failed, 1 ignored`；SKU 创建/编辑接口均返回 `success=true`；审计命中 `catalog.sku.create` 与 `catalog.sku.patch`；测试数据已清理。
- 覆盖的冻结文档条目：`docs/领域模型/全量领域模型与对象关系说明.md`（4.2 目录与商品聚合）、`docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5.2 SKU 接口）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2 商品创建、模板绑定与上架流程）。
- 覆盖的任务清单条目：`CAT-003`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：首次联调因服务未就绪导致连接失败，改为 health-check 等待后复测通过。

### BATCH-088

- 状态：通过
- 当前任务编号：CAT-004
- 当前批次目标：实现 `POST /api/v1/assets/{assetId}/raw-ingest-batches`，落地原始接入批次创建能力，并补齐权限校验、事务审计、错误码、OpenAPI 与最小验证。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-087` 已获人工审批通过，允许执行。
- 已阅读证据（文件 + 本批关注要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：`CAT-004` 描述、`depends_on`、DoD 与 technical_reference 锚点。
  2. `docs/开发任务/v1-core-开发任务清单.md`：`CAT-004` 可读说明与 `CAT-005/006` 顺序边界。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：先登记计划中、后编码、验证、待审批的强制顺序。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：严格冻结口径、不得越阶段扩展。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：沿用批次记录格式与审批门禁。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：已补录 `BATCH-087` 审批通过，可进入下一批。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：保持 V1 范围，不混入 V2/V3 正式能力。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：`catalog` 领域接口归属与边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结接口 `POST /api/v1/assets/{assetId}/raw-ingest-batches`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批先做审计事件，不新增跨服务业务 Topic。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `CAT_VALIDATION_FAILED / IAM_UNAUTHORIZED / OPS_INTERNAL` 映射。
  13. `docs/开发准备/测试用例矩阵正式版.md`：保留单测 + 手工 API + 审计回查最小闭环。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：继续在 `platform-core/modules/catalog` 内增量实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调 DB 走 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用 `DATABASE_URL` 配置注入方式。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 为业务主状态权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：维持模块化单体内聚实现方式。
- technical_reference 约束映射：
  - `docs/原始PRD/数据原样处理与产品化加工流程设计.md:L189`：原始接入登记必须产生 `raw_ingest_batch`，并包含来源与权利声明语义。
  - `docs/业务流程/业务流程图-V1-完整版.md:L157`：4.2A 主流程要求“记录批次、对象、来源、权利链、hash”，本批先完成批次入口。
  - `docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1`：`catalog.raw_ingest_batch` 字段与默认状态（`draft`）作为持久化真值约束。
- 已实现功能：
  1. 新增 `RawIngestBatch` 领域模型：`CreateRawIngestBatchRequest/RawIngestBatchView`。
  2. 新增仓储方法 `PostgresCatalogRepository::create_raw_ingest_batch`，按 `063_raw_processing_pipeline.sql` 写入 `catalog.raw_ingest_batch`。
  3. 新增接口 `POST /api/v1/assets/{assetId}/raw-ingest-batches`，含路径/请求校验、权限校验、事务审计与成功响应。
  4. 新增权限 `CatalogPermission::RawIngestWrite` 及角色矩阵测试。
  5. 更新 `packages/openapi/catalog.yaml`：新增 raw-ingest-batches 路径与 `CreateRawIngestBatchRequest/RawIngestBatch` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/service.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-人工审批记录.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18083`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`）：
     - 预置数据：`core.organization` + `catalog.data_asset`
     - 调用 `POST /api/v1/assets/{assetId}/raw-ingest-batches`
     - 查询 `catalog.raw_ingest_batch` 与 `audit.audit_event`
     - 清理测试数据（`raw_ingest_batch/data_asset/organization`）
  4. 数据库残留核对：验证 `raw_ingest_batch/data_asset/organization` 均为 `0`；审计表因 append-only 保留 1 条测试请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `40 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且批次状态为 `draft`；`audit.audit_event` 命中 `catalog.raw_ingest_batch.create|raw_ingest_batch|success`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据原样处理与产品化加工流程设计.md`（4.2 原始接入登记输出）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2A 原始接入区）、`docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql`（`catalog.raw_ingest_batch` 字段与默认状态）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（5.5 raw ingest 接口项）。
- 覆盖的任务清单条目：`CAT-004`
- 未覆盖项：`CAT-004` 描述中的“清单维护接口”细项在冻结清单中对应 `CAT-005` (`POST /api/v1/raw-ingest-batches/{id}/manifests`)，本批未越任务实现。
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：联调首轮携带不存在的 `x-user-id` 触发外键约束失败，已调整为不传 `x-user-id` 后复测通过。

### BATCH-089

- 状态：通过
- 当前任务编号：CAT-005
- 当前批次目标：实现 `POST /api/v1/raw-ingest-batches/{id}/manifests`，支持原始对象 URI、hash、格式、大小、归属记录，并补齐权限、事务审计、OpenAPI 与最小验证。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-088` 已获人工审批通过，允许执行。
- 已阅读证据（文件 + 本批关注要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：`CAT-005` 描述、DoD、acceptance 与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：`CAT-005` 在 `CAT-004` 后的顺序边界。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：计划中 -> 编码 -> 验证 -> 待审批固定步骤。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：冻结范围与不可越阶段约束。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：批次记录模板沿用。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：`TODO-PROC-BIL-001` 追溯保持。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：`BATCH-088` 已补录通过。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：保持 V1 范围与对象模型边界。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：`catalog` 接口归属。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结接口 `POST /api/v1/raw-ingest-batches/{id}/manifests`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批仍以审计事件为主，不新增业务 topic。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `CAT_VALIDATION_FAILED / IAM_UNAUTHORIZED / OPS_INTERNAL`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：单测 + 手工 API + 审计回查最小闭环。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：在 `platform-core/modules/catalog` 内增量实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用 `DATABASE_URL`。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 业务真值权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：模块化单体内聚实现。
- technical_reference 约束映射：
  - `docs/原始PRD/数据原样处理与产品化加工流程设计.md:L189`：原始接入登记输出应覆盖对象清单及对象 hash。
  - `docs/业务流程/业务流程图-V1-完整版.md:L157`：4.2A 要求记录对象与来源信息，作为后续格式识别输入。
  - `docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1`：`catalog.raw_object_manifest` 字段、默认状态与外键约束。
- 已实现功能：
  1. 新增原始对象清单模型：`CreateRawObjectManifestRequest`、`RawObjectManifestView`。
  2. 新增仓储方法：`get_raw_ingest_batch` 与 `create_raw_object_manifest`。
  3. 新增接口：`POST /api/v1/raw-ingest-batches/{id}/manifests`，包含路径/请求一致性校验、批次存在性校验、事务审计与成功响应。
  4. 新增权限拒绝测试：无权限角色访问清单创建接口返回 `403`。
  5. 更新 OpenAPI：新增 manifests 路径与 `CreateRawObjectManifestRequest/RawObjectManifest` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-人工审批记录.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 本地栈核验：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. 端到端联调（`APP_PORT=18084`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.raw_ingest_batch`
     - 调用 `POST /api/v1/raw-ingest-batches/{id}/manifests`
     - 回查 `catalog.raw_object_manifest` 与 `audit.audit_event`
     - 清理测试数据（`raw_object_manifest/raw_ingest_batch/data_asset/organization`）
  5. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `41 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且 `status=registered`；审计命中 `catalog.raw_object_manifest.create|raw_object_manifest|success`；业务表清理后残留 `0|0|0|0`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据原样处理与产品化加工流程设计.md`（4.2 原始接入登记输出）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2A 原始接入区对象记录）、`docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql`（`catalog.raw_object_manifest` 字段与默认状态）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（5.5 manifests 接口项）。
- 覆盖的任务清单条目：`CAT-005`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：联调时若不显式设置 `KAFKA_BROKERS=127.0.0.1:9094` 会触发 startup self-check 失败；已按本地栈口径修正并完成验证。

### BATCH-090

- 状态：通过
- 当前任务编号：CAT-006
- 当前批次目标：实现 `POST /api/v1/raw-object-manifests/{id}/detect-format`，保存格式识别结果并补齐权限、审计、OpenAPI 与最小验证。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-089` 已获人工审批通过，允许执行。
- 已阅读证据（文件 + 本批关注要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：`CAT-006` 描述、DoD、acceptance 与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：`CAT-006` 顺序与 `CAT-007` 边界。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：计划中 -> 编码 -> 验证 -> 待审批固定流程。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：冻结范围与不可越阶段。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：批次记录格式沿用。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：`TODO-PROC-BIL-001` 追溯保持。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：`BATCH-089` 已补录通过。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：保持 V1 范围。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：`catalog` 归属边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结接口 `POST /api/v1/raw-object-manifests/{id}/detect-format`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批继续以内审计为主，不新增业务 topic。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `CAT_VALIDATION_FAILED / IAM_UNAUTHORIZED / OPS_INTERNAL`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：单测 + 手工 API + 审计回查闭环。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按功能逻辑拆分实现，避免单文件过大。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用 `DATABASE_URL` / `KAFKA_BROKERS`。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 为业务真值权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：模块化单体内聚。
- technical_reference 约束映射：
  - `docs/原始PRD/数据原样处理与产品化加工流程设计.md:L189`：格式识别阶段必须产出对象族与置信度。
  - `docs/业务流程/业务流程图-V1-完整版.md:L157`：4.2A 分类识别区要求形成 `FormatDetectionResult`。
  - `docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1`：`catalog.format_detection_result` 字段、索引与默认状态约束。
- 已实现功能：
  1. 新增格式识别模型：`CreateFormatDetectionRequest`、`FormatDetectionResultView`。
  2. 新增仓储方法：`get_raw_object_manifest` 与 `create_format_detection_result`。
  3. 新增接口：`POST /api/v1/raw-object-manifests/{id}/detect-format`，包含路径/请求一致性校验、对象存在性校验、置信度区间校验、事务审计与成功响应。
  4. 新增权限拒绝测试：无权限角色访问 detect-format 接口返回 `403`。
  5. 更新 OpenAPI：新增 detect-format 路径与 `CreateFormatDetectionRequest/FormatDetectionResult` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-人工审批记录.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18085`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.raw_ingest_batch` + `catalog.raw_object_manifest`
     - 调用 `POST /api/v1/raw-object-manifests/{id}/detect-format`
     - 回查 `catalog.format_detection_result` 与 `audit.audit_event`
     - 清理测试数据（`format_detection_result/raw_object_manifest/raw_ingest_batch/data_asset/organization`）
  4. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `49 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且识别状态 `detected`；审计命中 `catalog.format_detection_result.create|format_detection_result|success`；业务表清理后残留 `0|0|0|0|0`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据原样处理与产品化加工流程设计.md`（4.3 格式识别输出项）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2A 分类识别区）、`docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql`（`catalog.format_detection_result` 字段与默认状态）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（5.5 detect-format 接口项）。
- 覆盖的任务清单条目：`CAT-006`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：沿用你当前的测试拆分结构（`modules/*/tests/mod.rs`），本批继续按功能域增量扩展，避免把全部逻辑继续堆积在单点文件。

### BATCH-091

- 状态：通过
- 当前任务编号：CAT-007
- 当前批次目标：实现 `POST /api/v1/raw-object-manifests/{id}/extraction-jobs`，先落地任务记录与状态机占位，不强耦合真实计算引擎。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-090` 已获人工审批通过，允许执行。
- 已阅读证据（文件 + 本批关注要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：`CAT-007` 描述、DoD、acceptance 与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：`CAT-007` 顺序与 `CAT-008` 边界。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：计划中 -> 编码 -> 验证 -> 待审批固定流程。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：冻结范围与不可越阶段。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：批次记录格式沿用。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：`TODO-PROC-BIL-001` 追溯保持。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：`BATCH-090` 已补录通过。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：保持 V1 范围。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：`catalog` 归属边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结接口 `POST /api/v1/raw-object-manifests/{id}/extraction-jobs`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批继续以内审计为主，不新增业务 topic。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `CAT_VALIDATION_FAILED / IAM_UNAUTHORIZED / OPS_INTERNAL`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：单测 + 手工 API + 审计回查闭环。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按功能逻辑拆分实现，避免单文件过大。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用 `DATABASE_URL` / `KAFKA_BROKERS`。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 为业务真值权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：模块化单体内聚。
- technical_reference 约束映射：
  - `docs/原始PRD/数据原样处理与产品化加工流程设计.md:L189`：抽取/标准化阶段以任务记录与状态推进为主。
  - `docs/业务流程/业务流程图-V1-完整版.md:L157`：4.2A 类内标准化区与加工处理区需要任务化承载。
  - `docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1`：`catalog.extraction_job` 字段、默认状态与索引约束。
- 已实现功能：
  1. 新增抽取任务模型：`CreateExtractionJobRequest`、`ExtractionJobView`。
  2. 新增仓储方法：`create_extraction_job`。
  3. 新增接口：`POST /api/v1/raw-object-manifests/{id}/extraction-jobs`，包含路径/请求一致性校验、对象存在性校验、事务审计与成功响应。
  4. 新增权限拒绝测试：无权限角色访问 extraction-jobs 接口返回 `403`。
  5. 更新 OpenAPI：新增 extraction-jobs 路径与 `CreateExtractionJobRequest/ExtractionJob` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-人工审批记录.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18086`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.raw_ingest_batch` + `catalog.raw_object_manifest`
     - 调用 `POST /api/v1/raw-object-manifests/{id}/extraction-jobs`
     - 回查 `catalog.extraction_job` 与 `audit.audit_event`
     - 清理测试数据（`extraction_job/raw_object_manifest/raw_ingest_batch/data_asset/organization`）
  4. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `50 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且任务状态 `draft`；审计命中 `catalog.extraction_job.create|extraction_job|success`；业务表清理后残留 `0|0|0|0|0`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据原样处理与产品化加工流程设计.md`（4.8/4.9 抽取与加工任务化）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2A 类内标准化与加工处理区）、`docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql`（`catalog.extraction_job` 字段与默认状态）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（5.5 extraction-jobs 接口项）。
- 覆盖的任务清单条目：`CAT-007`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：本批继续沿用你调整后的测试拆分结构，避免将新增逻辑与测试回堆到单一超大文件。

### BATCH-092

- 状态：计划中
- 当前任务编号：CAT-008
- 当前批次目标：实现 `POST /api/v1/assets/{versionId}/preview-artifacts`，支持样例文件、schema 预览、预览掩码策略，并补齐权限、审计、OpenAPI 与最小验证。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-091` 已获人工审批通过，允许执行。
- 已阅读证据（文件 + 本批关注要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：`CAT-008` 描述、DoD、acceptance 与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：`CAT-008` 顺序与 `CAT-009` 边界。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：计划中 -> 编码 -> 验证 -> 待审批固定流程。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：冻结范围与不可越阶段。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：沿用批次记录格式并先记“计划中”。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：`TODO-PROC-BIL-001` 追溯约束保持。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：`BATCH-091` 已补录通过。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：保持 V1 范围，不引入 V2/V3 正式能力。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：`catalog` 子域实现边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结接口 `POST /api/v1/assets/{versionId}/preview-artifacts`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批维持审计闭环，不新增业务 topic。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `CAT_VALIDATION_FAILED / IAM_UNAUTHORIZED / OPS_INTERNAL`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：执行单测 + 手工 API + 审计回查闭环。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：沿用功能分层，避免实现/测试堆积到单文件。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用 `DATABASE_URL`、`KAFKA_BROKERS`。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 作为业务主状态权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：模块化单体内聚实现。
- technical_reference 约束映射：
  - `docs/原始PRD/数据原样处理与产品化加工流程设计.md:L189`：产品包装阶段需产出 `sample / preview` 及 `schema` 相关工件。
  - `docs/业务流程/业务流程图-V1-完整版.md:L157`：4.2A 产品包装区明确“生成 preview_artifact / sample / manifest”。
  - `docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql:L1`：`catalog.preview_artifact` 字段、默认状态、索引与触发器约束。
- 预计涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`

### BATCH-092（待审批）

- 状态：通过
- 当前任务编号：CAT-008
- 当前批次目标：实现 `POST /api/v1/assets/{versionId}/preview-artifacts`，支持样例文件、schema 预览、预览掩码策略。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-091` 已获人工审批通过。
- 已实现功能：
  1. 新增预览工件模型：`CreatePreviewArtifactRequest`、`PreviewArtifactView`。
  2. 新增仓储方法：`create_preview_artifact`，写入 `catalog.preview_artifact`。
  3. 新增接口：`POST /api/v1/assets/{versionId}/preview-artifacts`，包含路径/请求一致性校验、资产版本存在性校验、可选原始清单存在性校验、事务审计。
  4. 新增权限拒绝测试：无权限角色访问 preview-artifacts 接口返回 `403`。
  5. 更新 OpenAPI：新增 preview-artifacts 路径与 `CreatePreviewArtifactRequest/PreviewArtifact` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-人工审批记录.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 本地数据库连通性核验：`psql postgres://datab@127.0.0.1:5432/datab -c 'select 1'`
  4. 端到端联调（`APP_PORT=18087`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.asset_version` + `catalog.raw_ingest_batch` + `catalog.raw_object_manifest`
     - 调用 `POST /api/v1/assets/{versionId}/preview-artifacts`
     - 回查 `catalog.preview_artifact` 与 `audit.audit_event`
     - 清理测试数据（`preview_artifact/raw_object_manifest/raw_ingest_batch/asset_version/data_asset/organization`）
  5. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `51 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且 `status=active`；审计命中 `catalog.preview_artifact.create|preview_artifact|success|req-cat008-preview-001`；业务表清理后残留 `0|0|0|0|0|0`，审计残留 `1`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据原样处理与产品化加工流程设计.md`（产品包装区 `sample/preview`）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2A 生成 `preview_artifact / sample / manifest`）、`docs/数据库设计/V1/upgrade/063_raw_processing_pipeline.sql`（`catalog.preview_artifact` 字段与默认状态）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（5.5 preview-artifacts 接口项）。
- 覆盖的任务清单条目：`CAT-008`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：联调首轮因测试插入的 `asset_version.allowed_region` 为 `NULL` 触发存在性查询反序列化错误；已按表约束改为 `ARRAY[]::text[]` 后复测通过。

### BATCH-093（待审批）

- 状态：通过
- 当前任务编号：CAT-009
- 当前批次目标：实现 `PUT /api/v1/products/{id}/metadata-profile`，覆盖十大元信息域最小结构。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-092` 已获人工审批通过。
- 已实现功能：
  1. 新增元信息档案模型：`PutProductMetadataProfileRequest`、`ProductMetadataProfileView`。
  2. 新增仓储方法：`upsert_product_metadata_profile`，按 `(product_id, metadata_version_no)` 执行 upsert，并持久化十大元信息域 JSON 结构。
  3. 新增接口：`PUT /api/v1/products/{id}/metadata-profile`，包含路径/请求一致性校验、仅支持 `metadata_version_no=1` 的 V1 约束、商品存在性校验、草稿态校验、事务审计。
  4. 新增 JSON 归一化：未提供或非对象型 JSON 字段统一落 `{}`，避免出现 `null` 结构漂移。
  5. 新增权限拒绝测试：无权限角色访问 metadata-profile 接口返回 `403`。
  6. 更新 OpenAPI：新增 metadata-profile 路径与 `PutProductMetadataProfileRequest/ProductMetadataProfile` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-人工审批记录.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18088`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.asset_version` + `catalog.product`
     - 调用 `PUT /api/v1/products/{id}/metadata-profile`
     - 回查 `catalog.product_metadata_profile` 与 `audit.audit_event`
     - 清理测试数据（`product_metadata_profile/product/asset_version/data_asset/organization`）
  4. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `52 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且十大域 JSON 已回写；审计命中 `catalog.product_metadata_profile.upsert|product_metadata_profile|success|req-cat009-meta-002`；业务表清理后残留 `0|0|0|0|0`，审计残留 `1`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据商品元信息与数据契约设计.md`（3.1 十大元信息域 + 3.2 契约独立建模）、`docs/数据库设计/V1/upgrade/062_data_product_metadata_contract.sql`（`catalog.product_metadata_profile` 十大 JSON 列与唯一键）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（`PUT /api/v1/products/{id}/metadata-profile` 冻结接口）。
- 覆盖的任务清单条目：`CAT-009`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：联调复核中确认 `metadata` 字段默认归一化为 `{}`，避免 `null` 与最小结构约束冲突。

### BATCH-094（待审批）

- 状态：通过
- 当前任务编号：CAT-010
- 当前批次目标：实现 `POST /api/v1/assets/{versionId}/field-definitions`。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-093` 已获人工审批通过。
- 已实现功能：
  1. 新增字段结构模型：`CreateAssetFieldDefinitionRequest`、`AssetFieldDefinitionView`。
  2. 新增仓储方法：`create_asset_field_definition`，写入 `catalog.asset_field_definition`。
  3. 新增接口：`POST /api/v1/assets/{versionId}/field-definitions`，包含路径/请求一致性校验、`field_name/field_path/field_type` 必填校验、资产版本存在性校验、事务审计。
  4. 新增 JSON 归一化：`enum_values_json` 非数组值归一化为 `[]`。
  5. 新增权限拒绝测试：无权限角色访问 field-definitions 接口返回 `403`。
  6. 更新 OpenAPI：新增 field-definitions 路径与 `CreateAssetFieldDefinitionRequest/AssetFieldDefinition` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-人工审批记录.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18089`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.asset_version`
     - 调用 `POST /api/v1/assets/{versionId}/field-definitions`
     - 回查 `catalog.asset_field_definition` 与 `audit.audit_event`
     - 清理测试数据（`asset_field_definition/asset_version/data_asset/organization`）
  4. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `53 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且字段定义结构回写正确；审计命中 `catalog.asset_field_definition.create|asset_field_definition|success|req-cat010-field-001`；业务表清理后残留 `0|0|0|0`，审计残留 `1`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据商品元信息与数据契约设计.md`（4.3 结构元信息对象化）、`docs/数据库设计/V1/upgrade/062_data_product_metadata_contract.sql`（`catalog.asset_field_definition` 字段与唯一约束）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（`POST /api/v1/assets/{versionId}/field-definitions` 冻结接口）。
- 覆盖的任务清单条目：`CAT-010`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：沿用按功能拆分实现策略，接口、模型、仓储和测试分别维护，避免单文件过大。

### BATCH-095（待审批）

- 状态：通过
- 当前任务编号：CAT-011
- 当前批次目标：实现 `POST /api/v1/assets/{versionId}/quality-reports`，保存指标、采样方式、报告 URI/hash。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-094` 已获人工审批通过。
- 已实现功能：
  1. 新增质量报告模型：`CreateAssetQualityReportRequest`、`AssetQualityReportView`。
  2. 新增仓储方法：`create_asset_quality_report`，写入 `catalog.asset_quality_report`。
  3. 新增接口：`POST /api/v1/assets/{versionId}/quality-reports`，包含路径/请求一致性校验、`report_no > 0` 校验、质量率字段 `[0,1]` 校验、资产版本存在性校验、事务审计。
  4. 新增 JSON 归一化：`coverage_range_json/freshness_json/metrics_json/metadata` 非对象值归一化为 `{}`。
  5. 新增权限拒绝测试：无权限角色访问 quality-reports 接口返回 `403`。
  6. 更新 OpenAPI：新增 quality-reports 路径与 `CreateAssetQualityReportRequest/AssetQualityReport` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-人工审批记录.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18090`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.asset_version`
     - 调用 `POST /api/v1/assets/{versionId}/quality-reports`
     - 回查 `catalog.asset_quality_report` 与 `audit.audit_event`
     - 清理测试数据（`asset_quality_report/asset_version/data_asset/organization`）
  4. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `54 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且质量报告结构回写正确；审计命中 `catalog.asset_quality_report.create|asset_quality_report|success|req-cat011-quality-002`；业务表清理后残留 `0|0|0|0`，审计残留 `1`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据商品元信息与数据契约设计.md`（4.4 质量元信息对象化）、`docs/数据库设计/V1/upgrade/062_data_product_metadata_contract.sql`（`catalog.asset_quality_report` 字段与唯一约束）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（`POST /api/v1/assets/{versionId}/quality-reports` 冻结接口）。
- 覆盖的任务清单条目：`CAT-011`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：联调首轮报错 `database operation failed: error serializing parameter 9`（`assessed_at` 参数类型）；已将 SQL 显式改为 `$10::text::timestamptz` 后复测通过。

### BATCH-096（待审批）

- 状态：通过
- 当前任务编号：CAT-012
- 当前批次目标：实现 `POST /api/v1/assets/{versionId}/processing-jobs`，记录输入来源、责任主体、处理摘要。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-095` 已获人工审批通过。
- 已实现功能：
  1. 新增加工任务模型：`CreateAssetProcessingJobRequest`、`CreateAssetProcessingJobInputSource`、`AssetProcessingJobView`、`AssetProcessingInputView`。
  2. 新增仓储方法：`create_asset_processing_job`，写入 `catalog.asset_processing_job` 并批量写入 `catalog.asset_processing_input`。
  3. 新增接口：`POST /api/v1/assets/{versionId}/processing-jobs`，包含路径/请求一致性校验、`processing_mode` 必填校验、`input_sources` 非空校验、输出/输入版本存在性校验、事务审计。
  4. 新增处理摘要归一化：`processing_summary_json` 非对象值归一化为 `{}`，并按 V1 存储口径落入 `metadata.processing_summary_json`。
  5. 新增测试拆分：新增 `tests/processing_jobs.rs`，独立覆盖 `input_sources` 校验；原权限拒绝用例保留在 `tests/mod.rs`。
  6. 更新 OpenAPI：新增 processing-jobs 路径与 `CreateAssetProcessingJobRequest/AssetProcessingJob` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`apps/platform-core/src/modules/catalog/tests/processing_jobs.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 本地数据库连通性核验：`psql postgres://datab:datab_local_pass@127.0.0.1:5432/datab -c 'select 1 as ok;'`
  4. 端到端联调（`APP_PORT=18091`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.asset_version`（输出版本 + 输入版本）
     - 调用 `POST /api/v1/assets/{versionId}/processing-jobs`
     - 回查 `catalog.asset_processing_job`、`catalog.asset_processing_input` 与 `audit.audit_event`
     - 清理测试数据（`asset_processing_input/asset_processing_job/asset_version/data_asset/organization`）
  5. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `56 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且 `processing_job_id=52692fb3-909a-41d1-b910-5003109002a5`；`catalog.asset_processing_job` 命中 `processing_mode=platform_managed`、`metadata.processing_summary_json` 回写成功；`catalog.asset_processing_input` 命中 `input_role=primary_input`；审计命中 `catalog.asset_processing_job.create|asset_processing_job|success|req-cat012-proc-002`；业务表清理后残留 `0|0`，审计残留 `1`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据原样处理与产品化加工流程设计.md`（交易前 12 步中的加工处理执行 + 基础加工责任链）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2A 加工处理区 `AssetProcessingJob / Input / evidence`）、`docs/数据库设计/V1/upgrade/062_data_product_metadata_contract.sql`（`catalog.asset_processing_job` 与 `catalog.asset_processing_input` 字段约束）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（`POST /api/v1/assets/{versionId}/processing-jobs` 冻结接口）。
- 覆盖的任务清单条目：`CAT-012`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：联调首轮因测试 SQL 使用历史字段 `org_code` 导致组织插入失败，已改按当前库结构（`org_name/org_type/status`）重跑并通过。

### BATCH-097（待审批）

- 状态：通过
- 当前任务编号：CAT-013
- 当前批次目标：实现数据契约接口 `POST /api/v1/skus/{id}/data-contracts` 与 `GET /api/v1/skus/{id}/data-contracts/{contractId}`。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-096` 已获人工审批通过。
- 已实现功能：
  1. 新增数据契约模型：`CreateDataContractRequest`、`DataContractView`。
  2. 新增仓储方法：`create_data_contract`、`get_data_contract`，读写 `contract.data_contract` 并按 `sku_id + data_contract_id` 关联查询。
  3. 新增接口：`POST /api/v1/skus/{id}/data-contracts`，包含路径/请求一致性校验、`contract_name` 必填校验、`version_no > 0` 校验、SKU 存在性校验、事务审计。
  4. 新增接口：`GET /api/v1/skus/{id}/data-contracts/{contractId}`，包含 SKU 存在性校验与契约归属校验。
  5. 新增测试拆分：新增 `tests/data_contracts.rs`，独立覆盖 `sku_id` 路径不一致校验；权限拒绝用例补充在 `tests/mod.rs`。
  6. 更新 OpenAPI：新增 data-contracts 路径与 `CreateDataContractRequest/DataContract` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`apps/platform-core/src/modules/catalog/tests/data_contracts.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18092`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.asset_version` + `catalog.product` + `catalog.product_sku`
     - 调用 `POST /api/v1/skus/{id}/data-contracts`
     - 调用 `GET /api/v1/skus/{id}/data-contracts/{contractId}`
     - 回查 `contract.data_contract` 与 `audit.audit_event`
     - 清理测试数据（`data_contract/product_sku/product/asset_version/data_asset/organization`）
  4. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `59 passed, 0 failed, 1 ignored`；`POST/GET` 接口均返回 `success=true` 且 `data_contract_id=eb42b70f-be78-47f5-9200-3ccaf60cd9c5`；`contract.data_contract` 命中 `contract_scope=sku`、`business_terms_json/processing_terms_json` 正确回写；审计命中 `catalog.data_contract.create|data_contract|success|req-cat013-contract-001`；业务数据清理后残留 `0|0`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据商品元信息与数据契约设计.md`（3.2 契约独立建模 + 4. 元信息域条款化）、`docs/数据库设计/V1/upgrade/062_data_product_metadata_contract.sql`（`contract.data_contract` 字段与约束）、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（`POST/GET /api/v1/skus/{id}/data-contracts...` 冻结接口）。
- 覆盖的任务清单条目：`CAT-013`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：按你的要求，本批未更新 `V1-Core-人工审批记录.md`，审批条目由你手工维护。

### BATCH-098（待审批）

- 状态：通过
- 当前任务编号：CAT-014
- 当前批次目标：实现可交付对象接口 `POST /api/v1/assets/{versionId}/objects`，区分原始对象、预览对象、交付对象、报告对象、结果对象。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-097` 已获人工审批通过。
- 已实现功能：
  1. 新增可交付对象模型：`CreateAssetObjectRequest`、`AssetObjectView`。
  2. 新增仓储方法：`create_asset_object`，事务内写入 `catalog.asset_object_binding` 与 `catalog.asset_storage_binding`，建立逻辑对象与存储绑定。
  3. 新增接口：`POST /api/v1/assets/{versionId}/objects`，包含路径/请求一致性校验、`object_kind/object_name/object_uri` 必填校验、`object_kind` 取值白名单（`raw_object|preview_object|delivery_object|report_object|result_object`）、资产版本存在性校验、事务审计。
  4. 新增测试拆分：新增 `tests/asset_objects.rs`，独立覆盖 `object_kind` 非法值校验；权限拒绝用例补充在 `tests/mod.rs`。
  5. 更新 OpenAPI：新增 objects 路径与 `CreateAssetObjectRequest/AssetObject` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`apps/platform-core/src/modules/catalog/tests/asset_objects.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18093`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.asset_version`
     - 调用 `POST /api/v1/assets/{versionId}/objects`
     - 回查 `catalog.asset_object_binding`、`catalog.asset_storage_binding` 与 `audit.audit_event`
     - 清理测试数据（`asset_storage_binding/asset_object_binding/asset_version/data_asset/organization`）
  4. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `61 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且 `asset_object_id=f77af3f2-72bb-4e40-8c17-1a8f00df72ed`、`asset_storage_binding_id=87a97398-a7a9-46f4-9e24-3e872da55275`；`asset_object_binding` 命中 `object_kind=delivery_object`；`asset_storage_binding` 命中 `storage_zone=product`、`object_uri=s3://product/cat014/delivery-package-v1.zip`；审计命中 `catalog.asset_object.create|asset_object|success|req-cat014-object-001`；清理后残留 `0|0`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据商品存储与分层存储设计.md`（分层闭环与对象分区）、`docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（`POST /api/v1/assets/{versionId}/objects` 冻结接口）、`docs/数据库设计/V1/upgrade/064_storage_layering_architecture.sql`（对象存储分层字段语义）、`docs/数据库设计/V1/upgrade/061_data_object_trade_modes.sql`（`catalog.asset_object_binding` 字段）。
- 覆盖的任务清单条目：`CAT-014`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：按你的要求，本批继续把新增测试能力拆分到独立文件，避免持续堆积到单一测试文件。

### BATCH-099（待审批）

- 状态：通过
- 当前任务编号：CAT-015
- 当前批次目标：实现版本发布/订阅策略接口 `PATCH /api/v1/assets/{assetId}/release-policy`。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-098` 已获人工审批通过。
- 已实现功能：
  1. 新增发布策略模型：`PatchAssetReleasePolicyRequest`、`AssetReleasePolicyView`。
  2. 新增仓储方法：`patch_asset_release_policy`，按 `asset_id` 批量更新 `catalog.asset_version` 的 `release_mode/is_revision_subscribable/update_frequency/release_notes_json`，并返回最新版本策略快照与更新版本数。
  3. 新增接口：`PATCH /api/v1/assets/{assetId}/release-policy`，包含资产存在性校验、最小变更字段校验、`release_mode` 枚举校验（`snapshot|revision`）、事务审计。
  4. 新增测试拆分：新增 `tests/release_policy.rs`，独立覆盖 `release_mode` 非法值校验；权限拒绝用例补充在 `tests/mod.rs`。
  5. 更新 OpenAPI：新增 release-policy 路径与 `PatchAssetReleasePolicyRequest/AssetReleasePolicy` schema。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`apps/platform-core/src/modules/catalog/tests/release_policy.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18094`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + 同资产两条 `catalog.asset_version`
     - 调用 `PATCH /api/v1/assets/{assetId}/release-policy`
     - 回查 `catalog.asset_version`（确认两版本策略均更新）与 `audit.audit_event`
     - 清理测试数据（`asset_version/data_asset/organization`）
  4. 数据残留核对：验证业务表残留均为 `0`；审计表按 append-only 保留请求记录。
- 验证结果：通过。`cargo test -p platform-core` 结果 `63 passed, 0 failed, 1 ignored`；API 返回 `success=true` 且 `applied_version_count=2`、`latest_version_no=2`；两条 `asset_version` 均更新为 `release_mode=revision`、`is_revision_subscribable=true`、`update_frequency=daily`、`release_notes_json={"note":"enable subscription"}`；审计命中 `catalog.asset.release_policy.patch|asset|success|req-cat015-release-001`；清理后残留 `0|0`。
- 覆盖的冻结文档条目：`docs/原始PRD/数据商品存储与分层存储设计.md`（闭环中的版本化治理）、`docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（冻结路径 `PATCH /api/v1/assets/{assetId}/release-policy`）、`docs/数据库设计/V1/upgrade/064_storage_layering_architecture.sql`（分层策略协同）、`docs/数据库设计/V1/upgrade/061_data_object_trade_modes.sql`（`asset_version` 发布策略字段）。
- 覆盖的任务清单条目：`CAT-015`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：按你的要求，本批将 release-policy 校验测试独立拆分到 `tests/release_policy.rs`，避免持续堆积在单一测试文件。

### BATCH-100（待审批）

- 状态：通过
- 当前任务编号：CAT-016, CAT-017, CAT-018, CAT-019
- 当前批次目标：补齐商品提审、Listing 状态机、三类审核接口、商品冻结/下架接口，并保持权限/审计/OpenAPI 一致。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 均已完成并通过审批；`BATCH-099` 已完成。
- 已实现功能：
  1. `CAT-016`：新增 `POST /api/v1/products/{id}/submit`，实现提交审核前置校验（元信息档案存在、至少一个 SKU、所有 SKU 具备模板绑定草稿标记、风险阻断校验），并将商品状态 `draft -> pending_review`。
  2. `CAT-017`：在 `catalog::service` 冻结基础 Listing 状态机与转移规则（`draft/pending_review/listed/delisted/frozen`），接口执行前进行状态合法性与转移合法性校验。
  3. `CAT-018`：新增 `POST /api/v1/review/subjects/{id}`、`POST /api/v1/review/products/{id}`、`POST /api/v1/review/compliance/{id}`；写入 `review.review_task` + `review.review_step`；产品审核 `approve/reject` 对应 `pending_review -> listed/draft`。
  4. `CAT-019`：新增 `POST /api/v1/products/{id}/suspend`，支持 `delist/freeze` 两种动作；接入 `catalog.product.suspend` 与 `risk.product.freeze` 双权限口径；执行状态转移与审计。
  5. 补齐 OpenAPI 路径与 schema：`SubmitProductRequest`、`SuspendProductRequest`、`ReviewDecisionRequest`、`ProductSubmit`、`ProductLifecycle`、`ReviewDecision`。
  6. 测试拆分：新增 `tests/listing_submit_review.rs` 与 `tests/suspend.rs`，并在 `tests/mod.rs` 补齐新增接口权限拒绝用例。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/domain.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/service.rs`、`apps/platform-core/src/modules/catalog/tests/mod.rs`、`apps/platform-core/src/modules/catalog/tests/listing_submit_review.rs`、`apps/platform-core/src/modules/catalog/tests/suspend.rs`、`packages/openapi/catalog.yaml`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 端到端联调（`APP_PORT=18095`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`，`KAFKA_BROKERS=127.0.0.1:9094`）：
     - 预置数据：`core.organization` + `catalog.data_asset` + `catalog.asset_version` + `catalog.product` + `catalog.product_metadata_profile` + `catalog.product_sku`
     - 调用 `POST /api/v1/products/{id}/submit`
     - 调用 `POST /api/v1/review/subjects/{id}`、`POST /api/v1/review/compliance/{id}`、`POST /api/v1/review/products/{id}`
     - 调用 `POST /api/v1/products/{id}/suspend`（`delist` + `freeze`）
     - 回查 `catalog.product`、`review.review_task`、`audit.audit_event`
     - 清理测试业务数据（保留审计 append-only）
  4. 数据残留核对：`catalog.product|catalog.product_sku|catalog.product_metadata_profile` 残留 `0|0|0`。
- 验证结果：通过。`cargo test -p platform-core` 结果 `72 passed, 0 failed, 1 ignored`。联调响应链路：
  - `req-cat016-submit-001`：`status=pending_review`
  - `req-cat018-product-001`：`status=listed`
  - `req-cat019-delist-001`：`previous_status=listed,status=delisted`
  - `req-cat019-freeze-001`：`previous_status=delisted,status=frozen`
  - 最终 `catalog.product.status=frozen`
  - `review.review_task`（`product_submit/product_review/compliance_review`）计数 `3`
  - 审计命中：`catalog.product.submit`、`catalog.review.subject`、`catalog.review.compliance`、`catalog.review.product`、`catalog.product.suspend`
- 覆盖的冻结文档条目：`docs/领域模型/全量领域模型与对象关系说明.md`（4.2 目录与商品聚合）、`docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（submit/product 相关冻结接口）、`docs/业务流程/业务流程图-V1-完整版.md`（4.2 商品创建与提交审核流程）、`docs/数据库设计/V1/upgrade/025_review_workflow.sql`（审核任务与步骤表结构）、`docs/权限设计/接口权限校验清单.md`（submit/review/suspend 权限口径）。
- 覆盖的任务清单条目：`CAT-016`, `CAT-017`, `CAT-018`, `CAT-019`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过（按“每 4~6 个 tasks 集中审批”节奏，当前批次已完成，等待集中审批）
- 备注：本批仍保持测试代码按功能拆分，不把新增逻辑持续堆积在单一文件。

### BATCH-101（待审批）

- 状态：通过
- 当前任务编号：CAT-016, CAT-017, CAT-018, CAT-019（审计修复批）
- 当前批次目标：修复上一版实现与冻结约束之间的缺口，补齐高风险 step-up、tenant 作用域、审核任务推进一致性、模板有效性强校验与事件闭环。
- 前置依赖核对结果：延续 `CAT-016~CAT-019` 依赖（`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005`）均已完成并审批通过；本批为同任务段审计返修。
- 已实现功能：
  1. 补齐 tenant + resource 作用域：对 `submit/review(compliance+product)/suspend` 增加 `x-tenant-id` 与 `seller_org_id/subject_id` 一致性校验（平台角色豁免）。
  2. 补齐冻结高风险二次认证：`suspend_mode=freeze` 强制 `x-step-up-challenge-id + x-user-id`，并校验 `iam.step_up_challenge` 为 `verified` 且未过期，且 action/ref 精确匹配 `risk.product.freeze + product + product_id`。
  3. 强化提审“模板齐全”校验：由“仅检查 sku metadata 非空”升级为“模板 UUID 合法 + `contract.template_definition` 存在且 `status=active` + `sku_type` 与 `applicable_sku_types` 兼容”。
  4. 修复产品审核任务推进：`submit` 创建 `product_review` pending 任务并写首步 `submit`；`review/products` 在同一 pending 任务追加审批 step 并闭合任务状态为 `approved/rejected`，不再新建平行任务。
  5. 补齐实体存在性校验：`review/subjects` 校验 `core.organization` 存在；`review/compliance` 校验目标 `product` 存在。
  6. 补齐事件闭环：在提审、状态变更（审核通过/驳回、下架/冻结）写入 `ops.outbox_event`。
  7. 新增测试：`rejects_freeze_without_step_up_header`（无 step-up 头直接拒绝）。
- 涉及文件：`apps/platform-core/src/modules/catalog/api.rs`、`apps/platform-core/src/modules/catalog/repository.rs`、`apps/platform-core/src/modules/catalog/tests/suspend.rs`、`docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 修复版联调（`APP_PORT=18096`，`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`）：
     - 预置 `organization/user/data_asset/asset_version/product/profile/sku/template_definition`
     - 调用 submit -> subject/compliance/product review -> suspend delist
     - 调用 IAM step-up check + verify
     - 携带 `x-step-up-challenge-id` 调用 suspend freeze
     - 回查 `catalog.product`、`review.review_task/review.review_step`、`ops.outbox_event`、`audit.audit_event`
     - 清理测试业务数据并核对残留
- 验证结果：通过。
  - 单测：`73 passed, 0 failed, 1 ignored`
  - 联调：
    - `submit` 后 `status=pending_review`
    - `review/products approve` 后 `status=listed`
    - `suspend delist` 后 `status=delisted`
    - `step-up verified + suspend freeze` 后 `status=frozen`
    - `product_review` 任务同一 `review_task_id`，`review_step` 数量为 `2`（submit + approve）
    - `ops.outbox_event` 命中 `catalog.product.submitted` 与 `catalog.product.status.changed`
    - 审计命中 `catalog.product.submit / catalog.review.subject / catalog.review.compliance / catalog.review.product / catalog.product.suspend`
  - 清理后残留：`catalog.product|catalog.product_sku|review.review_task|ops.outbox_event = 0|0|0|0`
- 覆盖的冻结文档条目：`docs/开发任务/v1-core-开发任务清单.csv`（CAT-016~019 DoD/acceptance）、`docs/权限设计/接口权限校验清单.md`（tenant+resource、risk freeze + step-up）、`docs/业务流程/业务流程图-V1-完整版.md`（提审->审核->上架->下架/冻结）、`docs/数据库设计/V1/upgrade/025_review_workflow.sql`（review task/step 模型）、`docs/数据库设计/V1/upgrade/050_audit_search_dev_ops.sql`（ops.outbox_event）。
- 覆盖的任务清单条目：`CAT-016`, `CAT-017`, `CAT-018`, `CAT-019`（审计修复）
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt`。
- 待人工审批结论：通过
- 备注：`docs/开发任务/V1-Core-人工审批记录.md` 继续由人工手工维护，本批未自动写入。

### BATCH-102（待审批）

- 状态：待审批
- 当前任务编号：CAT-020, CAT-021, CAT-022, CAT-023
- 当前批次目标：按顺序完成商品详情/卖方主页读接口、模板绑定与策略更新、搜索可见性字段打底、五条标准链路样例模板固化。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并通过审批；`BATCH-101` 已完成。
- 已实现功能：
  1. `CAT-020`：实现 `GET /api/v1/products/{id}` 与 `GET /api/v1/sellers/{orgId}/profile`，补齐权限校验、审计事件（`catalog.product.read`/`catalog.seller.profile.read`）、OpenAPI schema 与权限拒绝测试。
  2. `CAT-021`：实现 `POST /api/v1/products/{id}/bind-template`、`POST /api/v1/skus/{id}/bind-template`、`PATCH /api/v1/policies/{id}`；落地模板绑定写入（`contract.template_binding` + `catalog.product_sku.metadata.draft_template_id`）、策略更新（`contract.usage_policy`）、审计事件（`template.product.bind`/`template.sku.bind`/`template.policy.update`）、权限矩阵与 OpenAPI。
  3. `CAT-022`：在商品创建/编辑 DTO 与仓储层增加搜索可见性打底字段（`subtitle/industry/use_cases/data_classification/quality_score`）；在商品详情与卖方主页返回 `search_document_version/index_sync_status`，并接入 `search.product_search_document`、`search.seller_search_document` 读侧状态回显。
  4. `CAT-023`：新增五条标准链路样例模板中心 `GET /api/v1/catalog/standard-scenarios`，固化每条链路的标准商品模板、元信息模板、契约模板、审核样例；新增独立模块 `standard_scenarios.rs`，避免继续膨胀主 API 文件。
  5. 补充 DB smoke 测试：`repository_bind_template_to_sku_smoke`、`repository_patch_usage_policy_smoke`（受 `CATALOG_DB_SMOKE=1` 开关控制）。
- 涉及文件：
  - `apps/platform-core/src/modules/catalog/api.rs`
  - `apps/platform-core/src/modules/catalog/domain.rs`
  - `apps/platform-core/src/modules/catalog/mod.rs`
  - `apps/platform-core/src/modules/catalog/repository.rs`
  - `apps/platform-core/src/modules/catalog/service.rs`
  - `apps/platform-core/src/modules/catalog/standard_scenarios.rs`
  - `apps/platform-core/src/modules/catalog/tests/mod.rs`
  - `apps/platform-core/src/modules/catalog/tests/template_policy_db.rs`
  - `packages/openapi/catalog.yaml`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `psql postgres://datab:datab_local_pass@127.0.0.1:5432/datab -c "select 1"`（数据库连通）
  4. `CATALOG_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core repository_ -- --nocapture`（模板绑定/策略更新 DB smoke）
  5. `cargo run -p platform-core` 手工 API 联调尝试（受运行环境 socket 权限限制，启动阶段 Kafka metadata 探测失败，见下方“未覆盖项”）。
- 验证结果：
  - `cargo test -p platform-core`：`83 passed, 0 failed, 1 ignored`
  - DB smoke：`2 passed, 0 failed`
  - `psql` 连通性：通过
  - 运行时手工 API：受限于环境 `Operation not permitted`（rdkafka socket 创建失败），未能完成在线 HTTP 调用链路。
- 覆盖的冻结文档条目：
  - `docs/领域模型/全量领域模型与对象关系说明.md:L200`（商品/卖方聚合）
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md:L82`（商品详情、卖方主页、模板绑定）
  - `docs/原始PRD/数据对象产品族与交付模式增强设计.md:L292`（七类交易方式与模板绑定约束）
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`（首批场景到 SKU/模板映射）
  - `docs/原始PRD/商品搜索、排序与索引同步设计.md:L164`（搜索投影字段）
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md:L4824`（搜索架构与投影对象）
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md:L216`（首批五条标准链路）
  - `docs/原始PRD/数据产品分类与交易模式详细稿.md:L155`（V1 标准数据产品目录）
- 覆盖的任务清单条目：`CAT-020`, `CAT-021`, `CAT-022`, `CAT-023`
- 未覆盖项：
  - 运行态手工 HTTP 联调（服务启动被 Kafka metadata socket 权限限制阻塞）；已用 DB smoke + 单测 + OpenAPI 对齐补齐可执行验证证据。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)`/`TODO(V2-reserved)`/`TODO(V3-reserved)` 代码注释项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：待审批
- 备注：按你的新流程，本批为连续 4 个任务集中提审；实现中保持按功能拆分，新增样例逻辑独立到 `standard_scenarios.rs`，避免单文件持续膨胀。

### BATCH-103（待审批）

- 状态：通过
- 当前任务编号：CAT-020
- 当前批次目标：按冻结文档完成并核验商品详情与卖方主页读接口，补齐成功链路 + 审计痕迹证据，且仅覆盖单任务范围。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并通过审批；`BATCH-101` 已通过；本批不跨任务。
- 已实现功能：
  1. 新增 `CAT-020` 专用 DB smoke API 集成测试 `cat020_read_endpoints_db_smoke`：插入 `organization/data_asset/asset_version/product/sku` 测试数据，调用 `GET /api/v1/products/{id}` 与 `GET /api/v1/sellers/{orgId}/profile`，并核验 `audit.audit_event` 中 `catalog.product.read`、`catalog.seller.profile.read` 落库。
  2. 修复 `PostgresCatalogRepository::get_product_detail` 联表查询缺陷：在 `catalog.product` 与 `search.product_search_document` 联查后，显式使用 `p.` 前缀限定 `product` 列，消除生产路径 `column reference "product_id" is ambiguous` 的运行时错误。
  3. 维持 CAT-020 既有权限口径和审计行为，不引入 CAT-021+ 变更。
- 涉及文件：
  - `apps/platform-core/src/modules/catalog/repository.rs`
  - `apps/platform-core/src/modules/catalog/tests/mod.rs`
  - `apps/platform-core/src/modules/catalog/tests/cat020_read_db.rs`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `CATALOG_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core cat020_ -- --nocapture`
- 验证结果：
  - `cargo test -p platform-core`：`84 passed, 0 failed, 1 ignored`
  - `CAT-020` 定向 smoke：`1 passed, 0 failed`；已覆盖“插入测试数据 -> 读接口调用 -> 审计事件落库 -> 清理业务数据”全链路。
  - 定位并修复真实缺陷证据：修复前定向 smoke 报错 `SqlState(E42702) column reference "product_id" is ambiguous`，修复后同命令通过。
- 覆盖的冻结文档条目：
  - `docs/开发任务/v1-core-开发任务清单.csv`：CAT-020 DoD + acceptance（接口/DTO/权限/审计/错误码/最小测试 + 集成验证）
  - `docs/领域模型/全量领域模型与对象关系说明.md:L200`（Product 与 Seller 聚合事实源定位）
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md:L82`（V1 冻结 GET 接口路径）
  - `docs/业务流程/业务流程图-V1-完整版.md:L86`（商品流程链路审计可追溯）
  - `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（接口清单冻结）
  - `docs/开发准备/统一错误码字典正式版.md`（403/404/500 统一错误口径）
- 覆盖的任务清单条目：`CAT-020`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：按你的最新流程，本批为“单 task 自检完成后再提审”；`V1-Core-人工审批记录.md` 继续由人工手工维护，本批未自动写入。

### BATCH-104（待审批）

- 状态：待审批
- 当前任务编号：CAT-021
- 当前批次目标：按冻结文档完成模板绑定与策略更新接口的单任务闭环，并补齐 API 调用级 DB smoke 验证（含审计落库证据）。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已在历史批次完成并审批通过；`BATCH-103` 已通过，满足执行条件。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `CAT-021` 的 DoD / acceptance / technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `CAT-021` 说明与 CSV 一致，按单任务执行。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：执行“计划中 -> 编码 -> 验证 -> 待审批”顺序。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循 V1 范围与冻结边界，不扩展 V2/V3。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：按批次模板登记，先写计划中后实现。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：维护批次更新记录，追溯 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：已读取规则；按你要求本批不自动写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：读取 5.3.2A SKU/模板映射（L229 附近）。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 catalog 模块职责边界不越界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认冻结接口路径口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本任务以 DB 审计闭环为主，无新增 topic。
  12. `docs/开发准备/统一错误码字典正式版.md`：错误响应沿用统一 `ErrorResponse`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补充最小 API+DB smoke 集成验证。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：测试按功能拆分为独立文件。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：验证使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：使用标准 `DATABASE_URL` 注入。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + PostgreSQL + Axum 既有栈。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体，不做跨边界重构。
- technical_reference 约束映射：
  1. `docs/数据库设计/接口协议/目录与商品接口协议正式版.md:L82`：覆盖 `POST /api/v1/products/{id}/bind-template`、`POST /api/v1/skus/{id}/bind-template`、`PATCH /api/v1/policies/{id}` 与模板绑定规则。
  2. `docs/原始PRD/数据对象产品族与交付模式增强设计.md:L292`：沿用七类标准交易方式，模板与 `sku_type` 强匹配，不接受泛化 API 模板族。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：按首批标准场景与 V1 SKU 模板映射口径校验绑定行为。
- 已实现功能：
  1. 新增 `CAT-021` API 级 DB smoke 用例 `cat021_template_bind_and_policy_patch_db_smoke`，真实调用三条接口并验证响应字段。
  2. 在同一用例中校验模板绑定写入：`contract.template_binding` 至少落两条（product 绑定 + sku 绑定）。
  3. 校验 SKU 元数据回写：`catalog.product_sku.metadata.draft_template_id` 随 sku 绑定更新。
  4. 校验策略更新落库：`contract.usage_policy` 的 `policy_name/status/exportable` 与请求一致。
  5. 校验审计痕迹：`audit.audit_event` 命中 `template.product.bind`、`template.sku.bind`、`template.policy.update`。
- 涉及文件：
  - `apps/platform-core/src/modules/catalog/tests/cat021_template_policy_db.rs`
  - `apps/platform-core/src/modules/catalog/tests/mod.rs`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core cat021_ -- --nocapture`
  3. `cargo test -p platform-core`
  4. `CATALOG_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core cat021_ -- --nocapture`
- 验证结果：
  - `cargo test -p platform-core cat021_ -- --nocapture`：通过（`1 passed`，未开启 DB smoke 时走开关短路）。
  - `cargo test -p platform-core`：通过（`79 passed, 0 failed, 1 ignored`）。
  - DB smoke：沙箱内因 socket 权限限制报 `Operation not permitted`，提权后同命令通过（`1 passed, 0 failed`）。
- 覆盖的冻结文档条目：
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5.1~5.3 模板绑定路径与规则）
  - `docs/原始PRD/数据对象产品族与交付模式增强设计.md`（4. 七类标准交易方式）
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`（5.3.2A 首批场景到 V1 SKU 与模板映射）
- 覆盖的任务清单条目：`CAT-021`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：`V1-Core-人工审批记录.md` 按你的要求继续由人工维护，本批未自动写入。

### BATCH-105（待审批）

- 状态：通过
- 当前任务编号：CAT-022
- 当前批次目标：实现“商品与卖方搜索可见性字段”闭环，补齐搜索同步事件、搜索可见性读回与 API 级 DB smoke 证据。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-104` 已通过，可执行。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `CAT-022` DoD/acceptance/technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `CAT-022` 单任务执行口径。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中 -> 编码 -> 验证 -> 待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持 V1 范围，避免扩展。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：先登记计划中再编码。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次更新，无遗漏。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读规则，继续人工维护。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：读取第 44 章搜索同步边界。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认搜索同步由外围进程消费事件。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：核对 catalog 接口冻结范围。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认 `dtp.outbox.domain-events` 与搜索同步 topic 边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用统一错误结构。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐最小集成验证证据。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增测试文件，避免单文件膨胀。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调库使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：使用 `DATABASE_URL` 注入。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + PostgreSQL + outbox。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体 + 外围索引进程模式。
- technical_reference 约束映射：
  1. `docs/原始PRD/商品搜索、排序与索引同步设计.md:L164`：搜索投影至少承载 subtitle/industry/use_cases/data_classification/quality_score 与 `document_version/index_sync_status`。
  2. `docs/数据库设计/接口协议/目录与商品接口协议正式版.md:L82`：catalog 接口沿既有路径，不新增路径；字段通过商品创建/编辑与详情读取闭环。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L4824`：遵循“主数据写库 -> outbox 事件 -> 索引同步”的 V1 主链路。
- 已实现功能：
  1. 在 `create_product_draft` 与 `patch_product_draft` 中新增搜索同步 outbox 事件 `search.product.changed`（topic：`dtp.outbox.domain-events`）。
  2. 新增 `CAT-022` API 级 DB smoke：`cat022_search_visibility_fields_and_events_db_smoke`。
  3. smoke 用例覆盖创建商品 + 编辑搜索可见性字段（`searchable_text/subtitle/industry/use_cases/data_classification/quality_score`）。
  4. smoke 用例验证 `GET /api/v1/products/{id}` 与 `GET /api/v1/sellers/{orgId}/profile` 返回 `search_document_version/index_sync_status`。
  5. smoke 用例验证 outbox（`search.product.changed`）与审计（`catalog.product.patch`）落库。
- 涉及文件：
  - `apps/platform-core/src/modules/catalog/api/handlers/product_and_review.rs`
  - `apps/platform-core/src/modules/catalog/tests/mod.rs`
  - `apps/platform-core/src/modules/catalog/tests/cat022_search_visibility_db.rs`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core cat022_ -- --nocapture`
  3. `cargo test -p platform-core`
  4. `CATALOG_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core cat022_ -- --nocapture`
- 验证结果：
  - `cargo test -p platform-core cat022_ -- --nocapture`：通过（未开启 DB smoke 时走开关短路）。
  - `cargo test -p platform-core`：通过（`80 passed, 0 failed, 1 ignored`）。
  - DB smoke：沙箱内因权限限制返回 `Operation not permitted`；提权后通过（`1 passed, 0 failed`）。
- 覆盖的冻结文档条目：
  - `docs/原始PRD/商品搜索、排序与索引同步设计.md`（6. 搜索投影设计）
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5. V1 接口）
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`（44. 商品搜索、排序与索引同步设计）
  - `docs/业务流程/业务流程图-V1-完整版.md`（6.3.1/6.3.2 搜索同步事件语义）
- 覆盖的任务清单条目：`CAT-022`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：`V1-Core-人工审批记录.md` 继续按你的要求由人工维护，本批未自动写入。

### BATCH-106（计划中）

- 状态：通过
- 当前任务编号：CAT-023
- 当前批次目标：按 `CAT-023` 冻结要求完成五条标准链路模板接口闭环，补齐接口调用、审计落库与可重复 DB 联调验证证据。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-105` 已通过，可继续执行。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `CAT-023` 的 DoD、acceptance、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版任务描述与 CSV 一致，以 CSV 为准执行。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：执行“计划中 -> 编码 -> 验证 -> 待审批”固定流程。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循 V1 冻结边界，不引入 V2/V3 功能。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：先登记计划中，再追加待审批结果。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次记录，保持 TODO 追溯完整。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：仅读取规则，按人工要求不自动写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：读取 5.3.2 与 5.3.2A 场景与 SKU 模板映射。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 catalog 模块内实现，不跨服务边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认接口路径冻结不变。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本任务不新增 topic，仅补齐审计证据。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用统一错误响应结构。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐接口级 smoke + 审计校验。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增独立 handler 与测试文件，避免单文件膨胀。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调库使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：通过 `DATABASE_URL` 注入连接参数。
  17. `docs/开发准备/技术选型正式版.md`：维持 Rust + PostgreSQL + Axum 栈不变。
  18. `docs/开发准备/平台总体架构设计草案.md`：维持模块化单体边界，不做架构外扩。
- technical_reference 约束映射：
  1. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L216`：接口返回的 5 条场景与名称严格对应首批标准链路。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`primary_sku / supplementary_skus / 合同/验收/退款模板` 与映射表一致。
  3. `docs/原始PRD/数据产品分类与交易模式详细稿.md:L155`：仅使用 8 个标准 SKU 真值，不引入非冻结 SKU。
- 已实现功能：
  1. 新增独立 handler：`standard_scenarios.rs`，承载 `GET /api/v1/catalog/standard-scenarios`，避免继续膨胀 `product_and_review.rs`。
  2. 在 `standard-scenarios` 读取接口增加审计事件：`catalog.standard.scenarios.read`，`ref_type=catalog_standard_scenarios`，固定 `ref_id=00000000-0000-0000-0000-000000000023`。
  3. 保持无数据库环境可读：当未配置 `DATABASE_URL` 时不阻断场景模板返回；有库时写审计。
  4. 新增 `CAT-023` 专用 DB smoke：真实调用接口并验证响应含 `S1~S5`，同时校验审计落库。
  5. 维持既有路由与 OpenAPI 路径不变，不新增业务功能。
- 涉及文件：
  - `apps/platform-core/src/modules/catalog/api/handlers/standard_scenarios.rs`
  - `apps/platform-core/src/modules/catalog/api/handlers/mod.rs`
  - `apps/platform-core/src/modules/catalog/api/handlers/product_and_review.rs`
  - `apps/platform-core/src/modules/catalog/api/mod.rs`
  - `apps/platform-core/src/modules/catalog/tests/cat023_standard_scenarios_db.rs`
  - `apps/platform-core/src/modules/catalog/tests/mod.rs`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `CATALOG_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core cat023_ -- --nocapture`
- 验证结果：
  - `cargo test -p platform-core`：通过（`81 passed, 0 failed, 1 ignored`）。
  - `CAT-023` DB smoke：沙箱内首次因 socket 权限限制失败（`Operation not permitted`）；提权重跑通过（`1 passed, 0 failed`）。
  - DB smoke 用例已完成接口调用与审计落库核验，并在测试结束清理本次 `request_id` 审计数据。
- 覆盖的冻结文档条目：
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`（5.3.2 首批 5 条标准链路）
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`（5.3.2A 首批标准场景到 V1 SKU 与模板映射）
  - `docs/原始PRD/数据产品分类与交易模式详细稿.md`（6. V1 标准数据产品目录）
- 覆盖的任务清单条目：`CAT-023`
- 未覆盖项：无
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：`V1-Core-人工审批记录.md` 继续由人工维护，本批未自动写入。

### BATCH-107（待审批）

- 状态：通过
- 当前任务编号：CAT-024
- 当前批次目标：按冻结文档完成 Catalog/Listing/Review 集成验证：商品创建、SKU 创建、质量报告、契约发布、提交审核、审核通过/驳回、冻结（step-up）。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并审批通过；`BATCH-106` 已通过，可执行。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `CAT-024` DoD/acceptance/technical_reference，按单任务执行。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版描述与 CSV 一致，以 CSV 为准。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：执行“计划中 -> 编码 -> 验证 -> 待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循 V1 边界，不扩展 V2/V3。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：先写计划中，再补待审批结果。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次更新记录，保持 TODO 追溯。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：仅读规则，按你的要求不自动写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对目录/商品主流程与审核上架链路。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 catalog 模块边界，不跨服务扩展。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：核对 V1 路径与请求头约束。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：核对 outbox 事件口径，不把事件作为主状态。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用统一错误结构与状态码口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐 API+DB+curl 联调证据。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增独立测试文件，避免继续膨胀旧文件。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：使用环境变量注入 `DATABASE_URL`。
  17. `docs/开发准备/技术选型正式版.md`：保持 Rust + PostgreSQL + Axum + Kafka 边界。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体，不做额外拆分。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L200`：覆盖目录与商品聚合（asset/product/sku/metadata_profile/contract）。
  2. `docs/数据库设计/接口协议/目录与商品接口协议正式版.md:L82`：覆盖 `POST /api/v1/products`、`POST /api/v1/products/{id}/skus`、`POST /api/v1/assets/{versionId}/quality-reports`、`POST /api/v1/skus/{id}/data-contracts`、`POST /api/v1/products/{id}/submit`、`POST /api/v1/review/products/{id}`、`POST /api/v1/products/{id}/suspend`。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L86`：覆盖“创建 -> 元信息/质量/契约 -> 提交审核 -> 通过/驳回 -> 冻结”链路。
- 已实现功能：
  1. 新增 `CAT-024` 专用 DB smoke：`cat024_catalog_listing_review_end_to_end_db_smoke`（单测内真实 HTTP 路由调用 + 数据库断言 + 清理）。
  2. 覆盖主链路（approve+freeze）与分支链路（reject）双路径状态机断言：`pending_review -> listed -> frozen` 与 `pending_review -> draft`。
  3. 覆盖 step-up 冻结校验：插入 `core.user_account` + `iam.step_up_challenge(verified)` 后调用冻结接口。
  4. 覆盖审计/事件证据：`catalog.product.*`、`catalog.review.product`、`catalog.asset_quality_report.create`、`catalog.data_contract.create`、`ops.outbox_event(catalog.product.status.changed)`。
  5. 额外完成 `curl` 实测联调：真实启动 `platform-core`（本机 endpoint 覆盖）后执行 CAT-024 HTTP 链路并校验数据库计数，再清理测试数据。
- 涉及文件：
  - `apps/platform-core/src/modules/catalog/tests/cat024_catalog_listing_review_db.rs`
  - `apps/platform-core/src/modules/catalog/tests/mod.rs`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `CATALOG_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core cat024_ -- --nocapture`
  4. `make up-local`
  5. `APP_PORT=18080 APP_HOST=127.0.0.1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9092 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9092 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core`
  6. `curl` 联调链路（创建商品/sku、metadata-profile、quality-report、data-contract、submit、review approve/reject、freeze）+ `psql` 数据库证据校验。
- 验证结果：
  - `cargo test -p platform-core`：通过（`82 passed, 0 failed, 1 ignored`）。
  - `cat024_` DB smoke：通过（`1 passed, 0 failed`）。
  - `curl` 联调结果：`submit_a=pending_review review_a=listed freeze_a=frozen submit_b=pending_review review_b=draft`。
  - DB 证据：`quality_count=1 contract_count=1 audit_submit=2 audit_review=2 audit_suspend=1`。
- 覆盖的冻结文档条目：
  - `docs/领域模型/全量领域模型与对象关系说明.md`（4.2 目录与商品聚合）
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5. V1 接口）
  - `docs/业务流程/业务流程图-V1-完整版.md`（4.2 商品创建、模板绑定与上架流程）
  - `docs/开发准备/测试用例矩阵正式版.md`（接口 + 集成 + 联调证据）
- 覆盖的任务清单条目：`CAT-024`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：`V1-Core-人工审批记录.md` 按你的要求继续由你手工维护，本批未自动写入。

### BATCH-108（待审批）

- 状态：通过
- 当前任务编号：CAT-025
- 当前批次目标：生成 `docs/02-openapi/catalog.yaml` 第一版并与当前 Catalog 实现完成一致性校验。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并通过审批；`BATCH-107` 已通过，可继续执行。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `CAT-025` 交付物、DoD、acceptance、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版任务解释，与 CSV 一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按固定流程执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持 V1 冻结边界，不扩展功能。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：先登记计划中，再补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：仅读规则，继续由人工维护。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对目录/商品聚合与流程基线。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 catalog 模块边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 V1 接口冻结口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认审计/事件边界不变。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持统一错误码约束。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐文档与接口联调验证证据。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按既有文档目录落盘 OpenAPI。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调基于本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：用环境变量启动服务验证。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + PostgreSQL + Axum + Kafka。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体，不新增架构动作。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L200`：OpenAPI 覆盖目录与商品聚合相关对象。
  2. `docs/数据库设计/接口协议/目录与商品接口协议正式版.md:L82`：OpenAPI 路径与请求头/核心字段口径一致。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L86`：接口覆盖创建、提交、审核、冻结等流程节点。
- 已实现功能：
  1. 新增交付文件 `docs/02-openapi/catalog.yaml`（由 `packages/openapi/catalog.yaml` 同步生成第一版）。
  2. 完成 OpenAPI 与实现路由路径/方法一致性校验（`apps/platform-core/src/modules/catalog/router.rs` 对比 `docs/02-openapi/catalog.yaml`）。
  3. 完成手工 API 联调：`GET /api/v1/catalog/standard-scenarios`，返回 5 条标准场景并验证审计落库。
- 涉及文件：
  - `docs/02-openapi/catalog.yaml`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cmp -s docs/02-openapi/catalog.yaml packages/openapi/catalog.yaml`
  2. OpenAPI vs router 一致性脚本校验（路径/方法缺失与漂移检查）。
  3. `cargo fmt --all`
  4. `cargo test -p platform-core`
  5. `APP_PORT=18080 ... cargo run -p platform-core`
  6. `curl -X GET /api/v1/catalog/standard-scenarios` + `psql` 审计计数核验。
- 验证结果：
  - `catalog_openapi_synced=yes`（文档与包内规范一致）。
  - 路径/方法一致性脚本结果：`missing_paths=[] extra_paths=[] method_mismatch=[]`。
  - `cargo test -p platform-core`：通过（`82 passed, 0 failed, 1 ignored`）。
  - `curl` 联调：`scenario_count=5`、`scenario_codes=S1,S2,S3,S4,S5`、`audit_count=1`（request_id=`req-cat025-openapi-1776509305`）。
- 覆盖的冻结文档条目：
  - `docs/领域模型/全量领域模型与对象关系说明.md`（4.2 目录与商品聚合）
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5. V1 接口）
  - `docs/业务流程/业务流程图-V1-完整版.md`（4.2 商品创建、模板绑定与上架流程）
- 覆盖的任务清单条目：`CAT-025`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：通过
- 备注：`audit.audit_event` 为 append-only，联调验证后未执行审计数据删除；`V1-Core-人工审批记录.md` 继续由你手工维护。

### BATCH-109（计划中）

- 状态：计划中
- 当前任务编号：CAT-026
- 当前批次目标：生成 `docs/05-test-cases/catalog-review-cases.md`，覆盖上架规则、字段缺失、模板不匹配、风险阻断，并补齐接口联调验证证据。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并通过审批；`BATCH-108` 已通过，可继续执行。
- 预计涉及文件：
  - `docs/05-test-cases/catalog-review-cases.md`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 待人工审批结论：待审批

### BATCH-109（待审批）

- 状态：待审批
- 当前任务编号：CAT-026
- 当前批次目标：生成 `docs/05-test-cases/catalog-review-cases.md`，覆盖上架规则、字段缺失、模板不匹配、风险阻断，并提供联调验证证据。
- 前置依赖核对结果：`CORE-001; CORE-004; CORE-005; CORE-006; DB-004; DB-005` 已完成并通过审批；`BATCH-108` 已通过，可继续执行。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `CAT-026` 的 DoD/acceptance/technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版任务描述与 CSV 一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中 -> 编码 -> 验证 -> 待审批”流程执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持 V1 边界，不扩展任务。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：先记录计划中，再追加待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次更新，无新增 TODO。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：仅读规则，继续由人工维护。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认目录与审核主流程口径。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 catalog 模块职责边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认接口冻结面与请求头约束。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：保持事件与主状态边界不变。
  12. `docs/开发准备/统一错误码字典正式版.md`：校验错误码口径（`CAT_VALIDATION_FAILED` / `TRD_STATE_CONFLICT`）。
  13. `docs/开发准备/测试用例矩阵正式版.md`：按用例矩阵方式组织场景。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：文档落盘于 `docs/05-test-cases`。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：使用 core 栈联调。
  16. `docs/开发准备/配置项与密钥管理清单.md`：使用环境变量启动应用。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + PostgreSQL + Axum + Kafka。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体，不新增架构动作。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L200`：用例覆盖 DataAsset/DataAssetVersion/Product/SKU 关联前置关系。
  2. `docs/数据库设计/接口协议/目录与商品接口协议正式版.md:L82`：用例覆盖 V1 商品/SKU/模板绑定/提交审核关键接口与错误路径。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L86`：用例覆盖“保存草稿 -> 提交审核 -> 审核通过/驳回 -> 风险阻断”流程节点。
- 已实现功能：
  1. 新增文档 `docs/05-test-cases/catalog-review-cases.md`。
  2. 文档包含 8 条核心用例，覆盖：上架规则、字段缺失、模板不匹配、风险阻断。
  3. 文档明确了断言细则（状态机、模板族校验、风险阻断）与建议执行顺序。
  4. 补充联调验证：执行风险阻断场景，确认 `submit` 返回 `TRD_STATE_CONFLICT`，且不产生 `catalog.product.submit` 成功审计事件。
- 涉及文件：
  - `docs/05-test-cases/catalog-review-cases.md`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务：`APP_PORT=18080 ... KAFKA_BROKERS=127.0.0.1:9094 ... cargo run -p platform-core`
  4. `curl` + `psql` 执行风险阻断联调：构造 `metadata.risk_blocked=true` 商品并调用 `POST /api/v1/products/{id}/submit`。
- 验证结果：
  - `cargo test -p platform-core`：通过（`82 passed, 0 failed, 1 ignored`）。
  - 风险阻断联调：`error_code=TRD_STATE_CONFLICT`，`message=product is blocked by risk policy`。
  - 审计校验：`audit_submit_count=0`（风险阻断下未产生成功提交审计）。
- 覆盖的冻结文档条目：
  - `docs/领域模型/全量领域模型与对象关系说明.md`（4.2 目录与商品聚合）
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5. V1 接口）
  - `docs/业务流程/业务流程图-V1-完整版.md`（4.2 商品创建、模板绑定与上架流程）
- 覆盖的任务清单条目：`CAT-026`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：待审批
- 备注：`V1-Core-人工审批记录.md` 继续由你手工维护，本批未自动写入。

### BATCH-110（计划中）

- 状态：计划中
- 当前任务编号：TRADE-001
- 当前批次目标：实现询报价/样例申请/POC 申请最小数据模型（order/contract/authorization）并补齐 `packages/openapi/trade.yaml`，提供最小 API 联调与审计留痕验证。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已在历史批次完成并通过审批；`BATCH-109` 已通过，可继续执行。
- 预计涉及文件：
  - `apps/platform-core/src/modules/order/**`
  - `apps/platform-core/src/modules/contract/**`
  - `apps/platform-core/src/modules/authorization/**`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 待人工审批结论：待审批

### BATCH-110（待审批）

- 状态：待审批
- 当前任务编号：TRADE-001
- 当前批次目标：实现询报价/样例申请/POC 申请最小数据模型（order/contract/authorization）并补齐 `packages/openapi/trade.yaml`，提供最小 API 联调与审计留痕验证。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成并通过审批；`BATCH-109` 已通过，可继续执行。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-001` 描述、DoD、acceptance、`technical_reference`。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版说明与 CSV 对齐。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：执行“计划中 -> 编码 -> 验证 -> TODO -> 待审批”闭环。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持 V1 范围，不抢跑 `TRADE-003` 下单接口。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：已先写计划中，再补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：已同步 `BATCH-110`，无新增 TODO。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：仅读规则；继续由人工维护。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对核心交易链路步骤（发现/撮合 -> 合同/授权 -> 下单前置）。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 trade/order/contract/authorization 在 `platform-core` 边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：核对 Trade 子域接口风格与 `/api/v1` 约束。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认事件非主状态、审计与 outbox 边界不变。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：执行单测 + 接口联调 + DB 证据三层验证。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按模块目录拆分实现，避免单文件膨胀。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432` 与 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：使用 `DATABASE_URL/KAFKA_BROKERS/MINIO_ENDPOINT/OPENSEARCH_ENDPOINT` 启动联调。
  17. `docs/开发准备/技术选型正式版.md`：遵循 PostgreSQL 主状态、Kafka 事件分发原则。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体，交易对象围绕 Order/Contract/Authorization。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：在订单前置对象层引入 `rfq/sample_request/poc_request` typed payload，保留 Inquiry->Order 前置结构。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：对齐“步骤4发现与撮合”到“步骤5合同与授权”前的交易前置请求语义。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：对齐“买方搜索、选购与下单流程”中的询价/样例/POC 前置动作。
- 已实现功能：
  1. 在 `order` 模块实现 `PreOrderRequestKind`（`rfq/sample_request/poc_request`）、细节对象、前置请求聚合对象。
  2. 在 `contract` 模块补齐 `ContractExpectationSnapshot` 最小模型；在 `authorization` 模块补齐 `AuthorizationExpectationSnapshot` 最小模型。
  3. 新增 `POST /api/v1/trade/pre-requests`、`GET /api/v1/trade/pre-requests/{id}`，落库到 `trade.inquiry` 并写入 `audit.audit_event`。
  4. 新增 `order` 仓储读写层：把 typed payload 序列化到 `trade.inquiry.message_text`，读取时反序列化回领域对象。
  5. 更新 `packages/openapi/trade.yaml`：新增 pre-request 路径与 schema，并保留 `TRADE-030` 现有结构。
  6. 新增测试：权限拒绝测试 + `TRADE_DB_SMOKE` 条件下的 DB 烟测（创建/查询/落库/审计）。
- 涉及文件：
  - `apps/platform-core/src/lib.rs`
  - `apps/platform-core/src/modules/order/mod.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/domain/mod.rs`
  - `apps/platform-core/src/modules/order/domain/payment_state.rs`
  - `apps/platform-core/src/modules/order/domain/pre_request.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/dto/pre_request.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/repo/pre_request_repository.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade001_pre_request_db.rs`
  - `apps/platform-core/src/modules/contract/mod.rs`
  - `apps/platform-core/src/modules/contract/domain/mod.rs`
  - `apps/platform-core/src/modules/authorization/mod.rs`
  - `apps/platform-core/src/modules/authorization/domain/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动联调服务：
     `APP_PORT=18080 APP_HOST=127.0.0.1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core`
  4. `curl` 创建前置请求（`POST /api/v1/trade/pre-requests`）
  5. `curl` 查询前置请求（`GET /api/v1/trade/pre-requests/{id}`）
  6. `psql` 校验 `trade.inquiry` 落库与 `audit.audit_event` 审计记录。
- 验证结果：
  - `cargo test -p platform-core`：通过（`84 passed, 0 failed, 1 ignored`）。
  - API 联调：创建与查询均返回 `success=true`。
  - 样例联调证据：
    - `request_id=req-trade001-1776513669`
    - `inquiry_id=97429e46-16af-4ec6-8d41-76ac5a04e64b`
    - `request_kind=poc_request`
    - `trade.inquiry.status=open`
    - `trade.inquiry.message_text` 包含 `request_kind/details/contract_expectation/authorization_expectation` 完整 JSON
    - `audit.audit_event` 计数（create+read）=`2`
  - 说明：首次联调使用了不存在的产品/用户种子 ID 导致 `TRD_STATE_CONFLICT`，已切换到现有有效 seed 数据并完成通过验证。
- 覆盖的冻结文档条目：
  - `docs/领域模型/全量领域模型与对象关系说明.md`（4.4 交易与订单聚合）
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`（15 核心交易链路）
  - `docs/业务流程/业务流程图-V1-完整版.md`（4.3 买方搜索、选购与下单流程）
  - `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`（Trade 接口风格与版本路径）
- 覆盖的任务清单条目：`TRADE-001`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 待人工审批结论：待审批
- 备注：`V1-Core-人工审批记录.md` 按你的要求继续由你手工维护，本批未自动写入。
