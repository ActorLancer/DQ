# 上链异步链路冻结（CTX-010）

## 1. 核心规则

- 所有上链动作必须走异步事件链路。
- 业务请求不得同步阻塞等待链确认。

## 2. 主路径

`主对象事务提交 -> outbox_event -> Kafka -> fabric-adapter -> fabric-event-listener -> consistency 回写`

## 3. 执行约束

- 业务事务内必须同时写入：主对象、审计记录、`outbox_event`。
- `fabric-adapter` 仅处理链提交与回执，不作为业务主状态机。
- 链确认结果回写后，只能更新镜像/证明相关状态，不得越权篡改业务主状态。
- 失败事件进入 dead-letter 并支持 dry-run 重处理。
