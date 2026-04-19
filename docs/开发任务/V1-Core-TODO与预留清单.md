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

## 当前阻塞记录

| 编号 | 对应任务编号 | 类型 | 模块 | 文件路径 | 当前状态 | 原因 | 后续补齐条件 | 是否阻塞继续开发 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TODO-ENV-043-001 | ENV-043 | V1-gap | env-compose | `infra/docker/docker-compose.apps.local.example.yml` | closed | 已在 `BATCH-057` 补齐应用层 compose 占位文件，并完成 `docker compose config` 与本地自检/烟雾验证。 | 无；后续如进入应用容器化联调阶段，按该示例替换占位镜像为真实服务镜像。 | no |
| TODO-CTX-019-001 | CTX-019 | V1-gap | context | `docs/00-context/service-to-module-map.md` | closed | 任务清单要求的交付文件在仓库中缺失，导致 `CORE-032` 依赖文档基线不完整。 | 已在 `BATCH-050` 补齐 `docs/00-context/service-to-module-map.md` 并纳入审批。 | no |
| TODO-CTX-020-001 | CTX-020 | V1-gap | context | `docs/00-context/local-deployment-boundary.md` | closed | 任务清单要求的交付文件在仓库中缺失，导致本地部署边界冻结依据不完整。 | 已在 `BATCH-050` 补齐 `docs/00-context/local-deployment-boundary.md` 并纳入审批。 | no |
| TODO-DB-034-001 | DB-034 | V1-gap | db-seed | `db/seeds/031_sku_trigger_matrix.sql` | closed | 已在 `BATCH-078` 补齐 `BIL-023` 交付文档并完成 `031_sku_trigger_matrix.sql` + `verify-seed-031.sh` 落地，阻塞链解除。 | 无；后续若扩展 SKU，按同一矩阵表与文档双写规则追加并回归验证。 | no |
| TODO-PROC-BIL-001 | BIL-* | tech-debt | process-governance | `docs/开发任务/V1-Core-实施进度日志.md` | accepted | 历史执行顺序发生跨阶段偏移：在 IAM 阶段未完成前已进入并实现 `BIL-001~BIL-005` 与 `TRADE-030`。当前已获人工批准继续推进 `CAT~TRADE~DLV`，但必须保留该偏移追溯并在进入 BIL 阶段时执行一致性复核。 | 在 BIL 阶段完成“历史已实现任务 vs 冻结文档”逐条复核并补齐不一致项，形成专项审批记录后关闭。 | no |

## 当前非阻塞记录

