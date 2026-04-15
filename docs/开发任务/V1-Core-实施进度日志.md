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
- 新增 TODO / 预留项：无
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

- 状态：计划中
- 当前任务编号：CTX-001, CTX-002, CTX-003, CTX-004
- 当前批次目标：冻结 V1-Core 执行上下文，完成阅读索引、V1 核心守则、V1 范围边界和架构风格文档落盘，作为后续 BOOT/ENV/CORE 的统一前置约束。
- 前置依赖核对结果：四个任务 `depends_on` 均为空；当前仓库仅存在示例审批记录，未发现真实批次阻塞。
- 预计涉及文件：`docs/00-context/reading-index.md`、`docs/00-context/v1-core-guardrails.md`、`docs/00-context/v1-core-scope.md`、`docs/00-context/architecture-style.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 对照 CSV 核验四个任务交付路径与完成定义；2. 校验文档内容与 `V1` 全集成基线、技术选型、服务边界一致；3. 校验文档可被后续任务引用。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`CTX-001`, `CTX-002`, `CTX-003`, `CTX-004`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：严格按单批次执行，完成后进入人工审批等待。

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

- 状态：计划中
- 当前任务编号：CTX-005, CTX-006, CTX-007, CTX-008
- 当前批次目标：冻结生命周期主对象、V1 标准 SKU 真值、首批五条标准链路映射和运行模式边界，为后续域模型与交易编排任务提供统一约束。
- 前置依赖核对结果：四个任务 `depends_on` 均为空；用户已明确上一批审批通过，可进入本批次。
- 预计涉及文件：`docs/00-context/lifecycle-objects.md`、`docs/00-context/standard-sku-truth.md`、`docs/00-context/first-5-scenarios.md`、`docs/00-context/run-modes.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 校验交付文件存在；2. 对照 CSV 描述核验对象、SKU、链路、运行模式四个维度是否完整覆盖；3. 对照冻结文档条目核验术语一致性。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`、`领域模型/全量领域模型与对象关系说明.md`、`业务流程/业务流程图-V1-完整版.md`、`开发准备/技术选型正式版.md`
- 覆盖的任务清单条目：`CTX-005`, `CTX-006`, `CTX-007`, `CTX-008`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次仅做文档冻结，不引入 V2/V3 正式实现。

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

- 状态：计划中
- 当前任务编号：CTX-009, CTX-010, CTX-011, CTX-012
- 当前批次目标：冻结外部 Provider 适配边界、上链异步链路、搜索推荐边界与安全审计最低基线，确保后续实现不越界且具备可审计性。
- 前置依赖核对结果：四个任务 `depends_on` 均为空；用户确认上一批审批通过，可继续执行。
- 预计涉及文件：`docs/00-context/provider-boundary.md`、`docs/00-context/async-chain-write.md`、`docs/00-context/search-and-recommend-boundary.md`、`docs/00-context/security-and-audit-floor.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 校验交付文件存在；2. 对照 CSV 任务描述核验四类边界约束是否齐全；3. 对照冻结文档核验术语和职责边界一致性。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/事件模型与Topic清单正式版.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`权限设计/接口权限校验清单.md`
- 覆盖的任务清单条目：`CTX-009`, `CTX-010`, `CTX-011`, `CTX-012`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次仅做边界冻结文档，不引入实现代码变更。

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

