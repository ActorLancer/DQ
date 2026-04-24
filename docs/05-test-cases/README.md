# 05-test-cases

用于收敛测试用例执行清单、验收记录与回归基线输出。

约束：

- 当前批次若只做“口径收缩/事件拓扑冻结/命名统一”，不要求提前伪造尚未实现模块的测试样例文件。
- 但这不代表测试样例已完成；进入对应模块代码实现批次后，Agent 必须同步补齐测试样例文档、集成测试与 smoke 校验，不能把“设计口径已冻结”误报为“测试基线已落盘”。
- 除非文档明确标注为“容器内探测 / compose 网络内部调用”，宿主机启动应用、手工验收和 test-case 示例都必须使用宿主机地址边界：
  - Kafka：`127.0.0.1:9094`
  - 容器内 / compose 网络：`kafka:9092` 或容器内 `localhost:9092`
- 宿主机示例优先使用 `set -a; source infra/docker/.env.local; set +a` 载入运行时入口，避免手工散落 Kafka / DB / MinIO 地址后再次漂移。
- `./scripts/check-topic-topology.sh` 只用于通知 / Fabric / audit-anchor 相关关键静态 topology 与 route seed 校验；若要验证 `infra/kafka/topics.v1.json` 中全部 canonical topics 是否真实存在，应执行 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`。
- `./scripts/check-canonical-contracts.sh` 是 `TEST-028` 的正式 checker：本地默认 `full` 模式会串行执行 `check-openapi-schema.sh + check-topic-topology.sh + smoke-local.sh`，并额外校验 canonical consumer group、宿主机/容器 Kafka 边界与正式运行态文档中不存在旧 topic / 旧命名默认值；CI 则使用 `CANONICAL_CHECK_MODE=static` 跑静态子集。
- `./scripts/check-api-contract-baseline.sh` 是 `TEST-003` 的正式 checker：它只校验 API/OpenAPI 相关冻结契约，包括成功/失败 envelope、关键响应字段、错误码基线与订单状态机 action enum / 禁止错误码绑定，不替代 `TEST-028` 的 canonical smoke。
- `./scripts/check-migration-smoke.sh` 是 `TEST-004` 的正式 checker：它会启动当前 local core stack、初始化 MinIO buckets、执行 migration/seed roundtrip，并在最终升级后真实启动 `platform-core` 做健康与运行态回查；`./scripts/validate_database_migrations.sh` 仅作为兼容入口转发到该 checker。
- `./scripts/smoke-local.sh` 是 `TEST-005` 的正式 checker：它会自动确保 `core + observability + mocks` compose profile、执行基础 `migrate-up + seed-up`、初始化 MinIO buckets，并在宿主机 `127.0.0.1:8094` 启动或复用 `platform-core`，再回查 `check-local-stack full`、Keycloak realm、Grafana datasource、canonical topics、Kafka 双地址边界与关键 ops 控制面入口。
- `./scripts/check-compose-smoke.sh` 是 `TEST-016` 的正式 checker：它先执行 `smoke-local.sh` 完成 compose 级运行态 smoke，再执行 `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`，把 canonical topic、consumer group catalog 与关键 OpenAPI 归档漂移一起拦在 CI compose 作业中。
- `./scripts/check-schema-drift.sh` 是 `TEST-017` 的正式 checker：它会自带 core stack 前置、执行 `cargo sqlx prepare --workspace --check`、`check-query-compile.sh`、`db::entity` live catalog 校验与 `check-openapi-schema.sh`，统一拦截 migration / `.sqlx` / entity / OpenAPI 漂移。
- `./scripts/check-ci-minimal-matrix.sh` 是 `TEST-015` 的正式 checker：支持 `rust / ts / go / migration / openapi / all` 六个入口，本地与 CI 都必须复用它，不允许在 workflow 里另写第二套命令。
- 当前仓库已分别由以下文件承接三条事件的正式验收清单：
  - `notification.requested -> docs/05-test-cases/notification-cases.md`
  - `audit.anchor_requested / fabric.proof_submit_requested -> docs/05-test-cases/audit-consistency-cases.md`
  - `SEARCHREC consumer 幂等 / 双层 DLQ / reprocess -> docs/05-test-cases/search-rec-cases.md`


- `order-state-machine.md`：Order/Contract/Authorization 主交易链路 8 个标准 SKU 状态转换测试矩阵。
- `order-e2e-cases.md`：`TEST-006` 五条标准链路 order E2E 正式清单，固定 `fixtures/demo` 主订单基线、门户 live E2E 路径、后端 order/lifecycle/trace 回查与浏览器受控边界。
- `provider-switch-cases.md`：`TEST-007` provider 切换正式清单，冻结支付 / 签章 / 链写三类 provider 的 mock/real 切换断言、Fabric live smoke 与官方 checker。
- `outbox-consistency-cases.md`：`TEST-008` outbox 一致性正式清单，冻结事务成功写主对象 + 审计 + outbox、事务失败无脏 outbox，以及 `notification-worker` 重复消费不重复副作用的验收路径。
- `audit-completeness-cases.md`：`TEST-009` 审计完备性正式清单，冻结关键审计动作留痕、证据导出 step-up 约束、非法导出拒绝和 MinIO 导出对象回查。
- `search-rec-pg-authority-cases.md`：`TEST-010` 搜索 / 推荐回 PostgreSQL 校验正式清单，冻结 alias 切换后回 PG 过滤、OpenSearch 不可用 fallback，以及冻结 / 下架商品不得继续落入结果集或 `recommendation_result_item`。
- `payment-webhook-idempotency-cases.md`：`TEST-011` 支付 webhook 幂等正式清单，冻结 duplicate success、`success -> fail`、`timeout -> success` 三条回调保护，以及 `payment_intent / order_main / audit` 联查。
- `delivery-revocation-cases.md`：`TEST-012` 交付与断权正式清单，冻结文件票据撤权、share/API/sandbox 断权后的正式入口失败、`Redis / PostgreSQL / audit` 联查。
- `dispute-settlement-linkage-cases.md`：`TEST-013` 争议与结算联动正式清单，冻结 `POST /api/v1/cases` 触发的结算冻结，以及裁决后 `refund / compensation` 的正式入账、结算重算、审计与 outbox 联查。
- `audit-replay-dry-run-cases.md`：`TEST-014` 审计回放 dry-run 正式清单，冻结订单 replay job 的差异报告输出、MinIO report、`replay_result.diff_summary`、权限 / step-up / dry-run-only 约束与审计留痕。
- `ci-minimal-matrix-cases.md`：`TEST-015` CI 最小矩阵正式清单，冻结 Rust / TS / Go / migration / OpenAPI 五条 lane 的命令、失败定位与边界说明。
- `delivery-cases.md`：Delivery/Storage/Query Execution 子域的交付超时、重复开通、票据过期、撤权后访问、验收失败用例矩阵。
- `payment-billing-cases.md`：Billing/Payment/Settlement/Dispute 子域的回调乱序、重复回调、重复扣费防护与结算冻结回归矩阵。
- `migration-smoke-cases.md`：`TEST-004` 的正式 migration smoke 清单，固定 core stack、migration/seed roundtrip、seed_history 回查与 `platform-core` 启动验证入口。
- `local-environment-smoke-cases.md`：`TEST-005` 的正式本地环境 smoke 清单，固定 `core + observability + mocks` 组合、宿主机/容器 Kafka 双地址边界、realm / datasource / topic / ops 控制面回查与 CI 入口。
- `compose-smoke-cases.md`：`TEST-016` 的正式 compose CI smoke 清单，固定 `check-compose-smoke.sh` 入口、运行态 smoke 与 canonical 静态漂移收口、artifact 留存与失败定位边界。
- `schema-drift-cases.md`：`TEST-017` 的正式 schema drift 清单，冻结 SQLx metadata、离线 query compile、`db::entity` live catalog 与 OpenAPI 归档的 drift gate。
- `performance-smoke-cases.md`：`TEST-018` 的正式性能冒烟清单，冻结搜索、下单、交付、审计联查四条 API 的 `<= 2.0s` 基础门槛、Prometheus / metrics / request log 证据与业务测试数据清理边界。
- `failure-drill-cases.md`：`TEST-019` 的正式故障演练清单，冻结 Kafka 停机、OpenSearch 不可用、Fabric Adapter 停机、Mock Payment 延迟四类真实故障注入、恢复与回查口径。
- `rollback-recovery-cases.md`：`TEST-020` 的正式回滚恢复清单，冻结业务库 reset、基础 seed replay、环境重启与 demo 数据恢复的正式入口与回查边界。
- `v1-core-acceptance-checklist.md`：`TEST-021` 的正式 V1 退出验收清单，把退出门槛、五条标准链路 / 8 SKU 映射、官方 checker 与最终 sign-off 顺序收口为一张可执行 checklist。
- `five-standard-scenarios-e2e.md`：`TEST-022` 的五条标准链路顺序执行文档，逐条冻结 fixture 输入、目标状态与页面/API/审计验证点，供后续回归与 V1 sign-off 使用。
- `standard-sku-coverage-matrix.md`：`TEST-023` 的 8 个标准 SKU 覆盖矩阵，冻结每个 SKU 的主路径、异常路径、退款/争议证据、demo billing basis order 与 `S1~S5` 挂点，并把官方 checker 收口到 `check-standard-sku-coverage.sh`。
- `order-orchestration-cases.md`：`TEST-024` 的正式编排链路验收清单，冻结 `支付成功 -> 待交付 -> 交付完成 -> 待验收 -> 验收通过/拒收 -> 结算/退款` 顺序 gate、前后端复用边界与 `20+ order` sign-off artifact 结构。
- `share-ro-e2e.md`：`TEST-025` 的正式 `SHARE_RO` 端到端验收清单，冻结 seller/buyer 门户 role、grant/read/revoke portal live E2E、`trade012 / dlv006 / bil026` 后端证据与 `audit / outbox / DB` 汇总边界。
- `search-rec-cases.md`：`SEARCHREC-017` 正式冻结的 Search/Recommendation 验收矩阵，覆盖投影延迟、回 PostgreSQL 最终校验、推荐曝光/点击幂等、零结果兜底、统一鉴权 / step-up / 审计 / 错误码，以及 consumer 幂等、双层 DLQ 与 dry-run reprocess。
- `notification-cases.md`：通知链路验收清单，覆盖 `notification.requested -> dtp.notification.dispatch -> notification-worker`、`mock-log`、幂等、重试、DLQ、人工补发与审计联查。
- `web-smoke-cases.md`：`WEB-020` 冻结的 portal / console 最小页面 smoke 基线，覆盖页面路由、状态态面、浏览器受控 API 边界与 `WEB-018` live E2E 入口。
- `audit-consistency-cases.md`：已落地 `AUD-003~AUD-026` 与 `AUD-031` 的审计联查、证据包导出、replay dry-run、legal hold、anchor batch、canonical outbox / dead letter 查询、outbox publisher、Billing bridge published-only 边界、SEARCHREC dead letter dry-run 重处理、一致性联查、一致性修复 dry-run、`fabric-adapter` 四类摘要 handler与重复投递隔离、`fabric-event-listener` callback、`fabric-ca-admin` 证书治理、trade monitor 总览 / checkpoints、external facts 查询 / confirm、公平性事件查询 / handle、projection gaps 查询 / resolve、观测总览 / 日志镜像查询导出 / trace 联查 / 告警与 incident / SLO，以及 `GET /api/v1/developer/trace` 开发者状态联查验收矩阵；`AUD-028` 已把“链下成功链上失败、链上成功链下未更新、回调乱序 / 晚到、重复事件、修复演练”五类一致性场景正式映射到该文件，后续 `AUD-029+` 的剩余高风险控制面继续在同文件追加。
- `canonical-event-authority-cases.md`：`AUD-030` canonical envelope / route authority 验收清单，固定 `trade003 / cat022 / dlv002 / dlv029 / audit anchor retry` 五条 smoke 与 `tg_write_outbox` 退役静态回查。
- `TEST-028` checker：
  - 本地全量：`ENV_FILE=infra/docker/.env.local ./scripts/check-canonical-contracts.sh`
  - CI 静态：`CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`
