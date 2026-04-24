# V1 Core Acceptance Checklist

`TEST-021` 的正式目标，是把 `V1` 退出标准收口为一份可执行、可回查、可签收的验收清单。本文不替代各专项 case 文档和 checker；相反，它要求最终验收只能复用这些官方入口，不得再临时拼一套第二执行面。

## Authority

- 任务与顺序：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 退出门槛：`docs/00-context/v1-exit-criteria.md`
- 标准链路与 SKU authority：`docs/全集成文档/数据交易平台-全集成基线-V1.md` `5.3.2 / 5.3.2A`
- Phase 1 验收标准：`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md` `15.2`
- 场景 / demo 真值：`fixtures/demo/scenarios.json`、`fixtures/demo/orders.json`
- 业务闭环映射：`docs/00-context/v1-closed-loop-matrix.md`
- 官方专项入口：`docs/05-test-cases/README.md`

## Exit Rule

只有在以下条件同时满足时，才允许宣告 `V1-Core` 完成：

1. 本文 `ACC-*` 强制 gate 全部通过。
2. 五条标准链路全部有正式证据，且命名、SKU、模板与 fixture / frozen docs 一致。
3. 八个标准 SKU 都具备：
   - 一条主路径证据
   - 一条异常或阻断路径证据
   - 一条退款或争议路径证据
4. 最终 sign-off 记录能够列出不少于 `20` 个唯一 `order_id`，且整个连续演示过程中无关键错误。
5. 任何一条 gate 缺少可复核 artifact、审计、DB 回查或官方 checker 结果，都视为未通过。

一票否决：

- 只通过主流程，异常/幂等/回滚/恢复缺失
- 用 mock 页面、固定 JSON、孤立表记录替代真实闭环
- 使用非官方 checker、第二套 topic catalog、第二套 SKU/场景命名
- 无法把搜索/推荐/链回执/outbox 最终回到 PostgreSQL 和审计证据上

## Closed-Loop Mapping

| 闭环 | 正式含义 | 必须落到的 gate |
| --- | --- | --- |
| 交易闭环 | 下单、合同、授权、交付、验收、计费、结算主链路完整闭合 | `ACC-CONTRACT`、`ACC-MIGRATION`、`ACC-LOCAL`、`ACC-SCENARIO`、`ACC-PROVIDER`、`ACC-OUTBOX`、`ACC-PAYMENT`、`ACC-DELIVERY` |
| 仲裁闭环 | 争议打开后冻结结算，裁决后退款或赔付重算入账 | `ACC-DISPUTE`、`ACC-ORCH-20ORDERS` |
| 评分闭环 | `S5` 查询评分 / 结果包场景与搜索推荐最终回 PostgreSQL 放行 | `ACC-SEARCHREC`、`ACC-SCENARIO`、`ACC-SKU-COVERAGE` |
| 审计闭环 | 审计留痕、导出、replay、canonical authority、恢复可追溯 | `ACC-AUDIT`、`ACC-REPLAY`、`ACC-CANONICAL`、`ACC-RECOVERY` |

## Scenario Checklist

| 场景 | 主 SKU | 补充 SKU | Demo authority | 必须证明的验收事实 |
| --- | --- | --- | --- | --- |
| `S1` 工业设备运行指标 API 订阅 | `API_SUB` | `API_PPU` | `product_id=20000000-0000-0000-0000-000000000309`、主订单 `34000000-0000-0000-0000-000000000001` | 周期订阅主路径成立；按量加购不替代主 SKU；API 授权、调用计量、验收、计费、争议与审计可联查 |
| `S2` 工业质量与产线日报文件包交付 | `FILE_STD` | `FILE_SUB` | `product_id=20000000-0000-0000-0000-000000000310`、主订单 `34000000-0000-0000-0000-000000000002` | 文件包交付、下载票据、签收验收、一次性计费成立；周期文件订阅作为补充 SKU 可独立联查 |
| `S3` 供应链协同查询沙箱 | `SBX_STD` | `SHARE_RO` | `product_id=20000000-0000-0000-0000-000000000311`、主订单 `34000000-0000-0000-0000-000000000003` | 沙箱工作区与只读共享都是真实交付对象；导出限制、撤权、争议冻结、审计必须成立 |
| `S4` 零售门店经营分析 API / 报告订阅 | `API_SUB` | `RPT_STD` | `product_id=20000000-0000-0000-0000-000000000312`、主订单 `34000000-0000-0000-0000-000000000004` | API 订阅主路径成立；月报/结果包作为补充 SKU 可独立交付、验收和计费 |
| `S5` 商圈/门店选址查询服务 | `QRY_LITE` | `RPT_STD` | `product_id=20000000-0000-0000-0000-000000000313`、主订单 `34000000-0000-0000-0000-000000000005` | 查询模板授权、评分输出、结果包交付都可验证；最终结果和评分不能绕过 PostgreSQL 与审计放行 |