- 状态：计划中
- 当前任务编号：BOOT-003, BOOT-004, BOOT-005, BOOT-006, BOOT-007, BOOT-008, BOOT-009, BOOT-010, BOOT-011
- 当前批次目标：一次性完成 BOOT-003~011 的仓库基础能力收口，包括根 Makefile、统一脚本入口、多语言工作区规范、共享配置与 OpenAPI 骨架、DB 迁移目录规则、runbook 模板、fixtures 样例、CI workflow 占位。
- 前置依赖核对结果：上述 9 个任务依赖 `CTX-001/004/008/013/014`，均已完成且审批通过。
- 预计涉及文件：`Makefile`、`scripts/*.sh`、`docs/01-architecture/multi-language-workspace.md`、`packages/shared-config/**`、`packages/openapi/**`、`db/migrations/v1/**`、`docs/04-runbooks/**`、`fixtures/**`、`.github/workflows/*.yml`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 文件存在性与可执行性检查；2. 对照 CSV 的 9 个任务逐条核验交付路径与内容；3. 基本命令自检（Makefile target 可解析、workflow YAML 语法可读）。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/仓库拆分与目录结构建议.md`、`开发准备/配置项与密钥管理清单.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/测试用例矩阵正式版.md`
- 覆盖的任务清单条目：`BOOT-003`, `BOOT-004`, `BOOT-005`, `BOOT-006`, `BOOT-007`, `BOOT-008`, `BOOT-009`, `BOOT-010`, `BOOT-011`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：待审批
- 备注：按用户要求将 BOOT-001~036 改为 9 个任务一组推进。

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

- 状态：计划中
- 当前任务编号：BOOT-012, BOOT-013, BOOT-014, BOOT-015, BOOT-016, BOOT-017, BOOT-018, BOOT-019, BOOT-020
- 当前批次目标：一次性完成 BOOT-012~020 的工程治理文档收口：错误码映射、日志字段、模块依赖规则、Issue/PR 模板、ownership 矩阵、顶层目录 README、命名规范、版本策略、发布策略。
- 前置依赖核对结果：上述 9 个任务依赖 `CTX-001/004/008/013/014`，均已完成且审批通过。
- 预计涉及文件：`docs/01-architecture/error-codes.md`、`docs/01-architecture/logging-fields.md`、`docs/01-architecture/module-dependency-rules.md`、`docs/01-architecture/issue-template.md`、`docs/01-architecture/pr-template.md`、`docs/01-architecture/ownership-matrix.md`、`docs/01-architecture/naming-conventions.md`、`docs/01-architecture/versioning-policy.md`、`docs/01-architecture/release-policy.md`、顶层目录 `README.md` 补齐文件、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 文件存在性检查；2. 对照 CSV `BOOT-012~020` 检查每项交付覆盖；3. 顶层目录 README 覆盖检查。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`开发准备/测试用例矩阵正式版.md`、`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`BOOT-012`, `BOOT-013`, `BOOT-014`, `BOOT-015`, `BOOT-016`, `BOOT-017`, `BOOT-018`, `BOOT-019`, `BOOT-020`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：待审批
- 备注：按 BOOT 分组规则一次性推进，不在组内拆分审批。

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

- 状态：计划中
- 当前任务编号：CTX-022, CTX-023, CTX-024
- 当前批次目标：盘点当前仓库资产、本地部署与运维资产、任务与冻结文档基线差异，形成 `exists / partial / missing` 与“可复用/迁移/重写/去重”清单，为 `CTX-014` 提供前置输入。
- 前置依赖核对结果：三项任务共同依赖 `CTX-001`，已完成并通过审批；用户确认上一批审批通过，可继续执行。
- 预计涉及文件：`docs/00-context/current-repo-assets.md`、`docs/00-context/current-local-stack-assets.md`、`docs/00-context/current-task-baseline-gap.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 盘点实际目录与关键文件；2. 按任务要求输出分级清单；3. 对照 CSV `CTX-022~CTX-024` 检查交付覆盖完整性。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`、`开发任务/README.md`、`开发前设计文档/README.md`
- 覆盖的任务清单条目：`CTX-022`, `CTX-023`, `CTX-024`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次仅输出资产盘点与差异清单，不推进业务实现代码。

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

