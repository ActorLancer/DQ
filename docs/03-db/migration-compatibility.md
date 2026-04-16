# Migration 兼容性测试基线（DB-032）

## 目标

- 保证新增 migration 不破坏既有 seed。
- 保证本地执行 `migrate-down` 后可再次 `migrate-up`。
- 保证回滚演练后种子可重复执行且结果稳定。

## 执行入口

- 主入口：`db/scripts/verify-db-compatibility.sh`
- 依赖脚本：
  - `db/scripts/migrate-reset.sh`
  - `db/scripts/verify-migration-roundtrip.sh`
  - `db/scripts/seed-up.sh`
  - `db/scripts/verify-seed-001.sh`
  - `db/scripts/verify-seed-010-030.sh`
  - `db/scripts/verify-migration-065-068.sh`
  - `db/scripts/verify-migration-070.sh`

## 覆盖链路

1. 空库全量升级：`migrate-reset`
2. 全量种子执行：`seed-up`
3. 种子校验：`verify-seed-001` + `verify-seed-010-030`
4. 回滚演练：全量 `down` -> 全量 `up`
5. 回滚后重复种子执行：`seed-up`
6. 回滚后重复种子校验：`verify-seed-001` + `verify-seed-010-030`
7. 关键迁移段复核：`065~068` 与 `070`
8. 最终状态检查：`migrate-status` 无 pending

## 通过标准

- 脚本返回码为 `0`。
- `migrate-status` 的 `pending up versions` 为空。
- `public.seed_history` 至少包含：`001/010/020/030`。
- 回滚后 `migrate-up` 不出现 “已执行但无法重放” 类错误。

## 失败处理

- 任一环节失败即阻断后续任务推进。
- 需先修复 migration/seed/runner 一致性，再重新全链路验证。