| 编号 | 对应任务编号 | 类型 | 模块 | 文件路径 | 当前状态 | 原因 | 后续补齐条件 | 是否阻塞继续开发 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TODO-CORE-028-001 | CORE-028 | V1-gap | db | `apps/platform-core/crates/db/src/lib.rs` | closed | 已补齐 `OrderRepository` 的 PostgreSQL 实现与 `ORDER_REPOSITORY_BACKEND` 运行时切换装配。 | 无；后续仅需在真实业务表结构联调时补充更细粒度 SQL 回归样例。 | no |
| TODO-IAM-002-REPO-001 | IAM-002 | V1-gap | iam | `apps/platform-core/src/modules/iam/repository.rs` | closed | 初版实现为 API 直连 SQL，仓储层边界不清晰。 | 已在 `BATCH-084` 补齐 `PostgresIamRepository` 并由 API 复用，完成后关闭。 | no |
| TODO-IAM-003-JWT-001 | IAM-003 | V1-gap | iam/auth | `apps/platform-core/crates/auth/src/lib.rs` | closed | 仅有 mock token 解析，未满足 Keycloak claims 解析接入要求。 | 已在 `BATCH-084` 增加 `KeycloakClaimsJwtParser`（本地 claims 解析模式）并接入 `/api/v1/auth/me`。 | no |
| TODO-IAM-011-PATH-001 | IAM-011 | V1-gap | iam | `packages/openapi/iam.yaml` | closed | step-up 创建接口路径与冻结协议存在漂移。 | 已在 `BATCH-084` 补齐 `/api/v1/iam/step-up/challenges` 并保留 `/check` 兼容路径（deprecated）。 | no |
| TODO-IAM-016-TX-001 | IAM-016 | tech-debt | iam | `apps/platform-core/src/modules/iam/api.rs` | closed | 多个写接口存在“业务写入成功但审计写入失败导致接口失败”的事务一致性风险。 | 已在 `BATCH-084` 将 IAM 写接口统一改为“业务+审计同事务提交”。 | no |
| TODO-PROC-IAM-APPROVAL-001 | IAM-001~IAM-020 | tech-debt | process-governance | `docs/开发任务/V1-Core-人工审批记录.md` | closed | IAM 批次审批存在“口头通过”但审批文件缺少结构化条目，审计追溯不足。 | 已在 `BATCH-084` 补录 `BATCH-079~083` 审批记录条目。 | no |

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
- `BATCH-023`（`ENV-031`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-024`（`ENV-032`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-025`（`ENV-033`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-026`（`ENV-034`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-027`（`ENV-035`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-028`（`ENV-036`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-029`（`ENV-037`, `ENV-038`, `ENV-039`, `ENV-040`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-030`（`ENV-041`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-031`（`ENV-044`, `ENV-045`, `ENV-046`, `ENV-047`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-032`（`ENV-048`, `ENV-049`, `ENV-050`, `ENV-051`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-033`（`ENV-052`, `ENV-053`, `ENV-054`, `ENV-055`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-034`（`ENV-056`, `ENV-057`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-035`（`ENV-001`, `ENV-042`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-036`（`ENV-043`）：新增 `V1-gap` 阻塞项 `TODO-ENV-043-001`（`CORE-032` 前置未完成）。
- `BATCH-037`（`CORE-001`, `CORE-002`, `CORE-003`, `CORE-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-038`（`CORE-005`, `CORE-006`, `CORE-007`, `CORE-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-039`（`CORE-009`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-040`（`CORE-011`, `CORE-012`, `CORE-013`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-041`（`CORE-014`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-042`（`CORE-015`, `CORE-016`, `CORE-017`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-043`（`CORE-018`, `CORE-019`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-044`（`CORE-020`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-045`（`CORE-021`, `CORE-022`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-046`（`CORE-023`, `CORE-024`, `CORE-025`, `CORE-026`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-047`（`CORE-027`, `CORE-028`, `CORE-029`）：补记 `V1-gap` 项 `TODO-CORE-028-001`（非阻塞，追踪运行时持久化仓储接入）。
- `BATCH-048`（`CORE-030`, `CORE-031`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-049`（`CORE-033`, `CORE-034`, `CORE-035`, `CORE-036`）：新增阻塞项 `TODO-CTX-019-001`、`TODO-CTX-020-001`（历史前置任务交付文件缺失，影响后续依赖链）。
- `BATCH-050`（`CTX-019`, `CTX-020`）：补齐阻塞缺口并关闭 `TODO-CTX-019-001`、`TODO-CTX-020-001`。
- `BATCH-051`（`CORE-037`, `CORE-038`, `CORE-039`, `CORE-040`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-052`（`CORE-041`, `CORE-042`, `CORE-043`, `CORE-044`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-053`（`CORE-045`, `CORE-046`, `CORE-047`, `CORE-048`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-054`（`CORE-049`, `CORE-050`, `CORE-051`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-055`（`CORE-010`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-056`（`CORE-032`）：更新阻塞项 `TODO-ENV-043-001` 的阻塞原因与补齐条件，保持状态 `blocked`（待 `CORE-032` 审批通过后解除）。
- `BATCH-057`（`ENV-043`）：关闭阻塞项 `TODO-ENV-043-001`（已补齐 compose 占位文件并通过 compose config + 本地自检 + smoke 验证）。
- `BATCH-058`（`CORE-022`, `CORE-028`）：关闭 `TODO-CORE-028-001`（已补齐 PostgreSQL 仓储实现与运行时 DI 切换），并完成启动自检对 topic/bucket/alias 的存在性探测增强。
- `BATCH-059`（`DB-001`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-060`（`DB-002`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-061`（`DB-003`, `DB-004`, `DB-005`, `DB-006`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-062`（`DB-007`, `DB-008`, `DB-009`, `DB-010`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-063`（`DB-011`, `DB-012`, `DB-013`, `DB-014`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-064`（`DB-015`, `DB-016`, `DB-017`, `DB-018`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-065`（`DB-019`, `DB-020`, `DB-021`, `DB-022`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-066`（`DB-023`, `DB-024`, `DB-025`, `DB-026`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-067`（`DB-027`, `DB-028`, `DB-029`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-068`（`DB-030`, `DB-031`, `DB-032`, `DB-033`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-069`（`DB-035`）：新增阻塞项 `TODO-DB-034-001`（`DB-034` 依赖 `BIL-023` 未完成，触发强制暂停）。
- `BATCH-070`（`BIL-001`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-071`（`BIL-002`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-072`（`BIL-003`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-073`（`BIL-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-074`（`BIL-002`, `BIL-003` 返工）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-075`（`BIL-005`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-076`（`TRADE-030`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项。
- `BATCH-077`（流程纠偏：BIL 阶段冻结登记）：新增阻塞项 `TODO-PROC-BIL-001`，冻结“继续新增 BIL 任务”和“已完成 BIL 任务改动”，待 `IAM-001~IAM-020` 审批通过后再人工解冻。
- `BATCH-078`（`BIL-023`, `DB-034`）：关闭阻塞项 `TODO-DB-034-001`（已补齐 SKU 计费触发矩阵文档、`031` 种子与校验脚本并通过验证）；`TODO-PROC-BIL-001` 保持冻结，仅执行本次依赖解锁所需最小范围补齐。
- `BATCH-079`（`IAM-001`, `IAM-002`, `IAM-003`, `IAM-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-080`（`IAM-005`, `IAM-006`, `IAM-007`, `IAM-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-081`（`IAM-009`, `IAM-010`, `IAM-011`, `IAM-012`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-082`（`IAM-013`, `IAM-014`, `IAM-015`, `IAM-016`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-083`（`IAM-017`, `IAM-018`, `IAM-019`, `IAM-020`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项（`TODO-PROC-BIL-001` 继续保持冻结）。
- `BATCH-084`（`IAM-002`, `IAM-003`, `IAM-011`, `IAM-020` 缺口修复）：关闭 `TODO-IAM-002-REPO-001`、`TODO-IAM-003-JWT-001`、`TODO-IAM-011-PATH-001`、`TODO-IAM-016-TX-001`、`TODO-PROC-IAM-APPROVAL-001`。
- `BATCH-085`（`CAT-001`）：无新增 `V1-gap / V2-reserved / V3-reserved`；将 `TODO-PROC-BIL-001` 从 `blocked` 调整为 `accepted`，记录“已获人工批准继续后续阶段、进入 BIL 时必须执行一致性复核”的审计口径。
- `BATCH-086`（`CAT-002`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；保持 `TODO-PROC-BIL-001` 追溯约束不变。
- `BATCH-087`（`CAT-003`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-088`（`CAT-004`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-089`（`CAT-005`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-090`（`CAT-006`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-091`（`CAT-007`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-092`（`CAT-008`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-093`（`CAT-009`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-094`（`CAT-010`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-095`（`CAT-011`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-096`（`CAT-012`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-097`（`CAT-013`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-098`（`CAT-014`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-099`（`CAT-015`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-100`（`CAT-016`, `CAT-017`, `CAT-018`, `CAT-019`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-101`（`CAT-016~CAT-019` 审计修复）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-102`（`CAT-020`, `CAT-021`, `CAT-022`, `CAT-023`）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-103`（`CAT-020` 返工重做，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-104`（`CAT-021`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-105`（`CAT-022`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-106`（`CAT-023`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-107`（`CAT-024`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-108`（`CAT-025`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-109`（`CAT-026`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-110`（`TRADE-001`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-111`（`TRADE-002`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-112`（`TRADE-003`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-113`（`TRADE-004`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-114`（`TRADE-005`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-115`（`TRADE-006`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-116`（`TRADE-007`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-117`（`TRADE-008`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-118`（`TRADE-009`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-119`（`TRADE-010`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-120`（`TRADE-011`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-121`（`TRADE-012`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-122`（`TRADE-013`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-123`（`TRADE-014`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-124`（`TRADE-015`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-125`（`TRADE-016`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-126`（`TRADE-017`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-127`（`TRADE-018`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-128`（`TRADE-019`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-129`（`TRADE-020`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-130`（`TRADE-021`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-131`（`TRADE-022`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-132`（`TRADE-023`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-133`（`TRADE-024`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-134`（`TRADE-025`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；修复授权聚合查询中的联表歧义列后，`scope / subject / resource / action` 最小结构已在迁移结果、订单详情聚合与生命周期快照中稳定输出；`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-135`（`TRADE-026`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；合同确认链路已接入签章 provider 占位，`local/mock` 模式下通过 `provider-kit` 生成签章引用并持久化到 `contract.contract_signer.signature_digest`，`TODO-PROC-BIL-001` 追溯约束保持不变。
- `BATCH-136`（`TRADE-027`，单任务批次）：无新增 `V1-gap / V2-reserved / V3-reserved / tech-debt` 项；主交易链路集成 smoke 已覆盖下单、合同确认、锁资前校验、非法状态跳转、自动断权，并修正测试清理顺序以避免临时业务数据残留；`TODO-PROC-BIL-001` 追溯约束保持不变。