- 状态：计划中
- 当前任务编号：CTX-013, CTX-014, CTX-015
- 当前批次目标：冻结任务 ownership 策略、输出当前仓库与目标基线差距分析、冻结 V1 退出标准，形成后续 BOOT/ENV/CORE 的执行与验收门槛。
- 前置依赖核对结果：`CTX-013`/`CTX-015` 无依赖；`CTX-014` 依赖 `CTX-022; CTX-023; CTX-024`，已完成且用户确认审批通过。
- 预计涉及文件：`docs/00-context/ownership-strategy.md`、`docs/00-context/current-gap-analysis.md`、`docs/00-context/v1-exit-criteria.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 对照 CSV 描述逐项核验三份文档内容；2. 校验 `CTX-014` 是否引用前序盘点结论；3. 核验退出标准是否覆盖闭环、指标和审计检查要求。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`全集成文档/数据交易平台-全集成基线-V1.md`、`领域模型/全量领域模型与对象关系说明.md`、`开发准备/测试用例矩阵正式版.md`
- 覆盖的任务清单条目：`CTX-013`, `CTX-014`, `CTX-015`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次仅做上下文与验收边界冻结，不推进业务代码实现。

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

- 状态：计划中
- 当前任务编号：CTX-016, CTX-017, CTX-018
- 当前批次目标：补齐术语统一、专题文档到模块映射、V1 非目标清单，降低后续实现阶段命名漂移和范围外开发风险。
- 前置依赖核对结果：三项任务在 CSV 中 `depends_on` 为空；用户确认上一批审批通过，可继续执行。
- 预计涉及文件：`docs/00-context/term-glossary.md`、`docs/00-context/doc-to-module-map.md`、`docs/00-context/non-goals.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 校验三份交付文件存在；2. 检查术语覆盖与命名一致性；3. 检查 PRD 到模块/migration/OpenAPI/测试域映射完整性；4. 检查非目标清单与 V1 边界一致。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成映射索引.md`、`全集成文档/数据交易平台-全集成基线-V1.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`、`开发准备/测试用例矩阵正式版.md`
- 覆盖的任务清单条目：`CTX-016`, `CTX-017`, `CTX-018`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次为文档补齐，不引入实现代码改动。

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

- 状态：计划中
- 当前任务编号：CTX-021
- 当前批次目标：输出 V1-Core 闭环矩阵（8 SKU × 5 标准链路 × 合同/授权/交付/验收/计费/退款/争议/审计），明确每个交叉点的主触发点、状态推进点、证据对象、测试入口。
- 前置依赖核对结果：`CTX-021` 依赖 `CTX-006; CTX-007; CTX-015`，均已完成且获审批通过；`CTX-019/CTX-020` 依赖 `BOOT-002/ENV-001` 未完成，当前不可执行，先执行可落地任务。
- 预计涉及文件：`docs/00-context/v1-closed-loop-matrix.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 校验矩阵覆盖 5 条链路与 8 SKU；2. 校验每个交叉点均含“主触发点/状态推进点/证据对象/测试入口”；3. 对照 CSV 与 V1 冻结文档核验一致性。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`全集成文档/数据交易平台-全集成基线-V1.md`、`业务流程/业务流程图-V1-完整版.md`、`开发准备/测试用例矩阵正式版.md`
- 覆盖的任务清单条目：`CTX-021`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：当前批次为单任务文档冻结，完成后等待人工审批。

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

- 状态：计划中
- 当前任务编号：BOOT-021, BOOT-022, BOOT-023, BOOT-024
- 当前批次目标：完成仓库基础初始化收口：根 README、仓库元文件（ignore/editorconfig/gitattributes）、环境样例文件、增量初始化说明文档。
- 前置依赖核对结果：四项任务共同依赖 `CTX-014`，已完成并审批通过。
- 预计涉及文件：`README.md`、`.gitignore`、`.editorconfig`、`.gitattributes`、`.env.example`、`.env.local.example`、`.env.staging.example`、`.env.demo.example`、`docs/01-architecture/repo-init-notes.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 对照 CSV 核验交付路径文件存在；2. 检查命名与技术选型/目录边界一致；3. 校验环境样例不含真实密钥且与本地部署变量口径一致。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/技术选型正式版.md`、`开发准备/配置项与密钥管理清单.md`、`开发准备/本地开发环境与中间件部署清单.md`
- 覆盖的任务清单条目：`BOOT-021`, `BOOT-022`, `BOOT-023`, `BOOT-024`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次优先复用现有资产，以增量修改为主。

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

