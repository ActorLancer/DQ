# fabric-event-listener

`AUD-015` 起，`Go` 实现的 Fabric 事件监听进程已正式落地。

职责：

- 监听已提交的 Fabric source receipt
- 生成 `fabric.commit_confirmed / fabric.commit_failed`
- 发布到正式 topic：`dtp.fabric.callbacks`
- 回写 `ops.external_fact_receipt / audit.audit_event / ops.system_log / chain.chain_anchor / audit.anchor_batch`

当前 local 模式：

- source mode：`mock`
- 运行入口：`./scripts/fabric-event-listener-run.sh`
- bootstrap：`./scripts/fabric-event-listener-bootstrap.sh`
- test：`./scripts/fabric-event-listener-test.sh`

详细实操与回查见：

- `docs/04-runbooks/fabric-event-listener.md`
