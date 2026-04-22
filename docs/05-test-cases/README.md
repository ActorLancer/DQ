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
- 对于 `notification.requested / audit.anchor_requested / fabric.proof_submit_requested` 三条事件，后续实现阶段至少要同步补齐：
  - `docs/05-test-cases/audit-consistency-cases.md`
  - 通知事件链路验收清单
  - Fabric request / callback / reconcile 验收清单


- `order-state-machine.md`：Order/Contract/Authorization 主交易链路 8 个标准 SKU 状态转换测试矩阵。
- `delivery-cases.md`：Delivery/Storage/Query Execution 子域的交付超时、重复开通、票据过期、撤权后访问、验收失败用例矩阵。
- `payment-billing-cases.md`：Billing/Payment/Settlement/Dispute 子域的回调乱序、重复回调、重复扣费防护与结算冻结回归矩阵。
- `search-rec-cases.md`：Search/Recommendation 子域的搜索同步、搜索 API、推荐召回、行为回流、重建、别名切换与缓存失效验收清单。
- `notification-cases.md`：通知链路验收清单，覆盖 `notification.requested -> dtp.notification.dispatch -> notification-worker`、`mock-log`、幂等、重试、DLQ、人工补发与审计联查。
- `audit-consistency-cases.md`：已落地 `AUD-003~AUD-023` 的审计联查、证据包导出、replay dry-run、legal hold、anchor batch、canonical outbox / dead letter 查询、outbox publisher、SEARCHREC dead letter dry-run 重处理、一致性联查、一致性修复 dry-run、`fabric-adapter` 四类摘要 handler、`fabric-event-listener` callback、`fabric-ca-admin` 证书治理、trade monitor 总览 / checkpoints、external facts 查询 / confirm、公平性事件查询 / handle、projection gaps 查询 / resolve，以及观测总览 / 日志镜像查询导出 / trace 联查 / 告警与 incident / SLO 验收矩阵；后续 `AUD-024+` 的剩余高风险控制面继续在同文件追加。
