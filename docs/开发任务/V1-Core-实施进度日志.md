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

- 状态：待审批
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

- 状态：待审批
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

### BATCH-075（计划中）

- 状态：计划中
- 当前任务编号：BIL-005
- 当前批次目标：实现 `POST /api/v1/payments/webhooks/{provider}`，补齐签名占位、幂等、防重放、乱序保护、审计落地与最小验证。
- 前置依赖核对结果：`BIL-005` 依赖 `TRADE-003; TRADE-007; DB-007; ENV-020; CORE-008; CORE-009`，当前均已完成且审批通过。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/数据库设计/接口协议/支付域接口协议正式版.md`、`docs/原始PRD/支付、资金流与轻结算设计.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`

### BATCH-075（实施完成）

- 状态：待审批
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
- 待人工审批结论：待审批
- 备注：联调数据库容器 `luna-postgres-test`（`127.0.0.1:55432`），mock 支付容器 `datab-mock-payment-provider`（`127.0.0.1:8089`）。

### BATCH-076（计划中）

- 状态：计划中
- 当前任务编号：TRADE-030
- 当前批次目标：实现支付结果到订单推进编排器：支付成功推进到“已锁资/待交付”，支付失败推进到“支付失败待处理”，支付超时推进到“支付超时待补偿/待取消”，并保证状态不可倒退。
- 前置依赖核对结果：`TRADE-030` 依赖 `BIL-005; TRADE-007; CORE-014`，当前已满足（`BIL-005` 已实现待审批，`TRADE-007/CORE-014` 历史已完成并审批通过）。
- 涉及冻结文档：`docs/开发任务/v1-core-开发任务清单.csv`（单一任务源）、`docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/领域模型/全量领域模型与对象关系说明.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`、`docs/业务流程/业务流程图-V1-完整版.md`

### BATCH-076（实施完成）

- 状态：待审批
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
- 待人工审批结论：待审批
- 备注：联调数据库容器 `luna-postgres-test`（`127.0.0.1:55432`），mock 支付容器 `datab-mock-payment-provider`（`127.0.0.1:8089`）。
