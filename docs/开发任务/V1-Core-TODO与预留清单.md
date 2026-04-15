# V1-Core TODO 与预留清单

本文件用于汇总当前阶段未实现但已明确识别的缺口、技术债和 `V2/V3` 预留点。

## 记录规则

- 代码里出现的 `TODO(...)` 必须同步登记到本文件
- 每条记录都要标出是否阻塞继续开发
- 已完成补齐的 TODO 不删除，改状态为 `closed`

## 字段说明

- 编号
- 对应任务编号
- 类型
- 模块
- 文件路径
- 当前状态
- 原因
- 后续补齐条件
- 是否阻塞继续开发
- 计划补齐阶段
- 责任建议

---

## TODO 模板

| 编号 | 对应任务编号 | 类型 | 模块 | 文件路径 | 当前状态 | 原因 | 后续补齐条件 | 是否阻塞继续开发 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TODO-001 | TASK-ID | V1-gap / V2-reserved / V3-reserved / tech-debt | module-name | path/to/file | open | 简述原因 | 简述补齐条件 | yes / no |

## 示例记录

| 编号 | 对应任务编号 | 类型 | 模块 | 文件路径 | 当前状态 | 原因 | 后续补齐条件 | 是否阻塞继续开发 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TODO-EXAMPLE-001 | IAM-014 | V2-reserved | iam | `apps/platform-core/src/modules/iam/session.rs` | accepted | 当前阶段先落本地会话与设备信任基础能力，不实现企业级风险设备画像。 | 进入 `V2` 时补齐设备风险评分、异常登录画像与策略联动；需同步更新接口、错误码与审计事件。 | no |
| TODO-EXAMPLE-002 | AUD-009 | V1-gap | audit | `apps/platform-core/src/modules/audit/replay.rs` | open | 已完成回放入口，但尚未补齐回放结果的结构化差异摘要。 | 当前阶段必须补齐；补齐后需新增单测并更新实施日志。 | yes |

## 状态说明

- `open`：尚未处理
- `accepted`：已知缺口，当前允许继续
- `blocked`：阻塞继续开发
- `closed`：已补齐并验证

## 批次更新记录

- `BATCH-002`（`CTX-001`, `CTX-002`, `CTX-003`, `CTX-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-003`（`CTX-005`, `CTX-006`, `CTX-007`, `CTX-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-004`（`CTX-009`, `CTX-010`, `CTX-011`, `CTX-012`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-005`（`CTX-022`, `CTX-023`, `CTX-024`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-006`（`CTX-013`, `CTX-014`, `CTX-015`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-007`（`CTX-016`, `CTX-017`, `CTX-018`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-008`（`CTX-021`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-009`（`BOOT-021`, `BOOT-022`, `BOOT-023`, `BOOT-024`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-010`（`BOOT-029`, `BOOT-030`, `BOOT-031`, `BOOT-032`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-011`（`BOOT-033`, `BOOT-034`, `BOOT-035`, `BOOT-036`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-012`（`BOOT-001`, `BOOT-002`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-013`（`BOOT-003`, `BOOT-004`, `BOOT-005`, `BOOT-006`, `BOOT-007`, `BOOT-008`, `BOOT-009`, `BOOT-010`, `BOOT-011`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-014`（`BOOT-012`, `BOOT-013`, `BOOT-014`, `BOOT-015`, `BOOT-016`, `BOOT-017`, `BOOT-018`, `BOOT-019`, `BOOT-020`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-015`（`ENV-002`, `ENV-003`, `ENV-004`, `ENV-005`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-016`（`ENV-006`, `ENV-007`, `ENV-008`, `ENV-009`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-017`（`ENV-010`, `ENV-011`, `ENV-012`, `ENV-013`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-018`（`ENV-014`, `ENV-015`, `ENV-016`, `ENV-017`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-019`（`ENV-018`, `ENV-019`, `ENV-020`, `ENV-021`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-020`（`ENV-022`, `ENV-023`, `ENV-024`, `ENV-025`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-021`（`ENV-026`, `ENV-027`, `ENV-028`, `ENV-029`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-022`（`ENV-030`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