## SKU Coverage Rule

| SKU | 主挂点场景 | 主路径 evidence | 异常 / 阻断 evidence | 退款 / 争议 evidence |
| --- | --- | --- | --- | --- |
| `FILE_STD` | `S2` | `TEST-006` + `TEST-022` | `TEST-012` 断权 / 票据失效 | `TEST-013` 或 `TEST-024` |
| `FILE_SUB` | `S2` 补充 | `TEST-023` | `TEST-012` | `TEST-013` 或 `TEST-024` |
| `SHARE_RO` | `S3` 补充 | `TEST-023` | `TEST-012` share revoke | `TEST-013` 或 `TEST-024` |
| `API_SUB` | `S1`、`S4` | `TEST-006` | `TEST-011` webhook / provider / step-up 异常 | `TEST-013` 或 `TEST-024` |
| `API_PPU` | `S1` 补充 | `TEST-023` | `TEST-011` timeout / out-of-order | `TEST-013` 或 `TEST-024` |
| `QRY_LITE` | `S5` | `TEST-023` | `TEST-010` PG authority / `TEST-012` revoke | `TEST-013` 或 `TEST-024` |
| `SBX_STD` | `S3` | `TEST-006` + `TEST-023` | `TEST-012` sandbox terminate | `TEST-013` 或 `TEST-024` |
| `RPT_STD` | `S4`、`S5` 补充 | `TEST-023` | `TEST-012` report revoke / `TEST-010` authority | `TEST-013` 或 `TEST-024` |

说明：

- `TEST-021` 先冻结覆盖规则；`TEST-022~024` 会继续把场景明细、SKU 矩阵和编排链路展开成专门文档与 checker。
- 在 `TEST-023` 完成前，不允许把 “主路径已存在” 误报成 “8 SKU 覆盖矩阵已完成”。

## Mandatory Gates

