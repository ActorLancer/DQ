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
- `delivery-cases.md`：Delivery/Storage/Query Execution 子域的交付超时、重复开通、票据过期、撤权后访问、验收失败用例矩阵。
- `payment-billing-cases.md`：Billing/Payment/Settlement/Dispute 子域的回调乱序、重复回调、重复扣费防护与结算冻结回归矩阵。
- `migration-smoke-cases.md`：`TEST-004` 的正式 migration smoke 清单，固定 core stack、migration/seed roundtrip、seed_history 回查与 `platform-core` 启动验证入口。
- `local-environment-smoke-cases.md`：`TEST-005` 的正式本地环境 smoke 清单，固定 `core + observability + mocks` 组合、宿主机/容器 Kafka 双地址边界、realm / datasource / topic / ops 控制面回查与 CI 入口。
- `search-rec-cases.md`：`SEARCHREC-017` 正式冻结的 Search/Recommendation 验收矩阵，覆盖投影延迟、回 PostgreSQL 最终校验、推荐曝光/点击幂等、零结果兜底、统一鉴权 / step-up / 审计 / 错误码，以及 consumer 幂等、双层 DLQ 与 dry-run reprocess。
- `notification-cases.md`：通知链路验收清单，覆盖 `notification.requested -> dtp.notification.dispatch -> notification-worker`、`mock-log`、幂等、重试、DLQ、人工补发与审计联查。
- `web-smoke-cases.md`：`WEB-020` 冻结的 portal / console 最小页面 smoke 基线，覆盖页面路由、状态态面、浏览器受控 API 边界与 `WEB-018` live E2E 入口。
- `audit-consistency-cases.md`：已落地 `AUD-003~AUD-026` 与 `AUD-031` 的审计联查、证据包导出、replay dry-run、legal hold、anchor batch、canonical outbox / dead letter 查询、outbox publisher、Billing bridge published-only 边界、SEARCHREC dead letter dry-run 重处理、一致性联查、一致性修复 dry-run、`fabric-adapter` 四类摘要 handler与重复投递隔离、`fabric-event-listener` callback、`fabric-ca-admin` 证书治理、trade monitor 总览 / checkpoints、external facts 查询 / confirm、公平性事件查询 / handle、projection gaps 查询 / resolve、观测总览 / 日志镜像查询导出 / trace 联查 / 告警与 incident / SLO，以及 `GET /api/v1/developer/trace` 开发者状态联查验收矩阵；`AUD-028` 已把“链下成功链上失败、链上成功链下未更新、回调乱序 / 晚到、重复事件、修复演练”五类一致性场景正式映射到该文件，后续 `AUD-029+` 的剩余高风险控制面继续在同文件追加。
- `canonical-event-authority-cases.md`：`AUD-030` canonical envelope / route authority 验收清单，固定 `trade003 / cat022 / dlv002 / dlv029 / audit anchor retry` 五条 smoke 与 `tg_write_outbox` 退役静态回查。
- `TEST-028` checker：
  - 本地全量：`ENV_FILE=infra/docker/.env.local ./scripts/check-canonical-contracts.sh`
  - CI 静态：`CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`
