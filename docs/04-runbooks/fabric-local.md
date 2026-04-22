# Fabric Local 启动说明（ENV-022/023/024 / AUD-017）

## 命令入口

- `make fabric-up`
- `make fabric-down`
- `make fabric-reset`
- `make fabric-channel`
- `make fabric-ca-admin-bootstrap`
- `make fabric-ca-admin-test`
- `make fabric-ca-admin-run`

## 组件说明

- `Hyperledger Fabric CA`：`ca_org1 / ca_org2 / ca_orderer`
- `Hyperledger Fabric Orderer`：`orderer.example.com`
- `Hyperledger Fabric Peer`：`peer0.org1.example.com / peer0.org2.example.com`
- `Hyperledger Fabric Gateway`：由 `services/fabric-adapter/` 通过 Go SDK 直连 `peer0.org1.example.com:7051`
- Go 链码：`infra/fabric/chaincode/datab-audit-anchor/`

本地 Fabric 不再由 `infra/docker/docker-compose.local.yml` 承担 placeholder 容器；正式入口改为：

- `./infra/fabric/install-deps.sh`
- `./infra/fabric/patch-samples.sh`
- `./infra/fabric/fabric-up.sh`
- `./infra/fabric/deploy-chaincode.sh`

依赖、样例网络和二进制统一落在：

```text
third_party/external-deps/fabric/
```

## 事件拓扑冻结

- `audit.anchor_requested -> dtp.audit.anchor -> fabric-adapter`
- `fabric.proof_submit_requested -> dtp.fabric.requests -> fabric-adapter`
- `fabric-event-listener -> dtp.fabric.callbacks -> platform-core.consistency`
- `dtp.outbox.domain-events` 仅保留为通用主领域事件流，不作为 `fabric-adapter` 的正式消费入口
- 本地默认 consumer group：
  - `fabric-adapter -> cg-fabric-adapter`
  - `platform-core.consistency -> cg-platform-core-consistency`

## 当前批次边界

- `AUD-013` 已补齐 `fabric-adapter` 的 Go module、Kafka consumer、canonical envelope 解析、mock provider 与 PostgreSQL 回执回写。
- `AUD-014` 已在 `fabric-adapter` 内补齐四类摘要 handler 占位：`evidence_batch_root / order_summary / authorization_summary / acceptance_summary`，并把 `submission_kind / contract_name / transaction_name` 贯穿到回执、审计与系统日志。
- `AUD-015` 已补齐 `fabric-event-listener` 的 Go module、mock callback 轮询源、`dtp.fabric.callbacks` 发布、`ops.external_fact_receipt / audit.audit_event / ops.system_log / chain.chain_anchor / audit.anchor_batch` 回写。
- `AUD-016` 已补齐 Go 版 `fabric-ca-admin`，由其承接 `Fabric 身份签发 / 吊销 / 证书吊销` 执行面，Rust `platform-core` 仅保留公网 IAM 控制面与 step-up / 审计主体。
- `AUD-017` 已把 `fabric-adapter` 收敛为可切换 `mock / fabric-test-network` 的真实 Go provider，并把本地 Fabric 固定到 pinned `2.5.15 / 1.5.17` test-network，不再依赖 `latest/main` 漂移。
- 当前 `fabric-adapter` 的实操入口以 `docs/04-runbooks/fabric-adapter.md` 为准。
- 当前 `fabric-event-listener` 的实操入口以 `docs/04-runbooks/fabric-event-listener.md` 为准。
- 当前 `fabric-ca-admin` 的实操入口以 `docs/04-runbooks/fabric-ca-admin.md` 为准。
- 当前文档结论只能回答“链请求与回执应该怎么走”，不能替代后续代码实现所需的 OpenAPI、回调 DTO、重处理样例与集成测试。
- 进入 `AUD / integration / consistency` 代码实现批次后，Agent 必须同步补齐：
  - `packages/openapi/audit.yaml`
  - `packages/openapi/ops.yaml`
  - `docs/02-openapi/audit.yaml`
  - `docs/02-openapi/ops.yaml`
  - `docs/05-test-cases/audit-consistency-cases.md`
  - callback / CA admin / reconcile / dead-letter / retry runbook 的实操步骤

## 启动与验收

```bash
make up-fabric
./scripts/check-fabric-local.sh
./scripts/fabric-adapter-live-smoke.sh
```

其中：

- `make up-fabric` 会先拉起 `core` 基础设施，再运行真实 `test-network` 与 Go 链码部署
- `check-fabric-local.sh` 会校验 channel、链码版本/sequence 和 `Ping`
- `fabric-adapter-live-smoke.sh` 会通过真实 Gateway 提交一条 `audit.anchor_requested`，并回查 PostgreSQL + 审计 + 系统日志 + 账本

## Go 链码职责

当前 `datab-audit-anchor` 提供：

- `SubmitEvidenceBatchRoot`
- `SubmitOrderDigest`
- `SubmitAuthorizationDigest`
- `SubmitAcceptanceDigest`
- `GetAnchorByReference`

事件与状态：

- 提交返回真实 `transaction_id`
- provider 显式等待 `commit status`
- `GetAnchorByReference` 可用于 runbook / smoke 账本回查

## 排障补充

排查 Fabric 请求/回执口径时，优先检查：

1. `infra/kafka/topics.v1.json`
2. `docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`
3. `./scripts/check-topic-topology.sh`（覆盖 Fabric / notification / audit-anchor 相关关键静态拓扑、route seed 与当前数据库 `ops.event_route_policy` 运行态，不替代全量 smoke）

若 `audit.anchor_requested` 或 `fabric.proof_submit_requested` 未命中 route policy，不得继续按猜测 topic 联调。