- 状态：计划中
- 当前任务编号：BOOT-029, BOOT-030, BOOT-031, BOOT-032
- 当前批次目标：校准 `apps/`、`services/`、`workers/`、`packages/` 的目录边界与命名落位，形成可引用的目录约束基础。
- 前置依赖核对结果：四项任务共同依赖 `CTX-014`，已完成且审批通过。
- 预计涉及文件：`apps/`、`services/`、`workers/`、`packages/`、`docs/01-architecture/service-worker-package-layout.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 检查目标目录存在；2. 校验目录内边界说明覆盖任务要求的服务/worker/package 列表；3. 对照 CSV `BOOT-029~032` 核验交付。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/服务清单与服务边界正式版.md`、`开发准备/技术选型正式版.md`、`开发准备/接口清单与OpenAPI-Schema冻结表.md`
- 覆盖的任务清单条目：`BOOT-029`, `BOOT-030`, `BOOT-031`, `BOOT-032`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批以增量校准为主，不迁移既有业务代码。

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

- 状态：计划中
- 当前任务编号：BOOT-033, BOOT-034, BOOT-035, BOOT-036
- 当前批次目标：收敛 `db/`、`infra/`、`docs/`、`fixtures/` 与 `.github/workflows` 目录边界，为后续迁移、runbook、CI 与测试资产提供统一落位。
- 前置依赖核对结果：四项任务共同依赖 `CTX-014`，已完成且审批通过。
- 预计涉及文件：`db/`、`infra/`、`docs/`、`fixtures/`、`.github/workflows/`、`docs/01-architecture/repo-layout.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 检查目标目录存在与命名；2. 校验目录说明文件覆盖任务要求；3. 对照 CSV `BOOT-033~036` 逐条核验。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/测试用例矩阵正式版.md`
- 覆盖的任务清单条目：`BOOT-033`, `BOOT-034`, `BOOT-035`, `BOOT-036`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批仅做目录收敛与文档说明，不迁移已有业务实现。

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

- 状态：计划中
- 当前任务编号：BOOT-001, BOOT-002
- 当前批次目标：完成 BOOT 父任务收口，校验根目录基础文件与目录骨架完整性，并产出 `docs/01-architecture/repo-layout.md` 作为后续唯一目录参考。
- 前置依赖核对结果：`BOOT-001` 依赖 `BOOT-021~024` 已完成；`BOOT-002` 依赖 `BOOT-029~036` 已完成；用户确认前批已审批通过。
- 预计涉及文件：`README.md`、`.gitignore`、`.editorconfig`、`.gitattributes`、`.env.example`、`.env.local.example`、`.env.staging.example`、`.env.demo.example`、`docs/01-architecture/repo-layout.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 核验 BOOT-001 交付文件齐备且内容有效；2. 核验 BOOT-002 目录树与当前仓库一致；3. 校验 `repo-layout.md` 可作为后续任务唯一路径参考。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/仓库拆分与目录结构建议.md`、`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`BOOT-001`, `BOOT-002`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次以收口校验与目录文档固化为主，不新增业务实现。

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

- 状态：计划中
- 当前任务编号：ENV-002, ENV-003, ENV-004, ENV-005
- 当前批次目标：完成本地 compose 正式落位与 PostgreSQL 初始化目录收口，建立统一 network/volume/healthcheck 策略、override 示例、环境变量样例以及 initdb 脚本目录。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/docker-compose.local.yml`、`infra/docker/docker-compose.local.override.example.yml`、`infra/docker/.env.local`、`infra/postgres/initdb/`、`scripts/check-local-env.sh`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 校验四项交付路径文件存在；2. `docker compose -f infra/docker/docker-compose.local.yml config` 语法校验；3. 校验 `infra/postgres/initdb` 脚本可执行并包含 schema/role/init 逻辑。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/配置项与密钥管理清单.md`、`开发准备/仓库拆分与目录结构建议.md`
- 覆盖的任务清单条目：`ENV-002`, `ENV-003`, `ENV-004`, `ENV-005`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批仅推进 `ENV-002~005`，其他 ENV 任务不提前实现。

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

- 状态：计划中
- 当前任务编号：ENV-006, ENV-007, ENV-008, ENV-009
- 当前批次目标：完成 PostgreSQL 扩展启用与配置收口、DB 就绪检查脚本、Kafka KRaft 片段与 topic 初始化脚本，确保本地中间件链路可自检。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/postgres/postgresql.conf`、`infra/postgres/pg_hba.conf`、`db/scripts/check-db-ready.sh`、`infra/kafka/docker-compose*.yml`、`infra/kafka/*.sh`、`scripts/**`、`docs/04-runbooks/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 交付路径文件存在性检查；2. compose 语法检查；3. 脚本可执行与静态检查；4. 核验扩展/schema 初始化与 topic 初始化脚本内容。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/事件模型与Topic清单正式版.md`、`数据库设计/数据库设计总说明.md`
- 覆盖的任务清单条目：`ENV-006`, `ENV-007`, `ENV-008`, `ENV-009`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次仅推进 `ENV-006~009`，其余 ENV 条目保持未执行。

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