| Gate ID | 对应任务 | 官方入口 | 通过判定 | 必要证据 |
| --- | --- | --- | --- | --- |
| `ACC-CONTRACT` | `TEST-003` | `./scripts/check-api-contract-baseline.sh` | 成功/失败 envelope、关键字段、错误码与动作枚举冻结口径全部通过 | checker 输出、OpenAPI / API baseline 差异为 0 |
| `ACC-MIGRATION` | `TEST-004` | `ENV_FILE=infra/docker/.env.local ./scripts/check-migration-smoke.sh` | migration/seed roundtrip、`platform-core` health/runtime 回查通过 | migration smoke artifact、`seed_history` 与 health probe |
| `ACC-LOCAL` | `TEST-005` | `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh` | `core + observability + mocks` 正式本地栈可真实拉起 | `check-local-stack`、Keycloak realm、Grafana、canonical topics、Kafka 双地址边界 |
| `ACC-SCENARIO` | `TEST-006` | `ENV_FILE=infra/docker/.env.local ./scripts/check-order-e2e.sh` | 五条标准链路门户 E2E、后端 `order detail / lifecycle / developer trace` 全部通过 | Playwright artifact、后端 API 回查、order ids |
| `ACC-SKU-COVERAGE` | `TEST-023` | `TEST-023` 产出的正式 SKU 覆盖矩阵 checker | 8 个标准 SKU 都至少具备主路径、异常/阻断、退款/争议三类证据，且与五条标准链路映射一致 | SKU matrix artifact、场景/SKU 映射、order ids |
| `ACC-ORCH-20ORDERS` | `TEST-024` | `TEST-024` 产出的正式编排链路 checker | `支付成功 -> 待交付 -> 交付完成 -> 待验收 -> 验收通过/拒收 -> 结算/退款` 编排链路成立，并支撑最终 `20+ order` sign-off | 编排链路 artifact、webhook/交付/验收/结算 order ids |
| `ACC-PROVIDER` | `TEST-007` | `ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh` | 支付 / 签章 / 链写 provider mock/real 切换均不改业务代码 | live smoke 输出、provider config / artifact |
| `ACC-OUTBOX` | `TEST-008` | `ENV_FILE=infra/docker/.env.local ./scripts/check-outbox-consistency.sh` | 事务成功有 outbox、失败无脏副作用、重复消费不重复副作用 | `trade.order_main`、`ops.outbox_event`、通知消费证据 |
| `ACC-AUDIT` | `TEST-009` | `ENV_FILE=infra/docker/.env.local ./scripts/check-audit-completeness.sh` | 高风险动作必留痕，证据导出必须 step-up，非法导出被拒绝 | 审计事件、导出对象、拒绝结果 |
| `ACC-SEARCHREC` | `TEST-010` | `ENV_FILE=infra/docker/.env.local ./scripts/check-searchrec-pg-authority.sh` | 搜索 / 推荐最终回 PostgreSQL 放行，OpenSearch/Redis 不能充当真相源 | PG fallback、alias / visibility / recommendation evidence |
| `ACC-PAYMENT` | `TEST-011` | `ENV_FILE=infra/docker/.env.local ./scripts/check-payment-webhook-idempotency.sh` | duplicate success、`success -> fail`、`timeout -> success` 不得回退状态 | webhook artifact、`payment_intent / order_main / audit` 联查 |
| `ACC-DELIVERY` | `TEST-012` | `ENV_FILE=infra/docker/.env.local ./scripts/check-delivery-revocation.sh` | 撤权后文件票据、API key、share、sandbox 会话真实失效 | `Redis / PostgreSQL / audit` 联查 |
| `ACC-DISPUTE` | `TEST-013` | `ENV_FILE=infra/docker/.env.local ./scripts/check-dispute-settlement-linkage.sh` | 争议冻结结算，裁决后退款或赔付正确入账 | `billing_event / settlement_record / audit / outbox` 联查 |
| `ACC-REPLAY` | `TEST-014` | `ENV_FILE=infra/docker/.env.local ./scripts/check-audit-replay-dry-run.sh` | replay dry-run 仅输出差异，不改主状态；report / diff / step-up / audit 全部成立 | MinIO report、`diff_summary`、step-up/audit 证据 |
| `ACC-CI-MATRIX` | `TEST-015` | `./scripts/check-ci-minimal-matrix.sh all` | Rust / TS / Go / migration / OpenAPI 最小矩阵全部通过 | lane 日志、失败定位信息 |
| `ACC-COMPOSE` | `TEST-016` | `ENV_FILE=infra/docker/.env.local ./scripts/check-compose-smoke.sh` | compose 级 smoke + canonical static gate 全部通过 | compose artifact、`smoke-local` 结果 |
| `ACC-SCHEMA` | `TEST-017` | `ENV_FILE=infra/docker/.env.local ./scripts/check-schema-drift.sh` | migration / `.sqlx` / entity / OpenAPI 漂移全部为 0 | schema drift artifact |
| `ACC-PERFORMANCE` | `TEST-018` | `ENV_FILE=infra/docker/.env.local ./scripts/check-performance-smoke.sh` | 搜索、下单、交付、审计联查基础延迟门槛全部通过 | summary、Prometheus / metrics / request log artifact |
| `ACC-FAILURE` | `TEST-019` | `ENV_FILE=infra/docker/.env.local ./scripts/check-failure-drills.sh` | Kafka / OpenSearch / Fabric Adapter / Mock Payment 故障注入与恢复全部通过 | summary、lag、receipt、fallback artifact |
| `ACC-RECOVERY` | `TEST-020` | `ENV_FILE=infra/docker/.env.local ./scripts/check-rollback-recovery.sh` | 业务库 reset、base seed replay、runtime 重建、demo 数据恢复全部通过 | `summary.json`、demo seed artifact、post-reset zero-state evidence |
| `ACC-CANONICAL` | `TEST-028` | `ENV_FILE=infra/docker/.env.local ./scripts/check-canonical-contracts.sh` | canonical topic / route / consumer group / OpenAPI / 文档 authority 全部通过 | full canonical checker artifact |

