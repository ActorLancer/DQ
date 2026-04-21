# Fabric Local 启动说明（ENV-022/023/024）

## 命令入口

- `make fabric-up`
- `make fabric-down`
- `make fabric-reset`
- `make fabric-channel`

## 组件说明

- `fabric-ca`：本地 CA 容器占位
- `fabric-orderer`：本地 orderer 容器占位
- `fabric-peer`：本地 peer 容器占位

## 事件拓扑冻结

- `audit.anchor_requested -> dtp.audit.anchor -> fabric-adapter`
- `fabric.proof_submit_requested -> dtp.fabric.requests -> fabric-adapter`
- `fabric-event-listener -> dtp.fabric.callbacks -> platform-core.consistency`
- `dtp.outbox.domain-events` 仅保留为通用主领域事件流，不作为 `fabric-adapter` 的正式消费入口
- 本地默认 consumer group：
  - `fabric-adapter -> cg-fabric-adapter`
  - `platform-core.consistency -> cg-platform-core-consistency`

## 当前批次边界

- 本批次只冻结 Fabric 请求/回执的事件拓扑、命名和本地排障入口，不代表 `fabric-adapter` / `fabric-event-listener` / `platform-core.consistency` 已完成正式实现。
- 当前文档结论只能回答“链请求与回执应该怎么走”，不能替代后续代码实现所需的 OpenAPI、回调 DTO、重处理样例与集成测试。
- 进入 `AUD / integration / consistency` 代码实现批次后，Agent 必须同步补齐：
  - `packages/openapi/audit.yaml`
  - `packages/openapi/ops.yaml`
  - `docs/02-openapi/audit.yaml`
  - `docs/02-openapi/ops.yaml`
  - `docs/05-test-cases/audit-consistency-cases.md`
  - callback / reconcile / dead-letter / retry runbook 的实操步骤

## 链码占位部署

```bash
./infra/fabric/deploy-chaincode-placeholder.sh
```

该脚本生成链码接口占位清单，覆盖：
- 订单摘要
- 授权摘要
- 验收摘要
- 证据批次根

## 排障补充

排查 Fabric 请求/回执口径时，优先检查：

1. `infra/kafka/topics.v1.json`
2. `docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`
3. `./scripts/check-topic-topology.sh`（覆盖 Fabric / notification / audit-anchor 相关关键静态拓扑、route seed 与当前数据库 `ops.event_route_policy` 运行态，不替代全量 smoke）

若 `audit.anchor_requested` 或 `fabric.proof_submit_requested` 未命中 route policy，不得继续按猜测 topic 联调。