- 状态：计划中
- 当前任务编号：ENV-010, ENV-011, ENV-012, ENV-013
- 当前批次目标：完善 Kafka topic 初始化能力与本地默认策略文档，补齐 Redis 基础配置并冻结 key 命名模式，确保本地缓存/消息中间件口径可复用。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/kafka/init-topics.sh`、`docs/04-runbooks/kafka-topics.md`、`infra/redis/redis.conf`、`docs/04-runbooks/redis-keys.md`、`infra/docker/.env.local`、`infra/docker/docker-compose.local.yml`、`docs/04-runbooks/local-startup.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 交付路径存在性检查；2. compose 语法检查；3. 脚本语法检查；4. 文档关键字段覆盖检查（consumer group、DLQ、retention、cleanup policy、redis key 模式）。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/事件模型与Topic清单正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`
- 覆盖的任务清单条目：`ENV-010`, `ENV-011`, `ENV-012`, `ENV-013`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次仅推进 `ENV-010~013`。

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

- 状态：计划中
- 当前任务编号：ENV-014, ENV-015, ENV-016, ENV-017
- 当前批次目标：完成 MinIO bucket 与初始化脚本收口，完成 OpenSearch 模板/索引/别名初始化脚本，确保对象存储与搜索索引可一键初始化并可验证。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/minio/*`、`infra/opensearch/*`、`scripts/*`、`docs/04-runbooks/*`、`fixtures/local/*`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 交付路径文件存在性检查；2. 脚本语法检查；3. 脚本实跑（MinIO/OpenSearch 初始化）；4. 初始化结果查询校验（bucket/index/alias）。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/事件模型与Topic清单正式版.md`
- 覆盖的任务清单条目：`ENV-014`, `ENV-015`, `ENV-016`, `ENV-017`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次按“静态+实跑”双重校验执行。

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

- 状态：计划中
- 当前任务编号：ENV-018, ENV-019, ENV-020, ENV-021
- 当前批次目标：完成 Keycloak realm import 机制与平台本地 realm 占位文件，完善 Mock Payment Provider 容器与可触发的成功/失败/超时/退款/人工打款场景，并补齐 runbook。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/keycloak/**`、`infra/docker/docker-compose.local.yml`、`infra/mock-payment/**`、`docs/04-runbooks/mock-payment.md`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 交付路径存在性检查；2. compose 语法检查；3. 新增脚本/映射静态检查；4. 容器实跑检查（realm 可访问、mock-payment 场景可触发）。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`领域模型/全量领域模型与对象关系说明.md`
- 覆盖的任务清单条目：`ENV-018`, `ENV-019`, `ENV-020`, `ENV-021`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次按“静态+实跑”校验并在当前分支做本地 commit。

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

