# Audit Completeness Cases

`TEST-009` 的验收目标不是“审计表里有几行”，而是证明下面三条正式边界同时成立：

- 关键审计控制面动作会留下 `audit.audit_event`、`audit.access_audit`、`ops.system_log`。
- `POST /api/v1/audit/packages/export` 属于高风险动作，缺权限或缺少 step-up 必须被拒绝。
- 合法导出会真实写入 `audit.evidence_package`、`audit.evidence_manifest_item`、MinIO 导出对象，并绑定 `step_up_challenge_id`。

## 验收矩阵

| Case ID | 场景 | 入口 | 预期 |
| --- | --- | --- | --- |
| `AUDIT-COMP-001` | 非法导出被拒绝 | `rejects_package_export_without_permission` | `buyer_operator` 调用 `/api/v1/audit/packages/export` 返回 `403` |
| `AUDIT-COMP-002` | 缺 step-up 的导出被拒绝 | `package_export_requires_step_up` | `platform_audit_security` 缺少 `x-step-up-token / x-step-up-challenge-id` 时返回 `400` |
| `AUDIT-COMP-003` | 关键审计动作写入审计轨迹 | `audit_trace_api_db_smoke` | 订单审计联查、trace 查询、证据包导出、replay dry-run、legal hold、anchor retry 均留下正式审计 / 访问留痕 |
| `AUDIT-COMP-004` | 合法导出绑定 step-up 并落到 MinIO | `audit_trace_api_db_smoke` | `audit.evidence_package`、`audit.evidence_manifest_item`、MinIO 导出对象真实存在；`audit.audit_event(action_name='audit.package.export')`、`audit.access_audit(access_mode='export')`、`ops.system_log` 可回查 |

## 正式 Checker

宿主机正式入口：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-audit-completeness.sh
```

该 checker 会依次执行：

1. `cargo test -p platform-core rejects_package_export_without_permission -- --nocapture`
2. `cargo test -p platform-core package_export_requires_step_up -- --nocapture`
3. `smoke-local.sh`
4. `AUD_DB_SMOKE=1 cargo test -p platform-core audit_trace_api_db_smoke -- --nocapture`

## 主要回查点

- 路由拒绝：
  - HTTP `403 / 400`
  - 无新增 `audit.evidence_package`
- 合法导出与审计完备性：
  - `audit.audit_event(action_name='audit.package.export')`
  - `audit.access_audit(access_mode='export', step_up_challenge_id IS NOT NULL)`
  - `ops.system_log(message_text='audit package export executed: POST /api/v1/audit/packages/export')`
  - `audit.evidence_package`
  - `audit.evidence_manifest_item`
  - MinIO 导出对象

## 边界说明

- `TEST-009` 不替代 `AUD-005 / AUD-006 / AUD-007` 的专题 runbook；它把这些现有能力收口成 `TEST` 阶段的官方入口。
- `step-up` 的权威绑定对象仍是 `iam.step_up_challenge`；测试不得绕过正式 header / challenge 约束。
- 导出动作必须通过正式 API `POST /api/v1/audit/packages/export`，不得用脚本直接写 `audit.evidence_package` 或 MinIO 假造成功证据。