## Final Sign-Off Sequence

最终签收必须按以下顺序执行，且中途任一 gate 失败都不能跳过：

1. `ACC-CONTRACT`
2. `ACC-MIGRATION`
3. `ACC-LOCAL`
4. `ACC-SCENARIO`
5. `ACC-SKU-COVERAGE`
6. `ACC-ORCH-20ORDERS`
7. `ACC-PROVIDER`
8. `ACC-OUTBOX`
9. `ACC-AUDIT`
10. `ACC-SEARCHREC`
11. `ACC-PAYMENT`
12. `ACC-DELIVERY`
13. `ACC-DISPUTE`
14. `ACC-REPLAY`
15. `ACC-CI-MATRIX`
16. `ACC-COMPOSE`
17. `ACC-SCHEMA`
18. `ACC-PERFORMANCE`
19. `ACC-FAILURE`
20. `ACC-RECOVERY`
21. `ACC-CANONICAL`

## 20+ Orders Sign-Off Rule

`15.2` 要求 `Phase 1` 连续演示 `20` 笔以上订单无关键错误。最终验收记录必须显式列出唯一 `order_id` 来源，不接受“估计跑过很多次”的口头证明。

建议执行口径：

| 订单来源 | 最少数量 | 证据来源 |
| --- | --- | --- |
| `fixtures/demo/orders.json` 冻结 demo 主/补充订单 | `10` | demo fixture + `check-demo-seed.sh` |
| `TEST-006` 门户 live E2E 新建订单 | `5` | `check-order-e2e.sh` artifact / order ids |
| `TEST-023 / TEST-024` SKU 覆盖矩阵与编排链路新增订单 | `>=5` | 对应 checker artifact / DB 回查 |

通过判定：

1. 最终验收记录中唯一 `order_id` 总数 `>= 20`
2. 五条标准链路 `S1~S5` 都至少出现一次 live execution 证据
3. 8 个标准 SKU 都有主路径、异常/阻断、退款/争议三类证据归档
4. 连续演示窗口内无 `P0/P1` 关键错误、无无法解释的数据不一致

## Evidence Record Template

最终签收时，至少要补齐下面这张记录表：

| Gate ID | 执行日期 | 执行人 | Commit | 命令 | 结果 | 关键 artifact / order_id |
| --- | --- | --- | --- | --- | --- | --- |
| `ACC-CONTRACT` |  |  |  |  |  |  |
| `ACC-MIGRATION` |  |  |  |  |  |  |
| `ACC-LOCAL` |  |  |  |  |  |  |
| `ACC-SCENARIO` |  |  |  |  |  |  |
| `ACC-SKU-COVERAGE` |  |  |  |  |  |  |
| `ACC-ORCH-20ORDERS` |  |  |  |  |  |  |
| `ACC-PROVIDER` |  |  |  |  |  |  |
| `ACC-OUTBOX` |  |  |  |  |  |  |
| `ACC-AUDIT` |  |  |  |  |  |  |
| `ACC-SEARCHREC` |  |  |  |  |  |  |
| `ACC-PAYMENT` |  |  |  |  |  |  |
| `ACC-DELIVERY` |  |  |  |  |  |  |
| `ACC-DISPUTE` |  |  |  |  |  |  |
| `ACC-REPLAY` |  |  |  |  |  |  |
| `ACC-CI-MATRIX` |  |  |  |  |  |  |
| `ACC-COMPOSE` |  |  |  |  |  |  |
| `ACC-SCHEMA` |  |  |  |  |  |  |
| `ACC-PERFORMANCE` |  |  |  |  |  |  |
| `ACC-FAILURE` |  |  |  |  |  |  |
| `ACC-RECOVERY` |  |  |  |  |  |  |
| `ACC-CANONICAL` |  |  |  |  |  |  |

## Boundary

- 本文是 `V1` 最终验收的统一入口，不替代各专项 case 文档。
- 本文不直接宣告“已通过”；它只定义必须怎么通过、必须留下什么证据。
- 若后续 `TEST-022~028` 收紧了 checker 或 artifact 要求，本文必须同步更新，不能继续引用旧口径。