- 状态：计划中
- 当前任务编号：ENV-022, ENV-023, ENV-024, ENV-025
- 当前批次目标：接入 Fabric 本地测试网络与脚本包装，补充最小链码部署占位流程，并完成 OpenTelemetry Collector 统一采集转发配置与验证脚本。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/fabric/**`、`Makefile`、`infra/docker/docker-compose.local.yml`、`infra/otel/otel-collector-config.yaml`、`scripts/check-fabric-local.sh`、`scripts/check-otel-collector.sh`、`docs/04-runbooks/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：已完成代码草稿，当前补记计划后执行静态+实跑验证并按结果收口。
- 涉及文件：待验证后补充。
- 验证步骤：1. `docker compose ... config`；2. shell 脚本语法与可执行权限检查；3. Fabric 启停/通道/链码占位实跑；4. OTel Collector 启动与健康/指标端点校验；5. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`ENV-022`, `ENV-023`, `ENV-024`, `ENV-025`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本条为先前中断后的计划补记；后续以验证结果为准。

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

- 状态：计划中
- 当前任务编号：ENV-026, ENV-027, ENV-028, ENV-029
- 当前批次目标：补齐本地观测栈闭环，完成 Prometheus 抓取目标、Alertmanager 最小告警规则、Grafana 数据源与初始 dashboard，以及 Loki/Tempo 存储与保留策略配置。
- 前置依赖核对结果：四项任务共同依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/monitoring/**`、`infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`infra/grafana/**`、`scripts/**`、`docs/04-runbooks/**`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. Prometheus/Alertmanager/Grafana/Loki/Tempo 配置语法检查；2. `docker compose ... config`；3. 观测栈容器启动与健康检查；4. Prometheus targets/alerts、Grafana datasource/dashboard、Loki/Tempo 持久化目录核验；5. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`ENV-026`, `ENV-027`, `ENV-028`, `ENV-029`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次继续执行“静态+实跑”双重校验并按要求提交本地 commit。

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

- 状态：计划中
- 当前任务编号：ENV-030
- 当前批次目标：在本地 compose 中实现 `core`、`observability`、`fabric`、`demo` 四类 profile 机制，支持最小核心栈先启动并可按需叠加观测与链环境。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/docker-compose.local.yml`、`scripts/up-local.sh`、`docs/04-runbooks/**`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. compose 配置静态校验；2. `make up-local`（默认 core）并执行 `scripts/check-local-stack.sh core`；3. `fabric` profile 启动验证；4. profile 说明 runbook 校验。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`ENV-030`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次按复杂任务单批执行，完成后提交本地 commit。

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

- 状态：计划中
- 当前任务编号：ENV-031
- 当前批次目标：为基础服务补充 `curl`/`nc`/`mc`/`kcat`/`psql` 级别启动后自检，收敛到 `scripts/check-local-stack.sh` 统一入口。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`scripts/verify-local-stack.sh`、`scripts/check-local-stack.sh`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/**`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. shell 语法检查；2. `make up-local` 后执行 `scripts/check-local-stack.sh core/full`；3. 命令级自检项输出核验（psql/redis-cli/kcat or kafka-topics/mc/curl/nc）。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`
- 覆盖的任务清单条目：`ENV-031`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次按复杂任务单批执行，完成后提交本地 commit。

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

- 状态：计划中
- 当前任务编号：ENV-032
- 当前批次目标：建立 `make up-core`、`make up-observability`、`make up-fabric`、`make up-demo` 组合命令，支持 local/staging/demo 三套模式切换基础。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`Makefile`、`docs/04-runbooks/compose-profiles.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. `make` 命令入口静态检查；2. 依次执行 `make up-core`、`make up-observability`、`make up-fabric`、`make up-demo`；3. 执行 `scripts/check-local-stack.sh core/full` 验证。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`ENV-032`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次按复杂任务单批执行，完成后提交本地 commit。

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

- 状态：计划中
- 当前任务编号：ENV-033
- 当前批次目标：在 `docs/04-runbooks/local-startup.md` 明确本地启动顺序：基础设施 -> schema/migration -> seed -> 应用 -> 回执模拟。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`docs/04-runbooks/local-startup.md`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. runbook 结构和术语一致性检查；2. `make up-local`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/平台总体架构设计草案.md`
- 覆盖的任务清单条目：`ENV-033`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次为文档收口任务，命令保持与仓库现有入口一致。

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

- 状态：计划中
- 当前任务编号：ENV-034
- 当前批次目标：补充 `docker-compose.staging.example.yml` 占位文件，不部署真实生产资源，并明确后续 Helm/K8s 迁移组件映射关系。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`docker-compose.staging.example.yml`、`docs/04-runbooks/**`、`开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. `docker compose -f docker-compose.staging.example.yml config`；2. 映射文档一致性检查；3. `make up-local` + `scripts/check-local-stack.sh core` 基线验证。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`、`data_trading_blockchain_system_design_split/14-部署架构、容量规划与持续交付.md`
- 覆盖的任务清单条目：`ENV-034`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：本批次为“staging 示例编排 + K8s 映射占位”任务，不引入真实生产配置。

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

- 状态：计划中
- 当前任务编号：ENV-036
- 当前批次目标：新增 `scripts/prune-local.sh`，用于安全清理本地卷、网络、链状态与演示数据，避免误删其他开发项目容器。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`scripts/prune-local.sh`、`docs/04-runbooks/troubleshooting.md`、`scripts/README.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. `bash -n scripts/prune-local.sh`；2. `./scripts/prune-local.sh --dry-run`；3. `make up-local`；4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/本地开发环境与中间件部署清单.md`、`全集成文档/数据交易平台-全集成基线-V1.md`
- 覆盖的任务清单条目：`ENV-036`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：通过
- 备注：脚本默认 dry-run，只有显式 `--force` 才执行删除动作。

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

- 状态：计划中
- 当前任务编号：ENV-037, ENV-038, ENV-039, ENV-040
- 当前批次目标：一次性完成本地环境配置快照导出、端口矩阵文档、基础服务故障排查手册扩展与本地 smoke test 套件落地。
- 前置依赖核对结果：4 个任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`scripts/export-local-config.sh`、`scripts/smoke-local.sh`、`docs/04-runbooks/port-matrix.md`、`docs/04-runbooks/troubleshooting.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/**`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. 新增脚本语法与可执行验证；2. `make up-local` + `check-local-stack.sh core`；3. smoke test 套件至少一次成功；4. 文档路径与矩阵内容核对。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`全集成文档/数据交易平台-全集成基线-V1.md`
- 覆盖的任务清单条目：`ENV-037`, `ENV-038`, `ENV-039`, `ENV-040`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：待审批
- 备注：按“连续任务单批次”执行，完成后统一回报。

### BATCH-029

- 状态：待审批
- 当前任务编号：ENV-037, ENV-038, ENV-039, ENV-040
- 当前批次目标：一次性完成本地环境配置快照导出、端口矩阵文档、基础服务故障排查手册扩展与本地 smoke test 套件落地。
- 前置依赖核对结果：4 个任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`scripts/export-local-config.sh`、`scripts/smoke-local.sh`、`docs/04-runbooks/port-matrix.md`、`docs/04-runbooks/troubleshooting.md`、`docs/04-runbooks/local-startup.md`、`fixtures/local/**`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：新增 `scripts/export-local-config.sh`，将 compose 解析结果导出为只读快照（default/core/demo 三组）；新增 `docs/04-runbooks/port-matrix.md`，维护端口、URL、默认账号口径及初始 bucket/topic 矩阵；扩展 `docs/04-runbooks/troubleshooting.md`，分别补齐 PostgreSQL/Kafka/Keycloak/MinIO/OpenSearch/Fabric 启动失败诊断步骤；新增 `scripts/smoke-local.sh` 与 `fixtures/local/local-smoke-suite-manifest.json`，覆盖“数据库可迁移、bucket 已创建、realm 已导入、topic 已存在、Grafana 可登录、Mock Payment 可回调”六项 smoke 检查；同步更新 `local-startup.md` 与 `scripts/README.md`。
- 涉及文件：`scripts/export-local-config.sh`、`scripts/smoke-local.sh`、`docs/04-runbooks/port-matrix.md`、`docs/04-runbooks/troubleshooting.md`、`docs/04-runbooks/local-startup.md`、`scripts/README.md`、`fixtures/local/local-smoke-suite-manifest.json`、`fixtures/local/config-snapshots/.gitkeep`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 验证步骤：1. `bash -n scripts/export-local-config.sh scripts/smoke-local.sh scripts/prune-local.sh`；2. `./scripts/export-local-config.sh`；3. `make up-local`；4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`；5. `make up-demo`；6. `./infra/kafka/init-topics.sh`；7. `./infra/minio/init-minio.sh`；8. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh full`；9. `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`。
- 验证结果：通过。新增脚本语法检查通过；配置快照导出成功；`core`/`full` 自检均通过；smoke 套件六项检查全部通过。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/平台总体架构设计草案.md`、`全集成文档/数据交易平台-全集成基线-V1.md`（环境与可观测性约束）
- 覆盖的任务清单条目：`ENV-037`, `ENV-038`, `ENV-039`, `ENV-040`
- 未覆盖项：无
- 新增 TODO / 预留项：无
- 待人工审批结论：待审批
- 备注：Docker 相关验证命令在沙箱内不可达，已在沙箱外执行并通过。

### BATCH-030

- 状态：计划中
- 当前任务编号：ENV-041
- 当前批次目标：在 `fixtures/local/` 下补齐五条标准链路所需最小演示数据（企业主体、卖方、买方、产品、SKU、模板、订单、支付、交付样例）。
- 前置依赖核对结果：任务依赖 `BOOT-001/002/003/004`，均已完成且你已确认审批通过。
- 预计涉及文件：`fixtures/local/**`、`docs/04-runbooks/local-startup.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. fixtures JSON 语法检查；2. `make up-local`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`；4. smoke 至少一条链路可验证。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/技术选型正式版.md`、`开发准备/测试用例矩阵正式版.md`、`全集成文档/数据交易平台-全集成基线-V1.md`
- 覆盖的任务清单条目：`ENV-041`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：待审批
- 备注：当前批次按“复杂单任务”执行，完成后统一进入审批。

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
- 待人工审批结论：待审批
- 备注：Docker 相关命令在沙箱内不可达，已在沙箱外执行并通过。

### BATCH-031

- 状态：计划中
- 当前任务编号：ENV-044, ENV-045, ENV-046, ENV-047
- 当前批次目标：一次性完成 `docker-compose.local.yml` 的 PostgreSQL/Redis/Kafka/MinIO 服务块收敛，确保服务定义清晰且可被总 compose 聚合解析。
- 前置依赖核对结果：4 个任务均依赖 `BOOT-003; BOOT-004`，均已完成且你已确认审批通过。
- 预计涉及文件：`infra/docker/docker-compose.local.yml`、`docs/开发任务/V1-Core-TODO与预留清单.md`、`docs/开发任务/V1-Core-实施进度日志.md`
- 已实现功能：尚未开始，本条为计划记录。
- 涉及文件：待实现后补充。
- 验证步骤：1. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml config`；2. `make up-local`；3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`。
- 验证结果：待实现后补充。
- 覆盖的冻结文档条目：`开发准备/本地开发环境与中间件部署清单.md`、`开发准备/技术选型正式版.md`、`开发准备/事件模型与Topic清单正式版.md`
- 覆盖的任务清单条目：`ENV-044`, `ENV-045`, `ENV-046`, `ENV-047`
- 未覆盖项：待实现后补充。
- 新增 TODO / 预留项：待实现后补充。
- 待人工审批结论：待审批
- 备注：本批次按“连续 4 任务”执行，统一实现和统一汇报。

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
- 待人工审批结论：待审批
- 备注：Docker 相关命令在沙箱内不可达，已在沙箱外执行并通过。
